//! Native-only Roblox session integration for Thirst Alpha 0.3.
//! A Roblox session cookie is effectively a password. Never expose it to
//! Svelte, the Tauri IPC return value, logs, extensions, or third-party hosts.

use std::sync::Mutex;
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

#[derive(Default)]
pub struct AuthState {
    started: Mutex<Option<Instant>>,
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
    for header in headers.get_all(SET_COOKIE) {
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
        let started = state.started.lock().map_err(|_| "Consent state unavailable.".to_owned())?;
        started.as_ref().is_some_and(|time| time.elapsed() >= Duration::from_secs(CONSENT_SECS))
    };
    if !accepted {
        return Err("The 15-second consent review has not finished.".into());
    }
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

#[tauri::command]
pub async fn finish_roblox_login(app: AppHandle) -> Result<ConnectedAccount, String> {
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
pub async fn disconnect_roblox_account(app: AppHandle) -> Result<(), String> {
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
pub async fn cancel_roblox_login(app: AppHandle) -> Result<(), String> {
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
