//! Link LLVM IR with `libspanda_rt` via clang when available.

use spanda_core::sir::SirProgram;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{default_target_triple_for_host, emit_module_ir_with_options};

#[derive(Debug, Clone)]
pub struct CompileNativeOptions {
    pub output: PathBuf,
    pub clang: Option<String>,
    pub workspace_root: PathBuf,
    pub target_triple: Option<String>,
    pub hal_profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileNativeResult {
    pub llvm_ir_path: PathBuf,
    pub executable: PathBuf,
}

pub fn compile_native(
    sir: &SirProgram,
    opts: &CompileNativeOptions,
) -> Result<CompileNativeResult, String> {
    // Compile native.
    //
    // Parameters:
    // - `sir` — input value
    // - `opts` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_llvm::compile::compile_native(sir, opts);

    // Compute clang for the following logic.
    let clang =
        opts.clang.clone().or_else(detect_clang).ok_or_else(|| {
            "clang not found — install LLVM/clang to use compile-native".to_string()
        })?;
    let ir = emit_module_ir_with_options(
        sir,
        opts.target_triple.as_deref(),
        opts.hal_profile.as_deref(),
    );
    let build_dir = resolve_target_dir(&opts.workspace_root).join("spanda-native");
    std::fs::create_dir_all(&build_dir).map_err(|e| e.to_string())?;
    let llvm_ir_path = build_dir.join("program.ll");
    std::fs::write(&llvm_ir_path, ir).map_err(|e| e.to_string())?;
    let rt_lib = ensure_spanda_rt_staticlib(&opts.workspace_root)?;
    let output = if opts.output.is_absolute() {
        opts.output.clone()
    } else {
        opts.workspace_root.join(&opts.output)
    };
    let mut cmd = Command::new(clang);
    cmd.arg(llvm_ir_path.as_os_str())
        .arg(rt_lib.as_os_str())
        .arg("-o")
        .arg(output.as_os_str());
    let triple = opts
        .target_triple
        .clone()
        .or_else(|| hal_profile_triple(opts.hal_profile.as_deref()).map(str::to_string))
        .unwrap_or_else(|| default_target_triple_for_host().to_string());
    cmd.args(["-target", triple.as_str()]);

    // Take this path when cfg!(target os = "macos").
    if cfg!(target_os = "macos") {
        cmd.arg("-Wl,-no_warn_duplicate_libraries");
        cmd.args([
            "-framework",
            "Security",
            "-framework",
            "SystemConfiguration",
        ]);
        cmd.arg("-liconv");
    }
    let status = cmd
        .status()
        .map_err(|e| format!("failed to run clang: {e}"))?;

    // Handle output when the subprocess succeeds.
    if !status.success() {
        return Err(format!(
            "clang failed linking native binary (exit {status})"
        ));
    }
    Ok(CompileNativeResult {
        llvm_ir_path,
        executable: output,
    })
}

fn hal_profile_triple(profile: Option<&str>) -> Option<&'static str> {
    // Hal profile triple.
    //
    // Parameters:
    // - `profile` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_llvm::compile::hal_profile_triple(profile);

    // Match on profile? and handle each case.
    match profile? {
        "embedded-arm" => Some("aarch64-unknown-linux-gnu"),
        "esp32" => Some("xtensa-esp32-elf"),
        _ => None,
    }
}

fn detect_clang() -> Option<String> {
    // Detect clang.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_llvm::compile::detect_clang();

    // Iterate over ["clang", "clang-18", "clang-17", "clang-16"].
    for candidate in ["clang", "clang-18", "clang-17", "clang-16"] {
        // Take this path when Command::new(candidate).
        if Command::new(candidate)
            .arg("--version")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Some(candidate.to_string());
        }
    }
    None
}

fn resolve_target_dir(workspace_root: &Path) -> PathBuf {
    // Resolve target dir.
    //
    // Parameters:
    // - `workspace_root` — input value
    //
    // Returns:
    // PathBuf.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_llvm::compile::resolve_target_dir(workspace_root);

    // Produce var as the result.
    std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root.join("target"))
}

fn ensure_spanda_rt_staticlib(workspace_root: &Path) -> Result<PathBuf, String> {
    // Ensure spanda rt staticlib.
    //
    // Parameters:
    // - `workspace_root` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_llvm::compile::ensure_spanda_rt_staticlib(workspace_root);

    // Compute target dir for the following logic.
    let target_dir = resolve_target_dir(workspace_root);
    let profile = "debug";
    let lib_path = target_dir.join(profile).join("libspanda_rt.a");

    // Continue only when the path is a regular file.
    if lib_path.is_file() {
        return Ok(lib_path);
    }
    let mut cmd = Command::new("cargo");
    cmd.current_dir(workspace_root)
        .args(["build", "-p", "spanda-rt"]);

    // Handle the success value from var.
    if let Ok(dir) = std::env::var("CARGO_TARGET_DIR") {
        cmd.env("CARGO_TARGET_DIR", dir);
    }
    let status = cmd
        .status()
        .map_err(|e| format!("failed to build spanda-rt: {e}"))?;

    // Handle output when the subprocess succeeds.
    if !status.success() {
        return Err("cargo build -p spanda-rt failed".into());
    }

    // Continue only when the path is a regular file.
    if lib_path.is_file() {
        Ok(lib_path)
    } else {
        Err(format!(
            "spanda-rt static library not found at {}",
            lib_path.display()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_core::lower_to_sir;

    #[test]
    fn compile_native_when_clang_available() {
        // Compile native when clang available.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_llvm::compile::compile_native_when_clang_available();

        if detect_clang().is_none() {
            return;
        }
        let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
"#;
        let sir = lower_to_sir(source).unwrap();
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let out = workspace_root.join("target/spanda-native/test-bin");
        let result = compile_native(
            &sir,
            &CompileNativeOptions {
                output: out.clone(),
                clang: detect_clang(),
                workspace_root: workspace_root.clone(),
                target_triple: None,
                hal_profile: None,
            },
        )
        .expect("compile-native");
        assert!(result.executable.is_file());
    }
}
