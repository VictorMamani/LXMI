use lxmi_proton::ProtonScanner;
use lxmi_runtime::{GameInstallationAssessment, RuntimePlanner};
use lxmi_steam::SteamDiscoveryScanner;
use lxmi_xxmi::{
    parse_github_release, plan_installation_for_integration, verify_release_file, ImportLimits,
    IntegrationKind, ManagedStore, OfficialPackageKind, PackageAuthenticity, PackageKind,
    SignatureStatus,
};
use serde_json::Value;
use std::{
    env, fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn required_path(name: &str) -> PathBuf {
    PathBuf::from(
        env::var_os(name).unwrap_or_else(|| panic!("set {name} to the downloaded official asset")),
    )
}

fn same_official_package(
    store: &ManagedStore,
    expected_kind: PackageKind,
    tag: &str,
) -> Option<lxmi_xxmi::VerifiedPackage> {
    store
        .list()
        .unwrap()
        .into_iter()
        .find(|manifest| {
            manifest.kind == expected_kind
                && manifest
                    .upstream
                    .as_ref()
                    .is_some_and(|upstream| upstream.tag == tag)
        })
        .map(|manifest| store.verify(&manifest.id).unwrap())
}

fn import_or_verify(
    store: &ManagedStore,
    release: &lxmi_xxmi::UpstreamRelease,
    archive: &std::path::Path,
    sidecar: Option<(&lxmi_xxmi::ReleaseAsset, &std::path::Path)>,
    package_kind: PackageKind,
) -> lxmi_xxmi::VerifiedPackage {
    if let Some(existing) = same_official_package(store, package_kind, &release.tag) {
        assert_eq!(
            existing.manifest().authenticity,
            PackageAuthenticity::OfficialReleaseVerified
        );
        existing
    } else {
        store
            .import_official_archive(release, archive, sidecar)
            .unwrap()
    }
}

fn file_state(path: &std::path::Path) -> String {
    use sha2::{Digest, Sha256};
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "missing".into(),
        Err(error) => format!("error:{:?}", error.kind()),
        Ok(metadata) if metadata.file_type().is_symlink() => "symlink".into(),
        Ok(metadata) if metadata.is_dir() => "directory".into(),
        Ok(metadata) if metadata.is_file() => match fs::read(path) {
            Ok(bytes) => format!("file:{}:{:x}", metadata.len(), Sha256::digest(bytes)),
            Err(error) => format!("read-error:{:?}", error.kind()),
        },
        Ok(_) => "special".into(),
    }
}

fn fixture_release(
    metadata_path: &PathBuf,
    kind: OfficialPackageKind,
    commit: &str,
) -> lxmi_xxmi::UpstreamRelease {
    let bytes = fs::read(metadata_path).unwrap();
    let metadata: Value = serde_json::from_slice(&bytes).unwrap();
    let body = metadata
        .get("body")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let value = serde_json::json!({
        "id": metadata["id"],
        "tag_name": metadata["tag_name"],
        "draft": metadata["draft"],
        "prerelease": metadata["prerelease"],
        "html_url": metadata["html_url"],
        "published_at": metadata["published_at"],
        "body": body,
        "assets": metadata["assets"],
    });
    parse_github_release(
        &kind,
        &serde_json::to_vec(&value).unwrap(),
        commit,
        1_790_590_156,
    )
    .unwrap()
}

