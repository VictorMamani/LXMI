import { AlertTriangle } from "lucide-react";
import type { ReactNode } from "react";
import styles from "./InlineError.module.css";

/** Error line with an optional action (retry, dismiss…). */
export function InlineError({
  children,
  action,
}: {
  children: ReactNode;
  action?: ReactNode;
}) {
  return (
    <div className={styles.error} role="alert">
      <AlertTriangle size={14} strokeWidth={1.75} aria-hidden />
      <p>{children}</p>
      {action}
    </div>
  );
}
