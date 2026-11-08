import type { PublicServer } from "./types";

export type ServerSort = "default" | "lowest-ping" | "most-players" | "fewest-players" | "most-slots";
export type ServerFilters = {
  sort: ServerSort;
  hideFull: boolean;
  maxPing: number | null;
  search: string;
};

export const SERVER_UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export function isValidServerId(value: unknown): value is string {
  return typeof value === "string" && SERVER_UUID_PATTERN.test(value);
}

// A live Roblox instance is ephemeral. Thirst intentionally does not persist
// favorite server UUIDs; bookmarks are better attached to games/subplaces.
export function filterAndSortServers(
  servers: PublicServer[],
  filters: ServerFilters
): PublicServer[] {
  const query = filters.search.trim().toLowerCase();
  const matches = servers.filter(server => {
    if (filters.hideFull && server.playing >= server.maxPlayers) return false;
    if (filters.maxPing !== null && (server.ping === null || !Number.isFinite(server.ping) ||
      server.ping > filters.maxPing)) return false;
    if (query && !server.id.toLowerCase().includes(query)) return false;
    return true;
  });
  if (filters.sort === "default") return matches;
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
