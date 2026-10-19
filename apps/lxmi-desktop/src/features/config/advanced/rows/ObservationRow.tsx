import { Activity, Square } from "lucide-react";
import { useEffect, useState } from "react";
import { call } from "../../../../api/client";
import type { ObservationSnapshot } from "../../../../api/types";
import { Button } from "../../../../components/Button";
import { Dialog } from "../../../../components/Dialog";
import { InlineError } from "../../../../components/InlineError";
import { StatusDot } from "../../../../components/StatusDot";
import { useAction } from "../../../../hooks/useAction";
import { DiagnosticRow, Facts } from "../DiagnosticRow";

const POLL_MS = 1500;

/** Read-only observation of a ZZZ session the user starts from Steam. */
export function ObservationRow() {
  const [snapshot, setSnapshot] = useState<ObservationSnapshot | null>(null);
  const [confirming, setConfirming] = useState(false);
  const action = useAction<"start" | "stop">();
  const active = snapshot?.active ?? false;

  useEffect(() => {
    let cancelled = false;
    let timer: number | undefined;
    const poll = async () => {
      try {
        const next = await call("get_zzz_runtime_observation");
        if (cancelled) return;
        setSnapshot(next);
        if (next?.active) timer = window.setTimeout(poll, POLL_MS);
      } catch {
        // The row keeps its last snapshot; the next action reports errors.
      }
    };
    void poll();
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [active]);

  async function start() {
    setConfirming(false);
    const next = await action.run("start", () =>
      call("start_zzz_runtime_observation", {
        userConfirmedManualLaunch: true,
      }),
    );
    if (next) setSnapshot(next);
  }

  async function stop() {
    const next = await action.run("stop", () =>
      call("stop_zzz_runtime_observation"),
    );
    if (next !== undefined) setSnapshot(next);
  }

  return (
    <>
      <DiagnosticRow
        title="Observación en ejecución"
        status={
          <StatusDot status={active ? "accent" : "idle"} size="sm">
            {active ? "Activa" : "Inactiva"}
          </StatusDot>
        }
        summary={
          active
            ? `Observando · ${snapshot?.process ? `${snapshot.process.executable_name} (PID ${snapshot.process.pid})` : "esperando el proceso del juego"}`
            : "Solo lectura · requiere abrir el juego manualmente"
        }
        action={
          active ? (
            <Button
              icon={Square}
              disabled={action.busy}
              onClick={() => void stop()}
            >
              Detener
            </Button>
          ) : (
            <Button
              icon={Activity}
              disabled={action.busy}
              onClick={() => setConfirming(true)}
            >
              Iniciar
            </Button>
          )
        }
      >
        <Facts
          items={[
            ["Estado", snapshot?.status],
            ["Proceso", snapshot?.process?.executable_name],
            [
              "Runtime observado",
              snapshot?.observed_runtime.tool?.display_name ??
                snapshot?.observed_runtime.state,
            ],
            [
              "Prefix",
              snapshot?.prefix.observed_path ?? snapshot?.prefix.state,
            ],
            [
              "Avisos",
              snapshot && (snapshot.warnings.join(" · ") || "Ninguno"),
            ],
          ]}
        />
        {action.error && <InlineError>{action.error}</InlineError>}
      </DiagnosticRow>

      {confirming && (
        <Dialog
          eyebrow="Observación"
          title="Abrir el juego manualmente"
          onClose={() => setConfirming(false)}
          actions={
            <>
              <Button onClick={() => setConfirming(false)}>Cancelar</Button>
              <Button
                variant="primary"
                icon={Activity}
                onClick={() => void start()}
              >
                Empezar a observar
              </Button>
            </>
          }
        >
          <p>
            LXMI no inicia el juego: ábrelo tú desde Steam. Mientras tanto, LXMI
            lee la información de los procesos de tu usuario para identificar el
            runtime y el prefix. No escribe nada.
          </p>
        </Dialog>
      )}
    </>
  );
}
