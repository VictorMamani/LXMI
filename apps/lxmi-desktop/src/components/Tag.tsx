import type { ReactNode } from "react";
import styles from "./Tag.module.css";

/** Solid rust label such as EXPERIMENTAL or the version. */
export function Tag({ children }: { children: ReactNode }) {
  return <span className={styles.tag}>{children}</span>;
}
