import { useMemo, useState } from "react";
import type { Game, MockMod, MockProfile } from "./data";
import { Icon } from "./components/Icon";
import {
  ActionButton,
  EmptyState,
  PageHeading,
  StatusMark,
  Toggle,
} from "./components/Chrome";

export function HomeScreen({
  game,
  activeProfile,
  activeMods,
  launching,
  onPlay,
  onOpenMods,
  onOpenProfiles,
}: {
  game: Game;
  activeProfile: string;
  activeMods: number;
  launching: boolean;
  onPlay: () => void;
  onOpenMods: () => void;
  onOpenProfiles: () => void;
}) {
  return (
    <div className="home-screen">
      <section
        className={`game-hero game-hero--${game.id}`}
        aria-label={`${game.name}, resumen de juego`}
      >
        <div className="game-hero-art">
          <img className="hero-art-image" src={game.art} alt="" />
          <div className="hero-art-shade" />
          <div className="hero-title-lockup">
            <span className="hero-title-rule" />
            <h1>{game.name}</h1>
            <p>{game.subtitle}</p>
          </div>
          <span className="hero-art-note">
            Artwork abstracto · contenido de muestra
          </span>
        </div>
        <div className="game-hero-controls">
          <div className="hero-ready-line">
            <StatusMark>Preparado para jugar</StatusMark>
            <button className="text-link" onClick={onOpenProfiles}>
              Cambiar profile
            </button>
          </div>
          <h2>
            Tu partida,
            <br />a tu manera.
          </h2>
          <div className="hero-current-profile">
            <span className="profile-glyph">
              <Icon name="layers" size={17} />
            </span>
            <span>
              <small>PROFILE ACTUAL</small>
              <strong>{activeProfile}</strong>
            </span>
            <button
              className="inline-chevron"
              onClick={onOpenProfiles}
              aria-label="Ver profiles"
            >
              <Icon name="chevron" size={16} />
            </button>
          </div>
          <div className="hero-mod-summary">
            <button onClick={onOpenMods}>
              <span className="summary-count">{activeMods}</span>
              <span>
                <strong>mods activos</strong>
                <small>en este profile</small>
              </span>
              <Icon name="arrow" size={15} />
            </button>
            <span className="summary-divider" />
            <div className="summary-runtime">
              <span className="runtime-glyph">
                <Icon name="spark" size={15} />
              </span>
              <span>
                <strong>{game.runtime}</strong>
                <small>{game.modManager}</small>
              </span>
            </div>
          </div>
          <div className="hero-play-wrap">
            <ActionButton
              variant="primary"
              icon={launching ? undefined : "play"}
              onClick={onPlay}
              disabled={launching}
              className="play-button"
            >
              {launching ? "Iniciando…" : "Jugar"}
            </ActionButton>
            <span>Se abrirá desde Steam</span>
          </div>
        </div>
      </section>

      <div className="home-footline">
        <div className="home-footline-item">
          <span className="footline-symbol">
            <Icon name="compatibility" size={16} />
          </span>
          <span>
            <small>Compatibilidad</small>
            <strong>{game.runtime}</strong>
          </span>
        </div>
        <span className="footline-divider" />
        <div className="home-footline-item">
          <span className="footline-symbol">
            <Icon name="mods" size={16} />
          </span>
          <span>
            <small>Mod manager</small>
            <strong>{game.modManager}</strong>
          </span>
        </div>
        <span className="footline-divider" />
        <div className="home-footline-item home-prefix-item">
          <span className="footline-symbol">
            <Icon name="folder" size={16} />
          </span>
          <span>
            <small>Wine prefix</small>
            <strong>{game.prefix}</strong>
          </span>
        </div>
        <span className="home-footline-spacer" />
        <span className="sample-inline">
          <span /> Entorno de demostración
        </span>
      </div>
    </div>
  );
}

type ModFilter = "all" | "enabled" | "attention";

