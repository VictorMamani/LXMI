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
    steam_root: std::path::PathBuf,
    consumed: bool,
}
static ENTRY: Mutex<Option<EntryReview>> = Mutex::new(None);
static QUALIFIED_EXECUTOR: Mutex<Option<QualifiedLinuxRuntimeRecipe>> = Mutex::new(None);
pub fn qualify_executor() -> Result<QualifiedExecutorPanel, BridgeServiceError> {
    let mut state = QUALIFIED_EXECUTOR
        .try_lock()
        .map_err(|_| error("executor_busy"))?;
    *state = None;
    let root = xxmi_service::prefix_recovery_storage()?;
    let (_, steam) = xxmi_service::harmless_entry_runtime()?;
    let recipe = qualify_integrated_executor(&root, &steam).map_err(|e| BridgeServiceError {
        code: "linux_runtime_environment_unqualified".into(),
        detail: e.to_string(),
        stderr: None,
    })?;
    let panel = recipe.panel();
    *state = Some(recipe);
    Ok(panel)
}
pub fn environment_panel() -> Result<LinuxEnvironmentQualificationPanel, BridgeServiceError> {
    let mut panel = linux_environment_qualification_panel().map_err(error)?;
    panel.current_environment.runtime_kind = LinuxRuntimeKind::SteamLinuxRuntimeContainer;
    panel.current_environment.request_transport = BridgeRequestTransport::PrivateRequestFile;
    panel.current_environment.provenance=vec!["Integrated container executor requires successful current-session private self-test; no direct fallback.".into()];
    panel.limitations=vec!["Prior DirectProton UnexpectedMutation is preserved; real prefix restore remains NotTested.".into(),"Historical fixture qualification alone does not authorize this executor or consent.".into()];
    let state = QUALIFIED_EXECUTOR
        .try_lock()
        .map_err(|_| error("executor_busy"))?;
    if let Some(recipe) = state.as_ref() {
        recipe.require_qualified().map_err(error)?;
        panel.executor = Some(recipe.panel());
        panel.current_environment.runtime_kind = LinuxRuntimeKind::SteamLinuxRuntimeContainer;
        panel.current_environment.qualification = EnvironmentQualification::SteamRuntimeQualified;
        panel.current_environment.request_transport = BridgeRequestTransport::PrivateRequestFile;
        panel.real_prefix_entry = "executor_qualified_preflight_required".into();
        panel.blocker = "fresh_preflight_and_consent_required".into();
        panel.limitations = recipe.panel().limitations;
        panel.current_environment.provenance = vec!["Integrated executor private self-test passed in this native session; real entry not run.".into()];
    }
    Ok(panel)
}

