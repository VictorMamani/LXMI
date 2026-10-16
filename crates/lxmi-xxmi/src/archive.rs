use crate::{
    filesystem::Directory, validate_relative_path, ErrorCode, ImportLimits, Result, XxmiError,
};
use std::{
    collections::HashSet,
    io::{Read, Write},
    path::Path,
};
use zip::ZipArchive;

/// Extracts a ZIP into an LXMI-owned descriptor-anchored directory.
/// No path from the archive is used outside that directory.
pub(crate) fn extract_zip(
    path: &Path,
    destination: &Directory,
    limits: ImportLimits,
) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        XxmiError::new(
            ErrorCode::UnsafePath,
            Some(path.into()),
            "ZIP sin directorio padre.",
        )
    })?;
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            XxmiError::new(
                ErrorCode::UnsafePath,
                Some(path.into()),
                "Nombre ZIP inválido.",
            )
        })?;
    let directory = Directory::open_absolute(parent)?;
    let file = directory.open_file(filename)?;
    let metadata = file
        .metadata()
        .map_err(|error| XxmiError::io(path, error))?;
    if metadata.len() > limits.max_archive_bytes {
        return Err(XxmiError::new(
            ErrorCode::LimitExceeded,
            Some(path.into()),
            "El ZIP supera el tamaño máximo permitido.",
        ));
    }
    let mut archive = ZipArchive::new(file).map_err(|error| {
        XxmiError::new(
            ErrorCode::UnsafeArchive,
            Some(path.into()),
            error.to_string(),
        )
    })?;
    if archive.len() > limits.max_archive_entries {
        return Err(XxmiError::new(
            ErrorCode::LimitExceeded,
            Some(path.into()),
            "El ZIP contiene demasiadas entradas.",
        ));
    }

    let mut seen = HashSet::new();
    let mut total = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| {
            XxmiError::new(
                ErrorCode::UnsafeArchive,
                Some(path.into()),
                error.to_string(),
            )
        })?;
        let raw_name = entry.name().to_owned();
        if raw_name.starts_with('/')
            || raw_name.starts_with("\\\\")
            || raw_name.contains('\\')
            || raw_name.contains(':')
            || raw_name.as_bytes().contains(&0)
        {
            return Err(unsafe_entry(&raw_name));
        }
        let is_dir = entry.is_dir();
        let relative = if is_dir {
            raw_name.strip_suffix('/').unwrap_or(&raw_name)
        } else {
            &raw_name
        };
        validate_relative_path(relative).map_err(|_| unsafe_entry(&raw_name))?;
        if relative.split('/').count() > limits.max_depth {
            return Err(XxmiError::new(
                ErrorCode::LimitExceeded,
                Some(relative.into()),
                "El ZIP excede la profundidad máxima.",
            ));
        }
        if !seen.insert(relative.to_lowercase()) {
            return Err(XxmiError::new(
                ErrorCode::UnsafeArchive,
                Some(relative.into()),
                "El ZIP contiene rutas duplicadas o ambiguas por mayúsculas.",
            ));
        }

        if let Some(mode) = entry.unix_mode() {
            let file_type = mode & 0o170000;
            let expected = if is_dir { 0o040000 } else { 0o100000 };
            if file_type != 0 && file_type != expected {
                return Err(XxmiError::new(
                    ErrorCode::UnsafeArchive,
                    Some(relative.into()),
                    "El ZIP contiene un symlink, hardlink o archivo especial.",
                ));
            }
        }
        if is_dir {
            destination.ensure_directory(relative)?;
            continue;
        }

        let declared_size = entry.size();
        let compressed = entry.compressed_size();
        if declared_size > limits.max_file_bytes {
            return Err(XxmiError::new(
                ErrorCode::LimitExceeded,
                Some(relative.into()),
                "Un archivo del ZIP supera el límite permitido.",
            ));
        }
        let ratio_limit = compressed.saturating_mul(limits.max_compression_ratio);
        if declared_size > 0 && (compressed == 0 || declared_size > ratio_limit) {
            return Err(XxmiError::new(
                ErrorCode::LimitExceeded,
                Some(relative.into()),
                "La relación de compresión excede el límite permitido.",
            ));
        }
        total = total.checked_add(declared_size).ok_or_else(|| {
            XxmiError::new(ErrorCode::LimitExceeded, None, "Overflow de tamaño ZIP.")
        })?;
        if total > limits.max_total_bytes {
            return Err(XxmiError::new(
                ErrorCode::LimitExceeded,
                Some(path.into()),
                "El tamaño descomprimido total del ZIP excede el límite permitido.",
            ));
        }

        destination.ensure_parents(relative)?;
        let mut output = destination.create_new_file(relative)?;
        let mut written = 0_u64;
        let mut buffer = [0_u8; 32 * 1024];
        loop {
            let count = entry.read(&mut buffer).map_err(|error| {
                XxmiError::new(
                    ErrorCode::UnsafeArchive,
                    Some(relative.into()),
                    format!("No se pudo descomprimir o validar CRC: {error}"),
                )
            })?;
            if count == 0 {
                break;
            }
            written += count as u64;
            if written > declared_size || written > limits.max_file_bytes {
                return Err(XxmiError::new(
                    ErrorCode::LimitExceeded,
                    Some(relative.into()),
                    "El tamaño descomprimido excede lo declarado o el límite permitido.",
                ));
            }
            output
                .write_all(&buffer[..count])
                .map_err(|error| XxmiError::io(Path::new(relative), error))?;
        }
        if written != declared_size {
            return Err(XxmiError::new(
                ErrorCode::UnsafeArchive,
                Some(relative.into()),
                "El tamaño real no coincide con el directorio central del ZIP.",
            ));
        }
        output
            .sync_all()
            .map_err(|error| XxmiError::io(Path::new(relative), error))?;
    }
    destination.sync()
}

