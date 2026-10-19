import { Ellipsis } from "lucide-react";
import { useNavigation } from "../../app/navigation";
import { MenuButton } from "../../components/MenuButton";
import { games, type Game } from "../../domain/games";
import styles from "./GameCards.module.css";

/** Cover cards of the installed games; the active one is larger and lit. */
export function GameCards() {
  const { game: active, selectGame } = useNavigation();
  return (
    <nav className={styles.cards} aria-label="Games">
      {games.map((game) => (
        <GameCard
          key={game.id}
          game={game}
          active={game.id === active.id}
          onSelect={() => selectGame(game.id)}
        />
      ))}
    </nav>
  );
}

function GameCard({
  game,
  active,
  onSelect,
}: {
  game: Game;
  active: boolean;
  onSelect: () => void;
}) {
  const { selectGame, openConfig } = useNavigation();
  const manage = (section: "mods" | "spaces") => () => {
    selectGame(game.id);
    openConfig(section);
  };

  return (
    <div className={styles.card} data-active={active}>
      <button
        type="button"
        className={styles.cover}
        style={{ backgroundImage: `url(${game.media.card})` }}
        aria-pressed={active}
        aria-label={`${game.name} (${game.channel})`}
        onClick={onSelect}
      />
      <MenuButton
        className={styles.more}
        variant="glass"
        placement="bottom-end"
        items={[
          { label: "Manage", onSelect: manage("mods") },
          { label: "View details", onSelect: manage("spaces") },
          {
            label: "Uninstall",
            onSelect: () => {},
            danger: true,
            disabled: true,
            hint: "Steam manages the installation",
          },
        ]}
        trigger={(props) => (
          <button
            type="button"
            className={styles.moreButton}
            aria-label={`${game.name} options`}
            {...props}
          >
            <Ellipsis size={18} aria-hidden />
          </button>
        )}
      />
    </div>
  );
}
