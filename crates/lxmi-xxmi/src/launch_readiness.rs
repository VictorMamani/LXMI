//! Evidence-based readiness for a future, explicitly consented ZZZ experiment.
//! This module only evaluates facts; it never launches a process or changes files.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessEvidenceState {
    Verified,
    Missing,
    Incomplete,
    Invalid,
    Unreadable,
    Unknown,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessCapabilityState {
    Verified,
    UpstreamDocumented,
    Unverified,
    NotTested,
    Unsupported,
    Unknown,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZzmiLoaderMode {
    Hook,
    DirectInject,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpstreamLoaderStrategy {
    HookOnly,
    DirectInjectOnly,
    EitherDependingOnLaunchMode,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoaderStrategyOutcome {
    Hook,
    DirectInject,
    NoDecision,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookRequirementState {
    Established,
    NotEstablished,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoaderIsolationScope {
    ProcessBound,
    PrefixBound,
    WineServerBound,
    WineDesktopWithinServer,
    SessionWide,
    HostWide,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoaderCriterionState {
    Satisfied,
    Unsatisfied,
    Unknown,
    NotEstablished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZzmiGateComponent {
    PackageAuthenticity,
    ManagedRuntime,
    LoaderMechanism,
    LoaderIsolation,
    GameRuntimeContext,
    SteamLaunchTopology,
    ProcessIdentification,
    SteamProcessContext,
    PrefixContext,
    WineServerContext,
    UserConsent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SteamCompatConfigEvidenceState {
    NotInspected,
    Read,
    Missing,
    Invalid,
    Unreadable,
    UnsafePath,
    TooLarge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerGameToolMappingState {
    Configured,
    NoPerGameOverride,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeContextKnowledge {
    Known,
    Unknown,
    NotRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextToolKind {
    Proton,
    SteamLinuxRuntime,
    Other,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextToolOrigin {
    SteamLibrary,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextFindingState {
    Detected,
    Configured,
    Absent,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfiguredToolMappingEvidence {
    pub app_id: u32,
    pub internal_name: Option<String>,
    pub config: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledCompatibilityToolEvidence {
    pub internal_id: Option<String>,
    pub display_name: String,
    pub kind: ContextToolKind,
    pub origin: ContextToolOrigin,
    pub version: Option<String>,
    pub path: std::path::PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SteamProcessContextFinding {
    pub claim: String,
    pub state: ContextFindingState,
    pub source: String,
    pub detail: String,
}

/// Adds per-session process observations to readiness context without changing
/// the configured Steam mapping or the planner's runtime selection.
pub fn incorporate_runtime_observation(
    context: &mut SteamProtonProcessContextEvidence,
    observation: &lxmi_runtime::RuntimeObservationSummary,
) {
    use lxmi_runtime::{ObservationEvidenceState, RuntimeObservationStatus};

    context.runtime_observation = Some(observation.clone());
    if !observation.process_observed
        || observation.status == RuntimeObservationStatus::AmbiguousProcess
    {
        return;
    }

    context.process_context = RuntimeContextKnowledge::Known;
    if observation.app_id.state == ObservationEvidenceState::Verified
        && observation.runtime.state == ObservationEvidenceState::Verified
    {
        context.effective_runtime_selection = RuntimeContextKnowledge::Known;
    }
    if matches!(
        observation.prefix.state,
        ObservationEvidenceState::Observed
            | ObservationEvidenceState::DerivedFromObservedCompatdata
    ) {
        context.effective_prefix = RuntimeContextKnowledge::Known;
    }
    // A matching WINEPREFIX environment is useful observation, but it does not
    // prove which wineserver instance the game communicates with.

    let source = format!(
        "LXMI passive observation session at {} ms",
        observation.observed_at_unix_millis.unwrap_or_default()
    );
    context.findings.push(SteamProcessContextFinding {
        claim: "Steam/Proton/ZZZ runtime observation".into(),
        state: ContextFindingState::Detected,
        source: source.clone(),
        detail: "Ephemeral evidence from a uniquely identified process started manually by the user; this does not change configured Steam mapping or loader strategy.".into(),
    });
    context.findings.push(SteamProcessContextFinding {
        claim: "Runtime observed for this execution".into(),
        state: if observation.runtime.state == ObservationEvidenceState::Verified {
            ContextFindingState::Detected
        } else {
            ContextFindingState::Unknown
        },
        source: observation
            .runtime
            .evidence_source
            .clone()
            .unwrap_or_else(|| source.clone()),
        detail: "Observation applies only to this session. GameRuntimePlan selection and configured Steam mapping remain separate.".into(),
    });
    context.findings.push(SteamProcessContextFinding {
        claim: "compatdata observed".into(),
        state: if observation.compatdata.state == ObservationEvidenceState::ObservedAndMatches {
            ContextFindingState::Detected
        } else {
            ContextFindingState::Unknown
        },
        source: observation
            .compatdata
            .evidence_source
            .clone()
            .unwrap_or_else(|| source.clone()),
        detail: "The allowlisted process environment is compared with the manifest-derived compatdata path; no files are changed.".into(),
    });
    context.findings.push(SteamProcessContextFinding {
        claim: "Prefix observed or derived".into(),
        state: if matches!(
            observation.prefix.state,
            ObservationEvidenceState::Observed
                | ObservationEvidenceState::DerivedFromObservedCompatdata
        ) {
            ContextFindingState::Detected
        } else {
            ContextFindingState::Unknown
        },
        source: observation
            .prefix
            .evidence_source
            .clone()
            .unwrap_or_else(|| source.clone()),
        detail: "A prefix derived from STEAM_COMPAT_DATA_PATH remains explicitly derived; it says nothing about the loader's required context.".into(),
    });
}

/// Read-only observations. A configured mapping is not represented as the runtime
/// Steam actually launched; effective context remains independently unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SteamProtonProcessContextEvidence {
    pub app_id: u32,
    pub steam_root: Option<std::path::PathBuf>,
    pub library_root: Option<std::path::PathBuf>,
    pub app_manifest_path: Option<std::path::PathBuf>,
    pub compatdata_state: ReadinessEvidenceState,
    pub compatdata_path: Option<std::path::PathBuf>,
    pub prefix_state: ReadinessEvidenceState,
    pub prefix_path: Option<std::path::PathBuf>,
    pub config_path: Option<std::path::PathBuf>,
    pub config_state: SteamCompatConfigEvidenceState,
    pub config_issue: Option<String>,
    pub per_game_mapping_state: PerGameToolMappingState,
    pub per_game_mapping: Option<ConfiguredToolMappingEvidence>,
    pub global_default_mapping: Option<ConfiguredToolMappingEvidence>,
    pub matched_installed_tools: Vec<InstalledCompatibilityToolEvidence>,
    pub installed_compatibility_tools: Vec<InstalledCompatibilityToolEvidence>,
    pub effective_runtime_selection: RuntimeContextKnowledge,
    pub effective_prefix: RuntimeContextKnowledge,
    pub process_context: RuntimeContextKnowledge,
    pub loader_same_prefix_requirement: RuntimeContextKnowledge,
    pub loader_same_wineserver_requirement: RuntimeContextKnowledge,
    pub wineserver_relationship: RuntimeContextKnowledge,
    /// Ephemeral evidence from a user-started, read-only observation session.
    pub runtime_observation: Option<lxmi_runtime::RuntimeObservationSummary>,
    pub findings: Vec<SteamProcessContextFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoaderStrategyCriterion {
    pub id: String,
    pub state: LoaderCriterionState,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoaderIsolationEvidence {
    pub mode: ZzmiLoaderMode,
    pub scope: LoaderIsolationScope,
    pub state: ReadinessCapabilityState,
    pub evidence: String,
    pub limitation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoaderStrategyDecision {
    pub game_id: String,
    pub platform: String,
    pub upstream_strategy: UpstreamLoaderStrategy,
    pub outcome: LoaderStrategyOutcome,
    pub preferred_mode: Option<ZzmiLoaderMode>,
    pub hook_requirement: HookRequirementState,
    /// A candidate is a route worth investigating, not a compatibility claim or selection.
    pub direct_inject_candidate: bool,
    pub direct_inject_criteria: Vec<LoaderStrategyCriterion>,
    pub isolation: Vec<LoaderIsolationEvidence>,
    pub evidence: Vec<String>,
    pub known_limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZzmiGateComponentAssessment {
    pub component: ZzmiGateComponent,
    pub state: ZzmiRequirementState,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZzmiReadinessDecision {
    NotReady,
    NeedsEvidence,
    ReadyForControlledExperiment,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZzmiRequirementState {
    Satisfied,
    Unsatisfied,
    Unknown,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZzmiCapability {
    ExplicitLabProtonRuntime,
    DirectInjectInIsolatedLab,
    HookInIsolatedLab,
    RemoteDllViaDirectInject,
    RemoteDllViaHook,
    LoaderIdentitySemantics,
    ManagedRuntimePathViaProton,
    NativeSteamLaunchUpstream,
    DirectSteamLaunchZzz,
    ZzzSteamUnderLinuxProton,
    HookVisibilityInSteamContext,
    SamePrefixRequirement,
    GameSpecificLoaderCompatibility,
    ZzzProcessIdentificationInProton,
    UpstreamDirectInjectCustomLaunch,
    UpstreamGenericDirectInject,
    LoaderComponentLocalUse,
    DirectInjectProcessLifecycleInProton,
    ZzzTargetProcessMapping,
    AntiCheatBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZzmiCapabilityAssessment {
    pub capability: ZzmiCapability,
    pub state: ReadinessCapabilityState,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZzmiReadinessRequirement {
    pub id: String,
    pub state: ZzmiRequirementState,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlledGameExperimentGate {
    pub decision: ZzmiReadinessDecision,
    pub eligible_for_review: bool,
    pub explicit_user_consent_required: bool,
    pub consent_received: bool,
    pub launches_game: bool,
    pub rationale: String,
    pub components: Vec<ZzmiGateComponentAssessment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZzmiLaunchReadiness {
    pub game_id: String,
    pub game_name: String,
    pub app_id: u32,
    pub distribution: ReadinessCapabilityState,
    pub installation: ReadinessEvidenceState,
    pub game_content_completeness: ReadinessEvidenceState,
    pub executable: ReadinessEvidenceState,
    pub compatdata: ReadinessEvidenceState,
    pub prefix: ReadinessEvidenceState,
    pub zzmi_package: ReadinessEvidenceState,
    pub zzmi_authenticity: ReadinessEvidenceState,
    pub xxmi_libraries_package: ReadinessEvidenceState,
    pub xxmi_libraries_authenticity: ReadinessEvidenceState,
    pub managed_runtime: ReadinessEvidenceState,
    pub game_runtime_selection: ReadinessCapabilityState,
    pub selected_game_runtime: Option<String>,
    pub available_proton_tools: Vec<String>,
    pub explicit_lab_runtime: Option<String>,
    pub game_install_path: Option<std::path::PathBuf>,
    pub game_executable_path: Option<std::path::PathBuf>,
    pub compatdata_path: Option<std::path::PathBuf>,
    pub prefix_path: Option<std::path::PathBuf>,
    pub steam_process_context: Option<SteamProtonProcessContextEvidence>,
    pub managed_runtime_root: Option<std::path::PathBuf>,
    pub integration_version: Option<String>,
    pub xxmi_libraries_version: Option<String>,
    pub upstream_default_loader_mode: ZzmiLoaderMode,
    pub upstream_loader_strategy: UpstreamLoaderStrategy,
    pub hook_requirement: HookRequirementState,
    pub loader_strategy: LoaderStrategyDecision,
    pub capabilities: Vec<ZzmiCapabilityAssessment>,
    pub requirements: Vec<ZzmiReadinessRequirement>,
    pub blockers: Vec<String>,
    pub unknowns: Vec<String>,
    pub decision: ZzmiReadinessDecision,
    pub controlled_experiment_gate: ControlledGameExperimentGate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZzmiLaunchReadinessInput {
    pub game_id: String,
    pub game_name: String,
    pub app_id: u32,
    pub distribution: ReadinessCapabilityState,
    pub installation: ReadinessEvidenceState,
    pub game_content_completeness: ReadinessEvidenceState,
    pub executable: ReadinessEvidenceState,
    pub compatdata: ReadinessEvidenceState,
    pub prefix: ReadinessEvidenceState,
    pub zzmi_package: ReadinessEvidenceState,
    pub zzmi_authenticity: ReadinessEvidenceState,
    pub xxmi_libraries_package: ReadinessEvidenceState,
    pub xxmi_libraries_authenticity: ReadinessEvidenceState,
    pub managed_runtime: ReadinessEvidenceState,
    pub game_runtime_selection: ReadinessCapabilityState,
    pub selected_game_runtime: Option<String>,
    pub available_proton_tools: Vec<String>,
    pub explicit_lab_runtime: Option<String>,
    pub game_install_path: Option<std::path::PathBuf>,
    pub game_executable_path: Option<std::path::PathBuf>,
    pub compatdata_path: Option<std::path::PathBuf>,
    pub prefix_path: Option<std::path::PathBuf>,
    pub steam_process_context: Option<SteamProtonProcessContextEvidence>,
    pub managed_runtime_root: Option<std::path::PathBuf>,
    pub integration_version: Option<String>,
    pub xxmi_libraries_version: Option<String>,
    pub upstream_default_loader_mode: ZzmiLoaderMode,
    pub upstream_loader_strategy: UpstreamLoaderStrategy,
    pub hook_requirement: HookRequirementState,
    pub capabilities: Vec<ZzmiCapabilityAssessment>,
}

pub fn assess_loader_strategy(
    game_id: &str,
    platform: &str,
    upstream_strategy: UpstreamLoaderStrategy,
    hook_requirement: HookRequirementState,
    capabilities: &[ZzmiCapabilityAssessment],
) -> LoaderStrategyDecision {
    let capability = |name| {
        capabilities
            .iter()
            .find(|item| item.capability == name)
            .map(|item| item.state)
            .unwrap_or(ReadinessCapabilityState::Unknown)
    };
    let criterion = |id: &str, state, evidence: &str| LoaderStrategyCriterion {
        id: id.into(),
        state,
        evidence: evidence.into(),
    };
    let upstream_route_state = match upstream_strategy {
        UpstreamLoaderStrategy::DirectInjectOnly
        | UpstreamLoaderStrategy::EitherDependingOnLaunchMode => LoaderCriterionState::Satisfied,
        UpstreamLoaderStrategy::HookOnly => LoaderCriterionState::Unsatisfied,
        UpstreamLoaderStrategy::Unknown => LoaderCriterionState::Unknown,
    };
    let upstream_generic_state = match capability(ZzmiCapability::UpstreamGenericDirectInject) {
        ReadinessCapabilityState::Verified | ReadinessCapabilityState::UpstreamDocumented => {
            LoaderCriterionState::Satisfied
        }
        ReadinessCapabilityState::Unsupported | ReadinessCapabilityState::Blocked => {
            LoaderCriterionState::Unsatisfied
        }
        ReadinessCapabilityState::Unverified
        | ReadinessCapabilityState::NotTested
        | ReadinessCapabilityState::Unknown => LoaderCriterionState::Unknown,
    };
    let upstream_direct_state = if upstream_route_state == LoaderCriterionState::Satisfied
        && upstream_generic_state == LoaderCriterionState::Satisfied
    {
        LoaderCriterionState::Satisfied
    } else if upstream_route_state == LoaderCriterionState::Unsatisfied
        || upstream_generic_state == LoaderCriterionState::Unsatisfied
    {
        LoaderCriterionState::Unsatisfied
    } else {
        LoaderCriterionState::Unknown
    };
    let lab_direct_state =
        criterion_from_capability(capability(ZzmiCapability::DirectInjectInIsolatedLab));
    let remote_direct_state =
        criterion_from_capability(capability(ZzmiCapability::RemoteDllViaDirectInject));
    let component_local_use_state =
        criterion_from_capability(capability(ZzmiCapability::LoaderComponentLocalUse));
    let direct_inject_candidate = upstream_direct_state == LoaderCriterionState::Satisfied
        && lab_direct_state == LoaderCriterionState::Satisfied
        && remote_direct_state == LoaderCriterionState::Satisfied
        && component_local_use_state == LoaderCriterionState::Satisfied
        && hook_requirement != HookRequirementState::Established;

    let direct_inject_criteria = vec![
        criterion(
            "upstream_direct_inject_mode",
            upstream_direct_state,
            "XXMI Launcher v2.2.1 includes a generic Direct Inject path for Custom Launch; ZZMI keeps Hook as the default when Custom Launch is disabled.",
        ),
        criterion(
            "xxmi_libraries_generic_direct_inject",
            upstream_generic_state,
            "MigotoInjector uses the same 3dmloader.dll and generic Inject path for the integration class; this documents upstream capability, not ZZZ compatibility.",
        ),
        criterion(
            "loader_component_local_use",
            component_local_use_state,
            "The authenticated 3dmloader.dll component was reviewed for local laboratory use; this evidence does not authorize redistribution.",
        ),
        criterion(
            "hook_exclusive_requirement",
            match hook_requirement {
                HookRequirementState::Established => LoaderCriterionState::Unsatisfied,
                HookRequirementState::NotEstablished => LoaderCriterionState::NotEstablished,
                HookRequirementState::Unknown => LoaderCriterionState::Unknown,
            },
            "No upstream evidence of an exclusive Hook requirement for ZZMI was found. NotEstablished does not prove that Hook is unnecessary.",
        ),
        criterion(
            "direct_inject_component_in_lab",
            lab_direct_state,
            "The 3dmloader.dll Direct Inject path loaded the test DLL into the fixed LXMI process under Proton.",
        ),
        criterion(
            "remote_dll_in_lab",
            remote_direct_state,
            "The remote test DLL was visible and verified the marker and nonce in the LXMI target.",
        ),
        criterion(
            "zzmi_game_specific_compatibility",
            criterion_from_capability(capability(ZzmiCapability::GameSpecificLoaderCompatibility)),
            "Neither ZZMI nor ZZZ has been run with Direct Inject; the ZZMI-specific requirement remains unestablished.",
        ),
        criterion(
            "loader_identity_for_linux_runtime",
            criterion_from_capability(capability(ZzmiCapability::LoaderIdentitySemantics)),
            "The loader= configuration is documented for the upstream launcher; the identity of a loader run by LXMI under Proton has not been validated.",
        ),
        criterion(
            "steam_launch_context_under_proton",
            criterion_from_capability(capability(ZzmiCapability::ZzzSteamUnderLinuxProton)),
            "The reviewed Native Steam Launch documentation is for Windows and does not prove Steam Linux/Proton support.",
        ),
        criterion(
            "process_identification_in_proton",
            criterion_from_capability(capability(
                ZzmiCapability::ZzzProcessIdentificationInProton,
            )),
            "Upstream finds the process by name; its visibility in the actual ZZZ Steam/Proton context has not been tested.",
        ),
        criterion(
            "target_process_mapping",
            criterion_from_capability(capability(ZzmiCapability::ZzzTargetProcessMapping)),
            "The target name is confirmed by the upstream integration and local executable; resolving it as a Windows process from Steam/Proton still requires evidence.",
        ),
        criterion(
            "process_lifecycle_in_proton",
            criterion_from_capability(capability(
                ZzmiCapability::DirectInjectProcessLifecycleInProton,
            )),
            "The laboratory test created its own process; it does not prove LXMI can control the lifecycle of a target started by Steam/Proton.",
        ),
        criterion(
            "same_prefix_or_wineserver_context",
            criterion_from_capability(capability(ZzmiCapability::SamePrefixRequirement)),
            "The prefix/wineserver relationship required for ZZZ target detection has not been established.",
        ),
        criterion(
            "anti_cheat_boundary",
            criterion_from_capability(capability(ZzmiCapability::AntiCheatBoundary)),
            "The game has not been run and anti-cheat has not been studied or interacted with.",
        ),
    ];

    let direct_production_evidence_complete = [
        ZzmiCapability::GameSpecificLoaderCompatibility,
        ZzmiCapability::LoaderIdentitySemantics,
        ZzmiCapability::ZzzSteamUnderLinuxProton,
        ZzmiCapability::ZzzProcessIdentificationInProton,
        ZzmiCapability::DirectInjectProcessLifecycleInProton,
        ZzmiCapability::ZzzTargetProcessMapping,
        ZzmiCapability::SamePrefixRequirement,
        ZzmiCapability::AntiCheatBoundary,
    ]
    .into_iter()
    .all(|item| capability(item) == ReadinessCapabilityState::Verified);

    let hook_lab_verified = capability(ZzmiCapability::HookInIsolatedLab)
        == ReadinessCapabilityState::Verified
        && capability(ZzmiCapability::RemoteDllViaHook) == ReadinessCapabilityState::Verified;
    let hook_isolation_proven =
        hook_lab_verified && hook_requirement == HookRequirementState::Established;
    let outcome = match (hook_requirement, upstream_strategy) {
        (HookRequirementState::Established, _) if hook_isolation_proven => {
            LoaderStrategyOutcome::Hook
        }
        (HookRequirementState::Established, _) => LoaderStrategyOutcome::Blocked,
        (_, UpstreamLoaderStrategy::HookOnly) if hook_isolation_proven => {
            LoaderStrategyOutcome::Hook
        }
        (_, UpstreamLoaderStrategy::HookOnly) => LoaderStrategyOutcome::Blocked,
        (_, UpstreamLoaderStrategy::Unknown) => LoaderStrategyOutcome::NoDecision,
        (_, UpstreamLoaderStrategy::DirectInjectOnly)
        | (_, UpstreamLoaderStrategy::EitherDependingOnLaunchMode)
            if direct_inject_candidate && direct_production_evidence_complete =>
        {
            LoaderStrategyOutcome::DirectInject
        }
        (_, UpstreamLoaderStrategy::DirectInjectOnly)
        | (_, UpstreamLoaderStrategy::EitherDependingOnLaunchMode)
            if !direct_inject_candidate
                && matches!(lab_direct_state, LoaderCriterionState::Unsatisfied) =>
        {
            LoaderStrategyOutcome::Blocked
        }
        _ => LoaderStrategyOutcome::NoDecision,
    };
    let preferred_mode = match outcome {
        LoaderStrategyOutcome::Hook => Some(ZzmiLoaderMode::Hook),
        LoaderStrategyOutcome::DirectInject => Some(ZzmiLoaderMode::DirectInject),
        LoaderStrategyOutcome::NoDecision | LoaderStrategyOutcome::Blocked => None,
    };

    let direct_state = capability(ZzmiCapability::DirectInjectInIsolatedLab);
    let isolation = vec![
        LoaderIsolationEvidence {
            mode: ZzmiLoaderMode::Hook,
            scope: LoaderIsolationScope::WineDesktopWithinServer,
            state: ReadinessCapabilityState::UpstreamDocumented,
            evidence: "XXMI describes its Hook as global. In the reviewed Wine implementation, SetWindowsHookEx without a target thread registers the hook in the current desktop global table managed by wineserver; different WINEPREFIX values allow independent Wine sessions.".into(),
            limitation: "The Wine scope is not a Proton HOST TEST and does not prove the internal behavior of 3dmloader.dll. A desktop hook can reach other processes on the same desktop, so it does not satisfy the Hook Lab single-target guard.".into(),
        },
        LoaderIsolationEvidence {
            mode: ZzmiLoaderMode::DirectInject,
            scope: LoaderIsolationScope::ProcessBound,
            state: direct_state,
            evidence: "In LXMI 0.7, the harness passed only the PID of its own test child to the Inject export of 3dmloader.dll.".into(),
            limitation: "The normal upstream flow finds the game process by name; the owned-PID test does not verify ZZZ identification in Steam/Proton.".into(),
        },
    ];

    let evidence = vec![
        "In XXMI Launcher v2.2.1, standard launch inherits use_hook=true from ModelImporterPackage; ZZMI does not override it.".into(),
        "With Custom Launch enabled, custom_launch_inject_mode == Hook selects Hook; Inject selects Direct Inject and Bypass skips d3d11.dll unless it is added as an extra library.".into(),
        "The ZZMI configuration does not establish an exclusive technical requirement for Hook. Absence of that requirement alone does not prove Direct Inject compatibility with ZZZ.".into(),
    ];
    let known_limitations = vec![
        "Direct Inject has only been tested with the LXMI laboratory target and DLL, not with ZZMI/ZZZ.".into(),
        "The explicit laboratory Proton is not the Steam Proton selection for the game.".into(),
        "It has not been determined whether the helper process, Steam, and ZZZ must share a prefix or wineserver.".into(),
        "The Hook Lab policy still rejects Hook before resolving a runtime or starting processes.".into(),
    ];

    LoaderStrategyDecision {
        game_id: game_id.into(),
        platform: platform.into(),
        upstream_strategy,
        outcome,
        preferred_mode,
        hook_requirement,
        direct_inject_candidate,
        direct_inject_criteria,
        isolation,
        evidence,
        known_limitations,
    }
}

fn criterion_from_capability(state: ReadinessCapabilityState) -> LoaderCriterionState {
    match state {
        ReadinessCapabilityState::Verified => LoaderCriterionState::Satisfied,
        ReadinessCapabilityState::Unsupported | ReadinessCapabilityState::Blocked => {
            LoaderCriterionState::Unsatisfied
        }
        ReadinessCapabilityState::UpstreamDocumented
        | ReadinessCapabilityState::Unverified
        | ReadinessCapabilityState::NotTested
        | ReadinessCapabilityState::Unknown => LoaderCriterionState::Unknown,
    }
}

pub fn assess_zzmi_launch_readiness(input: ZzmiLaunchReadinessInput) -> ZzmiLaunchReadiness {
    let mut requirements = Vec::new();
    let mut blockers = Vec::new();
    let mut unknowns = Vec::new();

    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "game_installation",
        input.installation,
        "The game installation must be present and recognized.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "game_content_completeness",
        input.game_content_completeness,
        "The presence of the directory or executable alone does not prove that all game data is complete.",
    );
    push_capability_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "steam_distribution",
        input.distribution,
        "The Steam distribution must be identified; HoYoPlay and manual installations are separate paths.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "expected_executable",
        input.executable,
        "The expected Zenless Zone Zero executable must exist.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "compatdata",
        input.compatdata,
        "The compatdata location must be identified.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "game_prefix",
        input.prefix,
        "The game prefix state must be known; readiness assessment does not initialize it.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "zzmi_package",
        input.zzmi_package,
        "The ZZMI package must be available and validated.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "zzmi_authenticity",
        input.zzmi_authenticity,
        "ZZMI provenance and authenticity must be verified.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "xxmi_libraries_package",
        input.xxmi_libraries_package,
        "XXMI Libraries is a separate ZZMI dependency.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "xxmi_libraries_authenticity",
        input.xxmi_libraries_authenticity,
        "XXMI Libraries provenance and authenticity must be verified.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "managed_runtime",
        input.managed_runtime,
        "The assembled LXMI-managed runtime must pass integrity checks.",
    );
    push_capability_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "game_proton_selection",
        input.game_runtime_selection,
        "The Proton selected by Steam for ZZZ cannot be inferred from the laboratory runtime.",
    );
    if let Some(context) = &input.steam_process_context {
        push_capability_requirement(
            &mut requirements,
            &mut blockers,
            &mut unknowns,
            "effective_game_proton_context",
            context_knowledge_capability(context.effective_runtime_selection),
            "The local mapping may indicate a configuration, but does not observe the runtime Steam used for this session.",
        );
        push_capability_requirement(
            &mut requirements,
            &mut blockers,
            &mut unknowns,
            "effective_game_prefix_context",
            context_knowledge_capability(context.effective_prefix),
            "compatdata/pfx is an observed candidate path; it does not prove the effective prefix of a running process.",
        );
        push_capability_requirement(
            &mut requirements,
            &mut blockers,
            &mut unknowns,
            "steam_process_context",
            context_knowledge_capability(context.process_context),
            "No Steam/Proton/ZZZ processes were started or inspected in this increment.",
        );
        push_capability_requirement(
            &mut requirements,
            &mut blockers,
            &mut unknowns,
            "loader_same_prefix_requirement",
            context_knowledge_capability(context.loader_same_prefix_requirement),
            "No evidence determines whether a future loader must share the ZZZ prefix.",
        );
        push_capability_requirement(
            &mut requirements,
            &mut blockers,
            &mut unknowns,
            "loader_same_wineserver_requirement",
            context_knowledge_capability(context.loader_same_wineserver_requirement),
            "No evidence determines whether a future loader must share a wineserver with ZZZ.",
        );
        push_capability_requirement(
            &mut requirements,
            &mut blockers,
            &mut unknowns,
            "wineserver_relationship",
            context_knowledge_capability(context.wineserver_relationship),
            "No wineserver or Wine/Proton processes were observed.",
        );
    }
    let loader_strategy = assess_loader_strategy(
        &input.game_id,
        "linux_proton",
        input.upstream_loader_strategy,
        input.hook_requirement,
        &input.capabilities,
    );
    match loader_strategy.outcome {
        LoaderStrategyOutcome::Hook => {
            push_capability_requirement(
                &mut requirements,
                &mut blockers,
                &mut unknowns,
                "selected_loader_mode_in_lab",
                ReadinessCapabilityState::Verified,
                "The selected Hook mode has been tested in the laboratory.",
            );
        }
        LoaderStrategyOutcome::DirectInject => {
            push_capability_requirement(
                &mut requirements,
                &mut blockers,
                &mut unknowns,
                "selected_loader_mode_in_lab",
                ReadinessCapabilityState::Verified,
                "The selected Direct Inject mode has been tested in the laboratory.",
            );
        }
        LoaderStrategyOutcome::NoDecision => {
            push_capability_requirement(
                &mut requirements,
                &mut blockers,
                &mut unknowns,
                "loader_strategy_for_zzmi_linux",
                ReadinessCapabilityState::Unknown,
                "Upstream provides Hook by default and Direct Inject through Custom Launch; specific ZZMI/ZZZ/Linux compatibility has not been established.",
            );
        }
        LoaderStrategyOutcome::Blocked => {
            push_capability_requirement(
                &mut requirements,
                &mut blockers,
                &mut unknowns,
                "loader_strategy_for_zzmi_linux",
                ReadinessCapabilityState::Blocked,
                "No loader strategy satisfies the security policy with the available evidence.",
            );
        }
    }

    for (capability, id, detail) in [
        (
            ZzmiCapability::HookVisibilityInSteamContext,
            "hook_steam_context",
            "Hook visibility from the ZZZ Steam/Proton launch context has not been tested.",
        ),
        (
            ZzmiCapability::SamePrefixRequirement,
            "same_prefix_requirement",
            "It has not been determined whether the loader and ZZZ must share a Proton prefix or context.",
        ),
        (
            ZzmiCapability::GameSpecificLoaderCompatibility,
            "game_specific_loader_compatibility",
            "The laboratory test does not prove loader compatibility with ZZZ.",
        ),
        (
            ZzmiCapability::ZzzProcessIdentificationInProton,
            "process_identification_in_proton",
            "ZZZ target process identification from the loader context under Proton has not been tested.",
        ),
        (
            ZzmiCapability::AntiCheatBoundary,
            "anti_cheat_boundary",
            "It has not been verified that the game flow can stay within the boundary without interacting with anti-cheat.",
        ),
    ] {
        let state = input
            .capabilities
            .iter()
            .find(|item| item.capability == capability)
            .map(|item| item.state)
            .unwrap_or(ReadinessCapabilityState::Unknown);
        push_capability_requirement(
            &mut requirements,
            &mut blockers,
            &mut unknowns,
            id,
            state,
            detail,
        );
    }

    for (capability, id, detail) in [
        (
            ZzmiCapability::ZzzSteamUnderLinuxProton,
            "steam_zzz_launch_context",
            "The Native Steam Launch guide does not prove the actual ZZZ Steam Linux/Proton context.",
        ),
        (
            ZzmiCapability::NativeSteamLaunchUpstream,
            "native_steam_launch_topology",
            "The documented upstream ZZZ path describes Steam on Windows; its Linux/Proton topology requires separate evidence.",
        ),
    ] {
        let state = input
            .capabilities
            .iter()
            .find(|item| item.capability == capability)
            .map(|item| item.state)
            .unwrap_or(ReadinessCapabilityState::Unknown);
        push_capability_requirement(
            &mut requirements,
            &mut blockers,
            &mut unknowns,
            id,
            state,
            detail,
        );
    }

    let decision = if requirements
        .iter()
        .any(|item| item.state == ZzmiRequirementState::Blocked)
    {
        ZzmiReadinessDecision::Blocked
    } else if requirements
        .iter()
        .any(|item| item.state == ZzmiRequirementState::Unsatisfied)
    {
        ZzmiReadinessDecision::NotReady
    } else if requirements
        .iter()
        .any(|item| item.state == ZzmiRequirementState::Unknown)
    {
        ZzmiReadinessDecision::NeedsEvidence
    } else {
        ZzmiReadinessDecision::ReadyForControlledExperiment
    };
    let gate_components = build_gate_components(&requirements, &loader_strategy);
    let controlled_experiment_gate = ControlledGameExperimentGate {
        decision,
        eligible_for_review: decision == ZzmiReadinessDecision::ReadyForControlledExperiment,
        explicit_user_consent_required: true,
        consent_received: false,
        launches_game: false,
        components: gate_components,
        rationale: match decision {
            ZzmiReadinessDecision::NotReady => {
                "Observed requirements are missing; this assessment does not run the game.".into()
            }
            ZzmiReadinessDecision::NeedsEvidence => {
                "Some requirements are unknown; this assessment does not run the game.".into()
            }
            ZzmiReadinessDecision::ReadyForControlledExperiment => {
                "The evidence would permit reviewing an experiment; explicit consent and a separate implementation would still be required.".into()
            }
            ZzmiReadinessDecision::Blocked => {
                "A security restriction or requirement blocks experiment review.".into()
            }
        },
    };

    ZzmiLaunchReadiness {
        game_id: input.game_id,
        game_name: input.game_name,
        app_id: input.app_id,
        distribution: input.distribution,
        installation: input.installation,
        game_content_completeness: input.game_content_completeness,
        executable: input.executable,
        compatdata: input.compatdata,
        prefix: input.prefix,
        zzmi_package: input.zzmi_package,
        zzmi_authenticity: input.zzmi_authenticity,
        xxmi_libraries_package: input.xxmi_libraries_package,
        xxmi_libraries_authenticity: input.xxmi_libraries_authenticity,
        managed_runtime: input.managed_runtime,
        game_runtime_selection: input.game_runtime_selection,
        selected_game_runtime: input.selected_game_runtime,
        available_proton_tools: input.available_proton_tools,
        explicit_lab_runtime: input.explicit_lab_runtime,
        game_install_path: input.game_install_path,
        game_executable_path: input.game_executable_path,
        compatdata_path: input.compatdata_path,
        prefix_path: input.prefix_path,
        steam_process_context: input.steam_process_context,
        managed_runtime_root: input.managed_runtime_root,
        integration_version: input.integration_version,
        xxmi_libraries_version: input.xxmi_libraries_version,
        upstream_default_loader_mode: input.upstream_default_loader_mode,
        upstream_loader_strategy: input.upstream_loader_strategy,
        hook_requirement: input.hook_requirement,
        loader_strategy,
        capabilities: input.capabilities,
        requirements,
        blockers,
        unknowns,
        decision,
        controlled_experiment_gate,
    }
}

fn context_knowledge_capability(knowledge: RuntimeContextKnowledge) -> ReadinessCapabilityState {
    match knowledge {
        RuntimeContextKnowledge::Known => ReadinessCapabilityState::Verified,
        RuntimeContextKnowledge::Unknown => ReadinessCapabilityState::Unknown,
        RuntimeContextKnowledge::NotRequired => ReadinessCapabilityState::Unsupported,
    }
}

fn build_gate_components(
    requirements: &[ZzmiReadinessRequirement],
    strategy: &LoaderStrategyDecision,
) -> Vec<ZzmiGateComponentAssessment> {
    let state_for = |id: &str| {
        requirements
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.state)
            .unwrap_or(ZzmiRequirementState::Unknown)
    };
    let combine = |ids: &[&str]| {
        let states: Vec<_> = ids.iter().map(|id| state_for(id)).collect();
        if states.contains(&ZzmiRequirementState::Blocked) {
            ZzmiRequirementState::Blocked
        } else if states.contains(&ZzmiRequirementState::Unsatisfied) {
            ZzmiRequirementState::Unsatisfied
        } else if states.contains(&ZzmiRequirementState::Unknown) {
            ZzmiRequirementState::Unknown
        } else {
            ZzmiRequirementState::Satisfied
        }
    };
    let loader_mechanism_state = match strategy.outcome {
        LoaderStrategyOutcome::Hook | LoaderStrategyOutcome::DirectInject => {
            ZzmiRequirementState::Satisfied
        }
        LoaderStrategyOutcome::NoDecision => ZzmiRequirementState::Unknown,
        LoaderStrategyOutcome::Blocked => ZzmiRequirementState::Blocked,
    };
    let loader_isolation = strategy
        .preferred_mode
        .and_then(|mode| strategy.isolation.iter().find(|item| item.mode == mode));
    let (loader_isolation_state, loader_isolation_evidence) = match loader_isolation {
        Some(item)
            if item.state == ReadinessCapabilityState::Verified
                && item.scope == LoaderIsolationScope::ProcessBound =>
        {
            (ZzmiRequirementState::Satisfied, item.evidence.clone())
        }
        Some(item) if item.state == ReadinessCapabilityState::Blocked => {
            (ZzmiRequirementState::Blocked, item.limitation.clone())
        }
        Some(item) => (
            ZzmiRequirementState::Unknown,
            format!("{} {}", item.evidence, item.limitation),
        ),
        None => (
            ZzmiRequirementState::Unknown,
            "No strategy is preferred. The observed Hook has desktop scope within wineserver and has not been proven process-isolated; Direct Inject for ZZMI/ZZZ is also unverified.".into(),
        ),
    };

    vec![
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::PackageAuthenticity,
            state: combine(&[
                "zzmi_package",
                "zzmi_authenticity",
                "xxmi_libraries_package",
                "xxmi_libraries_authenticity",
            ]),
            evidence: "ZZMI and XXMI Libraries must be present and retain verified provenance.".into(),
        },
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::ManagedRuntime,
            state: state_for("managed_runtime"),
            evidence: "The assembled runtime is inspected in read-only mode.".into(),
        },
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::LoaderMechanism,
            state: loader_mechanism_state,
            evidence: format!(
                "Upstream: {:?}; LXMI strategy for {}: {:?}.",
                strategy.upstream_strategy, strategy.platform, strategy.outcome
            ),
        },
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::LoaderIsolation,
            state: loader_isolation_state,
            evidence: loader_isolation_evidence,
        },
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::GameRuntimeContext,
            state: combine(&[
                "game_proton_selection",
                "effective_game_proton_context",
            ]),
            evidence: "The ZZZ Proton selection requires separate evidence; the laboratory runtime is not inherited.".into(),
        },
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::SteamLaunchTopology,
            state: combine(&[
                "steam_distribution",
                "native_steam_launch_topology",
                "steam_zzz_launch_context",
            ]),
            evidence: "The documented Native Steam Launch path does not prove Steam Linux/Proton support.".into(),
        },
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::ProcessIdentification,
            state: state_for("process_identification_in_proton"),
            evidence: "ZZZ target identification from the loader in Proton has not been tested.".into(),
        },
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::SteamProcessContext,
            state: state_for("steam_process_context"),
            evidence: "The effective execution context cannot be inferred from files or installed runtimes.".into(),
        },
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::PrefixContext,
            state: combine(&[
                "compatdata",
                "game_prefix",
                "same_prefix_requirement",
                "effective_game_prefix_context",
                "loader_same_prefix_requirement",
            ]),
            evidence: "Detected compatdata/pfx does not determine whether the loader and game must share a prefix or wineserver.".into(),
        },
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::WineServerContext,
            state: combine(&[
                "wineserver_relationship",
                "loader_same_wineserver_requirement",
            ]),
            evidence: "The effective wineserver and its relationship to a future loader have not been inspected.".into(),
        },
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::UserConsent,
            state: ZzmiRequirementState::Unknown,
            evidence: "This read-only assessment does not collect consent; any future experiment will require consent before execution.".into(),
        },
    ]
}

fn push_evidence_requirement(
    requirements: &mut Vec<ZzmiReadinessRequirement>,
    blockers: &mut Vec<String>,
    unknowns: &mut Vec<String>,
    id: &str,
    state: ReadinessEvidenceState,
    evidence: &str,
) {
    let requirement_state = match state {
        ReadinessEvidenceState::Verified => ZzmiRequirementState::Satisfied,
        ReadinessEvidenceState::Missing | ReadinessEvidenceState::Incomplete => {
            ZzmiRequirementState::Unsatisfied
        }
        ReadinessEvidenceState::Invalid | ReadinessEvidenceState::Unreadable => {
            ZzmiRequirementState::Blocked
        }
        ReadinessEvidenceState::Unknown => ZzmiRequirementState::Unknown,
        ReadinessEvidenceState::Blocked => ZzmiRequirementState::Blocked,
    };
    record_requirement(
        requirements,
        blockers,
        unknowns,
        id,
        requirement_state,
        evidence,
    );
}

fn push_capability_requirement(
    requirements: &mut Vec<ZzmiReadinessRequirement>,
    blockers: &mut Vec<String>,
    unknowns: &mut Vec<String>,
    id: &str,
    state: ReadinessCapabilityState,
    evidence: &str,
) {
    let requirement_state = match state {
        ReadinessCapabilityState::Verified => ZzmiRequirementState::Satisfied,
        ReadinessCapabilityState::Unsupported => ZzmiRequirementState::Unsatisfied,
        ReadinessCapabilityState::Blocked => ZzmiRequirementState::Blocked,
        ReadinessCapabilityState::UpstreamDocumented
        | ReadinessCapabilityState::Unverified
        | ReadinessCapabilityState::NotTested
        | ReadinessCapabilityState::Unknown => ZzmiRequirementState::Unknown,
    };
    record_requirement(
        requirements,
        blockers,
        unknowns,
        id,
        requirement_state,
        evidence,
    );
}

fn record_requirement(
    requirements: &mut Vec<ZzmiReadinessRequirement>,
    blockers: &mut Vec<String>,
    unknowns: &mut Vec<String>,
    id: &str,
    state: ZzmiRequirementState,
    evidence: &str,
) {
    match state {
        ZzmiRequirementState::Blocked => blockers.push(format!("{id}: {evidence}")),
        ZzmiRequirementState::Unknown => unknowns.push(format!("{id}: {evidence}")),
        ZzmiRequirementState::Satisfied | ZzmiRequirementState::Unsatisfied => {}
    }
    requirements.push(ZzmiReadinessRequirement {
        id: id.into(),
        state,
        evidence: evidence.into(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capabilities(hook_state: ReadinessCapabilityState) -> Vec<ZzmiCapabilityAssessment> {
        [
            (
                ZzmiCapability::DirectInjectInIsolatedLab,
                ReadinessCapabilityState::Verified,
            ),
            (ZzmiCapability::HookInIsolatedLab, hook_state),
            (
                ZzmiCapability::RemoteDllViaDirectInject,
                ReadinessCapabilityState::Verified,
            ),
            (ZzmiCapability::RemoteDllViaHook, hook_state),
            (
                ZzmiCapability::ManagedRuntimePathViaProton,
                ReadinessCapabilityState::Verified,
            ),
            (
                ZzmiCapability::LoaderIdentitySemantics,
                ReadinessCapabilityState::UpstreamDocumented,
            ),
            (
                ZzmiCapability::NativeSteamLaunchUpstream,
                ReadinessCapabilityState::UpstreamDocumented,
            ),
            (
                ZzmiCapability::DirectSteamLaunchZzz,
                ReadinessCapabilityState::Unsupported,
            ),
            (
                ZzmiCapability::ZzzSteamUnderLinuxProton,
                ReadinessCapabilityState::Unverified,
            ),
            (
                ZzmiCapability::HookVisibilityInSteamContext,
                ReadinessCapabilityState::Unknown,
            ),
            (
                ZzmiCapability::SamePrefixRequirement,
                ReadinessCapabilityState::Unknown,
            ),
            (
                ZzmiCapability::GameSpecificLoaderCompatibility,
                ReadinessCapabilityState::NotTested,
            ),
            (
                ZzmiCapability::ZzzProcessIdentificationInProton,
                ReadinessCapabilityState::Unknown,
            ),
            (
                ZzmiCapability::UpstreamDirectInjectCustomLaunch,
                ReadinessCapabilityState::UpstreamDocumented,
            ),
            (
                ZzmiCapability::UpstreamGenericDirectInject,
                ReadinessCapabilityState::UpstreamDocumented,
            ),
            (
                ZzmiCapability::LoaderComponentLocalUse,
                ReadinessCapabilityState::Verified,
            ),
            (
                ZzmiCapability::DirectInjectProcessLifecycleInProton,
                ReadinessCapabilityState::Unknown,
            ),
            (
                ZzmiCapability::ZzzTargetProcessMapping,
                ReadinessCapabilityState::Verified,
            ),
            (
                ZzmiCapability::AntiCheatBoundary,
                ReadinessCapabilityState::Unknown,
            ),
        ]
        .into_iter()
        .map(|(capability, state)| ZzmiCapabilityAssessment {
            capability,
            state,
            evidence: format!("synthetic test evidence for {capability:?}"),
        })
        .collect()
    }

    fn ready_input() -> ZzmiLaunchReadinessInput {
        ZzmiLaunchReadinessInput {
            game_id: "zenless-zone-zero".into(),
            game_name: "Zenless Zone Zero".into(),
            app_id: 4_162_040,
            distribution: ReadinessCapabilityState::Verified,
            installation: ReadinessEvidenceState::Verified,
            game_content_completeness: ReadinessEvidenceState::Verified,
            executable: ReadinessEvidenceState::Verified,
            compatdata: ReadinessEvidenceState::Verified,
            prefix: ReadinessEvidenceState::Verified,
            zzmi_package: ReadinessEvidenceState::Verified,
            zzmi_authenticity: ReadinessEvidenceState::Verified,
            xxmi_libraries_package: ReadinessEvidenceState::Verified,
            xxmi_libraries_authenticity: ReadinessEvidenceState::Verified,
            managed_runtime: ReadinessEvidenceState::Verified,
            game_runtime_selection: ReadinessCapabilityState::Verified,
            selected_game_runtime: Some("Explicitly evidenced runtime".into()),
            available_proton_tools: vec!["lab fixture".into()],
            explicit_lab_runtime: Some("Lab runtime is separate".into()),
            game_install_path: None,
            game_executable_path: None,
            compatdata_path: None,
            prefix_path: None,
            steam_process_context: None,
            managed_runtime_root: None,
            integration_version: Some("fixture".into()),
            xxmi_libraries_version: Some("fixture".into()),
            upstream_default_loader_mode: ZzmiLoaderMode::Hook,
            upstream_loader_strategy: UpstreamLoaderStrategy::EitherDependingOnLaunchMode,
            hook_requirement: HookRequirementState::NotEstablished,
            capabilities: capabilities(ReadinessCapabilityState::Verified),
        }
    }

    fn context_evidence_with_mapping() -> SteamProtonProcessContextEvidence {
        SteamProtonProcessContextEvidence {
            app_id: 4_162_040,
            steam_root: Some("/steam".into()),
            library_root: Some("/steam".into()),
            app_manifest_path: Some("/steam/steamapps/appmanifest_4162040.acf".into()),
            compatdata_state: ReadinessEvidenceState::Verified,
            compatdata_path: Some("/steam/steamapps/compatdata/4162040".into()),
            prefix_state: ReadinessEvidenceState::Verified,
            prefix_path: Some("/steam/steamapps/compatdata/4162040/pfx".into()),
            config_path: Some("/steam/config/config.vdf".into()),
            config_state: SteamCompatConfigEvidenceState::Read,
            config_issue: None,
            per_game_mapping_state: PerGameToolMappingState::Configured,
            per_game_mapping: Some(ConfiguredToolMappingEvidence {
                app_id: 4_162_040,
                internal_name: Some("proton_experimental".into()),
                config: Some(String::new()),
                priority: Some("250".into()),
            }),
            global_default_mapping: None,
            matched_installed_tools: vec![InstalledCompatibilityToolEvidence {
                internal_id: Some("proton_experimental".into()),
                display_name: "Proton Experimental".into(),
                kind: ContextToolKind::Proton,
                origin: ContextToolOrigin::SteamLibrary,
                version: Some("fixture-version".into()),
                path: "/steam/steamapps/common/Proton - Experimental".into(),
            }],
            installed_compatibility_tools: vec![
                InstalledCompatibilityToolEvidence {
                    internal_id: Some("proton_experimental".into()),
                    display_name: "Proton Experimental".into(),
                    kind: ContextToolKind::Proton,
                    origin: ContextToolOrigin::SteamLibrary,
                    version: Some("fixture-version".into()),
                    path: "/steam/steamapps/common/Proton - Experimental".into(),
                },
                InstalledCompatibilityToolEvidence {
                    internal_id: Some("proton_9".into()),
                    display_name: "Proton 9".into(),
                    kind: ContextToolKind::Proton,
                    origin: ContextToolOrigin::SteamLibrary,
                    version: Some("fixture-version".into()),
                    path: "/steam/steamapps/common/Proton 9".into(),
                },
            ],
            effective_runtime_selection: RuntimeContextKnowledge::Unknown,
            effective_prefix: RuntimeContextKnowledge::Unknown,
            process_context: RuntimeContextKnowledge::Unknown,
            loader_same_prefix_requirement: RuntimeContextKnowledge::Unknown,
            loader_same_wineserver_requirement: RuntimeContextKnowledge::Unknown,
            wineserver_relationship: RuntimeContextKnowledge::Unknown,
            runtime_observation: None,
            findings: vec![SteamProcessContextFinding {
                claim: "Per-game compatibility-tool mapping".into(),
                state: ContextFindingState::Configured,
                source: "/steam/config/config.vdf".into(),
                detail: "synthetic mapping source".into(),
            }],
        }
    }

    #[test]
    fn blocked_hook_does_not_hide_direct_inject_candidate_or_select_it() {
        let mut input = ready_input();
        input.capabilities = capabilities(ReadinessCapabilityState::Blocked);
        let result = assess_zzmi_launch_readiness(input);
        assert_eq!(result.decision, ZzmiReadinessDecision::NeedsEvidence);
        assert_eq!(
            result.loader_strategy.upstream_strategy,
            UpstreamLoaderStrategy::EitherDependingOnLaunchMode
        );
        assert_eq!(
            result.loader_strategy.outcome,
            LoaderStrategyOutcome::NoDecision
        );
        assert_eq!(
            result.hook_requirement,
            HookRequirementState::NotEstablished
        );
        assert!(result.loader_strategy.direct_inject_candidate);
        assert_eq!(result.loader_strategy.preferred_mode, None);
        assert!(!result.controlled_experiment_gate.eligible_for_review);
        assert!(!result.controlled_experiment_gate.launches_game);
        assert!(result.capabilities.iter().any(|item| {
            item.capability == ZzmiCapability::HookInIsolatedLab
                && item.state == ReadinessCapabilityState::Blocked
        }));
        assert!(result
            .unknowns
            .iter()
            .any(|item| item.contains("loader_strategy_for_zzmi_linux")));
    }

    #[test]
    fn unknown_game_proton_never_inherits_lab_runtime() {
        let mut input = ready_input();
        input.game_runtime_selection = ReadinessCapabilityState::Unknown;
        input.selected_game_runtime = None;
        let result = assess_zzmi_launch_readiness(input);
        assert_eq!(
            result.game_runtime_selection,
            ReadinessCapabilityState::Unknown
        );
        assert_ne!(
            result.decision,
            ZzmiReadinessDecision::ReadyForControlledExperiment
        );
    }

    #[test]
    fn configured_mapping_and_multiple_candidates_do_not_claim_effective_runtime() {
        let mut input = ready_input();
        input.game_runtime_selection = ReadinessCapabilityState::Unknown;
        input.selected_game_runtime = None;
        input.available_proton_tools =
            vec!["Proton Experimental".into(), "GE-Proton fixture".into()];
        input.steam_process_context = Some(context_evidence_with_mapping());

        let result = assess_zzmi_launch_readiness(input);
        let context = result.steam_process_context.unwrap();
        assert_eq!(
            context.per_game_mapping_state,
            PerGameToolMappingState::Configured
        );
        assert_eq!(
            context.config_path.as_deref(),
            Some(std::path::Path::new("/steam/config/config.vdf"))
        );
        assert!(context.findings.iter().any(|finding| {
            finding.claim == "Per-game compatibility-tool mapping"
                && finding.source == "/steam/config/config.vdf"
        }));
        assert_eq!(
            context.effective_runtime_selection,
            RuntimeContextKnowledge::Unknown
        );
        assert_eq!(result.selected_game_runtime, None);
        assert_eq!(result.available_proton_tools.len(), 2);
        assert_eq!(context.installed_compatibility_tools.len(), 2);
        assert_eq!(context.matched_installed_tools.len(), 1);
        assert_eq!(result.decision, ZzmiReadinessDecision::NeedsEvidence);
        assert!(!result.controlled_experiment_gate.eligible_for_review);
        assert!(!result.controlled_experiment_gate.launches_game);
        assert!(result.requirements.iter().any(|requirement| {
            requirement.id == "effective_game_proton_context"
                && requirement.state == ZzmiRequirementState::Unknown
        }));
    }

    #[test]
    fn global_mapping_and_installed_candidates_do_not_imply_game_selection() {
        let mut input = ready_input();
        input.game_runtime_selection = ReadinessCapabilityState::Unknown;
        input.selected_game_runtime = None;
        input.available_proton_tools = vec!["Proton Experimental".into()];
        let mut context = context_evidence_with_mapping();
        context.per_game_mapping_state = PerGameToolMappingState::NoPerGameOverride;
        context.per_game_mapping = None;
        context.global_default_mapping = Some(ConfiguredToolMappingEvidence {
            app_id: 0,
            internal_name: Some("proton_experimental".into()),
            config: Some(String::new()),
            priority: Some("250".into()),
        });
        context.matched_installed_tools.clear();
        input.steam_process_context = Some(context);

        let result = assess_zzmi_launch_readiness(input);
        let context = result.steam_process_context.unwrap();
        assert_eq!(
            context.per_game_mapping_state,
            PerGameToolMappingState::NoPerGameOverride
        );
        assert_eq!(context.global_default_mapping.unwrap().app_id, 0);
        assert_eq!(
            context.effective_runtime_selection,
            RuntimeContextKnowledge::Unknown
        );
        assert_eq!(result.selected_game_runtime, None);
        assert_eq!(
            result.game_runtime_selection,
            ReadinessCapabilityState::Unknown
        );
        assert_eq!(result.decision, ZzmiReadinessDecision::NeedsEvidence);
    }

    #[test]
    fn compatdata_prefix_and_effective_context_remain_independent() {
        let mut input = ready_input();
        let mut context = context_evidence_with_mapping();
        context.compatdata_state = ReadinessEvidenceState::Missing;
        context.compatdata_path = Some("/steam/steamapps/compatdata/4162040".into());
        context.prefix_state = ReadinessEvidenceState::Missing;
        context.prefix_path = Some("/steam/steamapps/compatdata/4162040/pfx".into());
        input.compatdata = ReadinessEvidenceState::Missing;
        input.prefix = ReadinessEvidenceState::Missing;
        input.steam_process_context = Some(context);

        let result = assess_zzmi_launch_readiness(input);
        let context = result.steam_process_context.unwrap();
        assert_eq!(context.compatdata_state, ReadinessEvidenceState::Missing);
        assert_eq!(context.prefix_state, ReadinessEvidenceState::Missing);
        assert_eq!(context.effective_prefix, RuntimeContextKnowledge::Unknown);
        assert_eq!(result.installation, ReadinessEvidenceState::Verified);
        assert_eq!(result.decision, ZzmiReadinessDecision::NotReady);
    }

    #[test]
    fn missing_compatdata_is_not_confused_with_missing_game() {
        let mut input = ready_input();
        input.compatdata = ReadinessEvidenceState::Missing;
        let result = assess_zzmi_launch_readiness(input);
        assert_eq!(result.installation, ReadinessEvidenceState::Verified);
        assert_eq!(result.decision, ZzmiReadinessDecision::NotReady);
    }

    #[test]
    fn unknown_contexts_require_evidence_without_claiming_compatibility() {
        let mut input = ready_input();
        input.capabilities = capabilities(ReadinessCapabilityState::Verified);
        let result = assess_zzmi_launch_readiness(input);
        assert_eq!(result.decision, ZzmiReadinessDecision::NeedsEvidence);
        assert!(!result.controlled_experiment_gate.eligible_for_review);
        assert!(result
            .unknowns
            .iter()
            .any(|item| item.contains("same_prefix_requirement")));
        assert!(result
            .unknowns
            .iter()
            .any(|item| item.contains("game_specific_loader_compatibility")));
    }

    #[test]
    fn all_required_verified_evidence_only_opens_review_gate() {
        let mut input = ready_input();
        input.capabilities = capabilities(ReadinessCapabilityState::Verified)
            .into_iter()
            .map(|mut capability| {
                capability.state = ReadinessCapabilityState::Verified;
                capability
            })
            .collect();
        let result = assess_zzmi_launch_readiness(input);
        assert_eq!(
            result.decision,
            ZzmiReadinessDecision::ReadyForControlledExperiment
        );
        assert!(result.controlled_experiment_gate.eligible_for_review);
        assert!(
            result
                .controlled_experiment_gate
                .explicit_user_consent_required
        );
        assert!(!result.controlled_experiment_gate.consent_received);
        assert!(!result.controlled_experiment_gate.launches_game);
        assert_eq!(
            result.loader_strategy.outcome,
            LoaderStrategyOutcome::DirectInject
        );
        assert_eq!(
            result.loader_strategy.preferred_mode,
            Some(ZzmiLoaderMode::DirectInject)
        );
    }

    #[test]
    fn direct_inject_lab_success_does_not_imply_zzmi_compatibility_or_selection() {
        let result = assess_zzmi_launch_readiness(ready_input());
        assert!(result.loader_strategy.direct_inject_candidate);
        assert_eq!(
            result.loader_strategy.outcome,
            LoaderStrategyOutcome::NoDecision
        );
        assert_eq!(result.loader_strategy.preferred_mode, None);
        assert!(result
            .loader_strategy
            .direct_inject_criteria
            .iter()
            .any(|item| {
                item.id == "zzmi_game_specific_compatibility"
                    && item.state == LoaderCriterionState::Unknown
            }));
    }

    #[test]
    fn not_established_hook_requirement_is_not_treated_as_hook_not_required() {
        let result = assess_loader_strategy(
            "zenless-zone-zero",
            "linux_proton",
            UpstreamLoaderStrategy::EitherDependingOnLaunchMode,
            HookRequirementState::NotEstablished,
            &capabilities(ReadinessCapabilityState::Blocked),
        );
        assert_eq!(
            result.hook_requirement,
            HookRequirementState::NotEstablished
        );
        assert_eq!(result.outcome, LoaderStrategyOutcome::NoDecision);
        assert!(result
            .evidence
            .iter()
            .any(|item| item
                .contains("does not establish an exclusive technical requirement for Hook")));
        assert!(result.direct_inject_criteria.iter().any(|item| {
            item.id == "hook_exclusive_requirement"
                && item.state == LoaderCriterionState::NotEstablished
        }));
    }

    #[test]
    fn wine_hook_scope_is_not_process_bound_and_lab_guard_stays_blocked() {
        let result = assess_loader_strategy(
            "zenless-zone-zero",
            "linux_proton",
            UpstreamLoaderStrategy::EitherDependingOnLaunchMode,
            HookRequirementState::NotEstablished,
            &capabilities(ReadinessCapabilityState::Blocked),
        );
        let hook = result
            .isolation
            .iter()
            .find(|item| item.mode == ZzmiLoaderMode::Hook)
            .expect("hook isolation assessment exists");
        assert_eq!(hook.scope, LoaderIsolationScope::WineDesktopWithinServer);
        assert_eq!(hook.state, ReadinessCapabilityState::UpstreamDocumented);
        assert!(hook
            .limitation
            .contains("does not satisfy the Hook Lab single-target guard"));
    }

    #[test]
    fn steam_documentation_does_not_verify_steam_linux_or_proton() {
        let mut capabilities = capabilities(ReadinessCapabilityState::Verified);
        let steam_linux = capabilities
            .iter_mut()
            .find(|item| item.capability == ZzmiCapability::ZzzSteamUnderLinuxProton)
            .expect("Steam Linux capability exists");
        steam_linux.state = ReadinessCapabilityState::Unverified;
        let result = assess_loader_strategy(
            "zenless-zone-zero",
            "linux_proton",
            UpstreamLoaderStrategy::EitherDependingOnLaunchMode,
            HookRequirementState::NotEstablished,
            &capabilities,
        );
        assert_eq!(result.outcome, LoaderStrategyOutcome::NoDecision);
        assert!(result.direct_inject_criteria.iter().any(|item| {
            item.id == "steam_launch_context_under_proton"
                && item.state == LoaderCriterionState::Unknown
        }));
    }

    #[test]
    fn readiness_gate_exposes_independent_requirements_and_unknown_game_runtime() {
        let mut input = ready_input();
        input.game_runtime_selection = ReadinessCapabilityState::Unknown;
        input.selected_game_runtime = None;
        let result = assess_zzmi_launch_readiness(input);
        let gate = &result.controlled_experiment_gate;
        for component in [
            ZzmiGateComponent::PackageAuthenticity,
            ZzmiGateComponent::ManagedRuntime,
            ZzmiGateComponent::LoaderMechanism,
            ZzmiGateComponent::LoaderIsolation,
            ZzmiGateComponent::GameRuntimeContext,
            ZzmiGateComponent::SteamLaunchTopology,
            ZzmiGateComponent::ProcessIdentification,
            ZzmiGateComponent::PrefixContext,
            ZzmiGateComponent::UserConsent,
        ] {
            assert!(gate
                .components
                .iter()
                .any(|item| item.component == component));
        }
        assert_eq!(result.selected_game_runtime, None);
        assert_eq!(
            result.game_runtime_selection,
            ReadinessCapabilityState::Unknown
        );
        assert_eq!(result.loader_strategy.preferred_mode, None);
    }

    #[test]
    fn passive_observation_enriches_readiness_without_selecting_game_runtime_or_loader_context() {
        use lxmi_runtime::{
            ObservationEvidenceState, ProcessTopologyState, RuntimeObservationAppId,
            RuntimeObservationPath, RuntimeObservationSelection, RuntimeObservationStatus,
            RuntimeObservationSummary, WineServerObservation,
        };

        let mut context = context_evidence_with_mapping();
        let observed = RuntimeObservationSummary {
            status: RuntimeObservationStatus::Completed,
            active: false,
            process_observed: true,
            ambiguous_process_count: 0,
            app_id: RuntimeObservationAppId {
                state: ObservationEvidenceState::Verified,
                observed_values: vec![4_162_040],
                sources: vec!["STEAM_COMPAT_APP_ID (fixture PID)".into()],
            },
            runtime: RuntimeObservationSelection {
                state: ObservationEvidenceState::Verified,
                tool: Some(lxmi_runtime::ObservedRuntimeTool {
                    internal_id: Some("proton_experimental".into()),
                    display_name: "Proton Experimental".into(),
                    version: Some("fixture-version".into()),
                    path: "~/.steam/Proton Experimental".into(),
                }),
                evidence_source: Some("fixture process argv".into()),
            },
            compatdata: RuntimeObservationPath {
                state: ObservationEvidenceState::ObservedAndMatches,
                observed_path: Some("~/.steam/compatdata/4162040".into()),
                expected_path: Some("~/.steam/compatdata/4162040".into()),
                evidence_source: Some("STEAM_COMPAT_DATA_PATH".into()),
            },
            prefix: RuntimeObservationPath {
                state: ObservationEvidenceState::DerivedFromObservedCompatdata,
                observed_path: Some("~/.steam/compatdata/4162040/pfx".into()),
                expected_path: None,
                evidence_source: Some("STEAM_COMPAT_DATA_PATH + Proton layout".into()),
            },
            wineserver: WineServerObservation {
                state: ObservationEvidenceState::ObservedAndMatches,
                related_process_count: 1,
                evidence: "fixture WINEPREFIX match".into(),
            },
            process_topology_state: ProcessTopologyState::ObservedAncestry,
            observed_at_unix_millis: Some(123),
        };
        incorporate_runtime_observation(&mut context, &observed);

        let mut input = ready_input();
        input.game_runtime_selection = ReadinessCapabilityState::Unknown;
        input.selected_game_runtime = None;
        input.steam_process_context = Some(context);
        let readiness = assess_zzmi_launch_readiness(input);
        let process_context = readiness.steam_process_context.as_ref().unwrap();

        assert_eq!(readiness.selected_game_runtime, None);
        assert_eq!(
            readiness.game_runtime_selection,
            ReadinessCapabilityState::Unknown
        );
        assert_eq!(
            process_context.effective_runtime_selection,
            RuntimeContextKnowledge::Known
        );
        assert_eq!(
            process_context.effective_prefix,
            RuntimeContextKnowledge::Known
        );
        assert_eq!(
            process_context.wineserver_relationship,
            RuntimeContextKnowledge::Unknown
        );
        assert_eq!(
            process_context
                .runtime_observation
                .as_ref()
                .unwrap()
                .wineserver
                .state,
            ObservationEvidenceState::ObservedAndMatches
        );
        assert_eq!(
            process_context.loader_same_prefix_requirement,
            RuntimeContextKnowledge::Unknown
        );
        assert_eq!(
            process_context.loader_same_wineserver_requirement,
            RuntimeContextKnowledge::Unknown
        );
        assert_eq!(
            readiness.loader_strategy.outcome,
            LoaderStrategyOutcome::NoDecision
        );
        assert_eq!(readiness.decision, ZzmiReadinessDecision::NeedsEvidence);
        assert!(!readiness.controlled_experiment_gate.eligible_for_review);
        assert!(!readiness.controlled_experiment_gate.launches_game);
    }

    #[test]
    fn ambiguous_or_unverified_runtime_observation_does_not_promote_effective_selection() {
        use lxmi_runtime::{
            ObservationEvidenceState, ProcessTopologyState, RuntimeObservationAppId,
            RuntimeObservationPath, RuntimeObservationSelection, RuntimeObservationStatus,
            RuntimeObservationSummary, WineServerObservation,
        };

        let mut context = context_evidence_with_mapping();
        let observation = RuntimeObservationSummary {
            status: RuntimeObservationStatus::AmbiguousProcess,
            active: false,
            process_observed: false,
            ambiguous_process_count: 2,
            app_id: RuntimeObservationAppId {
                state: ObservationEvidenceState::Ambiguous,
                observed_values: vec![],
                sources: vec![],
            },
            runtime: RuntimeObservationSelection {
                state: ObservationEvidenceState::Ambiguous,
                tool: None,
                evidence_source: None,
            },
            compatdata: RuntimeObservationPath {
                state: ObservationEvidenceState::Ambiguous,
                observed_path: None,
                expected_path: None,
                evidence_source: None,
            },
            prefix: RuntimeObservationPath {
                state: ObservationEvidenceState::Ambiguous,
                observed_path: None,
                expected_path: None,
                evidence_source: None,
            },
            wineserver: WineServerObservation {
                state: ObservationEvidenceState::Ambiguous,
                related_process_count: 0,
                evidence: "multiple fixtures".into(),
            },
            process_topology_state: ProcessTopologyState::Ambiguous,
            observed_at_unix_millis: Some(456),
        };
        incorporate_runtime_observation(&mut context, &observation);
        assert_eq!(
            context.effective_runtime_selection,
            RuntimeContextKnowledge::Unknown
        );
        assert_eq!(context.effective_prefix, RuntimeContextKnowledge::Unknown);
        assert_eq!(context.process_context, RuntimeContextKnowledge::Unknown);
        assert_eq!(context.runtime_observation, Some(observation));
    }
}
