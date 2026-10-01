//! Read-only real-prefix review. No process executor or restoration API exists here.
use super::*;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::time::{SystemTime, UNIX_EPOCH};

const FILE_LIMIT: u64 = 32 * 1024 * 1024;
const DIRECTORY_LIMIT: usize = 256;
pub const ZZZ_OBSERVED_BUILD: &str = "1790244902 experimental-11.0-20260924-x86_64";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrefixPreflightState {
    EligibleForReview,
    NeedsEvidence,
    Blocked,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeBuildDrift {
    SameBuild,
    DifferentBuild,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollbackCapability {
    Full,
    Partial,
    MetadataOnly,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedPathEvidence {
    pub path: PathBuf,
    pub metadata: Option<PrefixMetadata>,
    pub sha256: Option<String>,
    pub directory_entries: Option<Vec<String>>,
    pub issue: Option<String>,
}

/// Hashes bounded regular files; never follows the final symlink. Directories are
/// shallow metadata inventories, not a claim that their full contents are protected.
fn selected_path(path: &Path, hash: bool, inventory: bool) -> SelectedPathEvidence {
    let mut result = SelectedPathEvidence {
        path: path.into(),
        metadata: None,
        sha256: None,
        directory_entries: None,
        issue: None,
    };
    let inspect = || -> io::Result<(PrefixMetadata, Option<String>, Option<Vec<String>>)> {
        let before = fs::symlink_metadata(path)?;
        let metadata = PrefixMetadata {
            size: before.len(),
            modified_seconds: before.mtime(),
            modified_nanoseconds: before.mtime_nsec(),
            inode: before.ino(),
            symlink_target: if before.file_type().is_symlink() {
                Some(fs::read_link(path)?)
            } else {
                None
            },
        };
        let digest = if hash {
            if !before.is_file() || before.len() > FILE_LIMIT {
                return Err(io::Error::other("not a bounded regular file"));
            }
            let file = OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                .open(path)?;
            let opened = file.metadata()?;
            if !opened.is_file() || (before.dev(), before.ino()) != (opened.dev(), opened.ino()) {
                return Err(io::Error::other("file identity changed"));
            }
            let mut bytes = Vec::new();
            file.take(FILE_LIMIT + 1).read_to_end(&mut bytes)?;
            if bytes.len() as u64 > FILE_LIMIT {
                return Err(io::Error::other("file exceeds policy"));
            }
            Some(format!("{:x}", Sha256::digest(bytes)))
        } else {
            None
        };
        let entries = if inventory {
            if !before.is_dir() {
                return Err(io::Error::other("not a directory"));
            }
            let mut entries = Vec::new();
            for entry in fs::read_dir(path)?.take(DIRECTORY_LIMIT + 1) {
                let entry = entry?;
                let name = entry.file_name().to_string_lossy().to_string();
                let m = fs::symlink_metadata(entry.path())?;
                let link = if m.file_type().is_symlink() {
                    Some(fs::read_link(entry.path())?)
                } else {
                    None
                };
                entries.push(format!(
                    "{name}:{}:{}:{}:{link:?}",
                    m.ino(),
                    m.len(),
                    m.mtime()
                ));
            }
            if entries.len() > DIRECTORY_LIMIT {
                return Err(io::Error::other(
                    "directory exceeds shallow inventory limit",
                ));
            }
            entries.sort();
            Some(entries)
        } else {
            None
        };
        let after = fs::symlink_metadata(path)?;
        if (
            before.dev(),
            before.ino(),
            before.len(),
            before.mtime(),
            before.mtime_nsec(),
        ) != (
            after.dev(),
            after.ino(),
            after.len(),
            after.mtime(),
            after.mtime_nsec(),
        ) {
            return Err(io::Error::other("path changed during snapshot"));
        }
        Ok((metadata, digest, entries))
    };
    match inspect() {
        Ok((metadata, hash, entries)) => {
            result.metadata = Some(metadata);
            result.sha256 = hash;
            result.directory_entries = entries;
        }
        Err(e) => result.issue = Some(format!("{}: {e}", safe_error_kind(&e))),
    }
    result
}
fn safe_error_kind(e: &io::Error) -> &'static str {
    match e.kind() {
        io::ErrorKind::NotFound => "missing",
        io::ErrorKind::PermissionDenied => "permission_denied",
        _ => "unavailable",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePrefixSnapshotPolicy {
    pub file_byte_limit: u64,
    pub directory_entry_limit: usize,
    pub exclusions: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePrefixBaseline {
    pub schema: u16,
    pub timestamp_unix_seconds: u64,
    pub prefix_identity: Option<PrefixFileIdentity>,
    pub selected_prefix_paths: Vec<SelectedPathEvidence>,
    pub protected_paths: Vec<SelectedPathEvidence>,
    pub policy: GamePrefixSnapshotPolicy,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineReceipt {
    pub manifest_path: PathBuf,
    pub sha256: String,
}

/// Declarative review only; no conversion into a process invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePrefixEntryPlan {
    pub helper: String,
    pub timeout_seconds: u16,
    pub user_consent_required: bool,
    pub executor_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePrefixEntryPreflight {
    pub entry_plan: GamePrefixEntryPlan,
    pub app_id: u32,
    pub compatdata: PathBuf,
    pub prefix_identity: Option<PrefixFileIdentity>,
    pub installed_build: Option<String>,
    pub observed_build: String,
    pub runtime_drift: RuntimeBuildDrift,
    pub activity_source: String,
    pub active_processes: usize,
    pub unresolved_processes: usize,
    pub baseline: GamePrefixBaseline,
    pub baseline_receipt: Option<BaselineReceipt>,
    pub rollback: RollbackCapability,
    pub rollback_limitations: Vec<String>,
    pub expected_mutations: Vec<String>,
    pub future_consent_required: bool,
    pub result: PrefixPreflightState,
    pub blockers: Vec<String>,
    pub execution_authorized: bool,
}

/// Inputs are resolved by application discovery, never supplied as frontend paths.
pub struct GamePrefixPreflightInput {
    pub compatdata: PathBuf,
    pub proton_root: PathBuf,
    pub protected_paths: Vec<PathBuf>,
    pub active_processes: usize,
    pub unresolved_processes: usize,
}
pub fn inspect_game_prefix(input: GamePrefixPreflightInput) -> GamePrefixEntryPreflight {
    let prefix = input.compatdata.join("pfx");
    let identity = prefix_file_identity(&prefix);
    let installed_build = File::open(input.proton_root.join("version"))
        .and_then(|file| {
            let mut bytes = Vec::new();
            file.take(4097).read_to_end(&mut bytes)?;
            Ok(bytes)
        })
        .ok()
        .filter(|b| b.len() <= 4096)
        .and_then(|b| String::from_utf8(b).ok())
        .map(|s| s.trim().to_owned());
    let drift = match installed_build.as_deref() {
        Some(ZZZ_OBSERVED_BUILD) => RuntimeBuildDrift::SameBuild,
        Some(_) => RuntimeBuildDrift::DifferentBuild,
        None => RuntimeBuildDrift::Unknown,
    };
    let selected = [
        "",
        "drive_c",
        "dosdevices",
        "system.reg",
        "user.reg",
        "userdef.reg",
    ]
    .into_iter()
    .map(|name| {
        selected_path(
            &prefix.join(name),
            name.ends_with(".reg"),
            name.is_empty() || name == "dosdevices",
        )
    })
    .collect::<Vec<_>>();
    let protected = input
        .protected_paths
        .iter()
        .map(|path| {
            let is_file = fs::symlink_metadata(path).is_ok_and(|m| m.is_file());
            // Large executables use metadata; full installation/package content is excluded.
            let bounded =
                is_file && fs::symlink_metadata(path).is_ok_and(|m| m.len() <= FILE_LIMIT);
            selected_path(path, bounded, false)
        })
        .collect::<Vec<_>>();
    let mut blockers = Vec::new();
    if identity.is_none() {
        blockers.push("prefix_identity_unverified".into());
    }
    if !input.compatdata.ends_with("steamapps/compatdata/4162040") {
        blockers.push("unexpected_compatdata_relationship".into());
    }
    if input.active_processes > 0 {
        blockers.push("game_or_prefix_active".into());
    }
    if input.unresolved_processes > 0 {
        blockers.push("active_context_unknown".into());
    }
    if drift != RuntimeBuildDrift::SameBuild {
        blockers.push("runtime_build_not_approved".into());
    }
    if selected
        .iter()
        .chain(protected.iter())
        .any(|e| e.issue.is_some())
    {
        blockers.push("snapshot_evidence_incomplete".into());
    }
    blockers.push("baseline_not_captured".into());
    blockers.push("rollback_insufficient_metadata_only".into());
    let result = if input.active_processes > 0 {
        PrefixPreflightState::Blocked
    } else {
        PrefixPreflightState::NeedsEvidence
    };
    let baseline = GamePrefixBaseline {schema:1,timestamp_unix_seconds:SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        prefix_identity:identity.clone(),selected_prefix_paths:selected,protected_paths:protected,
        policy:GamePrefixSnapshotPolicy {file_byte_limit:FILE_LIMIT,directory_entry_limit:DIRECTORY_LIMIT,
            exclusions:vec!["No full prefix/game tree, documents, registry values or environment persisted".into(),"Large files and directory contents covered only by selected metadata; no content rollback".into()]}};
    GamePrefixEntryPreflight {entry_plan:GamePrefixEntryPlan {helper:BRIDGE_HELPER_FILE.into(),timeout_seconds:20,user_consent_required:true,executor_available:false},app_id:4162040,compatdata:input.compatdata,prefix_identity:identity,
        installed_build,observed_build:ZZZ_OBSERVED_BUILD.into(),runtime_drift:drift,
        activity_source:"same-UID procfs allowlisted metadata; point-in-time observation, not a prefix lock".into(),
        active_processes:input.active_processes,unresolved_processes:input.unresolved_processes,baseline,baseline_receipt:None,
        rollback:RollbackCapability::MetadataOnly,
        rollback_limitations:vec!["Baseline detects selected changes but contains no restorable file copies".into(),"Proton/Wine may change files outside the selected registry/mapping inventory; full rollback is not established".into()],
        expected_mutations:game_prefix_entry_risk().expected_mutations,future_consent_required:true,result,blockers,execution_authorized:false}
}

/// Writes only a private snapshot manifest in a previously validated LXMI root.
/// Recording evidence never removes the rollback gate or authorizes prefix entry.
pub fn capture_game_prefix_baseline(
    report: &mut GamePrefixEntryPreflight,
    managed_root: &Path,
) -> io::Result<()> {
    let root = managed_root.canonicalize()?;
    let root_metadata = fs::symlink_metadata(&root)?;
    if root_metadata.uid() != unsafe { libc::getuid() } || root_metadata.mode() & 0o077 != 0 {
        return Err(io::Error::other(
            "managed baseline root must be private and owned",
        ));
    }
    if root != managed_root || root.components().any(|c|matches!(c,Component::Normal(n) if n=="Steam"||n=="steamapps"||n=="4162040"||n=="Zenless Zone Zero")) {
        return Err(io::Error::other("baseline storage is not an approved managed root"));
    }
    if report
        .baseline
        .selected_prefix_paths
        .iter()
        .chain(report.baseline.protected_paths.iter())
        .any(|p| p.issue.is_some())
    {
        return Err(io::Error::other("cannot capture incomplete baseline"));
    }
    if prefix_file_identity(&report.compatdata.join("pfx")) != report.prefix_identity {
        return Err(io::Error::other("prefix identity changed"));
    }
    let directory = root.join("snapshots");
    match fs::symlink_metadata(&directory) {
        Ok(m)
            if m.is_dir()
                && m.uid() == unsafe { libc::getuid() }
                && m.mode() & 0o077 == 0
                && !m.file_type().is_symlink() => {}
        Ok(_) => return Err(io::Error::other("unsafe snapshots directory")),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            fs::create_dir(&directory)?;
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))?;
        }
        Err(e) => return Err(e),
    }
    let bytes = serde_json::to_vec_pretty(&report.baseline)?;
    let hash = format!("{:x}", Sha256::digest(&bytes));
    let mut nonce = [0u8; 16];
    getrandom::fill(&mut nonce).map_err(|e| io::Error::other(e.to_string()))?;
    let nonce = nonce.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let path = directory.join(format!("zzz-prefix-{nonce}.json"));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(&path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    report.baseline_receipt = Some(BaselineReceipt {
        manifest_path: path,
        sha256: hash,
    });
    report.blockers.retain(|b| b != "baseline_not_captured");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (PathBuf, GamePrefixPreflightInput) {
        let root = env::temp_dir().join(format!(
            "lxmi-preflight-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let compat = root.join("steamapps/compatdata/4162040");
        let prefix = compat.join("pfx");
        fs::create_dir_all(prefix.join("drive_c")).unwrap();
        fs::create_dir(prefix.join("dosdevices")).unwrap();
        for name in ["system.reg", "user.reg", "userdef.reg"] {
            fs::write(prefix.join(name), b"fixture registry").unwrap();
        }
        let proton = root.join("proton");
        fs::create_dir(&proton).unwrap();
        fs::write(proton.join("version"), ZZZ_OBSERVED_BUILD).unwrap();
        (
            root,
            GamePrefixPreflightInput {
                compatdata: compat,
                proton_root: proton,
                protected_paths: Vec::new(),
                active_processes: 0,
                unresolved_processes: 0,
            },
        )
    }
    #[test]
    fn baseline_is_metadata_only_and_does_not_unlock_entry() {
        let (root, input) = fixture();
        let mut report = inspect_game_prefix(input);
        let storage = root.join("lxmi");
        fs::create_dir(&storage).unwrap();
        fs::set_permissions(&storage, fs::Permissions::from_mode(0o700)).unwrap();
        capture_game_prefix_baseline(&mut report, &storage).unwrap();
        let receipt = report.baseline_receipt.as_ref().unwrap();
        assert_eq!(
            receipt.sha256,
            format!(
                "{:x}",
                Sha256::digest(fs::read(&receipt.manifest_path).unwrap())
            )
        );
        assert_eq!(report.rollback, RollbackCapability::MetadataOnly);
        assert_eq!(report.result, PrefixPreflightState::NeedsEvidence);
        assert!(!report.execution_authorized);
        assert!(report
            .blockers
            .iter()
            .any(|b| b == "rollback_insufficient_metadata_only"));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn active_context_blocks_and_unknown_never_passes() {
        for (active, unknown, expected) in [
            (1, 0, PrefixPreflightState::Blocked),
            (0, 1, PrefixPreflightState::NeedsEvidence),
        ] {
            let (root, mut input) = fixture();
            input.active_processes = active;
            input.unresolved_processes = unknown;
            assert_eq!(inspect_game_prefix(input).result, expected);
            fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    fn changed_build_and_missing_registry_remain_unapproved() {
        let (root, input) = fixture();
        fs::write(input.proton_root.join("version"), "another build").unwrap();
        fs::remove_file(input.compatdata.join("pfx/user.reg")).unwrap();
        let report = inspect_game_prefix(input);
        assert_eq!(report.runtime_drift, RuntimeBuildDrift::DifferentBuild);
        assert!(report
            .blockers
            .iter()
            .any(|b| b == "snapshot_evidence_incomplete"));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn symlink_registry_is_not_hashed_and_game_baseline_destination_rejected() {
        let (root, input) = fixture();
        let registry = input.compatdata.join("pfx/user.reg");
        fs::remove_file(&registry).unwrap();
        std::os::unix::fs::symlink("system.reg", &registry).unwrap();
        let mut report = inspect_game_prefix(input);
        assert!(report
            .baseline
            .selected_prefix_paths
            .iter()
            .find(|p| p.path == registry)
            .unwrap()
            .sha256
            .is_none());
        let game_destination = report.compatdata.clone();
        assert!(capture_game_prefix_baseline(&mut report, &game_destination).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
