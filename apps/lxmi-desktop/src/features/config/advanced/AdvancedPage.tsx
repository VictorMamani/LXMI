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
      eyebrow="Diagnostics"
      title="Advanced tools"
      aside={
        <p className={styles.note}>None of these tools are required to play.</p>
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
            Laboratories <Tag>Experimental</Tag>
          </span>
          <span className={styles.labsSummary}>
            Runtime Bridge · Proton/Wine context · Loader Lab · Real entry into
            the prefix — may have side effects
          </span>
        </span>
        <span className={styles.labsOpen}>OPEN →</span>
      </button>
    </PageLayout>
  );
}
