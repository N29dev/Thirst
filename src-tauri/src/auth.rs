//! Native-only Roblox session integration for Thirst Alpha 0.3.
//! A Roblox session cookie is effectively a password. Never expose it to
//! Svelte, the Tauri IPC return value, logs, extensions, or third-party hosts.

use std::sync::Mutex;
#[cfg(target_os = "macos")]
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use reqwest::header::{COOKIE, SET_COOKIE};
use serde::{Deserialize, Serialize};
use tauri::{
    webview::{NewWindowResponse, WebviewWindowBuilder},
    AppHandle, Manager, State, Url, WebviewUrl,
};

const CONSENT_SECS: u64 = 15;
const KEYRING_SERVICE: &str = "dev.n29.thirst.roblox";
const KEYRING_NAME: &str = "primary-session";
#[cfg(target_os = "macos")]
const PENDING_KEYRING_NAME: &str = "pending-login";
#[cfg(target_os = "macos")]
const MACOS_HELPER: &str = concat!(env!("OUT_DIR"), "/ThirstLogin");

#[derive(Default)]
pub struct AuthState {
    started: Mutex<Option<Instant>>,
    #[cfg(target_os = "macos")]
    helper: Mutex<Option<Child>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedAccount {
    id: u64,
    name: String,
    display_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RobloxUser {
    id: u64,
    name: String,
    display_name: String,
}

fn roblox_login_host(url: &Url) -> bool {
    if url.scheme() != "https" || url.username() != "" || url.password().is_some() {
        return false;
    }
    url.host_str().is_some_and(|hostname| {
        hostname == "roblox.com" || hostname.ends_with(".roblox.com")
    })
}

fn keyring_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_NAME)
        .map_err(|_| "Could not initialize your operating system's secure credential store.".into())
}

// macOS uses a separate native Swift WKWebView, like AppleBlox's browser
// approach. It writes a *pending* cookie into Keychain; no cookie is transferred
// by stdout, command arguments, filesystem, IPC, or a frontend script.
#[cfg(target_os = "macos")]
fn pending_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, PENDING_KEYRING_NAME)
        .map_err(|_| "Could not initialize the temporary macOS Keychain entry.".into())
}

#[cfg(target_os = "macos")]
fn read_pending_session() -> Result<Option<String>, String> {
    match pending_entry()?.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("Could not read the pending macOS Keychain login entry.".into()),
    }
}

#[cfg(target_os = "macos")]
fn delete_pending_session() -> Result<(), String> {
    match pending_entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("Could not clear the pending macOS Keychain login entry.".into()),
    }
}

#[cfg(target_os = "macos")]
fn stop_native_helper(state: &AuthState) -> Result<(), String> {
    let mut helper = state.helper.lock()
        .map_err(|_| "Native login state could not be accessed.".to_owned())?;
    if let Some(mut child) = helper.take() {
        if child.try_wait().ok().flatten().is_none() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn start_native_helper(state: &AuthState) -> Result<(), String> {
    let mut helper = state.helper.lock()
        .map_err(|_| "Native login state could not be accessed.".to_owned())?;
    if let Some(child) = helper.as_mut() {
        if child.try_wait().map_err(|_| "Could not inspect the sign-in helper.".to_owned())?.is_none() {
            return Ok(());
        }
    }
    // Clear only the pending session. Never replace a stored account until
    // verify_cookie succeeds in finish_roblox_login.
    delete_pending_session()?;
    let child = Command::new(MACOS_HELPER)
        .arg(std::process::id().to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "Could not launch native macOS login. Xcode tools and a trusted Thirst build are required.".to_owned())?;
    *helper = Some(child);
    Ok(())
}

// Errors must never include cookie values, body text or request headers.
fn save_session(token: &str) -> Result<(), String> {
    keyring_entry()?
        .set_password(token)
        .map_err(|_| "macOS Keychain / Windows Credential Manager could not save the session.".into())
}

fn read_session() -> Result<Option<String>, String> {
    match keyring_entry()?.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("Cannot access your saved Roblox session in the OS credential store.".into()),
    }
}

fn delete_session() -> Result<(), String> {
    match keyring_entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("Could not remove the saved Roblox session from the OS credential store.".into()),
    }
}

fn rotated_cookie(headers: &reqwest::header::HeaderMap) -> Option<String> {
    // Roblox may rotate .ROBLOSECURITY in an HTTP Set-Cookie header.
    // Only trust cookies from the *fixed* authenticated users.roblox.com host
    // (this function must never be used with a redirect-following client).
    for header in headers.get_all(SET_COOKIE).iter() {
        let Ok(text) = header.to_str() else { continue };
        let Some(value) = text.split(';').next()
            .and_then(|cookie| cookie.trim().strip_prefix(".ROBLOSECURITY=")) else { continue };
        if !value.is_empty() && value.len() <= 8192 && !value.contains('\n') && !value.contains('\r') {
            return Some(value.to_owned());
        }
    }
    None
}

