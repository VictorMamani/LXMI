//! Evidence and qualification only. No real-prefix executor or frontend override.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinuxRuntimeKind {
    DirectProton,
    SteamLinuxRuntimeContainer,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentQualification {
    Unqualified,
    FixtureEquivalent,
    SteamRuntimeQualified,
    Conflicting,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeRequestTransport {
    Stdin,
    PrivateRequestFile,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrefixMutationClassification {
    StableExpected,
    VolatileExpected,
    EnvironmentSpecific,
    Unexpected,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxRuntimeEnvironment {
    pub runtime_kind: LinuxRuntimeKind,
    pub steam_linux_runtime: Option<String>,
    pub pressure_vessel: Option<String>,
    pub proton_tool: String,
    /// Explicit allowlist, never an inherited process environment dump.
    pub environment: BTreeMap<String, String>,
    pub mounts: Vec<String>,
    pub provenance: Vec<String>,
    pub request_transport: BridgeRequestTransport,
    pub qualification: EnvironmentQualification,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentQualificationEvidence {
    pub identical_initial_state_verified: bool,
    pub successful_runs: usize,
    pub upstream_mechanism_pinned: bool,
    pub deterministic_environment_and_mounts: bool,
    pub runtime_pins_unchanged: bool,
    pub mutations: Vec<PrefixMutationClassification>,
}
pub fn qualify_environment(
    kind: LinuxRuntimeKind,
    evidence: &EnvironmentQualificationEvidence,
) -> EnvironmentQualification {
    use EnvironmentQualification::*;
    use PrefixMutationClassification as M;
    if !evidence.runtime_pins_unchanged || evidence.mutations.contains(&M::Unexpected) {
        return Conflicting;
    }
    if !evidence.identical_initial_state_verified
        || evidence.successful_runs < 2
        || evidence.mutations.contains(&M::Unknown)
    {
        return Unqualified;
    }
    if kind == LinuxRuntimeKind::DirectProton
        && evidence.mutations.contains(&M::EnvironmentSpecific)
    {
        return Unqualified;
    }
    if kind == LinuxRuntimeKind::SteamLinuxRuntimeContainer
        && evidence.upstream_mechanism_pinned
        && evidence.deterministic_environment_and_mounts
    {
        return SteamRuntimeQualified;
    }
    FixtureEquivalent
}
pub fn request_transport(kind: LinuxRuntimeKind) -> BridgeRequestTransport {
    match kind {
        LinuxRuntimeKind::DirectProton => BridgeRequestTransport::Stdin,
        LinuxRuntimeKind::SteamLinuxRuntimeContainer => BridgeRequestTransport::PrivateRequestFile,
    }
}
/// Pure policy check; a status is never accepted from frontend as authorization.
pub fn require_qualified_environment(
    state: EnvironmentQualification,
) -> Result<(), super::RealPrefixEntryState> {
    if state == EnvironmentQualification::SteamRuntimeQualified {
        Ok(())
    } else {
        Err(super::RealPrefixEntryState::LinuxEnvironmentUnqualified)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn evidence() -> EnvironmentQualificationEvidence {
        EnvironmentQualificationEvidence {
            identical_initial_state_verified: true,
            successful_runs: 2,
            upstream_mechanism_pinned: true,
            deterministic_environment_and_mounts: true,
            runtime_pins_unchanged: true,
            mutations: vec![PrefixMutationClassification::VolatileExpected],
        }
    }
    #[test]
    fn unknown_never_hidden_by_volatile_field() {
        let mut e = evidence();
        e.mutations.push(PrefixMutationClassification::Unknown);
        assert_eq!(
            qualify_environment(LinuxRuntimeKind::SteamLinuxRuntimeContainer, &e),
            EnvironmentQualification::Unqualified
        );
    }
    #[test]
    fn qualification_requires_controlled_repeats_pins_and_mounts() {
        let mut e = evidence();
        assert_eq!(
            qualify_environment(LinuxRuntimeKind::SteamLinuxRuntimeContainer, &e),
            EnvironmentQualification::SteamRuntimeQualified
        );
        e.deterministic_environment_and_mounts = false;
        assert_eq!(
            qualify_environment(LinuxRuntimeKind::SteamLinuxRuntimeContainer, &e),
            EnvironmentQualification::FixtureEquivalent
        );
        e.successful_runs = 1;
        assert_eq!(
            qualify_environment(LinuxRuntimeKind::SteamLinuxRuntimeContainer, &e),
            EnvironmentQualification::Unqualified
        );
        e.runtime_pins_unchanged = false;
        assert_eq!(
            qualify_environment(LinuxRuntimeKind::SteamLinuxRuntimeContainer, &e),
            EnvironmentQualification::Conflicting
        );
    }
    #[test]
    fn direct_font_environment_change_is_not_qualified() {
        let mut e = evidence();
        e.mutations = vec![PrefixMutationClassification::EnvironmentSpecific];
        assert_eq!(
            qualify_environment(LinuxRuntimeKind::DirectProton, &e),
            EnvironmentQualification::Unqualified
        );
    }
    #[test]
    fn gate_accepts_only_qualified_and_transport_is_explicit() {
        for q in [
            EnvironmentQualification::Unqualified,
            EnvironmentQualification::FixtureEquivalent,
            EnvironmentQualification::Conflicting,
            EnvironmentQualification::Unknown,
        ] {
            assert!(require_qualified_environment(q).is_err());
        }
        assert!(
            require_qualified_environment(EnvironmentQualification::SteamRuntimeQualified).is_ok()
        );
        assert_eq!(
            request_transport(LinuxRuntimeKind::DirectProton),
            BridgeRequestTransport::Stdin
        );
        assert_eq!(
            request_transport(LinuxRuntimeKind::SteamLinuxRuntimeContainer),
            BridgeRequestTransport::PrivateRequestFile
        );
        assert!(super::super::require_qualified_harmless_entry_environment().is_err());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureEnvironmentQualificationSummary {
    pub strategy: String,
    pub qualification: EnvironmentQualification,
    pub helper_successful_runs: usize,
    pub font_differences: Vec<String>,
    pub pending_sections: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxEnvironmentQualificationPanel {
    pub current_environment: LinuxRuntimeEnvironment,
    pub fixtures: Vec<FixtureEnvironmentQualificationSummary>,
    pub evidence_sha256: String,
    pub evidence_source: String,
    pub real_prefix_entry: String,
    pub blocker: String,
    pub limitations: Vec<String>,
    #[serde(default)]
    pub executor: Option<super::QualifiedExecutorPanel>,
}
/// Reviewed fixture evidence, not a current-host revalidation or an execution capability.
pub fn linux_environment_qualification_panel() -> Result<LinuxEnvironmentQualificationPanel, String>
{
    let bytes = include_bytes!(
        "../../../../../06_Verificacion/evidencias/environment_qualification_0_8_8_1_summary.json"
    );
    let mut panel: LinuxEnvironmentQualificationPanel =
        serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if panel.current_environment.qualification != EnvironmentQualification::Unqualified
        || panel.real_prefix_entry != "blocked"
        || panel.blocker != "linux_runtime_environment_unqualified"
    {
        return Err("reviewed evidence cannot override real-prefix guard".into());
    }
    use sha2::{Digest, Sha256};
    let matrix = include_bytes!(
        "../../../../../06_Verificacion/evidencias/environment_qualification_0_8_8_1.json"
    );
    if format!("{:x}", Sha256::digest(matrix)) != panel.evidence_sha256 {
        return Err("fixture receipt integrity mismatch".into());
    }
    let report: serde_json::Value = serde_json::from_slice(matrix).map_err(|e| e.to_string())?;
    for fixture in &panel.fixtures {
        if qualification_from_fixture_report(&report, &fixture.strategy)? != fixture.qualification {
            return Err("qualification summary does not match reviewed fixture evidence".into());
        }
    }
    panel.current_environment.provenance.push("Reviewed historical fixture evidence; real prefix not executed, current host not revalidated by this command".into());
    Ok(panel)
}

/// Derive private-fixture qualification from the reviewed matrix. Never grants real entry.
fn qualification_from_fixture_report(
    report: &serde_json::Value,
    strategy: &str,
) -> Result<EnvironmentQualification, String> {
    if !matches!(strategy, "direct" | "container" | "container_run_wrapper") {
        return Err("unknown runtime strategy".into());
    }
    let steps = report["steps"].as_array().ok_or("fixture steps missing")?;
    let matching: Vec<_> = steps.iter().filter(|s| s["strategy"] == strategy).collect();
    if matching.len() != 2 || matching[0]["run"] != 1 || matching[1]["run"] != 2 {
        return Err("two controlled runs required".into());
    }
    let initial = report["clone_manifest_sha256"]
        .as_array()
        .ok_or("clone receipts missing")?;
    let initial_verified = initial.len() == 3
        && initial.iter().all(|v| {
            v == &initial[0]
                && v.as_str()
                    .is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        })
        && report["identical_initial_tree_verified"] == true;
    let mut mutations = Vec::new();
    for step in &matching {
        for sections in step["sections"]
            .as_object()
            .ok_or("section evidence missing")?
            .values()
        {
            for section in sections.as_array().ok_or("invalid section evidence")? {
                let classification: PrefixMutationClassification =
                    serde_json::from_value(section["classification"].clone())
                        .map_err(|e| e.to_string())?;
                mutations.push(classification);
            }
        }
    }
    for step in &matching {
        let before = step["selected_metadata_before"]
            .as_object()
            .ok_or("before metadata missing")?;
        let after = step["selected_metadata_after"]
            .as_object()
            .ok_or("after metadata missing")?;
        for name in before
            .keys()
            .chain(after.keys())
            .collect::<std::collections::BTreeSet<_>>()
        {
            let a = before.get(name);
            let b = after.get(name);
            if a == b {
                continue;
            }
            let Some((a, b)) = a.zip(b).filter(|(a, b)| a.is_object() && b.is_object()) else {
                mutations.push(PrefixMutationClassification::Unknown);
                continue;
            };
            let supported = |value: &serde_json::Value| {
                value.as_object().is_some_and(|v| {
                    v.len() == 7
                        && v.keys().all(|key| {
                            matches!(
                                key.as_str(),
                                "device"
                                    | "inode"
                                    | "mode"
                                    | "mtime"
                                    | "mtime_nsec"
                                    | "size"
                                    | "is_symlink"
                            )
                        })
                        && ["device", "inode", "mode", "size", "mtime_nsec"]
                            .iter()
                            .all(|key| value[*key].is_u64())
                        && value["mtime"].is_i64()
                        && value["is_symlink"].is_boolean()
                })
            };
            if !supported(a) || !supported(b) {
                mutations.push(PrefixMutationClassification::Unknown);
                continue;
            }
            if a["device"] != b["device"] || a["mode"] != b["mode"] || b["is_symlink"] != false {
                mutations.push(PrefixMutationClassification::Unexpected);
                continue;
            }
            if name == "." && a["inode"] != b["inode"] {
                mutations.push(PrefixMutationClassification::Unexpected);
                continue;
            }
            mutations.push(
                if matches!(
                    name.as_str(),
                    "." | "dosdevices" | "system.reg" | "user.reg" | "userdef.reg"
                ) {
                    PrefixMutationClassification::VolatileExpected
                } else {
                    PrefixMutationClassification::Unknown
                },
            );
        }
    }
    let allowlist = report["environment_allowlist"]
        .as_object()
        .ok_or("environment declaration missing")?;
    if allowlist.keys().any(|key| {
        !matches!(
            key.as_str(),
            "PATH"
                | "LANG"
                | "HOME"
                | "XDG_CONFIG_HOME"
                | "XDG_CACHE_HOME"
                | "XDG_DATA_HOME"
                | "STEAM_COMPAT_CLIENT_INSTALL_PATH"
                | "STEAM_COMPAT_DATA_PATH"
                | "LXMI_BRIDGE_TEST_MARKER"
                | "WINEDEBUG"
                | "PYTHONDONTWRITEBYTECODE"
        )
    }) {
        mutations.push(PrefixMutationClassification::Unknown);
    }
    let kind = if strategy == "direct" {
        LinuxRuntimeKind::DirectProton
    } else {
        LinuxRuntimeKind::SteamLinuxRuntimeContainer
    };
    let mounts_verified = kind == LinuxRuntimeKind::SteamLinuxRuntimeContainer
        && matching.iter().all(|step| {
            let metadata = &step["execution"]["container_metadata"];
            metadata["selected_environment"]["container"] == "pressure-vessel"
                && metadata["selected_mounts"]
                    .as_array()
                    .is_some_and(|mounts| {
                        [
                            "/usr",
                            "/run/host/usr",
                            "/run/host/fonts",
                            "/run/host/local-fonts",
                        ]
                        .iter()
                        .all(|point| {
                            mounts.iter().any(|m| {
                                m["mountpoint"] == *point
                                    && m["options"]
                                        .as_str()
                                        .is_some_and(|opts| opts.split(',').any(|o| o == "ro"))
                            })
                        })
                    })
        });
    let successful_runs = matching
        .iter()
        .filter(|step| {
            let bridge = &step["execution"]["bridge"];
            bridge["handshake"] == "succeeded"
                && bridge["runtime_manifest_hash_matches"] == true
                && bridge["environment_marker_matches"] == true
                && bridge["process"]["exit_code"] == 0
                && bridge["runtime_manifest_sha256"] == report["common_marker_sha256"]
                && step["dosdevices_before"] == step["dosdevices_after"]
        })
        .count();
    let pins = &report["runtime_pins"];
    let private_tool = report["private_tool_mirror"]["source_unchanged_during_copy"] == true
        && report["installed_source_selected_metadata_before"]
            == report["installed_source_selected_metadata_after"]
        && report["private_tool_mirror"]["verified_tree_manifest_sha256"]
            .as_str()
            .is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        && report["private_tool_mirror"]["mirror_root"]
            .as_str()
            .is_some_and(|s| {
                s.starts_with("/tmp/lxmi-environment-fixture-") && s.ends_with("/proton-tool")
            });
    let pinned = report["proton_version"] == "1790244902 experimental-11.0-20260924-x86_64"
        && pins["proton_version"]
            == "c5a6f6db5dc22a1cd865275c112a4a8e82f209f06ef9b27321027b2f15e36a95"
        && pins["toolmanifest"]
            == "bb81d44a687cc6f4ea7ae05fa9f0502f35c48e462051809dce3b2a58492e3498"
        && pins["runtime_entry_point"]
            == "caa39b5cde8ea955288b574b49b3416fd16e7be3f53a17249d673f5ecfdce2c1"
        && pins["pressure_vessel_wrap"]
            == "0439811997a081e4f90f898b7e98e34e46cff3745a0611c17309947fd47f4da9"
        && pins["proton_script"]
            == "077db31dda75c570c1a91014f9a814b785307bc91e5dfb1480b984dc34166a54"
        && pins["runtime_versions"]
            == "ef61e9bce47d60d02707661d116c8048ffa39291b84acee687a1ed4e430af6be"
        && pins["runtime_run"]
            == "d5069c037c53f204a9d7d0ecb501b430e57dc2d702d49c9cccd57d569ca87eec";
    Ok(qualify_environment(
        kind,
        &EnvironmentQualificationEvidence {
            identical_initial_state_verified: initial_verified,
            successful_runs,
            upstream_mechanism_pinned: pinned,
            deterministic_environment_and_mounts: private_tool
                && report["environment_allowlist"]["PYTHONDONTWRITEBYTECODE"] == "1"
                && mounts_verified
                && report["environment_allowlist"]["PATH"] == "/usr/bin:/bin"
                && report["environment_allowlist"]["LANG"] == "C.UTF-8",
            runtime_pins_unchanged: pinned,
            mutations,
        },
    ))
}

#[cfg(test)]
mod receipt_tests {
    use super::*;
    fn matrix() -> serde_json::Value {
        serde_json::from_slice(include_bytes!(
            "../../../../../06_Verificacion/evidencias/environment_qualification_0_8_8_1.json"
        ))
        .unwrap()
    }
    #[test]
    fn reviewed_matrix_and_dto_match_without_host_reads_or_authorization() {
        let panel = linux_environment_qualification_panel().unwrap();
        assert_eq!(panel.fixtures.len(), 3);
        assert_eq!(
            panel.fixtures[0].qualification,
            EnvironmentQualification::Unqualified
        );
        assert_eq!(
            panel.fixtures[1].qualification,
            EnvironmentQualification::SteamRuntimeQualified
        );
        assert_eq!(
            panel.fixtures[2].qualification,
            EnvironmentQualification::SteamRuntimeQualified
        );
        assert_eq!(panel.real_prefix_entry, "blocked");
        let encoded = serde_json::to_vec(&panel).unwrap();
        let decoded: LinuxEnvironmentQualificationPanel = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded.blocker, "linux_runtime_environment_unqualified");
        assert!(super::super::require_qualified_harmless_entry_environment().is_err());
    }
    #[test]
    fn additional_unknown_field_cannot_be_hidden_by_successful_runs() {
        let mut report = matrix();
        report["steps"][2]["sections"]["system.reg"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({"section":"unreviewed","classification":"unknown"}));
        assert_eq!(
            qualification_from_fixture_report(&report, "container").unwrap(),
            EnvironmentQualification::Unqualified
        );
    }
    #[test]
    fn missing_readonly_mount_or_bad_hash_protocol_degrades_qualification() {
        let mut report = matrix();
        report["steps"][2]["execution"]["container_metadata"]["selected_mounts"] =
            serde_json::json!([]);
        assert_eq!(
            qualification_from_fixture_report(&report, "container").unwrap(),
            EnvironmentQualification::FixtureEquivalent
        );
        report["steps"][2]["execution"]["bridge"]["runtime_manifest_hash_matches"] = false.into();
        assert_eq!(
            qualification_from_fixture_report(&report, "container").unwrap(),
            EnvironmentQualification::Unqualified
        );
    }
    #[test]
    fn non_equivalent_clone_or_runtime_drift_cannot_qualify() {
        let mut report = matrix();
        report["clone_manifest_sha256"][0] = "different".into();
        assert_eq!(
            qualification_from_fixture_report(&report, "container").unwrap(),
            EnvironmentQualification::Unqualified
        );
        report["runtime_pins"]["proton_script"] = "changed".into();
        assert_eq!(
            qualification_from_fixture_report(&report, "container").unwrap(),
            EnvironmentQualification::Conflicting
        );
    }
    #[test]
    fn installed_tool_or_changed_source_cannot_qualify() {
        let mut report = matrix();
        report["private_tool_mirror"]["mirror_root"] =
            "/home/user/.local/share/Steam/steamapps/common/Proton - Experimental".into();
        assert_eq!(
            qualification_from_fixture_report(&report, "container").unwrap(),
            EnvironmentQualification::FixtureEquivalent
        );
        let mut report = matrix();
        report["private_tool_mirror"]["source_unchanged_during_copy"] = false.into();
        assert_ne!(
            qualification_from_fixture_report(&report, "container").unwrap(),
            EnvironmentQualification::SteamRuntimeQualified
        );
        report["installed_source_selected_metadata_after"] = serde_json::json!({"changed": true});
        assert_ne!(
            qualification_from_fixture_report(&report, "container").unwrap(),
            EnvironmentQualification::SteamRuntimeQualified
        );
    }
    #[test]
    fn undeclared_environment_or_metadata_difference_blocks_qualification() {
        let mut report = matrix();
        report["environment_allowlist"]["UNRELATED_TOKEN"] = "synthetic-only".into();
        assert_eq!(
            qualification_from_fixture_report(&report, "container").unwrap(),
            EnvironmentQualification::Unqualified
        );
        let mut report = matrix();
        report["steps"][2]["selected_metadata_after"]["drive_c"]["mtime"] = 1.into();
        assert_eq!(
            qualification_from_fixture_report(&report, "container").unwrap(),
            EnvironmentQualification::Unqualified
        );
        let mut report = matrix();
        report["steps"][2]["selected_metadata_after"]["user.reg"]["unreviewed"] = true.into();
        assert_eq!(
            qualification_from_fixture_report(&report, "container").unwrap(),
            EnvironmentQualification::Unqualified
        );
    }
    #[test]
    fn root_identity_or_permissions_change_is_conflicting() {
        let mut report = matrix();
        report["steps"][2]["selected_metadata_after"]["."]["inode"] = 1.into();
        assert_eq!(
            qualification_from_fixture_report(&report, "container").unwrap(),
            EnvironmentQualification::Conflicting
        );
        let mut report = matrix();
        report["steps"][2]["selected_metadata_after"]["user.reg"]["mode"] = 0.into();
        assert_eq!(
            qualification_from_fixture_report(&report, "container").unwrap(),
            EnvironmentQualification::Conflicting
        );
    }
}
