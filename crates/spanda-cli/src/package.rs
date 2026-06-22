//! package support for Spanda.
//!
use spanda_driver::{check_with_registry, compile_with_registry, run_tests_with_registry};
use spanda_modules::load_project_modules;
use spanda_package::{
    adapter_verify_ok, add_dependency, collect_source_files, find_project_root, init_package,
    load_official_packages_for_source, publish_package, registry_info, remove_dependency,
    resolve_dependencies, search_registry, search_registry_merged, validate_package,
    verify_adapter_package, ApplicationPermissions, DependencySpec, Lockfile, PackageManifest,
    ResolveOptions, LOCKFILE_FILENAME, MANIFEST_FILENAME,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

pub fn usage_package() {
    // Usage package.
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
    // let result = spanda_cli::package::usage_package();

    // Produce eprintln! as the result.
    eprintln!(
        "Package commands:\n\
           spanda init [name] [--description <text>]\n\
           spanda build [--project <dir>]\n\
           spanda check [--project <dir>]\n\
           spanda test [--project <dir>]\n\
           spanda add <package> [--version <ver>] [--path <dir>] [--git <url>]\n\
           spanda remove <package>\n\
           spanda install [--project <dir>]\n\
           spanda update [--project <dir>]\n\
           spanda publish [--project <dir>]\n\
           spanda verify-adapter [--project <dir>] [--import <path>] [--package <name>]\n\
           spanda registry search <query>\n"
    );
}

fn project_root_from_cwd() -> PathBuf {
    // Project root from cwd.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // PathBuf.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::package::project_root_from_cwd();

    // Compute cwd for the following logic.
    let cwd = env::current_dir().unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        process::exit(1);
    });
    find_project_root(&cwd).unwrap_or_else(|| {
        eprintln!("Error: no spanda.toml found (run from a project directory or use --project)");
        process::exit(1);
    })
}

fn load_project(root: &Path) -> PackageManifest {
    // Load project.
    //
    // Parameters:
    // - `root` — input value
    //
    // Returns:
    // PackageManifest.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::package::load_project(root);

    // Produce unwrap or else as the result.
    PackageManifest::load_from_dir(root).unwrap_or_else(|e| {
        eprintln!("Error loading manifest: {e}");
        process::exit(1);
    })
}

pub fn cmd_init(args: &[String]) {
    // Cmd init.
    //
    // Parameters:
    // - `args` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::package::cmd_init(args);

    // Create mutable name for accumulating results.
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut i = 0;

    // Repeat while i < args.len().
    while i < args.len() {
        // Match on as str and handle each case.
        match args[i].as_str() {
            "--description" => {
                i += 1;
                description = args.get(i).cloned();
            }
            other if !other.starts_with('-') && name.is_none() => name = Some(other.to_string()),
            other => {
                eprintln!("Unknown argument: {other}");
                usage_package();
                process::exit(1);
            }
        }
        i += 1;
    }
    let cwd = env::current_dir().unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        process::exit(1);
    });

    // Match on as deref and handle each case.
    match init_package(&cwd, name.as_deref(), description.as_deref()) {
        Ok(root) => {
            println!("✓ Initialized Spanda package at {}", root.display());
            println!("  Edit src/main.sd and run `spanda check`");
        }
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}

pub fn cmd_build(args: &[String]) {
    // Cmd build.
    //
    // Parameters:
    // - `args` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::package::cmd_build(args);

    // Compute root for the following logic.
    let root = parse_project_arg(args);
    let manifest = load_project(&root);
    let _ = run_install_inner(&root, &manifest, false);
    let registry = load_project_modules(&root).unwrap_or_else(|e| {
        eprintln!("Error loading modules: {e}");
        for d in e.diagnostics() {
            eprintln!("  [{}:{}] {}", d.line, d.column, d.message);
        }
        process::exit(1);
    });
    let files = collect_source_files(&root).unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        process::exit(1);
    });

    // Skip further work when files is empty.
    if files.is_empty() {
        eprintln!("Error: no .sd source files found");
        process::exit(1);
    }

    // Handle each file in the listing.
    for file in &files {
        let source = fs::read_to_string(file).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {e}", file.display());
            process::exit(1);
        });

        // Handle the error returned from compile.
        if let Err(e) = compile_with_registry(&source, &registry) {
            eprintln!("Build failed for {}: {e}", file.display());
            process::exit(1);
        }
    }
    println!(
        "✓ Built {} ({} source file(s))",
        manifest.package.name,
        files.len()
    );
}

