import type { GameMetadata, SavedGame } from "./types";
import { isTrustedIconUrl, validPlaceId } from "./library";
import { isValidServerId } from "./servers";

export type PlayerSession = {
  placeId: number;
  serverId: string;
  joinedAt: number;
  connectedAt: number | null;
  endedAt: number | null;
};

export type ActivityEntry = {
  placeId: number;
  name: string;
  iconUrl?: string;
  serverId: string | null;
  joinedAt: number;
  // Optional for activity entries persisted before connection tracking.
  connectedAt?: number | null;
  endedAt: number | null;
  source: "player-log" | "launch-request";
};

const SETTINGS = "thirst.activity.tracking.v1";
const SINCE = "thirst.activity.since.v1";
const HISTORY = "thirst.activity.history.v1";
const KEEP_FOR = 30 * 24 * 60 * 60_000;
const MAX_ENTRIES = 80;
const SEVEN_DAYS = 7 * 24 * 60 * 60_000;

function cleanTime(value: unknown, now: number): value is number {
  return typeof value === "number" && Number.isSafeInteger(value) &&
    value > now - KEEP_FOR && value <= now + 60_000;
}

function knownName(games: SavedGame[], placeId: number): string {
  for (const game of games) {
    if (game.placeId === placeId) return game.name;
    const subplace = game.subplaces.find(item => item.placeId === placeId);
    if (subplace) return subplace.name + " · " + game.name;
  }
  return "Place " + placeId;
}

export function activityIcon(games: SavedGame[], placeId: number): string | null {
  const game = games.find(g => g.placeId === placeId ||
    g.subplaces.some(s => s.placeId === placeId));
  return isTrustedIconUrl(game?.iconUrl) ? game!.iconUrl! : null;
}

export function historyEnabled(): boolean {
  try { return localStorage.getItem(SETTINGS) === "on"; }
  catch { return false; }
}

// This opt-in timestamp remains stable even after seven days. The rolling
// history scan limit is NOT suitable for invalidating an in-flight scan.
export function trackingEpoch(): number {
  const now = Date.now();
  try {
    const stored = localStorage.getItem(SINCE);
    const parsed = stored === null ? NaN : Number(stored);
    if (Number.isSafeInteger(parsed) && parsed > 0 && parsed <= now) return parsed;
  } catch { /* no persisted setting */ }
  return now;
}

export function trackingSince(): number {
  return Math.max(trackingEpoch(), Date.now() - SEVEN_DAYS + 60_000);
}

export function setHistoryEnabled(enabled: boolean): void {
  // Re-enabling starts a new observation window. No older activity is
  // imported without explicit user consent.
  if (enabled && !historyEnabled()) localStorage.setItem(SINCE, String(Date.now()));
  localStorage.setItem(SETTINGS, enabled ? "on" : "off");
}

export function readActivityHistory(): ActivityEntry[] {
  const now = Date.now();
  try {
    const data: unknown = JSON.parse(localStorage.getItem(HISTORY) || "[]");
    if (!Array.isArray(data)) return [];
    return data.filter((item): item is ActivityEntry => {
      if (!item || typeof item !== "object") return false;
      const e = item as ActivityEntry;
      return validPlaceId(e.placeId) &&
        typeof e.name === "string" && e.name.length > 0 && e.name.length <= 120 &&
        (e.iconUrl === undefined || isTrustedIconUrl(e.iconUrl)) &&
        (e.serverId === null || isValidServerId(e.serverId)) &&
        cleanTime(e.joinedAt, now) &&
        (e.connectedAt === undefined || e.connectedAt === null ||
          (cleanTime(e.connectedAt, now) && e.connectedAt >= e.joinedAt)) &&
        (e.endedAt === null || (cleanTime(e.endedAt, now) && e.endedAt >= e.joinedAt)) &&
        (e.connectedAt === undefined || e.connectedAt === null || e.endedAt === null ||
          e.connectedAt <= e.endedAt) &&
        (e.source === "player-log" || e.source === "launch-request");
    }).sort((a, b) => b.joinedAt - a.joinedAt)
      .slice(0, MAX_ENTRIES);
  } catch { return []; }
}

function persist(entries: ActivityEntry[]): ActivityEntry[] {
  const bounded = entries.sort((a, b) => b.joinedAt - a.joinedAt)
    .slice(0, MAX_ENTRIES);
  localStorage.setItem(HISTORY, JSON.stringify(bounded));
  return bounded;
}

