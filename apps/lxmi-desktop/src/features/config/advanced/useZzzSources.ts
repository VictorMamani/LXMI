import { call } from "../../../api/client";
import { resolveSources } from "../../../domain/runtime";
import { useAsync } from "../../../hooks/useAsync";

const loadZzzSources = async () =>
  resolveSources(
    await call("inspect_game_mod_runtime", {
      gameId: "zenless-zone-zero",
      importerId: null,
      librariesId: null,
    }),
  );

/**
 * The ZZZ diagnostics (readiness, assembly, bridge…) take ZZMI package ids
 * regardless of the game space that is open, so they resolve them here.
 */
export function useZzzSources() {
  const { data } = useAsync(loadZzzSources, "zzz-sources");
  return {
    zzmiId: data?.importer || null,
    librariesId: data?.libraries || null,
  };
}
