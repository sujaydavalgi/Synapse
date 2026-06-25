//! Mission resource estimation from hardware profiles and program structure.

use serde::{Deserialize, Serialize};
use spanda_ast::foundations::{DeployDecl, MissionDecl, TaskDecl};
use spanda_ast::nodes::{BehaviorDecl, Program, RobotDecl, Stmt};
use spanda_hardware::{build_profile_registry, HardwareProfile};

const ESTIMATED_TASK_COST_MS: f64 = 5.0;
const DEFAULT_DURATION_HOURS: f64 = 1.0;
const TRACE_STORAGE_MB: f64 = 128.0;

/// Output format for mission estimates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EstimateFormat {
    #[default]
    Text,
    Json,
}

/// Options for mission resource estimation.
#[derive(Debug, Clone, Default)]
pub struct EstimateOptions {
    pub target: Option<String>,
}

/// Confidence for a single resource estimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceConfidence {
    High,
    Medium,
    Low,
}

/// Estimated value for one mission resource dimension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceEstimate {
    pub resource: String,
    pub value: f64,
    pub unit: String,
    pub confidence: ResourceConfidence,
    pub detail: String,
}

/// Mission resource estimation rollup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionEstimateReport {
    pub program: String,
    pub robot: Option<String>,
    pub target: Option<String>,
    pub resources: Vec<ResourceEstimate>,
    pub assumptions: Vec<String>,
    pub within_budget: bool,
}

/// Estimate mission resource usage for a Spanda program.
pub fn estimate_mission(
    program: &Program,
    source_label: &str,
    options: &EstimateOptions,
) -> MissionEstimateReport {
    // Compose battery, CPU, memory, storage, network, and duration estimates.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `source_label` — file label
    // - `options` — optional deploy target override
    //
    // Returns:
    // Mission resource estimate report.
    //
    // Options:
    // `EstimateOptions::target` selects hardware profile.
    //
    // Example:
    // let report = estimate_mission(&program, "mission.sd", &EstimateOptions::default());

    let registry = build_profile_registry(program);
    let (robot_name, target_name) = resolve_robot_and_target(program, options);
    let profile = target_name
        .as_ref()
        .and_then(|name| registry.get(name))
        .map(|profile| profile.clone())
        .or_else(|| registry.values().next().cloned());

    let mut assumptions = Vec::new();
    let robot = robot_name
        .as_ref()
        .and_then(|name| find_robot(program, name));

    let duration_hours = robot
        .and_then(|robot_decl| mission_duration_hours(robot_decl))
        .unwrap_or_else(|| {
            assumptions.push(format!(
                "mission duration not declared — assuming {DEFAULT_DURATION_HOURS}h"
            ));
            DEFAULT_DURATION_HOURS
        });

    let duration_confidence = if robot.is_some_and(|r| mission_duration_hours(r).is_some()) {
        ResourceConfidence::High
    } else {
        ResourceConfidence::Low
    };

    let mut resources = Vec::new();
    resources.push(ResourceEstimate {
        resource: "duration".into(),
        value: duration_hours,
        unit: "hours".into(),
        confidence: duration_confidence,
        detail: format!("Planned mission runtime {duration_hours:.2} h"),
    });

    if let Some(robot_decl) = robot {
        let cpu_pct = estimate_cpu_pct(robot_decl);
        resources.push(ResourceEstimate {
            resource: "cpu".into(),
            value: cpu_pct,
            unit: "percent".into(),
            confidence: ResourceConfidence::High,
            detail: format!("Aggregate control-loop CPU duty cycle {cpu_pct:.1}%"),
        });

        let network_mbps = estimate_network_mbps(robot_decl);
        resources.push(ResourceEstimate {
            resource: "network".into(),
            value: network_mbps,
            unit: "mbps".into(),
            confidence: if network_mbps > 0.0 {
                ResourceConfidence::Medium
            } else {
                ResourceConfidence::Low
            },
            detail: format!("Estimated telemetry bandwidth {network_mbps:.2} Mbps"),
        });
    } else {
        assumptions.push("no robot selected — CPU and network estimates omitted".into());
    }

    if let Some(profile) = profile.as_ref() {
        let battery = estimate_battery_wh(profile, duration_hours);
        resources.push(ResourceEstimate {
            resource: "battery".into(),
            value: battery.required_wh,
            unit: "wh".into(),
            confidence: battery.confidence,
            detail: battery.detail,
        });

        let memory_mb = estimate_memory_mb(profile, robot);
        resources.push(ResourceEstimate {
            resource: "memory".into(),
            value: memory_mb.value,
            unit: "mb".into(),
            confidence: memory_mb.confidence,
            detail: memory_mb.detail,
        });

        let storage_mb = estimate_storage_mb(robot);
        resources.push(ResourceEstimate {
            resource: "storage".into(),
            value: storage_mb,
            unit: "mb".into(),
            confidence: ResourceConfidence::Medium,
            detail: format!("Estimated trace/log storage {storage_mb:.0} MB"),
        });
    } else {
        assumptions.push("hardware profile unknown — power and memory estimates omitted".into());
    }

    let within_budget = resources.iter().all(|resource| resource_within_budget(resource, profile.as_ref()));
    MissionEstimateReport {
        program: source_label.into(),
        robot: robot_name,
        target: target_name,
        resources,
        assumptions,
        within_budget,
    }
}

