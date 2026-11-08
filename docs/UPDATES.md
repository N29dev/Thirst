# Updating Thirst

Thirst now checks **official public GitHub Releases** once after startup (with a short delay), unless disabled in **Settings → Thirst updates**. You can also click **Check now**. A check contacts only \`api.github.com/repos/N29dev/Thirst/releases/latest\` with no Roblox session, user identity or cookies, and uses semantic version comparison against the installed app version.

**Current limitation:** This feature is **automatic release detection**, not automatic installation. As of the first implementation there are **no published GitHub Releases**, only development artifacts attached to Actions workflow runs. For this reason Settings currently reports "No official release published yet." It does not promote GitHub Actions artifacts to releases or install anything.

A newer published version will show a notice and link to \`https://github.com/N29dev/Thirst/releases/latest\`. The release notes are limited in length and shown as plain text. The app never runs a downloaded executable, updater script or arbitrary external URL.

## Enabling safe in-app installation later

Tauri v2 requires **cryptographically signed updater artifacts**. Unlike macOS ad-hoc or Developer ID code signing, this is an independent updater signature based on a private/public key pair. Tauri rejects updates with invalid signatures; signature validation cannot be disabled.

1. Generate one long-lived updater signing key pair **on a trusted development machine**, for example using Tauri's \`signer generate\` CLI. Store and back up the **private** key securely, outside the repository, and protect it with a password. Never commit it or expose it to the app frontend.
2. Configure Tauri's updater plugin with the **public** key in \`tauri.conf.json\`, using the fixed HTTPS endpoint \`https://github.com/N29dev/Thirst/releases/latest/download/latest.json\`.
3. Enable \`bundle.createUpdaterArtifacts\` for a dedicated signed release workflow. Sign macOS app updater archives and Windows NSIS artifacts with the **same updater private key**, held in GitHub Actions encrypted secrets or protected environment secrets. Generate and upload their \`.sig\` files; \`latest.json\` must embed the signatures and give exact platform-specific artifact URLs.
4. Ensure app versions in \`package.json\`, \`src-tauri/Cargo.toml\` and \`src-tauri/tauri.conf.json\` match the release's semantic version. Test upgrades and downgrades/rollback behavior using separate test installations before activating installs.
5. For public macOS distribution, add Developer ID signing and notarization and verify bundled Swift helper signing. For Windows, add publisher code signing where feasible. Neither code signing system replaces Tauri's updater signatures.
6. Only then enable the updater's explicit **Download & install** action in Thirst. Show version and release notes before installation, progress while downloading, and a restart confirmation. Do not silently install on launch, overwrite Roblox, or install unverified unsigned Actions artifacts.

The official [Tauri v2 Updater documentation](https://v2.tauri.app/plugin/updater/) describes the signature, manifest, \`createUpdaterArtifacts\` and JavaScript installation flows.

## Why development builds are not auto-installed

GitHub Actions artifacts are temporary, can expire, and are not versioned or cryptographically signed for Tauri's updater. They are useful for testing builds but cannot serve as the trusted update channel. The current macOS bundle is **ad-hoc signed only**, so a user may still have to use a limited Gatekeeper exception to test it; that exception is not a safe update mechanism. A malformed or compromised update could expose an authenticated Roblox session saved in the launcher. Keep update verification strict.

## Smoke-test checklist

- [ ] With **no** GitHub Releases, Settings shows the no-releases state without an error.
- [ ] If GitHub is offline/rate-limited, the check fails non-fatally, with a useful message for manual retries.
- [ ] Check for updates once on startup and confirm no repeating background polling.
- [ ] Turning automatic checks off prevents startup checks; manual **Check now** still works.
- [ ] When a stable release tag newer than the installed version is published, it shows the correct version, release notes and official link.
- [ ] No Roblox Keychain or Credential Manager prompt appears as a result of checking updates.
- [ ] Native update-check Rust unit tests and macOS/Windows desktop builds pass.
- [ ] Future: install a version-signed test update end-to-end on both platforms before enabling one-click installation.

This document describes **updating Thirst itself**, not updating Roblox.
