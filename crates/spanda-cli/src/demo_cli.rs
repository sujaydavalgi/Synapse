//! One-command showcase demos for evaluators (`spanda demo <name>`).
//!
use spanda_driver::{check, run, verify_compatibility_with_registry, RunOptions};
use spanda_hardware::VerifyOptions;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

fn bundled_examples_root() -> Option<PathBuf> {
    // Description:
    //     Bundled examples root.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: Option<PathBuf>
    //         Return value from `bundled_examples_root`.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::bundled_examples_root();

    let bundled = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bundled-examples");
    if bundled.join("examples/showcase/README.md").is_file() {
        Some(bundled)
    } else {
        None
    }
}

fn repo_root() -> PathBuf {
    // Description:
    //     Repo root.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: PathBuf
    //         Return value from `repo_root`.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::repo_root();

    if let Some(bundled) = bundled_examples_root() {
        return bundled;
    }

    if let Ok(root) = env::var("SPANDA_ROOT") {
        let path = PathBuf::from(root);
        if path.join("examples/showcase/README.md").is_file() {
            return path;
        }
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd);
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.to_path_buf());
            if let Some(grand) = parent.parent() {
                candidates.push(grand.to_path_buf());
            }
        }
    }

    for start in candidates {
        let mut dir = start;
        for _ in 0..8 {
            if dir.join("examples/showcase/README.md").is_file() {
                return dir;
            }
            if !dir.pop() {
                break;
            }
        }
    }

    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn showcase(root: &Path, parts: &[&str]) -> PathBuf {
    // Description:
    //     Showcase.
    //
    // Inputs:
    //     roo: &Path
    //         Caller-supplied roo.
    //     parts: &[&str]
    //         Caller-supplied parts.
    //
    // Outputs:
    //     result: PathBuf
    //         Return value from `showcase`.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::showcase(roo, parts);

    let mut path = root.join("examples/showcase");
    for part in parts {
        path.push(part);
    }
    path
}

fn require_file(path: &Path) -> &Path {
    // Description:
    //     Require file.
    //
    // Inputs:
    //     path: &Path
    //         Caller-supplied path.
    //
    // Outputs:
    //     result: &Path
    //         Return value from `require_file`.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::require_file(path);

    if !path.is_file() {
        eprintln!(
            "Demo file not found: {}\n\
             Clone the Spanda repository or set SPANDA_ROOT to the repo root.",
            path.display()
        );
        process::exit(1);
    }
    path
}

fn repo_root_containing_showcase(parts: &[&str]) -> PathBuf {
    // Resolve repository root that contains a showcase file (trust demos may be outside bundled examples).
    //
    // Parameters:
    // - `parts` — path segments under examples/showcase/
    //
    // Returns:
    // Repository root containing the showcase file.
    //
    // Options:
    // `SPANDA_ROOT` when it contains the showcase path.
    //
    // Example:
    // let root = repo_root_containing_showcase(&["package_tampering", "approved.sd"]);

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(root) = env::var("SPANDA_ROOT") {
        candidates.push(PathBuf::from(root));
    }
    if let Ok(cwd) = env::current_dir() {
        let mut dir = cwd;
        for _ in 0..8 {
            candidates.push(dir.clone());
            if !dir.pop() {
                break;
            }
        }
    }
    candidates.push(repo_root());
    for root in candidates {
        if showcase(&root, parts).is_file() {
            return root;
        }
    }
    eprintln!(
        "Showcase file not found: examples/showcase/{}\n\
         Set SPANDA_ROOT to the Spanda repository root.",
        parts.join("/")
    );
    process::exit(1);
}

fn read_source(path: &Path) -> String {
    // Description:
    //     Read source.
    //
    // Inputs:
    //     path: &Path
    //         Caller-supplied path.
    //
    // Outputs:
    //     result: String
    //         Return value from `read_source`.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::read_source(path);

    std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {e}", path.display());
        process::exit(1);
    })
}

fn spanda_bin() -> String {
    // Description:
    //     Spanda bin.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: String
    //         Return value from `spanda_bin`.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::spanda_bin();

    if let Ok(bin) = env::var("SPANDA_BIN") {
        return bin;
    }
    if let Ok(exe) = env::current_exe() {
        return exe.to_string_lossy().into_owned();
    }
    "spanda".into()
}

