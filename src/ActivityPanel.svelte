<script lang="ts">
  import { onMount } from "svelte";
  import { fetchPublicServers, lookupGame, recentPlayerSessions } from "./lib/backend";
  import {
    activityDuration, activityIcon, addActivityMetadata, clearActivityHistory, historyEnabled,
    mergeObservedSessions, readActivityHistory, setHistoryEnabled,
    shortActivityTime, trackingSince
  } from "./lib/activity";
  import type { ActivityEntry } from "./lib/activity";
  import type { SavedGame } from "./lib/types";

  export let games: SavedGame[] = [];
  export let onLaunch: (placeId: number, jobId?: string | null) => Promise<boolean>;

  let tracking = false;
  let entries: ActivityEntry[] = [];
  let scanning = false;
  let rejoining = false;
  let resolvingPlaceId: number | null = null;
  let lastRejoinCheck = 0;
  let message = "";
  let error = "";

  onMount(() => {
    tracking = historyEnabled();
    entries = readActivityHistory();
    if (tracking) void refresh(false);
    // Local log scan only, only while Home is mounted and visible.
    // No Roblox requests, user tokens, external telemetry or persistent worker.
    const timer = setInterval(() => {
      if (tracking && document.visibilityState === "visible") void refresh(false);
    }, 30_000);
    return () => clearInterval(timer);
  });

  async function refresh(manual = true) {
    if (scanning || !tracking) return;
    scanning = true;
    if (manual) { error = ""; message = ""; }
    try {
      const before = entries.length;
      const observed = await recentPlayerSessions(trackingSince());
      entries = mergeObservedSessions(entries, observed, games, trackingSince());
      if (manual) {
        message = observed.length
          ? "Read " + observed.length + " recent Player join events. " +
            Math.max(0, entries.length - before) + " new history entries."
          : "No join events found since tracking was enabled. Roblox logs may vary by version.";
      }
    } catch (reason) {
      if (manual) error = "Couldn't read Roblox Player logs: " + String(reason);
    } finally {
      scanning = false;
    }
  }

  function setTracking(next: boolean) {
    try {
      setHistoryEnabled(next);
      tracking = next;
      error = "";
      message = next
        ? "History is on. Thirst will read recent local Roblox Player logs only while this Home screen is visible."
        : "History tracking is off. Saved history stays on this device until you clear it.";
      if (next) void refresh(false);
    } catch {
      error = "Could not save history preference.";
    }
  }

  function clear() {
    try {
      entries = clearActivityHistory();
      error = "";
      message = "Local game history cleared. Previously recorded joins will not be reimported.";
    } catch {
      error = "Could not clear game history.";
    }
  }

  async function resolveDetails(placeId: number) {
    if (resolvingPlaceId !== null) return;
    resolvingPlaceId = placeId;
    error = "";
    try {
      // Explicit user action, not background metadata polling.
      const metadata = await lookupGame(placeId);
      entries = addActivityMetadata(entries, placeId, metadata);
      message = "Game title and artwork updated from Roblox.";
    } catch (reason) {
      error = "Couldn't find public details for that place: " + String(reason);
    } finally {
      resolvingPlaceId = null;
    }
  }

  async function rejoin(entry: ActivityEntry) {
    if (rejoining) return;
    const now = Date.now();
    if (lastRejoinCheck > now) {
      message = "Rejoin lookup is cooling down. You can use Open game instead.";
      return;
    }
    rejoining = true;
    error = "";
    message = "";
    try {
      // Roblox public server listings are paginated and not exhaustive. Only
      // target the old instance when it is positively found with a free slot.
      let target: string | null = null;
      let explanation = "Opened the game; exact server not verified.";
      if (entry.serverId && now - entry.joinedAt < 15 * 60_000) {
        lastRejoinCheck = now + 15_000;
        try {
          let cursor: string | null = null;
          let found = false;
          for (let page = 0; page < 3; page++) {
            const result = await fetchPublicServers(entry.placeId, cursor, false);
            const server = result.data.find(s => s.id.toLowerCase() === entry.serverId?.toLowerCase());
            if (server) {
              found = true;
              if (server.playing < server.maxPlayers) {
                target = server.id;
                explanation = "Opening the verified public server in Roblox.";
              } else {
                explanation = "Previous server is full; opening the game instead.";
              }
              break;
            }
            cursor = result.nextPageCursor;
            if (!cursor) break;
          }
          if (!found) {
            explanation = "Could not verify that instance in sampled public pages; opening the game instead.";
          }
        } catch {
          explanation = "Public server lookup is unavailable; opening the game instead.";
        }
      } else {
        explanation = "That server entry is older than 15 minutes or has no instance ID. Opening the game instead.";
      }
      const opened = await onLaunch(entry.placeId, target);
      if (opened) message = explanation;
      else error = "Roblox didn't open. Check the error notification.";
    } finally {
      rejoining = false;
    }
  }
