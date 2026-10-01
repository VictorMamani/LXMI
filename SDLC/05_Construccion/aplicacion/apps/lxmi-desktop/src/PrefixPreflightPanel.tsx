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

export default function PrefixPreflightPanel() {
  const [report, setReport] = useState<Preflight | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  async function inspect(captureBaseline: boolean) {
    setBusy(true);
    setError(null);
    try {
      setReport(
        await invoke<Preflight>("inspect_zzz_prefix_preflight", {
          captureBaseline,
        }),
      );
    } catch (cause) {
      const detail = cause as { code?: string; detail?: string };
      setError(
        `${detail.code ?? "preflight_failed"}: ${detail.detail ?? String(cause)}`,
      );
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
          ? "Inspeccionando metadatos…"
          : report
            ? `Preflight: ${report.result}`
            : "Sin inspección. Consulta el prefix para revisar evidencia y limitaciones."}
      </p>
      {error && <p role="alert">{error}</p>}
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
