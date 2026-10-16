import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import type { ModLibrary } from "./runtimeTypes";

type Mod = {
  id: string;
  name: string;
  files: { relative_path: string; size: number; sha256: string }[];
};
type Profile = { name: string; mod_ids: string[] };
type Library = {
  game_id: string;
  mods: Mod[];
  profiles: Profile[];
  revision: number;
  issues: { code: string; detail: string }[];
};
type Prepared = { profile: Profile; mods_path: string };

function count(value: number, singular: string, plural: string): string {
  return `${value} ${value === 1 ? singular : plural}`;
}

function fileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const divisor = bytes < 1024 * 1024 ? 1024 : 1024 * 1024;
  return `${(bytes / divisor).toLocaleString("es", { maximumFractionDigits: 1 })} ${divisor === 1024 ? "KiB" : "MiB"}`;
}

function errorMessage(error: unknown): string {
  if (typeof error === "object" && error !== null && "detail" in error) {
    return String(error.detail);
  }
  return error instanceof Error ? error.message : String(error);
}

export default function ModManagerPanel({
  gameId,
  onGameChange,
  onLibraryChange,
}: {
  gameId?: string;
  onGameChange?: (game: string) => void;
  onLibraryChange?: (library: ModLibrary) => void;
}) {
  const [localGame, setLocalGame] = useState("wuthering-waves");
  const game = gameId ?? localGame;
  function setGame(value: string) {
    setLocalGame(value);
    onGameChange?.(value);
  }
  return (
    <section
      className="panel mod-manager"
      aria-labelledby="mod-manager-heading"
    >
      <div className="section-heading">
        <h2 id="mod-manager-heading">Mods y perfiles</h2>
        <span className="section-note">Biblioteca local</span>
      </div>
      <p className="discovery-note">
        Conserva tus mods y prepara una selección para cada juego. Los
        originales se mantienen intactos; las copias preparadas se guardan
        dentro de LXMI.
      </p>
      <div className="xxmi-form mod-game-picker">
        <label htmlFor="mod-game">Juego de la biblioteca</label>
        <select
          id="mod-game"
          value={game}
          onChange={(e) => setGame(e.target.value)}
        >
          <option value="wuthering-waves">Wuthering Waves · WWMI</option>
          <option value="zenless-zone-zero">Zenless Zone Zero · ZZMI</option>
        </select>
      </div>
      <GameLibrary key={game} game={game} onLibraryChange={onLibraryChange} />
    </section>
  );
}

