//! Synthetic mods only. These tests never inspect Steam or execute a runtime.
use lxmi_xxmi::{ErrorCode, ImportLimits, ManagedStore, ModLibrary};
use std::{
    fs,
    io::Write,
    os::unix::fs::{symlink, PermissionsExt},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const ZZZ: &str = "zenless-zone-zero";
const WUWA: &str = "wuthering-waves";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "lxmi-mods-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn library(&self) -> ModLibrary {
        ModLibrary::new(ManagedStore::at(self.0.join("lxmi"), ImportLimits::default()).unwrap())
    }
    fn source(&self, name: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(path.join("Textures")).unwrap();
        fs::write(
            path.join("skin.ini"),
            format!("; synthetic {name}\r\n[Constants]\r\nglobal $skin = 1\r\n"),
        )
        .unwrap();
        fs::write(path.join("Textures/skin.dds"), b"synthetic texture").unwrap();
        path
    }
    fn zip(&self, entries: &[(&str, &[u8])]) -> PathBuf {
        let path = self.0.join("mod.zip");
        let mut zip = zip::ZipWriter::new(fs::File::create(&path).unwrap());
        for (name, bytes) in entries {
            zip.start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap();
        path
    }
    fn package(&self, id: &str) -> PathBuf {
        self.0.join("lxmi/mods").join(ZZZ).join("packages").join(id)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn shader_replacement_pack_imports_without_ini_and_rejects_unrecognized_files() {
    let f = Fixture::new();
    let source = f.0.join("shader-pack");
    fs::create_dir_all(source.join("ShaderFixes")).unwrap();
    fs::write(
        source.join("ShaderFixes/02549f5219c55aa4-ps_replace.txt"),
        b"synthetic shader",
    )
    .unwrap();
    fs::write(source.join("README.txt"), b"Install in ShaderFixes").unwrap();
    let library = f.library();
    let item = library.import(ZZZ, "Shader pack", &source).unwrap();
    library.save_profile(ZZZ, "Shaders", &[item.id], 0).unwrap();
    let prepared = library.prepare_profile(ZZZ, "Shaders", 1).unwrap();
    assert_eq!(prepared.files.len(), 2);
    fs::write(source.join("ShaderFixes/unknown.txt"), b"unsupported").unwrap();
    assert_eq!(
        library
            .import(ZZZ, "Invalid shaders", &source)
            .unwrap_err()
            .code,
        ErrorCode::InvalidPackage
    );
    assert_eq!(library.list(ZZZ).unwrap().mods.len(), 1);
}

#[test]
fn listing_is_read_only_and_unknown_game_is_rejected() {
    let f = Fixture::new();
    assert!(f.library().list(ZZZ).unwrap().mods.is_empty());
    assert!(!f.0.join("lxmi").exists());
    assert_eq!(
        f.library().list("../Steam").unwrap_err().code,
        ErrorCode::WrongGameIntegration
    );
}

#[test]
fn imports_preserve_originals_deduplicate_and_separate_games() {
    let f = Fixture::new();
    let source = f.source("first");
    let before = fs::read(source.join("skin.ini")).unwrap();
    let lib = f.library();
    let a = lib.import(ZZZ, "First skin", &source).unwrap();
    let b = lib.import(ZZZ, "Duplicate", &source).unwrap();
    assert_eq!(a.id, b.id);
    assert_eq!(b.name, "First skin");
    assert_eq!(fs::read(source.join("skin.ini")).unwrap(), before);
    assert_eq!(lib.list(ZZZ).unwrap().mods.len(), 1);
    assert!(lib.list(WUWA).unwrap().mods.is_empty());
    let other = lib
        .import(WUWA, "Explicit WuWa assignment", &source)
        .unwrap();
    assert_eq!(other.game_id, WUWA);
    let mode = fs::metadata(f.package(&a.id).join("payload/skin.ini"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[test]
fn zip_import_and_folder_import_have_identical_content_identity() {
    let f = Fixture::new();
    let source = f.source("first");
    let ini = fs::read(source.join("skin.ini")).unwrap();
    let archive = f.zip(&[
        ("skin.ini", &ini),
        ("Textures/skin.dds", b"synthetic texture"),
    ]);
    let a = f.library().import(ZZZ, "ZIP skin", &archive).unwrap();
    let b = f.library().import(ZZZ, "Folder skin", &source).unwrap();
    assert_eq!(a.id, b.id);
    assert_eq!(f.library().list(ZZZ).unwrap().mods.len(), 1);
}

#[test]
fn rejects_zip_traversal_and_cleans_staging() {
    let f = Fixture::new();
    let archive = f.zip(&[("../escape.ini", b"[Constants]")]);
    assert_eq!(
        f.library().import(ZZZ, "Bad", &archive).unwrap_err().code,
        ErrorCode::UnsafePath
    );
    assert!(!f.0.join("escape.ini").exists());
    assert_eq!(
        fs::read_dir(f.0.join("lxmi/mods").join(ZZZ).join("staging"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn rejects_symlinks_hardlinks_and_case_collisions() {
    for kind in ["symlink", "hardlink", "case"] {
        let f = Fixture::new();
        let source = f.source("first");
        match kind {
            "symlink" => symlink(source.join("skin.ini"), source.join("alias.ini")).unwrap(),
            "hardlink" => fs::hard_link(source.join("skin.ini"), source.join("alias.ini")).unwrap(),
            _ => fs::write(source.join("SKIN.ini"), b"[Constants]").unwrap(),
        }
        assert_eq!(
            f.library().import(ZZZ, "Unsafe", &source).unwrap_err().code,
            ErrorCode::UnsafePath
        );
    }
}

#[test]
fn rejects_windows_reserved_names_and_ambiguous_components() {
    for name in [
        "CON.ini",
        "textures./skin.ini",
        "AUX/resource.dds",
        "LPT1.dds",
        "bad?.dds",
    ] {
        let f = Fixture::new();
        let source = f.source("first");
        let path = source.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"synthetic").unwrap();
        assert_eq!(
            f.library()
                .import(ZZZ, "Unsafe Windows path", &source)
                .unwrap_err()
                .code,
            ErrorCode::UnsafePath
        );
    }
}

#[test]
fn concurrent_profile_writers_cannot_lose_an_update() {
    let f = Fixture::new();
    f.library().save_profile(ZZZ, "Seed", &[], 0).unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
    let handles: Vec<_> = (0..8)
        .map(|index| {
            let library = f.library();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                library.save_profile(ZZZ, &format!("Writer {index}"), &[], 1)
            })
        })
        .collect();
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert!(results
        .iter()
        .filter_map(|r| r.as_ref().err())
        .all(|e| e.code == ErrorCode::Busy));
    assert_eq!(f.library().list(ZZZ).unwrap().revision, 2);
}

#[test]
fn rejects_non_mods_runtime_binaries_and_overlapping_sources() {
    let f = Fixture::new();
    let source = f.source("first");
    fs::remove_file(source.join("skin.ini")).unwrap();
    assert_eq!(
        f.library().import(ZZZ, "No INI", &source).unwrap_err().code,
        ErrorCode::InvalidPackage
    );
    fs::write(source.join("skin.ini"), b"[Constants]").unwrap();
    fs::write(source.join("loader.DLL"), b"not executable").unwrap();
    assert_eq!(
        f.library()
            .import(ZZZ, "Runtime", &source)
            .unwrap_err()
            .code,
        ErrorCode::InvalidPackage
    );
    assert_eq!(
        f.library().import(ZZZ, "Overlap", &f.0).unwrap_err().code,
        ErrorCode::UnsafePath
    );
}

#[test]
fn enforces_resource_limits_without_publishing_a_partial_import() {
    let f = Fixture::new();
    let source = f.source("first");
    let limits = ImportLimits {
        max_file_bytes: 4,
        ..ImportLimits::default()
    };
    let lib = ModLibrary::with_limits(ManagedStore::at(f.0.join("lxmi"), limits).unwrap(), limits);
    assert_eq!(
        lib.import(ZZZ, "Too large", &source).unwrap_err().code,
        ErrorCode::LimitExceeded
    );
    assert!(lib.list(ZZZ).unwrap().mods.is_empty());
}

#[test]
fn profile_selection_is_persistent_reversible_and_preparation_is_immutable() {
    let f = Fixture::new();
    let lib = f.library();
    let a = lib.import(ZZZ, "A", &f.source("first")).unwrap();
    let b = lib.import(ZZZ, "B", &f.source("second")).unwrap();
    let state = lib
        .save_profile(ZZZ, "Daily", &[a.id.clone(), b.id.clone()], 0)
        .unwrap();
    let original = lib.prepare_profile(ZZZ, "Daily", state.revision).unwrap();
    assert_eq!(original.files.len(), 4);
    assert!(original.mods_path.starts_with(f.0.join("lxmi")));
    assert_eq!(
        fs::read(original.mods_path.join(format!("lxmi_{}/skin.ini", a.id))).unwrap(),
        fs::read(f.0.join("first/skin.ini")).unwrap()
    );
    let state = lib
        .save_profile(ZZZ, "Daily", std::slice::from_ref(&a.id), state.revision)
        .unwrap();
    let single = lib.prepare_profile(ZZZ, "Daily", state.revision).unwrap();
    assert_ne!(original.mods_path, single.mods_path);
    assert!(original.mods_path.exists());
    assert!(!single.mods_path.join(format!("lxmi_{}", b.id)).exists());
    let reloaded = f.library().list(ZZZ).unwrap();
    assert_eq!(reloaded.profiles[0].mod_ids, vec![a.id.clone()]);
    let state = lib
        .save_profile(ZZZ, "Daily", &[a.id, b.id], reloaded.revision)
        .unwrap();
    assert_eq!(
        lib.prepare_profile(ZZZ, "Daily", state.revision)
            .unwrap()
            .mods_path,
        original.mods_path
    );
    assert!(!f.0.join("Steam").exists());
}

#[test]
fn empty_profile_prepares_an_empty_mods_tree() {
    let f = Fixture::new();
    let state = f.library().save_profile(ZZZ, "No mods", &[], 0).unwrap();
    let report = f
        .library()
        .prepare_profile(ZZZ, "No mods", state.revision)
        .unwrap();
    assert!(report.files.is_empty());
    assert_eq!(fs::read_dir(report.mods_path).unwrap().count(), 0);
}

#[test]
fn rejects_cross_game_unknown_duplicate_and_traversal_mod_references() {
    let f = Fixture::new();
    let lib = f.library();
    let a = lib.import(ZZZ, "A", &f.source("first")).unwrap();
    assert_eq!(
        lib.save_profile(WUWA, "Mixed", std::slice::from_ref(&a.id), 0)
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
    assert_eq!(
        lib.save_profile(ZZZ, "Missing", &["0".repeat(64)], 0)
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
    assert_eq!(
        lib.save_profile(ZZZ, "Duplicate", &[a.id.clone(), a.id], 0)
            .unwrap_err()
            .code,
        ErrorCode::InvalidMetadata
    );
    assert_eq!(
        lib.save_profile(ZZZ, "Escape", &["../other".into()], 0)
            .unwrap_err()
            .code,
        ErrorCode::UnsafePath
    );
}

#[test]
fn stale_writers_and_stale_preparation_cannot_overwrite_saved_state() {
    let f = Fixture::new();
    let lib = f.library();
    let saved = lib.save_profile(ZZZ, "First", &[], 0).unwrap();
    assert_eq!(
        lib.save_profile(ZZZ, "Second", &[], 0).unwrap_err().code,
        ErrorCode::Busy
    );
    assert_eq!(
        lib.prepare_profile(ZZZ, "First", 0).unwrap_err().code,
        ErrorCode::Busy
    );
    assert_eq!(lib.list(ZZZ).unwrap().profiles, saved.profiles);
}

#[test]
fn corruption_is_reported_and_blocks_profile_save_and_preparation() {
    let f = Fixture::new();
    let lib = f.library();
    let a = lib.import(ZZZ, "A", &f.source("first")).unwrap();
    let state = lib
        .save_profile(ZZZ, "First", std::slice::from_ref(&a.id), 0)
        .unwrap();
    fs::write(f.package(&a.id).join("payload/skin.ini"), b"tampered").unwrap();
    let listing = lib.list(ZZZ).unwrap();
    assert!(listing.mods.is_empty());
    assert_eq!(listing.issues[0].code, ErrorCode::ChecksumMismatch);
    assert_eq!(
        lib.save_profile(ZZZ, "First", &[a.id], state.revision)
            .unwrap_err()
            .code,
        ErrorCode::ChecksumMismatch
    );
    assert_eq!(
        lib.prepare_profile(ZZZ, "First", state.revision)
            .unwrap_err()
            .code,
        ErrorCode::ChecksumMismatch
    );
}

#[test]
fn modified_prepared_copy_is_preserved_and_never_silently_reused() {
    let f = Fixture::new();
    let lib = f.library();
    let a = lib.import(ZZZ, "A", &f.source("first")).unwrap();
    let state = lib
        .save_profile(ZZZ, "First", std::slice::from_ref(&a.id), 0)
        .unwrap();
    let report = lib.prepare_profile(ZZZ, "First", state.revision).unwrap();
    let changed = report.mods_path.join(format!("lxmi_{}/skin.ini", a.id));
    fs::write(&changed, b"changed copy").unwrap();
    assert_eq!(
        lib.prepare_profile(ZZZ, "First", state.revision)
            .unwrap_err()
            .code,
        ErrorCode::ChecksumMismatch
    );
    assert_eq!(fs::read(changed).unwrap(), b"changed copy");
    assert_eq!(lib.list(ZZZ).unwrap().mods.len(), 1);
}

#[test]
fn metadata_symlinks_are_rejected_without_writing_their_target() {
    let f = Fixture::new();
    let lib = f.library();
    lib.save_profile(ZZZ, "First", &[], 0).unwrap();
    let path = f.0.join("lxmi/mods").join(ZZZ).join("profiles.json");
    let outside = f.0.join("outside.json");
    fs::write(&outside, b"sentinel").unwrap();
    fs::remove_file(&path).unwrap();
    symlink(&outside, &path).unwrap();
    assert_eq!(
        lib.save_profile(ZZZ, "Other", &[], 1).unwrap_err().code,
        ErrorCode::UnsafePath
    );
    assert_eq!(fs::read(outside).unwrap(), b"sentinel");
}

fn model_source(f: &Fixture) -> PathBuf {
    let path = f.0.join("model");
    fs::create_dir_all(path.join("Meshes")).unwrap();
    fs::write(
        path.join("model.ini"),
        "[TextureOverrideTestaBodyPosition]\r\nvb0 = ResourceTestaBodyPosition\r\n\
         [TextureOverrideTestaBodyA]\r\nib = ResourceTestaBodyIB\r\ndrawindexed = 6, 0, 0\r\n\
         [ResourceTestaBodyPosition]\r\nstride = 40\r\nfilename = Meshes\\TestaBodyPosition.buf\r\n\
         [ResourceTestaBodyIB]\r\nformat = DXGI_FORMAT_R32_UINT\r\nfilename = Meshes\\TestaBodyA.ib\r\n",
    )
    .unwrap();
    let mut vb = vec![];
    for [x, z] in [[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]] {
        for v in [x, 0.0, z, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0] {
            vb.extend(f32::to_le_bytes(v));
        }
    }
    fs::write(path.join("Meshes/TestaBodyPosition.buf"), vb).unwrap();
    let ib: Vec<u8> = [0u32, 1, 2, 0, 2, 3]
        .iter()
        .flat_map(|i| i.to_le_bytes())
        .collect();
    fs::write(path.join("Meshes/TestaBodyA.ib"), ib).unwrap();
    path
}

#[test]
fn previews_are_optional_cached_and_removable() {
    use lxmi_xxmi::{ModKind, PreviewState, PreviewView};
    let f = Fixture::new();
    let library = f.library();
    assert!(library.mod_details(ZZZ).unwrap().is_empty());
    let model = library
        .import(ZZZ, "Testa outfit", &model_source(&f))
        .unwrap();
    let texture = library.import(ZZZ, "Skin", &f.source("skin")).unwrap();

    let details = library.mod_details(ZZZ).unwrap();
    let of = |id: &str| details.iter().find(|d| d.mod_id == id).unwrap().clone();
    assert_eq!(of(&model.id).insight.kind, ModKind::Model);
    assert_eq!(of(&model.id).insight.character.as_deref(), Some("Testa"));
    assert_eq!(of(&model.id).preview, PreviewState::Missing);
    assert!(!of(&texture.id).insight.previewable);

    let generated = library.generate_mod_preview(ZZZ, &model.id).unwrap();
    assert_eq!(generated.preview, PreviewState::Ready);
    for view in PreviewView::ALL {
        let png = library.mod_preview(ZZZ, &model.id, view).unwrap();
        assert_eq!(&png[..4], b"\x89PNG");
    }
    // Cached: a second request does not render again or fail.
    assert_eq!(
        library
            .generate_mod_preview(ZZZ, &model.id)
            .unwrap()
            .preview,
        PreviewState::Ready
    );
    assert_eq!(
        library
            .generate_mod_preview(ZZZ, &texture.id)
            .unwrap()
            .preview,
        PreviewState::Unavailable
    );
    assert_eq!(
        library
            .mod_preview(ZZZ, &texture.id, PreviewView::Front)
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );

    library.clear_mod_previews(ZZZ).unwrap();
    assert!(library
        .mod_details(ZZZ)
        .unwrap()
        .iter()
        .all(|d| d.preview == PreviewState::Missing));
    // The mods themselves are untouched.
    assert_eq!(library.list(ZZZ).unwrap().mods.len(), 2);
}
