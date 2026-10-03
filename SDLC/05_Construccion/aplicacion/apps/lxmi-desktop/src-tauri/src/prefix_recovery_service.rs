//! Fixed ZZZ source and fixed private destinations. No arbitrary path/PID API.
use crate::xxmi_service::{self, BridgeServiceError};
use lxmi_bridge::*;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::{io, sync::Mutex};

static REVIEW: Mutex<Review> = Mutex::new(Review {
    preflight: None,
    recovery: None,
    historical_recovery: None,
    reconciliation: None,
    accepted_protected: None,
    steam_review: None,
    steam_review_receipt: None,
});
struct Review {
    preflight: Option<GamePrefixEntryPreflight>,
    recovery: Option<PrefixRecoveryQualification>,
    historical_recovery: Option<PrefixRecoveryQualification>,
    reconciliation: Option<RecoverySourceDriftReport>,
    accepted_protected: Option<GamePrefixBaseline>,
    steam_review: Option<lxmi_steam::SteamProtectedStateReview>,
    steam_review_receipt: Option<BaselineReceipt>,
}
#[derive(Serialize)]
pub struct PrefixRecoveryPanel {
    pub preflight: GamePrefixEntryPreflight,
    pub recovery: Option<PrefixRecoveryQualification>,
    pub historical_recovery: Option<PrefixRecoveryQualification>,
    pub reconciliation: Option<RecoverySourceDriftReport>,
    pub steam_review: Option<lxmi_steam::SteamProtectedStateReview>,
    pub steam_review_receipt: Option<BaselineReceipt>,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryAction {
    CreateBackup,
    VerifyBackup,
    TestRestore,
    CaptureAcceptedBaseline,
    CreateCurrentBackup,
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

const OLD_BACKUP_ID: &str = "f3551122468ed68b825a0182922f3be2";
const OLD_BACKUP_SHA: &str = "db1d884b02ede5435f64520b92fcddf9e4688d56701c1329536d9abe38fa155f";
const BASELINE_FILE: &str = "zzz-prefix-594c3adb21910164a8729f44b49753ff.json";
const BASELINE_SHA: &str = "00eb5975b17546006154e6fd6cb6f696a47a0be3b84e37f389e988a89bdbdea2";
const ATTEMPT_BEFORE: &str = "real-prefix-entry-699f4768f21d30842653eb404baeb237-before.json";
const ATTEMPT_BEFORE_SHA: &str = "6c44ae8fd97b8ef55348a0b789b48fb1138a8d7b279fe6d6be1fa66d497b3926";
const ATTEMPT_AFTER: &str = "real-prefix-entry-699f4768f21d30842653eb404baeb237-after.json";
const ATTEMPT_AFTER_SHA: &str = "1f2b3d6e7bdaecdc15f14a40a1ebfd3ddae62a387e493f6cfa1ce802fc9da246";
const PROTON_SOURCE_SHA: &str = "9d0bb73e241bac979afeda526b589fce6f26ee2d5e8822b7aadc710b179421bb";

fn source_identity_expected(identity: &PrefixFileIdentity) -> bool {
    identity.uid == 1000
        && identity.device == 66306
        && identity.inode == 11538172
        && identity
            .canonical_path
            .ends_with("steamapps/compatdata/4162040/pfx")
}
fn proton_source_fingerprint() -> Result<String, BridgeServiceError> {
    let (runtime, _) = xxmi_service::harmless_entry_runtime()?;
    let root = runtime
        .proton_script
        .parent()
        .ok_or_else(|| error("proton_root_missing"))?;
    current_recovery_tree_fingerprint(root).map_err(error)
}
fn receipt(root: &std::path::Path, name: &str, sha: &str) -> BaselineReceipt {
    BaselineReceipt {
        manifest_path: root.join("snapshots").join(name),
        sha256: sha.into(),
    }
}
fn attempt_baseline(bytes: &[u8], after: bool) -> Result<GamePrefixBaseline, BridgeServiceError> {
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(error)?;
    serde_json::from_value(value[if after { "after" } else { "before" }]["baseline"].clone())
        .map_err(error)
}

fn steam_review_valid(
    review: &Review,
    baseline: &GamePrefixBaseline,
) -> Result<bool, BridgeServiceError> {
    let Some(proof) = review.steam_review.as_ref() else {
        return Ok(false);
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(error)?
        .as_secs();
    Ok(now >= proof.after.captured_at
        && now - proof.after.captured_at <= 600
        && baseline.steam_semantic_state.as_ref().is_some_and(|s| {
            reviewed_steam_state_matches(proof, s)
                && steam_semantics_bind_protected_paths(s, &baseline.protected_paths)
        }))
}
pub fn review_steam_protected() -> Result<PrefixRecoveryPanel, BridgeServiceError> {
    {
        let mut review = REVIEW.try_lock().map_err(|_| error("prefix_review_busy"))?;
        review.steam_review = None;
        review.steam_review_receipt = None;
        review.reconciliation = None;
        review.accepted_protected = None;
        let before = xxmi_service::inspect_zzz_prefix_preflight_raw(false)?;
        inactive(&before.compatdata).map_err(error)?;
        if !before
            .prefix_identity
            .as_ref()
            .is_some_and(source_identity_expected)
            || before.runtime_drift != RuntimeBuildDrift::SameBuild
        {
            return Err(error("prefix_identity_or_runtime_changed"));
        }
        let (_, steam) = xxmi_service::harmless_entry_runtime()?;
        let proof = lxmi_steam::review_zzz_steam_state(&steam, 5)
            .map_err(|_| error("steam_protected_unparseable_or_active_write"))?;
        let after = xxmi_service::inspect_zzz_prefix_preflight_raw(false)?;
        inactive(&after.compatdata).map_err(error)?;
        if before.prefix_identity != after.prefix_identity
            || after.runtime_drift != RuntimeBuildDrift::SameBuild
        {
            return Err(error("prefix_identity_or_runtime_changed"));
        }
        if after
            .baseline
            .steam_semantic_state
            .as_ref()
            .is_none_or(|s| !reviewed_steam_state_matches(&proof, s))
            && proof.assessment == lxmi_steam::SteamProtectedStateAssessment::ReviewedAndAcceptable
        {
            return Err(error("steam_semantics_changed_after_observation"));
        }
        review.steam_review_receipt = Some(
            save_steam_protected_review(&xxmi_service::prefix_recovery_storage()?, &proof)
                .map_err(error)?,
        );
        review.steam_review = Some(proof);
    }
    reconcile()
}

pub fn reconcile() -> Result<PrefixRecoveryPanel, BridgeServiceError> {
    let mut review = REVIEW.try_lock().map_err(|_| error("prefix_review_busy"))?;
    // Invalidate previous refresh capability before collecting any new evidence.
    review.reconciliation = None;
    review.accepted_protected = None;
    let root = xxmi_service::prefix_recovery_storage()?;
    let before = xxmi_service::inspect_zzz_prefix_preflight_raw(false)?;
    inactive(&before.compatdata).map_err(error)?;
    let identity = before
        .prefix_identity
        .as_ref()
        .ok_or_else(|| error("prefix_identity_unverified"))?;
    if !source_identity_expected(identity) {
        return Err(error("prefix_identity_changed"));
    }
    if before.runtime_drift != RuntimeBuildDrift::SameBuild {
        return Err(error("runtime_changed"));
    }
    let mut historical = load_recovery_generation(&root, OLD_BACKUP_ID, identity).map_err(error)?;
    if historical.manifest_sha256 != OLD_BACKUP_SHA {
        return Err(error("old_backup_manifest_identity_changed"));
    }
    verify_prefix_recovery(&root, &mut historical).map_err(error)?;
    if historical.restore_path.is_some() {
        verify_prefix_restore(&root, &mut historical).map_err(error)?;
    }
    let baseline_bytes =
        read_prefix_recovery_evidence(&root, BASELINE_FILE, BASELINE_SHA).map_err(error)?;
    let baseline: GamePrefixBaseline = serde_json::from_slice(&baseline_bytes).map_err(error)?;
    if baseline.prefix_identity.as_ref() != Some(identity) {
        return Err(error("historical_baseline_identity_mismatch"));
    }
    let a =
        read_prefix_recovery_evidence(&root, ATTEMPT_BEFORE, ATTEMPT_BEFORE_SHA).map_err(error)?;
    let b =
        read_prefix_recovery_evidence(&root, ATTEMPT_AFTER, ATTEMPT_AFTER_SHA).map_err(error)?;
    let attempt_before = attempt_baseline(&a, false)?;
    let attempt_after = attempt_baseline(&b, true)?;
    if attempt_before.prefix_identity.as_ref() != Some(identity)
        || attempt_after.prefix_identity.as_ref() != Some(identity)
    {
        return Err(error("historical_attempt_identity_mismatch"));
    }
    let (mut entries, fingerprint, consistent) =
        inspect_recovery_source_drift(&root, &mut historical, || inactive(&before.compatdata))
            .map_err(error)?;
    let mut registry = Vec::new();
    for name in ["system.reg", "user.reg", "userdef.reg"] {
        let (old, current) =
            read_reconciliation_registry(&root, &historical, name).map_err(error)?;
        let old_sha = format!("{:x}", sha2::Sha256::digest(&old));
        let current_sha = format!("{:x}", sha2::Sha256::digest(&current));
        let matching = |baseline: &GamePrefixBaseline, hash: &str| {
            baseline
                .selected_prefix_paths
                .iter()
                .find(|p| p.path == identity.canonical_path.join(name))
                .and_then(|p| p.sha256.as_deref())
                == Some(hash)
        };
        let historical_before = matching(&attempt_before, &old_sha);
        let historical_after = matching(&attempt_after, &current_sha);
        let sections = diagnose_fixture_registry_sections(&old, &current)
            .map_err(|e| error(format!("registry_diagnosis: {e:?}")))?;
        let diagnosis = diagnose_registry_change(&old, &current, &old_sha, &current_sha)
            .map_err(|e| error(format!("registry_diagnosis: {e:?}")))?;
        let classification =
            classify_reconciled_registry(&sections, historical_before && historical_after);
        if let Some(entry) = entries.iter_mut().find(|e| e.relative_path == name) {
            let hash_matches = entry.old.as_ref().and_then(|e| e.sha256.as_deref())
                == Some(old_sha.as_str())
                && entry.current.as_ref().and_then(|e| e.sha256.as_deref())
                    == Some(current_sha.as_str());
            let metadata_compatible = matches!((&entry.old,&entry.current),(Some(x),Some(y)) if x.mode==y.mode && x.uid==y.uid && x.gid==y.gid && x.xattr_hashes==y.xattr_hashes);
            if entry.change == RecoveryDriftKind::Modified && hash_matches && metadata_compatible {
                entry.classification = classification;
                entry.evidence.push(format!("Registry semantic comparison; exact historical before/after hashes: {historical_before}/{historical_after}. Fonts stay historical environment-specific, never normalized."));
            }
        }
        registry.push(RecoveryRegistryDrift{relative_path:name.into(),sections,diagnosis,historical_before_hash_matches:historical_before,historical_after_hash_matches:historical_after,classification,evidence:vec!["Hashes correlate full hive contents with recorded prior attempt, not merely similar paths. Component causality remains DERIVED from pinned upstream + private controlled matrix; no new source mutation.".into()]});
    }
    let proton_fingerprint = proton_source_fingerprint()?;
    let after = xxmi_service::inspect_zzz_prefix_preflight_raw(false)?;
    inactive(&after.compatdata).map_err(error)?;
    let mut protected = compare_recovery_protected_state(
        &baseline.protected_paths,
        &after.baseline.protected_paths,
    );
    if before.baseline.protected_paths != after.baseline.protected_paths {
        protected.extend(compare_recovery_protected_state(
            &before.baseline.protected_paths,
            &after.baseline.protected_paths,
        ));
    }
    let steam_accepted = steam_review_valid(&review, &after.baseline)?;
    let mut protected_unresolved = false;
    for drift in &mut protected {
        let reviewed_path = after
            .baseline
            .steam_semantic_state
            .as_ref()
            .is_some_and(|state| {
                drift.path == state.config_raw.path || drift.path == state.localconfig_raw.path
            });
        if steam_accepted && reviewed_path {
            drift.classification = RecoveryDriftClassification::CurrentSteamStateReviewed;
            drift.evidence = "Historical raw difference preserved; exact past semantic diff/writer UNKNOWN. Current selected Steam state independently reviewed and stable; no launch overrides or explicit game/global runtime mapping. This is not attribution or historical semantic equality.".into();
        } else {
            protected_unresolved = true;
        }
    }
    let consistent = consistent
        && current_recovery_tree_fingerprint(&identity.canonical_path).map_err(error)?
            == fingerprint;
    let assessment = assess_current_prefix(
        true,
        after.prefix_identity.as_ref() == Some(identity),
        historical.integrity.state == RecoveryVerificationState::Verified,
        consistent,
        after.runtime_drift == RuntimeBuildDrift::SameBuild,
        !protected_unresolved && steam_accepted && proton_fingerprint == PROTON_SOURCE_SHA,
        &entries,
    );
    let mut blockers = Vec::new();
    if !consistent {
        blockers.push("source_changed_during_reconciliation".into());
    }
    if protected_unresolved {
        blockers.push("protected_state_needs_review".into());
    }
    if !steam_accepted {
        blockers.push("steam_protected_semantic_review_required".into());
    }
    if proton_fingerprint != PROTON_SOURCE_SHA {
        blockers.push("approved_proton_source_changed".into());
    }
    if entries.iter().any(|e| {
        matches!(
            e.classification,
            RecoveryDriftClassification::Unknown | RecoveryDriftClassification::Unexpected
        )
    }) {
        blockers.push("source_material_drift_unknown_or_unexpected".into());
    }
    let mut report=RecoverySourceDriftReport{schema:1,timestamp_unix_seconds:std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(error)?.as_secs(),old_backup_id:historical.backup_id.clone(),old_backup_manifest_sha256:historical.manifest_sha256.clone(),old_backup_integrity:historical.integrity.clone(),source_identity:identity.clone(),current_tree_sha256:fingerprint,source_consistent_during_comparison:consistent,summary:recovery_drift_summary(&entries),entries,registry,protected_drift:protected,steam_semantic_review:review.steam_review.clone(),historical_baseline:receipt(&root,BASELINE_FILE,BASELINE_SHA),historical_attempt_evidence:vec![receipt(&root,ATTEMPT_BEFORE,ATTEMPT_BEFORE_SHA),receipt(&root,ATTEMPT_AFTER,ATTEMPT_AFTER_SHA)],proton_source_matches_qualified_source:proton_fingerprint==PROTON_SOURCE_SHA,proton_source_manifest_sha256:proton_fingerprint,assessment,blockers,evidence_receipt:None,limitations:vec!["Read-only source; full recovery inventory, no symlink dereference. No raw registry/config values persisted.".into(),"Acceptance permits only a new current recovery generation, never consent/helper/rollback or blanket acceptance of future mutations.".into(),"Same-UID activity observations and sequential scans are not an atomic source lock. Protected coverage is the selective 0.8.7 policy; no whole-game/Steam claim.".into()]};
    report.evidence_receipt = Some(save_recovery_source_drift(&root, &report).map_err(error)?);
    review.accepted_protected = Some(after.baseline.clone());
    review.reconciliation = Some(report);
    review.historical_recovery = Some(historical.clone());
    if review.recovery.is_none() {
        review.recovery = latest_recovery_qualification(&root, identity)
            .map_err(error)?
            .or(Some(historical));
    }
    review.preflight = Some(after.clone());
    if assessment != CurrentPrefixStateAssessment::UnderstoodAndAcceptable {
        let p = review.preflight.as_mut().unwrap();
        p.result = PrefixPreflightState::NeedsEvidence;
        p.blockers
            .push("recovery_source_reconciliation_not_accepted".into());
    }
    Ok(PrefixRecoveryPanel {
        preflight: review.preflight.clone().unwrap(),
        recovery: review.recovery.clone(),
        historical_recovery: review.historical_recovery.clone(),
        reconciliation: review.reconciliation.clone(),
        steam_review: review.steam_review.clone(),
        steam_review_receipt: review.steam_review_receipt.clone(),
    })
}

fn validate_reconciliation_fresh(
    review: &Review,
) -> Result<GamePrefixEntryPreflight, BridgeServiceError> {
    let report = review
        .reconciliation
        .as_ref()
        .ok_or_else(|| error("source_reconciliation_required"))?;
    if report.assessment != CurrentPrefixStateAssessment::UnderstoodAndAcceptable {
        return Err(error("source_reconciliation_not_accepted"));
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(error)?
        .as_secs();
    if now.saturating_sub(report.timestamp_unix_seconds) > 600 {
        return Err(error("source_reconciliation_expired"));
    }
    let root = xxmi_service::prefix_recovery_storage()?;
    let old = review
        .historical_recovery
        .as_ref()
        .ok_or_else(|| error("historical_recovery_required"))?;
    let mut old = old.clone();
    verify_prefix_recovery(&root, &mut old).map_err(error)?;
    let current = xxmi_service::inspect_zzz_prefix_preflight_raw(false)?;
    inactive(&current.compatdata).map_err(error)?;
    if current.prefix_identity.as_ref() != Some(&report.source_identity)
        || current.runtime_drift != RuntimeBuildDrift::SameBuild
    {
        return Err(error("source_identity_or_runtime_changed"));
    }
    if review
        .accepted_protected
        .as_ref()
        .is_none_or(|b| !protected_baselines_match(b, &current.baseline))
        || !steam_review_valid(review, &current.baseline)?
        || proton_source_fingerprint()? != report.proton_source_manifest_sha256
    {
        return Err(error("accepted_protected_state_changed"));
    }
    if current_recovery_tree_fingerprint(&report.source_identity.canonical_path).map_err(error)?
        != report.current_tree_sha256
    {
        return Err(error("accepted_source_state_changed"));
    }
    inactive(&current.compatdata).map_err(error)?;
    Ok(current)
}
pub fn inspect(capture: bool) -> Result<PrefixRecoveryPanel, BridgeServiceError> {
    let mut review = REVIEW.try_lock().map_err(|_| error("prefix_review_busy"))?;
    let mut preflight = xxmi_service::inspect_zzz_prefix_preflight_raw(false)?;
    let root = xxmi_service::prefix_recovery_storage()?;
    if review.recovery.is_none() {
        if let Some(identity) = preflight.prefix_identity.as_ref() {
            review.recovery = latest_recovery_qualification(&root, identity).map_err(error)?;
        }
    }
    // Reuse a captured baseline only when its complete selected evidence still matches.
    // Read-only queries never create a new baseline.
    if let Some(previous) = review.preflight.as_ref() {
        if previous.prefix_identity == preflight.prefix_identity
            && previous.baseline.selected_prefix_paths == preflight.baseline.selected_prefix_paths
            && protected_baselines_match(&previous.baseline, &preflight.baseline)
            && previous.baseline_receipt.is_some()
        {
            preflight.baseline_receipt = previous.baseline_receipt.clone();
            preflight.blockers.retain(|b| b != "baseline_not_captured");
        }
    }
    if capture && !steam_review_valid(&review, &preflight.baseline)? {
        return Err(error("steam_protected_semantic_review_required"));
    }
    let reconciliation_accepted = review
        .reconciliation
        .as_ref()
        .is_none_or(|r| r.assessment == CurrentPrefixStateAssessment::UnderstoodAndAcceptable);
    if capture && !reconciliation_accepted {
        return Err(error("source_reconciliation_not_accepted"));
    }
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
                if capture {
                    capture_game_prefix_baseline(&mut preflight, &root).map_err(error)?;
                }
                apply_prefix_recovery(&mut preflight, recovery);
            } else {
                if capture {
                    return Err(error(
                        "use_capture_accepted_baseline_after_source_reconciliation",
                    ));
                }
                preflight
                    .blockers
                    .push("recovery_source_current_mismatch".into());
            }
        }
    }
    if let Some(report) = review.reconciliation.as_ref() {
        if report.assessment != CurrentPrefixStateAssessment::UnderstoodAndAcceptable {
            preflight.result = PrefixPreflightState::NeedsEvidence;
            preflight
                .blockers
                .push("recovery_source_reconciliation_not_accepted".into());
            if capture {
                return Err(error("source_reconciliation_not_accepted"));
            }
        }
    }
    if capture && preflight.baseline_receipt.is_none() {
        inactive(&preflight.compatdata).map_err(error)?;
        capture_game_prefix_baseline(&mut preflight, &root).map_err(error)?;
    }
    if !steam_review_valid(&review, &preflight.baseline)? {
        preflight.result = PrefixPreflightState::NeedsEvidence;
        preflight
            .blockers
            .push("steam_protected_semantic_review_required".into());
    }
    review.preflight = Some(preflight.clone());
    Ok(PrefixRecoveryPanel {
        preflight,
        recovery: review.recovery.clone(),
        historical_recovery: review.historical_recovery.clone(),
        reconciliation: review.reconciliation.clone(),
        steam_review: review.steam_review.clone(),
        steam_review_receipt: review.steam_review_receipt.clone(),
    })
}
pub fn run(action: RecoveryAction) -> Result<PrefixRecoveryPanel, BridgeServiceError> {
    let mut review = REVIEW.try_lock().map_err(|_| error("prefix_review_busy"))?;
    let root = xxmi_service::prefix_recovery_storage()?;
    match action {
        RecoveryAction::CaptureAcceptedBaseline => {
            let mut fresh = validate_reconciliation_fresh(&review)?;
            capture_game_prefix_baseline(&mut fresh, &root).map_err(error)?;
            review.preflight = Some(fresh);
        }
        RecoveryAction::CreateCurrentBackup => {
            let fresh = validate_reconciliation_fresh(&review)?;
            let previous = review
                .preflight
                .as_ref()
                .ok_or_else(|| error("capture_accepted_baseline_required"))?;
            if previous.baseline_receipt.is_none()
                || !protected_baselines_match(&previous.baseline, &fresh.baseline)
                || previous.baseline.selected_prefix_paths != fresh.baseline.selected_prefix_paths
            {
                return Err(error("capture_accepted_baseline_required_or_changed"));
            }
            if review
                .recovery
                .as_ref()
                .is_some_and(|r| r.backup_id != OLD_BACKUP_ID)
            {
                return Err(error("current_generation_already_created"));
            }
            let identity = fresh
                .prefix_identity
                .as_ref()
                .ok_or_else(|| error("prefix_identity_unverified"))?;
            let mut backup = create_prefix_recovery(&root, identity, ZZZ_OBSERVED_BUILD, || {
                inactive(&fresh.compatdata)
            })
            .map_err(error)?;
            let after = xxmi_service::inspect_zzz_prefix_preflight_raw(false)?;
            backup.protected_paths_unchanged = Some(
                protected_baselines_match(&fresh.baseline, &after.baseline)
                    && steam_review_valid(&review, &after.baseline)?,
            );
            save_recovery_qualification(&root, &backup).map_err(error)?;
            review.recovery = Some(backup);
        }
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
        historical_recovery: review.historical_recovery.clone(),
        reconciliation: review.reconciliation.clone(),
        steam_review: review.steam_review.clone(),
        steam_review_receipt: review.steam_review_receipt.clone(),
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
    if !protected_baselines_match(&before.baseline, &entry.before.baseline)
        || !steam_review_valid(&_review, &before.baseline)?
    {
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
        if !protected_baselines_match(&current.baseline, &result.before.baseline)
            || !steam_review_valid(&_review, &current.baseline)
                .map_err(|_| fail("steam semantic review invalid"))?
        {
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
