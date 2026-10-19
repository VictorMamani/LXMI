import { ScanLine } from "lucide-react";
import { call } from "../../../../api/client";
import { Button } from "../../../../components/Button";
import { StatusDot } from "../../../../components/StatusDot";
import { plural } from "../../../../domain/format";
import { useAsync } from "../../../../hooks/useAsync";
import { DiagnosticRow, Facts } from "../DiagnosticRow";
import { useZzzSources } from "../useZzzSources";

export function ReadinessRow() {
  const sources = useZzzSources();
  const readiness = useAsync(
    () => call("inspect_zzz_launch_readiness", sources),
    `${sources.zzmiId}:${sources.librariesId}`,
  );
  const data = readiness.data;
  const blockers = data?.blockers.length ?? 0;
  const pending =
    data?.requirements.filter((r) => r.state !== "satisfied").length ?? 0;

  return (
    <DiagnosticRow
      title="Launch readiness"
      status={
        data && (
          <StatusDot status={blockers ? "warn" : "ok"} size="sm">
            {blockers ? plural(blockers, "bloqueo") : "No blockers"}
          </StatusDot>
        )
      }
      summary={
        readiness.error ??
        (data
          ? `${pending ? plural(pending, "pending requirement", "pending requirements") : "Requirements met"} · ${data.selected_game_runtime ? `Steam context/${data.selected_game_runtime}` : "no Steam runtime selected"}`
          : "Evaluating…")
      }
      action={
        <Button
          icon={ScanLine}
          disabled={readiness.loading}
          onClick={() => void readiness.reload().catch(() => {})}
        >
          {readiness.loading ? "Evaluating…" : "Evaluate"}
        </Button>
      }
    >
      <Facts
        items={[
          ["Decision", data?.decision],
          ...(data?.requirements.map((r): [string, string] => [
            r.id.replaceAll("_", " "),
            `${r.state} · ${r.evidence}`,
          ]) ?? []),
          ["Blockers", data && (data.blockers.join(" · ") || "None")],
          ["Unknowns", data && (data.unknowns.join(" · ") || "None")],
        ]}
      />
    </DiagnosticRow>
  );
}
