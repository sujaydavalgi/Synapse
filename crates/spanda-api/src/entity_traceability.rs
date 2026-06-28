//! Unified entity + program + digital-thread traceability overlay.
//!
use crate::program::parse_program_file;
use crate::state::ControlCenterState;
use serde::{Deserialize, Serialize};
use spanda_config::{
    apply_traceability_overlay, DigitalThreadTraceabilityLink, EntityRegistry,
    ProgramGraphTraceabilityEdge,
};
use spanda_graph::{
    annotate_entity_ids, build_alignment_context, build_dependency_graph,
    program_graph_entity_edges, query_digital_thread, DependencyGraph, DigitalThreadQuery,
    DigitalThreadReport,
};

/// Filters for unified traceability across entity and program graphs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EntityTraceabilityQuery {
    #[serde(default)]
    pub entity_id: Option<String>,
    #[serde(default)]
    pub capability: Option<String>,
    #[serde(default)]
    pub device_id: Option<String>,
}

/// Unified traceability report combining entity registry, program graph, and digital thread.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityTraceabilityReport {
    pub query: EntityTraceabilityQuery,
    pub source: Option<String>,
    pub aligned_node_count: usize,
    pub program_edge_count: usize,
    pub device_link_count: usize,
    pub unified_chain: Vec<String>,
    pub program_graph: DependencyGraph,
    pub digital_thread: DigitalThreadReport,
    pub entity_paths: Vec<Vec<String>>,
}

/// Build digital-thread and program-graph overlays for the entity registry.
pub fn traceability_overlay_inputs(
    state: &ControlCenterState,
) -> (
    Vec<DigitalThreadTraceabilityLink>,
    Vec<ProgramGraphTraceabilityEdge>,
    Option<DependencyGraph>,
) {
    let Some(path) = state.program_path.as_ref() else {
        return (Vec::new(), Vec::new(), None);
    };
    let Ok((program, _, label)) = parse_program_file(path) else {
        return (Vec::new(), Vec::new(), None);
    };
    let mut graph = build_dependency_graph(&program, &label, state.resolved.as_ref());
    let ctx = build_alignment_context(&graph, state.resolved.as_ref());
    annotate_entity_ids(&mut graph, &ctx);
    let program_edges: Vec<ProgramGraphTraceabilityEdge> = program_graph_entity_edges(&graph, &ctx)
        .into_iter()
        .map(|edge| ProgramGraphTraceabilityEdge {
            from_entity_id: edge.from_entity_id,
            to_entity_id: edge.to_entity_id,
            relation: edge.relation,
        })
        .collect();
    let thread = query_digital_thread(
        &program,
        &label,
        state.resolved.as_ref(),
        &DigitalThreadQuery::default(),
    );
    let device_links: Vec<DigitalThreadTraceabilityLink> = thread
        .device_links
        .iter()
        .flat_map(|link| {
            link.related_capabilities
                .iter()
                .map(move |capability| DigitalThreadTraceabilityLink {
                    device_id: link.device_id.clone(),
                    capability: capability.clone(),
                    assigned_robot: link.assigned_robot.clone(),
                })
        })
        .collect();
    (device_links, program_edges, Some(graph))
}

/// Apply traceability overlays onto a registry clone used by entity APIs.
pub fn enrich_entity_registry(state: &ControlCenterState, registry: &mut EntityRegistry) {
    let (device_links, program_edges, _) = traceability_overlay_inputs(state);
    if device_links.is_empty() && program_edges.is_empty() {
        return;
    }
    apply_traceability_overlay(registry, &device_links, &program_edges);
}

