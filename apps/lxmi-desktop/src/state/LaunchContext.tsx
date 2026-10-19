import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { call } from "../api/client";
import { errorMessage } from "../api/errors";
import type { LaunchStatus } from "../api/types";
import { launchStepIndex } from "../domain/launch";
import { readStored, writeStored } from "../lib/storage";

const POLL_MS = 1000;
const CONSENT_KEY = "lxmi.modLaunchAcknowledged.v1";
const lastLaunchKey = (gameId: string) => `lxmi.lastModLaunch.${gameId}`;

export function readLastLaunch(gameId: string): number | null {
  const value = Number(readStored(lastLaunchKey(gameId)));
  return Number.isFinite(value) && value > 0 ? value : null;
}

type LaunchSession = {
  status: LaunchStatus | null;
  /** Last step reached, so a failure can point at where it stopped. */
  lastReached: number;
  /** The user closed the panel of this status (`game:phase`). */
  dismissedKey: string | null;
  error: string | null;
  starting: boolean;
  start: (gameId: string) => Promise<void>;
  cancel: () => Promise<void>;
  dismiss: () => void;
  clearError: () => void;
};

const LaunchContext = createContext<LaunchSession | null>(null);

const keyOf = (status: LaunchStatus | null) =>
  status ? `${status.game_id}:${status.phase}` : null;

/**
 * «Jugar con mods» session. The backend runs one launch at a time, so it
 * lives above the screens and survives navigating to configuration and back.
 */
export function LaunchProvider({ children }: { children: ReactNode }) {
  const [status, setStatus] = useState<LaunchStatus | null>(null);
  const [lastReached, setLastReached] = useState(0);
  const [dismissedKey, setDismissedKey] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [starting, setStarting] = useState(false);
  const [pollToken, setPollToken] = useState(0);

  useEffect(() => {
    let cancelled = false;
    let timer: number | undefined;
    const poll = async () => {
      try {
        const next = await call("game_with_mods_status");
        if (cancelled) return;
        setStatus(next);
        const index = next ? launchStepIndex(next.phase) : -1;
        if (index >= 0) setLastReached(index);
        if (next && !next.finished) timer = window.setTimeout(poll, POLL_MS);
      } catch (reason) {
        if (!cancelled) setError(errorMessage(reason));
      }
    };
    void poll();
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [pollToken]);

  const start = useCallback(async (gameId: string) => {
    setError(null);
    setStarting(true);
    try {
      setStatus(await call("start_game_with_mods", { gameId }));
      setLastReached(0);
      writeStored(lastLaunchKey(gameId), String(Date.now()));
      setPollToken((token) => token + 1);
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setStarting(false);
    }
  }, []);

  const cancel = useCallback(async () => {
    try {
      await call("cancel_game_with_mods");
      setPollToken((token) => token + 1);
    } catch (reason) {
      setError(errorMessage(reason));
    }
  }, []);

  const value = useMemo<LaunchSession>(
    () => ({
      status,
      lastReached,
      dismissedKey,
      error,
      starting,
      start,
      cancel,
      dismiss: () => setDismissedKey(keyOf(status)),
      clearError: () => setError(null),
    }),
    [status, lastReached, dismissedKey, error, starting, start, cancel],
  );
  return (
    <LaunchContext.Provider value={value}>{children}</LaunchContext.Provider>
  );
}

/** The launch session seen from one game. */
export function useModLaunch(gameId: string) {
  const session = useContext(LaunchContext);
  if (!session) throw new Error("useModLaunch requiere LaunchProvider");
  const { status } = session;
  const own = status?.game_id === gameId ? status : null;
  return {
    ...session,
    /** Launch of this game, if any. */
    status: own,
    /** The progress panel should be visible. */
    visible: own !== null && keyOf(own) !== session.dismissedKey,
    /** Another game is currently launching. */
    busyElsewhere:
      status !== null && !status.finished && status.game_id !== gameId,
    start: () => session.start(gameId),
    consentGiven: () => readStored(CONSENT_KEY) === "yes",
    giveConsent: () => writeStored(CONSENT_KEY, "yes"),
  };
}
