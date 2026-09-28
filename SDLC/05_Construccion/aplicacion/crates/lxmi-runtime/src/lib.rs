use std::path::PathBuf;

use lxmi_core::{
    CompatibilityRuntimePolicy, Game, GameDistribution, GameExecutableStatus,
    GameInstallationStatus, ProtonCompatDataStatus, SUPPORTED_GAMES,
};
use lxmi_proton::{
    CompatibilityTool, CompatibilityToolDiscoveryResult, CompatibilityToolDiscoveryStatus,
    CompatibilityToolIssue, CompatibilityToolKind, CompatibilityToolSource,
    CompatibilityToolStatus,
};
use lxmi_steam::{SteamDiscoveryResult, SteamGameDiscoveryStatus, SteamGameInstallation};
use tracing::info;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCandidate {
    pub internal_id: Option<String>,
    pub display_name: String,
    pub path: PathBuf,
    pub source: CompatibilityToolSource,
    pub version: Option<String>,
}

impl From<&CompatibilityTool> for RuntimeCandidate {
    fn from(tool: &CompatibilityTool) -> Self {
        Self {
            internal_id: tool.internal_id.clone(),
            display_name: tool.display_name.clone(),
            path: tool.path.clone(),
            source: tool.source,
            version: tool.version.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeCandidateDiscovery {
    Complete,
    Partial,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimePlatform {
    Steam,
}

impl From<CompatibilityToolDiscoveryStatus> for RuntimeCandidateDiscovery {
    fn from(status: CompatibilityToolDiscoveryStatus) -> Self {
        match status {
            CompatibilityToolDiscoveryStatus::Complete => Self::Complete,
            CompatibilityToolDiscoveryStatus::Partial => Self::Partial,
            CompatibilityToolDiscoveryStatus::NotAvailable => Self::Unavailable,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeSelectionUnknownReason {
    NoReliablePerGameSelectionEvidence,
    GamePolicyUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeSelection {
    Unknown {
        reason: RuntimeSelectionUnknownReason,
    },
    Selected {
        candidate: RuntimeCandidate,
    },
    NotRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameInstallationCompleteness {
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameInstallationAssessment {
    NotFound {
        expected_steam_app_ids: Vec<u32>,
    },
    Detected {
        steam_app_id: u32,
        manifest_name: String,
        steam_library: PathBuf,
        install_path: PathBuf,
        distribution: GameDistribution,
        executable_path: Option<PathBuf>,
        executable_status: GameExecutableStatus,
        directory_status: GameInstallationStatus,
        completeness: GameInstallationCompleteness,
    },
    Unknown {
        discovery_status: SteamGameDiscoveryStatus,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompatDataAssessment {
    NotRequired,
    Unknown {
        discovery_status: SteamGameDiscoveryStatus,
    },
    NotFound {
        expected_path: PathBuf,
    },
    Found {
        path: PathBuf,
    },
    Invalid {
        path: PathBuf,
    },
    Unreadable {
        path: PathBuf,
        status: ProtonCompatDataStatus,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefixAssessment {
    NotRequired,
    Unknown {
        discovery_status: SteamGameDiscoveryStatus,
    },
    NotInitialized {
        expected_path: PathBuf,
    },
    CandidateFound {
        path: PathBuf,
    },
    Invalid {
        path: PathBuf,
    },
    Unreadable {
        path: PathBuf,
        status: ProtonCompatDataStatus,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchRequirementKind {
    GameInstallationPresent,
    ProtonCandidateAvailable,
    RuntimeSelectionDetermined,
    CompatDataInitialized,
    PrefixAvailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequirementState {
    Satisfied,
    Missing,
    Unknown,
    NotRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaunchRequirementStatus {
    pub requirement: LaunchRequirementKind,
    pub state: RequirementState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeReadiness {
    Ready,
    Incomplete,
    NeedsInitialization,
    Unknown,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeEvidence {
    GameDiscoveryEvaluated {
        status: SteamGameDiscoveryStatus,
    },
    GameManifestObserved {
        app_id: u32,
        name: String,
        library: PathBuf,
    },
    GameDirectoryObserved {
        path: PathBuf,
        status: GameInstallationStatus,
        completeness: GameInstallationCompleteness,
    },
    RuntimeDiscoveryEvaluated {
        status: RuntimeCandidateDiscovery,
        proton_candidate_count: usize,
    },
    ProtonCandidateObserved {
        display_name: String,
        path: PathBuf,
    },
    RuntimeSelectionNotObserved {
        reason: RuntimeSelectionUnknownReason,
    },
    CompatDataObserved {
        path: PathBuf,
        status: ProtonCompatDataStatus,
    },
    PrefixObserved {
        expected_path: PathBuf,
        state: PrefixEvidenceState,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixEvidenceState {
    FoundCandidate,
    NotFound,
    Invalid,
    Unreadable,
    Unknown,
    NotRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeIssueCode {
    GameNotFound,
    GameDiscoveryIncomplete,
    RuntimeDiscoveryUnavailable,
    RuntimeDiscoveryIncomplete,
    NoProtonCandidates,
    RuntimeSelectionUnknown,
    CompatDataNotFound,
    PrefixNotInitialized,
    CompatDataUnreadable,
    CompatDataInvalid,
    InstallationIncompleteOrUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeIssueSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeIssue {
    pub code: RuntimeIssueCode,
    pub severity: RuntimeIssueSeverity,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRuntimePlan {
    pub game: Game,
    pub platform: RuntimePlatform,
    pub installation: GameInstallationAssessment,
    pub runtime_policy: CompatibilityRuntimePolicy,
    pub selection: RuntimeSelection,
    pub candidate_discovery: RuntimeCandidateDiscovery,
    pub available_runtime_candidates: Vec<RuntimeCandidate>,
    pub compatdata: CompatDataAssessment,
    pub prefix: PrefixAssessment,
    pub requirements: Vec<LaunchRequirementStatus>,
    pub readiness: RuntimeReadiness,
    pub evidence: Vec<RuntimeEvidence>,
    pub issues: Vec<RuntimeIssue>,
    pub compatibility_tool_issues: Vec<CompatibilityToolIssue>,
}

pub struct RuntimePlanner;

impl RuntimePlanner {
    pub fn plan_all(
        discovery: &SteamDiscoveryResult,
        compatibility_tools: &CompatibilityToolDiscoveryResult,
    ) -> Vec<GameRuntimePlan> {
        info!(games = SUPPORTED_GAMES.len(), "Runtime planning started");
        let plans = SUPPORTED_GAMES
            .iter()
            .copied()
            .map(|game| {
                let installation = discovery
                    .games
                    .games
                    .iter()
                    .find(|entry| entry.installation.game.id == game.id);
                plan_game(
                    game,
                    discovery.games.status,
                    installation,
                    compatibility_tools,
                )
            })
            .collect::<Vec<_>>();
        info!(plans = plans.len(), "Runtime planning completed");
        plans
    }
}

pub fn plan_game(
    game: Game,
    game_discovery_status: SteamGameDiscoveryStatus,
    installation: Option<&SteamGameInstallation>,
    compatibility_tools: &CompatibilityToolDiscoveryResult,
) -> GameRuntimePlan {
    let candidate_discovery = compatibility_tools.status.into();
    let available_runtime_candidates = compatibility_tools
        .tools
        .iter()
        .filter(|tool| {
            tool.kind == CompatibilityToolKind::Proton
                && tool.status == CompatibilityToolStatus::Valid
        })
        .map(RuntimeCandidate::from)
        .collect::<Vec<_>>();
    let installation_assessment = assess_installation(game, game_discovery_status, installation);
    let (compatdata, prefix) = assess_compatdata(
        game.compatibility_runtime_policy,
        installation,
        game_discovery_status,
    );
    let selection = match game.compatibility_runtime_policy {
        CompatibilityRuntimePolicy::Expected => RuntimeSelection::Unknown {
            reason: RuntimeSelectionUnknownReason::NoReliablePerGameSelectionEvidence,
        },
        CompatibilityRuntimePolicy::NotRequired => RuntimeSelection::NotRequired,
        CompatibilityRuntimePolicy::Unknown => RuntimeSelection::Unknown {
            reason: RuntimeSelectionUnknownReason::GamePolicyUnknown,
        },
    };

    let mut evidence = vec![RuntimeEvidence::GameDiscoveryEvaluated {
        status: game_discovery_status,
    }];
    if let Some(installation) = installation {
        evidence.push(RuntimeEvidence::GameManifestObserved {
            app_id: installation.steam_app_id,
            name: installation.manifest_name.clone(),
            library: installation.steam_library.clone(),
        });
        evidence.push(RuntimeEvidence::GameDirectoryObserved {
            path: installation.installation.install_path.clone(),
            status: installation.installation.status,
            completeness: GameInstallationCompleteness::Unknown,
        });
        evidence.push(RuntimeEvidence::CompatDataObserved {
            path: installation.compatdata.compatdata_path.clone(),
            status: installation.compatdata.status,
        });
        let expected_prefix_path = installation
            .compatdata
            .prefix_path
            .clone()
            .unwrap_or_else(|| installation.compatdata.compatdata_path.join("pfx"));
        let prefix_state = match &prefix {
            PrefixAssessment::NotRequired => PrefixEvidenceState::NotRequired,
            PrefixAssessment::NotInitialized { .. } => PrefixEvidenceState::NotFound,
            PrefixAssessment::CandidateFound { .. } => PrefixEvidenceState::FoundCandidate,
            PrefixAssessment::Invalid { .. } => PrefixEvidenceState::Invalid,
            PrefixAssessment::Unreadable { .. } => PrefixEvidenceState::Unreadable,
            PrefixAssessment::Unknown { .. } => PrefixEvidenceState::Unknown,
        };
        evidence.push(RuntimeEvidence::PrefixObserved {
            expected_path: expected_prefix_path,
            state: prefix_state,
        });
    }
    evidence.push(RuntimeEvidence::RuntimeDiscoveryEvaluated {
        status: candidate_discovery,
        proton_candidate_count: available_runtime_candidates.len(),
    });
    evidence.extend(available_runtime_candidates.iter().map(|candidate| {
        RuntimeEvidence::ProtonCandidateObserved {
            display_name: candidate.display_name.clone(),
            path: candidate.path.clone(),
        }
    }));
    if let RuntimeSelection::Unknown { reason } = &selection {
        evidence.push(RuntimeEvidence::RuntimeSelectionNotObserved { reason: *reason });
    }

    let requirements = build_requirements(
        game.compatibility_runtime_policy,
        &installation_assessment,
        candidate_discovery,
        available_runtime_candidates.len(),
        &selection,
        &compatdata,
        &prefix,
    );
    let readiness = assess_readiness(
        game.compatibility_runtime_policy,
        game_discovery_status,
        &installation_assessment,
        candidate_discovery,
        &available_runtime_candidates,
        &selection,
        &prefix,
    );
    let issues = collect_issues(
        game.compatibility_runtime_policy,
        &installation_assessment,
        candidate_discovery,
        &available_runtime_candidates,
        &selection,
        &compatdata,
        &prefix,
    );

    info!(game = game.id, "Game installation state evaluated");
    info!(
        game = game.id,
        candidates = available_runtime_candidates.len(),
        discovery = ?candidate_discovery,
        "Compatibility runtime candidates evaluated"
    );
    if matches!(selection, RuntimeSelection::Unknown { .. }) {
        info!(game = game.id, "Runtime selection unknown");
    }
    if matches!(prefix, PrefixAssessment::NotInitialized { .. }) {
        info!(game = game.id, "Compatdata not initialized");
    }
    info!(game = game.id, readiness = ?readiness, "Runtime readiness calculated");

    GameRuntimePlan {
        game,
        platform: RuntimePlatform::Steam,
        installation: installation_assessment,
        runtime_policy: game.compatibility_runtime_policy,
        selection,
        candidate_discovery,
        available_runtime_candidates,
        compatdata,
        prefix,
        requirements,
        readiness,
        evidence,
        issues,
        compatibility_tool_issues: compatibility_tools.issues.clone(),
    }
}

fn assess_installation(
    game: Game,
    discovery_status: SteamGameDiscoveryStatus,
    installation: Option<&SteamGameInstallation>,
) -> GameInstallationAssessment {
    match installation {
        Some(installation) => GameInstallationAssessment::Detected {
            steam_app_id: installation.steam_app_id,
            manifest_name: installation.manifest_name.clone(),
            steam_library: installation.steam_library.clone(),
            install_path: installation.installation.install_path.clone(),
            distribution: installation.installation.distribution,
            executable_path: installation.installation.executable_path.clone(),
            executable_status: installation.installation.executable_status,
            directory_status: installation.installation.status,
            completeness: GameInstallationCompleteness::Unknown,
        },
        None if discovery_status == SteamGameDiscoveryStatus::Complete => {
            GameInstallationAssessment::NotFound {
                expected_steam_app_ids: game.steam_app_ids.to_vec(),
            }
        }
        None => GameInstallationAssessment::Unknown { discovery_status },
    }
}

fn assess_compatdata(
    policy: CompatibilityRuntimePolicy,
    installation: Option<&SteamGameInstallation>,
    discovery_status: SteamGameDiscoveryStatus,
) -> (CompatDataAssessment, PrefixAssessment) {
    if policy == CompatibilityRuntimePolicy::NotRequired {
        return (
            CompatDataAssessment::NotRequired,
            PrefixAssessment::NotRequired,
        );
    }
    if policy == CompatibilityRuntimePolicy::Unknown {
        return (
            CompatDataAssessment::Unknown { discovery_status },
            PrefixAssessment::Unknown { discovery_status },
        );
    }
    let Some(installation) = installation else {
        return (
            CompatDataAssessment::Unknown { discovery_status },
            PrefixAssessment::Unknown { discovery_status },
        );
    };
    let compatdata = &installation.compatdata;
    let expected_prefix_path = compatdata
        .prefix_path
        .clone()
        .unwrap_or_else(|| compatdata.compatdata_path.join("pfx"));
    match compatdata.status {
        ProtonCompatDataStatus::NotFound => (
            CompatDataAssessment::NotFound {
                expected_path: compatdata.compatdata_path.clone(),
            },
            PrefixAssessment::NotInitialized {
                expected_path: expected_prefix_path,
            },
        ),
        ProtonCompatDataStatus::CompatDataFound => (
            CompatDataAssessment::Found {
                path: compatdata.compatdata_path.clone(),
            },
            PrefixAssessment::NotInitialized {
                expected_path: expected_prefix_path,
            },
        ),
        ProtonCompatDataStatus::PrefixFound => match &compatdata.prefix_path {
            Some(path) => (
                CompatDataAssessment::Found {
                    path: compatdata.compatdata_path.clone(),
                },
                PrefixAssessment::CandidateFound { path: path.clone() },
            ),
            None => (
                CompatDataAssessment::Invalid {
                    path: compatdata.compatdata_path.clone(),
                },
                PrefixAssessment::Invalid {
                    path: expected_prefix_path,
                },
            ),
        },
        ProtonCompatDataStatus::Invalid => (
            CompatDataAssessment::Invalid {
                path: compatdata.compatdata_path.clone(),
            },
            PrefixAssessment::Invalid {
                path: expected_prefix_path,
            },
        ),
        ProtonCompatDataStatus::PermissionDenied | ProtonCompatDataStatus::IoError => (
            CompatDataAssessment::Unreadable {
                path: compatdata.compatdata_path.clone(),
                status: compatdata.status,
            },
            PrefixAssessment::Unreadable {
                path: expected_prefix_path,
                status: compatdata.status,
            },
        ),
    }
}

fn build_requirements(
    policy: CompatibilityRuntimePolicy,
    installation: &GameInstallationAssessment,
    discovery: RuntimeCandidateDiscovery,
    candidate_count: usize,
    selection: &RuntimeSelection,
    compatdata: &CompatDataAssessment,
    prefix: &PrefixAssessment,
) -> Vec<LaunchRequirementStatus> {
    let installation_state = match installation {
        GameInstallationAssessment::Detected {
            directory_status: GameInstallationStatus::Installed,
            ..
        } => RequirementState::Satisfied,
        GameInstallationAssessment::Detected {
            directory_status:
                GameInstallationStatus::DirectoryMissing | GameInstallationStatus::DirectoryInvalid,
            ..
        }
        | GameInstallationAssessment::NotFound { .. } => RequirementState::Missing,
        GameInstallationAssessment::Detected {
            directory_status: GameInstallationStatus::PermissionDenied,
            ..
        }
        | GameInstallationAssessment::Unknown { .. } => RequirementState::Unknown,
    };
    let policy_state = match policy {
        CompatibilityRuntimePolicy::Expected => None,
        CompatibilityRuntimePolicy::NotRequired => Some(RequirementState::NotRequired),
        CompatibilityRuntimePolicy::Unknown => Some(RequirementState::Unknown),
    };
    let candidate_state = match policy_state {
        Some(state) => state,
        None if candidate_count > 0 => RequirementState::Satisfied,
        None => match discovery {
            RuntimeCandidateDiscovery::Complete => RequirementState::Missing,
            RuntimeCandidateDiscovery::Partial | RuntimeCandidateDiscovery::Unavailable => {
                RequirementState::Unknown
            }
        },
    };
    let selection_state = match selection {
        RuntimeSelection::Selected { .. } => RequirementState::Satisfied,
        RuntimeSelection::Unknown { .. } => RequirementState::Unknown,
        RuntimeSelection::NotRequired => RequirementState::NotRequired,
    };
    let compatdata_state = match (policy, compatdata) {
        (CompatibilityRuntimePolicy::NotRequired, _) => RequirementState::NotRequired,
        (CompatibilityRuntimePolicy::Unknown, _) => RequirementState::Unknown,
        (_, CompatDataAssessment::Found { .. }) => RequirementState::Satisfied,
        (_, CompatDataAssessment::NotFound { .. }) => RequirementState::Missing,
        (
            _,
            CompatDataAssessment::Unknown { .. }
            | CompatDataAssessment::Invalid { .. }
            | CompatDataAssessment::Unreadable { .. },
        ) => RequirementState::Unknown,
        (_, CompatDataAssessment::NotRequired) => RequirementState::NotRequired,
    };
    let prefix_state = match (policy, prefix) {
        (CompatibilityRuntimePolicy::NotRequired, _) => RequirementState::NotRequired,
        (CompatibilityRuntimePolicy::Unknown, _) => RequirementState::Unknown,
        (_, PrefixAssessment::CandidateFound { .. }) => RequirementState::Satisfied,
        (_, PrefixAssessment::NotInitialized { .. }) => RequirementState::Missing,
        (
            _,
            PrefixAssessment::Unknown { .. }
            | PrefixAssessment::Invalid { .. }
            | PrefixAssessment::Unreadable { .. },
        ) => RequirementState::Unknown,
        (_, PrefixAssessment::NotRequired) => RequirementState::NotRequired,
    };
    vec![
        LaunchRequirementStatus {
            requirement: LaunchRequirementKind::GameInstallationPresent,
            state: installation_state,
        },
        LaunchRequirementStatus {
            requirement: LaunchRequirementKind::ProtonCandidateAvailable,
            state: candidate_state,
        },
        LaunchRequirementStatus {
            requirement: LaunchRequirementKind::RuntimeSelectionDetermined,
            state: selection_state,
        },
        LaunchRequirementStatus {
            requirement: LaunchRequirementKind::CompatDataInitialized,
            state: compatdata_state,
        },
        LaunchRequirementStatus {
            requirement: LaunchRequirementKind::PrefixAvailable,
            state: prefix_state,
        },
    ]
}

fn assess_readiness(
    policy: CompatibilityRuntimePolicy,
    discovery_status: SteamGameDiscoveryStatus,
    installation: &GameInstallationAssessment,
    candidate_discovery: RuntimeCandidateDiscovery,
    candidates: &[RuntimeCandidate],
    selection: &RuntimeSelection,
    prefix: &PrefixAssessment,
) -> RuntimeReadiness {
    match installation {
        GameInstallationAssessment::NotFound { .. } => {
            return if discovery_status == SteamGameDiscoveryStatus::Complete {
                RuntimeReadiness::Blocked
            } else {
                RuntimeReadiness::Unknown
            };
        }
        GameInstallationAssessment::Unknown { .. } => return RuntimeReadiness::Unknown,
        GameInstallationAssessment::Detected {
            directory_status, ..
        } => match directory_status {
            GameInstallationStatus::DirectoryMissing | GameInstallationStatus::DirectoryInvalid => {
                return RuntimeReadiness::Blocked;
            }
            GameInstallationStatus::PermissionDenied => return RuntimeReadiness::Unknown,
            GameInstallationStatus::Installed => {}
        },
    }

    match policy {
        CompatibilityRuntimePolicy::Unknown => RuntimeReadiness::Unknown,
        CompatibilityRuntimePolicy::NotRequired => RuntimeReadiness::Ready,
        CompatibilityRuntimePolicy::Expected => {
            if candidates.is_empty() {
                return match candidate_discovery {
                    RuntimeCandidateDiscovery::Complete => RuntimeReadiness::Incomplete,
                    RuntimeCandidateDiscovery::Partial | RuntimeCandidateDiscovery::Unavailable => {
                        RuntimeReadiness::Unknown
                    }
                };
            }
            match prefix {
                PrefixAssessment::NotInitialized { .. } => RuntimeReadiness::NeedsInitialization,
                PrefixAssessment::Invalid { .. } => RuntimeReadiness::Blocked,
                PrefixAssessment::Unreadable { .. } => RuntimeReadiness::Unknown,
                PrefixAssessment::Unknown { .. } => RuntimeReadiness::Unknown,
                PrefixAssessment::CandidateFound { .. } => match selection {
                    RuntimeSelection::Selected { candidate }
                        if candidates.iter().any(|available| available == candidate) =>
                    {
                        RuntimeReadiness::Ready
                    }
                    RuntimeSelection::NotRequired => RuntimeReadiness::Ready,
                    RuntimeSelection::Selected { .. } | RuntimeSelection::Unknown { .. } => {
                        RuntimeReadiness::Incomplete
                    }
                },
                PrefixAssessment::NotRequired => RuntimeReadiness::Unknown,
            }
        }
    }
}

fn collect_issues(
    policy: CompatibilityRuntimePolicy,
    installation: &GameInstallationAssessment,
    candidate_discovery: RuntimeCandidateDiscovery,
    candidates: &[RuntimeCandidate],
    selection: &RuntimeSelection,
    compatdata: &CompatDataAssessment,
    prefix: &PrefixAssessment,
) -> Vec<RuntimeIssue> {
    let mut issues = Vec::new();
    match installation {
        GameInstallationAssessment::NotFound { .. } => issues.push(RuntimeIssue {
            code: RuntimeIssueCode::GameNotFound,
            severity: RuntimeIssueSeverity::Warning,
            path: None,
        }),
        GameInstallationAssessment::Unknown { .. } => issues.push(RuntimeIssue {
            code: RuntimeIssueCode::GameDiscoveryIncomplete,
            severity: RuntimeIssueSeverity::Info,
            path: None,
        }),
        GameInstallationAssessment::Detected {
            directory_status: GameInstallationStatus::DirectoryMissing,
            install_path,
            ..
        } => issues.push(RuntimeIssue {
            code: RuntimeIssueCode::InstallationIncompleteOrUnknown,
            severity: RuntimeIssueSeverity::Warning,
            path: Some(install_path.clone()),
        }),
        GameInstallationAssessment::Detected {
            directory_status:
                GameInstallationStatus::DirectoryInvalid | GameInstallationStatus::PermissionDenied,
            install_path,
            ..
        } => issues.push(RuntimeIssue {
            code: RuntimeIssueCode::InstallationIncompleteOrUnknown,
            severity: RuntimeIssueSeverity::Warning,
            path: Some(install_path.clone()),
        }),
        _ => {}
    }
    match candidate_discovery {
        RuntimeCandidateDiscovery::Unavailable => issues.push(RuntimeIssue {
            code: RuntimeIssueCode::RuntimeDiscoveryUnavailable,
            severity: RuntimeIssueSeverity::Info,
            path: None,
        }),
        RuntimeCandidateDiscovery::Partial => issues.push(RuntimeIssue {
            code: RuntimeIssueCode::RuntimeDiscoveryIncomplete,
            severity: RuntimeIssueSeverity::Warning,
            path: None,
        }),
        RuntimeCandidateDiscovery::Complete
            if candidates.is_empty() && policy == CompatibilityRuntimePolicy::Expected =>
        {
            issues.push(RuntimeIssue {
                code: RuntimeIssueCode::NoProtonCandidates,
                severity: RuntimeIssueSeverity::Warning,
                path: None,
            })
        }
        RuntimeCandidateDiscovery::Complete => {}
    }
    if let RuntimeSelection::Unknown { .. } = selection {
        issues.push(RuntimeIssue {
            code: RuntimeIssueCode::RuntimeSelectionUnknown,
            severity: RuntimeIssueSeverity::Info,
            path: None,
        });
    }
    match compatdata {
        CompatDataAssessment::NotFound { expected_path } => {
            if policy == CompatibilityRuntimePolicy::Expected {
                issues.push(RuntimeIssue {
                    code: RuntimeIssueCode::CompatDataNotFound,
                    severity: RuntimeIssueSeverity::Info,
                    path: Some(expected_path.clone()),
                });
            }
        }
        CompatDataAssessment::Unreadable { path, .. } => issues.push(RuntimeIssue {
            code: RuntimeIssueCode::CompatDataUnreadable,
            severity: RuntimeIssueSeverity::Warning,
            path: Some(path.clone()),
        }),
        CompatDataAssessment::Invalid { path } => issues.push(RuntimeIssue {
            code: RuntimeIssueCode::CompatDataInvalid,
            severity: RuntimeIssueSeverity::Warning,
            path: Some(path.clone()),
        }),
        CompatDataAssessment::Found { .. }
        | CompatDataAssessment::Unknown { .. }
        | CompatDataAssessment::NotRequired => {}
    }
    if policy == CompatibilityRuntimePolicy::Expected {
        if let PrefixAssessment::NotInitialized { expected_path } = prefix {
            issues.push(RuntimeIssue {
                code: RuntimeIssueCode::PrefixNotInitialized,
                severity: RuntimeIssueSeverity::Info,
                path: Some(expected_path.clone()),
            });
        }
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::{
        plan_game, CompatDataAssessment, GameInstallationAssessment, PrefixAssessment,
        RuntimeIssueCode, RuntimeReadiness, RuntimeSelection,
    };
    use lxmi_core::{
        GameDistribution, GameExecutableStatus, GameInstallation, GameInstallationStatus,
        ProtonCompatData, ProtonCompatDataStatus, SUPPORTED_GAMES, WUTHERING_WAVES_APP_ID,
    };
    use lxmi_proton::{
        CompatibilityTool, CompatibilityToolDiscoveryResult, CompatibilityToolDiscoveryStatus,
        CompatibilityToolKind, CompatibilityToolSource, CompatibilityToolStatus,
    };
    use lxmi_steam::{SteamGameDiscoveryStatus, SteamGameInstallation};
    use std::path::PathBuf;

    fn proton_tool(name: &str, path: &str) -> CompatibilityTool {
        CompatibilityTool {
            internal_id: Some(name.to_ascii_lowercase().replace(' ', "-")),
            display_name: name.to_owned(),
            path: PathBuf::from(path),
            metadata_path: None,
            source: CompatibilityToolSource::SteamLibrary,
            kind: CompatibilityToolKind::Proton,
            version: Some("synthetic-version".to_owned()),
            status: CompatibilityToolStatus::Valid,
        }
    }

    fn tool_discovery(tools: Vec<CompatibilityTool>) -> CompatibilityToolDiscoveryResult {
        CompatibilityToolDiscoveryResult {
            status: CompatibilityToolDiscoveryStatus::Complete,
            tools,
            issues: Vec::new(),
        }
    }

    fn installed_game(status: ProtonCompatDataStatus) -> SteamGameInstallation {
        let compatdata_path = PathBuf::from("/synthetic/steamapps/compatdata/3513350");
        SteamGameInstallation {
            installation: GameInstallation {
                game: SUPPORTED_GAMES[0],
                install_path: PathBuf::from("/synthetic/steamapps/common/Wuthering Waves"),
                status: GameInstallationStatus::Installed,
                distribution: GameDistribution::Steam,
                executable_path: None,
                executable_status: GameExecutableStatus::NotScanned,
            },
            steam_app_id: WUTHERING_WAVES_APP_ID,
            manifest_name: "Wuthering Waves (synthetic)".to_owned(),
            steam_library: PathBuf::from("/synthetic/Steam"),
            compatdata: ProtonCompatData {
                app_id: WUTHERING_WAVES_APP_ID,
                compatdata_path,
                prefix_path: (status == ProtonCompatDataStatus::PrefixFound)
                    .then(|| PathBuf::from("/synthetic/steamapps/compatdata/3513350/pfx")),
                status,
            },
        }
    }

    fn plan(
        game_installation: Option<&SteamGameInstallation>,
        game_status: SteamGameDiscoveryStatus,
        tools: &CompatibilityToolDiscoveryResult,
    ) -> super::GameRuntimePlan {
        plan_game(SUPPORTED_GAMES[0], game_status, game_installation, tools)
    }

    #[test]
    fn installed_game_with_proton_candidate_and_no_compatdata_needs_initialization() {
        let installation = installed_game(ProtonCompatDataStatus::NotFound);
        let tools = tool_discovery(vec![proton_tool(
            "Proton Experimental",
            "/synthetic/Proton Experimental",
        )]);

        let result = plan(
            Some(&installation),
            SteamGameDiscoveryStatus::Complete,
            &tools,
        );

        assert_eq!(result.readiness, RuntimeReadiness::NeedsInitialization);
        assert!(matches!(
            result.compatdata,
            CompatDataAssessment::NotFound { .. }
        ));
        assert!(matches!(
            result.prefix,
            PrefixAssessment::NotInitialized { .. }
        ));
    }

    #[test]
    fn multiple_candidates_do_not_create_a_runtime_selection() {
        let installation = installed_game(ProtonCompatDataStatus::NotFound);
        let tools = tool_discovery(vec![
            proton_tool("Proton Experimental", "/synthetic/proton-experimental"),
            proton_tool("GE-Proton 9-23", "/synthetic/ge-proton-9-23"),
        ]);

        let result = plan(
            Some(&installation),
            SteamGameDiscoveryStatus::Complete,
            &tools,
        );

        assert_eq!(result.available_runtime_candidates.len(), 2);
        assert!(matches!(result.selection, RuntimeSelection::Unknown { .. }));
    }

    #[test]
    fn prefix_found_does_not_prove_selected_runtime_or_readiness() {
        let installation = installed_game(ProtonCompatDataStatus::PrefixFound);
        let tools = tool_discovery(vec![proton_tool(
            "Proton Experimental",
            "/synthetic/proton",
        )]);

        let result = plan(
            Some(&installation),
            SteamGameDiscoveryStatus::Complete,
            &tools,
        );

        assert!(matches!(
            result.compatdata,
            CompatDataAssessment::Found { .. }
        ));
        assert!(matches!(
            result.prefix,
            PrefixAssessment::CandidateFound { .. }
        ));
        assert!(matches!(result.selection, RuntimeSelection::Unknown { .. }));
        assert_eq!(result.readiness, RuntimeReadiness::Incomplete);
    }

    #[test]
    fn complete_discovery_without_game_blocks_the_plan() {
        let tools = tool_discovery(vec![proton_tool(
            "Proton Experimental",
            "/synthetic/proton",
        )]);
        let result = plan(None, SteamGameDiscoveryStatus::Complete, &tools);

        assert_eq!(result.readiness, RuntimeReadiness::Blocked);
        assert!(matches!(
            result.installation,
            GameInstallationAssessment::NotFound { .. }
        ));
    }

    #[test]
    fn missing_proton_candidates_make_the_plan_incomplete() {
        let installation = installed_game(ProtonCompatDataStatus::NotFound);
        let tools = tool_discovery(Vec::new());
        let result = plan(
            Some(&installation),
            SteamGameDiscoveryStatus::Complete,
            &tools,
        );

        assert_eq!(result.readiness, RuntimeReadiness::Incomplete);
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == RuntimeIssueCode::NoProtonCandidates));
    }

    #[test]
    fn unknown_compatibility_tools_are_not_proton_candidates() {
        let installation = installed_game(ProtonCompatDataStatus::NotFound);
        let mut unknown = proton_tool("Unknown tool", "/synthetic/unknown");
        unknown.kind = CompatibilityToolKind::Unknown;
        let result = plan(
            Some(&installation),
            SteamGameDiscoveryStatus::Complete,
            &tool_discovery(vec![unknown]),
        );

        assert!(result.available_runtime_candidates.is_empty());
        assert_eq!(result.readiness, RuntimeReadiness::Incomplete);
    }

    #[test]
    fn steam_linux_runtime_is_not_a_proton_candidate() {
        let installation = installed_game(ProtonCompatDataStatus::NotFound);
        let mut runtime = proton_tool("Steam Linux Runtime", "/synthetic/slr");
        runtime.kind = CompatibilityToolKind::SteamLinuxRuntime;
        let result = plan(
            Some(&installation),
            SteamGameDiscoveryStatus::Complete,
            &tool_discovery(vec![runtime]),
        );

        assert!(result.available_runtime_candidates.is_empty());
        assert_eq!(result.readiness, RuntimeReadiness::Incomplete);
    }

    #[test]
    fn unreadable_compatdata_is_preserved_as_unknown() {
        let installation = installed_game(ProtonCompatDataStatus::PermissionDenied);
        let tools = tool_discovery(vec![proton_tool(
            "Proton Experimental",
            "/synthetic/proton",
        )]);
        let result = plan(
            Some(&installation),
            SteamGameDiscoveryStatus::Complete,
            &tools,
        );

        assert!(matches!(
            result.compatdata,
            CompatDataAssessment::Unreadable {
                status: ProtonCompatDataStatus::PermissionDenied,
                ..
            }
        ));
        assert!(matches!(result.prefix, PrefixAssessment::Unreadable { .. }));
        assert_eq!(result.readiness, RuntimeReadiness::Unknown);
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == RuntimeIssueCode::CompatDataUnreadable));
    }

    #[test]
    fn incomplete_game_discovery_does_not_claim_that_game_is_missing() {
        let result = plan(
            None,
            SteamGameDiscoveryStatus::Partial,
            &tool_discovery(Vec::new()),
        );

        assert_eq!(result.readiness, RuntimeReadiness::Unknown);
        assert!(matches!(
            result.installation,
            GameInstallationAssessment::Unknown { .. }
        ));
    }

    #[test]
    fn single_proton_candidate_does_not_imply_selected_runtime() {
        let installation = installed_game(ProtonCompatDataStatus::NotFound);
        let tools = tool_discovery(vec![proton_tool(
            "Proton Experimental",
            "/synthetic/proton-experimental",
        )]);

        let result = plan(
            Some(&installation),
            SteamGameDiscoveryStatus::Complete,
            &tools,
        );

        assert_eq!(result.available_runtime_candidates.len(), 1);
        assert_eq!(
            result.selection,
            RuntimeSelection::Unknown {
                reason: super::RuntimeSelectionUnknownReason::NoReliablePerGameSelectionEvidence,
            }
        );
    }
}
