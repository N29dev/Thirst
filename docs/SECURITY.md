# Thirst security — experimental session authentication

> **Important:** Thirst's new Roblox connection is an **experimental full-session authentication feature**. It has not been independently security-audited or validated against all Roblox account/login flows. Test with a separate, low-value account. Never upload your `.ROBLOSECURITY` cookie to GitHub, Discord, issue trackers, logs, or screenshots.

## Exact session risk

After the user's explicit consent, Thirst retrieves the `.ROBLOSECURITY` cookie from **its own isolated official Roblox sign-in webview**. This is a bearer credential that can provide broad account access, potentially bypassing 2-step verification if stolen. A 15-second warning and checkbox enforce informed consent; **they do not themselves protect the cookie**.

Roblox warns users not to disclose their cookies and may change cookie formats/rotation. A third-party launcher cannot promise zero risk.

## Implemented security boundaries

1. Consent starts only when the user clicks Connect. An independent Rust `Instant` timer rejects requests to open the login window before **15 seconds**. The UI also requires an acknowledgement checkbox.
2. On macOS, a **separate Swift/WKWebView process** opens the fixed `https://www.roblox.com/login` URL using a **non-persistent** web data store and default system browser user agent; on Windows, a Tauri incognito webview is used. Main-frame navigation is restricted to HTTPS Roblox domains. On macOS, CAPTCHA service domains are allowed only as third-party frames, which may receive their own data under their providers' policies. No Roblox session cookie is explicitly sent to CAPTCHA hosts by Thirst.
3. The login webview is **not included in Tauri's main-window capability**. Roblox websites cannot invoke the launcher commands. The application never asks users to paste credentials or cookies into Svelte.
4. On macOS, a Swift helper checks its own HttpOnly, Secure Roblox cookie store on a timer and writes the cookie directly to a **temporary** macOS Keychain entry (`pending-login`). Rust's `finish_roblox_login` reads and validates this pending entry, then promotes it to the primary keychain entry; the frontend polls the native helper's **exit status rather than the Keychain** and automatically begins verification when ready; the helper never passes the cookie in stdout, arguments or JS. On Windows, native Rust reads the cookie from the isolated Tauri webview. Neither flow logs or returns the cookie through frontend IPC.
5. Native account verification uses **only** `https://users.roblox.com/v1/users/authenticated`, with HTTPS, a fixed host, disabled redirects, and a timeout. An additional **off-by-default, per-run, opt-in** avatar experiment can use the existing Keychain cookie in native Rust for the fixed `https://games.roblox.com/v1/games/{placeId}/servers/Public` endpoint and `https://thumbnails.roblox.com/v1/batch` only. The frontend cannot read the secret. These extra requests do not occur unless the user enables the specific Settings switch. Roblox may rotate cookies in responses from these reviewed hosts.
6. The credential is stored using macOS Keychain / Windows Credential Manager through the `keyring` library. There are no cookie fields in JavaScript, localStorage, SQLite, or config files.
7. Signed-in account information returned to Svelte is limited to Roblox ID, username and display name.
8. Disconnect removes the locally stored credential. On macOS, cancellation terminates the Swift helper and removes its pending keychain credential; the non-persistent browser session is discarded when the helper terminates. Removing the local credential **does not revoke Roblox's server-side session**. Use Roblox Settings → Security to revoke sessions.
9. The app has no remote Thirst backend, third-party telemetry or plugin API. Session-bearing requests never go to RoValra, Cloudflare, Discord, or Roblox CDN hosts. Unauthenticated region requests contain public Roblox place/server IDs only, with a transparent default-on toggle and remembered opt-out. For known BL:R experiences, native Rust first checks the Glutony Worker at a fixed HTTPS URL, which may query RoValra and share cached sanitized results with the website/Discord bot; unmatched/non-BL:R servers may be queried directly from RoValra. No request is made at startup; requesting a public server list triggers a region lookup when enabled. By default, public server and thumbnail requests are unauthenticated. With the separate explicit per-run switch, sessions may be sent to Roblox's fixed public-server and thumbnail API hosts only.
10. Sensitive request headers/cookies and errors are never logged in the new auth module.

## Known limitations and outstanding work

