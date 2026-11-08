//! Privacy-preserving, on-demand avatar preview for public Roblox servers.
//! Only anonymous public player tokens are cached briefly in Rust memory.
//! Neither Roblox player IDs nor tokens are returned to the Svelte frontend,
//! and this module never accesses the authenticated Roblox session.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{public_client, trusted_icon_url, validate_place_id, ServerPage};

const TOKEN_TTL: Duration = Duration::from_secs(300);
const MAX_CACHED_SERVERS: usize = 250;
const MAX_PREVIEW_AVATARS: usize = 5;

struct CachedTokens {
    saved_at: Instant,
    tokens: Vec<String>,
}

#[derive(Default)]
pub struct PreviewCache {
    servers: Mutex<HashMap<(u64, String), CachedTokens>>,
}

impl PreviewCache {
    pub fn remember(&self, place_id: u64, page: &ServerPage) {
        let Ok(mut cache) = self.servers.lock() else { return };
        cache.retain(|_, entry| entry.saved_at.elapsed() < TOKEN_TTL);
        // Bound memory even if someone paginates through many games/servers.
        if cache.len() > MAX_CACHED_SERVERS { cache.clear(); }
        for server in page.data.iter().take(25) {
            let Ok(uuid) = uuid::Uuid::parse_str(&server.id) else { continue };
            let tokens: Vec<String> = server.player_tokens.as_deref().unwrap_or(&[]).iter()
                .filter(|token| !token.is_empty() && token.len() <= 1024 && token.is_ascii())
                .take(MAX_PREVIEW_AVATARS)
                .cloned()
                .collect();
            if !tokens.is_empty() {
                cache.insert((place_id, uuid.to_string()), CachedTokens {
                    saved_at: Instant::now(), tokens
                });
            }
        }
    }

