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
type InjectResult = {
  mode: Mode;
  outcome: string;
  nonce: string;
  marker_verified: boolean;
  target_launcher_identity: {
    pid: number;
    start_time_ticks: number | null;
  } | null;
  target: { pid: number; path_readable: boolean; wine_prefix: string };
  injector: {
    pid: number;
    target_pid: number;
    path_readable: boolean;
    identity_bound: boolean;
    target_identity: {
      result: string;
      expected_windows_pid: number;
      observed_windows_pid: number;
      expected_image: string;
      observed_image: string;
      expected_creation_time: string;
      observed_creation_time: string;
    };
    inject_called: boolean;
    inject_code: number;
    windows_error: number;
    wine_prefix: string;
  };
  identity: Record<string, string>;
  context_target: Context;
  context_injector: Context;
  warnings: string[];
};
type ReentryResult = {
  result: string;
  prefix_relation: string;
  named_object_visible: boolean;
  target_identity_verified: boolean;
  initial_target_received_event: boolean;
  fixture: InjectResult & {
    reentry_evidence: {
      target_ready_before_second_invocation: boolean;
      prefix_a: {
        canonical_path: string;
        device: number;
        inode: number;
      } | null;
      prefix_b: {
        canonical_path: string;
        device: number;
        inode: number;
      } | null;
      endpoint_a: {
        derived_socket_path: string | null;
        socket_observed: boolean;
      };
      endpoint_b: {
        derived_socket_path: string | null;
        socket_observed: boolean;
      };
    };
  };
};
type Runtime = {
  proton_script: string;
  display_name: string;
  version: string | null;
};

