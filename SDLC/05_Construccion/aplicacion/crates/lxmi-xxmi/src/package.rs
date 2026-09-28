use crate::{
    filesystem::{missing, Directory},
    *,
};
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::{Path, PathBuf},
};

// Structural contract checked against WWMI-PACKAGE-v1.0.0.zip, not an authenticity signature.
pub const WWMI_REQUIRED_FILES: &[&str] = &[
    "d3dx.ini",
    "Core/WWMI/WuWa-Model-Importer.ini",
    "Core/WWMI/KeyBindings.ini",
    "Core/WWMI/WWMI-Utilities.ini",
    "Core/WWMI/help.ini",
    "Core/WWMI/Fonts/LiberationSans-Bold.dds",
    "Core/WWMI/Shaders/BlendRemapper.hlsl",
    "Core/WWMI/Shaders/ShapeKeyApplier.hlsl",
    "Core/WWMI/Shaders/ShapeKeyLoader.hlsl",
    "Core/WWMI/Shaders/ShapeKeyMultiplier.hlsl",
    "Core/WWMI/Shaders/ShapeKeyOverrider.hlsl",
    "Core/WWMI/Shaders/SkapeKeySetter.hlsl",
    "Core/WWMI/Shaders/SkeletonMerger.hlsl",
    "Core/WWMI/Shaders/SkeletonRemapper.hlsl",
    "Core/WWMI/Shaders/TextPrinter.hlsl",
    "Core/WWMI/Notifications/ErrorCompatibilityModeDisabled.md",
    "Core/WWMI/Notifications/ErrorOldVersionMod.md",
    "Core/WWMI/Notifications/ErrorOldVersionWWMI.md",
    "Core/WWMI/Notifications/HuntingModeGuide.md",
    "Core/WWMI/Notifications/UserGuide.md",
];
/// Bounded structural contract checked against the current upstream ZZMI package tree.
/// The package is not authenticated by this inspection.
pub const ZZMI_REQUIRED_FILES: &[&str] = &[
    "d3dx.ini",
    "Core/ZZMI/main.ini",
    "Core/ZZMI/d3dx_patch.ini",
    "Core/ZZMI/help.ini",
    "Core/ZZMI/Libraries/Includes.ini",
];
const LIBRARY_FILES: &[&str] = &[
    "3dmloader.dll",
    "d3d11.dll",
    "d3dcompiler_47.dll",
    "Manifest.json",
];
pub(crate) const WWMI_REFERENCE: &str =
    "SpectrumQT/WWMI-Package release v1.0.0; layout only; signatures not verified";
pub(crate) const ZZMI_REFERENCE: &str =
    "leotorrez/ZZMI-Package current upstream tree; layout only; signatures not verified";

pub fn validate_directory(path: &Path, limits: ImportLimits) -> Result<PackageInspection> {
    let dir = Directory::open_absolute(path)?;
    inspect_package(&dir, limits)
}

