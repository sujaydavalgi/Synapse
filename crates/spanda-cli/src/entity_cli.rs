//! CLI for unified entity model — list, inspect, graph, verify, and query.
//!
use spanda_config::{
    build_entity_registry, config_flag_from_args, ConfigResolver, EntityQuery, SpandaManifest,
};
use spanda_readiness::{
    evaluate_entity_health, evaluate_entity_readiness, verify_entity, EntityHealthOptions,
    EntityReadinessOptions, EntityVerifyOptions,
};
use spanda_trust::{evaluate_entity_trust, EntityTrustOptions};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn project_root(args: &[String]) -> PathBuf {
    if let Some(flag) = config_flag_from_args(args) {
        return flag.parent().unwrap_or(&flag).to_path_buf();
    }
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    SpandaManifest::find_project_root(&cwd).unwrap_or(cwd)
}

fn load_resolved(root: &Path) -> spanda_config::ResolvedSystemConfig {
    ConfigResolver::new()
        .with_validation(false)
        .resolve_from_dir(root)
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            process::exit(1);
        })
}

fn entity_id_arg(args: &[String], usage: &str) -> String {
    args.iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("{usage}");
            process::exit(1);
        })
}

fn json_output(args: &[String]) -> bool {
    args.iter().any(|a| a == "--json")
}

fn parse_program(path: &Path) -> Option<spanda_ast::nodes::Program> {
    let source = fs::read_to_string(path).ok()?;
    let tokens = spanda_lexer::tokenize(&source).ok()?;
    spanda_parser::parse(tokens).ok()
}

fn program_from_args(args: &[String]) -> Option<spanda_ast::nodes::Program> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--program" {
            if let Some(file) = args.get(i + 1) {
                return parse_program(Path::new(file));
            }
        }
    }
    None
}

/// Dispatch `spanda entity` subcommands.
pub fn entity_dispatch(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("");
    match sub {
        "list" => cmd_list(&args[1..]),
        "inspect" => cmd_inspect(&args[1..]),
        "graph" => cmd_graph(&args[1..]),
        "relationships" => cmd_relationships(&args[1..]),
        "traceability" => cmd_traceability(&args[1..]),
        "readiness" => cmd_readiness(&args[1..]),
        "health" => cmd_health(&args[1..]),
        "trust" => cmd_trust(&args[1..]),
        "verify" => cmd_verify(&args[1..]),
        "search" | "query" => cmd_query(&args[1..]),
        _ => {
            eprintln!(
                "Usage:\n  \
                 spanda entity list [--kind KIND] [--health STATUS] [--json] [--config <spanda.toml>]\n  \
                 spanda entity inspect <id> [--json] [--config <spanda.toml>]\n  \
                 spanda entity graph [--json] [--config <spanda.toml>]\n  \
                 spanda entity relationships <id> [--json] [--config <spanda.toml>]\n  \
                 spanda entity traceability [--entity-id ID] [--json] [--config <spanda.toml>]\n  \
                 spanda entity readiness <id> [--json] [--config <spanda.toml>]\n  \
                 spanda entity health <id> [--json] [--config <spanda.toml>]\n  \
                 spanda entity trust <id> [--json] [--config <spanda.toml>]\n  \
                 spanda entity verify <id> [--program file.sd] [--dependencies] [--json] [--config <spanda.toml>]\n  \
                 spanda entity query [--kind KIND] [--tag TAG] [--json] [--config <spanda.toml>]\n  \
                 spanda entity search <text> [--json] [--config <spanda.toml>]"
            );
            process::exit(1);
        }
    }
}

fn cmd_list(args: &[String]) {
    let root = project_root(args);
    let resolved = load_resolved(&root);
    let registry = build_entity_registry(&resolved);
    let query = query_from_args(args);
    let result = registry.query(&query);
    if json_output(args) {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "count": result.count,
                "entities": result.entities,
            }))
            .unwrap()
        );
        return;
    }
    println!("Entities ({}):", result.count);
    for entity in &result.entities {
        println!(
            "  {} [{}] health={} readiness={}",
            entity.id,
            entity.kind(),
            entity.health_status.as_str(),
            entity.readiness_status.as_str()
        );
    }
}

