//! All payloads are synthetic, non-functional text. No upstream binaries included.
use lxmi_core::*;
use lxmi_proton::*;
use lxmi_steam::*;
use lxmi_xxmi::*;
use std::{
    fs,
    os::unix::fs::{symlink, PermissionsExt},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Tree(PathBuf);
impl Tree {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "lxmi-xxmi-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn store(&self) -> ManagedStore {
        ManagedStore::at(self.0.join("lxmi"), ImportLimits::default()).unwrap()
    }
    fn wwmi(&self) -> PathBuf {
        let p = self.0.join("WWMI-source");
        fs::create_dir(&p).unwrap();
        for f in WWMI_REQUIRED_FILES {
            let path = p.join(f);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"synthetic resource; not executable").unwrap();
        }
        fs::write(
            p.join("d3dx.ini"),
            "[Include]\ninclude = Core\\WWMI\\WuWa-Model-Importer.ini\n",
        )
        .unwrap();
        fs::write(
            p.join("Core/WWMI/WuWa-Model-Importer.ini"),
            "namespace=WWMIv1\n[Constants]\nglobal $wwmi_version = synthetic-2026\n",
        )
        .unwrap();
        p
    }
    fn zzmi(&self) -> PathBuf {
        let p = self.0.join("ZZMI-source");
        fs::create_dir(&p).unwrap();
        for file in ZZMI_REQUIRED_FILES {
            let path = p.join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, b"synthetic ZZMI fixture; not upstream payload").unwrap();
        }
        fs::write(
            p.join("d3dx.ini"),
            "[Loader]\ntarget = ZenlessZoneZero.exe\n[Include]\ninclude = Core\\ZZMI\\main.ini\n",
        )
        .unwrap();
        fs::write(
            p.join("Core/ZZMI/main.ini"),
            "[Constants]\nglobal $version = synthetic-zzmi-1.45\n",
        )
        .unwrap();
        p
    }
    fn libs(&self) -> PathBuf {
        let p = self.0.join("libraries");
        fs::create_dir(&p).unwrap();
        for n in ["3dmloader.dll", "d3d11.dll", "d3dcompiler_47.dll"] {
            fs::write(p.join(n), b"synthetic; not a DLL").unwrap();
        }
        fs::write(p.join("Manifest.json"),r#"{"version":"nightly-example","signatures":{"3dmloader.dll":"fixture","d3d11.dll":"fixture","d3dcompiler_47.dll":"fixture"}}"#).unwrap();
        p
    }
    fn platform(&self) -> lxmi_runtime::GameRuntimePlan {
        let game_path = self.0.join("game");
        fs::create_dir_all(&game_path).unwrap();
        fs::write(game_path.join("sentinel"), "unchanged").unwrap();
        let game = SteamGameInstallation {
            installation: GameInstallation {
                game: SUPPORTED_GAMES[0],
                install_path: game_path,
                status: GameInstallationStatus::Installed,
                distribution: GameDistribution::Steam,
                executable_path: None,
                executable_status: GameExecutableStatus::NotScanned,
            },
            steam_app_id: 3513350,
            manifest_name: "synthetic".into(),
            steam_library: self.0.join("Steam"),
            compatdata: ProtonCompatData {
                app_id: 3513350,
                compatdata_path: self.0.join("Steam/steamapps/compatdata/3513350"),
                prefix_path: None,
                status: ProtonCompatDataStatus::NotFound,
            },
        };
        lxmi_runtime::plan_game(
            SUPPORTED_GAMES[0],
            SteamGameDiscoveryStatus::Complete,
            Some(&game),
            &CompatibilityToolDiscoveryResult {
                status: CompatibilityToolDiscoveryStatus::Complete,
                tools: vec![],
                issues: vec![],
            },
        )
    }
    fn zzz_platform(&self) -> lxmi_runtime::GameRuntimePlan {
        let game_path = self.0.join("Zenless Zone Zero");
        fs::create_dir_all(game_path.join("games/ZenlessZoneZero Game")).unwrap();
        let executable = game_path.join("games/ZenlessZoneZero Game/ZenlessZoneZero.exe");
        fs::write(&executable, b"synthetic executable marker").unwrap();
        fs::write(game_path.join("sentinel"), "unchanged").unwrap();
        let installation = SteamGameInstallation {
            installation: GameInstallation {
                game: SUPPORTED_GAMES[1],
                install_path: game_path,
                status: GameInstallationStatus::Installed,
                distribution: GameDistribution::Steam,
                executable_path: Some(executable),
                executable_status: GameExecutableStatus::Found,
            },
            steam_app_id: ZENLESS_ZONE_ZERO_APP_ID,
            manifest_name: "Zenless Zone Zero (synthetic)".into(),
            steam_library: self.0.join("Steam"),
            compatdata: ProtonCompatData {
                app_id: ZENLESS_ZONE_ZERO_APP_ID,
                compatdata_path: self.0.join("Steam/steamapps/compatdata/4162040"),
                prefix_path: None,
                status: ProtonCompatDataStatus::NotFound,
            },
        };
        lxmi_runtime::plan_game(
            SUPPORTED_GAMES[1],
            SteamGameDiscoveryStatus::Complete,
            Some(&installation),
            &CompatibilityToolDiscoveryResult {
                status: CompatibilityToolDiscoveryStatus::Complete,
                tools: vec![],
                issues: vec![],
            },
        )
    }
}
impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn snapshot(path: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn visit(base: &Path, path: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
        let mut entries = fs::read_dir(path)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect::<Vec<_>>();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                out.push((p.strip_prefix(base).unwrap().into(), vec![]));
                visit(base, &p, out)
            } else {
                out.push((p.strip_prefix(base).unwrap().into(), fs::read(&p).unwrap()));
            }
        }
    }
    let mut out = vec![];
    visit(path, path, &mut out);
    out
}
#[test]
fn valid_wwmi_fixture_has_evidence_and_raw_version() {
    let t = Tree::new();
    let p = t.wwmi();
    let v = validate_directory(&p, ImportLimits::default()).unwrap();
    assert_eq!(v.kind, PackageKind::GameIntegration(IntegrationKind::Wwmi));
    assert_eq!(v.version.unwrap().raw, "synthetic-2026");
    assert!(!v.evidence.is_empty());
}