pub fn cmd_check_project(args: &[String]) {
    // Cmd check project.
    //
    // Parameters:
    // - `args` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::package::cmd_check_project(args);

    // Compute root for the following logic.
    let root = parse_project_arg(args);
    let manifest = load_project(&root);
    validate_project(&root, &manifest);
    let registry = load_project_modules(&root).unwrap_or_else(|e| {
        eprintln!("Error loading modules: {e}");
        for d in e.diagnostics() {
            eprintln!("  [{}:{}] {}", d.line, d.column, d.message);
        }
        process::exit(1);
    });
    let files = collect_source_files(&root).unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        process::exit(1);
    });
    let mut failed = false;

    // Handle each file in the listing.
    for file in &files {
        let source = fs::read_to_string(file).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {e}", file.display());
            process::exit(1);
        });

        // Match on check and handle each case.
        match check_with_registry(&source, &registry) {
            Ok(()) => println!("✓ {} — no type errors", file.display()),
            Err(e) => {
                failed = true;
                eprintln!("Type errors in {}:", file.display());

                // Process each diagnostic.
                for d in e.diagnostics() {
                    eprintln!("  [{}:{}] {}", d.line, d.column, d.message);
                }
            }
        }
    }

    // Take this path when failed.
    if failed {
        process::exit(1);
    }

    // Skip further work when files is empty.
    if files.is_empty() {
        eprintln!("Warning: no .sd source files found");
    }
}

pub fn cmd_test(args: &[String]) {
    // Cmd test.
    //
    // Parameters:
    // - `args` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::package::cmd_test(args);

    let mut json = false;
    let mut compile_fail = false;
    let mut filter: Option<String> = None;
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    let mut project_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--compile-fail" => compile_fail = true,
            "--filter" => {
                i += 1;
                filter = args.get(i).cloned();
            }
            "--project" => project_mode = true,
            other if !other.starts_with('-') => {
                paths.push(std::path::PathBuf::from(other));
            }
            other => {
                eprintln!("Unknown test argument: {other}");
                process::exit(1);
            }
        }
        i += 1;
    }

    let start = std::time::Instant::now();
    let root = if project_mode || paths.is_empty() {
        parse_project_arg(args)
    } else {
        std::env::current_dir().unwrap_or_default()
    };

    let had_paths = !paths.is_empty();
    let test_files: Vec<std::path::PathBuf> = if paths.is_empty() {
        let test_dir = root.join("tests");
        if !test_dir.is_dir() {
            if json {
                println!(
                    r#"{{"passed":0,"failed":0,"skipped":0,"duration_ms":0,"tests":[]}}"#
                );
            } else {
                println!("✓ No tests directory — nothing to run");
            }
            return;
        }
        let files = collect_source_files(&root).unwrap_or_else(|e| {
            eprintln!("Error: {e}");
            process::exit(1);
        });
        files
            .into_iter()
            .filter(|f| f.starts_with(&test_dir))
            .collect()
    } else {
        paths
    };

    if test_files.is_empty() {
        if json {
            println!(r#"{{"passed":0,"failed":0,"skipped":0,"duration_ms":0,"tests":[]}}"#);
        } else {
            println!("✓ No test files found");
        }
        return;
    }

    let manifest = if project_mode || !had_paths {
        Some(load_project(&root))
    } else {
        None
    };
    if manifest.is_some() {
        validate_project(&root, manifest.as_ref().unwrap());
    }

    let registry = if project_mode || !had_paths {
        load_project_modules(&root).unwrap_or_else(|e| {
            eprintln!("Error loading modules: {e}");
            process::exit(1);
        })
    } else {
        spanda_typecheck::ModuleRegistry::new()
    };

    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;
    let mut entries = Vec::new();

    for file in test_files {
        let file_str = file.display().to_string();
        if file.extension().and_then(|e| e.to_str()) != Some("sd") {
            skipped += 1;
            continue;
        }
        let source = fs::read_to_string(&file).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {e}", file.display());
            process::exit(1);
        });

        if let Some(ref f) = filter {
            if !source.contains(f) && !file_str.contains(f) {
                skipped += 1;
                continue;
            }
        }

        let check_result = check_with_registry(&source, &registry);
        if compile_fail {
            if check_result.is_ok() {
                failed += 1;
                entries.push(serde_json::json!({
                    "file": file_str,
                    "status": "failed",
                    "message": "expected compile error but check passed"
                }));
                if !json {
                    eprintln!("✗ {file_str} — expected compile error");
                }
            } else {
                passed += 1;
                entries.push(serde_json::json!({
                    "file": file_str,
                    "status": "passed",
                    "kind": "compile-fail"
                }));
                if !json {
                    println!("✓ {file_str} — compile-fail test passed");
                }
            }
            continue;
        }

        if let Err(e) = check_result {
            failed += 1;
            entries.push(serde_json::json!({
                "file": file_str,
                "status": "failed",
                "message": format!("{e}")
            }));
            if !json {
                eprintln!("Test failed (type errors) {file_str}: {e}");
            }
            continue;
        }

        match run_tests_with_registry(&source, &registry) {
            Ok(result) if result.failed == 0 => {
                passed += result.passed.max(1);
                entries.push(serde_json::json!({
                    "file": file_str,
                    "status": "passed",
                    "tests": result.passed
                }));
                if !json {
                    println!("✓ {file_str} ({} test(s))", result.passed);
                }
            }
            Ok(result) => {
                failed += result.failed.max(1);
                entries.push(serde_json::json!({
                    "file": file_str,
                    "status": "failed",
                    "logs": result.logs
                }));
                if !json {
                    eprintln!("Test failed {file_str}:");
                    for log in result.logs {
                        eprintln!("  {log}");
                    }
                }
            }
            Err(e) => {
                failed += 1;
                entries.push(serde_json::json!({
                    "file": file_str,
                    "status": "failed",
                    "message": format!("{e}")
                }));
                if !json {
                    eprintln!("Test failed {file_str}: {e}");
                }
            }
        }
    }

    let duration_ms = start.elapsed().as_millis();
    if json {
        let report = serde_json::json!({
            "passed": passed,
            "failed": failed,
            "skipped": skipped,
            "duration_ms": duration_ms,
            "tests": entries
        });
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    }

    if failed > 0 {
        process::exit(1);
    }
}