/// Build a unified traceability report for Control Center and SDK consumers.
pub fn build_entity_traceability_report(
    state: &ControlCenterState,
    query: &EntityTraceabilityQuery,
) -> Result<EntityTraceabilityReport, String> {
    let Some(path) = state.program_path.as_ref() else {
        return Err("program not loaded — set --program for traceability".into());
    };
    let (program, _, label) = parse_program_file(path).map_err(|message| message.to_string())?;
    let thread_query = DigitalThreadQuery {
        capability: query.capability.clone(),
        device_id: query.device_id.clone(),
        node_id: query.entity_id.clone(),
        lifecycle_phase: None,
    };
    let mut graph = build_dependency_graph(&program, &label, state.resolved.as_ref());
    let ctx = build_alignment_context(&graph, state.resolved.as_ref());
    annotate_entity_ids(&mut graph, &ctx);
    let aligned_node_count = graph
        .nodes
        .iter()
        .filter(|node| node.metadata.contains_key("entity_id"))
        .count();
    let program_edges = program_graph_entity_edges(&graph, &ctx);
    let digital_thread =
        query_digital_thread(&program, &label, state.resolved.as_ref(), &thread_query);
    let device_link_count = digital_thread.device_links.len();
    let registry = state.entity_registry();
    let unified_chain =
        build_unified_chain(query, &registry, &graph, &digital_thread, &program_edges);
    let entity_paths = entity_paths_for_query(query, &registry, &unified_chain);
    Ok(EntityTraceabilityReport {
        query: query.clone(),
        source: Some(label),
        aligned_node_count,
        program_edge_count: program_edges.len(),
        device_link_count,
        unified_chain,
        program_graph: graph,
        digital_thread,
        entity_paths,
    })
}

fn build_unified_chain(
    query: &EntityTraceabilityQuery,
    registry: &EntityRegistry,
    graph: &DependencyGraph,
    thread: &DigitalThreadReport,
    program_edges: &[spanda_graph::ProgramGraphEntityEdge],
) -> Vec<String> {
    let mut chain = Vec::new();
    if let Some(entity_id) = query.entity_id.as_ref() {
        if registry.entities.contains_key(entity_id) {
            chain.push(entity_id.clone());
        }
    }
    if let Some(device_id) = query.device_id.as_ref() {
        if registry.entities.contains_key(device_id) && !chain.contains(device_id) {
            chain.push(device_id.clone());
        }
    }
    for link in &thread.device_links {
        if query.device_id.as_deref() == Some(link.device_id.as_str())
            || query.entity_id.as_deref() == Some(link.device_id.as_str())
        {
            if let Some(robot) = link.assigned_robot.as_ref() {
                if !chain.contains(robot) {
                    chain.push(robot.clone());
                }
            }
            for capability in &link.related_capabilities {
                if query
                    .capability
                    .as_deref()
                    .is_none_or(|cap| cap == capability)
                {
                    let marker = format!("capability:{capability}");
                    if !chain.contains(&marker) {
                        chain.push(marker);
                    }
                }
            }
        }
    }
    for edge in program_edges {
        if query.entity_id.as_deref() == Some(edge.from_entity_id.as_str())
            || query.entity_id.as_deref() == Some(edge.to_entity_id.as_str())
        {
            if !chain.contains(&edge.from_entity_id) {
                chain.push(edge.from_entity_id.clone());
            }
            if !chain.contains(&edge.to_entity_id) {
                chain.push(edge.to_entity_id.clone());
            }
        }
    }
    for node in &graph.nodes {
        if let Some(entity_id) = node.metadata.get("entity_id") {
            if query.entity_id.as_deref() == Some(entity_id.as_str()) && !chain.contains(entity_id)
            {
                chain.push(entity_id.clone());
            }
        }
    }
    chain
}

fn entity_paths_for_query(
    query: &EntityTraceabilityQuery,
    registry: &EntityRegistry,
    seed_chain: &[String],
) -> Vec<Vec<String>> {
    let Some(entity_id) = query
        .entity_id
        .as_ref()
        .filter(|id| registry.entities.contains_key(id.as_str()))
    else {
        return Vec::new();
    };
    let mut paths = vec![vec![entity_id.clone()]];
    for item in seed_chain {
        if item == entity_id || item.starts_with("capability:") {
            continue;
        }
        if registry.entities.contains_key(item) {
            paths.push(vec![entity_id.clone(), item.clone()]);
        }
    }
    paths
}
