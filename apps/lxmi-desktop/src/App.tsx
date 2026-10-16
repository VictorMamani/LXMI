import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { version } from "../package.json";
import { ActionButton, LauncherBar, type ScreenId } from "./components/Chrome";
import { Icon } from "./components/Icon";
import DiagnosticsApp from "./DiagnosticsApp";
import { games, type Game, type GameId } from "./gameCatalog";
import ModLaunchCard from "./ModLaunchCard";
import ModManagerPanel from "./ModManagerPanel";
import RuntimePanel from "./RuntimePanel";
import {
  errorMessage,
  type ModLibrary,
  type RuntimeSources,
  type RuntimeStatus,
} from "./runtimeTypes";

const emptySources: RuntimeSources = { importer: "", libraries: "" };
async function load(game: Game, sources: RuntimeSources) {
  return Promise.all([
    invoke<ModLibrary>("list_mod_library", { gameId: game.backendId }),
    invoke<RuntimeStatus>("inspect_game_mod_runtime", {
      gameId: game.backendId,
      importerId: sources.importer || null,
      librariesId: sources.libraries || null,
    }),
  ]);
}

export default function App() {
  const [gameId, setGameId] = useState<GameId>("wuwa");
  const [screen, setScreen] = useState<ScreenId>("home");
  const game = games.find((item) => item.id === gameId) ?? games[0];
  return (
    <div className="lxmi-app" data-game={game.id} data-screen={screen}>
      <div className="launcher-backdrop" aria-hidden="true">
        <img key={game.id} src={game.art} alt="" />
      </div>
      <LauncherBar
        games={games}
        selectedGame={gameId}
        screen={screen}
        onSelectGame={setGameId}
        onNavigate={setScreen}
      />
      <main className="workspace-view" id="main-content">
        <GameWorkspace
          key={gameId}
          game={game}
          screen={screen}
          onNavigate={setScreen}
          onGameChange={(id) => {
            const selected = games.find((item) => item.backendId === id);
            if (selected) setGameId(selected.id);
          }}
        />
      </main>
    </div>
  );
}

