//! Fixed native transport only; its copied executable must be inside an owned recipe session.
fn main() {
    if let Err(error) = lxmi_bridge::enter_fixture_container_transport() {
        eprintln!("LXMI qualified transport failed: {error}");
        std::process::exit(1);
    }
}
