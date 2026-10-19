import { AlertTriangle, Check, Repeat } from "lucide-react";
import styles from "./Toast.module.css";

export type ToastTone = "info" | "success" | "error";

const icons = { info: Repeat, success: Check, error: AlertTriangle };

/** Short uppercase change notice with a cyan left rule. */
export function Toast({ text, tone }: { text: string; tone: ToastTone }) {
  const Icon = icons[tone];
  return (
    <div
      className={`${styles.toast} ${styles[tone]}`}
      role="status"
      aria-live="polite"
    >
      <Icon size={14} strokeWidth={2} aria-hidden />
      <span>{text}</span>
    </div>
  );
}
