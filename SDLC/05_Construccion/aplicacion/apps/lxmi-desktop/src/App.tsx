import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import XxmiPanel from "./XxmiPanel";

type SystemInfo = {
  os: string;
  architecture: string;
  homeDirectory: string | null;
  xdgDataHome: string | null;
  xdgDataDirs: string[];
};

type SteamStatus =
  | "not_installed"
  | "detected"
  | "configuration_missing"
  | "invalid_configuration"
  | "permission_denied"
  | "home_directory_unavailable"
  | "internal_error";

type SteamIssue = {
  code: string;
  path: string | null;
  detail: string | null;
};

type SteamScan = {
  status: SteamStatus;
  installations: {
    rootPath: string;
    libraries: { path: string; isDefault: boolean }[];
  }[];
  issues: SteamIssue[];
  gameScanStatus: "not_available" | "complete" | "partial";
  manifestsParsed: number;
  ignoredUnknownApps: number;
  games: SteamGame[];
  gameIssues: SteamGameIssue[];
  compatibilityToolsStatus: "not_available" | "complete" | "partial";
  compatibilityTools: CompatibilityTool[];
  compatibilityToolIssues: CompatibilityToolIssue[];
  runtimePlans: RuntimePlan[];
};

type CompatibilityTool = {
  internalId: string | null;
  displayName: string;
  path: string;
  metadataPath: string | null;
  source: "steam_library" | "custom";
  kind: "proton" | "steam_linux_runtime" | "other" | "unknown";
  version: string | null;
  status: "valid" | "incomplete" | "invalid_metadata" | "unreadable";
};

type CompatibilityToolIssue = {
  code: string;
  severity: "info" | "warning" | "error";
  path: string;
  detail: string | null;
};

type RuntimeCandidate = {
  internalId: string | null;
  displayName: string;
  path: string;
  source: "steam_library" | "custom";
  version: string | null;
};

type RuntimePlan = {
  gameId: string;
  gameName: string;
  expectedSteamAppIds: number[];
  platform: "steam";
  runtimePolicy: "expected" | "not_required" | "unknown";
  installation:
    | { state: "not_found"; expectedSteamAppIds: number[] }
    | {
        state: "detected";
        steamAppId: number;
        manifestName: string;
        steamLibrary: string;
        installPath: string;
        distribution: "steam" | "hoyoplay" | "manual" | "unknown";
        executablePath: string | null;
        executableStatus: SteamGame["executableStatus"];
        directoryStatus: SteamGame["installStatus"];
        completeness: "unknown";
      }
    | {
        state: "unknown";
        discoveryStatus: "not_available" | "complete" | "partial";
      };
  selection:
    | { state: "unknown"; reason: string }
    | { state: "selected"; candidate: RuntimeCandidate }
    | { state: "not_required" };
  candidateDiscovery: "complete" | "partial" | "unavailable";
  availableRuntimeCandidates: RuntimeCandidate[];
  compatdata:
    | { state: "not_required" }
    | { state: "unknown"; discoveryStatus: string }
    | { state: "not_found"; expectedPath: string }
    | { state: "found"; path: string }
    | { state: "invalid"; path: string }
    | { state: "unreadable"; path: string; status: string };
  prefix:
    | { state: "not_required" }
    | { state: "unknown"; discoveryStatus: string }
    | { state: "not_initialized"; expectedPath: string }
    | { state: "candidate_found"; path: string }
    | { state: "invalid"; path: string }
    | { state: "unreadable"; path: string; status: string };
  readiness:
    "ready" | "incomplete" | "needs_initialization" | "unknown" | "blocked";
  requirements: { requirement: string; state: string }[];
  evidence: { kind: string; [key: string]: string | number | boolean }[];
  issues: {
    code: string;
    severity: "info" | "warning" | "error";
    path: string | null;
  }[];
  compatibilityToolIssues: CompatibilityToolIssue[];
};

