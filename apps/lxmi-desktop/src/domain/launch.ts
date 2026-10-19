import type { LaunchPhase, LaunchStatus } from "../api/types";

/** The six visible steps of «Play with mods», mapped from backend phases. */
export const launchSteps: readonly { label: string; phases: LaunchPhase[] }[] =
  [
    { label: "Profile copy", phases: ["preparing"] },
    {
      label: "HoYoPlay through Steam",
      phases: ["opening_launcher", "waiting_for_launcher"],
    },
    { label: "Hook ready", phases: ["arming"] },
    { label: "Press 'Play' in HoYoPlay", phases: ["waiting_for_play"] },
    { label: "Mods loaded", phases: ["checking_stability"] },
    { label: "In game", phases: ["running"] },
  ];

export type StepState = "done" | "current" | "pending" | "failed";

export function launchStepIndex(phase: LaunchPhase): number {
  return launchSteps.findIndex((step) => step.phases.includes(phase));
}

/**
 * State of every step for a status. A failed or cancelled launch keeps the
 * last reached step marked as failed so the user sees where it stopped.
 */
export function launchStepStates(
  status: LaunchStatus,
  lastReached: number,
): StepState[] {
  const index = launchStepIndex(status.phase);
  const interrupted = status.phase === "failed" || status.phase === "cancelled";
  const current = interrupted ? lastReached : index;
  return launchSteps.map((_, i) => {
    if (interrupted)
      return i < current ? "done" : i === current ? "failed" : "pending";
    if (i < current || (i === current && status.phase === "running"))
      return "done";
    return i === current ? "current" : "pending";
  });
}

export function isLaunchActive(
  status: LaunchStatus | null,
): status is LaunchStatus {
  return status !== null && !status.finished;
}
