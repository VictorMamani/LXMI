//! Non-invasive checks before «Play with mods». Only metadata and file existence:
//! this never starts Steam, Proton, the helper or the game. Messages name files, never
//! absolute paths of the user's home.
use crate::host::{LoadMode, ModLaunchTarget};
use std::{fs, path::Path};

/// Arguments the launch may pass to `steam -applaunch`. Anything else is refused.
pub const ALLOWED_LAUNCH_ARGUMENTS: &[&str] = &["-dx11"];
/// Files the Direct Inject importer must have next to its `d3dx.ini`.
const DIRECT_INJECT_FILES: &[&str] = &["d3dx.ini", "d3d11.dll", "d3dcompiler_47.dll"];

/// Everything the preflight reads. `importer_root` is the runtime's importer folder
/// (`<app root>/<ZZMI|WWMI>`); `runtime_target` is the `target` the assembler recorded.
pub struct PreflightInput<'a> {
    pub target: &'a ModLaunchTarget,
    pub importer_root: &'a Path,
    pub runtime_target: &'a str,
}

fn is_file(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.is_file())
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Mode and importer must pair as validated: ZZZ with Hook/ZZMI, WWMI with Direct Inject.
fn mode_matches_importer(mode: LoadMode, folder: &str) -> bool {
    matches!(
        (mode, folder),
        (LoadMode::Hook, "ZZMI") | (LoadMode::DirectInject, "WWMI")
    )
}

/// Reasons the launch must not start. Empty when every static check passes.
pub fn preflight(input: &PreflightInput<'_>) -> Vec<String> {
    let target = input.target;
    let mut blockers = Vec::new();

    if !is_file(&target.game_executable) {
        blockers.push(format!(
            "The game executable was not found ({}).",
            file_name(&target.game_executable)
        ));
    }
    if !file_name(&target.game_executable).eq_ignore_ascii_case(input.runtime_target) {
        blockers.push(format!(
            "The importer expects {} but the game process is {}.",
            input.runtime_target,
            file_name(&target.game_executable)
        ));
    }
    if !fs::symlink_metadata(target.compat_data.join("pfx")).is_ok_and(|m| m.is_dir()) {
        blockers.push(
            "The game Proton prefix is missing. Open the game from Steam once and try again."
                .into(),
        );
    }
    if !mode_matches_importer(target.load_mode, target.integration_folder) {
        blockers.push("The loading strategy does not match this importer.".into());
    }
    if input.importer_root.file_name().and_then(|n| n.to_str()) != Some(target.integration_folder) {
        blockers.push("The importer folder does not match the game.".into());
    }
    if target.load_mode == LoadMode::DirectInject {
        for name in DIRECT_INJECT_FILES {
            if !is_file(&input.importer_root.join(name)) {
                blockers.push(format!(
                    "{name} is missing from the WWMI runtime. Prepare the runtime again."
                ));
            }
        }
    }
    blockers.extend(argument_blockers(target));
    blockers
}

