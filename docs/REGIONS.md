# Thirst region labels — experimental local reports

**Current capability:** This feature does not automatically discover a Roblox server's datacenter. Roblox public server listings used by Thirst do not expose a trusted region field, and the launcher does not estimate regions from Roblox-reported ping. Unknown is the correct default.

## How it works

1. Browse servers, join the server, and check its region using information actually available to you in-game. Do not guess from the displayed Roblox-reported ping.
2. Back in Thirst, click **Unknown** in that server's Region column, select the broad region, and click **Save locally**.
3. The entry is displayed as **your unverified local report**, not a Roblox/Thirst verified measurement. You can filter by your reported region, filter Unknown entries, edit or clear the report.
4. Entries expire **two hours after the last save** and are tied to the exact public `placeId` and server UUID. If the instance ends, the report will not identify any replacement server.

No data is transmitted to Thirst, RoValra, Roblox or third-party analytics by the region-report feature. The only persistent fields are a Roblox place ID, server UUID, region code, observation timestamp and a fixed source value (`local-report`). Stored records are validated, capped to 200, and expired/invalid records are not surfaced.

## Future verified-provider support

An automatic region provider must supply a region, the exact server identity, its **data source**, the observation time and an explicit confidence or verification level. A confidence based solely on server ping or user location is not sufficient. Server regions must be labeled by source (e.g. `Provider · verified`, `Community report · unverified`), and unknown must remain available.

Third-party provider access requires a reviewed license or authorization, minimal data sharing, a short cache TTL, and a clear disable switch. We **do not call RoValra's API** or copy its extension implementation: its public terms restrict use in competing tools and unauthorized scraping.

## Limitations

- These reports are **local** to the device; other users do not receive them.
- Changing network/ping does not change the region label.
- A report is not proof that Roblox placed a server in that datacenter.
- Server identity and membership are ephemeral; no persistent-server favorites.
- Labels can be edited or cleared without deleting game or subplace bookmarks.

This is a bridge toward a future authorized source, not a substitute for actual verified server-region discovery.
