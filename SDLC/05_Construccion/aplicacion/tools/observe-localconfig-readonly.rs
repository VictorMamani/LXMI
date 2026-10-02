use lxmi_steam::{parse_key_values_document, KeyValuesValue};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    time::{Duration, Instant},
};
const SOURCE: &str =
    "/home/university/.local/share/Steam/userdata/1416423935/config/localconfig.vdf";
fn flatten(
    entries: Vec<(String, KeyValuesValue)>,
    parent: Vec<String>,
    out: &mut BTreeMap<Vec<String>, String>,
) -> Result<(), String> {
    for (key, value) in entries {
        let mut path = parent.clone();
        path.push(key.to_ascii_lowercase());
        match value {
            KeyValuesValue::Scalar(v) => {
                if out.insert(path, v).is_some() {
                    return Err("duplicate_case_insensitive_key".into());
                }
            }
            KeyValuesValue::Object(v) => flatten(v, path, out)?,
        }
    }
    Ok(())
}
fn hash<'a>(records: impl Iterator<Item = (&'a Vec<String>, &'a String)>) -> String {
    let mut h = Sha256::new();
    for (p, v) in records {
        for k in p {
            h.update((k.len() as u64).to_le_bytes());
            h.update(k.as_bytes());
        }
        h.update((v.len() as u64).to_le_bytes());
        h.update(v.as_bytes());
    }
    format!("{:x}", h.finalize())
}
fn sample() -> Result<
    (
        u64,
        i64,
        i64,
        String,
        String,
        String,
        bool,
        BTreeMap<Vec<String>, String>,
    ),
    String,
> {
    let mut f = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NOATIME)
        .open(SOURCE)
        .map_err(|e| e.to_string())?;
    let m = f.metadata().map_err(|e| e.to_string())?;
    if !m.is_file() || m.uid() != unsafe { libc::getuid() } || m.len() > 2 * 1024 * 1024 {
        return Err("unsafe_source".into());
    }
    let mut data = Vec::new();
    f.by_ref()
        .take(2 * 1024 * 1024 + 1)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    let after = fs::symlink_metadata(SOURCE).map_err(|e| e.to_string())?;
    if data.len() > 2 * 1024 * 1024
        || (m.ino(), m.len(), m.mtime(), m.mtime_nsec())
            != (after.ino(), after.len(), after.mtime(), after.mtime_nsec())
    {
        return Err("source_changed_during_sample".into());
    }
    let entries = parse_key_values_document(std::str::from_utf8(&data).map_err(|_| "not_utf8")?)
        .map_err(|e| e.to_string())?;
    let mut map = BTreeMap::new();
    flatten(entries, vec![], &mut map)?;
    let game: Vec<String> = [
        "userlocalconfigstore",
        "software",
        "valve",
        "steam",
        "apps",
        "4162040",
    ]
    .iter()
    .map(|s| (*s).into())
    .collect();
    let whole = hash(map.iter());
    let game_hash = hash(map.iter().filter(|(p, _)| p.starts_with(&game)));
    let mut launch = game;
    launch.push("launchoptions".into());
    let present = map.contains_key(&launch);
    Ok((
        m.ino(),
        m.mtime(),
        m.mtime_nsec(),
        format!("{:x}", Sha256::digest(&data)),
        whole,
        game_hash,
        present,
        map,
    ))
}
fn safe_schema_path(path: &[String]) -> String {
    const ALLOWED: &[&str] = &[
        "userlocalconfigstore",
        "software",
        "valve",
        "steam",
        "apps",
        "friends",
        "webstorage",
        "system",
        "lastplayedtimesync",
        "lastplayedtimessynctime",
        "favoriteserverslastupdatetime",
        "appinfochangenumber",
        "librarydisplaysize",
        "recentlocalplayedgameids",
        "lastplayed",
        "lastupdated",
        "lastupdate",
        "lastsync",
        "lastused",
        "timestamp",
        "playtime",
        "playtime2wks",
        "launchoptions",
        "needscloudsync",
        "compattoolmapping",
        "personaname",
        "avatar",
        "broadcast",
        "settings",
        "interface",
        "chat",
        "streaming",
    ];
    path.iter()
        .map(|key| {
            if ALLOWED.contains(&key.as_str()) || key == "4162040" {
                key.as_str()
            } else {
                "<redacted-key>"
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}
fn main() {
    let began = Instant::now();
    let mut previous = None;
    let mut reported = 0;
    let mut changes = 0;
    while began.elapsed() < Duration::from_secs(240) {
        match sample() {
            Ok(s) => {
                if let Some(old) = previous.as_ref() {
                    let old: &(
                        u64,
                        i64,
                        i64,
                        String,
                        String,
                        String,
                        bool,
                        BTreeMap<Vec<String>, String>,
                    ) = old;
                    if old.3 != s.3 {
                        changes += 1;
                        let changed = old
                            .7
                            .keys()
                            .chain(s.7.keys())
                            .filter(|p| old.7.get(*p) != s.7.get(*p))
                            .collect::<std::collections::BTreeSet<_>>();
                        println!("FILE_CHANGED elapsed_seconds={} semantic_changed={} zzz_branch_changed={} launch_option_presence_changed={} changed_leaf_count={}",began.elapsed().as_secs(),old.4!=s.4,old.5!=s.5,old.6!=s.6,changed.len());
                        for p in changed {
                            let category = if p.iter().any(|k| k == "4162040") {
                                "ZZZ branch"
                            } else if p.len() > 1 && p[1] == "software" {
                                "Other software branch"
                            } else {
                                "Other user configuration branch"
                            };
                            println!(
                                "CHANGED_CATEGORY {category} schema_path={} values_redacted=true",
                                safe_schema_path(p)
                            );
                        }
                    }
                }
                if previous.is_none() {
                    println!("INITIAL inode={} mtime={}.{} file_sha256={} semantic_sha256={} zzz_branch_sha256={} launch_options_present={} zzz_leaf_count={}",s.0,s.1,s.2,s.3,s.4,s.5,s.6,s.7.keys().filter(|p|p.iter().any(|k|k=="4162040")).count());
                }
                previous = Some(s);
            }
            Err(e) => println!("SAMPLE_UNAVAILABLE {e}"),
        }
        if began.elapsed().as_secs() / 30 > reported {
            reported = began.elapsed().as_secs() / 30;
            println!(
                "PROGRESS elapsed_seconds={} file_change_events={changes}",
                began.elapsed().as_secs()
            );
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    println!(
        "COMPLETE duration_seconds={} file_change_events={changes}",
        began.elapsed().as_secs()
    );
}
