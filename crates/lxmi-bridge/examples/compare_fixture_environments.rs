//! Opt-in diagnostic: harmless bridge helper only, fresh /tmp LXMI prefix, no game entry.
use lxmi_bridge::{
    diagnose_registry_change, prefix_file_identity, run_fixture_environment_step, BridgeTestConfig,
    ExplicitBridgeRuntime, FixtureLinuxEnvironment,
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read},
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

fn read(path: &Path, limit: u64) -> io::Result<Vec<u8>> {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NOATIME)
        .open(path)?;
    let m = file.metadata()?;
    if !m.is_file() || m.uid() != unsafe { libc::getuid() } || m.len() > limit {
        return Err(io::Error::other("unsafe_or_oversized_file"));
    }
    let mut bytes = Vec::new();
    file.by_ref().take(limit + 1).read_to_end(&mut bytes)?;
    let after = fs::symlink_metadata(path)?;
    if bytes.len() as u64 > limit
        || (m.dev(), m.ino(), m.len(), m.mtime(), m.mtime_nsec())
            != (
                after.dev(),
                after.ino(),
                after.len(),
                after.mtime(),
                after.mtime_nsec(),
            )
    {
        return Err(io::Error::other("file_changed_during_read"));
    }
    Ok(bytes)
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn registries(prefix: &Path) -> io::Result<BTreeMap<String, Vec<u8>>> {
    ["system.reg", "user.reg", "userdef.reg"]
        .into_iter()
        .map(|name| Ok((name.into(), read(&prefix.join(name), 8 * 1024 * 1024)?)))
        .collect()
}
fn settled(prefix: &Path) -> io::Result<BTreeMap<String, Vec<u8>>> {
    // Bounded natural settle, no wineserver command or signals. Require two stable samples.
    thread::sleep(Duration::from_secs(5));
    for _ in 0..5 {
        let a = registries(prefix)?;
        thread::sleep(Duration::from_secs(2));
        let b = registries(prefix)?;
        if a == b {
            return Ok(b);
        }
    }
    Err(io::Error::other("fixture_registry_did_not_settle"))
}
fn protected_state() -> io::Result<serde_json::Value> {
    let steam = Path::new("/home/university/.local/share/Steam");
    let mut paths = vec![
        steam.join("config/config.vdf"),
        steam.join("steamapps/appmanifest_4162040.acf"),
        steam.join(
            "steamapps/common/Zenless Zone Zero/games/ZenlessZoneZero Game/ZenlessZoneZero.exe",
        ),
        steam.join("userdata/1416344634/config/localconfig.vdf"),
        steam.join("userdata/1416423935/config/localconfig.vdf"),
    ];
    let prefix = steam.join("steamapps/compatdata/4162040/pfx");
    paths.extend(["system.reg", "user.reg", "userdef.reg"].map(|s| prefix.join(s)));
    let mut state = BTreeMap::new();
    for path in paths {
        // Selective coverage; executable uses metadata to avoid rehashing a large game file.
        let m = fs::symlink_metadata(&path)?;
        let sha = if path.extension().is_some_and(|e| e == "exe") {
            None
        } else {
            Some(hash(&read(&path, 16 * 1024 * 1024)?))
        };
        state.insert(path.display().to_string(), serde_json::json!({"dev":m.dev(),"ino":m.ino(),"size":m.len(),"mtime":m.mtime(),"mtime_ns":m.mtime_nsec(),"sha256":sha}));
    }
    Ok(serde_json::json!({"prefix_identity":prefix_file_identity(&prefix),"selected_paths":state}))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args_os().nth(1).is_none()
        && std::env::current_exe()?.file_name()
            == Some(std::ffi::OsStr::new("fixture-container-transport"))
    {
        lxmi_bridge::enter_fixture_container_transport()
            .map_err(|e| io::Error::other(e.to_string()))?;
        return Ok(());
    }
    if std::env::var("LXMI_RUN_FIXTURE_ENVIRONMENT_COMPARISON").as_deref() != Ok("YES") {
        return Err(io::Error::other("opt_in_required").into());
    }
    let steam = Path::new("/home/university/.local/share/Steam");
    let proton = steam.join("steamapps/common/Proton - Experimental");
    let version = String::from_utf8(read(&proton.join("version"), 4096)?)?;
    if version.trim() != "1790244902 experimental-11.0-20260924-x86_64" {
        return Err(io::Error::other("runtime_drift_no_substitution").into());
    }
    let helpers = Path::new("/home/university/.local/share/lxmi/helpers");
    let helper = read(&helpers.join("lxmi-bridge-helper.exe"), 64 * 1024 * 1024)?;
    if hash(&helper) != "2a1939728e18bd3190b8dbb102d943bd5d65fa509617dbe42d48c45d02463fd2" {
        return Err(io::Error::other("helper_identity_changed").into());
    }
    let protected_before = protected_state()?;
    let mut nonce_bytes = [0u8; 16];
    getrandom::fill(&mut nonce_bytes).map_err(|e| io::Error::other(e.to_string()))?;
    let nonce: String = nonce_bytes.iter().map(|b| format!("{b:02x}")).collect();
    let root = PathBuf::from(format!("/tmp/lxmi-environment-fixture-{nonce}"));
    fs::DirBuilder::new().mode(0o700).create(&root)?;
    let transport = read(&std::env::current_exe()?, 256 * 1024 * 1024)?;
    fs::write(
        root.join("fixture-container-transport.sha256"),
        hash(&transport),
    )?;
    fs::write(root.join("fixture-container-transport"), transport)?;
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(
        root.join("fixture-container-transport"),
        fs::Permissions::from_mode(0o700),
    )?;
    fs::create_dir(root.join("helpers"))?;
    fs::write(root.join("helpers/lxmi-bridge-helper.exe"), helper)?;
    fs::write(
        root.join("helpers/lxmi-bridge-helper.json"),
        read(&helpers.join("lxmi-bridge-helper.json"), 16384)?,
    )?;
    let marker_root = root.join("marker");
    fs::create_dir(&marker_root)?;
    fs::write(
        marker_root.join("runtime-manifest.json"),
        serde_json::to_vec(&serde_json::json!({"schema":1,"fixture_nonce":nonce}))?,
    )?;
    let config = BridgeTestConfig {
        managed_root: root.clone(),
        runtime_root: marker_root,
        explicit_runtime: ExplicitBridgeRuntime {
            display_name: "Proton - Experimental".into(),
            version: Some(version.trim().into()),
            proton_script: proton.join("proton"),
        },
        steam_client_install_path: steam.to_path_buf(),
        prefix_mode: lxmi_bridge::PrefixMode::IsolatedTemporaryPrefix,
    };
    let prefix = root.join("test-prefixes/bridge-v1/compatdata/pfx");
    let mut prior = None;
    let mut reports = Vec::new();
    // Separate normal cold-repeat changes from a Linux environment transition, then reverse it.
    for (label, environment) in [
        (
            "container_initialize",
            FixtureLinuxEnvironment::SteamRuntime4,
        ),
        ("container_repeat", FixtureLinuxEnvironment::SteamRuntime4),
        ("container_to_direct", FixtureLinuxEnvironment::Direct),
        ("direct_repeat", FixtureLinuxEnvironment::Direct),
        (
            "direct_to_container",
            FixtureLinuxEnvironment::SteamRuntime4,
        ),
    ] {
        eprintln!("fixture step started: {label}");
        let step = match run_fixture_environment_step(&config, environment) {
            Ok(s) => s,
            Err(e) => {
                // Do not print raw runtime logs. Keep typed failure and verify selective protected state.
                eprintln!(
                    "{}",
                    serde_json::json!({"step":label,"error_code":e.code,"protected_state_unchanged":protected_before == protected_state()?,"fixture_root":root})
                );
                return Err(io::Error::other("fixture_step_failed_no_retry").into());
            }
        };
        let current = settled(&prefix)?;
        let mut diagnosis = BTreeMap::new();
        if let Some(ref old) = prior {
            let old: &BTreeMap<String, Vec<u8>> = old;
            for (name, bytes) in &current {
                diagnosis.insert(
                    name,
                    diagnose_registry_change(&old[name], bytes, &hash(&old[name]), &hash(bytes))
                        .map_err(|e| io::Error::other(format!("{e:?}")))?,
                );
            }
        }
        reports.push(serde_json::json!({"step":label,"environment":environment,"outer_executable":step.outer_executable,"outer_arguments":step.outer_arguments,
            "entry_point_sha256":step.entry_point_sha256,"runtime_versions_sha256":step.runtime_versions_sha256,"runtime_versions":step.runtime_versions,
            "private_variable_directory":step.private_variable_directory,"helper_sha256":step.bridge.actual_helper_sha256,"handshake":step.bridge.handshake,
            "exit_code":step.bridge.process.exit_code,"marker_hash_match":step.bridge.runtime_manifest_hash_matches,"nonce_verified":true,
            "prefix_identity":prefix_file_identity(&prefix),"registry_hashes":current.iter().map(|(k,v)|(k,hash(v))).collect::<BTreeMap<_,_>>(),"diagnosis":diagnosis}));
        prior = Some(current);
        if protected_before != protected_state()? {
            return Err(io::Error::other("protected_path_changed_stop").into());
        }
    }
    let report = serde_json::json!({"schema":1,"fixture_root":root,"proton_version":version.trim(),"steps":reports,
        "protected_state_before":protected_before,"protected_state_unchanged":true,"original_game_entry_result":"UnexpectedMutation","game_entry_authorized":false});
    let serialized = serde_json::to_vec_pretty(&report)?;
    fs::write(root.join("evidence.json"), &serialized)?;
    println!(
        "{}",
        serde_json::json!({"evidence_path":root.join("evidence.json"),"sha256":hash(&serialized),"protected_state_unchanged":true,"game_entry_authorized":false})
    );
    Ok(())
}
