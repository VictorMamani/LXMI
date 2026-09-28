use crate::{
    filesystem::{missing, Directory},
    package::hash_file,
    *,
};
use lxmi_core::{GameDistribution, GameInstallationStatus};
use lxmi_runtime::{
    GameInstallationAssessment, GameRuntimePlan, PrefixAssessment, RuntimeCandidateDiscovery,
    RuntimeSelection,
};

pub fn integration_for_game(game_id: &str) -> Option<IntegrationKind> {
    match game_id {
        "wuthering-waves" => Some(IntegrationKind::Wwmi),
        "zenless-zone-zero" => Some(IntegrationKind::Zzmi),
        _ => None,
    }
}

pub fn assess(
    platform: &GameRuntimePlan,
    wwmi: Option<&VerifiedPackage>,
    libraries: Option<&VerifiedPackage>,
) -> IntegrationAssessment {
    assess_for_integration(platform, IntegrationKind::Wwmi, wwmi, libraries)
}

pub fn assess_for_integration(
    platform: &GameRuntimePlan,
    integration: IntegrationKind,
    package: Option<&VerifiedPackage>,
    libraries: Option<&VerifiedPackage>,
) -> IntegrationAssessment {
    use IntegrationRequirementState::{NotRequired, Satisfied, Unknown, Unsatisfied};

    let supported = integration_for_game(platform.game.id) == Some(integration);
    let installed = match platform.installation {
        GameInstallationAssessment::Detected {
            directory_status: GameInstallationStatus::Installed,
            ..
        } => Satisfied,
        GameInstallationAssessment::Unknown { .. }
        | GameInstallationAssessment::Detected {
            directory_status: GameInstallationStatus::PermissionDenied,
            ..
        } => Unknown,
        _ => Unsatisfied,
    };
    let expected_kind = PackageKind::GameIntegration(integration);
    let package_state = if package.is_some_and(|value| value.manifest.kind == expected_kind) {
        Satisfied
    } else {
        Unsatisfied
    };
    let libraries_state =
        if libraries.is_some_and(|value| value.manifest.kind == PackageKind::XxmiLibraries) {
            Satisfied
        } else {
            Unsatisfied
        };
    let distribution = match &platform.installation {
        GameInstallationAssessment::Detected { distribution, .. } => match distribution {
            GameDistribution::Steam => "steam",
            GameDistribution::HoYoPlay => "hoyoplay",
            GameDistribution::Manual => "manual",
            GameDistribution::Unknown => "unknown",
        },
        GameInstallationAssessment::NotFound { .. }
        | GameInstallationAssessment::Unknown { .. } => "unknown",
    };
    let runtime_state = if !platform.available_runtime_candidates.is_empty() {
        Satisfied
    } else if platform.candidate_discovery == RuntimeCandidateDiscovery::Complete {
        Unsatisfied
    } else {
        Unknown
    };
    let runtime_selection = match platform.selection {
        RuntimeSelection::Unknown { .. } => Unknown,
        RuntimeSelection::Selected { .. } => Satisfied,
        RuntimeSelection::NotRequired => NotRequired,
    };
    let prefix_state = match platform.prefix {
        PrefixAssessment::CandidateFound { .. } => Satisfied,
        PrefixAssessment::NotInitialized { .. } => Unsatisfied,
        PrefixAssessment::NotRequired => NotRequired,
        _ => Unknown,
    };
    let integration_name = match integration {
        IntegrationKind::Wwmi => "WWMIPackageAvailable",
        IntegrationKind::Zzmi => "ZZMIPackageAvailable",
        IntegrationKind::Gimi => "GIMIPackageAvailable",
        IntegrationKind::Unknown => "IntegrationPackageAvailable",
    };
    let integration_evidence = match integration {
        IntegrationKind::Wwmi => "WWMIv1 structure was checked; authenticity and launch remain unverified.",
        IntegrationKind::Zzmi => "ZZMI structure was checked against the current upstream package tree; authenticity and launch remain unverified.",
        IntegrationKind::Gimi | IntegrationKind::Unknown => "No structural contract is implemented for this integration.",
    };
    let package_layout_known = supported && package_state == Satisfied;
    let requirements = vec![
        requirement(
            "SupportedGame",
            if supported { Satisfied } else { Unsatisfied },
            "Explicit game-to-integration mapping in lxmi-xxmi.",
        ),
        requirement(
            "GameInstalled",
            installed,
            "Directory observation only; game completeness remains unknown.",
        ),
        requirement(
            "DistributionDetected",
            if distribution == "unknown" { Unknown } else { Satisfied },
            "Distribution is derived from the discovery source; only Steam discovery is implemented in this increment.",
        ),
        requirement(
            "SteamIntegrationSupport",
            if distribution == "steam" { Unknown } else { NotRequired },
            "Upstream Steam-specific integration support is not verified.",
        ),
        requirement(
            "LinuxProtonCompatibility",
            Unknown,
            "No ZZZ/ZZMI launch test on Linux/Proton has been performed.",
        ),
        requirement(
            "XXMIPackageAvailable",
            libraries_state,
            "XXMI Libraries is a separate package dependency; LXMI does not verify its upstream signatures.",
        ),
        requirement(integration_name, package_state, integration_evidence),
        requirement(
            "RuntimeCandidateAvailable",
            runtime_state,
            "Available runtimes are candidates; no per-game selection is inferred.",
        ),
        requirement(
            "RuntimeSelection",
            runtime_selection,
            "Selection is copied from the platform assessment, never inferred from candidate count.",
        ),
        requirement(
            "GamePrefixInitialized",
            prefix_state,
            "A prefix path is only a filesystem candidate, not proof of health or compatibility.",
        ),
        requirement(
            "RequiredFilesKnown",
            if package_layout_known { Satisfied } else { Unknown },
            "Bounded structural paths were compared with the upstream package layout.",
        ),
        requirement(
            "WriteAccessAvailable",
            Unknown,
            "No write probe was made. This increment has no apply operation.",
        ),
        requirement(
            "LaunchCompatibility",
            Unknown,
            "The game and importer have not been launched together.",
        ),
    ];
    let planning_possible = supported
        && installed == Satisfied
        && package_state == Satisfied
        && libraries_state == Satisfied;

    IntegrationAssessment {
        game_id: platform.game.id.into(),
        integration,
        distribution: distribution.into(),
        game_support: if supported { Satisfied } else { Unsatisfied },
        steam_support: PlatformVerification::Unverified,
        linux_proton_support: PlatformVerification::Unverified,
        platform_compatibility_verified: false,
        requirements,
        planning_possible,
        launch_compatibility: LaunchCompatibility::NotVerified,
    }
}