    fn get(&self, place_id: u64, server_id: &str) -> Result<Vec<String>, String> {
        let lock = self.servers.lock().map_err(|_| "Avatar preview cache unavailable.".to_owned())?;
        Ok(lock.get(&(place_id, server_id.to_owned()))
            .filter(|value| value.saved_at.elapsed() < TOKEN_TTL)
            .map(|value| value.tokens.clone())
            .unwrap_or_default())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    request_id: String,
    token: String,
    target_id: u64,
    #[serde(rename = "type")]
    thumbnail_type: &'static str,
    size: &'static str,
    format: &'static str,
    is_circular: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResultItem {
    #[serde(default)]
    request_id: Option<String>,
    #[serde(default)]
    image_url: Option<String>,
    #[serde(default)]
    state: Option<String>,
}

#[derive(Deserialize)]
struct Batch {
    data: Vec<ResultItem>,
}

/// Both modes use only the fixed Roblox thumbnail host. Cookie mode is
/// explicit opt-in, never the default. The bearer credential never crosses IPC.
async fn request_thumbnail_batch(requests: &[Request], authenticated: bool) -> Result<Batch, String> {
    use reqwest::header::COOKIE;

    let session = if authenticated {
        Some(crate::auth::thumbnail_session()?)
    } else {
        None
    };
    let client = public_client()?; // fixed HTTPS hosts; redirects disabled
    let endpoint = "https://thumbnails.roblox.com/v1/batch";
    let mut builder = client.post(endpoint).json(requests);
    if let Some(cookie) = &session {
        builder = builder.header(COOKIE, format!(".ROBLOSECURITY={cookie}"));
    }
    let mut response = builder.send().await
        .map_err(|_| "Could not reach Roblox's thumbnail service.".to_owned())?;

    // Roblox sometimes requires a CSRF challenge for cookie-authenticated
    // POSTs. Retry at most once to the SAME fixed host, never a redirect.
    if response.status().as_u16() == 403 && session.is_some() {
        if let Some(csrf) = response.headers().get("x-csrf-token").cloned() {
            let mut retry = client.post(endpoint).json(requests)
                .header("x-csrf-token", csrf);
            if let Some(cookie) = &session {
                retry = retry.header(COOKIE, format!(".ROBLOSECURITY={cookie}"));
            }
            response = retry.send().await
                .map_err(|_| "Roblox thumbnail authentication retry failed.".to_owned())?;
        }
    }

    if response.status().as_u16() == 401 {
        return Err("Roblox rejected the saved session. Check the account in Settings.".into());
    }
    if response.status().as_u16() == 403 {
        return Err("Roblox blocked the thumbnail request (HTTP 403), including with the optional session. This may be a token or service limitation.".into());
    }
    if response.status().as_u16() == 429 {
        return Err("Roblox rate-limited avatar previews. Try again later.".into());
    }
    if !response.status().is_success() {
        return Err(format!("Roblox avatar previews unavailable (HTTP {}).", response.status().as_u16()));
    }

    // Session rotation is accepted only from the fixed Roblox thumbnail host.
    if let Some(cookie) = &session {
        crate::auth::rotate_thumbnail_session(cookie, response.headers())?;
    }
    response.json::<Batch>().await
        .map_err(|_| "Roblox returned an unexpected thumbnail response.".to_owned())
}

#[tauri::command]
pub async fn server_avatar_preview(
    place_id: u64,
    server_id: String,
    authenticated: bool,
    cache: State<'_, PreviewCache>,
) -> Result<Vec<String>, String> {
    validate_place_id(place_id)?;
    let uuid = uuid::Uuid::parse_str(&server_id)
        .map_err(|_| "Invalid public Roblox server ID.".to_owned())?;
    // No requester-controlled token, URL, HTTP host, or authentication header.
    let tokens = cache.get(place_id, &uuid.to_string())?;
    if tokens.is_empty() {
        return Err("Roblox did not provide public avatar preview tokens for this loaded server. Refresh the list or try another server.".into());
    }

    let requests: Vec<Request> = tokens.into_iter()
        .map(|token| Request {
            request_id: format!("0:{token}:AvatarHeadshot:150x150:png:regular"),
            token,
            target_id: 0,
            thumbnail_type: "AvatarHeadShot",
            size: "150x150",
            format: "png",
            is_circular: false,
        })
        .collect();
    let data = request_thumbnail_batch(&requests, authenticated).await?;
    Ok(data.data.into_iter().filter_map(|item| {
        if item.state.as_deref() != Some("Completed") { return None; }
        item.image_url.filter(|value| value.len() <= 1200 && trusted_icon_url(value))
    }).take(MAX_PREVIEW_AVATARS).collect())
}

/// One bounded batch for the initial eight visible servers. No session cookie
/// is used. This is invoked only when a signed-in user opens/refreshes servers
/// with automatic previews enabled. Anonymous tokens never leave native Rust.
#[tauri::command]
pub async fn server_avatar_previews(
    place_id: u64,
    server_ids: Vec<String>,
    authenticated: bool,
    cache: State<'_, PreviewCache>,
) -> Result<HashMap<String, Vec<String>>, String> {
    validate_place_id(place_id)?;
    if server_ids.is_empty() || server_ids.len() > 8 {
        return Err("Choose between one and eight loaded servers for avatar previews.".into());
    }

    let mut requests = Vec::<Request>::new();
    // A single token might occur in multiple places; map the opaque request ID
    // to all corresponding public server UUIDs, without returning that ID to JS.
    let mut request_owners: HashMap<String, Vec<String>> = HashMap::new();
    let mut output: HashMap<String, Vec<String>> = HashMap::new();
    for id in server_ids {
        let uuid = uuid::Uuid::parse_str(&id)
            .map_err(|_| "Invalid server ID in avatar preview request.".to_owned())?
            .to_string();
        if output.contains_key(&uuid) { continue; }
        output.insert(uuid.clone(), Vec::new());
        for token in cache.get(place_id, &uuid)? {
            let request_id = format!("0:{token}:AvatarHeadshot:150x150:png:regular");
            request_owners.entry(request_id.clone()).or_default().push(uuid.clone());
            if !requests.iter().any(|item| item.request_id == request_id) {
                requests.push(Request {
                    request_id,
                    token,
                    target_id: 0,
                    thumbnail_type: "AvatarHeadShot",
                    size: "150x150",
                    format: "png",
                    is_circular: false,
                });
            }
        }
    }
    if requests.is_empty() {
        // Absence is normal: Roblox may omit public playerTokens.
        return Ok(output);
    }

    let results = request_thumbnail_batch(&requests, authenticated).await?;
    for item in results.data.into_iter().take(40) {
        if item.state.as_deref() != Some("Completed") { continue; }
        let Some(request_id) = item.request_id else { continue };
        let Some(owners) = request_owners.get(&request_id) else { continue };
        let Some(url) = item.image_url else { continue };
        if url.len() > 1200 || !trusted_icon_url(&url) { continue; }
        for id in owners {
            if let Some(images) = output.get_mut(id) {
                if images.len() < MAX_PREVIEW_AVATARS && !images.contains(&url) {
                    images.push(url.clone());
                }
            }
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PublicServer;

    #[test]
    fn limits_and_expires_anonymous_public_tokens() {
        let cache = PreviewCache::default();
        let page = ServerPage {
            data: vec![PublicServer {
                id: "550e8400-e29b-41d4-a716-446655440000".into(),
                playing: 8,
                max_players: 10,
                ping: Some(70.0),
                player_tokens: Some(vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into(), "f".into()]),
            }],
            next_page_cursor: None,
        };
        cache.remember(1818, &page);
        assert_eq!(cache.get(1818, "550e8400-e29b-41d4-a716-446655440000").unwrap().len(), 5);
        assert!(cache.get(1819, "550e8400-e29b-41d4-a716-446655440000").unwrap().is_empty());
    }
}