type SteamGame = {
  id: string;
  name: string;
  steamAppId: number;
  manifestName: string;
  installPath: string;
  installStatus:
    | "installed"
    | "directory_missing"
    | "directory_invalid"
    | "permission_denied";
  distribution: "steam" | "hoyoplay" | "manual" | "unknown";
  executablePath: string | null;
  executableStatus: "found" | "not_found" | "search_incomplete" | "not_scanned";
  steamLibrary: string;
  compatdataStatus:
    | "not_found"
    | "compatdata_found"
    | "prefix_found"
    | "invalid"
    | "permission_denied"
    | "io_error";
  compatdataPath: string;
  prefixPath: string | null;
};

type SteamGameIssue = {
  code: string;
  severity: "warning" | "error";
  path: string;
  detail: string | null;
};

type LoadState<T> =
  { kind: "loading" } | { kind: "ready"; value: T } | { kind: "error" };

const steamStatusText: Record<SteamStatus, string> = {
  not_installed: "No se encontró Steam en las ubicaciones revisadas.",
  detected: "Steam detectado.",
  configuration_missing:
    "Se encontró una posible instalación de Steam, pero falta su configuración de bibliotecas.",
  invalid_configuration:
    "Se encontró Steam, pero libraryfolders.vdf no se pudo interpretar.",
  permission_denied: "No se pudo leer una ruta de Steam por falta de permisos.",
  home_directory_unavailable:
    "No se encontró un directorio home absoluto para el usuario.",
  internal_error: "Ocurrió un error del sistema durante la detección.",
};

const issueText: Record<string, string> = {
  steam_root_unreadable: "No se pudo leer esta ruta de Steam.",
  steamapps_missing: "La ruta no contiene un directorio steamapps.",
  configuration_missing: "No se encontró libraryfolders.vdf.",
  configuration_unreadable: "No se pudo leer libraryfolders.vdf.",
  configuration_not_regular_file:
    "La configuración no es un archivo normal y se omitió.",
  invalid_configuration: "La configuración de bibliotecas no es válida.",
  invalid_library_path: "La ruta registrada no es absoluta y se omitió.",
  library_missing:
    "La biblioteca registrada no existe o no contiene steamapps.",
  library_permission_denied: "No se pudo leer la biblioteca registrada.",
  filesystem_error: "La operación de lectura encontró un error del sistema.",
};

const gameIssueText: Record<string, string> = {
  steamapps_unavailable: "No se pudo inspeccionar el directorio steamapps.",
  invalid_manifest_filename:
    "El nombre de un manifest no contiene un AppID válido.",
  manifest_not_regular_file:
    "Se omitió un manifest que no es un archivo normal.",
  manifest_too_large: "Se omitió un manifest que supera el límite de 2 MiB.",
  manifest_invalid: "Un manifest está malformado o le faltan datos requeridos.",
  manifest_id_mismatch:
    "El AppID del manifest no coincide con su nombre de archivo.",
  game_directory_missing:
    "El manifest existe, pero no está su carpeta de instalación.",
  game_directory_invalid:
    "La ruta indicada para el juego no es un directorio seguro.",
  game_directory_permission_denied:
    "No se pudo inspeccionar la carpeta del juego.",
  permission_denied: "No se pudo leer una ruta por falta de permisos.",
  compatdata_invalid: "La ruta compatdata o pfx no es un directorio normal.",
  filesystem_error: "La inspección encontró un error del sistema de archivos.",
};

const installStatusText: Record<SteamGame["installStatus"], string> = {
  installed: "Directorio de instalación presente",
  directory_missing: "Manifest encontrado; falta el directorio de instalación",
  directory_invalid: "La ruta de instalación no es válida",
  permission_denied: "No se pudo comprobar la ruta de instalación",
};

const executableStatusText: Record<SteamGame["executableStatus"], string> = {
  found: "Ejecutable esperado encontrado",
  not_found: "No se encontró el ejecutable esperado",
  search_incomplete: "Búsqueda limitada; resultado no concluyente",
  not_scanned: "Ejecutable no inspeccionado",
};

