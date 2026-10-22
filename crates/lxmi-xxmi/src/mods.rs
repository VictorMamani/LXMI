//! Local mod library. Originals and prepared profile snapshots are immutable.
//! Preparation never deploys to a runtime, game or prefix and never reloads a game.
mod previews;

use crate::{
    archive::extract_zip,
    filesystem::{missing, Directory},
    package::{fingerprint, inventory},
    ErrorCode, ImportLimits, ManagedStore, PackageFile, Result, XxmiError,
};
pub use previews::{ModDetails, PreviewState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::File,
    io::Read,
    os::fd::AsRawFd,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
const MAX_MODS: usize = 256;
const MAX_PROFILES: usize = 64;
const MAX_METADATA: u64 = 2 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    pub schema_version: u32,
    pub game_id: String,
    pub id: String,
    pub name: String,
    pub files: Vec<PackageFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModProfile {
    pub name: String,
    pub mod_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfileState {
    schema_version: u32,
    game_id: String,
    revision: u64,
    profiles: Vec<ModProfile>,
}

#[derive(Debug, Serialize)]
pub struct ModLibrarySnapshot {
    pub game_id: String,
    pub mods: Vec<ModManifest>,
    pub profiles: Vec<ModProfile>,
    pub revision: u64,
    pub issues: Vec<XxmiError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparedModProfile {
    pub schema_version: u32,
    pub game_id: String,
    pub profile: ModProfile,
    pub mods_path: PathBuf,
    pub files: Vec<PackageFile>,
}

#[derive(Clone)]
pub struct ModLibrary {
    store: ManagedStore,
    limits: ImportLimits,
}

/// Character packs and combined profiles can exceed the runtime package limits.
pub(crate) fn mod_limits() -> ImportLimits {
    ImportLimits {
        max_total_bytes: 4 * 1024 * 1024 * 1024,
        ..ImportLimits::default()
    }
}

impl ModLibrary {
    pub fn new(store: ManagedStore) -> Self {
        Self {
            store,
            limits: mod_limits(),
        }
    }

    /// Explicit limits are useful for constrained hosts and isolated tests.
    pub fn with_limits(store: ManagedStore, limits: ImportLimits) -> Self {
        Self { store, limits }
    }

    fn existing(&self, game: &str) -> Result<Directory> {
        validate_game(game)?;
        let root = Directory::open_absolute(self.store.root())?;
        root.assert_private()?;
        let dir = root.descend(&format!("mods/{game}"))?;
        dir.assert_private()?;
        Ok(dir)
    }

    fn initialize(&self, game: &str) -> Result<Directory> {
        validate_game(game)?;
        let root = self.store.initialize()?;
        let dir = root.private_child("mods")?.private_child(game)?;
        dir.private_child("packages")?;
        dir.private_child("staging")?;
        dir.private_child("prepared")?;
        Ok(dir)
    }

    pub fn list(&self, game: &str) -> Result<ModLibrarySnapshot> {
        validate_game(game)?;
        let dir = match self.existing(game) {
            Ok(dir) => dir,
            Err(e) if missing(&e) => {
                return Ok(ModLibrarySnapshot {
                    game_id: game.into(),
                    mods: vec![],
                    profiles: vec![],
                    revision: 0,
                    issues: vec![],
                })
            }
            Err(e) => return Err(e),
        };
        let state = read_profiles(&dir, game)?;
        let packages = dir.child("packages")?;
        let mut mods = vec![];
        let mut issues = vec![];
        for id in packages.entries(MAX_MODS)? {
            match self.verify_in(&dir, game, &id) {
                Ok(m) => mods.push(m),
                Err(mut e) => {
                    e.path = Some(
                        self.store
                            .root()
                            .join("mods")
                            .join(game)
                            .join("packages")
                            .join(&id),
                    );
                    issues.push(e);
                }
            }
        }
        mods.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then(a.id.cmp(&b.id))
        });
        Ok(ModLibrarySnapshot {
            game_id: game.into(),
            mods,
            profiles: state.profiles,
            revision: state.revision,
            issues,
        })
    }

    fn verify_in(&self, dir: &Directory, game: &str, id: &str) -> Result<ModManifest> {
        validate_id(id)?;
        let package = dir.descend(&format!("packages/{id}"))?;
        package.assert_private()?;
        let manifest: ModManifest = decode(&package.read_file("lxmi-mod.json", MAX_METADATA)?)?;
        let files = inventory(&package.child("payload")?, self.limits)?;
        validate_mod_files(&files)?;
        validate_name(&manifest.name)?;
        if manifest.schema_version != 1
            || manifest.game_id != game
            || manifest.id != id
            || manifest.files != files
            || fingerprint(&files)? != id
        {
            return Err(error(ErrorCode::ChecksumMismatch, "The mod changed since it was imported. Import the original again with different content."));
        }
        Ok(manifest)
    }

    /// The game is explicitly assigned by the user, not inferred from an INI.
    pub fn import(&self, game: &str, name: &str, source: &Path) -> Result<ModManifest> {
        validate_game(game)?;
        validate_name(name)?;
        if source
            .extension()
            .is_some_and(|e| ["rar", "7z"].iter().any(|x| e.eq_ignore_ascii_case(x)))
        {
            return Err(error(
                ErrorCode::UnsupportedArchive,
                "Extract the RAR or 7z and import its directory. This version only extracts ZIP archives.",
            ));
        }
        if source.starts_with(self.store.root()) || self.store.root().starts_with(source) {
            return Err(error(
                ErrorCode::UnsafePath,
                "The source and LXMI storage must not overlap.",
            ));
        }
        // Open every source path component before creating private storage.
        let is_zip = source
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("zip"));
        let input = if is_zip {
            let parent = Directory::open_absolute(source.parent().ok_or_else(|| {
                error(ErrorCode::UnsafePath, "The ZIP has no parent directory.")
            })?)?;
            let base = source
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| error(ErrorCode::UnsafePath, "Invalid ZIP name."))?;
            parent.open_file(base)?;
            None
        } else {
            Some(Directory::open_absolute(source)?)
        };
        let dir = self.initialize(game)?;
        let _lock = LibraryLock::acquire(&dir)?;
        let staging = dir.child("staging")?;
        let stage_name = temp_name();
        let stage = staging.new_private_child(&stage_name)?;
        let result = (|| {
            let payload = stage.new_private_child("payload")?;
            if let Some(input) = input {
                let files = inventory(&input, self.limits)?;
                validate_mod_files(&files)?;
                copy_files(&input, &payload, &files)?;
                if inventory(&payload, self.limits)? != files {
                    return Err(error(
                        ErrorCode::ChecksumMismatch,
                        "The source changed during copying.",
                    ));
                }
            } else {
                extract_zip(source, &payload, self.limits)?;
            }
            let files = inventory(&payload, self.limits)?;
            validate_mod_files(&files)?;
            let id = fingerprint(&files)?;
            let packages = dir.child("packages")?;
            if packages.metadata(&id)?.is_some() {
                return self.verify_in(&dir, game, &id);
            }
            if packages.entries(MAX_MODS)?.len() >= MAX_MODS {
                return Err(error(
                    ErrorCode::LimitExceeded,
                    "The limit of 256 mods per game has been reached.",
                ));
            }
            let manifest = ModManifest {
                schema_version: 1,
                game_id: game.into(),
                id: id.clone(),
                name: name.trim().into(),
                files,
            };
            stage.write_new("lxmi-mod.json", &encode(&manifest)?)?;
            stage.sync()?;
            staging.promote(&stage_name, &packages, &id)?;
            Ok(manifest)
        })();
        // Only our own unpromoted staging directory is removed.
        if staging.metadata(&stage_name)?.is_some() {
            staging.remove_stage(&stage_name)?;
        }
        result
    }

    /// Optimistic revision plus a cross-process lock prevents lost profile edits.
    pub fn save_profile(
        &self,
        game: &str,
        name: &str,
        ids: &[String],
        expected_revision: u64,
    ) -> Result<ModLibrarySnapshot> {
        validate_game(game)?;
        validate_name(name)?;
        validate_ids(ids)?;
        let dir = self.initialize(game)?;
        let _lock = LibraryLock::acquire(&dir)?;
        let mut state = read_profiles(&dir, game)?;
        if state.revision != expected_revision {
            return Err(error(
                ErrorCode::Busy,
                "Profiles changed in another window. Refresh the library and save again.",
            ));
        }
        for id in ids {
            self.verify_in(&dir, game, id)?;
        }
        let mut mod_ids = ids.to_vec();
        mod_ids.sort();
        let profile = ModProfile {
            name: name.trim().into(),
            mod_ids,
        };
        if let Some(existing) = state
            .profiles
            .iter_mut()
            .find(|p| p.name.to_lowercase() == profile.name.to_lowercase())
        {
            *existing = profile;
        } else {
            if state.profiles.len() >= MAX_PROFILES {
                return Err(error(
                    ErrorCode::LimitExceeded,
                    "The limit of 64 profiles per game has been reached.",
                ));
            }
            state.profiles.push(profile);
        }
        state.revision = state
            .revision
            .checked_add(1)
            .ok_or_else(|| error(ErrorCode::LimitExceeded, "Revision limit reached."))?;
        let temporary = temp_name();
        dir.write_new(&temporary, &encode(&state)?)?;
        // Paths are descriptor-anchored; rename replaces only our state file.
        std::fs::rename(
            dir.path().join(&temporary),
            dir.path().join("profiles.json"),
        )
        .map_err(|e| XxmiError::io(Path::new("profiles.json"), e))?;
        dir.sync()?;
        self.list(game)
    }

    /// Builds a complete immutable Mods tree. It has no effect on a running game.
    pub fn prepare_profile(
        &self,
        game: &str,
        name: &str,
        expected_revision: u64,
    ) -> Result<PreparedModProfile> {
        validate_game(game)?;
        validate_name(name)?;
        let dir = self.existing(game)?;
        let _lock = LibraryLock::acquire(&dir)?;
        let state = read_profiles(&dir, game)?;
        if state.revision != expected_revision {
            return Err(error(
                ErrorCode::Busy,
                "The profile changed. Refresh the library before preparing it.",
            ));
        }
        let profile = state
            .profiles
            .into_iter()
            .find(|p| p.name == name)
            .ok_or_else(|| {
                error(
                    ErrorCode::NotFound,
                    "Save the profile before preparing its copy.",
                )
            })?;
        let mut manifests = vec![];
        let mut expected_files = vec![];
        let mut bytes = 0u64;
        for id in &profile.mod_ids {
            let manifest = self.verify_in(&dir, game, id)?;
            for file in &manifest.files {
                bytes = bytes
                    .checked_add(file.size)
                    .ok_or_else(|| error(ErrorCode::LimitExceeded, "The profile is too large."))?;
                expected_files.push(PackageFile {
                    relative_path: format!("lxmi_{id}/{}", file.relative_path),
                    ..file.clone()
                });
            }
            manifests.push(manifest);
        }
        if bytes > self.limits.max_total_bytes
            || expected_files.len() + manifests.len() > self.limits.max_files
        {
            return Err(error(
                ErrorCode::LimitExceeded,
                "The selection exceeds the prepared profile limits.",
            ));
        }
        expected_files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        let id = format!(
            "{:x}",
            Sha256::digest(encode(&(game, &profile, &expected_files))?)
        );
        let prepared = dir.child("prepared")?;
        let report = PreparedModProfile {
            schema_version: 1,
            game_id: game.into(),
            profile,
            mods_path: self
                .store
                .root()
                .join("mods")
                .join(game)
                .join("prepared")
                .join(&id)
                .join("Mods"),
            files: expected_files,
        };
        if prepared.metadata(&id)?.is_some() {
            let existing = prepared.child(&id)?;
            let stored: PreparedModProfile =
                decode(&existing.read_file("lxmi-profile.json", MAX_METADATA)?)?;
            if encode(&stored)? != encode(&report)?
                || inventory(&existing.child("Mods")?, profile_limits(self.limits))? != report.files
            {
                return Err(error(
                    ErrorCode::ChecksumMismatch,
                    "The prepared copy changed. It is retained for review and will not be overwritten.",
                ));
            }
            return Ok(report);
        }
        let staging = dir.child("staging")?;
        let stage_name = temp_name();
        let stage = staging.new_private_child(&stage_name)?;
        let result = (|| {
            let mods = stage.new_private_child("Mods")?;
            for manifest in &manifests {
                let source = dir.descend(&format!("packages/{}/payload", manifest.id))?;
                let target = mods.new_private_child(&format!("lxmi_{}", manifest.id))?;
                copy_files(&source, &target, &manifest.files)?;
            }
            if inventory(&mods, profile_limits(self.limits))? != report.files {
                return Err(error(
                    ErrorCode::ChecksumMismatch,
                    "The mods changed during preparation.",
                ));
            }
            stage.write_new("lxmi-profile.json", &encode(&report)?)?;
            stage.sync()?;
            staging.promote(&stage_name, &prepared, &id)?;
            Ok(report)
        })();
        if staging.metadata(&stage_name)?.is_some() {
            staging.remove_stage(&stage_name)?;
        }
        result
    }
}

