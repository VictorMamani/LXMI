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
  return `${kind} ${pkg.version?.raw ?? "sin versión"}`;
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
        label:
          kind === "assembly"
            ? "Plan de ensamblado ZZMI"
            : "Topología de lanzamiento",
        value,
      });
  }

  return (
    <>
      <DiagnosticRow
        title="Paquetes XXMI"
        status={
          data && (
            <StatusDot status={issues ? "warn" : "ok"} size="sm">
              {issues ? plural(issues, "incidencia") : "OK"}
            </StatusDot>
          )
        }
        summary={
          status.error ??
          (data
            ? `${plural(verified, "paquete verificado", "paquetes verificados")} en el almacén · ${issues ? plural(issues, "incidencia") : "sin incidencias"}`
            : "Consultando el almacén…")
        }
        action={
          <Button icon={FolderInput} onClick={() => setImporting(true)}>
            Importar carpeta
          </Button>
        }
      >
        <Facts
          items={[
            ["Almacén", data?.storage_root],
            ...(data?.packages.map((pkg): [string, string] => [
              packageLabel(pkg),
              `${pkg.id.slice(0, 12)} · ${pkg.authenticity === VERIFIED ? "firma oficial verificada" : "sin autenticar"}`,
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
            Revisar ensamblado ZZMI
          </Button>
          <Button
            variant="outline"
            disabled={action.busy || !sources.zzmiId}
            onClick={() => void inspect("topology")}
          >
            Topología de lanzamiento
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
            toast(`${packageLabel(pkg)} importado`, "success");
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
      eyebrow="Paquetes XXMI"
      title="Importar carpeta"
      onClose={onClose}
      width={520}
    >
      <form
        onSubmit={(event) => void submit(event)}
        style={{ display: "contents" }}
      >
        <TextField
          label="Carpeta del paquete"
          placeholder="/home/usuario/Descargas/ZZMI"
          value={path}
          onChange={(event) => setPath(event.target.value)}
          hint="Se verifica y se guarda en el almacén de LXMI; no se instala en el juego."
          autoFocus
        />
        {action.error && <InlineError>{action.error}</InlineError>}
        <DetailActions>
          <Button onClick={onClose}>Cancelar</Button>
          <Button
            type="submit"
            variant="primary"
            icon={FolderInput}
            disabled={action.busy || !path.trim()}
          >
            {action.busy ? "Verificando…" : "Importar"}
          </Button>
        </DetailActions>
      </form>
    </Dialog>
  );
}
