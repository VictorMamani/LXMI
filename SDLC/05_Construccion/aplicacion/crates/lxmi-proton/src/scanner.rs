use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use lxmi_core::{SteamInstallation, SteamLibrary};
use tracing::{debug, info, warn};

use crate::manifest::{
    parse_compatibility_tool_vdf, parse_tool_manifest_vdf, CompatibilityToolMetadata,
};
use crate::model::{
    CompatibilityTool, CompatibilityToolDiscoveryResult, CompatibilityToolDiscoveryStatus,
    CompatibilityToolIssue, CompatibilityToolIssueCode, CompatibilityToolIssueSeverity,
    CompatibilityToolKind, CompatibilityToolSource, CompatibilityToolStatus,
};

const MAX_METADATA_BYTES: u64 = 2 * 1024 * 1024;
const MAX_VERSION_BYTES: u64 = 16 * 1024;
const MAX_RUNTIME_VERSIONS_BYTES: u64 = 64 * 1024;
const PROTON_LAYER: &str = "proton";
const STEAM_LINUX_RUNTIME_LAYERS: &[&str] = &["container-runtime", "scout-in-container"];

pub struct ProtonScanner;

impl ProtonScanner {
    pub fn scan(&self, installations: &[SteamInstallation]) -> CompatibilityToolDiscoveryResult {
        if installations.is_empty() {
            return CompatibilityToolDiscoveryResult::not_available();
        }

        info!("Proton discovery started");
        let mut tools = Vec::new();
        let mut issues = Vec::new();
        let mut seen_libraries = HashSet::new();
        let mut seen_roots = HashSet::new();

        for installation in installations {
            for library in &installation.libraries {
                if seen_libraries.insert(library.path.clone()) {
                    scan_steam_library(library, &mut tools, &mut issues);
                }
            }
            if seen_roots.insert(installation.root_path.clone()) {
                scan_custom_tools(&installation.root_path, &mut tools, &mut issues);
            }
        }

        tools.sort_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then_with(|| left.internal_id.cmp(&right.internal_id))
        });
        tools.dedup_by(|left, right| {
            left.path == right.path && left.internal_id == right.internal_id
        });

        let status = if issues.is_empty() {
            CompatibilityToolDiscoveryStatus::Complete
        } else {
            CompatibilityToolDiscoveryStatus::Partial
        };
        info!(
            tools = tools.len(),
            proton_tools = tools
                .iter()
                .filter(|tool| tool.kind == CompatibilityToolKind::Proton)
                .count(),
            issues = issues.len(),
            "Proton discovery completed"
        );
        CompatibilityToolDiscoveryResult {
            status,
            tools,
            issues,
        }
    }
}

fn scan_steam_library(
    library: &SteamLibrary,
    tools: &mut Vec<CompatibilityTool>,
    issues: &mut Vec<CompatibilityToolIssue>,
) {
    let steamapps = library.path.join("steamapps");
    let common = steamapps.join("common");
    match inspect_directory(&common) {
        Ok(DirectoryState::Missing) => return,
        Ok(DirectoryState::Present) => {}
        Err(error) => {
            push_issue(
                issues,
                error.issue_code(),
                error.severity(),
                common,
                Some(error.to_string()),
            );
            return;
        }
    }

    let entries = match fs::read_dir(&common) {
        Ok(entries) => entries,
        Err(error) => {
            push_io_issue(
                issues,
                CompatibilityToolIssueCode::DirectoryUnreadable,
                &common,
                error,
            );
            return;
        }
    };

    info!(library = %library.path.display(), "Scanning Steam compatibility tools");
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                push_io_issue(
                    issues,
                    CompatibilityToolIssueCode::EntryUnreadable,
                    &common,
                    error,
                );
                continue;
            }
        };
        let path = entry.path();
        let directory_state = match inspect_directory(&path) {
            Ok(state) => state,
            Err(error) => {
                push_issue(
                    issues,
                    error.issue_code_for_directory(false),
                    error.severity(),
                    path,
                    Some(error.to_string()),
                );
                continue;
            }
        };
        if directory_state == DirectoryState::Missing {
            continue;
        }

        let display_name = entry.file_name().to_string_lossy().into_owned();
        if let Some(tool) = inspect_tool_directory(
            &path,
            CompatibilityToolSource::SteamLibrary,
            None,
            None,
            display_name,
            issues,
        ) {
            if tool.kind == CompatibilityToolKind::Proton
                && tool.status == CompatibilityToolStatus::Valid
            {
                info!(path = %tool.path.display(), "Proton runtime validated");
            } else {
                debug!(
                    path = %tool.path.display(),
                    kind = ?tool.kind,
                    status = ?tool.status,
                    "Compatibility tool candidate found"
                );
            }
            tools.push(tool);
        }
    }
}

