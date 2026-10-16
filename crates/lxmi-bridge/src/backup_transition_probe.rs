//! Diagnostic only: immutable pre-entry backup to two disposable private prefixes.
//! This module cannot issue real-prefix consent or remove the durable one-shot guard.
use super::*;

const BACKUP: &str = "f4ea33c0c047d893319296420fd86784";
const MANIFEST: &str = "182929afbefb7e08b226cb7e1651a44a66df95e55e12422c58752bf81ed40f80";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupTransitionFinding {
    InitialFontTransitionThenStable,
    PersistentFontMutation,
    UnknownMutation,
    ReplicaMismatch,
    NoFontTransition,
    Incomplete,
}

fn font_section(section: &str) -> bool {
    matches!(
        section,
        "system.reg:Software\\Microsoft\\Windows NT\\CurrentVersion\\Fonts"
            | "system.reg:Software\\Microsoft\\Windows\\CurrentVersion\\Fonts"
            | "user.reg:Software\\Wine\\Fonts\\External Fonts"
    )
}

/// Findings describe this private topology only. They never authorize a real invocation.
pub fn assess_backup_transition(
    replicas: &[Vec<Vec<FixtureRegistrySectionMutation>>],
) -> BackupTransitionFinding {
    use BackupTransitionFinding as F;
    use PrefixMutationClassification as C;
    if replicas.len() != 2 || replicas.iter().any(|r| r.len() != 2) {
        return F::Incomplete;
    }
    if replicas.iter().flatten().flatten().any(|s| {
        !matches!(s.classification, C::StableExpected | C::VolatileExpected)
            && !(s.classification == C::EnvironmentSpecific && font_section(&s.section))
    }) {
        return F::UnknownMutation;
    }
    if replicas
        .iter()
        .any(|r| r[1].iter().any(|s| font_section(&s.section)))
    {
        return F::PersistentFontMutation;
    }
    let initial_fonts = |r: &Vec<Vec<FixtureRegistrySectionMutation>>| {
        r[0].iter()
            .filter(|s| font_section(&s.section))
            .map(|s| {
                (
                    s.section.clone(),
                    s.changed_fields.clone(),
                    s.before_sha256.clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    if initial_fonts(&replicas[0]) != initial_fonts(&replicas[1]) {
        return F::ReplicaMismatch;
    }
    if initial_fonts(&replicas[0]).is_empty() {
        F::NoFontTransition
    } else {
        F::InitialFontTransitionThenStable
    }
}

fn failed(message: impl ToString) -> BridgeError {
    BridgeError::new(BridgeErrorCode::UnsafePrefixContext, message.to_string())
}

fn settled(prefix: &Path) -> Result<QualifiedRegistryCapture, BridgeError> {
    thread::sleep(Duration::from_secs(5));
    for _ in 0..5 {
        let a = qualified_registry_capture(prefix)?;
        thread::sleep(Duration::from_secs(2));
        let b = qualified_registry_capture(prefix)?;
        if a == b {
            return Ok(b);
        }
    }
    Err(failed("private probe registry did not settle; no retry"))
}

fn registry_hashes(registry: &QualifiedRegistryCapture) -> BTreeMap<String, String> {
    registry
        .iter()
        .map(|(k, v)| (k.clone(), format!("{:x}", Sha256::digest(v))))
        .collect()
}

/// Fixed historical backup only, no arbitrary source/destination or executable input.
/// The caller must first obtain a fresh recipe through the existing private self-test.
pub fn investigate_pre_entry_backup(
    storage: &Path,
    steam_root: &Path,
    recipe: &QualifiedLinuxRuntimeRecipe,
) -> Result<PathBuf, BridgeError> {
    recipe.require_qualified()?;
    let identity = PrefixFileIdentity {
        // Historical identity, not a read of the current real prefix.
        canonical_path: steam_root.join("steamapps/compatdata/4162040/pfx"),
        uid: 1000,
        device: 66306,
        inode: 11538172,
    };
    let mut backup = load_recovery_generation(storage, BACKUP, &identity).map_err(failed)?;
    if backup.manifest_sha256 != MANIFEST || backup.proton_build != ZZZ_OBSERVED_BUILD {
        return Err(failed("fixed pre-entry backup identity changed"));
    }
    verify_prefix_recovery(storage, &mut backup).map_err(failed)?;
    if backup.integrity.state != RecoveryVerificationState::Verified {
        return Err(failed("pre-entry backup integrity not verified"));
    }
    let expected_backup = storage
        .join("backups/zenless-zone-zero/prefix")
        .join(BACKUP);
    if backup.backup_path != expected_backup {
        return Err(failed("unexpected historical backup path"));
    }
    let tree = safe_existing_directory(storage, &expected_backup.join("tree"))?;
    let source_before = prefix_recovery::private_tree_inventory(&tree).map_err(failed)?;
    let initial_registry = qualified_registry_capture(&tree)?;
    let mut contexts = Vec::new();
    let mut clone_manifests = Vec::new();
    for _ in 0..2 {
        let root = recipe.new_entry_session(storage)?;
        let compatdata = prepare_isolated_test_context(&root)?;
        let prefix = compatdata.join("pfx");
        let manifest = prefix_recovery::clone_private_tree(&tree, &prefix).map_err(failed)?;
        if qualified_registry_capture(&prefix)? != initial_registry {
            return Err(failed("private clone initial registry mismatch"));
        }
        clone_manifests.push(manifest);
        contexts.push((root, compatdata));
    }
    if clone_manifests[0] != clone_manifests[1] {
        return Err(failed("private clone initial tree mismatch"));
    }
    let mut evidence = Vec::new();
    let mut replicas = Vec::new();
    let mut metadata_matches = true;
    for (index, (root, compatdata)) in contexts.iter().enumerate() {
        let prefix = compatdata.join("pfx");
        let prefix_identity = prefix_file_identity(&prefix);
        let config = BridgeTestConfig {
            managed_root: root.clone(),
            runtime_root: root.join("marker"),
            explicit_runtime: recipe.runtime(),
            steam_client_install_path: steam_root.into(),
            prefix_mode: PrefixMode::IsolatedTemporaryPrefix,
        };
        let mut before = initial_registry.clone();
        let mut sections_per_run = Vec::new();
        for run in 1..=2 {
            eprintln!("private historical-state replica {} / run {run}", index + 1);
            let before_metadata = qualified_prefix_metadata(&prefix)?;
            let invocation = RealPrefixInvocation {
                compatdata: compatdata.clone(),
                working_directory: root.join("test-prefixes/bridge-v1"),
                nonce: random_nonce()?,
            };
            let bridge = recipe.execute(&config, &invocation, false, None)?;
            let after = settled(&prefix)?;
            if prefix_file_identity(&prefix) != prefix_identity {
                return Err(failed("private prefix root identity changed"));
            }
            let after_metadata = qualified_prefix_metadata(&prefix)?;
            let metadata_match =
                qualified_prefix_metadata_matches(&before_metadata, &after_metadata);
            metadata_matches &= metadata_match;
            let sections = qualified_registry_diff(&before, &after)?;
            let mut diagnosis = BTreeMap::new();
            for (hive, bytes) in &after {
                diagnosis.insert(
                    hive.clone(),
                    diagnose_registry_change(
                        &before[hive],
                        bytes,
                        &format!("{:x}", Sha256::digest(&before[hive])),
                        &format!("{:x}", Sha256::digest(bytes)),
                    )
                    .map_err(|e| failed(format!("{e:?}")))?,
                );
            }
            evidence.push(serde_json::json!({"replica":index+1,"run":run,"prefix_identity":prefix_identity,
                "bridge":bridge,"sections":sections,"diagnosis":diagnosis,
                "registry_before_sha256":registry_hashes(&before),"registry_after_sha256":registry_hashes(&after),
                "metadata_before":before_metadata,"metadata_after":after_metadata,"metadata_matches":metadata_match}));
            sections_per_run.push(sections);
            before = after;
        }
        replicas.push(sections_per_run);
    }
    recipe.require_qualified()?;
    verify_prefix_recovery(storage, &mut backup).map_err(failed)?;
    if backup.integrity.state != RecoveryVerificationState::Verified
        || source_before != prefix_recovery::private_tree_inventory(&tree).map_err(failed)?
    {
        return Err(failed(
            "historical backup changed during private investigation",
        ));
    }
    let report = serde_json::json!({"schema":1,"purpose":"historical initial-state transition; diagnostic only",
        "backup_id":BACKUP,"manifest_sha256":MANIFEST,"backup_verified_and_unchanged":true,
        "identical_initial_tree_manifests":clone_manifests,"initial_registry_sha256":registry_hashes(&initial_registry),
        "recipe":recipe.panel(),"runs":evidence,"finding":assess_backup_transition(&replicas),
        "selected_metadata_matches":metadata_matches,"real_prefix_executed":false,"real_permission_granted":false,
        "limitations":["Restored pfx content, not compatdata parent or original filesystem inode identities",
            "Private result cannot accept the failed real attempt or authorize another attempt",
            "Initial font transition remains EnvironmentSpecific; no mutation classifier was broadened"]});
    let path = contexts[0].0.join("historical-transition-probe.json");
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&path)
        .map_err(failed)?;
    file.write_all(&serde_json::to_vec_pretty(&report).map_err(failed)?)
        .map_err(failed)?;
    file.sync_all().map_err(failed)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fonts() -> FixtureRegistrySectionMutation {
        FixtureRegistrySectionMutation {
            section: "user.reg:Software\\Wine\\Fonts\\External Fonts".into(),
            changed_fields: vec!["font".into()],
            before_sha256: Some("same".into()),
            after_sha256: Some("other".into()),
            classification: PrefixMutationClassification::EnvironmentSpecific,
            evidence: "diagnostic".into(),
        }
    }
    #[test]
    fn initial_transition_diagnostic_never_qualifies_real_mutation() {
        let r = vec![vec![vec![fonts()], vec![]], vec![vec![fonts()], vec![]]];
        assert_eq!(
            assess_backup_transition(&r),
            BackupTransitionFinding::InitialFontTransitionThenStable
        );
        assert_eq!(
            qualify_real_executor_evidence(
                LinuxRuntimeKind::SteamLinuxRuntimeContainer,
                2,
                true,
                true,
                true,
                &[fonts().classification]
            ),
            RealPrefixExecutorQualification::FixtureMutationMismatch
        );
    }
    #[test]
    fn recurring_fonts_unknown_fields_inconsistent_initial_state_and_incomplete_block_findings() {
        let mut r = vec![vec![vec![fonts()], vec![]], vec![vec![fonts()], vec![]]];
        r[0][1].push(fonts());
        assert_eq!(
            assess_backup_transition(&r),
            BackupTransitionFinding::PersistentFontMutation
        );
        r[0][1].clear();
        r[0][0][0].changed_fields.push("extra".into());
        assert_eq!(
            assess_backup_transition(&r),
            BackupTransitionFinding::ReplicaMismatch
        );
        r[0][0][0].classification = PrefixMutationClassification::Unknown;
        assert_eq!(
            assess_backup_transition(&r),
            BackupTransitionFinding::UnknownMutation
        );
        assert_eq!(
            assess_backup_transition(&[]),
            BackupTransitionFinding::Incomplete
        );
        assert_eq!(
            assess_backup_transition(&vec![vec![vec![], vec![]]; 2]),
            BackupTransitionFinding::NoFontTransition
        );
    }
    #[test]
    fn unqualified_recipe_rejected_before_storage_access() {
        let error = investigate_pre_entry_backup(
            Path::new("/not-created"),
            Path::new("/not-created"),
            &QualifiedLinuxRuntimeRecipe::unqualified_test_recipe(),
        )
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("linux_runtime_environment_unqualified"));
    }
}