export function ModsScreen({
  game,
  mods,
  onToggle,
  onInstall,
  onDetails,
}: {
  game: Game;
  mods: MockMod[];
  onToggle: (id: string) => void;
  onInstall: () => void;
  onDetails: (mod: MockMod) => void;
}) {
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<ModFilter>("all");
  const [sort, setSort] = useState("active");
  const gameMods = mods.filter((mod) => mod.gameId === game.id);
  const visibleMods = useMemo(() => {
    const normalizedQuery = query.trim().toLocaleLowerCase();
    const result = gameMods.filter((mod) => {
      const matchesQuery =
        !normalizedQuery ||
        `${mod.name} ${mod.author} ${mod.character}`
          .toLocaleLowerCase()
          .includes(normalizedQuery);
      const matchesFilter =
        filter === "all" ||
        (filter === "enabled" ? mod.enabled : Boolean(mod.conflict));
      return matchesQuery && matchesFilter;
    });
    return result.sort((left, right) => {
      if (sort === "active" && left.enabled !== right.enabled) {
        return Number(right.enabled) - Number(left.enabled);
      }
      return sort === "character"
        ? left.character.localeCompare(right.character)
        : left.name.localeCompare(right.name);
    });
  }, [filter, gameMods, query, sort]);
  const enabledCount = gameMods.filter((mod) => mod.enabled).length;

  return (
    <div className="mods-screen">
      <PageHeading
        title="Mods"
        description={`${game.name} · ${enabledCount} activos de ${gameMods.length}`}
        action={
          <ActionButton icon="plus" onClick={onInstall}>
            Instalar mod
          </ActionButton>
        }
      />
      <div className="mod-toolbar">
        <div className="mod-filters" role="group" aria-label="Filtrar mods">
          {(
            [
              ["all", "Todos", gameMods.length],
              ["enabled", "Activos", enabledCount],
              [
                "attention",
                "Revisar",
                gameMods.filter((mod) => mod.conflict).length,
              ],
            ] as [ModFilter, string, number][]
          ).map(([id, label, count]) => (
            <button
              key={id}
              className={`filter-tab ${filter === id ? "is-selected" : ""}`}
              onClick={() => setFilter(id)}
              aria-pressed={filter === id}
            >
              {label}
              <span>{count}</span>
            </button>
          ))}
        </div>
        <div className="mod-tools">
          <label className="search-field">
            <Icon name="search" size={16} />
            <span className="sr-only">Buscar mods</span>
            <input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Buscar mods"
            />
            {query && (
              <button
                onClick={() => setQuery("")}
                aria-label="Limpiar búsqueda"
              >
                <Icon name="close" size={14} />
              </button>
            )}
          </label>
          <label className="sort-control">
            <Icon name="sort" size={16} />
            <span className="sr-only">Ordenar mods</span>
            <select
              value={sort}
              onChange={(event) => setSort(event.target.value)}
            >
              <option value="active">Activos primero</option>
              <option value="name">Nombre</option>
              <option value="character">Personaje</option>
            </select>
          </label>
        </div>
      </div>

      {visibleMods.length > 0 ? (
        <div className="mod-list" aria-label="Biblioteca de mods">
          {visibleMods.map((mod) => (
            <article
              className={`mod-row ${!mod.enabled ? "is-disabled" : ""}`}
              key={mod.id}
            >
              <button
                className={`mod-art mod-art--${mod.artwork}`}
                onClick={() => onDetails(mod)}
                aria-label={`Detalles de ${mod.name}`}
              >
                <span className="mod-art-orbit" />
                <span className="mod-art-mark" />
              </button>
              <div className="mod-main">
                <div className="mod-title-line">
                  <button
                    className="mod-title-button"
                    onClick={() => onDetails(mod)}
                  >
                    {mod.name}
                  </button>
                  {mod.conflict && (
                    <span className="warning-word">
                      <Icon name="warning" size={13} /> Conflicto potencial
                    </span>
                  )}
                </div>
                <div className="mod-byline">
                  <span>{mod.author}</span>
                  <i />
                  {mod.character}
                  <i />v{mod.version}
                  <i />
                  {mod.source}
                </div>
                {mod.conflict && (
                  <p className="mod-conflict-note">{mod.conflict}</p>
                )}
              </div>
              <div className="mod-toggle-wrap">
                <span
                  className={mod.enabled ? "mod-state-on" : "mod-state-off"}
                >
                  {mod.enabled ? "Activado" : "Desactivado"}
                </span>
                <Toggle
                  checked={mod.enabled}
                  onChange={() => onToggle(mod.id)}
                  label={`${mod.enabled ? "Desactivar" : "Activar"} ${mod.name}`}
                />
              </div>
            </article>
          ))}
        </div>
      ) : (
        <EmptyState
          title={
            query
              ? "No hay coincidencias"
              : filter === "enabled"
                ? "No hay mods activos"
                : "No hay mods por revisar"
          }
          detail={
            query
              ? "Prueba con otro nombre, autor o personaje."
              : "Puedes cambiar el filtro o volver a ver todos los mods."
          }
          action={
            query ? (
              <button className="text-link" onClick={() => setQuery("")}>
                Limpiar búsqueda
              </button>
            ) : (
              <button className="text-link" onClick={() => setFilter("all")}>
                Ver todos
              </button>
            )
          }
        />
      )}
      <div className="mod-list-footer">
        <span>Biblioteca local · datos de muestra</span>
        <button className="text-link" onClick={onInstall}>
          Añadir desde archivo
        </button>
      </div>
    </div>
  );
}