fn scan_custom_tools(
    steam_root: &Path,
    tools: &mut Vec<CompatibilityTool>,
    issues: &mut Vec<CompatibilityToolIssue>,
) {
    let directory = steam_root.join("compatibilitytools.d");
    match inspect_directory(&directory) {
        Ok(DirectoryState::Missing) => return,
        Ok(DirectoryState::Present) => {}
        Err(error) => {
            push_issue(
                issues,
                error.issue_code_for_directory(true),
                error.severity(),
                directory,
                Some(error.to_string()),
            );
            return;
        }
    }

    info!(path = %directory.display(), "Compatibility tools directory found");
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) => {
            push_io_issue(
                issues,
                CompatibilityToolIssueCode::DirectoryUnreadable,
                &directory,
                error,
            );
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                push_io_issue(
                    issues,
                    CompatibilityToolIssueCode::EntryUnreadable,
                    &directory,
                    error,
                );
                continue;
            }
        };
        let path = entry.path();
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) => {
                push_io_issue(
                    issues,
                    CompatibilityToolIssueCode::EntryUnreadable,
                    &path,
                    error,
                );
                continue;
            }
        };
        if metadata.file_type().is_symlink() {
            push_issue(
                issues,
                CompatibilityToolIssueCode::SymlinkRejected,
                CompatibilityToolIssueSeverity::Warning,
                path,
                Some("compatibility tool entries cannot be symlinks".to_owned()),
            );
            continue;
        }

        let (manifest_path, install_base, fallback_name) = if metadata.is_dir() {
            let manifest = path.join("compatibilitytool.vdf");
            (
                manifest,
                path.clone(),
                entry.file_name().to_string_lossy().into_owned(),
            )
        } else if metadata.is_file()
            && entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case("compatibilitytool.vdf")
        {
            (
                path.clone(),
                directory.clone(),
                "Custom compatibility tool".to_owned(),
            )
        } else {
            continue;
        };

        let contents = match read_optional_text(&manifest_path, MAX_METADATA_BYTES) {
            Ok(Some(contents)) => contents,
            Ok(None) => continue,
            Err(error) => {
                push_issue(
                    issues,
                    error.issue_code(),
                    error.severity(),
                    manifest_path.clone(),
                    Some(error.to_string()),
                );
                tools.push(invalid_custom_candidate(
                    &install_base,
                    &manifest_path,
                    fallback_name,
                    CompatibilityToolStatus::Unreadable,
                ));
                continue;
            }
        };

        let metadata_entries = match parse_compatibility_tool_vdf(&contents) {
            Ok(metadata_entries) => metadata_entries,
            Err(error) => {
                warn!(
                    path = %manifest_path.display(),
                    error = %error,
                    "Invalid compatibility tool metadata"
                );
                push_issue(
                    issues,
                    CompatibilityToolIssueCode::MetadataInvalid,
                    CompatibilityToolIssueSeverity::Warning,
                    manifest_path.clone(),
                    Some(error.to_string()),
                );
                tools.push(invalid_custom_candidate(
                    &install_base,
                    &manifest_path,
                    fallback_name,
                    CompatibilityToolStatus::InvalidMetadata,
                ));
                continue;
            }
        };

        for metadata in metadata_entries {
            info!(
                path = %manifest_path.display(),
                tool_id = %metadata.internal_id,
                "Compatibility metadata parsed"
            );
            match resolve_install_path(&install_base, &metadata.install_path) {
                Ok(resolved_path) => match validate_directory_without_symlinks(&resolved_path) {
                    Ok(()) => {
                        let fallback = metadata
                            .display_name
                            .clone()
                            .unwrap_or_else(|| metadata.internal_id.clone());
                        let tool = inspect_tool_directory(
                            &resolved_path,
                            CompatibilityToolSource::Custom,
                            Some(&metadata),
                            Some(manifest_path.clone()),
                            fallback,
                            issues,
                        );
                        if let Some(tool) = tool {
                            tools.push(tool);
                        } else {
                            tools.push(CompatibilityTool {
                                internal_id: Some(metadata.internal_id.clone()),
                                display_name: metadata
                                    .display_name
                                    .clone()
                                    .unwrap_or_else(|| metadata.internal_id.clone()),
                                path: resolved_path,
                                metadata_path: Some(manifest_path.clone()),
                                source: CompatibilityToolSource::Custom,
                                kind: CompatibilityToolKind::Other,
                                version: None,
                                status: CompatibilityToolStatus::Incomplete,
                            });
                        }
                    }
                    Err(error) => {
                        let (code, severity, status) = match &error {
                            FilePathError::Io(error) if error.kind() == io::ErrorKind::NotFound => {
                                (
                                    CompatibilityToolIssueCode::ToolDirectoryMissing,
                                    CompatibilityToolIssueSeverity::Warning,
                                    CompatibilityToolStatus::Incomplete,
                                )
                            }
                            FilePathError::Symlink => (
                                CompatibilityToolIssueCode::SymlinkRejected,
                                CompatibilityToolIssueSeverity::Warning,
                                CompatibilityToolStatus::InvalidMetadata,
                            ),
                            _ => (
                                error.issue_code(),
                                error.severity(),
                                CompatibilityToolStatus::InvalidMetadata,
                            ),
                        };
                        push_issue(
                            issues,
                            code,
                            severity,
                            resolved_path.clone(),
                            Some(error.to_string()),
                        );
                        tools.push(CompatibilityTool {
                            internal_id: Some(metadata.internal_id.clone()),
                            display_name: metadata
                                .display_name
                                .clone()
                                .unwrap_or_else(|| metadata.internal_id.clone()),
                            path: resolved_path,
                            metadata_path: Some(manifest_path.clone()),
                            source: CompatibilityToolSource::Custom,
                            kind: CompatibilityToolKind::Unknown,
                            version: None,
                            status,
                        });
                    }
                },
                Err(error) => {
                    push_issue(
                        issues,
                        CompatibilityToolIssueCode::InstallPathUnsafe,
                        CompatibilityToolIssueSeverity::Warning,
                        manifest_path.clone(),
                        Some(error),
                    );
                    tools.push(CompatibilityTool {
                        internal_id: Some(metadata.internal_id.clone()),
                        display_name: metadata
                            .display_name
                            .clone()
                            .unwrap_or_else(|| metadata.internal_id.clone()),
                        path: install_base.clone(),
                        metadata_path: Some(manifest_path.clone()),
                        source: CompatibilityToolSource::Custom,
                        kind: CompatibilityToolKind::Unknown,
                        version: None,
                        status: CompatibilityToolStatus::InvalidMetadata,
                    });
                }
            }
        }
    }
}

