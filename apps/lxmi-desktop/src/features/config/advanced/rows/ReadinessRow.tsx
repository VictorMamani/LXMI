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
      title="Preparación de lanzamiento"
      status={
        data && (
          <StatusDot status={blockers ? "warn" : "ok"} size="sm">
            {blockers ? plural(blockers, "bloqueo") : "Sin bloqueos"}
          </StatusDot>
        )
      }
      summary={
        readiness.error ??
        (data
          ? `${pending ? plural(pending, "requisito pendiente", "requisitos pendientes") : "Requisitos cumplidos"} · ${data.selected_game_runtime ? `contexto Steam/${data.selected_game_runtime}` : "runtime de Steam sin seleccionar"}`
          : "Evaluando…")
      }
      action={
        <Button
          icon={ScanLine}
          disabled={readiness.loading}
          onClick={() => void readiness.reload().catch(() => {})}
        >
          {readiness.loading ? "Evaluando…" : "Evaluar"}
        </Button>
      }
    >
      <Facts
        items={[
          ["Decisión", data?.decision],
          ...(data?.requirements.map((r): [string, string] => [
            r.id.replaceAll("_", " "),
            `${r.state} · ${r.evidence}`,
          ]) ?? []),
          ["Bloqueos", data && (data.blockers.join(" · ") || "Ninguno")],
          ["Desconocidos", data && (data.unknowns.join(" · ") || "Ninguno")],
        ]}
      />
    </DiagnosticRow>
  );
}
