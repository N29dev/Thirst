//! Native Thirst API. The UI cannot pass arbitrary URLs or shell commands.
//! Session credentials are isolated in auth.rs, never returned over Tauri IPC.

mod auth;
mod server_previews;
mod rovalra;

use std::time::Duration;
use tauri_plugin_opener::OpenerExt;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PublicServer {
    id: String,
    playing: u32,
    max_players: u32,
    #[serde(default)]
    ping: Option<f64>,
    // Anonymous public player tokens never leave native Rust. The webview only
    // receives validated Roblox CDN image URLs when a user requests a preview.
    #[serde(default, skip_serializing)]
    player_tokens: Option<Vec<String>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServerPage {
    data: Vec<PublicServer>,
    next_page_cursor: Option<String>,
}

fn validate_place_id(id: u64) -> Result<(), String> {
    if id == 0 || id > 9_007_199_254_740_991 {
        return Err("Invalid Roblox place ID.".to_owned());
    }
    Ok(())
}

fn make_launch_url(place_id: u64, job_id: Option<&str>) -> Result<String, String> {
    validate_place_id(place_id)?;
    let mut url = format!("roblox://experiences/start?placeId={place_id}");
    if let Some(job) = job_id {
        let id = uuid::Uuid::parse_str(job)
            .map_err(|_| "Invalid Roblox server ID.".to_owned())?;
        url.push_str("&gameInstanceId=");
        url.push_str(&id.to_string());
    }
    Ok(url)
}

#[tauri::command]
fn launch_place(
    app: tauri::AppHandle,
    place_id: u64,
    job_id: Option<String>,
) -> Result<(), String> {
    let url = make_launch_url(place_id, job_id.as_deref())?;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|_| "Could not open Roblox. Make sure Roblox is installed.".to_owned())
}

#[tauri::command]
fn platform_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "windows") {
        "Windows"
    } else {
        "Desktop"
    }
}

