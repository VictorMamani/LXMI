fn main() {
    if let Err(error) = lxmi_desktop_lib::run() {
        eprintln!("LXMI failed to start: {error}");
        std::process::exit(1);
    }
}
