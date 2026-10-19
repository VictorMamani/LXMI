import { invoke, isTauri } from "@tauri-apps/api/core";
import type { CommandArgs, CommandName, CommandResult } from "./commands";

export type Transport = <K extends CommandName>(
  command: K,
  args: CommandArgs<K>,
) => Promise<CommandResult<K>>;

const tauriTransport: Transport = (command, args) =>
  invoke(command, (args ?? undefined) as Record<string, unknown> | undefined);

/**
 * Outside the Tauri window (plain `npm run dev` in a browser, e.g. on Windows)
 * the app talks to an in-memory demo backend so the interface can be reviewed
 * without Linux, Steam or the games. The demo module is only bundled in dev.
 */
let transport: Promise<Transport> | null = null;
function resolveTransport(): Promise<Transport> {
  if (isTauri()) return Promise.resolve(tauriTransport);
  if (import.meta.env.DEV)
    return import("./mock/mockTransport").then((m) => m.mockTransport);
  return Promise.reject(new Error("LXMI must run inside its desktop window."));
}

export const isDemoMode = !isTauri();

export function call<K extends CommandName>(
  command: K,
  ...args: CommandArgs<K> extends void ? [] : [CommandArgs<K>]
): Promise<CommandResult<K>> {
  transport ??= resolveTransport();
  return transport.then((send) => send(command, args[0] as CommandArgs<K>));
}
