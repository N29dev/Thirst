# Thirst security model (alpha)

The current developer preview only uses Roblox deep links, **unauthenticated public server listings, game metadata, game icons and subplace discovery**.

## Current boundaries

- JavaScript cannot run shell commands or access Roblox cookies; Rust exposes only three narrowly defined commands.
- The native launch command accepts a numeric place ID and optional UUID server ID. It builds a fixed `roblox://experiences/start` deep link; it never interpolates arguments into shell commands.
- Public server calls use `https://games.roblox.com`; on-demand game details and universe discovery use `https://apis.roblox.com`, `https://games.roblox.com` and `https://thumbnails.roblox.com`; subplace discovery uses `https://develop.roblox.com`. All request hostnames are fixed in native Rust, with HTTPS, redirects disabled and a nine-second timeout.\n- Icon URLs returned to the webview are checked against the Roblox CDN host suffix (`rbxcdn.com`); image loading is restricted by CSP.
- Bookmarks and Lite Mode use localStorage, **exclusively for non-sensitive data**.
- No Thirst server, analytics, plugin injection, background polling, or hidden browser automation is included.
- CSP prevents remote frames or scripts; Tauri window permissions are minimal.

## AppleBlox-style login: planned but not implemented

A browser session cookie is powerful enough to compromise an account if stolen. We will not ship cookie capture until we can satisfy these requirements:

1. Open only Roblox-owned HTTPS login pages within an appropriately isolated login flow. Do not ask the user to paste cookies, passwords, or 2FA codes into our UI.
2. Check effective origin and redirects, prevent cross-origin cookie exposure, and explicitly isolate the authentication browser profile.
3. Handle all session tokens in native code. The Svelte webview and plugins must never receive raw cookie bytes or a generic authenticated network proxy.
4. Store credentials in macOS Keychain or Windows Credential Manager with OS access controls, not localStorage, SQLite, text files or logs.
5. Restrict cookie-bearing requests to an audited allowlist of Roblox endpoints; disable cross-host redirects and use safe HTTP timeouts.
6. Redact all sensitive headers, tickets and cookies from logs, crash reports, errors and diagnostics.
7. Provide local removal and explain that revoking the Roblox session itself may need separate action on Roblox.
8. Never transmit cookies to Thirst servers, RoValra, Cloudflare, Discord, analytics, or extensions.
9. Audit macOS and Windows implementations separately and run a public security review before beta.
10. Follow Roblox's applicable API terms; cookie authentication does not bypass API limits.

## Security reports

Do not include live credentials or exploitable details in public GitHub issues. Contact the maintainer privately until private vulnerability reporting is enabled.

## Current state

| Area | State |
| --- | --- |
| Local bookmarks | Available |
| Public server calls | Available |\n| On-demand public metadata, icons and subplaces | Available, subject to Roblox's API limits |
| Validated native Roblox deep links | Available |
| Cookie access or session extraction | Not implemented |
| macOS Keychain and Windows Credential Manager integration | Not implemented |
| Account-specific features | Not implemented |
