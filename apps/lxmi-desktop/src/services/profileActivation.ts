import { call } from "../api/client";
import type { ModLibrary, RuntimeSources, RuntimeStatus } from "../api/types";

export type ApplyResult =
  | { applied: true }
  /** Saved, but there is no assembled runtime to apply it to yet. */
  | { applied: false; reason: string };

/**
 * Selects a saved profile for the next launch. Reviewing and activating are
 * one step for the user: the review's plan is accepted right away.
 */
export async function applyProfile(params: {
  gameId: string;
  profileName: string;
  profileRevision: number;
  status: RuntimeStatus;
  sources: RuntimeSources;
}): Promise<ApplyResult> {
  const { gameId, profileName, profileRevision, status, sources } = params;
  if (!status.base_runtime || !sources.importer || !sources.libraries)
    return {
      applied: false,
      reason: "Prepare the mod runtime in Spaces to apply the profile.",
    };
  const runtime = {
    gameId,
    importerId: sources.importer,
    librariesId: sources.libraries,
  };
  const plan = await call("review_mod_profile_activation", {
    ...runtime,
    profileName,
    profileRevision,
    selectionRevision: status.selection.revision,
  });
  await call("activate_mod_profile", {
    ...runtime,
    profileName: plan.profile_name,
    profileRevision: plan.profile_revision,
    selectionRevision: plan.selection_revision,
    planId: plan.plan_id,
  });
  return { applied: true };
}

/** «Save and apply»: saves the edited selection, then applies it. */
export async function saveAndApplyProfile(params: {
  gameId: string;
  library: ModLibrary;
  profileName: string;
  modIds: string[];
  changed: boolean;
  status: RuntimeStatus;
  sources: RuntimeSources;
}): Promise<ApplyResult> {
  const { gameId, library, profileName, modIds, changed } = params;
  const revision = changed
    ? (
        await call("save_mod_profile", {
          gameId,
          name: profileName,
          modIds,
          expectedRevision: library.revision,
        })
      ).revision
    : library.revision;
  return applyProfile({ ...params, profileRevision: revision });
}
