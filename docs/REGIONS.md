# Thirst region labels — local notes and optional RoValra provider

## Temporary RoValra integration

Thirst includes an **experimental, off-by-default, per-run opt-in** that requests reported server regions from [RoValra](https://www.rovalra.com/). RoValra is independent of Thirst. **Region data by RoValra** is credited in the app. Thirst is a desktop launcher and does not attempt to replace RoValra's browser extension. The provider's [API Terms of Use](https://www.rovalra.com/tou/) restrict competitive uses, require credits and permit access to be blocked without notice; RoValra has not individually endorsed or approved this specific integration. If the provider asks Thirst to stop, the integration can be disabled and removed.

After you enable **Use RoValra regions (temporary)** in Server Browser, Thirst sends a single HTTPS GET to `https://apis.rovalra.com/v1/servers/details`, containing only:
- The public numeric Roblox place ID
- Up to 25 public Roblox server UUIDs visible in the current list

**No `.ROBLOSECURITY` cookie, Roblox user ID, player tokens, private access code, local IP geolocation, or credentials are transmitted.** The request is sent by native Rust, to an exact hardcoded host, with redirects disabled. The response is reduced to city, region and country text; any IP addresses or datacenter IDs supplied by RoValra are discarded. Results are kept in memory for the current server list, are not persisted or re-uploaded, and disappear when the place changes, the list refreshes or the toggle is disabled.

Thirst limits requests to one per 30 seconds, increasing to two minutes on a 429 response, and stops fetching if access is denied. Results are shown as **RoValra · provider-reported**, not independently verified Roblox locations. Their broad continent filters are derived from the reported country only; unrecognized countries remain Unknown. Inaccurate or missing provider values are never guessed from ping.

## Your own temporary region reports

You can also click a server's region to label it manually after checking in-game. This does not call any third-party API. The local report is displayed distinctly and overrides any provider label on your device. You can edit or delete it. Each report is tied to the exact public place ID and ephemeral server UUID and expires after **two hours**.

Local reports are stored in localStorage and never uploaded. Data consists only of a place ID, server UUID, broad-region code, observation timestamp and a fixed `local-report` source. Records are validated and limited to 200 entries.

## Known limitations

- RoValra results are an external service's information, **not independently verified** by Thirst.
- Its API or Terms of Use may change, service might go offline, or a query may be blocked or unsupported.
- Neither local nor provider regions represent the player's actual network ping.
- The default remains **Unknown** and the server list works without any region service.
- A saved server UUID is not a durable identifier for a server that has ended.
- This integration does **not** scrape RoValra pages, redistribute its database, or copy GPL-licensed extension code.

A future first-party or explicitly licensed provider can replace the experimental option without changing the rest of the browser.