export function ProfilesScreen({
  game,
  profiles,
  activeId,
  selectedId,
  mods,
  onSelect,
  onActivate,
  onCreate,
  onDuplicate,
  onEdit,
}: {
  game: Game;
  profiles: MockProfile[];
  activeId: string;
  selectedId: string;
  mods: MockMod[];
  onSelect: (id: string) => void;
  onActivate: (id: string) => void;
  onCreate: () => void;
  onDuplicate: (profile: MockProfile) => void;
  onEdit: (profile: MockProfile) => void;
}) {
  const gameProfiles = profiles.filter((profile) => profile.gameId === game.id);
  const selected =
    gameProfiles.find((profile) => profile.id === selectedId) ??
    gameProfiles[0];
  const selectedMods = selected
    ? selected.modIds
        .map((id) => mods.find((mod) => mod.id === id))
        .filter((mod): mod is MockMod => Boolean(mod))
    : [];
  const currentActive = selected?.id === activeId;

  return (
    <div className="profiles-screen">
      <PageHeading
        title="Profiles"
        description={`Una configuración distinta para cada forma de jugar · ${game.name}`}
        action={
          <ActionButton icon="plus" onClick={onCreate}>
            Crear profile
          </ActionButton>
        }
      />
      {selected ? (
        <div className="profile-layout">
          <section className="profile-list" aria-label="Profiles disponibles">
            <p className="section-overline">
              MIS PROFILES <span>{gameProfiles.length}</span>
            </p>
            {gameProfiles.map((profile) => (
              <button
                className={`profile-list-row ${selected.id === profile.id ? "is-selected" : ""}`}
                key={profile.id}
                onClick={() => onSelect(profile.id)}
                aria-pressed={selected.id === profile.id}
              >
                <span
                  className={`profile-row-mark ${profile.id === activeId ? "is-active" : ""}`}
                />
                <span className="profile-row-copy">
                  <strong>{profile.name}</strong>
                  <small>
                    {profile.modIds.length} mods · {profile.lastUsed}
                  </small>
                </span>
                {selected.id === profile.id && (
                  <Icon name="chevron" size={16} />
                )}
              </button>
            ))}
            <div className="profile-list-note">
              <span className="profile-row-mark is-empty" />
              <span>Sin mods activos</span>
            </div>
          </section>
          <section
            className="profile-detail"
            aria-label={`Detalles del profile ${selected.name}`}
          >
            <div className="profile-detail-heading">
              <div>
                <div className="profile-detail-status">
                  {currentActive ? (
                    <StatusMark>En uso</StatusMark>
                  ) : (
                    <span className="status-mark status-mark--quiet">
                      <span className="status-mark-dot" />
                      Disponible
                    </span>
                  )}
                </div>
                <h2>{selected.name}</h2>
                <p>{selected.description}</p>
              </div>
              <span className="profile-detail-glyph">
                <Icon name="layers" size={22} />
              </span>
            </div>
            <div className="profile-detail-meta">
              <div>
                <small>JUEGO</small>
                <strong>{game.name}</strong>
              </div>
              <div>
                <small>MODS ACTIVOS</small>
                <strong>{selectedMods.length}</strong>
              </div>
              <div>
                <small>ÚLTIMO USO</small>
                <strong>{selected.lastUsed}</strong>
              </div>
            </div>
            <div className="profile-mods-preview">
              <div className="preview-heading">
                <span>Incluye</span>
                <span>{selectedMods.length} elementos</span>
              </div>
              {selectedMods.length ? (
                selectedMods.slice(0, 4).map((mod) => (
                  <div className="profile-mod-preview-row" key={mod.id}>
                    <span className={`tiny-art tiny-art--${mod.artwork}`} />
                    <span>{mod.name}</span>
                    <Icon name="check" size={15} />
                  </div>
                ))
              ) : (
                <p className="vanilla-note">
                  Este profile inicia el juego sin modificaciones activas.
                </p>
              )}
              {selectedMods.length > 4 && (
                <span className="more-items">
                  y {selectedMods.length - 4} más
                </span>
              )}
            </div>
            <div className="profile-actions">
              <ActionButton
                variant="primary"
                disabled={currentActive}
                onClick={() => onActivate(selected.id)}
              >
                {currentActive ? "Profile activo" : "Usar este profile"}
              </ActionButton>
              <ActionButton icon="copy" onClick={() => onDuplicate(selected)}>
                Duplicar
              </ActionButton>
              <button
                className="profile-edit-link"
                onClick={() => onEdit(selected)}
              >
                Editar contenido
              </button>
            </div>
          </section>
        </div>
      ) : (
        <EmptyState
          title="Crea tu primer profile"
          detail="Guarda combinaciones de mods para alternar entre ellas."
          action={
            <ActionButton icon="plus" onClick={onCreate}>
              Crear profile
            </ActionButton>
          }
        />
      )}
    </div>
  );
}