function GameLibrary({
  game,
  onLibraryChange,
}: {
  game: string;
  onLibraryChange?: (library: ModLibrary) => void;
}) {
  const [library, setLibrary] = useState<Library | null>(null);
  const [pending, setPending] = useState<string | null>("load");
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState("");
  const [source, setSource] = useState("");
  const [modName, setModName] = useState("");
  const [profileName, setProfileName] = useState("");
  const [selected, setSelected] = useState<string[]>([]);
  const [query, setQuery] = useState("");
  const [prepared, setPrepared] = useState<Prepared | null>(null);

  useEffect(() => {
    let cancelled = false;
    invoke<Library>("list_mod_library", { gameId: game })
      .then((value) => {
        if (!cancelled) {
          setLibrary(value);
          onLibraryChange?.(value);
        }
      })
      .catch((reason: unknown) => {
        if (!cancelled) setError(errorMessage(reason));
      })
      .finally(() => {
        if (!cancelled) setPending(null);
      });
    return () => {
      cancelled = true;
    };
  }, [game, onLibraryChange]);

  async function run(action: string, operation: () => Promise<void>) {
    setPending(action);
    setError(null);
    setMessage("");
    try {
      await operation();
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setPending(null);
    }
  }

  const savedProfile = library?.profiles.find(
    (p) => p.name.toLowerCase() === profileName.trim().toLowerCase(),
  );
  const dirty =
    !savedProfile ||
    savedProfile.mod_ids.length !== selected.length ||
    selected.some((id) => !savedProfile.mod_ids.includes(id));
  const unavailable = selected.filter(
    (id) => !library?.mods.some((mod) => mod.id === id),
  );
  const filteredMods = library?.mods.filter((mod) =>
    mod.name.toLocaleLowerCase().includes(query.toLocaleLowerCase()),
  );
  const busy = pending !== null;

  return (
    <div aria-busy={busy}>
      <div className="mod-library-toolbar">
        <p className="discovery-note" role="status">
          {pending === "load"
            ? "Leyendo biblioteca…"
            : library
              ? `${count(library.mods.length, "mod", "mods")} · ${count(selected.length, "seleccionado", "seleccionados")} para el perfil`
              : "No se pudo leer la biblioteca."}
        </p>
        <button
          className="secondary-button"
          type="button"
          disabled={busy}
          onClick={() =>
            void run("refresh", async () => {
              setLibrary(
                await invoke<Library>("list_mod_library", { gameId: game }),
              );
              setPrepared(null);
              setMessage("Biblioteca actualizada. Tu selección se conserva.");
            })
          }
        >
          {pending === "refresh" ? "Actualizando…" : "Actualizar biblioteca"}
        </button>
      </div>

      {error && (
        <p className="message message-error" role="alert">
          {error}
        </p>
      )}
      <p className="mod-feedback" role="status" aria-live="polite">
        {message}
      </p>

      {library && (
        <>
          <form
            className="xxmi-form mod-import-form"
            onSubmit={(event) => {
              event.preventDefault();
              void run("import", async () => {
                const mod = await invoke<Mod>("import_local_mod", {
                  gameId: game,
                  name: modName.trim(),
                  source: source.trim(),
                });
                const next = await invoke<Library>("list_mod_library", {
                  gameId: game,
                });
                const duplicate = library.mods.some((m) => m.id === mod.id);
                setLibrary(next);
                onLibraryChange?.(next);
                setSource("");
                setModName("");
                setMessage(
                  duplicate
                    ? `«${mod.name}» ya estaba en la biblioteca; se conservó una sola copia.`
                    : `«${mod.name}» importado. Márcalo para añadirlo al perfil.`,
                );
              });
            }}
          >
            <h3>Importar un mod</h3>
            <div className="mod-import-fields">
              <div>
                <label htmlFor="mod-name">Nombre del mod</label>
                <input
                  id="mod-name"
                  required
                  maxLength={100}
                  value={modName}
                  disabled={busy}
                  onChange={(e) => setModName(e.target.value)}
                  autoComplete="off"
                  placeholder="Nombre para tu biblioteca"
                />
              </div>
              <div>
                <label htmlFor="mod-source">Ruta de la carpeta o ZIP</label>
                <input
                  id="mod-source"
                  required
                  value={source}
                  disabled={busy}
                  onChange={(e) => setSource(e.target.value)}
                  autoComplete="off"
                  spellCheck={false}
                  placeholder="/home/usuario/Descargas/mi-mod.zip"
                  aria-describedby="mod-import-help"
                />
              </div>
            </div>
            <p id="mod-import-help" className="discovery-note">
              Pega una ruta absoluta. El mod debe contener un INI. El juego se
              asigna según la biblioteca elegida; LXMI no comprueba su
              compatibilidad.
            </p>
            <button
              className="primary-button"
              type="submit"
              disabled={busy || !source.trim() || !modName.trim()}
            >
              {pending === "import" ? "Importando…" : "Importar mod"}
            </button>
          </form>

          {library.issues.length > 0 && (
            <div className="message message-error" role="alert">
              <p>
                Hay mods que no superan la verificación y no se pueden
                seleccionar.
              </p>
              <ul>
                {library.issues.map((issue, index) => (
                  <li key={index}>{issue.detail}</li>
                ))}
              </ul>
            </div>
          )}

          <div className="mod-workspace">
            <div className="mod-selection">
              <div className="xxmi-form mod-filter">
                <label htmlFor="mod-filter">Buscar en la biblioteca</label>
                <input
                  id="mod-filter"
                  type="search"
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  placeholder="Buscar por nombre"
                  disabled={library.mods.length === 0}
                />
              </div>
              {library.mods.length === 0 ? (
                <p className="mod-empty">
                  Esta biblioteca todavía no tiene mods verificados. Importa una
                  carpeta o un ZIP para empezar tu primer perfil.
                </p>
              ) : filteredMods?.length === 0 ? (
                <p className="mod-empty">
                  No hay mods que coincidan con «{query}».
                </p>
              ) : (
                <ul className="mod-list" aria-label="Mods de la biblioteca">
                  {filteredMods?.map((mod) => (
                    <li key={mod.id}>
                      <label className="mod-row">
                        <input
                          type="checkbox"
                          checked={selected.includes(mod.id)}
                          disabled={busy}
                          onChange={(e) => {
                            setSelected(
                              e.target.checked
                                ? [...selected, mod.id]
                                : selected.filter((id) => id !== mod.id),
                            );
                            setPrepared(null);
                          }}
                        />
                        <span>
                          <strong>{mod.name}</strong>
                          <span className="mod-file-count">
                            {count(mod.files.length, "archivo", "archivos")} ·{" "}
                            {fileSize(
                              mod.files.reduce((sum, f) => sum + f.size, 0),
                            )}
                          </span>
                        </span>
                      </label>
                    </li>
                  ))}
                </ul>
              )}
            </div>

            <form
              className="xxmi-form mod-profile-form"
              onSubmit={(event) => {
                event.preventDefault();
                void run("save", async () => {
                  const next = await invoke<Library>("save_mod_profile", {
                    gameId: game,
                    name: profileName.trim(),
                    modIds: selected,
                    expectedRevision: library.revision,
                  });
                  setLibrary(next);
                  onLibraryChange?.(next);
                  setProfileName(profileName.trim());
                  setPrepared(null);
                  setMessage(
                    `Perfil «${profileName.trim()}» guardado con ${count(selected.length, "mod", "mods")}.`,
                  );
                });
              }}
            >
              <h3>Tu perfil</h3>
              <label htmlFor="saved-mod-profile">
                Recuperar una selección guardada
              </label>
              <select
                id="saved-mod-profile"
                value={savedProfile?.name ?? ""}
                disabled={busy}
                onChange={(e) => {
                  const profile = library.profiles.find(
                    (p) => p.name === e.target.value,
                  );
                  setProfileName(profile?.name ?? "");
                  setSelected(profile?.mod_ids ?? []);
                  setPrepared(null);
                  setMessage(
                    profile
                      ? `Selección de «${profile.name}» recuperada.`
                      : "Nuevo perfil. Marca los mods que quieras incluir.",
                  );
                }}
              >
                <option value="">Nuevo perfil</option>
                {library.profiles.map((p) => (
                  <option key={p.name} value={p.name}>
                    {p.name}
                  </option>
                ))}
              </select>
              {savedProfile && dirty && (
                <button
                  className="secondary-button"
                  type="button"
                  disabled={busy}
                  onClick={() => {
                    setSelected([...savedProfile.mod_ids]);
                    setPrepared(null);
                    setMessage(
                      `Selección guardada de «${savedProfile.name}» recuperada.`,
                    );
                  }}
                >
                  Recuperar selección guardada
                </button>
              )}
              <label htmlFor="mod-profile-name">Nombre del perfil</label>
              <input
                id="mod-profile-name"
                required
                maxLength={100}
                value={profileName}
                disabled={busy}
                onChange={(e) => {
                  setProfileName(e.target.value);
                  setPrepared(null);
                }}
                placeholder="Por ejemplo, mi selección diaria"
              />
              <p className="discovery-note">
                Marca los mods que quieres incluir. Desmarcarlos los retira del
                perfil y conserva su copia en la biblioteca.
              </p>
              {unavailable.length > 0 && (
                <div className="message-error" role="alert">
                  <p>
                    Este perfil contiene{" "}
                    {count(
                      unavailable.length,
                      "mod ausente o alterado",
                      "mods ausentes o alterados",
                    )}
                    .
                  </p>
                  <button
                    className="secondary-button"
                    type="button"
                    disabled={busy}
                    onClick={() => {
                      setSelected(
                        selected.filter((id) => !unavailable.includes(id)),
                      );
                      setPrepared(null);
                    }}
                  >
                    Quitar los no disponibles de la selección
                  </button>
                </div>
              )}
              <div className="mod-profile-actions">
                <button
                  className="primary-button"
                  type="submit"
                  disabled={
                    busy ||
                    !profileName.trim() ||
                    unavailable.length > 0 ||
                    !dirty
                  }
                >
                  {pending === "save"
                    ? "Guardando…"
                    : savedProfile
                      ? "Guardar cambios del perfil"
                      : "Guardar perfil"}
                </button>
                <button
                  className="secondary-button"
                  type="button"
                  disabled={busy || dirty || unavailable.length > 0}
                  onClick={() =>
                    void run("prepare", async () => {
                      const result = await invoke<Prepared>(
                        "prepare_mod_profile",
                        {
                          gameId: game,
                          name: savedProfile?.name,
                          expectedRevision: library.revision,
                        },
                      );
                      setPrepared(result);
                      setMessage(
                        `Copia del perfil «${result.profile.name}» preparada dentro de LXMI.`,
                      );
                    })
                  }
                >
                  {pending === "prepare"
                    ? "Preparando…"
                    : "Preparar copia del perfil"}
                </button>
              </div>
              <p className="discovery-note">
                {dirty
                  ? "Guarda la selección antes de preparar su copia. "
                  : ""}
                Preparar no activa mods en el juego. La conexión con el cargador
                y los cambios sin recarga están pendientes.
              </p>
            </form>
          </div>
          {prepared && (
            <div className="mod-prepared" role="status">
              <h3>Copia preparada · {prepared.profile.name}</h3>
              <p className="path-value">{prepared.mods_path}</p>
              <p className="discovery-note">
                Contiene{" "}
                {count(
                  prepared.profile.mod_ids.length,
                  "mod seleccionado",
                  "mods seleccionados",
                )}
                . Las preparaciones anteriores se conservan.
              </p>
            </div>
          )}
          <details className="mod-limits">
            <summary>Formatos y límites de esta versión</summary>
            <p className="discovery-note">
              Carpetas extraídas y ZIP de hasta 32 MiB; hasta 512 MiB extraídos,
              128 MiB por archivo y 4096 entradas. Se admiten 256 mods y 64
              perfiles por juego. RAR y 7z deben extraerse antes. Los
              ejecutables y DLL se gestionan en la sección XXMI. Los hashes
              detectan cambios locales; no certifican el origen ni detectan
              conflictos entre mods.
            </p>
          </details>
        </>
      )}
    </div>
  );
}