/// Arguments must be allowed, unique, and include `-dx11` for WWMI.
fn argument_blockers(target: &ModLaunchTarget) -> Vec<String> {
    let mut blockers = Vec::new();
    let args = target.launch_arguments;
    for (index, arg) in args.iter().enumerate() {
        if !ALLOWED_LAUNCH_ARGUMENTS.contains(arg) {
            blockers.push(format!("Launch argument is not allowed: {arg}."));
        }
        if args[..index].contains(arg) {
            blockers.push(format!("Duplicate launch argument: {arg}."));
        }
    }
    if target.load_mode == LoadMode::DirectInject && !args.contains(&"-dx11") {
        blockers.push("WWMI requires the -dx11 argument to launch Wuthering Waves.".into());
    }
    blockers
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);

    /// Fresh fixture: `game/Client-Win64-Shipping.exe`, `compat/pfx`, `app/WWMI/`.
    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "lxmi-preflight-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            fs::create_dir_all(root.join("game")).unwrap();
            fs::create_dir_all(root.join("compat/pfx")).unwrap();
            fs::create_dir_all(root.join("app/WWMI")).unwrap();
            fs::write(root.join("game/Client-Win64-Shipping.exe"), b"").unwrap();
            for name in DIRECT_INJECT_FILES {
                fs::write(root.join("app/WWMI").join(name), b"").unwrap();
            }
            Self { root }
        }
        fn target(
            &self,
            mode: LoadMode,
            folder: &'static str,
            args: &'static [&'static str],
        ) -> ModLaunchTarget {
            ModLaunchTarget {
                game_id: "wuthering-waves".into(),
                steam_app_id: 3_513_350,
                game_executable: self.root.join("game/Client-Win64-Shipping.exe"),
                launcher_executable: self.root.join("game/Wuthering Waves.exe"),
                compat_data: self.root.join("compat"),
                steam_root: self.root.clone(),
                integration_folder: folder,
                launcher_label: "Wuthering Waves",
                launch_arguments: args,
                load_mode: mode,
            }
        }
        fn run(&self, target: &ModLaunchTarget) -> Vec<String> {
            preflight(&PreflightInput {
                target,
                importer_root: &self.root.join("app/WWMI"),
                runtime_target: "Client-Win64-Shipping.exe",
            })
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).ok();
        }
    }

    #[test]
    fn a_complete_wwmi_setup_passes_without_starting_anything() {
        let f = Fixture::new();
        let t = f.target(LoadMode::DirectInject, "WWMI", &["-dx11"]);
        assert!(f.run(&t).is_empty());
    }

    #[test]
    fn missing_runtime_files_are_reported_by_name() {
        let f = Fixture::new();
        fs::remove_file(f.root.join("app/WWMI/d3dcompiler_47.dll")).unwrap();
        let t = f.target(LoadMode::DirectInject, "WWMI", &["-dx11"]);
        let blockers = f.run(&t);
        assert_eq!(blockers.len(), 1);
        assert!(blockers[0].contains("d3dcompiler_47.dll"));
    }

    #[test]
    fn a_missing_prefix_blocks_the_launch() {
        let f = Fixture::new();
        fs::remove_dir_all(f.root.join("compat/pfx")).unwrap();
        let t = f.target(LoadMode::DirectInject, "WWMI", &["-dx11"]);
        assert!(f.run(&t).iter().any(|b| b.contains("Proton prefix")));
    }

    #[test]
    fn a_different_game_process_than_the_runtime_target_blocks_the_launch() {
        let f = Fixture::new();
        let t = f.target(LoadMode::DirectInject, "WWMI", &["-dx11"]);
        let blockers = preflight(&PreflightInput {
            target: &t,
            importer_root: &f.root.join("app/WWMI"),
            runtime_target: "ZenlessZoneZero.exe",
        });
        assert!(blockers
            .iter()
            .any(|b| b.contains("expects ZenlessZoneZero.exe")));
    }

    #[test]
    fn wwmi_requires_dx11_and_refuses_unknown_or_repeated_arguments() {
        let f = Fixture::new();
        let missing = f.target(LoadMode::DirectInject, "WWMI", &[]);
        assert!(f.run(&missing).iter().any(|b| b.contains("-dx11")));

        let unknown = f.target(LoadMode::DirectInject, "WWMI", &["-dx11", "-krqlv=hd"]);
        assert!(f.run(&unknown).iter().any(|b| b.contains("-krqlv=hd")));

        let repeated = f.target(LoadMode::DirectInject, "WWMI", &["-dx11", "-dx11"]);
        assert!(f.run(&repeated).iter().any(|b| b.contains("Duplicate")));
    }

    #[test]
    fn hook_and_inject_never_cross_importers() {
        let f = Fixture::new();
        // Hook may not load the WWMI importer, and Direct Inject may not load ZZMI.
        let hook_on_wwmi = f.target(LoadMode::Hook, "WWMI", &[]);
        assert!(f
            .run(&hook_on_wwmi)
            .iter()
            .any(|b| b.contains("does not match")));
        let inject_on_zzmi = f.target(LoadMode::DirectInject, "ZZMI", &["-dx11"]);
        assert!(f
            .run(&inject_on_zzmi)
            .iter()
            .any(|b| b.contains("does not match")));
    }

    #[test]
    fn the_game_executable_must_exist() {
        let f = Fixture::new();
        fs::remove_file(f.root.join("game/Client-Win64-Shipping.exe")).unwrap();
        let t = f.target(LoadMode::DirectInject, "WWMI", &["-dx11"]);
        assert!(f.run(&t).iter().any(|b| b.contains("game executable")));
    }
}
