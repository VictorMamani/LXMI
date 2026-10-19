import styles from "./Checkbox.module.css";

/** Square check of the mod table: cyan frame with a filled centre. */
export function Checkbox({
  checked,
  onChange,
  label,
  disabled,
}: {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
  disabled?: boolean;
}) {
  return (
    <span className={styles.box} data-checked={checked}>
      <input
        type="checkbox"
        className={styles.input}
        checked={checked}
        disabled={disabled}
        aria-label={label}
        onChange={(event) => onChange(event.target.checked)}
      />
      <span className={styles.mark} aria-hidden />
    </span>
  );
}
