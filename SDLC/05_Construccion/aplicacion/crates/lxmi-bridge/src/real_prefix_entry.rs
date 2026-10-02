//! One-shot harmless helper entry. No game executable, loader or PID is accepted.
use super::*;
use std::os::unix::fs::{MetadataExt, PermissionsExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealPrefixEntryState {
    Verified,
    VerifiedWithExpectedMutations,
    ConsentMissing,
    ActiveContext,
    RuntimeChanged,
    PrefixIdentityChanged,
    BackupInvalid,
    HelperFailed,
    ProtectedPathChanged,
    UnexpectedMutation,
    LinuxEnvironmentUnqualified,
    Blocked,
}

pub const HARMLESS_ENTRY_ENVIRONMENT_BLOCKER: &str =
    "Linux environment is not qualified for another real-prefix entry. The fixture reproduced font-table changes when switching between Steam Linux Runtime and direct Proton, and device ContainerId changes also occurred on repeated execution. No automatic retry or restore; the original UnexpectedMutation remains unchanged.";

/// The only implemented real-prefix executor is direct Proton. Its Linux environment
/// and mutation policy have not been qualified after the stopped host experiment.
/// Neither fresh consent nor a new baseline approves that environment.
pub fn require_qualified_harmless_entry_environment() -> Result<(), RealPrefixEntryState> {
    Err(RealPrefixEntryState::LinuxEnvironmentUnqualified)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrefixMutationKind {
    ExpectedWineMutation,
    PossibleWineMutation,
    ProtectedPathMutation,
    Unknown,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefixMutation {
    pub path: PathBuf,
    pub kind: PrefixMutationKind,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealPrefixEntryResult {
    pub schema: u16,
    pub session_nonce: String,
    pub result: RealPrefixEntryState,
    pub consent_accepted: bool,
    pub helper_attempted: bool,
    pub helper: Option<BridgeTestResult>,
    pub helper_error: Option<BridgeError>,
    pub before: GamePrefixEntryPreflight,
    pub after: Option<GamePrefixEntryPreflight>,
    pub mutations: Vec<PrefixMutation>,
    pub backup_id: String,
    pub backup_manifest_sha256: String,
    pub backup_verified_after: bool,
    pub prefix_identity_unchanged: bool,
    pub protected_paths_unchanged: bool,
    pub runtime_unchanged: bool,
    pub game_context_inactive_after: bool,
    pub evidence_receipt: Option<BaselineReceipt>,
    pub warnings: Vec<String>,
}
impl RealPrefixEntryResult {
    pub fn gate_passed(&self) -> bool {
        matches!(
            self.result,
            RealPrefixEntryState::Verified | RealPrefixEntryState::VerifiedWithExpectedMutations
        ) && self.consent_accepted
            && self.helper.is_some()
            && self.backup_verified_after
            && self.prefix_identity_unchanged
            && self.protected_paths_unchanged
            && self.runtime_unchanged
            && self.game_context_inactive_after
    }
}
/// No public constructor or fields: validation is mandatory before execution.
pub struct ApprovedHarmlessPrefixEntry {
    config: BridgeTestConfig,
    capability: RealPrefixInvocation,
}
pub fn validate_harmless_prefix_entry(
    root: &Path,
    runtime: ExplicitBridgeRuntime,
    steam_root: PathBuf,
    before: &GamePrefixEntryPreflight,
    recovery: &PrefixRecoveryQualification,
    consent: bool,
    nonce: &str,
) -> Result<ApprovedHarmlessPrefixEntry, RealPrefixEntryState> {
    if !consent {
        return Err(RealPrefixEntryState::ConsentMissing);
    }
    if before.active_processes != 0 || before.unresolved_processes != 0 {
        return Err(RealPrefixEntryState::ActiveContext);
    }
    require_qualified_harmless_entry_environment()?;
    if before.runtime_drift != RuntimeBuildDrift::SameBuild
        || before.installed_build.as_deref() != Some(ZZZ_OBSERVED_BUILD)
    {
        return Err(RealPrefixEntryState::RuntimeChanged);
    }
    let identity = before
        .prefix_identity
        .as_ref()
        .ok_or(RealPrefixEntryState::PrefixIdentityChanged)?;
    if before.app_id != 4162040
        || !before.compatdata.ends_with("steamapps/compatdata/4162040")
        || before.compatdata.canonicalize().ok().as_ref() != Some(&before.compatdata)
        || identity.canonical_path != before.compatdata.join("pfx")
        || prefix_file_identity(&identity.canonical_path).as_ref() != Some(identity)
        || identity != &recovery.source_identity
    {
        return Err(RealPrefixEntryState::PrefixIdentityChanged);
    }
    if recovery.integrity.state != RecoveryVerificationState::Verified
        || recovery
            .restore_verification
            .as_ref()
            .is_none_or(|r| r.state != RecoveryVerificationState::Verified)
        || !recovery.source_unchanged_during_capture
        || recovery.protected_paths_unchanged != Some(true)
        || recovery.proton_build != ZZZ_OBSERVED_BUILD
    {
        return Err(RealPrefixEntryState::BackupInvalid);
    }
    if before.result != PrefixPreflightState::EligibleForReview
        || before.baseline_receipt.is_none()
        || !before.blockers.is_empty()
        || before.baseline.protected_paths.is_empty()
        || before
            .baseline
            .selected_prefix_paths
            .iter()
            .chain(&before.baseline.protected_paths)
            .any(|e| e.issue.is_some())
    {
        return Err(RealPrefixEntryState::Blocked);
    }
    if nonce.len() != 32 || !nonce.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(RealPrefixEntryState::Blocked);
    }
    let root = root
        .canonicalize()
        .map_err(|_| RealPrefixEntryState::Blocked)?;
    let meta = fs::symlink_metadata(&root).map_err(|_| RealPrefixEntryState::Blocked)?;
    if meta.uid()!=unsafe{libc::getuid()} || meta.mode() & 0o077 != 0
        || root.components().any(|c| matches!(c,Component::Normal(n) if n=="Steam" || n=="steamapps" || n=="4162040" || n=="Zenless Zone Zero")) {
        return Err(RealPrefixEntryState::Blocked);
    }
    let script = validate_proton_script(&runtime.proton_script)
        .map_err(|_| RealPrefixEntryState::RuntimeChanged)?;
    let tool_root = script
        .parent()
        .ok_or(RealPrefixEntryState::RuntimeChanged)?;
    if fs::read_to_string(tool_root.join("version"))
        .ok()
        .as_deref()
        .map(str::trim)
        != Some(ZZZ_OBSERVED_BUILD)
        || tool_root
            .file_name()
            .is_none_or(|n| n != "Proton - Experimental")
    {
        return Err(RealPrefixEntryState::RuntimeChanged);
    }
    match inspect_bridge_options(&root, None).helper {
        HelperState::Available {
            integrity_matches: true,
            ..
        } => {}
        _ => return Err(RealPrefixEntryState::HelperFailed),
    }
    // New files are only under LXMI, never in the prefix or managed runtime.
    let session = root.join("tests/real-prefix-entry-v1").join(nonce);
    ensure_controlled_directory(&root, &session).map_err(|_| RealPrefixEntryState::Blocked)?;
    fs::set_permissions(&session, fs::Permissions::from_mode(0o700))
        .map_err(|_| RealPrefixEntryState::Blocked)?;
    for dir in ["home", "config", "cache", "data", "marker"] {
        ensure_controlled_directory(&root, &session.join(dir))
            .map_err(|_| RealPrefixEntryState::Blocked)?;
    }
    let marker = serde_json::to_vec(&serde_json::json!({"schema":1,"operation":"harmless_real_prefix_entry","nonce":nonce,"created_at_unix_seconds":before.baseline.timestamp_unix_seconds})).map_err(|_|RealPrefixEntryState::Blocked)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(session.join("marker").join(MARKER_FILE))
        .map_err(|_| RealPrefixEntryState::Blocked)?;
    file.write_all(&marker)
        .and_then(|()| file.sync_all())
        .map_err(|_| RealPrefixEntryState::Blocked)?;
    Ok(ApprovedHarmlessPrefixEntry {
        config: BridgeTestConfig {
            managed_root: root,
            runtime_root: session.join("marker"),
            explicit_runtime: runtime,
            steam_client_install_path: steam_root,
            prefix_mode: PrefixMode::ApprovedHarmlessGamePrefix,
        },
        capability: RealPrefixInvocation {
            compatdata: before.compatdata.clone(),
            working_directory: session,
            nonce: nonce.into(),
        },
    })
}
/// Reuses the 0.6 protocol/executor; this capability cannot choose any executable.
pub fn execute_harmless_prefix_entry(
    plan: ApprovedHarmlessPrefixEntry,
) -> Result<BridgeTestResult, BridgeError> {
    require_qualified_harmless_entry_environment().map_err(|_| {
        BridgeError::new(
            BridgeErrorCode::UnsafePrefixContext,
            HARMLESS_ENTRY_ENVIRONMENT_BLOCKER,
        )
    })?;
    run_helper_bridge(&plan.config, &NativeProcessExecutor, Some(&plan.capability))
}

pub fn compare_prefix_entry(
    before: &GamePrefixBaseline,
    after: &GamePrefixBaseline,
) -> Vec<PrefixMutation> {
    let mut changes = Vec::new();
    for (protected, old, new) in [
        (true, &before.protected_paths, &after.protected_paths),
        (
            false,
            &before.selected_prefix_paths,
            &after.selected_prefix_paths,
        ),
    ] {
        let paths = old
            .iter()
            .chain(new)
            .map(|p| p.path.clone())
            .collect::<std::collections::BTreeSet<_>>();
        for path in paths {
            let a = old.iter().find(|p| p.path == path);
            let b = new.iter().find(|p| p.path == path);
            if a == b {
                continue;
            }
            let (kind, reason) = if protected {
                (
                    PrefixMutationKind::ProtectedPathMutation,
                    "Protected evidence changed; attribution is not assumed",
                )
            } else if a.zip(b).is_some_and(|(a, b)| {
                a.issue.is_none()
                    && b.issue.is_none()
                    && a.sha256.is_some()
                    && a.sha256 == b.sha256
                    && a.directory_entries == b.directory_entries
                    && a.path.extension().is_some_and(|e| e == "reg")
            }) {
                (PrefixMutationKind::ExpectedWineMutation,"Registry metadata changed with identical content hash; consistent with registry flush, not proof of cause")
            } else if path.extension().is_some_and(|e| e == "reg") {
                (
                    PrefixMutationKind::PossibleWineMutation,
                    "Registry content or accessibility changed; not automatically accepted",
                )
            } else {
                (PrefixMutationKind::Unknown,"Structure/mapping/metadata changed outside the narrow accepted registry-metadata policy")
            };
            changes.push(PrefixMutation {
                path,
                kind,
                reason: reason.into(),
            });
        }
    }
    changes
}
/// Safe diagnostic: path/field names only, never configuration contents or values.
pub fn describe_protected_path_changes(
    before: &GamePrefixBaseline,
    after: &GamePrefixBaseline,
) -> String {
    let old = &before.protected_paths;
    let new = &after.protected_paths;
    let paths = old
        .iter()
        .chain(new)
        .map(|p| p.path.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let mut details = Vec::new();
    for path in paths {
        let a = old.iter().find(|e| e.path == path);
        let b = new.iter().find(|e| e.path == path);
        if a == b {
            continue;
        }
        let fields = match (a, b) {
            (Some(a), Some(b)) => {
                let mut fields = Vec::new();
                if a.metadata != b.metadata {
                    fields.push("metadata");
                }
                if a.sha256 != b.sha256 {
                    fields.push("content hash");
                }
                if a.directory_entries != b.directory_entries {
                    fields.push("directory inventory");
                }
                if a.issue != b.issue {
                    fields.push("readability");
                }
                fields.join(", ")
            }
            _ => "protected path set".into(),
        };
        details.push(format!("{} ({fields})", path.display()));
    }
    format!("Stopped before Proton/helper execution. Protected evidence changed since review: {}. No retry or restore was performed; writer/cause is unknown.",details.join("; "))
}

pub fn classify_prefix_entry(result: &RealPrefixEntryResult) -> RealPrefixEntryState {
    if !result.consent_accepted {
        return RealPrefixEntryState::ConsentMissing;
    }
    if result
        .mutations
        .iter()
        .any(|m| m.kind == PrefixMutationKind::ProtectedPathMutation)
        || !result.protected_paths_unchanged
    {
        return RealPrefixEntryState::ProtectedPathChanged;
    }
    if !result.prefix_identity_unchanged {
        return RealPrefixEntryState::PrefixIdentityChanged;
    }
    if !result.runtime_unchanged {
        return RealPrefixEntryState::RuntimeChanged;
    }
    if !result.backup_verified_after {
        return RealPrefixEntryState::BackupInvalid;
    }
    if !result.game_context_inactive_after {
        return RealPrefixEntryState::ActiveContext;
    }
    if result.helper.is_none() || result.helper_error.is_some() {
        return RealPrefixEntryState::HelperFailed;
    }
    if result
        .mutations
        .iter()
        .any(|m| m.kind != PrefixMutationKind::ExpectedWineMutation)
    {
        return RealPrefixEntryState::UnexpectedMutation;
    }
    if result.mutations.is_empty() {
        RealPrefixEntryState::Verified
    } else {
        RealPrefixEntryState::VerifiedWithExpectedMutations
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixture_environment_diagnosis_does_not_authorize_real_entry() {
        assert_eq!(
            require_qualified_harmless_entry_environment(),
            Err(RealPrefixEntryState::LinuxEnvironmentUnqualified)
        );
        let mut result = result_fixture();
        result.result = RealPrefixEntryState::LinuxEnvironmentUnqualified;
        assert!(!result.gate_passed());
    }
    fn baseline() -> GamePrefixBaseline {
        GamePrefixBaseline {
            schema: 1,
            timestamp_unix_seconds: 0,
            prefix_identity: None,
            selected_prefix_paths: vec![SelectedPathEvidence {
                path: "/fixture/system.reg".into(),
                metadata: Some(PrefixMetadata {
                    size: 3,
                    modified_seconds: 0,
                    modified_nanoseconds: 0,
                    inode: 1,
                    symlink_target: None,
                }),
                sha256: Some("abc".into()),
                directory_entries: None,
                issue: None,
            }],
            protected_paths: vec![],
            policy: GamePrefixSnapshotPolicy {
                file_byte_limit: 1,
                directory_entry_limit: 1,
                exclusions: vec![],
            },
        }
    }
    #[test]
    fn registry_changes_are_not_silently_accepted() {
        let a = baseline();
        let mut b = a.clone();
        b.selected_prefix_paths[0]
            .metadata
            .as_mut()
            .unwrap()
            .modified_seconds = 1;
        assert_eq!(
            compare_prefix_entry(&a, &b)[0].kind,
            PrefixMutationKind::ExpectedWineMutation
        );
        b.selected_prefix_paths[0].sha256 = Some("changed".into());
        assert_eq!(
            compare_prefix_entry(&a, &b)[0].kind,
            PrefixMutationKind::PossibleWineMutation
        );
        b.selected_prefix_paths[0].issue = Some("denied".into());
        assert_eq!(
            compare_prefix_entry(&a, &b)[0].kind,
            PrefixMutationKind::PossibleWineMutation
        );
    }
    #[test]
    fn protected_changes_and_mapping_changes_block() {
        let mut a = baseline();
        a.protected_paths = a.selected_prefix_paths.clone();
        let mut b = a.clone();
        b.protected_paths.clear();
        assert_eq!(
            compare_prefix_entry(&a, &b)[0].kind,
            PrefixMutationKind::ProtectedPathMutation
        );
        a.protected_paths.clear();
        a.selected_prefix_paths[0].path = "/fixture/dosdevices".into();
        b = a.clone();
        b.selected_prefix_paths[0].directory_entries = Some(vec!["different".into()]);
        assert_eq!(
            compare_prefix_entry(&a, &b)[0].kind,
            PrefixMutationKind::Unknown
        );
    }
    fn result_fixture() -> RealPrefixEntryResult {
        let before = inspect_game_prefix(GamePrefixPreflightInput {
            compatdata: "/absent/steamapps/compatdata/4162040".into(),
            proton_root: "/absent/proton".into(),
            protected_paths: vec![],
            active_processes: 0,
            unresolved_processes: 0,
        });
        RealPrefixEntryResult {
            schema: 1,
            session_nonce: "0123456789abcdef0123456789abcdef".into(),
            result: RealPrefixEntryState::Blocked,
            consent_accepted: true,
            helper_attempted: false,
            helper: None,
            helper_error: None,
            before,
            after: None,
            mutations: vec![],
            backup_id: "fixture".into(),
            backup_manifest_sha256: "fixture".into(),
            backup_verified_after: true,
            prefix_identity_unchanged: true,
            protected_paths_unchanged: true,
            runtime_unchanged: true,
            game_context_inactive_after: true,
            evidence_receipt: None,
            warnings: vec![],
        }
    }
    #[test]
    fn failure_states_never_open_gate() {
        let base = result_fixture();
        assert_eq!(
            classify_prefix_entry(&base),
            RealPrefixEntryState::HelperFailed
        );
        let mut r = base.clone();
        r.consent_accepted = false;
        assert_eq!(
            classify_prefix_entry(&r),
            RealPrefixEntryState::ConsentMissing
        );
        let mut r = base.clone();
        r.prefix_identity_unchanged = false;
        assert_eq!(
            classify_prefix_entry(&r),
            RealPrefixEntryState::PrefixIdentityChanged
        );
        let mut r = base.clone();
        r.runtime_unchanged = false;
        assert_eq!(
            classify_prefix_entry(&r),
            RealPrefixEntryState::RuntimeChanged
        );
        let mut r = base.clone();
        r.backup_verified_after = false;
        assert_eq!(
            classify_prefix_entry(&r),
            RealPrefixEntryState::BackupInvalid
        );
        let mut r = base.clone();
        r.game_context_inactive_after = false;
        assert_eq!(
            classify_prefix_entry(&r),
            RealPrefixEntryState::ActiveContext
        );
        let mut r = base.clone();
        r.protected_paths_unchanged = false;
        assert_eq!(
            classify_prefix_entry(&r),
            RealPrefixEntryState::ProtectedPathChanged
        );
        for state in [
            RealPrefixEntryState::ConsentMissing,
            RealPrefixEntryState::ActiveContext,
            RealPrefixEntryState::RuntimeChanged,
            RealPrefixEntryState::PrefixIdentityChanged,
            RealPrefixEntryState::BackupInvalid,
            RealPrefixEntryState::HelperFailed,
            RealPrefixEntryState::ProtectedPathChanged,
            RealPrefixEntryState::UnexpectedMutation,
            RealPrefixEntryState::Blocked,
        ] {
            let mut r = base.clone();
            r.result = state;
            assert!(!r.gate_passed());
        }
    }
    #[test]
    fn consent_and_activity_rejected_before_any_marker_or_execution() {
        let mut r = result_fixture();
        let recovery:PrefixRecoveryQualification=serde_json::from_value(serde_json::json!({
            "protected_paths_unchanged":true,"backup_id":"fixture","backup_path":"/not-created","manifest_sha256":"fixture","source_identity":{"canonical_path":"/not-created","device":0,"inode":0,"uid":0},"proton_build":"fixture","strategy":"fixture","source_size":{"apparent_bytes":0,"allocated_bytes":0,"files":0,"directories":0,"symlinks":0},"backup_size":{"apparent_bytes":0,"allocated_bytes":0,"files":0,"directories":0,"symlinks":0},"available_bytes_before":0,"available_bytes_after":0,"integrity":{"state":"verified","matching_files":0,"matching_entries":0,"mismatches":[],"unsupported_metadata":[]},"source_unchanged_during_capture":true,"restore_path":null,"restore_verification":null,"real_prefix_restore":"not_tested","rollback":"metadata_only","restore_plan":{"destination":"/not-created","game_closed_required":true,"inactive_context_required":true,"identity_revalidation_required":true,"verified_backup_required":true,"explicit_confirmation_required":true,"protected_paths_required":true,"real_target_executor_available":false},"retention_policy":"fixture"
        })).unwrap();
        let runtime = ExplicitBridgeRuntime {
            display_name: "fixture".into(),
            version: None,
            proton_script: "/never-executed".into(),
        };
        assert!(matches!(
            validate_harmless_prefix_entry(
                Path::new("/never-created"),
                runtime.clone(),
                "/steam".into(),
                &r.before,
                &recovery,
                false,
                &r.session_nonce
            ),
            Err(RealPrefixEntryState::ConsentMissing)
        ));
        r.before.active_processes = 1;
        assert!(matches!(
            validate_harmless_prefix_entry(
                Path::new("/never-created"),
                runtime,
                "/steam".into(),
                &r.before,
                &recovery,
                true,
                &r.session_nonce
            ),
            Err(RealPrefixEntryState::ActiveContext)
        ));
    }
    #[test]
    fn protected_change_diagnostic_names_fields_without_private_values() {
        let mut a = baseline();
        a.protected_paths = a.selected_prefix_paths.clone();
        let mut b = a.clone();
        b.protected_paths[0].metadata.as_mut().unwrap().inode = 2;
        b.protected_paths[0].sha256 = Some("private-hash-not-for-diagnostic".into());
        let message = describe_protected_path_changes(&a, &b);
        assert!(message.contains("/fixture/system.reg (metadata, content hash)"));
        assert!(message.contains("before Proton/helper"));
        assert!(!message.contains("private-hash-not-for-diagnostic"));
        b.protected_paths.clear();
        assert!(describe_protected_path_changes(&a, &b).contains("protected path set"));
    }
}
