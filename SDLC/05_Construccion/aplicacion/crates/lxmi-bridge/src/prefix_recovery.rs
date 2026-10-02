//! Full-prefix recovery evidence. No source writes and no real-target restore API.
use super::*;
use std::collections::HashSet;
use std::ffi::{CStr, CString};
use std::os::fd::{AsRawFd, IntoRawFd};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_ENTRIES: usize = 100_000;
const MAX_BYTES: u64 = 32 * 1024 * 1024 * 1024;
const MAX_FILE: u64 = 8 * 1024 * 1024 * 1024;
const MAX_MANIFEST: u64 = 64 * 1024 * 1024;
const RESERVE: u64 = 256 * 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(600);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryEntryKind {
    Directory,
    RegularFile,
    Symlink,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryEntry {
    pub relative_path: String,
    pub kind: RecoveryEntryKind,
    pub size: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub mtime_seconds: i64,
    pub mtime_nanoseconds: i64,
    pub symlink_target: Option<String>,
    pub sha256: Option<String>,
    pub xattr_hashes: BTreeMap<String, String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryTreeSize {
    pub apparent_bytes: u64,
    pub allocated_bytes: u64,
    pub files: usize,
    pub directories: usize,
    pub symlinks: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefixRecoveryManifest {
    pub schema_version: u16,
    pub backup_id: String,
    pub created_at_unix_seconds: u64,
    pub game_id: String,
    pub app_id: u32,
    pub source_identity: PrefixFileIdentity,
    pub proton_build: String,
    pub strategy: String,
    pub source_size: RecoveryTreeSize,
    pub entries: Vec<RecoveryEntry>,
    pub policy_limits: Vec<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryVerificationState {
    Verified,
    Mismatch,
    Incomplete,
    Failed,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryVerification {
    pub state: RecoveryVerificationState,
    pub matching_files: usize,
    pub matching_entries: usize,
    pub mismatches: Vec<String>,
    pub unsupported_metadata: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePrefixRestorePlan {
    pub destination: PathBuf,
    pub game_closed_required: bool,
    pub inactive_context_required: bool,
    pub identity_revalidation_required: bool,
    pub verified_backup_required: bool,
    pub explicit_confirmation_required: bool,
    pub protected_paths_required: bool,
    pub real_target_executor_available: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefixRecoveryQualification {
    #[serde(default)]
    pub protected_paths_unchanged: Option<bool>,
    pub backup_id: String,
    pub backup_path: PathBuf,
    pub manifest_sha256: String,
    pub source_identity: PrefixFileIdentity,
    pub proton_build: String,
    pub strategy: String,
    pub source_size: RecoveryTreeSize,
    pub backup_size: RecoveryTreeSize,
    pub available_bytes_before: u64,
    pub available_bytes_after: u64,
    pub integrity: RecoveryVerification,
    pub source_unchanged_during_capture: bool,
    pub restore_path: Option<PathBuf>,
    pub restore_verification: Option<RecoveryVerification>,
    pub real_prefix_restore: String,
    pub rollback: RollbackCapability,
    pub restore_plan: GamePrefixRestorePlan,
    pub retention_policy: String,
}
fn err(code: &str) -> io::Error {
    io::Error::other(code.to_owned())
}
fn tick(started: Instant) -> io::Result<()> {
    if started.elapsed() > TIMEOUT {
        Err(err("recovery_timeout"))
    } else {
        Ok(())
    }
}
fn component(value: &str) -> io::Result<()> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.contains(['/', '\0'])
        || value.len() > 255
    {
        Err(err("unsafe_recovery_component"))
    } else {
        Ok(())
    }
}
fn relative(value: &str) -> io::Result<()> {
    if value == "." {
        return Ok(());
    }
    if value.len() > 4096 {
        return Err(err("recovery_path_limit"));
    }
    for part in value.split('/') {
        component(part)?;
    }
    Ok(())
}
fn fd_path(file: &File) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{}", file.as_raw_fd()))
}
fn open_directory(path: &Path) -> io::Result<File> {
    // Opening root-owned ancestors does not enumerate them. Enumeration uses
    // O_NOATIME for current-user prefix/storage directories, with no fallback.
    let owned = fs::symlink_metadata(path).is_ok_and(|m| m.uid() == unsafe { libc::getuid() });
    OpenOptions::new()
        .read(true)
        .custom_flags(
            libc::O_DIRECTORY
                | libc::O_NOFOLLOW
                | libc::O_CLOEXEC
                | if owned { libc::O_NOATIME } else { 0 },
        )
        .open(path)
}
fn absolute_directory(path: &Path) -> io::Result<File> {
    if !path.is_absolute() {
        return Err(err("absolute_directory_required"));
    }
    let mut file = open_directory(Path::new("/"))?;
    for part in path.components() {
        match part {
            Component::RootDir => {}
            Component::Normal(name) => file = open_directory(&fd_path(&file).join(name))?,
            _ => return Err(err("unsafe_absolute_directory")),
        }
    }
    Ok(file)
}
fn descend(root: &File, path: &str) -> io::Result<File> {
    relative(path)?;
    let mut dir = root.try_clone()?;
    if path != "." {
        for part in path.split('/') {
            dir = open_directory(&fd_path(&dir).join(part))?;
        }
    }
    Ok(dir)
}
fn parent(root: &File, path: &str) -> io::Result<(File, String)> {
    relative(path)?;
    if path == "." {
        return Err(err("root_is_not_a_file"));
    }
    let (dir, name) = path.rsplit_once('/').unwrap_or((".", path));
    Ok((descend(root, dir)?, name.into()))
}
fn read_file(dir: &File, name: &str) -> io::Result<File> {
    component(name)?;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_NOATIME | libc::O_CLOEXEC)
        .open(fd_path(dir).join(name))?;
    if !file.metadata()?.is_file() {
        return Err(err("unsupported_recovery_entry"));
    }
    Ok(file)
}
// fdopendir retains O_NOATIME; reopening via std::fs::read_dir would not.
fn names(dir: &File) -> io::Result<Vec<String>> {
    let fd = dir.try_clone()?.into_raw_fd();
    let stream = unsafe { libc::fdopendir(fd) };
    if stream.is_null() {
        unsafe { libc::close(fd) };
        return Err(io::Error::last_os_error());
    }
    struct Stream(*mut libc::DIR);
    impl Drop for Stream {
        fn drop(&mut self) {
            unsafe { libc::closedir(self.0) };
        }
    }
    let stream = Stream(stream);
    let mut result = Vec::new();
    unsafe { libc::rewinddir(stream.0) };
    loop {
        unsafe {
            *libc::__errno_location() = 0;
        }
        let entry = unsafe { libc::readdir(stream.0) };
        if entry.is_null() {
            if io::Error::last_os_error().raw_os_error() != Some(0) {
                return Err(io::Error::last_os_error());
            }
            break;
        }
        let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) }
            .to_str()
            .map_err(|_| err("non_utf8_recovery_entry"))?;
        if name == "." || name == ".." {
            continue;
        }
        component(name)?;
        result.push(name.into());
        if result.len() > MAX_ENTRIES {
            return Err(err("recovery_entry_limit"));
        }
    }
    result.sort();
    Ok(result)
}
fn attrs(file: &File) -> io::Result<BTreeMap<String, Vec<u8>>> {
    let count = unsafe { libc::flistxattr(file.as_raw_fd(), std::ptr::null_mut(), 0) };
    if count < 0 {
        return Err(io::Error::last_os_error());
    }
    if count > 65536 {
        return Err(err("xattr_limit"));
    }
    let mut bytes = vec![0u8; count as usize];
    if count > 0
        && unsafe { libc::flistxattr(file.as_raw_fd(), bytes.as_mut_ptr().cast(), bytes.len()) }
            != count
    {
        return Err(err("xattr_changed"));
    }
    let mut result = BTreeMap::new();
    for key in bytes.split(|b| *b == 0).filter(|s| !s.is_empty()) {
        let key = std::str::from_utf8(key).map_err(|_| err("non_utf8_xattr"))?;
        let ckey = CString::new(key).map_err(|_| err("invalid_xattr"))?;
        let len =
            unsafe { libc::fgetxattr(file.as_raw_fd(), ckey.as_ptr(), std::ptr::null_mut(), 0) };
        if !(0..=1024 * 1024).contains(&len) {
            return Err(err("xattr_unavailable_or_limit"));
        }
        let mut value = vec![0u8; len as usize];
        if unsafe {
            libc::fgetxattr(
                file.as_raw_fd(),
                ckey.as_ptr(),
                value.as_mut_ptr().cast(),
                value.len(),
            )
        } != len
        {
            return Err(err("xattr_changed"));
        }
        let total: usize = result.values().map(Vec::len).sum();
        if total + value.len() > 1024 * 1024 {
            return Err(err("aggregate_xattr_limit"));
        }
        result.insert(key.into(), value);
    }
    Ok(result)
}
fn hashes(values: &BTreeMap<String, Vec<u8>>) -> BTreeMap<String, String> {
    values
        .iter()
        .map(|(key, value)| (key.clone(), format!("{:x}", Sha256::digest(value))))
        .collect()
}
fn digest(file: &mut File, started: Instant) -> io::Result<String> {
    let mut hash = Sha256::new();
    let mut buf = [0u8; 128 * 1024];
    let mut size = 0;
    loop {
        tick(started)?;
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        size += n as u64;
        if size > MAX_FILE {
            return Err(err("recovery_file_limit"));
        }
        hash.update(&buf[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
type SourceStamp = (u64, u64, i64, i64);
struct Tree {
    entries: Vec<RecoveryEntry>,
    stamps: BTreeMap<String, SourceStamp>,
    size: RecoveryTreeSize,
}
fn scan(root: &File, started: Instant) -> io::Result<Tree> {
    let device = root.metadata()?.dev();
    let mut tree = Tree {
        entries: Vec::new(),
        stamps: BTreeMap::new(),
        size: RecoveryTreeSize {
            apparent_bytes: 0,
            allocated_bytes: 0,
            files: 0,
            directories: 0,
            symlinks: 0,
        },
    };
    let mut manifest_bytes = 8192usize;
    let mut queue = vec![(".".to_owned(), 0usize)];
    let mut allocated = HashSet::new();
    while let Some((path, depth)) = queue.pop() {
        tick(started)?;
        if depth > 64 || tree.entries.len() >= MAX_ENTRIES {
            return Err(err("recovery_tree_limit"));
        }
        let (holder, location) = if path == "." {
            (root.try_clone()?, fd_path(root))
        } else {
            let (dir, name) = parent(root, &path)?;
            let p = fd_path(&dir).join(name);
            (dir, p)
        };
        let before = if path == "." {
            root.metadata()?
        } else {
            fs::symlink_metadata(&location)?
        };
        if before.dev() != device || before.uid() != unsafe { libc::getuid() } {
            return Err(err("unsupported_mount_or_owner"));
        }
        let mut item = RecoveryEntry {
            relative_path: path.clone(),
            kind: RecoveryEntryKind::Directory,
            size: 0,
            mode: before.mode() & 0o7777,
            uid: before.uid(),
            gid: before.gid(),
            mtime_seconds: before.mtime(),
            mtime_nanoseconds: before.mtime_nsec(),
            symlink_target: None,
            sha256: None,
            xattr_hashes: BTreeMap::new(),
        };
        if before.is_dir() {
            let dir = if path == "." {
                root.try_clone()?
            } else {
                open_directory(&location)?
            };
            if dir.metadata()?.ino() != before.ino() {
                return Err(err("source_changed"));
            }
            item.xattr_hashes = hashes(&attrs(&dir)?);
            tree.size.directories += 1;
            for name in names(&dir)?.into_iter().rev() {
                queue.push((
                    if path == "." {
                        name
                    } else {
                        format!("{path}/{name}")
                    },
                    depth + 1,
                ));
            }
        } else if before.is_file() {
            let name = path.rsplit('/').next().ok_or_else(|| err("unsafe_entry"))?;
            let mut file = read_file(&holder, name)?;
            if file.metadata()?.ino() != before.ino() || before.len() > MAX_FILE {
                return Err(err("source_changed_or_file_limit"));
            }
            item.kind = RecoveryEntryKind::RegularFile;
            item.size = before.len();
            item.xattr_hashes = hashes(&attrs(&file)?);
            item.sha256 = Some(digest(&mut file, started)?);
            tree.size.files += 1;
            tree.size.apparent_bytes = tree
                .size
                .apparent_bytes
                .checked_add(before.len())
                .ok_or_else(|| err("size_overflow"))?;
            if tree.size.apparent_bytes > MAX_BYTES {
                return Err(err("recovery_byte_limit"));
            }
        } else if before.file_type().is_symlink() {
            item.kind = RecoveryEntryKind::Symlink;
            item.symlink_target = Some(
                fs::read_link(&location)?
                    .into_os_string()
                    .into_string()
                    .map_err(|_| err("non_utf8_symlink"))?,
            );
            tree.size.symlinks += 1;
            let p = CString::new(location.as_os_str().as_encoded_bytes())
                .map_err(|_| err("invalid_symlink_path"))?;
            if unsafe { libc::llistxattr(p.as_ptr(), std::ptr::null_mut(), 0) } != 0 {
                return Err(err("unsupported_symlink_xattrs"));
            }
        } else {
            return Err(err("unsupported_recovery_entry"));
        }
        let after = if path == "." {
            root.metadata()?
        } else {
            fs::symlink_metadata(&location)?
        };
        let stamp = |m: &fs::Metadata| (m.dev(), m.ino(), m.ctime(), m.ctime_nsec());
        if stamp(&before) != stamp(&after)
            || before.len() != after.len()
            || before.mtime() != after.mtime()
            || before.mtime_nsec() != after.mtime_nsec()
        {
            return Err(err("source_changed"));
        }
        tree.stamps.insert(path, stamp(&before));
        if allocated.insert((before.dev(), before.ino())) {
            tree.size.allocated_bytes += before.blocks() * 512;
        }
        manifest_bytes += serde_json::to_vec_pretty(&item)?.len();
        if manifest_bytes as u64 > MAX_MANIFEST {
            return Err(err("manifest_size_limit"));
        }
        tree.entries.push(item);
    }
    tree.entries
        .sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(tree)
}
/// Reuse the descriptor-anchored recovery engine for controlled lab clones only.
pub(crate) fn clone_private_tree(source: &Path, destination: &Path) -> io::Result<String> {
    let started = Instant::now();
    let source = absolute_directory(source)?;
    let before = scan(&source, started)?;
    let parent = absolute_directory(destination.parent().ok_or_else(|| err("missing_parent"))?)?;
    disk_gate(before.size.apparent_bytes, free_bytes(&parent)?)?;
    fs::DirBuilder::new().mode(0o700).create(destination)?;
    let destination = absolute_directory(destination)?;
    copy_tree(&source, &destination, &before.entries, started)?;
    let copied = scan(&destination, started)?;
    if verify_tree(&before.entries, &copied).state != RecoveryVerificationState::Verified
        || !same_source(&before, &scan(&source, started)?)
    {
        return Err(err("fixture_clone_integrity_or_source_changed"));
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&before.entries)?)
    ))
}

pub(crate) fn private_tree_inventory(path: &Path) -> io::Result<Vec<RecoveryEntry>> {
    Ok(scan(&absolute_directory(path)?, Instant::now())?.entries)
}

fn same_source(a: &Tree, b: &Tree) -> bool {
    a.entries == b.entries && a.stamps == b.stamps
}
fn verify_tree(expected: &[RecoveryEntry], actual: &Tree) -> RecoveryVerification {
    let map = actual
        .entries
        .iter()
        .map(|e| (e.relative_path.as_str(), e))
        .collect::<BTreeMap<_, _>>();
    let wanted = expected
        .iter()
        .map(|e| e.relative_path.as_str())
        .collect::<HashSet<_>>();
    let mut result = RecoveryVerification {
        state: RecoveryVerificationState::Verified,
        matching_files: 0,
        matching_entries: 0,
        mismatches: Vec::new(),
        unsupported_metadata: Vec::new(),
    };
    for e in expected {
        if map.get(e.relative_path.as_str()).is_some_and(|a| **a == *e) {
            result.matching_entries += 1;
            result.matching_files += usize::from(e.kind == RecoveryEntryKind::RegularFile);
        } else if result.mismatches.len() < 32 {
            result.mismatches.push(e.relative_path.clone());
        }
    }
    for e in &actual.entries {
        if !wanted.contains(e.relative_path.as_str()) && result.mismatches.len() < 32 {
            result
                .mismatches
                .push(format!("unexpected:{}", e.relative_path));
        }
    }
    if result.matching_entries != expected.len() || actual.entries.len() != expected.len() {
        result.state = RecoveryVerificationState::Mismatch;
    }
    result
}
fn metadata(file: &File, e: &RecoveryEntry, values: &BTreeMap<String, Vec<u8>>) -> io::Result<()> {
    if unsafe { libc::fchown(file.as_raw_fd(), e.uid, e.gid) } != 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::fchmod(file.as_raw_fd(), e.mode) } != 0 {
        return Err(io::Error::last_os_error());
    }
    for (key, value) in values {
        let key = CString::new(key.as_str()).map_err(|_| err("invalid_xattr"))?;
        if unsafe {
            libc::fsetxattr(
                file.as_raw_fd(),
                key.as_ptr(),
                value.as_ptr().cast(),
                value.len(),
                0,
            )
        } != 0
        {
            return Err(io::Error::last_os_error());
        }
    }
    let times = [
        libc::timespec {
            tv_sec: 0,
            tv_nsec: libc::UTIME_OMIT,
        },
        libc::timespec {
            tv_sec: e.mtime_seconds,
            tv_nsec: e.mtime_nanoseconds,
        },
    ];
    if unsafe { libc::futimens(file.as_raw_fd(), times.as_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
fn copy_tree(
    source: &File,
    destination: &File,
    entries: &[RecoveryEntry],
    started: Instant,
) -> io::Result<()> {
    let mut ordered = entries.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|e| {
        e.relative_path.matches('/').count() + usize::from(e.relative_path != ".")
    });
    for e in &ordered {
        tick(started)?;
        if e.relative_path == "." {
            continue;
        }
        let (dest, name) = parent(destination, &e.relative_path)?;
        let out = fd_path(&dest).join(&name);
        match e.kind {
            RecoveryEntryKind::Directory => fs::DirBuilder::new().mode(0o700).create(out)?,
            RecoveryEntryKind::RegularFile => {
                let (src, base) = parent(source, &e.relative_path)?;
                let mut input = read_file(&src, &base)?;
                let mut output = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
                    .open(out)?;
                let mut hash = Sha256::new();
                let mut bytes = 0u64;
                let mut buf = [0u8; 128 * 1024];
                loop {
                    tick(started)?;
                    let n = input.read(&mut buf)?;
                    if n == 0 {
                        break;
                    }
                    bytes += n as u64;
                    if bytes > e.size {
                        return Err(err("source_changed"));
                    }
                    output.write_all(&buf[..n])?;
                    hash.update(&buf[..n]);
                }
                if bytes != e.size || Some(format!("{:x}", hash.finalize())) != e.sha256 {
                    return Err(err("source_changed"));
                }
                let values = attrs(&input)?;
                if hashes(&values) != e.xattr_hashes {
                    return Err(err("source_xattrs_changed"));
                }
                metadata(&output, e, &values)?;
                output.sync_all()?;
            }
            RecoveryEntryKind::Symlink => {
                std::os::unix::fs::symlink(
                    e.symlink_target
                        .as_ref()
                        .ok_or_else(|| err("missing_symlink_target"))?,
                    &out,
                )?;
                let name = CString::new(name).map_err(|_| err("invalid_name"))?;
                if unsafe {
                    libc::fchownat(
                        dest.as_raw_fd(),
                        name.as_ptr(),
                        e.uid,
                        e.gid,
                        libc::AT_SYMLINK_NOFOLLOW,
                    )
                } != 0
                {
                    return Err(io::Error::last_os_error());
                }
                let times = [
                    libc::timespec {
                        tv_sec: 0,
                        tv_nsec: libc::UTIME_OMIT,
                    },
                    libc::timespec {
                        tv_sec: e.mtime_seconds,
                        tv_nsec: e.mtime_nanoseconds,
                    },
                ];
                if unsafe {
                    libc::utimensat(
                        dest.as_raw_fd(),
                        name.as_ptr(),
                        times.as_ptr(),
                        libc::AT_SYMLINK_NOFOLLOW,
                    )
                } != 0
                {
                    return Err(io::Error::last_os_error());
                }
            }
        }
    }
    for e in ordered
        .into_iter()
        .rev()
        .filter(|e| e.kind == RecoveryEntryKind::Directory)
    {
        let src = descend(source, &e.relative_path)?;
        let dest = descend(destination, &e.relative_path)?;
        let values = attrs(&src)?;
        if hashes(&values) != e.xattr_hashes {
            return Err(err("source_xattrs_changed"));
        }
        metadata(&dest, e, &values)?;
        dest.sync_all()?;
    }
    Ok(())
}
fn private_child(parent: &File, name: &str, new: bool) -> io::Result<File> {
    component(name)?;
    let path = fd_path(parent).join(name);
    match fs::DirBuilder::new().mode(0o700).create(&path) {
        Ok(()) => {}
        Err(e) if !new && e.kind() == io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e),
    }
    let dir = open_directory(&path)?;
    let m = dir.metadata()?;
    if m.uid() != unsafe { libc::getuid() } || m.mode() & 0o077 != 0 {
        return Err(err("private_owned_directory_required"));
    }
    Ok(dir)
}
fn storage(root: &Path) -> io::Result<File> {
    if root.canonicalize()?!=root || root.components().any(|c|matches!(c,Component::Normal(n) if n=="Steam"||n=="steamapps"||n=="4162040"||n=="Zenless Zone Zero")){return Err(err("backup_destination_rejected"));}
    let dir = absolute_directory(root)?;
    let m = dir.metadata()?;
    if m.uid() != unsafe { libc::getuid() } || m.mode() & 0o077 != 0 {
        return Err(err("private_owned_storage_required"));
    }
    Ok(dir)
}
fn free_bytes(dir: &File) -> io::Result<u64> {
    let mut v = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    if unsafe { libc::fstatvfs(dir.as_raw_fd(), v.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let v = unsafe { v.assume_init() };
    v.f_bavail
        .checked_mul(v.f_frsize)
        .ok_or_else(|| err("space_overflow"))
}
fn disk_gate(size: u64, available: u64) -> io::Result<()> {
    let required = size
        .checked_mul(2)
        .and_then(|n| n.checked_add(RESERVE))
        .ok_or_else(|| err("space_overflow"))?;
    if available < required {
        return Err(err("insufficient_backup_and_restore_space"));
    }
    Ok(())
}
fn new_id() -> io::Result<String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| err(&e.to_string()))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
fn validate_id(id: &str) -> io::Result<()> {
    if id.len() != 32
        || !id
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(err("invalid_backup_id"));
    }
    Ok(())
}
fn write_json(dir: &File, name: &str, value: &impl Serialize) -> io::Result<String> {
    component(name)?;
    let bytes = serde_json::to_vec_pretty(value)?;
    if bytes.len() as u64 > MAX_MANIFEST {
        return Err(err("manifest_limit"));
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(fd_path(dir).join(name))?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
fn finalize(parent: &File, staging: &str, final_name: &str) -> io::Result<()> {
    let from = CString::new(staging).map_err(|_| err("invalid_id"))?;
    let to = CString::new(final_name).map_err(|_| err("invalid_id"))?;
    if unsafe {
        libc::renameat2(
            parent.as_raw_fd(),
            from.as_ptr(),
            parent.as_raw_fd(),
            to.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    } != 0
    {
        return Err(io::Error::last_os_error());
    }
    parent.sync_all()
}

fn cleanup_staging(parent: &File, name: &str) -> io::Result<()> {
    component(name)?;
    fn make_removable(dir: &File, depth: usize) -> io::Result<()> {
        if depth > 64 {
            return Err(err("cleanup_depth_limit"));
        }
        if unsafe { libc::fchmod(dir.as_raw_fd(), 0o700) } != 0 {
            return Err(io::Error::last_os_error());
        }
        for name in names(dir)? {
            let path = fd_path(dir).join(name);
            if fs::symlink_metadata(&path)?.is_dir() {
                make_removable(&open_directory(&path)?, depth + 1)?;
            }
        }
        Ok(())
    }
    let dir = open_directory(&fd_path(parent).join(name))?;
    make_removable(&dir, 0)?;
    fs::remove_dir_all(fd_path(parent).join(name))
}
pub fn create_prefix_recovery(
    root: &Path,
    expected: &PrefixFileIdentity,
    build: &str,
    ensure_inactive: impl Fn() -> io::Result<()>,
) -> io::Result<PrefixRecoveryQualification> {
    ensure_inactive()?;
    if !expected
        .canonical_path
        .ends_with("steamapps/compatdata/4162040/pfx")
        || prefix_file_identity(&expected.canonical_path).as_ref() != Some(expected)
    {
        return Err(err("prefix_identity_changed"));
    }
    let managed = storage(root)?;
    let source = absolute_directory(&expected.canonical_path)?;
    let started = Instant::now();
    let before = scan(&source, started)?;
    let available = free_bytes(&managed)?;
    disk_gate(before.size.apparent_bytes, available)?;
    ensure_inactive()?;
    if prefix_file_identity(&expected.canonical_path).as_ref() != Some(expected) {
        return Err(err("prefix_identity_changed"));
    }
    let backups = private_child(&managed, "backups", false)?;
    let game = private_child(&backups, "zenless-zone-zero", false)?;
    let prefixes = private_child(&game, "prefix", false)?;
    let id = new_id()?;
    let temp = format!("{id}.tmp");
    let stage = private_child(&prefixes, &temp, true)?;
    let run = || -> io::Result<PrefixRecoveryQualification> {
        let tree = private_child(&stage, "tree", true)?;
        copy_tree(&source, &tree, &before.entries, started)?;
        let backup = scan(&tree, started)?;
        let integrity = verify_tree(&before.entries, &backup);
        if integrity.state != RecoveryVerificationState::Verified {
            return Err(err("backup_integrity_mismatch"));
        }
        ensure_inactive()?;
        let after = scan(&source, started)?;
        if !same_source(&before, &after)
            || prefix_file_identity(&expected.canonical_path).as_ref() != Some(expected)
        {
            return Err(err("source_changed_during_capture"));
        }
        let manifest=PrefixRecoveryManifest{schema_version:1,backup_id:id.clone(),created_at_unix_seconds:SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),game_id:"zenless-zone-zero".into(),app_id:4162040,source_identity:expected.clone(),proton_build:build.into(),strategy:"descriptor_anchored_recursive_copy".into(),source_size:before.size.clone(),entries:before.entries.clone(),
            policy_limits:vec!["Full prefix tree, no symlink dereference; no devices, sockets, FIFOs, cross-device mounts, foreign owners or non-UTF8 paths".into(),"Preserve UID/GID, mode, nanosecond mtime, regular/dir xattrs including ACLs; symlink xattrs rejected".into(),"Atime/ctime/inodes/birthtime, sparse allocation and hardlink topology are not restored; hardlinks become independent file copies".into(),"100000 entries; depth64; file8GiB; logical32GiB; 600 seconds per operation; no atomic source freeze".into()]};
        let hash = write_json(&stage, "manifest.json", &manifest)?;
        write_json(&stage, "manifest-sha256.json", &hash)?;
        stage.sync_all()?;
        let report=PrefixRecoveryQualification{protected_paths_unchanged:None,backup_id:id.clone(),backup_path:root.join("backups/zenless-zone-zero/prefix").join(&id),manifest_sha256:hash,source_identity:expected.clone(),proton_build:build.into(),strategy:manifest.strategy,source_size:before.size.clone(),backup_size:backup.size,available_bytes_before:available,available_bytes_after:free_bytes(&managed)?,integrity,source_unchanged_during_capture:true,restore_path:None,restore_verification:None,real_prefix_restore:"not_tested".into(),rollback:RollbackCapability::Partial,
            restore_plan:GamePrefixRestorePlan{destination:expected.canonical_path.clone(),game_closed_required:true,inactive_context_required:true,identity_revalidation_required:true,verified_backup_required:true,explicit_confirmation_required:true,protected_paths_required:true,real_target_executor_available:false},retention_policy:"Future retention: latest 2 plus active qualification; manual cleanup only, no automatic deletion implemented. No overwrite. One backup per review session.".into()};
        finalize(&prefixes, &temp, &id)?;
        Ok(report)
    };
    let result = run();
    if result.is_err() {
        cleanup_staging(&prefixes, &temp)?;
    }
    result
}
fn load_backup(
    root: &Path,
    report: &PrefixRecoveryQualification,
) -> io::Result<(File, PrefixRecoveryManifest)> {
    validate_id(&report.backup_id)?;
    let managed = storage(root)?;
    let path = format!("backups/zenless-zone-zero/prefix/{}", report.backup_id);
    let dir = descend(&managed, &path)?;
    let file = read_file(&dir, "manifest.json")?;
    let mut bytes = Vec::new();
    file.take(MAX_MANIFEST + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_MANIFEST
        || format!("{:x}", Sha256::digest(&bytes)) != report.manifest_sha256
    {
        return Err(err("manifest_integrity_mismatch"));
    }
    let manifest: PrefixRecoveryManifest = serde_json::from_slice(&bytes)?;
    if manifest.schema_version != 1
        || manifest.backup_id != report.backup_id
        || manifest.source_identity != report.source_identity
        || manifest.app_id != 4162040
    {
        return Err(err("manifest_identity_mismatch"));
    }
    for entry in &manifest.entries {
        relative(&entry.relative_path)?;
    }
    Ok((dir, manifest))
}
pub fn verify_prefix_recovery(
    root: &Path,
    report: &mut PrefixRecoveryQualification,
) -> io::Result<()> {
    verify_prefix_recovery_at(root, report, Instant::now())
}
fn verify_prefix_recovery_at(
    root: &Path,
    report: &mut PrefixRecoveryQualification,
    started: Instant,
) -> io::Result<()> {
    let checked = || -> io::Result<RecoveryVerification> {
        let (dir, manifest) = load_backup(root, report)?;
        let tree = descend(&dir, "tree")?;
        Ok(verify_tree(&manifest.entries, &scan(&tree, started)?))
    };
    match checked() {
        Ok(value) => {
            report.integrity = value;
            if report.integrity.state != RecoveryVerificationState::Verified {
                report.rollback = RollbackCapability::MetadataOnly;
            }
            Ok(())
        }
        Err(e) => {
            report.integrity.state = RecoveryVerificationState::Failed;
            report.rollback = RollbackCapability::MetadataOnly;
            Err(e)
        }
    }
}
pub fn test_prefix_restore(
    root: &Path,
    report: &mut PrefixRecoveryQualification,
) -> io::Result<()> {
    let started = Instant::now();
    verify_prefix_recovery_at(root, report, started)?;
    if report.integrity.state != RecoveryVerificationState::Verified {
        return Err(err("backup_not_verified"));
    }
    if report.restore_path.is_some() {
        return Err(err("restore_test_already_exists"));
    }
    let (backup, manifest) = load_backup(root, report)?;
    let managed = storage(root)?;
    let tests = private_child(&managed, "restore-tests", false)?;
    disk_gate(report.source_size.apparent_bytes, free_bytes(&managed)?)?;
    let temp = format!("{}.tmp", report.backup_id);
    let stage = private_child(&tests, &temp, true)?;
    let run = || -> io::Result<RecoveryVerification> {
        let restored = private_child(&stage, "tree", true)?;
        let source = descend(&backup, "tree")?;
        copy_tree(&source, &restored, &manifest.entries, started)?;
        let verification = verify_tree(&manifest.entries, &scan(&restored, started)?);
        if verification.state != RecoveryVerificationState::Verified {
            return Err(err("restore_verification_mismatch"));
        }
        write_json(&stage, "restore-verification.json", &verification)?;
        stage.sync_all()?;
        finalize(&tests, &temp, &report.backup_id)?;
        Ok(verification)
    };
    match run() {
        Ok(value) => {
            report.restore_verification = Some(value);
            report.restore_path = Some(root.join("restore-tests").join(&report.backup_id));
            report.rollback = RollbackCapability::RecoverableBackupVerifiedOffTarget;
            report.available_bytes_after = free_bytes(&managed)?;
            Ok(())
        }
        Err(e) => {
            cleanup_staging(&tests, &temp)?;
            Err(e)
        }
    }
}
pub fn recovery_source_matches(
    root: &Path,
    report: &PrefixRecoveryQualification,
) -> io::Result<bool> {
    if prefix_file_identity(&report.source_identity.canonical_path).as_ref()
        != Some(&report.source_identity)
    {
        return Ok(false);
    }
    let (_, manifest) = load_backup(root, report)?;
    let source = absolute_directory(&report.source_identity.canonical_path)?;
    Ok(
        verify_tree(&manifest.entries, &scan(&source, Instant::now())?).state
            == RecoveryVerificationState::Verified,
    )
}

pub fn verify_prefix_restore(
    root: &Path,
    report: &mut PrefixRecoveryQualification,
) -> io::Result<()> {
    let (_, manifest) = load_backup(root, report)?;
    let managed = storage(root)?;
    let restored = descend(
        &managed,
        &format!("restore-tests/{}/tree", report.backup_id),
    )?;
    let verification = verify_tree(&manifest.entries, &scan(&restored, Instant::now())?);
    if verification.state != RecoveryVerificationState::Verified {
        report.rollback = RollbackCapability::MetadataOnly;
    }
    report.restore_verification = Some(verification);
    Ok(())
}

/// Mutable evidence journal lives outside the finalized backup content.
pub fn save_recovery_qualification(
    root: &Path,
    report: &PrefixRecoveryQualification,
) -> io::Result<()> {
    validate_id(&report.backup_id)?;
    let managed = storage(root)?;
    let journal = private_child(&managed, "snapshots", false)?;
    let temporary = format!("recovery-{}.tmp", new_id()?);
    write_json(&journal, &temporary, report)?;
    fs::rename(
        fd_path(&journal).join(&temporary),
        fd_path(&journal).join(format!("recovery-{}.json", report.backup_id)),
    )?;
    journal.sync_all()
}

pub fn latest_recovery_qualification(
    root: &Path,
    identity: &PrefixFileIdentity,
) -> io::Result<Option<PrefixRecoveryQualification>> {
    let managed = storage(root)?;
    let journal = match descend(&managed, "snapshots") {
        Ok(dir) => dir,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let mut candidates = Vec::new();
    for name in names(&journal)?
        .into_iter()
        .filter(|s| s.starts_with("recovery-") && s.ends_with(".json"))
        .take(64)
    {
        let file = read_file(&journal, &name)?;
        let mut bytes = Vec::new();
        file.take(128 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 128 * 1024 {
            return Err(err("qualification_journal_limit"));
        }
        let report: PrefixRecoveryQualification = serde_json::from_slice(&bytes)?;
        if &report.source_identity == identity {
            let (_, manifest) = load_backup(root, &report)?;
            candidates.push((manifest.created_at_unix_seconds, report));
        }
    }
    candidates.sort_by_key(|(time, _)| *time);
    Ok(candidates.pop().map(|(_, report)| report))
}
pub fn apply_prefix_recovery(
    report: &mut GamePrefixEntryPreflight,
    recovery: &PrefixRecoveryQualification,
) {
    if recovery.integrity.state != RecoveryVerificationState::Verified
        || !recovery.source_unchanged_during_capture
        || recovery.protected_paths_unchanged != Some(true)
        || recovery
            .restore_verification
            .as_ref()
            .is_none_or(|v| v.state != RecoveryVerificationState::Verified)
        || report.prefix_identity.as_ref() != Some(&recovery.source_identity)
        || report.installed_build.as_ref() != Some(&recovery.proton_build)
    {
        return;
    }
    report.rollback = RollbackCapability::RecoverableBackupVerifiedOffTarget;
    report.rollback_limitations=vec!["Restore procedure verified only in LXMI sandbox; real-prefix restore NotTested, no real-target restore executor".into(),"Recovery scope is full pfx content/selected metadata, not compatdata parent, Steam state, game files or atomic source snapshot".into(),"A later helper entry still requires fresh inactive/build/identity/baseline checks and explicit consent".into()];
    report
        .blockers
        .retain(|b| b != "rollback_insufficient_metadata_only");
    if report.blockers.is_empty() {
        report.result = PrefixPreflightState::EligibleForReview;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{symlink, PermissionsExt};
    struct Fixture {
        base: PathBuf,
        storage: PathBuf,
        source: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let base = env::temp_dir().join(format!("lxmi-recovery-{}", new_id().unwrap()));
            let source = base.join("steamapps/compatdata/4162040/pfx");
            fs::create_dir_all(source.join("drive_c/windows")).unwrap();
            fs::create_dir(source.join("dosdevices")).unwrap();
            fs::write(source.join("system.reg"), b"fixture registry\n").unwrap();
            fs::write(source.join("drive_c/windows/test.txt"), b"test bytes").unwrap();
            symlink("../drive_c", source.join("dosdevices/c:")).unwrap();
            symlink("/", source.join("dosdevices/z:")).unwrap();
            let storage = base.join("lxmi");
            fs::DirBuilder::new().mode(0o700).create(&storage).unwrap();
            Self {
                base,
                storage,
                source,
            }
        }
        fn create(&self) -> PrefixRecoveryQualification {
            create_prefix_recovery(
                &self.storage,
                &prefix_file_identity(&self.source).unwrap(),
                ZZZ_OBSERVED_BUILD,
                || Ok(()),
            )
            .unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.base);
        }
    }
    #[test]
    fn full_tree_backup_and_off_target_restore_preserve_policy() {
        let f = Fixture::new();
        let src = open_directory(&f.source).unwrap();
        let before = scan(&src, Instant::now()).unwrap();
        let mut r = f.create();
        assert_eq!(r.integrity.state, RecoveryVerificationState::Verified);
        assert!(r.source_unchanged_during_capture);
        assert_eq!(r.source_size.symlinks, 2);
        assert_eq!(
            fs::read_link(r.backup_path.join("tree/dosdevices/z:")).unwrap(),
            Path::new("/")
        );
        verify_prefix_recovery(&f.storage, &mut r).unwrap();
        test_prefix_restore(&f.storage, &mut r).unwrap();
        assert_eq!(
            r.rollback,
            RollbackCapability::RecoverableBackupVerifiedOffTarget
        );
        assert_eq!(r.restore_verification.as_ref().unwrap().matching_files, 2);
        assert!(!r.restore_plan.real_target_executor_available);
        assert_eq!(r.real_prefix_restore, "not_tested");
        assert!(recovery_source_matches(&f.storage, &r).unwrap());
        assert!(same_source(&before, &scan(&src, Instant::now()).unwrap()));
        assert!(test_prefix_restore(&f.storage, &mut r).is_err());
    }
    #[test]
    fn tampered_backup_blocks_restore() {
        let f = Fixture::new();
        let mut r = f.create();
        fs::write(r.backup_path.join("tree/system.reg"), b"modified").unwrap();
        verify_prefix_recovery(&f.storage, &mut r).unwrap();
        assert_eq!(r.integrity.state, RecoveryVerificationState::Mismatch);
        assert!(test_prefix_restore(&f.storage, &mut r).is_err());
        assert!(r.restore_path.is_none());
    }
    #[test]
    fn manifest_tamper_fails_and_does_not_replace_original() {
        let f = Fixture::new();
        let mut r = f.create();
        fs::write(r.backup_path.join("manifest.json"), b"{}").unwrap();
        assert!(verify_prefix_recovery(&f.storage, &mut r).is_err());
        assert_eq!(r.integrity.state, RecoveryVerificationState::Failed);
    }
    #[test]
    fn source_change_is_detected_and_staging_is_cleaned() {
        let f = Fixture::new();
        let calls = std::cell::Cell::new(0);
        let result = create_prefix_recovery(
            &f.storage,
            &prefix_file_identity(&f.source).unwrap(),
            ZZZ_OBSERVED_BUILD,
            || {
                let n = calls.get() + 1;
                calls.set(n);
                if n == 3 {
                    fs::write(f.source.join("system.reg"), b"changed by fixture").unwrap();
                }
                Ok(())
            },
        );
        assert!(result.is_err());
        let f = Fixture::new();
        let calls = std::cell::Cell::new(0);
        let result = create_prefix_recovery(
            &f.storage,
            &prefix_file_identity(&f.source).unwrap(),
            ZZZ_OBSERVED_BUILD,
            || {
                let n = calls.get() + 1;
                calls.set(n);
                if n == 2 {
                    fs::write(f.source.join("system.reg"), b"changed before copy").unwrap();
                }
                Ok(())
            },
        );
        assert!(result.is_err());
        let prefixes = f.storage.join("backups/zenless-zone-zero/prefix");
        assert_eq!(fs::read_dir(prefixes).unwrap().count(), 0);
    }
    #[test]
    fn inactive_and_identity_are_mandatory() {
        let f = Fixture::new();
        let id = prefix_file_identity(&f.source).unwrap();
        assert!(
            create_prefix_recovery(&f.storage, &id, ZZZ_OBSERVED_BUILD, || Err(err("active")))
                .is_err()
        );
        let mut wrong = id;
        wrong.inode += 1;
        assert!(create_prefix_recovery(&f.storage, &wrong, ZZZ_OBSERVED_BUILD, || Ok(())).is_err());
        assert!(!f.storage.join("backups").exists());
    }
    #[test]
    fn special_file_and_symlink_destination_rejected() {
        let f = Fixture::new();
        let fifo = CString::new(f.source.join("pipe").as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
        assert!(create_prefix_recovery(
            &f.storage,
            &prefix_file_identity(&f.source).unwrap(),
            ZZZ_OBSERVED_BUILD,
            || Ok(())
        )
        .is_err());
        let alias = f.base.join("alias");
        symlink(&f.storage, &alias).unwrap();
        assert!(storage(&alias).is_err());
        assert!(storage(&f.source).is_err());
    }
    #[test]
    fn paths_disk_gate_and_unique_ids() {
        for path in ["../prefix", "/absolute", "a//b", "a/../b"] {
            assert!(relative(path).is_err());
        }
        assert!(relative("dosdevices/c:").is_ok());
        assert!(disk_gate(100, RESERVE + 199).is_err());
        assert!(disk_gate(100, RESERVE + 200).is_ok());
        assert!(disk_gate(u64::MAX, u64::MAX).is_err());
        assert!(validate_id("../pfx").is_err());
        assert_ne!(new_id().unwrap(), new_id().unwrap());
        let f = Fixture::new();
        let first = f.create();
        let second = f.create();
        assert_ne!(first.backup_id, second.backup_id);
        assert!(first.backup_path.exists());
    }
    #[test]
    fn unexpected_entry_permissions_and_source_change_fail_verification() {
        let f = Fixture::new();
        let mut r = f.create();
        fs::set_permissions(
            r.backup_path.join("tree/system.reg"),
            fs::Permissions::from_mode(0o400),
        )
        .unwrap();
        verify_prefix_recovery(&f.storage, &mut r).unwrap();
        assert_eq!(r.integrity.state, RecoveryVerificationState::Mismatch);
        let mut r = f.create();
        fs::write(r.backup_path.join("tree/unexpected"), b"extra").unwrap();
        verify_prefix_recovery(&f.storage, &mut r).unwrap();
        assert_eq!(r.integrity.state, RecoveryVerificationState::Mismatch);
        fs::write(f.source.join("system.reg"), b"changed source").unwrap();
        assert!(!recovery_source_matches(&f.storage, &r).unwrap());
    }
    #[test]
    fn xattrs_and_nanosecond_mtime_roundtrip() {
        let f = Fixture::new();
        let dir = open_directory(&f.source).unwrap();
        let file = read_file(&dir, "system.reg").unwrap();
        let key = CString::new("user.lxmi-test").unwrap();
        assert_eq!(
            unsafe {
                libc::fsetxattr(
                    file.as_raw_fd(),
                    key.as_ptr(),
                    b"value".as_ptr().cast(),
                    5,
                    0,
                )
            },
            0
        );
        let times = [
            libc::timespec {
                tv_sec: 0,
                tv_nsec: libc::UTIME_OMIT,
            },
            libc::timespec {
                tv_sec: 1_700_000_000,
                tv_nsec: 123456789,
            },
        ];
        assert_eq!(
            unsafe { libc::futimens(file.as_raw_fd(), times.as_ptr()) },
            0
        );
        let mut r = f.create();
        test_prefix_restore(&f.storage, &mut r).unwrap();
        assert_eq!(
            r.restore_verification.unwrap().state,
            RecoveryVerificationState::Verified
        );
    }
}

#[cfg(test)]
mod gate_tests {
    use super::*;
    #[test]
    fn off_target_qualification_is_required_and_does_not_remove_other_blockers() {
        let base = env::temp_dir().join(format!("lxmi-recovery-gate-{}", new_id().unwrap()));
        let compat = base.join("steamapps/compatdata/4162040");
        let prefix = compat.join("pfx");
        fs::create_dir_all(prefix.join("drive_c")).unwrap();
        fs::create_dir(prefix.join("dosdevices")).unwrap();
        for name in ["system.reg", "user.reg", "userdef.reg"] {
            fs::write(prefix.join(name), b"fixture").unwrap();
        }
        let proton = base.join("proton");
        fs::create_dir(&proton).unwrap();
        fs::write(proton.join("version"), ZZZ_OBSERVED_BUILD).unwrap();
        let root = base.join("lxmi");
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        let mut preflight = inspect_game_prefix(GamePrefixPreflightInput {
            compatdata: compat,
            proton_root: proton,
            protected_paths: vec![prefix.join("system.reg")],
            active_processes: 0,
            unresolved_processes: 0,
        });
        capture_game_prefix_baseline(&mut preflight, &root).unwrap();
        let mut recovery = create_prefix_recovery(
            &root,
            preflight.prefix_identity.as_ref().unwrap(),
            ZZZ_OBSERVED_BUILD,
            || Ok(()),
        )
        .unwrap();
        apply_prefix_recovery(&mut preflight, &recovery);
        assert_eq!(preflight.result, PrefixPreflightState::NeedsEvidence);
        test_prefix_restore(&root, &mut recovery).unwrap();
        recovery.protected_paths_unchanged = Some(true);
        preflight.blockers.push("active_context_unknown".into());
        apply_prefix_recovery(&mut preflight, &recovery);
        assert_eq!(preflight.result, PrefixPreflightState::NeedsEvidence);
        preflight.blockers.retain(|b| b != "active_context_unknown");
        apply_prefix_recovery(&mut preflight, &recovery);
        assert_eq!(preflight.result, PrefixPreflightState::EligibleForReview);
        assert!(!preflight.execution_authorized);
        assert!(!preflight.entry_plan.executor_available);
        assert!(preflight.future_consent_required);
        save_recovery_qualification(&root, &recovery).unwrap();
        let loaded =
            latest_recovery_qualification(&root, preflight.prefix_identity.as_ref().unwrap())
                .unwrap()
                .unwrap();
        assert_eq!(loaded.backup_id, recovery.backup_id);
        let path = loaded
            .restore_path
            .as_ref()
            .unwrap()
            .join("tree/system.reg");
        fs::write(path, b"restore tampered").unwrap();
        verify_prefix_restore(&root, &mut recovery).unwrap();
        assert_eq!(recovery.rollback, RollbackCapability::MetadataOnly);
        fs::remove_dir_all(base).unwrap();
    }
    #[test]
    fn immutable_id_and_descriptor_parent_block_symlink_escape() {
        let base = env::temp_dir().join(format!("lxmi-recovery-safety-{}", new_id().unwrap()));
        fs::DirBuilder::new().mode(0o700).create(&base).unwrap();
        let root = storage(&base).unwrap();
        let parent = private_child(&root, "backups", true).unwrap();
        private_child(&parent, "existing", true).unwrap();
        private_child(&parent, "stage", true).unwrap();
        assert!(finalize(&parent, "stage", "existing").is_err());
        std::os::unix::fs::symlink("/", fd_path(&root).join("escape")).unwrap();
        assert!(descend(&root, "escape/tmp").is_err());
        fs::remove_dir_all(base).unwrap();
    }
}
