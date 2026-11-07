export type SavedPlace = {
  placeId: number;
  name: string;
};

export type SavedGame = {
  placeId: number;
  name: string;
  subplaces: SavedPlace[];
  addedAt: number;
  universeId?: number;
  rootPlaceId?: number;
  iconUrl?: string;
};

export type GameMetadata = {
  universeId: number;
  rootPlaceId: number;
  name: string;
  playing: number | null;
  iconUrl: string | null;
};

export type DiscoveredPlace = {
  id: number;
  name: string;
};

export type SubplacesPage = {
  data: DiscoveredPlace[];
  nextPageCursor: string | null;
};

export type PublicServer = {
  id: string;
  playing: number;
  maxPlayers: number;
  ping: number | null;
  fps: number | null;
};

export type ServerPage = {
  data: PublicServer[];
  nextPageCursor: string | null;
};
