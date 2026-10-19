import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  type ReactNode,
} from "react";
import { call } from "../api/client";
import type { ModLibrary, RuntimeSources, RuntimeStatus } from "../api/types";
import type { Game } from "../domain/games";
import { resolveSources } from "../domain/runtime";
import { useAsync, type AsyncState } from "../hooks/useAsync";

export type RuntimeSnapshot = {
  status: RuntimeStatus;
  sources: RuntimeSources;
};

/** Inspects the runtime twice: first to learn the packages, then with them. */
async function loadRuntime(game: Game): Promise<RuntimeSnapshot> {
  const gameId = game.backendId;
  const first = await call("inspect_game_mod_runtime", {
    gameId,
    importerId: null,
    librariesId: null,
  });
  const sources = resolveSources(first);
  if (!sources.importer || !sources.libraries)
    return { status: first, sources };
  const status = await call("inspect_game_mod_runtime", {
    gameId,
    importerId: sources.importer,
    librariesId: sources.libraries,
  });
  return { status, sources };
}

type GameData = {
  game: Game;
  library: AsyncState<ModLibrary>;
  runtime: AsyncState<RuntimeSnapshot>;
  /** Reloads library and runtime together, e.g. after applying a profile. */
  refresh: () => Promise<void>;
};

const GameDataContext = createContext<GameData | null>(null);

export function GameDataProvider({
  game,
  children,
}: {
  game: Game;
  children: ReactNode;
}) {
  const library = useAsync(
    () => call("list_mod_library", { gameId: game.backendId }),
    game.id,
  );
  const runtime = useAsync(() => loadRuntime(game), game.id);
  const { reload: reloadLibrary } = library;
  const { reload: reloadRuntime } = runtime;
  const refresh = useCallback(async () => {
    await Promise.all([reloadLibrary(), reloadRuntime()]);
  }, [reloadLibrary, reloadRuntime]);

  const value = useMemo(
    () => ({ game, library, runtime, refresh }),
    [game, library, runtime, refresh],
  );
  return (
    <GameDataContext.Provider value={value}>
      {children}
    </GameDataContext.Provider>
  );
}

export function useGameData(): GameData {
  const value = useContext(GameDataContext);
  if (!value) throw new Error("useGameData requiere GameDataProvider");
  return value;
}
