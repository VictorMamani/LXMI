//! Opt-in, bounded passive observation while the user starts ZZZ from Steam.
//! Uses production allowlisted procfs parsing. No raw environment or command line output.
use lxmi_core::{SystemInfo, ZENLESS_ZONE_ZERO_APP_ID};
use lxmi_proton::{CompatibilityToolKind, CompatibilityToolStatus, ProtonScanner};
use lxmi_runtime::{ObservationTarget, ProcfsObserver, RuntimeCandidate};
use lxmi_steam::SteamDiscoveryScanner;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("LXMI_CONFIRM_PASSIVE_ZZZ_OBSERVATION").as_deref() != Ok("YES") {
        return Err(
            "Confirm the user's manual Steam launch with LXMI_CONFIRM_PASSIVE_ZZZ_OBSERVATION=YES"
                .into(),
        );
    }
    let seconds: u64 = std::env::args()
        .nth(1)
        .map(|v| v.parse())
        .transpose()?
        .unwrap_or(180);
    if !(1..=600).contains(&seconds) {
        return Err("Observation duration must be 1..=600 seconds".into());
    }
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let game = discovery
        .games
        .games
        .iter()
        .find(|g| g.steam_app_id == ZENLESS_ZONE_ZERO_APP_ID)
        .ok_or("ZZZ Steam manifest not detected")?;
    let target = ObservationTarget {
        app_id: game.steam_app_id,
        expected_executable_names: game
            .installation
            .game
            .expected_executable_names
            .iter()
            .map(|s| (*s).into())
            .collect(),
        game_install_path: Some(game.installation.install_path.clone()),
        expected_executable_path: game.installation.executable_path.clone(),
        expected_compatdata_path: Some(game.compatdata.compatdata_path.clone()),
        home_directory: SystemInfo::current().home_directory,
    };
    let candidates: Vec<_> = ProtonScanner
        .scan(&discovery.steam.installations)
        .tools
        .iter()
        .filter(|t| {
            t.kind == CompatibilityToolKind::Proton && t.status == CompatibilityToolStatus::Valid
        })
        .map(RuntimeCandidate::from)
        .collect();
    let observer = ProcfsObserver::for_current_user()?;
    let started = Instant::now();
    let mut previous = None;
    println!(
        "{}",
        serde_json::json!({"status":"waiting_for_manual_steam_launch","app_id":ZENLESS_ZONE_ZERO_APP_ID,"duration_seconds":seconds,"read_only":true})
    );
    while started.elapsed() < Duration::from_secs(seconds) {
        let scan = observer.scan(&target, &candidates);
        let summary = serde_json::json!({
            "status": match scan.matches.len() {0=>"waiting_for_game",1=>"observed",_=>"ambiguous"},
            "diagnostics": scan.diagnostics, "warnings":scan.warnings,
            "matches":scan.matches.iter().map(|m| serde_json::json!({
                "app_id":m.app_id,"runtime":m.runtime,"compatdata":m.compatdata,
                "prefix":m.prefix,"wineserver":m.wineserver,"environment_evidence":m.environment_evidence
            })).collect::<Vec<_>>()
        });
        // Process counts fluctuate while a game runs; emit context changes only.
        let context = serde_json::json!({
            "status":summary["status"], "matches":summary["matches"], "warnings":summary["warnings"]
        });
        if previous.as_ref() != Some(&context) {
            println!("{summary}");
            previous = Some(context);
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    println!(
        "{}",
        serde_json::json!({"status":"observation_window_closed","elapsed_seconds":started.elapsed().as_secs(),"read_only":true})
    );
    Ok(())
}
