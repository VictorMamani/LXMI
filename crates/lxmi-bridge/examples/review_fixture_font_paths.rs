//! Read-only review of already-owned private fixture artifacts; never invokes a runtime.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let evidence: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../data/environment_qualification_0_8_8_1.json"
    ))?;
    let source = std::path::Path::new(evidence["seed_root"].as_str().ok_or("seed root")?);
    let compat = std::path::Path::new(
        evidence["steps"][0]["execution"]["bridge"]["compatdata_path"]
            .as_str()
            .ok_or("fixture")?,
    );
    let destination = compat
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .ok_or("fixture root")?;
    let report = lxmi_bridge::review_environment_fixture_sections(source, destination)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    for (hive, sections) in &report {
        for section in sections.iter().filter(|s| s.section.contains("Fonts")) {
            let observed = evidence["steps"][0]["sections"][hive]
                .as_array()
                .ok_or("sections")?
                .iter()
                .find(|s| s["section"] == section.section)
                .ok_or("section missing")?;
            if observed["before_sha256"].as_str() != section.before_sha256.as_deref()
                || observed["after_sha256"].as_str() != section.after_sha256.as_deref()
            {
                return Err("font section changed since the recorded host run".into());
            }
        }
    }
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
