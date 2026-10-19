import type { LucideIcon } from "lucide-react";
import type { ButtonHTMLAttributes } from "react";
import styles from "./Button.module.css";

type Variant = "primary" | "outline" | "ghost";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: Variant;
  icon?: LucideIcon;
  /** Accent for destructive or experimental actions. */
  tone?: "default" | "rust";
};

/** «Btn / Primario», «Btn / Contorno» and «Btn / Fantasma» of the design. */
export function Button({
  variant = "ghost",
  icon: Icon,
  tone = "default",
  className,
  children,
  type = "button",
  ...props
}: ButtonProps) {
  return (
    <button
      type={type}
      className={[
        styles.button,
        styles[variant],
        tone === "rust" && styles.rust,
        className,
      ]
        .filter(Boolean)
        .join(" ")}
      {...props}
    >
      {Icon && (
        <Icon
          size={variant === "ghost" ? 15 : 16}
          strokeWidth={1.75}
          aria-hidden
        />
      )}
      <span>{children}</span>
    </button>
  );
}
