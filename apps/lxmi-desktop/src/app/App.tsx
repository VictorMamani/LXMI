import { ConfigScreen } from "../features/config/ConfigScreen";
import { HomeScreen } from "../features/home/HomeScreen";
import { GameDataProvider } from "../state/GameDataContext";
import { LaunchProvider } from "../state/LaunchContext";
import { SystemProvider } from "../state/SystemContext";
import { ToastProvider } from "../state/ToastContext";
import { isDemoMode } from "../api/client";
import { NavigationProvider, useNavigation } from "./navigation";
import { TitleBar } from "./TitleBar";

export default function App() {
  return (
    <NavigationProvider>
      {!isDemoMode && <TitleBar />}
      <SystemProvider>
        <LaunchProvider>
          <ToastProvider>
            <ActiveGame />
          </ToastProvider>
        </LaunchProvider>
      </SystemProvider>
    </NavigationProvider>
  );
}

function ActiveGame() {
  const { game, view } = useNavigation();
  return (
    <GameDataProvider game={game}>
      {view.screen === "home" ? (
        <HomeScreen />
      ) : (
        <ConfigScreen section={view.section} />
      )}
    </GameDataProvider>
  );
}
