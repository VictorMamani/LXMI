use lxmi_steam::SteamDiscoveryScanner;
use lxmi_xxmi::{
    download_and_import_official, DownloadCache, GitHubReleaseProvider, ImportLimits, ManagedStore,
    OfficialPackageKind, PackageAuthenticity, PackageKind, ReleaseProvider,
};
use std::{
    env, fs,
    time::{SystemTime, UNIX_EPOCH},
};

/// This test is deliberately ignored: run only to exercise real GitHub HTTP downloads and
/// explicitly promote the selected official packages into an isolated temporary LXMI store.
#[test]
#[ignore = "explicit network test; set LXMI_CONFIRM_OFFICIAL_DOWNLOAD_AND_STORE=YES"]
fn downloads_verifies_and_imports_pinned_official_assets_through_lxmi_pipeline() {
    assert_eq!(
        env::var("LXMI_CONFIRM_OFFICIAL_DOWNLOAD_AND_STORE").as_deref(),
        Ok("YES"),
        "explicitly set LXMI_CONFIRM_OFFICIAL_DOWNLOAD_AND_STORE=YES"
    );
    let tick = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let test_root =
        env::temp_dir().join(format!("lxmi-upstream-http-{}-{tick}", std::process::id()));
    fs::create_dir_all(test_root.join("cache")).unwrap();
    fs::create_dir_all(test_root.join("store")).unwrap();
    let store_root = test_root.join("store/lxmi");
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let store = ManagedStore::at(store_root, ImportLimits::default()).unwrap();
    let mut protected = Vec::new();
    for installation in &discovery.steam.installations {
        protected.push(installation.root_path.clone());
        protected.extend(
            installation
                .libraries
                .iter()
                .map(|library| library.path.clone()),
        );
    }
    for game in &discovery.games.games {
        protected.push(game.installation.install_path.clone());
        protected.push(game.compatdata.compatdata_path.clone());
    }
    store.ensure_outside(&protected).unwrap();

    let provider = GitHubReleaseProvider::new().unwrap();
    let cache = DownloadCache::at(test_root.join("cache/lxmi/downloads")).unwrap();
    for (kind, tag, release_id, commit) in [
        (
            OfficialPackageKind::Zzmi,
            "v1.5.0",
            393_483_881,
            "e59f87047cd405c5db5476d3b1b499574bc43d67",
        ),
        (
            OfficialPackageKind::XxmiLibraries,
            "v1.1.7",
            387_957_029,
            "6bf6a746a198c82fb34dbd336cac7ad8e0f4ddc9",
        ),
    ] {
        let release = provider.release_by_tag(kind.clone(), tag).unwrap();
        assert_eq!(release.release_id, release_id);
        assert_eq!(release.commit, commit);
        let asset_name = match kind {
            OfficialPackageKind::Zzmi => "ZZMI-PACKAGE-v1.5.0.zip",
            OfficialPackageKind::XxmiLibraries => "XXMI-PACKAGE-v1.1.7.zip",
        };
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .unwrap();
        let package =
            download_and_import_official(&store, &cache, &provider, kind.clone(), tag).unwrap();
        assert_eq!(
            package.authenticity,
            PackageAuthenticity::OfficialReleaseVerified
        );
        assert_eq!(package.upstream.as_ref().unwrap().release_id, release_id);
        assert_eq!(package.upstream.as_ref().unwrap().tag, tag);
        if kind == OfficialPackageKind::XxmiLibraries {
            assert!(
                package
                    .upstream
                    .as_ref()
                    .unwrap()
                    .component_signatures_verified
            );
            assert_eq!(package.kind, PackageKind::XxmiLibraries);
        } else {
            assert_eq!(
                package.kind,
                PackageKind::GameIntegration(lxmi_xxmi::IntegrationKind::Zzmi)
            );
        }
        println!(
            "REAL_GITHUB_DOWNLOAD repository={} tag={} release_id={} commit={} published_at={} metadata_retrieved_at={} asset_id={} asset={} size={} sha256={} authenticity={:?} components={}",
            package.upstream.as_ref().unwrap().repository,
            package.upstream.as_ref().unwrap().tag,
            package.upstream.as_ref().unwrap().release_id,
            package.upstream.as_ref().unwrap().commit,
            release.published_at,
            release.metadata_retrieved_at,
            asset.id,
            asset.name,
            asset.size,
            asset.sha256.as_deref().unwrap_or("missing"),
            package.authenticity,
            package.upstream.as_ref().unwrap().component_signatures_verified,
        );
    }
    let inventory = store.list().unwrap();
    assert!(inventory.iter().any(|package| {
        package.kind == PackageKind::GameIntegration(lxmi_xxmi::IntegrationKind::Zzmi)
            && package.authenticity == PackageAuthenticity::OfficialReleaseVerified
    }));
    assert!(inventory.iter().any(|package| {
        package.kind == PackageKind::XxmiLibraries
            && package.authenticity == PackageAuthenticity::OfficialReleaseVerified
    }));
    let cache_root = test_root.join("cache/lxmi/downloads");
    let cached_assets = fs::read_dir(&cache_root)
        .unwrap()
        .flat_map(|release| fs::read_dir(release.unwrap().path()).unwrap())
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        cached_assets.len(),
        3,
        "two ZIPs and the Libraries manifest"
    );
    assert!(cached_assets.iter().all(|name| !name.ends_with(".partial")));
    println!("REAL_HTTP_CACHE assets={cached_assets:?}");
    fs::remove_dir_all(test_root).unwrap();
}
