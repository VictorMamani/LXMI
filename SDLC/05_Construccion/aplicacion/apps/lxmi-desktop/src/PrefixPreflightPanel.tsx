import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

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
type Panel = { preflight: Preflight; recovery: Recovery | null };
type Action = "create_backup" | "verify_backup" | "test_restore";
function storageSize(bytes: number) {
  return `${bytes.toLocaleString()} bytes · ${(bytes / 1024 / 1024).toFixed(1)} MiB`;
}

export default function PrefixPreflightPanel() {
  const [report, setReport] = useState<Preflight | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [recovery, setRecovery] = useState<Recovery | null>(null);
  const [operation, setOperation] = useState("");
  function accept(panel: Panel) {
    setReport(panel.preflight);
    setRecovery(panel.recovery);
  }
  function fail(cause: unknown) {
    const detail = cause as { code?: string; detail?: string };
    setError(
      `${detail.code ?? "recovery_failed"}: ${detail.detail ?? String(cause)}`,
    );
    setReport(null);
    setRecovery(null);
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
        almacenamiento LXMI; no contiene un backup recuperable. Este panel no
        ejecuta Proton, helpers ni el juego.
      </p>
      <div className="actions">
        <button disabled={busy} onClick={() => void inspect(false)}>
          Consultar prefix preflight
        </button>
        <button disabled={busy} onClick={() => void inspect(true)}>
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
