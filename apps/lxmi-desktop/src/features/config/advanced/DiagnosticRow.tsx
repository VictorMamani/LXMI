import { ChevronRight } from "lucide-react";
import { useId, useState, type ReactNode } from "react";
import styles from "./DiagnosticRow.module.css";

/** Folded diagnostic group: title + state, one-line summary, one action. */
export function DiagnosticRow({
  title,
  status,
  summary,
  action,
  children,
}: {
  title: string;
  status: ReactNode;
  summary: ReactNode;
  action: ReactNode;
  children: ReactNode;
}) {
  const [open, setOpen] = useState(false);
  const panelId = useId();
  return (
    <section className={styles.row} data-open={open}>
      <div className={styles.header}>
        <button
          type="button"
          className={styles.toggle}
          aria-expanded={open}
          aria-controls={panelId}
          onClick={() => setOpen((value) => !value)}
        >
          <ChevronRight
            size={16}
            strokeWidth={1.75}
            className={styles.chevron}
            aria-hidden
          />
          <span className={styles.text}>
            <span className={styles.titleLine}>
              <span className={styles.title}>{title}</span>
              {status}
            </span>
            <span className={styles.summary}>{summary}</span>
          </span>
        </button>
        <div className={styles.action}>{action}</div>
      </div>
      {open && (
        <div id={panelId} className={styles.details}>
          {children}
        </div>
      )}
    </section>
  );
}

/** Key/value facts inside an unfolded row. */
export function Facts({ items }: { items: [string, ReactNode][] }) {
  return (
    <dl className={styles.facts}>
      {items.map(([label, value]) => (
        <div key={label} className={styles.fact}>
          <dt>{label}</dt>
          <dd>{value ?? "—"}</dd>
        </div>
      ))}
    </dl>
  );
}

/** Secondary actions inside an unfolded row. */
export function DetailActions({ children }: { children: ReactNode }) {
  return <div className={styles.detailActions}>{children}</div>;
}
