import { Radar } from "lucide-react";
import { Button } from "../../../../components/Button";
import { StatusDot } from "../../../../components/StatusDot";
import { plural } from "../../../../domain/format";
import { useSystem } from "../../../../state/SystemContext";
import { DiagnosticRow, Facts } from "../DiagnosticRow";

const steamStatus: Record<string, string> = {
  detected: "OK",
  not_installed: "Steam not found",
  configuration_missing: "Not configured",
  invalid_configuration: "Invalid configuration",
  permission_denied: "Permission denied",
  home_directory_unavailable: "No HOME",
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
    scan && plural(scan.installations.length, "installation"),
    scan &&
      plural(
        scan.runtimePlans.filter((p) => p.installation.state === "detected")
          .length,
        "supported game",
        "supported games",
      ),
    proton?.displayName,
  ].filter(Boolean);

  return (
    <DiagnosticRow
      title="System and Steam"
      status={
        steam.loading ? (
          <StatusDot status="idle" size="sm">
            Scanning
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
      summary={steam.error ?? (summary.join(" · ") || "Checking…")}
      action={
        <Button
          icon={Radar}
          disabled={steam.loading}
          onClick={() => void steam.reload().catch(() => {})}
        >
          Scan
        </Button>
      }
    >
      <Facts
        items={[
          ["Home directory", system.data?.homeDirectory],
          ["Steam", scan?.installations.map((i) => i.rootPath).join(", ")],
          [
            "Libraries",
            scan?.installations
              .flatMap((i) => i.libraries.map((l) => l.path))
              .join(", "),
          ],
          ["Detected games", scan?.games.map((g) => g.name).join(", ")],
          [
            "Compatibility tools",
            scan?.compatibilityTools.map((t) => t.displayName).join(", "),
          ],
          [
            "Issues",
            scan &&
              (scan.issues.length
                ? scan.issues.map((i) => i.detail ?? i.code).join(" · ")
                : "None"),
          ],
        ]}
      />
    </DiagnosticRow>
  );
}
