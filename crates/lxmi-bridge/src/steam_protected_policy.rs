//! Raw evidence remains diagnostic; reviewed selected Steam semantics gate two exact files.
use super::*;
use lxmi_steam::{
    SteamProtectedSemanticState, SteamProtectedStateAssessment, SteamProtectedStateReview,
    SteamSemanticStability,
};

pub fn steam_semantics_bind_protected_paths(
    state: &SteamProtectedSemanticState,
    paths: &[SelectedPathEvidence],
) -> bool {
    [&state.config_raw, &state.localconfig_raw]
        .iter()
        .all(|raw| {
            paths.iter().filter(|p| p.path == raw.path).count() == 1
                && paths.iter().find(|p| p.path == raw.path).is_some_and(|p| {
                    p.issue.is_none()
                        && p.sha256.as_deref() == Some(&raw.sha256)
                        && p.metadata.as_ref().is_some_and(|m| {
                            m.size == raw.size
                                && m.inode == raw.inode
                                && m.modified_seconds == raw.modified_seconds
                                && m.modified_nanoseconds == raw.modified_nanoseconds
                                && m.symlink_target.is_none()
                        })
                })
        })
}
pub fn reviewed_steam_state_matches(
    review: &SteamProtectedStateReview,
    state: &SteamProtectedSemanticState,
) -> bool {
    review.assessment == SteamProtectedStateAssessment::ReviewedAndAcceptable
        && matches!(
            review.stability,
            SteamSemanticStability::Stable | SteamSemanticStability::RawChangedSemanticStable
        )
        && review.observation_seconds >= 1
        && lxmi_steam::assess_steam_semantics(state)
            == SteamProtectedStateAssessment::ReviewedAndAcceptable
        && matches!(
            lxmi_steam::compare_steam_stability(&review.after, state),
            SteamSemanticStability::Stable | SteamSemanticStability::RawChangedSemanticStable
        )
}
pub fn steam_path_semantics_match(
    a: &GamePrefixBaseline,
    b: &GamePrefixBaseline,
    path: &Path,
) -> bool {
    let Some((x, y)) = a
        .steam_semantic_state
        .as_ref()
        .zip(b.steam_semantic_state.as_ref())
    else {
        return false;
    };
    if !steam_semantics_bind_protected_paths(x, &a.protected_paths)
        || !steam_semantics_bind_protected_paths(y, &b.protected_paths)
    {
        return false;
    }
    if ![&x.config_raw.path, &x.localconfig_raw.path].contains(&&path.to_path_buf()) {
        return false;
    }
    lxmi_steam::assess_steam_semantics(x) == SteamProtectedStateAssessment::ReviewedAndAcceptable
        && lxmi_steam::assess_steam_semantics(y)
            == SteamProtectedStateAssessment::ReviewedAndAcceptable
        && matches!(
            lxmi_steam::compare_steam_stability(x, y),
            SteamSemanticStability::Stable | SteamSemanticStability::RawChangedSemanticStable
        )
}
pub fn protected_baselines_match(a: &GamePrefixBaseline, b: &GamePrefixBaseline) -> bool {
    if a.protected_paths.len() != b.protected_paths.len() {
        return false;
    }
    let paths = a
        .protected_paths
        .iter()
        .map(|p| &p.path)
        .collect::<std::collections::BTreeSet<_>>();
    if paths.len() != a.protected_paths.len() {
        return false;
    }
    a.protected_paths.iter().all(|old| {
        let matches = b
            .protected_paths
            .iter()
            .filter(|p| p.path == old.path)
            .collect::<Vec<_>>();
        matches.len() == 1 && (matches[0] == old || steam_path_semantics_match(a, b, &old.path))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (PathBuf, GamePrefixBaseline) {
        let root = std::env::temp_dir().join(format!(
            "lxmi-semantic-policy-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("config")).unwrap();
        fs::create_dir_all(root.join("userdata/1416423935/config")).unwrap();
        fs::write(
            root.join("config/config.vdf"),
            r#""InstallConfigStore" { "Software" { "Valve" { "Steam" { } } } }"#,
        )
        .unwrap();
        fs::write(root.join("userdata/1416423935/config/localconfig.vdf"),r#""UserLocalConfigStore" { "Software" { "Valve" { "Steam" { "apps" { "4162040" { "LastPlayed" "1" } } } } } }"#).unwrap();
        let state = lxmi_steam::inspect_zzz_steam_semantic_state(&root).unwrap();
        let baseline = from_state(state);
        (root, baseline)
    }
    fn from_state(state: SteamProtectedSemanticState) -> GamePrefixBaseline {
        let protected_paths = [&state.config_raw, &state.localconfig_raw]
            .into_iter()
            .map(|r| SelectedPathEvidence {
                path: r.path.clone(),
                metadata: Some(PrefixMetadata {
                    size: r.size,
                    modified_seconds: r.modified_seconds,
                    modified_nanoseconds: r.modified_nanoseconds,
                    inode: r.inode,
                    symlink_target: None,
                }),
                sha256: Some(r.sha256.clone()),
                directory_entries: None,
                issue: None,
            })
            .collect();
        GamePrefixBaseline {
            schema: 2,
            timestamp_unix_seconds: state.captured_at,
            steam_semantic_state: Some(state),
            prefix_identity: None,
            selected_prefix_paths: vec![],
            protected_paths,
            policy: GamePrefixSnapshotPolicy {
                file_byte_limit: 2048,
                directory_entry_limit: 1,
                exclusions: vec![],
            },
        }
    }
    #[test]
    fn raw_change_is_diagnostic_but_semantic_change_is_protected() {
        let (root, a) = fixture();
        fs::write(root.join("userdata/1416423935/config/localconfig.vdf"),r#""UserLocalConfigStore" { "Software" { "Valve" { "Steam" { "apps" { "4162040" { "LastPlayed" "2" } } } } } }"#).unwrap();
        let b = from_state(lxmi_steam::inspect_zzz_steam_semantic_state(&root).unwrap());
        assert_ne!(a.protected_paths, b.protected_paths);
        assert!(protected_baselines_match(&a, &b));
        assert!(compare_prefix_entry(&a, &b)
            .iter()
            .all(|m| m.kind == PrefixMutationKind::UnrelatedSteamState));
        fs::write(root.join("userdata/1416423935/config/localconfig.vdf"),r#""UserLocalConfigStore" { "Software" { "Valve" { "Steam" { "apps" { "4162040" { "LaunchOptions" "-changed" } } } } } }"#).unwrap();
        let c = from_state(lxmi_steam::inspect_zzz_steam_semantic_state(&root).unwrap());
        assert!(!protected_baselines_match(&a, &c));
        assert!(compare_prefix_entry(&a, &c)
            .iter()
            .any(|m| m.kind == PrefixMutationKind::ProtectedPathMutation));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn missing_unbound_and_unknown_semantics_do_not_exempt_raw_changes() {
        let (root, a) = fixture();
        let mut b = a.clone();
        b.protected_paths[0].sha256 = Some("tampered".into());
        assert!(!protected_baselines_match(&a, &b));
        b = a.clone();
        b.steam_semantic_state = None;
        b.protected_paths[0].metadata.as_mut().unwrap().inode += 1;
        assert!(!protected_baselines_match(&a, &b));
        b = a.clone();
        b.steam_semantic_state
            .as_mut()
            .unwrap()
            .config
            .issues
            .push("unknown".into());
        b.protected_paths[0].metadata.as_mut().unwrap().inode += 1;
        assert!(!protected_baselines_match(&a, &b));
        b = a.clone();
        b.protected_paths.push(b.protected_paths[0].clone());
        assert!(!protected_baselines_match(&a, &b));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn review_is_required_and_other_protected_paths_stay_strict() {
        let (root, mut a) = fixture();
        let state = a.steam_semantic_state.as_ref().unwrap().clone();
        let mut review = lxmi_steam::review_zzz_steam_state(&root, 1).unwrap();
        assert!(reviewed_steam_state_matches(&review, &state));
        review.assessment = SteamProtectedStateAssessment::NeedsEvidence;
        assert!(!reviewed_steam_state_matches(&review, &state));
        a.protected_paths.push(SelectedPathEvidence {
            path: root.join("ZenlessZoneZero.exe"),
            metadata: None,
            sha256: Some("original".into()),
            directory_entries: None,
            issue: None,
        });
        let mut b = a.clone();
        b.protected_paths.last_mut().unwrap().sha256 = Some("changed".into());
        assert!(!protected_baselines_match(&a, &b));
        fs::remove_dir_all(root).unwrap();
    }
}
