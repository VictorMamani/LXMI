import { useMemo, useState } from "react";
import type { ModLibrary } from "../../../api/types";
import { selectionDiff } from "../../../domain/mods";

/**
 * Profile being edited on the Mods page: which profile is open and the mods
 * ticked in it. Nothing is persisted until «Guardar y aplicar».
 */
export function useProfileDraft(
  library: ModLibrary | null,
  appliedProfile: string | null,
) {
  const initialName =
    library?.profiles.find((p) => p.name === appliedProfile)?.name ??
    library?.profiles[0]?.name ??
    null;
  // `null` means «the default profile» until the user picks one explicitly.
  const [chosenName, setChosenName] = useState<string | null>(null);
  const [draft, setDraft] = useState<string[] | null>(null);

  const name = chosenName ?? initialName;
  const saved = library?.profiles.find((p) => p.name === name) ?? null;
  const savedIds = useMemo(() => saved?.mod_ids ?? [], [saved]);
  const selected = draft ?? savedIds;
  const diff = useMemo(
    () => selectionDiff(savedIds, selected),
    [savedIds, selected],
  );

  return {
    name,
    /** The open profile exists in the library (it is not a new one). */
    isSaved: saved !== null,
    selected,
    diff,
    open(profileName: string, initialSelection?: string[]) {
      setChosenName(profileName);
      setDraft(initialSelection ?? null);
    },
    toggle(modId: string, checked: boolean) {
      setDraft(
        checked ? [...selected, modId] : selected.filter((id) => id !== modId),
      );
    },
    revert() {
      setDraft(saved ? null : []);
    },
    /** After saving, the library becomes the source of truth again. */
    commit() {
      setDraft(null);
    },
  };
}
