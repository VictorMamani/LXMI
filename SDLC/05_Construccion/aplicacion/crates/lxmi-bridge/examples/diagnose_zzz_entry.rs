//! Manual READ-ONLY SOURCE diagnostic for the already-completed 0.8.8 session.
//! No command/PID/path input, no Wine/Proton executor, no authorization output.
use lxmi_bridge::{diagnose_registry_change, prefix_file_identity, RealPrefixEntryResult};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Read},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};

fn read_source(path: &Path) -> io::Result<Vec<u8>> {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NOATIME)
        .open(path)?;
    let before = file.metadata()?;
    if !before.is_file()
        || before.uid() != unsafe { libc::getuid() }
        || before.len() > 8 * 1024 * 1024
    {
        return Err(io::Error::other("unsafe_or_oversized_source"));
    }
    let mut bytes = Vec::new();
    file.by_ref()
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    let after = fs::symlink_metadata(path)?;
    if bytes.len() > 8 * 1024 * 1024
        || (
            before.dev(),
            before.ino(),
            before.len(),
            before.mtime(),
            before.mtime_nsec(),
        ) != (
            after.dev(),
            after.ino(),
            after.len(),
            after.mtime(),
            after.mtime_nsec(),
        )
    {
        return Err(io::Error::other("source_changed_during_read"));
    }
    Ok(bytes)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new("/home/university/.local/share/lxmi");
    let receipt_bytes = read_source(
        &root.join("snapshots/real-prefix-entry-699f4768f21d30842653eb404baeb237-after.json"),
    )?;
    if format!("{:x}", Sha256::digest(&receipt_bytes))
        != "1f2b3d6e7bdaecdc15f14a40a1ebfd3ddae62a387e493f6cfa1ce802fc9da246"
    {
        return Err(io::Error::other("receipt_integrity_mismatch").into());
    }
    let receipt: RealPrefixEntryResult = serde_json::from_slice(&receipt_bytes)?;
    let prefix = Path::new("/home/university/.local/share/Steam/steamapps/compatdata/4162040/pfx");
    let backup =
        root.join("backups/zenless-zone-zero/prefix/f3551122468ed68b825a0182922f3be2/tree");
    if prefix_file_identity(prefix) != receipt.before.prefix_identity
        || receipt.backup_id != "f3551122468ed68b825a0182922f3be2"
        || receipt.session_nonce != "699f4768f21d30842653eb404baeb237"
    {
        return Err(io::Error::other("receipt_or_prefix_identity_mismatch").into());
    }
    let after = receipt
        .after
        .as_ref()
        .ok_or_else(|| io::Error::other("after_missing"))?;
    for name in ["system.reg", "user.reg", "userdef.reg"] {
        let path = prefix.join(name);
        let old_hash = receipt
            .before
            .baseline
            .selected_prefix_paths
            .iter()
            .find(|e| e.path == path)
            .and_then(|e| e.sha256.as_deref())
            .ok_or_else(|| io::Error::other("before_hash_missing"))?;
        let new_hash = after
            .baseline
            .selected_prefix_paths
            .iter()
            .find(|e| e.path == path)
            .and_then(|e| e.sha256.as_deref())
            .ok_or_else(|| io::Error::other("after_hash_missing"))?;
        let report = diagnose_registry_change(
            &read_source(&backup.join(name))?,
            &read_source(&path)?,
            old_hash,
            new_hash,
        )
        .map_err(|e| io::Error::other(format!("{e:?}")))?;
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({"registry":name,"diagnosis":report}))?
        );
    }
    // Identified categories do not prove harmlessness or change the immutable original result.
    println!(
        "original_result={:?}; gate_remains_closed=true; no_retry=true",
        receipt.result
    );
    Ok(())
}
