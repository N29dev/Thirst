# Thirst 0.4 — build and test

Thirst's desktop app runs on macOS and Windows. Roblox must already be installed and registered as the Roblox protocol handler. Desktop login, including the macOS Swift/WKWebView helper, is kept native and uses the OS credential store.

## Developer mode

~~~bash
git clone https://github.com/N29dev/Thirst.git
cd Thirst
npm install
npm run tauri dev
~~~

## Unsigned test builds

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

The macOS package is an .app bundle. The Windows package is an NSIS .exe. Artifacts are attached to successful packaging workflow runs. These are developer-test packages, not signed/notarized public releases. macOS Gatekeeper and Windows SmartScreen may warn until publisher signing and real-device verification are complete. Do not disable operating-system protections globally.

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