fn run_spanda(subcommand: &str, file: &Path, extra: &[&str]) {
    // Description:
    //     Run spanda.
    //
    // Inputs:
    //     subcommand: &str
    //         Caller-supplied subcommand.
    //     file: &Path
    //         Caller-supplied file.
    //     extra: &[&str]
    //         Caller-supplied extra.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::run_spanda(subcommand, file, extra);

    let spanda = spanda_bin();
    let mut cmd = Command::new(&spanda);
    cmd.arg(subcommand).arg(file);
    for flag in extra {
        cmd.arg(flag);
    }
    let status = cmd.status().unwrap_or_else(|e| {
        eprintln!("Failed to run {spanda}: {e}");
        process::exit(1);
    });
    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }
}

fn run_spanda_args(args: &[&str]) {
    // Description:
    //     Run spanda args.
    //
    // Inputs:
    //     args: &[&str]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::run_spanda_args(args);

    let spanda = spanda_bin();
    let status = Command::new(&spanda)
        .args(args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Failed to run {spanda}: {e}");
            process::exit(1);
        });
    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }
}

fn run_spanda_args_allow_fail(args: &[&str]) {
    // Description:
    //     Run spanda args allow fail.
    //
    // Inputs:
    //     args: &[&str]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::run_spanda_args_allow_fail(args);

    let spanda = spanda_bin();
    let _ = Command::new(&spanda).args(args).status();
}

fn expect_check_fail(file: &Path) {
    // Description:
    //     Expect check fail.
    //
    // Inputs:
    //     file: &Path
    //         Caller-supplied file.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::expect_check_fail(file);

    let source = read_source(file);
    if check(&source).is_ok() {
        eprintln!(
            "Expected compile error for {} (unsafe path should be rejected)",
            file.display()
        );
        process::exit(1);
    }
    println!(
        "✓ {} — compile-time safety gate rejected unsafe code",
        file.display()
    );
}

fn demo_rover(root: &Path) {
    // Description:
    //     Demo rover.
    //
    // Inputs:
    //     roo: &Path
    //         Caller-supplied roo.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::demo_rover(roo);

    let rover_dir = root.join("examples/showcase/autonomous_rover");
    let rover_path = rover_dir.join("src/rover.sd");
    let rover_sd = require_file(&rover_path);
    let trace = rover_dir.join("src/rover.trace");

    println!("== Autonomous Rover — flagship platform demo ==\n");

    if rover_dir.join("spanda.toml").is_file() {
        println!("→ Installing packages…");
        let spanda = env::var("SPANDA_BIN").unwrap_or_else(|_| "spanda".into());
        let status = Command::new(&spanda)
            .arg("install")
            .current_dir(&rover_dir)
            .status()
            .unwrap_or_else(|e| {
                eprintln!("Failed to run {spanda} install: {e}");
                process::exit(1);
            });
        if !status.success() {
            process::exit(status.code().unwrap_or(1));
        }
    }

    println!("→ Verify hardware fit");
    run_spanda("verify", rover_sd, &["--json", "--target", "RoverV1"]);

    println!("→ Simulate patrol");
    run_spanda("sim", rover_sd, &[]);

    if trace.is_file() {
        println!("→ Replay recorded mission");
        run_spanda("replay", &trace, &[]);
    }

    println!("\nDemo complete. See examples/showcase/autonomous_rover/README.md");
}

fn demo_safety(root: &Path) {
    // Description:
    //     Demo safety.
    //
    // Inputs:
    //     roo: &Path
    //         Caller-supplied roo.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::demo_safety(roo);

    let unsafe_path = showcase(root, &["unsafe_ai", "unsafe.sd"]);
    let safe_path = showcase(root, &["unsafe_ai", "safe.sd"]);
    let unsafe_sd = require_file(&unsafe_path);
    let safe_sd = require_file(&safe_path);

    println!("== Safety — ActionProposal vs SafeAction ==\n");
    println!("→ Unsafe: ActionProposal sent directly to actuator (must fail)");
    expect_check_fail(unsafe_sd);

    println!("→ Safe: safety.validate() gate (must pass)");
    run_spanda("check", safe_sd, &[]);

    println!("\nDemo complete. See examples/showcase/unsafe_ai/README.md");
}

