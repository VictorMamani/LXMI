export type GameId = "wuwa" | "zzz";

export type Game = {
  id: GameId;
  name: string;
  subtitle: string;
  art: string;
  runtime: string;
  runtimeSource: string;
  modManager: string;
  prefix: string;
  appId?: string;
};

export type MockMod = {
  id: string;
  gameId: GameId;
  name: string;
  author: string;
  character: string;
  version: string;
  source: string;
  enabled: boolean;
  artwork: string;
  conflict?: string;
};

export type MockProfile = {
  id: string;
  gameId: GameId;
  name: string;
  description: string;
  lastUsed: string;
  modIds: string[];
};

export const mockGames: Game[] = [
  {
    id: "wuwa",
    name: "Wuthering Waves",
    subtitle: "Un mundo que responde a tu ritmo.",
    art: "/artwork/wuwa-abstract.svg",
    runtime: "Proton Experimental",
    runtimeSource: "Valve",
    modManager: "WWMI · configurado",
    prefix: "Aún no inicializado",
    appId: "3513350",
  },
  {
    id: "zzz",
    name: "Zenless Zone Zero",
    subtitle: "La ciudad espera después del último tren.",
    art: "/artwork/zzz-abstract.svg",
    runtime: "GE-Proton 9-23",
    runtimeSource: "Community build",
    modManager: "ZZMI · configurado",
    prefix: "Disponible",
  },
];

export const initialMods: MockMod[] = [
  {
    id: "jiyan-vergil",
    gameId: "wuwa",
    name: "Jiyan — Vergil Yamato Reskin",
    author: "Morrow Studio",
    character: "Jiyan",
    version: "1.2",
    source: "Importación local",
    enabled: true,
    artwork: "jiyan",
  },
  {
    id: "yinlin-cyberwave",
    gameId: "wuwa",
    name: "Yinlin — Cyberwave Retexture",
    author: "Aether Lab",
    character: "Yinlin",
    version: "2.0",
    source: "Importación local",
    enabled: true,
    artwork: "yinlin",
    conflict: "Comparte un recurso de textura con Neon Strings.",
  },
  {
    id: "dark-hud",
    gameId: "wuwa",
    name: "Minimal Dark HUD",
    author: "Northstar",
    character: "Interfaz",
    version: "1.4",
    source: "Importación local",
    enabled: true,
    artwork: "hud",
  },
  {
    id: "chixia-crimson",
    gameId: "wuwa",
    name: "Chixia — Crimson Ranger",
    author: "Morrow Studio",
    character: "Chixia",
    version: "1.1",
    source: "Importación local",
    enabled: true,
    artwork: "chixia",
  },
  {
    id: "sanhua-glass",
    gameId: "wuwa",
    name: "Sanhua — Glass Petals",
    author: "Soft Signal",
    character: "Sanhua",
    version: "1.0",
    source: "Importación local",
    enabled: true,
    artwork: "sanhua",
  },
  {
    id: "resonance-halo",
    gameId: "wuwa",
    name: "Resonance Halo",
    author: "Faint Echo",
    character: "Efecto visual",
    version: "0.8",
    source: "Importación local",
    enabled: true,
    artwork: "halo",
  },
  {
    id: "neon-strings",
    gameId: "wuwa",
    name: "Neon Strings — Test Variant",
    author: "Aether Lab",
    character: "Yinlin",
    version: "0.6",
    source: "Importación local",
    enabled: false,
    artwork: "strings",
    conflict: "Comparte un recurso de textura con Cyberwave Retexture.",
  },
  {
    id: "anby-noir",
    gameId: "zzz",
    name: "Anby — Night Shift",
    author: "Afterhours",
    character: "Anby",
    version: "1.0",
    source: "Importación local",
    enabled: true,
    artwork: "anby",
  },
  {
    id: "city-noise",
    gameId: "zzz",
    name: "City Noise Reduction",
    author: "Lowlight",
    character: "Interfaz",
    version: "1.3",
    source: "Importación local",
    enabled: true,
    artwork: "city",
  },
];

export const initialProfiles: MockProfile[] = [
  {
    id: "default",
    gameId: "wuwa",
    name: "Default",
    description: "Tu selección habitual para recorrer Huanglong.",
    lastUsed: "Hoy · 19:42",
    modIds: [
      "jiyan-vergil",
      "yinlin-cyberwave",
      "dark-hud",
      "chixia-crimson",
      "sanhua-glass",
      "resonance-halo",
    ],
  },
  {
    id: "vanilla",
    gameId: "wuwa",
    name: "Vanilla",
    description: "El juego tal como viene, sin modificaciones activas.",
    lastUsed: "Ayer · 22:08",
    modIds: [],
  },
  {
    id: "screenshots",
    gameId: "wuwa",
    name: "Screenshots",
    description: "Una composición más limpia para capturas.",
    lastUsed: "12 sep · 16:21",
    modIds: ["jiyan-vergil", "sanhua-glass", "resonance-halo"],
  },
  {
    id: "custom",
    gameId: "wuwa",
    name: "Custom",
    description: "Un espacio para probar combinaciones propias.",
    lastUsed: "Nunca",
    modIds: ["yinlin-cyberwave", "dark-hud"],
  },
  {
    id: "new-eridu",
    gameId: "zzz",
    name: "Night Route",
    description: "Un conjunto ligero para salir de patrulla.",
    lastUsed: "Hoy · 18:05",
    modIds: ["anby-noir", "city-noise"],
  },
];

export const mockRuntimes = [
  {
    id: "proton-experimental",
    name: "Proton Experimental",
    source: "Valve",
    version: "bleeding-edge",
    selected: true,
    kind: "Proton",
  },
  {
    id: "ge-proton-9-23",
    name: "GE-Proton 9-23",
    source: "Custom",
    version: "9-23",
    selected: false,
    kind: "Proton",
  },
  {
    id: "proton-10",
    name: "Proton 10",
    source: "Valve",
    version: "10.0-2",
    selected: false,
    kind: "Proton",
  },
];