/// Run explicitly with verified, temporarily downloaded upstream assets; normal tests stay offline.
#[test]
#[ignore = "requires official release metadata/assets downloaded to a temporary directory"]
fn cryptographically_verifies_and_imports_pinned_real_packages_offline() {
    let zzmi_metadata = required_path("LXMI_ZZMI_RELEASE_JSON");
    let libraries_metadata = required_path("LXMI_LIBRARIES_RELEASE_JSON");
    let zzmi_zip = required_path("LXMI_ZZMI_ZIP");
    let libraries_zip = required_path("LXMI_LIBRARIES_ZIP");
    let libraries_manifest = required_path("LXMI_LIBRARIES_MANIFEST_JSON");
    let zzmi_commit = env::var("LXMI_ZZMI_COMMIT").expect("set LXMI_ZZMI_COMMIT");
    let libraries_commit = env::var("LXMI_LIBRARIES_COMMIT").expect("set LXMI_LIBRARIES_COMMIT");
    let zzmi_release = fixture_release(&zzmi_metadata, OfficialPackageKind::Zzmi, &zzmi_commit);
    let libraries_release = fixture_release(
        &libraries_metadata,
        OfficialPackageKind::XxmiLibraries,
        &libraries_commit,
    );
    assert_eq!(
        verify_release_file(
            &OfficialPackageKind::Zzmi,
            zzmi_release.signature_base64.as_deref(),
            &zzmi_zip
        ),
        SignatureStatus::Verified
    );
    assert_eq!(
        verify_release_file(
            &OfficialPackageKind::XxmiLibraries,
            libraries_release.signature_base64.as_deref(),
            &libraries_zip
        ),
        SignatureStatus::Verified
    );

    let external_store_root = env::var_os("LXMI_UPSTREAM_STORE_ROOT").map(PathBuf::from);
    let temp = external_store_root.is_none().then(|| {
        let tick = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("lxmi-upstream-real-{}-{tick}", std::process::id()))
    });
    let store_root = external_store_root
        .clone()
        .or_else(|| temp.as_ref().map(|path| path.join("lxmi")))
        .unwrap();
    if let Some(temp) = &temp {
        fs::create_dir_all(temp).unwrap();
    }
    let store = ManagedStore::at(store_root, ImportLimits::default()).unwrap();

    // This real-host mode checks the store boundary before it imports any bytes. It is optional
    // so the normal ignored test remains useful on machines without Steam/ZZZ.
    let real_platform = if let Some(expected_executable) = env::var_os("LXMI_REAL_ZZZ_EXECUTABLE") {
        let discovery = SteamDiscoveryScanner::from_environment().scan();
        let tools = ProtonScanner.scan(&discovery.steam.installations);
        let plans = RuntimePlanner::plan_all(&discovery, &tools);
        let platform = plans
            .iter()
            .find(|plan| plan.game.id == "zenless-zone-zero")
            .expect("local Steam discovery should include ZZZ")
            .clone();
        match &platform.installation {
            GameInstallationAssessment::Detected {
                executable_path: Some(path),
                install_path,
                steam_library,
                ..
            } => {
                assert_eq!(path, &PathBuf::from(expected_executable));
                let mut protected = vec![install_path.clone(), steam_library.clone()];
                for steam in &discovery.steam.installations {
                    protected.push(steam.root_path.clone());
                    protected.extend(steam.libraries.iter().map(|library| library.path.clone()));
                }
                for game in &discovery.games.games {
                    protected.push(game.installation.install_path.clone());
                    protected.push(game.compatdata.compatdata_path.clone());
                }
                store.ensure_outside(&protected).unwrap();
                Some(platform)
            }
            other => panic!("expected detected local ZZZ executable, got {other:?}"),
        }
    } else {
        None
    };

    let zzmi = import_or_verify(
        &store,
        &zzmi_release,
        &zzmi_zip,
        None,
        PackageKind::GameIntegration(IntegrationKind::Zzmi),
    );
    let sidecar = libraries_release
        .assets
        .iter()
        .find(|asset| asset.name == "Manifest.json")
        .unwrap();
    let libraries = import_or_verify(
        &store,
        &libraries_release,
        &libraries_zip,
        Some((sidecar, libraries_manifest.as_path())),
        PackageKind::XxmiLibraries,
    );
    assert_eq!(
        zzmi.manifest().authenticity,
        PackageAuthenticity::OfficialReleaseVerified
    );
    assert_eq!(
        libraries.manifest().authenticity,
        PackageAuthenticity::OfficialReleaseVerified
    );
    assert!(
        libraries
            .manifest()
            .upstream
            .as_ref()
            .unwrap()
            .component_signatures_verified
    );
    assert!(store
        .list()
        .unwrap()
        .iter()
        .any(|package| package.id == zzmi.manifest().id));
    assert!(store
        .list()
        .unwrap()
        .iter()
        .any(|package| package.id == libraries.manifest().id));

    if let Some(platform) = real_platform {
        let mappings_before = plan_installation_for_integration(
            &store,
            &platform,
            IntegrationKind::Zzmi,
            &zzmi.manifest().id,
            Some(&libraries.manifest().id),
        )
        .unwrap();
        let candidate = mappings_before
            .dry_run
            .comparison_root_candidate
            .clone()
            .expect("real game executable should produce a candidate root");
        let before = mappings_before
            .dry_run
            .files
            .iter()
            .filter_map(|file| {
                file.comparison_path
                    .as_ref()
                    .map(|path| (path.clone(), file_state(path)))
            })
            .collect::<Vec<_>>();
        assert!(!mappings_before.executable);
        assert!(!mappings_before.dry_run.apply_allowed);
        assert!(!mappings_before.dry_run.writes_performed);
        assert!(mappings_before.dry_run.configured_target_root.is_none());
        assert_eq!(
            mappings_before.dry_run.comparison_root_candidate,
            Some(candidate.clone())
        );
        let after = before
            .iter()
            .map(|(path, _)| (path.clone(), file_state(path)))
            .collect::<Vec<_>>();
        assert_eq!(
            before, after,
            "dry-run must leave every candidate unchanged"
        );
        println!(
            "REAL_DRY_RUN zzz_executable={} packages=2 mappings={} comparison_root={} statuses={:?} apply_allowed={} writes_performed={}",
            env::var("LXMI_REAL_ZZZ_EXECUTABLE").unwrap(),
            mappings_before.dry_run.files.len(),
            candidate.display(),
            mappings_before
                .dry_run
                .files
                .iter()
                .map(|file| file.status)
                .fold(std::collections::BTreeMap::new(), |mut counts, status| {
                    *counts.entry(format!("{status:?}")).or_insert(0usize) += 1;
                    counts
                }),
            mappings_before.dry_run.apply_allowed,
            mappings_before.dry_run.writes_performed,
        );
    }

    if let Some(temp) = temp {
        fs::remove_dir_all(temp).unwrap();
    }
}