fn profile_limits(mut limits: ImportLimits) -> ImportLimits {
    limits.max_depth += 1;
    limits
}

fn read_profiles(dir: &Directory, game: &str) -> Result<ProfileState> {
    if dir.metadata("profiles.json")?.is_none() {
        return Ok(ProfileState {
            schema_version: 1,
            game_id: game.into(),
            revision: 0,
            profiles: vec![],
        });
    }
    let state: ProfileState = decode(&dir.read_file("profiles.json", MAX_METADATA)?)?;
    if state.schema_version != 1 || state.game_id != game || state.profiles.len() > MAX_PROFILES {
        return Err(error(ErrorCode::InvalidMetadata, "Invalid profile state."));
    }
    let mut seen = HashSet::new();
    for profile in &state.profiles {
        validate_name(&profile.name)?;
        validate_ids(&profile.mod_ids)?;
        if !seen.insert(profile.name.to_lowercase()) {
            return Err(error(
                ErrorCode::InvalidMetadata,
                "Duplicate profile names.",
            ));
        }
    }
    Ok(state)
}

pub(crate) fn copy_files(
    source: &Directory,
    target: &Directory,
    files: &[PackageFile],
) -> Result<()> {
    for file in files {
        copy_file_as(source, target, file, &file.relative_path)?;
    }
    target.sync()
}

