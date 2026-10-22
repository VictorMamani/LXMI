import type { ModDetails, ModKind, ModManifest } from "../api/types";

export function modSize(mod: ModManifest): number {
  return mod.files.reduce((total, file) => total + file.size, 0);
}

/**
 * The library does not store a character yet; by convention mod names follow
 * "Character — Variant", so the part before the dash is shown when present.
 */
export function characterOf(name: string): string | null {
  const match = /^(.+?)\s+[—–-]\s+.+$/.exec(name);
  return match ? match[1] : null;
}

/** Character detected in the mod's INI files, else the name convention. */
export function modCharacter(
  mod: ModManifest,
  details: ModDetails | undefined,
): string | null {
  return details?.character ?? characterOf(mod.name);
}

export type KindFilter = "all" | "model" | "texture" | "ui" | "other";

export const kindFilters: { id: KindFilter; label: string }[] = [
  { id: "all", label: "All" },
  { id: "model", label: "Models" },
  { id: "texture", label: "Textures" },
  { id: "ui", label: "UI" },
  { id: "other", label: "Other" },
];

export const kindLabel: Record<ModKind, string> = {
  model: "Model",
  texture: "Texture",
  ui: "UI",
  shader: "Shader",
  other: "Other",
};

/** Mods without details yet (still loading) appear only under "All". */
export function matchesKind(
  details: ModDetails | undefined,
  filter: KindFilter,
): boolean {
  if (filter === "all") return true;
  if (!details) return false;
  if (filter === "other")
    return details.kind === "shader" || details.kind === "other";
  return details.kind === filter;
}

/** Search matches the mod name or its detected character. */
export function matchesQuery(
  mod: ModManifest,
  details: ModDetails | undefined,
  query: string,
): boolean {
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) return true;
  return [mod.name, modCharacter(mod, details) ?? ""].some((text) =>
    text.toLocaleLowerCase().includes(needle),
  );
}

/** 3D models that still need a preview, in library order. */
export function pendingPreviews(
  mods: readonly ModManifest[],
  details: ReadonlyMap<string, ModDetails>,
): string[] {
  return mods
    .filter((mod) => {
      const entry = details.get(mod.id);
      return entry?.previewable && entry.preview === "missing";
    })
    .map((mod) => mod.id);
}

export type ModChange = "added" | "removed" | null;

/** Compare identities, not counts: swapping one mod must still need applying. */
export function sameModSelection(
  left: readonly string[],
  right: readonly string[],
) {
  const ids = new Set(left);
  return ids.size === new Set(right).size && right.every((id) => ids.has(id));
}

/** Difference between the saved profile and the selection being edited. */
export function selectionDiff(
  saved: readonly string[],
  draft: readonly string[],
) {
  const savedSet = new Set(saved);
  const draftSet = new Set(draft);
  const added = draft.filter((id) => !savedSet.has(id));
  const removed = saved.filter((id) => !draftSet.has(id));
  return {
    added,
    removed,
    count: added.length + removed.length,
    changeOf(id: string): ModChange {
      if (draftSet.has(id) && !savedSet.has(id)) return "added";
      if (savedSet.has(id) && !draftSet.has(id)) return "removed";
      return null;
    },
  };
}
