import { Button } from "../../components/Button";
import { Dialog } from "../../components/Dialog";
import styles from "./ShortcutsDialog.module.css";

const shortcuts: [string, string][] = [
  ["Q / E", "Previous / next game"],
  ["M", "Manage mods"],
  ["Esc", "Return home from settings"],
];

export function ShortcutsDialog({ onClose }: { onClose: () => void }) {
  return (
    <Dialog
      title="Keyboard shortcuts"
      onClose={onClose}
      width={420}
      actions={
        <Button variant="outline" onClick={onClose}>
          Close
        </Button>
      }
    >
      <dl className={styles.list}>
        {shortcuts.map(([keys, action]) => (
          <div key={keys} className={styles.row}>
            <dt>
              {keys.split(" / ").map((key) => (
                <kbd key={key}>{key}</kbd>
              ))}
            </dt>
            <dd>{action}</dd>
          </div>
        ))}
      </dl>
    </Dialog>
  );
}