/// Format a mission estimate report for CLI output.
pub fn format_mission_estimate(report: &MissionEstimateReport, format: EstimateFormat) -> String {
    // Render mission estimates as JSON or human-readable text.
    //
    // Parameters:
    // - `report` — mission estimate report
    // - `format` — text or JSON output
    //
    // Returns:
    // Formatted report string.
    //
    // Options:
    // None.
    //
    // Example:
    // println!("{}", format_mission_estimate(&report, EstimateFormat::Text));

    match format {
        EstimateFormat::Json => serde_json::to_string_pretty(report).unwrap_or_else(|e| e.to_string()),
        EstimateFormat::Text => format_estimate_text(report),
    }
}

struct BatteryEstimate {
    required_wh: f64,
    confidence: ResourceConfidence,
    detail: String,
}

struct MemoryEstimate {
    value: f64,
    confidence: ResourceConfidence,
    detail: String,
}

fn format_estimate_text(report: &MissionEstimateReport) -> String {
    let mut lines = vec![
        format!("Mission estimate: {}", report.program),
        format!(
            "Target: {}",
            report
                .target
                .as_deref()
                .unwrap_or("(unknown)")
        ),
        format!(
            "Budget: {}",
            if report.within_budget {
                "WITHIN"
            } else {
                "EXCEEDED"
            }
        ),
        String::new(),
    ];

    for resource in &report.resources {
        lines.push(format!(
            "- {}: {:.2} {} ({:?}) — {}",
            resource.resource, resource.value, resource.unit, resource.confidence, resource.detail
        ));
    }

    if !report.assumptions.is_empty() {
        lines.push(String::new());
        lines.push("Assumptions:".into());
        for assumption in &report.assumptions {
            lines.push(format!("  - {assumption}"));
        }
    }

    lines.join("\n").trim_end().to_string()
}

fn resolve_robot_and_target(
    program: &Program,
    options: &EstimateOptions,
) -> (Option<String>, Option<String>) {
    let Program::Program {
        robots,
        deployments,
        ..
    } = program;

    if let Some(target) = options.target.clone() {
        let robot = deployments.iter().find_map(|deploy| match deploy {
            DeployDecl::DeployDecl { robot_name, targets, .. } if targets.iter().any(|t| t == &target) => {
                Some(robot_name.clone())
            }
            _ => None,
        }).or_else(|| robots.first().map(robot_name));
        return (robot, Some(target));
    }

    if let Some(deploy) = deployments.first() {
        let DeployDecl::DeployDecl {
            robot_name,
            targets,
            ..
        } = deploy;
        return (Some(robot_name.clone()), targets.first().cloned());
    }

    (
        robots.first().map(robot_name),
        None,
    )
}

fn find_robot<'a>(program: &'a Program, name: &str) -> Option<&'a RobotDecl> {
    let Program::Program { robots, .. } = program;
    robots.iter().find(|robot| robot_name(robot) == name)
}

fn robot_name(robot: &RobotDecl) -> String {
    match robot {
        RobotDecl::RobotDecl { name, .. } => name.clone(),
    }
}

fn mission_duration_hours(robot: &RobotDecl) -> Option<f64> {
    let RobotDecl::RobotDecl { mission, .. } = robot;
    let mission = mission.as_ref()?;
    let MissionDecl::MissionDecl { duration_hours, .. } = mission;
    duration_hours.filter(|hours| *hours > 0.0)
}

