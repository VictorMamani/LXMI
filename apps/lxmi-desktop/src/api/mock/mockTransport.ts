/**
 * In-memory stand-in for the Tauri backend, used only by `npm run dev` in a
 * regular browser. It mimics the observable behaviour of each command closely
 * enough to review every screen; it never touches the file system.
 */
import type { Transport } from "../client";
import type { CommandArgs, CommandName, CommandResult } from "../commands";
import type {
  LaunchPhase,
  LaunchStatus,
  ModLibrary,
  ModProfile,
  ObservationSnapshot,
  PrefixPanel,
  RuntimeStatus,
  XxmiPackage,
} from "../types";
import { initialLibraries, initialPackages, latestReleases } from "./fixtures";
import { launchLauncher } from "../../domain/launch";

type Selection = RuntimeStatus["selection"];
type Handlers = {
  [K in CommandName]?: (args: CommandArgs<K>) => CommandResult<K>;
};

const LATENCY_MS = 280;
const libraries = initialLibraries();
const packages: XxmiPackage[] = initialPackages();
const integrationOf: Record<string, string> = {
  "zenless-zone-zero": "zzmi",
  "wuthering-waves": "wwmi",
};
const selections: Record<string, Selection> = {
  "zenless-zone-zero": {
    revision: 3,
    current: {
      profile_name: "Combat",
      mod_count: 4,
      runtime_id: "rt-zzz",
      integration_package_id: "zzmi-2.4.1-7f3a",
      libraries_package_id: "libs-0.9.3-a41d",
    },
    previous: { profile_name: "Screenshots" },
  },
  "wuthering-waves": {
    revision: 1,
    current: {
      profile_name: "Exploration",
      mod_count: 4,
      runtime_id: "rt-wuwa",
      integration_package_id: "wwmi-0.8.2-1c9e",
      libraries_package_id: "libs-0.9.3-a41d",
    },
    previous: null,
  },
};
const appliedProfiles: Record<string, ModProfile | null> = Object.fromEntries(
  Object.entries(selections).map(([gameId, selection]) => [
    gameId,
    structuredClone(
      libraries[gameId].profiles.find(
        (profile) => profile.name === selection.current?.profile_name,
      ) ?? null,
    ),
  ]),
);
const previousProfiles: Record<string, ModProfile | null> = Object.fromEntries(
  Object.entries(selections).map(([gameId, selection]) => [
    gameId,
    structuredClone(
      libraries[gameId].profiles.find(
        (profile) => profile.name === selection.previous?.profile_name,
      ) ?? null,
    ),
  ]),
);
const previousSelections: Record<string, Selection["current"]> =
  Object.fromEntries(
    Object.entries(selections).map(([gameId, selection]) => [
      gameId,
      selection.previous && selection.current
        ? {
            ...selection.current,
            profile_name: selection.previous.profile_name,
            mod_count: previousProfiles[gameId]?.mod_ids.length ?? 0,
            runtime_id: `previous-${gameId}`,
          }
        : null,
    ]),
  );
let launch: { status: LaunchStatus; startedAt: number } | null = null;
let observation: ObservationSnapshot | null = null;

function fail(detail: string): never {
  throw { code: "demo", detail };
}

function library(gameId: string): ModLibrary {
  return libraries[gameId] ?? fail(`Unknown game: ${gameId}`);
}

function packagesFor(kind: string) {
  return packages
    .filter((p) =>
      kind === "xxmi_libraries"
        ? p.kind === "xxmi_libraries"
        : typeof p.kind === "object" && p.kind.game_integration === kind,
    )
    .map((p) => ({
      id: p.id,
      version: p.version,
      authenticity: p.authenticity ?? "not_authenticated",
    }));
}

function mockBase(gameId: string): NonNullable<RuntimeStatus["base_runtime"]> {
  return {
    runtime_id: `rt-${gameId}`,
    importer_root: `~/.local/share/lxmi/runtimes/${gameId}`,
    target_process:
      gameId === "zenless-zone-zero"
        ? "ZenlessZoneZero.exe"
        : "Client-Win64-Shipping.exe",
    file_count: 214,
  };
}