export function CompatibilityScreen({
  game,
  selectedRuntime,
  onSelectRuntime,
  onOpenDiagnostics,
}: {
  game: Game;
  selectedRuntime: string;
  onSelectRuntime: (id: string) => void;
  onOpenDiagnostics: () => void;
}) {
  const runtimes = [
    {
      id: "proton-experimental",
      name: "Proton Experimental",
      vendor: "Valve",
      version: "bleeding-edge",
      note: "Actualizaciones continuas",
    },
    {
      id: "ge-proton-9-23",
      name: "GE-Proton 9-23",
      vendor: "Custom",
      version: "9-23",
      note: "Compilación de la comunidad",
    },
    {
      id: "proton-10",
      name: "Proton 10",
      vendor: "Valve",
      version: "10.0-2",
      note: "Canal estable",
    },
  ];
  return (
    <div className="compatibility-screen">
      <PageHeading
        title="Compatibilidad"
        description={`Steam, Proton y el entorno de ${game.name}.`}
      />
      <div className="compatibility-layout">
        <section className="runtime-section">
          <div className="section-heading-row">
            <div>
              <h2>Runtime</h2>
              <p>Herramientas disponibles en este equipo.</p>
            </div>
            <span className="runtime-count">{runtimes.length} instalados</span>
          </div>
          <div className="runtime-list">
            {runtimes.map((runtime) => {
              const selected = selectedRuntime === runtime.id;
              return (
                <button
                  className={`runtime-row ${selected ? "is-selected" : ""}`}
                  key={runtime.id}
                  onClick={() => onSelectRuntime(runtime.id)}
                  aria-pressed={selected}
                >
                  <span className="runtime-symbol">
                    <Icon name="spark" size={18} />
                  </span>
                  <span className="runtime-main">
                    <strong>{runtime.name}</strong>
                    <small>
                      {runtime.vendor} <i /> {runtime.note}
                    </small>
                  </span>
                  <span className="runtime-version">{runtime.version}</span>
                  <span
                    className={
                      selected
                        ? "runtime-selected-label"
                        : "runtime-select-label"
                    }
                  >
                    {selected ? "Preferido" : "Elegir"}
                  </span>
                </button>
              );
            })}
          </div>
          <p className="runtime-footnote">
            La selección se guarda solo en este prototipo. LXMI todavía no
            cambia el runtime que usa Steam.
          </p>
        </section>

        <aside className="prefix-panel">
          <div className="prefix-panel-heading">
            <span className="prefix-icon">
              <Icon name="folder" size={18} />
            </span>
            <span>Wine prefix</span>
          </div>
          <div className="prefix-state">
            <span className="prefix-state-dot" />
            {game.prefix}
          </div>
          <p>
            Steam suele inicializarlo cuando el juego se abre por primera vez
            mediante su biblioteca. No hace falta configurarlo ahora.
          </p>
          <div className="prefix-foot">
            <span>compatdata</span>
            <span className="prefix-path-placeholder">
              Ruta disponible en detalles
            </span>
          </div>
          <button className="text-link" onClick={onOpenDiagnostics}>
            Ver detalles técnicos <Icon name="arrow" size={14} />
          </button>
        </aside>
      </div>

      <section className="compatibility-bottom">
        <div className="compat-bottom-copy">
          <span className="compat-bottom-icon">
            <Icon name="compatibility" size={17} />
          </span>
          <span>
            <strong>WWMI</strong>
            <small>Administrador de mods para Wuthering Waves</small>
          </span>
        </div>
        <StatusMark>Configurado</StatusMark>
        <span className="compat-bottom-note">
          Estado ilustrativo para el prototipo
        </span>
      </section>
    </div>
  );
}

