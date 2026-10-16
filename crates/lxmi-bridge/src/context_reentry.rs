//! Non-loading late entry probe. Uses the existing fixture coordinator/executor.
use super::*;
use std::os::unix::fs::{FileTypeExt, MetadataExt};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefixFileIdentity {
    pub canonical_path: PathBuf,
    pub device: u64,
    pub inode: u64,
    pub uid: u32,
}
pub fn prefix_file_identity(prefix: &Path) -> Option<PrefixFileIdentity> {
    let path = prefix.canonicalize().ok()?;
    let metadata = fs::metadata(&path).ok()?;
    if !metadata.is_dir() || metadata.uid() != unsafe { libc::getuid() } {
        return None;
    }
    Some(PrefixFileIdentity {
        canonical_path: path,
        device: metadata.dev(),
        inode: metadata.ino(),
        uid: metadata.uid(),
    })
}
pub fn same_prefix_object(
    a: &Option<PrefixFileIdentity>,
    b: &Option<PrefixFileIdentity>,
) -> ContextRelation {
    match (a, b) {
        (Some(a), Some(b)) if (a.uid, a.device, a.inode) == (b.uid, b.device, b.inode) => {
            ContextRelation::Same
        }
        (Some(_), Some(_)) => ContextRelation::Different,
        _ => ContextRelation::Unknown,
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WineServerEndpointEvidence {
    pub state: ContextEvidenceState,
    pub prefix: Option<PrefixFileIdentity>,
    pub derived_socket_path: Option<PathBuf>,
    pub socket_observed: bool,
    pub socket_device: Option<u64>,
    pub socket_inode: Option<u64>,
    pub source: String,
    pub server_pid: Option<u32>,
}
pub(crate) fn endpoint(prefix: &Path) -> WineServerEndpointEvidence {
    let identity = prefix_file_identity(prefix);
    let path = identity.as_ref().map(|i| {
        PathBuf::from(format!(
            "/tmp/.wine-{}/server-{:x}-{:x}/socket",
            i.uid, i.device, i.inode
        ))
    });
    // Metadata only on the derived candidate; no socket connection or process scan.
    let observed = path.as_ref().and_then(|p| {
        let parent = fs::symlink_metadata(p.parent()?).ok()?;
        let base = fs::symlink_metadata(p.parent()?.parent()?).ok()?;
        let m = fs::symlink_metadata(p).ok()?;
        let uid = identity.as_ref()?.uid;
        (base.is_dir()
            && base.uid() == uid
            && parent.is_dir()
            && parent.uid() == uid
            && m.file_type().is_socket()
            && m.uid() == uid)
            .then_some(m)
    });
    WineServerEndpointEvidence { state: ContextEvidenceState::Derived, prefix:identity, derived_socket_path:path,
        socket_observed:observed.is_some(),socket_device:observed.as_ref().map(|m|m.dev()),socket_inode:observed.as_ref().map(|m|m.ino()),
        source:"Wine wine-11.0 endpoint formula UID/dev/inode; candidate socket metadata observed independently. Not proof of peer connection or exact Experimental implementation.".into(), server_pid:None }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefixMetadata {
    pub size: u64,
    pub modified_seconds: i64,
    pub modified_nanoseconds: i64,
    pub inode: u64,
    pub symlink_target: Option<PathBuf>,
}
pub(crate) fn prefix_snapshot(prefix: &Path) -> BTreeMap<String, Option<PrefixMetadata>> {
    [
        "",
        "system.reg",
        "user.reg",
        "userdef.reg",
        "dosdevices",
        "dosdevices/c:",
        "dosdevices/z:",
    ]
    .into_iter()
    .map(|relative| {
        let path = prefix.join(relative);
        let value = fs::symlink_metadata(&path).ok().map(|m| PrefixMetadata {
            size: m.len(),
            modified_seconds: m.mtime(),
            modified_nanoseconds: m.mtime_nsec(),
            inode: m.ino(),
            symlink_target: if m.file_type().is_symlink() {
                fs::read_link(path).ok()
            } else {
                None
            },
        });
        (relative.into(), value)
    })
    .collect()
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReentryEvidence {
    pub target_ready_before_second_invocation: bool,
    pub prefix_a: Option<PrefixFileIdentity>,
    pub prefix_b: Option<PrefixFileIdentity>,
    pub endpoint_a: WineServerEndpointEvidence,
    pub endpoint_b: WineServerEndpointEvidence,
    pub endpoint_a_after_exit: WineServerEndpointEvidence,
    pub prefix_before: BTreeMap<String, Option<PrefixMetadata>>,
    pub prefix_after: BTreeMap<String, Option<PrefixMetadata>>,
}
pub(crate) fn collect_evidence(
    a: &Path,
    b: &Path,
    before: BTreeMap<String, Option<PrefixMetadata>>,
) -> ReentryEvidence {
    ReentryEvidence {
        target_ready_before_second_invocation: true,
        prefix_a: prefix_file_identity(a),
        prefix_b: prefix_file_identity(b),
        endpoint_a: endpoint(a),
        endpoint_b: endpoint(b),
        endpoint_a_after_exit: endpoint(a),
        prefix_before: before,
        prefix_after: BTreeMap::new(),
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextReentryState {
    Verified,
    PrefixMismatch,
    TargetNotVisible,
    NamedObjectNotVisible,
    TargetIdentityMismatch,
    DifferentServerEvidence,
    Ambiguous,
    Unknown,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePrefixEntryRisk {
    pub expected_mutations: Vec<String>,
    pub reversible: ContextEvidenceState,
    pub backup_required: bool,
    pub user_consent_required: bool,
    pub unknowns: Vec<String>,
    pub execution_authorized: bool,
}
pub fn game_prefix_entry_risk() -> GamePrefixEntryRisk {
    GamePrefixEntryRisk {expected_mutations:vec!["Wine registry flushes, prefix initialization/metadata and drive mappings may change during entry".into(),"Proton maintenance is version dependent; runinprefix is not read-only".into()],reversible:ContextEvidenceState::Unknown,backup_required:true,user_consent_required:true,unknowns:vec!["Exact game-prefix mutations and rollback boundary not tested".into()],execution_authorized:false}
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextReentryResult {
    pub result: ContextReentryState,
    pub fixture: DirectContextResult,
    pub prefix_relation: ContextRelation,
    pub named_object_visible: bool,
    pub target_identity_verified: bool,
    pub initial_target_received_event: bool,
    pub game_prefix_risk: GamePrefixEntryRisk,
}
fn classify(
    ready: bool,
    prefix: ContextRelation,
    event: bool,
    received: bool,
    identity: TargetIdentityResult,
) -> ContextReentryState {
    if !ready {
        return ContextReentryState::Unknown;
    }
    if prefix == ContextRelation::Different {
        return ContextReentryState::PrefixMismatch;
    }
    if identity == TargetIdentityResult::PidNotVisible {
        return ContextReentryState::TargetNotVisible;
    }
    if identity != TargetIdentityResult::Verified {
        return ContextReentryState::TargetIdentityMismatch;
    }
    if !event || !received {
        return ContextReentryState::NamedObjectNotVisible;
    }
    if prefix != ContextRelation::Same {
        return ContextReentryState::Unknown;
    }
    ContextReentryState::Verified
}
pub fn run_context_reentry(
    config: &ContextLabConfig,
    application_root: &Path,
    protected_roots: &[PathBuf],
) -> Result<ContextReentryResult, BridgeError> {
    let mut fixture =
        crate::direct_context::run_reentry_core(config, application_root, protected_roots)?;
    if fixture.injector.inject_called
        || fixture
            .source_hashes
            .keys()
            .any(|name| name.ends_with(".dll"))
    {
        return Err(BridgeError::new(
            BridgeErrorCode::ProtocolMismatch,
            "Non-loading probe attempted loader operation",
        ));
    }
    let evidence = fixture.reentry_evidence.as_ref().ok_or_else(|| {
        BridgeError::new(
            BridgeErrorCode::ProtocolMismatch,
            "Target must be ready before helper invocation",
        )
    })?;
    let prefix_relation = same_prefix_object(&evidence.prefix_a, &evidence.prefix_b);
    let named_object_visible = fixture.injector.named_event_visible == Some(true);
    let initial_target_received_event = fixture.target.event_received == Some(true);
    let mut result = classify(
        evidence.target_ready_before_second_invocation,
        prefix_relation,
        named_object_visible,
        initial_target_received_event,
        fixture.injector.target_identity.result,
    );
    if result == ContextReentryState::Verified
        && (!fixture.target.path_readable
            || !fixture.injector.path_readable
            || fixture.identity.marker_path_mapping != ContextRelation::Same
            || fixture.identity.drive_mapping != ContextRelation::Same)
    {
        result = ContextReentryState::Unknown;
    }
    let target_identity_verified =
        DirectInjectPreflightGate::allows(&fixture.injector.target_identity, &fixture.nonce);
    fixture.game_join_plan.loader_context_evidence.push(format!("ContextReentry mode={:?} fixture={result:?}; GamePrefixReentry=NotTested; WineServer=Derived/Unknown; not loader/game compatibility", fixture.mode));
    fixture.warnings=vec!["No loader copied or loaded; no Inject call; only owned PID metadata query".into(),"Same prefix object and observed candidate socket do not prove server PID or universal game requirements".into()];
    Ok(ContextReentryResult {
        result,
        fixture,
        prefix_relation,
        named_object_visible,
        target_identity_verified,
        initial_target_received_event,
        game_prefix_risk: game_prefix_entry_risk(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn file(inode: u64) -> Option<PrefixFileIdentity> {
        Some(PrefixFileIdentity {
            canonical_path: "/fixture".into(),
            device: 1,
            inode,
            uid: 1000,
        })
    }
    #[test]
    fn prefix_device_inode_comparison_is_not_textual() {
        let a = file(2);
        let mut b = a.clone();
        b.as_mut().unwrap().canonical_path = "/alias".into();
        assert_eq!(same_prefix_object(&a, &b), ContextRelation::Same);
        assert_eq!(same_prefix_object(&a, &file(3)), ContextRelation::Different);
        assert_eq!(same_prefix_object(&a, &None), ContextRelation::Unknown);
    }
    #[test]
    fn late_target_and_helper_require_combined_evidence() {
        assert_eq!(
            classify(
                true,
                ContextRelation::Same,
                true,
                true,
                TargetIdentityResult::Verified
            ),
            ContextReentryState::Verified
        );
        assert_eq!(
            classify(
                false,
                ContextRelation::Same,
                true,
                true,
                TargetIdentityResult::Verified
            ),
            ContextReentryState::Unknown
        );
    }
    #[test]
    fn wrong_prefix_control_is_not_verified() {
        assert_eq!(
            classify(
                true,
                ContextRelation::Different,
                false,
                false,
                TargetIdentityResult::PidNotVisible
            ),
            ContextReentryState::PrefixMismatch
        );
    }
    #[test]
    fn filesystem_only_never_proves_reentry() {
        assert_eq!(
            classify(
                true,
                ContextRelation::Same,
                false,
                false,
                TargetIdentityResult::Verified
            ),
            ContextReentryState::NamedObjectNotVisible
        );
    }
    #[test]
    fn disappeared_target_or_creation_mismatch_rejects_entry() {
        for state in [
            TargetIdentityResult::ProcessExited,
            TargetIdentityResult::CreationTimeMismatch,
            TargetIdentityResult::PidCollision,
        ] {
            assert_ne!(
                classify(true, ContextRelation::Same, true, true, state),
                ContextReentryState::Verified
            );
        }
    }
    #[test]
    fn game_risk_is_declarative_and_closed() {
        let r = game_prefix_entry_risk();
        assert!(!r.execution_authorized);
        assert!(r.backup_required && r.user_consent_required);
        assert_eq!(r.reversible, ContextEvidenceState::Unknown);
    }
}
