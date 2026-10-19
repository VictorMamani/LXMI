import { Archive } from "lucide-react";
import { useState } from "react";
import { call } from "../../../../api/client";
import type { PrefixPanel, PrefixRecoveryAction } from "../../../../api/types";
import { Button } from "../../../../components/Button";
import { InlineError } from "../../../../components/InlineError";
import { StatusDot } from "../../../../components/StatusDot";
import { useAction } from "../../../../hooks/useAction";
import { useAsync } from "../../../../hooks/useAsync";
import { DetailActions, DiagnosticRow, Facts } from "../DiagnosticRow";

const loadPreflight = () =>
  call("inspect_zzz_prefix_preflight", { captureBaseline: false });

const recoveryActions: { action: PrefixRecoveryAction; label: string }[] = [
  { action: "verify_backup", label: "Verificar copia" },
  { action: "test_restore", label: "Probar restauración" },
  { action: "create_current_backup", label: "Copia del estado actual" },
];

function summarize(panel: PrefixPanel): string {
  const { recovery, preflight } = panel;
  const backup = recovery
    ? `Copia de recuperación ${recovery.integrity.state === "verified" ? "verificada" : recovery.integrity.state}`
    : "Sin copia de recuperación";
  const drift =
    preflight.runtime_drift === "none"
      ? "sin desviaciones"
      : `desviación: ${preflight.runtime_drift}`;
  return `${backup} · ${drift}`;
}

export function PrefixRow() {
  const preflight = useAsync(loadPreflight, "prefix");
  const action = useAction<string>();
  const [panel, setPanel] = useState<PrefixPanel | null>(null);
  const current = panel ?? preflight.data;
  const blockers = current?.preflight.blockers.length ?? 0;

  async function run(name: string, operation: () => Promise<PrefixPanel>) {
    const next = await action.run(name, operation);
    if (next) setPanel(next);
  }

  return (
    <DiagnosticRow
      title="Prefix de ZZZ"
      status={
        current && (
          <StatusDot status={blockers ? "warn" : "ok"} size="sm">
            {blockers ? `${blockers} bloqueos` : "OK"}
          </StatusDot>
        )
      }
      summary={
        preflight.error ??
        (current ? summarize(current) : "Revisando el prefix…")
      }
      action={
        <Button
          icon={Archive}
          disabled={action.busy}
          onClick={() =>
            void run("create", () =>
              call("run_zzz_prefix_recovery", { action: "create_backup" }),
            )
          }
        >
          {action.pending === "create" ? "Creando…" : "Crear copia"}
        </Button>
      }
    >
      <Facts
        items={[
          ["Resultado", current?.preflight.result],
          ["Procesos activos", current?.preflight.active_processes],
          ["Reversión", current?.preflight.rollback],
          ["Copia", current?.recovery?.backup_path],
          [
            "Bloqueos",
            current && (current.preflight.blockers.join(" · ") || "Ninguno"),
          ],
        ]}
      />
      <DetailActions>
        {recoveryActions.map(({ action: recovery, label }) => (
          <Button
            key={recovery}
            variant="outline"
            disabled={action.busy}
            onClick={() =>
              void run(recovery, () =>
                call("run_zzz_prefix_recovery", { action: recovery }),
              )
            }
          >
            {label}
          </Button>
        ))}
        <Button
          variant="outline"
          disabled={action.busy}
          onClick={() =>
            void run("reconcile", () => call("reconcile_zzz_prefix_recovery"))
          }
        >
          Conciliar copias
        </Button>
        <Button
          variant="outline"
          disabled={action.busy}
          onClick={() =>
            void run("steam", () => call("review_zzz_steam_protected_state"))
          }
        >
          Estado protegido de Steam
        </Button>
        <Button
          variant="outline"
          disabled={action.busy}
          onClick={() =>
            void run("baseline", () =>
              call("inspect_zzz_prefix_preflight", { captureBaseline: true }),
            )
          }
        >
          Capturar línea base
        </Button>
      </DetailActions>
      {action.error && <InlineError>{action.error}</InlineError>}
    </DiagnosticRow>
  );
}
