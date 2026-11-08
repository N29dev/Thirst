import { invoke } from "@tauri-apps/api/core";
import type { GameMetadata, ServerPage, SubplacesPage } from "./types";

export function launchPlace(placeId: number, jobId: string | null = null): Promise<void> {
  return invoke<void>("launch_place", { placeId, jobId });
}

export function fetchPublicServers(placeId: number, cursor: string | null, authenticated = false): Promise<ServerPage> {
  return invoke<ServerPage>("list_public_servers", { placeId, cursor, authenticated });
}

// Anonymous player-token requests, optionally using an OS-stored session in native Rust. The cookie never crosses IPC.
export function fetchServerAvatarPreview(placeId: number, serverId: string, authenticated = false): Promise<string[]> {
  return invoke<string[]>("server_avatar_preview", { placeId, serverId, authenticated });
}

// One bounded request for up to eight servers' anonymous avatar images.
export function fetchServerAvatarPreviews(placeId: number, serverIds: string[], authenticated = false): Promise<Record<string, string[]>> {
  return invoke<Record<string, string[]>>("server_avatar_previews", { placeId, serverIds, authenticated });
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

export type ConnectedAccount = {
  id: number;
  name: string;
  displayName: string;
};

// Authentication remains native-only. No function below returns a session cookie.
export function beginRobloxConsent(): Promise<void> {
  return invoke<void>("begin_roblox_consent");
}

export function openRobloxLogin(acknowledged: boolean): Promise<void> {
  return invoke<void>("open_roblox_login", { acknowledged });
}

// Reloads only the already-open, origin-restricted native Roblox webview.
export function reloadRobloxLogin(): Promise<void> {
  return invoke<void>("reload_roblox_login");
}

// Only a status string; the actual cookie never crosses IPC.
export function pollRobloxLoginStatus(): Promise<"waiting" | "ready" | "cancelled" | "not-started" | "manual"> {
  return invoke("roblox_login_status");
}

export function finishRobloxLogin(): Promise<ConnectedAccount> {
  return invoke<ConnectedAccount>("finish_roblox_login");
}

export function connectedRobloxAccount(): Promise<ConnectedAccount | null> {
  return invoke<ConnectedAccount | null>("connected_roblox_account");
}

export function disconnectRobloxAccount(): Promise<void> {
  return invoke<void>("disconnect_roblox_account");
}

export function cancelRobloxLogin(): Promise<void> {
  return invoke<void>("cancel_roblox_login");
}
