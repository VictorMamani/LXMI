import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import RealPrefixEntryPanel from "./RealPrefixEntryPanel";

type Preflight = {
  result: string;
  prefix_identity: {
    canonical_path: string;
    uid: number;
    device: number;
    inode: number;
  } | null;
  installed_build: string | null;
  observed_build: string;
  runtime_drift: string;
  active_processes: number;
  unresolved_processes: number;
  rollback: string;
  rollback_limitations: string[];
  blockers: string[];
  expected_mutations: string[];
  baseline_receipt: { manifest_path: string; sha256: string } | null;
  baseline: {
    selected_prefix_paths: {
      path: string;
      sha256: string | null;
      issue: string | null;
    }[];
    protected_paths: {
      path: string;
      sha256: string | null;
      issue: string | null;
    }[];
    policy: { exclusions: string[] };
  };
};

type Verification = {
  state: string;
  matching_files: number;
  matching_entries: number;
  mismatches: string[];
  unsupported_metadata: string[];
};
type Recovery = {
  backup_id: string;
  backup_path: string;
  manifest_sha256: string;
  strategy: string;
  source_size: {
    apparent_bytes: number;
    allocated_bytes: number;
    files: number;
    directories: number;
    symlinks: number;
  };
  backup_size: { apparent_bytes: number; allocated_bytes: number };
  available_bytes_before: number;
  available_bytes_after: number;
  integrity: Verification;
  source_unchanged_during_capture: boolean;
  restore_path: string | null;
  restore_verification: Verification | null;
  real_prefix_restore: string;
  rollback: string;
  retention_policy: string;
};
type Reconciliation = {
  old_backup_id: string;
  old_backup_integrity: Verification;
  summary: {
    added: number;
    removed: number;
    modified: number;
    type_changed: number;
    symlink_target_changed: number;
    metadata_only_changed: number;
    unknown_or_unexpected: number;
  };
  assessment: string;
  source_consistent_during_comparison: boolean;
  protected_drift: { path: string; evidence: string }[];
  proton_source_matches_qualified_source: boolean;
  blockers: string[];
  evidence_receipt: { manifest_path: string; sha256: string } | null;
  registry: {
    relative_path: string;
    classification: string;
    historical_before_hash_matches: boolean;
    historical_after_hash_matches: boolean;
    diagnosis: {
      font_sections: number;
      font_values_added: number;
      font_values_modified: number;
    };
    sections: { section: string; classification: string }[];
  }[];
};
type SteamField = { key: string; state: string; safe_value: string | null };
type SteamSemantics = {
  compat_tool_mapping_present: boolean;
  game_mapping_present: boolean;
  global_mapping_present: boolean;
  game_mapping: SteamField[];
  global_mapping: SteamField[];
  launch_options: SteamField[];
  launch_overrides: SteamField[];
  relevant_game_properties: SteamField[];
  issues: string[];
  fingerprint: string;
};
type SteamReview = {
  assessment: string;
  stability: string;
  observation_seconds: number;
  historical_writer: string;
  historical_exact_diff: string;
  after: {
    config_raw: { path: string; sha256: string };
    localconfig_raw: { path: string; sha256: string };
    config: SteamSemantics;
    localconfig: SteamSemantics;
    semantic_fingerprint: string;
  };
};
type Panel = {
  preflight: Preflight;
  recovery: Recovery | null;
  historical_recovery: Recovery | null;
  reconciliation: Reconciliation | null;
  steam_review: SteamReview | null;
  steam_review_receipt: { manifest_path: string; sha256: string } | null;
};
type Action =
  | "create_backup"
  | "verify_backup"
  | "test_restore"
  | "capture_accepted_baseline"
  | "create_current_backup";
function storageSize(bytes: number) {
  return `${bytes.toLocaleString()} bytes · ${(bytes / 1024 / 1024).toFixed(1)} MiB`;
}

