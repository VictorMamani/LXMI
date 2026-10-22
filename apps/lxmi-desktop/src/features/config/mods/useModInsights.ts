import { useCallback, useEffect, useMemo, useState } from "react";
import { call } from "../../../api/client";
import type { ModDetails, ModManifest } from "../../../api/types";
import { pendingPreviews } from "../../../domain/mods";
import { useAsync } from "../../../hooks/useAsync";
import { readStored, writeStored } from "../../../lib/storage";
import { forgetPreviewImages } from "./usePreviewImage";

const PREVIEWS_KEY = "lxmi.mods.previews";

export type PreviewProgress = {
  done: number;
  total: number;
  failed: number;
  /** Mod being rendered right now; null once the queue is done. */
  current: string | null;
};

/**
 * Detected kind and character of every mod, plus the optional preview
 * generator. Rendering takes seconds per model, so it only runs while the
 * user keeps previews on, one mod at a time, and stops when turned off.
 */
export function useModInsights(
  gameId: string,
  mods: readonly ModManifest[] | null,
) {
  const modKey = mods?.map((mod) => mod.id).join(",") ?? null;
  const details = useAsync(
    () =>
      modKey === null
        ? Promise.resolve([])
        : call("list_mod_details", { gameId }),
    `${gameId}|${modKey}`,
  );
  // Results of this session's renders, layered over the loaded details.
  const [rendered, setRendered] = useState<{
    gameId: string;
    byId: ReadonlyMap<string, ModDetails>;
  }>({ gameId, byId: new Map() });
  const [enabled, setEnabledState] = useState(
    () => readStored(PREVIEWS_KEY) === "on",
  );
  const [progress, setProgress] = useState<PreviewProgress | null>(null);

  const byId = useMemo(() => {
    const merged = new Map(
      (details.data ?? []).map((entry) => [entry.mod_id, entry]),
    );
    if (rendered.gameId === gameId)
      for (const [id, entry] of rendered.byId) merged.set(id, entry);
    return merged;
  }, [details.data, rendered, gameId]);

  const loaded = details.data;
  useEffect(() => {
    if (!enabled || !loaded || !mods) return;
    const queue = pendingPreviews(
      mods,
      new Map(loaded.map((entry) => [entry.mod_id, entry])),
    );
    if (queue.length === 0) return;
    let cancelled = false;
    async function run(ids: string[]) {
      const state: PreviewProgress = {
        done: 0,
        total: ids.length,
        failed: 0,
        current: null,
      };
      setProgress({ ...state });
      for (const modId of ids) {
        state.current = modId;
        setProgress({ ...state });
        try {
          const entry = await call("generate_mod_preview", { gameId, modId });
          if (cancelled) return;
          setRendered((current) => ({
            gameId,
            byId: new Map(current.gameId === gameId ? current.byId : []).set(
              modId,
              entry,
            ),
          }));
        } catch {
          // The mod stays without a preview; the next refresh retries it.
          state.failed++;
        }
        if (cancelled) return;
        state.done++;
        state.current = null;
        setProgress({ ...state });
      }
    }
    void run(queue);
    return () => {
      cancelled = true;
      setProgress(null);
    };
  }, [enabled, loaded, mods, gameId]);

  const setEnabled = useCallback((value: boolean) => {
    writeStored(PREVIEWS_KEY, value ? "on" : "off");
    setEnabledState(value);
  }, []);

  const { reload } = details;
  /** Turns previews off and deletes every cached image of this game. */
  const clearPreviews = useCallback(async () => {
    setEnabled(false);
    await call("clear_mod_previews", { gameId });
    forgetPreviewImages(gameId);
    setRendered({ gameId, byId: new Map() });
    await reload();
  }, [gameId, reload, setEnabled]);

  const running = progress !== null && progress.done < progress.total;
  return {
    details: byId,
    loading: details.loading,
    error: details.error,
    enabled,
    setEnabled,
    progress: enabled ? progress : null,
    running: enabled && running,
    clearPreviews,
  };
}
