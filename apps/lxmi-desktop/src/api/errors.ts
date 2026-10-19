/** Backend errors arrive as `{ code, detail }`; anything else is shown as text. */
export function errorMessage(reason: unknown): string {
  if (typeof reason === "object" && reason !== null && "detail" in reason)
    return String(reason.detail);
  return reason instanceof Error ? reason.message : String(reason);
}
