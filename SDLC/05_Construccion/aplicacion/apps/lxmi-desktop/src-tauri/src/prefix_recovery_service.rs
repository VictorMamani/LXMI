//! Fixed ZZZ source and fixed private destinations. No arbitrary path/PID API.
use crate::xxmi_service::{self, BridgeServiceError};
use lxmi_bridge::*;
use serde::{Deserialize, Serialize};
use std::{io, sync::Mutex};

static REVIEW: Mutex<Review> = Mutex::new(Review {
    preflight: None,
    recovery: None,
});
struct Review {
    preflight: Option<GamePrefixEntryPreflight>,
    recovery: Option<PrefixRecoveryQualification>,
}
#[derive(Serialize)]
pub struct PrefixRecoveryPanel {
    pub preflight: GamePrefixEntryPreflight,
    pub recovery: Option<PrefixRecoveryQualification>,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryAction {
    CreateBackup,
    VerifyBackup,
    TestRestore,
}
fn error(detail: impl ToString) -> BridgeServiceError {
    BridgeServiceError {
        code: "prefix_recovery_failed".into(),
        detail: detail.to_string(),
        stderr: None,
    }
}
fn inactive(compat: &std::path::Path) -> io::Result<()> {
    let evidence =
        lxmi_runtime::ProcfsObserver::for_current_user()?.inspect_prefix_activity(compat);
    if !evidence.observed_inactive() {
        return Err(io::Error::other("prefix_active_or_unresolved"));
    }
    Ok(())
}
pub fn inspect(capture: bool) -> Result<PrefixRecoveryPanel, BridgeServiceError> {
    let mut review = REVIEW.try_lock().map_err(|_| error("prefix_review_busy"))?;
    let mut preflight = xxmi_service::inspect_zzz_prefix_preflight_raw(capture)?;
    let root = xxmi_service::prefix_recovery_storage()?;
    if review.recovery.is_none() {
        if let Some(identity) = preflight.prefix_identity.as_ref() {
            review.recovery = latest_recovery_qualification(&root, identity).map_err(error)?;
        }
    }
    // Selected baseline is always fresh on reassessment after a verified recovery.
    if let Some(recovery) = review.recovery.as_mut() {
        if recovery.protected_paths_unchanged == Some(false) {
            preflight
                .blockers
                .push("recovery_protected_paths_changed".into());
            preflight.result = PrefixPreflightState::Blocked;
        }
        verify_prefix_recovery(&root, recovery).map_err(error)?;
        if recovery.restore_path.is_some() {
            verify_prefix_restore(&root, recovery).map_err(error)?;
        }
        if recovery.integrity.state == RecoveryVerificationState::Verified
            && recovery
                .restore_verification
                .as_ref()
                .is_some_and(|v| v.state == RecoveryVerificationState::Verified)
        {
            inactive(&preflight.compatdata).map_err(error)?;
            if recovery_source_matches(&root, recovery).map_err(error)? {
                if preflight.baseline_receipt.is_none() {
                    capture_game_prefix_baseline(&mut preflight, &root).map_err(error)?;
                }
                apply_prefix_recovery(&mut preflight, recovery);
            } else {
                preflight
                    .blockers
                    .push("recovery_source_current_mismatch".into());
            }
        }
    }
    review.preflight = Some(preflight.clone());
    Ok(PrefixRecoveryPanel {
        preflight,
        recovery: review.recovery.clone(),
    })
}
pub fn run(action: RecoveryAction) -> Result<PrefixRecoveryPanel, BridgeServiceError> {
    let mut review = REVIEW.try_lock().map_err(|_| error("prefix_review_busy"))?;
    let root = xxmi_service::prefix_recovery_storage()?;
    match action {
        RecoveryAction::CreateBackup => {
            if review.recovery.is_some() {
                return Err(error("backup_already_exists_in_review"));
            }
            let previous = review
                .preflight
                .as_ref()
                .ok_or_else(|| error("preflight_required"))?;
            let identity = previous
                .prefix_identity
                .as_ref()
                .ok_or_else(|| error("prefix_identity_unverified"))?;
            if previous.baseline_receipt.is_none() {
                return Err(error("capture_baseline_required"));
            }
            let fresh = xxmi_service::inspect_zzz_prefix_preflight_raw(false)?;
            if fresh.prefix_identity.as_ref() != Some(identity) {
                return Err(error("prefix_identity_changed"));
            }
            if fresh.runtime_drift != RuntimeBuildDrift::SameBuild
                || fresh.installed_build != previous.installed_build
            {
                return Err(error("runtime_changed"));
            }
            if fresh.blockers.iter().any(|s| {
                s == "snapshot_evidence_incomplete" || s == "unexpected_compatdata_relationship"
            }) {
                return Err(error("preflight_structure_or_protection_incomplete"));
            }
            inactive(&fresh.compatdata).map_err(error)?;
            let protected_before = fresh.baseline.protected_paths.clone();
            if protected_before != previous.baseline.protected_paths {
                return Err(error("protected_paths_changed_since_baseline"));
            }
            let build = fresh
                .installed_build
                .as_deref()
                .ok_or_else(|| error("build_unknown"))?;
            let mut result =
                create_prefix_recovery(&root, identity, build, || inactive(&fresh.compatdata))
                    .map_err(error)?;
            save_recovery_qualification(&root, &result).map_err(error)?;
            let after = xxmi_service::inspect_zzz_prefix_preflight_raw(false)?;
            result.protected_paths_unchanged =
                Some(protected_before == after.baseline.protected_paths);
            save_recovery_qualification(&root, &result).map_err(error)?;
            review.recovery = Some(result);
        }
        RecoveryAction::VerifyBackup => {
            let recovery = review
                .recovery
                .as_mut()
                .ok_or_else(|| error("backup_missing"))?;
            let result = verify_prefix_recovery(&root, recovery);
            save_recovery_qualification(&root, recovery).map_err(error)?;
            result.map_err(error)?;
        }
        RecoveryAction::TestRestore => {
            let recovery = review
                .recovery
                .as_mut()
                .ok_or_else(|| error("backup_missing"))?;
            test_prefix_restore(&root, recovery).map_err(error)?;
            save_recovery_qualification(&root, recovery).map_err(error)?;
        }
    }
    let preflight = review
        .preflight
        .clone()
        .ok_or_else(|| error("preflight_required"))?;
    Ok(PrefixRecoveryPanel {
        preflight,
        recovery: review.recovery.clone(),
    })
}

struct EntryReview {
    token: String,
    created: std::time::Instant,
    before: GamePrefixEntryPreflight,
    recovery: PrefixRecoveryQualification,
    runtime: ExplicitBridgeRuntime,
    steam_root: std::path::PathBuf,
    consumed: bool,
}
static ENTRY: Mutex<Option<EntryReview>> = Mutex::new(None);
#[derive(Serialize)]
pub struct HarmlessEntryReviewPanel {
    pub review_token: String,
    pub preflight: GamePrefixEntryPreflight,
    pub recovery: PrefixRecoveryQualification,
    pub runtime: ExplicitBridgeRuntime,
    pub consent_text: &'static str,
}
const CONSENT: &str = "LXMI will run a harmless Windows helper using the compatibility prefix associated with Zenless Zone Zero. Proton/Wine may update metadata inside that prefix. LXMI will not launch the game, load ZZMI, inject DLLs, install mods, or modify Steam launch options. Consent applies only to this experiment.";
pub fn prepare_entry() -> Result<HarmlessEntryReviewPanel, BridgeServiceError> {
    qualified_entry_environment()?;
    // Read-only for Steam/prefix; fresh baseline is stored only under LXMI.
    let panel = inspect(true)?;
    if panel.preflight.result != PrefixPreflightState::EligibleForReview {
        return Err(error(format!(
            "entry_preflight_not_eligible: {:?}",
            panel.preflight.blockers
        )));
    }
    let recovery = panel.recovery.ok_or_else(|| error("backup_invalid"))?;
    let (runtime, steam_root) = xxmi_service::harmless_entry_runtime()?;
    let mut state = ENTRY.try_lock().map_err(|_| error("entry_busy"))?;
    if state.as_ref().is_some_and(|r| r.consumed) {
        return Err(error("entry_already_attempted_no_retry"));
    }
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(error)?;
    let token = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    *state = Some(EntryReview {
        token: token.clone(),
        created: std::time::Instant::now(),
        before: panel.preflight.clone(),
        recovery: recovery.clone(),
        runtime: runtime.clone(),
        steam_root,
        consumed: false,
    });
    Ok(HarmlessEntryReviewPanel {
        review_token: token,
        preflight: panel.preflight,
        recovery,
        runtime,
        consent_text: CONSENT,
    })
}
pub fn run_entry(
    review_token: String,
    consent_accepted: bool,
) -> Result<RealPrefixEntryResult, BridgeServiceError> {
    if !consent_accepted {
        return Err(entry_error("consent_missing"));
    }
    qualified_entry_environment()?;
    let mut state = ENTRY.try_lock().map_err(|_| error("entry_busy"))?;
    let entry = state
        .as_mut()
        .ok_or_else(|| entry_error("entry_review_required"))?;
    if entry.consumed
        || entry.token != review_token
        || entry.created.elapsed() > std::time::Duration::from_secs(600)
    {
        return Err(entry_error("entry_review_expired_or_consumed"));
    }
    // Hold the shared recovery lock through execution: no concurrent backup/restore operation.
    let _review = REVIEW.try_lock().map_err(|_| error("prefix_review_busy"))?;
    entry.consumed = true;
    let root = xxmi_service::prefix_recovery_storage()?;
    verify_prefix_recovery(&root, &mut entry.recovery).map_err(error)?;
    verify_prefix_restore(&root, &mut entry.recovery).map_err(error)?;
    if !recovery_source_matches(&root, &entry.recovery).map_err(error)? {
        return Err(entry_error("backup_invalid"));
    }
    let mut before = xxmi_service::inspect_zzz_prefix_preflight_raw(true)?;
    if before.prefix_identity != entry.before.prefix_identity {
        return Err(entry_error("prefix_identity_changed"));
    }
    if before.baseline.protected_paths != entry.before.baseline.protected_paths {
        return Err(BridgeServiceError {
            code: "protected_path_changed".into(),
            detail: describe_protected_path_changes(&entry.before.baseline, &before.baseline),
            stderr: None,
        });
    }
    inactive(&before.compatdata).map_err(error)?;
    apply_prefix_recovery(&mut before, &entry.recovery);
    // Storage check before marker/environment staging.
    let storage_path =
        std::ffi::CString::new(root.as_os_str().as_encoded_bytes()).map_err(error)?;
    let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    if unsafe { libc::statvfs(storage_path.as_ptr(), stats.as_mut_ptr()) } != 0 {
        return Err(error(io::Error::last_os_error()));
    }
    let stats = unsafe { stats.assume_init() };
    if (stats.f_bavail as u128) * (stats.f_frsize as u128) < 256 * 1024 * 1024 {
        return Err(entry_error("insufficient_storage"));
    }
    let plan = validate_harmless_prefix_entry(
        &root,
        entry.runtime.clone(),
        entry.steam_root.clone(),
        &before,
        &entry.recovery,
        true,
        &entry.token,
    )
    .map_err(|s| entry_error(&format!("{s:?}")))?;
    let mut result=RealPrefixEntryResult{schema:1,session_nonce:entry.token.clone(),result:RealPrefixEntryState::Blocked,consent_accepted:true,helper_attempted:false,helper:None,helper_error:None,before,after:None,mutations:vec![],backup_id:entry.recovery.backup_id.clone(),backup_manifest_sha256:entry.recovery.manifest_sha256.clone(),backup_verified_after:false,prefix_identity_unchanged:false,protected_paths_unchanged:false,runtime_unchanged:false,game_context_inactive_after:false,evidence_receipt:None,warnings:vec!["Point-in-time inactivity checks are not an atomic prefix lock. Source paths are never restored automatically.".into(),"Snapshot coverage is selective; no whole-installation claim. Timeout kills only the owned launcher, never global wineserver; unresolved activity blocks the gate.".into()]};
    save_entry_evidence(&root, &result, "before")?;
    // Final same-UID activity and identity check after all costly integrity work.
    inactive(&result.before.compatdata).map_err(error)?;
    if prefix_file_identity(&result.before.compatdata.join("pfx")) != result.before.prefix_identity
    {
        return Err(entry_error("prefix_identity_changed"));
    }
    result.helper_attempted = true;
    match execute_harmless_prefix_entry(plan) {
        Ok(value) => result.helper = Some(value),
        Err(e) => result.helper_error = Some(e),
    }
    // No Wine cleanup command or signals. Allow natural metadata flushes to settle.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        if inactive(&result.before.compatdata).is_ok() {
            result.game_context_inactive_after = true;
            break;
        }
        if std::time::Instant::now() >= deadline {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    match xxmi_service::inspect_zzz_prefix_preflight_raw(false) {
        Ok(after) => {
            result.prefix_identity_unchanged =
                after.prefix_identity == result.before.prefix_identity;
            result.runtime_unchanged = after.runtime_drift == RuntimeBuildDrift::SameBuild
                && after.installed_build == result.before.installed_build;
            result.mutations = compare_prefix_entry(&result.before.baseline, &after.baseline);
            result.protected_paths_unchanged =
                after.baseline.protected_paths == result.before.baseline.protected_paths;
            result.after = Some(after);
        }
        Err(e) => result
            .warnings
            .push(format!("after_snapshot_unavailable: {}", e.detail)),
    }
    if verify_prefix_recovery(&root, &mut entry.recovery).is_ok()
        && entry.recovery.integrity.state == RecoveryVerificationState::Verified
    {
        result.backup_verified_after = true;
    }
    result.result = classify_prefix_entry(&result);
    result.evidence_receipt = Some(save_entry_evidence(&root, &result, "after")?);
    Ok(result)
}
fn qualified_entry_environment() -> Result<(), BridgeServiceError> {
    require_qualified_harmless_entry_environment().map_err(|_| BridgeServiceError {
        code: "linux_runtime_environment_unqualified".into(),
        detail: HARMLESS_ENTRY_ENVIRONMENT_BLOCKER.into(),
        stderr: None,
    })
}
fn save_entry_evidence(
    root: &std::path::Path,
    result: &RealPrefixEntryResult,
    phase: &str,
) -> Result<BaselineReceipt, BridgeServiceError> {
    use sha2::{Digest, Sha256};
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let directory = root.join("snapshots");
    let meta = std::fs::symlink_metadata(&directory).map_err(error)?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err(error("unsafe_snapshot_directory"));
    }
    let bytes = serde_json::to_vec_pretty(result).map_err(error)?;
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    let manifest_path = directory.join(format!(
        "real-prefix-entry-{}-{phase}.json",
        result.session_nonce
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(&manifest_path)
        .map_err(error)?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(error)?;
    Ok(BaselineReceipt {
        manifest_path,
        sha256,
    })
}

fn entry_error(code: &str) -> BridgeServiceError {
    BridgeServiceError {
        code: code.into(),
        detail:
            "Harmless entry blocked; no automatic retry or restore. Inspect the reviewed evidence."
                .into(),
        stderr: None,
    }
}

#[cfg(test)]
mod environment_guard_tests {
    use super::*;
    #[test]
    fn unqualified_environment_blocks_commands_before_host_inspection() {
        let error = match prepare_entry() {
            Err(e) => e,
            Ok(_) => panic!("unqualified real entry review must not pass"),
        };
        assert_eq!(error.code, "linux_runtime_environment_unqualified");
        let error = run_entry("ignored_fixture_token".into(), true).unwrap_err();
        assert_eq!(error.code, "linux_runtime_environment_unqualified");
        assert_eq!(
            run_entry("ignored_fixture_token".into(), false)
                .unwrap_err()
                .code,
            "consent_missing"
        );
    }
}
