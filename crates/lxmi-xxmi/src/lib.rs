//! Offline XXMI package inspection, managed staging and declarative planning.
//! No process execution and no installation executor are provided.
mod activation;
mod archive;
mod assembly;
mod cache;
mod crypto;
mod filesystem;
mod launch_readiness;
mod mapping;
mod model;
mod mods;
mod package;
mod planning;
mod release;
mod store;
mod topology;

pub use activation::{
    activate_profile, copy_profile_runtime_for_launch, profile_activation_status,
    resolve_profile_runtime, review_profile_activation, rollback_profile_activation,
    ActivationPlan, ActiveProfileSelection, ProfileRuntimeRef,
};
pub use assembly::{
    assemble_importer_runtime, assemble_zzmi_runtime, existing_importer_runtime,
    existing_zzmi_runtime, plan_importer_assembly, plan_zzmi_assembly, ManagedRuntime,
    RuntimeAssemblyFile, RuntimeAssemblyPlan, RuntimeConfigPlan, RuntimeManifest,
};
pub use cache::{CachedAsset, DownloadCache};
pub use crypto::{
    verify_component_signature, verify_file as verify_signature_file, verify_release_file,
    verify_release_signature,
};
pub use filesystem::validate_relative_path;
pub use launch_readiness::{
    assess_loader_strategy, assess_zzmi_launch_readiness, incorporate_runtime_observation,
    ConfiguredToolMappingEvidence, ContextFindingState, ContextToolKind, ContextToolOrigin,
    ControlledGameExperimentGate, HookRequirementState, InstalledCompatibilityToolEvidence,
    LoaderCriterionState, LoaderIsolationEvidence, LoaderIsolationScope, LoaderStrategyCriterion,
    LoaderStrategyDecision, LoaderStrategyOutcome, PerGameToolMappingState,
    ReadinessCapabilityState, ReadinessEvidenceState, RuntimeContextKnowledge,
    SteamCompatConfigEvidenceState, SteamProcessContextFinding, SteamProtonProcessContextEvidence,
    UpstreamLoaderStrategy, ZzmiCapability, ZzmiCapabilityAssessment, ZzmiGateComponent,
    ZzmiGateComponentAssessment, ZzmiLaunchReadiness, ZzmiLaunchReadinessInput, ZzmiLoaderMode,
    ZzmiReadinessDecision, ZzmiReadinessRequirement, ZzmiRequirementState,
};
pub use model::*;
pub use mods::{ModLibrary, ModLibrarySnapshot, ModManifest, ModProfile, PreparedModProfile};
pub use package::{inspect_runtime, validate_directory, WWMI_REQUIRED_FILES, ZZMI_REQUIRED_FILES};
pub use planning::{
    assess, assess_for_integration, integration_for_game, plan_installation,
    plan_installation_for_integration,
};
pub use release::{
    asset_for_release, download_and_import_official, parse_github_release,
    repository as official_repository, FixtureReleaseProvider, GitHubReleaseProvider,
    ReleaseProvider,
};
pub use store::{ManagedStore, VerifiedPackage};
pub use topology::{
    inspect_launch_topology, inspect_prefix_dosdevices, map_linux_path, DosDeviceMapping,
    DosDeviceState, DosDevicesReport, DosDevicesState, LaunchTopologyPlan, LoaderStrategy,
    MappingStatus, SamePrefixRequirement, TopologyReadiness, WindowsPathMapping,
};
