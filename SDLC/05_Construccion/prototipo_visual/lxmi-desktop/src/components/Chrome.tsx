import type { ReactNode } from "react";
import type { Game, GameId } from "../data";
import { Icon, type IconName } from "./Icon";

export type ScreenId =
  "home" | "mods" | "profiles" | "compatibility" | "settings" | "diagnostics";

const primaryNavigation: { id: ScreenId; label: string; icon: IconName }[] = [
  { id: "home", label: "Inicio", icon: "home" },
  { id: "mods", label: "Mods", icon: "mods" },
  { id: "profiles", label: "Profiles", icon: "layers" },
  { id: "compatibility", label: "Compatibilidad", icon: "compatibility" },
];

export function GameRail({
  games,
  selectedGame,
  onSelectGame,
  onAddGame,
  onNavigate,
}: {
  games: Game[];
  selectedGame: GameId;
  onSelectGame: (id: GameId) => void;
  onAddGame: () => void;
  onNavigate: (screen: ScreenId) => void;
}) {
  return (
    <aside className="game-rail" aria-label="LXMI y juegos">
      <button
        className="wordmark"
        aria-label="LXMI, ir al inicio"
        onClick={() => onNavigate("home")}
      >
        <span className="brand-mark" aria-hidden="true">
          <span />
        </span>
        <span className="wordmark-text">lxmi</span>
      </button>

      <div className="rail-rule" />
      <div className="game-rail-list" aria-label="Juegos">
        {games.map((game) => (
          <button
            className={`game-rail-item ${selectedGame === game.id ? "is-selected" : ""}`}
            key={game.id}
            onClick={() => onSelectGame(game.id)}
            aria-label={`Seleccionar ${game.name}`}
            aria-pressed={selectedGame === game.id}
            title={game.name}
          >
            <span className={`rail-cover rail-cover--${game.id}`}>
              <img src={game.art} alt="" />
            </span>
            <span className="rail-selection-indicator" />
          </button>
        ))}
        <button
          className="rail-add-game"
          onClick={onAddGame}
          aria-label="Añadir juego"
          title="Añadir juego"
        >
          <Icon name="plus" size={19} />
        </button>
      </div>
    </aside>
  );
}

export function WorkspaceHeader({
  game,
  screen,
  onNavigate,
}: {
  game: Game;
  screen: ScreenId;
  onNavigate: (screen: ScreenId) => void;
}) {
  const label =
    screen === "settings"
      ? "Configuración"
      : screen === "diagnostics"
        ? "Diagnósticos"
        : game.name;
  return (
    <header className="workspace-header">
      <div className="header-context">
        <span className="context-game-mark" aria-hidden="true">
          <span />
        </span>
        <span className="header-game-name">{label}</span>
      </div>
      <nav className="primary-navigation" aria-label="Navegación principal">
        {primaryNavigation.map((item) => (
          <button
            key={item.id}
            className={`navigation-tab ${screen === item.id ? "is-current" : ""}`}
            aria-current={screen === item.id ? "page" : undefined}
            onClick={() => onNavigate(item.id)}
          >
            <Icon name={item.icon} size={16} />
            <span>{item.label}</span>
          </button>
        ))}
      </nav>
      <div className="header-actions">
        <button
          className={`header-icon-button ${screen === "diagnostics" ? "is-current" : ""}`}
          onClick={() => onNavigate("diagnostics")}
          title="Diagnósticos"
          aria-label="Diagnósticos"
        >
          <Icon name="monitor" size={17} />
        </button>
        <span className="header-separator" aria-hidden="true" />
        <button
          className={`header-icon-button ${screen === "settings" ? "is-current" : ""}`}
          onClick={() => onNavigate("settings")}
          title="Configuración"
          aria-label="Configuración"
        >
          <Icon name="settings" size={17} />
        </button>
      </div>
    </header>
  );
}

export function ActionButton({
  children,
  icon,
  onClick,
  variant = "secondary",
  disabled = false,
  className = "",
  type = "button",
}: {
  children: ReactNode;
  icon?: IconName;
  onClick?: () => void;
  variant?: "primary" | "secondary" | "quiet" | "small";
  disabled?: boolean;
  className?: string;
  type?: "button" | "submit";
}) {
  return (
    <button
      type={type}
      className={`action-button action-button--${variant} ${className}`}
      onClick={onClick}
      disabled={disabled}
    >
      {icon && <Icon name={icon} size={16} />}
      <span>{children}</span>
    </button>
  );
}

export function IconButton({
  icon,
  label,
  onClick,
  className = "",
  disabled = false,
}: {
  icon: IconName;
  label: string;
  onClick: () => void;
  className?: string;
  disabled?: boolean;
}) {
  return (
    <button
      className={`icon-button ${className}`}
      onClick={onClick}
      aria-label={label}
      title={label}
      disabled={disabled}
    >
      <Icon name={icon} size={17} />
    </button>
  );
}

export function Toggle({
  checked,
  onChange,
  label,
  disabled = false,
}: {
  checked: boolean;
  onChange: () => void;
  label: string;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      className={`toggle ${checked ? "is-on" : ""}`}
      role="switch"
      aria-checked={checked}
      aria-label={label}
      onClick={onChange}
      disabled={disabled}
    >
      <span className="toggle-knob" />
    </button>
  );
}

export function StatusMark({
  tone = "good",
  children,
}: {
  tone?: "good" | "quiet" | "warning" | "error";
  children: ReactNode;
}) {
  return (
    <span className={`status-mark status-mark--${tone}`}>
      <span className="status-mark-dot" aria-hidden="true" />
      {children}
    </span>
  );
}

export function PageHeading({
  title,
  description,
  action,
}: {
  title: string;
  description?: string;
  action?: ReactNode;
}) {
  return (
    <div className="page-heading">
      <div>
        <h1>{title}</h1>
        {description && <p>{description}</p>}
      </div>
      {action && <div className="page-heading-action">{action}</div>}
    </div>
  );
}

export function Toast({
  message,
  onClose,
}: {
  message: string;
  onClose: () => void;
}) {
  return (
    <div className="toast" role="status" aria-live="polite">
      <span className="toast-mark">
        <Icon name="check" size={15} />
      </span>
      <span>{message}</span>
      <button onClick={onClose} aria-label="Cerrar aviso">
        <Icon name="close" size={15} />
      </button>
    </div>
  );
}

export function EmptyState({
  title,
  detail,
  action,
}: {
  title: string;
  detail: string;
  action?: ReactNode;
}) {
  return (
    <div className="empty-state">
      <span className="empty-state-glyph">
        <Icon name="search" size={20} />
      </span>
      <h3>{title}</h3>
      <p>{detail}</p>
      {action}
    </div>
  );
}

export function ScreenPanel({
  children,
  className = "",
}: {
  children: ReactNode;
  className?: string;
}) {
  return <section className={`screen-panel ${className}`}>{children}</section>;
}
