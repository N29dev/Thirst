import type { SavedGame } from "./types";

const STORAGE_KEY = "thirst.library.v1";
const SETTINGS_KEY = "thirst.preferences.v1";

// Only public game metadata/preferences belong here. NEVER write tokens or cookies.
export function validPlaceId(id: number): boolean {
  return Number.isSafeInteger(id) && id > 0;
}

export function isTrustedIconUrl(value: unknown): value is string {
  if (typeof value !== "string" || value.length > 1200) return false;
  try {
    const url = new URL(value);
    return url.protocol === "https:" &&
      (url.hostname === "rbxcdn.com" || url.hostname.endsWith(".rbxcdn.com")) &&
      !url.username && !url.password && url.port === "";
  } catch {
    return false;
  }
}

export function readLibrary(): SavedGame[] {
  try {
    const data: unknown = JSON.parse(localStorage.getItem(STORAGE_KEY) || "[]");
    if (!Array.isArray(data)) return [];
    return data.slice(0, 200).filter((item): item is SavedGame => {
      if (!item || typeof item !== "object") return false;
      const game = item as SavedGame;
      return validPlaceId(game.placeId) && typeof game.name === "string" &&
        game.name.length <= 100 && Array.isArray(game.subplaces);
    }).map(game => ({
      placeId: game.placeId,
      name: game.name,
      addedAt: Number.isFinite(game.addedAt) ? game.addedAt : Date.now(),
      universeId: validPlaceId(game.universeId ?? 0) ? game.universeId : undefined,
      rootPlaceId: validPlaceId(game.rootPlaceId ?? 0) ? game.rootPlaceId : undefined,
      iconUrl: isTrustedIconUrl(game.iconUrl) ? game.iconUrl : undefined,
      subplaces: game.subplaces.slice(0, 100).filter(
        place => validPlaceId(place.placeId) &&
          typeof place.name === "string" && place.name.length <= 100
      ).map(place => ({
        placeId: place.placeId,
        name: place.name,
        pinned: place.pinned === true
      }))
    }));
  } catch {
    return [];
  }
}

export function writeLibrary(games: SavedGame[]): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(games));
}

export function readLiteMode(): boolean {
  return localStorage.getItem(SETTINGS_KEY) === "lite";
}

export function writeLiteMode(enabled: boolean): void {
  localStorage.setItem(SETTINGS_KEY, enabled ? "lite" : "full");
}

// This is a non-sensitive UI preference. No account ID or cookie is stored.
const AUTO_AVATAR_KEY = "thirst.preferences.auto-avatars.v1";

export function readAutoAvatarPreviews(): boolean {
  return localStorage.getItem(AUTO_AVATAR_KEY) !== "off";
}

export function writeAutoAvatarPreviews(enabled: boolean): void {
  localStorage.setItem(AUTO_AVATAR_KEY, enabled ? "on" : "off");
}

// Non-sensitive UI preference, independent of the order in which bookmarks
// are stored. The default is the existing saved order.
const LIBRARY_SORT_KEY = "thirst.preferences.library-sort.v1";
export type LibrarySort = "saved" | "name" | "newest" | "oldest";

export function readLibrarySort(): LibrarySort {
  const value = localStorage.getItem(LIBRARY_SORT_KEY);
  return value === "name" || value === "newest" || value === "oldest" ? value : "saved";
}

export function writeLibrarySort(value: LibrarySort): void {
  localStorage.setItem(LIBRARY_SORT_KEY, value);
}

export function parsePlaceId(text: string): number | null {
  const raw = text.trim();
  const match = raw.match(/^https:\/\/(?:www\.)?roblox\.com\/games\/(\d+)(?:[/?#].*)?$/i);
  const value = /^\d+$/.test(raw) ? raw : match?.[1];
  if (!value) return null;
  const id = Number(value);
  return validPlaceId(id) ? id : null;
}
