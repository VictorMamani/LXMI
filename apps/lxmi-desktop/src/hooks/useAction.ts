import { useCallback, useState } from "react";
import { errorMessage } from "../api/errors";

/**
 * Tracks one user-triggered operation at a time: which one is running and the
 * error of the last attempt. `run` never rejects; it resolves to `undefined`
 * when the operation failed.
 */
export function useAction<Name extends string = string>() {
  const [pending, setPending] = useState<Name | null>(null);
  const [error, setError] = useState<string | null>(null);

  const run = useCallback(
    async <T>(name: Name, operation: () => Promise<T>) => {
      setPending(name);
      setError(null);
      try {
        return await operation();
      } catch (reason) {
        setError(errorMessage(reason));
        return undefined;
      } finally {
        setPending(null);
      }
    },
    [],
  );

  return {
    pending,
    busy: pending !== null,
    error,
    clearError: () => setError(null),
    run,
  };
}
