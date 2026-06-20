//! package support for Spanda.
//!
use spanda_core::{check, load_project_modules, run_tests_with_registry};
use spanda_package::{
    add_dependency, collect_source_files, find_project_root, init_package, publish_package,
    registry_info, remove_dependency, resolve_dependencies, search_registry,
    search_registry_merged, validate_package, ApplicationPermissions, DependencySpec, Lockfile,
    PackageManifest, ResolveOptions, LOCKFILE_FILENAME, MANIFEST_FILENAME,
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
           spanda publish [--project <dir>]\n\
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
        if let Err(e) = spanda_core::compile(&source) {
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
        match check(&source) {
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

    // Compute root for the following logic.
    let root = parse_project_arg(args);
    let manifest = load_project(&root);
    validate_project(&root, &manifest);
    let test_dir = root.join("tests");

    // Treat the path as a directory and scan its contents.
    if !test_dir.is_dir() {
        println!("✓ No tests directory — nothing to run");
        return;
    }
    let files = collect_source_files(&root).unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        process::exit(1);
    });
    let test_files: Vec<_> = files
        .into_iter()
        .filter(|f| f.starts_with(&test_dir))
        .collect();

    // Skip further work when test files is empty.
    if test_files.is_empty() {
        println!("✓ No test files found");
        return;
    }
    let mut failed = false;

    // Handle each file in the listing.
    for file in test_files {
        let source = fs::read_to_string(&file).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {e}", file.display());
            process::exit(1);
        });

        // Handle the error returned from check.
        if let Err(e) = check(&source) {
            failed = true;
            eprintln!("Test failed (type errors) {}: {e}", file.display());
        } else {
            let registry = load_project_modules(&root).unwrap_or_default();

            // Match on run tests with registry and handle each case.
            match run_tests_with_registry(&source, &registry) {
                Ok(result) if result.failed == 0 => {
                    println!("✓ {} ({} test(s))", file.display(), result.passed);
                }
                Ok(result) => {
                    failed = true;
                    eprintln!("Test failed {}:", file.display());

                    // Process each log.
                    for log in result.logs {
                        eprintln!("  {log}");
                    }
                }
                Err(e) => {
                    failed = true;
                    eprintln!("Test failed {}: {e}", file.display());
                }
            }
        }
    }

    // Take this path when failed.
    if failed {
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
    }
    lockfile
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
