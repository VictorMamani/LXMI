export type GameId = "wuwa" | "zzz";
export type Game = {
  id: GameId;
  backendId: string;
  name: string;
  /** Original LXMI illustration; never official game artwork. */
  art: string;
  short: string;
  importer: string;
  subtitle: string;
};
export const games: Game[] = [
  {
    id: "wuwa",
    backendId: "wuthering-waves",
    name: "Wuthering Waves",
    art: "/artwork/wuwa-launcher.svg",
    short: "WW",
    importer: "WWMI",
    subtitle: "Tu biblioteca. Tu forma de jugar.",
  },
  {
    id: "zzz",
    backendId: "zenless-zone-zero",
    name: "Zenless Zone Zero",
    art: "/artwork/zzz-launcher.svg",
    short: "ZZZ",
    importer: "ZZMI",
    subtitle: "Tu colección en Nueva Eridu.",
  },
];