fn cmd_inspect(args: &[String]) {
    let id = entity_id_arg(
        args,
        "Usage: spanda entity inspect <id> [--json] [--config <spanda.toml>]",
    );
    let root = project_root(args);
    let resolved = load_resolved(&root);
    let registry = build_entity_registry(&resolved);
    let Some(entity) = registry.get(&id) else {
        eprintln!("entity '{id}' not found");
        process::exit(1);
    };
    if json_output(args) {
        println!("{}", serde_json::to_string_pretty(entity).unwrap());
        return;
    }
    println!("Entity: {}", entity.id);
    println!("  kind: {}", entity.kind());
    if let Some(name) = entity.display_name.as_ref().or(entity.name.as_ref()) {
        println!("  name: {name}");
    }
    println!("  health: {}", entity.health_status.as_str());
    println!("  readiness: {}", entity.readiness_status.as_str());
    println!("  trust: {}", entity.trust_status.as_str());
    if !entity.capabilities.is_empty() {
        println!("  capabilities: {}", entity.capabilities.join(", "));
    }
    if let Some(parent) = &entity.parent_id {
        println!("  parent: {parent}");
    }
}

fn cmd_graph(args: &[String]) {
    let root = project_root(args);
    let resolved = load_resolved(&root);
    let registry = build_entity_registry(&resolved);
    let graph = registry.graph();
    if json_output(args) {
        println!("{}", serde_json::to_string_pretty(&graph).unwrap());
        return;
    }
    println!(
        "Entity graph: {} nodes, {} edges",
        graph.nodes.len(),
        graph.edges.len()
    );
    for edge in &graph.edges {
        println!(
            "  {} --{}--> {}",
            edge.from_id,
            edge.kind.as_str(),
            edge.to_id
        );
    }
}

fn cmd_relationships(args: &[String]) {
    let id = entity_id_arg(
        args,
        "Usage: spanda entity relationships <id> [--json] [--config <spanda.toml>]",
    );
    let root = project_root(args);
    let resolved = load_resolved(&root);
    let registry = build_entity_registry(&resolved);
    if registry.get(&id).is_none() {
        eprintln!("entity '{id}' not found");
        process::exit(1);
    }
    let relationships: Vec<_> = registry
        .relationships_for(&id)
        .into_iter()
        .cloned()
        .collect();
    let impact = registry.impact_analysis(&id);
    let dependencies = registry.dependency_chain(&id);
    if json_output(args) {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "entity_id": id,
                "relationships": relationships,
                "impact": impact,
                "dependency_chain": dependencies,
            }))
            .unwrap()
        );
        return;
    }
    println!("Relationships for {id}:");
    for rel in &relationships {
        println!("  {:?} {} -> {}", rel.kind, rel.from_id, rel.to_id);
    }
    println!("Dependency chain: {}", dependencies.join(" -> "));
}

fn cmd_traceability(args: &[String]) {
    let root = project_root(args);
    let resolved = load_resolved(&root);
    let registry = build_entity_registry(&resolved);
    let entity_id = args
        .iter()
        .position(|a| a == "--entity-id")
        .and_then(|i| args.get(i + 1).cloned());
    let chain = if let Some(ref id) = entity_id {
        registry.dependency_chain(id)
    } else {
        Vec::new()
    };
    if json_output(args) {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "entity_id": entity_id,
                "dependency_chain": chain,
                "entity_count": registry.entities.len(),
            }))
            .unwrap()
        );
        return;
    }
    if let Some(id) = entity_id {
        println!("Traceability chain for {id}:");
        for step in chain {
            if let Some(entity) = registry.get(&step) {
                println!("  {} [{}]", entity.id, entity.kind());
            } else {
                println!("  {step} [missing]");
            }
        }
    } else {
        println!(
            "Entity registry: {} entities (use --entity-id for dependency chain)",
            registry.entities.len()
        );
    }
}

fn cmd_readiness(args: &[String]) {
    let id = entity_id_arg(
        args,
        "Usage: spanda entity readiness <id> [--json] [--config <spanda.toml>]",
    );
    let root = project_root(args);
    let resolved = load_resolved(&root);
    let registry = build_entity_registry(&resolved);
    if registry.get(&id).is_none() {
        eprintln!("entity '{id}' not found");
        process::exit(1);
    }
    let program = program_from_args(args);
    let mut readiness_options = EntityReadinessOptions {
        program,
        now_ms: 0.0,
        include_dependencies: args.iter().any(|a| a == "--dependencies"),
        platform_audit: None,
    };
    let report = evaluate_entity_readiness(
        &id,
        &registry,
        &resolved,
        &mut readiness_options,
    )
    .expect("entity readiness report");
    if json_output(args) {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        return;
    }
    println!(
        "Readiness for {}: {} (score {:?}, mission_ready {})",
        id, report.readiness_status, report.score, report.mission_ready
    );
    for issue in &report.issues {
        println!("  [{}] {}: {}", issue.severity, issue.factor, issue.message);
    }
}