fn requirement(
    name: &str,
    state: IntegrationRequirementState,
    evidence: &str,
) -> IntegrationRequirement {
    IntegrationRequirement {
        name: name.into(),
        state,
        evidence: evidence.into(),
    }
}

pub fn plan_installation(
    store: &ManagedStore,
    platform: &GameRuntimePlan,
    wwmi_id: &str,
    libs_id: Option<&str>,
) -> Result<InstallationPlan> {
    plan_installation_for_integration(store, platform, IntegrationKind::Wwmi, wwmi_id, libs_id)
}

pub fn plan_installation_for_integration(
    store: &ManagedStore,
    platform: &GameRuntimePlan,
    integration: IntegrationKind,
    package_id: &str,
    libs_id: Option<&str>,
) -> Result<InstallationPlan> {
    tracing::info!(
        ?integration,
        game = platform.game.id,
        "Integration planning started"
    );
    if let GameInstallationAssessment::Detected {
        install_path,
        steam_library,
        ..
    } = &platform.installation
    {
        store.ensure_outside(&[install_path.clone(), steam_library.clone()])?;
    }
    if integration_for_game(platform.game.id) != Some(integration) {
        return Err(XxmiError::new(
            ErrorCode::WrongGameIntegration,
            None,
            "La integración solicitada no está asignada a este juego.",
        ));
    }
    let package = store.verify(package_id)?;
    if package.manifest.kind != PackageKind::GameIntegration(integration) {
        return Err(XxmiError::new(
            ErrorCode::WrongGameIntegration,
            None,
            "El paquete no corresponde a la integración de este juego.",
        ));
    }
    let libraries = libs_id.map(|id| store.verify(id)).transpose()?;
    if libraries
        .as_ref()
        .is_some_and(|value| value.manifest.kind != PackageKind::XxmiLibraries)
    {
        return Err(XxmiError::new(
            ErrorCode::InvalidPackage,
            None,
            "Se esperaba el paquete separado XXMI Libraries.",
        ));
    }
    let assessment =
        assess_for_integration(platform, integration, Some(&package), libraries.as_ref());
    let game_executable_candidate = match &platform.installation {
        GameInstallationAssessment::Detected {
            executable_path, ..
        } => executable_path.clone(),
        _ => None,
    };
    let runtime_name = match integration {
        IntegrationKind::Wwmi => "wwmi",
        IntegrationKind::Zzmi => "zzmi",
        IntegrationKind::Gimi => "gimi",
        IntegrationKind::Unknown => "unknown",
    };
    // This is a proposed LXMI-managed staging destination, never a game or prefix path.
    let target = store.root().join("runtimes").join(runtime_name);
    let existing = match Directory::open_absolute(&target) {
        Ok(directory) => Some(directory),
        Err(error) if missing(&error) => None,
        Err(error) => return Err(error),
    };
    let mut files = Vec::new();
    let mut destinations = std::collections::HashSet::new();
    for source_package in std::iter::once(&package).chain(libraries.iter()) {
        for file in &source_package.manifest.files {
            // XXMI Libraries' loader and manifest stay in managed storage. This plan only stages
            // the two DLL payload files inside LXMI; it does not place them next to the game.
            if source_package.manifest.kind == PackageKind::XxmiLibraries
                && !["d3d11.dll", "d3dcompiler_47.dll"].contains(&file.relative_path.as_str())
            {
                continue;
            }
            if !destinations.insert(file.relative_path.to_lowercase()) {
                return Err(XxmiError::new(
                    ErrorCode::InvalidPackage,
                    None,
                    "Colisión de destinos entre paquetes.",
                ));
            }
            validate_relative_path(&file.relative_path)?;
            let old = match &existing {
                None => None,
                Some(directory) => match hash_file(
                    directory,
                    &file.relative_path,
                    ImportLimits::default().max_file_bytes,
                ) {
                    Ok((_, hash)) => Some(hash),
                    Err(error) if missing(&error) => None,
                    Err(error) => return Err(error),
                },
            };
            let action = match &old {
                None => FileAction::Create,
                Some(hash) if *hash == file.sha256 => FileAction::Unchanged,
                Some(_) => FileAction::ReplaceWithBackup,
            };
            files.push(PlannedFile {
                source: source_package.payload_path.join(&file.relative_path),
                target: target.join(&file.relative_path),
                sha256: file.sha256.clone(),
                action,
                previous_sha256: old,
                backup_required: action == FileAction::ReplaceWithBackup,
            });
        }
    }
    tracing::info!(
        files = files.len(),
        ?integration,
        "Integration plan generated"
    );
    Ok(InstallationPlan {
        assessment,
        files,
        managed_target: target,
        game_executable_candidate,
        configuration_changes: vec![
            "El destino incluido en este plan pertenece al almacenamiento administrado por LXMI; no es el directorio del juego.".into(),
            "Steam y Linux/Proton permanecen sin verificar para la integración seleccionada.".into(),
            "La configuración requerida para aplicar archivos al juego todavía no está definida; no se modifican launch options, prefix ni Steam.".into(),
        ],
        warnings: vec![
            "Plan declarativo para revisión; no existe una operación apply en LXMI 0.5.1.".into(),
            "SHA-256 detecta cambios desde la importación local, pero no autentica el origen ni prueba compatibilidad.".into(),
            "La compatibilidad del paquete con esta distribución y Linux/Proton no está verificada.".into(),
            "Antes de una futura instalación se deben definir destino real, backups, journal, rollback y compatibilidad.".into(),
        ],
        executable: false,
    })
}
