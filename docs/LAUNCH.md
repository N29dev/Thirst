# Roblox installation & launch diagnostics

Thirst launches Roblox through its registered `roblox://experiences/start` protocol. Sending the URL **does not mean Roblox finished opening or joined a server**. The separate, opt-in Game History records join attempts and network connection markers when Roblox writes them to local Player logs.

## Settings → Roblox & Launch

When opened, the panel checks the local installation, its available version metadata, its registered `roblox://` handler and whether the Roblox Player process appears to be running. **Re-detect** repeats these checks on demand. The information is a local snapshot; Thirst does not poll it in the background.

- **macOS:** Check `/Applications/Roblox.app` and `~/Applications/Roblox.app`, followed by an indexed Spotlight search if necessary. Each candidate must have `Contents/Info.plist`, a `Contents/MacOS/RobloxPlayer` executable, and Roblox's verified `com.roblox.RobloxPlayer` bundle ID. The version comes from `CFBundleShortVersionString`. LaunchServices is queried for the default `roblox` URL scheme handler, which can be missing or assigned to another app.
- **Windows:** Check `%LOCALAPPDATA%\Roblox\Versions\*` for `RobloxPlayerBeta.exe` (up to 32 newest version folders), plus the standard direct executable location. When present, the version folder's name is reported instead of an invented application version. The Windows registry is checked for a registered `roblox` launch command. Installation and handler detection are separate.
- **Manual location:** Browse with a system dialog or paste a path. Select the whole `Roblox.app` on macOS or `RobloxPlayerBeta.exe` on Windows. The native layer validates the candidate before writing a small path-only JSON setting under Thirst's per-user config directory. **Clear override** removes that setting and returns to automatic discovery. Invalid saved overrides do not block fallback discovery.
- **Show in folder:** Reveals the *validated* Roblox installation in Finder or Explorer. This is not an executable installer. **Official download** opens only `https://www.roblox.com/download`, with no Thirst-hosted binary or updater.

None of these commands reads or returns the Roblox session cookie, edits Roblox files, downloads an installer, or tries to change the system's registered URL handler.

## Troubleshooting

| What you see | Meaning and next step |
| --- | --- |
| Not found | Install Roblox through the official download page, or browse to an existing official installation. |
| Found but handler missing | Try opening Roblox directly or reinstalling it via Roblox's official setup; re-detect afterward. |
| Different handler | Another app owns the `roblox://` scheme. Launching may open it instead of Roblox. Review your platform's app association. |
| Registered, Roblox not running | The system has a handler, but Roblox Player is not currently detected. This does not establish whether a past launch succeeded. |
| Launch request accepted | The operating system accepted the URL. It is **not proof of a successful game join**. Check Game History for the separate connection marker. |

## Scope and testing

The implementation borrows general UX ideas from AppleBlox's [path manager](https://github.com/AppleBlox/appleblox/blob/main/frontend/src/windows/main/ts/roblox/path-manager.ts) and [path selector](https://github.com/AppleBlox/appleblox/blob/main/frontend/src/windows/main/components/roblox/path-selector.svelte), but uses original Rust detection, operating-system APIs and validation.

**Still requires real-device testing:** moved or renamed Roblox installations, nonstandard macOS application locations, Windows installed versions, missing/wrong URL handlers, official installer recovery, and post-launch process detection. macOS/Windows CI builds check compilation; only actual device tests establish that platform state and UI values are accurate. Avoid interpreting a returned process status or protocol registration as a completed launch.

Thirst intentionally does **not** add a second Roblox updater, modify the client, run arbitrary user-supplied commands, or install unsigned executables.
