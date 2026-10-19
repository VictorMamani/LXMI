import { useNavigation } from "../../app/navigation";
import { StatusDot } from "../../components/StatusDot";
import { games } from "../../domain/games";
import { useToast } from "../../state/ToastContext";
import { useRuntimeSummary } from "../../state/useRuntimeSummary";
import styles from "./CartridgeStrip.module.css";

/**
 * Fixed strip above every configuration page. Each game is a cartridge;
 * click or Q / E to switch the space being configured.
 */
export function CartridgeStrip({
  onSwitch,
}: {
  onSwitch: (step: 1 | -1) => void;
}) {
  const { game: active, selectGame } = useNavigation();
  const toast = useToast();

  return (
    <div className={styles.strip}>
      <button
        type="button"
        className={styles.key}
        onClick={() => onSwitch(-1)}
        aria-label="Juego anterior (Q)"
      >
        Q
      </button>
      <div
        className={styles.cartridges}
        role="tablist"
        aria-label="Espacios de juego"
      >
        {games.map((game) => (
          <button
            key={game.id}
            type="button"
            role="tab"
            aria-selected={game.id === active.id}
            className={styles.cartridge}
            onClick={() => {
              if (game.id === active.id) return;
              selectGame(game.id);
              toast(`Espacio ${game.channel} cargado`);
            }}
          >
            <img className={styles.thumb} src={game.media.card} alt="" />
            <span className={styles.text}>
              <span className={styles.channel}>{game.channel}</span>
              <span className={styles.name}>{game.name}</span>
            </span>
          </button>
        ))}
      </div>
      <button
        type="button"
        className={styles.key}
        onClick={() => onSwitch(1)}
        aria-label="Juego siguiente (E)"
      >
        E
      </button>
      <RuntimeIndicator />
    </div>
  );
}

function RuntimeIndicator() {
  const { game } = useNavigation();
  const summary = useRuntimeSummary();
  if (summary.loading && !summary.status)
    return (
      <span className={styles.status}>
        <StatusDot status="idle">Consultando…</StatusDot>
      </span>
    );
  const version = summary.importerVersion
    ? ` · ${game.importer} ${summary.importerVersion}`
    : "";
  return (
    <span className={styles.status}>
      {summary.ready ? (
        <StatusDot status="ok">Runtime listo{version}</StatusDot>
      ) : summary.status?.base_runtime ? (
        <StatusDot status="accent">Sin perfil aplicado{version}</StatusDot>
      ) : (
        <StatusDot status="warn">Runtime sin preparar</StatusDot>
      )}
    </span>
  );
}
