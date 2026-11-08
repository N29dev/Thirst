# Thirst game history and rejoin (experimental)

History is **opt-in and local-only**. On the Overview page, enable **Game history** and join a Roblox game. While Overview is visible, Thirst scans the bounded tail of recent Roblox **Player** logs every 30 seconds. **Refresh** triggers a manual scan; you can also turn tracking off, clear saved history, and launch games normally without using this feature.

## What is collected

Only the Roblox place ID, the instance UUID when present, the timestamp when Player logged the join attempt, and (when the log provides it) a leave/disconnect timestamp. Thirst never returns raw Player log messages, player IP addresses, cookies, usernames or machine file paths to the web UI. The native reader is limited to fixed Roblox log folders and at most twelve recent log files, reading at most 512 KiB from each.

- macOS Player logs: \`~/Library/Logs/Roblox\`
- Windows Player logs: \`%LOCALAPPDATA%\Roblox\logs\`

Those paths are also described in [Roblox's official support instructions](https://en.help.roblox.com/hc/en-us/articles/360016022492-How-to-Retrieve-Roblox-Studio-Logs).

History is retained in the launcher's existing local browser storage only, at most **80 entries for 30 days**. Tracking is disabled by default. Enabling it starts a new observation window and **doesn't import previous games**. Disabling tracking keeps the existing history until you clear it, while clearing advances the observation window so old log lines aren't repopulated.

**Limits:** A "Joining game" log message is a **join attempt**, not proof the session completed. Game durations are approximations only when Roblox writes a recognized leave/disconnect marker; otherwise they are not displayed. Some Roblox versions, launch methods, log names or permissions may prevent observation. The UI labels log-derived entries separately from Thirst's existing manual "Recently opened" shortcuts. The current implementation observes only when Thirst's **Home/Overview page is visible**; it does not run a persistent background service after the app closes.

### Optional metadata

Saved-game titles and verified Roblox CDN icons appear automatically from your existing library. For an unsaved place, choose **Get title & art** to request public game details on demand from Roblox. This is a separate user-initiated network request; simply tracking history never makes remote requests.

## Rejoin behavior

**Rejoin** attempts the exact previous public instance only if its log entry is newer than **15 minutes**. It checks at most **three public server pages** (75 entries, if available), with no authentication cookie. If it positively finds the instance and it has a free slot, Thirst launches that instance. If the server is full, too old, cannot be confirmed in the sampled pages, or the API is unavailable, Thirst opens the game/place instead and explains why. A negative lookup **does not mean the server no longer exists**, because Roblox's public server listing is paginated and can be incomplete.

**Open game** always opens the place without attempting an instance match. Instance UUIDs are ephemeral and aren't permanent bookmarks. Rejoin checks have a short local cooldown and make no background network requests.

## Tests and status

- Rust unit tests cover timestamped join/leave extraction, invalid IDs, deduplication, implicit closure on teleport, and not returning raw log data.
- Svelte + TypeScript builds validate the UI integration; desktop Rust checks run on macOS and Windows in GitHub Actions.
- Still needs **real-device validation** with recent Roblox Player versions on both platforms. We must confirm that the current client writes the expected markers and measure CPU/memory usage during long sessions before calling this a release-ready activity tracker.

The implementation is independent of AppleBlox/Bloxstrap code. Their public designs helped identify useful features, but no GPL-licensed source has been copied.
