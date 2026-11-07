import { invoke } from "@tauri-apps/api/core";
import type { GameMetadata, ServerPage, SubplacesPage } from "./types";

export function launchPlace(placeId: number, jobId: string | null = null): Promise<void> {
  return invoke<void>("launch_place", { placeId, jobId });
}

export function fetchPublicServers(placeId: number, cursor: string | null): Promise<ServerPage> {
  return invoke<ServerPage>("list_public_servers", { placeId, cursor });
}

export function lookupGame(placeId: number): Promise<GameMetadata> {
  return invoke<GameMetadata>("lookup_game", { placeId });
}

export function discoverSubplaces(universeId: number, cursor: string | null): Promise<SubplacesPage> {
  return invoke<SubplacesPage>("discover_subplaces", { universeId, cursor });
}

export function platformName(): Promise<string> {
  return invoke<string>("platform_name");
}
