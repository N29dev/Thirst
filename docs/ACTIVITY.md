# Thirst game history and rejoin (experimental)

History is **opt-in and local-only**. On the Overview page, enable **Game history** and join a Roblox game. While Overview is visible, Thirst scans the bounded tail of recent Roblox **Player** logs every 30 seconds. **Refresh** triggers a manual scan; you can also turn tracking off, clear saved history, and launch games normally without using this feature.

## What is collected

Only the Roblox place ID, instance UUID when present, the time of the join attempt, a separate **server-connection marker** when Player logs one, and (when present) the disconnect timestamp. No network address, port, or player identity is returned by the native parser. Thirst never returns raw Player log messages, player IP addresses, cookies, usernames or machine file paths to the web UI. The native reader is limited to fixed Roblox log folders and at most twelve recent log files, reading at most 512 KiB from each.

- macOS Player logs: \`~/Library/Logs/Roblox\`
- Windows Player logs: \`%LOCALAPPDATA%\Roblox\logs\`

Those paths are also described in [Roblox's official support instructions](https://en.help.roblox.com/hc/en-us/articles/360016022492-How-to-Retrieve-Roblox-Studio-Logs).

History is retained in the launcher's existing local browser storage only, at most **80 entries for 30 days**. Tracking is disabled by default. Enabling it starts a new observation window and **doesn't import previous games**. Disabling tracking keeps the existing history until you clear it, while clearing advances the observation window so old log lines aren't repopulated.

**Limits:** A "Joining game" log message is a **join attempt**, not proof the session completed. Thirst now distinguishes a subsequent Player network connection event from an unconfirmed attempt. Even a connection marker **is not proof gameplay finished loading**. Approximate connected time is shown only if the same session has both connection and disconnect markers; otherwise it is omitted. Some Roblox versions, launch methods, log names or permissions may prevent observation. The UI labels log-derived entries separately from Thirst's existing manual "Recently opened" shortcuts. The current implementation observes only when Thirst's **Home/Overview page is visible**; it does not run a persistent background service after the app closes.

### Optional metadata

Saved-game titles and verified Roblox CDN icons appear automatically from your existing library. For an unsaved place, choose **Get title & art** to request public game details on demand from Roblox. This is a separate user-initiated network request; simply tracking history never makes remote requests.

## Rejoin behavior

**Rejoin** considers a known instance only for observed Player-log entries seen within **two hours** of the last session timestamp. It scans at most **six public server pages** (up to 150 entries, if returned), with no authentication cookie or automatic background lookup. If it positively finds the instance with a free slot **in the sampled listing**, Thirst asks Roblox to launch that instance (which can still fill up before join). If the server is full, too old, cannot be confirmed in the sampled pages, or the API is unavailable, Thirst opens the game/place instead and explains why. A negative lookup **does not mean the server no longer exists**, because Roblox's public server listing is paginated and can be incomplete.

**Open game** always opens the place without attempting an instance match. Instance UUIDs are ephemeral and aren't permanent bookmarks. Rejoin checks run only on a direct user click; repeated checks are not scheduled. If Roblox rate-limits the listing, the normal place opens instead.

## Tests and status

- Rust unit tests cover timestamps, valid/invalid join markers, duplicate join handling, connection/leave parsing, teleport closure, and a regression proving raw server IPs never enter IPC output.
- Svelte + TypeScript builds validate the UI integration; desktop Rust checks run on macOS and Windows in GitHub Actions.
- Still needs **real-device validation** with recent Roblox Player versions on both platforms. We must confirm that the current client writes the expected markers and measure CPU/memory usage during long sessions before calling this a release-ready activity tracker.

AppleBlox's [Player log event categories](https://github.com/AppleBlox/appleblox/blob/main/frontend/src/windows/main/ts/roblox/instance.ts) were used to understand the distinction between joining, server connection and disconnect. Thirst's native bounded parser and local history remain independent implementations; no GPL-licensed watcher or stored IP-geolocation logic was copied.
