//! Opt-in import/review of local mods using the same domain API as the desktop.
//! `apply` selects a private runtime; neither mode starts a game or writes a prefix.
use lxmi_xxmi::*;
use std::path::PathBuf;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    if std::env::var("LXMI_CONFIRM_LOCAL_PROFILE").as_deref() != Ok("YES") {
        return Err("Set LXMI_CONFIRM_LOCAL_PROFILE=YES to import local copies".into());
    }
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 8 || (args[1] != "prepare" && args[1] != "apply") {
        return Err("Usage: local_profile_review prepare|apply /absolute/lxmi game importer-id libraries-id profile-name source...".into());
    }
    let game = &args[3];
    let store = ManagedStore::at(PathBuf::from(&args[2]), ImportLimits::default())?;
    let discovery = lxmi_steam::SteamDiscoveryScanner::from_environment().scan();
    let tools = lxmi_proton::ProtonScanner.scan(&discovery.steam.installations);
    let platform = lxmi_runtime::RuntimePlanner::plan_all(&discovery, &tools)
        .into_iter()
        .find(|p| p.game.id == *game)
        .ok_or("No game plan")?;
    let mut protected = Vec::new();
    for installation in &discovery.steam.installations {
        protected.push(installation.root_path.clone());
        protected.extend(installation.libraries.iter().map(|l| l.path.clone()));
    }
    store.ensure_outside(&protected)?;
    let base = assemble_importer_runtime(&store, &platform, &args[4], &args[5], &protected)?;
    let library = ModLibrary::new(store.clone());
    let before = library.list(game)?;
    if !before.issues.is_empty() {
        return Err("Library has issues; inspect them in the desktop before changing it".into());
    }
    let mut mods = Vec::new();
    for source in &args[7..] {
        let source = PathBuf::from(source);
        let name = source
            .file_stem()
            .and_then(|n| n.to_str())
            .ok_or("Invalid source name")?;
        mods.push(library.import(game, name, &source)?);
    }
    let mut ids: Vec<_> = mods.iter().map(|m| m.id.clone()).collect();
    ids.sort();
    ids.dedup();
    let current = library.list(game)?;
    let profile_name = &args[6];
    let saved = match current
        .profiles
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case(profile_name))
    {
        Some(p) if p.mod_ids == ids => current,
        Some(_) => {
            return Err("This profile already has a different selection; choose a new name".into())
        }
        None => library.save_profile(game, profile_name, &ids, current.revision)?,
    };
    let selection_before = profile_activation_status(&store, game)?;
    let plan = review_profile_activation(
        &store,
        &base,
        profile_name,
        saved.revision,
        selection_before.revision,
    )?;
    let selection_after = if args[1] == "apply" {
        activate_profile(
            &store,
            &base,
            profile_name,
            saved.revision,
            selection_before.revision,
            &plan.plan_id,
        )?
    } else {
        selection_before.clone()
    };
    let selected = resolve_profile_runtime(&store, &base)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "mode": args[1], "game_id": game, "mods": mods, "base_runtime": base,
            "plan": plan, "selection_before": selection_before, "selection_after": selection_after,
            "resolved_runtime": selected, "game_executed": false, "prefix_modified": false,
            "originals_modified": false, "ini_compatibility_verified": false
        }))?
    );
    Ok(())
}