#[test]
fn valid_zzmi_fixture_uses_upstream_backed_structure_and_raw_version() {
    let tree = Tree::new();
    let package = tree.zzmi();
    let inspection = validate_directory(&package, ImportLimits::default()).unwrap();

    assert_eq!(
        inspection.kind,
        PackageKind::GameIntegration(IntegrationKind::Zzmi)
    );
    assert_eq!(inspection.version.unwrap().raw, "synthetic-zzmi-1.45");
    assert!(inspection
        .evidence
        .iter()
        .any(|item| item.contains("authenticity")));
}

#[test]
fn zzmi_fixture_requires_the_upstream_declared_files_and_target() {
    let tree = Tree::new();
    let package = tree.zzmi();
    fs::remove_file(package.join("Core/ZZMI/help.ini")).unwrap();
    let error = validate_directory(&package, ImportLimits::default()).unwrap_err();
    assert_eq!(error.code, ErrorCode::MissingRequiredFile);
    assert!(error.detail.contains("Core/ZZMI/help.ini"));

    let other_tree = Tree::new();
    let package = other_tree.zzmi();
    fs::write(
        package.join("d3dx.ini"),
        "[Loader]\ntarget = unrelated.exe\n[Include]\ninclude = Core\\ZZMI\\main.ini\n",
    )
    .unwrap();
    assert_eq!(
        validate_directory(&package, ImportLimits::default())
            .unwrap_err()
            .code,
        ErrorCode::InvalidPackage
    );
}

