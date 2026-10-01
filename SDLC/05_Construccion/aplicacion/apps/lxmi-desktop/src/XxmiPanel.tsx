import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

type Package = {
  id: string;
  imported_unix_seconds: number;
  kind: "xxmi_libraries" | { game_integration: string };
  version: { raw: string; evidence: string } | null;
  files: { relative_path: string; size: number; sha256: string }[];
  authenticity?:
    | "not_authenticated"
    | "missing_signature"
    | "signature_invalid"
    | "official_release_verified";
  upstream?: {
    repository: string;
    release_id: number;
    tag: string;
    commit: string;
    asset_name: string;
    download_sha256: string;
    signature_status: string;
    component_signatures_verified: boolean;
    metadata_retrieved_at: string;
  } | null;
};
type OfficialRelease = {
  package_kind: "zzmi" | "xxmi_libraries";
  repository: string;
  release_id: number;
  tag: string;
  commit: string;
  release_url: string;
  version: string;
  published_at: string;
  metadata_retrieved_at: string;
  signature_base64: string | null;
  source_trust: "official" | "untrusted";
  assets: {
    id: number;
    name: string;
    download_url: string;
    size: number;
    content_type: string | null;
    sha256: string | null;
  }[];
};
type Diagnostic = { code: string; path: string | null; detail: string };
type Status = {
  storage_root: string;
  packages: Package[];
  observations: {
    path: string;
    integration_kind: "wwmi" | "zzmi" | "gimi" | "unknown" | null;
    integration: string;
    libraries: string;
    evidence: string[];
    issues: Diagnostic[];
  }[];
  issues: Diagnostic[];
};
type Plan = {
  assessment: {
    game_id: string;
    integration: "wwmi" | "zzmi" | "gimi" | "unknown";
    distribution: "steam" | "hoyoplay" | "manual" | "unknown";
    game_support: string;
    steam_support: "verified" | "unverified" | "unsupported";
    linux_proton_support: "verified" | "unverified" | "unsupported";
    platform_compatibility_verified: boolean;
    planning_possible: boolean;
    requirements: { name: string; state: string; evidence: string }[];
  };
  managed_target: string;
  game_executable_candidate: string | null;
  files: {
    source: string;
    target: string;
    action: string;
    backup_required: boolean;
  }[];
  warnings: string[];
  configuration_changes: string[];
  deployment_mapping: {
    source_package_id: string;
    source_relative_path: string;
    target_relative_path: string;
    operation: string;
    target_root_basis: string;
    evidence: string;
  }[];
  dry_run: {
    configured_target_root: string | null;
    comparison_root_candidate: string | null;
    comparison_root_evidence: string;
    root_is_authoritative: boolean;
    files: {
      source_package_id: string;
      source_path: string;
      target_relative_path: string;
      comparison_path: string | null;
      expected_sha256: string;
      existing_sha256: string | null;
      status: string;
      evidence: string;
    }[];
    safety: string;
    apply_allowed: boolean;
    writes_performed: boolean;
  };
};
type ManagedRuntime = {
  runtime_id: string;
  app_root: string;
  importer_root: string;
  manifest_path: string;
  loader_library_path: string;
  zzmi_version: string | null;
  libraries_version: string | null;
  target_process: string;
  loader_process_identity: string;
  file_count: number;
  newly_created: boolean;
  installed_into_game: boolean;
};
type AssemblyPlan = {
  runtime_id: string;
  game_id: string;
  integration: string;
  app_root: string;
  importer_root: string;
  import_path_from_app_root: string;
  zzmi_package_id: string;
  libraries_package_id: string;
  zzmi_version: string | null;
  libraries_version: string | null;
  target_process: string;
  loader_library_source: string;
  loader_library_sha256: string;
  config: {
    source_relative_path: string;
    source_sha256: string;
    derived_sha256: string;
    target_process: string;
    loader_process_identity: string;
    loader_identity_change: string | null;
  };
  files: {
    source_component: string;
    source_relative_path: string;
    destination_relative_path: string;
    size: number;
    sha256: string;
    derived: boolean;
    reason: string;
  }[];
  directories_to_create: string[];
  evidence: string[];
  warnings: string[];
};
type WindowsPathMapping = {
  status: string;
  windows_path: string | null;
};
type LaunchTopology = {
  game_name: string;
  distribution: string;
  steam_app_id: number | null;
  game_installation: string;
  game_executable: string | null;
  compatdata: string;
  prefix_path: string | null;
  selected_proton: string;
  available_proton_candidates: {
    display_name: string;
    version: string | null;
  }[];
  app_root: string | null;
  importer_folder: string;
  importer_root: string | null;
  loader_library_path: string | null;
  loader_library_windows_mapping: WindowsPathMapping | null;
  game_executable_windows_mapping: WindowsPathMapping | null;
  importer_root_windows_mapping: WindowsPathMapping | null;
  dosdevices: {
    state: string;
    mappings: {
      drive_letter: string;
      raw_target: string | null;
      state: string;
    }[];
  } | null;
  loader_strategy: string;
  loader_process_identity: string | null;
  same_prefix_requirement: string;
  module: string;
  readiness: string;
  capabilities: { name: string; state: string; evidence: string }[];
  unknowns: string[];
  blockers: string[];
  execution_enabled: boolean;
  external_files_modified: boolean;
};
type BridgeRuntimeOption = {
  display_name: string;
  version: string | null;
  source: string;
  proton_script: string;
};
type RuntimeBridgePanel = {
  options: {
    helper:
      | { state: "missing" }
      | {
          state: "available";
          expected_sha256: string;
          actual_sha256: string;
          integrity_matches: boolean;
          helper_version: string;
          protocol_version: number;
          build_target: string;
        }
      | { state: "invalid"; detail: string };
    managed_runtime_available: boolean;
    managed_runtime_path: string | null;
    compatdata_path: string;
    prefix_path: string;
    prefix_mode: string;
    warning: string;
  };
  runtimes: BridgeRuntimeOption[];
  game_runtime_selection: string;
};
type BridgeTestResult = {
  explicit_runtime: { display_name: string; version: string | null };
  prefix_mode: string;
  compatdata_path: string;
  helper_version: string;
  helper_protocol_version: number;
  handshake: string;
  runtime_path_linux: string;
  runtime_path_windows: string;
  path_visibility: string;
  runtime_manifest_sha256: string;
  runtime_manifest_hash_matches: boolean;
  environment_marker_matches: boolean;
  process: {
    exit_code: number;
    elapsed_millis: number;
    stderr: string;
  };
  warnings: string[];
};
type LoaderLabMode =
  | "baseline"
  | "positive"
  | "hook"
  | "missing_target"
  | "missing_dll"
  | "wrong_nonce";
