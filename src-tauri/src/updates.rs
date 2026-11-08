//! Native, on-demand Thirst release checker.
//!
//! Connects only to GitHub's fixed public API endpoint, with no Roblox
//! session or browser credentials. This is intentionally NOT an updater:
//! unsigned GitHub Actions artifacts must never be auto-installed.

use serde::{Deserialize, Serialize};
use tauri::Manager;
use std::time::Duration;

const RELEASES_API: &str = "https://api.github.com/repos/N29dev/Thirst/releases/latest";
const RELEASES_PAGE: &str = "https://github.com/N29dev/Thirst/releases/latest";
const MAX_RESPONSE: usize = 96 * 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheck {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub release_url: String,
    pub notes: Option<String>,
    pub status: &'static str,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

fn inspect_release(current: &str, release: GithubRelease) -> Result<UpdateCheck, String> {
    if release.draft || release.prerelease {
        return Err("The newest GitHub release is not a stable release.".into());
    }
    let latest = release.tag_name.strip_prefix('v').unwrap_or(&release.tag_name);
    let latest = semver::Version::parse(latest)
        .map_err(|_| "GitHub's latest release has an invalid version tag.".to_owned())?;
    let current = semver::Version::parse(current)
        .map_err(|_| "Thirst's installed version is invalid.".to_owned())?;
    let available = latest > current;
    let notes = release.body
        .map(|body| body.trim().chars().take(1000).collect::<String>())
        .filter(|body| !body.is_empty());

    Ok(UpdateCheck {
        current_version: current.to_string(),
        latest_version: Some(latest.to_string()),
        update_available: available,
        release_url: RELEASES_PAGE.to_owned(),
        notes,
        status: if available { "available" } else { "up-to-date" },
    })
}

#[tauri::command]
pub async fn check_thirst_update(app: tauri::AppHandle) -> Result<UpdateCheck, String> {
    let current = app.package_info().version.to_string();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(7))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Thirst-release-check/0.4 (https://github.com/N29dev/Thirst)")
        .build()
        .map_err(|_| "Could not create the release checker.".to_owned())?;
    let response = client.get(RELEASES_API)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .send().await
        .map_err(|_| "Could not connect to GitHub to check for updates.".to_owned())?;

    if response.status().as_u16() == 404 {
        return Ok(UpdateCheck {
            current_version: current, latest_version: None,
            update_available: false, release_url: RELEASES_PAGE.into(),
            notes: None, status: "no-releases",
        });
    }
    if matches!(response.status().as_u16(), 403 | 429) {
        return Err("GitHub's public release check is temporarily rate-limited.".into());
    }
    if !response.status().is_success() {
        return Err(format!("GitHub release check returned HTTP {}.", response.status()));
    }

    if response.content_length().is_some_and(|length| length > MAX_RESPONSE as u64) {
        return Err("GitHub returned an unexpectedly large release response.".into());
    }
    let mut data = Vec::new();
    let mut response = response;
    while let Some(chunk) = response.chunk().await
        .map_err(|_| "Could not read the GitHub release response.".to_owned())? {
        if data.len().saturating_add(chunk.len()) > MAX_RESPONSE {
            return Err("GitHub release metadata exceeded the size limit.".into());
        }
        data.extend_from_slice(&chunk);
    }

    let release = serde_json::from_slice::<GithubRelease>(&data)
        .map_err(|_| "GitHub returned unexpected release metadata.".to_owned())?;
    inspect_release(&current, release)
}

#[tauri::command]
pub fn open_thirst_releases(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(RELEASES_PAGE.to_string(), None::<&str>)
        .map_err(|_| "Could not open Thirst's GitHub releases page.".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str) -> GithubRelease {
        GithubRelease {
            tag_name: tag.into(), body: Some("Bug fixes".into()),
            draft: false, prerelease: false,
        }
    }

    #[test]
    fn semver_detects_a_newer_release_without_installing_it() {
        let found = inspect_release("0.4.0", release("v0.5.0")).unwrap();
        assert!(found.update_available);
        assert_eq!(found.latest_version.as_deref(), Some("0.5.0"));
        assert_eq!(found.release_url, RELEASES_PAGE);
        assert_eq!(found.status, "available");
    }

    #[test]
    fn older_or_same_version_does_not_trigger_an_update() {
        assert!(!inspect_release("0.5.0", release("0.5.0")).unwrap().update_available);
        assert!(!inspect_release("0.5.0", release("0.4.1")).unwrap().update_available);
        assert!(inspect_release("0.4.0", release("not-a-version")).is_err());
    }

    #[test]
    fn prereleases_and_drafts_are_rejected() {
        let mut candidate = release("v0.5.0");
        candidate.prerelease = true;
        assert!(inspect_release("0.4.0", candidate).is_err());
        let mut candidate = release("v0.5.0");
        candidate.draft = true;
        assert!(inspect_release("0.4.0", candidate).is_err());
    }

    #[test]
    fn long_release_notes_are_capped() {
        let mut candidate = release("v0.5.0");
        candidate.body = Some("x".repeat(5000));
        assert_eq!(inspect_release("0.4.0", candidate)
            .unwrap().notes.unwrap().chars().count(), 1000);
    }
}
