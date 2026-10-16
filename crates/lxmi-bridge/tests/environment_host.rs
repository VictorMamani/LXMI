//! Opt-in only: the fixed harness creates all roots itself; no game prefix input.
#[test]
#[ignore = "opt-in private prefix DirectProton / SteamRuntime / run-wrapper matrix; build example first"]
fn controlled_private_environment_matrix() {
    assert_eq!(
        std::env::var("LXMI_RUN_ENVIRONMENT_QUALIFICATION").as_deref(),
        Ok("YES")
    );
    let example = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples/qualify_fixture_environments");
    assert!(
        example.is_file(),
        "Build the fixed qualification example first"
    );
    let output = std::process::Command::new(example)
        .env("LXMI_RUN_ENVIRONMENT_QUALIFICATION", "YES")
        .output()
        .expect("fixed private fixture harness");
    assert!(
        output.status.success(),
        "private fixture matrix failed; no automatic retry"
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["real_prefix_accessed"], false);
    assert_eq!(report["game_entry_authorized"], false);
}
