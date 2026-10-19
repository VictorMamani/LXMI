import type { ComponentType } from "react";
import { useNavigation, type ConfigSection } from "../../app/navigation";
import { adjacentGame } from "../../domain/games";
import { useHotkeys } from "../../hooks/useHotkeys";
import { useToast } from "../../state/ToastContext";
import { AboutPage } from "./about/AboutPage";
import { AdvancedPage } from "./advanced/AdvancedPage";
import { LabsPage } from "./advanced/LabsPage";
import { CartridgeStrip } from "./CartridgeStrip";
import styles from "./ConfigScreen.module.css";
import { ModsPage } from "./mods/ModsPage";
import { SharePage } from "./share/SharePage";
import { Sidebar } from "./Sidebar";
import { SpacesPage } from "./spaces/SpacesPage";

const pages: Record<ConfigSection, ComponentType> = {
  spaces: SpacesPage,
  mods: ModsPage,
  share: SharePage,
  advanced: AdvancedPage,
  labs: LabsPage,
  about: AboutPage,
};

export function ConfigScreen({ section }: { section: ConfigSection }) {
  const { game, selectGame, goHome } = useNavigation();
  const toast = useToast();
  const Page = pages[section];

  function switchGame(step: 1 | -1) {
    const next = adjacentGame(game.id, step);
    if (next.id === game.id) return;
    selectGame(next.id);
    toast(`Espacio ${next.channel} cargado`);
  }

  useHotkeys({
    q: () => switchGame(-1),
    e: () => switchGame(1),
    Escape: goHome,
  });

  return (
    <div className={styles.screen}>
      <Sidebar active={section} />
      <div className={styles.main}>
        <CartridgeStrip onSwitch={switchGame} />
        {/* Keyed by game: switching cartridges reloads the page content. */}
        <div key={game.id} className={styles.page}>
          <Page />
        </div>
      </div>
    </div>
  );
}