export default function PrefixPreflightPanel() {
  const [report, setReport] = useState<Preflight | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [recovery, setRecovery] = useState<Recovery | null>(null);
  const [operation, setOperation] = useState("");
  const [reconciliation, setReconciliation] = useState<Reconciliation | null>(
    null,
  );
  const [historicalRecovery, setHistoricalRecovery] = useState<Recovery | null>(
    null,
  );
  const [steamReview, setSteamReview] = useState<SteamReview | null>(null);
  const [steamReceipt, setSteamReceipt] =
    useState<Panel["steam_review_receipt"]>(null);
  function accept(panel: Panel) {
    setSteamReview(panel.steam_review);
    setSteamReceipt(panel.steam_review_receipt);
    setReport(panel.preflight);
    setRecovery(panel.recovery);
    setReconciliation(panel.reconciliation);
    setHistoricalRecovery(panel.historical_recovery);
  }
  function fail(cause: unknown) {
    const detail = cause as { code?: string; detail?: string };
    setError(
      `${detail.code ?? "recovery_failed"}: ${detail.detail ?? String(cause)}`,
    );
    // Preserve historical evidence on failures; stale reconciliation cannot grant refresh.
    setReconciliation(null);
    setReport(null);
  }
  async function recover(action: Action) {
    setBusy(true);
    setError(null);
    setOperation(
      action === "create_backup"
        ? "Inventariando, copiando y verificando el prefix; máximo 600 s."
        : action === "test_restore"
          ? "Restaurando solo en sandbox LXMI y comparando el árbol; máximo 600 s."
          : "Verificando todos los archivos del backup.",
    );
    try {
      accept(await invoke<Panel>("run_zzz_prefix_recovery", { action }));
    } catch (cause) {
      fail(cause);
    } finally {
      setBusy(false);
    }
  }
  async function reviewSteam() {
    setBusy(true);
    setError(null);
    setOperation(
      "Revisando solo configuración relevante de ZZZ, estabilidad durante 5 s y recuperación histórica. No se ejecuta Proton.",
    );
    try {
      accept(await invoke<Panel>("review_zzz_steam_protected_state"));
    } catch (cause) {
      setSteamReview(null);
      fail(cause);
    } finally {
      setBusy(false);
    }
  }
  async function reconcileSource() {
    setBusy(true);
    setError(null);
    setOperation(
      "Comparando el árbol completo sin seguir symlinks y revisando estado protegido; no se ejecuta Proton.",
    );
    try {
      accept(await invoke<Panel>("reconcile_zzz_prefix_recovery"));
    } catch (cause) {
      fail(cause);
    } finally {
      setBusy(false);
    }
  }
  async function inspect(captureBaseline: boolean) {
    setBusy(true);
    setError(null);
    setOperation("Inspeccionando y reevaluando la evidencia del prefix…");
    try {
      accept(
        await invoke<Panel>("inspect_zzz_prefix_preflight", {
          captureBaseline,
        }),
      );
    } catch (cause) {
      fail(cause);
    } finally {
      setBusy(false);
    }
  }
  return (
    <details className="runtime-evidence bridge-test-panel">
      <summary>Advanced · ZZZ Prefix Preflight</summary>
      <p>
        Inspección de solo lectura del prefix de ZZZ. El baseline se guarda en
        almacenamiento LXMI; no contiene un backup recuperable. Esta inspección
        no ejecuta Proton ni el juego. La prueba de entry separada requiere
        consentimiento explícito.
      </p>
      <div className="actions">
        <button disabled={busy} onClick={() => void inspect(false)}>
          Consultar prefix preflight
        </button>
        <button
          disabled={
            busy ||
            Boolean(
              report?.blockers.includes("recovery_source_current_mismatch"),
            ) ||
            Boolean(
              reconciliation &&
              reconciliation.assessment !== "understood_and_acceptable",
            )
          }
          onClick={() => void inspect(true)}
        >
          Capture Prefix Baseline
        </button>
      </div>
      <p role="status">
        {busy
          ? operation
          : report
            ? `Preflight: ${report.result}`
            : "Sin inspección. Consulta el prefix para revisar evidencia y limitaciones."}
      </p>
      {error && <p role="alert">{error}</p>}
      <h4>Steam Protected State</h4>
      <p>
        Revisión de solo lectura de config.vdf y localconfig.vdf. Conserva
        hashes diagnósticos y protege las opciones de arranque y compatibilidad
        de ZZZ. No guarda archivos completos, credenciales, tickets ni datos
        privados.
      </p>
      <button disabled={busy} onClick={() => void reviewSteam()}>
        Run Steam Protected State Review
      </button>
      {steamReview && (
        <>
          <dl>
            <dt>Steam protected assessment</dt>
            <dd>{steamReview.assessment}</dd>
            <dt>Stability observation</dt>
            <dd>
              {steamReview.stability} · {steamReview.observation_seconds} s
            </dd>
            <dt>Historical writer</dt>
            <dd>{steamReview.historical_writer}</dd>
            <dt>Historical exact diff</dt>
            <dd>{steamReview.historical_exact_diff}</dd>
            <dt>Current semantic fingerprint</dt>
            <dd>{steamReview.after.semantic_fingerprint}</dd>
            <dt>Selected review evidence</dt>
            <dd>
              {steamReceipt?.manifest_path} · SHA-256 {steamReceipt?.sha256}
            </dd>
          </dl>
          {(["config", "localconfig"] as const).map((kind) => {
            const state = steamReview.after[kind];
            const raw = steamReview.after[`${kind}_raw`];
            return (
              <div key={kind}>
                <h5>{kind}.vdf</h5>
                <dl>
                  <dt>Raw hash · diagnostic</dt>
                  <dd>
                    {raw.sha256} · historical raw drift retained separately
                  </dd>
                  <dt>Relevant semantic state</dt>
                  <dd>
                    {state.issues.length
                      ? state.issues.join(" · ")
                      : "Reviewed; selected fields understood"}
                  </dd>
                  <dt>CompatToolMapping</dt>
                  <dd>
                    {state.compat_tool_mapping_present
                      ? "Present"
                      : "Absent in this document"}
                  </dd>
                  <dt>AppID 4162040 explicit mapping</dt>
                  <dd>
                    {state.game_mapping.length
                      ? state.game_mapping
                          .map((f) => `${f.key}: ${f.safe_value ?? f.state}`)
                          .join(" · ")
                      : "Absent · configured selection Unknown"}
                  </dd>
                  <dt>Global mapping · AppID 0</dt>
                  <dd>
                    {state.global_mapping.length
                      ? state.global_mapping
                          .map((f) => `${f.key}: ${f.safe_value ?? f.state}`)
                          .join(" · ")
                      : "Absent"}
                  </dd>
                  <dt>ZZZ launch options</dt>
                  <dd>
                    {state.launch_options.length
                      ? state.launch_options.map((f) => f.state).join(" · ")
                      : "Absent"}
                  </dd>
                  <dt>ZZZ launch override</dt>
                  <dd>
                    {state.launch_overrides.length
                      ? state.launch_overrides.map((f) => f.state).join(" · ")
                      : "Absent"}
                  </dd>
                  <dt>Selected game properties</dt>
                  <dd>
                    {state.relevant_game_properties.length
                      ? state.relevant_game_properties
                          .map((f) => `${f.key}: ${f.safe_value ?? f.state}`)
                          .join(" · ")
                      : "None affecting this reviewed experiment"}
                  </dd>
                </dl>
              </div>
            );
          })}
        </>
      )}
      <h4>Recovery source reconciliation</h4>
      <p>
        Comparación de solo lectura contra el backup histórico. Los hashes no
        identifican al autor de una escritura. Una nueva recuperación exige
        diferencias entendidas y estado protegido aceptable; no autoriza helper,
        consentimiento ni restore real.
      </p>
      <button disabled={busy} onClick={() => void reconcileSource()}>
        Reconcile Recovery Source — Read Only
      </button>
      {reconciliation && (
        <>
          <dl>
            <dt>Old backup · HISTORICAL</dt>
            <dd>
              {reconciliation.old_backup_id} ·{" "}
              {reconciliation.old_backup_integrity.state}
            </dd>
            <dt>Current prefix vs old backup</dt>
            <dd>
              {Object.values(reconciliation.summary).some((n) => n > 0)
                ? "Mismatch"
                : "Matching within manifest policy"}
            </dd>
            <dt>Drift summary</dt>
            <dd>
              {reconciliation.summary.added} added ·{" "}
              {reconciliation.summary.removed} removed ·{" "}
              {reconciliation.summary.modified} modified ·{" "}
              {reconciliation.summary.type_changed} type changed ·{" "}
              {reconciliation.summary.symlink_target_changed} symlink changes ·{" "}
              {reconciliation.summary.metadata_only_changed} metadata only
            </dd>
            <dt>Unresolved material drift</dt>
            <dd>{reconciliation.summary.unknown_or_unexpected}</dd>
            <dt>Source consistency</dt>
            <dd>
              {reconciliation.source_consistent_during_comparison
                ? "Verified during comparison"
                : "UNKNOWN / changed"}
            </dd>
            <dt>Assessment</dt>
            <dd>{reconciliation.assessment}</dd>
            <dt>Approved Proton source</dt>
            <dd>
              {reconciliation.proton_source_matches_qualified_source
                ? "Matches qualified source inventory"
                : "Changed / needs review"}
            </dd>
            <dt>Complete drift evidence</dt>
            <dd>
              {reconciliation.evidence_receipt?.manifest_path} · SHA-256{" "}
              {reconciliation.evidence_receipt?.sha256}
            </dd>
          </dl>
          {reconciliation.registry.map((r) => (
            <p key={r.relative_path}>
              {r.relative_path}: {r.classification} · historical before/after
              hash correlation: {String(r.historical_before_hash_matches)} /{" "}
              {String(r.historical_after_hash_matches)} ·{" "}
              {r.diagnosis.font_sections} Fonts sections ·{" "}
              {r.diagnosis.font_values_added} added values /{" "}
              {r.diagnosis.font_values_modified} rewritten values
            </p>
          ))}
          {reconciliation.protected_drift.length > 0 && (
            <>
              <h5>Historical protected raw differences</h5>
              <ul>
                {reconciliation.protected_drift.map((d, i) => (
                  <li key={`${d.path}-${i}`}>
                    {d.path} · {d.evidence}
                  </li>
                ))}
              </ul>
            </>
          )}
          <ul>
            {reconciliation.blockers.map((b) => (
              <li key={b}>{b}</li>
            ))}
          </ul>
          <div className="actions">
            <button
              disabled={
                busy ||
                reconciliation.assessment !== "understood_and_acceptable"
              }
              onClick={() => void recover("capture_accepted_baseline")}
            >
              Capture Fresh Accepted Baseline
            </button>
            <button
              disabled={
                busy ||
                reconciliation.assessment !== "understood_and_acceptable" ||
                !report?.baseline_receipt ||
                Boolean(
                  recovery &&
                  recovery.backup_id !== reconciliation.old_backup_id,
                )
              }
              onClick={() => void recover("create_current_backup")}
            >
              Create Current Recovery Backup
            </button>
          </div>
        </>
      )}
      {historicalRecovery && (
        <p>
          Historical recovery preserved: {historicalRecovery.backup_id} ·{" "}
          {historicalRecovery.integrity.state} · restore{" "}
          {historicalRecovery.restore_verification?.state ?? "not tested"}. No
          automatic deletion or replacement.
        </p>
      )}
      <h4>Recovery</h4>
      <p>
        Crea una copia privada del prefix de compatibilidad de ZZZ y prueba su
        restauración solo en almacenamiento LXMI. No modifica la instalación ni
        ejecuta Proton o el juego. Se exige contexto inactivo y espacio para dos
        copias más 256 MiB de reserva. Una copia por revisión; limpieza manual.
      </p>
      <div className="actions">
        <button
          disabled={
            busy ||
            recovery !== null ||
            !report?.baseline_receipt ||
            report.active_processes > 0 ||
            report.unresolved_processes > 0
          }
          onClick={() => void recover("create_backup")}
        >
          Create Recovery Backup
        </button>
        <button
          disabled={busy || !recovery}
          onClick={() => void recover("verify_backup")}
        >
          Verify Backup
        </button>
        <button
          disabled={
            busy ||
            recovery?.integrity.state !== "verified" ||
            recovery.restore_path !== null
          }
          onClick={() => void recover("test_restore")}
        >
          Test Restore in LXMI Sandbox
        </button>
        <button
          disabled={busy || !recovery?.restore_verification}
          onClick={() => void inspect(false)}
        >
          Re-evaluate Prefix Preflight
        </button>
      </div>
      {recovery ? (
        <dl>
          <dt>Recovery backup</dt>
          <dd>
            {recovery.backup_id} · {recovery.backup_path}
          </dd>
          <dt>Strategy</dt>
          <dd>{recovery.strategy}</dd>
          <dt>Source size</dt>
          <dd>
            {storageSize(recovery.source_size.apparent_bytes)} ·{" "}
            {recovery.source_size.files} files ·{" "}
            {recovery.source_size.directories} directories ·{" "}
            {recovery.source_size.symlinks} symlinks
          </dd>
          <dt>Backup size</dt>
          <dd>
            {storageSize(recovery.backup_size.apparent_bytes)} logical ·{" "}
            {storageSize(recovery.backup_size.allocated_bytes)} allocated
          </dd>
          <dt>Disk available · before / after</dt>
          <dd>
            {storageSize(recovery.available_bytes_before)} /{" "}
            {storageSize(recovery.available_bytes_after)}
          </dd>
          <dt>Integrity</dt>
          <dd>
            {recovery.integrity.state} · {recovery.integrity.matching_entries}{" "}
            matching entries · SHA-256 {recovery.manifest_sha256}
          </dd>
          <dt>Source consistency</dt>
          <dd>
            {recovery.source_unchanged_during_capture
              ? "Verified unchanged during capture"
              : "Unknown"}
          </dd>
          <dt>Restore test</dt>
          <dd>
            {recovery.restore_verification
              ? `${recovery.restore_verification.state} off-target · ${recovery.restore_verification.matching_files} matching files · ${recovery.restore_path}`
              : "Not tested"}
          </dd>
          <dt>Real-prefix restore</dt>
          <dd>Not tested · no executor</dd>
          <dt>Rollback capability</dt>
          <dd>{recovery.rollback}</dd>
        </dl>
      ) : (
        <p>
          Recovery backup: Not created or not inspected. Capture the baseline
          before creating a backup.
        </p>
      )}
      <RealPrefixEntryPanel
        preflightBlockers={report?.blockers ?? []}
        preflightEligible={
          report?.result === "eligible_for_review" &&
          report.runtime_drift === "same_build" &&
          report.active_processes === 0 &&
          report.unresolved_processes === 0 &&
          recovery?.integrity.state === "verified"
        }
      />
      {report && (
        <div>
          <dl>
            <dt>Identidad prefix · READ-ONLY HOST</dt>
            <dd>
              {report.prefix_identity
                ? `${report.prefix_identity.canonical_path} · UID ${report.prefix_identity.uid} · device ${report.prefix_identity.device} · inode ${report.prefix_identity.inode}`
                : "UNKNOWN"}
            </dd>
            <dt>Build instalado</dt>
            <dd>{report.installed_build ?? "UNKNOWN"}</dd>
            <dt>Build observado en 0.8.3</dt>
            <dd>{report.observed_build}</dd>
            <dt>Runtime drift</dt>
            <dd>{report.runtime_drift}</dd>
            <dt>Actividad · snapshot puntual</dt>
            <dd>
              {report.active_processes} coincidencias ·{" "}
              {report.unresolved_processes} procesos sin resolver
            </dd>
            <dt>Rollback</dt>
            <dd>{report.rollback}</dd>
            <dt>Baseline</dt>
            <dd>
              {report.baseline_receipt
                ? `${report.baseline_receipt.manifest_path} · SHA-256 ${report.baseline_receipt.sha256}`
                : "No capturado"}
            </dd>
            <dt>Entrada al prefix</dt>
            <dd>No autorizada · consentimiento futuro independiente</dd>
          </dl>
          <h4>Limitaciones y evidencia pendiente</h4>
          <ul>
            {[...report.blockers, ...report.rollback_limitations].map(
              (value) => (
                <li key={value}>{value}</li>
              ),
            )}
          </ul>
          <h4>Inventario selectivo</h4>
          <ul>
            {report.baseline.selected_prefix_paths.map((path) => (
              <li key={path.path}>
                {path.path} ·{" "}
                {path.issue ??
                  (path.sha256 ? "hash registrado" : "metadatos registrados")}
              </li>
            ))}
          </ul>
          <h4>Paths protegidos · cobertura selectiva</h4>
          <ul>
            {report.baseline.protected_paths.map((path) => (
              <li key={path.path}>
                {path.path} ·{" "}
                {path.issue ??
                  (path.sha256 ? "hash registrado" : "solo metadatos")}
              </li>
            ))}
          </ul>
          <h4>Mutaciones posibles de Proton/Wine</h4>
          <ul>
            {report.expected_mutations.map((value) => (
              <li key={value}>{value}</li>
            ))}
          </ul>
          <ul>
            {report.baseline.policy.exclusions.map((value) => (
              <li key={value}>{value}</li>
            ))}
          </ul>
        </div>
      )}
    </details>
  );
}
