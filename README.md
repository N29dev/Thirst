# Thirst 💧

**Stay thirsty for more.** A lightweight, visually polished, local-first Roblox launcher for macOS and Windows.

Built independently with **Tauri 2 + Rust + Svelte + TypeScript**. Inspired by ideas from AppleBlox and Lunar, but not affiliated with Roblox, AppleBlox, or Lunar Client.

> Alpha 0.3.0 — **experimental developer preview, not a production installer**. An optional full-session Roblox login is now available for testing, **without an independent security audit**. **Never paste your Roblox session cookie into this app, an issue, or Discord.**

## What works now

- Dark desktop interface with a low-overhead Lite Mode and subtle cursor-following lighting on cards; effects pause when idle and are disabled with Lite Mode or reduced motion.
- Save favorite Roblox experiences using place IDs or official game URLs. Thirst retrieves their public names and icons on demand, with manual-name fallback.
- Save and launch individual subplaces under an experience, or discover publicly listed places on demand (with pagination).
- Quickly launch saved experiences **and individually pinned subplaces** directly from the Overview grid (up to 24 tiles). Use the star next to a saved subplace to pin/unpin it; the tile's server button goes straight to that subplace's public servers.
- Fetch public Roblox servers on demand with player counts, Roblox-reported ping and FPS, pagination, and instance-specific launch links.
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

Native bundling is currently disabled in `src-tauri/tauri.conf.json`. The macOS Swift helper is compiled by `swiftc` during a **developer build** and is loaded from Cargo's output directory; packaged apps still need signed sidecar distribution. Xcode Command Line Tools are required on macOS. Signed macOS / Windows installers will be part of a later milestone.

## Roadmap

| Capability | State |
| --- | --- |
| App shell, UI, Lite Mode | Implemented |
| Quick-launch icons, on-demand game details, public subplace discovery | Implemented |
| Favorite games and subplaces | Implemented locally |
| Public server listing + direct-server launch | Implemented, subject to Roblox availability |
| AppleBlox-style browser login + OS secure storage | **Experimental, requires real-device security testing** |
| FastFlags and Roblox installation management | Planned |
| Rich Presence, themes, updates | Planned |
| RoValra provider with explicit authorization | Research |
| Multi-instance and multi-account support | Research |
| Signed, tested macOS and Windows releases | Planned |

## macOS Keychain permission prompts

In development, the native Swift login helper and the Rust launcher are separate executables with separate Keychain access permissions. macOS may ask more than once when the helper saves a temporary credential and Thirst verifies and stores it.

Thirst minimizes unnecessary approvals by tracking native login completion through the helper's exit code **without reading Keychain repeatedly**, loading account status once per app session (unless you press **Check status**), and skipping writes when a cookie has not changed. Developer builds may still prompt again after recompilation. Properly signed, stable-identity releases are the long-term solution.

**Never select "Allow all applications to access this item," disable Keychain protections, or change your keychain to remove password prompts.** Check that each prompt names the expected Thirst executable; cancel unfamiliar permission requests. The explicit **Check status** action re-verifies the account and may require Keychain permission.

## Security

See [docs/SECURITY.md](docs/SECURITY.md) before trying to log in. The experimental native login **does read a Roblox session cookie** from its isolated official-login webview after explicit consent. A stolen cookie can compromise your Roblox account. The credential is stored using OS secure storage but no implementation can guarantee it cannot be stolen. Use a dedicated test account, do not publish or paste any cookie, and revoke testing sessions on Roblox afterwards.

Roblox's public experience and subplace endpoints can fail, be restricted, or return incomplete pages; manual bookmarks still work. Icons are loaded only from Roblox CDN hosts. There is no background game-metadata polling. The Roblox public server API may be unavailable, rate-limited, or return incomplete pages. Displayed ping is Roblox's server metric, not an estimate of the user's own latency. Roblox must be installed and registered to handle deep links.

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
