import { ChevronUp, Layers, Play } from "lucide-react";
import { MenuButton } from "../../components/MenuButton";
import styles from "./PlayActions.module.css";

/**
 * JUGAR (with mods) plus a chevron that holds «Jugar sin mods», so that
 * option does not need its own button; GESTIONAR MODS below.
 */
export function PlayActions({
  canPlayWithMods,
  canPlayWithout,
  playing,
  hint,
  onPlay,
  onPlayWithout,
  onManage,
}: {
  canPlayWithMods: boolean;
  canPlayWithout: boolean;
  playing: boolean;
  /** Why mods cannot load, when that is the case. */
  hint: string | null;
  onPlay: () => void;
  onPlayWithout: () => void;
  onManage: () => void;
}) {
  return (
    <div className={styles.actions}>
      <div className={styles.play}>
        <button
          type="button"
          className={styles.playMain}
          disabled={!canPlayWithMods || playing}
          onClick={onPlay}
        >
          <Play size={18} strokeWidth={1.75} aria-hidden />
          <span>{playing ? "INICIANDO…" : "JUGAR"}</span>
        </button>
        <span className={styles.divider} aria-hidden />
        <MenuButton
          className={styles.moreWrap}
          variant="glass"
          placement="top-end"
          items={[
            {
              label: "Jugar sin mods",
              onSelect: onPlayWithout,
              disabled: !canPlayWithout,
            },
          ]}
          trigger={(props) => (
            <button
              type="button"
              className={styles.playMore}
              aria-label="Más opciones de juego"
              disabled={playing}
              {...props}
            >
              <ChevronUp size={18} strokeWidth={1.75} aria-hidden />
            </button>
          )}
        />
      </div>
      <button type="button" className={styles.manage} onClick={onManage}>
        <Layers size={16} strokeWidth={1.75} aria-hidden />
        <span>GESTIONAR MODS</span>
      </button>
      <p className={styles.hint} data-warning={hint !== null || undefined}>
        {hint ?? "▲ jugar sin mods"}
      </p>
    </div>
  );
}
