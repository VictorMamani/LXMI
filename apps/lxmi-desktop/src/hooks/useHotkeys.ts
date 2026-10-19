import { useEffect, useRef } from "react";

type KeyMap = Partial<Record<string, () => void>>;

function isTyping(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  return (
    target.isContentEditable ||
    ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName)
  );
}

/**
 * Global single-key shortcuts (`q`, `e`, `Escape`…). Ignored while typing,
 * with modifiers held, or while a modal dialog is open.
 */
export function useHotkeys(keys: KeyMap, enabled = true) {
  const latest = useRef(keys);
  useEffect(() => {
    latest.current = keys;
  });

  useEffect(() => {
    if (!enabled) return;
    function onKeyDown(event: KeyboardEvent) {
      if (
        event.ctrlKey ||
        event.metaKey ||
        event.altKey ||
        isTyping(event.target)
      )
        return;
      if (document.querySelector('dialog[open], [role="menu"]')) return;
      const handler =
        latest.current[
          event.key.length === 1 ? event.key.toLowerCase() : event.key
        ];
      if (!handler) return;
      event.preventDefault();
      handler();
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [enabled]);
}