fn unsafe_entry(name: &str) -> XxmiError {
    XxmiError::new(
        ErrorCode::UnsafePath,
        Some(name.into()),
        "El ZIP contiene una ruta absoluta, traversal o separador ambiguo.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        fs::File,
        io::Write,
        time::{SystemTime, UNIX_EPOCH},
    };
    use zip::{write::SimpleFileOptions, ZipWriter};

    fn temp() -> std::path::PathBuf {
        let tick = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("lxmi-zip-{}-{tick}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn make_zip(path: &Path, entries: &[(&str, &[u8], u32)]) {
        let mark_symlink = entries
            .iter()
            .any(|(name, _, mode)| *name == "link" && mode & 0o170000 == 0o120000);
        let file = File::create(path).unwrap();
        let mut writer = ZipWriter::new(file);
        for (name, contents, mode) in entries {
            writer
                .start_file(*name, SimpleFileOptions::default().unix_permissions(*mode))
                .unwrap();
            writer.write_all(contents).unwrap();
        }
        writer.finish().unwrap();
        if mark_symlink {
            let mut bytes = fs::read(path).unwrap();
            let central_header = bytes
                .windows(4)
                .position(|value| value == [0x50, 0x4b, 0x01, 0x02])
                .unwrap();
            bytes[central_header + 5] = 3; // Unix host identifier in "version made by".
            let external_attributes = (0o120777_u32) << 16;
            bytes[central_header + 38..central_header + 42]
                .copy_from_slice(&external_attributes.to_le_bytes());
            fs::write(path, bytes).unwrap();
        }
    }

    #[test]
    fn extracts_regular_package_files_under_the_selected_stage() {
        let root = temp();
        let zip_path = root.join("valid.zip");
        make_zip(
            &zip_path,
            &[
                ("d3dx.ini", b"safe", 0o100600),
                ("Core/ZZMI/main.ini", b"config", 0o100600),
            ],
        );
        let output_path = root.join("payload");
        fs::create_dir(&output_path).unwrap();
        let output = Directory::open_absolute(&output_path).unwrap();
        extract_zip(&zip_path, &output, ImportLimits::default()).unwrap();
        assert_eq!(fs::read(output_path.join("d3dx.ini")).unwrap(), b"safe");
        assert_eq!(
            fs::read(output_path.join("Core/ZZMI/main.ini")).unwrap(),
            b"config"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_traversal_absolute_paths_symlinks_and_case_collisions() {
        for (name, entries) in [
            (
                "traversal",
                vec![("../outside.txt", b"escape".as_slice(), 0o100600)],
            ),
            (
                "absolute",
                vec![("/outside.txt", b"escape".as_slice(), 0o100600)],
            ),
            ("symlink", vec![("link", b"target".as_slice(), 0o120777)]),
            (
                "case",
                vec![
                    ("Core/File.ini", b"a".as_slice(), 0o100600),
                    ("core/file.ini", b"b".as_slice(), 0o100600),
                ],
            ),
        ] {
            let root = temp();
            let zip_path = root.join(format!("{name}.zip"));
            make_zip(&zip_path, &entries);
            let output_path = root.join("payload");
            fs::create_dir(&output_path).unwrap();
            let output = Directory::open_absolute(&output_path).unwrap();
            assert!(
                extract_zip(&zip_path, &output, ImportLimits::default()).is_err(),
                "{name}"
            );
            assert!(!root.join("outside.txt").exists());
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn rejects_zip_bomb_ratio_and_too_many_entries() {
        let root = temp();
        let zip_path = root.join("ratio.zip");
        make_zip(
            &zip_path,
            &[("repeated.bin", &vec![0; 16 * 1024], 0o100600)],
        );
        let output_path = root.join("payload");
        fs::create_dir(&output_path).unwrap();
        let output = Directory::open_absolute(&output_path).unwrap();
        let limits = ImportLimits {
            max_compression_ratio: 2,
            ..ImportLimits::default()
        };
        assert!(extract_zip(&zip_path, &output, limits).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