pub(crate) fn copy_file_as(
    source: &Directory,
    target: &Directory,
    file: &PackageFile,
    destination: &str,
) -> Result<()> {
    target.ensure_parents(destination)?;
    let mut input = source.open_file(&file.relative_path)?.take(file.size + 1);
    let mut output = target.create_new_file(destination)?;
    let copied = std::io::copy(&mut input, &mut output)
        .map_err(|e| XxmiError::io(Path::new(destination), e))?;
    if copied != file.size {
        return Err(error(
            ErrorCode::ChecksumMismatch,
            "The file changed during copying.",
        ));
    }
    output
        .sync_all()
        .map_err(|e| XxmiError::io(Path::new(destination), e))
}

/// Only the standard hash-named shader replacements are deployed to ShaderFixes.
pub(crate) fn is_shader_fix(path: &str) -> bool {
    let Some(name) = path.strip_prefix("ShaderFixes/") else {
        return false;
    };
    let Some((hash, suffix)) = name.split_once('-') else {
        return false;
    };
    hash.len() == 16
        && hash.bytes().all(|b| b.is_ascii_hexdigit())
        && matches!(
            suffix,
            "ps_replace.txt" | "ps_replace.bin" | "vs_replace.txt" | "vs_replace.bin"
        )
}

fn validate_mod_files(files: &[PackageFile]) -> Result<()> {
    for file in files {
        for component in file.relative_path.split('/') {
            let stem = component
                .split('.')
                .next()
                .unwrap_or("")
                .to_ascii_uppercase();
            let reserved_device = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                || ["COM", "LPT"].iter().any(|prefix| {
                    stem.strip_prefix(prefix)
                        .is_some_and(|n| n.len() == 1 && matches!(n.as_bytes()[0], b'1'..=b'9'))
                });
            if component.ends_with(['.', ' '])
                || component.contains(['<', '>', '"', '|', '?', '*'])
                || reserved_device
            {
                return Err(error(
                    ErrorCode::UnsafePath,
                    "The mod contains ambiguous or Windows-reserved filenames.",
                ));
            }
        }
    }
    let has_ini = files.iter().any(|f| {
        f.size > 0
            && Path::new(&f.relative_path)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("ini"))
    });
    let shader_pack = files
        .iter()
        .any(|f| f.size > 0 && is_shader_fix(&f.relative_path))
        && files.iter().all(|f| {
            is_shader_fix(&f.relative_path)
                || f.relative_path.eq_ignore_ascii_case("README.txt")
                || f.relative_path.eq_ignore_ascii_case("README.md")
        });
    if !has_ini && !shader_pack {
        return Err(error(
            ErrorCode::InvalidPackage,
            "Select a mod with an INI or a shader replacement package in ShaderFixes.",
        ));
    }
    if files.iter().any(|f| {
        Path::new(&f.relative_path).extension().is_some_and(|e| {
            ["exe", "dll", "msi"]
                .iter()
                .any(|x| e.eq_ignore_ascii_case(x))
        })
    }) {
        return Err(error(ErrorCode::InvalidPackage, "This library accepts mods with INI files and resources. Import runtime executables and DLLs from the XXMI section."));
    }
    Ok(())
}