pub fn cmd_add(args: &[String]) {
    // Cmd add.
    //
    // Parameters:
    // - `args` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::package::cmd_add(args);

    // skip further work when args is empty.
    if args.is_empty() {
        eprintln!("Usage: spanda add <package> [--version <ver>] [--path <dir>] [--git <url>]");
        process::exit(1);
    }
    let name = &args[0];
    let root = project_root_from_cwd();
    let mut version: Option<String> = None;
    let mut path: Option<String> = None;
    let mut git: Option<String> = None;
    let mut i = 1;

    // Repeat while i < args.len().
    while i < args.len() {
        // Match on as str and handle each case.
        match args[i].as_str() {
            "--version" => {
                i += 1;
                version = args.get(i).cloned();
            }
            "--path" => {
                i += 1;
                path = args.get(i).cloned();
            }
            "--git" => {
                i += 1;
                git = args.get(i).cloned();
            }
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
        i += 1;
    }
    let spec = if let Some(p) = path {
        DependencySpec::Detail(spanda_package::DependencyDetail {
            version: None,
            path: Some(PathBuf::from(p)),
            git: None,
            branch: None,
            tag: None,
            rev: None,
        })
    } else if let Some(g) = git {
        DependencySpec::Detail(spanda_package::DependencyDetail {
            version: version.clone(),
            path: None,
            git: Some(g),
            branch: None,
            tag: None,
            rev: None,
        })
    } else {
        DependencySpec::Version(version.unwrap_or_else(|| "0.1.0".into()))
    };
    add_dependency(&root, name, spec).unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        process::exit(1);
    });
    println!("✓ Added dependency '{name}' to {MANIFEST_FILENAME}");
}

