/**
 * Data shapes returned by the Tauri backend (`src-tauri/src/*_commands.rs`).
 * Only the fields the interface reads are declared; the backend may send more.
 */

export type BackendGameId = "zenless-zone-zero" | "wuthering-waves";

export type PackageFile = {
  relative_path: string;
  size: number;
  sha256: string;
};

export type ModManifest = {
  id: string;
  name: string;
  files: PackageFile[];
};

export type ModProfile = { name: string; mod_ids: string[] };

export type ModLibrary = {
  game_id: string;
  mods: ModManifest[];
  profiles: ModProfile[];
  revision: number;
  issues: { code: string; detail: string }[];
};

export type Authenticity =
  | "not_authenticated"
  | "missing_signature"
  | "signature_invalid"
  | "official_release_verified";

export type RuntimePackage = {
  id: string;
  version: { raw: string } | null;
  authenticity: Authenticity | string;
};

/** Package ids used to assemble the mod runtime of a game. */
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
  applied_profile: ModProfile | null;
  launch_with_mods_available: boolean;
  launch_blockers: string[];
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

export type LaunchPhase =
  | "preparing"
  | "opening_launcher"
  | "waiting_for_launcher"
  | "arming"
  | "waiting_for_play"
  | "checking_stability"
  | "running"
  | "failed"
  | "cancelled";

export type LaunchStatus = {
  game_id: string;
  profile_name: string | null;
  phase: LaunchPhase;
  message: string;
  finished: boolean;
  mods_loaded: boolean;
  launch_dir: string | null;
  helper_log: string[];
};

export type PackageKind = "zzmi" | "wwmi" | "xxmi_libraries";

export type OfficialRelease = {
  package_kind: PackageKind | string;
  tag: string;
  version: string;
  published_at: string;
  release_url: string;
};

export type XxmiPackage = {
  id: string;
  imported_unix_seconds: number;
  kind: "xxmi_libraries" | { game_integration: string };
  version: { raw: string } | null;
  files: PackageFile[];
  authenticity?: Authenticity;
};

export type Diagnostic = { code: string; path: string | null; detail: string };

export type XxmiStatus = {
  storage_root: string;
  packages: XxmiPackage[];
  issues: Diagnostic[];
};

export type SystemInfo = {
  os: string;
  architecture: string;
  homeDirectory: string | null;
};

export type SteamStatus =
  | "not_installed"
  | "detected"
  | "configuration_missing"
  | "invalid_configuration"
  | "permission_denied"
  | "home_directory_unavailable"
  | "internal_error";

export type RuntimeCandidate = {
  internalId: string | null;
  displayName: string;
  path: string;
  version: string | null;
};

export type RuntimePlan = {
  gameId: string;
  gameName: string;
  installation:
    | { state: "not_found" }
    | {
        state: "detected";
        steamAppId: number;
        steamLibrary: string;
        installPath: string;
      }
    | { state: "unknown" };
  selection:
    | { state: "unknown"; reason: string }
    | { state: "selected"; candidate: RuntimeCandidate }
    | { state: "not_required" };
  readiness:
    "ready" | "incomplete" | "needs_initialization" | "unknown" | "blocked";
};

export type SteamScan = {
  status: SteamStatus;
  installations: {
    rootPath: string;
    libraries: { path: string; isDefault: boolean }[];
  }[];
  issues: { code: string; path: string | null; detail: string | null }[];
  games: { id: string; name: string; steamAppId: number }[];
  compatibilityTools: {
    displayName: string;
    version: string | null;
    kind: string;
  }[];
  runtimePlans: RuntimePlan[];
};

export type LaunchReadiness = {
  game_name: string;
  decision: string;
  blockers: string[];
  unknowns: string[];
  requirements: { id: string; state: string; evidence: string }[];
  selected_game_runtime: string | null;
};

export type PrefixRecoveryAction =
  | "create_backup"
  | "verify_backup"
  | "test_restore"
  | "capture_accepted_baseline"
  | "create_current_backup";

export type PrefixRecovery = {
  backup_id: string;
  backup_path: string;
  integrity: { state: string; mismatches: string[] };
  real_prefix_restore: string;
};

export type PrefixPanel = {
  preflight: {
    result: string;
    runtime_drift: string;
    active_processes: number;
    blockers: string[];
    rollback: string;
  };
  recovery: PrefixRecovery | null;
  current_recovery_matches_source: boolean;
};

export type ObservationSnapshot = {
  session_id: number;
  status: string;
  active: boolean;
  elapsed_millis: number;
  process: { pid: number; executable_name: string } | null;
  observed_runtime: { state: string; tool: { display_name: string } | null };
  prefix: { state: string; observed_path: string | null };
  warnings: string[];
};

export type ProtonRuntimeOption = {
  display_name: string;
  version: string | null;
  proton_script: string;
};

export type ContextLabMode = "same_prefix" | "different_prefixes";

export type LoaderLabMode =
  | "baseline"
  | "positive"
  | "hook"
  | "missing_target"
  | "missing_dll"
  | "wrong_nonce";