fn demo_verify(root: &Path) {
    // Description:
    //     Demo verify.
    //
    // Inputs:
    //     roo: &Path
    //         Caller-supplied roo.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::demo_verify(roo);

    let fail_path = showcase(root, &["hardware_verification", "mission_missing_lidar.sd"]);
    let pass_path = showcase(root, &["hardware_verification", "mission_with_lidar.sd"]);
    let fail_sd = require_file(&fail_path);
    let pass_sd = require_file(&pass_path);

    println!("== Hardware verification — obstacle avoidance needs Lidar ==\n");

    println!("→ Mission on robot without Lidar (must fail)");
    let source = read_source(fail_sd);
    let options = VerifyOptions::default();
    let report = verify_compatibility_with_registry(&source, &options, None).unwrap_or_else(|e| {
        eprintln!("Verify failed: {e}");
        process::exit(1);
    });
    if report.compatible {
        eprintln!("Expected verification failure for {}", fail_sd.display());
        process::exit(1);
    }
    println!("✓ Verification failed as expected — missing Lidar sensor");

    println!("→ Same mission on robot with Lidar (must pass)");
    run_spanda("verify", pass_sd, &["--json", "--target", "RoverV1"]);

    println!("\nDemo complete. See examples/showcase/hardware_verification/README.md");
}

fn demo_fleet(root: &Path) {
    // Description:
    //     Demo fleet.
    //
    // Inputs:
    //     roo: &Path
    //         Caller-supplied roo.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::demo_fleet(roo);

    let fleet_path = showcase(root, &["fleet_management", "fleet.sd"]);
    let fleet_sd = require_file(&fleet_path);

    println!("== Fleet — multi-robot coordination ==\n");
    run_spanda("check", fleet_sd, &[]);
    run_spanda_args(&["fleet", "run", fleet_sd.to_str().unwrap()]);

    println!("\nDemo complete. See examples/showcase/fleet_management/README.md");
}

fn demo_health(root: &Path) {
    // Description:
    //     Demo health.
    //
    // Inputs:
    //     roo: &Path
    //         Caller-supplied roo.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::demo_health(roo);

    let health_path = showcase(root, &["health_monitoring", "rover.sd"]);
    let health_sd = require_file(&health_path);

    println!("== Health monitoring — fault injection ==\n");
    run_spanda("check", health_sd, &[]);
    run_spanda_args(&["health", "robot", health_sd.to_str().unwrap(), "--json"]);

    println!("→ Simulate with injected health faults");
    let source = read_source(health_sd);
    let opts = RunOptions {
        inject_health_faults: true,
        ..Default::default()
    };
    run(&source, opts).unwrap_or_else(|e| {
        eprintln!("Simulation failed: {e}");
        process::exit(1);
    });
    println!("✓ Simulation completed with --inject-health-faults");

    println!("\nDemo complete. See examples/showcase/health_monitoring/README.md");
}

fn demo_readiness(root: &Path) {
    // Description:
    //     Demo readiness.
    //
    // Inputs:
    //     roo: &Path
    //         Caller-supplied roo.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::demo_readiness(roo);

    let readiness_path = showcase(root, &["readiness", "rover.sd"]);
    let readiness_sd = require_file(&readiness_path);

    println!("== Operational readiness — deploy-target scoring ==\n");
    run_spanda("check", readiness_sd, &[]);
    run_spanda_args(&[
        "readiness",
        readiness_sd.to_str().unwrap(),
        "--target",
        "RoverV1",
        "--json",
    ]);

    println!("→ Runtime readiness with injected health faults");
    run_spanda_args_allow_fail(&[
        "readiness",
        readiness_sd.to_str().unwrap(),
        "--target",
        "RoverV1",
        "--runtime",
        "--inject-health-faults",
    ]);
    println!("(non-ready score is expected when health faults are injected)");

    println!("\nDemo complete. See examples/showcase/readiness/rover.sd and docs/readiness.md");
}

