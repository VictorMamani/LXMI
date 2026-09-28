#[cfg(test)]
use crate::filesystem::{missing, Directory};
use crate::*;
use std::path::Path;

const LIBRARY_DLLS_DEPLOYED_BY_UPSTREAM: &[&str] = &["d3d11.dll", "d3dcompiler_47.dll"];

pub(crate) fn map_zzmi_installation(
    integration: &VerifiedPackage,
    libraries: Option<&VerifiedPackage>,
    _game_executable: Option<&Path>,
) -> Result<(Vec<DeploymentMapping>, DryRunInstallation)> {
    if integration.manifest.kind != PackageKind::GameIntegration(IntegrationKind::Zzmi) {
        return Err(XxmiError::new(
            ErrorCode::WrongGameIntegration,
            None,
            "El mapeo objetivo de 0.5.2 requiere un paquete ZZMI.",
        ));
    }
    if libraries.is_some_and(|package| package.manifest.kind != PackageKind::XxmiLibraries) {
        return Err(XxmiError::new(
            ErrorCode::InvalidPackage,
            None,
            "Se esperaba el paquete separado XXMI Libraries.",
        ));
    }

    let root_basis = "XXMI Launcher ModelImporterConfig.importer_path; es configurable y separado de game_folder";
    let mut mappings = Vec::new();
    for file in &integration.manifest.files {
        validate_relative_path(&file.relative_path)?;
        mappings.push(DeploymentMapping {
            source_package_id: integration.manifest.id.clone(),
            source_relative_path: file.relative_path.clone(),
            target_relative_path: file.relative_path.clone(),
            operation: DeploymentOperation::MergePackageIntoConfiguredImporterDirectory,
            target_root_basis: root_basis.into(),
            evidence: "XXMI Launcher move_contents(downloaded_asset_path, importer_path) copies package contents into configured importer_path.".into(),
        });
    }
    if let Some(libraries) = libraries {
        for file in &libraries.manifest.files {
            if !LIBRARY_DLLS_DEPLOYED_BY_UPSTREAM.contains(&file.relative_path.as_str()) {
                continue;
            }
            validate_relative_path(&file.relative_path)?;
            mappings.push(DeploymentMapping {
                source_package_id: libraries.manifest.id.clone(),
                source_relative_path: file.relative_path.clone(),
                target_relative_path: file.relative_path.clone(),
                operation: DeploymentOperation::DeployRuntimeDllToConfiguredImporterDirectory,
                target_root_basis: root_basis.into(),
                evidence: "XXMI Launcher MigotoPackage.deploy_package_files copies d3d11.dll and d3dcompiler_47.dll to importer_path; 3dmloader.dll stays in the managed package for loader use.".into(),
            });
        }
    }
    mappings.sort_by(|a, b| a.target_relative_path.cmp(&b.target_relative_path));

    // The executable-directory comparison performed by 0.5.2 is historical only.
    // The active importer root is now assembled under LXMI-managed XDG storage.
    let comparison_root_candidate: Option<std::path::PathBuf> = None;
    let comparison_root_evidence = "Comparación de 0.5.2 contra la carpeta del ejecutable: antecedente histórico solamente, no target aprobado ni usado. El importer root activo de LXMI 0.5.3 queda bajo XDG/lxmi/runtimes/zenless-zone-zero/zzmi/<runtime-id>/ZZMI.".to_owned();

    let mut dry_run_files = Vec::with_capacity(mappings.len());
    for mapping in &mappings {
        let source_package = if mapping.source_package_id == integration.manifest.id {
            integration
        } else {
            libraries.ok_or_else(|| {
                XxmiError::new(
                    ErrorCode::InvalidPackage,
                    None,
                    "No está disponible el paquete origen del mapeo.",
                )
            })?
        };
        let source = source_package
            .payload_path
            .join(&mapping.source_relative_path);
        let expected_sha256 = source_package
            .manifest
            .files
            .iter()
            .find(|file| file.relative_path == mapping.source_relative_path)
            .map(|file| file.sha256.clone())
            .ok_or_else(|| {
                XxmiError::new(
                    ErrorCode::InvalidPackage,
                    Some(source.clone()),
                    "El archivo de mapeo no está en el inventario validado.",
                )
            })?;
        dry_run_files.push(DryRunFile {
            source_package_id: mapping.source_package_id.clone(),
            source_path: source,
            target_relative_path: mapping.target_relative_path.clone(),
            comparison_path: None,
            expected_sha256,
            existing_sha256: None,
            status: DryRunStatus::Unresolved,
            evidence: "Mapping retained for 0.5.2 traceability; no game directory is inspected or proposed as a target. The managed runtime assembly supersedes this comparison.".into(),
        });
    }
    Ok((
        mappings,
        DryRunInstallation {
            configured_target_root: None,
            comparison_root_candidate,
            comparison_root_evidence,
            root_is_authoritative: false,
            files: dry_run_files,
            safety: InstallSafetyState::PlatformCompatibilityUnverified,
            apply_allowed: false,
            writes_performed: false,
        },
    ))
}