fn inspect_tool_directory(
    path: &Path,
    source: CompatibilityToolSource,
    compatibility_metadata: Option<&CompatibilityToolMetadata>,
    compatibility_metadata_path: Option<PathBuf>,
    fallback_name: String,
    issues: &mut Vec<CompatibilityToolIssue>,
) -> Option<CompatibilityTool> {
    let tool_manifest_path = path.join("toolmanifest.vdf");
    let compatibility_vdf_path = path.join("compatibilitytool.vdf");
    let proton_entrypoint_path = path.join("proton");
    let version_path = path.join("version");
    let runtime_versions_path = path.join("VERSIONS.txt");

    let tool_manifest_text = read_optional_text(&tool_manifest_path, MAX_METADATA_BYTES);
    let compatibility_text = if compatibility_metadata.is_none() {
        read_optional_text(&compatibility_vdf_path, MAX_METADATA_BYTES)
    } else {
        Ok(None)
    };
    let proton_entrypoint = inspect_optional_regular_file(&proton_entrypoint_path);
    let version_text = read_optional_text(&version_path, MAX_VERSION_BYTES);
    let runtime_versions_text =
        read_optional_text(&runtime_versions_path, MAX_RUNTIME_VERSIONS_BYTES);

    let mut recognized = compatibility_metadata.is_some();
    let mut core_read_status = None;
    let mut tool_manifest = None;

    match &tool_manifest_text {
        Ok(Some(contents)) => {
            recognized = true;
            match parse_tool_manifest_vdf(contents) {
                Ok(metadata) => tool_manifest = Some(metadata),
                Err(error) => {
                    core_read_status = Some(CompatibilityToolStatus::InvalidMetadata);
                    push_issue(
                        issues,
                        CompatibilityToolIssueCode::MetadataInvalid,
                        CompatibilityToolIssueSeverity::Warning,
                        tool_manifest_path.clone(),
                        Some(error.to_string()),
                    );
                }
            }
        }
        Err(error) => {
            recognized = true;
            core_read_status = Some(status_for_file_error(error));
            push_file_error(issues, &tool_manifest_path, error);
        }
        Ok(None) => {}
    }

    let discovered_metadata = match &compatibility_text {
        Ok(Some(contents)) => {
            recognized = true;
            match parse_compatibility_tool_vdf(contents) {
                Ok(entries) => entries.into_iter().next(),
                Err(error) => {
                    core_read_status = Some(CompatibilityToolStatus::InvalidMetadata);
                    push_issue(
                        issues,
                        CompatibilityToolIssueCode::MetadataInvalid,
                        CompatibilityToolIssueSeverity::Warning,
                        compatibility_vdf_path.clone(),
                        Some(error.to_string()),
                    );
                    None
                }
            }
        }
        Err(error) => {
            recognized = true;
            core_read_status = Some(status_for_file_error(error));
            push_file_error(issues, &compatibility_vdf_path, error);
            None
        }
        Ok(None) => None,
    };

    match &proton_entrypoint {
        Ok(true) => recognized = true,
        Err(error) => {
            recognized = true;
            core_read_status.get_or_insert_with(|| status_for_file_error(error));
            push_file_error(issues, &proton_entrypoint_path, error);
        }
        Ok(false) => {}
    }

    if !recognized {
        return None;
    }

    let metadata = compatibility_metadata.or(discovered_metadata.as_ref());
    let layer = tool_manifest
        .as_ref()
        .and_then(|manifest| manifest.compatmanager_layer_name.as_deref());
    let has_proton_entrypoint = matches!(&proton_entrypoint, Ok(true));
    let has_version = matches!(&version_text, Ok(Some(_)));
    let has_runtime_versions = matches!(&runtime_versions_text, Ok(Some(_)));

    let kind = if layer.is_some_and(|layer| layer.eq_ignore_ascii_case(PROTON_LAYER))
        || (metadata.is_some() && has_proton_entrypoint)
        || (has_proton_entrypoint && has_version && tool_manifest.is_none())
    {
        CompatibilityToolKind::Proton
    } else if layer.is_some_and(|layer| {
        STEAM_LINUX_RUNTIME_LAYERS
            .iter()
            .any(|known| layer.eq_ignore_ascii_case(known))
    }) {
        CompatibilityToolKind::SteamLinuxRuntime
    } else if metadata.is_some() {
        CompatibilityToolKind::Other
    } else {
        CompatibilityToolKind::Unknown
    };

    let mut status = core_read_status.unwrap_or(CompatibilityToolStatus::Valid);
    if status == CompatibilityToolStatus::Valid {
        match kind {
            CompatibilityToolKind::Proton if !has_proton_entrypoint => {
                status = CompatibilityToolStatus::Incomplete;
            }
            CompatibilityToolKind::Proton if tool_manifest.is_none() && metadata.is_none() => {
                status = CompatibilityToolStatus::Incomplete;
            }
            CompatibilityToolKind::SteamLinuxRuntime if !has_runtime_versions => {
                status = CompatibilityToolStatus::Incomplete;
            }
            _ => {}
        }
    }

    let version = match kind {
        CompatibilityToolKind::SteamLinuxRuntime => runtime_versions_text
            .ok()
            .flatten()
            .and_then(|text| steam_runtime_depot_version(&text)),
        _ => version_text.ok().flatten().and_then(nonempty_trimmed),
    };
    let display_name = metadata
        .and_then(|metadata| metadata.display_name.as_deref())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(&fallback_name)
        .to_owned();
    let internal_id = metadata.map(|metadata| metadata.internal_id.clone());
    let metadata_path = compatibility_metadata_path.or_else(|| {
        if tool_manifest.is_some()
            && matches!(
                kind,
                CompatibilityToolKind::Proton | CompatibilityToolKind::SteamLinuxRuntime
            )
        {
            Some(tool_manifest_path.clone())
        } else if discovered_metadata.is_some() {
            Some(compatibility_vdf_path.clone())
        } else {
            None
        }
    });

    if status == CompatibilityToolStatus::Valid && kind == CompatibilityToolKind::Proton {
        info!(path = %path.display(), "Proton runtime validated");
    } else {
        debug!(path = %path.display(), kind = ?kind, status = ?status, "Compatibility tool candidate found");
    }

    Some(CompatibilityTool {
        internal_id,
        display_name,
        path: path.to_path_buf(),
        metadata_path,
        source,
        kind,
        version,
        status,
    })
}

