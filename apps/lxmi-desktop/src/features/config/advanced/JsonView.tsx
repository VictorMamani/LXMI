import styles from "./JsonView.module.css";

/** Raw backend result, for evidence that has no dedicated view. */
export function JsonView({ label, value }: { label: string; value: unknown }) {
  return (
    <details className={styles.json} open>
      <summary>{label}</summary>
      <pre>{JSON.stringify(value, null, 2)}</pre>
    </details>
  );
}
