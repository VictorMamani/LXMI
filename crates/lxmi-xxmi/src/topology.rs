//! Read-only Wine/Proton path-map inspection and declarative launch topology.
use crate::{filesystem::Directory, ManagedRuntime};
use lxmi_core::{GameDistribution, GameInstallationStatus};
use lxmi_runtime::{
    CompatDataAssessment, GameInstallationAssessment, GameRuntimePlan, PrefixAssessment,
    RuntimeSelection,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DosDeviceState {
    Directory,
    MissingTarget,
    NotDirectory,
    NotSymlink,
    UnsafeTarget,
    Unreadable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DosDeviceMapping {
    pub drive_letter: char,
    pub link_path: PathBuf,
    pub raw_target: Option<PathBuf>,
    pub normalized_target: Option<PathBuf>,
    pub is_symlink: bool,
    pub state: DosDeviceState,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DosDevicesState {
    Found,
    NotFound,
    Unreadable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DosDevicesReport {
    pub prefix_path: PathBuf,
    pub directory_path: PathBuf,
    pub state: DosDevicesState,
    pub mappings: Vec<DosDeviceMapping>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MappingStatus {
    Mapped,
    Unmapped,
    Ambiguous,
    Unsafe,
    PrefixUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowsPathMapping {
    pub linux_path: PathBuf,
    pub status: MappingStatus,
    pub windows_path: Option<String>,
    pub selected_drive: Option<char>,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoaderStrategy {
    UpstreamPortableLauncher,
    UpstreamLoaderHelper,
    LxmiWindowsHelper,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SamePrefixRequirement {
    Required,
    NotRequired,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopologyReadiness {
    PlannedIncomplete,
    GameNotDetected,
    RuntimeNotAssembled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityEvidenceState {
    UpstreamDocumented,
    UnavailableInUpstreamDocs,
    Unverified,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyCapability {
    pub name: String,
    pub state: CapabilityEvidenceState,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtonCandidateSummary {
    pub display_name: String,
    pub version: Option<String>,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyRequirement {
    pub name: String,
    pub state: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchTopologyPlan {
    pub game_id: String,
    pub game_name: String,
    pub distribution: String,
    pub steam_app_id: Option<u32>,
    pub game_installation: String,
    pub game_executable: Option<PathBuf>,
    pub target_process: Option<String>,
    pub compatdata: String,
    pub prefix_path: Option<PathBuf>,
    pub selected_proton: String,
    pub available_proton_candidates: Vec<ProtonCandidateSummary>,
    pub app_root: Option<PathBuf>,
    pub importer_folder: String,
    pub importer_root: Option<PathBuf>,
    pub loader_library_path: Option<PathBuf>,
    pub loader_library_windows_mapping: Option<WindowsPathMapping>,
    pub game_executable_windows_mapping: Option<WindowsPathMapping>,
    pub importer_root_windows_mapping: Option<WindowsPathMapping>,
    pub dosdevices: Option<DosDevicesReport>,
    pub loader_strategy: LoaderStrategy,
    pub loader_process_identity: Option<String>,
    pub same_prefix_requirement: SamePrefixRequirement,
    pub module: String,
    pub readiness: TopologyReadiness,
    pub capabilities: Vec<TopologyCapability>,
    pub requirements: Vec<TopologyRequirement>,
    pub evidence: Vec<String>,
    pub unknowns: Vec<String>,
    pub blockers: Vec<String>,
    pub execution_enabled: bool,
    pub external_files_modified: bool,
}

/// Inspects only the immediate `pfx/dosdevices` entries and symlink targets.
/// It never walks into a mapped drive or creates/modifies a link.
pub fn inspect_prefix_dosdevices(prefix_path: &Path) -> DosDevicesReport {
    let directory_path = prefix_path.join("dosdevices");
    let directory = match Directory::open_absolute(&directory_path) {
        Ok(directory) => directory,
        Err(error) if error.code == crate::ErrorCode::NotFound => {
            return DosDevicesReport {
                prefix_path: prefix_path.to_owned(),
                directory_path,
                state: DosDevicesState::NotFound,
                mappings: Vec::new(),
                issues: vec!["El directorio dosdevices no está presente.".into()],
            };
        }
        Err(error) => {
            return DosDevicesReport {
                prefix_path: prefix_path.to_owned(),
                directory_path,
                state: DosDevicesState::Unreadable,
                mappings: Vec::new(),
                issues: vec![error.to_string()],
            };
        }
    };

    let mut mappings = Vec::new();
    let mut issues = Vec::new();
    let mut entries = Vec::new();
    match fs::read_dir(directory.path()) {
        Ok(read_dir) => {
            for (index, entry) in read_dir.enumerate() {
                if index >= 4096 {
                    issues.push(
                        "dosdevices contiene más de 4096 entradas; se detuvo la inspección.".into(),
                    );
                    break;
                }
                match entry {
                    Ok(entry) => {
                        if let Ok(name) = entry.file_name().into_string() {
                            if parse_drive_letter(&name).is_some() {
                                entries.push(name);
                            }
                        }
                    }
                    Err(error) => issues.push(format!("No se pudo listar dosdevices: {error}")),
                }
            }
        }
        Err(error) => issues.push(format!("No se pudo listar dosdevices: {error}")),
    }
    entries.sort();
    for entry in entries {
        let Some(drive_letter) = parse_drive_letter(&entry) else {
            continue;
        };
        let link_path = directory_path.join(&entry);
        let metadata = match fs::symlink_metadata(directory.path().join(&entry)) {
            Ok(metadata) => metadata,
            Err(error) => {
                issues.push(format!("No se pudo inspeccionar {entry}: {error}"));
                mappings.push(DosDeviceMapping {
                    drive_letter,
                    link_path,
                    raw_target: None,
                    normalized_target: None,
                    is_symlink: false,
                    state: DosDeviceState::Unreadable,
                    evidence: error.to_string(),
                });
                continue;
            }
        };
        if !metadata.file_type().is_symlink() {
            mappings.push(DosDeviceMapping {
                drive_letter,
                link_path,
                raw_target: None,
                normalized_target: None,
                is_symlink: false,
                state: DosDeviceState::NotSymlink,
                evidence: "Drive entry is not a symlink; not followed.".into(),
            });
            continue;
        }
        let raw_target = match fs::read_link(directory.path().join(&entry)) {
            Ok(target) => target,
            Err(error) => {
                issues.push(format!("No se pudo leer el target de {entry}: {error}"));
                mappings.push(DosDeviceMapping {
                    drive_letter,
                    link_path,
                    raw_target: None,
                    normalized_target: None,
                    is_symlink: true,
                    state: DosDeviceState::Unreadable,
                    evidence: error.to_string(),
                });
                continue;
            }
        };
        let normalized_target = match normalize_target(&directory_path, &raw_target) {
            Some(target) => target,
            None => {
                mappings.push(DosDeviceMapping {
                    drive_letter,
                    link_path,
                    raw_target: Some(raw_target),
                    normalized_target: None,
                    is_symlink: true,
                    state: DosDeviceState::UnsafeTarget,
                    evidence: "Target is not a safe, normalized absolute path.".into(),
                });
                continue;
            }
        };
        let state = match fs::symlink_metadata(&normalized_target) {
            Ok(target_metadata) if target_metadata.file_type().is_symlink() => {
                DosDeviceState::UnsafeTarget
            }
            Ok(target_metadata) if target_metadata.is_dir() => DosDeviceState::Directory,
            Ok(_) => DosDeviceState::NotDirectory,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                DosDeviceState::MissingTarget
            }
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                DosDeviceState::Unreadable
            }
            Err(_) => DosDeviceState::UnsafeTarget,
        };
        mappings.push(DosDeviceMapping {
            drive_letter,
            link_path,
            raw_target: Some(raw_target.clone()),
            normalized_target: Some(normalized_target.clone()),
            is_symlink: true,
            state,
            evidence: format!(
                "readlink target {} → lexical path {}; target was not traversed recursively.",
                raw_target.display(),
                normalized_target.display()
            ),
        });
    }
    mappings.sort_by_key(|mapping| mapping.drive_letter.to_ascii_lowercase());
    let state = if issues.is_empty() {
        DosDevicesState::Found
    } else {
        DosDevicesState::Unreadable
    };
    tracing::info!(mappings = mappings.len(), state = ?state, "Proton dosdevices inspected read-only");
    DosDevicesReport {
        prefix_path: prefix_path.to_owned(),
        directory_path,
        state,
        mappings,
        issues,
    }
}

/// Maps a normalized absolute Linux path through the longest valid observed drive mapping.
pub fn map_linux_path(path: &Path, report: &DosDevicesReport) -> WindowsPathMapping {
    if !path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::CurDir | Component::Prefix(_)
            )
        })
    {
        return WindowsPathMapping {
            linux_path: path.to_owned(),
            status: MappingStatus::Unsafe,
            windows_path: None,
            selected_drive: None,
            evidence: "Only an absolute normalized Linux path can be mapped.".into(),
        };
    }
    if has_symlink_component(path) {
        return WindowsPathMapping {
            linux_path: path.to_owned(),
            status: MappingStatus::Unsafe,
            windows_path: None,
            selected_drive: None,
            evidence: "A path component is a symlink or could not be safely inspected; no Windows mapping was inferred.".into(),
        };
    }
    let mut matches = report
        .mappings
        .iter()
        .filter(|mapping| mapping.state == DosDeviceState::Directory)
        .filter_map(|mapping| {
            let target = mapping.normalized_target.as_ref()?;
            path.strip_prefix(target)
                .ok()
                .map(|remainder| (target, mapping.drive_letter, remainder))
        })
        .collect::<Vec<_>>();
    if matches.is_empty() {
        return WindowsPathMapping {
            linux_path: path.to_owned(),
            status: if report.state == DosDevicesState::NotFound {
                MappingStatus::PrefixUnavailable
            } else {
                MappingStatus::Unmapped
            },
            windows_path: None,
            selected_drive: None,
            evidence: "No valid observed drive mapping contains this path.".into(),
        };
    }
    matches.sort_by_key(|candidate| std::cmp::Reverse(candidate.0.as_os_str().len()));
    let longest = matches[0].0.as_os_str().len();
    let finalists = matches
        .iter()
        .take_while(|candidate| candidate.0.as_os_str().len() == longest)
        .collect::<Vec<_>>();
    let unique_drives = finalists
        .iter()
        .map(|value| value.1.to_ascii_lowercase())
        .collect::<std::collections::HashSet<_>>();
    if unique_drives.len() > 1 {
        return WindowsPathMapping {
            linux_path: path.to_owned(),
            status: MappingStatus::Ambiguous,
            windows_path: None,
            selected_drive: None,
            evidence: "Multiple equally specific drive mappings resolve this Linux path.".into(),
        };
    }
    let (_, drive, remainder) = matches[0];
    let tail = remainder
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\\");
    let windows_path = if tail.is_empty() {
        format!("{}:\\", drive.to_ascii_uppercase())
    } else {
        format!("{}:\\{}", drive.to_ascii_uppercase(), tail)
    };
    WindowsPathMapping {
        linux_path: path.to_owned(),
        status: MappingStatus::Mapped,
        windows_path: Some(windows_path),
        selected_drive: Some(drive.to_ascii_uppercase()),
        evidence: format!(
            "Mapped through observed {}: symlink to {}; longest matching target selected.",
            drive.to_ascii_uppercase(),
            matches[0].0.display()
        ),
    }
}

fn has_symlink_component(path: &Path) -> bool {
    let mut current = PathBuf::from("/");
    for component in path.components() {
        let Component::Normal(name) = component else {
            continue;
        };
        current.push(name);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => return true,
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return false,
            Err(_) => return true,
        }
    }
    false
}

pub fn inspect_launch_topology(
    platform: &GameRuntimePlan,
    runtime: Option<&ManagedRuntime>,
) -> LaunchTopologyPlan {
    tracing::info!(game = platform.game.id, "Launch topology planning started");
    let detected = match &platform.installation {
        GameInstallationAssessment::Detected { .. } => true,
        GameInstallationAssessment::NotFound { .. } => false,
        GameInstallationAssessment::Unknown { .. } => false,
    };
    let (distribution, steam_app_id, game_installation, game_executable) =
        match &platform.installation {
            GameInstallationAssessment::Detected {
                steam_app_id,
                distribution,
                executable_path,
                directory_status,
                ..
            } => (
                distribution_name(*distribution).to_owned(),
                Some(*steam_app_id),
                installation_status(*directory_status).to_owned(),
                executable_path.clone(),
            ),
            GameInstallationAssessment::NotFound { .. } => {
                ("steam".into(), None, "not_detected".into(), None)
            }
            GameInstallationAssessment::Unknown { .. } => {
                ("unknown".into(), None, "unknown".into(), None)
            }
        };
    let target_process = game_executable
        .as_ref()
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .map(str::to_owned);
    let (compatdata, prefix_path): (String, Option<PathBuf>) =
        match (&platform.compatdata, &platform.prefix) {
            (CompatDataAssessment::Found { .. }, PrefixAssessment::CandidateFound { path }) => {
                ("found".into(), Some(path.clone()))
            }
            (CompatDataAssessment::NotFound { .. }, PrefixAssessment::NotInitialized { .. }) => {
                ("not_initialized".into(), None)
            }
            (CompatDataAssessment::Unknown { .. }, _) | (_, PrefixAssessment::Unknown { .. }) => {
                ("unknown".into(), None)
            }
            (CompatDataAssessment::Unreadable { .. }, _)
            | (_, PrefixAssessment::Unreadable { .. }) => ("unreadable".into(), None),
            (CompatDataAssessment::Invalid { .. }, _) | (_, PrefixAssessment::Invalid { .. }) => {
                ("invalid".into(), None)
            }
            _ => ("not_found".into(), None),
        };
    let dosdevices = prefix_path
        .as_ref()
        .map(|path| inspect_prefix_dosdevices(path));
    let game_executable_windows_mapping = match (&game_executable, &dosdevices) {
        (Some(path), Some(report)) => Some(map_linux_path(path, report)),
        _ => None,
    };
    let importer_root = runtime.map(|value| value.importer_root.clone());
    let app_root = runtime.map(|value| value.app_root.clone());
    let importer_root_windows_mapping = match (&importer_root, &dosdevices) {
        (Some(path), Some(report)) => Some(map_linux_path(path, report)),
        _ => None,
    };
    let loader_library_windows_mapping = match (runtime, &dosdevices) {
        (Some(value), Some(report)) => Some(map_linux_path(&value.loader_library_path, report)),
        _ => None,
    };
    let selected_proton = match &platform.selection {
        RuntimeSelection::Unknown { .. } => "unknown".into(),
        RuntimeSelection::Selected { candidate } => candidate.display_name.clone(),
        RuntimeSelection::NotRequired => "not_required".into(),
    };
    let available_proton_candidates = platform
        .available_runtime_candidates
        .iter()
        .map(|candidate| ProtonCandidateSummary {
            display_name: candidate.display_name.clone(),
            version: candidate.version.clone(),
            path: candidate.path.clone(),
        })
        .collect::<Vec<_>>();

    let mut evidence = vec![
        "La topología es un plan de datos; no ejecuta Steam, Proton, Wine, helper ni juego.".into(),
        "La raíz App.Root de LXMI se calcula bajo XDG y la ruta relativa upstream ZZMI/ resuelve al importer root.".into(),
    ];
    if let Some(report) = &dosdevices {
        evidence.push(format!(
            "dosdevices state {:?}; {} drive mappings inspected without recursion.",
            report.state,
            report.mappings.len()
        ));
    }
    if let Some(runtime) = runtime {
        evidence.push(format!(
            "Managed runtime {} verified with {} files and a generated manifest.",
            runtime.runtime_id, runtime.file_count
        ));
    }
    if let Some(mapping) = &game_executable_windows_mapping {
        evidence.push(mapping.evidence.clone());
    }
    if let Some(mapping) = &importer_root_windows_mapping {
        evidence.push(mapping.evidence.clone());
    }
    let capabilities = vec![
        TopologyCapability {
            name: "ZZMI integration package".into(),
            state: CapabilityEvidenceState::UpstreamDocumented,
            evidence: "ZZMI is an upstream XXMI Model Importer package for ZZZ; this does not prove Linux execution.".into(),
        },
        TopologyCapability {
            name: "Native Steam Launch (XXMI Launcher)".into(),
            state: CapabilityEvidenceState::UpstreamDocumented,
            evidence: "The XXMI Launcher wiki documents Native Steam Launch for ZZZ.".into(),
        },
        TopologyCapability {
            name: "Direct Steam Launch (XXMI Launcher)".into(),
            state: CapabilityEvidenceState::UnavailableInUpstreamDocs,
            evidence: "The XXMI Launcher wiki says ZZZ Direct Steam Launch support will be implemented later.".into(),
        },
        TopologyCapability {
            name: "Portable XXMI Launcher under Wine".into(),
            state: CapabilityEvidenceState::UpstreamDocumented,
            evidence: "Upstream documents Portable on Linux via Wine 9.22+ and Microsoft VC++ Redistributable; this does not verify ZZZ launched by Steam/Proton.".into(),
        },
        TopologyCapability {
            name: "ZZZ Steam + Proton + XXMI injection".into(),
            state: CapabilityEvidenceState::Unverified,
            evidence: "LXMI has not launched this combination; no injection or game modification has occurred.".into(),
        },
    ];
    let requirements = vec![
        TopologyRequirement { name: "Game installation".into(), state: game_installation.clone(), evidence: "Steam manifest and bounded executable discovery.".into() },
        TopologyRequirement { name: "Proton selection".into(), state: if selected_proton == "unknown" { "unknown" } else { "observed" }.into(), evidence: "Runtime candidates do not imply the per-game selected runtime.".into() },
        TopologyRequirement { name: "compatdata/prefix".into(), state: compatdata.clone(), evidence: "Passive filesystem discovery only; prefix health is not verified.".into() },
        TopologyRequirement { name: "Managed importer assembly".into(), state: if runtime.is_some() { "present" } else { "missing" }.into(), evidence: "Only under the LXMI XDG data root; never inside game or prefix.".into() },
        TopologyRequirement { name: "Windows loader identity".into(), state: "unknown".into(), evidence: "ZZMI d3dx.ini allowlists XXMI Launcher.exe; LXMI native is not that Windows process.".into() },
        TopologyRequirement { name: "Same-prefix helper requirement".into(), state: "unknown".into(), evidence: "No pinned upstream contract establishes whether a separate helper must share the game's prefix.".into() },
        TopologyRequirement { name: "Linux/Proton injection".into(), state: "unverified".into(), evidence: "A portable launcher supported under Wine is not evidence that Steam ZZZ + Proton injection works.".into() },
    ];
    let mut unknowns = vec![
        "Which Proton tool Steam selects for ZZZ remains unknown.".into(),
        "Whether a Windows helper must run inside the same prefix remains unknown.".into(),
        "A native LXMI-to-Windows IPC/argument bridge has not been designed or tested.".into(),
        "The loader process identity still names XXMI Launcher.exe; replacement behavior is unverified.".into(),
        "ZZZ Steam + Linux/Proton + model importer injection remains unverified.".into(),
    ];
    let mut blockers = vec![
        "No launch strategy is selected or executable.".into(),
        "No helper binary has been selected or license-cleared for redistribution.".into(),
        "No anti-cheat or protection behavior has been investigated or bypassed.".into(),
    ];
    if runtime.is_none() {
        blockers.push("The managed ZZMI runtime has not been assembled.".into());
    }
    if selected_proton == "unknown" {
        unknowns.push("Runtime selection is not inferred from available candidates.".into());
    }
    if game_executable_windows_mapping
        .as_ref()
        .is_some_and(|mapping| mapping.status != MappingStatus::Mapped)
    {
        unknowns
            .push("The observed game executable has no unambiguous Windows path mapping.".into());
    }
    if importer_root_windows_mapping
        .as_ref()
        .is_some_and(|mapping| mapping.status != MappingStatus::Mapped)
    {
        unknowns.push("The managed importer root has no unambiguous Windows path mapping.".into());
    }
    let readiness = if !detected {
        TopologyReadiness::GameNotDetected
    } else if runtime.is_none() {
        TopologyReadiness::RuntimeNotAssembled
    } else {
        TopologyReadiness::PlannedIncomplete
    };
    tracing::info!(game = platform.game.id, readiness = ?readiness, "Launch topology plan completed");
    LaunchTopologyPlan {
        game_id: platform.game.id.into(),
        game_name: platform.game.name.into(),
        distribution,
        steam_app_id,
        game_installation,
        game_executable,
        target_process,
        compatdata,
        prefix_path,
        selected_proton,
        available_proton_candidates,
        app_root,
        importer_folder: "ZZMI/".into(),
        importer_root,
        loader_library_path: runtime.map(|value| value.loader_library_path.clone()),
        loader_library_windows_mapping,
        game_executable_windows_mapping,
        importer_root_windows_mapping,
        dosdevices,
        loader_strategy: LoaderStrategy::Unknown,
        loader_process_identity: runtime.map(|value| value.loader_process_identity.clone()),
        same_prefix_requirement: SamePrefixRequirement::Unknown,
        module: "d3d11.dll".into(),
        readiness,
        capabilities,
        requirements,
        evidence,
        unknowns,
        blockers,
        execution_enabled: false,
        external_files_modified: false,
    }
}

fn parse_drive_letter(name: &str) -> Option<char> {
    let mut chars = name.chars();
    let letter = chars.next()?;
    if !letter.is_ascii_alphabetic() || chars.next()? != ':' || chars.next().is_some() {
        return None;
    }
    Some(letter)
}

fn normalize_target(directory: &Path, target: &Path) -> Option<PathBuf> {
    let joined = if target.is_absolute() {
        target.to_owned()
    } else {
        directory.join(target)
    };
    let mut normalized = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::RootDir => normalized.push("/"),
            Component::Normal(value) => normalized.push(value),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() || normalized.as_os_str().is_empty() {
                    return None;
                }
            }
            Component::Prefix(_) => return None,
        }
    }
    normalized.is_absolute().then_some(normalized)
}

fn distribution_name(distribution: GameDistribution) -> &'static str {
    match distribution {
        GameDistribution::Steam => "steam",
        GameDistribution::HoYoPlay => "hoyoplay",
        GameDistribution::Manual => "manual",
        GameDistribution::Unknown => "unknown",
    }
}

fn installation_status(status: GameInstallationStatus) -> &'static str {
    match status {
        GameInstallationStatus::Installed => "present",
        GameInstallationStatus::DirectoryMissing => "missing",
        GameInstallationStatus::DirectoryInvalid => "invalid",
        GameInstallationStatus::PermissionDenied => "unreadable",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Tree(PathBuf);
    impl Tree {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "lxmi-dosdevices-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            Self(root)
        }
        fn prefix(&self) -> PathBuf {
            let prefix = self.0.join("pfx");
            fs::create_dir_all(prefix.join("dosdevices")).unwrap();
            fs::create_dir_all(prefix.join("drive_c")).unwrap();
            fs::create_dir_all(self.0.join("steam")).unwrap();
            fs::create_dir_all(self.0.join("managed/runtime")).unwrap();
            symlink("../drive_c", prefix.join("dosdevices/c:")).unwrap();
            symlink(self.0.join("steam"), prefix.join("dosdevices/s:")).unwrap();
            symlink("/", prefix.join("dosdevices/z:")).unwrap();
            prefix
        }
    }
    impl Drop for Tree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn dosdevices_are_read_without_recursively_following_targets_and_longest_mapping_wins() {
        let tree = Tree::new();
        let prefix = tree.prefix();
        let report = inspect_prefix_dosdevices(&prefix);
        assert_eq!(report.state, DosDevicesState::Found);
        assert_eq!(report.mappings.len(), 3);
        let game = map_linux_path(&tree.0.join("steam/games/ZZZ.exe"), &report);
        assert_eq!(game.status, MappingStatus::Mapped);
        assert_eq!(game.windows_path.as_deref(), Some("S:\\games\\ZZZ.exe"));
        let managed = map_linux_path(&tree.0.join("managed/runtime"), &report);
        assert_eq!(managed.status, MappingStatus::Mapped);
        assert_eq!(managed.selected_drive, Some('Z'));
    }

    #[test]
    fn duplicate_equally_specific_mappings_are_ambiguous() {
        let tree = Tree::new();
        let prefix = tree.prefix();
        symlink(tree.0.join("steam"), prefix.join("dosdevices/t:")).unwrap();
        let report = inspect_prefix_dosdevices(&prefix);
        let path = tree.0.join("steam/file");
        assert_eq!(
            map_linux_path(&path, &report).status,
            MappingStatus::Ambiguous
        );
    }

    #[test]
    fn non_directory_drive_target_is_unsafe_and_never_used() {
        let tree = Tree::new();
        let prefix = tree.prefix();
        fs::write(tree.0.join("ordinary-file"), b"x").unwrap();
        symlink(tree.0.join("ordinary-file"), prefix.join("dosdevices/u:")).unwrap();
        let report = inspect_prefix_dosdevices(&prefix);
        assert_eq!(
            report
                .mappings
                .iter()
                .find(|m| m.drive_letter == 'u')
                .unwrap()
                .state,
            DosDeviceState::NotDirectory
        );
        assert_eq!(
            map_linux_path(&tree.0.join("ordinary-file"), &report).status,
            MappingStatus::Mapped
        );
    }

    #[test]
    fn symlinked_linux_path_is_not_mapped_through_z_drive() {
        let tree = Tree::new();
        let prefix = tree.prefix();
        let link = tree.0.join("steam-link");
        symlink(tree.0.join("steam"), &link).unwrap();
        symlink(&link, prefix.join("dosdevices/v:")).unwrap();
        let report = inspect_prefix_dosdevices(&prefix);
        assert_eq!(
            report
                .mappings
                .iter()
                .find(|mapping| mapping.drive_letter == 'v')
                .unwrap()
                .state,
            DosDeviceState::UnsafeTarget
        );
        assert_eq!(
            map_linux_path(&link.join("games/ZZZ.exe"), &report).status,
            MappingStatus::Unsafe
        );
    }

    #[test]
    fn unsafe_non_normalized_linux_path_is_rejected() {
        let tree = Tree::new();
        let prefix = tree.prefix();
        let report = inspect_prefix_dosdevices(&prefix);
        assert_eq!(
            map_linux_path(Path::new("/tmp/../etc/passwd"), &report).status,
            MappingStatus::Unsafe
        );
    }
}
