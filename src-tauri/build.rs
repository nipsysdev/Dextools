fn main() {
    // Set linker flags required by storage-bindings
    println!("cargo:rustc-link-arg=-Wl,--defsym=__rust_probestack=0");

    tauri_build::build()
}
