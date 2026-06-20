use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let src = manifest_dir.join("src/bridge/spanda_cpp_bridge.cpp");
    let bin = out_dir.join("spanda_cpp_bridge");
    let obj = out_dir.join("spanda_cpp_bridge.o");
    let static_lib = out_dir.join("libspanda_cpp_bridge.a");

    println!("cargo:rerun-if-changed={}", src.display());

    let cxx = env::var("CXX").unwrap_or_else(|_| "c++".into());
    let cpp_native = env::var("CARGO_FEATURE_CPP_NATIVE").is_ok();

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

    if !cpp_native {
        return;
    }

    let obj_ok = Command::new(&cxx)
        .args([
            "-std=c++17",
            "-DSPANDA_CPP_LIBRARY",
            "-c",
            src.to_str().unwrap(),
            "-o",
            obj.to_str().unwrap(),
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !obj_ok || !obj.is_file() {
        println!("cargo:warning=spanda-cpp-bridge: failed to compile in-process C++ bridge object");
        return;
    }

    let ar = env::var("AR").unwrap_or_else(|_| "ar".into());
    let lib_ok = Command::new(&ar)
        .args([
            "rcs",
            static_lib.to_str().unwrap(),
            obj.to_str().unwrap(),
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if lib_ok && static_lib.is_file() {
        println!("cargo:rustc-link-search=native={}", out_dir.display());
        println!("cargo:rustc-link-lib=static=spanda_cpp_bridge");
        if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
            println!("cargo:rustc-link-lib=c++");
        } else {
            println!("cargo:rustc-link-lib=stdc++");
        }
        println!("cargo:rustc-env=SPANDA_CPP_NATIVE=1");
    } else {
        println!("cargo:warning=spanda-cpp-bridge: failed to archive in-process C++ bridge");
    }
}