pub fn cmd_remove(args: &[String]) {
    // Cmd remove.
    //
    // Parameters:
    // - `args` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::package::cmd_remove(args);

    // skip further work when args is empty.
    if args.is_empty() {
        eprintln!("Usage: spanda remove <package>");
        process::exit(1);
    }
    let name = &args[0];
    let root = project_root_from_cwd();

    // Match on remove dependency and handle each case.
    match remove_dependency(&root, name) {
        Ok(true) => println!("✓ Removed dependency '{name}'"),
        Ok(false) => {
            eprintln!("Dependency '{name}' not found");
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}

pub fn cmd_install(args: &[String]) {
    // Cmd install.
    //
    // Parameters:
    // - `args` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::package::cmd_install(args);

    // Compute root for the following logic.
    let root = parse_project_arg(args);
    let manifest = load_project(&root);
    run_install_inner(&root, &manifest, true);
}

pub fn cmd_update(args: &[String]) {
    // Cmd update — refresh spanda.lock and vendored packages to latest compatible versions.
    //
    // Parameters:
    // - `args` — optional `--project <dir>`
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::package::cmd_update(args);

    // Resolve project root and re-run dependency resolution.
    let root = parse_project_arg(args);
    let manifest = load_project(&root);
    println!("Updating dependencies for {}…", manifest.package.name);
    run_install_inner(&root, &manifest, true);
    println!(
        "✓ Updated {} (spanda.lock refreshed)",
        manifest.package.name
    );
}

fn run_install_inner(root: &Path, manifest: &PackageManifest, verbose: bool) -> Lockfile {
    // Run install inner.
    //
    // Parameters:
    // - `root` — input value
    // - `manifest` — input value
    // - `verbose` — input value
    //
    // Returns:
    // Lockfile.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::package::run_install_inner(root, manifest, verbose);

    // Produce validate project as the result.
    validate_project(root, manifest);
    let result =
        resolve_dependencies(root, manifest, &ResolveOptions::default()).unwrap_or_else(|e| {
            eprintln!("Error resolving dependencies: {e}");
            process::exit(1);
        });
    let lockfile = Lockfile::new(manifest, result.lockfile_deps);
    lockfile.save_to_dir(root).unwrap_or_else(|e| {
        eprintln!("Error writing lockfile: {e}");
        process::exit(1);
    });

    // Match on vendor dependencies and handle each case.
    match spanda_package::vendor_dependencies(root, &lockfile) {
        Ok(vendor) => {
            // Take this path when verbose.
            if verbose {
                // Handle each entry in vendored.
                for item in &vendor.vendored {
                    println!("  vendored {item}");
                }

                // Process each warning.
                for w in &vendor.warnings {
                    eprintln!("  ⚠ {w}");
                }
            }
        }
        Err(e) if verbose => eprintln!("  ⚠ vendor: {e}"),
        Err(_) => {}
    }

    // Take this path when verbose.
    if verbose {
        println!(
            "✓ Installed {} dependencies → {LOCKFILE_FILENAME}",
            lockfile.dependencies.len()
        );

        // Process each warning.
        for w in &result.warnings {
            eprintln!("  ⚠ {w}");
        }
        let official = spanda_package::official_packages_from_lockfile(&lockfile);
        if !official.is_empty() {
            println!("  official packages: {}", official.join(", "));
        }
    }
    lockfile
}

/// Resolve official lean-core package names for a source file path.
pub fn official_packages_for_source(source: &Path) -> Vec<String> {
    load_official_packages_for_source(source)
}

pub fn module_registry_for_source(source: &Path) -> Option<spanda_typecheck::ModuleRegistry> {
    // Load vendored project modules when the source file belongs to a package project.
    //
    // Parameters:
    // - `source` — path to a `.sd` source file
    //
    // Returns:
    // Module registry when a project root with modules is found.
    //
    // Options:
    // None.
    //
    // Example:
    // let registry = module_registry_for_source(Path::new("src/rover.sd"));

    let start = if source.is_file() {
        source.parent()?
    } else {
        source
    };
    let root = find_project_root(start)?;
    load_project_modules(&root).ok()
}

pub fn cmd_publish(args: &[String]) {
    // Cmd publish.
    //
    // Parameters:
    // - `args` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::package::cmd_publish(args);

    // Compute root for the following logic.
    let root = parse_project_arg(args);
    let manifest = load_project(&root);
    validate_project(&root, &manifest);
    let _ = run_install_inner(&root, &manifest, false);

    // Match on publish package and handle each case.
    match publish_package(&root, &manifest) {
        Ok(report) => {
            println!(
                "✓ Package '{}' v{} bundled at {}",
                manifest.package.name,
                manifest.package.version,
                report.bundle_path.display()
            );

            // Take this path when report.uploaded.
            if report.uploaded {
                println!(
                    "✓ Uploaded to {}",
                    report.upload_url.as_deref().unwrap_or("registry")
                );
            } else if std::env::var("SPANDA_REGISTRY_URL").is_ok() {
                println!("  (bundle kept locally — registry upload failed or skipped)");
            } else {
                println!("  Set SPANDA_REGISTRY_URL to upload the bundle remotely");
            }
        }
        Err(e) => {
            eprintln!("Error publishing: {e}");
            process::exit(1);
        }
    }
}

pub fn cmd_registry_search(args: &[String]) {
    // Cmd registry search.
    //
    // Parameters:
    // - `args` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::package::cmd_registry_search(args);

    // skip further work when args is empty.
    if args.is_empty() {
        eprintln!("Usage: spanda registry search <query>");
        process::exit(1);
    }
    let query = args.join(" ");
    let local = search_registry(&query);
    let merged = search_registry_merged(&query);

    // Skip further work when merged is empty.
    if merged.is_empty() {
        println!("No packages matching '{query}'");
        return;
    }

    // Iterate over each name in merged.
    for name in merged {
        // Emit output when registry info provides a info.
        if let Some(info) = registry_info(&name) {
            let version = info.versions.last().map(String::as_str).unwrap_or("?");
            println!(
                "{} v{} — {} [{}] safety={}",
                info.name, version, info.description, info.category, info.safety_level
            );
            continue;
        }

        // Emit output when name == name) provides a entry.
        if let Some(entry) = local.iter().find(|entry| entry.name == name) {
            println!(
                "{} v{} — {} [{}] safety={}",
                entry.name,
                entry.versions.last().unwrap_or(&"?"),
                entry.description,
                entry.category,
                entry.safety_level().as_str()
            );
        }
    }
}

pub fn cmd_registry_info(args: &[String]) {
    // Cmd registry info.
    //
    // Parameters:
    // - `args` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::package::cmd_registry_info(args);

    // skip further work when args is empty.
    if args.is_empty() {
        eprintln!("Usage: spanda registry info <package>");
        process::exit(1);
    }
    let name = &args[0];

    // Match on registry info and handle each case.
    match registry_info(name) {
        Some(info) => {
            println!("{}", serde_json::to_string_pretty(&info).unwrap());
        }
        None => {
            eprintln!("Package '{name}' not found in local or remote registry");
            process::exit(1);
        }
    }
}

fn validate_project(root: &Path, manifest: &PackageManifest) {
    // Validate project.
    //
    // Parameters:
    // - `root` — input value
    // - `manifest` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::package::validate_project(root, manifest);

    // Compute perms for the following logic.
    let perms = ApplicationPermissions::permissive();

    // Match on validate package and handle each case.
    match validate_package(manifest, &perms) {
        Ok(report) => {
            // Process each warning.
            for w in &report.warnings {
                eprintln!("  ⚠ {w}");
            }
        }
        Err(e) => {
            eprintln!("Package validation failed: {e}");
            process::exit(1);
        }
    }
    let _ = root;
}

fn parse_project_arg(args: &[String]) -> PathBuf {
    // Parse project arg.
    //
    // Parameters:
    // - `args` — input value
    //
    // Returns:
    // PathBuf.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::package::parse_project_arg(args);

    // Create mutable project for accumulating results.
    let mut project: Option<PathBuf> = None;
    let mut i = 0;

    // Repeat while i < args.len().
    while i < args.len() {
        // Take the branch when args[i] equals "--project".
        if args[i] == "--project" {
            i += 1;
            project = args.get(i).map(PathBuf::from);
        }
        i += 1;
    }
    project.unwrap_or_else(project_root_from_cwd)
}

pub fn cmd_verify_adapter(args: &[String]) {
    // Validate a project package adapter section against registry metadata.
    let root = parse_project_arg(args);
    let manifest = load_project(&root);
    let mut import_path: Option<String> = None;
    let mut package_name: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--import" if i + 1 < args.len() => {
                import_path = Some(args[i + 1].clone());
                i += 1;
            }
            "--package" if i + 1 < args.len() => {
                package_name = Some(args[i + 1].clone());
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }
    if import_path.is_none() && package_name.is_none() {
        import_path = Some("navigation.nav2".into());
    }
    let issues = verify_adapter_package(&manifest, import_path.as_deref(), package_name.as_deref())
        .unwrap_or_else(|e| {
            eprintln!("Adapter verify failed: {e}");
            process::exit(1);
        });
    for issue in &issues {
        let icon = match issue.severity {
            spanda_package::AdapterVerifySeverity::Pass => "✓",
            spanda_package::AdapterVerifySeverity::Warning => "⚠",
            spanda_package::AdapterVerifySeverity::Error => "✗",
        };
        println!("  {icon} {}", issue.message);
    }
    if !adapter_verify_ok(&issues) {
        process::exit(1);
    }
    println!(
        "✓ Adapter package verification passed for {}",
        manifest.package.name
    );
}