#[derive(Serialize)]
pub struct HarmlessEntryReviewPanel {
    pub review_token: String,
    pub preflight: GamePrefixEntryPreflight,
    pub recovery: PrefixRecoveryQualification,
    pub runtime: ExplicitBridgeRuntime,
    pub consent_text: &'static str,
}
const CONSENT: &str = "LXMI will run ONE harmless Windows helper using ZZZ's real compatibility prefix through the qualified Steam Runtime container and a verified private Proton copy. Proton/Wine may update prefix metadata. ZZZ must remain closed. LXMI will not launch the game, load ZZMI, inject DLLs, install mods, or modify Steam launch options. Recovery backup exists; real-prefix restore is NOT TESTED. This fresh consent applies only to this reviewed experiment and cannot be reused.";
pub fn prepare_entry() -> Result<HarmlessEntryReviewPanel, BridgeServiceError> {
    qualified_entry_environment()?;
    let storage = xxmi_service::prefix_recovery_storage()?;
    if storage
        .join("tests/qualified-real-entry-0_8_8_2-once.json")
        .exists()
    {
        return Err(entry_error("entry_already_attempted_no_retry"));
    }
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
fn fresh_review(
    consumed: bool,
    elapsed: std::time::Duration,
    expected: &str,
    provided: &str,
) -> bool {
    !consumed && elapsed <= std::time::Duration::from_secs(600) && expected == provided
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
    if !fresh_review(
        entry.consumed,
        entry.created.elapsed(),
        &entry.token,
        &review_token,
    ) {
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
    let recipe = qualified_entry_environment()?;
    let plan = validate_harmless_prefix_entry(
        &root,
        &recipe,
        entry.steam_root.clone(),
        &before,
        &entry.recovery,
        true,
        &entry.token,
    )
    .map_err(|s| entry_error(&format!("{s:?}")))?;
    let mut result=RealPrefixEntryResult{schema:1,session_nonce:entry.token.clone(),result:RealPrefixEntryState::Blocked,consent_accepted:true,helper_attempted:false,executor_qualified:true,game_became_active:false,executor_recipe_fingerprint:Some(recipe.panel().fingerprint),helper:None,helper_error:None,before,after:None,mutations:vec![],backup_id:entry.recovery.backup_id.clone(),backup_manifest_sha256:entry.recovery.manifest_sha256.clone(),backup_verified_after:false,prefix_identity_unchanged:false,protected_paths_unchanged:false,runtime_unchanged:false,game_context_inactive_after:false,evidence_receipt:None,warnings:vec!["Point-in-time inactivity checks are not an atomic prefix lock. Source paths are never restored automatically.".into(),"Snapshot coverage is selective; no whole-installation claim. Timeout kills only the owned launcher, never global wineserver; unresolved activity blocks the gate.".into()]};
    save_entry_evidence(&root, &result, "before")?;
    // Final same-UID activity and identity check after all costly integrity work.
    inactive(&result.before.compatdata).map_err(error)?;
    if prefix_file_identity(&result.before.compatdata.join("pfx")) != result.before.prefix_identity
    {
        return Err(entry_error("prefix_identity_changed"));
    }
    let before_registry =
        qualified_registry_capture(&result.before.compatdata.join("pfx")).map_err(error)?;
    let before_metadata =
        qualified_prefix_metadata(&result.before.compatdata.join("pfx")).map_err(error)?;
    let game_seen = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (seen, done, compat) = (
        game_seen.clone(),
        stop.clone(),
        result.before.compatdata.clone(),
    );
    let watcher = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
        while !done.load(std::sync::atomic::Ordering::SeqCst)
            && std::time::Instant::now() < deadline
        {
            if lxmi_runtime::ProcfsObserver::for_current_user()
                .map(|o| o.inspect_prefix_activity(&compat).game_name_matches > 0)
                .unwrap_or(true)
            {
                seen.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
    let final_check = || -> Result<(), lxmi_bridge::BridgeError> {
        let fail = |detail: &str| lxmi_bridge::BridgeError {
            code: lxmi_bridge::BridgeErrorCode::UnsafePrefixContext,
            detail: detail.into(),
            stderr: None,
        };
        if entry.created.elapsed() > std::time::Duration::from_secs(600) {
            return Err(fail("consent expired before invocation"));
        }
        let current =
            xxmi_service::inspect_zzz_prefix_preflight_raw(false).map_err(|e| fail(&e.detail))?;
        if current.prefix_identity != result.before.prefix_identity
            || current.runtime_drift != RuntimeBuildDrift::SameBuild
        {
            return Err(fail("prefix identity/runtime changed before invocation"));
        }
        if current.baseline.protected_paths != result.before.baseline.protected_paths {
            return Err(fail("protected paths changed before invocation"));
        }
        if game_seen.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(fail("GameBecameActive"));
        }
        inactive(&result.before.compatdata).map_err(|_| fail("ActiveContext before invocation"))?;
        use std::os::unix::fs::OpenOptionsExt;
        let bytes=serde_json::to_vec(&serde_json::json!({"schema":1,"review_nonce":entry.token,"recipe":recipe.panel().fingerprint,"real_restore":"not_tested"})).map_err(|e|fail(&e.to_string()))?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(root.join("tests/qualified-real-entry-0_8_8_2-once.json"))
            .map_err(|e| fail(&format!("one-shot reservation failed: {e}")))?;
        std::io::Write::write_all(&mut file, &bytes)
            .and_then(|()| file.sync_all())
            .map_err(|e| fail(&e.to_string()))?;
        Ok(())
    };
    let execution = execute_harmless_prefix_entry(plan, &final_check);
    stop.store(true, std::sync::atomic::Ordering::SeqCst);
    if watcher.join().is_err() {
        game_seen.store(true, std::sync::atomic::Ordering::SeqCst);
    }
    result.game_became_active = game_seen.load(std::sync::atomic::Ordering::SeqCst);
    result.helper_attempted = true;
    match execution {
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
            match qualified_registry_capture(&result.before.compatdata.join("pfx"))
                .and_then(|regs| qualified_registry_diff(&before_registry, &regs))
                .and_then(|sections| {
                    qualified_prefix_metadata(&result.before.compatdata.join("pfx"))
                        .map(|metadata| (sections, metadata))
                }) {
                Ok((sections, metadata)) => {
                    result.mutations = compare_qualified_prefix_entry(
                        &result.before.baseline,
                        &after.baseline,
                        &sections,
                        qualified_prefix_metadata_matches(&before_metadata, &metadata),
                    )
                }
                Err(e) => result.mutations.push(PrefixMutation {
                    path: result.before.compatdata.clone(),
                    kind: PrefixMutationKind::Unknown,
                    reason: format!("Qualified comparison unavailable: {e}"),
                }),
            }
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
fn qualified_entry_environment() -> Result<QualifiedLinuxRuntimeRecipe, BridgeServiceError> {
    let state = QUALIFIED_EXECUTOR
        .try_lock()
        .map_err(|_| error("executor_busy"))?;
    let recipe=state.as_ref().ok_or_else(|| BridgeServiceError {code:"linux_runtime_environment_unqualified".into(),detail:"Integrated container executor has no successful private self-test in this session. Real entry remains blocked; no direct fallback.".into(),stderr:None})?;
    recipe.require_qualified().map_err(|e| BridgeServiceError {
        code: "linux_runtime_environment_unqualified".into(),
        detail: e.to_string(),
        stderr: None,
    })?;
    Ok(recipe.clone())
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
    #[test]
    fn consent_review_is_fresh_single_use_and_session_bound() {
        let now = std::time::Duration::from_secs(0);
        assert!(fresh_review(false, now, "new", "new"));
        assert!(!fresh_review(true, now, "new", "new"));
        assert!(!fresh_review(false, now, "new", "old"));
        assert!(!fresh_review(
            false,
            std::time::Duration::from_secs(601),
            "new",
            "new"
        ));
    }
}
