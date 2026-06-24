//! Mission assurance static analysis: knowledge models.
//!
use crate::types::{
    CapabilityOntology, ComponentModel, DependencyGraph, MissionKnowledgeBase, SystemModel,
};
use spanda_ast::assurance_decl::KnowledgeModelDecl;
use spanda_ast::nodes::Program;

/// Extract system models from program knowledge_model declarations.
pub fn extract_knowledge_base(program: &Program) -> MissionKnowledgeBase {
    let Program::Program {
        knowledge_models, ..
    } = program;
    let models = knowledge_models
        .iter()
        .map(|decl| {
            let KnowledgeModelDecl::KnowledgeModelDecl {
                name,
                components,
                dependencies,
                ..
            } = decl;
            SystemModel {
                name: name.clone(),
                components: components
                    .iter()
                    .map(|c| ComponentModel {
                        name: c.name.clone(),
                    })
                    .collect(),
                dependencies: DependencyGraph {
                    edges: dependencies
                        .iter()
                        .map(|d| (d.capability.clone(), d.requires.clone()))
                        .collect(),
                },
            }
        })
        .collect();
    MissionKnowledgeBase { models }
}

/// Build capability ontology from knowledge model dependencies.
pub fn capability_ontology(program: &Program) -> Vec<CapabilityOntology> {
    let kb = extract_knowledge_base(program);
    kb.models
        .iter()
        .flat_map(|m| {
            m.dependencies
                .edges
                .iter()
                .map(|(cap, reqs)| CapabilityOntology {
                    capability: cap.clone(),
                    requires: reqs.clone(),
                })
        })
        .collect()
}

/// Validate knowledge model completeness against declared robot sensors/actuators.
pub fn validate_knowledge_models(program: &Program) -> Vec<String> {
    let mut issues = Vec::new();
    let kb = extract_knowledge_base(program);
    let Program::Program { robots, .. } = program;

    for model in &kb.models {
        if model.components.is_empty() {
            issues.push(format!(
                "Knowledge model '{}' has no components declared",
                model.name
            ));
        }
        for (cap, reqs) in &model.dependencies.edges {
            if reqs.is_empty() {
                issues.push(format!(
                    "Knowledge model '{}': dependency '{}' has empty requires list",
                    model.name, cap
                ));
            }
        }
    }

    if kb.models.is_empty() && !robots.is_empty() {
        issues.push("Robot declared but no knowledge_model defined".into());
    }

    issues
}