fn demo_self_healing(root: &Path) {
    // Description:
    //     Demo self healing.
    //
    // Inputs:
    //     roo: &Path
    //         Caller-supplied roo.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::demo_self_healing(roo);

    let healing_path = showcase(root, &["self_healing", "rover.sd"]);
    let fleet_path = showcase(root, &["fleet_recovery", "fleet.sd"]);
    let healing_sd = require_file(&healing_path);
    let fleet_sd = require_file(&fleet_path);
    let heal_file = healing_sd.to_str().unwrap();
    let fleet_file = fleet_sd.to_str().unwrap();

    println!("== Self-healing — recovery policies, validation, fleet mesh ==\n");
    run_spanda("check", healing_sd, &[]);
    run_spanda_args(&["heal", heal_file]);
    run_spanda_args(&["recover", heal_file, "--failure", "gps"]);
    run_spanda_args(&["recovery", "knowledge", heal_file]);
    run_spanda_args(&["sim", heal_file, "--inject-failure", "gps"]);
    run_spanda_args(&["analyze-failure", heal_file, "--with-recovery"]);
    run_spanda_args(&["heal", fleet_file]);

    println!("\nDemo complete. See examples/showcase/self_healing/ and docs/self-healing.md");
}

fn demo_continuity(root: &Path) {
    let continuity_path = showcase(root, &["continuity", "warehouse.sd"]);
    let fleet_path = showcase(root, &["fleet_succession", "delivery.sd"]);
    let continuity_sd = require_file(&continuity_path);
    let fleet_sd = require_file(&fleet_path);
    let warehouse = continuity_sd.to_str().unwrap();
    let delivery = fleet_sd.to_str().unwrap();

    println!("== Mission continuity — takeover, delegation, succession ==\n");
    run_spanda("check", continuity_sd, &[]);
    run_spanda_args(&[
        "continuity",
        warehouse,
        "--failed",
        "ScannerAlpha",
        "--progress",
        "72",
        "--trigger",
        "robot_failed",
    ]);
    run_spanda_args(&[
        "takeover",
        warehouse,
        "--failed",
        "ScannerAlpha",
        "--successor",
        "ScannerBeta",
        "--progress",
        "72",
    ]);
    run_spanda_args(&[
        "delegate",
        warehouse,
        "--failed",
        "ScannerAlpha",
        "--to",
        "ScannerBeta",
        "--progress",
        "60",
    ]);
    run_spanda_args(&[
        "succession",
        delivery,
        "--failed",
        "CourierA",
        "--scope",
        "fleet",
    ]);

    println!("\nDemo complete. See examples/showcase/continuity/ and docs/mission-continuity.md");
}

fn demo_assurance(root: &Path) {
    // Description:
    //     Demo assurance.
    //
    // Inputs:
    //     roo: &Path
    //         Caller-supplied roo.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::demo_assurance(roo);

    let assurance_path = showcase(root, &["assurance", "rover.sd"]);
    let assurance_sd = require_file(&assurance_path);
    let file = assurance_sd.to_str().unwrap();

    println!("== Mission assurance — knowledge, state, anomaly, resilience ==\n");
    run_spanda("check", assurance_sd, &[]);
    run_spanda_args(&["assure", file, "--json"]);
    run_spanda_args(&["anomaly", "scan", file]);
    run_spanda_args(&["state", "estimate", file]);
    run_spanda_args(&["prognostics", file]);
    run_spanda_args(&["mission", "verify", file]);
    run_spanda_args(&["resilience", "check", file]);
    run_spanda_args(&["mitigation", "plan", file]);
    run_spanda_args(&["readiness", file, "--target", "RoverV1", "--json"]);

    println!("\nDemo complete. See examples/showcase/assurance/README.md and examples/anomaly/learned_navigation.sd");
}

fn demo_differentiation(root: &Path) {
    let path = showcase(root, &["differentiation", "warehouse.sd"]);
    let file = require_file(&path);
    let sd = file.to_str().unwrap();

    println!("== Differentiation NOW — contracts, coverage, explainability ==\n");
    run_spanda("check", file, &[]);
    run_spanda_args(&["contract", "verify", sd]);
    run_spanda_args(&["safety-coverage", sd]);
    run_spanda_args(&["recovery-coverage", sd]);
    run_spanda_args(&["explain", sd]);
    run_spanda_args(&["explain", "readiness", "--file", sd]);

    println!("\nDemo complete. See examples/showcase/differentiation/ and docs/differentiation-roadmap.md");
}

