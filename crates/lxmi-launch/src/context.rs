//! Allowlisted procfs reading for one Steam-started Windows process of the current
//! user. No memory reads, signals, or environment values beyond the allowlist.
use serde::Serialize;
use std::{
    collections::HashMap,
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

const ALLOWED: [&str; 6] = [
    "SteamAppId",
    "SteamGameId",
    "STEAM_COMPAT_DATA_PATH",
    "WINEPREFIX",
    "STEAM_COMPAT_TOOL_PATHS",
    "STEAM_COMPAT_CLIENT_INSTALL_PATH",
];

#[derive(Debug, Clone)]
pub struct ProcessQuery {
    pub executable: PathBuf,
    pub compat_data: PathBuf,
    pub app_id: u32,
}

impl ProcessQuery {
    /// Linux truncates `comm` to 15 bytes ("ZenlessZoneZero").
    fn comm_prefix(&self) -> String {
        let name = self
            .executable
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        name.chars().take(15).collect()
    }
}

/// The Steam/Proton context a helper must join so its hook shares the wineserver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SteamProcessContext {
    pub pid: u32,
    pub start_ticks: u64,
    pub compat_data: PathBuf,
    pub proton_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub steam_client_dir: Option<PathBuf>,
}

fn canonical(path: &Path) -> Option<PathBuf> {
    fs::canonicalize(path).ok()
}

fn start_ticks(stat: &str) -> Option<u64> {
    // Field 22; `comm` (field 2) may contain spaces, so count after the last ')'.
    stat.rsplit_once(')')?
        .1
        .split_whitespace()
        .nth(19)?
        .parse()
        .ok()
}

/// Maps `S:\a\b.exe` through `<prefix>/dosdevices/s:` or accepts an absolute Linux path.
fn resolve_argument(argument: &str, prefix: &Path) -> Option<PathBuf> {
    let normalized = argument.replace('\\', "/");
    let bytes = normalized.as_bytes();
    if bytes.len() > 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'/' {
        let drive = prefix
            .join("dosdevices")
            .join(normalized[..2].to_ascii_lowercase());
        return canonical(&drive.join(&normalized[3..]));
    }
    if normalized.starts_with('/') {
        return canonical(Path::new(&normalized));
    }
    None
}

fn read_environment(pid_dir: &Path) -> HashMap<String, String> {
    let mut env = HashMap::new();
    if let Ok(bytes) = fs::read(pid_dir.join("environ")) {
        for field in bytes.split(|b| *b == 0) {
            let text = String::from_utf8_lossy(field);
            if let Some((key, value)) = text.split_once('=') {
                if ALLOWED.contains(&key) {
                    env.insert(key.to_string(), value.to_string());
                }
            }
        }
    }
    env
}

fn tool_dirs(paths: &str) -> Option<(PathBuf, PathBuf)> {
    let mut proton = None;
    let mut runtime = None;
    for entry in paths.split(':').filter(|p| p.starts_with('/')) {
        let dir = PathBuf::from(entry);
        let is_file = |name: &str| fs::symlink_metadata(dir.join(name)).is_ok_and(|m| m.is_file());
        if proton.is_none() && is_file("proton") {
            proton = Some(dir);
        } else if runtime.is_none() && is_file("_v2-entry-point") {
            runtime = Some(dir);
        }
    }
    Some((proton?, runtime?))
}

/// Processes of the current user whose argv names exactly `query.executable`.
struct Matches {
    processes: Vec<(u32, PathBuf)>,
    compat: PathBuf,
    prefix: PathBuf,
}

