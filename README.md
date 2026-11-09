# Thirst 💧

**Stay thirsty for more.** A lightweight, visually polished, local-first Roblox launcher for macOS and Windows.

Built independently with **Tauri 2 + Rust + Svelte + TypeScript**. Inspired by ideas from AppleBlox and Lunar, but not affiliated with Roblox, AppleBlox, or Lunar Client.

> Alpha 0.4.0 — **developer preview, not a signed production installer**. An optional full-session Roblox login is now available for testing, **without an independent security audit**. **Never paste your Roblox session cookie into this app, an issue, or Discord.**

## What works now

- **Branded app icon:** a custom neon droplet source at `src-tauri/icons/thirst.svg`, automatically converted to a native macOS `.icns`, Windows `.ico`, and retina PNG variants before dev/build. The same artwork appears in the sidebar. The bundle workflow verifies that the macOS app actually includes its icon. New builds must replace older installed apps; the Dock may cache the old placeholder.
- **Refined desktop interface:** matching SVG navigation icons and clear active-page indicators, short screen-entry and card-reveal transitions, polished hover/press feedback, more readable panel hierarchy, responsive server-loading skeletons, and visible keyboard focus. No continuously animated wallpaper or network fonts. **Lite Mode and reduced-motion preferences disable decorative motion**; performance remains the priority. See [Visual design notes](docs/VISUALS.md).
- Save favorite Roblox experiences using place IDs or official game URLs. Thirst retrieves their public names and icons on demand, with manual-name fallback.
- Sort My Library by saved order, name A–Z, newest or oldest. **Import/export library backups** as JSON; imports merge saved games and pinned subplaces. Local-only, no cookies or Roblox credentials.
- **Recent launches:** quick reopen of Roblox places previously opened through Thirst; these are launch requests, not proof of an actual join.
- **Automatic Thirst update checks (not auto-install):** after startup Thirst checks the latest stable GitHub Release once, with a Settings opt-out and a manual Check now button. It compares semantic versions and opens the official releases page for installation. This does **not** download or execute developer builds. Currently no official releases have been published. Signed in-app installation requires a separately configured trusted update channel; see [Update guide](docs/UPDATES.md).
- **Game history and rejoin (experimental):** opt-in local Roblox Player log parser (up to 12 recent log tails) distinguishes join attempts, network connections and disconnects without exposing server IPs, logs or cookies to Svelte. Keeps up to 80 events for 30 days. A visible Thirst window scans every 30 seconds on Overview or 60 seconds on other tabs, never as a background service after exit. Exact-server rejoin checks at most six public server pages for recent sessions and otherwise opens the game; no server availability is guaranteed. Needs validation with the current Roblox Player on real macOS and Windows devices. See [Activity guide](docs/ACTIVITY.md).
- Save and launch individual subplaces under an experience, or discover publicly listed places on demand (with pagination).
- Launch saved experiences **and individually pinned subplaces** from image-forward Quickplay cards (up to 24). Every tile has a separate server-browser action and an obvious experience/subplace label. Roblox images fade in and show a styled initial fallback if absent or unavailable; existing artwork is never fetched in a background sweep. Star saved subplaces in Library to pin them on Quickplay.
- Fetch public Roblox servers on demand with player counts, Roblox-reported ping, pagination, and instance-specific launch links. Server FPS is deliberately omitted from the browser because it does not predict a player's game FPS.
- Sort and filter **loaded public servers** (player count, free slots, reported ping, and reported regions), with search by job UUID. Copy server IDs from their row. Temporary server IDs are deliberately not favorited; pin games and subplaces instead.
- **Local region notes (experimental):** after checking a server in-game, label it as NA, Europe, Asia and other broad regions. These are explicitly marked *your report*, never verified geolocation. Notes are stored only on your device, tied to the exact place and instance UUID, and expire after two hours. Filter reported regions or Unknown servers. Local reports never transmit data. The temporary RoValra region provider is **enabled by default when you manually refresh the public server list**, and can be disabled in Server Browser. The off preference persists across restarts. For BL:R 8v8, Pro and Main, Thirst first asks the existing Glutony Worker for RoValra-sourced locations already shared with the BL:R website and Discord bot. Glutony checks its short-lived D1 cache for up to 25 public server UUIDs, **including UUIDs absent from its current BL:R sample**. For cache misses, its guarded read-through endpoint queries RoValra and writes sanitized results back to D1 for the website and bot to reuse. The endpoint limits missing lookups to 25 IDs per minute per caller using an HMAC-pseudonymous quota. If Glutony is unavailable or denies a request, Thirst falls back to `apis.rovalra.com`; non-BL:R games still use direct RoValra. Shared writes require Glutony D1 migrations 0007 and 0008. To diagnose a missing region, open [Glutony's live region-cache status](https://glutony.scp343hey.workers.dev/api/servers/regions/status) and confirm `cacheReady`, `quotaReady`, and `quotaSecretReady` are true. The Worker distinguishes cached data from newly persisted results. Unknown server locations can be rechecked with **Look up regions** after one minute without refreshing the entire Roblox server list; an API retry can only discover locations RoValra actually knows. Both routes transmit only public numeric place IDs and server UUIDs. Neither route sends your Roblox cookie, account ID, player tokens or secret API keys. Glutony and RoValra may each observe the source IP of requests made to them. Provider results are attributed, in memory only, unverified by Thirst, and can be disabled instantly. Where a known country is provided, a Unicode country flag is shown beside a compact city-and-country label; the full location remains in the tooltip.
- Users with a verified saved Roblox login see **automatic anonymous avatar thumbnails** for up to the first eight loaded servers (maximum five previews per server) in **one bounded batch** after manual Refresh. Guests can expand one server's avatars on demand. By default, thumbnails use no session cookie. In Settings, connected users may separately enable experimental session-backed server and avatar requests for the current run; native Rust then attaches the saved cookie only to fixed `games.roblox.com` and `thumbnails.roblox.com` endpoints. This optional mode may prompt Keychain and cannot guarantee avatar results. Raw player tokens remain in native Rust, cached at most five minutes, and are never sent to UI scripts. Auto-previews can be disabled in Settings and are off in Lite Mode.
- **Fast Flags workspace:** polished editor with search, graphics profiles, drag-and-drop and pasted JSON imports, named-preset compatibility (`name`/`flags`/`enabled`), visible disabled entries, merge/replace choice, and two export formats (shareable named preset or native `ClientAppSettings.json`). Only enabled, allowlisted Player flags can be saved; imports never apply automatically. See [Fast Flags guide](docs/FASTFLAGS.md).
- Launch Roblox using its registered desktop deep-link protocol on macOS or Windows.
- Keep bookmarks and preferences locally. No Thirst account, telemetry, or background polling.
- Optional **AppleBlox-style** Roblox session login: a 15-second Rust consent gate; on macOS, a separate **Swift/WKWebView helper** with non-persistent storage, automatic native cookie detection and a temporary Keychain entry; on Windows, an isolated Tauri webview. Native code verifies the cookie with Roblox before storing the primary credential. On macOS the app automatically finishes once the helper closes successfully; manual Finish remains available as a fallback. No cookie is returned to Svelte.

## Run locally

Requirements: Node.js 22+, Rust stable 1.90+, the platform-specific [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/), and Roblox installed for launch testing.

```bash
git clone https://github.com/N29dev/Thirst.git
cd Thirst
npm install
npm run tauri dev
```

For UI-only preview, run `npm run dev`. Native launch and server features require the Tauri window.

To run checks:

```bash
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

Test bundling is enabled. Run `npm run bundle:mac` on macOS (Apple Silicon `.app` including the Swift/WKWebView login helper) or `npm run bundle:windows` on Windows (NSIS installer). GitHub Actions builds these artifacts through [the test-bundle workflow](https://github.com/N29dev/Thirst/actions/workflows/package.yml). The macOS app and its login helper are now **ad-hoc signed** and verified in CI, but they are **not Developer ID signed or notarized**, so macOS may still require an individual exception in Privacy & Security. Xcode Command Line Tools are required for local builds. Read [the release and Gatekeeper troubleshooting guide](docs/RELEASE.md) before installing.

## Roadmap

| Capability | State |
| --- | --- |
| App shell, responsive UI, Lite Mode and accessible motion | Implemented; real-device visual QA ongoing |
| Quick-launch icons, on-demand game details, public subplace discovery | Implemented |
| Favorite games and subplaces | Implemented locally |
| Public server listing + direct-server launch | Implemented, subject to Roblox availability |
| AppleBlox-style browser login + OS secure storage | **Experimental, requires real-device security testing** |
| Roblox Player Fast Flags editor and graphics presets | Implemented; needs real-device Roblox validation |
| Roblox installer/updater | Not bundled; uses existing official Roblox installation |
| Recent launches and local library backup/restore | Implemented |
| Opt-in Roblox Player log history + conservative rejoin | Initial implementation; needs current macOS/Windows Roblox testing |
| Thirst release checks | Implemented; no published GitHub Releases yet, installer remains manual |
| Automatic signed Thirst installs | Pending Tauri signing keys, signed release artifacts and platform testing |
| Rich Presence and themes | Future roadmap |
| Attributed RoValra regions with flags | Experimental, on by default when manually refreshing servers; remembered opt-out; subject to provider terms |
| Multi-instance and multi-account support | Research |
| Ad-hoc signed macOS ARM64 / unsigned Windows test artifacts | CI packaging implemented; real-device testing required |
| Signed, notarized macOS and Windows releases | Not yet released; certificates and verification required |

## macOS Keychain permission prompts

In development, the native Swift login helper and the Rust launcher are separate executables with separate Keychain access permissions. macOS may ask more than once when the helper saves a temporary credential and Thirst verifies and stores it.

Thirst minimizes unnecessary approvals by tracking native login completion through the helper's exit code, loading account status once per app session, and keeping **one native-only unlocked session in process memory** after the first Keychain access. Manual server refreshes and avatar batches reuse the unlocked value instead of prompting for a Keychain password twice per refresh. The in-memory copy is wrapped in `Zeroizing<String>`, replaced on Roblox cookie rotation, and discarded on disconnect, pressing **Lock session memory** in Settings, or quitting Thirst. A new process may require one Keychain approval. Cookie material never enters the Svelte UI, browser storage, D1 or logs. This avoids repeat prompts, but keeping a credential in RAM for the app's lifetime increases its exposure if the running process is compromised. Developer builds may still prompt after recompilation; properly signed, stable-identity releases are the long-term solution.

**Never select "Allow all applications to access this item," disable Keychain protections, or change your keychain to remove password prompts.** Check that each prompt names the expected Thirst executable; cancel unfamiliar permission requests. The explicit **Check status** action re-verifies the account using the unlocked native session when available. **Lock session memory** clears the in-process copy without deleting the saved credential; the next authenticated request may prompt for Keychain permission.

## Security

See [docs/SECURITY.md](docs/SECURITY.md) before trying to log in. The experimental native login **does read a Roblox session cookie** from its isolated official-login webview after explicit consent. A stolen cookie can compromise your Roblox account. The credential is stored using OS secure storage but no implementation can guarantee it cannot be stolen. Use a dedicated test account, do not publish or paste any cookie, and revoke testing sessions on Roblox afterwards.

Server sorting/filtering applies to fetched pages only, not all active servers. Public server requests use an 8-second app-side cooldown and a 45-second backoff after rate limiting; no background request loops. **Regions show Unknown** until RoValra (directly or through Glutony's shared cache) returns a provider-reported location or you add your own local report. The RoValra option allows immediate manual retries (without a fixed cooldown), keeps one lookup in flight at a time, preserves a 120-second backoff after HTTP 429, and caps each batch at 25 servers. Data may be incomplete or unavailable; it is not independently verified by Thirst. RoValra may block access or request that this integration be withdrawn. See [docs/REGIONS.md](docs/REGIONS.md) and [RoValra terms](https://www.rovalra.com/tou/). Preview avatars have no usernames and only appear when Roblox provides public preview data. Roblox's public experience and subplace endpoints can fail, be restricted, or return incomplete pages; manual bookmarks still work. Icons are loaded only from Roblox CDN hosts. There is no background game-metadata polling. The Roblox public server API may be unavailable, rate-limited, or return incomplete pages. Displayed ping is Roblox's server metric, not an estimate of the user's own latency. Roblox must be installed and registered to handle deep links.

## Structure

```text
src/                   Svelte UI + local-only library
src/lib/backend.ts     Typed native calls
src-tauri/src/lib.rs   Rust-side validation, launch, public server API
src-tauri/src/auth.rs  Native Roblox session verification and Keychain storage
src-tauri/native/macos/ThirstLogin.swift  Independent native macOS login helper
src-tauri/build.rs  Compiles Swift helper for macOS developer builds
src-tauri/capabilities Minimal Tauri permissions
docs/SECURITY.md       Security policy and login requirements
```

## License

MIT — see [LICENSE](LICENSE).

## AppleBlox source and licensing

Thirst's macOS login behavior was informed by AppleBlox's public architecture and documentation. The Swift helper in this repo is an independent implementation; no AppleBlox GPL-3.0-only source code was copied. AppleBlox is GPL-3.0-only and is not affiliated with Thirst.
