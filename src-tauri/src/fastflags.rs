//! Roblox Player Fast Flags, limited to Roblox's public local-config allowlist.
//! No session credentials, Roblox account identifiers, or network access.
//! Only writes ClientAppSettings.json at a fixed per-OS Player settings path.
//!
//! macOS: ~/Library/Preferences/com.roblox.Roblox/ClientAppSettings.json
//! Windows: %LOCALAPPDATA%/Roblox/ClientSettings/ClientAppSettings.json
//! (The bundle/version-specific legacy paths are intentionally not changed.)

use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const MAX_FILE_SIZE: u64 = 256 * 1024;
const MAX_FLAGS: usize = 18;

// Published by Roblox on 2025-09-29. Roblox may revise the allowlist; this
// snapshot cannot guarantee that a given version of the Player honors a flag.
const ALLOWED: &[&str] = &[
    "DFIntCSGLevelOfDetailSwitchingDistance",
    "DFIntCSGLevelOfDetailSwitchingDistanceL12",
    "DFIntCSGLevelOfDetailSwitchingDistanceL23",
    "DFIntCSGLevelOfDetailSwitchingDistanceL34",
    "FFlagHandleAltEnterFullscreenManually",
    "DFFlagTextureQualityOverrideEnabled",
    "DFIntTextureQualityOverride",
    "FIntDebugForceMSAASamples",
    "DFFlagDisableDPIScale",
    "FFlagDebugGraphicsPreferD3D11",
    "FFlagDebugSkyGray",
    "DFFlagDebugPauseVoxelizer",
    "DFIntDebugFRMQualityLevelOverride",
    "FIntFRMMaxGrassDistance",
    "FIntFRMMinGrassDistance",
    "FFlagDebugGraphicsPreferVulkan",
    "FFlagDebugGraphicsPreferOpenGL",
    "FIntGrassMovementReducedMotionFactor",
];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FastFlagStatus {
    path: String,
    platform: &'static str,
    player_detected: bool,
    file_exists: bool,
    flags: BTreeMap<String, String>,
    ignored_keys: Vec<String>,
    note: String,
}

fn flag_is_allowed(name: &str) -> bool {
    ALLOWED.contains(&name)
}

fn parse_flag_value(name: &str, value: &Value) -> Result<String, String> {
    let is_boolean = name.starts_with("FFlag") || name.starts_with("DFFlag");
    let raw = match value {
        Value::String(text) => text.trim().to_owned(),
        Value::Bool(b) if is_boolean => if *b { "True" } else { "False" }.into(),
        Value::Number(n) if !is_boolean => n.to_string(),
        _ => return Err(format!("{name} has an invalid value type.")),
    };
    if is_boolean {
        if raw.eq_ignore_ascii_case("true") { return Ok("True".into()); }
        if raw.eq_ignore_ascii_case("false") { return Ok("False".into()); }
        return Err(format!("{name} must be True or False."));
    }
    let number = raw.parse::<i32>()
        .map_err(|_| format!("{name} must be a signed 32-bit integer."))?;
    Ok(number.to_string())
}

fn settings_path() -> Result<(PathBuf, &'static str, bool), String> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")
            .ok_or("Cannot locate the current macOS user directory.")?;
        let path = PathBuf::from(home)
            .join("Library/Preferences/com.roblox.Roblox/ClientAppSettings.json");
        let installed = Path::new("/Applications/Roblox.app").exists() ||
            std::env::var_os("HOME").is_some_and(|h|
                PathBuf::from(h).join("Applications/Roblox.app").exists());
        return Ok((path, "macOS", installed));
    }
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var_os("LOCALAPPDATA")
            .ok_or("Cannot locate Windows LocalAppData.")?;
        let base = PathBuf::from(base).join("Roblox");
        let installed = base.exists();
        return Ok((base.join("ClientSettings/ClientAppSettings.json"), "Windows", installed));
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err("Fast Flags are supported only on macOS and Windows.".into())
    }
}

fn read_document(path: &Path) -> Result<Map<String, Value>, String> {
    if !path.exists() {
        return Ok(Map::new());
    }
    // Never modify a symbolic link that might point outside Roblox settings.
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| "Cannot read Roblox Player settings metadata.".to_owned())?;
    if !metadata.file_type().is_file() {
        return Err("Roblox Player settings path is not a regular file.".into());
    }
    if metadata.len() > MAX_FILE_SIZE {
        return Err("The existing Roblox settings file is unexpectedly large.".into());
    }
    let bytes = fs::read(path)
        .map_err(|_| "Cannot read Roblox Player settings. Check file permissions.".to_owned())?;
    if bytes.is_empty() {
        return Ok(Map::new());
    }
    match serde_json::from_slice::<Value>(&bytes) {
        Ok(Value::Object(map)) => Ok(map),
        _ => Err("Roblox ClientAppSettings.json is not a valid JSON object. Fix it manually before applying presets.".into()),
    }
}

fn read_flags(path: &Path) -> Result<(BTreeMap<String, String>, Vec<String>), String> {
    let document = read_document(path)?;
    let mut flags = BTreeMap::new();
    let mut ignored = Vec::new();
    for (key, value) in document {
        if flag_is_allowed(&key) {
            if let Ok(value) = parse_flag_value(&key, &value) {
                flags.insert(key, value);
            } else {
                ignored.push(key);
            }
        } else {
            ignored.push(key);
        }
    }
    ignored.truncate(100);
    Ok((flags, ignored))
}

