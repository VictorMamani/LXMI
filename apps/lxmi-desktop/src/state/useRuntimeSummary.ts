import { packagesInUse, steamPlanFor, versionOf } from "../domain/runtime";
import { useGameData } from "./GameDataContext";
import { useSystem } from "./SystemContext";

/** Facts about the active game shown on several screens. */
export function useRuntimeSummary() {
  const { game, runtime } = useGameData();
  const { steam } = useSystem();
  const snapshot = runtime.data;
  const status = snapshot?.status ?? null;
  const used = snapshot
    ? packagesInUse(snapshot.status, snapshot.sources)
    : null;
  const current = status?.selection.current ?? null;

  return {
    loading: runtime.loading,
    status,
    sources: snapshot?.sources ?? null,
    profileName: current?.profile_name ?? null,
    modCount: current?.mod_count ?? 0,
    importerVersion: versionOf(used?.importer),
    librariesVersion: versionOf(used?.libraries),
    steam: steamPlanFor(steam.data, game.backendId),
    /** A runtime is assembled and the next launch can load mods. */
    ready: Boolean(status?.base_runtime && status.launch_with_mods_available),
  };
}
