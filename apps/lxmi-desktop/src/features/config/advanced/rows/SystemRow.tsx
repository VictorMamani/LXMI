import { Radar } from "lucide-react";
import { Button } from "../../../../components/Button";
import { StatusDot } from "../../../../components/StatusDot";
import { plural } from "../../../../domain/format";
import { useSystem } from "../../../../state/SystemContext";
import { DiagnosticRow, Facts } from "../DiagnosticRow";

const steamStatus: Record<string, string> = {
  detected: "OK",
  not_installed: "Sin Steam",
  configuration_missing: "Sin configurar",
  invalid_configuration: "Configuración inválida",
  permission_denied: "Sin permisos",
  home_directory_unavailable: "Sin HOME",
  internal_error: "Error",
};

export function SystemRow() {
  const { system, steam } = useSystem();
  const scan = steam.data;
  const proton = scan?.compatibilityTools.find(
    (tool) => tool.kind === "proton",
  );

  const summary = [
    system.data &&
      `${system.data.os === "linux" ? "Linux" : system.data.os} ${system.data.architecture}`,
    scan && plural(scan.installations.length, "instalación", "instalaciones"),
    scan &&
      plural(
        scan.runtimePlans.filter((p) => p.installation.state === "detected")
          .length,
        "juego compatible",
        "juegos compatibles",
      ),
    proton?.displayName,
  ].filter(Boolean);

  return (
    <DiagnosticRow
      title="Sistema y Steam"
      status={
        steam.loading ? (
          <StatusDot status="idle" size="sm">
            Escaneando
          </StatusDot>
        ) : scan ? (
          <StatusDot
            status={scan.status === "detected" ? "ok" : "warn"}
            size="sm"
          >
            {steamStatus[scan.status] ?? scan.status}
          </StatusDot>
        ) : (
          <StatusDot status="error" size="sm">
            Error
          </StatusDot>
        )
      }
      summary={steam.error ?? (summary.join(" · ") || "Consultando…")}
      action={
        <Button
          icon={Radar}
          disabled={steam.loading}
          onClick={() => void steam.reload().catch(() => {})}
        >
          Escanear
        </Button>
      }
    >
      <Facts
        items={[
          ["Directorio personal", system.data?.homeDirectory],
          ["Steam", scan?.installations.map((i) => i.rootPath).join(", ")],
          [
            "Bibliotecas",
            scan?.installations
              .flatMap((i) => i.libraries.map((l) => l.path))
              .join(", "),
          ],
          ["Juegos detectados", scan?.games.map((g) => g.name).join(", ")],
          [
            "Herramientas de compatibilidad",
            scan?.compatibilityTools.map((t) => t.displayName).join(", "),
          ],
          [
            "Incidencias",
            scan &&
              (scan.issues.length
                ? scan.issues.map((i) => i.detail ?? i.code).join(" · ")
                : "Ninguna"),
          ],
        ]}
      />
    </DiagnosticRow>
  );
}
