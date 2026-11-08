# Thirst region labels — local notes and optional RoValra provider

## Temporary RoValra integration

Thirst includes an **experimental region integration, enabled by default when you manually refresh a server list**, that requests reported server regions from [RoValra](https://www.rovalra.com/). You can disable it in Server Browser; this opt-out is stored locally and respected after restart. No request is made solely from starting the launcher. RoValra is independent of Thirst. **Region data by RoValra** is credited in the app. Thirst is a desktop launcher and does not attempt to replace RoValra's browser extension. The provider's [API Terms of Use](https://www.rovalra.com/tou/) restrict competitive uses, require credits and permit access to be blocked without notice; RoValra has not individually endorsed or approved this specific integration. If the provider asks Thirst to stop, the integration can be disabled and removed.

When the **RoValra regions (default on)** switch is enabled and you manually refresh a server list, Thirst sends public IDs to an approved, fixed HTTPS endpoint. For BL:R 8v8, Pro and Main, it first checks Glutony's shared cache at `https://glutony.scp343hey.workers.dev/api/servers/regions/shared?mode=...&ids=...`. This reads locations shared with the BL:R website and Discord bot. Glutony can resolve unknown IDs from its **recent public server sample** through RoValra, then store sanitized short-lived results in D1 (positive cache 3 minutes; missing 45 seconds). For public IDs outside that sample, Thirst falls back to `https://apis.rovalra.com/v1/servers/details`; these fallback results are not uploaded to Glutony. Non-BL:R games also use RoValra directly.

Both request paths contain only:
- The public numeric Roblox place ID
- Up to 25 public Roblox server UUIDs visible in the current list

**No `.ROBLOSECURITY` cookie, Roblox user ID, player tokens, private access code, local IP geolocation, Cloudflare API token or Discord credentials are transmitted.** The request is sent by native Rust to exact hardcoded hosts, with redirects disabled. Both Glutony and RoValra can observe the source IP of an HTTPS request. The response is reduced to city, region and country text; any IP addresses or datacenter IDs supplied by RoValra are discarded. Results are kept in Thirst's memory for the current server list and disappear when the place changes, the list refreshes or the toggle is disabled. Only Glutony persists sanitized region details in its shared Cloudflare D1 cache; the Thirst app itself does not persist or upload raw region results. An additional one-time D1 migration is necessary on the Cloudflare Worker.

Thirst limits requests to one per 30 seconds, increasing to two minutes on a 429 response, and stops fetching if access is denied. Results are shown as **RoValra · provider-reported**, not independently verified Roblox locations. Recognized country values display Unicode flag emoji (for example 🇧🇷 and 🇺🇸). Each chip shows a compact city-and-country label, while the full city/region/country is available on hover. Their broad continent filters are derived from the reported country only; unrecognized countries remain Unknown. Inaccurate or missing provider values are never guessed from ping.

## Glutony Worker deployment

This requires `migrations/0007_shared_server_regions.sql` in [N29dev/Glutony](https://github.com/N29dev/Glutony). Apply the migration to the existing D1 database from a Cloudflare-authenticated terminal:

```bash
npx wrangler d1 migrations apply glutony-customs --remote
```

Normal Glutony deployment and Discord interaction registration do not apply D1 migrations. Until it is applied, the Worker keeps its existing in-memory RoValra caching behavior, and Thirst's direct fallback still functions.

No Cloudflare/Discord/Roblox token should be embedded in Thirst to authorize a shared read. The Worker endpoint only accepts public UUIDs present in a recent public server sample. If the provider's terms restrict this shared use, the provider integration must be disabled or replaced with an authorized source.

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
- Because RoValra is a third-party HTTPS service, its operator can see your network source IP and the public place/server IDs sent by Thirst. No account cookie is included.

A future first-party or explicitly licensed provider can replace the experimental option without changing the rest of the browser.
