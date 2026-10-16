#![cfg(target_os = "linux")]

use lxmi_bridge::{
    run_bridge_test, BridgeTestConfig, ExplicitBridgeRuntime, HandshakeState, PathVisibility,
    PrefixMode,
};
use std::{env, path::PathBuf};

fn required_path(name: &str) -> PathBuf {
    env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("set {name} explicitly to opt into the Proton host test"))
}

#[test]
#[ignore = "launches an explicitly selected Proton and writes only the LXMI-owned test prefix"]
fn proton_helper_reads_managed_runtime_and_returns_verified_handshake() {
    assert_eq!(
        env::var("LXMI_RUN_PROTON_BRIDGE_HOST_TEST").as_deref(),
        Ok("YES"),
        "set LXMI_RUN_PROTON_BRIDGE_HOST_TEST=YES to acknowledge this host test"
    );

    let config = BridgeTestConfig {
        managed_root: required_path("LXMI_HOST_MANAGED_ROOT"),
        runtime_root: required_path("LXMI_HOST_RUNTIME_ROOT"),
        explicit_runtime: ExplicitBridgeRuntime {
            display_name: env::var("LXMI_HOST_PROTON_DISPLAY_NAME")
                .expect("set the explicit bridge-test Proton display name"),
            version: env::var("LXMI_HOST_PROTON_VERSION").ok(),
            proton_script: required_path("LXMI_HOST_PROTON_SCRIPT"),
        },
        steam_client_install_path: required_path("LXMI_HOST_STEAM_ROOT"),
        prefix_mode: PrefixMode::IsolatedTemporaryPrefix,
    };

    let result = match run_bridge_test(&config) {
        Ok(result) => result,
        Err(error) => {
            eprintln!(
                "HOST_BRIDGE_ERROR={}",
                serde_json::to_string(&error).expect("bridge error serializes")
            );
            panic!("host bridge failed: {error}");
        }
    };

    assert_eq!(result.handshake, HandshakeState::Succeeded);
    assert_eq!(result.path_visibility, PathVisibility::MappedAndReadable);
    assert!(result.runtime_manifest_hash_matches);
    assert!(result.environment_marker_matches);
    assert_eq!(result.process.exit_code, 0);
    assert_eq!(result.prefix_mode, PrefixMode::IsolatedTemporaryPrefix);
    assert!(result.compatdata_path.starts_with(&config.managed_root));
    assert!(!result
        .compatdata_path
        .to_string_lossy()
        .contains("compatdata/4162040"));
    assert!(!result
        .process
        .arguments
        .iter()
        .any(|argument| argument.contains("ZenlessZoneZero.exe")));

    println!(
        "HOST_BRIDGE_RESULT={}",
        serde_json::to_string(&result).expect("bridge result serializes")
    );
}