fn estimate_cpu_pct(robot: &RobotDecl) -> f64 {
    let RobotDecl::RobotDecl {
        tasks,
        behaviors,
        ..
    } = robot;
    let mut total = 0.0;
    for task in tasks {
        let TaskDecl::TaskDecl { interval_ms, .. } = task;
        total += (ESTIMATED_TASK_COST_MS / interval_ms.max(1.0)) * 100.0;
    }
    for behavior in behaviors {
        let BehaviorDecl::BehaviorDecl { body, .. } = behavior;
        for interval in collect_loop_intervals(body) {
            total += (ESTIMATED_TASK_COST_MS / interval.max(1.0)) * 100.0;
        }
    }
    total
}

fn estimate_network_mbps(robot: &RobotDecl) -> f64 {
    let RobotDecl::RobotDecl { topics, .. } = robot;
    if topics.is_empty() {
        return 0.0;
    }
    topics.len() as f64 * 0.5
}

fn estimate_battery_wh(profile: &HardwareProfile, duration_hours: f64) -> BatteryEstimate {
    let required_wh = profile.power_draw_w * duration_hours;
    if let Some(capacity) = profile.battery_wh {
        let detail = if required_wh > capacity {
            format!(
                "Mission requires {required_wh:.1} Wh but battery supports {capacity:.1} Wh"
            )
        } else {
            format!(
                "Mission energy {required_wh:.1} Wh within battery capacity {capacity:.1} Wh"
            )
        };
        BatteryEstimate {
            required_wh,
            confidence: ResourceConfidence::High,
            detail,
        }
    } else {
        BatteryEstimate {
            required_wh,
            confidence: ResourceConfidence::Medium,
            detail: format!(
                "Estimated draw {required_wh:.1} Wh (battery capacity unknown on target)"
            ),
        }
    }
}

fn estimate_memory_mb(profile: &HardwareProfile, robot: Option<&RobotDecl>) -> MemoryEstimate {
    let base = profile.memory_mb.unwrap_or(1024.0) * 0.25;
    let mut extra = 0.0;
    if let Some(RobotDecl::RobotDecl {
        sensors,
        agents,
        ..
    }) = robot
    {
        extra += sensors.len() as f64 * 48.0;
        extra += agents.len() as f64 * 256.0;
    }
    let value = base + extra;
    MemoryEstimate {
        value,
        confidence: if profile.memory_mb.is_some() {
            ResourceConfidence::Medium
        } else {
            ResourceConfidence::Low
        },
        detail: format!("Estimated runtime memory footprint {value:.0} MB"),
    }
}

fn estimate_storage_mb(robot: Option<&RobotDecl>) -> f64 {
    let loops = robot
        .map(|robot_decl| {
            let RobotDecl::RobotDecl { behaviors, .. } = robot_decl;
            behaviors
                .iter()
                .map(|behavior| {
                    let BehaviorDecl::BehaviorDecl { body, .. } = behavior;
                    collect_loop_intervals(body).len()
                })
                .sum::<usize>()
        })
        .unwrap_or(0);
    if loops > 0 {
        TRACE_STORAGE_MB
    } else {
        32.0
    }
}

fn resource_within_budget(resource: &ResourceEstimate, profile: Option<&HardwareProfile>) -> bool {
    match resource.resource.as_str() {
        "cpu" => resource.value <= 80.0,
        "battery" => profile
            .and_then(|profile| profile.battery_wh)
            .is_none_or(|capacity| resource.value <= capacity),
        "memory" => profile
            .and_then(|profile| profile.memory_mb)
            .is_none_or(|capacity| resource.value <= capacity),
        "storage" => profile
            .and_then(|profile| profile.storage_mb)
            .is_none_or(|capacity| resource.value <= capacity),
        _ => true,
    }
}

fn collect_loop_intervals(stmts: &[Stmt]) -> Vec<f64> {
    let mut intervals = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::LoopStmt {
                interval_ms, body, ..
            } => {
                intervals.push(*interval_ms);
                intervals.extend(collect_loop_intervals(body));
            }
            Stmt::IfStmt {
                then_branch,
                else_branch,
                ..
            } => {
                intervals.extend(collect_loop_intervals(then_branch));
                if let Some(branch) = else_branch {
                    intervals.extend(collect_loop_intervals(branch));
                }
            }
            _ => {}
        }
    }
    intervals
}