type SettingsSection =
  | "general"
  | "game"
  | "launcher"
  | "compatibility"
  | "wwmi"
  | "mods"
  | "advanced";

const settingsSections: { id: SettingsSection; label: string }[] = [
  { id: "general", label: "General" },
  { id: "game", label: "Juego" },
  { id: "launcher", label: "Launcher" },
  { id: "compatibility", label: "Compatibilidad" },
  { id: "wwmi", label: "XXMI / WWMI" },
  { id: "mods", label: "Mods" },
  { id: "advanced", label: "Avanzado" },
];

export function SettingsScreen({
  game,
  onOpenDiagnostics,
  onToast,
}: {
  game: Game;
  onOpenDiagnostics: () => void;
  onToast: (message: string) => void;
}) {
  const [section, setSection] = useState<SettingsSection>("general");
  const [launchOnLogin, setLaunchOnLogin] = useState(false);
  const [closeToTray, setCloseToTray] = useState(true);
  const [confirmSwitch, setConfirmSwitch] = useState(true);
  return (
    <div className="settings-screen">
      <PageHeading
        title="Configuración"
        description="Ajusta LXMI a tu forma de jugar."
      />
      <div className="settings-layout">
        <nav className="settings-nav" aria-label="Secciones de configuración">
          {settingsSections.map((item) => (
            <button
              key={item.id}
              className={section === item.id ? "is-selected" : ""}
              onClick={() => setSection(item.id)}
              aria-pressed={section === item.id}
            >
              {item.label}
              <Icon name="chevron" size={14} />
            </button>
          ))}
        </nav>
        <section className="settings-content">
          <div className="settings-section-heading">
            <h2>
              {settingsSections.find((item) => item.id === section)?.label}
            </h2>
            <p>
              {section === "general"
                ? "Preferencias de la aplicación en este equipo."
                : `Preferencias de ${game.name}.`}
            </p>
          </div>
          {section === "general" && (
            <div className="settings-fields">
              <SettingsSelect
                label="Idioma"
                value="Español"
                options={["Español", "English"]}
              />
              <SettingsSelect
                label="Apariencia"
                value="Oscura"
                options={["Oscura", "Sistema"]}
              />
              <SettingsToggle
                title="Iniciar LXMI al entrar al escritorio"
                detail="LXMI permanece en segundo plano después de iniciar sesión."
                checked={launchOnLogin}
                onChange={() => setLaunchOnLogin(!launchOnLogin)}
              />
              <SettingsToggle
                title="Minimizar al cerrar ventana"
                detail="Mantén LXMI disponible desde la bandeja del sistema."
                checked={closeToTray}
                onChange={() => setCloseToTray(!closeToTray)}
              />
            </div>
          )}
          {section === "game" && (
            <div className="settings-fields">
              <SettingsSelect
                label="Juego asociado"
                value={game.name}
                options={[game.name, "Detectar al iniciar"]}
              />
              <SettingsPath
                label="Carpeta del juego"
                value={
                  game.id === "wuwa"
                    ? "/home/usuario/.local/share/Steam/steamapps/common/Wuthering Waves"
                    : "/home/usuario/Games/ZenlessZoneZero"
                }
                onAction={() =>
                  onToast("Abrir carpeta es una acción simulada en esta vista.")
                }
              />
              <SettingsToggle
                title="Recordar último profile"
                detail="Al abrir el juego, conservar el profile que usaste la última vez."
                checked={confirmSwitch}
                onChange={() => setConfirmSwitch(!confirmSwitch)}
              />
            </div>
          )}
          {section === "launcher" && (
            <div className="settings-fields">
              <SettingsToggle
                title="Mostrar confirmación antes de jugar"
                detail="Permite revisar el profile activo antes de abrir Steam."
                checked={confirmSwitch}
                onChange={() => setConfirmSwitch(!confirmSwitch)}
              />
              <SettingsSelect
                label="Al salir del juego"
                value="Volver a LXMI"
                options={["Volver a LXMI", "Mantener en segundo plano"]}
              />
            </div>
          )}
          {section === "compatibility" && (
            <div className="settings-fields">
              <SettingsSelect
                label="Runtime preferido"
                value={game.runtime}
                options={[game.runtime, "GE-Proton 9-23", "Proton 10"]}
              />
              <SettingsPath
                label="Wine prefix"
                value={
                  game.prefix === "Aún no inicializado"
                    ? "Todavía no creado"
                    : "/home/usuario/.local/share/Steam/steamapps/compatdata/..."
                }
                onAction={onOpenDiagnostics}
              />
              <p className="settings-note">
                Elegir un runtime aquí es solo una interacción de muestra. La
                app actual no modifica la preferencia de Steam.
              </p>
            </div>
          )}
          {section === "wwmi" && (
            <div className="settings-fields">
              <SettingsToggle
                title="Gestionar WWMI desde LXMI"
                detail="Mantener el runtime y sus archivos configurados para este juego."
                checked
                onChange={() =>
                  onToast("El control WWMI todavía no está conectado.")
                }
              />
              <SettingsPath
                label="Directorio del runtime"
                value="Gestionado por el prototipo"
                onAction={onOpenDiagnostics}
              />
            </div>
          )}
          {section === "mods" && (
            <div className="settings-fields">
              <SettingsPath
                label="Biblioteca de mods"
                value="~/.local/share/lxmi/library/wuwa"
                onAction={() =>
                  onToast("La biblioteca de mods es un dato de muestra.")
                }
              />
              <SettingsToggle
                title="Avisar sobre conflictos potenciales"
                detail="Mostrar advertencias cuando dos mods declaran recursos coincidentes."
                checked
                onChange={() =>
                  onToast("Preferencia simulada para el prototipo.")
                }
              />
            </div>
          )}
          {section === "advanced" && (
            <div className="settings-fields">
              <div className="advanced-setting">
                <span>
                  <strong>Información de diagnóstico</strong>
                  <small>
                    Manifiestos, rutas, runtime y registros recientes.
                  </small>
                </span>
                <ActionButton icon="diagnostics" onClick={onOpenDiagnostics}>
                  Abrir diagnósticos
                </ActionButton>
              </div>
              <SettingsToggle
                title="Incluir datos técnicos al exportar un informe"
                detail="No incluye archivos personales ni el contenido de tus mods."
                checked={false}
                onChange={() =>
                  onToast("Preferencia simulada para el prototipo.")
                }
              />
            </div>
          )}
          <div className="settings-save-note">
            <span className="save-note-dot" /> Los cambios se muestran solo en
            esta demostración.
          </div>
        </section>
      </div>
    </div>
  );
}