fn matching_processes(proc_root: &Path, uid: u32, query: &ProcessQuery) -> Result<Matches, String> {
    let executable = canonical(&query.executable)
        .ok_or_else(|| format!("No se encontró {}.", query.executable.display()))?;
    let compat = canonical(&query.compat_data).ok_or_else(|| {
        "No existe el compatdata del juego. Ábrelo una vez desde Steam sin mods.".to_string()
    })?;
    let prefix = compat.join("pfx");
    let comm_prefix = query.comm_prefix();
    let entries = fs::read_dir(proc_root).map_err(|e| format!("No se pudo leer procfs: {e}"))?;
    let mut matches = Vec::new();
    for entry in entries.flatten() {
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        let dir = entry.path();
        if fs::metadata(&dir).map(|m| m.uid()).ok() != Some(uid) {
            continue;
        }
        let Ok(comm) = fs::read_to_string(dir.join("comm")) else {
            continue;
        };
        if !comm.trim_end().starts_with(&comm_prefix) {
            continue;
        }
        let Ok(cmdline) = fs::read(dir.join("cmdline")) else {
            continue;
        };
        let exact = cmdline
            .split(|b| *b == 0)
            .filter(|a| !a.is_empty())
            .any(|a| {
                resolve_argument(&String::from_utf8_lossy(a), &prefix).as_deref()
                    == Some(executable.as_path())
            });
        if exact {
            matches.push((pid, dir));
        }
    }
    Ok(Matches {
        processes: matches,
        compat,
        prefix,
    })
}

/// Number of running processes of exactly this executable (any context).
pub fn count_processes(proc_root: &Path, uid: u32, query: &ProcessQuery) -> Result<usize, String> {
    Ok(matching_processes(proc_root, uid, query)?.processes.len())
}