function runtimeStatus(
  gameId: string,
  importerId: string | null,
  librariesId: string | null,
): RuntimeStatus {
  const selection = selections[gameId];
  const current = selection.current;
  if (
    current &&
    importerId &&
    librariesId &&
    (current.integration_package_id !== importerId ||
      current.libraries_package_id !== librariesId)
  )
    fail(
      "The selected profile uses a different base runtime. Roll it back or prepare it again for these packages.",
    );
  const ready = selection.current !== null;
  return {
    game_id: gameId,
    installed: true,
    importer_packages: packagesFor(integrationOf[gameId]),
    libraries_packages: packagesFor("xxmi_libraries"),
    base_runtime: importerId && librariesId ? mockBase(gameId) : null,
    selection: structuredClone(selection),
    applied_profile: structuredClone(appliedProfiles[gameId]),
    launch_with_mods_available: ready,
    launch_blockers: ready ? [] : ["Apply a profile before playing with mods."],
    launch_experimental: gameId === "wuthering-waves",
  };
}

const LAUNCH_TIMELINE: LaunchPhase[] = [
  "preparing",
  "opening_launcher",
  "arming",
  "waiting_for_play",
  "checking_stability",
  "running",
];
const LAUNCH_STEP_MS = 2200;

function currentLaunch(): LaunchStatus | null {
  if (!launch || launch.status.finished) return launch?.status ?? null;
  const step = Math.floor((Date.now() - launch.startedAt) / LAUNCH_STEP_MS);
  const phase = LAUNCH_TIMELINE[Math.min(step, LAUNCH_TIMELINE.length - 1)];
  launch.status = {
    ...launch.status,
    phase,
    finished: phase === "running",
    mods_loaded: step >= 4,
    message:
      phase === "waiting_for_play"
        ? `Press 'Play' in ${launchLauncher(launch.status.game_id)} to continue.`
        : phase === "running"
          ? "The game is running. The mod module is loaded; LXMI does not verify the effect of each mod."
          : "Preparing to load mods…",
  };
  return launch.status;
}

function prefixPanel(): PrefixPanel {
  return {
    preflight: {
      result: "eligible",
      runtime_drift: "none",
      active_processes: 0,
      blockers: [],
      rollback: "available",
    },
    recovery: {
      backup_id: "bk-20261014",
      backup_path: "~/.local/share/lxmi/prefix-backups/bk-20261014",
      integrity: { state: "verified", mismatches: [] },
      real_prefix_restore: "not_performed",
    },
    current_recovery_matches_source: true,
  };
}

const protonRuntimes = [
  {
    display_name: "Proton 9.0-4",
    version: "9.0-4",
    proton_script: "/mnt/games/SteamLibrary/steamapps/common/Proton 9.0/proton",
  },
];

