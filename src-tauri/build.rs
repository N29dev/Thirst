fn main() {
    #[cfg(target_os = "macos")]
    {
        use std::path::PathBuf;
        use std::process::Command;

        println!("cargo:rerun-if-changed=native/macos/ThirstLogin.swift");
        let source = PathBuf::from("native/macos/ThirstLogin.swift");
        let output = PathBuf::from(std::env::var_os("OUT_DIR")
            .expect("Cargo must supply OUT_DIR"))
            .join("ThirstLogin");

        // Swift helper is built from our own source in the active cargo build.
        // This output location is for developer builds only. Distributable apps
        // will require a signed, packaged helper rather than this OUT_DIR path.
        let status = Command::new("swiftc")
            .arg("-O")
            .arg(&source)
            .arg("-framework").arg("AppKit")
            .arg("-framework").arg("WebKit")
            .arg("-framework").arg("Security")
            .arg("-o").arg(&output)
            .status()
            .expect("Xcode command-line tools (swiftc) are required on macOS");
        assert!(status.success(), "Compiling the native Thirst login helper failed");
    }

    tauri_build::build()
}
