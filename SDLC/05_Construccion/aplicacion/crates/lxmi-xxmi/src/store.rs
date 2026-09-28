use crate::{
    filesystem::{missing, Directory},
    package::{fingerprint, hash_file, inspect_package, WWMI_REFERENCE, ZZMI_REFERENCE},
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
        if !matches!(manifest.schema_version, 1 | 2)
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
                        "SpectrumQT/XXMI-Libs-Package package layout; local import does not authenticate signatures".into()
                    }
                    PackageKind::GameIntegration(IntegrationKind::Wwmi) => WWMI_REFERENCE.into(),
                    PackageKind::GameIntegration(IntegrationKind::Zzmi) => ZZMI_REFERENCE.into(),
                    PackageKind::GameIntegration(
                        IntegrationKind::Gimi | IntegrationKind::Unknown,
                    ) => "Unverified XXMI integration layout".into(),
                },
                authenticity: PackageAuthenticity::NotAuthenticated,
                upstream: None,
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

    /// Imports a pinned official release archive after verifying the upstream asset signature,
    /// optional GitHub SHA-256, safe ZIP structure, and (for XXMI Libraries) DLL signatures.
    /// Writes only inside the LXMI-owned managed store.
    pub fn import_official_archive(
        &self,
        release: &UpstreamRelease,
        archive_path: &Path,
        companion: Option<(&ReleaseAsset, &Path)>,
    ) -> Result<VerifiedPackage> {
        if release.source_trust != SourceTrust::Official
            || release.repository != crate::official_repository(&release.package_kind)
        {
            return Err(XxmiError::new(
                ErrorCode::ReleaseUnavailable,
                None,
                "Fuente no está en la allowlist oficial.",
            ));
        }
        let expected_name = format!(
            "{}-v{}.zip",
            crate::release::expected_asset_name(&release.package_kind, &release.tag)?,
            release.version
        );
        let asset = crate::asset_for_release(release, &expected_name)?;
        let archive_parent = archive_path.parent().ok_or_else(|| {
            XxmiError::new(
                ErrorCode::UnsafePath,
                Some(archive_path.into()),
                "Asset sin padre.",
            )
        })?;
        let archive_dir = Directory::open_absolute(archive_parent)?;
        let archive_name = archive_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                XxmiError::new(
                    ErrorCode::UnsafePath,
                    Some(archive_path.into()),
                    "Nombre del asset inválido.",
                )
            })?;
        let (archive_size, archive_sha256) =
            hash_file(&archive_dir, archive_name, self.limits.max_archive_bytes)?;
        if archive_size != asset.size
            || asset
                .sha256
                .as_ref()
                .is_some_and(|expected| expected != &archive_sha256)
        {
            return Err(XxmiError::new(
                ErrorCode::ChecksumMismatch,
                Some(archive_path.into()),
                "El tamaño o SHA-256 del asset no coincide con la metadata de release.",
            ));
        }
        let signature_status = crate::verify_release_file(
            &release.package_kind,
            release.signature_base64.as_deref(),
            archive_path,
        );
        if signature_status != SignatureStatus::Verified {
            return Err(XxmiError::new(
                match signature_status {
                    SignatureStatus::Missing => ErrorCode::MissingSignature,
                    _ => ErrorCode::InvalidSignature,
                },
                Some(archive_path.into()),
                format!("La firma upstream del ZIP no se verificó: {signature_status:?}."),
            ));
        }

        let companion_bytes = match (&release.package_kind, companion) {
            (OfficialPackageKind::XxmiLibraries, Some((metadata, path))) => {
                if metadata.name != "Manifest.json" {
                    return Err(XxmiError::new(
                        ErrorCode::InvalidMetadata,
                        Some(path.into()),
                        "El asset companion esperado es Manifest.json.",
                    ));
                }
                let parent = path.parent().ok_or_else(|| {
                    XxmiError::new(
                        ErrorCode::UnsafePath,
                        Some(path.into()),
                        "Manifest sin padre.",
                    )
                })?;
                let dir = Directory::open_absolute(parent)?;
                let name = path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .ok_or_else(|| {
                        XxmiError::new(
                            ErrorCode::UnsafePath,
                            Some(path.into()),
                            "Nombre Manifest inválido.",
                        )
                    })?;
                let (size, hash) = hash_file(&dir, name, 64 * 1024)?;
                if size != metadata.size
                    || metadata
                        .sha256
                        .as_ref()
                        .is_some_and(|expected| expected != &hash)
                {
                    return Err(XxmiError::new(
                        ErrorCode::ChecksumMismatch,
                        Some(path.into()),
                        "El Manifest companion no coincide con la metadata oficial.",
                    ));
                }
                Some(dir.read_file(name, 64 * 1024)?)
            }
            (OfficialPackageKind::XxmiLibraries, None) => {
                return Err(XxmiError::new(
                    ErrorCode::MissingRequiredFile,
                    None,
                    "XXMI Libraries requiere su asset Manifest.json separado.",
                ));
            }
            (OfficialPackageKind::Zzmi, Some(_)) => {
                return Err(XxmiError::new(
                    ErrorCode::InvalidMetadata,
                    None,
                    "ZZMI no declara un asset companion.",
                ));
            }
            (OfficialPackageKind::Zzmi, None) => None,
        };

        let root = self.initialize()?;
        let staging = root.child("staging")?;
        let packages = root.descend("packages/xxmi")?;
        let stage_name = format!(
            "release-{}-{}",
            std::process::id(),
            NEXT_STAGE.fetch_add(1, Ordering::Relaxed)
        );
        let stage = staging.new_private_child(&stage_name)?;
        let result = (|| {
            let payload = stage.private_child("payload")?;
            crate::archive::extract_zip(archive_path, &payload, self.limits)?;
            if let Some(bytes) = companion_bytes.as_ref() {
                if payload.metadata("Manifest.json")?.is_some() {
                    return Err(XxmiError::new(ErrorCode::InvalidPackage, None, "El ZIP incluye Manifest.json además del asset companion; no se fusiona por reemplazo."));
                }
                payload.write_new("Manifest.json", bytes)?;
            }
            let staged = inspect_package(&payload, self.limits)?;
            let expected_kind = match release.package_kind {
                OfficialPackageKind::Zzmi => PackageKind::GameIntegration(IntegrationKind::Zzmi),
                OfficialPackageKind::XxmiLibraries => PackageKind::XxmiLibraries,
            };
            if staged.kind != expected_kind {
                return Err(XxmiError::new(
                    ErrorCode::WrongGameIntegration,
                    None,
                    "El contenido extraído no coincide con el tipo de paquete de la release.",
                ));
            }
            let components_verified = if release.package_kind == OfficialPackageKind::XxmiLibraries
            {
                let manifest: serde_json::Value =
                    serde_json::from_slice(&payload.read_file("Manifest.json", 64 * 1024)?)
                        .map_err(|error| {
                            XxmiError::new(ErrorCode::InvalidMetadata, None, error.to_string())
                        })?;
                let signatures = manifest
                    .get("signatures")
                    .and_then(serde_json::Value::as_object)
                    .ok_or_else(|| {
                        XxmiError::new(
                            ErrorCode::InvalidMetadata,
                            None,
                            "Manifest.json no contiene firmas.",
                        )
                    })?;
                for filename in ["3dmloader.dll", "d3d11.dll", "d3dcompiler_47.dll"] {
                    let signature = signatures
                        .get(filename)
                        .and_then(serde_json::Value::as_str)
                        .ok_or_else(|| {
                            XxmiError::new(
                                ErrorCode::MissingSignature,
                                Some(filename.into()),
                                "Falta una firma de componente upstream.",
                            )
                        })?;
                    let path = payload.path().join(filename);
                    if crate::crypto::verify_component_in_directory(&payload, filename, signature)
                        != SignatureStatus::Verified
                    {
                        return Err(XxmiError::new(
                            ErrorCode::InvalidSignature,
                            Some(path),
                            format!("Firma upstream inválida para {filename}."),
                        ));
                    }
                }
                true
            } else {
                false
            };
            let files = staged.files;
            let id = fingerprint(&files)?;
            if packages.metadata(&id)?.is_some() {
                let existing = self.verify(&id)?;
                if existing.manifest.authenticity == PackageAuthenticity::OfficialReleaseVerified
                    && existing
                        .manifest
                        .upstream
                        .as_ref()
                        .is_some_and(|source| source.release_id == release.release_id)
                {
                    return Ok(existing);
                }
                return Err(XxmiError::new(ErrorCode::Busy, Some(self.root.join("packages/xxmi").join(&id)), "El contenido ya existe con otra procedencia; no se atribuye autenticidad cambiando solo el manifiesto."));
            }
            let package_manifest_asset = companion.map(|(metadata, _)| metadata);
            let provenance = UpstreamProvenance {
                repository: release.repository.clone(),
                release_id: release.release_id,
                tag: release.tag.clone(),
                commit: release.commit.clone(),
                release_url: release.release_url.clone(),
                asset_name: asset.name.clone(),
                asset_url: asset.download_url.clone(),
                published_at: release.published_at.clone(),
                metadata_retrieved_at: release.metadata_retrieved_at.clone(),
                downloaded_at: crate::release::format_utc(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|_| {
                            XxmiError::new(ErrorCode::Io, None, "Reloj anterior a Unix epoch.")
                        })?
                        .as_secs(),
                ),
                download_sha256: archive_sha256.clone(),
                expected_asset_sha256: asset.sha256.clone(),
                signature_base64: release.signature_base64.clone(),
                signature_status,
                companion_asset_name: package_manifest_asset.map(|value| value.name.clone()),
                companion_asset_sha256: package_manifest_asset
                    .and_then(|value| value.sha256.clone()),
                component_signatures_verified: components_verified,
            };
            let manifest = PackageManifest {
                schema_version: 2,
                id: id.clone(),
                ecosystem: "xxmi".into(),
                kind: staged.kind,
                version: staged.version,
                source: PackageSource::OfficialRelease {
                    repository: release.repository.clone(),
                    tag: release.tag.clone(),
                    asset: asset.name.clone(),
                },
                imported_unix_seconds: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| {
                        XxmiError::new(ErrorCode::Io, None, "Reloj anterior a Unix epoch.")
                    })?
                    .as_secs(),
                files,
                layout_reference: format!(
                    "{} {} release {} commit {}; archive authenticity verified",
                    release.repository, release.tag, release.release_id, release.commit
                ),
                authenticity: PackageAuthenticity::OfficialReleaseVerified,
                upstream: Some(provenance),
            };
            stage.write_new(
                "lxmi-package.json",
                &serde_json::to_vec_pretty(&manifest).map_err(|error| {
                    XxmiError::new(ErrorCode::InvalidMetadata, None, error.to_string())
                })?,
            )?;
            payload.sync()?;
            stage.sync()?;
            staging.promote(&stage_name, &packages, &id)?;
            tracing::info!(package_id=%id, tag=%release.tag, "Official package validated and promoted");
            self.verify(&id)
        })();
        if result.is_err() {
            if let Err(error) = staging.remove_stage(&stage_name) {
                if !missing(&error) {
                    tracing::warn!(code=?error.code, "Incomplete official package stage requires cleanup");
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
