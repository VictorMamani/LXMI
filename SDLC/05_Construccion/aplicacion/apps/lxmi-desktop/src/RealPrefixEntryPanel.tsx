import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

type EnvironmentPanel = {
  current_environment: { qualification: string; runtime_kind: string };
  executor: {
    qualification: string;
    fingerprint: string;
    wrapper: string;
    runtime_root: string;
    private_proton: string;
    fixture_runs: number;
    real_execution: string;
    identity: { proton_build: string; pressure_vessel_sha256: string };
  } | null;
  fixtures: {
    strategy: string;
    qualification: string;
    helper_successful_runs: number;
    font_differences: string[];
    pending_sections: string[];
  }[];
  evidence_sha256: string;
  evidence_source: string;
  real_prefix_entry: string;
  blocker: string;
  limitations: string[];
};
type Review = {
  review_token: string;
  consent_text: string;
  preflight: {
    result: string;
    active_processes: number;
    unresolved_processes: number;
    runtime_drift: string;
    installed_build: string | null;
    prefix_identity: { canonical_path: string; device: number; inode: number };
  };
  recovery: {
    backup_id: string;
    manifest_sha256: string;
    integrity: { state: string };
    real_prefix_restore: string;
  };
};
type Result = {
  result: string;
  helper_attempted: boolean;
  helper: {
    handshake: string;
    helper_protocol_version: number;
    runtime_manifest_hash_matches: boolean;
    runtime_path_windows: string;
    process: { exit_code: number; elapsed_millis: number };
  } | null;
  helper_error: { code: string; detail: string } | null;
  mutations: { path: string; kind: string; reason: string }[];
  prefix_identity_unchanged: boolean;
  protected_paths_unchanged: boolean;
  runtime_unchanged: boolean;
  backup_verified_after: boolean;
  game_context_inactive_after: boolean;
  evidence_receipt: { manifest_path: string; sha256: string } | null;
  warnings: string[];
};
export default function RealPrefixEntryPanel({
  preflightEligible = false,
  preflightBlockers = [],
}: {
  preflightEligible?: boolean;
  preflightBlockers?: string[];
}) {
  const [environment, setEnvironment] = useState<EnvironmentPanel | null>(null);
  const [environmentError, setEnvironmentError] = useState<string | null>(null);
  useEffect(() => {
    let mounted = true;
    void invoke<EnvironmentPanel>("inspect_linux_environment_qualification")
      .then((value) => {
        if (mounted) setEnvironment(value);
      })
      .catch((cause: unknown) => {
        if (mounted) setEnvironmentError(String(cause));
      });
    return () => {
      mounted = false;
    };
  }, []);
  // Read-only qualification evidence never grants execution; backend also enforces this.
  const environmentBlocked =
    environment?.executor?.qualification !== "qualified";
  const [review, setReview] = useState<Review | null>(null);
  const [consent, setConsent] = useState(false);
  const [result, setResult] = useState<Result | null>(null);
  const [busy, setBusy] = useState(false);
  const [attempted, setAttempted] = useState(false);
  const [error, setError] = useState<string | null>(null);
  function fail(cause: unknown) {
    const value = cause as { code?: string; detail?: string };
    setError(
      `${value.code ?? "prefix_entry_failed"}: ${value.detail ?? String(cause)}`,
    );
  }
  async function qualifyExecutor() {
    setBusy(true);
    setEnvironmentError(null);
    setReview(null);
    setConsent(false);
    try {
      await invoke("qualify_zzz_prefix_executor");
      setEnvironment(
        await invoke<EnvironmentPanel>(
          "inspect_linux_environment_qualification",
        ),
      );
    } catch (cause) {
      setEnvironmentError(
        String((cause as { detail?: string }).detail ?? cause),
      );
    } finally {
      setBusy(false);
    }
  }
  async function prepare() {
    setBusy(true);
    setError(null);
    setConsent(false);
    setReview(null);
    try {
      setReview(await invoke<Review>("prepare_zzz_harmless_prefix_entry"));
    } catch (cause) {
      fail(cause);
    } finally {
      setBusy(false);
    }
  }
  async function run() {
    if (!review || !consent || attempted) return;
    setBusy(true);
    setError(null);
    setAttempted(true);
    setConsent(false);
    try {
      setResult(
        await invoke<Result>("run_zzz_harmless_prefix_entry", {
          reviewToken: review.review_token,
          consentAccepted: true,
        }),
      );
    } catch (cause) {
      fail(cause);
    } finally {
      setBusy(false);
    }
  }
  return (
    <section className="runtime-evidence bridge-test-panel">
      <h4>Real Prefix Entry · Harmless helper</h4>
      <p>
        ZZZ debe permanecer cerrado. Esta prueba ejecuta únicamente el helper
        LXMI y lee un marker externo; no ejecuta el juego ni un loader. La
        restauración del prefix real no está probada.
      </p>
      <h4>Linux runtime environment</h4>
      <p role="status">
        Current: {environment?.current_environment.qualification ?? "UNKNOWN"} ·
        Real prefix entry:{" "}
        {environmentBlocked || !preflightEligible
          ? "Blocked · executor / prefix preflight requirements"
          : "Not run · fresh consent required"}
      </p>
      <p>
        Consultar evidencia no ejecuta Proton. El self-test de calificación
        ejecuta el helper en un prefix privado LXMI y puede tardar varios
        minutos al copiar y verificar Proton. No usa el prefix real.
      </p>
      {!preflightEligible && (
        <p role="status">
          Prefix preflight blocks real review:{" "}
          {preflightBlockers.length
            ? preflightBlockers.join(" · ")
            : "Fresh eligible prefix preflight required"}
        </p>
      )}
      <p>
        Previous attempt: DirectProton · UnexpectedMutation / blocked. Its
        evidence remains preserved.
      </p>
      <button
        disabled={busy || attempted}
        onClick={() => void qualifyExecutor()}
      >
        Qualify Executor Using Private Prefix
      </button>
      {environment?.executor && (
        <dl>
          <dt>Executor fixture self-test</dt>
          <dd>
            {environment.executor.qualification} ·{" "}
            {environment.executor.fixture_runs} measured runs
          </dd>
          <dt>Execution strategy</dt>
          <dd>Steam Runtime Container</dd>
          <dt>Steam Runtime</dt>
          <dd>{environment.executor.runtime_root}</dd>
          <dt>pressure-vessel</dt>
          <dd>
            {environment.executor.runtime_root}
            /pressure-vessel/bin/pressure-vessel-wrap
            {" · SHA-256 "}
            {environment.executor.identity.pressure_vessel_sha256}
          </dd>
          <dt>Wrapper</dt>
          <dd>{environment.executor.wrapper}</dd>
          <dt>Proton</dt>
          <dd>Private verified copy · {environment.executor.private_proton}</dd>
          <dt>Build</dt>
          <dd>{environment.executor.identity.proton_build}</dd>
          <dt>Recipe identity</dt>
          <dd>{environment.executor.fingerprint}</dd>
          <dt>Request transport</dt>
          <dd>Private file · hash / nonce / consume-once verified</dd>
          <dt>Real execution</dt>
          <dd>Not run · fresh consent required</dd>
        </dl>
      )}
      {environmentError && (
        <p role="alert">
          No se pudo leer la evidencia de calificación. La prueba real sigue
          bloqueada: {environmentError}
        </p>
      )}
      {environment ? (
        <>
          <p>Reason: {environment.blocker}</p>
          {environment.fixtures.map((fixture) => (
            <div key={fixture.strategy}>
              <h5>{fixture.strategy}</h5>
              <p>
                {fixture.qualification} · Helper:{" "}
                {fixture.helper_successful_runs} successful fixture runs
              </p>
              <ul>
                {fixture.font_differences.map((value) => (
                  <li key={value}>{value}</li>
                ))}
              </ul>
              <p>Pending differences:</p>
              {fixture.pending_sections.length ? (
                <ul>
                  {fixture.pending_sections.map((section) => (
                    <li key={section}>{section} · UNKNOWN</li>
                  ))}
                </ul>
              ) : (
                <p>
                  No unclassified registry sections in these runs; other
                  coverage limits still apply.
                </p>
              )}
            </div>
          ))}
          <p>
            Evidence: {environment.evidence_source} · SHA-256{" "}
            {environment.evidence_sha256}
          </p>
          <ul>
            {environment.limitations.map((value) => (
              <li key={value}>{value}</li>
            ))}
          </ul>
        </>
      ) : (
        <p>
          Consultando evidencia de fixtures; la entrada real permanece
          bloqueada.
        </p>
      )}
      <button
        disabled={busy || attempted || environmentBlocked || !preflightEligible}
        onClick={() => void prepare()}
      >
        Review Harmless Prefix Entry
      </button>
      <p role="status">
        {busy
          ? "Operación en curso. Copiar y verificar Proton puede tardar varios minutos; cada helper tiene un timeout de 60 s. No cierres LXMI durante la comparación."
          : attempted
            ? result
              ? `Result: ${result.result}`
              : "Attempt ended — inspect error; no automatic retry."
            : "Helper: Not run · consent required"}
      </p>
      {error && <p role="alert">{error}</p>}
      {!review && <button disabled>Run Harmless Prefix Entry Test</button>}
      {review && (
        <>
          <dl>
            <dt>Preflight</dt>
            <dd>{review.preflight.result}</dd>
            <dt>Game / prefix activity</dt>
            <dd>
              {review.preflight.active_processes === 0 &&
              review.preflight.unresolved_processes === 0
                ? "Closed / inactive at review"
                : "Active or unresolved"}
            </dd>
            <dt>Prefix identity</dt>
            <dd>
              {review.preflight.prefix_identity.canonical_path} · device{" "}
              {review.preflight.prefix_identity.device} · inode{" "}
              {review.preflight.prefix_identity.inode}
            </dd>
            <dt>Runtime</dt>
            <dd>
              {review.preflight.runtime_drift} ·{" "}
              {review.preflight.installed_build ?? "UNKNOWN"}
            </dd>
            <dt>Recovery backup</dt>
            <dd>
              {review.recovery.integrity.state} · {review.recovery.backup_id}
            </dd>
            <dt>Real restore</dt>
            <dd>Not tested</dd>
          </dl>
          <p>{review.consent_text}</p>
          <label>
            <input
              type="checkbox"
              checked={consent}
              disabled={busy || attempted}
              onChange={(event) => setConsent(event.target.checked)}
            />{" "}
            I explicitly consent to this single harmless real-prefix entry
            experiment. I will keep ZZZ closed.
          </label>
          <div className="actions">
            <button
              disabled={busy || !consent || attempted || environmentBlocked}
              onClick={() => void run()}
            >
              Run Harmless Prefix Entry Test
            </button>
          </div>
        </>
      )}
      {result && (
        <>
          <dl>
            <dt>Result</dt>
            <dd>{result.result}</dd>
            <dt>Helper</dt>
            <dd>
              {result.helper
                ? `Success · protocol ${result.helper.helper_protocol_version} · handshake ${result.helper.handshake} · hash ${result.helper.runtime_manifest_hash_matches ? "matched" : "mismatch"} · exit ${result.helper.process.exit_code}`
                : result.helper_error
                  ? `${result.helper_error.code}: ${result.helper_error.detail}`
                  : "Not run"}
            </dd>
            <dt>Windows marker path</dt>
            <dd>{result.helper?.runtime_path_windows ?? "Not observed"}</dd>
            <dt>Prefix identity after</dt>
            <dd>
              {result.prefix_identity_unchanged
                ? "Verified unchanged"
                : "Changed / unknown"}
            </dd>
            <dt>Runtime after</dt>
            <dd>
              {result.runtime_unchanged ? "SameBuild" : "Changed / unknown"}
            </dd>
            <dt>Protected paths</dt>
            <dd>
              {result.protected_paths_unchanged
                ? "Unchanged within monitored coverage"
                : "Changed / unavailable"}
            </dd>
            <dt>Backup after</dt>
            <dd>
              {result.backup_verified_after
                ? "Verified; original backup preserved"
                : "Invalid / unavailable"}
            </dd>
            <dt>Game context after</dt>
            <dd>
              {result.game_context_inactive_after
                ? "Inactive at check"
                : "Active / unresolved"}
            </dd>
            <dt>Evidence</dt>
            <dd>
              {result.evidence_receipt
                ? `${result.evidence_receipt.manifest_path} · SHA-256 ${result.evidence_receipt.sha256}`
                : "Receipt unavailable"}
            </dd>
          </dl>
          <h4>Before / after mutations</h4>
          {result.mutations.length === 0 ? (
            <p>
              No differences detected within the selected snapshot coverage.
            </p>
          ) : (
            <ul>
              {result.mutations.map((change) => (
                <li key={change.path}>
                  {change.path} · {change.kind} · {change.reason}
                </li>
              ))}
            </ul>
          )}
          <ul>
            {result.warnings.map((warning) => (
              <li key={warning}>{warning}</li>
            ))}
          </ul>
        </>
      )}
      <p>
        No real-prefix rollback, game launch or injection control is available
        here. Loader strategy and game compatibility remain unverified.
      </p>
    </section>
  );
}
