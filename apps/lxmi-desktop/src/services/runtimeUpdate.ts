import { call } from "../api/client";
import type {
  OfficialRelease,
  RuntimeSources,
  RuntimeStatus,
} from "../api/types";
import type { PendingUpdate } from "../domain/runtime";
import { applyProfile } from "./profileActivation";

let releasesCache: Promise<OfficialRelease[]> | null = null;

/** Official XXMI releases, queried once per session unless forced. */
export function officialReleases(force = false): Promise<OfficialRelease[]> {
  if (force || !releasesCache) {
    releasesCache = call("xxmi_check_official_releases");
    releasesCache.catch(() => {
      releasesCache = null;
    });
  }
  return releasesCache;
}

/**
 * «Actualizar»: downloads the newer official packages, assembles the runtime
 * and, if a profile was applied, applies it again on the new runtime.
 */
export async function updateRuntime(params: {
  gameId: string;
  status: RuntimeStatus;
  sources: RuntimeSources;
  updates: PendingUpdate[];
}): Promise<void> {
  const { gameId, status, updates } = params;
  const sources = { ...params.sources };
  for (const update of updates) {
    const pkg = await call("xxmi_download_official_package", {
      packageKind: update.kind,
      tag: update.tag,
    });
    if (update.kind === "xxmi_libraries") sources.libraries = pkg.id;
    else sources.importer = pkg.id;
  }
  if (!sources.importer || !sources.libraries)
    throw new Error("Faltan paquetes oficiales para ensamblar el runtime.");

  const runtime = {
    gameId,
    importerId: sources.importer,
    librariesId: sources.libraries,
  };
  await call("prepare_game_mod_runtime", runtime);

  const current = status.selection.current;
  if (!current || updates.length === 0) return;
  const [fresh, library] = await Promise.all([
    call("inspect_game_mod_runtime", runtime),
    call("list_mod_library", { gameId }),
  ]);
  await applyProfile({
    gameId,
    profileName: current.profile_name,
    profileRevision: library.revision,
    status: fresh,
    sources,
  });
}