fn invalid_custom_candidate(
    path: &Path,
    metadata_path: &Path,
    display_name: String,
    status: CompatibilityToolStatus,
) -> CompatibilityTool {
    CompatibilityTool {
        internal_id: None,
        display_name,
        path: path.to_path_buf(),
        metadata_path: Some(metadata_path.to_path_buf()),
        source: CompatibilityToolSource::Custom,
        kind: CompatibilityToolKind::Unknown,
        version: None,
        status,
    }
}

fn steam_runtime_depot_version(contents: &str) -> Option<String> {
    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .find_map(|line| {
            let mut columns = line.split_whitespace();
            let name = columns.next()?;
            let version = columns.next()?;
            (name.eq_ignore_ascii_case("depot") && version != "-").then(|| version.to_owned())
        })
}

fn nonempty_trimmed(contents: String) -> Option<String> {
    let trimmed = contents.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn resolve_install_path(base: &Path, declared: &str) -> Result<PathBuf, String> {
    let declared = declared.trim();
    if declared.is_empty() {
        return Err("install_path is empty".to_owned());
    }
    let declared_path = Path::new(declared);
    for component in declared_path.components() {
        if matches!(component, Component::ParentDir | Component::Prefix(_)) {
            return Err("install_path contains a parent or platform prefix component".to_owned());
        }
    }
    let resolved = if declared_path.is_absolute() {
        declared_path.to_path_buf()
    } else {
        if declared_path
            .components()
            .any(|component| matches!(component, Component::RootDir))
        {
            return Err("relative install_path contains a root component".to_owned());
        }
        base.join(declared_path)
    };
    if !resolved.is_absolute() {
        return Err("resolved install_path is not absolute".to_owned());
    }
    Ok(resolved)
}

fn inspect_directory(path: &Path) -> Result<DirectoryState, FilePathError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(FilePathError::Symlink),
        Ok(metadata) if metadata.is_dir() => Ok(DirectoryState::Present),
        Ok(_) => Err(FilePathError::NotDirectory),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(DirectoryState::Missing),
        Err(error) => Err(FilePathError::Io(error)),
    }
}

