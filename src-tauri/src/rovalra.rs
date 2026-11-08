//! Optional RoValra API provider for per-server public region metadata.
//! RoValra is a separate service, not affiliated with Thirst.
//! Explicit opt-in only. Never reads or forwards .ROBLOSECURITY credentials.
//! There is no scraping, hidden endpoint discovery, or background polling.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::State;

const MAX_SERVERS: usize = 25;
const LOOKUP_COOLDOWN: Duration = Duration::from_secs(30);
const RATE_LIMIT_COOLDOWN: Duration = Duration::from_secs(120);

#[derive(Default)]
pub struct RegionProviderState {
    next_allowed: Mutex<Option<Instant>>,
}

#[derive(Debug, Deserialize)]
struct ProviderEnvelope {
    status: Option<String>,
    #[serde(default)]
    disabled: bool,
    #[serde(default)]
    servers: Vec<ProviderServer>,
}

#[derive(Debug, Deserialize)]
struct ProviderServer {
    server_id: String,
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderRegion {
    server_id: String,
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
    source: &'static str,
    via: &'static str,
}

fn safe_location(value: Option<String>) -> Option<String> {
    value.and_then(|s| {
        let s = s.trim();
        if s.is_empty() || s.len() > 80 || s.eq_ignore_ascii_case("unknown") ||
            !s.chars().all(|c| c.is_alphanumeric() || c.is_whitespace() ||
                matches!(c, '-' | ',' | '.' | '\'' | '(' | ')')) {
            None
        } else {
            Some(s.to_owned())
        }
    })
}

fn normalise_result(source: ProviderServer, ids: &HashSet<String>) -> Option<ProviderRegion> {
    let id = uuid::Uuid::parse_str(&source.server_id).ok()?.to_string();
    if !ids.contains(&id) {
        return None;
    }
    let city = safe_location(source.city);
    let region = safe_location(source.region);
    let country = safe_location(source.country);
    if city.is_none() && region.is_none() && country.is_none() {
        return None;
    }
    Some(ProviderRegion {
        server_id: id,
        city,
        region,
        country,
        source: "rovalra",
        via: "direct",
    })
}

// Glutony shares cached RoValra locations with the user's BL:R website and
// Discord bot. A distributed Thirst build must never contain an API token:
// this endpoint is read-only and limits lookups to active sampled BL:R servers.
const GLUTONY_REGIONS_URL: &str =
    "https://glutony.scp343hey.workers.dev/api/servers/regions/shared";

fn glutony_mode(place_id: u64) -> Option<&'static str> {
    match place_id {
        124087144919678 => Some("8v8"),
        99999183305180 => Some("pro"),
        18668065416 => Some("main"),
        _ => None,
    }
}

#[derive(Deserialize)]
struct SharedRegionEnvelope {
    regions: HashMap<String, SharedRegionEntry>,
}

#[derive(Deserialize)]
struct SharedRegionEntry {
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
}

fn parse_shared_regions(
    body: SharedRegionEnvelope,
    requested: &HashSet<String>,
) -> Vec<ProviderRegion> {
    body.regions.into_iter().filter_map(|(id, entry)| {
        let id = uuid::Uuid::parse_str(&id).ok()?.to_string();
        let mut region = normalise_result(ProviderServer {
            server_id: id,
            city: entry.city,
            region: entry.state,
            country: entry.country,
        }, requested)?;
        region.via = "glutony";
        Some(region)
    }).collect()
}

async fn lookup_shared_regions(
    client: &reqwest::Client,
    place_id: u64,
    ids: &[String],
    requested: &HashSet<String>,
) -> Result<Vec<ProviderRegion>, String> {
    let Some(mode) = glutony_mode(place_id) else { return Ok(Vec::new()) };
    // Only send public BL:R place IDs and server UUIDs. Never send cookies,
    // Roblox identity, Cloudflare secrets, or raw tokens.
    let response = client.get(GLUTONY_REGIONS_URL)
        .query(&[("mode", mode), ("ids", &ids.join(","))])
        .send().await
        .map_err(|_| "Glutony shared region cache is unreachable.".to_owned())?;
    if !response.status().is_success() {
        return Err(format!("Glutony region cache unavailable (HTTP {}).", response.status().as_u16()));
    }
    let body = response.bytes().await
        .map_err(|_| "Could not read Glutony's region response.".to_owned())?;
    if body.len() > 128 * 1024 {
        return Err("Shared region response is unexpectedly large.".into());
    }
    let parsed: SharedRegionEnvelope = serde_json::from_slice(&body)
        .map_err(|_| "Glutony's shared region response is not recognized.".to_owned())?;
    Ok(parse_shared_regions(parsed, requested))
}

