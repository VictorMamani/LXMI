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
        title="Runtime observation"
        status={
          <StatusDot status={active ? "accent" : "idle"} size="sm">
            {active ? "Active" : "Inactive"}
          </StatusDot>
        }
        summary={
          active
            ? `Observing · ${snapshot?.process ? `${snapshot.process.executable_name} (PID ${snapshot.process.pid})` : "waiting for the game process"}`
            : "Read-only · requires opening the game manually"
        }
        action={
          active ? (
            <Button
              icon={Square}
              disabled={action.busy}
              onClick={() => void stop()}
            >
              Stop
            </Button>
          ) : (
            <Button
              icon={Activity}
              disabled={action.busy}
              onClick={() => setConfirming(true)}
            >
              Start
            </Button>
          )
        }
      >
        <Facts
          items={[
            ["Status", snapshot?.status],
            ["Process", snapshot?.process?.executable_name],
            [
              "Observed runtime",
              snapshot?.observed_runtime.tool?.display_name ??
                snapshot?.observed_runtime.state,
            ],
            [
              "Prefix",
              snapshot?.prefix.observed_path ?? snapshot?.prefix.state,
            ],
            ["Warnings", snapshot && (snapshot.warnings.join(" · ") || "None")],
          ]}
        />
        {action.error && <InlineError>{action.error}</InlineError>}
      </DiagnosticRow>

      {confirming && (
        <Dialog
          eyebrow="Observation"
          title="Open the game manually"
          onClose={() => setConfirming(false)}
          actions={
            <>
              <Button onClick={() => setConfirming(false)}>Cancel</Button>
              <Button
                variant="primary"
                icon={Activity}
                onClick={() => void start()}
              >
                Start observing
              </Button>
            </>
          }
        >
          <p>
            LXMI does not start the game: open it from Steam yourself.
            Meanwhile, LXMI reads your user process information to identify the
            runtime and prefix. It does not write anything.
          </p>
        </Dialog>
      )}
    </>
  );
}