fn demo_maturity(root: &Path) {
    let path = showcase(root, &["readiness", "rover.sd"]);
    let file = require_file(&path);
    let sd = file.to_str().unwrap();

    println!("== Platform maturity Phase A — graph, explain, trust, gates ==\n");
    run_spanda("check", file, &[]);
    run_spanda_args(&["graph", sd, "--format", "text"]);
    run_spanda_args(&["explain", sd]);
    run_spanda_args(&["trust", "spanda-mqtt"]);
    run_spanda_args(&["trust", sd]);
    run_spanda_args_allow_fail(&["deploy", "gate", sd]);

    println!(
        "\nDemo complete. See docs/platform-maturity-roadmap.md and docs/dependency-graphs.md"
    );
}

fn demo_trust(root: &Path) {
    let trust_root = repo_root_containing_showcase(&["package_tampering", "approved.sd"]);
    let _ = root;
    crate::bundled_registry::ensure_bundled_registry_env();

    let pkg_approved = showcase(&trust_root, &["package_tampering", "approved.sd"]);
    let pkg_tampered = showcase(&trust_root, &["package_tampering", "tampered.sd"]);
    let mission_approved = showcase(&trust_root, &["mission_tampering", "approved.sd"]);
    let mission_modified = showcase(&trust_root, &["mission_tampering", "modified.sd"]);
    let intrusion_trace = showcase(&trust_root, &["runtime_intrusion", "intrusion.trace"]);
    let gps_rover = showcase(&trust_root, &["gps_spoofing", "rover.sd"]);
    let gps_trace = showcase(&trust_root, &["gps_spoofing", "spoof.trace"]);
    let tamper_policy = showcase(&trust_root, &["tamper_policy", "rover.sd"]);

    require_file(&pkg_approved);
    require_file(&pkg_tampered);
    require_file(&mission_approved);
    require_file(&mission_modified);
    require_file(&intrusion_trace);
    require_file(&gps_rover);
    require_file(&gps_trace);
    require_file(&tamper_policy);

    let approved = pkg_approved.to_str().unwrap();
    let tampered = pkg_tampered.to_str().unwrap();
    let mission_ok = mission_approved.to_str().unwrap();
    let mission_bad = mission_modified.to_str().unwrap();
    let intrusion = intrusion_trace.to_str().unwrap();
    let gps_program = gps_rover.to_str().unwrap();
    let gps_tr = gps_trace.to_str().unwrap();
    let policy = tamper_policy.to_str().unwrap();

    println!("== Trust & tamper platform demo ==\n");

    println!("--- Package tampering ---");
    run_spanda_args(&["tamper-check", approved]);
    run_spanda_args_allow_fail(&["tamper-check", tampered]);

    println!("\n--- Mission integrity ---");
    run_spanda_args(&["integrity", mission_ok, "--baseline", mission_ok]);
    run_spanda_args_allow_fail(&["integrity", mission_bad, "--baseline", mission_ok]);

    println!("\n--- Runtime intrusion trace ---");
    run_spanda_args_allow_fail(&["tamper-check", intrusion]);
    run_spanda_args_allow_fail(&["diagnose", "tamper", intrusion]);

    println!("\n--- GPS spoofing ---");
    run_spanda_args(&["spoof-check", gps_program]);
    run_spanda_args_allow_fail(&["spoof-check", gps_tr]);

    println!("\n--- Tamper policy runtime ---");
    run_spanda_args_allow_fail(&["tamper-check", policy]);
    run_spanda_args_allow_fail(&["sim", policy, "--inject-security-faults"]);

    let secure_boot = showcase(&trust_root, &["secure_boot", "rover.sd"]);
    if secure_boot.is_file() {
        println!("\n--- Secure boot contracts ---");
        run_spanda_args_allow_fail(&["tamper-check", secure_boot.to_str().unwrap()]);
    }

    println!("\nDemo complete. See examples/showcase/*/README.md and docs/tamper-detection.md");
}

