# Thirst roadmap

This document tracks features requested during the first successful macOS alpha test.

## Alpha 0.1 — Foundation

- [x] Local favorites by experience and place ID.
- [x] Public server listing and specific-server launch.
- [x] Lightweight UI with Lite Mode.
- [x] Desktop notification auto-dismiss (3.5s success, 8s error, resets when another toast appears).
- [ ] Test installed-Roblox discovery, missing-handler errors, server paging under throttling, and persistence on Windows.
- [ ] Confirm stable signed macOS/Windows installers.

## Alpha 0.2 — Polished library and navigation

- [ ] Compact favorite-game icon strip / quick-launch grid on Overview.
- [ ] Retrieve official game icons and names using public Roblox metadata endpoints; cache locally and lazy-load.
- [ ] Automatically discover available subplaces where Roblox exposes them; manual IDs remain available as a fallback.
- [ ] Soft pointer-following light/hover effect on actionable cards, using CSS-only or requestAnimationFrame and honoring reduced-motion and Lite Mode settings.
- [ ] Sort saved games, recent games, pinned subplaces and recent server joins.
- [ ] Toast notifications have accessible dismissal and auto-hide; verify no overlap with game launch.

## Alpha 0.3 — Account and consent

- [ ] AppleBlox-style **official Roblox browser login**, in an isolated native context.
- [ ] macOS Keychain and Windows Credential Manager handling in native Rust/OS code only.
- [ ] Strict allowlist for authenticated Roblox endpoints, no third-party transfers, no cookie injection into the Svelte frontend, no plugins holding raw secrets.
- [ ] Account-level features behind an explicit opt-in, with sign-out/revocation controls.
- [ ] Security threat model, tests, and independent review prior to public beta.

## Alpha 0.4 — Advanced server tools

- [ ] Region/provider architecture. Default to **Unknown** unless data is verified; don't confuse Roblox-reported server ping with the local user's latency.
- [ ] Investigate allowed RoValra integration or licensed provider access; don't scrape or redistribute service data without permission.
- [ ] Optional initial ~5 player avatar thumbnails **when** a public Roblox token/thumbnail API supports it. This should not attempt to identify players from tokens.
- [ ] Opt-in friends-in-server indicator, **only when** authorized presence data is accessible and players' Roblox visibility settings permit it.
- [ ] Filters for region (with source/confidence), free slots, ping, FPS, version, and recent/favorite servers.
- [ ] Use caching, pagination, throttling and timeouts to prevent unnecessary rate limiting.

## Alpha 0.5+ — Wider feature parity

- [ ] Supported FastFlags and graphics presets.
- [ ] Discord Rich Presence, themes, installation and update management.
- [ ] Multi-instance and multi-account options only when stable and compliant.
- [ ] Plugin API with narrowly scoped permissions and no credential access.
- [ ] Cross-platform packaging, startup/idle CPU+RAM benchmarks, and signed releases.

## Design principles

1. Never make gameplay slower by leaving expensive launcher tasks running in the background.
2. The UI must remain useful without signing in.
3. Build features around documented, authorized access; do not bypass Roblox account/privacy restrictions.
4. Keep session credentials on-device, isolated from UI JavaScript and third parties.
5. Never display guessed server regions or inferred user identities as facts.