#[tauri::command]
pub fn read_fast_flags() -> Result<FastFlagStatus, String> {
    let (path, platform, player_detected) = settings_path()?;
    let (flags, ignored_keys) = read_flags(&path)?;
    Ok(FastFlagStatus {
        path: path.to_string_lossy().into_owned(),
        platform,
        player_detected,
        file_exists: path.exists(),
        flags,
        ignored_keys,
        note: if platform == "macOS" {
            "User-level Roblox Player preferences. Roblox updates and client policies can affect which values apply. Legacy app-bundle flags are not modified.".into()
        } else {
            "Roblox Player ClientSettings in LocalAppData. Unsupported keys may be ignored by Roblox. Thirst does not modify version-specific client installations.".into()
        },
    })
}

/// Applies only allowlisted keys. Preserves unrelated pre-existing JSON keys
/// rather than unexpectedly deleting another tool's settings. No arbitrary
/// file paths may cross IPC. An original-file backup is saved once on first
/// modification, and saving has no effect until the Roblox client restarts.
#[tauri::command]
pub fn write_fast_flags(flags: BTreeMap<String, String>) -> Result<FastFlagStatus, String> {
    let (path, _, _) = settings_path()?;
    if flags.len() > MAX_FLAGS {
        return Err("Too many flags; only Roblox's published Player allowlist is supported.".into());
    }
    let mut converted = BTreeMap::new();
    for (key, value) in flags {
        if !flag_is_allowed(&key) {
            return Err(format!("{key} is not on Roblox's published Player allowlist."));
        }
        // Use JSON strings for maximum compatibility with Roblox's local
        // settings parser. The input must match the expected flag data type.
        converted.insert(key.clone(), parse_flag_value(&key, &Value::String(value))?);
    }
    let mut existing = read_document(&path)?;
    if let Some(parent) = path.parent() {
        if parent.exists() {
            let metadata = fs::symlink_metadata(parent)
                .map_err(|_| "Cannot access Roblox Player settings folder.".to_owned())?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err("Roblox Player settings folder is not a normal directory.".into());
            }
        }
        fs::create_dir_all(parent)
            .map_err(|_| "Cannot create Roblox ClientSettings folder. Check permissions.".to_owned())?;
    }
    if path.exists() {
        let backup = path.with_file_name("ClientAppSettings.thirst-backup.json");
        if !backup.exists() {
            fs::copy(&path, &backup)
                .map_err(|_| "Could not back up existing Roblox settings; no changes were made.".to_owned())?;
        }
    }
    // Only overwrite keys explicitly owned by this editor; unrelated legacy
    // keys are carried forward unchanged, but Roblox may ignore them.
    let known: BTreeSet<&str> = ALLOWED.iter().copied().collect();
    existing.retain(|key, _| !known.contains(key.as_str()));
    for (key, value) in converted {
        existing.insert(key, Value::String(value));
    }
    let encoded = serde_json::to_vec_pretty(&Value::Object(existing))
        .map_err(|_| "Could not serialize Roblox Player settings.".to_owned())?;
    if encoded.len() > MAX_FILE_SIZE as usize {
        return Err("The Roblox settings file is too large to save.".into());
    }
    fs::write(&path, encoded)
        .map_err(|_| "Cannot save Roblox settings. Quit Roblox and check file permissions.".to_owned())?;
    read_fast_flags()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_official_allowlist_and_rejects_old_fps_hacks() {
        assert_eq!(ALLOWED.len(), 18);
        assert!(flag_is_allowed("DFIntTextureQualityOverride"));
        assert!(flag_is_allowed("FIntDebugForceMSAASamples"));
        assert!(!flag_is_allowed("DFIntTaskSchedulerTargetFps"));
        assert!(!flag_is_allowed("FFlagDebugDisableTelemetryV2"));
    }

    #[test]
    fn parses_boolean_and_numeric_values_correctly() {
        assert_eq!(parse_flag_value("DFFlagTextureQualityOverrideEnabled",
            &Value::Bool(true)).unwrap(), "True");
        assert_eq!(parse_flag_value("DFFlagTextureQualityOverrideEnabled",
            &Value::String("false".into())).unwrap(), "False");
        assert_eq!(parse_flag_value("FIntDebugForceMSAASamples",
            &Value::Number(4.into())).unwrap(), "4");
        assert!(parse_flag_value("FIntDebugForceMSAASamples",
            &Value::String("NaN".into())).is_err());
        assert!(parse_flag_value("FFlagDebugSkyGray",
            &Value::String("10".into())).is_err());
    }

    #[test]
    fn reads_known_values_and_keeps_unsupported_separate() {
        let folder = std::env::temp_dir().join(format!(
            "thirst-fastflags-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&folder).unwrap();
        let path = folder.join("ClientAppSettings.json");
        fs::write(&path, r#"{"DFFlagTextureQualityOverrideEnabled":"True","DFIntTextureQualityOverride":2,"DFIntTaskSchedulerTargetFps":999}"#).unwrap();
        let (flags, ignored) = read_flags(&path).unwrap();
        assert_eq!(flags.get("DFIntTextureQualityOverride").map(String::as_str),Some("2"));
        assert!(ignored.contains(&"DFIntTaskSchedulerTargetFps".to_string()));
        let _ = fs::remove_dir_all(&folder);
    }
}