#[test]
fn game_to_integration_registry_keeps_wwmi_and_adds_zzmi() {
    assert_eq!(
        integration_for_game("wuthering-waves"),
        Some(IntegrationKind::Wwmi)
    );
    assert_eq!(
        integration_for_game("zenless-zone-zero"),
        Some(IntegrationKind::Zzmi)
    );
    assert_eq!(integration_for_game("unknown"), None);
}

#[test]
fn runtime_discovery_identifies_synthetic_zzmi_structure_without_authenticating_it() {
    let tree = Tree::new();
    let package = tree.zzmi();
    let runtime = inspect_runtime(&package);

    assert_eq!(runtime.integration_kind, Some(IntegrationKind::Zzmi));
    assert_eq!(runtime.integration, Presence::StructurallyPresent);
    assert_eq!(
        runtime.version.as_ref().map(|version| version.raw.as_str()),
        Some("synthetic-zzmi-1.45")
    );
    assert_eq!(
        runtime.launch_compatibility,
        LaunchCompatibility::NotVerified
    );
}

#[test]
fn importing_zzmi_uses_the_same_managed_store_pipeline_and_preserves_source() {
    let tree = Tree::new();
    let source = tree.zzmi();
    let before = snapshot(&source);
    let store = tree.store();
    let imported = store.import_directory(&source).unwrap();

    assert_eq!(
        imported.manifest().kind,
        PackageKind::GameIntegration(IntegrationKind::Zzmi)
    );
    assert_eq!(snapshot(&source), before);
    assert_eq!(store.list().unwrap().len(), 1);
    assert!(imported
        .manifest()
        .layout_reference
        .contains("leotorrez/ZZMI-Package"));
    assert_eq!(
        imported.manifest().authenticity,
        PackageAuthenticity::NotAuthenticated
    );
}

#[test]
fn integration_packages_are_rejected_for_the_other_game() {
    let tree = Tree::new();
    let store = tree.store();
    let zzmi = store.import_directory(&tree.zzmi()).unwrap();
    let wwmi = store.import_directory(&tree.wwmi()).unwrap();
    let zzz = tree.zzz_platform();
    let wuw = tree.platform();

    assert_eq!(
        plan_installation_for_integration(
            &store,
            &zzz,
            IntegrationKind::Wwmi,
            &wwmi.manifest().id,
            None,
        )
        .unwrap_err()
        .code,
        ErrorCode::WrongGameIntegration
    );
    assert_eq!(
        plan_installation_for_integration(
            &store,
            &wuw,
            IntegrationKind::Zzmi,
            &zzmi.manifest().id,
            None,
        )
        .unwrap_err()
        .code,
        ErrorCode::WrongGameIntegration
    );
}

#[test]
fn zzz_assessment_keeps_steam_and_linux_proton_support_unverified() {
    let tree = Tree::new();
    let store = tree.store();
    let zzmi = store.import_directory(&tree.zzmi()).unwrap();
    let libraries = store.import_directory(&tree.libs()).unwrap();
    let assessment = assess_for_integration(
        &tree.zzz_platform(),
        IntegrationKind::Zzmi,
        Some(&zzmi),
        Some(&libraries),
    );

    assert_eq!(assessment.game_id, "zenless-zone-zero");
    assert_eq!(assessment.distribution, "steam");
    assert_eq!(
        assessment.game_support,
        IntegrationRequirementState::Satisfied
    );
    assert_eq!(assessment.steam_support, PlatformVerification::Unverified);
    assert_eq!(
        assessment.linux_proton_support,
        PlatformVerification::Unverified
    );
    assert!(!assessment.platform_compatibility_verified);
    assert!(assessment.planning_possible);
    assert_eq!(
        assessment.launch_compatibility,
        LaunchCompatibility::NotVerified
    );
}

