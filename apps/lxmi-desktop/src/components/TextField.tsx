import type { LucideIcon } from "lucide-react";
import { useId, type InputHTMLAttributes } from "react";
import styles from "./TextField.module.css";

type TextFieldProps = InputHTMLAttributes<HTMLInputElement> & {
  label?: string;
  hint?: string;
  icon?: LucideIcon;
};

/** Outlined input; the label is visually hidden when an icon replaces it. */
export function TextField({
  label,
  hint,
  icon: Icon,
  className,
  ...props
}: TextFieldProps) {
  const id = useId();
  return (
    <div className={[styles.field, className].filter(Boolean).join(" ")}>
      {label && (
        <label htmlFor={id} className={Icon ? "visually-hidden" : styles.label}>
          {label}
        </label>
      )}
      <div className={styles.control}>
        {Icon && (
          <Icon
            size={14}
            strokeWidth={1.75}
            aria-hidden
            className={styles.icon}
          />
        )}
        <input
          id={id}
          className={styles.input}
          autoComplete="off"
          spellCheck={false}
          {...props}
        />
      </div>
      {hint && <p className={styles.hint}>{hint}</p>}
    </div>
  );
}
