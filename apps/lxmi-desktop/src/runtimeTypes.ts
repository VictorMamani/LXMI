export type ModLibrary = {
  game_id: string;
  mods: {
    id: string;
    name: string;
    files: { relative_path: string; size: number; sha256: string }[];
  }[];
  profiles: { name: string; mod_ids: string[] }[];
  revision: number;
  issues: { code: string; detail: string }[];
};
export type RuntimePackage = {
  id: string;
  version: { raw: string } | null;
  authenticity: string;
};
export type RuntimeSources = { importer: string; libraries: string };
export type RuntimeStatus = {
  game_id: string;
  installed: boolean;
  importer_packages: RuntimePackage[];
  libraries_packages: RuntimePackage[];
  base_runtime: {
    runtime_id: string;
    importer_root: string;
    target_process: string;
    file_count: number;
  } | null;
  selection: {
    revision: number;
    current: {
      profile_name: string;
      mod_count: number;
      runtime_id: string;
      integration_package_id: string;
      libraries_package_id: string;
    } | null;
    previous: { profile_name: string } | null;
  };
  launch_with_mods_available: boolean;
  launch_blockers: string[];
  launch_experimental: boolean;
};
export type LaunchStatus = {
  game_id: string;
  profile_name: string | null;
  phase:
    | "preparing"
    | "opening_launcher"
    | "waiting_for_launcher"
    | "arming"
    | "waiting_for_play"
    | "checking_stability"
    | "running"
    | "failed"
    | "cancelled";
  message: string;
  finished: boolean;
  mods_loaded: boolean;
  launch_dir: string | null;
  helper_log: string[];
};
export type ActivationPlan = {
  plan_id: string;
  game_id: string;
  profile_name: string;
  profile_revision: number;
  selection_revision: number;
  runtime_id: string;
  importer_root: string;
  mod_count: number;
  file_count: number;
  total_bytes: number;
  required_launch_arguments: string[];
};
export function errorMessage(reason: unknown): string {
  if (typeof reason === "object" && reason !== null && "detail" in reason)
    return String(reason.detail);
  return reason instanceof Error ? reason.message : String(reason);
}