#[test]
fn zzmi_plan_requires_libraries_and_never_targets_the_game_directory() {
    let tree = Tree::new();
    let store = tree.store();
    let zzmi = store.import_directory(&tree.zzmi()).unwrap();
    let libraries = store.import_directory(&tree.libs()).unwrap();
    let platform = tree.zzz_platform();
    let game_path = match &platform.installation {
        lxmi_runtime::GameInstallationAssessment::Detected { install_path, .. } => {
            install_path.clone()
        }
        _ => panic!("synthetic ZZZ installation should be detected"),
    };
    let before_game = snapshot(&game_path);
    let before_store = snapshot(store.root());

    let plan = plan_installation_for_integration(
        &store,
        &platform,
        IntegrationKind::Zzmi,
        &zzmi.manifest().id,
        Some(&libraries.manifest().id),
    )
    .unwrap();

    assert!(!plan.executable);
    assert!(plan.assessment.planning_possible);
    assert_eq!(
        plan.assessment.steam_support,
        PlatformVerification::Unverified
    );
    assert_eq!(
        plan.assessment.linux_proton_support,
        PlatformVerification::Unverified
    );
    assert_eq!(
        plan.game_executable_candidate,
        Some(game_path.join("games/ZenlessZoneZero Game/ZenlessZoneZero.exe"))
    );
    assert!(plan.managed_target.starts_with(store.root()));
    assert!(plan.files.iter().all(|file| {
        file.target.starts_with(store.root()) && !file.target.starts_with(&game_path)
    }));
    assert_eq!(snapshot(&game_path), before_game);
    assert_eq!(snapshot(store.root()), before_store);
}

