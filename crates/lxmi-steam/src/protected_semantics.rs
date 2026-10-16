//! Bounded read-only ZZZ config review. Raw VDF and unrelated values never leave this module.
use crate::{parse_key_values_document, KeyValuesValue};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::{self, Read},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const STEAM_SEMANTIC_POLICY: u16 = 1;
const LIMIT: u64 = 2 * 1024 * 1024;
const APP: &str = "4162040";
const LOCAL: &str = "userdata/1416423935/config/localconfig.vdf";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SteamRelevantField {
    pub key: String,
    pub state: String,
    pub safe_value: Option<String>,
    pub value_sha256: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SteamFileSemantics {
    pub document_kind: String,
    pub compat_tool_mapping_present: bool,
    pub game_mapping_present: bool,
    pub global_mapping_present: bool,
    pub game_mapping: Vec<SteamRelevantField>,
    pub global_mapping: Vec<SteamRelevantField>,
    pub launch_options: Vec<SteamRelevantField>,
    pub launch_overrides: Vec<SteamRelevantField>,
    pub steam_play: Vec<SteamRelevantField>,
    pub relevant_game_properties: Vec<SteamRelevantField>,
    pub game_nodes: usize,
    pub excluded_categories: Vec<String>,
    pub issues: Vec<String>,
    pub fingerprint: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SteamRawFileEvidence {
    pub path: PathBuf,
    pub sha256: String,
    pub size: u64,
    pub modified_seconds: i64,
    pub modified_nanoseconds: i64,
    pub device: u64,
    pub inode: u64,
    pub uid: u32,
    pub mode: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SteamProtectedSemanticState {
    pub schema_version: u16,
    pub captured_at: u64,
    pub app_id: u32,
    pub config_raw: SteamRawFileEvidence,
    pub localconfig_raw: SteamRawFileEvidence,
    pub config: SteamFileSemantics,
    pub localconfig: SteamFileSemantics,
    pub semantic_fingerprint: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SteamProtectedStateAssessment {
    ReviewedAndAcceptable,
    RelevantConfigurationChanged,
    UnsafeLaunchOverride,
    CompatibilityMappingChanged,
    UnknownRelevantState,
    Unparseable,
    ActiveSteamWrite,
    NeedsEvidence,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SteamSemanticStability {
    Stable,
    RawChangedSemanticStable,
    SemanticChanged,
    Unknown,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamProtectedStateReview {
    pub schema_version: u16,
    pub before: SteamProtectedSemanticState,
    pub after: SteamProtectedSemanticState,
    pub observation_seconds: u64,
    pub stability: SteamSemanticStability,
    pub assessment: SteamProtectedStateAssessment,
    pub historical_writer: String,
    pub historical_exact_diff: String,
    pub evidence: Vec<String>,
}
fn hash(bytes: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(bytes.as_ref()))
}
fn issue(s: &'static str) -> io::Error {
    io::Error::other(s)
}
fn scalar(v: &KeyValuesValue) -> Option<&str> {
    match v {
        KeyValuesValue::Scalar(s) => Some(s),
        _ => None,
    }
}
fn sensitive(k: &str) -> bool {
    [
        "token",
        "ticket",
        "password",
        "secret",
        "credential",
        "auth",
        "decryption",
        "friends",
        "chat",
        "cloud",
    ]
    .iter()
    .any(|s| k.contains(s))
}
fn excluded(k: &str) -> bool {
    sensitive(k)
        || k.starts_with("4162040_eula_")
        || matches!(
            k,
            "depots"
                | "shadercachemanager"
                | "tools"
                | "badgedata"
                | "playtime2wks"
                | "lastplayed"
                | "playtime"
                | "playtime2weeks"
                | "playtimeforever"
                | "playtime_disconnected"
                | "autocloudenabled"
                | "cloud"
                | "eulaversion"
                | "mostrecent"
                | "lastupdated"
                | "playtimefrom"
                | "playtimeto"
                | "badgeprogress"
                | "newdlc"
                | "newdlcplaytime"
                | "uninstalldate"
                | "lastlaunch"
                | "lastlaunchtime"
                | "achievementprogress"
                | "inprogress"
                | "appticket"
                | "steamcontrollerconfigdetails"
        )
}
fn field(key: String, v: &KeyValuesValue, allow_literal: bool) -> SteamRelevantField {
    let value = scalar(v);
    let safe = value.filter(|s| {
        allow_literal
            && s.len() <= 128
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b))
    });
    SteamRelevantField {
        key,
        state: match value {
            Some("") => "empty",
            Some(_) => {
                if safe.is_some() {
                    "selected_literal"
                } else {
                    "redacted"
                }
            }
            None => "object_redacted",
        }
        .into(),
        safe_value: safe.map(str::to_owned),
        value_sha256: value.filter(|s| !s.is_empty()).map(hash),
    }
}
fn walk(
    entries: &[(String, KeyValuesValue)],
    context: &str,
    game: bool,
    out: &mut SteamFileSemantics,
    depth: usize,
) {
    if depth > 64 {
        out.issues.push("depth_limit".into());
        return;
    }
    let mut seen = BTreeSet::new();
    for (key, value) in entries {
        let k = key.to_ascii_lowercase();
        if k.starts_with("#") {
            out.issues.push("unsupported_keyvalues_directive".into());
            continue;
        }
        // Privacy exclusions are categories only: no unrelated path, value or content hash.
        if excluded(&k) {
            continue;
        }
        let relevant = game
            || k == APP
            || k == "compattoolmapping"
            || matches!(
                k.as_str(),
                "steamplay"
                    | "steamplayenabled"
                    | "steamplayforalltitles"
                    | "enablecompatibilitytools"
            );
        if (relevant
            || matches!(
                k.as_str(),
                "installconfigstore"
                    | "userlocalconfigstore"
                    | "software"
                    | "valve"
                    | "steam"
                    | "apps"
                    | "userappconfig"
            ))
            && !seen.insert(k.clone())
        {
            out.issues.push("duplicate_relevant_key".into());
        }
        if k == "compattoolmapping" {
            out.compat_tool_mapping_present = true;
            if let KeyValuesValue::Object(mapping) = value {
                let mut mapping_seen = BTreeSet::new();
                for (id, v) in mapping {
                    if id != APP && id != "0" {
                        continue;
                    }
                    if !mapping_seen.insert(id) {
                        out.issues.push("duplicate_mapping".into());
                    }
                    if id == APP {
                        out.game_mapping_present = true;
                    } else {
                        out.global_mapping_present = true;
                    }
                    let dest = if id == APP {
                        &mut out.game_mapping
                    } else {
                        &mut out.global_mapping
                    };
                    if let KeyValuesValue::Object(fields) = v {
                        let mut fields_seen = BTreeSet::new();
                        for (name, v) in fields {
                            let name = name.to_ascii_lowercase();
                            if !fields_seen.insert(name.clone()) {
                                out.issues.push("duplicate_mapping_field".into());
                            }
                            if !matches!(name.as_str(), "name" | "config" | "priority")
                                || scalar(v).is_none()
                            {
                                out.issues.push("unknown_mapping_field".into());
                            }
                            if sensitive(&name) {
                                out.issues.push("sensitive_relevant_field_excluded".into());
                                continue;
                            }
                            dest.push(field(
                                format!(
                                    "{context}/compattoolmapping/{id}/{}",
                                    if matches!(name.as_str(), "name" | "config" | "priority") {
                                        name.clone()
                                    } else {
                                        format!("unknown-key-{}", hash(&name))
                                    }
                                ),
                                v,
                                scalar(v).is_some_and(|s| match name.as_str() {
                                    "name" => s.starts_with("proton_") && s.len() < 64,
                                    "priority" => {
                                        s.len() < 8 && s.bytes().all(|b| b.is_ascii_digit())
                                    }
                                    "config" => matches!(s, "" | "0" | "1"),
                                    _ => false,
                                }),
                            ));
                        }
                    } else {
                        out.issues.push("mapping_not_object".into());
                    }
                }
            } else {
                out.issues.push("mapping_not_object".into());
            }
            continue;
        }
        if k == APP {
            if context.ends_with("/userappconfig") {
                match scalar(value).and_then(|s| crate::vdf::parse_binary_key_values_hex(s).ok()) {
                    Some(document) => {
                        if document.len() != 1 || document[0].0 != "internal" {
                            out.issues.push("unknown_userappconfig_root".into());
                            continue;
                        }
                        if let KeyValuesValue::Object(fields) = &document[0].1 {
                            let mut seen = BTreeSet::new();
                            for (name, v) in fields {
                                if !seen.insert(name.to_ascii_lowercase()) {
                                    out.issues.push("duplicate_userappconfig_field".into());
                                }
                                if name.eq_ignore_ascii_case("language")
                                    && scalar(v).is_some_and(|s| {
                                        matches!(
                                            s,
                                            "english"
                                                | "spanish"
                                                | "latam"
                                                | "schinese"
                                                | "tchinese"
                                                | "japanese"
                                                | "koreana"
                                                | "german"
                                                | "french"
                                                | "russian"
                                                | "portuguese"
                                                | "brazilian"
                                                | "italian"
                                                | ""
                                        )
                                    })
                                {
                                    out.relevant_game_properties.push(field(
                                        format!("{context}/4162040/internal/language"),
                                        v,
                                        true,
                                    ));
                                } else {
                                    out.issues.push("unknown_userappconfig_field".into());
                                }
                            }
                        } else {
                            out.issues.push("userappconfig_root_not_object".into());
                        }
                    }
                    None => out.issues.push("unparseable_userappconfig".into()),
                }
                continue;
            }
            out.game_nodes += 1;
            if let KeyValuesValue::Object(fields) = value {
                walk(
                    fields,
                    &format!("{context}/game/{APP}"),
                    true,
                    out,
                    depth + 1,
                );
            } else {
                out.issues.push("game_node_not_object".into());
            }
            continue;
        }
        if matches!(
            k.as_str(),
            "steamplay" | "steamplayenabled" | "steamplayforalltitles" | "enablecompatibilitytools"
        ) {
            match value {
                KeyValuesValue::Scalar(s) if matches!(s.as_str(), "0" | "1" | "") => out
                    .steam_play
                    .push(field(format!("{context}/{k}"), value, true)),
                _ => out.issues.push("unknown_steam_play_setting".into()),
            }
            continue;
        }
        if game {
            let name = format!(
                "{context}/{}",
                if matches!(
                    k.as_str(),
                    "launchoptions"
                        | "launchoption"
                        | "launchcommand"
                        | "executable"
                        | "command"
                        | "workingdirectory"
                        | "compattool"
                        | "compatibilitytool"
                        | "config"
                        | "launch"
                        | "properties"
                        | "selectedlaunchoption"
                        | "launchoptionindex"
                        | "compattoolname"
                        | "compattoolconfig"
                ) {
                    k.clone()
                } else {
                    format!("unknown-key-{}", hash(&k))
                }
            );
            if matches!(k.as_str(), "launchoptions" | "launchoption") {
                out.launch_options.push(field(name, value, false));
                continue;
            }
            if matches!(
                k.as_str(),
                "launchcommand" | "executable" | "command" | "workingdirectory"
            ) {
                out.launch_overrides.push(field(name, value, false));
                continue;
            }
            if let KeyValuesValue::Object(children) = value {
                walk(children, &name, true, out, depth + 1);
            } else {
                out.relevant_game_properties.push(field(name, value, false));
                out.issues.push("unknown_game_property".into());
            }
        } else if let KeyValuesValue::Object(children) = value {
            // No unrelated node names are retained. Fixed structural vocabulary only.
            let structural = if matches!(
                k.as_str(),
                "installconfigstore"
                    | "userlocalconfigstore"
                    | "software"
                    | "valve"
                    | "steam"
                    | "apps"
                    | "appconfig"
                    | "config"
                    | "userappconfig"
            ) {
                k.as_str()
            } else {
                "excluded-node"
            };
            walk(
                children,
                &format!("{context}/{structural}"),
                false,
                out,
                depth + 1,
            );
        } else if scalar(value).is_some_and(|s| {
            let s = s.to_ascii_lowercase().replace('\\', "/");
            s.contains("zenless zone zero")
                || s.contains("zenlesszonezero.exe")
                || s.contains("compatdata/4162040")
                || s == APP
        }) {
            // References outside recognized game schema cannot be assumed benign.
            out.relevant_game_properties.push(field(
                format!("{context}/reference-key-{}", hash(&k)),
                value,
                false,
            ));
            out.issues.push("unclassified_game_reference".into());
        }
    }
}
pub fn extract_zzz_steam_semantics(input: &str, local: bool) -> io::Result<SteamFileSemantics> {
    let entries = parse_key_values_document(input).map_err(|_| issue("unparseable_keyvalues"))?;
    let root = if local {
        "UserLocalConfigStore"
    } else {
        "InstallConfigStore"
    };
    if entries
        .iter()
        .filter(|(k, _)| k.eq_ignore_ascii_case(root))
        .count()
        != 1
        || !entries
            .iter()
            .any(|(k, v)| k.eq_ignore_ascii_case(root) && matches!(v, KeyValuesValue::Object(_)))
    {
        return Err(issue("unexpected_steam_document_root"));
    }
    let mut out=SteamFileSemantics{document_kind:if local{"localconfig"}else{"config"}.into(),compat_tool_mapping_present:false,game_mapping_present:false,global_mapping_present:false,game_mapping:vec![],global_mapping:vec![],launch_options:vec![],launch_overrides:vec![],steam_play:vec![],relevant_game_properties:vec![],game_nodes:0,excluded_categories:vec!["credentials/tokens/account/friends/chat/cloud: contents excluded".into(),"game usage/recent-play/UI metadata: values excluded".into(),"other games and unrelated client state: contents excluded; direct game references checked outside sensitive sections".into()],issues:vec![],fingerprint:String::new()};
    walk(&entries, "root", false, &mut out, 0);
    for fields in [
        &mut out.game_mapping,
        &mut out.global_mapping,
        &mut out.launch_options,
        &mut out.launch_overrides,
        &mut out.steam_play,
        &mut out.relevant_game_properties,
    ] {
        fields.sort_by(|a, b| a.key.cmp(&b.key));
    }
    out.issues.sort();
    out.issues.dedup();
    out.fingerprint = hash(serde_json::to_vec(&out)?);
    Ok(out)
}

fn read_document(root: &Path, relative: &str) -> io::Result<(SteamRawFileEvidence, String)> {
    if relative != "config/config.vdf" && relative != LOCAL {
        return Err(issue("steam_semantic_path_rejected"));
    }
    let canonical = root.canonicalize()?;
    if canonical != root {
        return Err(issue("steam_root_must_be_canonical"));
    }
    let mut parent = canonical.clone();
    let mut dir = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
        .open(&canonical)?;
    let parts: Vec<_> = relative.split('/').collect();
    for part in &parts[..parts.len() - 1] {
        parent.push(part);
        let name = std::ffi::CString::new(*part).map_err(|_| issue("invalid_steam_path"))?;
        let fd = unsafe {
            libc::openat(
                dir.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            return Err(issue("unsafe_steam_parent"));
        }
        dir = unsafe { std::fs::File::from_raw_fd(fd) };
        let m = dir.metadata()?;
        if !m.is_dir() || m.file_type().is_symlink() || m.uid() != unsafe { libc::getuid() } {
            return Err(issue("unsafe_steam_parent"));
        }
    }
    let path = canonical.join(relative);
    let name = std::ffi::CString::new(*parts.last().ok_or_else(|| issue("invalid_steam_path"))?)
        .map_err(|_| issue("invalid_steam_path"))?;
    let fd = unsafe {
        libc::openat(
            dir.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY
                | libc::O_NOFOLLOW
                | libc::O_NONBLOCK
                | libc::O_NOATIME
                | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(issue("unsafe_or_unreadable_steam_file"));
    }
    let file = unsafe { std::fs::File::from_raw_fd(fd) };
    let before = file.metadata()?;
    if !before.is_file() || before.len() > LIMIT || before.uid() != unsafe { libc::getuid() } {
        return Err(issue("unsafe_or_oversize_steam_file"));
    }
    let mut bytes = Vec::new();
    (&file).take(LIMIT + 1).read_to_end(&mut bytes)?;
    let after = file.metadata()?;
    let path_after = fs::symlink_metadata(&path)?;
    let stamp = |m: &fs::Metadata| {
        (
            m.dev(),
            m.ino(),
            m.len(),
            m.mtime(),
            m.mtime_nsec(),
            m.ctime(),
            m.ctime_nsec(),
            m.mode(),
            m.uid(),
        )
    };
    if bytes.len() as u64 > LIMIT
        || stamp(&before) != stamp(&after)
        || stamp(&before) != stamp(&path_after)
        || path.canonicalize()? != path
    {
        return Err(issue("active_steam_write_or_path_changed"));
    }
    let raw = SteamRawFileEvidence {
        path,
        sha256: hash(&bytes),
        size: before.len(),
        modified_seconds: before.mtime(),
        modified_nanoseconds: before.mtime_nsec(),
        device: before.dev(),
        inode: before.ino(),
        uid: before.uid(),
        mode: before.mode(),
    };
    Ok((
        raw,
        String::from_utf8(bytes).map_err(|_| issue("steam_vdf_not_utf8"))?,
    ))
}
pub fn inspect_zzz_steam_semantic_state(root: &Path) -> io::Result<SteamProtectedSemanticState> {
    let (config_raw, config) = read_document(root, "config/config.vdf")?;
    let (localconfig_raw, localconfig) = read_document(root, LOCAL)?;
    let config = extract_zzz_steam_semantics(&config, false)?;
    let localconfig = extract_zzz_steam_semantics(&localconfig, true)?;
    let semantic_fingerprint = hash(serde_json::to_vec(&(
        STEAM_SEMANTIC_POLICY,
        4162040,
        &config.fingerprint,
        &localconfig.fingerprint,
    ))?);
    Ok(SteamProtectedSemanticState {
        schema_version: STEAM_SEMANTIC_POLICY,
        captured_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| issue("clock_unavailable"))?
            .as_secs(),
        app_id: 4162040,
        config_raw,
        localconfig_raw,
        config,
        localconfig,
        semantic_fingerprint,
    })
}
pub fn assess_steam_semantics(
    state: &SteamProtectedSemanticState,
) -> SteamProtectedStateAssessment {
    if state.app_id != 4162040 || state.schema_version != STEAM_SEMANTIC_POLICY {
        return SteamProtectedStateAssessment::UnknownRelevantState;
    }
    let files = [&state.config, &state.localconfig];
    if files.iter().any(|s| !s.issues.is_empty()) {
        return SteamProtectedStateAssessment::UnknownRelevantState;
    }
    if files.iter().any(|s| {
        s.launch_options
            .iter()
            .chain(&s.launch_overrides)
            .any(|f| f.state != "empty")
    }) {
        return SteamProtectedStateAssessment::UnsafeLaunchOverride;
    }
    // New mappings are not inherently unsafe, but require separate explicit build approval.
    if files
        .iter()
        .any(|s| s.game_mapping_present || s.global_mapping_present)
    {
        return SteamProtectedStateAssessment::CompatibilityMappingChanged;
    }
    if files.iter().any(|s| !s.steam_play.is_empty()) {
        return SteamProtectedStateAssessment::NeedsEvidence;
    }
    SteamProtectedStateAssessment::ReviewedAndAcceptable
}
pub fn compare_steam_stability(
    a: &SteamProtectedSemanticState,
    b: &SteamProtectedSemanticState,
) -> SteamSemanticStability {
    if a.schema_version != b.schema_version
        || a.app_id != b.app_id
        || a.schema_version != STEAM_SEMANTIC_POLICY
        || a.app_id != 4162040
        || a.config_raw.path != b.config_raw.path
        || a.localconfig_raw.path != b.localconfig_raw.path
        || a.config_raw.device != b.config_raw.device
        || a.localconfig_raw.device != b.localconfig_raw.device
        || a.config_raw.uid != b.config_raw.uid
        || a.localconfig_raw.uid != b.localconfig_raw.uid
        || a.config_raw.mode != b.config_raw.mode
        || a.localconfig_raw.mode != b.localconfig_raw.mode
    {
        return SteamSemanticStability::Unknown;
    }
    if a.semantic_fingerprint != b.semantic_fingerprint {
        return SteamSemanticStability::SemanticChanged;
    }
    if a.config_raw != b.config_raw || a.localconfig_raw != b.localconfig_raw {
        SteamSemanticStability::RawChangedSemanticStable
    } else {
        SteamSemanticStability::Stable
    }
}
pub fn review_zzz_steam_state(root: &Path, seconds: u64) -> io::Result<SteamProtectedStateReview> {
    if !(1..=30).contains(&seconds) {
        return Err(issue("invalid_observation_interval"));
    }
    let before = inspect_zzz_steam_semantic_state(root)?;
    std::thread::sleep(std::time::Duration::from_secs(seconds));
    let after = inspect_zzz_steam_semantic_state(root)?;
    let stability = compare_steam_stability(&before, &after);
    let assessment = match stability {
        SteamSemanticStability::SemanticChanged => {
            SteamProtectedStateAssessment::RelevantConfigurationChanged
        }
        SteamSemanticStability::Unknown => SteamProtectedStateAssessment::ActiveSteamWrite,
        _ => assess_steam_semantics(&after),
    };
    Ok(SteamProtectedStateReview{schema_version:STEAM_SEMANTIC_POLICY,before,after,observation_seconds:seconds,stability,assessment,historical_writer:"UNKNOWN".into(),historical_exact_diff:"NOT RECONSTRUCTABLE from hash-only historical baselines".into(),evidence:vec!["Current selected semantics reviewed, not a reconstructed historical key-level diff.".into(),"Absent static mappings preserve configured selection Unknown; previous observed Proton remains separate.".into(),"Full-file hashes/metadata retained diagnostically; only approved relevant semantics govern these two files. All other protected paths remain strict.".into()]})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cfg(body: &str) -> String {
        format!("\"InstallConfigStore\" {{ \"Software\" {{ \"Valve\" {{ \"Steam\" {{ {body} }} }} }} }}")
    }
    fn local(body: &str) -> String {
        format!("\"UserLocalConfigStore\" {{ \"Software\" {{ \"Valve\" {{ \"Steam\" {{ \"apps\" {{ \"4162040\" {{ {body} }} }} }} }} }} }}")
    }
    #[test]
    fn mappings_are_separate_and_explicit() {
        let s=extract_zzz_steam_semantics(&cfg(r#""CompatToolMapping" { "4162040" { "name" "proton_experimental" "priority" "250" } "0" { "name" "proton_9" } "10" { "name" "unrelated" } }"#),false).unwrap();
        assert!(s.compat_tool_mapping_present);
        assert_eq!(s.game_mapping.len(), 2);
        assert_eq!(s.global_mapping.len(), 1);
        assert!(!serde_json::to_string(&s).unwrap().contains("unrelated\""));
    }
    #[test]
    fn launch_options_and_overrides_redacted() {
        let s = extract_zzz_steam_semantics(
            &local(r#""LaunchOptions" "PASSWORD=secret %command%" "Executable" "private.exe""#),
            true,
        )
        .unwrap();
        assert_eq!(s.launch_options.len(), 1);
        assert_eq!(s.launch_overrides.len(), 1);
        let json = serde_json::to_string(&s).unwrap();
        assert!(!json.contains("secret"));
        assert!(!json.contains("private.exe"));
        assert_eq!(s.launch_options[0].state, "redacted");
    }
    #[test]
    fn unrelated_changes_do_not_change_fingerprint() {
        let a = extract_zzz_steam_semantics(
            &local(r#""LastPlayed" "1" "Playtime" "12" "Cloud" { "private" "secret" }"#),
            true,
        )
        .unwrap();
        let b = extract_zzz_steam_semantics(
            &local(r#""LastPlayed" "2" "Playtime" "13" "Cloud" { "private" "changed" }"#),
            true,
        )
        .unwrap();
        assert_eq!(a.fingerprint, b.fingerprint);
        assert!(!serde_json::to_string(&a).unwrap().contains("secret"));
    }
    #[test]
    fn relevant_change_and_unknown_are_not_hidden() {
        let a = extract_zzz_steam_semantics(&local(""), true).unwrap();
        let b = extract_zzz_steam_semantics(&local(r#""LaunchOptions" "-foo""#), true).unwrap();
        assert_ne!(a.fingerprint, b.fingerprint);
        let c = extract_zzz_steam_semantics(&local(r#""Mystery" "1""#), true).unwrap();
        assert!(!c.issues.is_empty());
    }
    #[test]
    fn malformed_duplicate_and_path_reference_fail_closed() {
        assert!(extract_zzz_steam_semantics("{", false).is_err());
        let d = extract_zzz_steam_semantics(
            &local(r#""LaunchOptions" "" "launchoptions" "bad""#),
            true,
        )
        .unwrap();
        assert!(d.issues.contains(&"duplicate_relevant_key".into()));
        let d =
            extract_zzz_steam_semantics(&cfg(r#""Unknown" "Z:/compatdata/4162040/pfx""#), false)
                .unwrap();
        assert!(d.issues.contains(&"unclassified_game_reference".into()));
    }

    struct Synthetic {
        root: PathBuf,
    }
    impl Synthetic {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "lxmi-steam-semantics-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(root.join("config")).unwrap();
            fs::create_dir_all(root.join("userdata/1416423935/config")).unwrap();
            fs::write(
                root.join("config/config.vdf"),
                cfg(r#""CompatToolMapping" { "10" { "name" "other" } }"#),
            )
            .unwrap();
            fs::write(root.join(LOCAL), local(r#""LastPlayed" "1""#)).unwrap();
            Self { root }
        }
    }
    impl Drop for Synthetic {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }
    #[test]
    fn controlled_stability_and_raw_unrelated_change() {
        let f = Synthetic::new();
        let review = review_zzz_steam_state(&f.root, 1).unwrap();
        assert_eq!(
            review.assessment,
            SteamProtectedStateAssessment::ReviewedAndAcceptable
        );
        assert_eq!(review.stability, SteamSemanticStability::Stable);
        fs::write(f.root.join(LOCAL), local(r#""LastPlayed" "2""#)).unwrap();
        let current = inspect_zzz_steam_semantic_state(&f.root).unwrap();
        assert_ne!(
            review.after.localconfig_raw.sha256,
            current.localconfig_raw.sha256
        );
        assert_eq!(
            compare_steam_stability(&review.after, &current),
            SteamSemanticStability::RawChangedSemanticStable
        );
        fs::write(f.root.join(LOCAL), local(r#""LaunchOptions" "-unsafe""#)).unwrap();
        let current = inspect_zzz_steam_semantic_state(&f.root).unwrap();
        assert_eq!(
            compare_steam_stability(&review.after, &current),
            SteamSemanticStability::SemanticChanged
        );
        assert_eq!(
            assess_steam_semantics(&current),
            SteamProtectedStateAssessment::UnsafeLaunchOverride
        );
    }
    #[test]
    fn mappings_unknown_fields_and_malformed_documents_block() {
        let f = Synthetic::new();
        fs::write(
            f.root.join("config/config.vdf"),
            cfg(r#""CompatToolMapping" { "4162040" { "name" "proton_experimental" } }"#),
        )
        .unwrap();
        assert_eq!(
            assess_steam_semantics(&inspect_zzz_steam_semantic_state(&f.root).unwrap()),
            SteamProtectedStateAssessment::CompatibilityMappingChanged
        );
        fs::write(f.root.join(LOCAL), local(r#""Mystery" "data""#)).unwrap();
        assert_eq!(
            assess_steam_semantics(&inspect_zzz_steam_semantic_state(&f.root).unwrap()),
            SteamProtectedStateAssessment::UnknownRelevantState
        );
        fs::write(f.root.join(LOCAL), "{broken").unwrap();
        assert!(inspect_zzz_steam_semantic_state(&f.root).is_err());
    }
    #[test]
    fn private_sections_excluded_and_binary_user_config_checked() {
        let blob = b"\0internal\0\x01language\0english\0\x08\x08"
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let doc = format!(
            r#""UserLocalConfigStore" {{ "apptickets" {{ "4162040" "secret-ticket" }} "depots" {{ "4162040" {{ "DecryptionKey" "secret-key" }} }} "UserAppConfig" {{ "4162040" "{blob}" }} }}"#
        );
        let s = extract_zzz_steam_semantics(&doc, true).unwrap();
        assert!(s.issues.is_empty());
        assert_eq!(
            s.relevant_game_properties[0].safe_value.as_deref(),
            Some("english")
        );
        let json = serde_json::to_string(&s).unwrap();
        assert!(!json.contains("secret-ticket"));
        assert!(!json.contains("secret-key"));
        let bad = doc.replace(&blob, "deadbeef");
        assert!(extract_zzz_steam_semantics(&bad, true)
            .unwrap()
            .issues
            .contains(&"unparseable_userappconfig".into()));
    }
    #[test]
    fn readers_reject_symlink_parent_file_paths_and_oversize() {
        let f = Synthetic::new();
        assert!(read_document(&f.root, "../config/config.vdf").is_err());
        let path = f.root.join(LOCAL);
        fs::remove_file(&path).unwrap();
        std::os::unix::fs::symlink(f.root.join("config/config.vdf"), &path).unwrap();
        assert!(inspect_zzz_steam_semantic_state(&f.root).is_err());
        fs::remove_file(&path).unwrap();
        fs::write(&path, local("")).unwrap();
        let file = OpenOptions::new().write(true).open(&path).unwrap();
        file.set_len(LIMIT + 1).unwrap();
        assert!(inspect_zzz_steam_semantic_state(&f.root).is_err());
        fs::remove_dir_all(f.root.join("userdata/1416423935/config")).unwrap();
        std::os::unix::fs::symlink(
            f.root.join("config"),
            f.root.join("userdata/1416423935/config"),
        )
        .unwrap();
        assert!(inspect_zzz_steam_semantic_state(&f.root).is_err());
    }
}
