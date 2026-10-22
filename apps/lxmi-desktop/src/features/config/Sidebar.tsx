import { ArrowLeft } from "lucide-react";
import { version } from "../../../package.json";
import { useNavigation, type ConfigSection } from "../../app/navigation";
import { Button } from "../../components/Button";
import styles from "./Sidebar.module.css";

const sections: { id: ConfigSection; label: string }[] = [
  { id: "spaces", label: "Spaces" },
  { id: "mods", label: "Mods" },
  { id: "share", label: "Share" },
  { id: "advanced", label: "Advanced" },
  { id: "about", label: "About" },
];

export function Sidebar({ active }: { active: ConfigSection }) {
  const { openConfig, goHome } = useNavigation();
  // Laboratories live under «Advanced».
  const current = active === "labs" ? "advanced" : active;

  return (
    <aside className={styles.sidebar}>
      <div className={styles.brand}>
        <p className={styles.logo}>LXMI</p>
        <p className={styles.subtitle}>Settings · v{version}</p>
      </div>

      <nav aria-label="Settings sections">
        <ul className={styles.nav}>
          {sections.map((section) => (
            <li key={section.id}>
              <button
                type="button"
                className={styles.link}
                aria-current={section.id === current ? "page" : undefined}
                onClick={() => openConfig(section.id)}
              >
                {section.label}
              </button>
            </li>
          ))}
        </ul>
      </nav>

      <div className={styles.footer}>
        <Button variant="outline" icon={ArrowLeft} onClick={goHome}>
          Back
        </Button>
        <p className={styles.hint}>
          <kbd>ESC</kbd> to go back
        </p>
      </div>
    </aside>
  );
}
