//! One qualified container recipe. Capabilities are produced only after private self-test.
use super::*;
use fixture_environment::{RuntimeFixtureExecutor, TRANSPORT_FILE};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};

const BUILD: &str = "1790244902 experimental-11.0-20260924-x86_64";
const TOOL_MANIFEST: &str = "9d0bb73e241bac979afeda526b589fce6f26ee2d5e8822b7aadc710b179421bb";
const HELPER_HASH: &str = "2a1939728e18bd3190b8dbb102d943bd5d65fa509617dbe42d48c45d02463fd2";
const MATRIX_HASH: &str = "0797a5ed44ea490ad53b5ac582d80aa77360f57cfe3d25a9ac2e81c15e509403";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinuxRuntimeRecipeIdentity {
    pub schema: u16,
    pub steam_runtime_versions_sha256: String,
    pub pressure_vessel_sha256: String,
    pub entry_point_sha256: String,
    pub wrapper_sha256: String,
    pub proton_build: String,
    pub proton_copy_manifest_sha256: String,
    pub environment_policy_version: u16,
    pub request_transport_version: u16,
    pub context_binding_version: u16,
    pub qualification_evidence_sha256: String,
}
impl LinuxRuntimeRecipeIdentity {
    pub fn qualified_fixture() -> Self {
        Self {
            schema: 1,
            steam_runtime_versions_sha256:
                "ef61e9bce47d60d02707661d116c8048ffa39291b84acee687a1ed4e430af6be".into(),
            pressure_vessel_sha256:
                "0439811997a081e4f90f898b7e98e34e46cff3745a0611c17309947fd47f4da9".into(),
            entry_point_sha256: "caa39b5cde8ea955288b574b49b3416fd16e7be3f53a17249d673f5ecfdce2c1"
                .into(),
            wrapper_sha256: "d5069c037c53f204a9d7d0ecb501b430e57dc2d702d49c9cccd57d569ca87eec"
                .into(),
            proton_build: BUILD.into(),
            proton_copy_manifest_sha256: TOOL_MANIFEST.into(),
            environment_policy_version: 1,
            request_transport_version: 1,
            context_binding_version: 1,
            qualification_evidence_sha256: MATRIX_HASH.into(),
        }
    }
    pub fn fingerprint(&self) -> String {
        format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(self).expect("recipe identity"))
        )
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealPrefixExecutorQualification {
    Qualified,
    RecipeMismatch,
    FixtureMutationMismatch,
    PrivateProtonInvalid,
    RuntimeChanged,
    TransportFailure,
    UnknownMutation,
    Unqualified,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualifiedExecutorPanel {
    pub qualification: RealPrefixExecutorQualification,
    pub identity: LinuxRuntimeRecipeIdentity,
    pub fingerprint: String,
    pub runtime_root: PathBuf,
    pub wrapper: PathBuf,
    pub private_proton: PathBuf,
    pub fixture_runs: usize,
    pub recipe_source: String,
    pub real_execution: String,
    pub limitations: Vec<String>,
}
#[derive(Clone)]
pub struct QualifiedLinuxRuntimeRecipe {
    root: PathBuf,
    steam_root: PathBuf,
    mirror: FixtureProtonMirror,
    inventory: Vec<RecoveryEntry>,
    identity: LinuxRuntimeRecipeIdentity,
    transport_hash: String,
    qualification: RealPrefixExecutorQualification,
    fixture_runs: usize,
}
fn failure(code: &str) -> BridgeError {
    BridgeError::new(BridgeErrorCode::UnsafePrefixContext, code)
}
fn io_error(e: impl ToString) -> BridgeError {
    BridgeError::new(BridgeErrorCode::Io, e.to_string())
}

pub(crate) fn private_recipe_root(path: &Path) -> Result<PathBuf, BridgeError> {
    if let Ok(root) = fixture_environment::fixture_root(path) {
        return Ok(root);
    }
    let root = path.canonicalize().map_err(io_error)?;
    let m = fs::symlink_metadata(path).map_err(io_error)?;
    let name = root
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or_default();
    let parent = root
        .parent()
        .ok_or_else(|| failure("private session parent"))?;
    if root!=path || !m.is_dir() || m.file_type().is_symlink() || m.uid()!=unsafe{libc::getuid()} || m.mode() & 0o077!=0
        || name.len()!=32 || !name.bytes().all(|b|b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || parent.file_name()!=Some(std::ffi::OsStr::new("qualified-entry-v1"))
        || parent.parent().and_then(Path::file_name)!=Some(std::ffi::OsStr::new("tests"))
        || parent.parent().and_then(Path::parent).and_then(Path::file_name)!=Some(std::ffi::OsStr::new("lxmi"))
        || root.components().any(|c|matches!(c,Component::Normal(n) if n=="Steam" || n=="steamapps" || n=="4162040" || n=="Zenless Zone Zero")) {
        return Err(failure("qualified recipe requires fixed private LXMI session storage"));
    }
    Ok(root)
}
#[derive(Serialize, Deserialize)]
struct TransportContext {
    schema: u16,
    real: bool,
    compatdata: PathBuf,
    prefix_identity: PrefixFileIdentity,
    steam_root: PathBuf,
    session_nonce: Option<String>,
    expires_at: u64,
    recipe_fingerprint: String,
}
pub(crate) fn transport_context(root: &Path) -> Result<Option<PathBuf>, BridgeError> {
    if fixture_environment::fixture_root(root).is_ok() {
        return Ok(None);
    }
    let path = safe_existing_file(root, &root.join("entry-context.json"))?;
    let bytes = read_bounded_file(&path, 16 * 1024).map_err(io_error)?;
    if env::var("LXMI_QUALIFIED_CONTEXT_SHA256").ok()
        != Some(format!("{:x}", Sha256::digest(&bytes)))
    {
        return Err(failure("context binding hash mismatch"));
    }
    let c: TransportContext = serde_json::from_slice(&bytes).map_err(io_error)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(io_error)?
        .as_secs();
    let expected = if c.real {
        c.steam_root.join("steamapps/compatdata/4162040")
    } else {
        root.join("test-prefixes/bridge-v1/compatdata")
    };
    if c.schema != 1
        || c.expires_at < now
        || c.expires_at > now + 600
        || c.compatdata != expected
        || c.compatdata.canonicalize().map_err(io_error)? != expected
        || c.prefix_identity.canonical_path != expected.join("pfx")
        || prefix_file_identity(&expected.join("pfx")).as_ref() != Some(&c.prefix_identity)
        || c.recipe_fingerprint != LinuxRuntimeRecipeIdentity::qualified_fixture().fingerprint()
        || (c.real
            && c.session_nonce.as_deref() != env::var("LXMI_FIXTURE_REQUEST_NONCE").ok().as_deref())
    {
        return Err(failure(
            "context binding expired or prefix/nonce/recipe changed",
        ));
    }
    Ok(Some(c.compatdata))
}

fn write_private(path: &Path, bytes: &[u8]) -> Result<(), BridgeError> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(io_error)?;
    f.write_all(bytes)
        .and_then(|()| f.sync_all())
        .map_err(io_error)
}
fn copy_fixed(source: &Path, destination: &Path, limit: u64) -> Result<String, BridgeError> {
    let bytes = read_bounded_file(source, limit).map_err(io_error)?;
    write_private(destination, &bytes)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
fn stage_session(root: &Path, helpers: &Path) -> Result<(), BridgeError> {
    for name in [BRIDGE_HELPER_FILE, BRIDGE_HELPER_MANIFEST_FILE] {
        copy_fixed(
            &safe_existing_file(helpers, &helpers.join(name))?,
            &root.join("helpers").join(name),
            HELPER_MAX_BYTES,
        )?;
    }
    if sha256_file(
        &root.join("helpers").join(BRIDGE_HELPER_FILE),
        HELPER_MAX_BYTES,
    )
    .map_err(io_error)?
        != HELPER_HASH
    {
        return Err(failure("fixed helper changed"));
    }
    let executable = safe_existing_file(helpers, &helpers.join("qualified-runtime-transport"))?;
    copy_fixed(&executable, &root.join(TRANSPORT_FILE), 256 * 1024 * 1024)?;
    fs::set_permissions(
        root.join(TRANSPORT_FILE),
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )
    .map_err(io_error)
}
fn session(storage: &Path) -> Result<PathBuf, BridgeError> {
    let storage = storage.canonicalize().map_err(io_error)?;
    if storage.file_name() != Some(std::ffi::OsStr::new("lxmi")) {
        return Err(failure("LXMI storage required"));
    }
    let parent = storage.join("tests/qualified-entry-v1");
    ensure_controlled_directory(&storage, &parent)?;
    let root = parent.join(random_nonce()?);
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .map_err(io_error)?;
    for name in [
        "helpers",
        "marker",
        "test-prefixes/bridge-v1/home",
        "test-prefixes/bridge-v1/config",
        "test-prefixes/bridge-v1/cache",
        "test-prefixes/bridge-v1/data",
        "runtime-variable",
    ] {
        ensure_controlled_directory(&root, &root.join(name))?;
    }
    write_private(
        &root.join("marker").join(MARKER_FILE),
        br#"{"schema":1,"operation":"qualified_executor_fixture"}"#,
    )?;
    Ok(root)
}
struct QualifiedContainerExecutor<'a> {
    recipe: &'a QualifiedLinuxRuntimeRecipe,
    context_hash: String,
    root: PathBuf,
    final_check: Option<&'a dyn Fn() -> Result<(), BridgeError>>,
}
impl BridgeProcessExecutor for QualifiedContainerExecutor<'_> {
    fn execute(&self, invocation: &ProcessInvocation) -> Result<ProcessOutput, ProcessFailure> {
        self.recipe
            .verify()
            .map_err(|e| ProcessFailure::Launch(io::Error::other(e.to_string())))?;
        if let Some(check) = self.final_check {
            check().map_err(|e| ProcessFailure::Launch(io::Error::other(e.to_string())))?;
        }
        let mut controlled = fixture_environment::controlled_fixture_invocation(invocation);
        controlled.environment.insert(
            "LXMI_QUALIFIED_CONTEXT_SHA256".into(),
            self.context_hash.clone().into(),
        );
        RuntimeFixtureExecutor {
            entry: &self.recipe.wrapper(),
            variable: &self.root.join("runtime-variable"),
        }
        .execute(&controlled)
    }
}
impl QualifiedLinuxRuntimeRecipe {
    fn wrapper(&self) -> PathBuf {
        self.steam_root
            .join("steamapps/common/SteamLinuxRuntime_4/run")
    }
    #[cfg(test)]
    pub(crate) fn unqualified_test_recipe() -> Self {
        Self {
            root: "/not-created".into(),
            steam_root: "/not-created".into(),
            mirror: FixtureProtonMirror {
                source_tool: "/not-created".into(),
                mirror_root: "/not-created".into(),
                verified_tree_manifest_sha256: TOOL_MANIFEST.into(),
                source_unchanged_during_copy: false,
            },
            inventory: vec![],
            identity: LinuxRuntimeRecipeIdentity::qualified_fixture(),
            transport_hash: String::new(),
            qualification: RealPrefixExecutorQualification::Unqualified,
            fixture_runs: 0,
        }
    }
    pub fn panel(&self) -> QualifiedExecutorPanel {
        QualifiedExecutorPanel {qualification:self.qualification,identity:self.identity.clone(),fingerprint:self.identity.fingerprint(),runtime_root:self.wrapper().parent().unwrap().into(),wrapper:self.wrapper(),private_proton:self.mirror.mirror_root.clone(),fixture_runs:self.fixture_runs,recipe_source:MATRIX_HASH.into(),real_execution:"not_run".into(),limitations:vec!["Prior DirectProton UnexpectedMutation preserved; no retry or real restore.".into(),"Private self-test qualification is not real-prefix consent; real execution requires fresh preflight and one-shot review.".into()]}
    }
    pub fn require_qualified(&self) -> Result<(), BridgeError> {
        if self.qualification != RealPrefixExecutorQualification::Qualified {
            return Err(failure("linux_runtime_environment_unqualified"));
        }
        self.verify()
    }
    fn verify(&self) -> Result<(), BridgeError> {
        if self.identity != LinuxRuntimeRecipeIdentity::qualified_fixture() {
            return Err(failure("recipe mismatch"));
        }
        let runtime = self.wrapper().parent().unwrap().to_path_buf();
        for (name, expected) in [
            ("run", &self.identity.wrapper_sha256),
            ("_v2-entry-point", &self.identity.entry_point_sha256),
            ("VERSIONS.txt", &self.identity.steam_runtime_versions_sha256),
            (
                "pressure-vessel/bin/pressure-vessel-wrap",
                &self.identity.pressure_vessel_sha256,
            ),
        ] {
            let path = safe_existing_file(&self.steam_root, &runtime.join(name))?;
            if sha256_file(&path, 64 * 1024 * 1024).map_err(io_error)? != *expected {
                return Err(failure("runtime recipe changed"));
            }
        }
        if fs::read_to_string(
            self.steam_root
                .join("steamapps/common/Proton - Experimental/version"),
        )
        .map_err(io_error)?
        .trim()
            != BUILD
        {
            return Err(failure("RuntimeChanged"));
        }
        fixture_environment::fixture_proton(&self.root)?;
        if sha256_file(&self.root.join(TRANSPORT_FILE), 256 * 1024 * 1024).map_err(io_error)?
            != self.transport_hash
        {
            return Err(failure("transport changed since self-test"));
        }
        let inventory =
            prefix_recovery::private_tree_inventory(&self.mirror.mirror_root).map_err(io_error)?;
        if !tool_inventory_matches(&self.inventory, &inventory) {
            return Err(failure("PrivateProtonInvalid"));
        }
        Ok(())
    }
    pub(crate) fn new_entry_session(&self, storage: &Path) -> Result<PathBuf, BridgeError> {
        self.require_qualified()?;
        let root = session(storage)?;
        stage_session(&root, &storage.join("helpers"))?;
        if sha256_file(&root.join(TRANSPORT_FILE), 256 * 1024 * 1024).map_err(io_error)?
            != self.transport_hash
        {
            return Err(failure("executor binary changed; new self-test required"));
        }
        attach_fixture_proton_mirror(&root, &self.mirror)?;
        Ok(root)
    }
    pub(crate) fn runtime(&self) -> ExplicitBridgeRuntime {
        ExplicitBridgeRuntime {
            display_name: "Qualified private Proton Experimental".into(),
            version: Some(BUILD.into()),
            proton_script: self.mirror.mirror_root.join("proton"),
        }
    }
    pub(crate) fn execute(
        &self,
        config: &BridgeTestConfig,
        capability: &RealPrefixInvocation,
        real: bool,
        final_check: Option<&dyn Fn() -> Result<(), BridgeError>>,
    ) -> Result<BridgeTestResult, BridgeError> {
        if real && final_check.is_none() {
            return Err(failure("real entry final preflight callback required"));
        }
        let root = private_recipe_root(&config.managed_root)?;
        let context = TransportContext {
            schema: 1,
            real,
            compatdata: capability.compatdata.clone(),
            prefix_identity: prefix_file_identity(&capability.compatdata.join("pfx"))
                .ok_or_else(|| failure("prefix identity unavailable"))?,
            steam_root: self.steam_root.clone(),
            session_nonce: if real {
                Some(capability.nonce.clone())
            } else {
                None
            },
            expires_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(io_error)?
                .as_secs()
                + 600,
            recipe_fingerprint: self.identity.fingerprint(),
        };
        let bytes = serde_json::to_vec(&context).map_err(io_error)?;
        let context_path = root.join("entry-context.json");
        if context_path.exists() {
            fs::remove_file(&context_path).map_err(io_error)?;
        }
        write_private(&context_path, &bytes)?;
        let mut result = run_helper_bridge(
            config,
            &QualifiedContainerExecutor {
                recipe: self,
                context_hash: format!("{:x}", Sha256::digest(bytes)),
                root: root.clone(),
                final_check,
            },
            Some(capability),
        );
        fs::remove_file(context_path).map_err(io_error)?;
        // Container metadata is owned, nonce-correlated and bounded. Remove after collection.
        let receipt = root.join(format!("container-runtime-{}.json", capability.nonce));
        if result.is_ok() {
            let metadata_bytes =
                read_bounded_file(&safe_existing_file(&root, &receipt)?, 16 * 1024)
                    .map_err(io_error)?;
            let metadata: serde_json::Value =
                serde_json::from_slice(&metadata_bytes).map_err(io_error)?;
            if !container_receipt_matches(&metadata, &capability.nonce) {
                return Err(failure("container mount/environment receipt mismatch"));
            }
            let proof = root.join(format!(
                "qualified-container-evidence-{}.json",
                capability.nonce
            ));
            write_private(&proof, &metadata_bytes)?;
            if let Ok(bridge) = result.as_mut() {
                bridge.warnings.push(format!(
                    "Qualified wrapper {} · recipe {} · owned container metadata {} · SHA-256 {:x}",
                    self.wrapper().display(),
                    self.identity.fingerprint(),
                    proof.display(),
                    Sha256::digest(&metadata_bytes)
                ));
            }
        }
        if receipt.exists() {
            fs::remove_file(receipt).map_err(io_error)?;
        }
        if root
            .join(format!("bridge-request-{}.json", capability.nonce))
            .exists()
        {
            return Err(failure("request not consumed/cleaned"));
        }
        result
    }
}
fn tool_inventory_matches(before: &[RecoveryEntry], after: &[RecoveryEntry]) -> bool {
    if before.len() != after.len() {
        return false;
    }
    before.iter().zip(after).all(|(a, b)| {
        if a.relative_path == "dist.lock" {
            a.kind == b.kind
                && a.relative_path == b.relative_path
                && a.mode == b.mode
                && a.uid == b.uid
                && a.gid == b.gid
                && a.size == 0
                && b.size == 0
        } else {
            a == b
        }
    })
}
pub fn qualify_integrated_executor(
    storage: &Path,
    steam_root: &Path,
) -> Result<QualifiedLinuxRuntimeRecipe, BridgeError> {
    // No real-prefix discovery or access. All writes and execution are private.
    let root = session(storage)?;
    stage_session(&root, &storage.join("helpers"))?;
    let mirror = stage_fixture_proton_mirror(&root, steam_root)?;
    if mirror.verified_tree_manifest_sha256 != TOOL_MANIFEST {
        return Err(failure(
            "PrivateProtonInvalid: source tree differs from qualified copy; no silent refresh",
        ));
    }
    let mut recipe = QualifiedLinuxRuntimeRecipe {
        root: root.clone(),
        steam_root: steam_root.canonicalize().map_err(io_error)?,
        inventory: prefix_recovery::private_tree_inventory(&mirror.mirror_root)
            .map_err(io_error)?,
        mirror,
        identity: LinuxRuntimeRecipeIdentity::qualified_fixture(),
        transport_hash: sha256_file(&root.join(TRANSPORT_FILE), 256 * 1024 * 1024)
            .map_err(io_error)?,
        qualification: RealPrefixExecutorQualification::Unqualified,
        fixture_runs: 0,
    };
    let config = BridgeTestConfig {
        managed_root: root.clone(),
        runtime_root: root.join("marker"),
        explicit_runtime: recipe.runtime(),
        steam_client_install_path: recipe.steam_root.clone(),
        prefix_mode: PrefixMode::IsolatedTemporaryPrefix,
    };
    let compatdata = prepare_isolated_test_context(&root)?;
    ensure_controlled_directory(&root, &compatdata.join("pfx"))?;
    let invoke = || -> Result<RealPrefixInvocation, BridgeError> {
        Ok(RealPrefixInvocation {
            compatdata: compatdata.clone(),
            working_directory: root.join("test-prefixes/bridge-v1"),
            nonce: random_nonce()?,
        })
    };
    recipe.execute(&config, &invoke()?, false, None)?;
    thread::sleep(Duration::from_secs(7));
    let baseline = qualified_registry_capture(&compatdata.join("pfx"))?;
    let before_identity = prefix_file_identity(&compatdata.join("pfx"));
    let mut previous = baseline.clone();
    let mut previous_metadata = qualified_prefix_metadata(&compatdata.join("pfx"))?;
    let mut evidence = Vec::new();
    let mut classifications = Vec::new();
    for _ in 0..2 {
        let bridge = recipe.execute(&config, &invoke()?, false, None)?;
        thread::sleep(Duration::from_secs(7));
        let after = qualified_registry_capture(&compatdata.join("pfx"))?;
        let sections = qualified_registry_diff(&previous, &after)?;
        let metadata = qualified_prefix_metadata(&compatdata.join("pfx"))?;
        if !qualified_prefix_metadata_matches(&previous_metadata, &metadata) {
            return Err(failure("FixtureMutationMismatch: selected metadata or mappings changed outside qualified policy"));
        }
        if sections.iter().any(|s| {
            !matches!(
                s.classification,
                PrefixMutationClassification::StableExpected
                    | PrefixMutationClassification::VolatileExpected
            )
        }) {
            return Err(failure(
                "FixtureMutationMismatch: unknown or font/environment mutation",
            ));
        }
        if prefix_file_identity(&compatdata.join("pfx")) != before_identity {
            return Err(failure("fixture prefix identity changed"));
        }
        classifications.extend(sections.iter().map(|s| s.classification));
        evidence.push(serde_json::json!({"bridge":bridge,"sections":sections,"metadata_before":previous_metadata,"metadata_after":metadata}));
        previous_metadata = metadata;
        previous = after;
        recipe.fixture_runs += 1;
    }
    recipe.verify()?;
    recipe.qualification = qualify_real_executor_evidence(
        LinuxRuntimeKind::SteamLinuxRuntimeContainer,
        recipe.fixture_runs,
        true,
        true,
        true,
        &classifications,
    );
    recipe.require_qualified()?;
    write_private(&root.join("executor-qualification.json"),&serde_json::to_vec_pretty(&serde_json::json!({"schema":1,"panel":recipe.panel(),"runs":evidence,"real_prefix_accessed":false})).map_err(io_error)?)?;
    Ok(recipe)
}
fn container_receipt_matches(value: &serde_json::Value, nonce: &str) -> bool {
    value["request_nonce"] == nonce
        && value["selected_environment"]["container"] == "pressure-vessel"
        && value["selected_mounts"].as_array().is_some_and(|mounts| {
            [
                "/usr",
                "/run/host/usr",
                "/run/host/fonts",
                "/run/host/local-fonts",
            ]
            .iter()
            .all(|p| {
                mounts.iter().any(|m| {
                    m["mountpoint"] == *p
                        && m["options"]
                            .as_str()
                            .is_some_and(|opts| opts.split(',').any(|v| v == "ro"))
                })
            })
        })
}
/// Selected metadata and shallow inventories only. No recursive user-document reads.
pub fn qualified_prefix_metadata(prefix: &Path) -> Result<serde_json::Value, BridgeError> {
    let mut out = serde_json::Map::new();
    for relative in [
        ".",
        "drive_c",
        "dosdevices",
        "system.reg",
        "user.reg",
        "userdef.reg",
        ".update-timestamp",
        "creation_sync_guard",
        "drive_c/windows/Fonts",
        "drive_c/windows/fonts",
    ] {
        let path = prefix.join(relative);
        let m = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                out.insert(relative.into(), serde_json::Value::Null);
                continue;
            }
            Err(e) => return Err(io_error(e)),
        };
        if m.file_type().is_symlink()
            && matches!(relative, "drive_c/windows/Fonts" | "drive_c/windows/fonts")
        {
            out.insert(relative.into(),serde_json::json!({"symlink_target":fs::read_link(&path).map_err(io_error)?,"uid":m.uid(),"mode":m.mode(),"device":m.dev(),"inode":m.ino(),"mtime":m.mtime(),"mtime_nsec":m.mtime_nsec()}));
            continue;
        }
        if m.file_type().is_symlink()
            || m.uid() != unsafe { libc::getuid() }
            || !(m.is_dir() || m.is_file())
        {
            return Err(failure("unsafe selected prefix metadata"));
        }
        let mut inventory = BTreeMap::new();
        if m.is_dir() {
            for entry in fs::read_dir(&path).map_err(io_error)?.take(4097) {
                let entry = entry.map_err(io_error)?;
                let value = fs::symlink_metadata(entry.path()).map_err(io_error)?;
                let kind = if value.file_type().is_symlink() {
                    "symlink"
                } else if value.is_dir() {
                    "directory"
                } else if value.is_file() {
                    "file"
                } else {
                    return Err(failure("unsupported selected entry"));
                };
                let target = if kind == "symlink" {
                    Some(fs::read_link(entry.path()).map_err(io_error)?)
                } else {
                    None
                };
                inventory.insert(entry.file_name().to_string_lossy().to_string(),serde_json::json!({"kind":kind,"mode":value.mode(),"uid":value.uid(),"symlink_target":target}));
            }
            if inventory.len() > 4096 {
                return Err(failure("selected directory inventory exceeds policy"));
            }
        }
        out.insert(relative.into(),serde_json::json!({"device":m.dev(),"inode":m.ino(),"uid":m.uid(),"mode":m.mode(),"size":m.len(),"mtime":m.mtime(),"mtime_nsec":m.mtime_nsec(),"entries":inventory}));
    }
    Ok(out.into())
}
pub fn qualified_prefix_metadata_matches(
    before: &serde_json::Value,
    after: &serde_json::Value,
) -> bool {
    let (Some(a), Some(b)) = (before.as_object(), after.as_object()) else {
        return false;
    };
    if a.len() != b.len() {
        return false;
    }
    a.iter().all(|(name, old)| {
        let Some(new) = b.get(name) else {
            return false;
        };
        if old == new {
            return true;
        }
        if !matches!(
            name.as_str(),
            "." | "dosdevices" | "system.reg" | "user.reg" | "userdef.reg"
        ) || !old.is_object()
            || !new.is_object()
        {
            return false;
        }
        let (Some(old_fields), Some(new_fields)) = (old.as_object(), new.as_object()) else {
            return false;
        };
        if old_fields.keys().ne(new_fields.keys())
            || old_fields.len() != 8
            || new_fields.len() != 8
            || old_fields.keys().any(|k| {
                !matches!(
                    k.as_str(),
                    "device"
                        | "inode"
                        | "uid"
                        | "mode"
                        | "size"
                        | "mtime"
                        | "mtime_nsec"
                        | "entries"
                )
            })
        {
            return false;
        }
        old["device"] == new["device"]
            && old["uid"] == new["uid"]
            && old["mode"] == new["mode"]
            && old["entries"] == new["entries"]
            && (!matches!(name.as_str(), "." | "dosdevices") || old["inode"] == new["inode"])
    })
}
pub type QualifiedRegistryCapture = BTreeMap<String, Vec<u8>>;
pub fn qualified_registry_capture(prefix: &Path) -> Result<QualifiedRegistryCapture, BridgeError> {
    let mut result = BTreeMap::new();
    for name in ["system.reg", "user.reg", "userdef.reg"] {
        let path = safe_existing_file(prefix, &prefix.join(name))?;
        let bytes = read_bounded_file(&path, 8 * 1024 * 1024).map_err(io_error)?;
        result.insert(name.into(), bytes);
    }
    Ok(result)
}
pub fn qualified_registry_diff(
    before: &QualifiedRegistryCapture,
    after: &QualifiedRegistryCapture,
) -> Result<Vec<FixtureRegistrySectionMutation>, BridgeError> {
    let mut changes = Vec::new();
    for name in ["system.reg", "user.reg", "userdef.reg"] {
        let mut sections = diagnose_fixture_registry_sections(
            before
                .get(name)
                .ok_or_else(|| failure("registry before missing"))?,
            after
                .get(name)
                .ok_or_else(|| failure("registry after missing"))?,
        )
        .map_err(|e| io_error(format!("{e:?}")))?;
        for s in &mut sections {
            s.section = format!("{name}:{}", s.section);
        }
        changes.extend(sections);
    }
    Ok(changes)
}
/// Pure transition for tests and receipt interpretation; never creates an execution capability.
pub fn qualify_real_executor_evidence(
    kind: LinuxRuntimeKind,
    runs: usize,
    recipe_matches: bool,
    private_tool_valid: bool,
    transport_valid: bool,
    mutations: &[PrefixMutationClassification],
) -> RealPrefixExecutorQualification {
    use RealPrefixExecutorQualification as Q;
    if kind != LinuxRuntimeKind::SteamLinuxRuntimeContainer || !recipe_matches {
        return Q::RecipeMismatch;
    }
    if !private_tool_valid {
        return Q::PrivateProtonInvalid;
    }
    if !transport_valid {
        return Q::TransportFailure;
    }
    if mutations.contains(&PrefixMutationClassification::Unknown) {
        return Q::UnknownMutation;
    }
    if mutations.iter().any(|m| {
        !matches!(
            m,
            PrefixMutationClassification::StableExpected
                | PrefixMutationClassification::VolatileExpected
        )
    }) {
        return Q::FixtureMutationMismatch;
    }
    if runs < 2 {
        return Q::Unqualified;
    }
    Q::Qualified
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recipe_identity_binds_all_policy_components() {
        let expected = LinuxRuntimeRecipeIdentity::qualified_fixture();
        for field in 0..8 {
            let mut changed = expected.clone();
            match field {
                0 => changed.wrapper_sha256 = "wrong".into(),
                1 => changed.proton_build = "changed".into(),
                2 => changed.request_transport_version = 9,
                3 => changed.environment_policy_version = 9,
                4 => changed.steam_runtime_versions_sha256 = "changed".into(),
                5 => changed.pressure_vessel_sha256 = "changed".into(),
                6 => changed.entry_point_sha256 = "changed".into(),
                _ => changed.proton_copy_manifest_sha256 = "changed".into(),
            }
            assert_ne!(changed.fingerprint(), expected.fingerprint());
        }
    }
    #[test]
    fn private_tool_integrity_does_not_ignore_executable_changes() {
        let a = RecoveryEntry {
            relative_path: "proton".into(),
            kind: RecoveryEntryKind::RegularFile,
            size: 1,
            mode: 0o700,
            uid: 1000,
            gid: 1000,
            mtime_seconds: 0,
            mtime_nanoseconds: 0,
            symlink_target: None,
            sha256: Some("expected".into()),
            xattr_hashes: BTreeMap::new(),
        };
        let mut b = a.clone();
        b.sha256 = Some("tampered".into());
        assert!(!tool_inventory_matches(&[a], &[b]));
    }
    #[test]
    fn registry_font_or_unknown_difference_is_not_qualified_volatility() {
        let a = BTreeMap::from([
            ("system.reg".into(), b"WINE REGISTRY Version 2\n".to_vec()),
            ("user.reg".into(), b"WINE REGISTRY Version 2\n".to_vec()),
            ("userdef.reg".into(), b"WINE REGISTRY Version 2\n".to_vec()),
        ]);
        let mut b = a.clone();
        b.insert(
            "system.reg".into(),
            b"WINE REGISTRY Version 2\n[Unknown] 1\n\"new\"=\"value\"\n".to_vec(),
        );
        assert_eq!(
            qualified_registry_diff(&a, &b).unwrap()[0].classification,
            PrefixMutationClassification::Unknown
        );
    }
    #[test]
    fn container_receipt_requires_nonce_container_and_each_readonly_mount() {
        let mounts: Vec<_> = [
            "/usr",
            "/run/host/usr",
            "/run/host/fonts",
            "/run/host/local-fonts",
        ]
        .into_iter()
        .map(|p| serde_json::json!({"mountpoint":p,"options":"ro,nodev"}))
        .collect();
        let mut value = serde_json::json!({"request_nonce":"fixture","selected_environment":{"container":"pressure-vessel"},"selected_mounts":mounts});
        assert!(container_receipt_matches(&value, "fixture"));
        assert!(!container_receipt_matches(&value, "wrong"));
        value["selected_mounts"][0]["options"] = "rw".into();
        assert!(!container_receipt_matches(&value, "fixture"));
    }
    #[test]
    fn selected_metadata_allows_only_known_rewrites_not_unknown_fields() {
        let old = serde_json::json!({"system.reg":{"device":1,"inode":1,"uid":1000,"mode":0o644,"size":3,"mtime":0,"mtime_nsec":0,"entries":{}},"drive_c":null});
        let mut changed = old.clone();
        changed["system.reg"]["inode"] = 2.into();
        assert!(qualified_prefix_metadata_matches(&old, &changed));
        changed["system.reg"]["unknown"] = true.into();
        assert!(!qualified_prefix_metadata_matches(&old, &changed));
        let mut changed = old.clone();
        changed["drive_c"] = serde_json::json!({"changed":true});
        assert!(!qualified_prefix_metadata_matches(&old, &changed));
    }
    #[test]
    fn unqualified_recipe_cannot_construct_a_real_capability() {
        assert!(QualifiedLinuxRuntimeRecipe::unqualified_test_recipe()
            .require_qualified()
            .is_err());
    }
    #[test]
    fn executor_transition_requires_two_valid_container_runs_and_known_mutations() {
        use RealPrefixExecutorQualification as Q;
        let k = LinuxRuntimeKind::SteamLinuxRuntimeContainer;
        assert_eq!(
            qualify_real_executor_evidence(
                k,
                2,
                true,
                true,
                true,
                &[PrefixMutationClassification::VolatileExpected]
            ),
            Q::Qualified
        );
        assert_eq!(
            qualify_real_executor_evidence(
                LinuxRuntimeKind::DirectProton,
                2,
                true,
                true,
                true,
                &[]
            ),
            Q::RecipeMismatch
        );
        assert_eq!(
            qualify_real_executor_evidence(k, 2, false, true, true, &[]),
            Q::RecipeMismatch
        );
        assert_eq!(
            qualify_real_executor_evidence(k, 2, true, false, true, &[]),
            Q::PrivateProtonInvalid
        );
        assert_eq!(
            qualify_real_executor_evidence(k, 2, true, true, false, &[]),
            Q::TransportFailure
        );
        assert_eq!(
            qualify_real_executor_evidence(
                k,
                2,
                true,
                true,
                true,
                &[PrefixMutationClassification::Unknown]
            ),
            Q::UnknownMutation
        );
        assert_eq!(
            qualify_real_executor_evidence(
                k,
                2,
                true,
                true,
                true,
                &[PrefixMutationClassification::EnvironmentSpecific]
            ),
            Q::FixtureMutationMismatch
        );
        assert_eq!(
            qualify_real_executor_evidence(k, 1, true, true, true, &[]),
            Q::Unqualified
        );
    }
}