#[test]
fn zzmi_plan_marks_the_separate_libraries_dependency_missing() {
    let tree = Tree::new();
    let store = tree.store();
    let zzmi = store.import_directory(&tree.zzmi()).unwrap();
    let assessment = assess_for_integration(
        &tree.zzz_platform(),
        IntegrationKind::Zzmi,
        Some(&zzmi),
        None,
    );

    assert!(!assessment.planning_possible);
    assert!(assessment.requirements.iter().any(|requirement| {
        requirement.name == "XXMIPackageAvailable"
            && requirement.state == IntegrationRequirementState::Unsatisfied
    }));
}
#[test]
fn package_kind_uses_the_frontend_ipc_shape() {
    assert_eq!(
        serde_json::to_value(PackageKind::GameIntegration(IntegrationKind::Wwmi)).unwrap(),
        serde_json::json!({ "game_integration": "wwmi" })
    );
    assert_eq!(
        serde_json::to_value(PackageKind::XxmiLibraries).unwrap(),
        serde_json::json!("xxmi_libraries")
    );
}
#[test]
fn missing_required_files_are_named() {
    let t = Tree::new();
    let p = t.wwmi();
    fs::remove_file(p.join("Core/WWMI/help.ini")).unwrap();
    let e = validate_directory(&p, ImportLimits::default()).unwrap_err();
    assert_eq!(e.code, ErrorCode::MissingRequiredFile);
    assert!(e.detail.contains("help.ini"));
}
#[test]
fn unknown_integration_is_rejected() {
    let t = Tree::new();
    let e = validate_directory(&t.0, ImportLimits::default()).unwrap_err();
    assert_eq!(e.code, ErrorCode::UnsupportedIntegration);
}
#[test]
fn unknown_namespace_is_not_assumed_wwmi() {
    let t = Tree::new();
    let p = t.wwmi();
    fs::write(
        p.join("Core/WWMI/WuWa-Model-Importer.ini"),
        "namespace=ZZMI",
    )
    .unwrap();
    assert_eq!(
        validate_directory(&p, ImportLimits::default())
            .unwrap_err()
            .code,
        ErrorCode::UnsupportedIntegration
    );
}
#[test]
fn libraries_are_separate_from_game_integration() {
    let t = Tree::new();
    let v = validate_directory(&t.libs(), ImportLimits::default()).unwrap();
    assert_eq!(v.kind, PackageKind::XxmiLibraries);
    assert_eq!(v.version.unwrap().raw, "nightly-example");
}
#[test]
fn random_dll_does_not_prove_runtime() {
    let t = Tree::new();
    fs::write(t.0.join("d3d11.dll"), "random").unwrap();
    assert_eq!(inspect_runtime(&t.0).libraries, Presence::Incomplete);
    assert!(validate_directory(&t.0, ImportLimits::default()).is_err());
}
#[test]
fn invalid_library_manifest_is_rejected() {
    let t = Tree::new();
    let p = t.libs();
    fs::write(p.join("Manifest.json"), "{}").unwrap();
    assert_eq!(
        validate_directory(&p, ImportLimits::default())
            .unwrap_err()
            .code,
        ErrorCode::InvalidMetadata
    );
}
#[test]
fn sha256_matches_known_vector() {
    let t = Tree::new();
    let p = t.wwmi();
    fs::write(p.join("example.txt"), "abc").unwrap();
    let v = validate_directory(&p, ImportLimits::default()).unwrap();
    assert_eq!(
        v.files
            .iter()
            .find(|f| f.relative_path == "example.txt")
            .unwrap()
            .sha256,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
#[test]
fn source_preserved_staging_promoted_and_manifest_is_external() {
    let t = Tree::new();
    let p = t.wwmi();
    let before = snapshot(&p);
    let s = t.store();
    let imported = s.import_directory(&p).unwrap();
    assert_eq!(snapshot(&p), before);
    assert!(!imported.payload_path().join("lxmi-package.json").exists());
    assert!(imported
        .payload_path()
        .parent()
        .unwrap()
        .join("lxmi-package.json")
        .exists());
    assert_eq!(fs::read_dir(s.root().join("staging")).unwrap().count(), 0);
    assert_eq!(s.list().unwrap().len(), 1);
}
#[test]
fn changed_file_checksum_mismatch() {
    let t = Tree::new();
    let s = t.store();
    let v = s.import_directory(&t.wwmi()).unwrap();
    fs::write(v.payload_path().join("Core/WWMI/help.ini"), "changed").unwrap();
    assert_eq!(
        s.verify(&v.manifest().id).unwrap_err().code,
        ErrorCode::ChecksumMismatch
    );
}
#[test]
fn tampered_manifest_rejected() {
    let t = Tree::new();
    let s = t.store();
    let v = s.import_directory(&t.wwmi()).unwrap();
    let p = v.payload_path().parent().unwrap().join("lxmi-package.json");
    let mut m: serde_json::Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
    m["version"]["raw"] = "forged".into();
    fs::write(p, serde_json::to_vec(&m).unwrap()).unwrap();
    assert_eq!(
        s.verify(&v.manifest().id).unwrap_err().code,
        ErrorCode::ChecksumMismatch
    );
}
#[test]
fn duplicate_package_is_reused_without_overwrite() {
    let t = Tree::new();
    let p = t.wwmi();
    let s = t.store();
    let a = s.import_directory(&p).unwrap();
    let before = snapshot(s.root());
    let b = s.import_directory(&p).unwrap();
    assert_eq!(a.manifest().id, b.manifest().id);
    assert_eq!(snapshot(s.root()), before);
}
#[test]
fn safe_relative_paths() {
    for p in ["d3dx.ini", "Core/WWMI/help.ini", "a space/file.txt"] {
        validate_relative_path(p).unwrap();
    }
}
#[test]
fn traversal_and_absolute_paths_rejected() {
    for p in [
        "../x",
        "a/../../x",
        "/etc/passwd",
        "C:\\windows\\file",
        "a\\..\\b",
        "a//b",
        "./a",
        "a/./b",
        "a/",
        "a\0b",
    ] {
        assert!(validate_relative_path(p).is_err(), "{p}");
    }
}
#[test]
fn unsafe_symlink_rejected() {
    let t = Tree::new();
    let p = t.wwmi();
    symlink("/etc/passwd", p.join("escape")).unwrap();
    assert_eq!(
        t.store().import_directory(&p).unwrap_err().code,
        ErrorCode::UnsafePath
    );
    assert!(!t.store().root().exists());
}
#[test]
fn symlinked_source_ancestor_rejected() {
    let t = Tree::new();
    let p = t.wwmi();
    symlink(&p, t.0.join("alias")).unwrap();
    assert_eq!(
        validate_directory(&t.0.join("alias"), ImportLimits::default())
            .unwrap_err()
            .code,
        ErrorCode::UnsafePath
    );
}
#[test]
fn hardlink_rejected() {
    let t = Tree::new();
    let p = t.wwmi();
    fs::write(t.0.join("outside"), "external").unwrap();
    fs::hard_link(t.0.join("outside"), p.join("hardlink")).unwrap();
    assert_eq!(
        validate_directory(&p, ImportLimits::default())
            .unwrap_err()
            .code,
        ErrorCode::UnsafePath
    );
}
#[test]
fn managed_root_symlink_rejected() {
    let t = Tree::new();
    let p = t.wwmi();
    let outside = t.0.join("outside");
    fs::create_dir(&outside).unwrap();
    symlink(&outside, t.store().root()).unwrap();
    assert_eq!(
        t.store().import_directory(&p).unwrap_err().code,
        ErrorCode::UnsafePath
    );
    assert_eq!(fs::read_dir(outside).unwrap().count(), 0);
}
#[test]
fn managed_storage_requires_private_permissions() {
    let t = Tree::new();
    let p = t.wwmi();
    fs::create_dir(t.store().root()).unwrap();
    fs::set_permissions(t.store().root(), fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(
        t.store().import_directory(&p).unwrap_err().code,
        ErrorCode::StorageUnavailable
    );
}
#[test]
fn source_storage_overlap_rejected() {
    let t = Tree::new();
    assert_eq!(
        t.store().import_directory(&t.0).unwrap_err().code,
        ErrorCode::UnsafePath
    );
}
#[test]
fn configured_limits_reject_large_import_before_storage_write() {
    let t = Tree::new();
    let p = t.wwmi();
    let s = ManagedStore::at(
        t.0.join("lxmi"),
        ImportLimits {
            max_file_bytes: 8,
            ..ImportLimits::default()
        },
    )
    .unwrap();
    assert_eq!(
        s.import_directory(&p).unwrap_err().code,
        ErrorCode::LimitExceeded
    );
    assert!(!s.root().exists());
}
#[test]
fn entry_and_total_limits_are_enforced() {
    let t = Tree::new();
    let p = t.wwmi();
    for limits in [
        ImportLimits {
            max_files: 2,
            ..ImportLimits::default()
        },
        ImportLimits {
            max_total_bytes: 32,
            ..ImportLimits::default()
        },
        ImportLimits {
            max_depth: 2,
            ..ImportLimits::default()
        },
    ] {
        assert_eq!(
            validate_directory(&p, limits).unwrap_err().code,
            ErrorCode::LimitExceeded
        );
    }
}
#[test]
fn archive_is_explicitly_unsupported() {
    let t = Tree::new();
    let p = t.0.join("package.zip");
    fs::write(&p, b"not extracted").unwrap();
    assert_eq!(
        t.store().import_directory(&p).unwrap_err().code,
        ErrorCode::UnsupportedArchive
    );
}
#[test]
fn case_collisions_are_rejected() {
    let t = Tree::new();
    let p = t.wwmi();
    fs::write(p.join("D3DX.INI"), "collision").unwrap();
    assert_eq!(
        validate_directory(&p, ImportLimits::default())
            .unwrap_err()
            .code,
        ErrorCode::UnsafePath
    );
}
#[test]
fn runtime_absent_is_normal() {
    let t = Tree::new();
    let r = inspect_runtime(&t.0.join("absent"));
    assert_eq!(r.integration, Presence::Absent);
    assert!(r.issues.is_empty());
}
#[test]
fn runtime_structurally_present_is_not_compatibility() {
    let t = Tree::new();
    let r = inspect_runtime(&t.wwmi());
    assert_eq!(r.integration, Presence::StructurallyPresent);
    assert_eq!(r.launch_compatibility, LaunchCompatibility::NotVerified);
}
#[test]
fn runtime_incomplete_has_evidence() {
    let t = Tree::new();
    let p = t.wwmi();
    fs::remove_file(p.join("Core/WWMI/help.ini")).unwrap();
    let r = inspect_runtime(&p);
    assert_eq!(r.integration, Presence::Incomplete);
    assert!(!r.issues.is_empty());
}
#[test]
fn discovery_never_writes() {
    let t = Tree::new();
    let p = t.wwmi();
    let before = snapshot(&t.0);
    inspect_runtime(&p);
    assert_eq!(snapshot(&t.0), before);
}
#[test]
fn validation_never_executes_package_content() {
    let t = Tree::new();
    let p = t.wwmi();
    fs::write(
        p.join("run.sh"),
        format!("#!/bin/sh\ntouch '{}/EXECUTED'\n", t.0.display()),
    )
    .unwrap();
    fs::set_permissions(p.join("run.sh"), fs::Permissions::from_mode(0o755)).unwrap();
    t.store().import_directory(&p).unwrap();
    assert!(!t.0.join("EXECUTED").exists());
}
#[test]
fn plan_is_reviewable_and_never_mutates_filesystem() {
    let t = Tree::new();
    let s = t.store();
    let wwmi = s.import_directory(&t.wwmi()).unwrap();
    let libs = s.import_directory(&t.libs()).unwrap();
    let platform = t.platform();
    let before = snapshot(&t.0);
    let plan = plan_installation(
        &s,
        &platform,
        &wwmi.manifest().id,
        Some(&libs.manifest().id),
    )
    .unwrap();
    assert!(!plan.executable);
    assert!(plan.assessment.planning_possible);
    assert!(!plan.files.is_empty());
    assert!(plan
        .files
        .iter()
        .all(|f| f.target.starts_with(s.root()) && !f.backup_required));
    assert_eq!(snapshot(&t.0), before);
    assert_eq!(
        plan.assessment.launch_compatibility,
        LaunchCompatibility::NotVerified
    );
}
#[test]
fn single_package_does_not_imply_compatibility() {
    let t = Tree::new();
    let v = t.store().import_directory(&t.wwmi()).unwrap();
    let a = assess(&t.platform(), Some(&v), None);
    assert_eq!(a.launch_compatibility, LaunchCompatibility::NotVerified);
    assert!(a
        .requirements
        .iter()
        .any(|r| r.name == "XXMIPackageAvailable"
            && r.state == IntegrationRequirementState::Unsatisfied));
}
#[test]
fn wrong_game_integration_is_rejected() {
    let t = Tree::new();
    let v = t.store().import_directory(&t.wwmi()).unwrap();
    let mut p = t.platform();
    p.game = Game {
        id: "other",
        name: "other",
        steam_app_ids: &[42],
        expected_executable_names: &[],
        compatibility_runtime_policy: CompatibilityRuntimePolicy::Unknown,
    };
    assert_eq!(
        plan_installation(&t.store(), &p, &v.manifest().id, None)
            .unwrap_err()
            .code,
        ErrorCode::WrongGameIntegration
    );
}
#[test]
fn plan_records_replacement_and_required_backup() {
    let t = Tree::new();
    let s = t.store();
    let v = s.import_directory(&t.wwmi()).unwrap();
    fs::create_dir_all(s.root().join("runtimes/wwmi")).unwrap();
    fs::write(s.root().join("runtimes/wwmi/d3dx.ini"), "older").unwrap();
    let p = plan_installation(&s, &t.platform(), &v.manifest().id, None).unwrap();
    let f = p
        .files
        .iter()
        .find(|f| f.target.ends_with("d3dx.ini"))
        .unwrap();
    assert_eq!(f.action, FileAction::ReplaceWithBackup);
    assert!(f.backup_required);
    assert!(f.previous_sha256.is_some());
}
#[test]
fn plan_rejects_symlink_destination() {
    let t = Tree::new();
    let s = t.store();
    let v = s.import_directory(&t.wwmi()).unwrap();
    fs::create_dir_all(s.root().join("runtimes")).unwrap();
    symlink(t.0.join("game"), s.root().join("runtimes/wwmi")).unwrap();
    assert_eq!(
        plan_installation(&s, &t.platform(), &v.manifest().id, None)
            .unwrap_err()
            .code,
        ErrorCode::UnsafePath
    );
}
#[test]
fn listing_and_constructor_do_not_create_storage() {
    let t = Tree::new();
    let s = t.store();
    assert!(s.list().unwrap().is_empty());
    assert!(!s.root().exists());
}
#[test]
fn forged_package_id_rejected() {
    let t = Tree::new();
    assert_eq!(
        t.store().verify("../../outside").unwrap_err().code,
        ErrorCode::UnsafePath
    );
}
#[test]
fn protected_game_storage_overlap_rejected() {
    let t = Tree::new();
    assert_eq!(
        t.store()
            .ensure_outside(std::slice::from_ref(&t.0))
            .unwrap_err()
            .code,
        ErrorCode::UnsafePath
    );
}
#[test]
fn xdg_storage_path_is_respected() {
    let t = Tree::new();
    let mut info = SystemInfo::current();
    info.xdg_data_home = Some(t.0.clone());
    assert_eq!(
        ManagedStore::from_system(&info).unwrap().root(),
        t.0.join("lxmi")
    );
}
#[test]
fn invalid_import_leaves_no_valid_package_or_stage() {
    let t = Tree::new();
    let p = t.wwmi();
    fs::remove_file(p.join("d3dx.ini")).unwrap();
    assert!(t.store().import_directory(&p).is_err());
    assert!(t.store().list().unwrap().is_empty());
    assert!(!t.store().root().exists());
}

#[test]
fn unreadable_package_is_not_reported_as_absent() {
    let t = Tree::new();
    let p = t.wwmi();
    fs::set_permissions(p.join("d3dx.ini"), fs::Permissions::from_mode(0o000)).unwrap();
    let result = inspect_runtime(&p);
    assert!(result
        .issues
        .iter()
        .any(|e| e.code == ErrorCode::PermissionDenied));
    assert_ne!(result.integration, Presence::Absent);
}
#[test]
fn empty_version_is_unknown() {
    let t = Tree::new();
    let p = t.wwmi();
    fs::write(
        p.join("Core/WWMI/WuWa-Model-Importer.ini"),
        "namespace=WWMIv1\n",
    )
    .unwrap();
    assert!(validate_directory(&p, ImportLimits::default())
        .unwrap()
        .version
        .is_none());
}
#[test]
fn library_presence_does_not_authenticate_signature() {
    let t = Tree::new();
    let r = inspect_runtime(&t.libs());
    assert_eq!(r.libraries, Presence::StructurallyPresent);
    assert_eq!(r.launch_compatibility, LaunchCompatibility::NotVerified);
}
#[test]
fn plan_rechecks_managed_bytes_and_rejects_changed_payload() {
    let t = Tree::new();
    let s = t.store();
    let p = s.import_directory(&t.wwmi()).unwrap();
    fs::write(
        p.payload_path().join("Core/WWMI/help.ini"),
        "changed after import",
    )
    .unwrap();
    assert_eq!(
        plan_installation(&s, &t.platform(), &p.manifest().id, None)
            .unwrap_err()
            .code,
        ErrorCode::ChecksumMismatch
    );
}
#[test]
fn unavailable_game_keeps_plan_blocked() {
    let t = Tree::new();
    let s = t.store();
    let p = s.import_directory(&t.wwmi()).unwrap();
    let mut platform = t.platform();
    platform.installation = lxmi_runtime::GameInstallationAssessment::NotFound {
        expected_steam_app_ids: vec![3513350],
    };
    let plan = plan_installation(&s, &platform, &p.manifest().id, None).unwrap();
    assert!(!plan.assessment.planning_possible);
    assert!(plan.game_executable_candidate.is_none());
}
