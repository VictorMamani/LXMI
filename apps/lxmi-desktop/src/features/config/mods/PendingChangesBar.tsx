import { Check, RotateCcw } from "lucide-react";
import { Button } from "../../../components/Button";
import styles from "./PendingChangesBar.module.css";

/** Bottom bar of the Mods page: summary, REVERT and SAVE AND APPLY. */
export function PendingChangesBar({
  title,
  detail,
  highlighted,
  canRevert,
  canApply,
  applying,
  onRevert,
  onApply,
}: {
  title: string;
  detail: string;
  highlighted: boolean;
  canRevert: boolean;
  canApply: boolean;
  applying: boolean;
  onRevert: () => void;
  onApply: () => void;
}) {
  return (
    <div className={styles.bar}>
      <div className={styles.summary} role="status" aria-live="polite">
        <p className={styles.title} data-highlighted={highlighted}>
          {title}
        </p>
        <p className={styles.detail}>{detail}</p>
      </div>
      <Button
        icon={RotateCcw}
        disabled={!canRevert || applying}
        onClick={onRevert}
      >
        Revert
      </Button>
      <Button
        variant="primary"
        icon={Check}
        disabled={!canApply || applying}
        onClick={onApply}
      >
        {applying ? "Applying…" : "Save and apply"}
      </Button>
    </div>
  );
}