fn demo_spoof(root: &Path) {
    let spoof_root = repo_root_containing_showcase(&["gps_spoofing", "rover.sd"]);
    let _ = root;
    crate::bundled_registry::ensure_bundled_registry_env();

    let gps_rover = showcase(&spoof_root, &["gps_spoofing", "rover.sd"]);
    let gps_trace = showcase(&spoof_root, &["gps_spoofing", "spoof.trace"]);
    require_file(&gps_rover);
    require_file(&gps_trace);

    let gps_program = gps_rover.to_str().unwrap();
    let gps_tr = gps_trace.to_str().unwrap();

    println!("== GPS spoofing detection demo ==\n");

    println!("--- Program coverage (expect PASS) ---");
    run_spanda_args(&["spoof-check", gps_program]);

    println!("\n--- Trace plausibility (expect FAIL) ---");
    run_spanda_args_allow_fail(&["spoof-check", gps_tr]);

    println!("\n--- Tamper diagnosis ---");
    run_spanda_args_allow_fail(&["diagnose", "tamper", gps_tr]);

    println!("\n--- Mock ML backend merge ---");
    env::set_var("SPANDA_SPOOFING_ML_BACKEND", "mock");
    run_spanda_args_allow_fail(&["spoof-check", gps_tr]);
    env::remove_var("SPANDA_SPOOFING_ML_BACKEND");

    println!("\nDemo complete. See examples/showcase/gps_spoofing/README.md and docs/spoofing-detection.md");
}

fn demo_gaps(root: &Path) {
    let gaps_root = repo_root_containing_showcase(&["secure_boot", "rover.sd"]);
    let _ = root;
    crate::bundled_registry::ensure_bundled_registry_env();

    let secure_boot = showcase(&gaps_root, &["secure_boot", "rover.sd"]);
    let defense = showcase(&gaps_root, &["compliance", "defense_rover.sd"]);
    let gps_trace = showcase(&gaps_root, &["gps_spoofing", "spoof.trace"]);
    require_file(&secure_boot);
    require_file(&defense);
    require_file(&gps_trace);

    let sb = secure_boot.to_str().unwrap();
    let defense_sd = defense.to_str().unwrap();
    let trace = gps_trace.to_str().unwrap();

    let vendor_jetson = showcase(
        &gaps_root,
        &["secure_boot", "fixtures", "jetson-tpm-vendor.sh"],
    );
    let vendor_ak = showcase(
        &gaps_root,
        &["secure_boot", "fixtures", "vendor-ak-chain.sh"],
    );
    let trust_store = showcase(&gaps_root, &["secure_boot", "fixtures", "trust-store"]);

    println!("== Platform maturity gap closure demo ==\n");

    println!("--- Vendor TPM SDK adapter ---");
    env::set_var("SPANDA_TPM_BACKEND", "vendor");
    if vendor_jetson.is_file() {
        env::set_var(
            "SPANDA_TPM_VENDOR_SDK",
            vendor_jetson.to_string_lossy().to_string(),
        );
    }
    run_spanda_args_allow_fail(&["tamper-check", sb]);
    env::remove_var("SPANDA_TPM_BACKEND");
    env::remove_var("SPANDA_TPM_VENDOR_SDK");

    println!("\n--- Remote AK cert chain validation ---");
    env::set_var("SPANDA_TPM_BACKEND", "vendor");
    if vendor_ak.is_file() {
        env::set_var(
            "SPANDA_TPM_VENDOR_SDK",
            vendor_ak.to_string_lossy().to_string(),
        );
    }
    if trust_store.is_dir() {
        env::set_var(
            "SPANDA_ATTESTATION_TRUST_STORE",
            trust_store.to_string_lossy().to_string(),
        );
    }
    run_spanda_args_allow_fail(&["tamper-check", sb]);
    env::remove_var("SPANDA_TPM_BACKEND");
    env::remove_var("SPANDA_TPM_VENDOR_SDK");
    env::remove_var("SPANDA_ATTESTATION_TRUST_STORE");

    println!("\n--- Compliance accreditation export ---");
    run_spanda_args(&["compliance", "report", defense_sd, "--profile", "defense"]);

    println!("\n--- Spoofing confidence filter ---");
    env::set_var("SPANDA_SPOOFING_MIN_CONFIDENCE", "0.99");
    run_spanda_args_allow_fail(&["spoof-check", trace]);
    env::remove_var("SPANDA_SPOOFING_MIN_CONFIDENCE");

    println!("\n--- Operator confirmation gate ---");
    run_spanda_args_allow_fail(&["spoof-check", trace]);

    println!("\nDemo complete. See docs/platform-maturity-roadmap.md and scripts/gaps_smoke.sh");
}

