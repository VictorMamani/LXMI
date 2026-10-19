import type { ReactNode } from "react";
import styles from "./StatusDot.module.css";

export type Status = "ok" | "accent" | "idle" | "warn" | "error";

/** «Status / Punto»: square dot plus a short uppercase label. */
export function StatusDot({
  status,
  children,
  size = "md",
}: {
  status: Status;
  children: ReactNode;
  size?: "sm" | "md";
}) {
  return (
    <span className={`${styles.status} ${styles[status]} ${styles[size]}`}>
      <span className={styles.dot} aria-hidden />
      {children}
    </span>
  );
}
