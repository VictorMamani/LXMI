import {
  useEffect,
  useId,
  useRef,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
  type ReactNode,
} from "react";
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
  onKeyDown: (event: ReactKeyboardEvent<HTMLButtonElement>) => void;
  ref: (element: HTMLButtonElement | null) => void;
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
  const triggerElement = useRef<HTMLButtonElement | null>(null);
  const focusLast = useRef(false);
  const id = useId();

  useEffect(() => {
    if (!open) return;
    const enabledItems = () =>
      Array.from(
        root.current?.querySelectorAll<HTMLButtonElement>(
          '[role="menuitem"]:not(:disabled)',
        ) ?? [],
      );
    const menuItems = enabledItems();
    (focusLast.current ? menuItems.at(-1) : menuItems[0])?.focus();
    const onPointer = (event: PointerEvent) => {
      if (!root.current?.contains(event.target as Node)) setOpen(false);
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        setOpen(false);
        triggerElement.current?.focus();
      } else if (event.key === "Tab") {
        setOpen(false);
        triggerElement.current?.focus();
      } else if (["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
        event.preventDefault();
        event.stopPropagation();
        const entries = enabledItems();
        const index = entries.findIndex(
          (entry) => entry === document.activeElement,
        );
        const next =
          event.key === "Home"
            ? 0
            : event.key === "End"
              ? entries.length - 1
              : (index +
                  (event.key === "ArrowDown" ? 1 : -1) +
                  entries.length) %
                entries.length;
        entries[next]?.focus();
      } else if (
        event.key.length === 1 &&
        !event.ctrlKey &&
        !event.metaKey &&
        !event.altKey &&
        event.key !== " "
      ) {
        const entries = enabledItems();
        const index = entries.findIndex(
          (entry) => entry === document.activeElement,
        );
        const ordered = [
          ...entries.slice(index + 1),
          ...entries.slice(0, index + 1),
        ];
        ordered
          .find((entry) =>
            entry.textContent
              ?.trim()
              .toLowerCase()
              .startsWith(event.key.toLowerCase()),
          )
          ?.focus();
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
        onClick: () => {
          focusLast.current = false;
          setOpen((value) => !value);
        },
        onKeyDown: (event) => {
          if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
          event.preventDefault();
          focusLast.current = event.key === "ArrowUp";
          setOpen(true);
        },
        ref: (element) => {
          triggerElement.current = element;
        },
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
                tabIndex={-1}
                className={styles.item}
                data-danger={item.danger || undefined}
                data-selected={item.selected || undefined}
                disabled={item.disabled}
                title={item.hint}
                onClick={() => {
                  setOpen(false);
                  triggerElement.current?.focus();
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
