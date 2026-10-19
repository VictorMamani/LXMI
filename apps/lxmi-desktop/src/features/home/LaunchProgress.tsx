import { Check, LoaderCircle, Minus, RotateCcw, X } from "lucide-react";
import type { LaunchStatus } from "../../api/types";
import { Button } from "../../components/Button";
import { pad2 } from "../../domain/format";
import {
  launchSteps,
  launchStepStates,
  launchLauncher,
  type StepState,
} from "../../domain/launch";
import styles from "./LaunchProgress.module.css";

const stepIcon: Record<StepState, typeof Check> = {
  done: Check,
  current: LoaderCircle,
  pending: Minus,
  failed: X,
};

function title(status: LaunchStatus): string {
  if (status.phase === "running") return "IN GAME";
  if (status.phase === "failed") return "LOAD FAILED";
  if (status.phase === "cancelled") return "LOAD CANCELLED";
  return "LOADING MODS";
}

/** The six phases of `game_with_mods_status` with a segmented bar. */
export function LaunchProgress({
  status,
  lastReached,
  showQualityHint,
  onCancel,
  onRetry,
  onDismiss,
}: {
  status: LaunchStatus;
  lastReached: number;
  showQualityHint: boolean;
  onCancel: () => void;
  onRetry: () => void;
  onDismiss: () => void;
}) {
  const states = launchStepStates(status, lastReached);
  const reached = states.filter((s) => s !== "pending").length;
  const interrupted = status.phase === "failed" || status.phase === "cancelled";
  const cancellable = !status.finished && status.phase !== "checking_stability";

  let note = `Do not close ${launchLauncher(status.game_id)}`;
  if (interrupted) note = status.message;
  else if (status.phase === "running" || status.phase === "waiting_for_play")
    note = status.message;
  if (status.phase === "waiting_for_play" && showQualityHint)
    note +=
      " Modded characters: set Character Quality to High and disable High-Precision Animation.";

  return (
    <section
      className={styles.panel}
      aria-label="Loading progress"
      role="status"
      aria-live="polite"
    >
      <header className={styles.header}>
        <h2
          className={styles.title}
          data-state={interrupted ? "failed" : undefined}
        >
          {title(status)}
        </h2>
        <span className={styles.step}>
          {pad2(reached)} / {pad2(launchSteps.length)}
        </span>
      </header>

      <div className={styles.bar} aria-hidden>
        {states.map((state, index) => (
          <span key={index} className={styles.segment} data-state={state} />
        ))}
      </div>

      <ol className={styles.phases}>
        {launchSteps.map((step, index) => {
          const Icon = stepIcon[states[index]];
          return (
            <li
              key={step.label}
              className={styles.phase}
              data-state={states[index]}
            >
              <Icon
                size={14}
                strokeWidth={2}
                aria-hidden
                className={styles.phaseIcon}
              />
              <span>{step.label}</span>
            </li>
          );
        })}
      </ol>

      <footer className={styles.footer}>
        <p className={styles.note}>{note}</p>
        {cancellable && (
          <Button icon={X} onClick={onCancel}>
            Cancel
          </Button>
        )}
        {interrupted && (
          <Button icon={RotateCcw} onClick={onRetry}>
            Retry
          </Button>
        )}
        {status.finished && (
          <Button onClick={onDismiss} aria-label="Close progress">
            Close
          </Button>
        )}
      </footer>
    </section>
  );
}
