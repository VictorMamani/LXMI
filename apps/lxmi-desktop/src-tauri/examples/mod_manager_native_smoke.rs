//! Opt-in GTK/WebKit smoke test with real Tauri commands and private fixtures.
//! It dispatches DOM events inside its own webview; no desktop keyboard injection.
use std::{fs, path::PathBuf, time::Duration};
use tauri::{webview::PageLoadEvent, Listener, Manager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !cfg!(debug_assertions) {
        return Err("The native smoke example runs only in a debug build".into());
    }
    let root = PathBuf::from(std::env::var("LXMI_NATIVE_MOD_SMOKE_ROOT")?);
    if root.parent() != Some(std::env::temp_dir().as_path())
        || !root
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("lxmi-mod-native-smoke-"))
        || std::env::var_os("XDG_DATA_HOME").map(PathBuf::from) != Some(root.join("data"))
    {
        return Err(
            "Use a dedicated /tmp/lxmi-mod-native-smoke-* root and its data directory".into(),
        );
    }
    let phase = std::env::var("LXMI_NATIVE_MOD_SMOKE_PHASE").unwrap_or_else(|_| "exercise".into());
    if !matches!(phase.as_str(), "exercise" | "verify" | "activation") {
        return Err("Unknown smoke phase".into());
    }
    if phase == "verify" && !root.join("source-c.zip").is_file() {
        return Err("Verify requires source-c.zip; use tools/test-mod-manager-native.sh".into());
    }
    fs::create_dir_all(root.join("data"))?;
    if phase == "exercise" {
        if root.join("data/lxmi/mods").exists() {
            return Err(
                "Exercise requires a fresh empty library; use verify to inspect persisted state"
                    .into(),
            );
        }
        fs::create_dir(root.join("source-a"))?;
        fs::write(
            root.join("source-a/skin.ini"),
            b"; synthetic LXMI fixture A\r\n[Constants]\r\nglobal $lxmi_test = 1\r\n",
        )?;
        fs::write(
            root.join("source-a/texture.dds"),
            b"synthetic resource, not a real texture",
        )?;
        fs::create_dir(root.join("source-b"))?;
        fs::write(
            root.join("source-b/skin.ini"),
            b"; synthetic LXMI fixture B\n[Constants]\nglobal $lxmi_test = 2\n",
        )?;
        fs::create_dir(root.join("invalid-mod"))?;
        fs::write(root.join("invalid-mod/readme.txt"), b"No INI here")?;
    }
    let bindings = serde_json::json!({"root": root, "phase":phase});
    let script = format!(
        "window.__LXMI_NATIVE_SMOKE__ = {bindings};\n{}",
        include_str!("mod_manager_native_smoke.js")
    );
    let output = root.join(format!("native-{phase}.json"));
    lxmi_desktop_lib::desktop_builder()
        .setup(move |app| {
            let handle = app.handle().clone();
            app.listen("lxmi-native-mod-smoke", move |event| {
                let report: serde_json::Value = match serde_json::from_str(event.payload()) {
                    Ok(report) => report,
                    Err(_) => return,
                };
                println!("NATIVE_MOD_SMOKE {report}");
                if matches!(report["status"].as_str(), Some("passed" | "failed")) {
                    match fs::write(
                        &output,
                        serde_json::to_vec_pretty(&report).unwrap_or_default(),
                    ) {
                        Ok(()) => handle.exit(if report["status"] == "passed" { 0 } else { 1 }),
                        Err(error) => {
                            eprintln!("Cannot persist smoke evidence: {error}");
                            handle.exit(2);
                        }
                    }
                }
            });
            let watchdog = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(300));
                watchdog.exit(2);
            });
            Ok(())
        })
        .on_page_load(move |webview, payload| {
            if payload.event() == PageLoadEvent::Finished
                && payload.url().host_str() == Some("127.0.0.1")
            {
                if let Err(error) = webview.eval(&script) {
                    eprintln!("Cannot start native smoke: {error}");
                    webview.app_handle().exit(2);
                }
            }
        })
        .run(tauri::generate_context!())?;
    Ok(())
}
