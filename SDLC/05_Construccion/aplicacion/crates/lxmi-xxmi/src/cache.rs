use crate::{filesystem::Directory, validate_relative_path, ErrorCode, Result, XxmiError};
use sha2::{Digest, Sha256};
use std::{
    env,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_PARTIAL: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedAsset {
    pub path: PathBuf,
    pub size: u64,
    pub sha256: String,
    pub reused: bool,
}

#[derive(Debug, Clone)]
pub struct DownloadCache {
    root: PathBuf,
}

impl DownloadCache {
    pub fn from_environment() -> Result<Self> {
        let base = env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| {
                env::var_os("HOME")
                    .map(PathBuf::from)
                    .filter(|path| path.is_absolute())
                    .map(|home| home.join(".cache"))
            })
            .ok_or_else(|| {
                XxmiError::new(
                    ErrorCode::StorageUnavailable,
                    None,
                    "No existe una ruta XDG cache absoluta.",
                )
            })?;
        Self::at(base.join("lxmi/downloads"))
    }

    pub fn at(root: PathBuf) -> Result<Self> {
        if !root.is_absolute()
            || root.file_name().and_then(|name| name.to_str()) != Some("downloads")
        {
            return Err(XxmiError::new(
                ErrorCode::UnsafePath,
                Some(root),
                "La cache debe ser una ruta absoluta terminada en lxmi/downloads.",
            ));
        }
        for component in root.components() {
            if !matches!(
                component,
                std::path::Component::RootDir | std::path::Component::Normal(_)
            ) {
                return Err(XxmiError::new(
                    ErrorCode::UnsafePath,
                    Some(root),
                    "Ruta cache no normalizada.",
                ));
            }
        }
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn initialize(&self) -> Result<Directory> {
        let base = self.root.parent().and_then(Path::parent).ok_or_else(|| {
            XxmiError::new(
                ErrorCode::StorageUnavailable,
                None,
                "Cache sin directorio padre.",
            )
        })?;
        let base = Directory::open_absolute(base)?;
        base.private_child("lxmi")?.private_child("downloads")
    }

    pub fn store_reader<R: Read>(
        &self,
        release_id: u64,
        asset_id: u64,
        extension: &str,
        expected_sha256: Option<&str>,
        max_bytes: u64,
        reader: &mut R,
    ) -> Result<CachedAsset> {
        if !matches!(extension, "zip" | "json") {
            return Err(XxmiError::new(
                ErrorCode::UnsafePath,
                None,
                "Extensión de asset no permitida.",
            ));
        }
        let root = self.initialize()?;
        let release = root.private_child(&release_id.to_string())?;
        let name = format!("asset-{asset_id}.{extension}");
        let final_path = self.root.join(release_id.to_string()).join(&name);
        if let Some(meta) = release.metadata(&name)? {
            if !meta.is_file() {
                return Err(XxmiError::new(
                    ErrorCode::UnsafePath,
                    Some(final_path),
                    "Asset cache no es archivo regular.",
                ));
            }
            let (size, digest) = crate::package::hash_file(&release, &name, max_bytes)?;
            if expected_sha256.is_some_and(|expected| expected != digest) {
                return Err(XxmiError::new(
                    ErrorCode::ChecksumMismatch,
                    Some(final_path),
                    "El asset cache no coincide con el digest oficial.",
                ));
            }
            return Ok(CachedAsset {
                path: final_path,
                size,
                sha256: digest,
                reused: true,
            });
        }

        let partial = format!(
            ".partial-{}-{}",
            std::process::id(),
            NEXT_PARTIAL.fetch_add(1, Ordering::Relaxed)
        );
        let mut output = release.create_new_file(&partial)?;
        let mut digest = Sha256::new();
        let mut size = 0_u64;
        let mut buffer = [0_u8; 64 * 1024];
        let result = (|| {
            loop {
                let count = reader.read(&mut buffer).map_err(|error| {
                    XxmiError::new(
                        ErrorCode::Network,
                        None,
                        format!("Falló lectura de descarga: {error}"),
                    )
                })?;
                if count == 0 {
                    break;
                }
                size = size.checked_add(count as u64).ok_or_else(|| {
                    XxmiError::new(ErrorCode::LimitExceeded, None, "Overflow de descarga.")
                })?;
                if size > max_bytes {
                    return Err(XxmiError::new(
                        ErrorCode::LimitExceeded,
                        None,
                        "Asset supera el límite de descarga.",
                    ));
                }
                digest.update(&buffer[..count]);
                output
                    .write_all(&buffer[..count])
                    .map_err(|error| XxmiError::io(&final_path, error))?;
            }
            output
                .sync_all()
                .map_err(|error| XxmiError::io(&final_path, error))?;
            let sha256 = format!("{:x}", digest.finalize());
            if expected_sha256.is_some_and(|expected| expected != sha256) {
                return Err(XxmiError::new(
                    ErrorCode::ChecksumMismatch,
                    Some(final_path.clone()),
                    "El SHA-256 descargado no coincide con el digest oficial de GitHub.",
                ));
            }
            release.promote_file(&partial, &name)?;
            Ok(CachedAsset {
                path: final_path.clone(),
                size,
                sha256,
                reused: false,
            })
        })();
        if result.is_err() {
            if let Err(error) = release.remove_file(&partial) {
                if !matches!(error.code, ErrorCode::NotFound) {
                    tracing::warn!(code=?error.code, "Could not remove partial release download");
                }
            }
        }
        result
    }

    pub fn read_asset(&self, path: &Path, limit: u64) -> Result<Vec<u8>> {
        if !path.starts_with(&self.root) {
            return Err(XxmiError::new(
                ErrorCode::UnsafePath,
                Some(path.into()),
                "Asset fuera de la cache LXMI.",
            ));
        }
        let relative = path.strip_prefix(&self.root).map_err(|_| {
            XxmiError::new(
                ErrorCode::UnsafePath,
                Some(path.into()),
                "Asset fuera de la cache LXMI.",
            )
        })?;
        let relative = relative.to_str().ok_or_else(|| {
            XxmiError::new(
                ErrorCode::UnsafePath,
                Some(path.into()),
                "Ruta cache no UTF-8.",
            )
        })?;
        validate_relative_path(relative)?;
        let root = Directory::open_absolute(&self.root)?;
        let parent = relative.rsplit_once('/').map(|(parent, _)| parent);
        let name = relative.rsplit('/').next().unwrap_or(relative);
        let dir = match parent {
            Some(parent) => root.descend(parent)?,
            None => root,
        };
        dir.read_file(name, limit)
    }
}
