fn main() {
    // Link the ESP32-S3 linker script for memory layout
    println!("cargo:rustc-link-arg=-Tlinkall.x");
    println!("cargo:rustc-link-arg=-Trom_functions.x");

    // Re-run if board config changes
    println!("cargo:rerun-if-changed=boards/");
    println!("cargo:rerun-if-changed=.cargo/config.toml");
}
