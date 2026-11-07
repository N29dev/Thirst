//! Native Thirst API. The UI cannot pass arbitrary URLs or shell commands.
//! There is no session-cookie access in this early developer preview.

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
    #[serde(default)]
    fps: Option<f64>,
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

    let response = client
        .get(format!("https://games.roblox.com/v1/games/{place_id}/servers/Public"))
        .query(&query)
        .send()
        .await
        .map_err(|_| "Roblox server list is currently unreachable.".to_owned())?;

    if response.status().as_u16() == 429 {
        return Err("Roblox rate-limited this request. Try again later.".to_owned());
    }
    if !response.status().is_success() {
        return Err(format!(
            "Roblox returned HTTP {} for this server list.",
            response.status().as_u16()
        ));
    }

    response
        .json::<ServerPage>()
        .await
        .map_err(|_| "Roblox returned an unexpected server response.".to_owned())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            launch_place,
            list_public_servers,
            platform_name
        ])
        .run(tauri::generate_context!())
        .expect("error while starting Thirst");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_places() {
        assert!(make_launch_url(0, None).is_err());
        assert!(make_launch_url(u64::MAX, None).is_err());
        assert!(make_launch_url(123, Some("not-a-server-id")).is_err());
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
