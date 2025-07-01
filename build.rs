use pyo3_build_config::use_pyo3_cfgs;
use std::env;

fn main() {
    use_pyo3_cfgs();

    // Platform-specific optimizations
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    // Enable CPU-specific optimizations
    match target_arch.as_str() {
        "x86_64" => {
            println!("cargo:rustc-cfg=target_feature=\"sse2\"");
            println!("cargo:rustc-cfg=target_feature=\"sse4.1\"");
            if env::var("CARGO_CFG_TARGET_FEATURE").map_or(false, |f| f.contains("avx2")) {
                println!("cargo:rustc-cfg=target_feature=\"avx2\"");
            }
        }
        "aarch64" => {
            println!("cargo:rustc-cfg=target_feature=\"neon\"");
        }
        _ => {}
    }

    // Platform-specific compiler flags
    match target_os.as_str() {
        "linux" => {
            println!("cargo:rustc-link-arg=-Wl,-O1");
            println!("cargo:rustc-link-arg=-Wl,--as-needed");
            println!("cargo:rustc-link-arg=-Wl,--gc-sections");
        }
        "macos" => {
            println!("cargo:rustc-link-arg=-Wl,-dead_strip");
        }
        "windows" => {
            if target_env != "msvc" {
                println!("cargo:rustc-link-arg=-Wl,--gc-sections");
            }
        }
        _ => {}
    }

    // Release optimizations
    if env::var("PROFILE").map_or(false, |p| p == "release") {
        println!("cargo:rustc-cfg=release_mode");

        // Enable additional optimizations for release builds
        match target_arch.as_str() {
            "x86_64" => {
                println!("cargo:rustc-env=RUSTFLAGS=-C target-cpu=x86-64-v2");
            }
            _ => {}
        }
    }

    // Feature-based configurations
    if cfg!(feature = "simd") {
        println!("cargo:rustc-cfg=simd_enabled");
    }

    if cfg!(feature = "jemalloc") {
        println!("cargo:rustc-cfg=jemalloc_enabled");
    }

    // Rerun if build configuration changes
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");
    println!("cargo:rerun-if-env-changed=PROFILE");
}
