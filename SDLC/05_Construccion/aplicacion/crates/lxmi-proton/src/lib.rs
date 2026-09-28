mod manifest;
mod model;
mod scanner;

pub use manifest::{
    parse_compatibility_tool_vdf, parse_tool_manifest_vdf, CompatibilityMetadataError,
    CompatibilityToolMetadata, ToolManifestMetadata,
};
pub use model::{
    CompatibilityTool, CompatibilityToolDiscoveryResult, CompatibilityToolDiscoveryStatus,
    CompatibilityToolIssue, CompatibilityToolIssueCode, CompatibilityToolIssueSeverity,
    CompatibilityToolKind, CompatibilityToolSource, CompatibilityToolStatus,
};
pub use scanner::ProtonScanner;