- **Not independently audited.** Browser isolation, macOS and Windows keychain access and cookie rotation still require user-level testing and a security review.
- Login redirects, CAPTCHA providers, popups, changing Roblox login behavior and platform WebView updates can prevent some logins; do not relax the origin allowlist without a reviewed justification.
- Local malicious software or a compromised application/update process may still steal session credentials. Keychain encryption is not a guarantee against compromise.
- Roblox's cookie-based legacy APIs may be incompatible or restricted; this feature is not a rate-limit bypass and does not automatically grant access to unsupported endpoints.
- The Swift helper and Rust launcher are two separate executable identities. macOS may show multiple Keychain permission dialogs, especially after recompiling unsigned developer builds. Thirst minimizes redundant reads, but this does not eliminate every approval. Do not grant blanket access or weaken Keychain permissions.
- Production builds require code signing, distribution/update integrity, a secure release pipeline, and external security review. The Swift helper currently works only from the Cargo build output for **developer builds** and is not yet bundled into release installers.
- Do not use a valuable account during these alpha tests.

## Other application boundaries

- Native launch builds `roblox://experiences/start` from validated numeric place IDs and optional UUID server IDs; user input cannot become arbitrary shell commands.
- On-demand Roblox public requests use fixed HTTPS hosts with redirects disabled.
- Server avatars are loaded automatically for up to eight loaded servers when a connected Roblox account has been verified and the feature is enabled (default), or on demand for guests. Auto-loading can be disabled and is off in Lite Mode. The public `playerTokens` array is excluded from Tauri IPC serialization and held for at most five minutes in bounded native RAM. Only up to five validated Roblox CDN image URLs per server are returned to Svelte. A native Rust unit test checks that tokens cannot be serialized over IPC. The single bounded batch uses only the fixed `https://thumbnails.roblox.com/v1/batch` host. It sends **no Roblox session cookie by default**; the separate explicit native-only session switch permits it. Roblox's own documentation does not require cookies for this thumbnail endpoint, so an authenticated request may not improve unavailable images. The app does not look up or infer players' identities.
- Game icon URLs are accepted only from Roblox's CDN suffix `rbxcdn.com` and constrained by Content Security Policy.
- Saved games, Lite Mode and manual region reports use localStorage exclusively for nonsensitive metadata. Region reports contain only a public place ID, public server UUID, broad region code, fixed `local-report` source and time. Invalid/expired reports are discarded on read and expire after two hours. Manual reports are never uploaded, and no client IP geolocation is performed.
- **Optional RoValra region provider:** when the user manually refreshes public servers with RoValra lookup enabled (the default; disabled persistently by user choice), native Rust sends a public numeric place ID and up to 25 public server UUIDs to fixed HTTPS hosts. For known BL:R modes it checks `https://glutony.scp343hey.workers.dev/api/servers/regions/lookup` first; the Worker retrieves each requested BL:R server UUID from short-lived D1 or, for a cache miss, makes a quota-limited RoValra request and writes the sanitized response to D1 even when Glutony has not sampled that instance. Other/missing servers can use the fixed `https://apis.rovalra.com/v1/servers/details` URL. Neither request contains credentials, cookies, bot tokens or Roblox account IDs. Redirects are disabled, response size is bounded, Thirst holds results only in frontend memory and clearly attributes them. Glutony retains sanitized RoValra results in D1 for up to three minutes to serve its website and Discord bot, after applying D1 migrations 0007 (cache) and 0008 (quota). The Worker HMAC-pseudonymizes its public request source address for per-caller quota enforcement; raw client IPs are not stored in D1. The provider may see the request's network source IP as with any HTTPS request. The enabled/disabled preference is stored locally, defaulting to enabled. No RoValra requests are made solely on launching Thirst, and the provider never receives Roblox cookies. Region responses may be stale or inaccurate; this feature is not a verified location service.

## How to test safely

1. Install the app from code you reviewed and keep the machine free of untrusted software.
2. Use a **dedicated test Roblox account**. Confirm the modal starts at 15 seconds and the button remains disabled until both the countdown and explicit checkbox are satisfied.
3. Confirm the sign-in page really is the official Roblox website. On macOS its separate native window should close automatically after detecting a session; then click Finish connection in Thirst. On Windows complete login in the isolated Tauri window first.
4. Confirm only the username, display name and ID appear in the application; no token or cookie appears in the UI, browser DevTools output, logs, localStorage or crash information.
5. Test local disconnect. Afterwards, manually revoke the session from the Roblox account's security settings.

## Security reports

Never share real cookies or secrets in public issues. Contact the maintainer privately until GitHub private vulnerability reporting is available.
