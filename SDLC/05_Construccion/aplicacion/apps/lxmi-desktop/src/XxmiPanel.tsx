import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

type Package = {
  id: string;
  kind: "xxmi_libraries" | { game_integration: string };
  version: { raw: string; evidence: string } | null;
  files: { relative_path: string; size: number; sha256: string }[];
};
type Diagnostic = { code: string; path: string | null; detail: string };
type Status = {
  storage_root: string;
  packages: Package[];
  observations: {
    path: string;
    integration_kind: "wwmi" | "zzmi" | "gimi" | "unknown" | null;
    integration: string;
    libraries: string;
    evidence: string[];
    issues: Diagnostic[];
  }[];
  issues: Diagnostic[];
};
type Plan = {
  assessment: {
    game_id: string;
    integration: "wwmi" | "zzmi" | "gimi" | "unknown";
    distribution: "steam" | "hoyoplay" | "manual" | "unknown";
    game_support: string;
    steam_support: "verified" | "unverified" | "unsupported";
    linux_proton_support: "verified" | "unverified" | "unsupported";
    platform_compatibility_verified: boolean;
    planning_possible: boolean;
    requirements: { name: string; state: string; evidence: string }[];
  };
  managed_target: string;
  game_executable_candidate: string | null;
  files: {
    source: string;
    target: string;
    action: string;
    backup_required: boolean;
  }[];
  warnings: string[];
  configuration_changes: string[];
};
type GameId = "wuthering-waves" | "zenless-zone-zero";
const games: Record<GameId, { name: string; integration: "wwmi" | "zzmi" }> = {
  "wuthering-waves": { name: "Wuthering Waves", integration: "wwmi" },
  "zenless-zone-zero": { name: "Zenless Zone Zero", integration: "zzmi" },
};
const presence: Record<string, string> = {
  absent: "No detectado",
  structurally_present: "Estructura presente · origen no verificado",
  incomplete: "Estructura incompleta",
  unreadable: "No se pudo inspeccionar",
};
const states: Record<string, string> = {
  satisfied: "Presente",
  unsatisfied: "Pendiente",
  unknown: "No determinado",
  not_required: "No necesario",
};
function errorText(error: unknown): string {
  if (
    typeof error === "object" &&
    error !== null &&
    "detail" in error &&
    typeof error.detail === "string"
  )
    return error.detail;
  return "No se pudo completar la operación local. Revisa permisos y el diagnóstico de Steam.";
}