#[tauri::command]
async fn list_public_servers(
    place_id: u64,
    cursor: Option<String>,
    authenticated: bool,
    previews: tauri::State<'_, server_previews::PreviewCache>,
) -> Result<ServerPage, String> {
    validate_place_id(place_id)?;
    // Roblox host is fixed; no caller-controlled host, cookie or redirect.
    let mut query = vec![
        ("limit", "25".to_owned()),
        ("sortOrder", "Desc".to_owned()),
        ("excludeFullGames", "false".to_owned()),
    ];
    if let Some(next) = cursor {
        if next.is_empty() || next.len() > 512 || !next.is_ascii() {
            return Err("Invalid server pagination cursor.".to_owned());
        }
        query.push(("cursor", next));
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(9))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Thirst/0.1 (+https://github.com/N29dev/Thirst)")
        .build()
        .map_err(|_| "Could not configure the network client.".to_owned())?;

    let mut request = client
        .get(format!("https://games.roblox.com/v1/games/{place_id}/servers/Public"))
        .query(&query);
    // Explicit opt-in only. Never to RoValra, CDN hosts or the frontend.
    let session = if authenticated { Some(auth::thumbnail_session()?) } else { None };
    if let Some(cookie) = &session {
        request = request.header(reqwest::header::COOKIE, format!(".ROBLOSECURITY={cookie}"));
    }
    let response = request.send().await
        .map_err(|_| "Roblox server list is currently unreachable.".to_owned())?;
    if let Some(cookie) = &session {
        // Only the fixed games.roblox.com host, redirects disabled.
        auth::rotate_thumbnail_session(cookie, response.headers())?;
    }

    if response.status().as_u16() == 429 {
        return Err("Roblox rate-limited this request. Try again later.".to_owned());
    }
    if authenticated && matches!(response.status().as_u16(), 401 | 403) {
        return Err("Roblox rejected the optional authenticated server request. Turn off session-backed avatars in Settings to use public mode.".into());
    }
    if !response.status().is_success() {
        return Err(format!(
            "Roblox returned HTTP {} for this server list.",
            response.status().as_u16()
        ));
    }

    let page = response
        .json::<ServerPage>()
        .await
        .map_err(|_| "Roblox returned an unexpected server response.".to_owned())?;
    previews.remember(place_id, &page);
    Ok(page)
}


#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UniverseLookup {
    universe_id: u64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameInfo {
    root_place_id: u64,
    name: String,
    #[serde(default)]
    playing: Option<u64>,
}

#[derive(Debug, serde::Deserialize)]
struct GamesEnvelope {
    data: Vec<GameInfo>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Thumbnail {
    image_url: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ThumbnailsEnvelope {
    data: Vec<Thumbnail>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct GameMetadata {
    universe_id: u64,
    root_place_id: u64,
    name: String,
    playing: Option<u64>,
    icon_url: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DiscoveredPlace {
    id: u64,
    name: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubplacesPage {
    data: Vec<DiscoveredPlace>,
    next_page_cursor: Option<String>,
}

fn public_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(9))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Thirst/0.1 (+https://github.com/N29dev/Thirst)")
        .build()
        .map_err(|_| "Could not configure Roblox metadata requests.".to_owned())
}

fn validate_cursor(cursor: &str) -> Result<(), String> {
    if cursor.is_empty() || cursor.len() > 512 || !cursor.is_ascii() {
        return Err("Invalid pagination cursor.".to_owned());
    }
    Ok(())
}

// Do not permit arbitrary image URLs to reach the webview.
// Only HTTPS content hosted on Roblox's image CDN is accepted.
fn trusted_icon_url(url: &str) -> bool {
    match reqwest::Url::parse(url) {
        Ok(uri) => {
            if uri.scheme() != "https" || uri.username() != "" || uri.password().is_some() ||
                uri.port().is_some() {
                return false;
            }
            uri.host_str().is_some_and(|host| host == "rbxcdn.com" || host.ends_with(".rbxcdn.com"))
        }
        Err(_) => false,
    }
}

async fn check_public_response(
    response: reqwest::Response,
    feature: &str,
) -> Result<reqwest::Response, String> {
    match response.status().as_u16() {
        200..=299 => Ok(response),
        401 | 403 => Err(format!("Roblox restricted public {feature} access. Add the place manually.")),
        429 => Err(format!("Roblox rate-limited {feature}. Please try again later.")),
        code => Err(format!("Roblox {feature} is unavailable (HTTP {code}).")),
    }
}

#[tauri::command]
async fn lookup_game(place_id: u64) -> Result<GameMetadata, String> {
    validate_place_id(place_id)?;
    let client = public_client()?;

    let universe = client
        .get(format!("https://apis.roblox.com/universes/v1/places/{place_id}/universe"))
        .send().await
        .map_err(|_| "Could not reach Roblox to identify this experience.".to_owned())?;
    let universe = check_public_response(universe, "experience lookup").await?
        .json::<UniverseLookup>().await
        .map_err(|_| "Roblox did not recognize this place ID.".to_owned())?;
    validate_place_id(universe.universe_id)?;

    let details = client
        .get("https://games.roblox.com/v1/games")
        .query(&[("universeIds", universe.universe_id.to_string())])
        .send().await
        .map_err(|_| "Could not retrieve Roblox experience details.".to_owned())?;
    let mut details = check_public_response(details, "experience details").await?
        .json::<GamesEnvelope>().await
        .map_err(|_| "Roblox returned invalid experience details.".to_owned())?.data;
    let game = details.pop().ok_or("No public experience details were found for that place.")?;
    validate_place_id(game.root_place_id)?;

    // Game icons are optional. A timeout, moderation delay or missing thumbnail
    // must never prevent a game from being bookmarked.
    let icon_url = match client
        .get("https://thumbnails.roblox.com/v1/games/icons")
        .query(&[
            ("universeIds", universe.universe_id.to_string()),
            ("size", "150x150".to_owned()),
            ("format", "Png".to_owned()),
            ("isCircular", "false".to_owned()),
        ])
        .send().await
    {
        Ok(response) if response.status().is_success() => {
            response.json::<ThumbnailsEnvelope>().await.ok()
                .and_then(|envelope| envelope.data.into_iter().next())
                .and_then(|item| item.image_url)
                .filter(|uri| trusted_icon_url(uri))
        }
        _ => None,
    };

    Ok(GameMetadata {
        universe_id: universe.universe_id,
        root_place_id: game.root_place_id,
        name: game.name.chars().take(100).collect(),
        playing: game.playing,
        icon_url,
    })
}

#[tauri::command]
async fn discover_subplaces(
    universe_id: u64,
    cursor: Option<String>,
) -> Result<SubplacesPage, String> {
    validate_place_id(universe_id)?;
    let mut query = vec![("limit", "50".to_owned())];
    if let Some(next) = cursor {
        validate_cursor(&next)?;
        query.push(("cursor", next));
    }
    let client = public_client()?;
    let response = client
        .get(format!("https://develop.roblox.com/v1/universes/{universe_id}/places"))
        .query(&query)
        .send().await
        .map_err(|_| "Could not reach Roblox subplace discovery.".to_owned())?;
    let mut page = check_public_response(response, "subplace discovery").await?
        .json::<SubplacesPage>().await
        .map_err(|_| "Roblox returned an unexpected subplace response.".to_owned())?;
    page.data.retain(|place| validate_place_id(place.id).is_ok());
    page.data.truncate(50);
    page.next_page_cursor = page.next_page_cursor.filter(|value| validate_cursor(value).is_ok());
    Ok(page)
}

pub fn run() {
    tauri::Builder::default()
        .manage(auth::AuthState::default())
        .manage(server_previews::PreviewCache::default())
        .manage(rovalra::RegionProviderState::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            launch_place,
            list_public_servers,
            rovalra::lookup_rovalra_regions,
            server_previews::server_avatar_preview,
            server_previews::server_avatar_previews,
            lookup_game,
            discover_subplaces,
            platform_name,
            auth::begin_roblox_consent,
            auth::open_roblox_login,
            auth::roblox_login_status,
            auth::reload_roblox_login,
            auth::finish_roblox_login,
            auth::connected_roblox_account,
            auth::lock_roblox_session,
            auth::disconnect_roblox_account,
            auth::cancel_roblox_login
        ])
        .run(tauri::generate_context!())
        .expect("error while starting Thirst");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_preview_tokens_never_cross_frontend_serialization() {
        let server = PublicServer {
            id: "550e8400-e29b-41d4-a716-446655440000".into(),
            playing: 4,
            max_players: 10,
            ping: None,
            player_tokens: Some(vec!["SENSITIVE_PUBLIC_TOKEN".into()]),
        };
        let json = serde_json::to_string(&server).expect("serialize");
        assert!(!json.contains("playerTokens"));
        assert!(!json.contains("SENSITIVE_PUBLIC_TOKEN"));
        assert!(json.contains("maxPlayers"));
        let from_null: PublicServer = serde_json::from_str(
            r#"{"id":"550e8400-e29b-41d4-a716-446655440000","playing":1,"maxPlayers":10,"playerTokens":null}"#
        ).expect("nullable tokens");
        assert!(from_null.player_tokens.is_none());
    }

    #[test]
    fn rejects_invalid_places() {
        assert!(make_launch_url(0, None).is_err());
        assert!(make_launch_url(u64::MAX, None).is_err());
        assert!(make_launch_url(123, Some("not-a-server-id")).is_err());
    }

    #[test]
    fn only_trusted_roblox_cdn_icons_are_accepted() {
        assert!(trusted_icon_url("https://tr.rbxcdn.com/123/150/150/Image/Png"));
        assert!(!trusted_icon_url("http://tr.rbxcdn.com/123"));
        assert!(!trusted_icon_url("https://tr.rbxcdn.com.evil.test/icon.png"));
        assert!(!trusted_icon_url("https://name@tr.rbxcdn.com/icon.png"));
    }

    #[test]
    fn builds_expected_roblox_protocol() {
        assert_eq!(
            make_launch_url(1818, None).unwrap(),
            "roblox://experiences/start?placeId=1818"
        );
        assert_eq!(
            make_launch_url(1818, Some("550e8400-e29b-41d4-a716-446655440000")).unwrap(),
            "roblox://experiences/start?placeId=1818&gameInstanceId=550e8400-e29b-41d4-a716-446655440000"
        );
    }
}