export default function ContextLabPanel({
  onReentryResult,
}: {
  onReentryResult?: (state: string) => void;
}) {
  const [runtimes, setRuntimes] = useState<Runtime[]>([]);
  const [runtime, setRuntime] = useState("");
  const [reentryConsent, setReentryConsent] = useState(false);
  const [reentryResults, setReentryResults] = useState<
    Partial<Record<Mode, ReentryResult>>
  >({});
  const [consent, setConsent] = useState(false);
  const [running, setRunning] = useState<Mode | "inspect" | null>(null);
  const [error, setError] = useState("");
  const [injectResults, setInjectResults] = useState<
    Partial<Record<Mode, InjectResult>>
  >({});
  const [injectConsent, setInjectConsent] = useState(false);
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
  async function runInject(mode: Mode) {
    setRunning(mode);
    setError("");
    try {
      const result = await invoke<InjectResult>("run_direct_context_test", {
        protonScript: runtime,
        mode,
        sideEffectsAcknowledged: injectConsent,
      });
      setInjectResults((previous) => ({ ...previous, [mode]: result }));
    } catch (cause) {
      setError(describeError(cause));
    } finally {
      setRunning(null);
    }
  }
  async function runReentry(mode: Mode) {
    setRunning(mode);
    setError("");
    try {
      const result = await invoke<ReentryResult>("run_context_reentry_test", {
        protonScript: runtime,
        mode,
        sideEffectsAcknowledged: reentryConsent,
      });
      setReentryResults((previous) => ({ ...previous, [mode]: result }));
      if (mode === "same_prefix") onReentryResult?.(result.result);
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
      <section>
        <h3>Existing Context Re-entry</h3>
        <p>
          El target LXMI se inicia primero. Una segunda invocación independiente
          de Proton prueba evento e identidad en el contexto ya activo. No copia
          DLLs ni carga un loader. Proton escribe únicamente en prefixes
          privados del laboratorio.
        </p>
        <label>
          <input
            type="checkbox"
            checked={reentryConsent}
            disabled={running !== null}
            onChange={(event) => setReentryConsent(event.target.checked)}
          />{" "}
          Acepto el test aislado de reentrada sin loader.
        </label>
        <div className="action-row">
          <button
            disabled={!reentryConsent || !runtime || running !== null}
            onClick={() => void runReentry("same_prefix")}
          >
            Run Same-Prefix Re-entry
          </button>
          <button
            className="secondary-button"
            disabled={!reentryConsent || !runtime || running !== null}
            onClick={() => void runReentry("different_prefixes")}
          >
            Run Different-Prefix Control
          </button>
        </div>
        {(["same_prefix", "different_prefixes"] as const).map((mode) => {
          const result = reentryResults[mode];
          const evidence = result?.fixture.reentry_evidence;
          return (
            <section key={mode}>
              <h4>
                {mode === "same_prefix"
                  ? "Reentrada · mismo prefix"
                  : "Reentrada · prefixes distintos"}
              </h4>
              {!result || !evidence ? (
                <p>NO COMPROBADO · Todavía sin resultado.</p>
              ) : (
                <>
                  <p role="status">
                    HOST TEST · Re-entry: {result.result} · nonce{" "}
                    {result.fixture.nonce}
                  </p>
                  <dl className="diagnostic-grid">
                    <div>
                      <dt>Target ya activo antes de la segunda invocación</dt>
                      <dd>
                        {evidence.target_ready_before_second_invocation
                          ? "Confirmado por ready; helper iniciado después"
                          : "UNKNOWN"}
                      </dd>
                    </div>
                    <div>
                      <dt>Identidad del target</dt>
                      <dd>
                        {result.fixture.injector.target_identity.result} · Win32{" "}
                        {result.fixture.injector.windows_error}
                      </dd>
                      <dd className="path-value">
                        {result.fixture.injector.target_identity
                          .observed_image || "Imagen no disponible"}
                      </dd>
                      <dd>
                        Creación esperada / observada:{" "}
                        {
                          result.fixture.injector.target_identity
                            .expected_creation_time
                        }{" "}
                        /{" "}
                        {
                          result.fixture.injector.target_identity
                            .observed_creation_time
                        }
                      </dd>
                    </div>
                    <div>
                      <dt>Evento con nonce</dt>
                      <dd>
                        {result.named_object_visible ? "Visible" : "No visible"}{" "}
                        · señal recibida por A:{" "}
                        {result.initial_target_received_event ? "sí" : "no"}
                      </dd>
                    </div>
                    <div>
                      <dt>Prefix filesystem identity</dt>
                      <dd>{result.prefix_relation}</dd>
                      {[evidence.prefix_a, evidence.prefix_b].map((p, i) => (
                        <dd className="path-value" key={i}>
                          {i === 0 ? "A" : "B"}:{" "}
                          {p
                            ? `${p.canonical_path} · dev ${p.device} · inode ${p.inode}`
                            : "UNKNOWN"}
                        </dd>
                      ))}
                    </div>
                    <div>
                      <dt>Endpoint Wine candidato · DERIVED</dt>
                      {[evidence.endpoint_a, evidence.endpoint_b].map(
                        (e, i) => (
                          <dd className="path-value" key={i}>
                            {i === 0 ? "A" : "B"}:{" "}
                            {e.derived_socket_path ?? "UNKNOWN"} · socket
                            observado: {e.socket_observed ? "sí" : "no"}
                          </dd>
                        ),
                      )}
                      <dd>
                        PID de wineserver y conexión exacta de peers: UNKNOWN
                      </dd>
                    </div>
                    <div>
                      <dt>Filesystem / loader</dt>
                      <dd>
                        Marker A/B:{" "}
                        {result.fixture.target.path_readable &&
                        result.fixture.injector.path_readable
                          ? "legible"
                          : "no confirmado"}{" "}
                        · Inject: no llamado
                      </dd>
                    </div>
                  </dl>
                  <p>
                    Contexto ZZZ observado en 0.8.3: Proton Experimental y
                    prefix 4162040. Reentrada en prefix del juego: no probada;
                    compatibilidad loader: UNKNOWN; gate cerrado.
                  </p>
                </>
              )}
            </section>
          );
        })}
      </section>
      <section>
        <h3>Direct Inject Context Boundary</h3>
        <p>
          Solo ejecuta programas Windows propios de LXMI en prefixes Proton
          aislados. No interactúa con Steam ni juegos. El PID Windows debe
          vincularse con la ruta exacta de esta sesión, su fecha de creación y
          el nonce antes de llamar al loader oficial. Solo se consulta el PID
          del target propio; no se enumeran procesos.
        </p>
        <label>
          <input
            type="checkbox"
            checked={injectConsent}
            disabled={running !== null}
            onChange={(event) => setInjectConsent(event.target.checked)}
          />{" "}
          Acepto el test aislado de Direct Inject contra el fixture LXMI.
        </label>
        <div className="action-row">
          <button
            disabled={!injectConsent || !runtime || running !== null}
            onClick={() => void runInject("same_prefix")}
          >
            Run Same-Prefix Fixture
          </button>
          <button
            className="secondary-button"
            disabled={!injectConsent || !runtime || running !== null}
            onClick={() => void runInject("different_prefixes")}
          >
            Run Cross-Prefix Fixture
          </button>
        </div>
        {(["same_prefix", "different_prefixes"] as const).map((mode) => {
          const result = injectResults[mode];
          return (
            <section key={mode}>
              <h4>
                {mode === "same_prefix"
                  ? "Direct Inject · mismo prefix"
                  : "Direct Inject · prefixes distintos"}
              </h4>
              {!result ? (
                <p>NO COMPROBADO · Todavía sin resultado.</p>
              ) : (
                <>
                  <p role="status">
                    HOST TEST · {result.outcome} · Marker + nonce:{" "}
                    {result.marker_verified ? "PASS" : "no verificado"}
                  </p>
                  <dl className="diagnostic-grid">
                    <div>
                      <dt>Target propio</dt>
                      <dd>
                        PID Windows {result.target.pid} · filesystem{" "}
                        {result.target.path_readable ? "legible" : "no legible"}
                      </dd>
                      <dd className="path-value">
                        {result.target.wine_prefix}
                      </dd>
                    </div>
                    <div>
                      <dt>Runner propio</dt>
                      <dd>
                        PID Windows {result.injector.pid} · filesystem{" "}
                        {result.injector.path_readable
                          ? "legible"
                          : "no legible"}
                      </dd>
                      <dd className="path-value">
                        {result.injector.wine_prefix}
                      </dd>
                    </div>
                    <div>
                      <dt>Identidad antes de Inject</dt>
                      <dd>
                        {result.injector.target_identity.result} ·{" "}
                        {result.injector.identity_bound
                          ? "VERIFIED"
                          : "BLOCKED_NOT_ATTEMPTED"}
                      </dd>
                    </div>
                    <div>
                      <dt>Imagen exacta · esperada / consultada</dt>
                      <dd className="path-value">
                        {result.injector.target_identity.expected_image}
                      </dd>
                      <dd className="path-value">
                        {result.injector.target_identity.observed_image ||
                          "No disponible"}
                      </dd>
                    </div>
                    <div>
                      <dt>Creación FILETIME · esperada / consultada</dt>
                      <dd>
                        {result.injector.target_identity.expected_creation_time}{" "}
                        /{" "}
                        {result.injector.target_identity.observed_creation_time}
                      </dd>
                    </div>
                    <div>
                      <dt>
                        Child Linux del launcher Proton · distinto del PID
                        Windows
                      </dt>
                      <dd>
                        {result.target_launcher_identity
                          ? `${result.target_launcher_identity.pid} · start ticks ${result.target_launcher_identity.start_time_ticks ?? "UNKNOWN"}`
                          : "UNKNOWN"}
                      </dd>
                    </div>
                    <div>
                      <dt>API upstream</dt>
                      <dd>
                        {result.injector.inject_called
                          ? `Invocada · resultado ${result.injector.inject_code}`
                          : "No invocada"}{" "}
                        · Win32 {result.injector.windows_error}
                      </dd>
                    </div>
                    <div>
                      <dt>Wineserver</dt>
                      <dd>
                        UNKNOWN · candidatos target{" "}
                        {result.context_target.wineserver_candidates.join(
                          ", ",
                        ) || "ninguno"}{" "}
                        · runner{" "}
                        {result.context_injector.wineserver_candidates.join(
                          ", ",
                        ) || "ninguno"}
                      </dd>
                    </div>
                  </dl>
                  <ul>
                    {Object.entries(result.identity).map(([key, value]) => (
                      <li key={key}>
                        {key}: {value}
                      </li>
                    ))}
                  </ul>
                  <p>Nonce: {result.nonce}</p>
                  {!result.injector.inject_called && (
                    <p>
                      Injection was not attempted because the Windows PID could
                      not be bound unambiguously to LXMI's test target from this
                      Wine context.
                    </p>
                  )}
                  <p>
                    Filesystem compartido no demuestra acceso al proceso. Un
                    bloqueo de identidad no es un fallo observado de Inject.
                  </p>
                </>
              )}
            </section>
          );
        })}
      </section>
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
