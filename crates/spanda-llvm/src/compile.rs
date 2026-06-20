//! Link LLVM IR with `libspanda_rt` via clang when available.

use spanda_core::sir::SirProgram;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::emit_module_ir;

#[derive(Debug, Clone)]
pub struct CompileNativeOptions {
    pub output: PathBuf,
    pub clang: Option<String>,
    pub workspace_root: PathBuf,
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
    let clang = opts
        .clang
        .clone()
        .or_else(detect_clang)
        .ok_or_else(|| "clang not found — install LLVM/clang to use compile-native".to_string())?;

    let ir = emit_module_ir(sir);
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

    if cfg!(target_os = "macos") {
        cmd.arg("-Wl,-no_warn_duplicate_libraries");
        cmd.args(["-framework", "Security", "-framework", "SystemConfiguration"]);
        cmd.arg("-liconv");
    }

    let status = cmd.status().map_err(|e| format!("failed to run clang: {e}"))?;
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

fn detect_clang() -> Option<String> {
    for candidate in ["clang", "clang-18", "clang-17", "clang-16"] {
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
    std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root.join("target"))
}

fn ensure_spanda_rt_staticlib(workspace_root: &Path) -> Result<PathBuf, String> {
    let target_dir = resolve_target_dir(workspace_root);
    let profile = "debug";
    let lib_path = target_dir.join(profile).join("libspanda_rt.a");
    if lib_path.is_file() {
        return Ok(lib_path);
    }

    let mut cmd = Command::new("cargo");
    cmd.current_dir(workspace_root)
        .args(["build", "-p", "spanda-rt"]);
    if let Ok(dir) = std::env::var("CARGO_TARGET_DIR") {
        cmd.env("CARGO_TARGET_DIR", dir);
    }
    let status = cmd.status().map_err(|e| format!("failed to build spanda-rt: {e}"))?;
    if !status.success() {
        return Err("cargo build -p spanda-rt failed".into());
    }
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
            },
        )
        .expect("compile-native");
        assert!(result.executable.is_file());
    }
}
