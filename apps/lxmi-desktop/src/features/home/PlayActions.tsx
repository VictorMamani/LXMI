import { ChevronUp, Layers, Play } from "lucide-react";
import { MenuButton } from "../../components/MenuButton";
import styles from "./PlayActions.module.css";

/**
 * PLAY (with mods) plus a chevron that holds «Play without mods», so that
 * option does not need its own button; MANAGE MODS below.
 */
export function PlayActions({
  canPlayWithMods,
  canPlayWithout,
  playing,
  hint,
  experimental,
  onPlay,
  onPlayWithout,
  onManage,
}: {
  canPlayWithMods: boolean;
  canPlayWithout: boolean;
  playing: boolean;
  /** Why mods cannot load, when that is the case. */
  hint: string | null;
  experimental: boolean;
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
          <span>{playing ? "STARTING…" : "PLAY"}</span>
        </button>
        <span className={styles.divider} aria-hidden />
        <MenuButton
          className={styles.moreWrap}
          variant="glass"
          placement="top-end"
          items={[
            {
              label: "Play without mods",
              onSelect: onPlayWithout,
              disabled: !canPlayWithout,
            },
          ]}
          trigger={(props) => (
            <button
              type="button"
              className={styles.playMore}
              aria-label="More play options"
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
        <span>MANAGE MODS</span>
      </button>
      <p className={styles.hint} data-warning={hint !== null || undefined}>
        {hint ?? "▲ play without mods"}
      </p>
      {experimental && (
        <p className={styles.hint} data-warning>
          Experimental WWMI. Tested with dwproton 11.0-14. In Steam launch
          options, use <code>PROTON_DXVK_SAREK=1 %command%</code>.
        </p>
      )}
    </div>
  );
}
