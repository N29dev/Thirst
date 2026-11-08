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

- [x] Compact favorite-game icon grid on Overview, with launch and server buttons.
- [x] Retrieve official Roblox game names/icons when saving, store the validated CDN icon URL locally; existing bookmarks support explicit metadata refresh (no background polling).
- [x] On-demand public subplace discovery with paging and a Save action; manual IDs remain available as a fallback.
- [x] Subtle cursor-following lighting on actionable cards, using requestAnimationFrame only while moving the pointer (no idle loops); Lite Mode and reduced-motion respected.
- [x] Unified SVG icons for primary navigation, accessible active-page labels/tooltips, keyed short transitions between screens, restrained card/hero reveal motion, and responsive public-server loading skeletons.
- [x] Replace the macOS blank placeholder and old Windows icon: generate an ICNS/ICO/PNG iconset from the branded Thirst SVG source before bundling; explicitly reference these formats in Tauri configuration and verify their inclusion in CI.
- [ ] Confirm the new icon appears in the installed macOS Dock, Finder and Windows taskbar after replacing an older build (OS caches may need refreshing).
- [ ] Audit keyboard and screen-reader navigation, and visually test at 740–1600px widths on both macOS and Windows.
- [x] Let users **pin individual subplaces as standalone quick-launch tiles** on Overview, alongside experiences (e.g. BLR 8v8 / Pro / Customs), with direct launch and server browser shortcuts. Pins persist locally.
- [x] Sort the My Library experience list by saved order, name A–Z, newest and oldest, with a local-only preference. The Overview shortcut order remains unchanged.
- [x] Recently opened place shortcuts (not short-lived server UUIDs), with clearable local history.
- [x] Portable JSON backup of saved games and pinned subplaces; validated merge that preserves existing bookmarks.
- [x] Move saved experiences and subplaces up/down without altering IDs or pin state; Overview shortcuts follow the resulting saved order.
- [ ] Toast notifications have accessible dismissal and auto-hide; verify no overlap with game launch.

## Alpha 0.3 — Account and consent

- [x] Experimental AppleBlox-style **official Roblox browser login** in an incognito native context, with a native-enforced 15-second consent review.
- [x] macOS Keychain and Windows Credential Manager handling in native Rust/OS code only (implementation; real-device testing ongoing).
- [x] Strict allowlist for the initial authenticated Roblox verification endpoint, no third-party cookie transfers, no cookie in Svelte or plugins.
- [x] Opt-in connected account profile and local sign-out. Roblox server-side session revocation remains a manual Roblox Settings action.
- [ ] Optional macOS **Touch ID / user-presence protection** for saved Roblox sessions using `LocalAuthentication` and `SecAccessControl`, with password fallback. Design and test a safe migration from the existing Keychain entry, keep the secret native-only, and never claim this suppresses macOS's separate application-access consent dialogs.
- [ ] Security threat model, tests, and independent review prior to public beta.

## Alpha 0.4 — Advanced server tools