fn validate_directory_without_symlinks(path: &Path) -> Result<(), FilePathError> {
    if !path.is_absolute() {
        return Err(FilePathError::NotAbsolute);
    }
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir => current.push(Path::new("/")),
            Component::Normal(segment) => {
                current.push(segment);
                match fs::symlink_metadata(&current) {
                    Ok(metadata) if metadata.file_type().is_symlink() => {
                        return Err(FilePathError::Symlink)
                    }
                    Ok(metadata) if metadata.is_dir() => {}
                    Ok(_) => return Err(FilePathError::NotDirectory),
                    Err(error) => return Err(FilePathError::Io(error)),
                }
            }
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) => return Err(FilePathError::UnsafePath),
        }
    }
    Ok(())
}

fn read_optional_text(path: &Path, max_bytes: u64) -> Result<Option<String>, FilePathError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(FilePathError::Io(error)),
    };
    if metadata.file_type().is_symlink() {
        return Err(FilePathError::Symlink);
    }
    if !metadata.is_file() {
        return Err(FilePathError::NotRegularFile);
    }
    if metadata.len() > max_bytes {
        return Err(FilePathError::TooLarge);
    }

    let file = File::open(path).map_err(FilePathError::Io)?;
    let mut contents = String::new();
    file.take(max_bytes + 1)
        .read_to_string(&mut contents)
        .map_err(FilePathError::Io)?;
    if contents.len() as u64 > max_bytes {
        return Err(FilePathError::TooLarge);
    }
    Ok(Some(contents))
}

fn inspect_optional_regular_file(path: &Path) -> Result<bool, FilePathError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(FilePathError::Symlink),
        Ok(metadata) if metadata.is_file() => Ok(true),
        Ok(_) => Err(FilePathError::NotRegularFile),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(FilePathError::Io(error)),
    }
}

fn push_file_error(issues: &mut Vec<CompatibilityToolIssue>, path: &Path, error: &FilePathError) {
    push_issue(
        issues,
        error.issue_code(),
        error.severity(),
        path.to_path_buf(),
        Some(error.to_string()),
    );
}

fn push_io_issue(
    issues: &mut Vec<CompatibilityToolIssue>,
    code: CompatibilityToolIssueCode,
    path: &Path,
    error: io::Error,
) {
    let severity = if error.kind() == io::ErrorKind::PermissionDenied {
        CompatibilityToolIssueSeverity::Error
    } else {
        CompatibilityToolIssueSeverity::Warning
    };
    push_issue(
        issues,
        code,
        severity,
        path.to_path_buf(),
        Some(error.to_string()),
    );
}

fn push_issue(
    issues: &mut Vec<CompatibilityToolIssue>,
    code: CompatibilityToolIssueCode,
    severity: CompatibilityToolIssueSeverity,
    path: PathBuf,
    detail: Option<String>,
) {
    issues.push(CompatibilityToolIssue {
        code,
        severity,
        path,
        detail,
    });
}