fn validate_game(game: &str) -> Result<()> {
    if !matches!(game, "wuthering-waves" | "zenless-zone-zero") {
        return Err(error(
            ErrorCode::WrongGameIntegration,
            "Select Wuthering Waves or Zenless Zone Zero.",
        ));
    }
    Ok(())
}
fn validate_id(id: &str) -> Result<()> {
    if id.len() != 64
        || !id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(error(ErrorCode::UnsafePath, "Invalid mod identifier."));
    }
    Ok(())
}
fn validate_ids(ids: &[String]) -> Result<()> {
    if ids.len() > MAX_MODS {
        return Err(error(
            ErrorCode::LimitExceeded,
            "Too many mods in the profile.",
        ));
    }
    let mut seen = HashSet::new();
    for id in ids {
        validate_id(id)?;
        if !seen.insert(id) {
            return Err(error(
                ErrorCode::InvalidMetadata,
                "Duplicate mod in the profile.",
            ));
        }
    }
    Ok(())
}
fn validate_name(name: &str) -> Result<()> {
    if name.trim().is_empty() || name.len() > 100 || name.chars().any(char::is_control) {
        return Err(error(
            ErrorCode::InvalidMetadata,
            "Enter a name between 1 and 100 bytes without control characters.",
        ));
    }
    Ok(())
}
fn temp_name() -> String {
    format!(
        "mod-stage-{}-{}",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    )
}
fn error(code: ErrorCode, detail: &str) -> XxmiError {
    XxmiError::new(code, None, detail)
}
fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let bytes =
        serde_json::to_vec(value).map_err(|e| error(ErrorCode::InvalidMetadata, &e.to_string()))?;
    if bytes.len() as u64 > MAX_METADATA {
        return Err(error(
            ErrorCode::LimitExceeded,
            "The inventory or profiles exceed the metadata limit.",
        ));
    }
    Ok(bytes)
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    serde_json::from_slice(bytes).map_err(|e| error(ErrorCode::InvalidMetadata, &e.to_string()))
}

pub(crate) struct LibraryLock {
    _file: File,
}
impl LibraryLock {
    pub(crate) fn acquire(dir: &Directory) -> Result<Self> {
        if dir.metadata("library.lock")?.is_none() {
            // Another process may create the lock first; opening below validates it.
            if let Err(e) = dir.write_new("library.lock", b"") {
                if dir.metadata("library.lock")?.is_none() {
                    return Err(e);
                }
            }
        }
        let file = dir.open_file("library.lock")?;
        // SAFETY: flock borrows a live owned file descriptor. Closing it releases the lock.
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(error(
                ErrorCode::Busy,
                "The library is busy. Wait for the other operation to finish.",
            ));
        }
        Ok(Self { _file: file })
    }
}