type LoaderLabPanel = {
  status: {
    test_root: string;
    ready: boolean;
    manifest: {
      upstream: {
        package_id: string;
        repository: string;
        tag: string;
        commit: string;
      };
      upstream_loader_sha256: string;
      runner_sha256: string;
      target_sha256: string;
      test_dll_sha256: string;
    } | null;
    missing_or_invalid: string[];
  };
  component: {
    package_id: string;
    version: string | null;
    release_tag: string;
    commit: string;
    loader_sha256: string;
    release_signature: string;
    component_signatures_verified: boolean;
  } | null;
  component_issue: string | null;
  runtimes: BridgeRuntimeOption[];
};
type LoaderExperimentResult = {
  plan: {
    mode: LoaderLabMode;
    loader_mode: string;
    explicit_runtime: { display_name: string; version: string | null };
    prefix_path: string;
    target_executable: string;
    test_dll: string;
    upstream_loader: string;
    expected_marker: string;
    timeout_seconds: number;
  };
  mode: LoaderLabMode;
  outcome: "passed" | "expected_failure";
  loader_mode: string;
  explicit_runtime: { display_name: string; version: string | null };
  compatdata_path: string;
  prefix_path: string;
  target_path_linux: string;
  target_path_windows: string | null;
  test_dll_path_linux: string;
  test_dll_path_windows: string | null;
  upstream_loader_path_linux: string;
  expected_loader_sha256: string;
  actual_loader_sha256: string;
  target_started: boolean;
  target_ready: boolean;
  loader_started: boolean;
  upstream_inject_code: number | null;
  dll_loaded: boolean;
  marker_verified: boolean;
  nonce_verified: boolean;
  marker: {
    process: string;
    process_path_windows: string;
    dll_path_windows: string;
  } | null;
  process: { exit_code: number; elapsed_millis: number; stderr: string };
  capabilities: { name: string; state: string; evidence: string }[];
  warnings: string[];
};
type ZzzLaunchReadiness = {
  game_id: string;
  game_name: string;
  app_id: number;
  distribution: string;
  installation: string;
  game_content_completeness: string;
  executable: string;
  compatdata: string;
  prefix: string;
  zzmi_package: string;
  zzmi_authenticity: string;
  xxmi_libraries_package: string;
  xxmi_libraries_authenticity: string;
  managed_runtime: string;
  game_runtime_selection: string;
  selected_game_runtime: string | null;
  available_proton_tools: string[];
  explicit_lab_runtime: string | null;
  game_install_path: string | null;
  game_executable_path: string | null;
  compatdata_path: string | null;
  prefix_path: string | null;
  steam_process_context: SteamProtonProcessContextEvidence | null;
  managed_runtime_root: string | null;
  zzmi_version: string | null;
  xxmi_libraries_version: string | null;
  upstream_default_loader_mode: string;
  upstream_loader_strategy: string;
  hook_requirement: string;
  loader_strategy: {
    game_id: string;
    platform: string;
    upstream_strategy: string;
    outcome: string;
    preferred_mode: string | null;
    hook_requirement: string;
    direct_inject_candidate: boolean;
    direct_inject_criteria: { id: string; state: string; evidence: string }[];
    isolation: {
      mode: string;
      scope: string;
      state: string;
      evidence: string;
      limitation: string;
    }[];
    evidence: string[];
    known_limitations: string[];
  };
  capabilities: { capability: string; state: string; evidence: string }[];
  requirements: { id: string; state: string; evidence: string }[];
  blockers: string[];
  unknowns: string[];
  decision: string;
  controlled_experiment_gate: {
    decision: string;
    eligible_for_review: boolean;
    explicit_user_consent_required: boolean;
    consent_received: boolean;
    launches_game: boolean;
    rationale: string;
    components: { component: string; state: string; evidence: string }[];
  };
};
type SteamProtonProcessContextEvidence = {
  app_id: number;
  steam_root: string | null;
  library_root: string | null;
  app_manifest_path: string | null;
  compatdata_state: string;
  compatdata_path: string | null;
  prefix_state: string;
  prefix_path: string | null;
  config_path: string | null;
  config_state: string;
  config_issue: string | null;
  per_game_mapping_state: string;
  per_game_mapping: {
    app_id: number;
    internal_name: string | null;
    config: string | null;
    priority: string | null;
  } | null;
  global_default_mapping: {
    app_id: number;
    internal_name: string | null;
    config: string | null;
    priority: string | null;
  } | null;
  matched_installed_tools: ContextToolEvidence[];
  installed_compatibility_tools: ContextToolEvidence[];
  effective_runtime_selection: string;
  effective_prefix: string;
  process_context: string;
  loader_same_prefix_requirement: string;
  loader_same_wineserver_requirement: string;
  wineserver_relationship: string;
  runtime_observation: RuntimeObservationSummary | null;
  findings: {
    claim: string;
    state: string;
    source: string;
    detail: string;
  }[];
};
type ObservationTool = {
  internal_id: string | null;
  display_name: string;
  version: string | null;
  path: string;
};
type ObservationPath = {
  state: string;
  observed_path: string | null;
  expected_path: string | null;
  evidence_source: string | null;
};
type ObservationSummary = {
  state: string;
  observed_values: number[];
  sources: string[];
};
type RuntimeObservationSummary = {
  status: string;
  active: boolean;
  process_observed: boolean;
  ambiguous_process_count: number;
  app_id: ObservationSummary;
  runtime: {
    state: string;
    tool: ObservationTool | null;
    evidence_source: string | null;
  };
  compatdata: ObservationPath;
  prefix: ObservationPath;
  wineserver: {
    state: string;
    related_process_count: number;
    evidence: string;
  };
  process_topology_state: string;
  observed_at_unix_millis: number | null;
};
type ProcfsScanDiagnostics = {
  same_uid_processes_scanned: number;
  candidate_processes_considered: number;
  candidate_metadata_unavailable: number;
  candidate_environments_read: number;
  candidate_environments_unavailable: number;
  candidate_environments_permission_denied: number;
  accepted_contexts_environment_unavailable: number;
  accepted_contexts_environment_permission_denied: number;
  rejected_executable_mismatch: number;
  rejected_insufficient_identity_evidence: number;
  accepted_candidates: number;
};
type RuntimeObservationSnapshot = {
  session_id: number;
  status: string;
  active: boolean;
  app_id: number;
  started_at_unix_millis: number;
  updated_at_unix_millis: number;
  elapsed_millis: number;
  scan_diagnostics: ProcfsScanDiagnostics;
  process: {
    pid: number;
    parent_pid: number | null;
    executable_name: string;
    identity_evidence: string[];
  } | null;
  ambiguous_process_count: number;
  app_id_evidence: ObservationSummary;
  configured_runtime: {
    per_game_state: string;
    per_game_internal_id: string | null;
    global_state: string;
    global_internal_id: string | null;
    source: string | null;
  };
  available_proton_tools: ObservationTool[];
  observed_runtime: RuntimeObservationSummary["runtime"];
  compatdata: ObservationPath;
  prefix: ObservationPath;
  process_topology: {
    pid: number;
    parent_pid: number | null;
    process_name: string;
    executable_name: string | null;
    role: string;
  }[];
  process_topology_state: string;
  wineserver: RuntimeObservationSummary["wineserver"];
  environment_evidence: string;
  warnings: string[];
};
type ObservationCommandError = { code: string; detail: string };
type ContextToolEvidence = {
  internal_id: string | null;
  display_name: string;
  kind: string;
  origin: string;
  version: string | null;
  path: string;
};
type GameId = "wuthering-waves" | "zenless-zone-zero";
const games: Record<GameId, { name: string; integration: "wwmi" | "zzmi" }> = {
  "wuthering-waves": { name: "Wuthering Waves", integration: "wwmi" },
  "zenless-zone-zero": { name: "Zenless Zone Zero", integration: "zzmi" },
};
const presence: Record<string, string> = {
  absent: "No detectado",
  structurally_present: "Estructura presente · origen no verificado",
  incomplete: "Estructura incompleta",
  unreadable: "No se pudo inspeccionar",
};
const states: Record<string, string> = {
  satisfied: "Presente",
  unsatisfied: "Pendiente",
  unknown: "No determinado",
  not_required: "No necesario",
};
const readinessDecisionLabels: Record<string, string> = {
  not_ready: "Aún no listo",
  needs_evidence: "Falta evidencia",
  ready_for_controlled_experiment: "Listo para revisar un experimento",
  blocked: "Bloqueado",
};
const loaderOutcomeLabels: Record<string, string> = {
  hook: "Hook",
  direct_inject: "Direct Inject",
  no_decision: "Sin decisión",
  blocked: "Bloqueado",
};
function stateLabel(value: string): string {
  const labels: Record<string, string> = {
    verified: "Verificado",
    observed: "Observado",
    observed_and_matches: "Observado · coincide",
    observed_different: "Observado · distinto",
    derived_from_observed_compatdata: "Derivado del compatdata observado",
    conflicting: "Evidencia contradictoria",
    unavailable: "No disponible",
    permission_denied: "Permiso denegado",
    not_observed: "No observado",
    waiting_for_game: "Esperando inicio manual",
    completed: "Observación completada",
    process_not_found: "No se encontró un proceso coincidente",
    ambiguous_process: "Varios procesos candidatos",
    process_exited: "El proceso terminó",
    timed_out: "Ventana de observación finalizada",
    backend_error: "Fallo del backend de observación",
    observed_ancestry: "Ascendencia observada",
    partial_topology: "Topología parcial",
    upstream_documented: "Documentado upstream",
    unverified: "Sin verificar",
    not_tested: "No probado",
    unsupported: "No compatible",
    unknown: "Desconocido",
    blocked: "Bloqueado",
    read: "Leído",
    missing: "No encontrado",
    invalid: "Inválido",
    unreadable: "No se pudo leer",
    unsafe_path: "Ruta no segura",
    too_large: "Supera el límite de lectura",
    not_inspected: "No inspeccionado",
    configured: "Configurado",
    no_per_game_override: "Sin mapping específico para este juego",
    detected: "Detectado",
    absent: "Ausente",
    known: "Conocido",
    not_required: "No requerido",
    satisfied: "Presente",
    unsatisfied: "Pendiente",
    not_established: "No establecido",
    hook: "Hook",
    direct_inject: "Direct Inject",
    hook_only: "Solo Hook",
    direct_inject_only: "Solo Direct Inject",
    either_depending_on_launch_mode: "Depende del modo de lanzamiento",
  };
  return labels[value] ?? value.replaceAll("_", " ");
}
function errorText(error: unknown): string {
  if (
    typeof error === "object" &&
    error !== null &&
    "detail" in error &&
    typeof error.detail === "string"
  )
    return error.detail;
  return "No se pudo completar la operación local. Revisa permisos y el diagnóstico de Steam.";
}
function observationErrorValue(error: unknown): ObservationCommandError {
  if (typeof error === "object" && error !== null) {
    const candidate = error as { code?: unknown; detail?: unknown };
    return {
      code:
        typeof candidate.code === "string" ? candidate.code : "backend_error",
      detail:
        typeof candidate.detail === "string"
          ? candidate.detail
          : errorText(error),
    };
  }
  return { code: "backend_error", detail: errorText(error) };
}

