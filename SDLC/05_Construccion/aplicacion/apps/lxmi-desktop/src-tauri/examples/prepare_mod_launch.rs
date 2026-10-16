//! Prepares a «Jugar con mods» launch copy for the saved ZZZ profile and prints its
//! record. Starts no process; writes only under `$XDG_DATA_HOME/lxmi/launches/`.
fn main() {
    match lxmi_desktop_lib::prepare_mod_launch_copy("zenless-zone-zero") {
        Ok(prepared) => println!("{}", serde_json::to_string_pretty(&prepared).expect("json")),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
