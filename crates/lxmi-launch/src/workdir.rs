//! Private launch directory: writable copy of the selected runtime, verified loader
//! library and the embedded helper. The helper is the only executable in the tree:
//! upstream accepts any process whose image directory contains the module directory.
use crate::config::configure_loader;
use lxmi_xxmi::{copy_profile_runtime_for_launch, ManagedRuntime, ManagedStore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const HELPER_NAME: &str = "LXMI Loader.exe";
const RECORD: &str = "launch.json";
const USER_SETTINGS: &str = "d3dx_user.ini";
const KEEP_PREVIOUS: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparedLaunch {
    pub dir: PathBuf,
    pub game_id: String,
    pub runtime_id: String,
    pub created_unix_seconds: u64,
    pub file_count: usize,
    pub helper_sha256: String,
    pub loader_library_sha256: String,
    pub restored_user_settings: bool,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn private_dir(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(m) if m.is_dir() => Ok(()),
        Ok(_) => Err(format!(
            "{} no es un directorio administrado.",
            path.display()
        )),
        Err(_) => fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(path)
            .map_err(|e| format!("No se pudo crear {}: {e}", path.display())),
    }
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|e| format!("No se pudo crear {}: {e}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| format!("No se pudo escribir {}: {e}", path.display()))
}

fn read_regular(path: &Path, limit: u64) -> Option<Vec<u8>> {
    let meta = fs::symlink_metadata(path).ok()?;
    (meta.is_file() && meta.len() <= limit)
        .then(|| fs::read(path).ok())
        .flatten()
}

/// Previous launches of this game, newest first (directory names start with the time).
fn previous_launches(game_root: &Path) -> Vec<(PathBuf, Option<PreparedLaunch>)> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(game_root)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| e.path())
        .collect();
    dirs.sort();
    dirs.reverse();
    dirs.into_iter()
        .map(|dir| {
            let record = read_regular(&dir.join(RECORD), 64 * 1024)
                .and_then(|b| serde_json::from_slice(&b).ok());
            (dir, record)
        })
        .collect()
}

/// The caller must have checked that the game is not running: older launch copies
/// beyond the most recent ones are removed here.
pub fn prepare_launch_directory(
    store: &ManagedStore,
    runtime: &ManagedRuntime,
    helper: &[u8],
) -> Result<PreparedLaunch, String> {
    let game = runtime.manifest.game_id.clone();
    let game_root = store.root().join("launches").join(&game);
    private_dir(&store.root().join("launches"))?;
    private_dir(&game_root)?;
    let previous = previous_launches(&game_root);
    let created = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut nonce = [0u8; 8];
    getrandom(&mut nonce)?;
    let dir = game_root.join(format!(
        "{created:012}-{}",
        nonce.iter().map(|b| format!("{b:02x}")).collect::<String>()
    ));
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&dir)
        .map_err(|e| format!("No se pudo crear la carpeta de arranque: {e}"))?;

    let files = copy_profile_runtime_for_launch(store, runtime, &dir).map_err(|e| e.detail)?;
    let importer = dir.join(
        runtime
            .manifest
            .import_path_from_app_root
            .trim_end_matches('/'),
    );
    let ini_path = importer.join("d3dx.ini");
    let ini = read_regular(&ini_path, 4 * 1024 * 1024)
        .ok_or("d3dx.ini no está en la copia de arranque.")?;
    let configured =
        configure_loader(&ini, HELPER_NAME, &runtime.target_process).map_err(|e| e.to_string())?;
    fs::remove_file(&ini_path).map_err(|e| format!("No se pudo preparar d3dx.ini: {e}"))?;
    write_new(&ini_path, &configured)?;

    // Persisted mod toggles from the previous launch of this same profile runtime.
    let mut restored_user_settings = false;
    if let Some((old, _)) = previous.iter().find(|(_, r)| {
        r.as_ref()
            .is_some_and(|r| r.runtime_id == runtime.runtime_id)
    }) {
        if let Some(bytes) = read_regular(
            &old.join(&runtime.manifest.import_path_from_app_root)
                .join(USER_SETTINGS),
            1024 * 1024,
        ) {
            let target = importer.join(USER_SETTINGS);
            if fs::symlink_metadata(&target).is_err() {
                write_new(&target, &bytes)?;
                restored_user_settings = true;
            }
        }
    }

    let loader = read_regular(&runtime.loader_library_path, 16 * 1024 * 1024)
        .ok_or("Falta 3dmloader.dll en XXMI Libraries.")?;
    let loader_library_sha256 = sha256(&loader);
    if loader_library_sha256 != runtime.manifest.loader_library_sha256 {
        return Err("3dmloader.dll no coincide con el paquete verificado.".into());
    }
    write_new(&dir.join("3dmloader.dll"), &loader)?;
    write_new(&dir.join(HELPER_NAME), helper)?;
    let helper_sha256 = sha256(helper);
    if read_regular(&dir.join(HELPER_NAME), 16 * 1024 * 1024).map(|b| sha256(&b))
        != Some(helper_sha256.clone())
    {
        return Err("El helper no se escribió correctamente.".into());
    }
    let record = PreparedLaunch {
        dir: dir.clone(),
        game_id: game,
        runtime_id: runtime.runtime_id.clone(),
        created_unix_seconds: created,
        file_count: files.len(),
        helper_sha256,
        loader_library_sha256,
        restored_user_settings,
    };
    write_new(
        &dir.join(RECORD),
        &serde_json::to_vec_pretty(&record).map_err(|e| e.to_string())?,
    )?;

    for (old, _) in previous.into_iter().skip(KEEP_PREVIOUS) {
        if fs::symlink_metadata(&old).is_ok_and(|m| m.is_dir())
            && old.parent() == Some(game_root.as_path())
        {
            let _ = fs::remove_dir_all(&old);
        }
    }
    Ok(record)
}

fn getrandom(buffer: &mut [u8]) -> Result<(), String> {
    let mut file =
        fs::File::open("/dev/urandom").map_err(|e| format!("Sin fuente aleatoria: {e}"))?;
    std::io::Read::read_exact(&mut file, buffer).map_err(|e| format!("Sin fuente aleatoria: {e}"))
}

#[cfg(test)]
pub(crate) mod tests {
    use std::path::{Path, PathBuf};

    /// Minimal private temp directory; the workspace has no tempfile dependency.
    pub struct TempDir(PathBuf);
    impl TempDir {
        pub fn new() -> Self {
            let mut nonce = [0u8; 8];
            super::getrandom(&mut nonce).unwrap();
            let path = std::env::temp_dir().join(format!(
                "lxmi-launch-test-{}",
                nonce.iter().map(|b| format!("{b:02x}")).collect::<String>()
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
        pub fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn previous_launches_are_newest_first_and_read_records() {
        let tmp = TempDir::new();
        for name in ["000000000001-aa", "000000000003-cc", "000000000002-bb"] {
            std::fs::create_dir(tmp.path().join(name)).unwrap();
        }
        std::fs::write(tmp.path().join("000000000003-cc/launch.json"), "not json").unwrap();
        let found = super::previous_launches(tmp.path());
        let names: Vec<_> = found
            .iter()
            .map(|(p, _)| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert_eq!(
            names,
            ["000000000003-cc", "000000000002-bb", "000000000001-aa"]
        );
        assert!(found.iter().all(|(_, r)| r.is_none()));
    }
}
