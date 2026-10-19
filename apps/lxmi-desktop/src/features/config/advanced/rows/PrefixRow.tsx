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
  { action: "verify_backup", label: "Verify backup" },
  { action: "test_restore", label: "Test restore" },
  { action: "create_current_backup", label: "Back up current state" },
];

function summarize(panel: PrefixPanel): string {
  const { recovery, preflight } = panel;
  const backup = recovery
    ? `Recovery backup ${recovery.integrity.state === "verified" ? "verified" : recovery.integrity.state}`
    : "No recovery backup";
  const drift =
    preflight.runtime_drift === "none"
      ? "no drift"
      : `drift: ${preflight.runtime_drift}`;
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
      title="ZZZ prefix"
      status={
        current && (
          <StatusDot status={blockers ? "warn" : "ok"} size="sm">
            {blockers ? `${blockers} blockers` : "OK"}
          </StatusDot>
        )
      }
      summary={
        preflight.error ??
        (current ? summarize(current) : "Checking the prefix…")
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
          {action.pending === "create" ? "Creating…" : "Create backup"}
        </Button>
      }
    >
      <Facts
        items={[
          ["Result", current?.preflight.result],
          ["Active processes", current?.preflight.active_processes],
          ["Rollback", current?.preflight.rollback],
          ["Backup", current?.recovery?.backup_path],
          [
            "Blockers",
            current && (current.preflight.blockers.join(" · ") || "None"),
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
          Reconcile backups
        </Button>
        <Button
          variant="outline"
          disabled={action.busy}
          onClick={() =>
            void run("steam", () => call("review_zzz_steam_protected_state"))
          }
        >
          Protected Steam state
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
          Capture baseline
        </Button>
      </DetailActions>
      {action.error && <InlineError>{action.error}</InlineError>}
    </DiagnosticRow>
  );
}
