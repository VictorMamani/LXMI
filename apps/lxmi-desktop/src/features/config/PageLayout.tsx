import type { ReactNode } from "react";
import styles from "./PageLayout.module.css";

/** Scrollable body of a configuration page with its section header. */
export function PageLayout({
  eyebrow,
  title,
  aside,
  children,
  footer,
}: {
  eyebrow: string;
  title: ReactNode;
  /** Right side of the header: actions or a short note. */
  aside?: ReactNode;
  children: ReactNode;
  /** Fixed bar under the scrollable area (e.g. pending changes). */
  footer?: ReactNode;
}) {
  return (
    <>
      <div className={styles.body}>
        <header className={styles.header}>
          <div className={styles.titles}>
            <p className={styles.eyebrow}>{eyebrow}</p>
            {typeof title === "string" ? (
              <h1 className={styles.title}>{title}</h1>
            ) : (
              title
            )}
          </div>
          {aside && <div className={styles.aside}>{aside}</div>}
        </header>
        {children}
      </div>
      {footer}
    </>
  );
}

/** Big display heading used by the page titles. */
export function PageTitle({ children }: { children: ReactNode }) {
  return <h1 className={styles.title}>{children}</h1>;
}
