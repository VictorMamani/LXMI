use serde::Serialize;
use std::path::Path;

use lxmi_core::{
    GameDistribution, GameExecutableStatus, GameInstallationStatus, ProtonCompatDataStatus,
    SteamDetectionIssue, SteamDetectionIssueCode, SteamDetectionStatus, SteamInstallation,
    SteamLibrary, SteamScanResult, SystemInfo,
};
use lxmi_proton::{
    CompatibilityTool, CompatibilityToolDiscoveryResult, CompatibilityToolDiscoveryStatus,
    CompatibilityToolIssue, CompatibilityToolIssueCode, CompatibilityToolIssueSeverity,
    CompatibilityToolKind, CompatibilityToolSource, CompatibilityToolStatus, ProtonScanner,
};
use lxmi_runtime::{
    CompatDataAssessment, GameInstallationAssessment, GameInstallationCompleteness,
    GameRuntimePlan, LaunchRequirementStatus, PrefixAssessment, PrefixEvidenceState,
    RequirementState, RuntimeCandidate, RuntimeCandidateDiscovery, RuntimeEvidence, RuntimeIssue,
    RuntimeIssueSeverity, RuntimePlatform, RuntimeReadiness, RuntimeSelection,
    RuntimeSelectionUnknownReason,
};
use lxmi_steam::{
    SteamAppScanIssue, SteamAppScanIssueCode, SteamAppScanIssueSeverity, SteamDiscoveryResult,
    SteamDiscoveryScanner, SteamGameDiscoveryResult, SteamGameDiscoveryStatus,
    SteamGameInstallation,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfoDto {
    os: String,
    architecture: String,
    home_directory: Option<String>,
    xdg_data_home: Option<String>,
    xdg_data_dirs: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamScanDto {
    status: &'static str,
    installations: Vec<SteamInstallationDto>,
    issues: Vec<SteamIssueDto>,
    game_scan_status: &'static str,
    manifests_parsed: usize,
    ignored_unknown_apps: usize,
    games: Vec<SteamGameDto>,
    game_issues: Vec<SteamGameIssueDto>,
    compatibility_tools_status: &'static str,
    compatibility_tools: Vec<CompatibilityToolDto>,
    compatibility_tool_issues: Vec<CompatibilityToolIssueDto>,
    runtime_plans: Vec<RuntimePlanDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimePlanDto {
    game_id: &'static str,
    game_name: &'static str,
    expected_steam_app_ids: Vec<u32>,
    platform: &'static str,
    runtime_policy: &'static str,
    installation: RuntimeInstallationAssessmentDto,
    selection: RuntimeSelectionDto,
    candidate_discovery: &'static str,
    available_runtime_candidates: Vec<RuntimeCandidateDto>,
    compatdata: CompatDataAssessmentDto,
    prefix: PrefixAssessmentDto,
    readiness: &'static str,
    requirements: Vec<LaunchRequirementDto>,
    evidence: Vec<RuntimeEvidenceDto>,
    issues: Vec<RuntimeIssueDto>,
    compatibility_tool_issues: Vec<CompatibilityToolIssueDto>,
}

#[derive(Serialize)]
#[serde(
    tag = "state",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
enum RuntimeInstallationAssessmentDto {
    NotFound {
        expected_steam_app_ids: Vec<u32>,
    },
    Detected {
        steam_app_id: u32,
        manifest_name: String,
        steam_library: String,
        install_path: String,
        distribution: &'static str,
        executable_path: Option<String>,
        executable_status: &'static str,
        directory_status: &'static str,
        completeness: &'static str,
    },
    Unknown {
        discovery_status: &'static str,
    },
}

#[derive(Serialize)]
#[serde(
    tag = "state",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
enum RuntimeSelectionDto {
    Unknown { reason: &'static str },
    Selected { candidate: RuntimeCandidateDto },
    NotRequired,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCandidateDto {
    internal_id: Option<String>,
    display_name: String,
    path: String,
    source: &'static str,
    version: Option<String>,
}

#[derive(Serialize)]
#[serde(
    tag = "state",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
enum CompatDataAssessmentDto {
    NotRequired,
    Unknown { discovery_status: &'static str },
    NotFound { expected_path: String },
    Found { path: String },
    Invalid { path: String },
    Unreadable { path: String, status: &'static str },
}

#[derive(Serialize)]
#[serde(
    tag = "state",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
enum PrefixAssessmentDto {
    NotRequired,
    Unknown { discovery_status: &'static str },
    NotInitialized { expected_path: String },
    CandidateFound { path: String },
    Invalid { path: String },
    Unreadable { path: String, status: &'static str },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LaunchRequirementDto {
    requirement: &'static str,
    state: &'static str,
}

#[derive(Serialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
enum RuntimeEvidenceDto {
    GameDiscoveryEvaluated {
        status: &'static str,
    },
    GameManifestObserved {
        app_id: u32,
        name: String,
        library: String,
    },
    GameDirectoryObserved {
        path: String,
        status: &'static str,
        completeness: &'static str,
    },
    RuntimeDiscoveryEvaluated {
        status: &'static str,
        proton_candidate_count: usize,
    },
    ProtonCandidateObserved {
        display_name: String,
        path: String,
    },
    RuntimeSelectionNotObserved {
        reason: &'static str,
    },
    CompatDataObserved {
        path: String,
        status: &'static str,
    },
    PrefixObserved {
        expected_path: String,
        state: &'static str,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeIssueDto {
    code: &'static str,
    severity: &'static str,
    path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityToolDto {
    internal_id: Option<String>,
    display_name: String,
    path: String,
    metadata_path: Option<String>,
    source: &'static str,
    kind: &'static str,
    version: Option<String>,
    status: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityToolIssueDto {
    code: &'static str,
    severity: &'static str,
    path: String,
    detail: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamGameDto {
    id: &'static str,
    name: &'static str,
    steam_app_id: u32,
    manifest_name: String,
    install_path: String,
    install_status: &'static str,
    distribution: &'static str,
    executable_path: Option<String>,
    executable_status: &'static str,
    steam_library: String,
    compatdata_status: &'static str,
    compatdata_path: String,
    prefix_path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamGameIssueDto {
    code: &'static str,
    severity: &'static str,
    path: String,
    detail: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamInstallationDto {
    root_path: String,
    libraries: Vec<SteamLibraryDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamLibraryDto {
    path: String,
    is_default: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamIssueDto {
    code: &'static str,
    path: Option<String>,
    detail: Option<String>,
}

#[tauri::command]
pub fn get_system_info() -> SystemInfoDto {
    SystemInfoDto::from(SystemInfo::current())
}

#[tauri::command]
pub async fn scan_steam() -> SteamScanDto {
    match tauri::async_runtime::spawn_blocking(scan_steam_snapshot).await {
        Ok(result) => result,
        Err(error) => (
            SteamDiscoveryResult {
                steam: SteamScanResult {
                    status: SteamDetectionStatus::InternalError,
                    installations: Vec::new(),
                    issues: vec![SteamDetectionIssue {
                        code: SteamDetectionIssueCode::FilesystemError,
                        path: None,
                        detail: Some(error.to_string()),
                    }],
                },
                games: SteamGameDiscoveryResult {
                    status: SteamGameDiscoveryStatus::NotAvailable,
                    manifests_parsed: 0,
                    ignored_unknown_apps: 0,
                    games: Vec::new(),
                    issues: Vec::new(),
                },
            },
            CompatibilityToolDiscoveryResult::not_available(),
            Vec::new(),
        )
            .into(),
    }
}

fn scan_steam_snapshot() -> SteamScanDto {
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let compatibility_tools = ProtonScanner.scan(&discovery.steam.installations);
    let runtime_plans = lxmi_runtime::RuntimePlanner::plan_all(&discovery, &compatibility_tools);
    SteamScanDto::from((discovery, compatibility_tools, runtime_plans))
}

impl From<SystemInfo> for SystemInfoDto {
    fn from(info: SystemInfo) -> Self {
        Self {
            os: match info.os.as_str() {
                "linux" => "Linux".to_owned(),
                other => other.to_owned(),
            },
            architecture: info.architecture,
            home_directory: info.home_directory.as_deref().map(path_to_string),
            xdg_data_home: info.xdg_data_home.as_deref().map(path_to_string),
            xdg_data_dirs: info
                .xdg_data_dirs
                .iter()
                .map(|path| path_to_string(path))
                .collect(),
        }
    }
}

impl
    From<(
        SteamDiscoveryResult,
        CompatibilityToolDiscoveryResult,
        Vec<GameRuntimePlan>,
    )> for SteamScanDto
{
    fn from(
        (discovery, compatibility_tools, runtime_plans): (
            SteamDiscoveryResult,
            CompatibilityToolDiscoveryResult,
            Vec<GameRuntimePlan>,
        ),
    ) -> Self {
        Self {
            status: status_code(&discovery.steam.status),
            installations: discovery
                .steam
                .installations
                .iter()
                .map(SteamInstallationDto::from)
                .collect(),
            issues: discovery
                .steam
                .issues
                .iter()
                .map(SteamIssueDto::from)
                .collect(),
            game_scan_status: game_scan_status_code(&discovery.games.status),
            manifests_parsed: discovery.games.manifests_parsed,
            ignored_unknown_apps: discovery.games.ignored_unknown_apps,
            games: discovery
                .games
                .games
                .iter()
                .map(SteamGameDto::from)
                .collect(),
            game_issues: discovery
                .games
                .issues
                .iter()
                .map(SteamGameIssueDto::from)
                .collect(),
            compatibility_tools_status: compatibility_tools_status_code(
                &compatibility_tools.status,
            ),
            compatibility_tools: compatibility_tools
                .tools
                .iter()
                .map(CompatibilityToolDto::from)
                .collect(),
            compatibility_tool_issues: compatibility_tools
                .issues
                .iter()
                .map(CompatibilityToolIssueDto::from)
                .collect(),
            runtime_plans: runtime_plans.iter().map(RuntimePlanDto::from).collect(),
        }
    }
}

impl From<&GameRuntimePlan> for RuntimePlanDto {
    fn from(plan: &GameRuntimePlan) -> Self {
        Self {
            game_id: plan.game.id,
            game_name: plan.game.name,
            expected_steam_app_ids: plan.game.steam_app_ids.to_vec(),
            platform: match plan.platform {
                RuntimePlatform::Steam => "steam",
            },
            runtime_policy: match plan.runtime_policy {
                lxmi_core::CompatibilityRuntimePolicy::Expected => "expected",
                lxmi_core::CompatibilityRuntimePolicy::NotRequired => "not_required",
                lxmi_core::CompatibilityRuntimePolicy::Unknown => "unknown",
            },
            installation: RuntimeInstallationAssessmentDto::from(&plan.installation),
            selection: RuntimeSelectionDto::from(&plan.selection),
            candidate_discovery: candidate_discovery_code(plan.candidate_discovery),
            available_runtime_candidates: plan
                .available_runtime_candidates
                .iter()
                .map(RuntimeCandidateDto::from)
                .collect(),
            compatdata: CompatDataAssessmentDto::from(&plan.compatdata),
            prefix: PrefixAssessmentDto::from(&plan.prefix),
            readiness: runtime_readiness_code(plan.readiness),
            requirements: plan
                .requirements
                .iter()
                .map(LaunchRequirementDto::from)
                .collect(),
            evidence: plan.evidence.iter().map(RuntimeEvidenceDto::from).collect(),
            issues: plan.issues.iter().map(RuntimeIssueDto::from).collect(),
            compatibility_tool_issues: plan
                .compatibility_tool_issues
                .iter()
                .map(CompatibilityToolIssueDto::from)
                .collect(),
        }
    }
}

impl From<&GameInstallationAssessment> for RuntimeInstallationAssessmentDto {
    fn from(state: &GameInstallationAssessment) -> Self {
        match state {
            GameInstallationAssessment::NotFound {
                expected_steam_app_ids,
            } => Self::NotFound {
                expected_steam_app_ids: expected_steam_app_ids.clone(),
            },
            GameInstallationAssessment::Detected {
                steam_app_id,
                manifest_name,
                steam_library,
                install_path,
                distribution,
                executable_path,
                executable_status,
                directory_status,
                completeness,
            } => Self::Detected {
                steam_app_id: *steam_app_id,
                manifest_name: manifest_name.clone(),
                steam_library: path_to_string(steam_library),
                install_path: path_to_string(install_path),
                distribution: game_distribution_code(*distribution),
                executable_path: executable_path.as_deref().map(path_to_string),
                executable_status: game_executable_status_code(*executable_status),
                directory_status: installation_status_code(directory_status),
                completeness: installation_completeness_code(*completeness),
            },
            GameInstallationAssessment::Unknown { discovery_status } => Self::Unknown {
                discovery_status: game_scan_status_code(discovery_status),
            },
        }
    }
}

impl From<&RuntimeSelection> for RuntimeSelectionDto {
    fn from(selection: &RuntimeSelection) -> Self {
        match selection {
            RuntimeSelection::Unknown { reason } => Self::Unknown {
                reason: runtime_selection_unknown_reason_code(*reason),
            },
            RuntimeSelection::Selected { candidate } => Self::Selected {
                candidate: RuntimeCandidateDto::from(candidate),
            },
            RuntimeSelection::NotRequired => Self::NotRequired,
        }
    }
}

impl From<&RuntimeCandidate> for RuntimeCandidateDto {
    fn from(candidate: &RuntimeCandidate) -> Self {
        Self {
            internal_id: candidate.internal_id.clone(),
            display_name: candidate.display_name.clone(),
            path: path_to_string(&candidate.path),
            source: compatibility_tool_source_code(&candidate.source),
            version: candidate.version.clone(),
        }
    }
}

impl From<&CompatDataAssessment> for CompatDataAssessmentDto {
    fn from(state: &CompatDataAssessment) -> Self {
        match state {
            CompatDataAssessment::NotRequired => Self::NotRequired,
            CompatDataAssessment::Unknown { discovery_status } => Self::Unknown {
                discovery_status: game_scan_status_code(discovery_status),
            },
            CompatDataAssessment::NotFound { expected_path } => Self::NotFound {
                expected_path: path_to_string(expected_path),
            },
            CompatDataAssessment::Found { path } => Self::Found {
                path: path_to_string(path),
            },
            CompatDataAssessment::Invalid { path } => Self::Invalid {
                path: path_to_string(path),
            },
            CompatDataAssessment::Unreadable { path, status } => Self::Unreadable {
                path: path_to_string(path),
                status: compatdata_status_code(status),
            },
        }
    }
}

impl From<&PrefixAssessment> for PrefixAssessmentDto {
    fn from(state: &PrefixAssessment) -> Self {
        match state {
            PrefixAssessment::NotRequired => Self::NotRequired,
            PrefixAssessment::Unknown { discovery_status } => Self::Unknown {
                discovery_status: game_scan_status_code(discovery_status),
            },
            PrefixAssessment::NotInitialized { expected_path } => Self::NotInitialized {
                expected_path: path_to_string(expected_path),
            },
            PrefixAssessment::CandidateFound { path } => Self::CandidateFound {
                path: path_to_string(path),
            },
            PrefixAssessment::Invalid { path } => Self::Invalid {
                path: path_to_string(path),
            },
            PrefixAssessment::Unreadable { path, status } => Self::Unreadable {
                path: path_to_string(path),
                status: compatdata_status_code(status),
            },
        }
    }
}

impl From<&LaunchRequirementStatus> for LaunchRequirementDto {
    fn from(requirement: &LaunchRequirementStatus) -> Self {
        Self {
            requirement: match requirement.requirement {
                lxmi_runtime::LaunchRequirementKind::GameInstallationPresent => {
                    "game_installation_present"
                }
                lxmi_runtime::LaunchRequirementKind::ProtonCandidateAvailable => {
                    "proton_candidate_available"
                }
                lxmi_runtime::LaunchRequirementKind::RuntimeSelectionDetermined => {
                    "runtime_selection_determined"
                }
                lxmi_runtime::LaunchRequirementKind::CompatDataInitialized => {
                    "compatdata_initialized"
                }
                lxmi_runtime::LaunchRequirementKind::PrefixAvailable => "prefix_available",
            },
            state: requirement_state_code(requirement.state),
        }
    }
}

impl From<&RuntimeEvidence> for RuntimeEvidenceDto {
    fn from(evidence: &RuntimeEvidence) -> Self {
        match evidence {
            RuntimeEvidence::GameDiscoveryEvaluated { status } => Self::GameDiscoveryEvaluated {
                status: game_scan_status_code(status),
            },
            RuntimeEvidence::GameManifestObserved {
                app_id,
                name,
                library,
            } => Self::GameManifestObserved {
                app_id: *app_id,
                name: name.clone(),
                library: path_to_string(library),
            },
            RuntimeEvidence::GameDirectoryObserved {
                path,
                status,
                completeness,
            } => Self::GameDirectoryObserved {
                path: path_to_string(path),
                status: installation_status_code(status),
                completeness: installation_completeness_code(*completeness),
            },
            RuntimeEvidence::RuntimeDiscoveryEvaluated {
                status,
                proton_candidate_count,
            } => Self::RuntimeDiscoveryEvaluated {
                status: candidate_discovery_code(*status),
                proton_candidate_count: *proton_candidate_count,
            },
            RuntimeEvidence::ProtonCandidateObserved { display_name, path } => {
                Self::ProtonCandidateObserved {
                    display_name: display_name.clone(),
                    path: path_to_string(path),
                }
            }
            RuntimeEvidence::RuntimeSelectionNotObserved { reason } => {
                Self::RuntimeSelectionNotObserved {
                    reason: runtime_selection_unknown_reason_code(*reason),
                }
            }
            RuntimeEvidence::CompatDataObserved { path, status } => Self::CompatDataObserved {
                path: path_to_string(path),
                status: compatdata_status_code(status),
            },
            RuntimeEvidence::PrefixObserved {
                expected_path,
                state,
            } => Self::PrefixObserved {
                expected_path: path_to_string(expected_path),
                state: prefix_evidence_state_code(*state),
            },
        }
    }
}

impl From<&RuntimeIssue> for RuntimeIssueDto {
    fn from(issue: &RuntimeIssue) -> Self {
        Self {
            code: runtime_issue_code(issue.code),
            severity: match issue.severity {
                RuntimeIssueSeverity::Info => "info",
                RuntimeIssueSeverity::Warning => "warning",
                RuntimeIssueSeverity::Error => "error",
            },
            path: issue.path.as_deref().map(path_to_string),
        }
    }
}

impl From<&CompatibilityTool> for CompatibilityToolDto {
    fn from(tool: &CompatibilityTool) -> Self {
        Self {
            internal_id: tool.internal_id.clone(),
            display_name: tool.display_name.clone(),
            path: path_to_string(&tool.path),
            metadata_path: tool.metadata_path.as_deref().map(path_to_string),
            source: compatibility_tool_source_code(&tool.source),
            kind: compatibility_tool_kind_code(&tool.kind),
            version: tool.version.clone(),
            status: compatibility_tool_status_code(&tool.status),
        }
    }
}

impl From<&CompatibilityToolIssue> for CompatibilityToolIssueDto {
    fn from(issue: &CompatibilityToolIssue) -> Self {
        Self {
            code: compatibility_tool_issue_code(&issue.code),
            severity: match issue.severity {
                CompatibilityToolIssueSeverity::Info => "info",
                CompatibilityToolIssueSeverity::Warning => "warning",
                CompatibilityToolIssueSeverity::Error => "error",
            },
            path: path_to_string(&issue.path),
            detail: issue.detail.clone(),
        }
    }
}

impl From<&SteamGameInstallation> for SteamGameDto {
    fn from(game: &SteamGameInstallation) -> Self {
        Self {
            id: game.installation.game.id,
            name: game.installation.game.name,
            steam_app_id: game.steam_app_id,
            manifest_name: game.manifest_name.clone(),
            install_path: path_to_string(&game.installation.install_path),
            install_status: installation_status_code(&game.installation.status),
            distribution: game_distribution_code(game.installation.distribution),
            executable_path: game
                .installation
                .executable_path
                .as_deref()
                .map(path_to_string),
            executable_status: game_executable_status_code(game.installation.executable_status),
            steam_library: path_to_string(&game.steam_library),
            compatdata_status: compatdata_status_code(&game.compatdata.status),
            compatdata_path: path_to_string(&game.compatdata.compatdata_path),
            prefix_path: game.compatdata.prefix_path.as_deref().map(path_to_string),
        }
    }
}

impl From<&SteamAppScanIssue> for SteamGameIssueDto {
    fn from(issue: &SteamAppScanIssue) -> Self {
        Self {
            code: game_issue_code(&issue.code),
            severity: match issue.severity {
                SteamAppScanIssueSeverity::Warning => "warning",
                SteamAppScanIssueSeverity::Error => "error",
            },
            path: path_to_string(&issue.path),
            detail: issue.detail.clone(),
        }
    }
}

impl From<&SteamInstallation> for SteamInstallationDto {
    fn from(installation: &SteamInstallation) -> Self {
        Self {
            root_path: path_to_string(&installation.root_path),
            libraries: installation
                .libraries
                .iter()
                .map(SteamLibraryDto::from)
                .collect(),
        }
    }
}

impl From<&SteamLibrary> for SteamLibraryDto {
    fn from(library: &SteamLibrary) -> Self {
        Self {
            path: path_to_string(&library.path),
            is_default: library.is_default,
        }
    }
}

impl From<&SteamDetectionIssue> for SteamIssueDto {
    fn from(issue: &SteamDetectionIssue) -> Self {
        Self {
            code: issue_code(&issue.code),
            path: issue.path.as_deref().map(path_to_string),
            detail: issue.detail.clone(),
        }
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn status_code(status: &SteamDetectionStatus) -> &'static str {
    match status {
        SteamDetectionStatus::NotInstalled => "not_installed",
        SteamDetectionStatus::Detected => "detected",
        SteamDetectionStatus::ConfigurationMissing => "configuration_missing",
        SteamDetectionStatus::InvalidConfiguration => "invalid_configuration",
        SteamDetectionStatus::PermissionDenied => "permission_denied",
        SteamDetectionStatus::HomeDirectoryUnavailable => "home_directory_unavailable",
        SteamDetectionStatus::InternalError => "internal_error",
    }
}

fn issue_code(code: &SteamDetectionIssueCode) -> &'static str {
    match code {
        SteamDetectionIssueCode::SteamRootUnreadable => "steam_root_unreadable",
        SteamDetectionIssueCode::SteamappsMissing => "steamapps_missing",
        SteamDetectionIssueCode::ConfigurationMissing => "configuration_missing",
        SteamDetectionIssueCode::ConfigurationUnreadable => "configuration_unreadable",
        SteamDetectionIssueCode::ConfigurationNotRegularFile => "configuration_not_regular_file",
        SteamDetectionIssueCode::InvalidConfiguration => "invalid_configuration",
        SteamDetectionIssueCode::InvalidLibraryPath => "invalid_library_path",
        SteamDetectionIssueCode::LibraryMissing => "library_missing",
        SteamDetectionIssueCode::LibraryPermissionDenied => "library_permission_denied",
        SteamDetectionIssueCode::FilesystemError => "filesystem_error",
    }
}

fn game_scan_status_code(status: &SteamGameDiscoveryStatus) -> &'static str {
    match status {
        SteamGameDiscoveryStatus::NotAvailable => "not_available",
        SteamGameDiscoveryStatus::Complete => "complete",
        SteamGameDiscoveryStatus::Partial => "partial",
    }
}

fn installation_status_code(status: &GameInstallationStatus) -> &'static str {
    match status {
        GameInstallationStatus::Installed => "installed",
        GameInstallationStatus::DirectoryMissing => "directory_missing",
        GameInstallationStatus::DirectoryInvalid => "directory_invalid",
        GameInstallationStatus::PermissionDenied => "permission_denied",
    }
}

fn game_distribution_code(distribution: GameDistribution) -> &'static str {
    match distribution {
        GameDistribution::Steam => "steam",
        GameDistribution::HoYoPlay => "hoyoplay",
        GameDistribution::Manual => "manual",
        GameDistribution::Unknown => "unknown",
    }
}

fn game_executable_status_code(status: GameExecutableStatus) -> &'static str {
    match status {
        GameExecutableStatus::Found => "found",
        GameExecutableStatus::NotFound => "not_found",
        GameExecutableStatus::SearchIncomplete => "search_incomplete",
        GameExecutableStatus::NotScanned => "not_scanned",
    }
}

fn installation_completeness_code(completeness: GameInstallationCompleteness) -> &'static str {
    match completeness {
        GameInstallationCompleteness::Unknown => "unknown",
    }
}

fn compatdata_status_code(status: &ProtonCompatDataStatus) -> &'static str {
    match status {
        ProtonCompatDataStatus::NotFound => "not_found",
        ProtonCompatDataStatus::CompatDataFound => "compatdata_found",
        ProtonCompatDataStatus::PrefixFound => "prefix_found",
        ProtonCompatDataStatus::Invalid => "invalid",
        ProtonCompatDataStatus::PermissionDenied => "permission_denied",
        ProtonCompatDataStatus::IoError => "io_error",
    }
}

fn game_issue_code(code: &SteamAppScanIssueCode) -> &'static str {
    match code {
        SteamAppScanIssueCode::SteamAppsUnavailable => "steamapps_unavailable",
        SteamAppScanIssueCode::InvalidManifestFilename => "invalid_manifest_filename",
        SteamAppScanIssueCode::ManifestNotRegularFile => "manifest_not_regular_file",
        SteamAppScanIssueCode::ManifestTooLarge => "manifest_too_large",
        SteamAppScanIssueCode::ManifestInvalid => "manifest_invalid",
        SteamAppScanIssueCode::ManifestIdMismatch => "manifest_id_mismatch",
        SteamAppScanIssueCode::GameDirectoryMissing => "game_directory_missing",
        SteamAppScanIssueCode::GameDirectoryInvalid => "game_directory_invalid",
        SteamAppScanIssueCode::GameDirectoryPermissionDenied => "game_directory_permission_denied",
        SteamAppScanIssueCode::PermissionDenied => "permission_denied",
        SteamAppScanIssueCode::CompatDataInvalid => "compatdata_invalid",
        SteamAppScanIssueCode::FilesystemError => "filesystem_error",
    }
}

fn compatibility_tools_status_code(status: &CompatibilityToolDiscoveryStatus) -> &'static str {
    match status {
        CompatibilityToolDiscoveryStatus::NotAvailable => "not_available",
        CompatibilityToolDiscoveryStatus::Complete => "complete",
        CompatibilityToolDiscoveryStatus::Partial => "partial",
    }
}

fn compatibility_tool_source_code(source: &CompatibilityToolSource) -> &'static str {
    match source {
        CompatibilityToolSource::SteamLibrary => "steam_library",
        CompatibilityToolSource::Custom => "custom",
    }
}

fn compatibility_tool_kind_code(kind: &CompatibilityToolKind) -> &'static str {
    match kind {
        CompatibilityToolKind::Proton => "proton",
        CompatibilityToolKind::SteamLinuxRuntime => "steam_linux_runtime",
        CompatibilityToolKind::Other => "other",
        CompatibilityToolKind::Unknown => "unknown",
    }
}

fn compatibility_tool_status_code(status: &CompatibilityToolStatus) -> &'static str {
    match status {
        CompatibilityToolStatus::Valid => "valid",
        CompatibilityToolStatus::Incomplete => "incomplete",
        CompatibilityToolStatus::InvalidMetadata => "invalid_metadata",
        CompatibilityToolStatus::Unreadable => "unreadable",
    }
}

fn candidate_discovery_code(status: RuntimeCandidateDiscovery) -> &'static str {
    match status {
        RuntimeCandidateDiscovery::Complete => "complete",
        RuntimeCandidateDiscovery::Partial => "partial",
        RuntimeCandidateDiscovery::Unavailable => "unavailable",
    }
}

fn runtime_readiness_code(readiness: RuntimeReadiness) -> &'static str {
    match readiness {
        RuntimeReadiness::Ready => "ready",
        RuntimeReadiness::Incomplete => "incomplete",
        RuntimeReadiness::NeedsInitialization => "needs_initialization",
        RuntimeReadiness::Unknown => "unknown",
        RuntimeReadiness::Blocked => "blocked",
    }
}

fn runtime_selection_unknown_reason_code(reason: RuntimeSelectionUnknownReason) -> &'static str {
    match reason {
        RuntimeSelectionUnknownReason::NoReliablePerGameSelectionEvidence => {
            "no_reliable_per_game_selection_evidence"
        }
        RuntimeSelectionUnknownReason::GamePolicyUnknown => "game_policy_unknown",
    }
}

fn requirement_state_code(state: RequirementState) -> &'static str {
    match state {
        RequirementState::Satisfied => "satisfied",
        RequirementState::Missing => "missing",
        RequirementState::Unknown => "unknown",
        RequirementState::NotRequired => "not_required",
    }
}

fn prefix_evidence_state_code(state: PrefixEvidenceState) -> &'static str {
    match state {
        PrefixEvidenceState::FoundCandidate => "found_candidate",
        PrefixEvidenceState::NotFound => "not_found",
        PrefixEvidenceState::Invalid => "invalid",
        PrefixEvidenceState::Unreadable => "unreadable",
        PrefixEvidenceState::Unknown => "unknown",
        PrefixEvidenceState::NotRequired => "not_required",
    }
}

fn runtime_issue_code(code: lxmi_runtime::RuntimeIssueCode) -> &'static str {
    use lxmi_runtime::RuntimeIssueCode;
    match code {
        RuntimeIssueCode::GameNotFound => "game_not_found",
        RuntimeIssueCode::GameDiscoveryIncomplete => "game_discovery_incomplete",
        RuntimeIssueCode::RuntimeDiscoveryUnavailable => "runtime_discovery_unavailable",
        RuntimeIssueCode::RuntimeDiscoveryIncomplete => "runtime_discovery_incomplete",
        RuntimeIssueCode::NoProtonCandidates => "no_proton_candidates",
        RuntimeIssueCode::RuntimeSelectionUnknown => "runtime_selection_unknown",
        RuntimeIssueCode::CompatDataNotFound => "compatdata_not_found",
        RuntimeIssueCode::PrefixNotInitialized => "prefix_not_initialized",
        RuntimeIssueCode::CompatDataUnreadable => "compatdata_unreadable",
        RuntimeIssueCode::CompatDataInvalid => "compatdata_invalid",
        RuntimeIssueCode::InstallationIncompleteOrUnknown => "installation_incomplete_or_unknown",
    }
}

