use spanda_core::{check, load_project_modules, run_tests_with_registry};
use spanda_package::{
    add_dependency, collect_source_files, find_project_root, init_package, registry_info,
    remove_dependency, resolve_dependencies, search_registry, search_registry_merged,
    validate_package, ApplicationPermissions, DependencySpec, Lockfile, PackageManifest,
    ResolveOptions, LOCKFILE_FILENAME, MANIFEST_FILENAME,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

pub fn usage_package() {
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
    PackageManifest::load_from_dir(root).unwrap_or_else(|e| {
        eprintln!("Error loading manifest: {e}");
        process::exit(1);
    })
}

pub fn cmd_init(args: &[String]) {
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
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
    let root = parse_project_arg(args);
    let manifest = load_project(&root);
    let _ = run_install_inner(&root, &manifest, false);
    let files = collect_source_files(&root).unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        process::exit(1);
    });
    if files.is_empty() {
        eprintln!("Error: no .sd source files found");
        process::exit(1);
    }
    for file in &files {
        let source = fs::read_to_string(file).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {e}", file.display());
            process::exit(1);
        });
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
    let root = parse_project_arg(args);
    let manifest = load_project(&root);
    validate_project(&root, &manifest);
    let files = collect_source_files(&root).unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        process::exit(1);
    });
    let mut failed = false;
    for file in &files {
        let source = fs::read_to_string(file).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {e}", file.display());
            process::exit(1);
        });
        match check(&source) {
            Ok(()) => println!("✓ {} — no type errors", file.display()),
            Err(e) => {
                failed = true;
                eprintln!("Type errors in {}:", file.display());
                for d in e.diagnostics() {
                    eprintln!("  [{}:{}] {}", d.line, d.column, d.message);
                }
            }
        }
    }
    if failed {
        process::exit(1);
    }
    if files.is_empty() {
        eprintln!("Warning: no .sd source files found");
    }
}

pub fn cmd_test(args: &[String]) {
    let root = parse_project_arg(args);
    let manifest = load_project(&root);
    validate_project(&root, &manifest);
    let test_dir = root.join("tests");
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
    if test_files.is_empty() {
        println!("✓ No test files found");
        return;
    }
    let mut failed = false;
    for file in test_files {
        let source = fs::read_to_string(&file).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {e}", file.display());
            process::exit(1);
        });
        if let Err(e) = check(&source) {
            failed = true;
            eprintln!("Test failed (type errors) {}: {e}", file.display());
        } else {
            let registry = load_project_modules(&root).unwrap_or_default();
            match run_tests_with_registry(&source, &registry) {
                Ok(result) if result.failed == 0 => {
                    println!("✓ {} ({} test(s))", file.display(), result.passed);
                }
                Ok(result) => {
                    failed = true;
                    eprintln!("Test failed {}:", file.display());
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
    if failed {
        process::exit(1);
    }
}

pub fn cmd_add(args: &[String]) {
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
    while i < args.len() {
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
    if args.is_empty() {
        eprintln!("Usage: spanda remove <package>");
        process::exit(1);
    }
    let name = &args[0];
    let root = project_root_from_cwd();
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
    let root = parse_project_arg(args);
    let manifest = load_project(&root);
    run_install_inner(&root, &manifest, true);
}

fn run_install_inner(root: &Path, manifest: &PackageManifest, verbose: bool) -> Lockfile {
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
    match spanda_package::vendor_dependencies(root, &lockfile) {
        Ok(vendor) => {
            if verbose {
                for item in &vendor.vendored {
                    println!("  vendored {item}");
                }
                for w in &vendor.warnings {
                    eprintln!("  ⚠ {w}");
                }
            }
        }
        Err(e) if verbose => eprintln!("  ⚠ vendor: {e}"),
        Err(_) => {}
    }
    if verbose {
        println!(
            "✓ Installed {} dependencies → {LOCKFILE_FILENAME}",
            lockfile.dependencies.len()
        );
        for w in &result.warnings {
            eprintln!("  ⚠ {w}");
        }
    }
    lockfile
}

pub fn cmd_publish(args: &[String]) {
    let root = parse_project_arg(args);
    let manifest = load_project(&root);
    validate_project(&root, &manifest);
    let _ = run_install_inner(&root, &manifest, false);
    println!(
        "✓ Package '{}' v{} validated and ready for publish",
        manifest.package.name, manifest.package.version
    );
    println!("  (public registry not yet available — local validation only)");
}

pub fn cmd_registry_search(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: spanda registry search <query>");
        process::exit(1);
    }
    let query = args.join(" ");
    let local = search_registry(&query);
    let merged = search_registry_merged(&query);
    if merged.is_empty() {
        println!("No packages matching '{query}'");
        return;
    }
    for name in merged {
        if let Some(info) = registry_info(&name) {
            let version = info.versions.last().map(String::as_str).unwrap_or("?");
            println!(
                "{} v{} — {} [{}] safety={}",
                info.name, version, info.description, info.category, info.safety_level
            );
            continue;
        }
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
    if args.is_empty() {
        eprintln!("Usage: spanda registry info <package>");
        process::exit(1);
    }
    let name = &args[0];
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
    let perms = ApplicationPermissions::permissive();
    match validate_package(manifest, &perms) {
        Ok(report) => {
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
    let mut project: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--project" {
            i += 1;
            project = args.get(i).map(PathBuf::from);
        }
        i += 1;
    }
    project.unwrap_or_else(project_root_from_cwd)
}
