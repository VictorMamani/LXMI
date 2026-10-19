import { Button } from "../../components/Button";
import { Dialog } from "../../components/Dialog";
import styles from "./ShortcutsDialog.module.css";

const shortcuts: [string, string][] = [
  ["Q / E", "Juego anterior / siguiente"],
  ["M", "Gestionar mods"],
  ["Esc", "Volver al inicio desde configuración"],
];

export function ShortcutsDialog({ onClose }: { onClose: () => void }) {
  return (
    <Dialog
      title="Atajos de teclado"
      onClose={onClose}
      width={420}
      actions={
        <Button variant="outline" onClick={onClose}>
          Cerrar
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