pub(crate) fn inventory(dir: &Directory, limits: ImportLimits) -> Result<Vec<PackageFile>> {
    let mut files = Vec::new();
    let mut total = 0;
    let mut entries = 0;
    walk(dir, "", limits, &mut files, &mut total, &mut entries)?;
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(files)
}
fn walk(
    dir: &Directory,
    prefix: &str,
    limits: ImportLimits,
    files: &mut Vec<PackageFile>,
    total: &mut u64,
    entries: &mut usize,
) -> Result<()> {
    if prefix.split('/').count() > limits.max_depth {
        return Err(XxmiError::new(
            ErrorCode::LimitExceeded,
            None,
            "Profundidad máxima excedida.",
        ));
    }
    for name in dir.entries(limits.max_files)? {
        *entries += 1;
        if *entries > limits.max_files {
            return Err(XxmiError::new(
                ErrorCode::LimitExceeded,
                None,
                "Demasiadas entradas en el paquete.",
            ));
        }
        let rel = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        validate_relative_path(&rel)?;
        let meta = dir.metadata(&name)?.ok_or_else(|| {
            XxmiError::new(
                ErrorCode::InvalidPackage,
                Some(PathBuf::from(&rel)),
                "Entrada desapareció durante la inspección.",
            )
        })?;
        if meta.is_dir() {
            walk(&dir.child(&name)?, &rel, limits, files, total, entries)?;
        } else {
            let (size, hash) = hash_file(dir, &name, limits.max_file_bytes)?;
            *total = total.checked_add(size).ok_or_else(|| {
                XxmiError::new(ErrorCode::LimitExceeded, None, "Overflow de tamaño.")
            })?;
            if *total > limits.max_total_bytes {
                return Err(XxmiError::new(
                    ErrorCode::LimitExceeded,
                    None,
                    "Tamaño total excedido.",
                ));
            }
            files.push(PackageFile {
                relative_path: rel,
                size,
                sha256: hash,
            });
        }
    }
    // Windows filenames are case insensitive: reject ambiguous case collisions.
    let mut seen = std::collections::HashSet::new();
    for f in files.iter() {
        if !seen.insert(f.relative_path.to_lowercase()) {
            return Err(XxmiError::new(
                ErrorCode::UnsafePath,
                Some(PathBuf::from(&f.relative_path)),
                "Colisión de nombres ignorando mayúsculas.",
            ));
        }
    }
    Ok(())
}
pub(crate) fn hash_file(dir: &Directory, name: &str, limit: u64) -> Result<(u64, String)> {
    let mut f = dir.open_file(name)?;
    let mut h = Sha256::new();
    let mut buf = [0u8; 65536];
    let mut size = 0u64;
    loop {
        let n = f
            .read(&mut buf)
            .map_err(|e| XxmiError::io(Path::new(name), e))?;
        if n == 0 {
            break;
        }
        size += n as u64;
        if size > limit {
            return Err(XxmiError::new(
                ErrorCode::LimitExceeded,
                Some(PathBuf::from(name)),
                "Archivo demasiado grande.",
            ));
        }
        h.update(&buf[..n]);
    }
    Ok((size, format!("{:x}", h.finalize())))
}
pub(crate) fn fingerprint(files: &[PackageFile]) -> Result<String> {
    let bytes = serde_json::to_vec(files)
        .map_err(|e| XxmiError::new(ErrorCode::InvalidMetadata, None, e.to_string()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
fn text(dir: &Directory, name: &str) -> Result<String> {
    String::from_utf8(dir.read_file(name, 2 * 1024 * 1024)?).map_err(|_| {
        XxmiError::new(
            ErrorCode::InvalidMetadata,
            Some(PathBuf::from(name)),
            "Metadata no UTF-8.",
        )
    })
}
fn require(files: &[PackageFile], required: &[&str]) -> Result<()> {
    let missing = required
        .iter()
        .filter(|name| {
            !files
                .iter()
                .any(|f| f.relative_path == **name && f.size > 0)
        })
        .copied()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(XxmiError::new(
            ErrorCode::MissingRequiredFile,
            None,
            format!(
                "Faltan archivos requeridos o están vacíos: {}",
                missing.join(", ")
            ),
        ));
    }
    Ok(())
}
// Read a few declared metadata keys only. This is not an XXMI INI interpreter/transformer.
fn assignment<'a>(input: &'a str, key: &str) -> Option<&'a str> {
    input
        .lines()
        .filter(|l| !l.trim_start().starts_with([';', '#']))
        .filter_map(|l| l.split_once('='))
        .find_map(|(k, v)| (k.trim() == key).then(|| v.trim()))
}
pub(crate) fn inspect_package(dir: &Directory, limits: ImportLimits) -> Result<PackageInspection> {
    let files = inventory(dir, limits)?;
    let has_wwmi = files
        .iter()
        .any(|f| f.relative_path.starts_with("Core/WWMI/"));
    let has_zzmi = files
        .iter()
        .any(|f| f.relative_path.starts_with("Core/ZZMI/"));
    if has_wwmi && has_zzmi {
        return Err(XxmiError::new(
            ErrorCode::InvalidPackage,
            None,
            "El directorio mezcla payloads WWMI y ZZMI; importa una raíz de integración por separado.",
        ));
    }
    if has_wwmi {
        require(&files, WWMI_REQUIRED_FILES)?;
        let ini = text(dir, "Core/WWMI/WuWa-Model-Importer.ini")?;
        if assignment(&ini, "namespace") != Some("WWMIv1") {
            return Err(XxmiError::new(
                ErrorCode::UnsupportedIntegration,
                None,
                "Namespace WWMI desconocido; layout soportado: WWMIv1.",
            ));
        }
        let config = text(dir, "d3dx.ini")?;
        if !config
            .lines()
            .filter_map(|l| l.split_once('='))
            .any(|(k, v)| {
                k.trim() == "include"
                    && v.trim().replace('\\', "/") == "Core/WWMI/WuWa-Model-Importer.ini"
            })
        {
            return Err(XxmiError::new(
                ErrorCode::InvalidPackage,
                None,
                "d3dx.ini no referencia el núcleo WWMI conocido.",
            ));
        }
        let version = assignment(&ini, "global $wwmi_version").map(|raw| VersionInfo {
            raw: raw.to_owned(),
            evidence:
                "Core/WWMI/WuWa-Model-Importer.ini: global $wwmi_version (raw; not release tag)"
                    .into(),
        });
        Ok(PackageInspection{kind:PackageKind::GameIntegration(IntegrationKind::Wwmi),version,files,evidence:vec!["Known WWMIv1 namespace, include and required resource inventory present; authenticity and launch not verified".into()]})
    } else if has_zzmi {
        validate_zzmi_package(dir, files)
    } else if files
        .iter()
        .any(|f| f.relative_path == "Manifest.json" || f.relative_path == "d3d11.dll")
    {
        require(&files, LIBRARY_FILES)?;
        let manifest: serde_json::Value = serde_json::from_str(&text(dir, "Manifest.json")?)
            .map_err(|e| XxmiError::new(ErrorCode::InvalidMetadata, None, e.to_string()))?;
        let signatures = manifest
            .get("signatures")
            .and_then(|s| s.as_object())
            .ok_or_else(|| {
                XxmiError::new(
                    ErrorCode::InvalidMetadata,
                    None,
                    "Manifest.json no contiene signatures.",
                )
            })?;
        for name in &LIBRARY_FILES[..3] {
            if !signatures
                .get(*name)
                .and_then(|v| v.as_str())
                .is_some_and(|s| !s.is_empty())
            {
                return Err(XxmiError::new(
                    ErrorCode::InvalidMetadata,
                    None,
                    format!(
                        "Falta declaración de firma para {name}. No se verifican firmas en 0.5."
                    ),
                ));
            }
        }
        let version = manifest
            .get("version")
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .map(|raw| VersionInfo {
                raw: raw.into(),
                evidence: "Manifest.json (unverified upstream claim)".into(),
            });
        Ok(PackageInspection{kind:PackageKind::XxmiLibraries,version,files,evidence:vec!["Three XXMI library files plus declared signature manifest; signatures NOT verified".into()]})
    } else {
        Err(XxmiError::new(ErrorCode::UnsupportedIntegration,None,"No se reconoce un paquete WWMIv1, ZZMI ni XXMI Libraries. Selecciona la raíz del payload, no la carpeta contenedora."))
    }
}

