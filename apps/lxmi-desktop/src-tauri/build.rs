//! Builds the Windows mod loader helper with MinGW and embeds it in the app, so its
//! bytes (and hash) are fixed at compile time. Without MinGW the app still builds
//! and reports "Jugar con mods" as unavailable.
use std::{path::PathBuf, process::Command};

fn main() {
    println!("cargo::rustc-check-cfg=cfg(lxmi_mod_loader)");
    let source = PathBuf::from("../../../tools/lxmi-mod-loader/loader.c");
    println!("cargo:rerun-if-changed={}", source.display());
    let output = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR")).join("lxmi-loader.exe");
    let built = Command::new("x86_64-w64-mingw32-gcc")
        .args([
            "-O2",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-D_WIN32_WINNT=0x0601",
            "-DUNICODE",
            "-D_UNICODE",
            "-municode",
        ])
        .arg(&source)
        .arg("-o")
        .arg(&output)
        .status();
    match built {
        Ok(status) if status.success() => println!("cargo:rustc-cfg=lxmi_mod_loader"),
        _ => println!("cargo:warning=x86_64-w64-mingw32-gcc no disponible: «Jugar con mods» quedará deshabilitado"),
    }
    tauri_build::build()
}
