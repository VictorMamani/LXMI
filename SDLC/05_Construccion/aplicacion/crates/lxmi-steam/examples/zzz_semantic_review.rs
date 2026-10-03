//! Opt-in read-only host review. Never writes Steam files or launches processes.
fn main() -> std::io::Result<()> {
    let root = std::path::Path::new("/home/university/.local/share/Steam");
    let report = lxmi_steam::review_zzz_steam_state(root, 5)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