const compatdataStatusText: Record<SteamGame["compatdataStatus"], string> = {
  not_found: "No se encontró compatdata para este AppID",
  compatdata_found: "compatdata existe; no se encontró pfx",
  prefix_found: "Se encontró pfx (candidato a prefix)",
  invalid: "La ruta compatdata/pfx no es un directorio válido",
  permission_denied: "No se pudo leer compatdata/pfx por permisos",
  io_error: "No se pudo inspeccionar compatdata/pfx",
};

const compatibilityToolKindText: Record<CompatibilityTool["kind"], string> = {
  proton: "Proton",
  steam_linux_runtime: "Steam Linux Runtime",
  other: "Otra herramienta",
  unknown: "Tipo no identificado",
};

const compatibilityToolStatusText: Record<CompatibilityTool["status"], string> =
  {
    valid: "Estructura válida",
    incomplete: "Instalación incompleta",
    invalid_metadata: "Metadata inválida",
    unreadable: "No se pudo leer",
  };

const readinessText: Record<RuntimePlan["readiness"], string> = {
  ready: "Base detectada",
  incomplete: "Configuración pendiente",
  needs_initialization: "Compatdata por inicializar",
  unknown: "Estado por determinar",
  blocked: "Juego no disponible",
};

const compatibilityToolIssueText: Record<string, string> = {
  common_directory_invalid:
    "No se pudo inspeccionar steamapps/common en esta biblioteca.",
  custom_directory_invalid:
    "La ruta de herramientas custom no es un directorio válido.",
  directory_unreadable:
    "No se pudo leer un directorio de herramientas de compatibilidad.",
  entry_unreadable: "No se pudo leer una entrada del directorio.",
  symlink_rejected: "Se omitió el enlace simbólico sin seguir su destino.",
  file_not_regular: "Se omitió una metadata que no es un archivo normal.",
  file_too_large: "Se omitió un archivo que supera el límite de lectura.",
  metadata_invalid: "La metadata de una herramienta está malformada.",
  install_path_unsafe: "La ruta declarada por la herramienta no es segura.",
  tool_directory_missing:
    "La herramienta declara una carpeta de instalación que no existe.",
  filesystem_error: "La inspección encontró un error del sistema de archivos.",
};

function displayValue(value: string | null): string {
  return value ?? "No disponible";
}

