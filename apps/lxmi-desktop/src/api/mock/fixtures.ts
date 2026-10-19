import type { ModLibrary, ModManifest, XxmiPackage } from "../types";

const MB = 1024 * 1024;

function mod(
  id: string,
  name: string,
  megabytes: number,
  files = 3,
): ModManifest {
  const size = Math.round((megabytes * MB) / files);
  return {
    id,
    name,
    files: Array.from({ length: files }, (_, index) => ({
      relative_path: `${id}/part-${index}.buf`,
      size,
      sha256: `${id}${index}`.padEnd(64, "0"),
    })),
  };
}

export function initialLibraries(): Record<string, ModLibrary> {
  return {
    "zenless-zone-zero": {
      game_id: "zenless-zone-zero",
      revision: 4,
      issues: [],
      mods: [
        mod("ellen-verano", "Ellen — Uniforme de verano", 48),
        mod("anby-tactica", "Anby — Chaqueta táctica", 31),
        mod("nicole-gafas", "Nicole — Gafas retro", 6, 2),
        mod("hud-min", "HUD minimalista", 2, 1),
        mod("miyabi-kimono", "Miyabi — Kimono nocturno", 62),
        mod("bangboo-neon", "Bangboo — Pintura neón", 9, 2),
        mod("billy-gabardina", "Billy — Gabardina", 27),
      ],
      profiles: [
        {
          name: "Combate",
          mod_ids: ["ellen-verano", "anby-tactica", "hud-min", "miyabi-kimono"],
        },
        { name: "Capturas", mod_ids: ["hud-min"] },
      ],
    },
    "wuthering-waves": {
      game_id: "wuthering-waves",
      revision: 2,
      issues: [],
      mods: [
        mod("jinhsi-festivo", "Jinhsi — Atuendo festivo", 54),
        mod("changli-capa", "Changli — Capa roja", 38),
        mod("rover-abrigo", "Rover — Abrigo largo", 22),
        mod("ui-limpia", "UI limpia", 3, 1),
        mod("camellya-vestido", "Camellya — Vestido negro", 47),
        mod("yinlin-coleta", "Yinlin — Coleta corta", 12, 2),
        mod("calcharo-mascara", "Calcharo — Sin máscara", 19),
      ],
      profiles: [
        {
          name: "Exploración",
          mod_ids: [
            "jinhsi-festivo",
            "changli-capa",
            "rover-abrigo",
            "ui-limpia",
          ],
        },
      ],
    },
  };
}

function pkg(
  id: string,
  kind: XxmiPackage["kind"],
  version: string,
): XxmiPackage {
  return {
    id,
    kind,
    imported_unix_seconds: 1_760_000_000,
    version: { raw: version },
    files: [],
    authenticity: "official_release_verified",
  };
}

export function initialPackages(): XxmiPackage[] {
  return [
    pkg("zzmi-2.4.1-7f3a", { game_integration: "zzmi" }, "2.4.1"),
    pkg("wwmi-0.8.2-1c9e", { game_integration: "wwmi" }, "0.8.2"),
    pkg("libs-0.9.3-a41d", "xxmi_libraries", "0.9.3"),
  ];
}

export const latestReleases = [
  { package_kind: "zzmi", tag: "v2.5.0", version: "2.5.0" },
  { package_kind: "wwmi", tag: "v0.8.2", version: "0.8.2" },
  { package_kind: "xxmi_libraries", tag: "v0.9.3", version: "0.9.3" },
].map((release) => ({
  ...release,
  published_at: "2026-10-10T00:00:00Z",
  release_url: "https://github.com/SpectrumQT",
}));
