//! One independently authorized 0.8.8 experiment; append-only evidence in LXMI storage.
//! No reset, arbitrary destination, process or executor API.
use super::*;

const SLOT: &str = "current-source-entry-0_8_8-authorization.json";
const DIRECTORY: &str = "real-prefix-entry-experiments";
pub const CURRENT_ENTRY_BACKUP: &str = "2b29b84bbc592fa091950bf046b3d668";
pub const CURRENT_ENTRY_BACKUP_SHA: &str =
    "79685d4cdf3523d6acd5ab435a33a175438f488b871d13be3dd6cae0b8770597";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryExperimentStatus {
    PendingReview,
    ReadyForConsent,
    Authorized,
    Running,
    Succeeded,
    Blocked,
    Failed,
}
impl EntryExperimentStatus {
    pub fn consumed(self) -> bool {
        matches!(
            self,
            Self::Authorized | Self::Running | Self::Succeeded | Self::Blocked | Self::Failed
        )
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealPrefixEntryExperiment {
    pub schema: u16,
    pub experiment_id: String,
    pub session_nonce: String,
    pub created_at: u64,
    pub app_id: u32,
    pub source_baseline: BaselineReceipt,
    pub recovery_backup: String,
    pub recovery_manifest_sha256: String,
    pub executor_fingerprint: String,
    pub prefix_identity: PrefixFileIdentity,
    pub runtime_build: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryExperimentReview {
    pub experiment_id: String,
    pub reviewed_at: u64,
    pub review_token_sha256: String,
    pub source_baseline: BaselineReceipt,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryExperimentTerminal {
    pub experiment_id: String,
    pub status: EntryExperimentStatus,
    pub ended_at: u64,
    pub reason: String,
    pub result_receipt: Option<BaselineReceipt>,
}
#[derive(Debug, Clone, Serialize)]
pub struct EntryExperimentHistory {
    pub experiment: RealPrefixEntryExperiment,
    pub status: EntryExperimentStatus,
    pub consumed: bool,
    pub review: Option<EntryExperimentReview>,
    pub terminal: Option<EntryExperimentTerminal>,
    pub result: Option<RealPrefixEntryResult>,
}
#[derive(Serialize, Deserialize)]
struct Claim {
    experiment_id: String,
    review_token_sha256: String,
    accepted_at: u64,
}
fn now() -> io::Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_secs())
}
fn digest_valid(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
pub fn entry_review_token_hash(token: &str) -> io::Result<String> {
    validate_id(token)?;
    Ok(format!("{:x}", Sha256::digest(token.as_bytes())))
}
fn owned_directory(parent: &File, name: &str, create: bool) -> io::Result<File> {
    component(name)?;
    if create {
        match fs::DirBuilder::new()
            .mode(0o700)
            .create(fd_path(parent).join(name))
        {
            Ok(()) => parent.sync_all()?,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
    }
    let dir = descend(parent, name)?;
    if dir.metadata()?.uid() != unsafe { libc::getuid() } || dir.metadata()?.mode() & 0o077 != 0 {
        return Err(err("unsafe_experiment_directory"));
    }
    Ok(dir)
}
fn tests_directory(root: &Path) -> io::Result<File> {
    let managed = storage(root)?;
    // Legacy tests/ can be 0775 inside the already-private LXMI ancestor.
    let dir = descend(&managed, "tests")?;
    if dir.metadata()?.uid() != unsafe { libc::getuid() } {
        return Err(err("owned_experiment_tests_required"));
    }
    Ok(dir)
}
fn event_directory(root: &Path, id: &str, create: bool) -> io::Result<File> {
    validate_id(id)?;
    let tests = tests_directory(root)?;
    let experiments = owned_directory(&tests, DIRECTORY, create)?;
    owned_directory(&experiments, id, create)
}
fn write_once<T: Serialize>(dir: &File, name: &str, value: &T) -> io::Result<()> {
    component(name)?;
    let bytes = serde_json::to_vec_pretty(value)?;
    if bytes.len() > 64 * 1024 {
        return Err(err("experiment_event_limit"));
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(fd_path(dir).join(name))?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    dir.sync_all()
}
fn read_optional<T: serde::de::DeserializeOwned>(dir: &File, name: &str) -> io::Result<Option<T>> {
    let file = match read_file(dir, name) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let before = file.metadata()?;
    if before.uid() != unsafe { libc::getuid() }
        || before.mode() & 0o077 != 0
        || before.len() > 64 * 1024
    {
        return Err(err("unsafe_experiment_event"));
    }
    let mut bytes = Vec::new();
    (&file).take(64 * 1024 + 1).read_to_end(&mut bytes)?;
    let after = file.metadata()?;
    if bytes.len() > 64 * 1024
        || before.len() != after.len()
        || before.ctime() != after.ctime()
        || before.ctime_nsec() != after.ctime_nsec()
    {
        return Err(err("experiment_event_changed_during_read"));
    }
    Ok(Some(serde_json::from_slice(&bytes)?))
}
fn validate_experiment(e: &RealPrefixEntryExperiment) -> io::Result<()> {
    validate_id(&e.experiment_id)?;
    validate_id(&e.session_nonce)?;
    if e.schema != 1
        || e.app_id != 4162040
        || e.experiment_id == e.session_nonce
        || e.recovery_backup != CURRENT_ENTRY_BACKUP
        || e.recovery_manifest_sha256 != CURRENT_ENTRY_BACKUP_SHA
        || e.runtime_build != ZZZ_OBSERVED_BUILD
        || !digest_valid(&e.executor_fingerprint)
        || !digest_valid(&e.source_baseline.sha256)
    {
        return Err(err("experiment_binding_invalid"));
    }
    Ok(())
}
/// The fixed authorization slot is create-new, so this prompt can create only ONE experiment.
pub fn create_independent_entry_experiment(
    root: &Path,
    experiment: &RealPrefixEntryExperiment,
) -> io::Result<EntryExperimentHistory> {
    validate_experiment(experiment)?;
    let tests = tests_directory(root)?;
    write_once(
        &tests,
        SLOT,
        &serde_json::json!({"schema":1,"experiment_id":experiment.experiment_id}),
    )?;
    let dir = event_directory(root, &experiment.experiment_id, true)?;
    write_once(&dir, "reservation.json", experiment)?;
    read_independent_entry_experiment(root)?.ok_or_else(|| err("experiment_reservation_missing"))
}
/// Display-only; interrupted Authorized/Running records remain consumed and cannot resume.
pub fn read_independent_entry_experiment(
    root: &Path,
) -> io::Result<Option<EntryExperimentHistory>> {
    let tests = match tests_directory(root) {
        Ok(v) => v,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let Some(slot) = read_optional::<serde_json::Value>(&tests, SLOT)? else {
        return Ok(None);
    };
    let id = slot["experiment_id"]
        .as_str()
        .ok_or_else(|| err("experiment_slot_invalid"))?;
    if slot["schema"] != 1 {
        return Err(err("experiment_slot_invalid"));
    }
    let dir = event_directory(root, id, false)?;
    let e: RealPrefixEntryExperiment = read_optional(&dir, "reservation.json")?
        .ok_or_else(|| err("experiment_reservation_missing"))?;
    validate_experiment(&e)?;
    if e.experiment_id != id {
        return Err(err("experiment_slot_identity_mismatch"));
    }
    let review: Option<EntryExperimentReview> = read_optional(&dir, "review.json")?;
    let claim: Option<Claim> = read_optional(&dir, "authorized.json")?;
    let running: Option<Claim> = read_optional(&dir, "running.json")?;
    let terminal: Option<EntryExperimentTerminal> = read_optional(&dir, "terminal.json")?;
    if review
        .as_ref()
        .is_some_and(|r| r.experiment_id != id || !digest_valid(&r.review_token_sha256))
        || claim.as_ref().is_some_and(|c| {
            c.experiment_id != id
                || review
                    .as_ref()
                    .is_none_or(|r| r.review_token_sha256 != c.review_token_sha256)
        })
        || running.as_ref().is_some_and(|c| {
            claim.as_ref().is_none_or(|a| {
                a.experiment_id != c.experiment_id || a.review_token_sha256 != c.review_token_sha256
            })
        })
        || terminal.as_ref().is_some_and(|t| {
            t.experiment_id != id
                || !matches!(
                    t.status,
                    EntryExperimentStatus::Succeeded
                        | EntryExperimentStatus::Blocked
                        | EntryExperimentStatus::Failed
                )
        })
    {
        return Err(err("experiment_event_identity_mismatch"));
    }
    let status = if let Some(t) = &terminal {
        t.status
    } else if running.is_some() {
        EntryExperimentStatus::Running
    } else if claim.is_some() {
        EntryExperimentStatus::Authorized
    } else if review.is_some() {
        EntryExperimentStatus::ReadyForConsent
    } else {
        EntryExperimentStatus::PendingReview
    };
    if matches!(
        status,
        EntryExperimentStatus::Succeeded | EntryExperimentStatus::Failed
    ) && running.is_none()
    {
        return Err(err("experiment_terminal_without_running"));
    }
    let result = if let Some(receipt) = terminal.as_ref().and_then(|t| t.result_receipt.as_ref()) {
        let name = format!("real-prefix-entry-{}-after.json", e.session_nonce);
        if receipt.manifest_path != root.join("snapshots").join(&name) {
            return Err(err("experiment_result_path_mismatch"));
        }
        let content = read_prefix_recovery_evidence(root, &name, &receipt.sha256)?;
        let mut r: RealPrefixEntryResult = serde_json::from_slice(&content)?;
        if r.experiment_id.as_deref() != Some(id)
            || r.session_nonce != e.session_nonce
            || r.backup_id != e.recovery_backup
            || r.backup_manifest_sha256 != e.recovery_manifest_sha256
            || r.executor_recipe_fingerprint.as_deref() != Some(&e.executor_fingerprint)
            || (status == EntryExperimentStatus::Succeeded) != r.gate_passed()
        {
            return Err(err("experiment_result_identity_mismatch"));
        }
        r.evidence_receipt = Some(receipt.clone());
        Some(r)
    } else {
        None
    };
    if status == EntryExperimentStatus::Succeeded && result.is_none() {
        return Err(err("successful_experiment_result_missing"));
    }
    Ok(Some(EntryExperimentHistory {
        experiment: e,
        status,
        consumed: status.consumed(),
        review,
        terminal,
        result,
    }))
}
fn require_current(root: &Path, id: &str) -> io::Result<EntryExperimentHistory> {
    validate_id(id)?;
    let history = read_independent_entry_experiment(root)?
        .ok_or_else(|| err("independent_experiment_required"))?;
    if history.experiment.experiment_id != id {
        return Err(err("experiment_id_mismatch"));
    }
    Ok(history)
}
pub fn review_independent_entry_experiment(
    root: &Path,
    review: &EntryExperimentReview,
) -> io::Result<()> {
    let h = require_current(root, &review.experiment_id)?;
    if h.status != EntryExperimentStatus::PendingReview
        || !digest_valid(&review.review_token_sha256)
    {
        return Err(err("experiment_not_pending_review"));
    }
    write_once(
        &event_directory(root, &review.experiment_id, false)?,
        "review.json",
        review,
    )
}
pub fn consume_independent_entry_experiment(root: &Path, id: &str, token: &str) -> io::Result<()> {
    let h = require_current(root, id)?;
    let token_hash = entry_review_token_hash(token)?;
    if h.status != EntryExperimentStatus::ReadyForConsent
        || h.review
            .as_ref()
            .is_none_or(|r| r.review_token_sha256 != token_hash)
    {
        return Err(err("experiment_consent_mismatch_or_consumed"));
    }
    let claim = Claim {
        experiment_id: id.into(),
        review_token_sha256: token_hash,
        accepted_at: now()?,
    };
    let dir = event_directory(root, id, false)?;
    write_once(&dir, "authorized.json", &claim)?;
    write_once(&dir, "running.json", &claim)
}
pub fn finish_independent_entry_experiment(
    root: &Path,
    terminal: &EntryExperimentTerminal,
) -> io::Result<()> {
    let h = require_current(root, &terminal.experiment_id)?;
    if h.terminal.is_some()
        || !matches!(
            terminal.status,
            EntryExperimentStatus::Succeeded
                | EntryExperimentStatus::Blocked
                | EntryExperimentStatus::Failed
        )
        || (terminal.status != EntryExperimentStatus::Blocked
            && h.status != EntryExperimentStatus::Running)
    {
        return Err(err("experiment_terminal_transition_invalid"));
    }
    write_once(
        &event_directory(root, &terminal.experiment_id, false)?,
        "terminal.json",
        terminal,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture {
        root: PathBuf,
        e: RealPrefixEntryExperiment,
    }
    impl Fixture {
        fn new() -> Self {
            let mut n = [0u8; 16];
            getrandom::fill(&mut n).unwrap();
            let id = n.iter().map(|b| format!("{b:02x}")).collect::<String>();
            let root = std::env::temp_dir().join(format!("lxmi-experiment-{id}"));
            fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
            fs::create_dir(root.join("tests")).unwrap();
            let e = RealPrefixEntryExperiment {
                schema: 1,
                experiment_id: id,
                session_nonce: "e".repeat(32),
                created_at: now().unwrap(),
                app_id: 4162040,
                source_baseline: BaselineReceipt {
                    manifest_path: root.join("snapshots/baseline.json"),
                    sha256: "a".repeat(64),
                },
                recovery_backup: CURRENT_ENTRY_BACKUP.into(),
                recovery_manifest_sha256: CURRENT_ENTRY_BACKUP_SHA.into(),
                executor_fingerprint: "b".repeat(64),
                prefix_identity: PrefixFileIdentity {
                    canonical_path: root.join("pfx"),
                    uid: 1000,
                    device: 1,
                    inode: 2,
                },
                runtime_build: ZZZ_OBSERVED_BUILD.into(),
            };
            Self { root, e }
        }
        fn ready(&self) {
            create_independent_entry_experiment(&self.root, &self.e).unwrap();
            review_independent_entry_experiment(
                &self.root,
                &EntryExperimentReview {
                    experiment_id: self.e.experiment_id.clone(),
                    reviewed_at: now().unwrap(),
                    review_token_sha256: entry_review_token_hash(&"c".repeat(32)).unwrap(),
                    source_baseline: self.e.source_baseline.clone(),
                },
            )
            .unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }
    #[test]
    fn interrupted_authorization_remains_consumed() {
        let f = Fixture::new();
        f.ready();
        write_once(
            &event_directory(&f.root, &f.e.experiment_id, false).unwrap(),
            "authorized.json",
            &Claim {
                experiment_id: f.e.experiment_id.clone(),
                review_token_sha256: entry_review_token_hash(&"c".repeat(32)).unwrap(),
                accepted_at: now().unwrap(),
            },
        )
        .unwrap();
        let h = read_independent_entry_experiment(&f.root).unwrap().unwrap();
        assert_eq!(h.status, EntryExperimentStatus::Authorized);
        assert!(h.consumed);
        assert!(
            consume_independent_entry_experiment(&f.root, &f.e.experiment_id, &"c".repeat(32))
                .is_err()
        );
    }
    #[test]
    fn concurrent_consent_can_claim_only_once() {
        let f = Fixture::new();
        f.ready();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let handles = (0..2)
            .map(|_| {
                let root = f.root.clone();
                let id = f.e.experiment_id.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    consume_independent_entry_experiment(&root, &id, &"c".repeat(32)).is_ok()
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(
            handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .filter(|claimed| *claimed)
                .count(),
            1
        );
        assert!(
            read_independent_entry_experiment(&f.root)
                .unwrap()
                .unwrap()
                .consumed
        );
    }
    #[test]
    fn one_new_slot_preserves_legacy_and_forbids_third_experiment() {
        let f = Fixture::new();
        let legacy = f.root.join("tests/qualified-real-entry-0_8_8_2-once.json");
        fs::write(&legacy, b"legacy immutable evidence").unwrap();
        assert_eq!(
            create_independent_entry_experiment(&f.root, &f.e)
                .unwrap()
                .status,
            EntryExperimentStatus::PendingReview
        );
        let mut another = f.e.clone();
        another.experiment_id = "d".repeat(32);
        assert!(create_independent_entry_experiment(&f.root, &another).is_err());
        assert_eq!(fs::read(legacy).unwrap(), b"legacy immutable evidence");
    }
    #[test]
    fn wrong_consent_and_wrong_experiment_cannot_consume_or_run() {
        let f = Fixture::new();
        f.ready();
        assert!(
            consume_independent_entry_experiment(&f.root, &f.e.experiment_id, &"d".repeat(32))
                .is_err()
        );
        assert!(
            consume_independent_entry_experiment(&f.root, &"d".repeat(32), &"c".repeat(32))
                .is_err()
        );
        assert!(
            !read_independent_entry_experiment(&f.root)
                .unwrap()
                .unwrap()
                .consumed
        );
    }
    #[test]
    fn restart_running_and_terminal_never_reset_or_authorize_again() {
        let f = Fixture::new();
        f.ready();
        consume_independent_entry_experiment(&f.root, &f.e.experiment_id, &"c".repeat(32)).unwrap();
        assert!(
            read_independent_entry_experiment(&f.root)
                .unwrap()
                .unwrap()
                .consumed
        );
        assert!(
            consume_independent_entry_experiment(&f.root, &f.e.experiment_id, &"c".repeat(32))
                .is_err()
        );
        finish_independent_entry_experiment(
            &f.root,
            &EntryExperimentTerminal {
                experiment_id: f.e.experiment_id.clone(),
                status: EntryExperimentStatus::Blocked,
                ended_at: now().unwrap(),
                reason: "preflight mismatch".into(),
                result_receipt: None,
            },
        )
        .unwrap();
        assert_eq!(
            read_independent_entry_experiment(&f.root)
                .unwrap()
                .unwrap()
                .status,
            EntryExperimentStatus::Blocked
        );
        assert!(
            consume_independent_entry_experiment(&f.root, &f.e.experiment_id, &"c".repeat(32))
                .is_err()
        );
    }
    #[test]
    fn review_failure_consumes_pending_without_helper_and_without_reset() {
        let f = Fixture::new();
        create_independent_entry_experiment(&f.root, &f.e).unwrap();
        let t = EntryExperimentTerminal {
            experiment_id: f.e.experiment_id.clone(),
            status: EntryExperimentStatus::Blocked,
            ended_at: now().unwrap(),
            reason: "runtime changed".into(),
            result_receipt: None,
        };
        finish_independent_entry_experiment(&f.root, &t).unwrap();
        assert!(finish_independent_entry_experiment(&f.root, &t).is_err());
        assert!(
            read_independent_entry_experiment(&f.root)
                .unwrap()
                .unwrap()
                .consumed
        );
    }
    #[test]
    fn unbound_recovery_nonce_and_path_traversal_rejected() {
        let f = Fixture::new();
        let mut e = f.e.clone();
        e.recovery_backup = "f".repeat(32);
        assert!(create_independent_entry_experiment(&f.root, &e).is_err());
        e = f.e.clone();
        e.session_nonce = e.experiment_id.clone();
        assert!(create_independent_entry_experiment(&f.root, &e).is_err());
        e = f.e.clone();
        e.experiment_id = "../escape".into();
        assert!(create_independent_entry_experiment(&f.root, &e).is_err());
        assert!(read_independent_entry_experiment(&f.root)
            .unwrap()
            .is_none());
    }
    #[test]
    fn symlink_event_and_unsafe_mode_fail_closed() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let f = Fixture::new();
        f.ready();
        let path = f
            .root
            .join("tests")
            .join(DIRECTORY)
            .join(&f.e.experiment_id)
            .join("authorized.json");
        symlink("/outside/not-read", &path).unwrap();
        assert!(read_independent_entry_experiment(&f.root).is_err());
        fs::remove_file(&path).unwrap();
        fs::write(&path, b"{}").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(read_independent_entry_experiment(&f.root).is_err());
    }
}
