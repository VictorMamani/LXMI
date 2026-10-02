//! Read-only semantic diagnostics. Findings never authorize prefix entry or a loader.
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const LIMIT: usize = 8 * 1024 * 1024;
type Values = BTreeMap<String, String>;
type Registry = BTreeMap<String, Values>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistryDiagnosisError {
    TooLarge,
    IntegrityMismatch,
    UnsupportedFormat,
    DuplicateSection,
    DuplicateValue,
}
#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct RegistryChangeDiagnosis {
    pub content_changed: bool,
    pub changed_sections: usize,
    pub font_sections: usize,
    pub font_values_added: usize,
    pub font_values_removed: usize,
    pub font_values_modified: usize,
    pub run_host_to_native_font_paths: usize,
    pub native_to_run_host_font_paths: usize,
    pub rewritten_font_filename_matches: usize,
    pub device_container_id_sections: usize,
    pub empty_network_cards_key_removed: bool,
    pub unknown_sections: usize,
    /// Diagnosis is not permission; the original UnexpectedMutation remains unchanged.
    pub execution_authorized: bool,
}

fn parse(input: &[u8]) -> Result<Registry, RegistryDiagnosisError> {
    use RegistryDiagnosisError::*;
    if input.len() > LIMIT {
        return Err(TooLarge);
    }
    let text = std::str::from_utf8(input).map_err(|_| UnsupportedFormat)?;
    let mut lines = text.lines();
    if lines.next() != Some("WINE REGISTRY Version 2") {
        return Err(UnsupportedFormat);
    }
    let mut out = Registry::new();
    out.insert("\0header".into(), Values::new());
    let mut section: Option<String> = None;
    let mut pending = String::new();
    for line in lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with(";;") || line.starts_with("#time=") {
            continue;
        }
        if line == "#arch=win64" || line == "#arch=win32" {
            if out
                .get_mut("\0header")
                .ok_or(UnsupportedFormat)?
                .insert("arch".into(), line.into())
                .is_some()
            {
                return Err(UnsupportedFormat);
            }
            continue;
        }
        if line == "#link" && pending.is_empty() {
            let key = section.as_ref().ok_or(UnsupportedFormat)?;
            if out
                .get_mut(key)
                .ok_or(UnsupportedFormat)?
                .insert("\0link".into(), "true".into())
                .is_some()
            {
                return Err(UnsupportedFormat);
            }
            continue;
        }
        if line.starts_with('[') && pending.is_empty() {
            let end = line.rfind(']').ok_or(UnsupportedFormat)?;
            let suffix = line[end + 1..].trim();
            if !suffix.is_empty() && !suffix.bytes().all(|b| b.is_ascii_digit()) {
                return Err(UnsupportedFormat);
            }
            let key = line[1..end].replace("\\\\", "\\");
            if key.is_empty() || key.contains('\0') {
                return Err(UnsupportedFormat);
            }
            if out.insert(key.clone(), Values::new()).is_some() {
                return Err(DuplicateSection);
            }
            section = Some(key);
            continue;
        }
        if line.starts_with('#') {
            return Err(UnsupportedFormat);
        }
        pending.push_str(line);
        if pending.ends_with('\\') {
            pending.pop();
            continue;
        }
        let key = section.as_ref().ok_or(UnsupportedFormat)?;
        // '=' inside quoted names is not a delimiter.
        let mut quoted = false;
        let mut escaped = false;
        let delimiter = pending
            .char_indices()
            .find_map(|(i, c)| {
                if escaped {
                    escaped = false;
                    return None;
                }
                if c == '\\' {
                    escaped = true;
                    return None;
                }
                if c == '"' {
                    quoted = !quoted;
                }
                (c == '=' && !quoted).then_some(i)
            })
            .ok_or(UnsupportedFormat)?;
        let name = &pending[..delimiter];
        if !(name == "@" || (name.starts_with('"') && name.ends_with('"'))) {
            return Err(UnsupportedFormat);
        }
        let value = &pending[delimiter + 1..];
        if value.is_empty() {
            return Err(UnsupportedFormat);
        }
        if out
            .get_mut(key)
            .ok_or(UnsupportedFormat)?
            .insert(name.into(), value.into())
            .is_some()
        {
            return Err(DuplicateValue);
        }
        pending.clear();
    }
    if !pending.is_empty() {
        return Err(UnsupportedFormat);
    }
    Ok(out)
}