#[cfg(test)]
fn inspect_candidate(
    directory: &Directory,
    relative: &str,
    expected: &str,
) -> (DryRunStatus, Option<String>, String) {
    match directory.metadata(relative) {
        Ok(None) => (
            DryRunStatus::WouldCreate,
            None,
            "El archivo no existe en la carpeta candidata; todavía no es un target aprobado."
                .into(),
        ),
        Err(error) if missing(&error) => (
            DryRunStatus::WouldCreate,
            None,
            "Falta el archivo o un subdirectorio en la carpeta candidata.".into(),
        ),
        Err(error) if error.code == ErrorCode::PermissionDenied => {
            (DryRunStatus::PermissionIssue, None, error.detail)
        }
        Err(error) if error.code == ErrorCode::UnsafePath => {
            (DryRunStatus::UnsafeTarget, None, error.detail)
        }
        Err(error) => (DryRunStatus::Conflict, None, error.detail),
        Ok(Some(metadata)) if metadata.is_dir() => (
            DryRunStatus::Conflict,
            None,
            "La ruta candidata existe como directorio.".into(),
        ),
        Ok(Some(_)) => match crate::package::hash_file(
            directory,
            relative,
            ImportLimits::default().max_file_bytes,
        ) {
            Ok((_, current)) if current == expected => (
                DryRunStatus::AlreadyMatches,
                Some(current),
                "El hash del archivo candidato coincide; no se modificó.".into(),
            ),
            Ok((_, current)) => (
                DryRunStatus::WouldReplace,
                Some(current),
                "El hash del archivo candidato difiere; un apply requeriría backup y revisión."
                    .into(),
            ),
            Err(error) if error.code == ErrorCode::PermissionDenied => {
                (DryRunStatus::PermissionIssue, None, error.detail)
            }
            Err(error) if error.code == ErrorCode::UnsafePath => {
                (DryRunStatus::UnsafeTarget, None, error.detail)
            }
            Err(error) => (DryRunStatus::Conflict, None, error.detail),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::Digest;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp() -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("lxmi-dryrun-{}-{id}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn verified_package(
        kind: PackageKind,
        id: &str,
        files: Vec<PackageFile>,
        payload: PathBuf,
    ) -> VerifiedPackage {
        VerifiedPackage {
            manifest: PackageManifest {
                schema_version: 1,
                id: id.into(),
                ecosystem: "xxmi".into(),
                kind,
                version: None,
                source: PackageSource::Unknown,
                imported_unix_seconds: 0,
                files,
                layout_reference: "synthetic mapping test".into(),
                authenticity: PackageAuthenticity::NotAuthenticated,
                upstream: None,
            },
            payload_path: payload,
        }
    }

    #[test]
    fn candidate_dry_run_maps_upstream_paths_without_writing() {
        let root = temp();
        let payload = root.join("payload");
        fs::create_dir(&payload).unwrap();
        fs::write(payload.join("d3dx.ini"), b"config").unwrap();
        let executable = root.join("game").join("ZenlessZoneZero.exe");
        fs::create_dir_all(executable.parent().unwrap()).unwrap();
        fs::write(&executable, b"synthetic exe").unwrap();
        let files = vec![PackageFile {
            relative_path: "d3dx.ini".into(),
            size: 6,
            sha256: format!("{:x}", sha2::Sha256::digest(b"config")),
        }];
        let package = verified_package(
            PackageKind::GameIntegration(IntegrationKind::Zzmi),
            &"a".repeat(64),
            files,
            payload,
        );
        let before = fs::read(executable.parent().unwrap().join("ZenlessZoneZero.exe")).unwrap();
        let (mapping, dry_run) = map_zzmi_installation(&package, None, Some(&executable)).unwrap();
        assert_eq!(mapping[0].target_relative_path, "d3dx.ini");
        assert_eq!(
            mapping[0].operation,
            DeploymentOperation::MergePackageIntoConfiguredImporterDirectory
        );
        assert_eq!(dry_run.files[0].status, DryRunStatus::Unresolved);
        assert!(dry_run.comparison_root_candidate.is_none());
        assert!(dry_run.files[0].comparison_path.is_none());
        assert!(!dry_run.root_is_authoritative);
        assert!(!dry_run.apply_allowed);
        assert!(!dry_run.writes_performed);
        assert_eq!(before, fs::read(&executable).unwrap());
        assert!(!executable.parent().unwrap().join("d3dx.ini").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn dry_run_distinguishes_identical_and_different_existing_targets() {
        let root = temp();
        let directory = Directory::open_absolute(&root).unwrap();
        fs::write(root.join("same.ini"), b"same").unwrap();
        fs::write(root.join("changed.ini"), b"old").unwrap();
        let same_hash = format!("{:x}", sha2::Sha256::digest(b"same"));
        let new_hash = format!("{:x}", sha2::Sha256::digest(b"new"));
        assert_eq!(
            inspect_candidate(&directory, "same.ini", &same_hash).0,
            DryRunStatus::AlreadyMatches
        );
        assert_eq!(
            inspect_candidate(&directory, "changed.ini", &new_hash).0,
            DryRunStatus::WouldReplace
        );
        fs::remove_dir_all(root).unwrap();
    }
}
