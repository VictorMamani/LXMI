import { FlaskConical } from "lucide-react";
import { useNavigation } from "../../../app/navigation";
import { Tag } from "../../../components/Tag";
import { PageLayout } from "../PageLayout";
import styles from "./AdvancedPage.module.css";
import { ObservationRow } from "./rows/ObservationRow";
import { PackagesRow } from "./rows/PackagesRow";
import { PrefixRow } from "./rows/PrefixRow";
import { ReadinessRow } from "./rows/ReadinessRow";
import { SystemRow } from "./rows/SystemRow";

export function AdvancedPage() {
  const { openConfig } = useNavigation();
  return (
    <PageLayout
      eyebrow="Diagnóstico"
      title="Herramientas avanzadas"
      aside={
        <p className={styles.note}>Nada de esto es necesario para jugar.</p>
      }
    >
      <div className={styles.rows}>
        <SystemRow />
        <PackagesRow />
        <PrefixRow />
        <ReadinessRow />
        <ObservationRow />
      </div>

      <button
        type="button"
        className={styles.labs}
        onClick={() => openConfig("labs")}
      >
        <FlaskConical
          size={18}
          strokeWidth={1.75}
          aria-hidden
          className={styles.labsIcon}
        />
        <span className={styles.labsText}>
          <span className={styles.labsTitle}>
            Laboratorios <Tag>Experimental</Tag>
          </span>
          <span className={styles.labsSummary}>
            Runtime Bridge · Contexto Proton/Wine · Loader Lab · Entrada real al
            prefix — pueden tener efectos secundarios
          </span>
        </span>
        <span className={styles.labsOpen}>ABRIR →</span>
      </button>
    </PageLayout>
  );
}
