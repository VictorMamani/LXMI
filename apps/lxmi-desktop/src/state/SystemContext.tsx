import { createContext, useContext, type ReactNode } from "react";
import { call } from "../api/client";
import type { SteamScan, SystemInfo } from "../api/types";
import { useAsync, type AsyncState } from "../hooks/useAsync";

type SystemData = {
  system: AsyncState<SystemInfo>;
  /** Steam libraries, games and Proton selection; shared by every screen. */
  steam: AsyncState<SteamScan>;
};

const SystemContext = createContext<SystemData | null>(null);

const loadSystem = () => call("get_system_info");
const loadSteam = () => call("scan_steam");

export function SystemProvider({ children }: { children: ReactNode }) {
  const system = useAsync(loadSystem, "system");
  const steam = useAsync(loadSteam, "steam");
  return (
    <SystemContext.Provider value={{ system, steam }}>
      {children}
    </SystemContext.Provider>
  );
}

export function useSystem(): SystemData {
  const value = useContext(SystemContext);
  if (!value) throw new Error("useSystem requires SystemProvider");
  return value;
}