</script>

<section class="activity-section">
  <div class="section-heading activity-heading">
    <div><span class="eyebrow">LOCAL ACTIVITY</span><h2>Game history</h2></div>
    <div class="activity-actions">
      {#if tracking}<button class="link-button" disabled={scanning} onclick={() => void refresh()}>{scanning ? "Checking…" : "↻ Refresh"}</button>{/if}
      {#if entries.length}<button class="link-button" onclick={clear}>Clear history</button>{/if}
    </div>
  </div>
  <div class="activity-intro">
    <div>
      <strong>{tracking ? "Tracking enabled" : "History is optional"}</strong>
      <p>Detect join attempts from Roblox Player logs on your Mac or PC. Logs stay local; Thirst doesn't upload them or read your Roblox cookie.</p>
    </div>
    <label class="activity-toggle"><input type="checkbox" checked={tracking} onchange={e => setTracking(e.currentTarget.checked)}/><span class="activity-switch"></span><span>{tracking ? "On" : "Off"}</span></label>
  </div>
  {#if error}<p class="activity-alert activity-error" role="alert">{error}</p>{/if}
  {#if message}<p class="activity-alert" role="status">{message}</p>{/if}

  {#if entries.length}
    <div class="activity-grid">
      {#each entries.slice(0,8) as entry (entry.placeId + ":" + entry.joinedAt + ":" + entry.serverId)}
        <div class="activity-card">
          <div class="activity-cover">
            {#if entry.iconUrl || activityIcon(games,entry.placeId)}
              <img src={entry.iconUrl ?? activityIcon(games,entry.placeId) ?? ""} alt="" loading="lazy"/>
            {:else}
              <span class="activity-placeholder">◈</span>
            {/if}
            <span class="activity-origin">{entry.source === "player-log" ? "Player log" : "Launch request"}</span>
          </div>
          <div class="activity-details">
            <strong title={entry.name}>{entry.name}</strong>
            <small>{shortActivityTime(entry.joinedAt)} · Place {entry.placeId}</small>
            {#if activityDuration(entry)}
              <small>Approx. session: {activityDuration(entry)}</small>
            {/if}
            <div class="activity-card-actions">
              <button class="btn primary small" disabled={rejoining} onclick={() => void rejoin(entry)}>↺ Rejoin</button>
              <button class="btn secondary small" onclick={() => void onLaunch(entry.placeId,null)}>Open game</button>
              {#if entry.name.startsWith("Place ")}
                <button class="btn secondary small" disabled={resolvingPlaceId !== null}
                  onclick={() => void resolveDetails(entry.placeId)}>{resolvingPlaceId === entry.placeId ? "Loading…" : "Get title & art"}</button>
              {/if}
            </div>
          </div>
        </div>
      {/each}
    </div>
    <p class="activity-footnote">Click Get title & art to fetch optional public Roblox metadata. Player logs indicate a join attempt, not a guaranteed completed session. Exact-server rejoin checks up to 3 public server pages, then falls back to the place. History keeps at most 80 entries for 30 days.</p>
  {:else}
    <div class="activity-empty">
      <span>◌</span>
      <strong>{tracking ? "No tracked games yet" : "Your play history, under your control"}</strong>
      <p>{tracking
        ? "Join a Roblox game, then return here. The launcher checks local Player logs while Home is visible."
        : "Enable tracking to start recording future Roblox join attempts, even when you launch games outside Thirst."}</p>
    </div>
  {/if}
</section>

<style>
  .activity-section{margin-bottom:35px;min-width:0}
  .activity-heading{display:flex;align-items:end;justify-content:space-between;gap:10px;flex-wrap:wrap}
  .activity-actions{display:flex;align-items:center;gap:17px}
  .activity-actions .link-button:disabled{opacity:.45;cursor:not-allowed}
  .activity-intro{display:flex;align-items:center;justify-content:space-between;gap:20px;padding:18px 21px;margin:16px 0;border:1px solid #355064;border-radius:15px;background:linear-gradient(100deg,#142e3d,#111e2d)}
  .activity-intro strong{display:block;font-size:12px;color:#d7f7fa}
  .activity-intro p{font-size:11px;line-height:1.65;color:#9cb2c2;margin:5px 0 0;max-width:670px}
  .activity-toggle{display:flex;gap:8px;align-items:center;white-space:nowrap;cursor:pointer;color:#ccebf4;font-size:11px;font-weight:750}
  .activity-toggle input{position:absolute;width:1px;height:1px;opacity:0}
  .activity-switch{display:block;width:39px;height:22px;border-radius:20px;background:#243644;border:1px solid #496073;position:relative;transition:background .16s}
  .activity-switch:after{content:"";position:absolute;left:3px;top:3px;width:14px;height:14px;border-radius:50%;background:#a3b2be;transition:transform .16s}
  .activity-toggle input:checked + .activity-switch{background:#266d78;border-color:#70cbd6}
  .activity-toggle input:checked + .activity-switch:after{transform:translateX(16px);background:#b4f8fa}
  .activity-toggle input:focus-visible + .activity-switch{outline:2px solid #98e9f1;outline-offset:3px}
  .activity-alert{background:#143542;border:1px solid #2c6a73;color:#bce7ed;border-radius:10px;padding:12px 15px;font-size:11px;line-height:1.6;margin:10px 0}
  .activity-error{background:#3b252b;border-color:#924853;color:#fcc5ce}
  .activity-grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:14px;margin-top:15px}
  .activity-card{border:1px solid #324f60;border-radius:15px;overflow:hidden;background:#132334;box-shadow:0 6px 24px #0002;min-width:0}
  .activity-cover{height:145px;display:grid;place-items:center;position:relative;overflow:hidden;background:radial-gradient(circle at center,#27566e,#142637)}
  .activity-cover img{width:100%;height:100%;object-fit:cover}
  .activity-placeholder{font-size:60px;font-weight:300;color:#74c5d7;opacity:.8}
  .activity-origin{position:absolute;top:10px;left:10px;padding:6px 9px;background:#102938df;border:1px solid #468092;border-radius:30px;color:#b8eaf4;font-size:9px;font-weight:800}
  .activity-details{padding:14px}
  .activity-details>strong{display:block;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;font-size:12px;color:#e5f7fb}
  .activity-details>small{display:block;color:#88a6b8;font-size:10px;margin-top:7px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .activity-card-actions{display:flex;flex-wrap:wrap;gap:7px;margin-top:14px}
  .activity-card-actions .btn{font-size:10px;padding:7px 9px;min-height:32px}
  .activity-empty{border:1px dashed #375768;border-radius:16px;background:#101e2b;min-height:145px;display:flex;flex-direction:column;gap:8px;align-items:center;justify-content:center;text-align:center;padding:20px}
  .activity-empty>span{color:#87cad7;font-size:30px}
  .activity-empty strong{font-size:12px;color:#d6f0fa}
  .activity-empty p{max-width:490px;margin:0;color:#8ca9bd;font-size:11px;line-height:1.7}
  .activity-footnote{font-size:10px;color:#849fb0;line-height:1.7;margin:13px 0 0}
  @media(max-width:1200px){.activity-grid{grid-template-columns:repeat(3,minmax(0,1fr))}}
  @media(max-width:900px){.activity-grid{grid-template-columns:repeat(2,minmax(0,1fr))}}
  @media(max-width:560px){.activity-grid{grid-template-columns:1fr}.activity-intro{align-items:flex-start;flex-direction:column;gap:14px}.activity-cover{height:165px}}
</style>