fn string_value(value: &str) -> Option<String> {
    let value = value.strip_prefix('"')?.strip_suffix('"')?;
    Some(value.replace("\\\\", "\\").replace("\\\"", "\""))
}
fn is_fonts(key: &str) -> bool {
    matches!(
        key,
        "Software\\Microsoft\\Windows NT\\CurrentVersion\\Fonts"
            | "Software\\Microsoft\\Windows\\CurrentVersion\\Fonts"
            | "Software\\Wine\\Fonts\\External Fonts"
    )
}
fn is_device(key: &str) -> bool {
    let parts: Vec<_> = key.split('\\').collect();
    parts.len() == 6
        && parts[0] == "System"
        && parts[1] == "ControlSet001"
        && parts[2] == "Enum"
        && matches!(parts[3], "HID" | "WINEBUS")
        && !parts[4].is_empty()
        && !parts[5].is_empty()
}
/// Expected hashes must come from the reviewed before/after receipt, not a new baseline.
/// Values and arbitrary registry key names are never returned or logged.
pub fn diagnose_registry_change(
    before: &[u8],
    after: &[u8],
    expected_before_sha256: &str,
    expected_after_sha256: &str,
) -> Result<RegistryChangeDiagnosis, RegistryDiagnosisError> {
    if before.len() > LIMIT || after.len() > LIMIT {
        return Err(RegistryDiagnosisError::TooLarge);
    }
    if format!("{:x}", Sha256::digest(before)) != expected_before_sha256
        || format!("{:x}", Sha256::digest(after)) != expected_after_sha256
    {
        return Err(RegistryDiagnosisError::IntegrityMismatch);
    }
    let old = parse(before)?;
    let new = parse(after)?;
    let mut report = RegistryChangeDiagnosis {
        content_changed: before != after,
        ..Default::default()
    };
    let keys: BTreeSet<_> = old.keys().chain(new.keys()).collect();
    for key in keys {
        let a = old.get(key);
        let b = new.get(key);
        if a == b {
            continue;
        }
        report.changed_sections += 1;
        if is_fonts(key) {
            report.font_sections += 1;
            let empty = Values::new();
            let a = a.unwrap_or(&empty);
            let b = b.unwrap_or(&empty);
            for name in a.keys().chain(b.keys()).collect::<BTreeSet<_>>() {
                match (a.get(name), b.get(name)) {
                    (None, Some(_)) => report.font_values_added += 1,
                    (Some(_), None) => report.font_values_removed += 1,
                    (Some(x), Some(y)) if x != y => {
                        report.font_values_modified += 1;
                        if let (Some(x), Some(y)) = (string_value(x), string_value(y)) {
                            if x.starts_with("Z:\\run\\host\\")
                                && y.starts_with("Z:\\usr\\share\\fonts\\")
                            {
                                report.run_host_to_native_font_paths += 1;
                                if x.rsplit('\\').next() == y.rsplit('\\').next() {
                                    report.rewritten_font_filename_matches += 1;
                                }
                            } else if x.starts_with("Z:\\usr\\share\\fonts\\")
                                && y.starts_with("Z:\\run\\host\\")
                            {
                                report.native_to_run_host_font_paths += 1;
                                if x.rsplit('\\').next() == y.rsplit('\\').next() {
                                    report.rewritten_font_filename_matches += 1;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else if is_device(key) && a.is_some() && b.is_some() {
            let a = a.unwrap();
            let b = b.unwrap();
            let changed: BTreeSet<_> = a
                .keys()
                .chain(b.keys())
                .filter(|name| a.get(*name) != b.get(*name))
                .collect();
            if changed.len() == 1 && changed.contains(&"\"ContainerId\"".to_string()) {
                report.device_container_id_sections += 1;
            } else {
                report.unknown_sections += 1;
            }
        } else if key == "Software\\Microsoft\\Windows NT\\CurrentVersion\\NetworkCards"
            && a.is_some_and(Values::is_empty)
            && b.is_none()
        {
            report.empty_network_cards_key_removed = true;
        } else {
            report.unknown_sections += 1;
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(a: &str, b: &str) -> Result<RegistryChangeDiagnosis, RegistryDiagnosisError> {
        diagnose_registry_change(
            a.as_bytes(),
            b.as_bytes(),
            &format!("{:x}", Sha256::digest(a)),
            &format!("{:x}", Sha256::digest(b)),
        )
    }
    #[test]
    fn font_mount_rewrite_is_diagnostic_never_authorization() {
        let a = "WINE REGISTRY Version 2\n[Software\\\\Wine\\\\Fonts\\\\External Fonts] 1\n\"Font\"=\"Z:\\\\run\\\\host\\\\fonts\\\\font.ttf\"\n";
        let b = a.replace("run\\\\host\\\\fonts", "usr\\\\share\\\\fonts");
        let r = run(a, &b).unwrap();
        assert_eq!(r.font_values_modified, 1);
        assert_eq!(r.run_host_to_native_font_paths, 1);
        assert_eq!(r.rewritten_font_filename_matches, 1);
        assert!(!r.execution_authorized);
    }
    #[test]
    fn timestamps_are_distinct_from_semantic_changes() {
        let r = run(
            "WINE REGISTRY Version 2\n[A] 1\n#time=123\n\"x\"=dword:00000001\n",
            "WINE REGISTRY Version 2\n[A] 2\n#time=456\n\"x\"=dword:00000001\n",
        )
        .unwrap();
        assert!(r.content_changed);
        assert_eq!(r.changed_sections, 0);
        assert!(!r.execution_authorized);
    }
    #[test]
    fn reverse_mount_transition_remains_diagnostic_only() {
        let a = "WINE REGISTRY Version 2\n[Software\\\\Wine\\\\Fonts\\\\External Fonts]\n\"Font\"=\"Z:\\\\usr\\\\share\\\\fonts\\\\font.ttf\"\n";
        let b = a.replace("usr\\\\share\\\\fonts", "run\\\\host\\\\fonts");
        let r = run(a, &b).unwrap();
        assert_eq!(r.native_to_run_host_font_paths, 1);
        assert_eq!(r.run_host_to_native_font_paths, 0);
        assert_eq!(r.rewritten_font_filename_matches, 1);
        assert!(!r.execution_authorized);
    }
    #[test]
    fn malformed_duplicate_and_incomplete_inputs_fail_closed() {
        for (s, e) in [
            ("bad", RegistryDiagnosisError::UnsupportedFormat),
            (
                "WINE REGISTRY Version 2\n[A]\n[A]\n",
                RegistryDiagnosisError::DuplicateSection,
            ),
            (
                "WINE REGISTRY Version 2\n[A]\n\"x\"=1\n\"x\"=2\n",
                RegistryDiagnosisError::DuplicateValue,
            ),
            (
                "WINE REGISTRY Version 2\n[A]\n\"x\"=hex:01,\\\n",
                RegistryDiagnosisError::UnsupportedFormat,
            ),
        ] {
            assert_eq!(run(s, s).unwrap_err(), e);
        }
    }
    #[test]
    fn hash_mismatch_rejects_changed_source() {
        assert_eq!(
            diagnose_registry_change(b"a", b"b", "bad", "bad").unwrap_err(),
            RegistryDiagnosisError::IntegrityMismatch
        );
    }
    #[test]
    fn container_id_only_is_distinct_from_other_device_changes() {
        let a = "WINE REGISTRY Version 2\n[System\\\\ControlSet001\\\\Enum\\\\HID\\\\Fixture\\\\Instance] 1\n\"ContainerId\"=\"old\"\n";
        let b = a.replace("old", "new");
        assert_eq!(run(a, &b).unwrap().device_container_id_sections, 1);
        assert_eq!(
            run(a, &(b + "\"Service\"=\"changed\"\n"))
                .unwrap()
                .unknown_sections,
            1
        );
    }
    #[test]
    fn unknown_values_never_leak_to_report() {
        let a = "WINE REGISTRY Version 2\n[Private\\\\User]\n\"Password\"=\"sensitive-before\"\n";
        let b = a.replace("before", "after");
        let r = run(a, &b).unwrap();
        assert_eq!(r.unknown_sections, 1);
        let text = serde_json::to_string(&r).unwrap();
        assert!(!text.contains("sensitive"));
        assert!(!text.contains("Private"));
    }
    #[test]
    fn multiline_hex_and_equals_inside_value_names_parse() {
        let a = "WINE REGISTRY Version 2\n[A]\n\"x=y\"=hex:01,\\\n02\n";
        assert_eq!(run(a, a).unwrap().changed_sections, 0);
    }
    #[test]
    fn font_addition_removal_and_empty_key_removal_are_explicit() {
        let a="WINE REGISTRY Version 2\n[Software\\\\Wine\\\\Fonts\\\\External Fonts]\n\"old\"=\"a\"\n[Software\\\\Microsoft\\\\Windows NT\\\\CurrentVersion\\\\NetworkCards]\n";
        let b="WINE REGISTRY Version 2\n[Software\\\\Wine\\\\Fonts\\\\External Fonts]\n\"new\"=\"b\"\n";
        let r = run(a, b).unwrap();
        assert_eq!(r.font_values_added, 1);
        assert_eq!(r.font_values_removed, 1);
        assert!(r.empty_network_cards_key_removed);
        assert!(!r.execution_authorized);
    }
    #[test]
    fn registry_link_marker_and_architecture_are_not_discarded() {
        let a = "WINE REGISTRY Version 2\n#arch=win64\n[A]\n#link\n@=\"target\"\n";
        assert_eq!(run(a, a).unwrap().changed_sections, 0);
        assert_eq!(
            run(a, &a.replace("#arch=win64", "#arch=win32"))
                .unwrap()
                .unknown_sections,
            1
        );
        assert_eq!(
            run(a, &a.replace("#link\n", "")).unwrap().unknown_sections,
            1
        );
    }
    #[test]
    fn oversized_data_is_rejected_before_hashing() {
        assert_eq!(
            diagnose_registry_change(&vec![0; LIMIT + 1], b"", "", "").unwrap_err(),
            RegistryDiagnosisError::TooLarge
        );
    }
    #[test]
    fn nonempty_network_key_removal_stays_unknown() {
        let a="WINE REGISTRY Version 2\n[Software\\\\Microsoft\\\\Windows NT\\\\CurrentVersion\\\\NetworkCards]\n\"x\"=dword:00000001\n";
        let r = run(a, "WINE REGISTRY Version 2\n").unwrap();
        assert!(!r.empty_network_cards_key_removed);
        assert_eq!(r.unknown_sections, 1);
        assert!(!r.execution_authorized);
    }
}
