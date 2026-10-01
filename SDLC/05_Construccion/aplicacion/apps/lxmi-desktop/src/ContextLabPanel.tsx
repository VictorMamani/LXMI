import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Mode = "same_prefix" | "different_prefixes";
type Evidence = { state: string; value: string | null; source: string };
type Context = {
  prefix: Evidence;
  compatdata_root: Evidence;
  wineserver_identity: Evidence;
  wineserver_candidates: number[];
  drive_mapping: { windows_path: string | null; evidence: string };
};
type Peer = {
  pid: number;
  path_readable: boolean;
  probe_succeeded: boolean;
  windows_error: number;
};
type Result = {
  mode: Mode;
  nonce: string;
  explicit_runtime: { display_name: string; version: string | null };
  peer_a: Peer;
  peer_b: Peer;
  context_a: Context;
  context_b: Context;
  identity: Record<string, string>;
  probe_shared: boolean;
  warnings: string[];
};
type Runtime = {
  proton_script: string;
  display_name: string;
  version: string | null;
};

export default function ContextLabPanel() {
  const [runtimes, setRuntimes] = useState<Runtime[]>([]);
  const [runtime, setRuntime] = useState("");
  const [consent, setConsent] = useState(false);
  const [running, setRunning] = useState<Mode | "inspect" | null>(null);
  const [error, setError] = useState("");
  const [results, setResults] = useState<Partial<Record<Mode, Result>>>({});
  async function inspect() {
    setRunning("inspect");
    setError("");
    try {
      const panel = await invoke<{ runtimes: Runtime[] }>(
        "inspect_runtime_bridge",
        { zzmiId: null, librariesId: null },
      );
      setRuntimes(panel.runtimes);
    } catch (cause) {
      setError(describeError(cause));
    } finally {
      setRunning(null);
    }
  }
  async function run(mode: Mode) {
    setRunning(mode);
    setError("");
    try {
      const result = await invoke<Result>("run_context_lab_test", {
        protonScript: runtime,
        mode,
        sideEffectsAcknowledged: consent,
      });
      setResults((previous) => ({ ...previous, [mode]: result }));
    } catch (cause) {
      setError(describeError(cause));
    } finally {
      setRunning(null);
    }
  }
  return (
    <details className="runtime-evidence bridge-test-panel">
      <summary>Advanced · Proton/Wine Context Lab</summary>
      <p>
        Dos peers de LXMI prueban un evento Windows con nonce en prefixes
        privados. Proton puede crear o actualizar estos prefixes. No se usa el
        prefix de ZZZ ni se carga ningún loader.
      </p>
      <button
        className="secondary-button"
        disabled={running !== null}
        onClick={() => void inspect()}
      >
        Consultar Proton de laboratorio
      </button>
      <label htmlFor="context-lab-runtime">
        Proton explícito del laboratorio
      </label>
      <select
        id="context-lab-runtime"
        value={runtime}
        disabled={running !== null}
        onChange={(event) => setRuntime(event.target.value)}
      >
        <option value="">Elegir Proton…</option>
        {runtimes.map((tool) => (
          <option key={tool.proton_script} value={tool.proton_script}>
            {tool.display_name} · {tool.version ?? "versión desconocida"}
          </option>
        ))}
      </select>
      <label>
        <input
          type="checkbox"
          checked={consent}
          disabled={running !== null}
          onChange={(event) => setConsent(event.target.checked)}
        />{" "}
        Acepto crear prefixes aislados y ejecutar únicamente los peers LXMI.
      </label>
      <div className="action-row">
        <button
          disabled={!consent || !runtime || running !== null}
          onClick={() => void run("same_prefix")}
        >
          Probar mismo prefix
        </button>
        <button
          className="secondary-button"
          disabled={!consent || !runtime || running !== null}
          onClick={() => void run("different_prefixes")}
        >
          Probar prefixes distintos
        </button>
      </div>
      <p role="status">
        {running
          ? `En curso: ${running}. Incluye preparación de prefixes y espera acotada de peers.`
          : "Sin ejecución activa. Los últimos resultados permanecen visibles."}
      </p>
      {error && (
        <p role="alert">
          Prueba no completada: {error}. Si faltan peers, ejecuta
          tools/lxmi-context-peers/build.sh y vuelve a intentar.
        </p>
      )}
      {(["same_prefix", "different_prefixes"] as const).map((mode) => {
        const result = results[mode];
        return (
          <section key={mode}>
            <h3>
              {mode === "same_prefix" ? "Mismo prefix" : "Prefixes distintos"}
            </h3>
            {!result ? (
              <p>NO COMPROBADO · Todavía sin resultado.</p>
            ) : (
              <>
                <p>
                  Evento compartido: {result.probe_shared ? "sí" : "no"} · HOST
                  TEST · {result.explicit_runtime.display_name}
                </p>
                <dl className="diagnostic-grid">
                  {(["a", "b"] as const).map((peer) => {
                    const response =
                      peer === "a" ? result.peer_a : result.peer_b;
                    const context =
                      peer === "a" ? result.context_a : result.context_b;
                    return (
                      <div key={peer}>
                        <dt>Peer {peer.toUpperCase()}</dt>
                        <dd>
                          PID Windows {response.pid} · marker{" "}
                          {response.path_readable ? "legible" : "no legible"} ·
                          evento{" "}
                          {response.probe_succeeded
                            ? "accesible"
                            : "no accesible"}{" "}
                          · Win32 {response.windows_error}
                        </dd>
                        <dd className="path-value">
                          Prefix: {context.prefix.value ?? "UNKNOWN"} ·{" "}
                          {context.prefix.state}
                        </dd>
                        <dd className="path-value">
                          Ruta Windows:{" "}
                          {context.drive_mapping.windows_path ?? "UNKNOWN"}
                        </dd>
                        <dd>
                          Wineserver: {context.wineserver_identity.state} ·
                          candidatos efímeros{" "}
                          {context.wineserver_candidates.join(", ") ||
                            "ninguno"}
                        </dd>
                      </div>
                    );
                  })}
                </dl>
                <ul>
                  {Object.entries(result.identity).map(([name, relation]) => (
                    <li key={name}>
                      {name}: {relation}
                    </li>
                  ))}
                </ul>
                <p>
                  Visibilidad de un evento no prueba visibilidad de procesos ni
                  compatibilidad ZZMI. GameRuntimePlan.selected y la estrategia
                  loader permanecen separados.
                </p>
              </>
            )}
          </section>
        );
      })}
      <p>
        ZZZ: contexto observado en 0.8.3 con Proton Experimental y compatdata
        4162040. Requisitos del helper para compartir contexto: UNKNOWN.
        Experimento con el juego: no autorizado.
      </p>
    </details>
  );
}
function describeError(cause: unknown): string {
  if (cause && typeof cause === "object" && "detail" in cause) {
    return `${String(cause.detail)}${"code" in cause ? ` (${String(cause.code)})` : ""}`;
  }
  return String(cause);
}