fn status_for_file_error(error: &FilePathError) -> CompatibilityToolStatus {
    match error {
        FilePathError::Io(error) if error.kind() == io::ErrorKind::PermissionDenied => {
            CompatibilityToolStatus::Unreadable
        }
        FilePathError::Io(_) => CompatibilityToolStatus::Unreadable,
        _ => CompatibilityToolStatus::InvalidMetadata,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DirectoryState {
    Missing,
    Present,
}

#[derive(Debug)]
enum FilePathError {
    Symlink,
    NotDirectory,
    NotRegularFile,
    NotAbsolute,
    UnsafePath,
    TooLarge,
    Io(io::Error),
}

impl FilePathError {
    fn issue_code(&self) -> CompatibilityToolIssueCode {
        match self {
            Self::Symlink => CompatibilityToolIssueCode::SymlinkRejected,
            Self::NotDirectory => CompatibilityToolIssueCode::CommonDirectoryInvalid,
            Self::NotRegularFile => CompatibilityToolIssueCode::FileNotRegular,
            Self::NotAbsolute | Self::UnsafePath => CompatibilityToolIssueCode::InstallPathUnsafe,
            Self::TooLarge => CompatibilityToolIssueCode::FileTooLarge,
            Self::Io(error) if error.kind() == io::ErrorKind::PermissionDenied => {
                CompatibilityToolIssueCode::DirectoryUnreadable
            }
            Self::Io(_) => CompatibilityToolIssueCode::FilesystemError,
        }
    }

    fn severity(&self) -> CompatibilityToolIssueSeverity {
        match self {
            Self::Io(error) if error.kind() == io::ErrorKind::PermissionDenied => {
                CompatibilityToolIssueSeverity::Error
            }
            _ => CompatibilityToolIssueSeverity::Warning,
        }
    }
}

impl std::fmt::Display for FilePathError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Symlink => formatter.write_str("symbolic links are not followed"),
            Self::NotDirectory => formatter.write_str("path is not a regular directory"),
            Self::NotRegularFile => formatter.write_str("path is not a regular file"),
            Self::NotAbsolute => formatter.write_str("resolved path must be absolute"),
            Self::UnsafePath => formatter.write_str("path contains an unsafe component"),
            Self::TooLarge => formatter.write_str("metadata file exceeds its read limit"),
            Self::Io(error) => write!(formatter, "filesystem error: {error}"),
        }
    }
}

fn issue_code_for_directory_error(
    error: &FilePathError,
    custom: bool,
) -> CompatibilityToolIssueCode {
    match error {
        FilePathError::Symlink => CompatibilityToolIssueCode::SymlinkRejected,
        FilePathError::Io(error) if error.kind() == io::ErrorKind::PermissionDenied => {
            CompatibilityToolIssueCode::DirectoryUnreadable
        }
        FilePathError::Io(_) => CompatibilityToolIssueCode::FilesystemError,
        _ if custom => CompatibilityToolIssueCode::CustomDirectoryInvalid,
        _ => CompatibilityToolIssueCode::CommonDirectoryInvalid,
    }
}