fn compatibility_tool_issue_code(code: &CompatibilityToolIssueCode) -> &'static str {
    match code {
        CompatibilityToolIssueCode::CommonDirectoryInvalid => "common_directory_invalid",
        CompatibilityToolIssueCode::CustomDirectoryInvalid => "custom_directory_invalid",
        CompatibilityToolIssueCode::DirectoryUnreadable => "directory_unreadable",
        CompatibilityToolIssueCode::EntryUnreadable => "entry_unreadable",
        CompatibilityToolIssueCode::SymlinkRejected => "symlink_rejected",
        CompatibilityToolIssueCode::FileNotRegular => "file_not_regular",
        CompatibilityToolIssueCode::FileTooLarge => "file_too_large",
        CompatibilityToolIssueCode::MetadataInvalid => "metadata_invalid",
        CompatibilityToolIssueCode::InstallPathUnsafe => "install_path_unsafe",
        CompatibilityToolIssueCode::ToolDirectoryMissing => "tool_directory_missing",
        CompatibilityToolIssueCode::FilesystemError => "filesystem_error",
    }
}

#[cfg(test)]
mod tests {
    use super::{scan_steam_snapshot, status_code, RuntimeSelectionDto};
    use lxmi_core::SteamDetectionStatus;

    #[test]
    fn serializes_error_status_as_a_stable_frontend_code() {
        assert_eq!(
            status_code(&SteamDetectionStatus::ConfigurationMissing),
            "configuration_missing"
        );
    }

