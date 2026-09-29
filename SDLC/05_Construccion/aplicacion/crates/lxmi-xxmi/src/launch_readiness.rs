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
    pub managed_runtime_root: Option<std::path::PathBuf>,
    pub zzmi_version: Option<String>,
    pub xxmi_libraries_version: Option<String>,
    pub loader_mode: ZzmiLoaderMode,
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
    pub managed_runtime_root: Option<std::path::PathBuf>,
    pub zzmi_version: Option<String>,
    pub xxmi_libraries_version: Option<String>,
    pub loader_mode: ZzmiLoaderMode,
    pub capabilities: Vec<ZzmiCapabilityAssessment>,
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

    let required_loader_capability = match input.loader_mode {
        ZzmiLoaderMode::Hook => Some((
            ZzmiCapability::HookInIsolatedLab,
            ZzmiCapability::RemoteDllViaHook,
        )),
        ZzmiLoaderMode::DirectInject => Some((
            ZzmiCapability::DirectInjectInIsolatedLab,
            ZzmiCapability::RemoteDllViaDirectInject,
        )),
        ZzmiLoaderMode::Unknown => None,
    };
    if let Some((capability, remote_capability)) = required_loader_capability {
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
            "selected_loader_mode_in_lab",
            state,
            "El modo loader elegido debe probarse bajo Proton en el laboratorio restringido.",
        );
        let remote_state = input
            .capabilities
            .iter()
            .find(|item| item.capability == remote_capability)
            .map(|item| item.state)
            .unwrap_or(ReadinessCapabilityState::Unknown);
        push_capability_requirement(
            &mut requirements,
            &mut blockers,
            &mut unknowns,
            "remote_runtime_path_in_selected_loader_mode",
            remote_state,
            "La ruta del runtime remoto debe estar comprobada en el modo loader elegido.",
        );
    } else {
        push_capability_requirement(
            &mut requirements,
            &mut blockers,
            &mut unknowns,
            "selected_loader_mode_in_lab",
            ReadinessCapabilityState::Unknown,
            "El modo loader de ZZMI no está determinado.",
        );
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
    let controlled_experiment_gate = ControlledGameExperimentGate {
        decision,
        eligible_for_review: decision == ZzmiReadinessDecision::ReadyForControlledExperiment,
        explicit_user_consent_required: true,
        consent_received: false,
        launches_game: false,
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
        managed_runtime_root: input.managed_runtime_root,
        zzmi_version: input.zzmi_version,
        xxmi_libraries_version: input.xxmi_libraries_version,
        loader_mode: input.loader_mode,
        capabilities: input.capabilities,
        requirements,
        blockers,
        unknowns,
        decision,
        controlled_experiment_gate,
    }
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
            managed_runtime_root: None,
            zzmi_version: Some("fixture".into()),
            xxmi_libraries_version: Some("fixture".into()),
            loader_mode: ZzmiLoaderMode::Hook,
            capabilities: capabilities(ReadinessCapabilityState::Verified),
        }
    }

    #[test]
    fn hook_blocker_prevents_game_experiment_readiness() {
        let mut input = ready_input();
        input.capabilities = capabilities(ReadinessCapabilityState::Blocked);
        let result = assess_zzmi_launch_readiness(input);
        assert_eq!(result.decision, ZzmiReadinessDecision::Blocked);
        assert!(!result.controlled_experiment_gate.eligible_for_review);
        assert!(!result.controlled_experiment_gate.launches_game);
        assert!(result
            .blockers
            .iter()
            .any(|item| item.contains("selected_loader_mode_in_lab")));
        assert!(result
            .blockers
            .iter()
            .any(|item| item.contains("remote_runtime_path_in_selected_loader_mode")));
    }

    #[test]
    fn unknown_game_proton_never_inherits_lab_runtime() {
        let mut input = ready_input();
        input.game_runtime_selection = ReadinessCapabilityState::Unknown;
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
    }
}
