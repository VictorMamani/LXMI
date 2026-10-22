//! Minimal ordered reader for 3DMigoto INI files. Sections keep their raw lines
//! because command lists depend on order and repeated keys.

pub(crate) struct Section {
    pub name: String,
    pub lines: Vec<String>,
}

pub(crate) fn parse(text: &str) -> Vec<Section> {
    let mut sections: Vec<Section> = vec![];
    for raw in text.trim_start_matches('\u{feff}').lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            sections.push(Section {
                name: name.trim().to_owned(),
                lines: vec![],
            });
        } else if let Some(section) = sections.last_mut() {
            section.lines.push(line.to_owned());
        }
    }
    sections
}

pub(crate) fn split_kv(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once('=')?;
    Some((key.trim(), value.trim()))
}

/// `[Constants]` defaults (`global persist $var = 1`), used to pick the
/// default branch of toggles.
pub(crate) fn constants(sections: &[Section]) -> Vec<(String, String)> {
    let mut out = vec![];
    for section in sections
        .iter()
        .filter(|s| s.name.eq_ignore_ascii_case("Constants"))
    {
        for line in &section.lines {
            let Some((key, value)) = split_kv(line) else {
                continue;
            };
            let var = key
                .split_whitespace()
                .last()
                .and_then(|word| word.strip_prefix('$'));
            if let Some(var) = var {
                out.push((var.to_lowercase(), value.to_owned()));
            }
        }
    }
    out
}

/// Evaluates simple conditions (`$var == 1`, `$var`, joined by `&&`).
/// Anything else counts as true so a mod still renders its default look.
pub(crate) fn condition(expr: &str, defaults: &[(String, String)]) -> bool {
    expr.split("&&").all(|part| {
        let part = part
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')')
            .trim();
        let lookup = |var: &str| {
            defaults
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(var))
                .map(|(_, value)| value.as_str())
        };
        if let Some((left, right)) = part.split_once("==") {
            let Some(var) = left.trim().strip_prefix('$') else {
                return true;
            };
            return lookup(var).is_none_or(|value| value == right.trim());
        }
        if let Some(var) = part.strip_prefix('$') {
            if var.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return lookup(var).is_none_or(|value| !matches!(value, "0" | "" | "false"));
            }
        }
        true
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_section_order_and_skips_comments() {
        let sections = parse("\u{feff}; header\n[A]\nx = 1\n\n[B]\n; c\ny=2\nz = 3\n");
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].name, "A");
        assert_eq!(sections[1].lines, ["y=2", "z = 3"]);
    }

    #[test]
    fn evaluates_toggle_defaults() {
        let sections = parse("[Constants]\nglobal persist $swap = 1\nglobal $off = 0\n");
        let defaults = constants(&sections);
        assert!(condition("$swap == 1", &defaults));
        assert!(!condition("$swap == 0", &defaults));
        assert!(!condition("$off", &defaults));
        assert!(!condition("$swap == 1 && $off == 1", &defaults));
        assert!(condition("$unknown == 3", &defaults));
    }
}
