import { useCallback, useEffect, useRef, useState } from "react";
import { errorMessage } from "../api/errors";

export type AsyncState<T> = {
  data: T | null;
  error: string | null;
  loading: boolean;
  /** Runs the loader again; resolves with the new value (or rejects). */
  reload: () => Promise<T>;
};

/**
 * Loads data when `key` changes and exposes it with loading/error state.
 * Results of superseded loads are ignored, so switching games never shows
 * data of the previous one.
 */
export function useAsync<T>(
  load: () => Promise<T>,
  key: unknown,
): AsyncState<T> {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const generation = useRef(0);
  const loader = useRef(load);
  useEffect(() => {
    loader.current = load;
  });

  const reload = useCallback(async () => {
    const id = ++generation.current;
    setLoading(true);
    setError(null);
    try {
      const value = await loader.current();
      if (id === generation.current) setData(value);
      return value;
    } catch (reason) {
      if (id === generation.current) setError(errorMessage(reason));
      throw reason;
    } finally {
      if (id === generation.current) setLoading(false);
    }
  }, []);

  useEffect(() => {
    const counter = generation;
    setData(null);
    reload().catch(() => {});
    // Invalidate the in-flight load when the key changes or on unmount.
    return () => {
      counter.current++;
    };
  }, [key, reload]);

  return { data, error, loading, reload };
}
