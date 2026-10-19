import type { BackendGameId, PackageKind } from "../api/types";
import wuwaBackground from "../media/games/wuwa/background.webm";
import wuwaCard from "../media/games/wuwa/cover-card.webp";
import wuwaPoster from "../media/games/wuwa/poster.webp";
import zzzBackground from "../media/games/zzz/background.webm";
import zzzCard from "../media/games/zzz/cover-card.webp";
import zzzPoster from "../media/games/zzz/poster.webp";

export type GameId = "zzz" | "wuwa";

/**
 * A game "space". Each one is a cartridge/channel (CH·01, CH·02…); its art is
 * the only per-game colour in the interface.
 */
export type Game = {
  id: GameId;
  backendId: BackendGameId;
  channel: string;
  name: string;
  /** Game integration package of XXMI (ZZMI, WWMI…). */
  importer: string;
  importerKind: PackageKind;
  media: { background: string; poster: string; card: string };
};

export const games: readonly Game[] = [
  {
    id: "zzz",
    backendId: "zenless-zone-zero",
    channel: "CH·01",
    name: "Zenless Zone Zero",
    importer: "ZZMI",
    importerKind: "zzmi",
    media: { background: zzzBackground, poster: zzzPoster, card: zzzCard },
  },
  {
    id: "wuwa",
    backendId: "wuthering-waves",
    channel: "CH·02",
    name: "Wuthering Waves",
    importer: "WWMI",
    importerKind: "wwmi",
    media: { background: wuwaBackground, poster: wuwaPoster, card: wuwaCard },
  },
];

/** Channel shown for the free slot that follows the last game. */
export const nextFreeChannel = `CH·${String(games.length + 1).padStart(2, "0")}`;

export function gameById(id: GameId): Game {
  return games.find((game) => game.id === id) ?? games[0];
}

/** Cyclic neighbour, used by the Q / E shortcuts. */
export function adjacentGame(id: GameId, step: 1 | -1): Game {
  const index = games.findIndex((game) => game.id === id);
  return games[(index + step + games.length) % games.length];
}