async fn verify_cookie(token: &str) -> Result<(ConnectedAccount, Option<String>), String> {
    // This is the ONLY host to which this module sends the cookie.
    // Never share it with game metadata, analytics, RoValra, Cloudflare or plugins.
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Thirst/0.3 (https://github.com/N29dev/Thirst)")
        .build()
        .map_err(|_| "Could not create a secure Roblox session request.".to_owned())?;

    let response = client
        .get("https://users.roblox.com/v1/users/authenticated")
        .header(COOKIE, format!(".ROBLOSECURITY={token}"))
        .send()
        .await
        .map_err(|_| "Could not contact Roblox to check your session.".to_owned())?;

    match response.status().as_u16() {
        200 => {}
        401 | 403 => return Err("Roblox did not accept that session. Complete sign-in in the official window.".into()),
        429 => return Err("Roblox is rate-limiting login verification. Try later.".into()),
        status => return Err(format!("Roblox session verification unavailable (HTTP {status}).")),
    }
    let rotated = rotated_cookie(response.headers());
    let user = response
        .json::<RobloxUser>()
        .await
        .map_err(|_| "Roblox sent an unexpected account response.".to_owned())?;
    if user.id == 0 || user.name.len() > 100 || user.display_name.len() > 100 {
        return Err("Roblox returned invalid account information.".into());
    }
    Ok((ConnectedAccount {
        id: user.id, name: user.name, display_name: user.display_name
    }, rotated))
}

#[tauri::command]
pub fn begin_roblox_consent(state: State<'_, AuthState>) -> Result<(), String> {
    *state.started.lock().map_err(|_| "Cannot start login consent.".to_owned())? =
        Some(Instant::now());
    Ok(())
}

#[tauri::command]
pub async fn open_roblox_login(
    app: AppHandle,
    state: State<'_, AuthState>,
    acknowledged: bool,
) -> Result<(), String> {
    if !acknowledged {
        return Err("Read and acknowledge the Roblox cookie warning before proceeding.".into());
    }
    let accepted = {
        let mut started = state.started.lock().map_err(|_| "Consent state unavailable.".to_owned())?;
        let approved = started.as_ref().is_some_and(|time| time.elapsed() >= Duration::from_secs(CONSENT_SECS));
        // Every consent is single-use: an earlier approval cannot silently
        // authorize a future cookie capture without another warning.
        if approved {
            *started = None;
        }
        approved
    };
    if !accepted {
        return Err("A fresh 15-second consent review is required before Roblox sign-in.".into());
    }
    #[cfg(target_os = "macos")]
    {
        // Use AppleBlox-style native WKWebView on macOS instead of Tauri's
        // in-app webview, which is not rendering Roblox login reliably.
        let _ = app;
        return start_native_helper(&state);
    }

    #[cfg(not(target_os = "macos"))]
    {
    if let Some(existing) = app.get_webview_window("roblox-login") {
        existing.set_focus().map_err(|_| "Could not focus Roblox sign-in.".to_owned())?;
        return Ok(());
    }

    let url = Url::parse("https://www.roblox.com/login")
        .map_err(|_| "The official Roblox login URL is invalid.".to_owned())?;
    WebviewWindowBuilder::new(&app, "roblox-login", WebviewUrl::External(url))
        .title("Official Roblox Login — Thirst (isolated)")
        .inner_size(980.0, 760.0)
        .center()
        .incognito(true)
        .devtools(false)
        .on_navigation(roblox_login_host)
        .on_new_window(|_, _| NewWindowResponse::Deny)
        .build()
        .map_err(|_| "Could not open isolated Roblox login. Check that WebView is available.".to_owned())?;
    Ok(())
    }
}

/// Only the state string crosses IPC; the cookie stays in Keychain.
/// Used exclusively for a brief, user-initiated macOS login attempt.
#[tauri::command]
pub fn roblox_login_status(state: State<'_, AuthState>) -> Result<&'static str, String> {
    #[cfg(target_os = "macos")]
    {
        let mut lock = state.helper.lock()
            .map_err(|_| "Could not inspect the native login helper.".to_owned())?;
        let Some(child) = lock.as_mut() else { return Ok("not-started") };
        match child.try_wait().map_err(|_| "Could not inspect native login completion.".to_owned())? {
            None => Ok("waiting"),
            Some(_) => {
                drop(lock);
                if read_pending_session()?.is_some() {
                    Ok("ready")
                } else {
                    Ok("cancelled")
                }
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = state;
        Ok("manual")
    }
}

/// Retry an incomplete embedded login without relaxing navigation restrictions.
/// Does not access, transmit, or disclose any session cookie.
#[tauri::command]
pub async fn reload_roblox_login(app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let _ = app;
        return Err("Use the Reload button in the native Roblox login window on macOS.".into());
    }
    #[cfg(not(target_os = "macos"))]
    {
    let webview = app.get_webview_window("roblox-login")
        .ok_or("The isolated sign-in window is closed. Cancel and reconnect to reopen it.")?;
    webview.reload()
        .map_err(|_| "Could not reload the official Roblox sign-in page.".to_owned())
    }
}

#[tauri::command]
pub async fn finish_roblox_login(app: AppHandle) -> Result<ConnectedAccount, String> {
    #[cfg(target_os = "macos")]
    {
        let _ = app;
        let token = read_pending_session()?.ok_or(
            "Native Roblox sign-in is not complete yet. Finish in its window; it will close automatically."
        )?;
        if token.len() > 8192 || token.len() < 50 {
            return Err("The pending Roblox session is invalid.".into());
        }
        let (account, rotation) = verify_cookie(&token).await?;
        save_session(rotation.as_deref().unwrap_or(&token))?;
        delete_pending_session()?;
        return Ok(account);
    }

    #[cfg(not(target_os = "macos"))]
    {
    let webview = app.get_webview_window("roblox-login")
        .ok_or("Open the official Roblox login window first.")?;
    // This native API retrieves HttpOnly cookies; NO IPC invocation returns them.
    // The async Tauri command avoids a synchronous WebView2 cookie deadlock.
    let cookies = webview
        .cookies_for_url(Url::parse("https://www.roblox.com").expect("constant URL"))
        .map_err(|_| "Could not read the official login window's session.".to_owned())?;
    let token = cookies.into_iter()
        .find(|cookie| cookie.name() == ".ROBLOSECURITY")
        .map(|cookie| cookie.value().to_owned())
        .ok_or("No Roblox session yet. Finish signing in on the official Roblox page.")?;
    if token.is_empty() || token.len() > 8192 {
        return Err("Roblox supplied an invalid session.".into());
    }

    let (account, rotation) = verify_cookie(&token).await?;
    save_session(rotation.as_deref().unwrap_or(&token))?;
    // Only this explicit action stores the session. The incognito browser's
    // own cookies are discarded when closed (and are cleared here as well).
    let _ = webview.clear_all_browsing_data();
    let _ = webview.close();
    Ok(account)
    }
}

#[tauri::command]
pub async fn connected_roblox_account() -> Result<Option<ConnectedAccount>, String> {
    let Some(secret) = read_session()? else { return Ok(None) };
    let (account, rotated) = verify_cookie(&secret).await?;
    if let Some(next) = rotated {
        save_session(&next)?;
    }
    Ok(Some(account))
}

#[tauri::command]
pub async fn disconnect_roblox_account(app: AppHandle, state: State<'_, AuthState>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        stop_native_helper(&state)?;
        delete_pending_session()?;
    }
    delete_session()?;
    if let Some(login) = app.get_webview_window("roblox-login") {
        let _ = login.clear_all_browsing_data();
        let _ = login.close();
    }
    // This only clears Thirst's local credential. To invalidate it on the
    // Roblox servers, users must revoke sessions through Roblox settings.
    Ok(())
}

#[tauri::command]
pub async fn cancel_roblox_login(app: AppHandle, state: State<'_, AuthState>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        stop_native_helper(&state)?;
        delete_pending_session()?;
    }
    if let Some(login) = app.get_webview_window("roblox-login") {
        let _ = login.clear_all_browsing_data();
        let _ = login.close();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_https_roblox_is_allowed_in_login_navigation() {
        assert!(roblox_login_host(&Url::parse("https://www.roblox.com/login").unwrap()));
        assert!(roblox_login_host(&Url::parse("https://auth.roblox.com/v2").unwrap()));
        for bad in [
            "https://roblox.com.evil.test/login",
            "https://roblox-com.example/login",
            "http://www.roblox.com/login",
            "https://www.roblox.com@evil.test/login",
            "javascript:alert(1)",
            "https://roblоx.com/login",
        ] {
            assert!(!roblox_login_host(&Url::parse(bad).unwrap()), "{bad}");
        }
    }

    #[test]
    fn consent_is_at_least_fifteen_seconds() {
        assert_eq!(CONSENT_SECS, 15);
    }

    #[test]
    fn only_explicit_cookie_header_is_rotated() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(SET_COOKIE, "unrelated=ignored; Secure".parse().unwrap());
        assert_eq!(rotated_cookie(&headers), None);
        headers.append(SET_COOKIE, ".ROBLOSECURITY=changed; Secure; HttpOnly".parse().unwrap());
        assert_eq!(rotated_cookie(&headers), Some("changed".into()));
    }
}
