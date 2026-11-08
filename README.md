# Thirst 💧

**Stay thirsty for more.** A lightweight, visually polished, local-first Roblox launcher for macOS and Windows.

Built independently with **Tauri 2 + Rust + Svelte + TypeScript**. Inspired by ideas from AppleBlox and Lunar, but not affiliated with Roblox, AppleBlox, or Lunar Client.

> Alpha 0.3.0 — **experimental developer preview, not a production installer**. An optional full-session Roblox login is now available for testing, **without an independent security audit**. **Never paste your Roblox session cookie into this app, an issue, or Discord.**

## What works now

- Dark desktop interface with a low-overhead Lite Mode.
- Save favorite Roblox experiences using place IDs or official game URLs. Thirst retrieves their public names and icons on demand, with manual-name fallback.
- Save and launch individual subplaces under an experience, or discover publicly listed places on demand (with pagination).
- Quickly launch your 12 first saved games from the Overview icon grid; your game cover is shown where available.
- Fetch public Roblox servers on demand with player counts, Roblox-reported ping and FPS, pagination, and instance-specific launch links.
- Launch Roblox using its registered desktop deep-link protocol on macOS or Windows.
- Keep bookmarks and preferences locally. No Thirst account, telemetry, or background polling.
- Optional **AppleBlox-style** native Roblox session login: 15-second non-bypassable Rust consent gate, separate incognito Roblox webview, local macOS Keychain / Windows Credential Manager credential storage, no cookie in Svelte, verification with Roblox.

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

Native bundling is currently disabled in `src-tauri/tauri.conf.json`. Signed macOS / Windows installers will be part of a later milestone.

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

## Security

See [docs/SECURITY.md](docs/SECURITY.md) before trying to log in. The experimental native login **does read a Roblox session cookie** from its isolated official-login webview after explicit consent. A stolen cookie can compromise your Roblox account. The credential is stored using OS secure storage but no implementation can guarantee it cannot be stolen. Use a dedicated test account, do not publish or paste any cookie, and revoke testing sessions on Roblox afterwards.

Roblox's public experience and subplace endpoints can fail, be restricted, or return incomplete pages; manual bookmarks still work. Icons are loaded only from Roblox CDN hosts. There is no background game-metadata polling. The Roblox public server API may be unavailable, rate-limited, or return incomplete pages. Displayed ping is Roblox's server metric, not an estimate of the user's own latency. Roblox must be installed and registered to handle deep links.

## Structure

```text
src/                   Svelte UI + local-only library
src/lib/backend.ts     Typed native calls
src-tauri/src/lib.rs   Rust-side validation, launch, public server API
src-tauri/src/auth.rs  Native Roblox session sign-in and OS secure storage
src-tauri/capabilities Minimal Tauri permissions
docs/SECURITY.md       Security policy and login requirements
```

## License

MIT — see [LICENSE](LICENSE).