export default function App() {
  const [system, setSystem] = useState<LoadState<SystemInfo>>({
    kind: "loading",
  });
  const [steam, setSteam] = useState<LoadState<SteamScan>>({
    kind: "loading",
  });
  const [scanPending, setScanPending] = useState(false);

  useEffect(() => {
    let active = true;
    invoke<SystemInfo>("get_system_info")
      .then((value) => {
        if (active) setSystem({ kind: "ready", value });
      })
      .catch(() => {
        if (active) setSystem({ kind: "error" });
      });
    return () => {
      active = false;
    };
  }, []);

  async function scanSteam() {
    setScanPending(true);
    try {
      const result = await invoke<SteamScan>("scan_steam");
      setSteam({ kind: "ready", value: result });
    } catch {
      setSteam({ kind: "error" });
    } finally {
      setScanPending(false);
    }
  }

  return (
    <main className="shell">
      <header className="page-header">
        <div>
          <p className="eyebrow">LINUX · DEVELOPMENT BUILD</p>
          <h1>LXMI</h1>
          <p className="lede">
            Diagnóstico local de Steam y herramientas de compatibilidad. El
            escaneo solo lee rutas y metadata.
          </p>
        </div>
        <span className="version-tag">v0.5.3 · DESARROLLO</span>
      </header>

      <section className="panel" aria-labelledby="system-heading">
        <div className="section-heading">
          <div>
            <p className="section-index">01</p>
            <h2 id="system-heading">Sistema</h2>
          </div>
          <span className="section-note">Información de esta sesión</span>
        </div>

        {system.kind === "loading" && (
          <p role="status" className="muted">
            Consultando información del sistema…
          </p>
        )}
        {system.kind === "error" && (
          <p role="alert" className="message message-error">
            No se pudo obtener la información básica de Linux.
          </p>
        )}
        {system.kind === "ready" && (
          <dl className="details-grid">
            <div>
              <dt>OS</dt>
              <dd>{system.value.os}</dd>
            </div>
            <div>
              <dt>Arquitectura</dt>
              <dd>{system.value.architecture}</dd>
            </div>
            <div className="detail-wide">
              <dt>Home</dt>
              <dd className="path-value">
                {displayValue(system.value.homeDirectory)}
              </dd>
            </div>
            <div className="detail-wide">
              <dt>XDG data home</dt>
              <dd className="path-value">
                {displayValue(system.value.xdgDataHome)}
              </dd>
            </div>
            <div className="detail-wide">
              <dt>XDG data dirs</dt>
              {system.value.xdgDataDirs.length > 0 ? (
                <dd>
                  <ul className="path-list">
                    {system.value.xdgDataDirs.map((path) => (
                      <li className="path-value" key={path}>
                        {path}
                      </li>
                    ))}
                  </ul>
                </dd>
              ) : (
                <dd>No disponibles</dd>
              )}
            </div>
          </dl>
        )}
      </section>

      <section className="panel" aria-labelledby="steam-heading">
        <div className="section-heading">
          <div>
            <p className="section-index">02</p>
            <h2 id="steam-heading">Steam</h2>
          </div>
          <button
            className="primary-button"
            type="button"
            onClick={scanSteam}
            disabled={scanPending || system.kind !== "ready"}
          >
            {scanPending ? "Escaneando…" : "Scan Steam"}
          </button>
        </div>

        {steam.kind === "loading" && (
          <p className="muted">Aún no se ha iniciado un escaneo.</p>
        )}
        {steam.kind === "error" && (
          <p role="alert" className="message message-error">
            No se pudo completar el escaneo. Revisa el registro local de la
            aplicación.
          </p>
        )}
        {steam.kind === "ready" && (
          <div className="scan-result">
            <p
              className={"status-line status-" + steam.value.status}
              role="status"
            >
              <span className="status-mark" aria-hidden="true" />
              {steamStatusText[steam.value.status]}
            </p>

            {steam.value.installations.map((installation) => (
              <div className="installation" key={installation.rootPath}>
                <h3>Instalación</h3>
                <p className="path-value">{installation.rootPath}</p>
                <h3 className="subheading">Bibliotecas detectadas</h3>
                {installation.libraries.length > 0 ? (
                  <ul className="library-list">
                    {installation.libraries.map((library) => (
                      <li key={library.path}>
                        <span className="library-kind">
                          {library.isDefault ? "Principal" : "Adicional"}
                        </span>
                        <span className="path-value">{library.path}</span>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p className="muted">No hay bibliotecas verificadas.</p>
                )}
              </div>
            ))}

            {steam.value.issues.length > 0 && (
              <ul className="issue-list" aria-label="Avisos del escaneo">
                {steam.value.issues.map((issue, index) => (
                  <li
                    key={
                      issue.code + "-" + (issue.path ?? "none") + "-" + index
                    }
                  >
                    <span>
                      {issueText[issue.code] ?? "Se omitió una ruta no válida."}
                    </span>
                    {issue.path && (
                      <code className="path-value">{issue.path}</code>
                    )}
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}
      </section>

      <section className="panel" aria-labelledby="games-heading">
        <div className="section-heading">
          <div>
            <p className="section-index">03</p>
            <h2 id="games-heading">Juegos compatibles</h2>
          </div>
          <span className="section-note">Descubrimiento de solo lectura</span>
        </div>

        {steam.kind === "loading" && (
          <p className="muted">
            Escanea Steam para buscar juegos registrados en LXMI.
          </p>
        )}
        {steam.kind === "error" && (
          <p className="message message-error" role="alert">
            No se pudo obtener el resultado de descubrimiento de juegos.
          </p>
        )}
        {steam.kind === "ready" && (
          <div className="game-results">
            {steam.value.gameScanStatus === "not_available" ? (
              <>
                <p className="status-line" role="status">
                  Steam no está disponible para buscar juegos.
                </p>
                {steam.value.runtimePlans.map((plan) => (
                  <RuntimePlanSummary key={plan.gameId} plan={plan} />
                ))}
              </>
            ) : steam.value.games.length === 0 ? (
              <>
                <p className="status-line" role="status">
                  {steam.value.gameScanStatus === "partial"
                    ? "No se pudo confirmar la presencia de todos los juegos registrados; revisa los avisos."
                    : "No se encontraron juegos registrados en las bibliotecas Steam detectadas."}
                </p>
                {steam.value.runtimePlans.map((plan) => (
                  <RuntimePlanSummary key={plan.gameId} plan={plan} />
                ))}
              </>
            ) : (
              steam.value.games.map((game) => (
                <article className="game-installation" key={game.steamLibrary}>
                  <div className="game-title-row">
                    <div>
                      <h3>{game.name}</h3>
                      <p className="muted">
                        {installStatusText[game.installStatus]}
                      </p>
                    </div>
                    <span className="app-id">AppID {game.steamAppId}</span>
                  </div>
                  <dl className="details-grid game-details">
                    <div>
                      <dt>Distribución detectada</dt>
                      <dd>
                        {game.distribution === "steam"
                          ? "Steam"
                          : game.distribution}
                      </dd>
                    </div>
                    <div className="detail-wide">
                      <dt>Ruta de instalación esperada</dt>
                      <dd className="path-value">{game.installPath}</dd>
                    </div>
                    <div className="detail-wide">
                      <dt>Biblioteca Steam</dt>
                      <dd className="path-value">{game.steamLibrary}</dd>
                    </div>
                    <div className="detail-wide">
                      <dt>Ejecutable esperado</dt>
                      <dd>{executableStatusText[game.executableStatus]}</dd>
                      {game.executablePath && (
                        <p className="path-value">{game.executablePath}</p>
                      )}
                    </div>
                    <div className="detail-wide">
                      <dt>Compatdata</dt>
                      <dd>{compatdataStatusText[game.compatdataStatus]}</dd>
                      <p className="path-value">{game.compatdataPath}</p>
                    </div>
                    {game.prefixPath && (
                      <div className="detail-wide">
                        <dt>Carpeta pfx candidata</dt>
                        <dd className="path-value">{game.prefixPath}</dd>
                      </div>
                    )}
                  </dl>
                  <RuntimePlanSummary
                    plan={steam.value.runtimePlans.find(
                      (plan) => plan.gameId === game.id,
                    )}
                  />
                  <p className="discovery-note">
                    LXMI muestra herramientas instaladas por separado. No
                    determina cuál seleccionará Steam para este juego; la
                    presencia de compatdata/pfx tampoco confirma el runtime ni
                    la salud del prefix.
                  </p>
                </article>
              ))
            )}

            {steam.value.gameScanStatus !== "not_available" && (
              <p className="scan-summary muted">
                {steam.value.manifestsParsed} manifests válidos;{" "}
                {steam.value.ignoredUnknownApps} aplicaciones desconocidas
                omitidas del catálogo de LXMI.
              </p>
            )}

            {steam.value.gameIssues.length > 0 && (
              <ul
                className="issue-list game-issue-list"
                aria-label="Avisos de juegos"
              >
                {steam.value.gameIssues.map((issue, index) => (
                  <li
                    key={issue.code + "-" + issue.path + "-" + index}
                    className={issue.severity === "error" ? "issue-error" : ""}
                  >
                    <span>
                      {gameIssueText[issue.code] ??
                        "Se omitió un dato no válido."}
                    </span>
                    {issue.detail && <span>{issue.detail}</span>}
                    <code className="path-value">{issue.path}</code>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}
      </section>

      <section className="panel" aria-labelledby="proton-heading">
        <div className="section-heading">
          <div>
            <p className="section-index">04</p>
            <h2 id="proton-heading">Proton y compatibilidad</h2>
          </div>
          <span className="section-note">Herramientas instaladas</span>
        </div>

        {steam.kind === "loading" && (
          <p className="muted">Escanea Steam para buscar sus herramientas.</p>
        )}
        {steam.kind === "error" && (
          <p className="message message-error" role="alert">
            No se pudo obtener el resultado de herramientas de compatibilidad.
          </p>
        )}
        {steam.kind === "ready" && (
          <div>
            {steam.value.compatibilityToolsStatus === "not_available" ? (
              <p className="muted" role="status">
                Steam no está disponible para buscar herramientas.
              </p>
            ) : (
              <>
                {(() => {
                  const protonTools = steam.value.compatibilityTools.filter(
                    (tool) => tool.kind === "proton",
                  );
                  const validProtonCount = protonTools.filter(
                    (tool) => tool.status === "valid",
                  ).length;
                  return (
                    <p className="status-line" role="status">
                      <span className="status-mark" aria-hidden="true" />
                      {validProtonCount > 0
                        ? `${validProtonCount} herramienta${validProtonCount === 1 ? "" : "s"} Proton válida${validProtonCount === 1 ? "" : "s"} detectada${validProtonCount === 1 ? "" : "s"}.`
                        : "No se encontraron herramientas Proton instaladas en las ubicaciones examinadas."}
                      {protonTools.some((tool) => tool.status !== "valid") &&
                        ` ${protonTools.filter((tool) => tool.status !== "valid").length} candidato(s) incompleto(s).`}
                    </p>
                  );
                })()}

                {steam.value.compatibilityTools.length > 0 ? (
                  <ul
                    className="tool-list"
                    aria-label="Herramientas detectadas"
                  >
                    {steam.value.compatibilityTools.map((tool) => (
                      <li
                        className="tool-item"
                        key={`${tool.source}-${tool.path}-${tool.internalId ?? "no-id"}`}
                      >
                        <div className="tool-heading">
                          <div>
                            <h3>{tool.displayName}</h3>
                            <p className="muted">
                              {compatibilityToolKindText[tool.kind]} ·{" "}
                              {tool.source === "steam_library"
                                ? "Biblioteca Steam"
                                : "Custom"}
                            </p>
                          </div>
                          <span
                            className={`tool-status tool-status-${tool.status}`}
                          >
                            {compatibilityToolStatusText[tool.status]}
                          </span>
                        </div>
                        <dl className="details-grid tool-details">
                          <div>
                            <dt>Versión</dt>
                            <dd>{tool.version ?? "No disponible"}</dd>
                          </div>
                          {tool.internalId && (
                            <div>
                              <dt>ID interno</dt>
                              <dd>{tool.internalId}</dd>
                            </div>
                          )}
                          <div className="detail-wide">
                            <dt>Ruta</dt>
                            <dd className="path-value">{tool.path}</dd>
                          </div>
                          {tool.metadataPath && (
                            <div className="detail-wide">
                              <dt>Metadata</dt>
                              <dd className="path-value">
                                {tool.metadataPath}
                              </dd>
                            </div>
                          )}
                        </dl>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p className="muted">
                    No se encontraron herramientas de compatibilidad en las
                    ubicaciones examinadas.
                  </p>
                )}

                {steam.value.compatibilityToolsStatus === "partial" && (
                  <p className="muted">
                    El resultado es parcial; algunos directorios o archivos no
                    pudieron validarse.
                  </p>
                )}

                {steam.value.compatibilityToolIssues.length > 0 && (
                  <ul
                    className="issue-list tool-issue-list"
                    aria-label="Observaciones de herramientas de compatibilidad"
                  >
                    {steam.value.compatibilityToolIssues.map((issue, index) => (
                      <li
                        key={`${issue.code}-${issue.path}-${index}`}
                        className={
                          issue.severity === "error"
                            ? "issue-error"
                            : issue.severity === "info"
                              ? "issue-info"
                              : ""
                        }
                      >
                        <span>
                          {compatibilityToolIssueText[issue.code] ??
                            "No se pudo validar una herramienta."}
                        </span>
                        {issue.detail && issue.code !== "symlink_rejected" && (
                          <span>{issue.detail}</span>
                        )}
                        <code className="path-value">{issue.path}</code>
                      </li>
                    ))}
                  </ul>
                )}
              </>
            )}
          </div>
        )}
      </section>

      <XxmiPanel />

      <footer className="page-footer">
        <span>
          LXMI escribe importaciones solo en su almacenamiento; no ejecuta
          Proton/Wine ni modifica Steam, juegos o prefixes.
        </span>
        <span>BYTE-CX</span>
      </footer>
    </main>
  );
}

function RuntimePlanSummary({ plan }: { plan: RuntimePlan | undefined }) {
  if (!plan) return null;

  const selectedRuntime =
    plan.selection.state === "unknown"
      ? "No determinada"
      : plan.selection.state === "not_required"
        ? "No necesaria"
        : plan.selection.candidate.displayName;
  const candidateNames = plan.availableRuntimeCandidates.length
    ? plan.availableRuntimeCandidates
        .map((candidate) =>
          candidate.version
            ? `${candidate.displayName} · ${candidate.version}`
            : candidate.displayName,
        )
        .join(", ")
    : plan.candidateDiscovery === "unavailable"
      ? "No se pudo evaluar"
      : "Ninguno detectado";
  const prefixText =
    plan.prefix.state === "not_initialized"
      ? "No inicializado"
      : plan.prefix.state === "candidate_found"
        ? "Encontrado; queda como candidato"
        : plan.prefix.state === "not_required"
          ? "No requerido"
          : plan.prefix.state === "unknown"
            ? "No se pudo determinar"
            : plan.prefix.state === "invalid"
              ? "Ruta no válida"
              : plan.prefix.state === "unreadable"
                ? "No se pudo leer"
                : "No disponible";

  return (
    <section
      className="runtime-assessment"
      aria-label="Planificación del runtime"
    >
      <div className="runtime-assessment-heading">
        <h4>Preparación del entorno</h4>
        <span className={`runtime-readiness readiness-${plan.readiness}`}>
          {readinessText[plan.readiness]}
        </span>
      </div>
      <dl className="runtime-summary">
        <div>
          <dt>Runtime seleccionado</dt>
          <dd>{selectedRuntime}</dd>
        </div>
        <div>
          <dt>Candidatos Proton</dt>
          <dd>{candidateNames}</dd>
        </div>
        <div>
          <dt>Prefix</dt>
          <dd>{prefixText}</dd>
        </div>
      </dl>
      <details className="runtime-evidence">
        <summary>Evidencia de esta evaluación</summary>
        <ul>
          {plan.evidence.map((evidence, index) => (
            <li key={`${evidence.kind}-${index}`}>
              {runtimeEvidenceText(evidence)}
            </li>
          ))}
        </ul>
      </details>
    </section>
  );
}

function runtimeEvidenceText(
  evidence: RuntimePlan["evidence"][number],
): string {
  switch (evidence.kind) {
    case "game_discovery_evaluated":
      return `Descubrimiento de juegos: ${evidence.status ?? "sin estado"}.`;
    case "game_manifest_observed":
      return `Manifest ${evidence.appId ?? ""} (${evidence.name ?? "juego"}) observado en ${evidence.library ?? "biblioteca desconocida"}.`;
    case "game_directory_observed":
      return `Directorio del juego: ${evidence.status ?? "sin estado"} · ${evidence.path ?? "ruta desconocida"}. Completitud: ${evidence.completeness ?? "desconocida"}.`;
    case "runtime_discovery_evaluated":
      return `Discovery Proton: ${evidence.status ?? "sin estado"}; ${evidence.protonCandidateCount ?? 0} candidato(s).`;
    case "proton_candidate_observed":
      return `Candidato observado: ${evidence.displayName ?? "Proton"} · ${evidence.path ?? "ruta desconocida"}.`;
    case "runtime_selection_not_observed":
      return "No se encontró evidencia fiable de selección de runtime para este juego.";
    case "compat_data_observed":
      return `Compatdata: ${evidence.status ?? "sin estado"} · ${evidence.path ?? "ruta desconocida"}.`;
    case "prefix_observed":
      return `Prefix: ${evidence.state ?? "sin estado"} · ${evidence.expectedPath ?? "ruta desconocida"}.`;
    default:
      return "Evidencia registrada por LXMI.";
  }
}