- [x] First region-data foundation: privacy-preserving local reports by place + server UUID, broad-region filters, explicit *your report* badges, two-hour expiry and no external API calls. Default to **Unknown**; reports are unverified, not geolocation or user latency.
- [x] Temporary **RoValra** API provider enabled on manual server refresh by default, with a remembered opt-out, visible attribution, country flag emoji, compact labels, exact-host native requests, a 25-server batch cap, 30-second cooldown and no Roblox cookie transfer. BL:R modes use Glutony's short-lived read-through region cache (migrations 0007 and 0008 required): on misses, the Worker can query RoValra and update D1 even for servers absent from its own samples. Thirst retains direct fallback if Glutony is down or limited. Results remain in memory. Provider terms must be respected and access can be withdrawn.
- [ ] Seek explicit permission or a longer-term licensed provider for automatic regions. Do not confuse provider-reported regions with user latency or independently verified datacenter positions.
- [ ] Clarify use conditions with RoValra for the temporary direct/shared provider integration, and switch to a licensed data source if necessary. Do not treat proxying through Cloudflare as authorization or a way around the API terms.
- [x] On-demand previews of **up to five anonymous player avatars**; connected users receive automatic, one-batch previews for up to eight servers after manual refresh by default. Thirst stores public tokens only in bounded Rust memory and returns validated CDN image URLs, not user IDs. Auto previews are disableable and off in Lite Mode.
- [x] Experimental **per-run opt-in authenticated requests** for the public server list and avatar thumbnails only: fixed Roblox HTTPS hosts, native-only session, cookie rotation and CSRF retry, with disclosure in Settings. Public guest listings remain enabled and are the default. Roblox does not guarantee these requests improve availability.
- [ ] Expand authenticated features for friends/presence and owned private servers only after reviewing their endpoints and privacy permissions.
- [ ] Opt-in friends-in-server indicator, **only when** authorized presence data is accessible and players' Roblox visibility settings permit it.
- [x] Filter/sort loaded servers by free slots, player counts, reported ping, local reported regions/Unknown, and job-ID search. Copy a full server UUID by clicking it. No saved server UUIDs: Roblox instances are short-lived; only game/subplace favorites persist.
- [x] Separately attributed experimental region lookup from RoValra, enabled by default upon manual server refresh with local opt-out; no cookie forwarding or scraping. The provider may disallow access. Regions remain Unknown when neither a provider nor a local report returns a value.
- [ ] Confirm long-term provider permission and investigate server version filtering.
- [x] Basic manual-request cooldown (8 seconds) and extended 429 backoff (45 seconds), bounded 5-minute anonymous-avatar token cache, pagination, and native request timeouts.
- [ ] Further investigate API response caching, server discovery completeness, and long-running rate-limit behavior before public release.

## Alpha 0.5+ — Wider feature parity

- [x] Roblox Player FFlags loader/editor: published 18-key allowlist, native ClientAppSettings read/write, graphics presets, JSON import/export and first-write backup. Real-device validation remains necessary as Roblox can change behavior.
- [ ] Optional Discord Rich Presence if local game activity can be confirmed. Avoid a duplicate Roblox installer/updater, plugins, or themes without a real user need.
- [ ] Multi-instance and multi-account options only when stable and compliant.
- [ ] Plugin API with narrowly scoped permissions and no credential access.
- [x] Automated test-app packaging for macOS ARM64 and Windows; macOS app and native login helper are ad-hoc signed and signature-checked, but **not Developer ID signed or notarized**.
- [ ] Real-device smoke tests, startup/idle CPU+RAM benchmarks, verified code signing and notarized public releases.

## AppleBlox 0.9.0 feature study — Nov 8, 2026

