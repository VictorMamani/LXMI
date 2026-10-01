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
