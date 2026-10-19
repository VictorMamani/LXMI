import type { ModManifest } from "../api/types";

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
