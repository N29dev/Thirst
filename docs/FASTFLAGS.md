# Thirst Fast Flags guide

Thirst edits local **Roblox Player** settings. The [Roblox local Fast Flags allowlist](https://devforum.roblox.com/t/allowlist-for-local-client-configuration-via-fast-flags/3966569/) published in September 2025 contains 18 keys. Roblox can add or remove them without warning; a successful write does not prove that the client applied every value.

## How to use

Open **Fast Flags** in the sidebar. Thirst reads the current settings file from your operating system. Choose **Lower graphics**, **Balanced**, or **Higher quality**, or add individual allowed keys. You can also import an existing JSON file or paste JSON:

~~~json
{
  "DFFlagTextureQualityOverrideEnabled": "True",
  "DFIntTextureQualityOverride": "2",
  "FIntDebugForceMSAASamples": "2"
}
~~~

Review the draft, click **Save to Roblox**, and fully restart Roblox. Nothing is written to Roblox just from importing a preset. The editor can copy/export your draft, enable or disable individual imported flags, and preserve other unrelated entries already in the installed file. Unsupported or invalid imported entries are skipped.

### Import a named preset

Thirst accepts both raw Roblox `ClientAppSettings.json` objects and named presets that include a `flags` array, as in this example:

~~~json
{
  "name": "Example GK profile",
  "flags": [
    {"flag": "DFIntTextureQualityOverride", "value": "1", "enabled": true},
    {"flag": "DFFlagDisableDPIScale", "value": "True", "enabled": false}
  ],
  "type": "default",
  "games": []
}
~~~

Choose **Import preset** or drag the file onto the drop zone. By default this **replaces the current unsaved draft**; you may choose **Merge** to retain other active flags. Disabled items are shown separately and **are not written** to Roblox. Their values are retained for named-preset exports. Per-game presets in `games` are not applied or exported by Thirst.

### Export

Choose one of two formats:

- **Named preset:** creates a shareable `your-preset-name.json` in the same `name`, `flags`, `enabled`, `type`, `games` structure. Enabled and disabled supported flags round-trip.
- **Roblox settings:** creates `ClientAppSettings.json` with only enabled, allowlisted flags. There is no preset metadata or disabled flags.

**Copy** puts the selected export format on the clipboard. Exporting does not include Roblox cookies, account sessions or other private credentials.

## Install paths

- macOS: ~/Library/Preferences/com.roblox.Roblox/ClientAppSettings.json (AppleBlox-compatible user-level location).
- Windows: %LOCALAPPDATA%/Roblox/ClientSettings/ClientAppSettings.json.

The app displays the exact path it opens. Some Roblox versions or installation methods may use a different active path. Thirst never edits the signed Roblox.app bundle or Windows version folders. Before its first change to an existing settings file it creates a file named ClientAppSettings.thirst-backup.json beside the original.

## What the loader does not do

This is not a framerate unlocker, executor, or graphics injecter. Popular old scheduler overrides such as DFIntTaskSchedulerTargetFps are not on the published allowlist and will be ignored by Roblox Player. Presets affect only texture quality and anti-aliasing; the result depends on the client, platform, and GPU. Rendering backend preferences may be platform-specific. Full Roblox restart may be needed.

**Your Roblox login is untouched.** Fast Flags use only local Player settings, never read your cookie, and never alter the OS Keychain or Credential Manager.

Sources: [Roblox official allowlist](https://devforum.roblox.com/t/allowlist-for-local-client-configuration-via-fast-flags/3966569/), [AppleBlox file locations](https://github.com/AppleBlox/docs/blob/main/reference/file-locations.md), [Roblox community file path reference](https://devforum.roblox.com/t/fast-flags-clientappsettingsjson-folder-locations-for-most-os-eg-windows-macos-android-and-ios/3180597).
