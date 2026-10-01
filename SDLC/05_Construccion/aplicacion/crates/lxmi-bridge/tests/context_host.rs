use lxmi_bridge::{
    run_context_lab, stage_context_peers, ContextExperimentMode, ContextLabConfig,
    ExplicitBridgeRuntime,
};
use std::{env, path::PathBuf};
fn path(key: &str) -> PathBuf {
    env::var_os(key)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("Set {key} explicitly"))
}
#[test]
#[ignore = "opt-in: executes only LXMI peers under isolated Proton prefixes"]
fn isolated_context_peers() {
    assert_eq!(env::var("LXMI_RUN_CONTEXT_HOST_TEST").as_deref(), Ok("YES"));
    let root = path("LXMI_HOST_MANAGED_ROOT");
    stage_context_peers(&root, &path("LXMI_CONTEXT_BUILD_ROOT")).unwrap();
    let config = ContextLabConfig {
        managed_root: root,
        explicit_runtime: ExplicitBridgeRuntime {
            display_name: "Explicit Proton Experimental lab".into(),
            version: None,
            proton_script: path("LXMI_HOST_PROTON_SCRIPT"),
        },
        steam_client_install_path: path("LXMI_HOST_STEAM_ROOT"),
        mode: if env::var("LXMI_CONTEXT_MODE").as_deref() == Ok("different") {
            ContextExperimentMode::DifferentPrefixes
        } else {
            ContextExperimentMode::SamePrefix
        },
        side_effects_acknowledged: true,
    };
    let result = run_context_lab(&config)
        .unwrap_or_else(|e| panic!("CONTEXT_ERROR={}", serde_json::to_string(&e).unwrap()));
    assert!(result.peer_a.path_readable && result.peer_b.path_readable);
    assert!(!result.game_join_plan.game_experiment_authorized);
    println!(
        "HOST_CONTEXT_RESULT={}",
        serde_json::to_string(&result).unwrap()
    );
    // Host probe outcome is recorded, not forced to an assumed result.
}