export default function XxmiPanel() {
  const [status, setStatus] = useState<Status | null>(null);
  const [path, setPath] = useState("");
  const [gameId, setGameId] = useState<GameId>("zenless-zone-zero");
  const [selected, setSelected] = useState("");
  const [libraries, setLibraries] = useState("");
  const [plan, setPlan] = useState<Plan | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [releases, setReleases] = useState<OfficialRelease[]>([]);
  const [selectedTags, setSelectedTags] = useState<Record<string, string>>({});
  const [managedRuntime, setManagedRuntime] = useState<ManagedRuntime | null>(
    null,
  );
  const [topology, setTopology] = useState<LaunchTopology | null>(null);
  const [assemblyPlan, setAssemblyPlan] = useState<AssemblyPlan | null>(null);
  const [bridgePanel, setBridgePanel] = useState<RuntimeBridgePanel | null>(
    null,
  );
  const [bridgeRuntimePath, setBridgeRuntimePath] = useState("");
  const [bridgeResult, setBridgeResult] = useState<BridgeTestResult | null>(
    null,
  );
  const [bridgeSideEffectsAcknowledged, setBridgeSideEffectsAcknowledged] =
    useState(false);
  const [loaderLabPanel, setLoaderLabPanel] = useState<LoaderLabPanel | null>(
    null,
  );
  const [loaderRuntimePath, setLoaderRuntimePath] = useState("");
  const [loaderLabMode, setLoaderLabMode] = useState<LoaderLabMode>("baseline");
  const [loaderSideEffectsAcknowledged, setLoaderSideEffectsAcknowledged] =
    useState(false);
  const [loaderResult, setLoaderResult] =
    useState<LoaderExperimentResult | null>(null);
  const [zzzReadiness, setZzzReadiness] = useState<ZzzLaunchReadiness | null>(
    null,
  );
  const [runtimeObservation, setRuntimeObservation] =
    useState<RuntimeObservationSnapshot | null>(null);
  const [observationConsent, setObservationConsent] = useState(false);
  const [observationBusy, setObservationBusy] = useState(false);
  const [observationError, setObservationError] =
    useState<ObservationCommandError | null>(null);
  const observationSessionId = runtimeObservation?.session_id;
  const observationSessionActive = runtimeObservation?.active === true;

  useEffect(() => {
    if (!observationSessionActive) return;
    const timer = window.setInterval(() => {
      void invoke<RuntimeObservationSnapshot | null>(
        "get_zzz_runtime_observation",
      )
        .then((snapshot) => {
          if (snapshot) {
            setRuntimeObservation(snapshot);
            setObservationError(null);
          }
        })
        .catch((reason: unknown) =>
          setObservationError(observationErrorValue(reason)),
        );
    }, 1000);
    return () => window.clearInterval(timer);
  }, [observationSessionActive, observationSessionId]);

  function selectDefaultPackages(nextStatus: Status) {
    const preferredZzmi = nextStatus.packages
      .filter(
        (p) =>
          typeof p.kind === "object" &&
          p.kind.game_integration === "zzmi" &&
          p.authenticity === "official_release_verified",
      )
      .sort((a, b) => b.imported_unix_seconds - a.imported_unix_seconds)[0];
    const preferredLibraries = nextStatus.packages
      .filter(
        (p) =>
          p.kind === "xxmi_libraries" &&
          p.authenticity === "official_release_verified",
      )
      .sort((a, b) => b.imported_unix_seconds - a.imported_unix_seconds)[0];
    setSelected((current) => current || preferredZzmi?.id || "");
    setLibraries((current) => current || preferredLibraries?.id || "");
  }

  async function run(action: "inspect" | "import" | "plan") {
    setBusy(true);
    setError("");
    setNotice("");
    setPlan(null);
    if (action === "import") {
      setAssemblyPlan(null);
      setManagedRuntime(null);
      setTopology(null);
    }
    try {
      if (action === "plan") {
        setPlan(
          await invoke<Plan>("review_xxmi_plan", {
            gameId,
            packageId: selected,
            librariesId: libraries || null,
          }),
        );
        setNotice("Plan generado. No se aplicó ningún cambio.");
      } else {
        if (action === "import") {
          const imported = await invoke<Package>("import_xxmi_directory", {
            path: path.trim(),
          });
          setNotice(
            `Paquete verificado en almacenamiento LXMI: ${imported.id.slice(0, 12)}. No instalado en el juego.`,
          );
          if (imported.kind === "xxmi_libraries") setLibraries(imported.id);
          else if (typeof imported.kind === "object") {
            setSelected(imported.id);
            if (imported.kind.game_integration === "zzmi") {
              setGameId("zenless-zone-zero");
            } else if (imported.kind.game_integration === "wwmi") {
              setGameId("wuthering-waves");
            }
          }
        }
        const nextStatus = await invoke<Status>("xxmi_status");
        setStatus(nextStatus);
        selectDefaultPackages(nextStatus);
      }
    } catch (e: unknown) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }
  async function checkOfficialReleases() {
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const found = await invoke<OfficialRelease[]>(
        "xxmi_check_official_releases",
      );
      setReleases(found);
      setSelectedTags(
        Object.fromEntries(
          found.map((release) => [release.package_kind, release.tag]),
        ),
      );
      setNotice("Metadata oficial consultada. Nada se descargó todavía.");
    } catch (e: unknown) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }
  async function downloadOfficial(kind: "zzmi" | "xxmi_libraries") {
    const tag = selectedTags[kind];
    if (!tag) return;
    setBusy(true);
    setError("");
    setNotice("");
    setPlan(null);
    setAssemblyPlan(null);
    setManagedRuntime(null);
    setTopology(null);
    try {
      const imported = await invoke<Package>("xxmi_download_official_package", {
        packageKind: kind,
        tag,
      });
      const nextStatus = await invoke<Status>("xxmi_status");
      setStatus(nextStatus);
      selectDefaultPackages(nextStatus);
      if (kind === "zzmi") setSelected(imported.id);
      else setLibraries(imported.id);
      const verified = imported.authenticity === "official_release_verified";
      setNotice(
        verified
          ? `${kind === "zzmi" ? "ZZMI" : "XXMI Libraries"} ${imported.upstream?.tag ?? tag}: firma del ZIP verificada; ${imported.upstream?.component_signatures_verified ? "firmas de componentes verificadas; " : ""}inventario guardado en LXMI. No instalado en el juego.`
          : "El paquete no alcanzó el estado de autenticidad verificada; revisa el diagnóstico.",
      );
    } catch (e: unknown) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }

  async function prepareRuntime() {
    if (gameId !== "zenless-zone-zero" || !selected || !libraries) return;
    if (
      !assemblyPlan ||
      assemblyPlan.zzmi_package_id !== selected ||
      assemblyPlan.libraries_package_id !== libraries
    ) {
      setError(
        "Revisa de nuevo el plan de ensamblado para los paquetes seleccionados.",
      );
      return;
    }
    setBusy(true);
    setError("");
    setNotice("");
    setTopology(null);
    try {
      const runtime = await invoke<ManagedRuntime>("prepare_zzmi_runtime", {
        zzmiId: selected,
        librariesId: libraries,
      });
      setManagedRuntime(runtime);
      setNotice(
        `Runtime ensamblado en almacenamiento privado LXMI (${runtime.file_count} archivos). No se modificaron juego, Steam ni prefix.`,
      );
    } catch (e: unknown) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }

  async function reviewAssembly() {
    if (gameId !== "zenless-zone-zero" || !selected || !libraries) return;
    setBusy(true);
    setError("");
    setNotice("");
    setManagedRuntime(null);
    setTopology(null);
    try {
      const result = await invoke<AssemblyPlan>("review_zzmi_assembly", {
        zzmiId: selected,
        librariesId: libraries,
      });
      setAssemblyPlan(result);
      setNotice(
        "Plan de composición listo para revisar. Todavía no se escribieron archivos.",
      );
    } catch (e: unknown) {
      setAssemblyPlan(null);
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }

  async function inspectTopology() {
    if (gameId !== "zenless-zone-zero" || !selected || !libraries) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const result = await invoke<LaunchTopology>(
        "inspect_zzmi_launch_topology",
        { zzmiId: selected, librariesId: libraries },
      );
      setTopology(result);
      setNotice(
        "Topología inspeccionada en modo de solo lectura; nada se ejecutó.",
      );
    } catch (e: unknown) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }

  async function inspectBridge() {
    setBusy(true);
    setError("");
    setNotice("");
    setBridgeResult(null);
    setBridgeSideEffectsAcknowledged(false);
    try {
      const result = await invoke<RuntimeBridgePanel>(
        "inspect_runtime_bridge",
        {
          zzmiId: selected || null,
          librariesId: libraries || null,
        },
      );
      setBridgePanel(result);
      setBridgeRuntimePath((current) =>
        result.runtimes.some((runtime) => runtime.proton_script === current)
          ? current
          : "",
      );
      setNotice(
        "Discovery de runtimes para una prueba explícita completado. La selección de Proton del juego sigue desconocida.",
      );
    } catch (e: unknown) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }

  async function runBridgeTest() {
    if (
      !selected ||
      !libraries ||
      !bridgeRuntimePath ||
      !bridgeSideEffectsAcknowledged
    )
      return;
    setBusy(true);
    setError("");
    setNotice("");
    setBridgeResult(null);
    try {
      const result = await invoke<BridgeTestResult>("run_runtime_bridge_test", {
        zzmiId: selected,
        librariesId: libraries,
        protonScript: bridgeRuntimePath,
      });
      setBridgeResult(result);
      setNotice(
        "Bridge test completado. Solo se usó el prefix aislado de LXMI; ZZZ no fue iniciado ni inspeccionado como proceso.",
      );
    } catch (e: unknown) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }

  async function inspectLoaderLab() {
    setBusy(true);
    setError("");
    setNotice("");
    setLoaderResult(null);
    setLoaderSideEffectsAcknowledged(false);
    try {
      const result = await invoke<LoaderLabPanel>("inspect_loader_lab");
      setLoaderLabPanel(result);
      setLoaderRuntimePath((current) =>
        result.runtimes.some((runtime) => runtime.proton_script === current)
          ? current
          : "",
      );
      setNotice(
        "Loader Lab inspeccionado. Solo se admite el test target fijo de LXMI.",
      );
    } catch (e: unknown) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }

  async function prepareLoaderLab() {
    setBusy(true);
    setError("");
    setNotice("");
    setLoaderResult(null);
    try {
      await invoke("prepare_loader_lab");
      const result = await invoke<LoaderLabPanel>("inspect_loader_lab");
      setLoaderLabPanel(result);
      setNotice("Loader Lab preparado dentro del storage controlado de LXMI.");
    } catch (e: unknown) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }

  async function runLoaderLab() {
    if (
      loaderLabMode !== "hook" &&
      (!loaderRuntimePath || !loaderSideEffectsAcknowledged)
    )
      return;
    setBusy(true);
    setError("");
    setNotice("");
    setLoaderResult(null);
    try {
      const result = await invoke<LoaderExperimentResult>(
        "run_loader_lab_experiment",
        {
          protonScript: loaderRuntimePath,
          mode: loaderLabMode,
          sideEffectsAcknowledged: loaderSideEffectsAcknowledged,
        },
      );
      setLoaderResult(result);
      setNotice(
        result.outcome === "passed"
          ? "Experimento completado contra el único proceso de prueba de LXMI."
          : "El control negativo produjo el rechazo esperado.",
      );
    } catch (e: unknown) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }

  async function inspectZzzReadiness() {
    setBusy(true);
    setError("");
    setNotice("");
    setZzzReadiness(null);
    try {
      const result = await invoke<ZzzLaunchReadiness>(
        "inspect_zzz_launch_readiness",
        {
          zzmiId: selected || null,
          librariesId: libraries || null,
        },
      );
      setZzzReadiness(result);
      setNotice(
        "Readiness evaluada desde discovery y paquetes locales. No se inició ZZZ.",
      );
    } catch (e: unknown) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }

  async function startRuntimeObservation() {
    setObservationBusy(true);
    setObservationError(null);
    try {
      const snapshot = await invoke<RuntimeObservationSnapshot>(
        "start_zzz_runtime_observation",
        { userConfirmedManualLaunch: observationConsent },
      );
      setRuntimeObservation(snapshot);
      setObservationConsent(false);
    } catch (reason: unknown) {
      setObservationError(observationErrorValue(reason));
    } finally {
      setObservationBusy(false);
    }
  }

  async function stopRuntimeObservation() {
    setObservationBusy(true);
    setObservationError(null);
    try {
      const snapshot = await invoke<RuntimeObservationSnapshot | null>(
        "stop_zzz_runtime_observation",
      );
      if (snapshot) setRuntimeObservation(snapshot);
    } catch (reason: unknown) {
      setObservationError(observationErrorValue(reason));
    } finally {
      setObservationBusy(false);
    }
  }

  const packages = status?.packages ?? [];
  const zzmiRelease = releases.find(
    (release) => release.package_kind === "zzmi",
  );
  const librariesRelease = releases.find(
    (release) => release.package_kind === "xxmi_libraries",
  );
  const managedZzmi = packages
    .filter(
      (p) =>
        typeof p.kind === "object" &&
        p.kind.game_integration === "zzmi" &&
        p.authenticity === "official_release_verified",
    )
    .sort((a, b) => b.imported_unix_seconds - a.imported_unix_seconds);
  const managedLibraries = packages
    .filter(
      (p) =>
        p.kind === "xxmi_libraries" &&
        p.authenticity === "official_release_verified",
    )
    .sort((a, b) => b.imported_unix_seconds - a.imported_unix_seconds);
  const reviewedAssemblyMatchesSelection = Boolean(
    assemblyPlan &&
    assemblyPlan.zzmi_package_id === selected &&
    assemblyPlan.libraries_package_id === libraries,
  );
  return (
    <section
      className="panel xxmi-panel"
      aria-labelledby="xxmi-heading"
      aria-busy={busy}
    >
      <div className="section-heading">
        <h2 id="xxmi-heading">
          XXMI · {games[gameId].integration.toUpperCase()}
        </h2>
        <button
          className="primary-button"
          disabled={busy}
          onClick={() => void run("inspect")}
        >
          {busy ? "Procesando…" : "Inspeccionar paquetes"}
        </button>
      </div>
      <p className="message">
        {games[gameId].name}: importa una copia local y revisa el plan
        declarativo. El plan no se aplica al juego y la compatibilidad
        Steam/Linux/Proton sigue sin verificar.
      </p>
      <div className="runtime-assessment">
        <h3>Adquisición oficial · sin instalación al juego</h3>
        <p>
          La consulta de releases solo ocurre al pulsar el botón. Una firma
          upstream respalda la procedencia del asset firmado; el hash local
          detecta cambios posteriores. XXMI upstream recomienda usar su launcher
          para instalar estos paquetes. LXMI solo descarga, valida, guarda y
          prepara un dry-run; no instala en ZZZ.
        </p>
        <button
          className="primary-button"
          disabled={busy}
          onClick={() => void checkOfficialReleases()}
        >
          {busy ? "Consultando…" : "Consultar releases oficiales"}
        </button>
        {(
          [
            ["zzmi", "ZZMI", zzmiRelease],
            ["xxmi_libraries", "XXMI Libraries", librariesRelease],
          ] as const
        ).map(([kind, label, release]) => (
          <div className="xxmi-form" key={kind}>
            <h4>{label}</h4>
            {release ? (
              <>
                <p>
                  <a
                    href={release.release_url}
                    target="_blank"
                    rel="noreferrer"
                  >
                    {release.repository} · {release.tag} · release{" "}
                    {release.release_id}
                  </a>
                  {" · "}publicada{" "}
                  {new Date(release.published_at).toLocaleDateString()}
                </p>
                <p>
                  Commit {release.commit} · metadata consultada{" "}
                  {new Date(release.metadata_retrieved_at).toLocaleString()} ·
                  firma publicada:{" "}
                  {release.signature_base64
                    ? "sí; pendiente de verificar el asset"
                    : "no"}
                </p>
                <ul>
                  {release.assets.map((asset) => (
                    <li key={asset.id}>
                      {asset.name} · {asset.size.toLocaleString()} bytes ·
                      SHA-256 {asset.sha256 ?? "no publicado por la API"}
                    </li>
                  ))}
                </ul>
                <label htmlFor={`release-${kind}`}>Release exacta</label>
                <select
                  id={`release-${kind}`}
                  value={selectedTags[kind] ?? release.tag}
                  disabled={busy}
                  onChange={(event) =>
                    setSelectedTags({
                      ...selectedTags,
                      [kind]: event.target.value,
                    })
                  }
                >
                  <option value={release.tag}>
                    {release.tag} · ID {release.release_id}
                  </option>
                </select>
                <button
                  className="primary-button"
                  disabled={busy || !release.signature_base64}
                  onClick={() => void downloadOfficial(kind)}
                >
                  {busy
                    ? "Descargando y verificando…"
                    : `Descargar, verificar e importar ${label}`}
                </button>
              </>
            ) : (
              <p className="discovery-note">
                Aún no se consultó una release. No se ha realizado ninguna
                descarga.
              </p>
            )}
          </div>
        ))}
        <p>
          Paquetes oficiales verificados: ZZMI{" "}
          {managedZzmi
            .map((package_) => package_.upstream?.tag)
            .filter(Boolean)
            .join(", ") || "no disponible"}
          ; XXMI Libraries{" "}
          {managedLibraries
            .map((package_) => package_.upstream?.tag)
            .filter(Boolean)
            .join(", ") || "no disponible"}
          . Dependencia:{" "}
          {managedZzmi.length && managedLibraries.length
            ? "completa"
            : "pendiente"}
          . Compatibilidad Steam/Linux/Proton: no verificada.
        </p>
      </div>
      <div className="xxmi-form">
        <label htmlFor="xxmi-game">Juego e integración</label>
        <select
          id="xxmi-game"
          value={gameId}
          disabled={busy}
          onChange={(event) => {
            setGameId(event.target.value as GameId);
            setSelected("");
            setPlan(null);
            setManagedRuntime(null);
            setTopology(null);
            setAssemblyPlan(null);
            setBridgePanel(null);
            setBridgeRuntimePath("");
            setBridgeResult(null);
          }}
        >
          <option value="zenless-zone-zero">Zenless Zone Zero · ZZMI</option>
          <option value="wuthering-waves">Wuthering Waves · WWMI</option>
        </select>
      </div>
      {!status && (
        <p className="discovery-note">
          Inspecciona las ubicaciones conocidas para consultar el runtime y los
          paquetes administrados. No se buscan instalaciones externas
          arbitrarias.
        </p>
      )}
      {status && (
        <>
          <p className="discovery-note">
            {packages.length
              ? `${packages.length} inventarios de paquetes administrados comprobados; la autenticidad se indica por paquete.`
              : "Todavía no hay paquetes administrados disponibles."}
          </p>
          <details className="runtime-evidence">
            <summary>Ubicaciones inspeccionadas y evidencia</summary>
            <p className="path-value">Almacenamiento: {status.storage_root}</p>
            <ul>
              {status.observations.map((item) => (
                <li key={item.path}>
                  <code className="path-value">{item.path}</code>
                  <div>
                    {item.integration_kind?.toUpperCase() ?? "Integración"}:{" "}
                    {presence[item.integration]} · Bibliotecas:{" "}
                    {presence[item.libraries]}
                  </div>
                  {item.evidence.map((e) => (
                    <p key={e}>{e}</p>
                  ))}
                  {item.issues.map((e, i) => (
                    <p key={i}>{e.detail}</p>
                  ))}
                </li>
              ))}
            </ul>
          </details>
          {status.issues.map((e, i) => (
            <p className="message-error" key={i}>
              {e.detail}
            </p>
          ))}
        </>
      )}
      <form
        className="xxmi-form"
        onSubmit={(event) => {
          event.preventDefault();
          void run("import");
        }}
      >
        <label htmlFor="xxmi-path">Carpeta local del paquete</label>
        <input
          id="xxmi-path"
          value={path}
          onChange={(event) => setPath(event.target.value)}
          placeholder="/ruta/al/paquete-extraido"
          required
          disabled={busy}
          aria-describedby="xxmi-path-help xxmi-error"
          aria-invalid={Boolean(error)}
        />
        <p id="xxmi-path-help" className="discovery-note">
          ZZMI: raíz que contiene d3dx.ini y Core/ZZMI; WWMI: d3dx.ini y
          Core/WWMI. XXMI Libraries se importa aparte. ZIP oficial se obtiene
          mediante el flujo de releases anterior. Límite: 128 MiB por archivo y
          512 MiB en total. El hash local no autentica el origen.
        </p>
        <button
          className="primary-button"
          type="submit"
          disabled={busy || !path.trim()}
        >
          Importar copia local a LXMI (origen no autenticado)
        </button>
      </form>
      <div className="xxmi-form">
        <label htmlFor="integration-package">
          Paquete {games[gameId].integration.toUpperCase()} para revisar
        </label>
        <select
          id="integration-package"
          value={selected}
          disabled={busy}
          onChange={(event) => {
            setSelected(event.target.value);
            setPlan(null);
            setManagedRuntime(null);
            setTopology(null);
            setAssemblyPlan(null);
            setBridgePanel(null);
            setBridgeResult(null);
          }}
        >
          <option value="">Seleccionar paquete…</option>
          {packages
            .filter(
              (p) =>
                typeof p.kind === "object" &&
                p.kind.game_integration === games[gameId].integration,
            )
            .map((p) => (
              <option key={p.id} value={p.id}>
                {games[gameId].integration.toUpperCase()} ·{" "}
                {p.version?.raw ?? "versión desconocida"} · {p.id.slice(0, 12)}
              </option>
            ))}
        </select>
        <label htmlFor="xxmi-libraries">
          XXMI Libraries (paquete separado)
        </label>
        <select
          id="xxmi-libraries"
          value={libraries}
          disabled={busy}
          onChange={(event) => {
            setLibraries(event.target.value);
            setPlan(null);
            setManagedRuntime(null);
            setTopology(null);
            setAssemblyPlan(null);
            setBridgePanel(null);
            setBridgeResult(null);
          }}
        >
          <option value="">No disponible / no seleccionado</option>
          {packages
            .filter((p) => p.kind === "xxmi_libraries")
            .map((p) => (
              <option key={p.id} value={p.id}>
                XXMI · {p.version?.raw ?? "versión desconocida"} ·{" "}
                {p.id.slice(0, 12)}
              </option>
            ))}
        </select>
        <button
          className="primary-button"
          disabled={busy || !selected}
          onClick={() => void run("plan")}
        >
          Ver mapping histórico 0.5.2
        </button>
      </div>
      {gameId === "zenless-zone-zero" && (
        <div
          className="runtime-assessment"
          aria-labelledby="managed-runtime-heading"
        >
          <h3 id="managed-runtime-heading">ZZMI · runtime administrado</h3>
          <p>
            El importer queda fuera de la carpeta del juego. Preparar este
            runtime solo escribe bajo el almacenamiento XDG de LXMI; no inicia
            ZZZ ni modifica Steam, compatdata o el prefix.
          </p>
          <div className="xxmi-form">
            <button
              className="primary-button"
              disabled={busy || !selected || !libraries}
              onClick={() => void reviewAssembly()}
            >
              {busy ? "Revisando…" : "Revisar composición"}
            </button>
            <button
              className="primary-button"
              disabled={busy || !reviewedAssemblyMatchesSelection}
              onClick={() => void prepareRuntime()}
            >
              {busy ? "Preparando…" : "Preparar runtime administrado"}
            </button>
            <button
              className="secondary-button"
              disabled={busy || !selected || !libraries}
              onClick={() => void inspectTopology()}
            >
              Inspeccionar topología
            </button>
          </div>
          {assemblyPlan && (
            <details className="runtime-evidence" open>
              <summary>
                Composición revisada · {assemblyPlan.files.length} archivos
              </summary>
              <p>
                App.Root:{" "}
                <code className="path-value">{assemblyPlan.app_root}</code>
              </p>
              <p>
                Importer relativo:{" "}
                <code>{assemblyPlan.import_path_from_app_root}</code> · destino
                calculado:{" "}
                <code className="path-value">{assemblyPlan.importer_root}</code>
              </p>
              <p>
                Target: <code>{assemblyPlan.target_process}</code> · loader
                identity sin cambiar:{" "}
                <code>{assemblyPlan.config.loader_process_identity}</code>
              </p>
              <p>
                Config derivada:{" "}
                <code>{assemblyPlan.config.source_relative_path}</code> · origen
                SHA-256 <code>{assemblyPlan.config.source_sha256}</code> ·
                derivada SHA-256{" "}
                <code>{assemblyPlan.config.derived_sha256}</code>
              </p>
              <p>
                `3dmloader.dll` permanece en el paquete de Libraries:{" "}
                <code className="path-value">
                  {assemblyPlan.loader_library_source}
                </code>
              </p>
              <ul>
                {assemblyPlan.files.map((file) => (
                  <li
                    key={`${file.source_component}:${file.destination_relative_path}`}
                  >
                    {file.source_component}:{" "}
                    <code>{file.source_relative_path}</code> →{" "}
                    <code>{file.destination_relative_path}</code> ·{" "}
                    {file.size.toLocaleString()} bytes
                    {file.derived ? " · configuración derivada" : ""}
                  </li>
                ))}
              </ul>
              <ul>
                {assemblyPlan.evidence.map((item) => (
                  <li key={item}>{item}</li>
                ))}
                {assemblyPlan.warnings.map((item) => (
                  <li key={item}>{item}</li>
                ))}
              </ul>
              <p>
                Este plan prepara únicamente almacenamiento LXMI. No instala
                contenido en ZZZ y no autoriza compatibilidad ni lanzamiento.
              </p>
            </details>
          )}
          {(!selected || !libraries) && (
            <p className="discovery-note">
              Selecciona paquetes ZZMI y XXMI Libraries autenticados para
              continuar.
            </p>
          )}
          {managedRuntime && (
            <details className="runtime-evidence" open>
              <summary>
                Runtime{" "}
                {managedRuntime.newly_created ? "preparado" : "verificado"} ·{" "}
                {managedRuntime.file_count} archivos
              </summary>
              <p>
                ZZMI {managedRuntime.zzmi_version ?? "versión desconocida"} ·
                XXMI Libraries{" "}
                {managedRuntime.libraries_version ?? "versión desconocida"}
              </p>
              <p>
                App.Root:{" "}
                <code className="path-value">{managedRuntime.app_root}</code>
              </p>
              <p>
                Importer:{" "}
                <code className="path-value">
                  {managedRuntime.importer_root}
                </code>
              </p>
              <p>
                Target process: <code>{managedRuntime.target_process}</code>
              </p>
              <p>
                Loader identity:{" "}
                <code>{managedRuntime.loader_process_identity}</code> ·
                estrategia de helper: no seleccionada.
              </p>
              <p>
                Injector library administrada:{" "}
                <code className="path-value">
                  {managedRuntime.loader_library_path}
                </code>
              </p>
              <p>
                Instalado en el juego:{" "}
                {managedRuntime.installed_into_game ? "sí" : "no"}. Este runtime
                aún no es un helper ejecutable.
              </p>
              <p>
                Manifest:{" "}
                <code className="path-value">
                  {managedRuntime.manifest_path}
                </code>
              </p>
            </details>
          )}
          {topology && (
            <details className="runtime-evidence" open>
              <summary>
                Launch topology · {topology.readiness.replaceAll("_", " ")}
              </summary>
              <p>
                {topology.game_name} · {topology.distribution} · AppID{" "}
                {topology.steam_app_id ?? "desconocido"}
              </p>
              <p>
                Instalación: {topology.game_installation} · compatdata:{" "}
                {topology.compatdata} · prefix:{" "}
                {topology.prefix_path ?? "no detectado"}
              </p>
              <p>
                Proton seleccionado: {topology.selected_proton} · candidatos:{" "}
                {topology.available_proton_candidates.length}
              </p>
              <p>
                Ejecutable:{" "}
                <code className="path-value">
                  {topology.game_executable ?? "no detectado"}
                </code>
              </p>
              <p>
                Mapeo del ejecutable:{" "}
                {topology.game_executable_windows_mapping?.windows_path ??
                  topology.game_executable_windows_mapping?.status ??
                  "sin mapping"}
              </p>
              <p>
                Importer en Windows:{" "}
                {topology.importer_root_windows_mapping?.windows_path ??
                  topology.importer_root_windows_mapping?.status ??
                  "sin mapping"}
              </p>
              <p>
                Loader strategy: {topology.loader_strategy} · mismo prefix:{" "}
                {topology.same_prefix_requirement} · módulo: {topology.module}
              </p>
              <p>
                Ejecución habilitada: {topology.execution_enabled ? "sí" : "no"}{" "}
                · archivos externos modificados:{" "}
                {topology.external_files_modified ? "sí" : "no"}
              </p>
              {topology.dosdevices && (
                <details>
                  <summary>
                    Mapeos de Wine · {topology.dosdevices.state}
                  </summary>
                  <ul>
                    {topology.dosdevices.mappings.map((mapping) => (
                      <li key={mapping.drive_letter}>
                        <code>{mapping.drive_letter.toUpperCase()}:</code> ·{" "}
                        {mapping.state} ·{" "}
                        <code className="path-value">
                          {mapping.raw_target ?? "sin target"}
                        </code>
                      </li>
                    ))}
                  </ul>
                </details>
              )}
              <details>
                <summary>Capacidades y asuntos sin resolver</summary>
                <ul>
                  {topology.capabilities.map((item) => (
                    <li key={item.name}>
                      <strong>
                        {item.name}: {item.state}
                      </strong>{" "}
                      — {item.evidence}
                    </li>
                  ))}
                  {topology.unknowns.map((item) => (
                    <li key={item}>{item}</li>
                  ))}
                  {topology.blockers.map((item) => (
                    <li key={item}>{item}</li>
                  ))}
                </ul>
              </details>
            </details>
          )}
          <details className="runtime-evidence bridge-test-panel">
            <summary>Advanced · Runtime Bridge Test (desarrollo)</summary>
            <p>
              Esta prueba inicia únicamente el helper inocuo de LXMI mediante
              una versión Proton elegida explícitamente. No determina ni cambia
              el Proton que Steam selecciona para ZZZ. Proton inicializará un
              prefix aislado bajo el almacenamiento de LXMI; nunca se usa
              compatdata/4162040.
            </p>
            <button
              className="secondary-button"
              disabled={busy}
              onClick={() => void inspectBridge()}
            >
              {busy ? "Inspeccionando…" : "Inspeccionar bridge y runtimes"}
            </button>
            {bridgePanel && (
              <>
                <div>
                  <p>
                    Helper: {bridgePanel.options.helper.state}
                    {bridgePanel.options.helper.state === "available" && (
                      <>
                        {" · "}
                        {bridgePanel.options.helper.helper_version}
                        {" · SHA-256 "}
                        {bridgePanel.options.helper.integrity_matches
                          ? "coincide"
                          : "no coincide"}
                      </>
                    )}
                    {bridgePanel.options.helper.state === "invalid" && (
                      <> · {bridgePanel.options.helper.detail}</>
                    )}
                  </p>
                  {bridgePanel.options.helper.state === "missing" && (
                    <div className="discovery-note">
                      Falta el helper Windows. Desde la raíz de la aplicación,
                      compílalo para x86_64-pc-windows-gnu y prepara el hash
                      local:
                      <br />
                      <code className="path-value">
                        cargo build --manifest-path
                        tools/lxmi-bridge-helper/Cargo.toml --target
                        x86_64-pc-windows-gnu --release
                      </code>
                      <br />
                      <code className="path-value">
                        bash tools/install-bridge-helper.sh
                      </code>
                    </div>
                  )}
                </div>
                <p>
                  Runtime ZZMI administrado:{" "}
                  {bridgePanel.options.managed_runtime_available
                    ? bridgePanel.options.managed_runtime_path
                    : "no disponible para los paquetes seleccionados"}
                </p>
                <label htmlFor="bridge-test-runtime">
                  Runtime Proton explícito para esta prueba
                </label>
                <select
                  id="bridge-test-runtime"
                  value={bridgeRuntimePath}
                  disabled={busy || !bridgePanel.runtimes.length}
                  onChange={(event) => setBridgeRuntimePath(event.target.value)}
                >
                  <option value="">Elegir Proton…</option>
                  {bridgePanel.runtimes.map((runtime) => (
                    <option
                      key={runtime.proton_script}
                      value={runtime.proton_script}
                    >
                      {runtime.display_name} ·{" "}
                      {runtime.version ?? "versión desconocida"} ·{" "}
                      {runtime.source}
                    </option>
                  ))}
                </select>
                {!bridgePanel.runtimes.length && (
                  <p className="discovery-note">
                    No se encontró un Proton válido con entrypoint ejecutable.
                  </p>
                )}
                <p>
                  Selección del juego: {bridgePanel.game_runtime_selection} ·
                  prefix de prueba: {bridgePanel.options.prefix_mode} · Proton
                  puede crear o actualizar el prefix aislado de LXMI.
                </p>
                <label className="discovery-note">
                  <input
                    type="checkbox"
                    checked={bridgeSideEffectsAcknowledged}
                    disabled={busy}
                    onChange={(event) =>
                      setBridgeSideEffectsAcknowledged(event.target.checked)
                    }
                  />{" "}
                  Entiendo que Proton puede escribir en el prefix de prueba y
                  aplicar mantenimiento a su propia distribución (por ejemplo,
                  retirar archivos legacy de dist o procesar fixups). El prefix
                  de ZZZ no se usa.
                </label>
                <details>
                  <summary>Rutas de diagnóstico del test</summary>
                  <p>
                    Compatdata aislado:{" "}
                    <code className="path-value">
                      {bridgePanel.options.compatdata_path}
                    </code>
                  </p>
                  <p>{bridgePanel.options.warning}</p>
                </details>
                <button
                  className="primary-button"
                  disabled={
                    busy ||
                    !selected ||
                    !libraries ||
                    !bridgeRuntimePath ||
                    !bridgeSideEffectsAcknowledged ||
                    !bridgePanel.options.managed_runtime_available ||
                    bridgePanel.options.helper.state !== "available" ||
                    !bridgePanel.options.helper.integrity_matches
                  }
                  onClick={() => void runBridgeTest()}
                >
                  {busy ? "Ejecutando helper…" : "Run Runtime Bridge Test"}
                </button>
              </>
            )}
            {bridgeResult && (
              <div className="runtime-evidence" role="status">
                <strong>
                  Helper iniciado · handshake {bridgeResult.handshake} · path{" "}
                  {bridgeResult.path_visibility}
                </strong>
                <p>
                  {bridgeResult.explicit_runtime.display_name} · helper v
                  {bridgeResult.helper_version} · protocolo{" "}
                  {bridgeResult.helper_protocol_version} · exit{" "}
                  {bridgeResult.process.exit_code} ·{" "}
                  {bridgeResult.process.elapsed_millis} ms
                </p>
                <p>
                  Manifiesto runtime · SHA-256:{" "}
                  {bridgeResult.runtime_manifest_hash_matches
                    ? "coincide · "
                    : "no coincide · "}
                  <code className="path-value">
                    {bridgeResult.runtime_manifest_sha256}
                  </code>
                </p>
                <p>
                  Linux:{" "}
                  <code className="path-value">
                    {bridgeResult.runtime_path_linux}
                  </code>
                  <br />
                  Windows:{" "}
                  <code className="path-value">
                    {bridgeResult.runtime_path_windows}
                  </code>
                </p>
                <p>
                  Prefix aislado:{" "}
                  <code className="path-value">
                    {bridgeResult.compatdata_path}
                  </code>{" "}
                  · entorno propagado:{" "}
                  {bridgeResult.environment_marker_matches ? "sí" : "no"}
                </p>
                {bridgeResult.process.stderr && (
                  <details>
                    <summary>Diagnóstico stderr de Proton</summary>
                    <pre>{bridgeResult.process.stderr}</pre>
                  </details>
                )}
                <ul>
                  {bridgeResult.warnings.map((warning) => (
                    <li key={warning}>{warning}</li>
                  ))}
                </ul>
              </div>
            )}
          </details>
          <details className="runtime-evidence bridge-test-panel">
            <summary>Advanced · Loader Lab (solo proceso LXMI)</summary>
            <p>
              Prueba aislada con el export Direct Inject de 3dmloader.dll y una
              DLL inocua de LXMI. El runner crea un único
              lxmi-loader-test-target.exe y pasa al loader solo el PID de ese
              hijo. No acepta rutas ni procesos desde esta pantalla y no
              interactúa con ZZZ, Steam, HoYoPlay ni anti-cheat. Hook upstream
              instala un hook global y el backend lo bloquea porque no cumple el
              límite del proceso único del laboratorio.
            </p>
            <button
              className="secondary-button"
              disabled={busy}
              onClick={() => void inspectLoaderLab()}
            >
              {busy ? "Inspeccionando…" : "Inspeccionar Loader Lab"}
            </button>
            {loaderLabPanel && (
              <>
                <p>
                  Component license:{" "}
                  {loaderLabPanel.component
                    ? "release y firmas de componente verificadas; uso local del binario upstream"
                    : "no disponible"}
                  {loaderLabPanel.component && (
                    <>
                      {" · "}
                      {loaderLabPanel.component.release_tag} · commit{" "}
                      <code>{loaderLabPanel.component.commit}</code> · SHA-256{" "}
                      <code>{loaderLabPanel.component.loader_sha256}</code>
                    </>
                  )}
                </p>
                {loaderLabPanel.component_issue && (
                  <p className="discovery-note">
                    {loaderLabPanel.component_issue}
                  </p>
                )}
                <p>
                  Staging:{" "}
                  {loaderLabPanel.status.ready ? "preparado" : "pendiente"}
                  {" · "}
                  <code className="path-value">
                    {loaderLabPanel.status.test_root}
                  </code>
                </p>
                {!loaderLabPanel.status.ready && (
                  <>
                    {loaderLabPanel.status.missing_or_invalid.length > 0 && (
                      <ul>
                        {loaderLabPanel.status.missing_or_invalid.map(
                          (issue) => (
                            <li key={issue}>{issue}</li>
                          ),
                        )}
                      </ul>
                    )}
                    <button
                      className="secondary-button"
                      disabled={busy || !loaderLabPanel.component}
                      onClick={() => void prepareLoaderLab()}
                    >
                      {busy ? "Preparando…" : "Prepare Loader Lab"}
                    </button>
                    <p className="discovery-note">
                      Copia los ejecutables de test locales y el 3dmloader.dll
                      del package XXMI Libraries autenticado al storage LXMI. No
                      copia ningún archivo al juego.
                    </p>
                  </>
                )}
                {loaderLabPanel.status.ready && (
                  <>
                    <label htmlFor="loader-lab-runtime">
                      Bridge test Proton explícito
                    </label>
                    <select
                      id="loader-lab-runtime"
                      value={loaderRuntimePath}
                      disabled={busy || !loaderLabPanel.runtimes.length}
                      onChange={(event) =>
                        setLoaderRuntimePath(event.target.value)
                      }
                    >
                      <option value="">Elegir runtime de prueba…</option>
                      {loaderLabPanel.runtimes.map((runtime) => (
                        <option
                          key={runtime.proton_script}
                          value={runtime.proton_script}
                        >
                          {runtime.display_name} ·{" "}
                          {runtime.version ?? "versión desconocida"}
                        </option>
                      ))}
                    </select>
                    <label htmlFor="loader-lab-mode">Experimento</label>
                    <select
                      id="loader-lab-mode"
                      value={loaderLabMode}
                      disabled={busy}
                      onChange={(event) =>
                        setLoaderLabMode(event.target.value as LoaderLabMode)
                      }
                    >
                      <option value="baseline">
                        Baseline · target sin DLL
                      </option>
                      <option value="positive">
                        Direct Inject · DLL test LXMI
                      </option>
                      <option value="hook">
                        Hook · bloqueado por alcance global
                      </option>
                      <option value="missing_target">
                        Negativo · target ausente
                      </option>
                      <option value="missing_dll">
                        Negativo · DLL ausente
                      </option>
                      <option value="wrong_nonce">
                        Negativo · nonce distinto
                      </option>
                    </select>
                    {loaderLabMode === "hook" && (
                      <p className="discovery-note" role="status">
                        No se iniciará Proton. La DLL podría mapearse en otros
                        procesos antes de que [Loader]target filtre su
                        funcionalidad.
                      </p>
                    )}
                    <label className="discovery-note">
                      <input
                        type="checkbox"
                        checked={loaderSideEffectsAcknowledged}
                        disabled={busy || loaderLabMode === "hook"}
                        onChange={(event) =>
                          setLoaderSideEffectsAcknowledged(event.target.checked)
                        }
                      />{" "}
                      {loaderLabMode === "hook"
                        ? "El consentimiento no habilita Hook: el backend rechazará el modo por el límite de seguridad."
                        : "Confirmo que Proton puede inicializar/modificar únicamente el prefix aislado loader-v1 de LXMI, y que Direct Inject cargará la DLL test solo en el proceso temporal creado por este runner. No se usa el prefix de ZZZ ni se examinan otros procesos."}
                    </label>
                    <button
                      className="primary-button"
                      disabled={
                        busy ||
                        (loaderLabMode !== "hook" &&
                          (!loaderRuntimePath ||
                            !loaderSideEffectsAcknowledged)) ||
                        !loaderLabPanel.status.ready
                      }
                      onClick={() => void runLoaderLab()}
                    >
                      {busy
                        ? "Ejecutando experimento…"
                        : loaderLabMode === "hook"
                          ? "Comprobar bloqueo de seguridad"
                          : "Run Loader Compatibility Test"}
                    </button>
                  </>
                )}
              </>
            )}
            {loaderResult && (
              <div className="runtime-evidence" role="status">
                <strong>
                  {loaderResult.outcome === "passed"
                    ? "Experimento aprobado"
                    : "Rechazo negativo esperado"}{" "}
                  · {loaderResult.explicit_runtime.display_name}
                </strong>
                <p>
                  Target:{" "}
                  {loaderResult.target_started ? "iniciado" : "no iniciado"}
                  {" · "}ready: {loaderResult.target_ready ? "sí" : "no"}
                  {" · "}loader:{" "}
                  {loaderResult.loader_started ? "iniciado" : "no"}
                  {" · "}DLL: {loaderResult.dll_loaded ? "cargada" : "no"}
                  {" · "}marker:{" "}
                  {loaderResult.marker_verified ? "verificado" : "no"}
                  {" · "}nonce:{" "}
                  {loaderResult.nonce_verified ? "coincide" : "no coincide"}
                </p>
                <p>
                  Modo {loaderResult.loader_mode} · exit{" "}
                  {loaderResult.process.exit_code}
                  {" · "}
                  {loaderResult.process.elapsed_millis} ms · Proton solo usó el
                  prefix LXMI loader-v1.
                </p>
                <p>
                  Target Windows:{" "}
                  <code className="path-value">
                    {loaderResult.target_path_windows ?? "sin mapping"}
                  </code>
                  <br />
                  DLL Windows:{" "}
                  <code className="path-value">
                    {loaderResult.test_dll_path_windows ?? "sin mapping"}
                  </code>
                </p>
                <p>
                  Plan validado: sólo target, DLL y marker fijos de LXMI ·
                  timeout {loaderResult.plan.timeout_seconds}s.
                  <br />
                  Target Linux:{" "}
                  <code className="path-value">
                    {loaderResult.plan.target_executable}
                  </code>
                  <br />
                  DLL Linux:{" "}
                  <code className="path-value">
                    {loaderResult.plan.test_dll}
                  </code>
                </p>
                {loaderResult.marker && (
                  <p>
                    Proceso marker: {loaderResult.marker.process} · DLL visible
                    en{" "}
                    <code className="path-value">
                      {loaderResult.marker.dll_path_windows}
                    </code>
                  </p>
                )}
                <details>
                  <summary>Capacidades y diagnósticos</summary>
                  <ul>
                    {loaderResult.capabilities.map((item) => (
                      <li key={item.name}>
                        <strong>
                          {item.name}: {item.state}
                        </strong>{" "}
                        — {item.evidence}
                      </li>
                    ))}
                    {loaderResult.warnings.map((warning) => (
                      <li key={warning}>{warning}</li>
                    ))}
                  </ul>
                  {loaderResult.process.stderr && (
                    <pre>{loaderResult.process.stderr}</pre>
                  )}
                </details>
              </div>
            )}
          </details>
          <details className="runtime-evidence bridge-test-panel">
            <summary>Advanced · Runtime Observation de ZZZ</summary>
            <p>
              Si ZZZ ya está abierto, inicia la observación y LXMI lo buscará en
              el primer scan. Si todavía no está abierto, inícialo manualmente
              desde Steam mientras la sesión está activa. LXMI solo lee
              metadatos de procesos del usuario actual y un conjunto pequeño de
              variables allowlist; no lee memoria, no controla procesos y no
              inicia el juego. La sesión dura como máximo 90 segundos y no se
              guarda en disco.
            </p>
            <label className="observation-consent">
              <input
                type="checkbox"
                checked={observationConsent}
                disabled={observationBusy}
                onChange={(event) =>
                  setObservationConsent(event.target.checked)
                }
              />
              Confirmo que iniciaré Zenless Zone Zero manualmente desde Steam, o
              que ya está abierto, y acepto que LXMI observe sus metadatos de
              proceso durante esta sesión.
            </label>
            <div className="observation-actions">
              <button
                className="secondary-button"
                disabled={
                  observationBusy ||
                  !observationConsent ||
                  observationSessionActive
                }
                onClick={() => void startRuntimeObservation()}
              >
                {observationBusy
                  ? "Preparando observación…"
                  : "Start Observation"}
              </button>
              <button
                className="secondary-button"
                disabled={observationBusy || !observationSessionActive}
                onClick={() => void stopRuntimeObservation()}
              >
                Stop Observation
              </button>
            </div>
            {runtimeObservation ? (
              <div className="runtime-observation-result" role="status">
                <p>
                  Sesión {runtimeObservation.session_id} ·{" "}
                  <strong aria-live="polite">
                    {stateLabel(runtimeObservation.status)}
                  </strong>
                  {runtimeObservation.active && " · En curso"}
                  {` · ${Math.floor(runtimeObservation.elapsed_millis / 1000)} s`}
                  {runtimeObservation.process && (
                    <>
                      {" · "}
                      {runtimeObservation.process.executable_name} · PID{" "}
                      <code>{runtimeObservation.process.pid}</code>
                    </>
                  )}
                  {runtimeObservation.ambiguous_process_count > 0 &&
                    ` · ${runtimeObservation.ambiguous_process_count} procesos candidatos`}
                </p>
                {runtimeObservation.status === "waiting_for_game" && (
                  <p>
                    Scan en curso. Puede detectar un juego que ya estaba abierto
                    al comenzar; si no lo está, inicia Steam y pulsa Play tú
                    mismo.
                  </p>
                )}
                {runtimeObservation.status === "process_not_found" && (
                  <p>
                    La observación terminó sin encontrar un proceso de ZZZ
                    coincidente.
                  </p>
                )}
                {runtimeObservation.status === "timed_out" && (
                  <p>
                    La ventana terminó. Los datos parciales del último scan se
                    conservan abajo.
                  </p>
                )}
                <dl className="runtime-summary readiness-summary">
                  <div>
                    <dt>AppID observado</dt>
                    <dd>
                      {stateLabel(runtimeObservation.app_id_evidence.state)}
                      {runtimeObservation.app_id_evidence.observed_values.length
                        ? ` · ${runtimeObservation.app_id_evidence.observed_values.join(", ")}`
                        : " · sin valor en el entorno inspeccionado"}
                    </dd>
                  </div>
                  <div>
                    <dt>Mapping configurado por AppID</dt>
                    <dd>
                      {runtimeObservation.configured_runtime
                        .per_game_internal_id ??
                        stateLabel(
                          runtimeObservation.configured_runtime.per_game_state,
                        )}
                      {runtimeObservation.configured_runtime.global_internal_id
                        ? ` · default global ${runtimeObservation.configured_runtime.global_internal_id}`
                        : " · default global separado"}
                    </dd>
                  </div>
                  <div>
                    <dt>Proton observado en esta ejecución</dt>
                    <dd>
                      {runtimeObservation.observed_runtime.tool
                        ? `${runtimeObservation.observed_runtime.tool.display_name}${runtimeObservation.observed_runtime.tool.version ? ` · ${runtimeObservation.observed_runtime.tool.version}` : ""}`
                        : stateLabel(runtimeObservation.observed_runtime.state)}
                    </dd>
                  </div>
                  <div>
                    <dt>Proton instalado</dt>
                    <dd>
                      {runtimeObservation.available_proton_tools.length
                        ? runtimeObservation.available_proton_tools
                            .map(
                              (tool) =>
                                `${tool.display_name}${tool.version ? ` · ${tool.version}` : ""}`,
                            )
                            .join(", ")
                        : "No se detectaron candidatos válidos"}
                    </dd>
                  </div>
                  <div>
                    <dt>Compatdata efectivo</dt>
                    <dd>
                      {stateLabel(runtimeObservation.compatdata.state)}
                      {runtimeObservation.compatdata.observed_path && (
                        <>
                          <br />
                          <code className="path-value">
                            {runtimeObservation.compatdata.observed_path}
                          </code>
                        </>
                      )}
                    </dd>
                  </div>
                  <div>
                    <dt>Prefix efectivo</dt>
                    <dd>
                      {stateLabel(runtimeObservation.prefix.state)}
                      {runtimeObservation.prefix.observed_path && (
                        <>
                          <br />
                          <code className="path-value">
                            {runtimeObservation.prefix.observed_path}
                          </code>
                        </>
                      )}
                    </dd>
                  </div>
                  <div>
                    <dt>Wineserver</dt>
                    <dd>
                      {stateLabel(runtimeObservation.wineserver.state)} ·{" "}
                      {runtimeObservation.wineserver.evidence}
                    </dd>
                  </div>
                  <div>
                    <dt>Entorno allowlist · proceso y ascendencia</dt>
                    <dd>
                      {stateLabel(runtimeObservation.environment_evidence)}
                    </dd>
                  </div>
                  <div>
                    <dt>Topología observada</dt>
                    <dd>
                      {stateLabel(runtimeObservation.process_topology_state)}
                    </dd>
                  </div>
                  <div>
                    <dt>Scan procfs</dt>
                    <dd>
                      {
                        runtimeObservation.scan_diagnostics
                          .same_uid_processes_scanned
                      }{" "}
                      procesos del usuario ·{" "}
                      {
                        runtimeObservation.scan_diagnostics
                          .candidate_processes_considered
                      }{" "}
                      candidatos ·{" "}
                      {runtimeObservation.scan_diagnostics.accepted_candidates}{" "}
                      aceptados ·{" "}
                      {
                        runtimeObservation.scan_diagnostics
                          .rejected_insufficient_identity_evidence
                      }{" "}
                      descartados por evidencia insuficiente
                    </dd>
                  </div>
                </dl>
                {runtimeObservation.observed_runtime.evidence_source && (
                  <p>
                    Evidencia Proton:{" "}
                    {runtimeObservation.observed_runtime.evidence_source}
                  </p>
                )}
                {runtimeObservation.process && (
                  <p>
                    Identidad del proceso:{" "}
                    {runtimeObservation.process.identity_evidence
                      .map(stateLabel)
                      .join(" · ")}
                  </p>
                )}
                {runtimeObservation.process_topology.length > 0 && (
                  <details className="runtime-evidence">
                    <summary>Relaciones relevantes de proceso</summary>
                    <ul>
                      {runtimeObservation.process_topology.map((process) => (
                        <li key={`${process.pid}:${process.role}`}>
                          <strong>{stateLabel(process.role)}</strong> ·{" "}
                          {process.process_name} · PID {process.pid}
                          {process.parent_pid
                            ? ` · padre ${process.parent_pid}`
                            : ""}
                        </li>
                      ))}
                    </ul>
                  </details>
                )}
                {runtimeObservation.warnings.length > 0 && (
                  <ul className="observation-warnings">
                    {runtimeObservation.warnings.map((warning) => (
                      <li key={warning}>{warning}</li>
                    ))}
                  </ul>
                )}
                <p className="discovery-note">
                  La herramienta observada describe esta ejecución. El Proton
                  configurado, los candidatos disponibles y
                  GameRuntimePlan.selected siguen separados.
                </p>
                <button
                  className="secondary-button"
                  disabled={busy}
                  onClick={() => void inspectZzzReadiness()}
                >
                  Re-evaluar readiness con esta evidencia
                </button>
              </div>
            ) : (
              <p className="discovery-note">
                Aún no hay una sesión de observación. Se revisarán solamente
                procesos que coincidan con la instalación Steam detectada de
                ZZZ.
              </p>
            )}
          </details>
          <section
            className="observation-last-result"
            aria-live="off"
            aria-label="Resultado de la última observación de ZZZ"
          >
            <div className="observation-last-heading">
              Última observación de ZZZ
            </div>
            {observationError && (
              <p className="message-error" role="alert">
                <strong>Observation failed · {observationError.code}</strong>
                <br />
                {observationError.detail}
              </p>
            )}
            {runtimeObservation ? (
              <>
                <p>
                  <strong aria-live="polite">
                    {stateLabel(runtimeObservation.status)}
                  </strong>
                  {runtimeObservation.active && " · En curso"}
                  {` · ${Math.floor(runtimeObservation.elapsed_millis / 1000)} s`}
                </p>
                <dl className="observation-last-grid">
                  <div>
                    <dt>Proceso</dt>
                    <dd>
                      {runtimeObservation.process
                        ? `${runtimeObservation.process.executable_name} · PID ${runtimeObservation.process.pid}`
                        : runtimeObservation.status === "ambiguous_process"
                          ? `${runtimeObservation.ambiguous_process_count} procesos candidatos`
                          : "No identificado"}
                    </dd>
                  </div>
                  <div>
                    <dt>AppID</dt>
                    <dd>
                      {stateLabel(runtimeObservation.app_id_evidence.state)}
                      {runtimeObservation.app_id_evidence.observed_values.length
                        ? ` · ${runtimeObservation.app_id_evidence.observed_values.join(", ")} · LOCAL OBSERVED`
                        : ""}
                    </dd>
                  </div>
                  <div>
                    <dt>Proton observado</dt>
                    <dd>
                      {runtimeObservation.observed_runtime.tool
                        ? `${runtimeObservation.observed_runtime.tool.display_name}${runtimeObservation.observed_runtime.tool.version ? ` · ${runtimeObservation.observed_runtime.tool.version}` : ""} · LOCAL OBSERVED`
                        : stateLabel(runtimeObservation.observed_runtime.state)}
                    </dd>
                  </div>
                  <div>
                    <dt>Compatdata</dt>
                    <dd>
                      {stateLabel(runtimeObservation.compatdata.state)}
                      {["observed_and_matches", "observed_different"].includes(
                        runtimeObservation.compatdata.state,
                      ) && " · LOCAL OBSERVED"}
                      {runtimeObservation.compatdata.observed_path && (
                        <>
                          <br />
                          <code className="path-value">
                            {runtimeObservation.compatdata.observed_path}
                          </code>
                        </>
                      )}
                    </dd>
                  </div>
                  <div>
                    <dt>Prefix</dt>
                    <dd>
                      {stateLabel(runtimeObservation.prefix.state)}
                      {runtimeObservation.prefix.state ===
                      "derived_from_observed_compatdata"
                        ? " · DERIVED"
                        : runtimeObservation.prefix.state === "observed"
                          ? " · LOCAL OBSERVED"
                          : ""}
                      {runtimeObservation.prefix.observed_path && (
                        <>
                          <br />
                          <code className="path-value">
                            {runtimeObservation.prefix.observed_path}
                          </code>
                        </>
                      )}
                    </dd>
                  </div>
                  <div>
                    <dt>Topología · wineserver</dt>
                    <dd>
                      {stateLabel(runtimeObservation.process_topology_state)} ·{" "}
                      {stateLabel(runtimeObservation.wineserver.state)}
                    </dd>
                  </div>
                  <div>
                    <dt>Scan</dt>
                    <dd>
                      {
                        runtimeObservation.scan_diagnostics
                          .same_uid_processes_scanned
                      }{" "}
                      procesos ·{" "}
                      {
                        runtimeObservation.scan_diagnostics
                          .candidate_processes_considered
                      }{" "}
                      candidatos ·{" "}
                      {runtimeObservation.scan_diagnostics.accepted_candidates}{" "}
                      aceptados
                    </dd>
                  </div>
                </dl>
                {runtimeObservation.environment_evidence ===
                  "permission_denied" && (
                  <p className="discovery-note observation-diagnostics">
                    Linux denegó la lectura allowlist de <code>environ</code> en
                    el proceso observado o uno de sus ascendientes. El contador
                    de abajo mide lecturas denegadas de candidatos directos y
                    puede ser cero aunque un ascendiente no sea legible.
                  </p>
                )}
                {runtimeObservation.warnings.length > 0 && (
                  <ul className="observation-warnings">
                    {runtimeObservation.warnings.slice(0, 3).map((warning) => (
                      <li key={warning}>{warning}</li>
                    ))}
                  </ul>
                )}
                {runtimeObservation.environment_evidence ===
                  "permission_denied" && (
                  <p className="discovery-note observation-diagnostics">
                    Linux denegó la lectura allowlist de <code>environ</code> en
                    el proceso observado o uno de sus ascendientes. El contador
                    de candidatos directos no incluye todos los procesos de la
                    ascendencia.
                  </p>
                )}
                {(runtimeObservation.scan_diagnostics
                  .rejected_executable_mismatch > 0 ||
                  runtimeObservation.scan_diagnostics
                    .rejected_insufficient_identity_evidence > 0 ||
                  runtimeObservation.scan_diagnostics
                    .candidate_metadata_unavailable > 0 ||
                  runtimeObservation.scan_diagnostics
                    .candidate_environments_permission_denied > 0 ||
                  runtimeObservation.scan_diagnostics
                    .accepted_contexts_environment_unavailable > 0 ||
                  runtimeObservation.scan_diagnostics
                    .accepted_contexts_environment_permission_denied > 0) && (
                  <p className="discovery-note observation-diagnostics">
                    Candidatos directos:{" "}
                    {
                      runtimeObservation.scan_diagnostics
                        .rejected_executable_mismatch
                    }{" "}
                    nombre no coincide ·{" "}
                    {
                      runtimeObservation.scan_diagnostics
                        .rejected_insufficient_identity_evidence
                    }{" "}
                    evidencia insuficiente ·{" "}
                    {
                      runtimeObservation.scan_diagnostics
                        .candidate_metadata_unavailable
                    }{" "}
                    metadatos no disponibles ·{" "}
                    {
                      runtimeObservation.scan_diagnostics
                        .candidate_environments_permission_denied
                    }{" "}
                    entorno no legible · contextos aceptados con entorno no
                    disponible:{" "}
                    {
                      runtimeObservation.scan_diagnostics
                        .accepted_contexts_environment_unavailable
                    }{" "}
                    · con permiso denegado:{" "}
                    {
                      runtimeObservation.scan_diagnostics
                        .accepted_contexts_environment_permission_denied
                    }
                    .
                  </p>
                )}
              </>
            ) : (
              <p className="discovery-note">
                Aún no se ha iniciado una observación. El resultado permanecerá
                aquí al terminar, incluso si el panel avanzado está plegado.
              </p>
            )}
          </section>
          <details className="runtime-evidence bridge-test-panel">
            <summary>Zenless Zone Zero · Launch Readiness</summary>
            <p>
              Assessment declarativo. El Proton elegido en Loader Lab no se
              considera el runtime seleccionado para ZZZ. Esta vista no inicia
              el juego ni modifica Steam, compatdata o prefix.
            </p>
            <button
              className="secondary-button"
              disabled={busy}
              onClick={() => void inspectZzzReadiness()}
            >
              {busy ? "Evaluando…" : "Evaluar readiness de ZZZ"}
            </button>
            {zzzReadiness && (
              <div className="runtime-evidence" role="status">
                <h3>{zzzReadiness.game_name}</h3>
                <p>
                  Readiness:{" "}
                  <strong>
                    {readinessDecisionLabels[zzzReadiness.decision] ??
                      stateLabel(zzzReadiness.decision)}
                  </strong>{" "}
                  · AppID <code>{zzzReadiness.app_id}</code> · distribución{" "}
                  {stateLabel(zzzReadiness.distribution)}
                </p>
                <dl className="runtime-summary readiness-summary">
                  <div>
                    <dt>Estrategia LXMI</dt>
                    <dd>
                      {loaderOutcomeLabels[
                        zzzReadiness.loader_strategy.outcome
                      ] ?? stateLabel(zzzReadiness.loader_strategy.outcome)}
                      {zzzReadiness.loader_strategy.preferred_mode
                        ? ` · ${zzzReadiness.loader_strategy.preferred_mode}`
                        : " · sin modo preferido"}
                    </dd>
                  </div>
                  <div>
                    <dt>Hook · Loader Lab</dt>
                    <dd>
                      {stateLabel(
                        zzzReadiness.capabilities.find(
                          (item) => item.capability === "hook_in_isolated_lab",
                        )?.state ?? "unknown",
                      )}
                      {" · aislamiento por proceso no demostrado"}
                    </dd>
                  </div>
                  <div>
                    <dt>Direct Inject · Loader Lab</dt>
                    <dd>
                      {stateLabel(
                        zzzReadiness.capabilities.find(
                          (item) =>
                            item.capability === "direct_inject_in_isolated_lab",
                        )?.state ?? "unknown",
                      )}
                      {" · solo fixture LXMI"}
                    </dd>
                  </div>
                  <div>
                    <dt>ZZMI con ZZZ</dt>
                    <dd>
                      {stateLabel(
                        zzzReadiness.capabilities.find(
                          (item) =>
                            item.capability ===
                            "game_specific_loader_compatibility",
                        )?.state ?? "unknown",
                      )}
                    </dd>
                  </div>
                </dl>
                <p>
                  ZZMI usa Hook por defecto en el flujo estándar. Direct Inject
                  aparece en Custom Launch upstream, pero su prueba de
                  laboratorio no demuestra compatibilidad con ZZMI/ZZZ. Proton
                  seleccionado para el juego:{" "}
                  <strong>
                    {zzzReadiness.selected_game_runtime ?? "Desconocido"}
                  </strong>
                  . No se inició el juego.
                </p>
                <p>
                  Instalación {zzzReadiness.installation} · contenido{" "}
                  {zzzReadiness.game_content_completeness} · ejecutable{" "}
                  {zzzReadiness.executable}
                </p>
                <p>
                  Compatdata {zzzReadiness.compatdata} · prefix{" "}
                  {zzzReadiness.prefix}
                  <br />
                  Runtime administrado {zzzReadiness.managed_runtime}
                  {zzzReadiness.zzmi_version &&
                    ` · ZZMI ${zzzReadiness.zzmi_version}`}
                  {zzzReadiness.xxmi_libraries_version &&
                    ` · XXMI Libraries ${zzzReadiness.xxmi_libraries_version}`}
                </p>
                <p>
                  Proton seleccionado para el juego:{" "}
                  {zzzReadiness.selected_game_runtime ?? "Unknown"}
                  <br />
                  Proton explícito de laboratorio:{" "}
                  {zzzReadiness.explicit_lab_runtime ?? "no registrado"}
                </p>
                <p>
                  Herramientas Proton disponibles:{" "}
                  {zzzReadiness.available_proton_tools.length
                    ? zzzReadiness.available_proton_tools.join(", ")
                    : "ninguna detectada"}
                </p>
                {zzzReadiness.steam_process_context && (
                  <>
                    <p>
                      Contexto Steam / Proton · mapping por juego:{" "}
                      <strong>
                        {zzzReadiness.steam_process_context.per_game_mapping
                          ?.internal_name ||
                          stateLabel(
                            zzzReadiness.steam_process_context
                              .per_game_mapping_state,
                          )}
                      </strong>{" "}
                      · contexto efectivo:{" "}
                      {stateLabel(
                        zzzReadiness.steam_process_context
                          .effective_runtime_selection,
                      )}
                    </p>
                    <details>
                      <summary>Evidencia del contexto Steam / Proton</summary>
                      <p>
                        Steam root:{" "}
                        <code className="path-value">
                          {zzzReadiness.steam_process_context.steam_root ??
                            "no asociado"}
                        </code>
                        <br />
                        Biblioteca:{" "}
                        <code className="path-value">
                          {zzzReadiness.steam_process_context.library_root ??
                            "no detectada"}
                        </code>
                        <br />
                        Manifest:{" "}
                        <code className="path-value">
                          {zzzReadiness.steam_process_context
                            .app_manifest_path ?? "no detectado"}
                        </code>
                        <br />
                        config.vdf:{" "}
                        {stateLabel(
                          zzzReadiness.steam_process_context.config_state,
                        )}
                        {zzzReadiness.steam_process_context.config_path && (
                          <>
                            {" · "}
                            <code className="path-value">
                              {zzzReadiness.steam_process_context.config_path}
                            </code>
                          </>
                        )}
                        {zzzReadiness.steam_process_context.config_issue && (
                          <>
                            <br />
                            {zzzReadiness.steam_process_context.config_issue}
                          </>
                        )}
                        <br />
                        Configuración por AppID:{" "}
                        {zzzReadiness.steam_process_context.per_game_mapping
                          ? `tool id ${zzzReadiness.steam_process_context.per_game_mapping.internal_name || "no informado"}`
                          : stateLabel(
                              zzzReadiness.steam_process_context
                                .per_game_mapping_state,
                            )}
                        <br />
                        Default global:{" "}
                        {zzzReadiness.steam_process_context
                          .global_default_mapping?.internal_name ??
                          "no registrado"}
                        {" · no se promueve a selección del juego."}
                        <br />
                        Runtime efectivo:{" "}
                        {stateLabel(
                          zzzReadiness.steam_process_context
                            .effective_runtime_selection,
                        )}
                        <br />
                        Prefix efectivo:{" "}
                        {stateLabel(
                          zzzReadiness.steam_process_context.effective_prefix,
                        )}
                        <br />
                        Loader comparte prefix:{" "}
                        {stateLabel(
                          zzzReadiness.steam_process_context
                            .loader_same_prefix_requirement,
                        )}
                        <br />
                        Loader comparte wineserver:{" "}
                        {stateLabel(
                          zzzReadiness.steam_process_context
                            .loader_same_wineserver_requirement,
                        )}
                        <br />
                        Relación con wineserver:{" "}
                        {stateLabel(
                          zzzReadiness.steam_process_context
                            .wineserver_relationship,
                        )}
                      </p>
                      <h4>Compatibility tools instaladas</h4>
                      {zzzReadiness.steam_process_context
                        .installed_compatibility_tools.length > 0 ? (
                        <ul>
                          {zzzReadiness.steam_process_context.installed_compatibility_tools.map(
                            (tool) => (
                              <li key={`${tool.internal_id}:${tool.path}`}>
                                <strong>{tool.display_name}</strong> ·{" "}
                                {stateLabel(tool.kind)} ·{" "}
                                {stateLabel(tool.origin)}
                                {tool.version ? ` · ${tool.version}` : ""}
                                {tool.internal_id
                                  ? ` · ${tool.internal_id}`
                                  : ""}
                                <br />
                                <code className="path-value">{tool.path}</code>
                              </li>
                            ),
                          )}
                        </ul>
                      ) : (
                        <p>No se detectaron compatibility tools.</p>
                      )}
                      {zzzReadiness.steam_process_context
                        .matched_installed_tools.length > 0 && (
                        <p>
                          Herramienta configurada con coincidencia local:{" "}
                          {zzzReadiness.steam_process_context.matched_installed_tools
                            .map((tool) => tool.display_name)
                            .join(", ")}
                          . Este dato procede de config.vdf, no de la sesión
                          observada.
                        </p>
                      )}
                      <ul>
                        {zzzReadiness.steam_process_context.findings.map(
                          (finding, index) => (
                            <li key={`${finding.claim}:${index}`}>
                              <strong>
                                {finding.claim}: {stateLabel(finding.state)}
                              </strong>{" "}
                              — {finding.detail} <code>{finding.source}</code>
                            </li>
                          ),
                        )}
                      </ul>
                      {zzzReadiness.steam_process_context
                        .runtime_observation && (
                        <p>
                          Sesión observada:{" "}
                          {stateLabel(
                            zzzReadiness.steam_process_context
                              .runtime_observation.status,
                          )}
                          {" · "}
                          Proton en esa sesión:{" "}
                          {zzzReadiness.steam_process_context
                            .runtime_observation.runtime.tool?.display_name ??
                            stateLabel(
                              zzzReadiness.steam_process_context
                                .runtime_observation.runtime.state,
                            )}
                          . Esta evidencia es temporal y no modifica la
                          selección configurada ni el planner.
                        </p>
                      )}
                    </details>
                  </>
                )}
                <details>
                  <summary>Estrategia de loader para ZZMI</summary>
                  <p>
                    Default upstream:{" "}
                    <strong>
                      {stateLabel(zzzReadiness.upstream_default_loader_mode)}
                    </strong>
                    <br />
                    Configuración upstream:{" "}
                    {stateLabel(zzzReadiness.upstream_loader_strategy)}
                    <br />
                    Requisito técnico de Hook para ZZMI:{" "}
                    {stateLabel(zzzReadiness.hook_requirement)}
                    <br />
                    Decisión LXMI Linux/ZZZ:{" "}
                    <strong>
                      {loaderOutcomeLabels[
                        zzzReadiness.loader_strategy.outcome
                      ] ?? stateLabel(zzzReadiness.loader_strategy.outcome)}
                    </strong>
                    <br />
                    Preferencia seleccionada:{" "}
                    {zzzReadiness.loader_strategy.preferred_mode ?? "ninguna"}
                    <br />
                    Direct Inject es candidato de investigación:{" "}
                    {zzzReadiness.loader_strategy.direct_inject_candidate
                      ? "sí, sin compatibilidad ZZMI/ZZZ verificada"
                      : "no"}
                  </p>
                  <h4>Criterios Direct Inject</h4>
                  <ul>
                    {zzzReadiness.loader_strategy.direct_inject_criteria.map(
                      (item) => (
                        <li key={item.id}>
                          <strong>
                            {item.id.replaceAll("_", " ")}: {item.state}
                          </strong>{" "}
                          — {item.evidence}
                        </li>
                      ),
                    )}
                  </ul>
                  <h4>Alcance de aislamiento</h4>
                  <ul>
                    {zzzReadiness.loader_strategy.isolation.map((item) => (
                      <li key={item.mode}>
                        <strong>
                          {item.mode}: {item.scope.replaceAll("_", " ")} ·{" "}
                          {item.state}
                        </strong>{" "}
                        — {item.evidence} {item.limitation}
                      </li>
                    ))}
                  </ul>
                  <h4>Conclusión y límites</h4>
                  <ul>
                    {zzzReadiness.loader_strategy.evidence.map((item) => (
                      <li key={item}>{item}</li>
                    ))}
                    {zzzReadiness.loader_strategy.known_limitations.map(
                      (item) => (
                        <li key={item}>{item}</li>
                      ),
                    )}
                  </ul>
                </details>
                <ul>
                  {zzzReadiness.capabilities.map((item) => (
                    <li key={item.capability}>
                      <strong>
                        {item.capability.replace(/_/g, " ")}: {item.state}
                      </strong>{" "}
                      — {item.evidence}
                    </li>
                  ))}
                </ul>
                {zzzReadiness.blockers.length > 0 && (
                  <details>
                    <summary>Blockers</summary>
                    <ul>
                      {zzzReadiness.blockers.map((item) => (
                        <li key={item}>{item}</li>
                      ))}
                    </ul>
                  </details>
                )}
                {zzzReadiness.unknowns.length > 0 && (
                  <details>
                    <summary>Evidencia pendiente</summary>
                    <ul>
                      {zzzReadiness.unknowns.map((item) => (
                        <li key={item}>{item}</li>
                      ))}
                    </ul>
                  </details>
                )}
                <details>
                  <summary>Requisitos y rutas técnicas</summary>
                  <ul>
                    {zzzReadiness.requirements.map((item) => (
                      <li key={item.id}>
                        <strong>
                          {item.id}: {item.state}
                        </strong>{" "}
                        — {item.evidence}
                      </li>
                    ))}
                  </ul>
                  <p>
                    Juego:{" "}
                    <code className="path-value">
                      {zzzReadiness.game_install_path ?? "no detectado"}
                    </code>
                    <br />
                    Ejecutable:{" "}
                    <code className="path-value">
                      {zzzReadiness.game_executable_path ?? "no detectado"}
                    </code>
                    <br />
                    Compatdata:{" "}
                    <code className="path-value">
                      {zzzReadiness.compatdata_path ?? "no detectado"}
                    </code>
                    <br />
                    Prefix:{" "}
                    <code className="path-value">
                      {zzzReadiness.prefix_path ?? "no detectado"}
                    </code>
                    <br />
                    Runtime LXMI:{" "}
                    <code className="path-value">
                      {zzzReadiness.managed_runtime_root ?? "no detectado"}
                    </code>
                  </p>
                </details>
                <p>
                  Gate: {zzzReadiness.controlled_experiment_gate.decision} ·
                  elegible para revisión:{" "}
                  {zzzReadiness.controlled_experiment_gate.eligible_for_review
                    ? "sí"
                    : "no"}
                  · consentimiento explícito requerido:{" "}
                  {zzzReadiness.controlled_experiment_gate
                    .explicit_user_consent_required
                    ? "sí"
                    : "no"}
                  · consentimiento recibido:{" "}
                  {zzzReadiness.controlled_experiment_gate.consent_received
                    ? "sí"
                    : "no"}
                  · lanzó juego:{" "}
                  {zzzReadiness.controlled_experiment_gate.launches_game
                    ? "sí"
                    : "no"}
                </p>
                <p>{zzzReadiness.controlled_experiment_gate.rationale}</p>
                <details>
                  <summary>Componentes del gate</summary>
                  <ul>
                    {zzzReadiness.controlled_experiment_gate.components.map(
                      (item) => (
                        <li key={item.component}>
                          <strong>
                            {item.component.replaceAll("_", " ")}: {item.state}
                          </strong>{" "}
                          — {item.evidence}
                        </li>
                      ),
                    )}
                  </ul>
                </details>
              </div>
            )}
          </details>
        </div>
      )}
      <p id="xxmi-error" className="message-error" role="alert">
        {error}
      </p>
      <p className="message" role="status">
        {notice}
      </p>
      {plan && (
        <div className="runtime-assessment">
          <h3>Mapping histórico 0.5.2 · no es target activo</h3>
          <p>
            {plan.assessment.planning_possible
              ? "El paquete, las bibliotecas y el registro del juego permiten revisar un plan técnico. Esto no confirma compatibilidad ni habilita instalación."
              : "Falta evidencia o una dependencia para construir el plan. Revisa los requisitos pendientes."}
          </p>
          <p>
            Distribución: {plan.assessment.distribution} · Steam:{" "}
            {plan.assessment.steam_support} · Linux/Proton:{" "}
            {plan.assessment.linux_proton_support}
          </p>
          <p>
            <strong>Compatibilidad verificada: no.</strong>
          </p>
          <p>
            Staging histórico propuesto (no configuración activa):{" "}
            <code className="path-value">{plan.managed_target}</code>
          </p>
          <details className="runtime-evidence">
            <summary>Requisitos y pasos aún no ejecutables</summary>
            <ul>
              {plan.assessment.requirements.map((r) => (
                <li key={r.name}>
                  <strong>
                    {r.name}: {states[r.state]}
                  </strong>{" "}
                  — {r.evidence}
                </li>
              ))}
            </ul>
            <ul>
              {plan.configuration_changes.map((s) => (
                <li key={s}>{s}</li>
              ))}
            </ul>
          </details>
          <details className="runtime-evidence">
            <summary>
              {plan.deployment_mapping.length} rutas relativas históricas; sin
              comparación con el juego
            </summary>
            <p>
              Root configurado:{" "}
              {plan.dry_run.configured_target_root ?? "no capturado"} ·
              candidato comparado:{" "}
              <code className="path-value">
                {plan.dry_run.comparison_root_candidate ?? "sin candidato"}
              </code>
            </p>
            <p>{plan.dry_run.comparison_root_evidence}</p>
            <p>
              Safety: {plan.dry_run.safety} · apply permitido: no · escrituras
              realizadas: no.
            </p>
            <ul>
              {plan.dry_run.files.map((file) => (
                <li
                  key={`${file.source_package_id}:${file.target_relative_path}`}
                >
                  <strong>{file.status.replaceAll("_", " ")}</strong> ·{" "}
                  <code>{file.target_relative_path}</code>
                  <div>
                    Comparación:{" "}
                    <code className="path-value">
                      {file.comparison_path ?? "sin ruta"}
                    </code>
                  </div>
                  {file.existing_sha256 && (
                    <div>
                      Hash existente: <code>{file.existing_sha256}</code>
                    </div>
                  )}
                  <div>{file.evidence}</div>
                </li>
              ))}
            </ul>
          </details>
          <details className="runtime-evidence">
            <summary>{plan.files.length} archivos propuestos y backups</summary>
            <ul>
              {plan.files.map((f) => (
                <li key={f.target}>
                  <strong>
                    {f.action === "create"
                      ? "Crear"
                      : f.action === "unchanged"
                        ? "Conservar"
                        : "Reemplazar con backup"}
                  </strong>
                  <div>
                    Fuente: <code>{f.source}</code>
                  </div>
                  <div>
                    Destino: <code>{f.target}</code>
                  </div>
                </li>
              ))}
            </ul>
          </details>
          <ul className="xxmi-warnings">
            {plan.warnings.map((w) => (
              <li key={w}>{w}</li>
            ))}
          </ul>
        </div>
      )}
    </section>
  );
}
