import type { SavedGame } from "./types";
import { isTrustedIconUrl, validPlaceId } from "./library";

// Library JSON only. No sessions, cookies, browser profiles or private data.
export function exportLibraryJson(games:SavedGame[]):string {
  return JSON.stringify({format:"thirst-library-v1",games},null,2);
}

export function importLibraryJson(json:string):SavedGame[] {
  if(json.length>1_000_000) throw Error("Library file exceeds 1 MB.");
  let data:unknown;
  try { data=JSON.parse(json); } catch {throw Error("Library file is not valid JSON.");}
  if(!data || typeof data!=="object") throw Error("Library file must be a JSON object.");
  const bundle=data as {format?:unknown;games?:unknown};
  if(bundle.format!=="thirst-library-v1" || !Array.isArray(bundle.games)) {
    throw Error("This file is not a Thirst library backup.");
  }
  const seen = new Set<number>();
  const output:SavedGame[]=[];
  for(const raw of bundle.games.slice(0,200)) {
    if(!raw || typeof raw!=="object") continue;
    const game = raw as Partial<SavedGame>;
    if(!validPlaceId(game.placeId ?? 0) || typeof game.name!=="string" ||
       game.name.length<1 || game.name.length>100 || !Array.isArray(game.subplaces) ||
       seen.has(game.placeId!)) continue;
    const placeId=game.placeId!;
    seen.add(placeId);
    const seenSub = new Set<number>();
    const subplaces=game.subplaces.slice(0,100).filter(place => {
      if(!place || !validPlaceId(place.placeId) ||
         typeof place.name!=="string" || place.name.length<1 ||
         place.name.length>100 || seenSub.has(place.placeId)) return false;
      seenSub.add(place.placeId);
      return true;
    }).map(place=>({
      placeId:place.placeId,name:place.name,pinned:place.pinned===true
    }));
    output.push({
      placeId,name:game.name,
      addedAt:typeof game.addedAt === "number" && Number.isFinite(game.addedAt) && game.addedAt > 0 ? game.addedAt : Date.now(),
      subplaces,
      universeId:validPlaceId(game.universeId??0) ? game.universeId:undefined,
      rootPlaceId:validPlaceId(game.rootPlaceId??0)?game.rootPlaceId:undefined,
      iconUrl:isTrustedIconUrl(game.iconUrl)?game.iconUrl:undefined
    });
  }
  if(!output.length) throw Error("No valid saved experiences found in this backup.");
  return output;
}

export function mergeImportedLibrary(existing:SavedGame[],incoming:SavedGame[]):SavedGame[] {
  const byId = new Map(existing.map((g,i)=>[g.placeId,i]));
  const result=existing.map(g=>({...g,subplaces:[...g.subplaces]}));
  for(const game of incoming) {
    const index=byId.get(game.placeId);
    if(index===undefined) {
      if(result.length>=200) continue;
      byId.set(game.placeId,result.length);
      result.push({...game,subplaces:[...game.subplaces]});
      continue;
    }
    // Merging preserves existing custom names and choices. New subplaces and
    // pins from backups can be added without deleting anything from this PC.
    const saved=result[index];
    const archivedPins=new Set(game.subplaces.filter(p=>p.pinned).map(p=>p.placeId));
    // Existing custom subplace names are retained, but archived pins are
    // restored for matching places instead of silently being discarded.
    const current=saved.subplaces.map(p=>({
      ...p,
      pinned:p.pinned===true || archivedPins.has(p.placeId)
    }));
    const seen=new Set(current.map(p=>p.placeId));
    const added=game.subplaces.filter(p=>!seen.has(p.placeId))
      .slice(0,Math.max(0,100-current.length));
    result[index]={...saved,subplaces:[...current,...added]};
  }
  return result;
}
