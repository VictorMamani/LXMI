import { Plus } from "lucide-react";
import { call } from "../../../api/client";
import { useNavigation } from "../../../app/navigation";
import { plural } from "../../../domain/format";
import { games, nextFreeChannel } from "../../../domain/games";
import { useAsync } from "../../../hooks/useAsync";
import { useToast } from "../../../state/ToastContext";
import styles from "./SpaceShelf.module.css";

const loadModCounts = () =>
  Promise.all(
    games.map((game) =>
      call("list_mod_library", { gameId: game.backendId })
        .then((library) => library.mods.length)
        .catch(() => null),
    ),
  );

/** Every game as a cartridge, plus a free slot for future games. */
export function SpaceShelf() {
  const { game: active, selectGame } = useNavigation();
  const toast = useToast();
  const counts = useAsync(loadModCounts, "spaces");

  return (
    <section className={styles.shelf} aria-labelledby="spaces-heading">
      <h2 id="spaces-heading" className={styles.heading}>
        Otros espacios
      </h2>
      <div className={styles.slots}>
        {games.map((game, index) => {
          const isActive = game.id === active.id;
          const count = counts.data?.[index];
          return (
            <button
              key={game.id}
              type="button"
              className={styles.slot}
              data-active={isActive}
              aria-current={isActive || undefined}
              onClick={() => {
                if (isActive) return;
                selectGame(game.id);
                toast(`Espacio ${game.channel} cargado`);
              }}
            >
              <img className={styles.art} src={game.media.card} alt="" />
              <span className={styles.text}>
                <span className={styles.channel}>{game.channel}</span>
                <span className={styles.name}>{game.name}</span>
                <span className={styles.meta}>
                  {isActive
                    ? "activo"
                    : count == null
                      ? "listo"
                      : `${plural(count, "mod")} · listo`}
                </span>
              </span>
            </button>
          );
        })}
        <div className={`${styles.slot} ${styles.free}`} aria-disabled>
          <span className={styles.art}>
            <Plus size={18} strokeWidth={1.5} aria-hidden />
          </span>
          <span className={styles.text}>
            <span className={styles.channel}>{nextFreeChannel}</span>
            <span className={styles.name}>Ranura libre</span>
            <span className={styles.meta}>próximamente</span>
          </span>
        </div>
      </div>
    </section>
  );
}
