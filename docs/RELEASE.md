# Thirst 0.4 — build and test

Thirst's desktop app runs on macOS and Windows. Roblox must already be installed and registered as the Roblox protocol handler. Desktop login, including the macOS Swift/WKWebView helper, is kept native and uses the OS credential store.

## Developer mode

~~~bash
git clone https://github.com/N29dev/Thirst.git
cd Thirst
npm install
npm run tauri dev
~~~

## Ad-hoc signed test builds

Thirst has a [GitHub Actions test-bundle workflow](https://github.com/N29dev/Thirst/actions/workflows/package.yml) for a macOS Apple Silicon app and a Windows NSIS installer. The workflow checks the macOS app bundle actually contains its native Roblox login helper.

From source on macOS:

~~~bash
npm install
npm run bundle:mac
~~~

From source on Windows:

~~~powershell
npm install
npm run bundle:windows
~~~

The macOS package is an Apple Silicon .app bundle. Tauri now applies an **ad-hoc code signature** (`bundle.macOS.signingIdentity = "-"`); the separately bundled Swift login helper is also ad-hoc signed. GitHub Actions verifies both signatures before uploading the test build. This is **not** an Apple Developer ID signature and **not** notarization. macOS Gatekeeper can still require an individual user exception. Windows provides a test NSIS .exe and may trigger SmartScreen. These artifacts are developer previews, not a public, verified release.

GitHub Actions uploads `Thirst-macOS-arm64-ad-hoc`, which downloads as an artifact ZIP containing `Thirst-macOS-arm64.zip`. Extract the inner ZIP with Archive Utility to obtain `Thirst.app`, then drag it into Applications.

### macOS says "Thirst is damaged and can't be opened"

The message can indicate a blocked unsigned/unnotarized download **or** a genuinely corrupt or modified app. First download the latest build from this repository's own Actions workflow and extract the archive fully; don't run a partially extracted or modified bundle. Confirm your Mac is Apple Silicon (M-series); this workflow doesn't build an Intel version.

If you trust this specific build, after trying to open it:

1. Open **System Settings → Privacy & Security** and use **Open Anyway** if macOS offers that option.
2. If macOS still reports damage, verify the bundle before applying any exception:

~~~bash
codesign --verify --deep --strict --verbose=2 /Applications/Thirst.app
~~~

3. If signature verification succeeds and you are certain the bundle came directly from this project's own workflow, you can remove the quarantine metadata from **only this copy** to test it:

~~~bash
xattr -dr com.apple.quarantine /Applications/Thirst.app
open /Applications/Thirst.app
~~~

If verification fails, **do not override Gatekeeper**. Re-download or rebuild; include the `codesign` output in a bug report. The `xattr` command is not for unrelated or untrusted applications. Never disable Gatekeeper globally (`spctl --master-disable`).

For a warning-free public installer, Thirst still needs a Developer ID Application certificate, Apple notarization, and real-device launch testing. Ad-hoc signing cannot substitute for those requirements.

## Login and settings

- macOS bundles include the native Swift login helper in the app resources; developer builds load the same helper from Cargo output.
- No Roblox cookie is included in builds or in GitHub artifacts. Existing credentials remain in macOS Keychain or Windows Credential Manager, with a native-only in-memory session while Thirst runs.
- Fast Flags are separate Roblox Player ClientAppSettings.json settings. The project does not modify the Roblox app executable or bypass Roblox's allowlist.
- Library JSON backups contain only public saved game metadata and pinned subplace IDs, not credentials.
- Existing public server browsing works without signing into Roblox.

## Before a signed public release

- Test the installed macOS app and Windows installer (not just Rust tests) with the official Roblox login and launch handler.
- Verify Fast Flags apply in the current Roblox Player release on both platforms.
- Review each upstream provider's API usage conditions, including RoValra.
- Provide Apple Developer ID and Windows code-signing credentials, plus normal signing/notarization. None are stored in this repository.
