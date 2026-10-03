//! Opt-in fixed private investigation. No real prefix access or retry capability.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("LXMI_RUN_HISTORICAL_FONT_PROBE").as_deref() != Ok("YES") {
        return Err("explicit private probe opt-in required".into());
    }
    let storage = std::path::Path::new("/home/university/.local/share/lxmi");
    let steam = std::path::Path::new("/home/university/.local/share/Steam");
    eprintln!("fresh private executor qualification; no real prefix execution");
    let recipe = lxmi_bridge::qualify_integrated_executor(storage, steam)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    let path = lxmi_bridge::investigate_pre_entry_backup(storage, steam, &recipe)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    println!("private diagnostic artifact: {}", path.display());
    Ok(())
}
