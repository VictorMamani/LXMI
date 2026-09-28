use crate::{
    filesystem::{missing, Directory},
    package::{fingerprint, inspect_package, WWMI_REFERENCE, ZZMI_REFERENCE},
    *,
};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_STAGE: AtomicU64 = AtomicU64::new(0);
/// Verified handles are created only by re-reading managed bytes, never by trusting IPC metadata.
#[derive(Debug, Clone)]
pub struct VerifiedPackage {
    pub(crate) manifest: PackageManifest,
    pub(crate) payload_path: PathBuf,
}
impl VerifiedPackage {
    pub fn manifest(&self) -> &PackageManifest {
        &self.manifest
    }
    pub fn payload_path(&self) -> &Path {
        &self.payload_path
    }
}

#[derive(Debug, Clone)]
pub struct ManagedStore {
    root: PathBuf,
    limits: ImportLimits,
}
impl ManagedStore {
    /// Construction and listing are read-only. Only explicit import creates storage.
    pub fn from_system(info: &lxmi_core::SystemInfo) -> Result<Self> {
        let base = info.xdg_data_home.as_ref().ok_or_else(|| {
            XxmiError::new(
                ErrorCode::StorageUnavailable,
                None,
                "XDG_DATA_HOME/home no disponible.",
            )
        })?;
        Self::at(base.join("lxmi"), ImportLimits::default())
    }
    /// Explicit root supports isolated tests/development. Desktop IPC never accepts a storage root.
    pub fn at(root: PathBuf, limits: ImportLimits) -> Result<Self> {
        if !root.is_absolute() || root.file_name().and_then(|v| v.to_str()) != Some("lxmi") {
            return Err(XxmiError::new(
                ErrorCode::UnsafePath,
                Some(root),
                "El root administrado debe ser absoluto y terminar en lxmi.",
            ));
        }
        // Do not normalize away traversal.
        for part in root.components() {
            if !matches!(
                part,
                std::path::Component::RootDir | std::path::Component::Normal(_)
            ) {
                return Err(XxmiError::new(
                    ErrorCode::UnsafePath,
                    Some(root),
                    "Root no normalizado.",
                ));
            }
        }
        Ok(Self { root, limits })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn ensure_outside(&self, protected: &[PathBuf]) -> Result<()> {
        for p in protected {
            if self.root.starts_with(p) || p.starts_with(&self.root) {
                return Err(XxmiError::new(
                    ErrorCode::UnsafePath,
                    Some(self.root.clone()),
                    "El almacenamiento LXMI se solapa con Steam, juego o prefix.",
                ));
            }
        }
        Ok(())
    }
    fn existing(&self) -> Result<Directory> {
        let d = Directory::open_absolute(&self.root)?;
        d.assert_private()?;
        Ok(d)
    }
    fn initialize(&self) -> Result<Directory> {
        // XDG base must exist: do not create arbitrary ancestors, nor write outside LXMI.
        let parent = self.root.parent().ok_or_else(|| {
            XxmiError::new(ErrorCode::StorageUnavailable, None, "Sin directorio XDG.")
        })?;
        let base = Directory::open_absolute(parent)?;
        let root = base.private_child("lxmi")?;
        root.private_child("staging")?;
        root.private_child("packages")?.private_child("xxmi")?;
        Ok(root)
    }
    pub fn list(&self) -> Result<Vec<PackageManifest>> {
        let root = match self.existing() {
            Ok(d) => d,
            Err(e) if missing(&e) => return Ok(vec![]),
            Err(e) => return Err(e),
        };
        let packages = match root.descend("packages/xxmi") {
            Ok(d) => d,
            Err(e) if missing(&e) => return Ok(vec![]),
            Err(e) => return Err(e),
        };
        let mut result = Vec::new();
        for id in packages.entries(256)? {
            result.push(self.verify(&id)?.manifest);
        }
        Ok(result)
    }
    pub fn verify(&self, id: &str) -> Result<VerifiedPackage> {
        validate_id(id)?;
        let root = self.existing()?;
        let package = root.descend(&format!("packages/xxmi/{id}"))?;
        package.assert_private()?;
        let manifest: PackageManifest =
            serde_json::from_slice(&package.read_file("lxmi-package.json", 2 * 1024 * 1024)?)
                .map_err(|e| XxmiError::new(ErrorCode::InvalidMetadata, None, e.to_string()))?;
        let inspection = inspect_package(&package.child("payload")?, self.limits)?;
        if manifest.schema_version != 1
            || manifest.ecosystem != "xxmi"
            || manifest.id != id
            || fingerprint(&inspection.files)? != id
            || manifest.files != inspection.files
            || manifest.kind != inspection.kind
            || manifest.version != inspection.version
        {
            tracing::warn!("Checksum mismatch");
            return Err(XxmiError::new(ErrorCode::ChecksumMismatch,Some(self.root.join("packages/xxmi").join(id)),"El inventario, hash o metadata administrada no coincide con los archivos actuales."));
        }
        Ok(VerifiedPackage {
            manifest,
            payload_path: self.root.join("packages/xxmi").join(id).join("payload"),
        })
    }
    pub fn import_directory(&self, source: &Path) -> Result<VerifiedPackage> {
        tracing::info!("Package import started");
        if source.starts_with(&self.root) || self.root.starts_with(source) {
            return Err(XxmiError::new(
                ErrorCode::UnsafePath,
                Some(source.into()),
                "Fuente y almacenamiento no pueden solaparse.",
            ));
        }
        if source.extension().is_some_and(|e| {
            ["zip", "7z", "rar"]
                .iter()
                .any(|x| e.eq_ignore_ascii_case(x))
        }) {
            return Err(XxmiError::new(
                ErrorCode::UnsupportedArchive,
                Some(source.into()),
                "0.5 solo importa directorios locales ya extraídos.",
            ));
        }
        let input = Directory::open_absolute(source)?;
        let inspected = inspect_package(&input, self.limits)?;
        let id = fingerprint(&inspected.files)?;
        let root = self.initialize()?;
        let staging = root.child("staging")?;
        let packages = root.descend("packages/xxmi")?;
        if packages.metadata(&id)?.is_some() {
            return self.verify(&id);
        }
        // Exclusive creation and no-replace promotion support concurrent imports safely.
        let stage_name = format!(
            "import-{}-{}",
            std::process::id(),
            NEXT_STAGE.fetch_add(1, Ordering::Relaxed)
        );
        if staging.metadata(&stage_name)?.is_some() {
            return Err(XxmiError::new(
                ErrorCode::Busy,
                None,
                "Staging preexistente; no se reutiliza.",
            ));
        }
        let stage = staging.new_private_child(&stage_name)?;
        let result = (|| {
            let payload = stage.private_child("payload")?;
            for entry in &inspected.files {
                let bytes = input.read_file(&entry.relative_path, self.limits.max_file_bytes)?;
                if bytes.len() as u64 != entry.size
                    || format!("{:x}", Sha256::digest(&bytes)) != entry.sha256
                {
                    return Err(XxmiError::new(
                        ErrorCode::ChecksumMismatch,
                        Some(source.join(&entry.relative_path)),
                        "La fuente cambió durante el import.",
                    ));
                }
                payload.ensure_parents(&entry.relative_path)?;
                payload.write_new(&entry.relative_path, &bytes)?;
            }
            // Revalidate staged copy, not just source. No upstream bytes are edited.
            let staged = inspect_package(&payload, self.limits)?;
            if staged.files != inspected.files {
                return Err(XxmiError::new(
                    ErrorCode::ChecksumMismatch,
                    None,
                    "Staging no coincide con la fuente.",
                ));
            }
            let manifest = PackageManifest {
                schema_version: 1,
                id: id.clone(),
                ecosystem: "xxmi".into(),
                kind: staged.kind,
                version: staged.version,
                source: PackageSource::LocalDirectory {
                    path: source.into(),
                },
                imported_unix_seconds: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| {
                        XxmiError::new(ErrorCode::Io, None, "Reloj anterior a Unix epoch.")
                    })?
                    .as_secs(),
                files: staged.files,
                layout_reference: match staged.kind {
                    PackageKind::XxmiLibraries => {
                        "SpectrumQT/XXMI-Libs-Package v1.1.7 layout; signatures not verified".into()
                    }
                    PackageKind::GameIntegration(IntegrationKind::Wwmi) => WWMI_REFERENCE.into(),
                    PackageKind::GameIntegration(IntegrationKind::Zzmi) => ZZMI_REFERENCE.into(),
                    PackageKind::GameIntegration(
                        IntegrationKind::Gimi | IntegrationKind::Unknown,
                    ) => "Unverified XXMI integration layout".into(),
                },
                authenticity: PackageAuthenticity::NotAuthenticated,
            };
            let json = serde_json::to_vec_pretty(&manifest)
                .map_err(|e| XxmiError::new(ErrorCode::InvalidMetadata, None, e.to_string()))?;
            stage.write_new("lxmi-package.json", &json)?;
            payload.sync()?;
            stage.sync()?;
            staging.promote(&stage_name, &packages, &id)?;
            tracing::info!(package_id=%id,"Package promoted to managed store");
            self.verify(&id)
        })();
        if result.is_err() {
            // Cleanup only our stage. Failed cleanup is observable; stale stages are never listed as packages.
            if let Err(error) = staging.remove_stage(&stage_name) {
                if !missing(&error) {
                    tracing::warn!(code=?error.code,"Incomplete stage requires manual cleanup");
                }
            }
        }
        result
    }
}
fn validate_id(id: &str) -> Result<()> {
    if id.len() != 64
        || !id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(XxmiError::new(
            ErrorCode::UnsafePath,
            None,
            "ID SHA-256 inválido.",
        ));
    }
    Ok(())
}
