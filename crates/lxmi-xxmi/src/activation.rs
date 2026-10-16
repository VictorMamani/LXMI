//! Reversible selection of immutable private runtimes. No game process or prefix writes.
use crate::{
    filesystem::{missing, Directory},
    mods::{copy_file_as, copy_files, is_shader_fix, mod_limits, LibraryLock},
    package::{fingerprint, inventory},
    *,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const MAX_JSON: u64 = 4 * 1024 * 1024;
static NEXT: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileRuntimeRef {
    pub runtime_id: String,
    pub base_runtime_id: String,
    pub integration_package_id: String,
    pub libraries_package_id: String,
    pub profile_name: String,
    pub mod_count: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveProfileSelection {
    pub schema_version: u32,
    pub game_id: String,
    pub revision: u64,
    pub current: Option<ProfileRuntimeRef>,
    pub previous: Option<ProfileRuntimeRef>,
}
#[derive(Debug, Clone, Serialize)]
pub struct ActivationPlan {
    pub plan_id: String,
    pub game_id: String,
    pub profile_name: String,
    pub profile_revision: u64,
    pub selection_revision: u64,
    pub runtime_id: String,
    pub importer_root: PathBuf,
    pub mod_count: usize,
    pub file_count: usize,
    pub total_bytes: u64,
    pub required_launch_arguments: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DerivedRecord {
    schema_version: u32,
    game_id: String,
    runtime_id: String,
    base: RuntimeManifest,
    profile: ModProfile,
    files: Vec<PackageFile>,
}
struct PreparedActivation {
    plan: ActivationPlan,
    record: DerivedRecord,
    mods_path: PathBuf,
    base_files: Vec<PackageFile>,
}

pub fn profile_activation_status(
    store: &ManagedStore,
    game: &str,
) -> Result<ActiveProfileSelection> {
    game_spec(game)?;
    let root = match Directory::open_absolute(store.root()) {
        Ok(root) => root,
        Err(e) if missing(&e) => return Ok(empty_state(game)),
        Err(e) => return Err(e),
    };
    root.assert_private()?;
    let dir = match root.descend(&format!("mods/{game}")) {
        Ok(dir) => dir,
        Err(e) if missing(&e) => return Ok(empty_state(game)),
        Err(e) => return Err(e),
    };
    let state = read_state(&dir, game)?;
    if let Some(current) = &state.current {
        verify_reference(store, game, current)?;
    }
    Ok(state)
}

pub fn review_profile_activation(
    store: &ManagedStore,
    base: &ManagedRuntime,
    profile: &str,
    profile_revision: u64,
    selection_revision: u64,
) -> Result<ActivationPlan> {
    Ok(prepare_plan(store, base, profile, profile_revision, selection_revision)?.plan)
}

pub fn activate_profile(
    store: &ManagedStore,
    base: &ManagedRuntime,
    profile: &str,
    profile_revision: u64,
    selection_revision: u64,
    plan_id: &str,
) -> Result<ActiveProfileSelection> {
    let prepared = prepare_plan(store, base, profile, profile_revision, selection_revision)?;
    if prepared.plan.plan_id != plan_id {
        return Err(err(
            ErrorCode::Busy,
            "El plan cambió. Revisa la activación nuevamente.",
        ));
    }
    let root = store.initialize()?;
    let game = &prepared.plan.game_id;
    let dir = root.descend(&format!("mods/{game}"))?;
    let _lock = LibraryLock::acquire(&dir)?;
    let mut state = read_state(&dir, game)?;
    if state.revision != selection_revision
        || ModLibrary::new(store.clone()).list(game)?.revision != profile_revision
    {
        return Err(err(
            ErrorCode::Busy,
            "La selección o el perfil cambió en otra ventana. Actualiza y revisa nuevamente.",
        ));
    }
    let destination = root
        .private_child("profile-runtimes")?
        .private_child(game)?;
    if destination.metadata(&prepared.record.runtime_id)?.is_none() {
        let staging = root.child("staging")?;
        let name = temp_name();
        let stage = staging.new_private_child(&name)?;
        let result = (|| {
            let (_, folder) = game_spec(game)?;
            let importer = stage.new_private_child(folder)?;
            copy_files(
                &Directory::open_absolute(&base.importer_root)?,
                &importer,
                &prepared.base_files,
            )?;
            importer.new_private_child("Mods")?;
            let source = Directory::open_absolute(&prepared.mods_path)?;
            let mut limits = mod_limits();
            limits.max_depth += 1;
            let files = inventory(&source, limits)?;
            for file in &files {
                copy_file_as(
                    &source,
                    &importer,
                    file,
                    &profile_destination(&file.relative_path),
                )?;
            }
            if inventory(&importer, derived_limits())? != prepared.record.files {
                return Err(err(
                    ErrorCode::ChecksumMismatch,
                    "El runtime o los mods cambiaron durante la copia.",
                ));
            }
            let mut manifest = prepared.record.base.clone();
            manifest.runtime_id = prepared.record.runtime_id.clone();
            stage.write_new("runtime-manifest.json", &encode(&manifest)?)?;
            stage.write_new("lxmi-profile-runtime.json", &encode(&prepared.record)?)?;
            stage.sync()?;
            staging.promote(&name, &destination, &prepared.record.runtime_id)
        })();
        if staging.metadata(&name)?.is_some() {
            staging.remove_stage(&name)?;
        }
        result?;
    }
    let runtime = verify_derived(store, game, &prepared.record.runtime_id)?;
    let reference = ProfileRuntimeRef {
        runtime_id: runtime.runtime_id,
        base_runtime_id: base.runtime_id.clone(),
        integration_package_id: base.integration_package_id.clone(),
        libraries_package_id: base.libraries_package_id.clone(),
        profile_name: profile.into(),
        mod_count: prepared.record.profile.mod_ids.len(),
    };
    // Applying an identical selection is idempotent and retains the undo target.
    if state.current.as_ref() == Some(&reference) {
        return Ok(state);
    }
    state.previous = state.current.take();
    state.current = Some(reference);
    save_state(&dir, &mut state)?;
    Ok(state)
}

pub fn rollback_profile_activation(
    store: &ManagedStore,
    game: &str,
    expected_revision: u64,
) -> Result<ActiveProfileSelection> {
    game_spec(game)?;
    let root = Directory::open_absolute(store.root())?;
    root.assert_private()?;
    let dir = root.descend(&format!("mods/{game}"))?;
    let _lock = LibraryLock::acquire(&dir)?;
    let mut state = read_state(&dir, game)?;
    if state.revision != expected_revision {
        return Err(err(
            ErrorCode::Busy,
            "La selección cambió. Actualiza antes de revertir.",
        ));
    }
    if state.revision == 0 {
        return Err(err(ErrorCode::NotFound, "No hay una selección anterior."));
    }
    if let Some(previous) = &state.previous {
        verify_reference(store, game, previous)?;
    }
    std::mem::swap(&mut state.current, &mut state.previous);
    save_state(&dir, &mut state)?;
    Ok(state)
}

/// Resolves the selection for the launch topology/bridge. It never silently ignores a stale base.
pub fn resolve_profile_runtime(
    store: &ManagedStore,
    base: &ManagedRuntime,
) -> Result<ManagedRuntime> {
    let state = profile_activation_status(store, &base.manifest.game_id)?;
    match state.current {
        None => Ok(base.clone()),
        Some(reference) => {
            if reference.base_runtime_id != base.runtime_id
                || reference.integration_package_id != base.integration_package_id
                || reference.libraries_package_id != base.libraries_package_id
            {
                return Err(err(ErrorCode::WrongGameIntegration, "El perfil seleccionado usa otro runtime base. Reviértelo o vuelve a prepararlo para estos paquetes."));
            }
            verify_reference(store, &state.game_id, &reference)
        }
    }
}

/// Writable launch copy of the selected profile runtime. 3DMigoto writes its log,
/// ShaderCache and d3dx_user.ini next to its DLL, so the immutable runtime is never
/// handed to the game. The copy is re-inventoried against the derived record.
pub fn copy_profile_runtime_for_launch(
    store: &ManagedStore,
    runtime: &ManagedRuntime,
    launch_dir: &Path,
) -> Result<Vec<PackageFile>> {
    let game = runtime.manifest.game_id.clone();
    let (_, folder) = game_spec(&game)?;
    let verified = verify_derived(store, &game, &runtime.runtime_id)?;
    if verified.importer_root != runtime.importer_root {
        return Err(err(
            ErrorCode::UnsafePath,
            "El runtime seleccionado no es un runtime de perfil administrado.",
        ));
    }
    let root = Directory::open_absolute(&verified.app_root)?;
    let record: DerivedRecord = decode(&root.read_file("lxmi-profile-runtime.json", MAX_JSON)?)?;
    let launch = Directory::open_absolute(launch_dir)?;
    launch.assert_private()?;
    let target = launch.new_private_child(folder)?;
    copy_files(&root.child(folder)?, &target, &record.files)?;
    for directory in &record.base.directories {
        target.ensure_directory(directory)?;
    }
    if inventory(&target, derived_limits())? != record.files {
        return Err(err(
            ErrorCode::ChecksumMismatch,
            "La copia de arranque no coincide con el runtime verificado.",
        ));
    }
    Ok(record.files)
}

fn prepare_plan(
    store: &ManagedStore,
    base: &ManagedRuntime,
    profile: &str,
    profile_revision: u64,
    selection_revision: u64,
) -> Result<PreparedActivation> {
    let game = &base.manifest.game_id;
    let (slug, folder) = game_spec(game)?;
    check_id(&base.runtime_id, slug)?;
    let expected = store
        .root()
        .join("runtimes")
        .join(game)
        .join(slug)
        .join(&base.runtime_id);
    if base.app_root != expected || base.importer_root != expected.join(folder) {
        return Err(err(
            ErrorCode::UnsafePath,
            "Se requiere un runtime base administrado y verificado.",
        ));
    }
    let base_dir = Directory::open_absolute(&base.app_root)?;
    base_dir.assert_private()?;
    let disk_manifest: RuntimeManifest =
        decode(&base_dir.read_file("runtime-manifest.json", MAX_JSON)?)?;
    if disk_manifest != base.manifest {
        return Err(err(
            ErrorCode::ChecksumMismatch,
            "El manifiesto base cambió.",
        ));
    }
    let base_files = base
        .manifest
        .files
        .iter()
        .map(|f| PackageFile {
            relative_path: f.destination_relative_path.clone(),
            size: f.size,
            sha256: f.sha256.clone(),
        })
        .collect::<Vec<_>>();
    let importer = base_dir.child(folder)?;
    if !importer.child("Mods")?.entries(1)?.is_empty() {
        return Err(err(ErrorCode::InvalidPackage, "El runtime base contiene mods no gestionados. Impórtalos a la biblioteca antes de aplicar perfiles."));
    }
    if inventory(&importer, ImportLimits::default())? != base_files {
        return Err(err(
            ErrorCode::ChecksumMismatch,
            "El runtime base cambió o contiene archivos no inventariados.",
        ));
    }
    let prepared =
        ModLibrary::new(store.clone()).prepare_profile(game, profile, profile_revision)?;
    let state = profile_activation_status(store, game)?;
    if state.revision != selection_revision {
        return Err(err(
            ErrorCode::Busy,
            "La selección cambió. Actualiza antes de revisar.",
        ));
    }
    let mut files = base_files.clone();
    files.extend(prepared.files.iter().map(|f| PackageFile {
        relative_path: profile_destination(&f.relative_path),
        ..f.clone()
    }));
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    let mut destinations = std::collections::HashSet::new();
    if files
        .iter()
        .any(|f| !destinations.insert(f.relative_path.to_lowercase()))
    {
        return Err(err(ErrorCode::InvalidPackage, "Dos archivos del perfil intentan ocupar el mismo destino del runtime. Revisa los ShaderFixes seleccionados."));
    }
    let bytes = files
        .iter()
        .try_fold(0u64, |sum, f| sum.checked_add(f.size))
        .ok_or_else(|| err(ErrorCode::LimitExceeded, "Runtime demasiado grande."))?;
    if bytes > derived_limits().max_total_bytes || files.len() > derived_limits().max_files {
        return Err(err(
            ErrorCode::LimitExceeded,
            "La selección supera los límites del runtime derivado.",
        ));
    }
    let mut record = DerivedRecord {
        schema_version: 1,
        game_id: game.clone(),
        runtime_id: String::new(),
        base: base.manifest.clone(),
        profile: prepared.profile,
        files,
    };
    record.runtime_id = record_id(&record)?;
    let plan_id = digest(&encode(&(&record, profile_revision, selection_revision))?);
    let plan = ActivationPlan {
        plan_id,
        game_id: game.clone(),
        profile_name: profile.into(),
        profile_revision,
        selection_revision,
        runtime_id: record.runtime_id.clone(),
        importer_root: store
            .root()
            .join("profile-runtimes")
            .join(game)
            .join(&record.runtime_id)
            .join(folder),
        mod_count: record.profile.mod_ids.len(),
        file_count: record.files.len(),
        total_bytes: bytes,
        required_launch_arguments: if slug == "wwmi" {
            vec!["-dx11".into()]
        } else {
            vec![]
        },
    };
    Ok(PreparedActivation {
        plan,
        record,
        mods_path: prepared.mods_path,
        base_files,
    })
}

fn verify_reference(
    store: &ManagedStore,
    game: &str,
    reference: &ProfileRuntimeRef,
) -> Result<ManagedRuntime> {
    let runtime = verify_derived(store, game, &reference.runtime_id)?;
    let dir = Directory::open_absolute(&runtime.app_root)?;
    let record: DerivedRecord = decode(&dir.read_file("lxmi-profile-runtime.json", MAX_JSON)?)?;
    if reference.base_runtime_id != record.base.runtime_id
        || reference.profile_name != record.profile.name
        || reference.mod_count != record.profile.mod_ids.len()
        || reference.integration_package_id != runtime.integration_package_id
        || reference.libraries_package_id != runtime.libraries_package_id
    {
        return Err(err(
            ErrorCode::ChecksumMismatch,
            "La selección no coincide con el runtime preparado.",
        ));
    }
    Ok(runtime)
}

fn verify_derived(store: &ManagedStore, game: &str, id: &str) -> Result<ManagedRuntime> {
    let (slug, folder) = game_spec(game)?;
    check_id(id, "profile")?;
    let path = store.root().join("profile-runtimes").join(game).join(id);
    let root = Directory::open_absolute(&path)?;
    root.assert_private()?;
    let record: DerivedRecord = decode(&root.read_file("lxmi-profile-runtime.json", MAX_JSON)?)?;
    let manifest: RuntimeManifest = decode(&root.read_file("runtime-manifest.json", MAX_JSON)?)?;
    check_id(&record.base.runtime_id, slug)?;
    let mut expected_manifest = record.base.clone();
    expected_manifest.runtime_id = id.into();
    if record.schema_version != 1
        || record.game_id != game
        || record.base.game_id != game
        || record.runtime_id != id
        || record_id(&record)? != id
        || manifest != expected_manifest
        || inventory(&root.child(folder)?, derived_limits())? != record.files
    {
        return Err(err(
            ErrorCode::ChecksumMismatch,
            "El runtime de perfil cambió. Se conserva para revisión y no se utiliza.",
        ));
    }
    let importer = store.verify(&manifest.integration_package_id)?;
    let libraries = store.verify(&manifest.libraries_package_id)?;
    let integration = if slug == "wwmi" {
        IntegrationKind::Wwmi
    } else {
        IntegrationKind::Zzmi
    };
    if manifest.integration != integration
        || importer.manifest().kind != PackageKind::GameIntegration(integration)
        || importer.manifest().authenticity != PackageAuthenticity::OfficialReleaseVerified
        || libraries.manifest().kind != PackageKind::XxmiLibraries
        || libraries.manifest().authenticity != PackageAuthenticity::OfficialReleaseVerified
    {
        return Err(err(
            ErrorCode::InvalidSignature,
            "Los paquetes fuente no están verificados para este juego.",
        ));
    }
    let loader_relative = format!(
        "packages/xxmi/{}/payload/3dmloader.dll",
        manifest.libraries_package_id
    );
    if manifest.loader_library_store_path != loader_relative
        || !libraries.manifest().files.iter().any(|f| {
            f.relative_path == "3dmloader.dll" && f.sha256 == manifest.loader_library_sha256
        })
    {
        return Err(err(
            ErrorCode::ChecksumMismatch,
            "El loader no coincide con el paquete Libraries verificado.",
        ));
    }
    Ok(ManagedRuntime {
        runtime_id: id.into(),
        app_root: path.clone(),
        importer_root: path.join(folder),
        manifest_path: path.join("runtime-manifest.json"),
        loader_library_path: store.root().join(loader_relative),
        integration_package_id: manifest.integration_package_id.clone(),
        libraries_package_id: manifest.libraries_package_id.clone(),
        integration_version: manifest.integration_version.clone(),
        libraries_version: manifest.libraries_version.clone(),
        target_process: manifest.target_process.clone(),
        loader_process_identity: manifest.loader_process_identity.clone(),
        file_count: record.files.len(),
        manifest,
        newly_created: false,
        installed_into_game: false,
    })
}

fn read_state(dir: &Directory, game: &str) -> Result<ActiveProfileSelection> {
    if dir.metadata("active-runtime.json")?.is_none() {
        return Ok(empty_state(game));
    }
    let state: ActiveProfileSelection = decode(&dir.read_file("active-runtime.json", 64 * 1024)?)?;
    if state.schema_version != 1 || state.game_id != game {
        return Err(err(
            ErrorCode::InvalidMetadata,
            "Selección de runtime inválida.",
        ));
    }
    for reference in [&state.current, &state.previous].into_iter().flatten() {
        check_id(&reference.runtime_id, "profile")?;
        check_id(&reference.base_runtime_id, game_spec(game)?.0)?;
    }
    Ok(state)
}
fn save_state(dir: &Directory, state: &mut ActiveProfileSelection) -> Result<()> {
    state.revision = state
        .revision
        .checked_add(1)
        .ok_or_else(|| err(ErrorCode::LimitExceeded, "Revisión agotada."))?;
    let name = temp_name();
    dir.write_new(&name, &encode(state)?)?;
    std::fs::rename(
        dir.path().join(&name),
        dir.path().join("active-runtime.json"),
    )
    .map_err(|e| XxmiError::io(std::path::Path::new("active-runtime.json"), e))?;
    dir.sync()
}
fn record_id(record: &DerivedRecord) -> Result<String> {
    Ok(format!(
        "profile-{}",
        digest(&encode(&(
            &record.game_id,
            &record.base,
            &record.profile,
            fingerprint(&record.files)?
        ))?)
    ))
}
fn empty_state(game: &str) -> ActiveProfileSelection {
    ActiveProfileSelection {
        schema_version: 1,
        game_id: game.into(),
        revision: 0,
        current: None,
        previous: None,
    }
}
fn game_spec(game: &str) -> Result<(&'static str, &'static str)> {
    match game {
        "wuthering-waves" => Ok(("wwmi", "WWMI")),
        "zenless-zone-zero" => Ok(("zzmi", "ZZMI")),
        _ => Err(err(ErrorCode::WrongGameIntegration, "Juego desconocido.")),
    }
}
fn check_id(id: &str, prefix: &str) -> Result<()> {
    if !id.strip_prefix(&format!("{prefix}-")).is_some_and(|v| {
        v.len() == 64
            && v.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    }) {
        return Err(err(
            ErrorCode::UnsafePath,
            "Identificador de runtime inválido.",
        ));
    }
    Ok(())
}
fn derived_limits() -> ImportLimits {
    ImportLimits {
        max_depth: 26,
        max_total_bytes: mod_limits().max_total_bytes + ImportLimits::default().max_total_bytes,
        max_files: 8192,
        ..ImportLimits::default()
    }
}

fn profile_destination(path: &str) -> String {
    if let Some((_, relative)) = path.split_once('/') {
        if is_shader_fix(relative) {
            return relative.into();
        }
    }
    format!("Mods/{path}")
}
fn temp_name() -> String {
    format!(
        "profile-runtime-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let bytes =
        serde_json::to_vec(value).map_err(|e| err(ErrorCode::InvalidMetadata, &e.to_string()))?;
    if bytes.len() as u64 > MAX_JSON {
        return Err(err(
            ErrorCode::LimitExceeded,
            "Manifiesto demasiado grande.",
        ));
    }
    Ok(bytes)
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    serde_json::from_slice(bytes).map_err(|e| err(ErrorCode::InvalidMetadata, &e.to_string()))
}
fn err(code: ErrorCode, detail: &str) -> XxmiError {
    XxmiError::new(code, None, detail)
}
