//! Read-only game inspection. Optional explicit imports write only the supplied LXMI root.
//! cargo run -p lxmi-xxmi --example local_review -- /tmp/lxmi /path/WWMI [/path/XXMI-libs]
use lxmi_runtime::{GameInstallationAssessment, RuntimePlanner};
use lxmi_xxmi::*;
use std::path::PathBuf;
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    let discovery = lxmi_steam::SteamDiscoveryScanner::from_environment().scan();
    let tools = lxmi_proton::ProtonScanner.scan(&discovery.steam.installations);
    let plans = RuntimePlanner::plan_all(&discovery, &tools);
    let store = if let Some(root) = args.first() {
        ManagedStore::at(PathBuf::from(root), ImportLimits::default())?
    } else {
        ManagedStore::from_system(&lxmi_core::SystemInfo::current())?
    };
    let mut protected = Vec::new();
    for steam in &discovery.steam.installations {
        protected.push(steam.root_path.clone());
        protected.extend(steam.libraries.iter().map(|l| l.path.clone()));
    }
    store.ensure_outside(&protected)?;
    let wwmi = args
        .get(1)
        .map(|path| store.import_directory(&PathBuf::from(path)))
        .transpose()?;
    let libs = args
        .get(2)
        .map(|path| store.import_directory(&PathBuf::from(path)))
        .transpose()?;
    for platform in &plans {
        println!(
            "Game: {}\nSelection: {:?}\nPrefix: {:?}\nReadiness: {:?}",
            platform.game.name, platform.selection, platform.prefix, platform.readiness
        );
        if let GameInstallationAssessment::Detected { install_path, .. } = &platform.installation {
            for path in [
                install_path.clone(),
                install_path.join("WWMI"),
                install_path.join("Client/Binaries/Win64"),
            ] {
                println!("{}", serde_json::to_string_pretty(&inspect_runtime(&path))?);
            }
        }
        if let Some(wwmi) = &wwmi {
            let plan = plan_installation(
                &store,
                platform,
                &wwmi.manifest().id,
                libs.as_ref().map(|p| p.manifest().id.as_str()),
            )?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
    }
    Ok(())
}