const handlers: Handlers = {
  get_system_info: () => ({
    os: "linux",
    architecture: "x86_64",
    homeDirectory: "/home/demo",
  }),
  scan_steam: () => ({
    status: "detected",
    installations: [
      {
        rootPath: "/home/demo/.local/share/Steam",
        libraries: [{ path: "/mnt/games/SteamLibrary", isDefault: false }],
      },
    ],
    issues: [],
    games: [
      {
        id: "zenless-zone-zero",
        name: "Zenless Zone Zero",
        steamAppId: 3405340,
      },
      { id: "wuthering-waves", name: "Wuthering Waves", steamAppId: 3513350 },
    ],
    compatibilityTools: [
      { displayName: "Proton 9.0-4", version: "9.0-4", kind: "proton" },
    ],
    runtimePlans: ["zenless-zone-zero", "wuthering-waves"].map((gameId) => ({
      gameId,
      gameName: gameId,
      installation: {
        state: "detected" as const,
        steamAppId: 0,
        steamLibrary: "/mnt/games/SteamLibrary",
        installPath: `/mnt/games/SteamLibrary/steamapps/common/${gameId}`,
      },
      selection: {
        state: "selected" as const,
        candidate: {
          internalId: "proton_9",
          displayName: "Proton 9.0-4",
          path: "",
          version: "9.0-4",
        },
      },
      readiness: "ready" as const,
    })),
  }),

  list_mod_library: ({ gameId }) => structuredClone(library(gameId)),
  import_local_mod: ({ gameId, name, source }) => {
    if (!source.trim()) fail("Enter the path to a directory or ZIP archive.");
    const entry = {
      id: `demo-${Date.now()}`,
      name,
      files: [{ relative_path: "mod.ini", size: 4096, sha256: "0".repeat(64) }],
    };
    const lib = library(gameId);
    lib.mods.push(entry);
    lib.revision += 1;
    return entry;
  },
  save_mod_profile: ({ gameId, name, modIds, expectedRevision }) => {
    const lib = library(gameId);
    if (lib.revision !== expectedRevision)
      fail("The library changed while you were editing. Reload it.");
    const existing = lib.profiles.find((p) => p.name === name);
    if (existing) existing.mod_ids = [...modIds];
    else lib.profiles.push({ name, mod_ids: [...modIds] });
    lib.revision += 1;
    return structuredClone(lib);
  },

  inspect_game_mod_runtime: ({ gameId, importerId, librariesId }) =>
    runtimeStatus(gameId, importerId, librariesId),
  prepare_game_mod_runtime: ({ gameId }) => mockBase(gameId),
  review_mod_profile_activation: (args) => {
    if (
      args.profileRevision !== library(args.gameId).revision ||
      args.selectionRevision !== selections[args.gameId].revision
    )
      fail("The selection or profile changed. Refresh and review again.");
    const profile = library(args.gameId).profiles.find(
      (p) => p.name === args.profileName,
    );
    if (!profile) fail(`Profile '${args.profileName}' does not exist.`);
    return {
      plan_id: `plan-${Date.now()}`,
      game_id: args.gameId,
      profile_name: profile.name,
      profile_revision: args.profileRevision,
      selection_revision: args.selectionRevision,
      runtime_id: `rt-${args.gameId}`,
      importer_root: "",
      mod_count: profile.mod_ids.length,
      file_count: profile.mod_ids.length * 3,
      total_bytes: 0,
      required_launch_arguments: [],
    };
  },
  activate_mod_profile: (args) => {
    const selection = selections[args.gameId];
    if (
      args.profileRevision !== library(args.gameId).revision ||
      args.selectionRevision !== selection.revision
    )
      fail("The selection or profile changed. Refresh and review again.");
    const profile = library(args.gameId).profiles.find(
      (p) => p.name === args.profileName,
    );
    previousProfiles[args.gameId] = appliedProfiles[args.gameId];
    previousSelections[args.gameId] = selection.current;
    appliedProfiles[args.gameId] = structuredClone(profile ?? null);
    selections[args.gameId] = {
      revision: selection.revision + 1,
      previous: selection.current
        ? { profile_name: selection.current.profile_name }
        : null,
      current: {
        profile_name: args.profileName,
        mod_count: profile?.mod_ids.length ?? 0,
        runtime_id: [
          "profile",
          args.gameId,
          args.importerId,
          args.librariesId,
          args.profileName,
          ...(profile?.mod_ids ?? []),
        ].join(":"),
        integration_package_id: args.importerId ?? "",
        libraries_package_id: args.librariesId ?? "",
      },
    };
    return null;
  },
  rollback_mod_profile_activation: ({ gameId, selectionRevision }) => {
    const selection = selections[gameId];
    if (selectionRevision !== selection.revision)
      fail("The selection changed. Refresh before rolling back.");
    const restored = previousProfiles[gameId];
    const restoredSelection = previousSelections[gameId];
    previousSelections[gameId] = selection.current;
    previousProfiles[gameId] = appliedProfiles[gameId];
    appliedProfiles[gameId] = restored;
    selections[gameId] = {
      revision: selection.revision + 1,
      current: restoredSelection,
      previous: selection.current
        ? { profile_name: selection.current.profile_name }
        : null,
    };
    return null;
  },

  launch_game_without_mods: () => null,
  start_game_with_mods: ({ gameId }) => {
    launch = {
      startedAt: Date.now(),
      status: {
        game_id: gameId,
        profile_name: selections[gameId].current?.profile_name ?? null,
        phase: "preparing",
        message: "Copying the profile…",
        finished: false,
        mods_loaded: false,
        launch_dir: null,
        helper_log: [],
      },
    };
    return launch.status;
  },
  game_with_mods_status: () => currentLaunch(),
  cancel_game_with_mods: () => {
    if (!launch || launch.status.finished) return false;
    launch.status = {
      ...launch.status,
      phase: "cancelled",
      finished: true,
      message: "Launch cancelled.",
    };
    return true;
  },

  xxmi_status: () => ({
    storage_root: "~/.local/share/lxmi",
    packages: structuredClone(packages),
    issues: [],
  }),
  xxmi_check_official_releases: () => latestReleases,
  xxmi_download_official_package: ({ packageKind, tag }) => {
    const version = tag.replace(/^v/, "");
    const entry: XxmiPackage = {
      id: `${packageKind}-${version}-${Date.now().toString(16).slice(-4)}`,
      kind:
        packageKind === "xxmi_libraries"
          ? "xxmi_libraries"
          : { game_integration: packageKind },
      imported_unix_seconds: Date.now() / 1000,
      version: { raw: version },
      files: [],
      authenticity: "official_release_verified",
    };
    packages.push(entry);
    return entry;
  },
  import_xxmi_directory: () =>
    fail("Local directories cannot be read in demo mode."),
  review_zzmi_assembly: (args) => ({ demo: true, ...args, files: 214 }),
  inspect_zzmi_launch_topology: () => ({
    demo: true,
    target: "ZenlessZoneZero.exe",
  }),

  inspect_zzz_prefix_preflight: () => prefixPanel(),
  run_zzz_prefix_recovery: () => prefixPanel(),
  reconcile_zzz_prefix_recovery: () => prefixPanel(),
  review_zzz_steam_protected_state: () => prefixPanel(),

  inspect_zzz_launch_readiness: () => ({
    game_name: "Zenless Zone Zero",
    decision: "ready_for_controlled_review",
    blockers: [],
    unknowns: [],
    requirements: [
      { id: "steam_installation", state: "satisfied", evidence: "appmanifest" },
      { id: "proton_context", state: "satisfied", evidence: "config.vdf" },
    ],
    selected_game_runtime: "Proton 9.0-4",
  }),
  start_zzz_runtime_observation: () => {
    observation = {
      session_id: 1,
      status: "waiting_for_process",
      active: true,
      elapsed_millis: 0,
      process: null,
      observed_runtime: { state: "unknown", tool: null },
      prefix: { state: "unknown", observed_path: null },
      warnings: [],
    };
    return observation;
  },
  get_zzz_runtime_observation: () => observation,
  stop_zzz_runtime_observation: () => {
    if (observation)
      observation = { ...observation, active: false, status: "stopped" };
    return observation;
  },

  inspect_runtime_bridge: () => ({ runtimes: protonRuntimes, demo: true }),
  inspect_loader_lab: () => ({ runtimes: protonRuntimes, demo: true }),
  inspect_zzz_harmless_prefix_entry_history: () => ({
    current: null,
    demo: true,
  }),
  inspect_linux_environment_qualification: () => ({
    demo: true,
    qualification: "pending",
  }),
};

export const mockTransport: Transport = async <K extends CommandName>(
  command: K,
  args: CommandArgs<K>,
) => {
  await new Promise((resolve) => setTimeout(resolve, LATENCY_MS));
  const handler = handlers[command] as
    ((a: CommandArgs<K>) => CommandResult<K>) | undefined;
  if (!handler)
    fail(
      `'${command}' is unavailable in demo mode: it requires Linux and the desktop app.`,
    );
  return handler(args);
};
