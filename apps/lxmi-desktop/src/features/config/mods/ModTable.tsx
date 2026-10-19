import type { ModManifest } from "../../../api/types";
import { Checkbox } from "../../../components/Checkbox";
import { formatBytes, plural } from "../../../domain/format";
import { characterOf, modSize, type ModChange } from "../../../domain/mods";
import styles from "./ModTable.module.css";

const changeLabel: Record<Exclude<ModChange, null>, string> = {
  added: "+ NUEVO",
  removed: "− QUITADO",
};

/** Mod library with a check per mod for the open profile. */
export function ModTable({
  mods,
  selected,
  changeOf,
  disabled,
  emptyText,
  onToggle,
}: {
  mods: ModManifest[];
  selected: readonly string[];
  changeOf: (id: string) => ModChange;
  disabled: boolean;
  emptyText: string;
  onToggle: (id: string, checked: boolean) => void;
}) {
  return (
    <div className={styles.table} role="table" aria-label="Biblioteca de mods">
      <div className={styles.head} role="row">
        <span className={styles.check} aria-hidden />
        <span role="columnheader" className={styles.name}>
          Mod
        </span>
        <span role="columnheader" className={styles.character}>
          Personaje
        </span>
        <span role="columnheader" className={styles.number}>
          Tamaño
        </span>
        <span role="columnheader" className={styles.number}>
          Archivos
        </span>
      </div>

      {mods.length === 0 ? (
        <p className={styles.empty}>{emptyText}</p>
      ) : (
        <div role="rowgroup">
          {mods.map((mod) => {
            const checked = selected.includes(mod.id);
            const change = changeOf(mod.id);
            return (
              <label
                key={mod.id}
                className={styles.row}
                role="row"
                data-checked={checked}
                data-changed={change !== null}
              >
                <span className={styles.check} role="cell">
                  <Checkbox
                    checked={checked}
                    disabled={disabled}
                    label={`Incluir ${mod.name}`}
                    onChange={(value) => onToggle(mod.id, value)}
                  />
                </span>
                <span
                  className={`${styles.name} ${styles.modName}`}
                  role="cell"
                >
                  <span className={styles.ellipsis}>{mod.name}</span>
                  {change && (
                    <span className={styles.change}>{changeLabel[change]}</span>
                  )}
                </span>
                <span
                  className={`${styles.character} ${styles.data}`}
                  role="cell"
                >
                  {characterOf(mod.name) ?? "—"}
                </span>
                <span className={`${styles.number} ${styles.data}`} role="cell">
                  {formatBytes(modSize(mod))}
                </span>
                <span className={`${styles.number} ${styles.data}`} role="cell">
                  {plural(mod.files.length, "archivo")}
                </span>
              </label>
            );
          })}
        </div>
      )}
    </div>
  );
}
