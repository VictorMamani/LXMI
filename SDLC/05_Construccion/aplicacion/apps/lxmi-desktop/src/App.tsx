import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

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

const compatdataStatusText: Record<SteamGame["compatdataStatus"], string> = {
  not_found: "No se encontró compatdata para este AppID",
  compatdata_found: "compatdata existe; no se encontró pfx",
  prefix_found: "Se encontró pfx (candidato a prefix)",
  invalid: "La ruta compatdata/pfx no es un directorio válido",
  permission_denied: "No se pudo leer compatdata/pfx por permisos",
  io_error: "No se pudo inspeccionar compatdata/pfx",
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
          <p className="eyebrow">ENTORNO LINUX · INCREMENTO 0.1</p>
          <h1>LXMI</h1>
          <p className="lede">
            Diagnóstico local de Steam. Este escaneo solo lee rutas y
            configuración.
          </p>
        </div>
        <span className="version-tag">DESARROLLO</span>
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
          <p className="muted">Escanea Steam para buscar Wuthering Waves.</p>
        )}
        {steam.kind === "error" && (
          <p className="message message-error" role="alert">
            No se pudo obtener el resultado de descubrimiento de juegos.
          </p>
        )}
        {steam.kind === "ready" && (
          <div className="game-results">
            {steam.value.gameScanStatus === "not_available" ? (
              <p className="status-line" role="status">
                Steam no está disponible para buscar juegos.
              </p>
            ) : steam.value.games.length === 0 ? (
              <p className="status-line" role="status">
                {steam.value.gameScanStatus === "partial"
                  ? "No se pudo confirmar si Wuthering Waves está instalado; revisa los avisos."
                  : "Wuthering Waves no se encontró en las bibliotecas Steam detectadas."}
              </p>
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
                    <div className="detail-wide">
                      <dt>Ruta de instalación esperada</dt>
                      <dd className="path-value">{game.installPath}</dd>
                    </div>
                    <div className="detail-wide">
                      <dt>Biblioteca Steam</dt>
                      <dd className="path-value">{game.steamLibrary}</dd>
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
                  <p className="discovery-note">
                    La presencia de compatdata/pfx no confirma qué versión de
                    Proton se usa, ni que el prefix esté sano o activo.
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

      <footer className="page-footer">
        <span>LXMI lee configuración local; no modifica Steam ni juegos.</span>
        <span>BYTE-CX</span>
      </footer>
    </main>
  );
}