fn demo_compliance(root: &Path) {
    crate::bundled_registry::ensure_bundled_registry_env();

    let showcases = [
        ("defense_rover.sd", "defense"),
        ("medical_rover.sd", "medical"),
        ("automotive_rover.sd", "iso26262"),
        ("machinery_rover.sd", "iso13849"),
        ("iec61508_rover.sd", "iec61508"),
    ];

    println!("== Compliance profile showcases ==\n");
    run_spanda_args(&["compliance", "list"]);

    let mut defense_sd = None;
    for (file, profile) in showcases {
        let path = showcase(root, &["compliance", file]);
        let sd = require_file(&path);
        if profile == "defense" {
            defense_sd = Some(sd.to_str().unwrap().to_string());
        }
        println!("\n--- verify {profile} ({file}) ---");
        run_spanda_args(&["verify", sd.to_str().unwrap(), "--profile", profile]);
    }

    if let Some(defense) = defense_sd.as_deref() {
        println!("\n--- accreditation export (defense) ---");
        run_spanda_args(&["compliance", "report", defense, "--profile", "defense"]);
    }

    println!("\nDemo complete. See examples/showcase/compliance/README.md and docs/compliance-profiles.md");
}

pub fn demo_dispatch(args: &[String]) {
    // Description:
    //     Demo dispatch.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::demo_cli::demo_dispatch(args);

    let root = repo_root();
    let name = args.first().map(String::as_str).unwrap_or("");

    match name {
        "rover" => demo_rover(&root),
        "safety" => demo_safety(&root),
        "verify" => demo_verify(&root),
        "fleet" => demo_fleet(&root),
        "health" => demo_health(&root),
        "readiness" => demo_readiness(&root),
        "assurance" => demo_assurance(&root),
        "self-healing" | "selfhealing" | "healing" => demo_self_healing(&root),
        "continuity" | "takeover" | "succession" => demo_continuity(&root),
        "differentiation" | "diff" => demo_differentiation(&root),
        "maturity" | "platform-maturity" => demo_maturity(&root),
        "trust" | "tamper" | "security-trust" => demo_trust(&root),
        "spoof" | "spoofing" | "gps-spoofing" => demo_spoof(&root),
        "gaps" | "platform-gaps" | "maturity-gaps" => demo_gaps(&root),
        "compliance" | "profiles" => demo_compliance(&root),
        "" | "list" | "--help" | "-h" => {
            eprintln!(
                "Spanda showcase demos\n\n\
                 Usage:\n\
                   spanda demo <name>\n\n\
                 Demos:\n\
                   rover   — GPS, packages, verify, sim, replay (flagship)\n\
                   safety  — ActionProposal blocked; SafeAction passes\n\
                   verify  — hardware fit: missing Lidar fails, added Lidar passes\n\
                   fleet   — multi-robot fleet simulation\n\
                   health    — health checks with fault injection\n\
                   readiness — operational go/no-go with runtime health\n\
                   assurance — mission assurance CLI suite (assure, anomaly, state)\n\
                   self-healing — recovery policies, heal/recover/sim, fleet recovery\n\
                   continuity — mission continuity, takeover, delegation, succession\n\
                   differentiation — mission contracts, safety/recovery coverage, explain\n\
                   maturity — Phase A graph, explain, trust, deployment gates\n\
                   trust — package/mission tampering, spoofing, runtime intrusion, tamper_policy\n\
                   spoof — GPS spoof-check coverage, trace alerts, mock ML merge\n\
                   compliance — industry profile verification (defense, medical, ISO 26262, ISO 13849, IEC 61508)\n\
                   gaps — vendor TPM, AK chain, compliance export, confidence gates\n\n\
                 Set SPANDA_ROOT to the repository root if examples are not found.\n\
                 See examples/showcase/README.md"
            );
            if name.is_empty() {
                process::exit(1);
            }
        }
        other => {
            eprintln!("Unknown demo '{other}'. Run: spanda demo --help");
            process::exit(1);
        }
    }
}
