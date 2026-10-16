//! `[Loader]` identity of the launch copy. 3DMigoto's DllMain accepts the process
//! named by `loader` and the one named by `target`; every other process rejects it.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    MissingLoaderSection,
    LoaderEntries(usize),
    TargetEntries(usize),
    UnexpectedTarget(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingLoaderSection => write!(f, "d3dx.ini no tiene sección [Loader]."),
            Self::LoaderEntries(n) => write!(
                f,
                "d3dx.ini debe tener un único `loader` en [Loader]; hay {n}."
            ),
            Self::TargetEntries(n) => write!(
                f,
                "d3dx.ini debe tener un único `target` en [Loader]; hay {n}."
            ),
            Self::UnexpectedTarget(t) => write!(
                f,
                "El `target` de d3dx.ini ({t}) no es el ejecutable del juego."
            ),
        }
    }
}

fn key_value(line: &[u8]) -> Option<(String, String)> {
    let text = String::from_utf8_lossy(line);
    let text = text.trim_end_matches(['\r', '\n']);
    if text.trim_start().starts_with(';') {
        return None;
    }
    let (key, value) = text.split_once('=')?;
    Some((key.trim().to_ascii_lowercase(), value.trim().to_string()))
}

/// Rewrites only `[Loader] loader`, keeping every other byte and the line ending.
pub fn configure_loader(
    ini: &[u8],
    loader: &str,
    expected_target: &str,
) -> Result<Vec<u8>, ConfigError> {
    let mut output = Vec::with_capacity(ini.len() + loader.len());
    let mut section = String::new();
    let (mut seen_section, mut loaders, mut targets) = (false, 0usize, Vec::new());
    for line in ini.split_inclusive(|b| *b == b'\n') {
        let trimmed = String::from_utf8_lossy(line);
        let trimmed = trimmed.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = trimmed[1..trimmed.len() - 1].trim().to_ascii_lowercase();
            seen_section |= section == "loader";
            output.extend_from_slice(line);
            continue;
        }
        if section == "loader" {
            if let Some((key, value)) = key_value(line) {
                if key == "loader" {
                    loaders += 1;
                    let ending: &[u8] = if line.ends_with(b"\r\n") {
                        b"\r\n"
                    } else if line.ends_with(b"\n") {
                        b"\n"
                    } else {
                        b""
                    };
                    output.extend_from_slice(format!("loader = {loader}").as_bytes());
                    output.extend_from_slice(ending);
                    continue;
                }
                if key == "target" {
                    targets.push(value);
                }
            }
        }
        output.extend_from_slice(line);
    }
    if !seen_section {
        return Err(ConfigError::MissingLoaderSection);
    }
    if loaders != 1 {
        return Err(ConfigError::LoaderEntries(loaders));
    }
    if targets.len() != 1 {
        return Err(ConfigError::TargetEntries(targets.len()));
    }
    if !targets[0].eq_ignore_ascii_case(expected_target) {
        return Err(ConfigError::UnexpectedTarget(targets[0].clone()));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    const INI: &[u8] = b"; header\r\n[Loader]\r\ntarget= ZenlessZoneZero.exe\r\n; loader = commented\r\nloader = XXMI Launcher.exe\r\nrequire_admin = false\r\n\r\n[System]\r\nloader = untouched\r\n";

    #[test]
    fn rewrites_only_the_loader_entry_of_the_loader_section() {
        let out = configure_loader(INI, "LXMI Loader.exe", "ZenlessZoneZero.exe").unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("\r\nloader = LXMI Loader.exe\r\nrequire_admin"));
        assert!(text.contains("; loader = commented\r\n"));
        assert!(text.ends_with("[System]\r\nloader = untouched\r\n"));
        assert_eq!(
            text.len(),
            INI.len() - "XXMI Launcher.exe".len() + "LXMI Loader.exe".len()
        );
    }

    #[test]
    fn rejects_a_target_that_is_not_the_game() {
        assert_eq!(
            configure_loader(INI, "LXMI Loader.exe", "HYP.exe"),
            Err(ConfigError::UnexpectedTarget("ZenlessZoneZero.exe".into()))
        );
    }

    #[test]
    fn rejects_missing_or_duplicated_entries() {
        assert_eq!(
            configure_loader(b"[System]\nloader = x\n", "L", "T"),
            Err(ConfigError::MissingLoaderSection)
        );
        assert_eq!(
            configure_loader(b"[Loader]\ntarget = T\nloader = a\nloader = b\n", "L", "T"),
            Err(ConfigError::LoaderEntries(2))
        );
        assert_eq!(
            configure_loader(b"[Loader]\nloader = a\n", "L", "T"),
            Err(ConfigError::TargetEntries(0))
        );
    }
}
