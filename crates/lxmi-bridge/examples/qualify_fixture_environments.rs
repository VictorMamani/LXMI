//! Opt-in diagnostic: harmless bridge helper only, fresh /tmp LXMI prefix, no game entry.
use lxmi_bridge::{
    clone_environment_fixture_prefix, diagnose_fixture_registry_sections, diagnose_registry_change,
    prefix_file_identity, run_fixture_environment_step, BridgeTestConfig, ExplicitBridgeRuntime,
    FixtureLinuxEnvironment,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args_os().nth(1).is_none()
        && std::env::current_exe()?.file_name()
            == Some(std::ffi::OsStr::new("fixture-container-transport"))
    {
        lxmi_bridge::enter_fixture_container_transport()
            .map_err(|e| io::Error::other(e.to_string()))?;
        return Ok(());
    }
    if std::env::var("LXMI_RUN_ENVIRONMENT_QUALIFICATION").as_deref() != Ok("YES") {
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
    let pins_before = runtime_pins(steam, &proton)?;
    let mut seed = prepare(steam, &proton, &version, helpers, &helper)?;
    eprintln!("verifying private Proton tool copy; installed source will not execute");
    let mirror = lxmi_bridge::stage_fixture_proton_mirror(&seed.managed_root, steam)
        .map_err(|e| io::Error::other(e.to_string()))?;
    seed.explicit_runtime.proton_script = mirror.mirror_root.join("proton");
    let source_metadata = tool_source_metadata(&proton)?;
    let proton = mirror.mirror_root.clone();
    eprintln!("private container seed initialization");
    let seed_step = run_fixture_environment_step(&seed, FixtureLinuxEnvironment::SteamRuntime4)
        .map_err(|e| {
            let bytes = serde_json::to_vec(&e).unwrap();
            fs::write(seed.managed_root.join("failure.json"), bytes).unwrap();
            io::Error::other(format!(
                "{}; controlled failure artifact: {}",
                e,
                seed.managed_root.join("failure.json").display()
            ))
        })?;
    let seed_prefix = seed
        .managed_root
        .join("test-prefixes/bridge-v1/compatdata/pfx");
    let initial = settled(&seed_prefix)?;
    let common_marker = read(&seed.runtime_root.join("runtime-manifest.json"), 16384)?;
    let mut direct = prepare(steam, &proton, &version, helpers, &helper)?;
    let mut container = prepare(steam, &proton, &version, helpers, &helper)?;
    let mut run_wrapper = prepare(steam, &proton, &version, helpers, &helper)?;
    for config in [&mut direct, &mut container, &mut run_wrapper] {
        lxmi_bridge::attach_fixture_proton_mirror(&config.managed_root, &mirror)
            .map_err(|e| io::Error::other(e.to_string()))?;
        config.explicit_runtime.proton_script = mirror.mirror_root.join("proton");
    }
    let mut manifests = Vec::new();
    for config in [&direct, &container, &run_wrapper] {
        fs::write(
            config.runtime_root.join("runtime-manifest.json"),
            &common_marker,
        )?;
        eprintln!("complete private clone verification");
        manifests.push(
            clone_environment_fixture_prefix(&seed.managed_root, &config.managed_root)
                .map_err(|e| io::Error::other(e.to_string()))?,
        );
        if registries(
            &config
                .managed_root
                .join("test-prefixes/bridge-v1/compatdata/pfx"),
        )? != initial
        {
            return Err(io::Error::other("initial_registry_state_not_identical").into());
        }
    }
    if manifests.iter().any(|m| m != &manifests[0]) {
        return Err(io::Error::other("initial_complete_tree_not_identical").into());
    }
    let mut reports = Vec::new();
    for (config, environment, label) in [
        (&direct, FixtureLinuxEnvironment::Direct, "direct"),
        (
            &container,
            FixtureLinuxEnvironment::SteamRuntime4,
            "container",
        ),
        (
            &run_wrapper,
            FixtureLinuxEnvironment::SteamRuntimeRun,
            "container_run_wrapper",
        ),
    ] {
        let prefix = config
            .managed_root
            .join("test-prefixes/bridge-v1/compatdata/pfx");
        let mut prior = initial.clone();
        for run in 1..=2 {
            eprintln!("private fixture: {label} run {run}");
            let before_metadata = selected_metadata(&prefix)?;
            let before_mappings = mappings(&prefix)?;
            if runtime_pins(steam, &proton)? != pins_before {
                return Err(io::Error::other("runtime_pin_drift").into());
            }
            let step = run_fixture_environment_step(config, environment)
                .map_err(|e| io::Error::other(format!("fixture failed: {:?}; no retry", e.code)))?;
            let current = settled(&prefix)?;
            let after_metadata = selected_metadata(&prefix)?;
            if runtime_pins(steam, &proton)? != pins_before {
                return Err(io::Error::other("runtime_pin_drift").into());
            }
            let mut diagnosis = BTreeMap::new();
            let mut sections = BTreeMap::new();
            for (name, bytes) in &current {
                diagnosis.insert(
                    name,
                    diagnose_registry_change(
                        &prior[name],
                        bytes,
                        &hash(&prior[name]),
                        &hash(bytes),
                    )
                    .map_err(|e| io::Error::other(format!("{e:?}")))?,
                );
                sections.insert(
                    name,
                    diagnose_fixture_registry_sections(&prior[name], bytes)
                        .map_err(|e| io::Error::other(format!("{e:?}")))?,
                );
            }
            reports.push(serde_json::json!({"strategy":label,"run":run,"execution":step,
                "before_registry_hashes":prior.iter().map(|(k,v)|(k,hash(v))).collect::<BTreeMap<_,_>>(),
                "after_registry_hashes":current.iter().map(|(k,v)|(k,hash(v))).collect::<BTreeMap<_,_>>(),
                "prefix_identity":prefix_file_identity(&prefix),"diagnosis":diagnosis,"sections":sections,
                "dosdevices_before":before_mappings,"dosdevices_after":mappings(&prefix)?,"selected_metadata_before":before_metadata,"selected_metadata_after":after_metadata}));
            prior = current;
        }
    }
    if tool_source_metadata(&mirror.source_tool)? != source_metadata {
        return Err(io::Error::other("installed_tool_source_changed_stop").into());
    }
    let report = serde_json::json!({"schema":4,"private_tool_mirror":mirror,"installed_source_selected_metadata_before":source_metadata,"installed_source_selected_metadata_after":tool_source_metadata(&mirror.source_tool)?,"runtime_pins":pins_before,"captured_at_unix_seconds":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),"common_marker_sha256":hash(&common_marker),"proton_version":version.trim(),"proton_script_sha256":hash(&read(&proton.join("proton"),4*1024*1024)?),
        "toolmanifest_sha256":hash(&read(&proton.join("toolmanifest.vdf"),16384)?),
        "seed_root":seed.managed_root,"seed_execution":seed_step,"seed_registry_hashes":initial.iter().map(|(k,v)|(k,hash(v))).collect::<BTreeMap<_,_>>(),
        "clone_manifest_sha256":manifests,"identical_initial_tree_verified":true,
        "steps":reports,"game_entry_authorized":false,"real_prefix_accessed":false,
        "environment_allowlist":{"PATH":"/usr/bin:/bin","LANG":"C.UTF-8","HOME":"<fixture>/test-prefixes/bridge-v1/home","XDG_CONFIG_HOME":"<fixture>/test-prefixes/bridge-v1/config","XDG_CACHE_HOME":"<fixture>/test-prefixes/bridge-v1/cache","XDG_DATA_HOME":"<fixture>/test-prefixes/bridge-v1/data","STEAM_COMPAT_DATA_PATH":"<fixture>/test-prefixes/bridge-v1/compatdata","STEAM_COMPAT_CLIENT_INSTALL_PATH":steam,"LXMI_BRIDGE_TEST_MARKER":"lxmi-bridge-v1","WINEDEBUG":"-all","PYTHONDONTWRITEBYTECODE":"1"},
        "container_additions":["PRESSURE_VESSEL_VARIABLE_DIR=<fixture>/runtime-variable","LXMI_FIXTURE_INPUT_SHA256=<fresh request hash>","LXMI_FIXTURE_REQUEST_NONCE=<fresh bridge nonce>"],
        "request_transport":{"direct":"stdin","container":"private_request_file","mode":"0600","bounded_bytes":65536,"consumed_once":true},
        "coverage":"Three registry hives plus dosdevices mappings and complete initial clone integrity; not a complete before/after tree mutation inventory"});
    let serialized = serde_json::to_vec_pretty(&report)?;
    fs::write(
        seed.managed_root.join("qualification-evidence.json"),
        &serialized,
    )?;
    println!(
        "{}",
        serde_json::json!({"evidence_path":seed.managed_root.join("qualification-evidence.json"),"sha256":hash(&serialized),"real_prefix_accessed":false,"game_entry_authorized":false})
    );
    Ok(())
}

