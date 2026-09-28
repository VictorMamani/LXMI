//! Linux descriptor-anchored filesystem operations. Every untrusted component is
//! opened O_NOFOLLOW; /proc/self/fd refers to our open descriptor, not user input.
use crate::{ErrorCode, Result, XxmiError};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::AsRawFd,
        unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    },
    path::{Component, Path, PathBuf},
};

pub fn validate_relative_path(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 4096
        || value.contains(['\\', ':', '\0'])
        || value
            .split('/')
            .any(|c| c.is_empty() || c == "." || c == ".." || c.chars().any(char::is_control))
        || Path::new(value)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(XxmiError::new(
            ErrorCode::UnsafePath,
            Some(PathBuf::from(value)),
            "Se requiere una ruta relativa normal, sin traversal ni separadores ambiguos.",
        ));
    }
    Ok(())
}

pub(crate) struct Directory {
    file: File,
}
impl Directory {
    pub fn open_absolute(path: &Path) -> Result<Self> {
        if !path.is_absolute() {
            return Err(XxmiError::new(
                ErrorCode::UnsafePath,
                Some(path.to_owned()),
                "La ruta debe ser absoluta.",
            ));
        }
        let mut dir = Self {
            file: File::open("/").map_err(|e| XxmiError::io(path, e))?,
        };
        for c in path.components() {
            match c {
                Component::RootDir => {}
                Component::Normal(name) => {
                    let s = name.to_str().ok_or_else(|| {
                        XxmiError::new(
                            ErrorCode::UnsafePath,
                            Some(path.to_owned()),
                            "Ruta no UTF-8.",
                        )
                    })?;
                    dir = dir.child(s)?;
                }
                _ => {
                    return Err(XxmiError::new(
                        ErrorCode::UnsafePath,
                        Some(path.to_owned()),
                        "Ruta absoluta no normalizada.",
                    ))
                }
            }
        }
        Ok(dir)
    }
    pub fn path(&self) -> PathBuf {
        PathBuf::from(format!("/proc/self/fd/{}", self.file.as_raw_fd()))
    }
    pub fn child(&self, name: &str) -> Result<Self> {
        validate_relative_path(name)?;
        if name.contains('/') {
            return self.descend(name);
        }
        let path = self.path().join(name);
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY | libc::O_CLOEXEC)
            .open(&path)
            .map_err(|e| XxmiError::io(&path, e))?;
        Ok(Self { file })
    }
    pub fn descend(&self, relative: &str) -> Result<Self> {
        validate_relative_path(relative)?;
        let mut dir = Self {
            file: self
                .file
                .try_clone()
                .map_err(|e| XxmiError::io(&self.path(), e))?,
        };
        for c in relative.split('/') {
            dir = dir.child(c)?;
        }
        Ok(dir)
    }
    pub fn new_private_child(&self, name: &str) -> Result<Self> {
        validate_relative_path(name)?;
        if name.contains('/') {
            return Err(XxmiError::new(
                ErrorCode::UnsafePath,
                None,
                "Un componente requerido.",
            ));
        }
        let path = self.path().join(name);
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .map_err(|e| XxmiError::io(&path, e))?;
        let child = self.child(name)?;
        child.assert_private()?;
        Ok(child)
    }
    pub fn private_child(&self, name: &str) -> Result<Self> {
        validate_relative_path(name)?;
        if name.contains('/') {
            return Err(XxmiError::new(
                ErrorCode::UnsafePath,
                None,
                "Un solo componente requerido.",
            ));
        }
        let path = self.path().join(name);
        match fs::DirBuilder::new().mode(0o700).create(&path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(XxmiError::io(&path, e)),
        }
        let child = self.child(name)?;
        child.assert_private()?;
        Ok(child)
    }
    pub fn assert_private(&self) -> Result<()> {
        let m = self
            .file
            .metadata()
            .map_err(|e| XxmiError::io(&self.path(), e))?;
        // SAFETY: geteuid has no pointer arguments or preconditions.
        let uid = unsafe { libc::geteuid() };
        if m.uid() != uid || m.mode() & 0o077 != 0 {
            return Err(XxmiError::new(
                ErrorCode::StorageUnavailable,
                None,
                "El almacenamiento LXMI debe pertenecer al usuario y tener permisos 0700.",
            ));
        }
        Ok(())
    }
    pub fn entries(&self, limit: usize) -> Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in fs::read_dir(self.path()).map_err(|e| XxmiError::io(&self.path(), e))? {
            let entry = entry.map_err(|e| XxmiError::io(&self.path(), e))?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| XxmiError::new(ErrorCode::UnsafePath, None, "Nombre no UTF-8."))?;
            validate_relative_path(&name)?;
            names.push(name);
            if names.len() > limit {
                return Err(XxmiError::new(
                    ErrorCode::LimitExceeded,
                    None,
                    "Demasiadas entradas.",
                ));
            }
        }
        names.sort();
        let mut seen = std::collections::HashSet::new();
        if names.iter().any(|n| !seen.insert(n.to_lowercase())) {
            return Err(XxmiError::new(
                ErrorCode::UnsafePath,
                None,
                "Nombres ambiguos por mayúsculas/minúsculas.",
            ));
        }
        Ok(names)
    }
    pub fn metadata(&self, name: &str) -> Result<Option<fs::Metadata>> {
        validate_relative_path(name)?;
        let (parent, base) = self.parent_for(name)?;
        match fs::symlink_metadata(parent.path().join(base)) {
            Ok(m) if m.file_type().is_symlink() => Err(XxmiError::new(
                ErrorCode::UnsafePath,
                Some(PathBuf::from(name)),
                "Enlace simbólico rechazado.",
            )),
            Ok(m) => Ok(Some(m)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(XxmiError::io(Path::new(name), e)),
        }
    }
    fn parent_for(&self, name: &str) -> Result<(Self, String)> {
        validate_relative_path(name)?;
        match name.rsplit_once('/') {
            Some((p, b)) => Ok((self.descend(p)?, b.to_owned())),
            None => Ok((
                Self {
                    file: self
                        .file
                        .try_clone()
                        .map_err(|e| XxmiError::io(&self.path(), e))?,
                },
                name.to_owned(),
            )),
        }
    }
    pub fn read_file(&self, name: &str, limit: u64) -> Result<Vec<u8>> {
        let mut file = self.open_file(name)?;
        let mut bytes = Vec::new();
        (&mut file)
            .take(limit + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| XxmiError::io(Path::new(name), e))?;
        if bytes.len() as u64 > limit {
            return Err(XxmiError::new(
                ErrorCode::LimitExceeded,
                Some(PathBuf::from(name)),
                "Archivo supera el límite de lectura.",
            ));
        }
        Ok(bytes)
    }
    pub fn open_file(&self, name: &str) -> Result<File> {
        let (parent, base) = self.parent_for(name)?;
        let p = parent.path().join(base);
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open(&p)
            .map_err(|e| XxmiError::io(Path::new(name), e))?;
        let m = file
            .metadata()
            .map_err(|e| XxmiError::io(Path::new(name), e))?;
        if !m.is_file() || m.nlink() != 1 {
            return Err(XxmiError::new(
                ErrorCode::UnsafePath,
                Some(PathBuf::from(name)),
                "Solo archivos regulares sin hardlinks.",
            ));
        }
        Ok(file)
    }
    pub fn write_new(&self, name: &str, bytes: &[u8]) -> Result<()> {
        let (parent, base) = self.parent_for(name)?;
        let path = parent.path().join(base);
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&path)
            .map_err(|e| XxmiError::io(Path::new(name), e))?;
        f.write_all(bytes)
            .and_then(|()| f.sync_all())
            .map_err(|e| XxmiError::io(Path::new(name), e))
    }
    pub fn create_new_file(&self, name: &str) -> Result<File> {
        let (parent, base) = self.parent_for(name)?;
        let path = parent.path().join(base);
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&path)
            .map_err(|e| XxmiError::io(Path::new(name), e))
    }
    pub fn ensure_parents(&self, relative: &str) -> Result<()> {
        validate_relative_path(relative)?;
        if let Some((parent, _)) = relative.rsplit_once('/') {
            let mut dir = Self {
                file: self
                    .file
                    .try_clone()
                    .map_err(|e| XxmiError::io(&self.path(), e))?,
            };
            for c in parent.split('/') {
                dir = dir.private_child(c)?;
            }
        }
        Ok(())
    }
    pub fn ensure_directory(&self, relative: &str) -> Result<()> {
        validate_relative_path(relative)?;
        let mut dir = Self {
            file: self
                .file
                .try_clone()
                .map_err(|e| XxmiError::io(&self.path(), e))?,
        };
        for component in relative.split('/') {
            dir = dir.private_child(component)?;
        }
        Ok(())
    }
    pub fn sync(&self) -> Result<()> {
        self.file
            .sync_all()
            .map_err(|e| XxmiError::io(&self.path(), e))
    }
    pub fn promote(&self, from: &str, target: &Self, to: &str) -> Result<()> {
        validate_relative_path(from)?;
        validate_relative_path(to)?;
        let a = std::ffi::CString::new(from)
            .map_err(|_| XxmiError::new(ErrorCode::UnsafePath, None, "Nombre inválido"))?;
        let b = std::ffi::CString::new(to)
            .map_err(|_| XxmiError::new(ErrorCode::UnsafePath, None, "Nombre inválido"))?;
        // SAFETY: both FDs and NUL-terminated strings remain live during this syscall.
        let result = unsafe {
            libc::renameat2(
                self.file.as_raw_fd(),
                a.as_ptr(),
                target.file.as_raw_fd(),
                b.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        };
        if result != 0 {
            return Err(XxmiError::io(
                Path::new(to),
                std::io::Error::last_os_error(),
            ));
        }
        target.sync()?;
        self.sync()
    }
    pub fn promote_file(&self, from: &str, to: &str) -> Result<()> {
        validate_relative_path(from)?;
        validate_relative_path(to)?;
        let a = std::ffi::CString::new(from)
            .map_err(|_| XxmiError::new(ErrorCode::UnsafePath, None, "Nombre inválido"))?;
        let b = std::ffi::CString::new(to)
            .map_err(|_| XxmiError::new(ErrorCode::UnsafePath, None, "Nombre inválido"))?;
        // SAFETY: the directory fd and NUL-terminated names live during the syscall.
        let result = unsafe {
            libc::renameat2(
                self.file.as_raw_fd(),
                a.as_ptr(),
                self.file.as_raw_fd(),
                b.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        };
        if result != 0 {
            return Err(XxmiError::io(
                &self.path().join(to),
                std::io::Error::last_os_error(),
            ));
        }
        self.sync()
    }
    pub fn remove_file(&self, name: &str) -> Result<()> {
        let (parent, base) = self.parent_for(name)?;
        let c_name = std::ffi::CString::new(base)
            .map_err(|_| XxmiError::new(ErrorCode::UnsafePath, None, "Nombre inválido"))?;
        // SAFETY: the directory fd and file name are valid for unlinkat.
        let result = unsafe { libc::unlinkat(parent.file.as_raw_fd(), c_name.as_ptr(), 0) };
        if result != 0 {
            return Err(XxmiError::io(
                &parent.path().join(c_name.to_string_lossy().as_ref()),
                std::io::Error::last_os_error(),
            ));
        }
        Ok(())
    }
    /// Only for our private temporary directory, never an imported source.
    pub fn remove_stage(&self, name: &str) -> Result<()> {
        validate_relative_path(name)?;
        let dir = self.child(name)?;
        dir.assert_private()?;
        for child in dir.entries(8192)? {
            let path = dir.path().join(&child);
            let m = fs::symlink_metadata(&path).map_err(|e| XxmiError::io(&path, e))?;
            if m.is_dir() {
                dir.remove_stage(&child)?;
            } else {
                fs::remove_file(&path).map_err(|e| XxmiError::io(&path, e))?;
            }
        }
        fs::remove_dir(self.path().join(name)).map_err(|e| XxmiError::io(Path::new(name), e))
    }
}

pub(crate) fn missing(error: &XxmiError) -> bool {
    error.code == ErrorCode::NotFound
}