fn validate_zzmi_package(dir: &Directory, files: Vec<PackageFile>) -> Result<PackageInspection> {
    require(&files, ZZMI_REQUIRED_FILES)?;
    let config = text(dir, "d3dx.ini")?;
    let expected_target = assignment(&config, "target").is_some_and(|target| {
        ["ZenlessZoneZero.exe", "ZenlessZoneZeroBeta.exe"]
            .iter()
            .any(|expected| target.eq_ignore_ascii_case(expected))
    });
    let includes_main = config
        .lines()
        .filter(|line| !line.trim_start().starts_with([';', '#']))
        .filter_map(|line| line.split_once('='))
        .any(|(key, value)| {
            key.trim() == "include" && value.trim().replace('\\', "/") == "Core/ZZMI/main.ini"
        });
    if !expected_target || !includes_main {
        return Err(XxmiError::new(
            ErrorCode::InvalidPackage,
            None,
            "d3dx.ini no declara el ejecutable ZZMI conocido ni incluye Core/ZZMI/main.ini.",
        ));
    }

    let main_ini = text(dir, "Core/ZZMI/main.ini")?;
    let version = assignment(&main_ini, "global $version")
        .filter(|raw| !raw.is_empty())
        .map(|raw| VersionInfo {
            raw: raw.to_owned(),
            evidence: "Core/ZZMI/main.ini: global $version (raw; not an authenticated release tag)"
                .into(),
        });
    Ok(PackageInspection {
        kind: PackageKind::GameIntegration(IntegrationKind::Zzmi),
        version,
        files,
        evidence: vec![
            "Current upstream ZZMI root/config/resource paths are present; package authenticity, XXMI Libraries dependency, and launch compatibility are not verified by structural inspection.".into(),
        ],
    })
}