fn prepare(
    steam: &Path,
    proton: &Path,
    version: &str,
    helpers: &Path,
    helper: &[u8],
) -> io::Result<BridgeTestConfig> {
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
    Ok(BridgeTestConfig {
        managed_root: root.clone(),
        runtime_root: marker_root,
        explicit_runtime: ExplicitBridgeRuntime {
            display_name: "Proton - Experimental".into(),
            version: Some(version.trim().into()),
            proton_script: proton.join("proton"),
        },
        steam_client_install_path: steam.to_path_buf(),
        prefix_mode: lxmi_bridge::PrefixMode::IsolatedTemporaryPrefix,
    })
}
fn mappings(prefix: &Path) -> io::Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    for entry in fs::read_dir(prefix.join("dosdevices"))? {
        let entry = entry?;
        if !entry.file_type()?.is_symlink() {
            return Err(io::Error::other("unexpected_dosdevice_type"));
        }
        out.insert(
            entry.file_name().to_string_lossy().into_owned(),
            fs::read_link(entry.path())?.display().to_string(),
        );
    }
    Ok(out)
}

fn selected_metadata(prefix: &Path) -> io::Result<BTreeMap<String, serde_json::Value>> {
    [".","drive_c","dosdevices","system.reg","user.reg","userdef.reg",".update-timestamp","creation_sync_guard"].into_iter().map(|relative| {
        let value=match fs::symlink_metadata(prefix.join(relative)) {
            Ok(m) => serde_json::json!({"device":m.dev(),"inode":m.ino(),"mode":m.mode(),"size":m.len(),"mtime":m.mtime(),"mtime_nsec":m.mtime_nsec(),"is_symlink":m.file_type().is_symlink()}),
            Err(e) if e.kind()==io::ErrorKind::NotFound => serde_json::Value::Null,
            Err(e) => return Err(e),
        };
        Ok((relative.into(),value))
    }).collect()
}
fn runtime_pins(steam: &Path, proton: &Path) -> io::Result<BTreeMap<String, String>> {
    let runtime = steam.join("steamapps/common/SteamLinuxRuntime_4");
    [
        ("proton_script", proton.join("proton")),
        ("proton_version", proton.join("version")),
        ("toolmanifest", proton.join("toolmanifest.vdf")),
        ("runtime_entry_point", runtime.join("_v2-entry-point")),
        ("runtime_run", runtime.join("run")),
        ("runtime_versions", runtime.join("VERSIONS.txt")),
        (
            "pressure_vessel_wrap",
            runtime.join("pressure-vessel/bin/pressure-vessel-wrap"),
        ),
    ]
    .into_iter()
    .map(|(name, path)| Ok((name.into(), hash(&read(&path, 64 * 1024 * 1024)?))))
    .collect()
}

fn tool_source_metadata(root: &Path) -> io::Result<BTreeMap<String, serde_json::Value>> {
    ["proton","version","dist.lock","files/steampipe_fixups_mtime","steampipe_fixups.json","__pycache__"].into_iter().map(|name| {
        let value=match fs::symlink_metadata(root.join(name)) {
            Ok(m)=>serde_json::json!({"device":m.dev(),"inode":m.ino(),"mode":m.mode(),"size":m.len(),"mtime":m.mtime(),"mtime_nsec":m.mtime_nsec()}),
            Err(e) if e.kind()==io::ErrorKind::NotFound=>serde_json::Value::Null,
            Err(e)=>return Err(e),
        }; Ok((name.into(),value))
    }).collect()
}
