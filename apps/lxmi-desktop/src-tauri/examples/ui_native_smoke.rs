//! Opt-in, read-only UI/IPC smoke check. Requires the Vite development server.
//! Does not save profiles, prepare runtimes, start helpers, or launch games.
use std::time::Duration;
use tauri::{webview::PageLoadEvent, Listener, Manager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !cfg!(debug_assertions)
        || std::env::var("LXMI_CONFIRM_NATIVE_UI_SMOKE").as_deref() != Ok("YES")
    {
        return Err("Opt in with LXMI_CONFIRM_NATIVE_UI_SMOKE=YES in a debug build".into());
    }
    lxmi_desktop_lib::desktop_builder()
        .setup(|app| {
            let handle = app.handle().clone();
            app.listen("lxmi-native-ui-smoke", move |event| {
                let report: serde_json::Value = match serde_json::from_str(event.payload()) {
                    Ok(report) => report,
                    Err(_) => return,
                };
                println!("NATIVE_UI_SMOKE {report}");
                handle.exit(if report["status"] == "passed" { 0 } else { 1 });
            });
            let watchdog = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(120));
                watchdog.exit(2);
            });
            Ok(())
        })
        .on_page_load(|webview, payload| {
            if payload.event() == PageLoadEvent::Finished
                && payload.url().host_str() == Some("127.0.0.1")
            {
                if let Err(error) = webview.eval(include_str!("ui_native_smoke.js")) {
                    eprintln!("Cannot start native UI smoke check: {error}");
                    webview.app_handle().exit(2);
                }
            }
        })
        .run(tauri::generate_context!())?;
    Ok(())
}
