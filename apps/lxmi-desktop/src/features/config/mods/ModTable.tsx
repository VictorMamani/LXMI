import type { ModDetails, ModManifest } from "../../../api/types";
import { Checkbox } from "../../../components/Checkbox";
import { formatBytes, plural } from "../../../domain/format";
import {
  kindLabel,
  modCharacter,
  modSize,
  type ModChange,
} from "../../../domain/mods";
import styles from "./ModTable.module.css";

const changeLabel: Record<Exclude<ModChange, null>, string> = {
  added: "+ NEW",
  removed: "− REMOVED",
};

/** Mod library with a check per mod for the open profile. */
export function ModTable({
  mods,
  details,
  selected,
  changeOf,
  disabled,
  emptyText,
  onToggle,
}: {
  mods: ModManifest[];
  details: ReadonlyMap<string, ModDetails>;
  selected: readonly string[];
  changeOf: (id: string) => ModChange;
  disabled: boolean;
  emptyText: string;
  onToggle: (id: string, checked: boolean) => void;
}) {
  return (
    <div className={styles.table} role="table" aria-label="Mod library">
      <div className={styles.head} role="row">
        <span className={styles.check} aria-hidden />
        <span role="columnheader" className={styles.name}>
          Mod
        </span>
        <span role="columnheader" className={styles.kind}>
          Type
        </span>
        <span role="columnheader" className={styles.character}>
          Character
        </span>
        <span role="columnheader" className={styles.number}>
          Size
        </span>
        <span role="columnheader" className={styles.number}>
          Files
        </span>
      </div>

      {mods.length === 0 ? (
        <p className={styles.empty}>{emptyText}</p>
      ) : (
        <div role="rowgroup">
          {mods.map((mod) => {
            const checked = selected.includes(mod.id);
            const change = changeOf(mod.id);
            const info = details.get(mod.id);
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
                    label={`Include ${mod.name}`}
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
                <span className={`${styles.kind} ${styles.data}`} role="cell">
                  {info ? kindLabel[info.kind] : "—"}
                </span>
                <span
                  className={`${styles.character} ${styles.data}`}
                  role="cell"
                >
                  {modCharacter(mod, info) ?? "—"}
                </span>
                <span className={`${styles.number} ${styles.data}`} role="cell">
                  {formatBytes(modSize(mod))}
                </span>
                <span className={`${styles.number} ${styles.data}`} role="cell">
                  {plural(mod.files.length, "file")}
                </span>
              </label>
            );
          })}
        </div>
      )}
    </div>
  );
}
