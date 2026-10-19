import type { LaunchPhase, LaunchStatus } from "../api/types";

export function launchLauncher(gameId: string): string {
  return gameId === "wuthering-waves" ? "Wuthering Waves" : "HoYoPlay";
}

/** WWMI consent is separate from the previously acknowledged ZZZ hook flow. */
export function launchConsentKey(gameId: string): string {
  return gameId === "zenless-zone-zero"
    ? "lxmi.modLaunchAcknowledged.v1"
    : `lxmi.modLaunchAcknowledged.${gameId}.directInject.v1`;
}

export type LaunchStep = { label: string; phases: LaunchPhase[] };

/** ZZZ (hook): Steam opens HoYoPlay, the loader arms, the player presses Play. */
const hookSteps: readonly LaunchStep[] = [
  { label: "Profile copy", phases: ["preparing"] },
  {
    label: "Launcher through Steam",
    phases: ["opening_launcher", "waiting_for_launcher"],
  },
  { label: "Loader ready", phases: ["arming"] },
  { label: "Press 'Play' in the launcher", phases: ["waiting_for_play"] },
  { label: "Mod module loaded", phases: ["checking_stability"] },
  { label: "In game", phases: ["running"] },
];

/**
 * Wuthering Waves (Direct Inject): Steam starts the game directly, so the
 * loader is staged first and attaches as soon as the game appears.
 */
const directInjectSteps: readonly LaunchStep[] = [
  { label: "Profile copy", phases: ["preparing"] },
  { label: "Loader staged", phases: ["arming"] },
  {
    label: "Game through Steam",
    phases: ["opening_launcher", "waiting_for_launcher"],
  },
  { label: "Injecting WWMI", phases: ["waiting_for_play"] },
  { label: "Mod module loaded", phases: ["checking_stability"] },
  { label: "In game", phases: ["running"] },
];

/** The six visible steps of «Play with mods», mapped from backend phases. */
export function launchStepsFor(gameId: string): readonly LaunchStep[] {
  return gameId === "wuthering-waves" ? directInjectSteps : hookSteps;
}

export type StepState = "done" | "current" | "pending" | "failed";

export function launchStepIndex(gameId: string, phase: LaunchPhase): number {
  return launchStepsFor(gameId).findIndex((step) =>
    step.phases.includes(phase),
  );
}

/**
 * State of every step for a status. A failed or cancelled launch keeps the
 * last reached step marked as failed so the user sees where it stopped.
 */
export function launchStepStates(
  status: LaunchStatus,
  lastReached: number,
): StepState[] {
  const index = launchStepIndex(status.game_id, status.phase);
  const interrupted = status.phase === "failed" || status.phase === "cancelled";
  const current = interrupted ? lastReached : index;
  return launchStepsFor(status.game_id).map((_, i) => {
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
