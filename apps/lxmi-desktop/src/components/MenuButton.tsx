import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import styles from "./MenuButton.module.css";

export type MenuItem = {
  label: string;
  onSelect: () => void;
  disabled?: boolean;
  danger?: boolean;
  hint?: string;
  selected?: boolean;
};

type TriggerProps = {
  onClick: () => void;
  "aria-haspopup": "menu";
  "aria-expanded": boolean;
  "aria-controls": string;
};

/**
 * A trigger plus its dropdown. `glass` is the translucent menu of the home
 * screen; `panel` is the flat menu used in configuration.
 */
export function MenuButton({
  trigger,
  items,
  variant = "glass",
  placement = "bottom-end",
  className,
}: {
  trigger: (props: TriggerProps) => ReactNode;
  items: MenuItem[];
  variant?: "glass" | "panel";
  placement?: "bottom-end" | "bottom-start" | "top-end";
  className?: string;
}) {
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const id = useId();

  useEffect(() => {
    if (!open) return;
    const onPointer = (event: PointerEvent) => {
      if (!root.current?.contains(event.target as Node)) setOpen(false);
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.stopPropagation();
        setOpen(false);
      }
    };
    document.addEventListener("pointerdown", onPointer);
    document.addEventListener("keydown", onKey, true);
    return () => {
      document.removeEventListener("pointerdown", onPointer);
      document.removeEventListener("keydown", onKey, true);
    };
  }, [open]);

  return (
    <div
      ref={root}
      className={[styles.root, className].filter(Boolean).join(" ")}
    >
      {trigger({
        onClick: () => setOpen((value) => !value),
        "aria-haspopup": "menu",
        "aria-expanded": open,
        "aria-controls": id,
      })}
      {open && (
        <ul
          id={id}
          role="menu"
          className={`${styles.menu} ${styles[variant]} ${styles[placement]}`}
        >
          {items.map((item) => (
            <li key={item.label} role="none">
              <button
                type="button"
                role="menuitem"
                className={styles.item}
                data-danger={item.danger || undefined}
                data-selected={item.selected || undefined}
                disabled={item.disabled}
                title={item.hint}
                onClick={() => {
                  setOpen(false);
                  item.onSelect();
                }}
              >
                {item.label}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
