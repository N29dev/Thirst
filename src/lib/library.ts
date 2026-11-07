import type { SavedGame } from "./types";

const STORAGE_KEY = "thirst.library.v1";
const SETTINGS_KEY = "thirst.preferences.v1";

// This store is exclusively for non-sensitive UI preferences and public place IDs.
// Authentication credentials MUST NEVER be added here.
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
      subplaces: game.subplaces.slice(0, 100).filter(
        place => validPlaceId(place.placeId) &&
          typeof place.name === "string" && place.name.length <= 100
      )
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

export function validPlaceId(id: number): boolean {
  return Number.isSafeInteger(id) && id > 0;
}

export function parsePlaceId(text: string): number | null {
  // Accept a numeric place ID or an official Roblox game URL.
  const raw = text.trim();
  const match = raw.match(/^https:\/\/(?:www\.)?roblox\.com\/games\/(\d+)(?:[/?#].*)?$/i);
  const value = /^\d+$/.test(raw) ? raw : match?.[1];
  if (!value) return null;
  const id = Number(value);
  return validPlaceId(id) ? id : null;
}