Reference: [AppleBlox source](https://github.com/AppleBlox/appleblox), [0.9.0 changelog](https://github.com/AppleBlox/appleblox/blob/main/CHANGELOG_0.9.0.md), and the user's screenshots of Quickplay, Account, Integrations, Behavior, Engine, Mods, Appearance and Misc. These items are **ideas for independent implementation, not a port or a promise that all AppleBlox features work on current Roblox**. AppleBlox is GPL-3.0; Thirst is MIT. Do not copy its source, artwork, assets or bundled helpers into Thirst without resolving licensing obligations.

### Already covered in Thirst (do not build twice)

- [x] Basic Quickplay: saved experience/subplace tiles with verified Roblox icons and manual launches. Recent **launcher-initiated place** shortcuts already exist; these are **not** automatic play history.
- [x] Local library, ordering, pinned subplaces, portable library backups, and on-demand public server pages.
- [x] Native login to an official Roblox page with one connected account, Keychain/Credential Manager storage and explicit consent. **Not** multi-account management or automatic Roblox-cookie extraction.
- [x] Basic Fast Flags presets, search, allowlisted editing, named JSON import/export, disabled-flag handling, and first-overwrite backup. This does **not** make unsupported old FFlags work.
- [x] Manually refreshed, attributed region data via RoValra/Glutony and local region reports; server-reported ping is **not** a user's measured latency.
- [x] Desktop notifications and a local-first Lite Mode. Test-bundle workflow exists; public signing/notarization remains separate.

### P0 — Next: high-value quality of life, little runtime overhead

**Initial History/Rejoin implementation (Nov 8, 2026):**
- [x] Opt-in native Player-log join detection from fixed macOS/Windows log paths. Bounded to 12 recent logs × 512 KiB; raw logs, cookies, addresses and other private content never returned to the UI.
- [x] Local 30-day / 80-event history with distinct log attempts vs manually opened shortcuts, one-click Refresh and on-demand public title/art lookup. Observations are not proof of a completed join.
- [x] First rejoin flow: verify matching UUID with a free slot in at most three public server pages when recent (<15 minutes), otherwise explain and open the place. No fake availability claims or background Roblox requests.
- [ ] Validate on actual Roblox Player versions and capture example logs from both macOS and Windows; adjust patterns, confirm leave timestamps and test complete session/teleport/rejoin flows.
- [ ] Measure idle cost, rate-limit behavior and rejoin reliability; native log reader currently runs only while Overview is open and tracking is enabled.

- [ ] **Real recent-game history.** Add an optional, local-only Roblox session observer that records game identity, joined time, approximate duration and last-seen instance, with a small bounded database. Start disabled until consent; no full-time remote polling or cookie use. Show attractive cover-art Quickplay tiles and a history view; retain the existing manual/pinned library independently. Provide pause, clear, retention limits and fallback when log data is missing. _Done when_: it detects actual sessions not launched through Thirst, remains idle after Roblox exits, and never mislabels manually saved games as recently played.
- [ ] **Rejoin last game / last server.** A prominent Rejoin action should try the most recent *validated live* instance where the official launch flow allows, show when the instance has expired/full/private, and fall back clearly to launching the place. Do not blindly replay an old UUID or claim the exact server still exists. _Done when_: working rejoin and expired-server fallback are tested in macOS and Windows.
- [ ] **Roblox installation detection and repair UX.** Show detected official Roblox.app/Player location, install/version status and registered URL handler. Offer Re-detect, a manually chosen/validated installation path, open installation folder and a link to the **official Roblox installer** when absent. Do not add a second automatic updater or overwrite an installation without a demonstrated need. _Done when_: default and nonstandard installs, missing handlers and an uninstalled Roblox state all display actionable errors on both platforms.
- [ ] **Useful diagnostics.** Add bounded, locally stored structured launcher logs with rigorous credential/cookie/URL-token redaction; a readable Copy Debug Report and Export Debug Bundle with OS/build, failed operations and recent errors; direct Open Logs / Open App Data buttons. Reports must omit secrets and private player data by construction. _Done when_: a failed login, settings write or server request can be investigated without requesting a screenshot of private credentials.
- [ ] **Navigation and first-run polish.** Adopt the *clarity*, not the appearance, of AppleBlox's grouped settings. Keep Overview / Library / Servers / Fast Flags prominent; subdivide Settings into Account, Roblox & Launch, Integrations, Appearance, and Troubleshooting without creating empty pages. Add a lightweight first-run install/account explanation, helpful disabled-state descriptions and keyboard shortcuts only for common, visible actions. _Done when_: all screens work at narrow window sizes and do not add heavy idle animation or needless options.

### P1 — After P0: opt-in integrations and power-user functionality

- [ ] **Discord Rich Presence (minimal).** Optional, off by default. Display the validated current game and optionally elapsed play time while Roblox is running. Stop and clear on exit; allow hiding the game. A join button must remain off unless an authorized and privacy-respecting join target can be established. Avoid Bloxstrap SDK emulation or third-party scripts in v1. _Done when_: Discord absent/closed, opt-out, game exit and long idle sessions are handled without launcher slowdown.
- [ ] **Multiple accounts, native-only.** A deliberate manager for explicitly connected Roblox accounts: avatars/display names, active profile, expired-session warnings, add/switch/disconnect, and separate Keychain/Credential Manager entries. No cookie paste field, automatic scraping of a logged-in Roblox browser, or credentials exposed in Svelte. Require a clear session-switching model and re-test the consent/security boundary. _Done when_: switching does not accidentally launch the prior account, losing one account doesn't erase the other, and macOS Keychain prompts remain understandable.
- [ ] **Preferred region as a *server-finder preference*.** Prioritize candidates from loaded servers using attributable provider reports and the user's choice; retain Automatic by default. Clearly distinguish provider region, unverified local observation and measured latency; do not promise to force Roblox matchmaking into a region. Respect RoValra's conditions and availability. _Done when_: results re-sort correctly without invented low-ping estimates, and Unknown locations remain accessible.
- [ ] **Optional game-session details.** For locally observed sessions, display region *only when legitimately known*, approximate duration, and recent place counts. A region-on-join toast can be toggled off. Never export other players' presence or identifiers as part of history.
- [ ] **Advanced graphics controls within the current allowlist.** Group Fast Flags into readable Graphics / Geometry / Display settings, add clear warning labels for unsupported platforms, effective/ignored values and conflicting rendering backends. Investigate current Roblox behavior before promising AppleBlox's render-resolution, graphics-API or FPS-cap tricks; its Engine source has commented-out render-resolution controls in the reviewed snapshot. _Done when_: any exposed toggle actually affects a supported setting or is explicitly labeled experimental and tested.

### P2 — Optional only when users request it

- [ ] **Appearance personalization for Thirst.** A few built-in color themes/accent choices and a small icon library; use local assets and performance-safe CSS. Do not import arbitrary executable CSS or borrow AppleBlox artwork. App icon customization needs platform-specific packaging behavior.
- [ ] **Reversible Roblox asset customization.** Research custom fonts, icon recoloring and a local mods manager with previews, size/file counts, enable/disable and robust backups/restores across Roblox updates. First validate whether modifying the signed Roblox installation is supported and does not break updates or client integrity. No patching binaries, anti-cheat evasion, or unsafe remote mod installer.
- [x] **Automatic Thirst release discovery.** Native, bounded check of GitHub's stable Releases API once at startup (default on with opt-out) and manual Check now; semantic-version comparison, plain-text release notes, and a fixed official Releases link. No download or installation. No releases have been published yet; see [Update guide](UPDATES.md).
- [ ] **Signed in-app installation.** Add Tauri's updater plugin only when a long-lived private signing key can be stored securely, an embedded public key and signed manifest exist, and macOS/Windows installation and rollback have been tested. Never auto-install an unsigned Actions artifact. Do not add a separate Roblox background updater.
- [ ] **Multi-instance exploration.** Research official macOS/Windows restrictions and process ownership. Do not offer a mechanism that patches Roblox to defeat instance limits, modifies signatures or evades anti-cheat. Ship only if there is a supported and reliably tested approach.
- [ ] **Bootstrapper themes / Bloxstrap theme compatibility.** Consider only after users ask for it and the loader genuinely needs an independent bootstrap window. AppleBlox's XAML theme implementation is a substantial extra surface; not worth cloning simply for parity.

### Explicitly defer / avoid

- Automatic Roblox session-cookie import from another program, manual cookie entry, or relaxing credential isolation just to mimic AppleBlox account setup.
- Virtual-display FPS unlocking, legacy disallowed Fast Flags, forced datacenter selection, or claims of guaranteed ping reductions. Platform/client compatibility must be proven independently.
- Full AppleBlox-style background update services, Bloxstrap SDK, arbitrary plugins, a public mod workshop and custom scripting in the first public Thirst release. They introduce ongoing work and risk without improving the core launch/server experience.
- Copying AppleBlox's GPL-3.0 implementation/UI assets into MIT Thirst. Use the screenshots to study information architecture and feature scope only, then write independent Tauri/Rust/Svelte code.

**Suggested order:** P0 history → rejoin → installation detection → diagnostics → navigation; then P1 Rich Presence → region preference → account switching. Keep real-device macOS/Windows tests, provider terms and Developer ID signing/notarization as parallel release blockers rather than claiming a polished UI means a public-ready launcher.

## Design principles

1. Never make gameplay slower by leaving expensive launcher tasks running in the background.
2. The UI must remain useful without signing in.
3. Build features around documented, authorized access; do not bypass Roblox account/privacy restrictions.
4. Keep session credentials on-device, isolated from UI JavaScript and third parties.
5. Never display guessed server regions or inferred user identities as facts.
