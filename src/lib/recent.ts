import type { SavedGame } from "./types";

export type RecentLaunch = {
  placeId:number;
  name:string;
  openedAt:number;
};

const STORAGE = "thirst.recent-launches.v1";
const MAX_ITEMS = 12;

export function readRecentLaunches():RecentLaunch[] {
  try {
    const data:unknown = JSON.parse(localStorage.getItem(STORAGE) || "[]");
    if (!Array.isArray(data)) return [];
    const seen = new Set<number>();
    return data
      .filter((item):item is RecentLaunch =>
        item !== null && typeof item === "object" &&
        Number.isSafeInteger(item.placeId) && item.placeId > 0 &&
        typeof item.name === "string" && item.name.length <= 100 &&
        Number.isFinite(item.openedAt) && item.openedAt > 0 &&
        item.openedAt <= Date.now() + 60_000
      )
      .sort((a,b) => b.openedAt - a.openedAt)
      .filter(item => {
        if(seen.has(item.placeId)) return false;
        seen.add(item.placeId);
        return true;
      }).slice(0,MAX_ITEMS);
  } catch {
    return [];
  }
}

export function recordRecentLaunch(placeId:number,name:string):RecentLaunch[] {
  if(!Number.isSafeInteger(placeId) || placeId <= 0) return readRecentLaunches();
  const item:RecentLaunch = {
    placeId,
    name:name.trim().slice(0,100) || "Place " + placeId,
    openedAt:Date.now()
  };
  const updated = [item,...readRecentLaunches().filter(r=>r.placeId!==placeId)].slice(0,MAX_ITEMS);
  localStorage.setItem(STORAGE,JSON.stringify(updated));
  return updated;
}

export function clearRecentLaunches():RecentLaunch[] {
  localStorage.removeItem(STORAGE);
  return [];
}

export function recentPlaceName(games:SavedGame[],placeId:number):string {
  for(const game of games) {
    if(game.placeId===placeId) return game.name;
    const subplace=game.subplaces.find(p=>p.placeId===placeId);
    if(subplace) return subplace.name + " · " + game.name;
  }
  return "Place " + placeId;
}
