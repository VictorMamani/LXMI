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
    pub zzmi_version: Option<String>,
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
    pub zzmi_version: Option<String>,
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
            "XXMI Launcher v2.2.1 incluye una ruta genérica Direct Inject para Custom Launch; ZZMI conserva Hook por defecto cuando Custom Launch está desactivado.",
        ),
        criterion(
            "xxmi_libraries_generic_direct_inject",
            upstream_generic_state,
            "MigotoInjector usa el mismo 3dmloader.dll y la ruta genérica Inject para la clase de integración; esto documenta capacidad upstream, no compatibilidad con ZZZ.",
        ),
        criterion(
            "loader_component_local_use",
            component_local_use_state,
            "El componente 3dmloader.dll autenticado fue revisado para uso local de laboratorio; esta evidencia no autoriza su redistribución.",
        ),
        criterion(
            "hook_exclusive_requirement",
            match hook_requirement {
                HookRequirementState::Established => LoaderCriterionState::Unsatisfied,
                HookRequirementState::NotEstablished => LoaderCriterionState::NotEstablished,
                HookRequirementState::Unknown => LoaderCriterionState::Unknown,
            },
            "No se encontró evidencia upstream de un requisito exclusivo de Hook para ZZMI. NotEstablished no equivale a demostrar que Hook no sea necesario.",
        ),
        criterion(
            "direct_inject_component_in_lab",
            lab_direct_state,
            "La ruta Direct Inject de 3dmloader.dll cargó el DLL de prueba en el proceso LXMI fijado bajo Proton.",
        ),
        criterion(
            "remote_dll_in_lab",
            remote_direct_state,
            "El test DLL remoto fue visible y verificó marker/nonce en el target de LXMI.",
        ),
        criterion(
            "zzmi_game_specific_compatibility",
            criterion_from_capability(capability(ZzmiCapability::GameSpecificLoaderCompatibility)),
            "No se ha ejecutado ZZMI ni ZZZ con Direct Inject; el requisito específico de ZZMI permanece sin establecer.",
        ),
        criterion(
            "loader_identity_for_linux_runtime",
            criterion_from_capability(capability(ZzmiCapability::LoaderIdentitySemantics)),
            "La configuración loader= está documentada para el launcher upstream; no se ha validado la identidad de un loader ejecutado por LXMI bajo Proton.",
        ),
        criterion(
            "steam_launch_context_under_proton",
            criterion_from_capability(capability(ZzmiCapability::ZzzSteamUnderLinuxProton)),
            "La documentación Native Steam Launch consultada es para Windows y no prueba Steam Linux/Proton.",
        ),
        criterion(
            "process_identification_in_proton",
            criterion_from_capability(capability(
                ZzmiCapability::ZzzProcessIdentificationInProton,
            )),
            "Upstream busca el proceso por nombre; su visibilidad en el contexto real Steam/Proton de ZZZ no está probada.",
        ),
        criterion(
            "target_process_mapping",
            criterion_from_capability(capability(ZzmiCapability::ZzzTargetProcessMapping)),
            "El nombre de target está confirmado por la integración upstream y el ejecutable local; su resolución como proceso Windows desde Steam/Proton aún requiere evidencia.",
        ),
        criterion(
            "process_lifecycle_in_proton",
            criterion_from_capability(capability(
                ZzmiCapability::DirectInjectProcessLifecycleInProton,
            )),
            "El test de laboratorio creó su propio proceso; no demuestra que LXMI pueda controlar el ciclo de vida del target iniciado por Steam/Proton.",
        ),
        criterion(
            "same_prefix_or_wineserver_context",
            criterion_from_capability(capability(ZzmiCapability::SamePrefixRequirement)),
            "No se ha establecido qué relación de prefix/wineserver requiere la detección del target para ZZZ.",
        ),
        criterion(
            "anti_cheat_boundary",
            criterion_from_capability(capability(ZzmiCapability::AntiCheatBoundary)),
            "No se ha ejecutado el juego ni se ha estudiado/interactuado con anti-cheat.",
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
            evidence: "XXMI describe su Hook como global. En la implementación Wine consultada, SetWindowsHookEx sin thread target registra el hook en la tabla global del desktop actual, administrada por wineserver; WINEPREFIX distinto permite sesiones Wine independientes.".into(),
            limitation: "El alcance Wine no es un HOST TEST de Proton ni demuestra el comportamiento interno de 3dmloader.dll. Un hook de desktop puede alcanzar otros procesos del mismo desktop, por lo que no satisface el guard de target único del Hook Lab.".into(),
        },
        LoaderIsolationEvidence {
            mode: ZzmiLoaderMode::DirectInject,
            scope: LoaderIsolationScope::ProcessBound,
            state: direct_state,
            evidence: "En LXMI 0.7 el harness pasó únicamente el PID del hijo de test que creó LXMI al export Inject de 3dmloader.dll.".into(),
            limitation: "El flujo upstream normal encuentra por nombre el proceso de juego; el test de PID propio no verifica la identificación de ZZZ en Steam/Proton.".into(),
        },
    ];

    let evidence = vec![
        "En XXMI Launcher v2.2.1, el lanzamiento estándar hereda use_hook=true de ModelImporterPackage; ZZMI no lo sobrescribe.".into(),
        "Al activar Custom Launch, custom_launch_inject_mode == Hook selecciona Hook; Inject selecciona la ruta Direct Inject y Bypass omite d3d11.dll salvo que se añada como library extra.".into(),
        "La configuración de ZZMI no establece una necesidad técnica exclusiva de Hook. La ausencia de tal requisito no demuestra por sí sola compatibilidad de Direct Inject con ZZZ.".into(),
    ];
    let known_limitations = vec![
        "Direct Inject está probado solo con el target y DLL de laboratorio de LXMI, no con ZZMI/ZZZ.".into(),
        "El Proton explícito del laboratorio no es la selección Proton de Steam para el juego.".into(),
        "No se determinó si el proceso helper, Steam y ZZZ deben compartir prefix o wineserver.".into(),
        "La política del Hook Lab sigue rechazando Hook antes de resolver runtime o iniciar procesos.".into(),
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
        "La instalación del juego debe estar presente y reconocida.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "game_content_completeness",
        input.game_content_completeness,
        "La presencia del directorio/ejecutable no demuestra por sí sola que todos los datos del juego estén completos.",
    );
    push_capability_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "steam_distribution",
        input.distribution,
        "La distribución Steam debe estar identificada; HoYoPlay/manual son rutas distintas.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "expected_executable",
        input.executable,
        "Debe existir el ejecutable esperado de Zenless Zone Zero.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "compatdata",
        input.compatdata,
        "La ubicación compatdata debe estar identificada.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "game_prefix",
        input.prefix,
        "El estado del prefix del juego debe ser conocido; no se inicializa durante readiness.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "zzmi_package",
        input.zzmi_package,
        "El paquete ZZMI debe estar disponible y validado.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "zzmi_authenticity",
        input.zzmi_authenticity,
        "La procedencia/autenticidad ZZMI debe estar verificada.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "xxmi_libraries_package",
        input.xxmi_libraries_package,
        "XXMI Libraries es una dependencia independiente de ZZMI.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "xxmi_libraries_authenticity",
        input.xxmi_libraries_authenticity,
        "La procedencia/autenticidad de XXMI Libraries debe estar verificada.",
    );
    push_evidence_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "managed_runtime",
        input.managed_runtime,
        "El runtime ensamblado administrado por LXMI debe estar íntegro.",
    );
    push_capability_requirement(
        &mut requirements,
        &mut blockers,
        &mut unknowns,
        "game_proton_selection",
        input.game_runtime_selection,
        "El Proton elegido por Steam para ZZZ no se deduce del runtime elegido para el laboratorio.",
    );
    if let Some(context) = &input.steam_process_context {
        push_capability_requirement(
            &mut requirements,
            &mut blockers,
            &mut unknowns,
            "effective_game_proton_context",
            context_knowledge_capability(context.effective_runtime_selection),
            "El mapping local puede indicar una configuración, pero no observa qué runtime ejecutó Steam para esta sesión.",
        );
        push_capability_requirement(
            &mut requirements,
            &mut blockers,
            &mut unknowns,
            "effective_game_prefix_context",
            context_knowledge_capability(context.effective_prefix),
            "compatdata/pfx es una ruta candidata observada; no prueba el prefix efectivo de un proceso iniciado.",
        );
        push_capability_requirement(
            &mut requirements,
            &mut blockers,
            &mut unknowns,
            "steam_process_context",
            context_knowledge_capability(context.process_context),
            "No se iniciaron ni inspeccionaron procesos Steam/Proton/ZZZ en este incremento.",
        );
        push_capability_requirement(
            &mut requirements,
            &mut blockers,
            &mut unknowns,
            "loader_same_prefix_requirement",
            context_knowledge_capability(context.loader_same_prefix_requirement),
            "No hay evidencia que determine si un futuro loader debe compartir el prefix de ZZZ.",
        );
        push_capability_requirement(
            &mut requirements,
            &mut blockers,
            &mut unknowns,
            "loader_same_wineserver_requirement",
            context_knowledge_capability(context.loader_same_wineserver_requirement),
            "No hay evidencia que determine si un futuro loader debe compartir wineserver con ZZZ.",
        );
        push_capability_requirement(
            &mut requirements,
            &mut blockers,
            &mut unknowns,
            "wineserver_relationship",
            context_knowledge_capability(context.wineserver_relationship),
            "No se observaron wineserver ni procesos bajo Wine/Proton.",
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
                "El modo Hook seleccionado está probado en el laboratorio.",
            );
        }
        LoaderStrategyOutcome::DirectInject => {
            push_capability_requirement(
                &mut requirements,
                &mut blockers,
                &mut unknowns,
                "selected_loader_mode_in_lab",
                ReadinessCapabilityState::Verified,
                "El modo Direct Inject seleccionado está probado en el laboratorio.",
            );
        }
        LoaderStrategyOutcome::NoDecision => {
            push_capability_requirement(
                &mut requirements,
                &mut blockers,
                &mut unknowns,
                "loader_strategy_for_zzmi_linux",
                ReadinessCapabilityState::Unknown,
                "Upstream ofrece Hook por defecto y Direct Inject en Custom Launch; la compatibilidad específica ZZMI/ZZZ/Linux no está establecida.",
            );
        }
        LoaderStrategyOutcome::Blocked => {
            push_capability_requirement(
                &mut requirements,
                &mut blockers,
                &mut unknowns,
                "loader_strategy_for_zzmi_linux",
                ReadinessCapabilityState::Blocked,
                "No hay una estrategia loader que cumpla la política de seguridad con la evidencia disponible.",
            );
        }
    }

    for (capability, id, detail) in [
        (
            ZzmiCapability::HookVisibilityInSteamContext,
            "hook_steam_context",
            "No se ha probado la visibilidad del hook desde el contexto de lanzamiento Steam/Proton de ZZZ.",
        ),
        (
            ZzmiCapability::SamePrefixRequirement,
            "same_prefix_requirement",
            "No se ha determinado si loader y ZZZ deben compartir prefix/contexto Proton.",
        ),
        (
            ZzmiCapability::GameSpecificLoaderCompatibility,
            "game_specific_loader_compatibility",
            "La prueba de laboratorio no demuestra compatibilidad del loader con ZZZ.",
        ),
        (
            ZzmiCapability::ZzzProcessIdentificationInProton,
            "process_identification_in_proton",
            "No se ha probado la identificación del proceso objetivo ZZZ desde el contexto del loader bajo Proton.",
        ),
        (
            ZzmiCapability::AntiCheatBoundary,
            "anti_cheat_boundary",
            "No se ha verificado que el flujo de juego pueda mantenerse dentro del límite sin interacción con anti-cheat.",
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
            "La guía Native Steam Launch no demuestra el contexto real Steam Linux/Proton de ZZZ.",
        ),
        (
            ZzmiCapability::NativeSteamLaunchUpstream,
            "native_steam_launch_topology",
            "La ruta upstream documentada para ZZZ describe Steam en Windows; su topología Linux/Proton requiere evidencia propia.",
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
                "Faltan requisitos observados; esta evaluación no ejecuta el juego.".into()
            }
            ZzmiReadinessDecision::NeedsEvidence => {
                "Hay requisitos desconocidos; esta evaluación no ejecuta el juego.".into()
            }
            ZzmiReadinessDecision::ReadyForControlledExperiment => {
                "La evidencia permitiría revisar un experimento; aún harían falta consentimiento explícito y una implementación separada.".into()
            }
            ZzmiReadinessDecision::Blocked => {
                "Una restricción de seguridad o requisito bloquea la revisión del experimento.".into()
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
        zzmi_version: input.zzmi_version,
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
            "No hay estrategia preferida. El Hook observado tiene alcance de desktop dentro del wineserver, no está probado como aislado al proceso; Direct Inject para ZZMI/ZZZ tampoco está verificado.".into(),
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
            evidence: "ZZMI y XXMI Libraries deben estar presentes y mantener procedencia verificada.".into(),
        },
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::ManagedRuntime,
            state: state_for("managed_runtime"),
            evidence: "El runtime ensamblado se inspecciona en modo read-only.".into(),
        },
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::LoaderMechanism,
            state: loader_mechanism_state,
            evidence: format!(
                "Upstream: {:?}; estrategia LXMI para {}: {:?}.",
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
            evidence: "La selección Proton de ZZZ requiere evidencia propia; el runtime del laboratorio no se hereda.".into(),
        },
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::SteamLaunchTopology,
            state: combine(&[
                "steam_distribution",
                "native_steam_launch_topology",
                "steam_zzz_launch_context",
            ]),
            evidence: "La ruta Native Steam Launch documentada no demuestra Steam Linux/Proton.".into(),
        },
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::ProcessIdentification,
            state: state_for("process_identification_in_proton"),
            evidence: "La identificación del target ZZZ desde el loader en Proton no se ha probado.".into(),
        },
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::SteamProcessContext,
            state: state_for("steam_process_context"),
            evidence: "El contexto efectivo de ejecución no se deduce de archivos ni de runtimes instalados.".into(),
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
            evidence: "compatdata/pfx detectado no resuelve si loader y juego deben compartir prefix o wineserver.".into(),
        },
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::WineServerContext,
            state: combine(&[
                "wineserver_relationship",
                "loader_same_wineserver_requirement",
            ]),
            evidence: "El wineserver efectivo y su relación con un futuro loader no se inspeccionaron.".into(),
        },
        ZzmiGateComponentAssessment {
            component: ZzmiGateComponent::UserConsent,
            state: ZzmiRequirementState::Unknown,
            evidence: "Este assessment read-only no recoge consentimiento; cualquier experimento futuro lo requerirá antes de ejecutarse.".into(),
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
            zzmi_version: Some("fixture".into()),
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
            .any(|item| item.contains("no establece una necesidad técnica exclusiva de Hook")));
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
        assert!(hook.limitation.contains("no satisface el guard"));
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
}
