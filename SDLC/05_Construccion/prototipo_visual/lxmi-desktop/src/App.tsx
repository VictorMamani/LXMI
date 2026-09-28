import { useEffect, useMemo, useRef, useState } from "react";
import type {
  FormEvent,
  KeyboardEvent as ReactKeyboardEvent,
  ReactNode,
} from "react";
import {
  initialMods,
  initialProfiles,
  mockGames,
  mockRuntimes,
  type GameId,
  type MockMod,
  type MockProfile,
} from "./data";
import {
  ActionButton,
  GameRail,
  IconButton,
  Toggle,
  Toast,
  WorkspaceHeader,
  type ScreenId,
} from "./components/Chrome";
import { Icon } from "./components/Icon";
import {
  CompatibilityScreen,
  DiagnosticsScreen,
  HomeScreen,
  ModsScreen,
  ProfilesScreen,
  SettingsScreen,
} from "./Screens";

type DialogState =
  | { type: "add-game" }
  | { type: "install-mod" }
  | { type: "create-profile" }
  | { type: "mod-details"; mod: MockMod }
  | { type: "edit-profile"; profileId: string }
  | null;

function Dialog({
  title,
  children,
  onClose,
  size = "regular",
}: {
  title: string;
  children: ReactNode;
  onClose: () => void;
  size?: "regular" | "wide";
}) {
  const dialogRef = useRef<HTMLElement>(null);
  const returnFocusRef = useRef(
    document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null,
  );

  useEffect(() => {
    const returnFocus = returnFocusRef.current;
    const dialog = dialogRef.current;
    const initialFocus =
      dialog?.querySelector<HTMLElement>("[data-autofocus]") ??
      dialog?.querySelector<HTMLElement>(
        "button:not([disabled]), input:not([disabled]), select:not([disabled])",
      );
    initialFocus?.focus();

    return () => returnFocus?.focus();
  }, []);

  function keepFocusInside(event: ReactKeyboardEvent<HTMLElement>) {
    if (event.key !== "Tab") return;
    const focusable = Array.from(
      dialogRef.current?.querySelectorAll<HTMLElement>(
        "button:not([disabled]), input:not([disabled]), select:not([disabled]), [href], [tabindex]:not([tabindex='-1'])",
      ) ?? [],
    );
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  return (
    <div
      className="dialog-scrim"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <section
        className={`dialog dialog--${size}`}
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="dialog-title"
        tabIndex={-1}
        onKeyDown={keepFocusInside}
      >
        <div className="dialog-heading">
          <div>
            <h2 id="dialog-title">{title}</h2>
          </div>
          <IconButton icon="close" label="Cerrar ventana" onClick={onClose} />
        </div>
        {children}
      </section>
    </div>
  );
}

export default function App() {
  const [screen, setScreen] = useState<ScreenId>("home");
  const [gameId, setGameId] = useState<GameId>("wuwa");
  const [mods, setMods] = useState(initialMods);
  const [profiles, setProfiles] = useState(initialProfiles);
  const [activeProfiles, setActiveProfiles] = useState<Record<GameId, string>>({
    wuwa: "default",
    zzz: "new-eridu",
  });
  const [selectedProfile, setSelectedProfile] = useState("default");
  const [runtimeByGame, setRuntimeByGame] = useState<Record<GameId, string>>({
    wuwa: "proton-experimental",
    zzz: "ge-proton-9-23",
  });
  const [launching, setLaunching] = useState(false);
  const [dialog, setDialog] = useState<DialogState>(null);
  const [newProfileName, setNewProfileName] = useState("");
  const [toast, setToast] = useState("");
  const [profileSequence, setProfileSequence] = useState(1);
  const [editingModIds, setEditingModIds] = useState<string[]>([]);

  const baseGame = mockGames.find((game) => game.id === gameId) ?? mockGames[0];
  const runtime =
    mockRuntimes.find((item) => item.id === runtimeByGame[gameId])?.name ??
    baseGame.runtime;
  const currentGame = useMemo(
    () => ({ ...baseGame, runtime }),
    [baseGame, runtime],
  );
  const activeProfileId = activeProfiles[gameId];
  const activeProfile =
    profiles.find((profile) => profile.id === activeProfileId)?.name ??
    "Default";
  const activeMods = mods.filter(
    (mod) => mod.gameId === gameId && mod.enabled,
  ).length;
  const gameProfiles = profiles.filter((profile) => profile.gameId === gameId);
  const editingProfile =
    dialog?.type === "edit-profile"
      ? profiles.find((profile) => profile.id === dialog.profileId)
      : undefined;
  const editingMods = editingProfile
    ? mods.filter((mod) => mod.gameId === editingProfile.gameId)
    : [];

  useEffect(() => {
    if (!dialog) return undefined;
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") setDialog(null);
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [dialog]);

  useEffect(() => {
    if (!toast) return undefined;
    const timeout = window.setTimeout(() => setToast(""), 3200);
    return () => window.clearTimeout(timeout);
  }, [toast]);

  function showToast(message: string) {
    setToast(message);
  }

  function changeGame(id: GameId) {
    setGameId(id);
    setSelectedProfile(activeProfiles[id]);
    setScreen("home");
  }

  function createProfile(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const trimmedName = newProfileName.trim();
    if (!trimmedName) return;
    const nextId = `custom-${profileSequence}`;
    const created: MockProfile = {
      id: nextId,
      gameId,
      name: trimmedName,
      description: "Un profile creado durante esta demostración.",
      lastUsed: "Nunca",
      modIds: mods
        .filter((mod) => mod.gameId === gameId && mod.enabled)
        .map((mod) => mod.id),
    };
    setProfiles((current) => [...current, created]);
    setProfileSequence((current) => current + 1);
    setSelectedProfile(nextId);
    setNewProfileName("");
    setDialog(null);
    setScreen("profiles");
    showToast("Profile añadido a los datos de muestra.");
  }

  function duplicateProfile(profile: MockProfile) {
    const nextId = `custom-${profileSequence}`;
    const copy: MockProfile = {
      ...profile,
      id: nextId,
      name: `${profile.name} copia`,
      description: `Copia de ${profile.name}.`,
      lastUsed: "Nunca",
      modIds: [...profile.modIds],
    };
    setProfiles((current) => [...current, copy]);
    setProfileSequence((current) => current + 1);
    setSelectedProfile(nextId);
    setDialog(null);
    showToast("Copia creada en esta sesión de demostración.");
  }

  function openProfileEditor(profile: MockProfile) {
    setEditingModIds([...profile.modIds]);
    setDialog({ type: "edit-profile", profileId: profile.id });
  }

  function saveProfileEditor() {
    if (dialog?.type !== "edit-profile") return;
    const profile = profiles.find((item) => item.id === dialog.profileId);
    if (!profile) return;
    setProfiles((current) =>
      current.map((item) =>
        item.id === profile.id ? { ...item, modIds: [...editingModIds] } : item,
      ),
    );
    if (activeProfiles[profile.gameId] === profile.id) {
      setMods((current) =>
        current.map((mod) =>
          mod.gameId === profile.gameId
            ? { ...mod, enabled: editingModIds.includes(mod.id) }
            : mod,
        ),
      );
    }
    setDialog(null);
    showToast("Contenido del profile actualizado en esta sesión.");
  }

  function activateProfile(id: string) {
    const profile = profiles.find((item) => item.id === id);
    if (!profile) return;
    setActiveProfiles((current) => ({ ...current, [gameId]: id }));
    setSelectedProfile(id);
    setMods((current) =>
      current.map((mod) =>
        mod.gameId === gameId
          ? { ...mod, enabled: profile.modIds.includes(mod.id) }
          : mod,
      ),
    );
    showToast(`${profile.name} está activo en el prototipo.`);
  }

  function toggleMod(id: string) {
    setMods((current) =>
      current.map((mod) =>
        mod.id === id ? { ...mod, enabled: !mod.enabled } : mod,
      ),
    );
  }

  function startGame() {
    if (launching) return;
    setLaunching(true);
    window.setTimeout(() => {
      setLaunching(false);
      showToast("La acción de jugar es una simulación visual.");
    }, 900);
  }

  function renderScreen() {
    if (screen === "mods") {
      return (
        <ModsScreen
          game={currentGame}
          mods={mods}
          onToggle={toggleMod}
          onInstall={() => setDialog({ type: "install-mod" })}
          onDetails={(mod) => setDialog({ type: "mod-details", mod })}
        />
      );
    }
    if (screen === "profiles") {
      return (
        <ProfilesScreen
          game={currentGame}
          profiles={profiles}
          activeId={activeProfileId}
          selectedId={selectedProfile}
          mods={mods}
          onSelect={setSelectedProfile}
          onActivate={activateProfile}
          onCreate={() => {
            setNewProfileName("");
            setDialog({ type: "create-profile" });
          }}
          onDuplicate={duplicateProfile}
          onEdit={openProfileEditor}
        />
      );
    }
    if (screen === "compatibility") {
      return (
        <CompatibilityScreen
          game={currentGame}
          selectedRuntime={runtimeByGame[gameId]}
          onSelectRuntime={(id) => {
            setRuntimeByGame((current) => ({ ...current, [gameId]: id }));
            showToast("Runtime preferido actualizado solo en el prototipo.");
          }}
          onOpenDiagnostics={() => setScreen("diagnostics")}
        />
      );
    }
    if (screen === "settings") {
      return (
        <SettingsScreen
          game={currentGame}
          onOpenDiagnostics={() => setScreen("diagnostics")}
          onToast={showToast}
        />
      );
    }
    if (screen === "diagnostics")
      return <DiagnosticsScreen game={currentGame} onToast={showToast} />;
    return (
      <HomeScreen
        game={currentGame}
        activeProfile={activeProfile}
        activeMods={activeMods}
        launching={launching}
        onPlay={startGame}
        onOpenMods={() => setScreen("mods")}
        onOpenProfiles={() => setScreen("profiles")}
      />
    );
  }

  return (
    <div className={`lxmi-app lxmi-app--${gameId}`}>
      <GameRail
        games={mockGames}
        selectedGame={gameId}
        onSelectGame={changeGame}
        onAddGame={() => setDialog({ type: "add-game" })}
        onNavigate={setScreen}
      />
      <div className="workspace">
        <WorkspaceHeader
          game={currentGame}
          screen={screen}
          onNavigate={setScreen}
        />
        <main className={`workspace-view workspace-view--${screen}`}>
          {renderScreen()}
        </main>
        <footer className="workspace-statusbar">
          <span className="statusbar-left">
            <span className="statusbar-mark" /> Prototipo visual
          </span>
          <span>
            Datos, rutas y acciones de muestra · no conectados a Steam
          </span>
          <span className="statusbar-version">LXMI design study</span>
        </footer>
      </div>

      {toast && <Toast message={toast} onClose={() => setToast("")} />}
      {dialog?.type === "add-game" && (
        <Dialog title="Añadir juego" onClose={() => setDialog(null)}>
          <p className="dialog-description">
            Los juegos detectados aparecen en el rail. En esta demostración
            puedes cambiar entre los dos espacios de muestra.
          </p>
          <div className="dialog-game-choice">
            {mockGames.map((game) => (
              <button
                key={game.id}
                className="dialog-game-row"
                onClick={() => {
                  changeGame(game.id);
                  setDialog(null);
                }}
              >
                <span className={`dialog-game-art dialog-game-art--${game.id}`}>
                  <img src={game.art} alt="" />
                </span>
                <span>
                  <strong>{game.name}</strong>
                  <small>
                    {game.id === gameId
                      ? "Seleccionado"
                      : "Abrir juego de muestra"}
                  </small>
                </span>
                <Icon name="chevron" size={16} />
              </button>
            ))}
          </div>
          <div className="dialog-note">
            <span className="sample-indicator" /> El escaneo real de Steam
            pertenece a la aplicación LXMI, no a este prototipo.
          </div>
        </Dialog>
      )}
      {dialog?.type === "install-mod" && (
        <Dialog
          title="Instalar mod"
          onClose={() => setDialog(null)}
          size="wide"
        >
          <p className="dialog-description">
            Añade un archivo de mod a la biblioteca de {currentGame.name}.
          </p>
          <div className="drop-zone">
            <span className="drop-zone-icon">
              <Icon name="plus" size={19} />
            </span>
            <strong>Arrastra un archivo aquí</strong>
            <span>ZIP, 7z · solo una representación visual</span>
            <ActionButton
              onClick={() =>
                showToast(
                  "La selección de archivos no está conectada en el prototipo.",
                )
              }
            >
              Elegir archivo
            </ActionButton>
          </div>
          <div className="dialog-footer">
            <span>No se extrae ni se guarda ningún archivo.</span>
            <ActionButton variant="quiet" onClick={() => setDialog(null)}>
              Cerrar
            </ActionButton>
          </div>
        </Dialog>
      )}
      {dialog?.type === "create-profile" && (
        <Dialog title="Crear profile" onClose={() => setDialog(null)}>
          <p className="dialog-description">
            Guarda los mods activos de {currentGame.name} bajo otro nombre.
          </p>
          <form className="profile-create-form" onSubmit={createProfile}>
            <label htmlFor="profile-name">Nombre</label>
            <input
              id="profile-name"
              value={newProfileName}
              onChange={(event) => setNewProfileName(event.target.value)}
              placeholder="Por ejemplo, Exploración"
              autoFocus
              data-autofocus
              required
              maxLength={32}
            />
            <div className="dialog-form-meta">
              <span>{activeMods} mods se incluirán</span>
              <span>Solo datos locales de muestra</span>
            </div>
            <div className="dialog-footer">
              <ActionButton variant="quiet" onClick={() => setDialog(null)}>
                Cancelar
              </ActionButton>
              <ActionButton
                variant="primary"
                type="submit"
                disabled={!newProfileName.trim()}
              >
                Crear profile
              </ActionButton>
            </div>
          </form>
        </Dialog>
      )}
      {dialog?.type === "edit-profile" && editingProfile && (
        <Dialog
          title={`Editar ${editingProfile.name}`}
          onClose={() => setDialog(null)}
          size="wide"
        >
          <p className="dialog-description">
            Elige qué mods incluir en este profile. Los cambios se guardan solo
            en esta sesión de muestra.
          </p>
          <div className="edit-profile-mod-list">
            {editingMods.map((mod) => (
              <div className="edit-profile-mod-row" key={mod.id}>
                <span className={`tiny-art tiny-art--${mod.artwork}`} />
                <span className="edit-profile-mod-copy">
                  <strong>{mod.name}</strong>
                  <small>
                    {mod.character} · v{mod.version}
                  </small>
                </span>
                <Toggle
                  checked={editingModIds.includes(mod.id)}
                  onChange={() =>
                    setEditingModIds((current) =>
                      current.includes(mod.id)
                        ? current.filter((id) => id !== mod.id)
                        : [...current, mod.id],
                    )
                  }
                  label={`${editingModIds.includes(mod.id) ? "Quitar" : "Incluir"} ${mod.name}`}
                />
              </div>
            ))}
          </div>
          <div className="dialog-footer">
            <span>
              {editingModIds.length} mods seleccionados · solo prototipo
            </span>
            <ActionButton variant="quiet" onClick={() => setDialog(null)}>
              Cancelar
            </ActionButton>
            <ActionButton variant="primary" onClick={saveProfileEditor}>
              Guardar cambios
            </ActionButton>
          </div>
        </Dialog>
      )}
      {dialog?.type === "mod-details" && (
        <Dialog
          title={dialog.mod.name}
          onClose={() => setDialog(null)}
          size="wide"
        >
          <div className="mod-detail-dialog">
            <span className={`mod-art mod-art--${dialog.mod.artwork}`}>
              <span className="mod-art-orbit" />
              <span className="mod-art-mark" />
            </span>
            <div>
              <p className="dialog-description">
                Una ficha ficticia para explorar cómo podría verse el detalle de
                un mod.
              </p>
              <dl>
                <div>
                  <dt>Autor</dt>
                  <dd>{dialog.mod.author}</dd>
                </div>
                <div>
                  <dt>Personaje / tipo</dt>
                  <dd>{dialog.mod.character}</dd>
                </div>
                <div>
                  <dt>Versión</dt>
                  <dd>{dialog.mod.version}</dd>
                </div>
                <div>
                  <dt>Origen</dt>
                  <dd>{dialog.mod.source}</dd>
                </div>
              </dl>
            </div>
          </div>
          <div className="dialog-footer">
            <span>
              Contenido de muestra · no existe en una biblioteca real.
            </span>
            <ActionButton variant="quiet" onClick={() => setDialog(null)}>
              Cerrar
            </ActionButton>
          </div>
        </Dialog>
      )}
      {gameProfiles.length === 0 && (
        <span className="sr-only">No hay profiles para este juego.</span>
      )}
    </div>
  );
}