    #[test]
    #[ignore = "requiere ZZZ instalada localmente; solo inspecciona el filesystem"]
    fn local_steam_scan_finds_zzz_without_inferring_proton_selection() {
        let scan = scan_steam_snapshot();
        assert_eq!(scan.status, "detected");
        let game = scan
            .games
            .iter()
            .find(|game| game.id == "zenless-zone-zero")
            .expect("ZZZ manifest should be found in local Steam libraries");

        assert_eq!(game.steam_app_id, 4_162_040);
        assert_eq!(game.distribution, "steam");
        assert_eq!(game.install_status, "installed");
        assert_eq!(game.executable_status, "found");
        assert!(game
            .executable_path
            .as_deref()
            .is_some_and(|path| std::path::Path::new(path).is_file()));

        let plan = scan
            .runtime_plans
            .iter()
            .find(|plan| plan.game_id == "zenless-zone-zero")
            .expect("ZZZ runtime assessment should be present");
        assert!(matches!(
            &plan.selection,
            RuntimeSelectionDto::Unknown { .. }
        ));

        println!(
            "ZZZ: appid={}, distribution={}, install={}, executable={}, compatdata={}, prefix={}, proton_candidates={}, selection=unknown, readiness={}",
            game.steam_app_id,
            game.distribution,
            game.install_status,
            game.executable_status,
            game.compatdata_status,
            game.prefix_path.as_deref().unwrap_or("not found"),
            plan.available_runtime_candidates.len(),
            plan.readiness,
        );
    }
}
