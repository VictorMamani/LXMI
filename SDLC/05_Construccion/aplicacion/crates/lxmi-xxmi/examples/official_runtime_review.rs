//! Opt-in download/authentication/assembly under an explicit private test store.
//! No game, Wine, Proton, helper or loader is executed.
use lxmi_xxmi::*;
use std::{fs, path::PathBuf};
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    if std::env::var("LXMI_CONFIRM_OFFICIAL_DOWNLOAD_AND_STORE").as_deref() != Ok("YES") {
        return Err(
            "Set LXMI_CONFIRM_OFFICIAL_DOWNLOAD_AND_STORE=YES for this opt-in review".into(),
        );
    }
    let args: Vec<_> = std::env::args().collect();
    let root = PathBuf::from(args.get(1).ok_or("Expected explicit .../lxmi root")?);
    let game = args.get(2).map(String::as_str).unwrap_or("wuthering-waves");
    let kind = match game {
        "wuthering-waves" => OfficialPackageKind::Wwmi,
        "zenless-zone-zero" => OfficialPackageKind::Zzmi,
        _ => return Err("Unknown game".into()),
    };
    let store = ManagedStore::at(root.clone(), ImportLimits::default())?;
    let cache_base = root.parent().ok_or("Missing parent")?.join("cache");
    fs::create_dir_all(&cache_base)?;
    let cache = DownloadCache::at(cache_base.join("lxmi/downloads"))?;
    let provider = GitHubReleaseProvider::new()?;
    let importer_release = provider.latest_release(kind.clone())?;
    let libraries_release = provider.latest_release(OfficialPackageKind::XxmiLibraries)?;
    let importer =
        download_and_import_official(&store, &cache, &provider, kind, &importer_release.tag)?;
    let libraries = download_and_import_official(
        &store,
        &cache,
        &provider,
        OfficialPackageKind::XxmiLibraries,
        &libraries_release.tag,
    )?;
    let discovery = lxmi_steam::SteamDiscoveryScanner::from_environment().scan();
    let tools = lxmi_proton::ProtonScanner.scan(&discovery.steam.installations);
    let platform = lxmi_runtime::RuntimePlanner::plan_all(&discovery, &tools)
        .into_iter()
        .find(|p| p.game.id == game)
        .ok_or("No game plan")?;
    let mut protected = Vec::new();
    for installation in &discovery.steam.installations {
        protected.push(installation.root_path.clone());
        protected.extend(installation.libraries.iter().map(|l| l.path.clone()));
    }
    let runtime =
        assemble_importer_runtime(&store, &platform, &importer.id, &libraries.id, &protected)?;
    let report = serde_json::json!({"game":game,"importer":importer,"libraries":libraries,"runtime":runtime,"game_executed":false,"external_files_modified":false});
    fs::write(
        root.parent().unwrap().join("official-runtime-review.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!(
        "{}",
        serde_json::json!({"game":game,"importer_tag":importer_release.tag,"libraries_tag":libraries_release.tag,"runtime_id":runtime.runtime_id,"files":runtime.file_count,"game_executed":false})
    );
    Ok(())
}