fn cmd_health(args: &[String]) {
    let id = entity_id_arg(
        args,
        "Usage: spanda entity health <id> [--json] [--config <spanda.toml>]",
    );
    let root = project_root(args);
    let resolved = load_resolved(&root);
    let registry = build_entity_registry(&resolved);
    if registry.get(&id).is_none() {
        eprintln!("entity '{id}' not found");
        process::exit(1);
    }
    let program = program_from_args(args);
    let report = evaluate_entity_health(
        &id,
        &registry,
        &resolved,
        &EntityHealthOptions {
            program,
            now_ms: 0.0,
        },
    )
    .expect("entity health report");
    if json_output(args) {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        return;
    }
    println!("Health for {}: {}", id, report.health_status);
    for diag in &report.diagnostics {
        println!("  [{}] {}: {}", diag.severity, diag.category, diag.message);
    }
}

fn cmd_trust(args: &[String]) {
    let id = entity_id_arg(
        args,
        "Usage: spanda entity trust <id> [--json] [--config <spanda.toml>]",
    );
    let root = project_root(args);
    let resolved = load_resolved(&root);
    let registry = build_entity_registry(&resolved);
    if registry.get(&id).is_none() {
        eprintln!("entity '{id}' not found");
        process::exit(1);
    }
    let program = program_from_args(args);
    let (program_source, program_label) = program.as_ref().map_or((None, None), |_| {
        let path = args
            .iter()
            .position(|a| a == "--program")
            .and_then(|i| args.get(i + 1))
            .map(String::as_str);
        let label = path.and_then(|p| Path::new(p).file_name()?.to_str());
        (
            path.and_then(|p| fs::read_to_string(p).ok()),
            label.map(String::from),
        )
    });
    let report = evaluate_entity_trust(
        &id,
        &registry,
        &resolved,
        &EntityTrustOptions {
            program,
            program_source,
            program_label,
        },
    )
    .expect("entity trust report");
    if json_output(args) {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        return;
    }
    println!(
        "Trust for {}: {} (score {:?}, passed {})",
        id, report.trust_status, report.score, report.passed
    );
    for category in &report.categories {
        println!(
            "  {} score={} passed={} — {}",
            category.name, category.score, category.passed, category.detail
        );
    }
}

fn cmd_verify(args: &[String]) {
    let id = entity_id_arg(
        args,
        "Usage: spanda entity verify <id> [--program file.sd] [--dependencies] [--json] [--config <spanda.toml>]",
    );
    let include_dependencies = args.iter().any(|a| a == "--dependencies");
    let root = project_root(args);
    let resolved = load_resolved(&root);
    let registry = build_entity_registry(&resolved);
    let program = program_from_args(args);
    let options = EntityVerifyOptions {
        program,
        now_ms: 0.0,
        include_dependencies,
    };
    let Some(report) = verify_entity(&id, &registry, &resolved, &options) else {
        eprintln!("entity '{id}' not found");
        process::exit(1);
    };
    if json_output(args) {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        println!(
            "Verify {} [{}]: {}",
            report.entity_id,
            report.entity_type,
            if report.compatible {
                "compatible"
            } else {
                "incompatible"
            }
        );
        for finding in &report.findings {
            println!(
                "  [{}] {}: {}",
                finding.severity, finding.category, finding.message
            );
        }
    }
    if !report.compatible {
        process::exit(1);
    }
}

fn cmd_query(args: &[String]) {
    let root = project_root(args);
    let resolved = load_resolved(&root);
    let registry = build_entity_registry(&resolved);
    let mut query = query_from_args(args);
    if let Some(sub) = args.first() {
        if sub != "--kind" && sub != "--tag" && !sub.starts_with('-') && sub != "query" {
            query.search = Some(sub.clone());
        }
    }
    let result = registry.query(&query);
    if json_output(args) {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "count": result.count,
                "entities": result.entities,
            }))
            .unwrap()
        );
        return;
    }
    println!("Query results ({}):", result.count);
    for entity in &result.entities {
        println!("  {} [{}]", entity.id, entity.kind());
    }
}

fn query_from_args(args: &[String]) -> EntityQuery {
    let mut query = EntityQuery::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--kind" => {
                if let Some(kind) = args.get(i + 1) {
                    query.kind = Some(kind.clone());
                    i += 2;
                    continue;
                }
            }
            "--health" => {
                if let Some(health) = args.get(i + 1) {
                    query.health_status = Some(health.clone());
                    i += 2;
                    continue;
                }
            }
            "--readiness" => {
                if let Some(readiness) = args.get(i + 1) {
                    query.readiness_status = Some(readiness.clone());
                    i += 2;
                    continue;
                }
            }
            "--tag" => {
                if let Some(tag) = args.get(i + 1) {
                    query.tag = Some(tag.clone());
                    i += 2;
                    continue;
                }
            }
            "--trust" => {
                if let Some(trust) = args.get(i + 1) {
                    query.trust_status = Some(trust.clone());
                    i += 2;
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }
    query
}