/// Runs only when the user chooses the RoValra action. Place/server IDs are
/// public Roblox identifiers, but disclosure is still required in the UI.
#[tauri::command]
pub async fn lookup_rovalra_regions(
    place_id: u64,
    server_ids: Vec<String>,
    state: State<'_, RegionProviderState>,
) -> Result<Vec<ProviderRegion>, String> {
    crate::validate_place_id(place_id)?;
    if server_ids.is_empty() || server_ids.len() > MAX_SERVERS {
        return Err("Select between one and 25 loaded servers for region lookup.".into());
    }
    let mut ids = HashSet::new();
    for id in server_ids {
        let uuid = uuid::Uuid::parse_str(&id)
            .map_err(|_| "Invalid public Roblox server ID.".to_owned())?;
        ids.insert(uuid.to_string());
    }

    {
        let mut next_allowed = state.next_allowed.lock()
            .map_err(|_| "Region lookup state unavailable.".to_owned())?;
        if let Some(until) = *next_allowed {
            let now = Instant::now();
            if now < until {
                return Err(format!(
                    "RoValra request cooldown: retry after {} seconds.",
                    until.duration_since(now).as_secs() + 1
                ));
            }
        }
        // Reserve before the network request to prevent rapid parallel calls.
        *next_allowed = Some(Instant::now() + LOOKUP_COOLDOWN);
    }

    let mut sorted_ids: Vec<String> = ids.iter().cloned().collect();
    sorted_ids.sort();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(9))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Thirst/0.3 (+https://github.com/N29dev/Thirst; unaffiliated)")
        .build()
        .map_err(|_| "Could not configure the optional region lookup.".to_owned())?;

    // BL:R regions are first requested through Glutony's bounded D1-backed
    // shared cache. When a server wasn't in the bot's recent public sample,
    // direct RoValra remains a fallback rather than making it disappear.
    let mut shared = if glutony_mode(place_id).is_some() {
        lookup_shared_regions(&client, place_id, &sorted_ids, &ids)
            .await.unwrap_or_default()
    } else {
        Vec::new()
    };
    let known: HashSet<String> = shared.iter().map(|region| region.server_id.clone()).collect();
    let missing: Vec<String> = sorted_ids.into_iter()
        .filter(|id| !known.contains(id)).collect();
    if missing.is_empty() {
        shared.truncate(MAX_SERVERS);
        return Ok(shared);
    }

    // No credentials, user IDs, IP geolocation or arbitrary URLs from the UI.
    let response = client
        .get("https://apis.rovalra.com/v1/servers/details")
        .query(&[
            ("place_id", place_id.to_string()),
            ("server_ids", missing.join(",")),
        ])
        .send().await
        .map_err(|_| "RoValra is unreachable for servers not yet cached by Glutony.".to_owned())?;

    match response.status().as_u16() {
        429 => {
            if let Ok(mut next) = state.next_allowed.lock() {
                *next = Some(Instant::now() + RATE_LIMIT_COOLDOWN);
            }
            return Err("RoValra rate-limited the region lookup; try again later.".into());
        }
        401 | 403 => return Err("RoValra did not permit this request. Region lookup has been stopped; use local region reports.".into()),
        200 => {}
        status => return Err(format!("RoValra region lookup unavailable (HTTP {status}).")),
    }
    let body = response.bytes().await
        .map_err(|_| "Could not read RoValra's region response.".to_owned())?;
    if body.len() > 256 * 1024 {
        return Err("RoValra returned an unexpectedly large response.".into());
    }
    let envelope: ProviderEnvelope = serde_json::from_slice(&body)
        .map_err(|_| "RoValra region data format is currently unsupported.".to_owned())?;
    if envelope.disabled {
        if let Ok(mut next) = state.next_allowed.lock() {
            *next = Some(Instant::now() + RATE_LIMIT_COOLDOWN);
        }
        return Err("RoValra disabled region lookup for this experience.".into());
    }
    if envelope.status.as_deref() != Some("success") {
        return Err("RoValra did not return a successful region lookup.".into());
    }
    shared.extend(envelope.servers.into_iter()
        .filter_map(|item| normalise_result(item, &ids))
        .filter(|item| !known.contains(&item.server_id))
        .take(MAX_SERVERS));
    shared.truncate(MAX_SERVERS);
    Ok(shared)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_unknown_and_unrequested_servers() {
        let accepted = HashSet::from(["550e8400-e29b-41d4-a716-446655440000".to_owned()]);
        let valid = normalise_result(ProviderServer {
            server_id: "550e8400-e29b-41d4-a716-446655440000".into(),
            city: Some("Paris".into()),
            region: None,
            country: Some("FR".into()),
        }, &accepted).unwrap();
        assert_eq!(valid.source, "rovalra");
        assert_eq!(valid.country.as_deref(), Some("FR"));
        assert!(normalise_result(ProviderServer {
            server_id: "550e8400-e29b-41d4-a716-446655440001".into(),
            city: Some("Paris".into()), region: None, country: Some("FR".into()),
        }, &accepted).is_none());
    }

    #[test]
    fn maps_blr_modes_without_hardcoded_tokens() {
        assert_eq!(glutony_mode(124087144919678), Some("8v8"));
        assert_eq!(glutony_mode(99999183305180), Some("pro"));
        assert_eq!(glutony_mode(18668065416), Some("main"));
        assert_eq!(glutony_mode(9999), None);
    }

    #[test]
    fn parses_only_requested_shared_region_entries() {
        let id = "550e8400-e29b-41d4-a716-446655440000";
        let requested = HashSet::from([id.to_owned()]);
        let regions = HashMap::from([
            (id.to_owned(), SharedRegionEntry {
                city: Some("São Paulo".into()), state: None, country: Some("Brazil".into())
            }),
            ("550e8400-e29b-41d4-a716-446655440001".into(), SharedRegionEntry {
                city: Some("New York".into()), state: None, country: Some("United States".into())
            })
        ]);
        let result = parse_shared_regions(SharedRegionEnvelope {regions}, &requested);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].via, "glutony");
        assert_eq!(result[0].country.as_deref(), Some("Brazil"));
    }

    #[test]
    fn sanitizes_region_fields() {
        assert_eq!(safe_location(Some("Paris, France".into())).as_deref(), Some("Paris, France"));
        assert_eq!(safe_location(Some("<script>".into())), None);
        assert_eq!(safe_location(Some("Unknown".into())), None);
    }
}