function SettingsSelect({
  label,
  value,
  options,
}: {
  label: string;
  value: string;
  options: string[];
}) {
  const [selected, setSelected] = useState(value);
  return (
    <label className="settings-row settings-select-row">
      <span className="settings-label">{label}</span>
      <select
        value={selected}
        onChange={(event) => setSelected(event.target.value)}
      >
        {options.map((option) => (
          <option key={option}>{option}</option>
        ))}
      </select>
    </label>
  );
}

function SettingsPath({
  label,
  value,
  onAction,
}: {
  label: string;
  value: string;
  onAction: () => void;
}) {
  return (
    <div className="settings-row settings-path-row">
      <span className="settings-label">{label}</span>
      <div>
        <code>{value}</code>
        <button className="text-link" onClick={onAction}>
          Ver <Icon name="arrow" size={13} />
        </button>
      </div>
    </div>
  );
}

function SettingsToggle({
  title,
  detail,
  checked,
  onChange,
}: {
  title: string;
  detail: string;
  checked: boolean;
  onChange: () => void;
}) {
  return (
    <div className="settings-row settings-toggle-row">
      <span>
        <strong>{title}</strong>
        <small>{detail}</small>
      </span>
      <Toggle checked={checked} onChange={onChange} label={title} />
    </div>
  );
}

