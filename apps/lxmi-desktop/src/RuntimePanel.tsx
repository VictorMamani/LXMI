import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import type { Game } from "./gameCatalog";
import {
  errorMessage,
  type ActivationPlan,
  type ModLibrary,
  type RuntimeSources,
  type RuntimeStatus,
} from "./runtimeTypes";

type Release = { package_kind: string; tag: string };
export default function RuntimePanel({
  game,
  library,
  status,
  sources,
  onSourcesChange,
  onRefresh,
}: {
  game: Game;
  library: ModLibrary | null;
  status: RuntimeStatus | null;
  sources: RuntimeSources;
  onSourcesChange: (sources: RuntimeSources) => Promise<void>;
  onRefresh: (sources?: RuntimeSources) => Promise<void>;
}) {
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const [releases, setReleases] = useState<Release[]>([]);
  const [profileName, setProfileName] = useState("");
  const [plan, setPlan] = useState<ActivationPlan | null>(null);
  const selectedProfile = library?.profiles.some((p) => p.name === profileName)
    ? profileName
    : (library?.profiles[0]?.name ?? "");
  const validPlan =
    plan &&
    plan.profile_revision === library?.revision &&
    plan.selection_revision === status?.selection.revision &&
    plan.profile_name === selectedProfile
      ? plan
      : null;
  const args = {
    gameId: game.backendId,
    importerId: sources.importer,
    librariesId: sources.libraries,
  };
  async function run(name: string, operation: () => Promise<void>) {
    setBusy(name);
    setError("");
    setMessage("");
    try {
      await operation();
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setBusy(null);
    }
  }
  async function download(kind: string, tag: string) {
    await run(kind, async () => {
      const result = await invoke<{ id: string }>(
        "xxmi_download_official_package",
        { packageKind: kind, tag },
      );
      const next = {
        ...sources,
        ...(kind === "xxmi_libraries"
          ? { libraries: result.id }
          : { importer: result.id }),
      };
      setPlan(null);
      await onSourcesChange(next);
      setMessage("Paquete descargado y firma oficial verificada.");
    });
  }
  return (
    <section
      className="panel runtime-panel"
      aria-labelledby="runtime-panel-heading"
    >
      <div className="section-heading">
        <h2 id="runtime-panel-heading">Runtime y activación del perfil</h2>
        <button
          type="button"
          className="secondary-button"
          disabled={busy !== null}
          onClick={() => void run("refresh", () => onRefresh())}
        >
          Actualizar estado
        </button>
      </div>
      <p className="discovery-note">
        Prepara {game.importer} y aplica tu perfil a una copia privada. Puedes
        revertir la selección; el runtime base y los mods originales se
        conservan.
      </p>
      {error && (
        <p className="message message-error" role="alert">
          {error}
        </p>
      )}
      <p className="runtime-feedback" role="status">
        {message}
      </p>
      {!status ? (
        <p className="discovery-note">Leyendo el estado del runtime…</p>
      ) : (
        <>
          <div
            className="runtime-selection-line"
            data-selected={Boolean(status.selection.current)}
          >
            <span className="runtime-state-dot" />
            <div>
              <strong>
                {status.selection.current?.profile_name ??
                  "Sin perfil aplicado"}
              </strong>
              <small>
                {status.selection.current
                  ? `${status.selection.current.mod_count} mods en el runtime seleccionado para el próximo inicio`
                  : "Guarda un perfil y revisa su aplicación para empezar."}
              </small>
            </div>
          </div>
          <div className="runtime-grid">
            <div className="xxmi-form">
              <h3>Paquetes oficiales</h3>
              <label htmlFor="runtime-importer">Paquete {game.importer}</label>
              <select
                id="runtime-importer"
                value={sources.importer}
                disabled={busy !== null}
                onChange={(e) => {
                  setPlan(null);
                  void run("sources", () =>
                    onSourcesChange({ ...sources, importer: e.target.value }),
                  );
                }}
              >
                <option value="">Seleccionar paquete…</option>
                {status.importer_packages.map((p) => (
                  <option
                    key={p.id}
                    value={p.id}
                    disabled={p.authenticity !== "official_release_verified"}
                  >
                    {game.importer} {p.version?.raw ?? "sin versión"} ·{" "}
                    {p.id.slice(0, 8)}
                    {p.authenticity !== "official_release_verified"
                      ? " · sin autenticar"
                      : ""}
                  </option>
                ))}
              </select>
              <label htmlFor="runtime-libraries">XXMI Libraries</label>
              <select
                id="runtime-libraries"
                value={sources.libraries}
                disabled={busy !== null}
                onChange={(e) => {
                  setPlan(null);
                  void run("sources", () =>
                    onSourcesChange({ ...sources, libraries: e.target.value }),
                  );
                }}
              >
                <option value="">Seleccionar bibliotecas…</option>
                {status.libraries_packages.map((p) => (
                  <option
                    key={p.id}
                    value={p.id}
                    disabled={p.authenticity !== "official_release_verified"}
                  >
                    {p.version?.raw ?? "sin versión"} · {p.id.slice(0, 8)}
                    {p.authenticity !== "official_release_verified"
                      ? " · sin autenticar"
                      : ""}
                  </option>
                ))}
              </select>
              <button
                className="secondary-button"
                type="button"
                disabled={busy !== null}
                onClick={() =>
                  void run("releases", async () => {
                    setReleases(
                      await invoke<Release[]>("xxmi_check_official_releases"),
                    );
                    setMessage(
                      "Releases oficiales consultadas. Elige qué descargar.",
                    );
                  })
                }
              >
                {busy === "releases"
                  ? "Consultando…"
                  : "Consultar paquetes oficiales"}
              </button>
              {releases
                .filter(
                  (r) =>
                    r.package_kind === game.importer.toLowerCase() ||
                    r.package_kind === "xxmi_libraries",
                )
                .map((r) => (
                  <button
                    key={r.package_kind}
                    className="secondary-button"
                    type="button"
                    disabled={busy !== null}
                    onClick={() => void download(r.package_kind, r.tag)}
                  >
                    {busy === r.package_kind
                      ? "Descargando y verificando…"
                      : `Descargar ${r.package_kind === "xxmi_libraries" ? "XXMI Libraries" : game.importer} ${r.tag}`}
                  </button>
                ))}
              <button
                className="primary-button"
                type="button"
                disabled={
                  busy !== null ||
                  !sources.importer ||
                  !sources.libraries ||
                  !status.installed
                }
                onClick={() =>
                  void run("assemble", async () => {
                    await invoke("prepare_game_mod_runtime", args);
                    setPlan(null);
                    await onRefresh();
                    setMessage(
                      `${game.importer} ensamblado en el almacenamiento privado de LXMI.`,
                    );
                  })
                }
              >
                {busy === "assemble"
                  ? "Ensamblando…"
                  : status.base_runtime
                    ? "Verificar runtime preparado"
                    : `Preparar runtime ${game.importer}`}
              </button>
              {!status.installed && (
                <p className="discovery-note">
                  No se encontró una instalación completa de este juego en
                  Steam. Puedes organizar mods y perfiles; el ensamblado
                  requiere su ejecutable.
                </p>
              )}
              {status.base_runtime && (
                <p className="discovery-note">
                  Runtime base verificado · {status.base_runtime.file_count}{" "}
                  archivos · objetivo{" "}
                  <span className="path-value">
                    {status.base_runtime.target_process}
                  </span>
                </p>
              )}
            </div>
            <div className="xxmi-form">
              <h3>Selección para el próximo inicio</h3>
              <label htmlFor="activation-profile">Perfil guardado</label>
              <select
                id="activation-profile"
                value={selectedProfile}
                disabled={busy !== null || !library?.profiles.length}
                onChange={(e) => {
                  setProfileName(e.target.value);
                  setPlan(null);
                }}
              >
                {!library?.profiles.length && (
                  <option value="">Guarda tu primer perfil en Mods</option>
                )}
                {library?.profiles.map((p) => (
                  <option key={p.name} value={p.name}>
                    {p.name} · {p.mod_ids.length} mods
                  </option>
                ))}
              </select>
              <button
                className="secondary-button"
                type="button"
                disabled={
                  busy !== null || !status.base_runtime || !selectedProfile
                }
                onClick={() =>
                  void run("review", async () => {
                    setPlan(
                      await invoke<ActivationPlan>(
                        "review_mod_profile_activation",
                        {
                          ...args,
                          profileName: selectedProfile,
                          profileRevision: library?.revision,
                          selectionRevision: status.selection.revision,
                        },
                      ),
                    );
                  })
                }
              >
                {busy === "review"
                  ? "Revisando…"
                  : "Revisar aplicación del perfil"}
              </button>
              {validPlan && (
                <div className="activation-review">
                  <h3>Aplicar «{validPlan.profile_name}»</h3>
                  <p>
                    {validPlan.mod_count} mods · {validPlan.file_count} archivos
                    ·{" "}
                    {(validPlan.total_bytes / 1024 / 1024).toLocaleString(
                      "es",
                      { maximumFractionDigits: 1 },
                    )}{" "}
                    MiB
                  </p>
                  <p className="path-value">{validPlan.importer_root}</p>
                  {validPlan.required_launch_arguments.length > 0 && (
                    <p>
                      Argumentos requeridos:{" "}
                      <code>
                        {validPlan.required_launch_arguments.join(" ")}
                      </code>
                    </p>
                  )}
                  <p className="discovery-note">
                    Se crea una copia nueva y se cambia la selección de LXMI. No
                    modifica el juego ni inicia procesos.
                  </p>
                  <button
                    className="primary-button"
                    type="button"
                    disabled={busy !== null}
                    onClick={() =>
                      void run("apply", async () => {
                        await invoke("activate_mod_profile", {
                          ...args,
                          profileName: validPlan.profile_name,
                          profileRevision: validPlan.profile_revision,
                          selectionRevision: validPlan.selection_revision,
                          planId: validPlan.plan_id,
                        });
                        setPlan(null);
                        await onRefresh();
                        setMessage(
                          "Perfil aplicado al runtime privado. La selección anterior se conserva para revertir.",
                        );
                      })
                    }
                  >
                    {busy === "apply"
                      ? "Aplicando…"
                      : "Aplicar perfil al runtime"}
                  </button>
                </div>
              )}
              <button
                className="secondary-button"
                type="button"
                disabled={busy !== null || status.selection.revision === 0}
                onClick={() =>
                  void run("rollback", async () => {
                    await invoke("rollback_mod_profile_activation", {
                      gameId: game.backendId,
                      selectionRevision: status.selection.revision,
                    });
                    setPlan(null);
                    await onRefresh();
                    setMessage("Selección anterior recuperada.");
                  })
                }
              >
                {status.selection.current
                  ? "Revertir selección anterior"
                  : "Rehacer selección"}
              </button>
              <p className="discovery-note">
                Aplicar prepara los archivos que resolverá LXMI. La carga dentro
                del juego y el cambio sin recarga necesitan validación
                independiente.
              </p>
            </div>
          </div>
          <details className="runtime-launch-details">
            <summary>Estado de la carga dentro del juego</summary>
            <ul>
              {status.launch_blockers.map((blocker) => (
                <li key={blocker}>{blocker}</li>
              ))}
            </ul>
          </details>
        </>
      )}
    </section>
  );
}