export default function XxmiPanel() {
  const [status, setStatus] = useState<Status | null>(null);
  const [path, setPath] = useState("");
  const [gameId, setGameId] = useState<GameId>("zenless-zone-zero");
  const [selected, setSelected] = useState("");
  const [libraries, setLibraries] = useState("");
  const [plan, setPlan] = useState<Plan | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");

  async function run(action: "inspect" | "import" | "plan") {
    setBusy(true);
    setError("");
    setNotice("");
    setPlan(null);
    try {
      if (action === "plan") {
        setPlan(
          await invoke<Plan>("review_xxmi_plan", {
            gameId,
            packageId: selected,
            librariesId: libraries || null,
          }),
        );
        setNotice("Plan generado. No se aplicó ningún cambio.");
      } else {
        if (action === "import") {
          const imported = await invoke<Package>("import_xxmi_directory", {
            path: path.trim(),
          });
          setNotice(
            `Paquete verificado en almacenamiento LXMI: ${imported.id.slice(0, 12)}. No instalado en el juego.`,
          );
          if (imported.kind === "xxmi_libraries") setLibraries(imported.id);
          else if (typeof imported.kind === "object") {
            setSelected(imported.id);
            if (imported.kind.game_integration === "zzmi") {
              setGameId("zenless-zone-zero");
            } else if (imported.kind.game_integration === "wwmi") {
              setGameId("wuthering-waves");
            }
          }
        }
        setStatus(await invoke<Status>("xxmi_status"));
      }
    } catch (e: unknown) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }
  const packages = status?.packages ?? [];
  return (
    <section
      className="panel xxmi-panel"
      aria-labelledby="xxmi-heading"
      aria-busy={busy}
    >
      <div className="section-heading">
        <h2 id="xxmi-heading">
          XXMI · {games[gameId].integration.toUpperCase()}
        </h2>
        <button
          className="primary-button"
          disabled={busy}
          onClick={() => void run("inspect")}
        >
          {busy ? "Procesando…" : "Inspeccionar paquetes"}
        </button>
      </div>
      <p className="message">
        {games[gameId].name}: importa una copia local y revisa el plan
        declarativo. El plan no se aplica al juego y la compatibilidad
        Steam/Linux/Proton sigue sin verificar.
      </p>
      <div className="xxmi-form">
        <label htmlFor="xxmi-game">Juego e integración</label>
        <select
          id="xxmi-game"
          value={gameId}
          disabled={busy}
          onChange={(event) => {
            setGameId(event.target.value as GameId);
            setSelected("");
            setPlan(null);
          }}
        >
          <option value="zenless-zone-zero">Zenless Zone Zero · ZZMI</option>
          <option value="wuthering-waves">Wuthering Waves · WWMI</option>
        </select>
      </div>
      {!status && (
        <p className="discovery-note">
          Inspecciona las ubicaciones conocidas para consultar el runtime y los
          paquetes administrados. No se buscan instalaciones externas
          arbitrarias.
        </p>
      )}
      {status && (
        <>
          <p className="discovery-note">
            {packages.length
              ? `${packages.length} paquetes administrados verificados.`
              : "Todavía no hay paquetes administrados disponibles."}
          </p>
          <details className="runtime-evidence">
            <summary>Ubicaciones inspeccionadas y evidencia</summary>
            <p className="path-value">Almacenamiento: {status.storage_root}</p>
            <ul>
              {status.observations.map((item) => (
                <li key={item.path}>
                  <code className="path-value">{item.path}</code>
                  <div>
                    {item.integration_kind?.toUpperCase() ?? "Integración"}:{" "}
                    {presence[item.integration]} · Bibliotecas:{" "}
                    {presence[item.libraries]}
                  </div>
                  {item.evidence.map((e) => (
                    <p key={e}>{e}</p>
                  ))}
                  {item.issues.map((e, i) => (
                    <p key={i}>{e.detail}</p>
                  ))}
                </li>
              ))}
            </ul>
          </details>
          {status.issues.map((e, i) => (
            <p className="message-error" key={i}>
              {e.detail}
            </p>
          ))}
        </>
      )}
      <form
        className="xxmi-form"
        onSubmit={(event) => {
          event.preventDefault();
          void run("import");
        }}
      >
        <label htmlFor="xxmi-path">Carpeta local del paquete</label>
        <input
          id="xxmi-path"
          value={path}
          onChange={(event) => setPath(event.target.value)}
          placeholder="/ruta/al/paquete-extraido"
          required
          disabled={busy}
          aria-describedby="xxmi-path-help xxmi-error"
          aria-invalid={Boolean(error)}
        />
        <p id="xxmi-path-help" className="discovery-note">
          ZZMI: raíz que contiene d3dx.ini y Core/ZZMI; WWMI: d3dx.ini y
          Core/WWMI. XXMI Libraries se importa aparte junto con Manifest.json.
          Solo carpetas locales ya extraídas, no ZIP. Límite: 128 MiB por
          archivo y 512 MiB en total. El hash local no autentica el origen.
        </p>
        <button
          className="primary-button"
          type="submit"
          disabled={busy || !path.trim()}
        >
          Importar copia a LXMI
        </button>
      </form>
      <div className="xxmi-form">
        <label htmlFor="integration-package">
          Paquete {games[gameId].integration.toUpperCase()} para revisar
        </label>
        <select
          id="integration-package"
          value={selected}
          disabled={busy}
          onChange={(event) => {
            setSelected(event.target.value);
            setPlan(null);
          }}
        >
          <option value="">Seleccionar paquete…</option>
          {packages
            .filter(
              (p) =>
                typeof p.kind === "object" &&
                p.kind.game_integration === games[gameId].integration,
            )
            .map((p) => (
              <option key={p.id} value={p.id}>
                {games[gameId].integration.toUpperCase()} ·{" "}
                {p.version?.raw ?? "versión desconocida"} · {p.id.slice(0, 12)}
              </option>
            ))}
        </select>
        <label htmlFor="xxmi-libraries">
          XXMI Libraries (paquete separado)
        </label>
        <select
          id="xxmi-libraries"
          value={libraries}
          disabled={busy}
          onChange={(event) => {
            setLibraries(event.target.value);
            setPlan(null);
          }}
        >
          <option value="">No disponible / no seleccionado</option>
          {packages
            .filter((p) => p.kind === "xxmi_libraries")
            .map((p) => (
              <option key={p.id} value={p.id}>
                XXMI · {p.version?.raw ?? "versión desconocida"} ·{" "}
                {p.id.slice(0, 12)}
              </option>
            ))}
        </select>
        <button
          className="primary-button"
          disabled={busy || !selected}
          onClick={() => void run("plan")}
        >
          Revisar plan de instalación
        </button>
      </div>
      <p id="xxmi-error" className="message-error" role="alert">
        {error}
      </p>
      <p className="message" role="status">
        {notice}
      </p>
      {plan && (
        <div className="runtime-assessment">
          <h3>
            Plan {plan.assessment.integration.toUpperCase()} · sin aplicar
          </h3>
          <p>
            {plan.assessment.planning_possible
              ? "El paquete, las bibliotecas y el registro del juego permiten revisar un plan técnico. Esto no confirma compatibilidad ni habilita instalación."
              : "Falta evidencia o una dependencia para construir el plan. Revisa los requisitos pendientes."}
          </p>
          <p>
            Distribución: {plan.assessment.distribution} · Steam:{" "}
            {plan.assessment.steam_support} · Linux/Proton:{" "}
            {plan.assessment.linux_proton_support}
          </p>
          <p>
            <strong>Compatibilidad verificada: no.</strong>
          </p>
          <p>
            Destino propuesto fuera del juego:{" "}
            <code className="path-value">{plan.managed_target}</code>
          </p>
          <details className="runtime-evidence">
            <summary>Requisitos y pasos aún no ejecutables</summary>
            <ul>
              {plan.assessment.requirements.map((r) => (
                <li key={r.name}>
                  <strong>
                    {r.name}: {states[r.state]}
                  </strong>{" "}
                  — {r.evidence}
                </li>
              ))}
            </ul>
            <ul>
              {plan.configuration_changes.map((s) => (
                <li key={s}>{s}</li>
              ))}
            </ul>
          </details>
          <details className="runtime-evidence">
            <summary>{plan.files.length} archivos propuestos y backups</summary>
            <ul>
              {plan.files.map((f) => (
                <li key={f.target}>
                  <strong>
                    {f.action === "create"
                      ? "Crear"
                      : f.action === "unchanged"
                        ? "Conservar"
                        : "Reemplazar con backup"}
                  </strong>
                  <div>
                    Fuente: <code>{f.source}</code>
                  </div>
                  <div>
                    Destino: <code>{f.target}</code>
                  </div>
                </li>
              ))}
            </ul>
          </details>
          <ul className="xxmi-warnings">
            {plan.warnings.map((w) => (
              <li key={w}>{w}</li>
            ))}
          </ul>
        </div>
      )}
    </section>
  );
}
