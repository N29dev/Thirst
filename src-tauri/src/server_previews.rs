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
            let tokens: Vec<String> = server.player_tokens.iter()
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
    image_url: Option<String>,
    #[serde(default)]
    state: Option<String>,
}

#[derive(Deserialize)]
struct Batch {
    data: Vec<ResultItem>,
}

#[tauri::command]
pub async fn server_avatar_preview(
    place_id: u64,
    server_id: String,
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

    let requests: Vec<Request> = tokens.into_iter().enumerate()
        .map(|(index, token)| Request {
            request_id: format!("thirst-preview-{index}"),
            token,
            thumbnail_type: "AvatarHeadShot",
            size: "150x150",
            format: "Png",
            is_circular: false,
        })
        .collect();
    let client = public_client()?;
    let response = client.post("https://thumbnails.roblox.com/v1/batch")
        .json(&requests)
        .send().await
        .map_err(|_| "Roblox's public thumbnail service is currently unreachable.".to_owned())?;
    if response.status().as_u16() == 429 {
        return Err("Roblox rate-limited avatar previews. Try later.".into());
    }
    if !response.status().is_success() {
        return Err(format!("Roblox avatar previews unavailable (HTTP {}).", response.status().as_u16()));
    }
    let data = response.json::<Batch>().await
        .map_err(|_| "Roblox returned an unexpected avatar preview response.".to_owned())?;
    Ok(data.data.into_iter().filter_map(|item| {
        if item.state.as_deref() != Some("Completed") { return None; }
        item.image_url.filter(|value| value.len() <= 1200 && trusted_icon_url(value))
    }).take(MAX_PREVIEW_AVATARS).collect())
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
                player_tokens: vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into(), "f".into()],
            }],
            next_page_cursor: None,
        };
        cache.remember(1818, &page);
        assert_eq!(cache.get(1818, "550e8400-e29b-41d4-a716-446655440000").unwrap().len(), 5);
        assert!(cache.get(1819, "550e8400-e29b-41d4-a716-446655440000").unwrap().is_empty());
    }
}
