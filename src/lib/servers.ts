import type { PublicServer } from "./types";

export type ServerSort = "default" | "lowest-ping" | "most-players" | "fewest-players" | "most-slots";
export type ServerFilters = {
  sort: ServerSort;
  hideFull: boolean;
  maxPing: number | null;
  favoritesOnly: boolean;
  search: string;
};
export type FavoriteServer = {
  placeId: number;
  serverId: string;
  addedAt: number;
};

// Server UUIDs are public Roblox job IDs, never Roblox authorization tokens.
const STORAGE_KEY = "thirst.favorite-servers.v1";
const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const MAX_FAVORITES = 120;

export function isValidServerId(value: unknown): value is string {
  return typeof value === "string" && UUID_RE.test(value);
}

export function favoriteKey(placeId: number, serverId: string): string {
  return `${placeId}:${serverId.toLowerCase()}`;
}

export function readFavoriteServers(): FavoriteServer[] {
  try {
    const raw: unknown = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "[]");
    if (!Array.isArray(raw)) return [];
    const entries: FavoriteServer[] = [];
    const seen = new Set<string>();
    for (const value of raw.slice(0, MAX_FAVORITES)) {
      if (!value || typeof value !== "object") continue;
      const candidate = value as FavoriteServer;
      if (!Number.isSafeInteger(candidate.placeId) || candidate.placeId <= 0 ||
          !isValidServerId(candidate.serverId)) continue;
      const key = favoriteKey(candidate.placeId, candidate.serverId);
      if (seen.has(key)) continue;
      seen.add(key);
      entries.push({
        placeId: candidate.placeId,
        serverId: candidate.serverId.toLowerCase(),
        addedAt: Number.isFinite(candidate.addedAt) ? candidate.addedAt : Date.now()
      });
    }
    return entries;
  } catch {
    return [];
  }
}

export function writeFavoriteServers(favorites: FavoriteServer[]): void {
  // Never place cookies, auth tickets or CSRF tokens into this store.
  localStorage.setItem(STORAGE_KEY, JSON.stringify(favorites.slice(0, MAX_FAVORITES)));
}

export function filterAndSortServers(
  servers: PublicServer[],
  filters: ServerFilters,
  favorites: FavoriteServer[],
  placeId: number | null
): PublicServer[] {
  const known = new Set(favorites.map(item => favoriteKey(item.placeId, item.serverId)));
  const query = filters.search.trim().toLowerCase();
  const matches = servers.filter(server => {
    if (filters.hideFull && server.playing >= server.maxPlayers) return false;
    if (filters.maxPing !== null && (server.ping === null || !Number.isFinite(server.ping) ||
      server.ping > filters.maxPing)) return false;
    if (query && !server.id.toLowerCase().includes(query)) return false;
    if (filters.favoritesOnly && (placeId === null ||
      !known.has(favoriteKey(placeId, server.id)))) return false;
    return true;
  });
  if (filters.sort === "default") return matches;
  // Stable tie-breaking reduces distracting row reshuffles.
  return [...matches].sort((a, b) => {
    let diff = 0;
    if (filters.sort === "lowest-ping") diff =
      (a.ping !== null && Number.isFinite(a.ping) ? a.ping : Infinity) -
      (b.ping !== null && Number.isFinite(b.ping) ? b.ping : Infinity);
    if (filters.sort === "most-players") diff = b.playing - a.playing;
    if (filters.sort === "fewest-players") diff = a.playing - b.playing;
    if (filters.sort === "most-slots") diff =
      (b.maxPlayers - b.playing) - (a.maxPlayers - a.playing);
    return Number.isNaN(diff) || diff === 0 ? a.id.localeCompare(b.id) : diff;
  });
}