pub fn inspect_runtime(path: &Path) -> RuntimeDiscovery {
    tracing::info!("XXMI discovery started");
    let mut out = RuntimeDiscovery {
        path: path.to_owned(),
        integration_kind: None,
        integration: Presence::Absent,
        libraries: Presence::Absent,
        version: None,
        evidence: Vec::new(),
        issues: Vec::new(),
        launch_compatibility: LaunchCompatibility::NotVerified,
    };
    let dir = match Directory::open_absolute(path) {
        Ok(d) => d,
        Err(e) if missing(&e) => return out,
        Err(e) => {
            out.integration = Presence::Unreadable;
            out.libraries = Presence::Unreadable;
            out.issues.push(e);
            return out;
        }
    };
    // Inspect only bounded known files; never recursively scan an entire game.
    let mut files = Vec::new();
    let known_files = WWMI_REQUIRED_FILES
        .iter()
        .chain(ZZMI_REQUIRED_FILES.iter())
        .chain(LIBRARY_FILES.iter())
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    for name in known_files {
        match hash_file(&dir, name, 128 * 1024 * 1024) {
            Ok((size, sha256)) => files.push(PackageFile {
                relative_path: (*name).into(),
                size,
                sha256,
            }),
            Err(e) if missing(&e) => {}
            Err(e) => out.issues.push(e),
        }
    }
    if files
        .iter()
        .any(|f| f.relative_path.starts_with("Core/WWMI/") || f.relative_path == "d3dx.ini")
    {
        out.integration = Presence::Incomplete;
        match require(&files, WWMI_REQUIRED_FILES) {
            Ok(()) => {
                match (
                    text(&dir, "Core/WWMI/WuWa-Model-Importer.ini"),
                    text(&dir, "d3dx.ini"),
                ) {
                    (Ok(ini), Ok(config))
                        if assignment(&ini, "namespace") == Some("WWMIv1")
                            && config.lines().filter_map(|l| l.split_once('=')).any(
                                |(k, v)| {
                                    k.trim() == "include"
                                        && v.trim().replace('\\', "/")
                                            == "Core/WWMI/WuWa-Model-Importer.ini"
                                },
                            ) =>
                    {
                        out.integration = Presence::StructurallyPresent;
                        out.integration_kind = Some(IntegrationKind::Wwmi);
                        out.version =
                            assignment(&ini, "global $wwmi_version").map(|raw| VersionInfo {
                                raw: raw.into(),
                                evidence: "WWMI INI declaration; not authenticated".into(),
                            });
                        out.evidence
                            .push("WWMI known configuration and resource files present".into());
                    }
                    _ => out.issues.push(XxmiError::new(
                        ErrorCode::InvalidMetadata,
                        Some(path.into()),
                        "WWMI namespace/include no verificables.",
                    )),
                }
            }
            Err(e) => out.issues.push(e),
        }
    }
    if files
        .iter()
        .any(|file| file.relative_path.starts_with("Core/ZZMI/"))
    {
        out.integration = Presence::Incomplete;
        let zzmi_files = files
            .iter()
            .filter(|file| ZZMI_REQUIRED_FILES.contains(&file.relative_path.as_str()))
            .cloned()
            .collect();
        match validate_zzmi_package(&dir, zzmi_files) {
            Ok(inspection) => {
                out.integration = Presence::StructurallyPresent;
                out.integration_kind = Some(IntegrationKind::Zzmi);
                out.version = inspection.version;
                out.evidence.extend(inspection.evidence);
            }
            Err(error) => out.issues.push(error),
        }
    }
    if files.iter().any(|f| {
        ["d3d11.dll", "3dmloader.dll", "d3dcompiler_47.dll"].contains(&f.relative_path.as_str())
    }) {
        // Deployed importers may omit 3dmloader/manifest: report incomplete, never infer provenance.
        out.libraries = Presence::Incomplete;
        if require(&files, LIBRARY_FILES).is_ok() {
            if let Ok(manifest) = text(&dir, "Manifest.json").and_then(|s| {
                serde_json::from_str::<serde_json::Value>(&s)
                    .map_err(|e| XxmiError::new(ErrorCode::InvalidMetadata, None, e.to_string()))
            }) {
                if LIBRARY_FILES[..3].iter().all(|n| {
                    manifest["signatures"][*n]
                        .as_str()
                        .is_some_and(|s| !s.is_empty())
                }) {
                    out.libraries = Presence::StructurallyPresent;
                    out.evidence.push(
                        "Library set and signature declarations present; authenticity unknown"
                            .into(),
                    );
                }
            }
        }
    }
    if !out.issues.is_empty()
        && out.issues.iter().any(|e| {
            matches!(
                e.code,
                ErrorCode::PermissionDenied | ErrorCode::UnsafePath | ErrorCode::Io
            )
        })
    {
        out.evidence.push(
            "Discovery limited by unreadable or unsafe paths; absence is not conclusive".into(),
        );
        if out.integration == Presence::Absent {
            out.integration = Presence::Unreadable;
        }
        if out.libraries == Presence::Absent {
            out.libraries = Presence::Unreadable;
        }
    }
    if out.integration_kind.is_some()
        && files
            .iter()
            .any(|file| file.relative_path.starts_with("Core/WWMI/"))
        && files
            .iter()
            .any(|file| file.relative_path.starts_with("Core/ZZMI/"))
    {
        out.integration_kind = None;
        out.integration = Presence::Incomplete;
        out.issues.push(XxmiError::new(
            ErrorCode::InvalidPackage,
            Some(path.to_owned()),
            "Se observaron estructuras WWMI y ZZMI juntas; no se asigna una integración única.",
        ));
    }
    tracing::info!(integration=?out.integration,libraries=?out.libraries,"XXMI discovery completed");
    out
}
