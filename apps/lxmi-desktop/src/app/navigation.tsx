import {
  createContext,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { gameById, type Game, type GameId } from "../domain/games";

export type ConfigSection =
  "spaces" | "mods" | "share" | "advanced" | "labs" | "about";

export type View =
  { screen: "home" } | { screen: "config"; section: ConfigSection };

type Navigation = {
  view: View;
  game: Game;
  selectGame: (id: GameId) => void;
  goHome: () => void;
  openConfig: (section: ConfigSection) => void;
};

const NavigationContext = createContext<Navigation | null>(null);

/** Screen routing and the active game space. No URL routing is needed. */
export function NavigationProvider({ children }: { children: ReactNode }) {
  const [view, setView] = useState<View>({ screen: "home" });
  const [gameId, setGameId] = useState<GameId>("zzz");

  const value = useMemo<Navigation>(
    () => ({
      view,
      game: gameById(gameId),
      selectGame: setGameId,
      goHome: () => setView({ screen: "home" }),
      openConfig: (section) => setView({ screen: "config", section }),
    }),
    [view, gameId],
  );
  return (
    <NavigationContext.Provider value={value}>
      {children}
    </NavigationContext.Provider>
  );
}

export function useNavigation(): Navigation {
  const value = useContext(NavigationContext);
  if (!value) throw new Error("useNavigation requires NavigationProvider");
  return value;
}
