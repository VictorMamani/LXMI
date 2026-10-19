import { FolderInput } from "lucide-react";
import { useState, type FormEvent } from "react";
import { call } from "../../../../api/client";
import type { XxmiPackage } from "../../../../api/types";
import { Button } from "../../../../components/Button";
import { Dialog } from "../../../../components/Dialog";
import { InlineError } from "../../../../components/InlineError";
import { StatusDot } from "../../../../components/StatusDot";
import { TextField } from "../../../../components/TextField";
import { plural } from "../../../../domain/format";
import { VERIFIED } from "../../../../domain/runtime";
import { useAction } from "../../../../hooks/useAction";
import { useAsync } from "../../../../hooks/useAsync";
import { useToast } from "../../../../state/ToastContext";
import { DetailActions, DiagnosticRow, Facts } from "../DiagnosticRow";
import { JsonView } from "../JsonView";
import { useZzzSources } from "../useZzzSources";

const loadStatus = () => call("xxmi_status");

function packageLabel(pkg: XxmiPackage): string {
  const kind =
    pkg.kind === "xxmi_libraries"
      ? "XXMI Libraries"
      : pkg.kind.game_integration.toUpperCase();
  return `${kind} ${pkg.version?.raw ?? "no version"}`;
}

export function PackagesRow() {
  const status = useAsync(loadStatus, "xxmi");
  const sources = useZzzSources();
  const action = useAction<"assembly" | "topology">();
  const toast = useToast();
  const [importing, setImporting] = useState(false);
  const [evidence, setEvidence] = useState<{
    label: string;
    value: unknown;
  } | null>(null);

  const data = status.data;
  const verified =
    data?.packages.filter((p) => p.authenticity === VERIFIED).length ?? 0;
  const issues = data?.issues.length ?? 0;

  async function inspect(kind: "assembly" | "topology") {
    const args = sources;
    const value = await action.run(kind, () =>
      kind === "assembly"
        ? call("review_zzmi_assembly", args)
        : call("inspect_zzmi_launch_topology", args),
    );
    if (value !== undefined)
      setEvidence({
        label: kind === "assembly" ? "ZZMI assembly plan" : "Launch topology",
        value,
      });
  }

  return (
    <>
      <DiagnosticRow
        title="XXMI packages"
        status={
          data && (
            <StatusDot status={issues ? "warn" : "ok"} size="sm">
              {issues ? plural(issues, "issue") : "OK"}
            </StatusDot>
          )
        }
        summary={
          status.error ??
          (data
            ? `${plural(verified, "verified package", "verified packages")} in storage · ${issues ? plural(issues, "issue") : "no issues"}`
            : "Checking storage…")
        }
        action={
          <Button icon={FolderInput} onClick={() => setImporting(true)}>
            Import directory
          </Button>
        }
      >
        <Facts
          items={[
            ["Storage", data?.storage_root],
            ...(data?.packages.map((pkg): [string, string] => [
              packageLabel(pkg),
              `${pkg.id.slice(0, 12)} · ${pkg.authenticity === VERIFIED ? "official signature verified" : "unauthenticated"}`,
            ]) ?? []),
            ...(data?.issues.map((issue): [string, string] => [
              issue.code,
              issue.detail,
            ]) ?? []),
          ]}
        />
        <DetailActions>
          <Button
            variant="outline"
            disabled={action.busy || !sources.zzmiId}
            onClick={() => void inspect("assembly")}
          >
            Review ZZMI assembly
          </Button>
          <Button
            variant="outline"
            disabled={action.busy || !sources.zzmiId}
            onClick={() => void inspect("topology")}
          >
            Launch topology
          </Button>
        </DetailActions>
        {action.error && <InlineError>{action.error}</InlineError>}
        {evidence && <JsonView label={evidence.label} value={evidence.value} />}
      </DiagnosticRow>
      {importing && (
        <ImportPackageDialog
          onClose={() => setImporting(false)}
          onImported={(pkg) => {
            setImporting(false);
            void status.reload().catch(() => {});
            toast(`${packageLabel(pkg)} imported`, "success");
          }}
        />
      )}
    </>
  );
}

function ImportPackageDialog({
  onClose,
  onImported,
}: {
  onClose: () => void;
  onImported: (pkg: XxmiPackage) => void;
}) {
  const [path, setPath] = useState("");
  const action = useAction();

  async function submit(event: FormEvent) {
    event.preventDefault();
    const pkg = await action.run("import", () =>
      call("import_xxmi_directory", { path: path.trim() }),
    );
    if (pkg) onImported(pkg);
  }

  return (
    <Dialog
      eyebrow="XXMI packages"
      title="Import directory"
      onClose={onClose}
      width={520}
    >
      <form
        onSubmit={(event) => void submit(event)}
        style={{ display: "contents" }}
      >
        <TextField
          label="Package directory"
          placeholder="/home/user/Downloads/ZZMI"
          value={path}
          onChange={(event) => setPath(event.target.value)}
          hint="Verified and saved in LXMI storage; not installed in the game."
          autoFocus
        />
        {action.error && <InlineError>{action.error}</InlineError>}
        <DetailActions>
          <Button onClick={onClose}>Cancel</Button>
          <Button
            type="submit"
            variant="primary"
            icon={FolderInput}
            disabled={action.busy || !path.trim()}
          >
            {action.busy ? "Verifying…" : "Import"}
          </Button>
        </DetailActions>
      </form>
    </Dialog>
  );
}
