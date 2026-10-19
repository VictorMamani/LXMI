import { getCurrentWindow } from "@tauri-apps/api/window";
import { Copy, Minus, Square, X } from "lucide-react";
import { useEffect, useState } from "react";
import styles from "./TitleBar.module.css";

/**
 * Replaces the native window decorations (`decorations: false`): an invisible
 * drag strip along the top edge plus the three window controls. Double-click
 * on the strip toggles maximize. Only rendered inside the Tauri window.
 */
export function TitleBar() {
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    const window = getCurrentWindow();
    let unlisten: (() => void) | undefined;
    let disposed = false;
    const sync = () => void window.isMaximized().then(setMaximized);
    sync();
    void window.onResized(sync).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  const window = getCurrentWindow();
  return (
    <div className={styles.bar} data-tauri-drag-region>
      <div className={styles.controls}>
        <button
          type="button"
          className={styles.control}
          aria-label="Minimize"
          onClick={() => void window.minimize()}
        >
          <Minus size={15} strokeWidth={1.5} aria-hidden />
        </button>
        <button
          type="button"
          className={styles.control}
          aria-label={maximized ? "Restore" : "Maximize"}
          onClick={() => void window.toggleMaximize()}
        >
          {maximized ? (
            <Copy size={12} strokeWidth={1.5} aria-hidden />
          ) : (
            <Square size={12} strokeWidth={1.5} aria-hidden />
          )}
        </button>
        <button
          type="button"
          className={`${styles.control} ${styles.close}`}
          aria-label="Close"
          onClick={() => void window.close()}
        >
          <X size={16} strokeWidth={1.5} aria-hidden />
        </button>
      </div>
    </div>
  );
}