function GameWorkspace({
  game,
  screen,
  onNavigate,
  onGameChange,
}: {
  game: Game;
  screen: ScreenId;
  onNavigate: (screen: ScreenId) => void;
  onGameChange: (id: string) => void;
}) {
  const [library, setLibrary] = useState<ModLibrary | null>(null);
  const [runtime, setRuntime] = useState<RuntimeStatus | null>(null);
  const [sources, setSources] = useState(emptySources);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const [loading, setLoading] = useState(true);
  const [launching, setLaunching] = useState(false);
  const libraryChanged = useCallback(
    (value: ModLibrary) => setLibrary(value),
    [],
  );

  useEffect(() => {
    let cancelled = false;
    load(game, emptySources)
      .then(async ([mods, status]) => {
        let nextStatus = status;
        const current = status.selection.current;
        const selected = current
          ? {
              importer: current.integration_package_id,
              libraries: current.libraries_package_id,
            }
          : emptySources;
        if (current)
          nextStatus = await invoke<RuntimeStatus>("inspect_game_mod_runtime", {
            gameId: game.backendId,
            importerId: selected.importer,
            librariesId: selected.libraries,
          });
        if (!cancelled) {
          setLibrary(mods);
          setRuntime(nextStatus);
          setSources(selected);
        }
      })
      .catch((reason: unknown) => {
        if (!cancelled) setError(errorMessage(reason));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [game]);

  async function refresh(nextSources = sources) {
    setLoading(true);
    setError("");
    try {
      const [mods, status] = await load(game, nextSources);
      setLibrary(mods);
      setRuntime(status);
    } catch (reason) {
      setError(errorMessage(reason));
      throw reason;
    } finally {
      setLoading(false);
    }
  }
  async function changeSources(next: RuntimeSources) {
    setSources(next);
    await refresh(next);
  }
  async function launch() {
    setLaunching(true);
    setError("");
    setMessage("");
    try {
      await invoke("launch_game_without_mods", { gameId: game.backendId });
      setMessage(
        "Solicitud enviada a Steam. Este inicio no carga mods desde LXMI.",
      );
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setLaunching(false);
    }
  }
  const current = runtime?.selection.current;
  const profileName = current?.profile_name ?? "Sin perfil aplicado";
  const count = current?.mod_count ?? 0;

  return (
    <div className={`game-workspace game-workspace--${game.id}`}>
      {error && (
        <div className="workspace-error" role="alert">
          <p>{error}</p>
          <button
            className="secondary-button"
            type="button"
            disabled={loading}
            onClick={() => void refresh().catch(() => {})}
          >
            Volver a consultar
          </button>
        </div>
      )}
      {message && (
        <p className="workspace-feedback" role="status">
          {message}
        </p>
      )}
      {screen === "home" && (
        <div className="home-screen">
          <section
            className="launch-stage"
            aria-label={`${game.name}, resumen de juego`}
          >
            <div className="stage-title">
              <span className="importer-badge">{game.importer}</span>
              <h1>{game.name}</h1>
              <p>{game.subtitle}</p>
              <ul className="stage-facts">
                <li
                  data-state={
                    loading ? "loading" : runtime?.installed ? "good" : "quiet"
                  }
                >
                  {loading
                    ? "Consultando instalación…"
                    : runtime?.installed
                      ? "Instalación Steam detectada"
                      : "Instalación sin verificar"}
                </li>
                <li data-state={runtime?.base_runtime ? "good" : "quiet"}>
                  {current
                    ? `${game.importer} con perfil aplicado`
                    : runtime?.base_runtime
                      ? `${game.importer} preparado`
                      : `${game.importer} por preparar`}
                </li>
                <li data-state="quiet">
                  {library
                    ? `${library.mods.length} mods · ${library.profiles.length} ${library.profiles.length === 1 ? "perfil" : "perfiles"}`
                    : "Leyendo biblioteca…"}
                </li>
              </ul>
            </div>
            <div className="launch-dock">
              <button
                className="dock-profile"
                onClick={() => onNavigate("profiles")}
                aria-label="Ver perfiles"
              >
                <span className="profile-glyph">
                  <Icon name="layers" size={17} />
                </span>
                <span>
                  <small>Perfil seleccionado</small>
                  <strong>{profileName}</strong>
                </span>
                <span className="dock-count">
                  {count}
                  <small>{count === 1 ? "mod" : "mods"}</small>
                </span>
                <Icon name="chevron" size={16} />
              </button>
              <ModLaunchCard
                gameBackendId={game.backendId}
                runtime={runtime}
                loading={loading}
                launchingWithout={launching}
                onLaunchWithout={() => void launch()}
              />
              <button
                className="text-link dock-link"
                onClick={() => onNavigate("compatibility")}
              >
                Configurar carga de mods
              </button>
            </div>
          </section>
          <footer className="stage-footer">
            <span>LXMI {version}</span>
            {runtime?.importer_packages[0]?.version && (
              <span>
                {game.importer} {runtime.importer_packages[0].version.raw}
              </span>
            )}
            {runtime?.libraries_packages[0]?.version && (
              <span>
                XXMI Libraries {runtime.libraries_packages[0].version.raw}
              </span>
            )}
            <span>Copias privadas · el juego no se modifica</span>
          </footer>
        </div>
      )}
      {(screen === "mods" || screen === "profiles") && (
        <div className="manager-screen">
          <div className="page-heading">
            <div>
              <h1>{screen === "mods" ? "Mods" : "Perfiles"}</h1>
              <p>{game.name} · biblioteca y selecciones guardadas</p>
            </div>
            {screen === "profiles" && (
              <ActionButton
                icon="compatibility"
                onClick={() => onNavigate("compatibility")}
              >
                Aplicar al runtime
              </ActionButton>
            )}
          </div>
          <ModManagerPanel
            gameId={game.backendId}
            onGameChange={onGameChange}
            onLibraryChange={libraryChanged}
          />
          {screen === "profiles" && (
            <RuntimePanel
              game={game}
              library={library}
              status={runtime}
              sources={sources}
              onSourcesChange={changeSources}
              onRefresh={refresh}
            />
          )}
        </div>
      )}
      {screen === "compatibility" && (
        <div className="compatibility-screen">
          <div className="page-heading">
            <div>
              <h1>Compatibilidad</h1>
              <p>{game.name} · paquetes, runtime y selección reversible</p>
            </div>
            <ActionButton
              icon="monitor"
              onClick={() => onNavigate("diagnostics")}
            >
              Diagnósticos
            </ActionButton>
          </div>
          <RuntimePanel
            game={game}
            library={library}
            status={runtime}
            sources={sources}
            onSourcesChange={changeSources}
            onRefresh={refresh}
          />
        </div>
      )}
      {screen === "diagnostics" && <DiagnosticsApp />}
      {screen === "settings" && (
        <section className="settings-screen panel">
          <div className="page-heading">
            <div>
              <h1>Configuración</h1>
              <p>Preferencias y límites actuales de LXMI</p>
            </div>
          </div>
          <dl className="settings-facts">
            <dt>Versión</dt>
            <dd>{version}</dd>
            <dt>Juego seleccionado</dt>
            <dd>{game.name}</dd>
            <dt>Gestión de archivos</dt>
            <dd>Copias privadas con inventario y SHA-256</dd>
            <dt>Perfil para el próximo inicio</dt>
            <dd>{profileName}</dd>
            <dt>Cambios en el juego</dt>
            <dd>
              Ninguno. «Jugar con mods» usa una copia privada del perfil y la
              carga oficial de XXMI; la instalación y Steam no se modifican.
            </dd>
            <dt>Selección sin recarga</dt>
            <dd>Pendiente de integración con el runtime.</dd>
          </dl>
          <ActionButton
            icon="compatibility"
            onClick={() => onNavigate("compatibility")}
          >
            Configurar runtime y perfil
          </ActionButton>
        </section>
      )}
    </div>
  );
}