/// Rechecks the exact package selected by LXMI against the real local ZZZ install without writes.
#[test]
#[ignore = "requires LXMI_CONFIRM_REAL_ZZZ_DRY_RUN=YES, verified packages in managed storage, Steam and ZZZ"]
fn dry_runs_current_official_zzmi_and_libraries_against_local_zzz_read_only() {
    assert_eq!(
        env::var("LXMI_CONFIRM_REAL_ZZZ_DRY_RUN").as_deref(),
        Ok("YES"),
        "explicitly set LXMI_CONFIRM_REAL_ZZZ_DRY_RUN=YES"
    );
    let store_root = required_path("LXMI_UPSTREAM_STORE_ROOT");
    let expected_executable = required_path("LXMI_REAL_ZZZ_EXECUTABLE");
    let store = ManagedStore::at(store_root, ImportLimits::default()).unwrap();
    let packages = store.list().unwrap();
    let zzmi_manifest = packages
        .iter()
        .find(|package| {
            package.kind == PackageKind::GameIntegration(IntegrationKind::Zzmi)
                && package
                    .upstream
                    .as_ref()
                    .is_some_and(|upstream| upstream.tag == "v1.5.0")
        })
        .expect("current selected ZZMI v1.5.0 must be managed");
    let libraries_manifest = packages
        .iter()
        .find(|package| {
            package.kind == PackageKind::XxmiLibraries
                && package
                    .upstream
                    .as_ref()
                    .is_some_and(|upstream| upstream.tag == "v1.1.7")
        })
        .expect("selected XXMI Libraries v1.1.7 must be managed");
    let zzmi = store.verify(&zzmi_manifest.id).unwrap();
    let libraries = store.verify(&libraries_manifest.id).unwrap();
    assert_eq!(
        zzmi.manifest().authenticity,
        PackageAuthenticity::OfficialReleaseVerified
    );
    assert_eq!(
        libraries.manifest().authenticity,
        PackageAuthenticity::OfficialReleaseVerified
    );

    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let tools = ProtonScanner.scan(&discovery.steam.installations);
    let plans = RuntimePlanner::plan_all(&discovery, &tools);
    let platform = plans
        .iter()
        .find(|plan| plan.game.id == "zenless-zone-zero")
        .expect("local Steam discovery should include ZZZ");
    let executable = match &platform.installation {
        GameInstallationAssessment::Detected {
            executable_path: Some(executable_path),
            install_path,
            steam_library,
            ..
        } => {
            assert_eq!(executable_path, &expected_executable);
            let mut protected = vec![install_path.clone(), steam_library.clone()];
            for steam in &discovery.steam.installations {
                protected.push(steam.root_path.clone());
                protected.extend(steam.libraries.iter().map(|library| library.path.clone()));
            }
            for game in &discovery.games.games {
                protected.push(game.installation.install_path.clone());
                protected.push(game.compatdata.compatdata_path.clone());
            }
            store.ensure_outside(&protected).unwrap();
            executable_path.clone()
        }
        other => panic!("expected detected local ZZZ executable, got {other:?}"),
    };

    let plan = plan_installation_for_integration(
        &store,
        platform,
        IntegrationKind::Zzmi,
        &zzmi.manifest().id,
        Some(&libraries.manifest().id),
    )
    .unwrap();
    let comparison_root = plan
        .dry_run
        .comparison_root_candidate
        .as_ref()
        .expect("local executable should provide a comparison candidate");
    assert_eq!(comparison_root, executable.parent().unwrap());
    assert!(!plan.executable);
    assert!(!plan.dry_run.apply_allowed);
    assert!(!plan.dry_run.writes_performed);
    assert!(plan.dry_run.configured_target_root.is_none());

    let before = plan
        .dry_run
        .files
        .iter()
        .filter_map(|file| {
            file.comparison_path
                .as_ref()
                .map(|path| (path.clone(), file_state(path)))
        })
        .collect::<Vec<_>>();
    let after = before
        .iter()
        .map(|(path, _)| (path.clone(), file_state(path)))
        .collect::<Vec<_>>();
    assert_eq!(before, after, "dry-run must not mutate ZZZ candidate paths");
    let status_counts = plan.dry_run.files.iter().map(|file| file.status).fold(
        std::collections::BTreeMap::new(),
        |mut counts, status| {
            *counts.entry(format!("{status:?}")).or_insert(0usize) += 1;
            counts
        },
    );
    println!(
        "CURRENT_REAL_DRY_RUN game={} executable={} zzmi={} libraries={} mappings={} comparison_root={} statuses={status_counts:?} apply_allowed={} writes_performed={}",
        platform.game.name,
        executable.display(),
        zzmi.manifest().upstream.as_ref().unwrap().tag,
        libraries.manifest().upstream.as_ref().unwrap().tag,
        plan.dry_run.files.len(),
        comparison_root.display(),
        plan.dry_run.apply_allowed,
        plan.dry_run.writes_performed,
    );
}
