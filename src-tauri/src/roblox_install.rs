//! Roblox installation discovery and launch diagnostics. All filesystem and
//! process access is confined to native Rust; the Svelte UI receives no
//! credentials or executable contents.
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::Manager;
use tauri_plugin_opener::OpenerExt;

const OFFICIAL_DOWNLOAD_PAGE: &str = "https://www.roblox.com/download";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RobloxInstallStatus {
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub source: &'static str,
    pub protocol: &'static str,
    pub running: Option<bool>,
    pub detail: String,
    pub custom_path: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CustomLocation {
    path: String,
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join("roblox-location.json"))
        .map_err(|_| "Could not access Thirst's local settings directory.".to_owned())
}

fn saved_override(app: &tauri::AppHandle) -> Option<PathBuf> {
    let file = settings_path(app).ok()?;
    let bytes = fs::read(file).ok()?;
    if bytes.len() > 4096 { return None; }
    let custom: CustomLocation = serde_json::from_slice(&bytes).ok()?;
    if custom.path.len() > 2048 { return None; }
    Some(PathBuf::from(custom.path))
}

fn valid_candidate(path: &Path) -> bool {
    if !path.is_absolute() { return false; }
    #[cfg(target_os = "macos")]
    {
        if !path.file_name().is_some_and(|s| s.to_string_lossy().eq_ignore_ascii_case("Roblox.app")) {
            return false;
        }
        let plist = path.join("Contents/Info.plist");
        let player = path.join("Contents/MacOS/RobloxPlayer");
        if !plist.is_file() || !player.is_file() { return false; }
        // Validate the actual bundle identity rather than just a folder name.
        return plist_field(&plist, "CFBundleIdentifier")
            .is_some_and(|id| id == "com.roblox.RobloxPlayer");
    }
    #[cfg(target_os = "windows")]
    {
        path.is_file() && path.file_name().is_some_and(|name|
            name.to_string_lossy().eq_ignore_ascii_case("RobloxPlayerBeta.exe"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    { let _ = path; false }
}

#[cfg(target_os = "macos")]
fn plist_field(plist: &Path, key: &str) -> Option<String> {
    let output = Command::new("/usr/bin/plutil")
        .args(["-extract", key, "raw", "-o", "-"])
        .arg(plist)
        .output().ok()?;
    if !output.status.success() || output.stdout.len() > 1024 { return None; }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn installed_version(path: &Path) -> Option<String> {
    #[cfg(target_os = "macos")]
    { plist_field(&path.join("Contents/Info.plist"), "CFBundleShortVersionString") }
    #[cfg(target_os = "windows")]
    {
        path.parent()?.file_name().map(|s| s.to_string_lossy().to_string())
            .filter(|s| s.starts_with("version-"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    { let _ = path; None }
}

fn candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/Applications/Roblox.app"));
        if let Some(home) = std::env::var_os("HOME") {
            paths.push(PathBuf::from(home).join("Applications/Roblox.app"));
        }
        // An indexed Spotlight lookup finds moved/renamed installations
        // without a full-disk walk. Output is bounded and every path validated.
        if let Ok(result) = Command::new("/usr/bin/mdfind")
            .arg("kMDItemFSName == 'Roblox.app'")
            .output()
        {
            if result.status.success() {
                for line in String::from_utf8_lossy(
                    &result.stdout[..result.stdout.len().min(24_576)]
                ).lines().take(30) {
                    if line.ends_with("/Roblox.app") {
                        paths.push(PathBuf::from(line));
                    }
                }
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let base = PathBuf::from(local).join("Roblox");
            paths.push(base.join("RobloxPlayerBeta.exe"));
            let versions = base.join("Versions");
            if let Ok(entries) = fs::read_dir(versions) {
                let mut versions: Vec<_> = entries.flatten()
                    .filter(|entry| entry.path().is_dir())
                    .map(|entry| {
                        let changed = entry.metadata().ok().and_then(|m| m.modified().ok());
                        (changed, entry.path())
                    }).collect();
                versions.sort_unstable_by(|a,b| b.0.cmp(&a.0));
                for (_, path) in versions.into_iter().take(32) {
                    paths.push(path.join("RobloxPlayerBeta.exe"));
                }
            }
        }
    }
    paths
}

fn protocol_registration() -> &'static str {
    #[cfg(target_os = "macos")]
    { return mac_handler_status(); }
    #[cfg(target_os = "windows")]
    {
        for key in [
            r"HKCU\Software\Classes\roblox\shell\open\command",
            r"HKCR\roblox\shell\open\command",
        ] {
            if let Ok(output) = Command::new("reg").args(["query", key, "/ve"]).output() {
                if output.status.success() {
                    let value = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
                    if value.contains("roblox") { return "registered"; }
                }
            }
        }
        return "missing";
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    { "unverified" }
}

#[cfg(target_os = "macos")]
fn mac_handler_status() -> &'static str {
    use std::ffi::{c_char, c_void, CString};
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringCreateWithCString(allocator: *const c_void, value: *const c_char, encoding: u32) -> *const c_void;
        fn CFStringGetCString(value: *const c_void, buffer: *mut c_char, buffer_size: isize, encoding: u32) -> u8;
        fn CFRelease(value: *const c_void);
    }
    #[link(name = "CoreServices", kind = "framework")]
    extern "C" {
        fn LSCopyDefaultHandlerForURLScheme(scheme: *const c_void) -> *const c_void;
    }
    // CoreFoundation kCFStringEncodingUTF8.
    const UTF8: u32 = 0x08000100;
    let scheme = CString::new("roblox").expect("static string");
    // SAFETY: These CoreFoundation functions use owned references. Both
    // returned references are released exactly once, including error paths.
    unsafe {
        let key = CFStringCreateWithCString(std::ptr::null(), scheme.as_ptr(), UTF8);
        if key.is_null() { return "unverified"; }
        let handler = LSCopyDefaultHandlerForURLScheme(key);
        CFRelease(key);
        if handler.is_null() { return "missing"; }
        let mut buf = [0i8; 512];
        let ok = CFStringGetCString(handler, buf.as_mut_ptr(), buf.len() as isize, UTF8);
        CFRelease(handler);
        if ok == 0 { return "unverified"; }
        let id = std::ffi::CStr::from_ptr(buf.as_ptr()).to_string_lossy().to_ascii_lowercase();
        if id.contains("roblox") { "registered" } else { "other-app" }
    }
}

fn roblox_running() -> Option<bool> {
    #[cfg(target_os = "macos")]
    {
        // Executable-name-only check; no process IDs or command lines leave Rust.
        let result = Command::new("/usr/bin/pgrep").args(["-x", "RobloxPlayer"]).status().ok()?;
        Some(result.success())
    }
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq RobloxPlayerBeta.exe", "/NH"])
            .output().ok()?;
        if !output.status.success() { return None; }
        let text = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
        Some(text.lines().any(|line| line.trim_start().starts_with("robloxplayerbeta.exe ")))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    { None }
}

fn inspect(app: &tauri::AppHandle) -> RobloxInstallStatus {
    let manual = saved_override(app);
    let mut locations = Vec::new();
    if let Some(path) = manual.as_ref() { locations.push(("custom", path.clone())); }
    locations.extend(candidates().into_iter().map(|path| ("automatic", path)));
    let found = locations.into_iter().find_map(|(source, candidate)| {
        if valid_candidate(&candidate) { Some((source, candidate)) } else { None }
    });
    let protocol = protocol_registration();
    let running = roblox_running();

    match found {
        Some((source, path)) => {
            let version = installed_version(&path);
            let custom_path = source == "custom";
            let detail = match protocol {
                "registered" => "Roblox installation and launch protocol detected. An actual game launch has not been tested.",
                "missing" => "Roblox is installed, but the roblox:// handler was not found. Launching may fail; try opening Roblox directly to re-register it.",
                "other-app" => "Another app handles roblox://. Thirst may open that handler rather than this installation.",
                _ => "Roblox installation detected. Protocol registration could not be verified.",
            };
            RobloxInstallStatus {
                installed: true, path: Some(path.to_string_lossy().into_owned()),
                version, source, protocol, running, detail: detail.to_string(), custom_path,
            }
        },
        None => RobloxInstallStatus {
            installed:false, path:None, version:None, source:"not-found",
            protocol, running, custom_path:manual.is_some(),
            detail: if manual.is_some() {
                "The saved custom path is no longer valid, and no other Roblox installation was detected. Choose another path or clear your override."
            } else {
                "No Roblox installation found in common locations or system discovery. Install Roblox, or select an existing installation."
            }.to_owned(),
        },
    }
}

#[tauri::command]
pub async fn roblox_install_status(app: tauri::AppHandle) -> Result<RobloxInstallStatus, String> {
    tauri::async_runtime::spawn_blocking(move || inspect(&app))
        .await.map_err(|_| "Could not inspect the Roblox installation.".to_owned())
}

#[tauri::command]
pub async fn set_roblox_install_path(
    app: tauri::AppHandle, path: Option<String>
) -> Result<RobloxInstallStatus, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let file = settings_path(&app)?;
        if let Some(path) = path {
            if path.is_empty() || path.len() > 2048 {
                return Err("Choose a valid Roblox application location.".to_owned());
            }
            let input = PathBuf::from(path);
            if !valid_candidate(&input) {
                return Err("The selected location is not a valid Roblox Player installation.".to_owned());
            }
            let canonical = fs::canonicalize(&input)
                .map_err(|_| "Could not validate the selected Roblox path.".to_owned())?;
            fs::create_dir_all(file.parent().ok_or("Settings location unavailable.")?)
                .map_err(|_| "Could not create Thirst's settings directory.".to_owned())?;
            let serialized = serde_json::to_vec(&CustomLocation {
                path: canonical.to_string_lossy().into_owned()
            }).map_err(|_| "Could not save the Roblox location.".to_owned())?;
            fs::write(file, serialized).map_err(|_| "Could not save the Roblox location.".to_owned())?;
        } else if let Err(err) = fs::remove_file(file) {
            if err.kind() != std::io::ErrorKind::NotFound {
                return Err("Could not clear the saved Roblox path.".to_owned());
            }
        }
        Ok(inspect(&app))
    }).await.map_err(|_| "Could not update Roblox installation settings.".to_owned())?
}

#[tauri::command]
pub fn open_roblox_install_folder(app: tauri::AppHandle) -> Result<(), String> {
    let status = inspect(&app);
    let path = status.path.ok_or("No Roblox installation found.")?;
    #[cfg(target_os = "macos")]
    {
        return Command::new("/usr/bin/open").arg("-R").arg(path).status()
            .map_err(|_| "Could not reveal Roblox in Finder.".to_owned())
            .and_then(|status| if status.success() { Ok(()) } else { Err("Finder could not reveal Roblox.".to_owned()) });
    }
    #[cfg(target_os = "windows")]
    {
        return Command::new("explorer.exe")
            .arg(format!("/select,{}", path)).status()
            .map_err(|_| "Could not open Roblox in Explorer.".to_owned())
            .and_then(|status| if status.success() { Ok(()) } else { Err("Explorer could not reveal Roblox.".to_owned()) });
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    { let _ = path; Err("This platform is unsupported.".to_owned()) }
}

#[tauri::command]
pub fn open_roblox_download_page(app: tauri::AppHandle) -> Result<(), String> {
    app.opener().open_url(OFFICIAL_DOWNLOAD_PAGE.to_string(), None::<&str>)
        .map_err(|_| "Could not open Roblox's official download page.".to_owned())
}

#[tauri::command]
pub async fn roblox_running_status() -> Option<bool> {
    tauri::async_runtime::spawn_blocking(roblox_running).await.ok().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn custom_path_json_does_not_include_secrets() {
        let val = serde_json::to_string(&CustomLocation { path: "/Applications/Roblox.app".into() }).unwrap();
        assert!(val.contains("Roblox.app"));
        assert!(!val.contains("cookie"));
    }
    #[test]
    fn non_absolute_path_is_rejected() {
        assert!(!valid_candidate(Path::new("Roblox.app")));
        assert!(!valid_candidate(Path::new("../RobloxPlayerBeta.exe")));
    }
    #[test]
    fn fake_file_is_not_a_valid_player_installation() {
        assert!(!valid_candidate(Path::new("/does-not-exist/Roblox.app")));
    }
}
