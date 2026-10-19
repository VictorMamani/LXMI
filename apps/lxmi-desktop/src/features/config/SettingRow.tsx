import type { ReactNode } from "react";
import styles from "./SettingRow.module.css";

/** Label · value (+ status) · single action — the rows of «Spaces». */
export function SettingRow({
  label,
  value,
  status,
  detail,
  action,
}: {
  label: string;
  value: ReactNode;
  status?: ReactNode;
  detail?: ReactNode;
  action?: ReactNode;
}) {
  return (
    <div className={styles.row}>
      <p className={styles.label}>{label}</p>
      <div className={styles.content}>
        <p className={styles.value}>
          <span className={styles.ellipsis}>{value}</span>
          {status}
        </p>
        {detail && <p className={styles.detail}>{detail}</p>}
      </div>
      {action && <div className={styles.action}>{action}</div>}
    </div>
  );
}