/// `Ok(None)`: not running. `Err`: running but ambiguous or outside the expected context.
pub fn find_steam_process(
    proc_root: &Path,
    uid: u32,
    query: &ProcessQuery,
) -> Result<Option<SteamProcessContext>, String> {
    let Matches {
        processes: mut matches,
        compat,
        prefix,
    } = matching_processes(proc_root, uid, query)?;
    let comm_prefix = query.comm_prefix();
    let (pid, dir) = match matches.len() {
        0 => return Ok(None),
        1 => matches.remove(0),
        n => {
            return Err(format!(
                "Hay {n} procesos de {comm_prefix}; cierra los duplicados."
            ))
        }
    };
    let env = read_environment(&dir);
    let app = query.app_id.to_string();
    if env.get("SteamAppId") != Some(&app) && env.get("SteamGameId") != Some(&app) {
        return Err("El proceso no fue iniciado por Steam para este juego.".into());
    }
    let same = |key: &str, expected: &Path| {
        env.get(key)
            .and_then(|v| canonical(Path::new(v)))
            .is_some_and(|p| p == expected)
    };
    if !same("STEAM_COMPAT_DATA_PATH", &compat) || !same("WINEPREFIX", &prefix) {
        return Err("El proceso usa otro compatdata o prefix.".into());
    }
    let (proton_dir, runtime_dir) = env
        .get("STEAM_COMPAT_TOOL_PATHS")
        .and_then(|paths| tool_dirs(paths))
        .ok_or_else(|| {
            "No se identificaron Proton y Steam Linux Runtime en el contexto de Steam.".to_string()
        })?;
    let start_ticks = fs::read_to_string(dir.join("stat"))
        .ok()
        .and_then(|s| start_ticks(&s))
        .ok_or_else(|| "El proceso terminó durante la comprobación.".to_string())?;
    Ok(Some(SteamProcessContext {
        pid,
        start_ticks,
        compat_data: compat,
        proton_dir,
        runtime_dir,
        steam_client_dir: env
            .get("STEAM_COMPAT_CLIENT_INSTALL_PATH")
            .map(PathBuf::from),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workdir::tests::TempDir;
    use std::os::unix::fs::symlink;

    struct World {
        _tmp: TempDir,
        proc: PathBuf,
        query: ProcessQuery,
        library: PathBuf,
        tools: String,
    }

    fn world() -> World {
        let tmp = TempDir::new();
        let root = tmp.path().to_path_buf();
        let library = root.join("lib");
        let game = library.join("steamapps/common/ZZZ");
        fs::create_dir_all(&game).unwrap();
        fs::write(game.join("HYP.exe"), b"").unwrap();
        let compat = library.join("steamapps/compatdata/4162040");
        fs::create_dir_all(compat.join("pfx/dosdevices")).unwrap();
        symlink(&library, compat.join("pfx/dosdevices/s:")).unwrap();
        for (dir, file) in [("proton", "proton"), ("slr", "_v2-entry-point")] {
            fs::create_dir_all(root.join(dir)).unwrap();
            fs::write(root.join(dir).join(file), b"").unwrap();
        }
        let tools = format!(
            "{}:{}",
            root.join("proton").display(),
            root.join("slr").display()
        );
        let proc = root.join("proc");
        fs::create_dir_all(&proc).unwrap();
        World {
            query: ProcessQuery {
                executable: game.join("HYP.exe"),
                compat_data: compat,
                app_id: 4162040,
            },
            _tmp: tmp,
            proc,
            library,
            tools,
        }
    }

    fn process(w: &World, pid: u32, comm: &str, argv: &str, env: &[(&str, String)]) {
        let dir = w.proc.join(pid.to_string());
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("comm"), format!("{comm}\n")).unwrap();
        fs::write(dir.join("cmdline"), format!("{argv}\0")).unwrap();
        let env: Vec<u8> = env
            .iter()
            .flat_map(|(k, v)| format!("{k}={v}\0").into_bytes())
            .collect();
        fs::write(dir.join("environ"), env).unwrap();
        fs::write(
            dir.join("stat"),
            format!("{pid} (x y) S {}", "1 ".repeat(18) + "4242 9"),
        )
        .unwrap();
    }

    fn steam_env(w: &World) -> Vec<(&'static str, String)> {
        vec![
            ("SteamAppId", "4162040".into()),
            (
                "STEAM_COMPAT_DATA_PATH",
                w.query.compat_data.display().to_string(),
            ),
            (
                "WINEPREFIX",
                format!("{}/pfx/", w.query.compat_data.display()),
            ),
            ("STEAM_COMPAT_TOOL_PATHS", w.tools.clone()),
            ("LD_PRELOAD", "not-kept".into()),
        ]
    }

    fn uid() -> u32 {
        unsafe { libc::getuid() }
    }

    #[test]
    fn binds_the_launcher_through_its_steam_drive_letter() {
        let w = world();
        process(
            &w,
            77,
            "HYP.exe",
            r"S:\steamapps\common\ZZZ\HYP.exe",
            &steam_env(&w),
        );
        process(&w, 78, "HYP.exe", r"C:\other\HYP.exe", &steam_env(&w));
        let found = find_steam_process(&w.proc, uid(), &w.query)
            .unwrap()
            .unwrap();
        assert_eq!(found.pid, 77);
        assert_eq!(found.start_ticks, 4242);
        assert!(found.proton_dir.ends_with("proton") && found.runtime_dir.ends_with("slr"));
        assert!(w.library.exists());
    }

    #[test]
    fn absent_process_is_not_an_error() {
        let w = world();
        assert_eq!(find_steam_process(&w.proc, uid(), &w.query), Ok(None));
    }

    #[test]
    fn rejects_other_app_prefix_or_duplicates() {
        let w = world();
        let mut env = steam_env(&w);
        env[0].1 = "1".into();
        process(&w, 77, "HYP.exe", r"S:\steamapps\common\ZZZ\HYP.exe", &env);
        assert!(find_steam_process(&w.proc, uid(), &w.query).is_err());
        let mut env = steam_env(&w);
        env[2].1 = "/tmp/elsewhere/pfx".into();
        process(&w, 77, "HYP.exe", r"S:\steamapps\common\ZZZ\HYP.exe", &env);
        assert!(find_steam_process(&w.proc, uid(), &w.query).is_err());
        process(
            &w,
            77,
            "HYP.exe",
            r"S:\steamapps\common\ZZZ\HYP.exe",
            &steam_env(&w),
        );
        process(
            &w,
            79,
            "HYP.exe",
            r"S:\steamapps\common\ZZZ\HYP.exe",
            &steam_env(&w),
        );
        assert!(find_steam_process(&w.proc, uid(), &w.query)
            .unwrap_err()
            .contains("2 procesos"));
    }

    #[test]
    fn requires_proton_and_runtime_in_the_steam_context() {
        let w = world();
        let mut env = steam_env(&w);
        env[3].1 = "/nonexistent".into();
        process(&w, 77, "HYP.exe", r"S:\steamapps\common\ZZZ\HYP.exe", &env);
        assert!(find_steam_process(&w.proc, uid(), &w.query)
            .unwrap_err()
            .contains("Proton"));
    }
}
