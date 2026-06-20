use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let src = manifest_dir.join("src/bridge/spanda_cpp_bridge.cpp");
    let bin = out_dir.join("spanda_cpp_bridge");

    println!("cargo:rerun-if-changed={}", src.display());

    let cxx = env::var("CXX").unwrap_or_else(|_| "c++".into());
    let ok = Command::new(&cxx)
        .args([
            "-std=c++17",
            src.to_str().unwrap(),
            "-o",
            bin.to_str().unwrap(),
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if ok && bin.is_file() {
        println!("cargo:rustc-env=SPANDA_CPP_BRIDGE_BIN={}", bin.display());
    } else {
        println!(
            "cargo:warning=spanda-cpp-bridge: failed to compile C++ bridge helper; \
             set SPANDA_CPP_BRIDGE to a prebuilt binary for extern cpp fn"
        );
    }
}
