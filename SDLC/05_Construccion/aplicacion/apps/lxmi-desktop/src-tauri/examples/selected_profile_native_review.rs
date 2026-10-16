//! Opt-in read-only review of the current ZZZ profile in a real desktop webview.
//! Navigation and inspections only: no import, selection mutation, game or helper.
//! «Jugar con mods» must be enabled for the applied profile; it is not clicked.
use std::{fs, os::unix::fs::OpenOptionsExt, time::Duration};
use tauri::{webview::PageLoadEvent, Listener, Manager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !cfg!(debug_assertions)
        || std::env::var("LXMI_CONFIRM_NATIVE_PROFILE_REVIEW").as_deref() != Ok("YES")
    {
        return Err("Opt in with LXMI_CONFIRM_NATIVE_PROFILE_REVIEW=YES in a debug build".into());
    }
    let output = std::env::temp_dir().join("lxmi-zzz-selected-profile-native.json");
    // An existing report is evidence; do not silently replace it.
    let output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(output)?;
    let output = std::sync::Mutex::new(output);
    lxmi_desktop_lib::desktop_builder()
        .setup(move |app| {
            let handle = app.handle().clone();
            app.listen("lxmi-selected-profile-native-review", move |event| {
                let report: serde_json::Value = match serde_json::from_str(event.payload()) {
                    Ok(value) => value,
                    Err(_) => return,
                };
                let result = (|| -> std::io::Result<()> {
                    use std::io::Write;
                    let mut file = output
                        .lock()
                        .map_err(|_| std::io::Error::other("review lock"))?;
                    file.write_all(&serde_json::to_vec_pretty(&report)?)?;
                    file.sync_all()
                })();
                println!("NATIVE_PROFILE_REVIEW {}", report["status"]);
                handle.exit(if result.is_ok() && report["status"] == "passed" {
                    0
                } else {
                    1
                });
            });
            let watchdog = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(300));
                watchdog.exit(2);
            });
            Ok(())
        })
        .on_page_load(|webview, payload| {
            if payload.event() == PageLoadEvent::Finished
                && payload.url().host_str() == Some("127.0.0.1")
            {
                if let Err(error) = webview.eval(include_str!("selected_profile_native_review.js"))
                {
                    eprintln!("Cannot start native review: {error}");
                    webview.app_handle().exit(2);
                }
            }
        })
        .run(tauri::generate_context!())?;
    Ok(())
}