type DiagnosticTab = "manifest" | "prefix" | "runtime" | "environment" | "logs";

export function DiagnosticsScreen({
  game,
  onToast,
}: {
  game: Game;
  onToast: (message: string) => void;
}) {
  const [tab, setTab] = useState<DiagnosticTab>("manifest");
  const tabs: { id: DiagnosticTab; label: string }[] = [
    { id: "manifest", label: "Steam manifest" },
    { id: "prefix", label: "Wine prefix" },
    { id: "runtime", label: "Runtime" },
    { id: "environment", label: "Entorno" },
    { id: "logs", label: "Registros" },
  ];
  const values: Record<
    DiagnosticTab,
    { title: string; subtitle: string; rows: [string, string][] }
  > = {
    manifest: {
      title: "Steam manifest",
      subtitle:
        "Datos ilustrativos del manifiesto de la instalación seleccionada.",
      rows: [
        ["Aplicación", game.name],
        ["AppID", game.appId ?? "Dato de muestra"],
        ["Steam Library", "/home/usuario/.local/share/Steam"],
        [
          "Instalación",
          "/home/usuario/.local/share/Steam/steamapps/common/Wuthering Waves",
        ],
        ["Estado", "Directorio detectado"],
      ],
    },
    prefix: {
      title: "Wine prefix",
      subtitle: "El prefix es independiente de la carpeta del juego.",
      rows: [
        ["Estado", game.prefix],
        [
          "Compatdata",
          `/home/usuario/.local/share/Steam/steamapps/compatdata/${game.appId ?? "—"}`,
        ],
        [
          "Directorio pfx",
          game.prefix === "Aún no inicializado"
            ? "No encontrado"
            : "Ruta de muestra",
        ],
        ["Último acceso", "Sin dato de muestra"],
      ],
    },
    runtime: {
      title: "Runtime",
      subtitle:
        "Herramientas que el prototipo muestra como instaladas en el sistema.",
      rows: [
        ["Preferido en LXMI", game.runtime],
        ["Origen", game.runtimeSource],
        [
          "Versión",
          game.runtime === "Proton Experimental" ? "bleeding-edge" : "9-23",
        ],
        ["Selección efectiva en Steam", "No determinada"],
      ],
    },
    environment: {
      title: "Entorno",
      subtitle: "Valores de muestra; LXMI no los exporta ni modifica.",
      rows: [
        ["Sistema", "Ubuntu Linux x86_64"],
        ["Steam root", "/home/usuario/.steam/steam"],
        ["XDG data", "/home/usuario/.local/share"],
        ["Variable de compatibilidad", "No definida por LXMI"],
      ],
    },
    logs: {
      title: "Actividad reciente",
      subtitle: "Entradas inventadas para mostrar el formato de diagnóstico.",
      rows: [
        ["19:42:08", "Steam discovery completed"],
        ["19:42:07", "Proton Experimental found"],
        ["19:42:07", "Game manifest parsed"],
        ["19:42:06", "Wuthering Waves matched by AppID"],
      ],
    },
  };
  const current = values[tab];
  return (
    <div className="diagnostics-screen">
      <PageHeading
        title="Diagnósticos"
        description="Detalle técnico del juego seleccionado. Los valores de esta vista son de muestra."
      />
      <div
        className="diagnostic-tabs"
        role="tablist"
        aria-label="Categorías de diagnóstico"
      >
        {tabs.map((item) => (
          <button
            key={item.id}
            className={tab === item.id ? "is-selected" : ""}
            onClick={() => setTab(item.id)}
            role="tab"
            aria-selected={tab === item.id}
            aria-controls="diagnostic-panel"
          >
            {item.label}
          </button>
        ))}
      </div>
      <section
        className="diagnostic-panel"
        id="diagnostic-panel"
        role="tabpanel"
        aria-labelledby="diagnostic-title"
      >
        <div className="diagnostic-panel-heading">
          <div>
            <h2 id="diagnostic-title">{current.title}</h2>
            <p>{current.subtitle}</p>
          </div>
          <ActionButton
            icon="copy"
            onClick={() =>
              onToast("Copiar es una interacción simulada en esta vista.")
            }
          >
            Copiar valores
          </ActionButton>
        </div>
        {tab === "logs" && (
          <div className="diagnostic-error" role="alert">
            <span className="diagnostic-error-icon">
              <Icon name="warning" size={16} />
            </span>
            <span className="diagnostic-error-copy">
              <strong>Un manifiesto no se pudo leer</strong>
              <small>
                El error quedó aislado; el resto del escaneo de muestra terminó.
              </small>
            </span>
            <code>IO · no fatal</code>
            <button
              className="text-link"
              onClick={() =>
                onToast("Volver a escanear es una acción simulada.")
              }
            >
              Reintentar
            </button>
          </div>
        )}
        <dl className="diagnostic-rows">
          {current.rows.map(([label, value], index) => (
            <div key={`${label}-${index}`}>
              <dt>{label}</dt>
              <dd>{value}</dd>
            </div>
          ))}
        </dl>
        <div className="diagnostic-footer">
          <span>
            <span className="sample-indicator" /> Solo lectura · datos de
            demostración
          </span>
          <button
            className="text-link"
            onClick={() =>
              onToast("Abrir carpeta es una interacción simulada.")
            }
          >
            Abrir ubicación <Icon name="external" size={14} />
          </button>
        </div>
      </section>
    </div>
  );
}
