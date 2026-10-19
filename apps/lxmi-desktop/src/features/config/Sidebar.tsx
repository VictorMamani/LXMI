import { ArrowLeft } from "lucide-react";
import { version } from "../../../package.json";
import { useNavigation, type ConfigSection } from "../../app/navigation";
import { Button } from "../../components/Button";
import { Tag } from "../../components/Tag";
import styles from "./Sidebar.module.css";

const sections: { id: ConfigSection; label: string }[] = [
  { id: "spaces", label: "Espacios" },
  { id: "mods", label: "Mods" },
  { id: "share", label: "Compartir" },
  { id: "advanced", label: "Avanzado" },
  { id: "about", label: "Acerca de" },
];

export function Sidebar({ active }: { active: ConfigSection }) {
  const { openConfig, goHome } = useNavigation();
  // Laboratories live under «Avanzado».
  const current = active === "labs" ? "advanced" : active;

  return (
    <aside className={styles.sidebar}>
      <div className={styles.brand}>
        <p className={styles.logo}>MODHUB</p>
        <p className={styles.subtitle}>CONFIGURACIÓN</p>
      </div>

      <nav aria-label="Secciones de configuración">
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
          Volver
        </Button>
        <p className={styles.version}>
          <Tag>LXMI {version}</Tag>
          <span>ESC</span>
        </p>
      </div>
    </aside>
  );
}
