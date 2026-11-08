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
- [x] Soft pointer-following light/hover effect on actionable cards using requestAnimationFrame, honoring reduced-motion and Lite Mode settings.
- [x] Let users **pin individual subplaces as standalone quick-launch tiles** on Overview, alongside experiences (e.g. BLR 8v8 / Pro / Customs), with direct launch and server browser shortcuts. Pins persist locally.
- [x] Sort the My Library experience list by saved order, name A–Z, newest and oldest, with a local-only preference. The Overview shortcut order remains unchanged.
- [ ] Custom manual reordering of games and pinned subplaces. Recently joined servers are deliberately not bookmarked because instances are ephemeral.
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
- [x] Temporary **RoValra** API provider enabled on manual server refresh by default, with a remembered opt-out, visible attribution, country flag emoji, compact labels, exact-host native requests, a 25-server batch cap, 30-second cooldown and no Roblox cookie transfer. Results remain in memory. Provider terms must be respected and access can be withdrawn.
- [ ] Seek explicit permission or a longer-term licensed provider for automatic regions. Do not confuse provider-reported regions with user latency or independently verified datacenter positions.
- [ ] Clarify use conditions with RoValra before making the temporary region feature a default or promoting it as an alternative to their extension. No scraping or redistribution.
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
