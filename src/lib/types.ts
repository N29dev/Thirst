export type SavedPlace = {
  placeId: number;
  name: string;
};

export type SavedGame = {
  placeId: number;
  name: string;
  subplaces: SavedPlace[];
  addedAt: number;
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
