//! Read-only reconciliation. Assessment authorizes recovery capture, never execution.
use super::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryDriftKind {
    Added,
    Removed,
    Modified,
    TypeChanged,
    SymlinkTargetChanged,
    MetadataOnlyChanged,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryDriftClassification {
    KnownWineProtonMutation,
    KnownGameRuntimeMutation,
    KnownSteamMutation,
    CurrentSteamStateReviewed,
    PreviouslyQualifiedVolatile,
    MetadataOnly,
    Unexpected,
    Unknown,
    ProtectedPathRelevant,
}
#[derive(Debug, Clone, Serialize)]
pub struct RecoverySourceDriftEntry {
    pub relative_path: String,
    pub old: Option<RecoveryEntry>,
    pub current: Option<RecoveryEntry>,
    pub change: RecoveryDriftKind,
    pub classification: RecoveryDriftClassification,
    pub evidence: Vec<String>,
}
#[derive(Debug, Clone, Default, Serialize)]
pub struct RecoveryDriftSummary {
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
    pub type_changed: usize,
    pub symlink_target_changed: usize,
    pub metadata_only_changed: usize,
    pub unknown_or_unexpected: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CurrentPrefixStateAssessment {
    UnderstoodAndAcceptable,
    UnderstoodButNotAccepted,
    UnknownDifferences,
    ProtectedStateChanged,
    ActiveContext,
    IdentityMismatch,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryRefreshDecision {
    NewGenerationAllowed,
    UseExistingGeneration,
    Blocked,
}
pub fn recovery_refresh_decision(
    assessment: CurrentPrefixStateAssessment,
    historical_verified: bool,
    current_source_matches: bool,
) -> RecoveryRefreshDecision {
    if assessment != CurrentPrefixStateAssessment::UnderstoodAndAcceptable || !historical_verified {
        RecoveryRefreshDecision::Blocked
    } else if current_source_matches {
        RecoveryRefreshDecision::UseExistingGeneration
    } else {
        RecoveryRefreshDecision::NewGenerationAllowed
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct ProtectedStateDrift {
    pub path: PathBuf,
    pub old: Option<SelectedPathEvidence>,
    pub current: Option<SelectedPathEvidence>,
    pub classification: RecoveryDriftClassification,
    pub evidence: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct RecoveryRegistryDrift {
    pub relative_path: String,
    pub sections: Vec<FixtureRegistrySectionMutation>,
    pub diagnosis: RegistryChangeDiagnosis,
    pub historical_before_hash_matches: bool,
    pub historical_after_hash_matches: bool,
    pub classification: RecoveryDriftClassification,
    pub evidence: Vec<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct RecoverySourceDriftReport {
    pub schema: u16,
    pub timestamp_unix_seconds: u64,
    pub old_backup_id: String,
    pub old_backup_manifest_sha256: String,
    pub old_backup_integrity: RecoveryVerification,
    pub source_identity: PrefixFileIdentity,
    pub current_tree_sha256: String,
    pub source_consistent_during_comparison: bool,
    pub summary: RecoveryDriftSummary,
    pub entries: Vec<RecoverySourceDriftEntry>,
    pub registry: Vec<RecoveryRegistryDrift>,
    pub protected_drift: Vec<ProtectedStateDrift>,
    pub steam_semantic_review: Option<lxmi_steam::SteamProtectedStateReview>,
    pub historical_baseline: BaselineReceipt,
    pub historical_attempt_evidence: Vec<BaselineReceipt>,
    pub proton_source_manifest_sha256: String,
    pub proton_source_matches_qualified_source: bool,
    pub assessment: CurrentPrefixStateAssessment,
    pub blockers: Vec<String>,
    pub evidence_receipt: Option<BaselineReceipt>,
    pub limitations: Vec<String>,
}

/// Content/metadata diff of the existing recovery inventory; no symlink target reads.
pub fn compare_recovery_entries(
    old: &[RecoveryEntry],
    current: &[RecoveryEntry],
) -> io::Result<Vec<RecoverySourceDriftEntry>> {
    fn index(entries: &[RecoveryEntry]) -> io::Result<BTreeMap<&str, &RecoveryEntry>> {
        let mut out = BTreeMap::new();
        for entry in entries {
            if out.insert(entry.relative_path.as_str(), entry).is_some() {
                return Err(io::Error::other("duplicate recovery entry"));
            }
        }
        Ok(out)
    }
    let a = index(old)?;
    let b = index(current)?;
    let mut report = Vec::new();
    for path in a.keys().chain(b.keys()).copied().collect::<BTreeSet<_>>() {
        let x = a.get(path).copied();
        let y = b.get(path).copied();
        if x == y {
            continue;
        }
        let change = match (x, y) {
            (None, Some(_)) => RecoveryDriftKind::Added,
            (Some(_), None) => RecoveryDriftKind::Removed,
            (Some(x), Some(y)) if x.kind != y.kind => RecoveryDriftKind::TypeChanged,
            (Some(x), Some(y)) if x.symlink_target != y.symlink_target => {
                RecoveryDriftKind::SymlinkTargetChanged
            }
            (Some(x), Some(y)) if x.sha256 != y.sha256 || x.size != y.size => {
                RecoveryDriftKind::Modified
            }
            _ => RecoveryDriftKind::MetadataOnlyChanged,
        };
        let permissions_equal = matches!((x,y),(Some(x),Some(y)) if x.mode==y.mode && x.uid==y.uid && x.gid==y.gid && x.xattr_hashes==y.xattr_hashes);
        let classification =
            if change == RecoveryDriftKind::MetadataOnlyChanged && permissions_equal {
                RecoveryDriftClassification::MetadataOnly
            } else if matches!((x, y), (Some(_), Some(_))) && !permissions_equal {
                RecoveryDriftClassification::Unexpected
            } else {
                RecoveryDriftClassification::Unknown
            };
        report.push(RecoverySourceDriftEntry {
            relative_path: path.into(),
            old: x.cloned(),
            current: y.cloned(),
            change,
            classification,
            evidence: vec![if classification == RecoveryDriftClassification::MetadataOnly {
                "Only mtime metadata differs; content/size/type/target/mode/owner/xattrs match. Writer UNKNOWN; this does not authorize a helper or normalize source state."
            } else {
                "Full recovery manifest comparison; no name-based exemption. Material change requires concrete correlation or remains blocked."
            }.into()],
        });
    }
    Ok(report)
}
pub fn recovery_drift_summary(entries: &[RecoverySourceDriftEntry]) -> RecoveryDriftSummary {
    let mut s = RecoveryDriftSummary::default();
    for entry in entries {
        match entry.change {
            RecoveryDriftKind::Added => s.added += 1,
            RecoveryDriftKind::Removed => s.removed += 1,
            RecoveryDriftKind::Modified => s.modified += 1,
            RecoveryDriftKind::TypeChanged => s.type_changed += 1,
            RecoveryDriftKind::SymlinkTargetChanged => s.symlink_target_changed += 1,
            RecoveryDriftKind::MetadataOnlyChanged => s.metadata_only_changed += 1,
        }
        if matches!(
            entry.classification,
            RecoveryDriftClassification::Unknown
                | RecoveryDriftClassification::Unexpected
                | RecoveryDriftClassification::ProtectedPathRelevant
        ) {
            s.unknown_or_unexpected += 1;
        }
    }
    s
}
pub fn classify_reconciled_registry(
    sections: &[FixtureRegistrySectionMutation],
    exact_historical_hash_correlation: bool,
) -> RecoveryDriftClassification {
    use PrefixMutationClassification as C;
    use RecoveryDriftClassification as R;
    if sections
        .iter()
        .any(|s| matches!(s.classification, C::Unknown | C::Unexpected))
    {
        return R::Unknown;
    }
    if sections
        .iter()
        .any(|s| s.classification == C::EnvironmentSpecific)
    {
        return if exact_historical_hash_correlation {
            R::KnownWineProtonMutation
        } else {
            R::Unknown
        };
    }
    if !sections.is_empty()
        && sections
            .iter()
            .all(|s| s.classification == C::VolatileExpected)
    {
        R::PreviouslyQualifiedVolatile
    } else {
        R::KnownWineProtonMutation
    }
}
pub fn compare_recovery_protected_state(
    old: &[SelectedPathEvidence],
    current: &[SelectedPathEvidence],
) -> Vec<ProtectedStateDrift> {
    let a: BTreeMap<_, _> = old.iter().map(|p| (&p.path, p)).collect();
    let b: BTreeMap<_, _> = current.iter().map(|p| (&p.path, p)).collect();
    a.keys().chain(b.keys()).copied().collect::<BTreeSet<_>>().into_iter().filter_map(|path| {
        let x=a.get(path).copied();let y=b.get(path).copied();
        if x==y && x.is_some_and(|p|p.issue.is_none()) { return None; }
        Some(ProtectedStateDrift { path:path.clone(),old:x.cloned(),current:y.cloned(),classification:RecoveryDriftClassification::ProtectedPathRelevant,evidence:"Selected protected evidence differs from pinned 0.8.7 baseline or is incomplete. No writer/Steam semantic inference from hashes; recovery refresh prohibited until reviewed.".into() })
    }).collect()
}
pub fn assess_current_prefix(
    inactive: bool,
    identity_matches: bool,
    recovery_verified: bool,
    source_consistent: bool,
    runtime_same: bool,
    protected_acceptable: bool,
    entries: &[RecoverySourceDriftEntry],
) -> CurrentPrefixStateAssessment {
    use CurrentPrefixStateAssessment::*;
    if !inactive {
        return ActiveContext;
    }
    if !identity_matches {
        return IdentityMismatch;
    }
    if !protected_acceptable {
        return ProtectedStateChanged;
    }
    if !recovery_verified || !source_consistent || !runtime_same {
        return UnknownDifferences;
    }
    if entries.iter().any(|e| {
        matches!(
            e.classification,
            RecoveryDriftClassification::Unknown
                | RecoveryDriftClassification::Unexpected
                | RecoveryDriftClassification::ProtectedPathRelevant
        )
    }) {
        return UnknownDifferences;
    }
    UnderstoodAndAcceptable
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recovery_refresh_requires_accepted_changed_source_and_verified_history() {
        use CurrentPrefixStateAssessment::*;
        use RecoveryRefreshDecision::*;
        assert_eq!(
            recovery_refresh_decision(UnderstoodAndAcceptable, true, false),
            NewGenerationAllowed
        );
        assert_eq!(
            recovery_refresh_decision(UnderstoodAndAcceptable, true, true),
            UseExistingGeneration
        );
        assert_eq!(
            recovery_refresh_decision(UnderstoodAndAcceptable, false, false),
            Blocked
        );
        for assessment in [
            UnderstoodButNotAccepted,
            UnknownDifferences,
            ProtectedStateChanged,
            ActiveContext,
            IdentityMismatch,
        ] {
            for source_matches in [false, true] {
                assert_eq!(
                    recovery_refresh_decision(assessment, true, source_matches),
                    Blocked
                );
            }
        }
    }
    fn entry(path: &str) -> RecoveryEntry {
        RecoveryEntry {
            relative_path: path.into(),
            kind: RecoveryEntryKind::RegularFile,
            size: 1,
            mode: 0o100600,
            uid: 1000,
            gid: 1000,
            mtime_seconds: 1,
            mtime_nanoseconds: 0,
            symlink_target: None,
            sha256: Some("hash".into()),
            xattr_hashes: BTreeMap::new(),
        }
    }
    #[test]
    fn structural_diff_has_explicit_kinds_and_never_follows_links() {
        let a = entry("file");
        let mut b = a.clone();
        b.sha256 = Some("new".into());
        assert_eq!(
            compare_recovery_entries(std::slice::from_ref(&a), &[b]).unwrap()[0].change,
            RecoveryDriftKind::Modified
        );
        assert_eq!(
            compare_recovery_entries(&[], std::slice::from_ref(&a)).unwrap()[0].change,
            RecoveryDriftKind::Added
        );
        assert_eq!(
            compare_recovery_entries(std::slice::from_ref(&a), &[]).unwrap()[0].change,
            RecoveryDriftKind::Removed
        );
        let mut b = a.clone();
        b.kind = RecoveryEntryKind::Symlink;
        assert_eq!(
            compare_recovery_entries(std::slice::from_ref(&a), std::slice::from_ref(&b)).unwrap()
                [0]
            .change,
            RecoveryDriftKind::TypeChanged
        );
        let mut c = b.clone();
        c.symlink_target = Some("/outside/not-read".into());
        assert_eq!(
            compare_recovery_entries(&[b], &[c]).unwrap()[0].change,
            RecoveryDriftKind::SymlinkTargetChanged
        );
        let mut b = a.clone();
        b.mtime_seconds += 1;
        let diff = compare_recovery_entries(std::slice::from_ref(&a), &[b]).unwrap();
        assert_eq!(
            diff[0].classification,
            RecoveryDriftClassification::MetadataOnly
        );
        assert_eq!(
            assess_current_prefix(true, true, true, true, true, true, &diff),
            CurrentPrefixStateAssessment::UnderstoodAndAcceptable
        );
        let mut b = a.clone();
        b.mode = 0o100777;
        assert_eq!(
            compare_recovery_entries(&[a], &[b]).unwrap()[0].classification,
            RecoveryDriftClassification::Unexpected
        );
    }
    #[test]
    fn unknown_and_protected_state_block_refresh() {
        let diff = compare_recovery_entries(&[], &[entry("unknown")]).unwrap();
        assert_eq!(
            assess_current_prefix(true, true, true, true, true, true, &diff),
            CurrentPrefixStateAssessment::UnknownDifferences
        );
        assert_eq!(
            assess_current_prefix(true, true, true, true, true, false, &[]),
            CurrentPrefixStateAssessment::ProtectedStateChanged
        );
        assert_eq!(
            assess_current_prefix(false, true, true, true, true, true, &[]),
            CurrentPrefixStateAssessment::ActiveContext
        );
        assert_eq!(
            assess_current_prefix(true, false, true, true, true, true, &[]),
            CurrentPrefixStateAssessment::IdentityMismatch
        );
        assert_eq!(
            assess_current_prefix(true, true, true, false, true, true, &[]),
            CurrentPrefixStateAssessment::UnknownDifferences
        );
        assert!(compare_recovery_entries(&[entry("a"), entry("a")], &[]).is_err());
    }
    #[test]
    fn historical_font_correlation_and_volatility_do_not_hide_unknown_fields() {
        let mut s = FixtureRegistrySectionMutation {
            section: "synthetic".into(),
            changed_fields: vec![],
            before_sha256: None,
            after_sha256: None,
            classification: PrefixMutationClassification::VolatileExpected,
            evidence: "synthetic".into(),
        };
        assert_eq!(
            classify_reconciled_registry(std::slice::from_ref(&s), false),
            RecoveryDriftClassification::PreviouslyQualifiedVolatile
        );
        s.classification = PrefixMutationClassification::EnvironmentSpecific;
        assert_eq!(
            classify_reconciled_registry(std::slice::from_ref(&s), false),
            RecoveryDriftClassification::Unknown
        );
        assert_eq!(
            classify_reconciled_registry(std::slice::from_ref(&s), true),
            RecoveryDriftClassification::KnownWineProtonMutation
        );
        let mut unknown = s.clone();
        unknown.classification = PrefixMutationClassification::Unknown;
        assert_eq!(
            classify_reconciled_registry(&[s, unknown], true),
            RecoveryDriftClassification::Unknown
        );
    }
    #[test]
    fn protected_mismatch_includes_metadata_only_and_missing_entries() {
        let a = SelectedPathEvidence {
            path: "/protected/fixture".into(),
            metadata: None,
            sha256: Some("hash".into()),
            directory_entries: None,
            issue: None,
        };
        assert!(compare_recovery_protected_state(
            std::slice::from_ref(&a),
            std::slice::from_ref(&a)
        )
        .is_empty());
        let mut b = a.clone();
        b.issue = Some("unavailable".into());
        assert_eq!(
            compare_recovery_protected_state(std::slice::from_ref(&a), &[b]).len(),
            1
        );
        assert_eq!(
            compare_recovery_protected_state(&[a], &[])[0].classification,
            RecoveryDriftClassification::ProtectedPathRelevant
        );
    }
}
