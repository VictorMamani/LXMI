import type {
  ActivationPlan,
  ContextLabMode,
  LaunchReadiness,
  LaunchStatus,
  LoaderLabMode,
  ModLibrary,
  ModManifest,
  ObservationSnapshot,
  OfficialRelease,
  PrefixPanel,
  PrefixRecoveryAction,
  ProtonRuntimeOption,
  RuntimeStatus,
  SteamScan,
  SystemInfo,
  XxmiPackage,
  XxmiStatus,
} from "./types";

type RuntimeArgs = {
  gameId: string;
  importerId: string | null;
  librariesId: string | null;
};
type ActivationArgs = RuntimeArgs & {
  profileName: string;
  profileRevision: number;
  selectionRevision: number;
};
type ZzmiArgs = { zzmiId: string | null; librariesId: string | null };
type LabArgs<Mode> = {
  protonScript: string;
  mode: Mode;
  sideEffectsAcknowledged: boolean;
};

/**
 * Every Tauri command the interface calls, keyed by its registered name in
 * `src-tauri/src/lib.rs`. Argument keys are camelCase, as Tauri expects.
 * Results of laboratory commands are shown raw, so they stay `unknown`.
 */
export type Commands = {
  // System and Steam
  get_system_info: { args: void; result: SystemInfo };
  scan_steam: { args: void; result: SteamScan };

  // Mod library
  list_mod_library: { args: { gameId: string }; result: ModLibrary };
  import_local_mod: {
    args: { gameId: string; name: string; source: string };
    result: ModManifest;
  };
  save_mod_profile: {
    args: {
      gameId: string;
      name: string;
      modIds: string[];
      expectedRevision: number;
    };
    result: ModLibrary;
  };

  // Runtime and applied profile
  inspect_game_mod_runtime: { args: RuntimeArgs; result: RuntimeStatus };
  prepare_game_mod_runtime: {
    args: RuntimeArgs;
    result: NonNullable<RuntimeStatus["base_runtime"]>;
  };
  review_mod_profile_activation: {
    args: ActivationArgs;
    result: ActivationPlan;
  };
  activate_mod_profile: {
    args: ActivationArgs & { planId: string };
    result: unknown;
  };
  rollback_mod_profile_activation: {
    args: { gameId: string; selectionRevision: number };
    result: unknown;
  };

  // Launch
  launch_game_without_mods: { args: { gameId: string }; result: unknown };
  start_game_with_mods: { args: { gameId: string }; result: LaunchStatus };
  game_with_mods_status: { args: void; result: LaunchStatus | null };
  cancel_game_with_mods: { args: void; result: boolean };

  // XXMI packages
  xxmi_status: { args: void; result: XxmiStatus };
  xxmi_check_official_releases: { args: void; result: OfficialRelease[] };
  xxmi_download_official_package: {
    args: { packageKind: string; tag: string };
    result: XxmiPackage;
  };
  import_xxmi_directory: { args: { path: string }; result: XxmiPackage };
  review_zzmi_assembly: { args: ZzmiArgs; result: unknown };
  inspect_zzmi_launch_topology: { args: ZzmiArgs; result: unknown };

  // ZZZ prefix
  inspect_zzz_prefix_preflight: {
    args: { captureBaseline: boolean };
    result: PrefixPanel;
  };
  run_zzz_prefix_recovery: {
    args: { action: PrefixRecoveryAction };
    result: PrefixPanel;
  };
  reconcile_zzz_prefix_recovery: { args: void; result: PrefixPanel };
  review_zzz_steam_protected_state: { args: void; result: PrefixPanel };

  // Preparation and observation
  inspect_zzz_launch_readiness: { args: ZzmiArgs; result: LaunchReadiness };
  start_zzz_runtime_observation: {
    args: { userConfirmedManualLaunch: boolean };
    result: ObservationSnapshot;
  };
  get_zzz_runtime_observation: {
    args: void;
    result: ObservationSnapshot | null;
  };
  stop_zzz_runtime_observation: {
    args: void;
    result: ObservationSnapshot | null;
  };

  // Laboratories (experimental)
  inspect_runtime_bridge: {
    args: ZzmiArgs;
    result: { runtimes: ProtonRuntimeOption[] } & Record<string, unknown>;
  };
  run_runtime_bridge_test: {
    args: { zzmiId: string; librariesId: string; protonScript: string };
    result: unknown;
  };
  run_context_lab_test: { args: LabArgs<ContextLabMode>; result: unknown };
  run_direct_context_test: { args: LabArgs<ContextLabMode>; result: unknown };
  run_context_reentry_test: { args: LabArgs<ContextLabMode>; result: unknown };
  inspect_loader_lab: {
    args: void;
    result: { runtimes: ProtonRuntimeOption[] } & Record<string, unknown>;
  };
  prepare_loader_lab: { args: void; result: unknown };
  run_loader_lab_experiment: { args: LabArgs<LoaderLabMode>; result: unknown };
  inspect_linux_environment_qualification: { args: void; result: unknown };
  qualify_zzz_prefix_executor: { args: void; result: unknown };
  inspect_zzz_harmless_prefix_entry_history: {
    args: void;
    result: {
      current: { experiment: { experiment_id: string } } | null;
    } & Record<string, unknown>;
  };
  create_zzz_harmless_prefix_entry_experiment: {
    args: void;
    result: { experiment: { experiment_id: string } } & Record<string, unknown>;
  };
  prepare_zzz_harmless_prefix_entry: {
    args: { experimentId: string };
    result: {
      experiment_id: string;
      review_token: string;
      consent_text: string;
    } & Record<string, unknown>;
  };
  run_zzz_harmless_prefix_entry: {
    args: {
      experimentId: string;
      reviewToken: string;
      consentAccepted: boolean;
    };
    result: unknown;
  };
};

export type CommandName = keyof Commands;
export type CommandArgs<K extends CommandName> = Commands[K]["args"];
export type CommandResult<K extends CommandName> = Commands[K]["result"];