impl FilePathError {
    fn issue_code_for_directory(&self, custom: bool) -> CompatibilityToolIssueCode {
        issue_code_for_directory_error(self, custom)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use lxmi_core::{SteamInstallation, SteamLibrary};

    use crate::{
        CompatibilityToolDiscoveryStatus, CompatibilityToolIssueCode, CompatibilityToolKind,
        CompatibilityToolSource, CompatibilityToolStatus, ProtonScanner,
    };

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    struct TempTree(PathBuf);

    impl TempTree {
        fn new() -> Self {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let root =
                std::env::temp_dir().join(format!("lxmi-proton-test-{}-{id}", std::process::id()));
            fs::create_dir_all(&root).expect("synthetic fixture root should be created");
            Self(root)
        }

        fn root(&self) -> PathBuf {
            self.0.join("steam")
        }

        fn library(&self) -> PathBuf {
            self.root()
        }

        fn installation(&self) -> SteamInstallation {
            SteamInstallation {
                root_path: self.root(),
                libraries: vec![SteamLibrary {
                    path: self.library(),
                    is_default: true,
                }],
            }
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn create_library(tree: &TempTree) {
        fs::create_dir_all(tree.library().join("steamapps/common"))
            .expect("synthetic Steam common directory should be created");
        fs::create_dir_all(tree.root()).expect("synthetic Steam root should be created");
    }

    fn write_tool_manifest(directory: &Path, fixture: &str) {
        fs::create_dir_all(directory).expect("synthetic tool directory should be created");
        fs::write(directory.join("toolmanifest.vdf"), fixture)
            .expect("synthetic tool manifest should be written");
    }

    #[test]
    fn discovers_proton_by_tool_metadata_and_entrypoint_not_folder_name() {
        let tree = TempTree::new();
        create_library(&tree);
        let tool_dir = tree
            .library()
            .join("steamapps/common/runtime-folder-without-proton-name");
        write_tool_manifest(
            &tool_dir,
            include_str!("../tests/fixtures/proton/steam-proton/toolmanifest.vdf"),
        );
        fs::write(tool_dir.join("proton"), "synthetic entrypoint")
            .expect("synthetic Proton entrypoint should be written");
        fs::write(tool_dir.join("version"), "10.0-fixture\n")
            .expect("synthetic version should be written");

        let result = ProtonScanner.scan(&[tree.installation()]);
        assert_eq!(result.status, CompatibilityToolDiscoveryStatus::Complete);
        assert_eq!(result.tools.len(), 1);
        assert_eq!(result.tools[0].kind, CompatibilityToolKind::Proton);
        assert_eq!(result.tools[0].status, CompatibilityToolStatus::Valid);
        assert_eq!(result.tools[0].version.as_deref(), Some("10.0-fixture"));
        assert_eq!(
            result.tools[0].source,
            CompatibilityToolSource::SteamLibrary
        );
    }

    #[test]
    fn reports_missing_version_as_valid_unknown_version() {
        let tree = TempTree::new();
        create_library(&tree);
        let tool_dir = tree.library().join("steamapps/common/compat-tool");
        write_tool_manifest(
            &tool_dir,
            include_str!("../tests/fixtures/proton/steam-proton/toolmanifest.vdf"),
        );
        fs::write(tool_dir.join("proton"), "synthetic entrypoint")
            .expect("synthetic Proton entrypoint should be written");

        let result = ProtonScanner.scan(&[tree.installation()]);
        assert_eq!(result.tools[0].status, CompatibilityToolStatus::Valid);
        assert_eq!(result.tools[0].version, None);
    }

    #[test]
    fn detects_steam_linux_runtime_from_layer_and_versions_not_folder_name() {
        let tree = TempTree::new();
        create_library(&tree);
        let tool_dir = tree.library().join("steamapps/common/runtime-container");
        write_tool_manifest(
            &tool_dir,
            include_str!("../tests/fixtures/proton/linux-runtime/toolmanifest.vdf"),
        );
        fs::write(
            tool_dir.join("VERSIONS.txt"),
            include_str!("../tests/fixtures/proton/linux-runtime/VERSIONS.txt"),
        )
        .expect("synthetic runtime versions should be written");

        let result = ProtonScanner.scan(&[tree.installation()]);
        assert_eq!(result.tools.len(), 1);
        assert_eq!(
            result.tools[0].kind,
            CompatibilityToolKind::SteamLinuxRuntime
        );
        assert_eq!(result.tools[0].status, CompatibilityToolStatus::Valid);
        assert_eq!(result.tools[0].version.as_deref(), Some("4.0.fixture"));
    }

    #[test]
    fn ignores_a_folder_named_proton_without_compatibility_structure() {
        let tree = TempTree::new();
        create_library(&tree);
        fs::create_dir_all(tree.library().join("steamapps/common/Proton Fake"))
            .expect("irrelevant directory should be created");

        let result = ProtonScanner.scan(&[tree.installation()]);
        assert!(result.tools.is_empty());
        assert_eq!(result.status, CompatibilityToolDiscoveryStatus::Complete);
    }

    #[test]
    fn reports_proton_marker_without_entrypoint_as_incomplete() {
        let tree = TempTree::new();
        create_library(&tree);
        let tool_dir = tree.library().join("steamapps/common/incomplete-tool");
        write_tool_manifest(
            &tool_dir,
            include_str!("../tests/fixtures/proton/steam-proton/toolmanifest.vdf"),
        );

        let result = ProtonScanner.scan(&[tree.installation()]);
        assert_eq!(result.tools[0].kind, CompatibilityToolKind::Proton);
        assert_eq!(result.tools[0].status, CompatibilityToolStatus::Incomplete);
    }

    #[test]
    fn discovers_custom_proton_and_uses_declared_metadata() {
        let tree = TempTree::new();
        create_library(&tree);
        let custom_dir = tree.root().join("compatibilitytools.d/GE-Proton-fixture");
        fs::create_dir_all(&custom_dir).expect("custom tool directory should be created");
        fs::write(
            custom_dir.join("compatibilitytool.vdf"),
            include_str!("../tests/fixtures/proton/custom/compatibilitytool.vdf"),
        )
        .expect("synthetic custom metadata should be written");
        fs::write(custom_dir.join("proton"), "synthetic entrypoint")
            .expect("synthetic Proton entrypoint should be written");
        fs::write(custom_dir.join("version"), "GE-fixture-1\n")
            .expect("synthetic version should be written");

        let result = ProtonScanner.scan(&[tree.installation()]);
        let tool = result
            .tools
            .iter()
            .find(|tool| tool.source == CompatibilityToolSource::Custom)
            .expect("custom tool should be discovered");
        assert_eq!(tool.internal_id.as_deref(), Some("GE-Proton-fixture"));
        assert_eq!(tool.display_name, "GE-Proton fixture");
        assert_eq!(tool.kind, CompatibilityToolKind::Proton);
        assert_eq!(tool.status, CompatibilityToolStatus::Valid);
        assert_eq!(tool.version.as_deref(), Some("GE-fixture-1"));
    }

    #[test]
    fn rejects_custom_path_traversal_without_scanning_outside_tool_directory() {
        let tree = TempTree::new();
        create_library(&tree);
        let custom_dir = tree.root().join("compatibilitytools.d/unsafe");
        fs::create_dir_all(&custom_dir).expect("custom tool directory should be created");
        let metadata = include_str!("../tests/fixtures/proton/custom/compatibilitytool.vdf")
            .replace(
                "\"install_path\" \".\"",
                "\"install_path\" \"../../outside\"",
            );
        fs::write(custom_dir.join("compatibilitytool.vdf"), metadata)
            .expect("synthetic unsafe metadata should be written");

        let result = ProtonScanner.scan(&[tree.installation()]);
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == CompatibilityToolIssueCode::InstallPathUnsafe));
        assert!(result
            .tools
            .iter()
            .all(|tool| tool.status == CompatibilityToolStatus::InvalidMetadata));
    }

    #[test]
    fn accepts_documented_absolute_custom_path_when_each_directory_is_real() {
        let tree = TempTree::new();
        create_library(&tree);
        let compatibility_tools = tree.root().join("compatibilitytools.d");
        fs::create_dir_all(&compatibility_tools).expect("custom tools directory should be created");
        let external_tool = tree.0.join("external-proton");
        fs::create_dir_all(&external_tool).expect("synthetic external tool directory should exist");
        fs::write(
            compatibility_tools.join("compatibilitytool.vdf"),
            include_str!("../tests/fixtures/proton/custom/compatibilitytool.vdf").replace(
                "\"install_path\" \".\"",
                &format!("\"install_path\" \"{}\"", external_tool.display()),
            ),
        )
        .expect("metadata fixture should be written");
        fs::write(external_tool.join("proton"), "synthetic entrypoint")
            .expect("synthetic Proton entrypoint should be written");

        let result = ProtonScanner.scan(&[tree.installation()]);
        assert!(result.tools.iter().any(|tool| {
            tool.path == external_tool && tool.kind == CompatibilityToolKind::Proton
        }));
    }

    #[test]
    fn malformed_custom_metadata_is_reported_and_does_not_abort_scan() {
        let tree = TempTree::new();
        create_library(&tree);
        let custom_dir = tree.root().join("compatibilitytools.d/broken");
        fs::create_dir_all(&custom_dir).expect("custom tool directory should be created");
        fs::write(custom_dir.join("compatibilitytool.vdf"), "broken {")
            .expect("broken synthetic metadata should be written");
        let valid_dir = tree.library().join("steamapps/common/valid-proton");
        write_tool_manifest(
            &valid_dir,
            include_str!("../tests/fixtures/proton/steam-proton/toolmanifest.vdf"),
        );
        fs::write(valid_dir.join("proton"), "synthetic entrypoint")
            .expect("synthetic Proton entrypoint should be written");

        let result = ProtonScanner.scan(&[tree.installation()]);
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == CompatibilityToolIssueCode::MetadataInvalid));
        assert!(result
            .tools
            .iter()
            .any(|tool| tool.display_name == "valid-proton"));
    }

    #[test]
    fn no_steam_installations_returns_not_available() {
        let result = ProtonScanner.scan(&[]);
        assert_eq!(
            result.status,
            CompatibilityToolDiscoveryStatus::NotAvailable
        );
        assert!(result.tools.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_custom_directory_and_common_tool_directory() {
        use std::os::unix::fs::symlink;

        let tree = TempTree::new();
        create_library(&tree);
        let outside = tree.0.join("outside");
        fs::create_dir_all(&outside).expect("outside fixture directory should exist");
        symlink(
            &outside,
            tree.library().join("steamapps/common/Proton symlink"),
        )
        .expect("common directory symlink should be created");
        symlink(&outside, tree.root().join("compatibilitytools.d"))
            .expect("custom root symlink should be created");

        let result = ProtonScanner.scan(&[tree.installation()]);
        assert!(result.tools.is_empty());
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == CompatibilityToolIssueCode::SymlinkRejected));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_entrypoint_without_following_target() {
        use std::os::unix::fs::symlink;

        let tree = TempTree::new();
        create_library(&tree);
        let tool_dir = tree.library().join("steamapps/common/proton-tool");
        write_tool_manifest(
            &tool_dir,
            include_str!("../tests/fixtures/proton/steam-proton/toolmanifest.vdf"),
        );
        let outside = tree.0.join("outside-entrypoint");
        fs::write(&outside, "do not read").expect("fixture target should be written");
        symlink(&outside, tool_dir.join("proton"))
            .expect("synthetic entrypoint symlink should be created");

        let result = ProtonScanner.scan(&[tree.installation()]);
        assert_eq!(
            result.tools[0].status,
            CompatibilityToolStatus::InvalidMetadata
        );
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == CompatibilityToolIssueCode::SymlinkRejected));
    }
}