export function recordLaunchAttempt(
  entries: ActivityEntry[], placeId: number, games: SavedGame[],
  serverId: string | null = null, at = Date.now()
): ActivityEntry[] {
  if (!validPlaceId(placeId) || (serverId && !isValidServerId(serverId))) return entries;
  const attempt: ActivityEntry = {
    placeId, name: knownName(games, placeId),
    iconUrl: activityIcon(games, placeId) ?? undefined,
    serverId,
    joinedAt: at, connectedAt: null, endedAt: null, source: "launch-request"
  };
  return persist([attempt, ...entries]);
}

export function mergeObservedSessions(
  history: ActivityEntry[], observations: PlayerSession[],
  games: SavedGame[], since: number, now = Date.now()
): ActivityEntry[] {
  const result = [...history];
  for (const obs of observations) {
    if (!validPlaceId(obs.placeId) || !isValidServerId(obs.serverId) ||
      !cleanTime(obs.joinedAt, now) || obs.joinedAt < since ||
      (obs.connectedAt !== null && (!cleanTime(obs.connectedAt, now) ||
        obs.connectedAt < obs.joinedAt)) ||
      (obs.endedAt !== null && (!cleanTime(obs.endedAt, now) ||
        obs.endedAt < obs.joinedAt)) ||
      (obs.connectedAt !== null && obs.endedAt !== null &&
        obs.connectedAt > obs.endedAt)) continue;

    const existing = result.find(e => e.source === "player-log" &&
      e.placeId === obs.placeId && e.serverId === obs.serverId &&
      e.joinedAt === obs.joinedAt);
    if (existing) {
      // A newer truncated log tail can omit an old event, but it should
      // not erase a connection or disconnect we previously observed.
      if (obs.connectedAt !== null) existing.connectedAt = obs.connectedAt;
      if (obs.endedAt !== null) existing.endedAt = obs.endedAt;
      continue;
    }
    // Replace a launch request with the actual observed Player log event
    // when both refer to the same app-initiated join attempt.
    const matchedAttempt = result.findIndex(e => e.source === "launch-request" &&
      e.placeId === obs.placeId &&
      (e.serverId === null || e.serverId === obs.serverId) &&
      Math.abs(e.joinedAt - obs.joinedAt) <= 90_000);
    let name = knownName(games, obs.placeId);
    if (matchedAttempt >= 0) {
      const attempt = result.splice(matchedAttempt, 1)[0];
      if (name.startsWith("Place ")) name = attempt.name;
    }
    result.push({
      placeId: obs.placeId, name,
      iconUrl: activityIcon(games, obs.placeId) ?? undefined,
      serverId: obs.serverId,
      joinedAt: obs.joinedAt, connectedAt: obs.connectedAt,
      endedAt: obs.endedAt, source: "player-log"
    });
  }
  return persist(result);
}

export function addActivityMetadata(
  entries: ActivityEntry[], placeId: number, metadata: GameMetadata
): ActivityEntry[] {
  if (!validPlaceId(placeId)) return entries;
  const name = metadata.name.trim().slice(0, 100);
  const iconUrl = isTrustedIconUrl(metadata.iconUrl) ? metadata.iconUrl : undefined;
  if (!name) return entries;
  return persist(entries.map(entry => entry.placeId === placeId
    ? { ...entry, name, iconUrl: iconUrl ?? entry.iconUrl }
    : entry));
}

export function clearActivityHistory(): ActivityEntry[] {
  localStorage.removeItem(HISTORY);
  if (historyEnabled()) localStorage.setItem(SINCE, String(Date.now()));
  return [];
}

export function shortActivityTime(at: number, now = Date.now()): string {
  const age = Math.max(0, now - at);
  if (age < 60_000) return "Just now";
  if (age < 60 * 60_000) return Math.floor(age / 60_000) + "m ago";
  if (age < 24 * 60 * 60_000) return Math.floor(age / 3_600_000) + "h ago";
  return Math.floor(age / 86_400_000) + "d ago";
}

export function activityDuration(entry: ActivityEntry): string | null {
  // Playtime only starts after a network connection is observed.
  if (entry.endedAt === null || entry.connectedAt == null ||
    entry.source !== "player-log" || entry.endedAt < entry.connectedAt) return null;
  const minutes = Math.max(0, Math.floor((entry.endedAt - entry.connectedAt) / 60_000));
  return minutes >= 60 ? Math.floor(minutes / 60) + "h " + minutes % 60 + "m" : minutes + "m";
}
