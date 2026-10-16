//! Revalidate the cached, previously imported official Libraries 1.2.2.
//! No process execution, download, package mutation or game-prefix entry.
use lxmi_xxmi::*;
use std::{collections::BTreeMap, fs, path::Path};

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let root = Path::new("/tmp/lxmi-wwmi-official-review/lxmi");
    let id = "711ddd3312acfdd420662f5ded293e2d6386b7fcd58136f00cf7a41226c00d77";
    let store = ManagedStore::at(root.to_owned(), ImportLimits::default())?;
    let package = store.verify(id)?;
    let upstream = package
        .manifest()
        .upstream
        .as_ref()
        .ok_or("Missing provenance")?;
    if package.manifest().authenticity != PackageAuthenticity::OfficialReleaseVerified
        || upstream.tag != "v1.2.2"
        || upstream.repository != "SpectrumQT/XXMI-Libs-Package"
    {
        return Err("Unexpected cached package".into());
    }
    let archive = Path::new(
        "/tmp/lxmi-wwmi-official-review/cache/lxmi/downloads/404037520/asset-613439862.zip",
    );
    let archive_signature = verify_release_file(
        &OfficialPackageKind::XxmiLibraries,
        upstream.signature_base64.as_deref(),
        archive,
    );
    if archive_signature != SignatureStatus::Verified {
        return Err("Cached archive signature did not verify".into());
    }
    let companion: serde_json::Value =
        serde_json::from_slice(&fs::read(package.payload_path().join("Manifest.json"))?)?;
    let mut signatures = BTreeMap::new();
    for name in ["3dmloader.dll", "d3d11.dll", "d3dcompiler_47.dll"] {
        let signature = companion["signatures"][name]
            .as_str()
            .ok_or("Missing component signature")?;
        let status = verify_component_signature(&package.payload_path().join(name), signature);
        if status != SignatureStatus::Verified {
            return Err(format!("Component signature failed: {name}").into());
        }
        signatures.insert(name, status);
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "package_id": id, "version": "1.2.2", "payload_path": package.payload_path(),
            "archive_signature": archive_signature, "component_signatures": signatures,
            "files": package.manifest().files, "game_executed": false, "prefix_entered": false
        }))?
    );
    Ok(())
}
