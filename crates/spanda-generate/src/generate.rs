//! Template-based Spanda source generation (mock-first, guardrailed).

use crate::llm::refine_with_llm;
use crate::validate::validate_generated_source;
use serde::{Deserialize, Serialize};

/// Generation backend — template scaffolds or optional external LLM refinement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GenerateBackend {
    #[default]
    Template,
    Llm,
}

/// Kind of scaffold to generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerateKind {
    Mission,
    Robot,
    HealthPolicy,
}

/// Options for template generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateOptions {
    pub kind: GenerateKind,
    #[serde(default)]
    pub backend: GenerateBackend,
    pub robot_name: String,
    pub hardware_name: String,
    pub mission_name: String,
    pub behavior_name: String,
    pub health_policy_name: String,
    pub health_check_name: String,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            kind: GenerateKind::Mission,
            backend: GenerateBackend::Template,
            robot_name: "Rover".into(),
            hardware_name: "RoverV1".into(),
            mission_name: "Patrol".into(),
            behavior_name: "patrol_loop".into(),
            health_policy_name: "RoverPolicy".into(),
            health_check_name: "RoverHealth".into(),
        }
    }
}

/// Report for a generated scaffold with validation status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerationReport {
    pub kind: GenerateKind,
    pub backend: GenerateBackend,
    #[serde(default)]
    pub backend_note: Option<String>,
    pub source: String,
    pub validated: bool,
    pub validation_error: Option<String>,
}

/// Generate a minimal patrol mission program.
pub fn generate_mission_program(options: &GenerateOptions) -> GenerationReport {
    // Build a complete patrol mission scaffold and validate it.
    //
    // Parameters:
    // - `options` — naming options for robot, hardware, and mission
    //
    // Returns:
    // Generation report with source and validation status.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = generate_mission_program(&GenerateOptions::default());

    let source = render_mission_program(options);
    finalize_report(GenerateKind::Mission, options.backend, "mission", source)
}

/// Generate a rover hardware + robot scaffold.
pub fn generate_robot_program(options: &GenerateOptions) -> GenerationReport {
    // Build a rover robot scaffold and validate it.
    //
    // Parameters:
    // - `options` — naming options for robot and hardware
    //
    // Returns:
    // Generation report with source and validation status.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = generate_robot_program(&GenerateOptions::default());

    let source = render_robot_program(options);
    finalize_report(GenerateKind::Robot, options.backend, "robot", source)
}

/// Generate health check and health policy blocks for a robot.
pub fn generate_health_policy(options: &GenerateOptions) -> GenerationReport {
    // Build health policy scaffold blocks and validate them in a wrapper program.
    //
    // Parameters:
    // - `options` — naming options for robot and health declarations
    //
    // Returns:
    // Generation report with source and validation status.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = generate_health_policy(&GenerateOptions::default());

    let source = render_health_policy_program(options);
    finalize_report(GenerateKind::HealthPolicy, options.backend, "health-policy", source)
}

/// Format a generation report for CLI output.
pub fn format_generation_report(report: &GenerationReport, json: bool) -> String {
    // Render generation report as JSON or plain source text.
    //
    // Parameters:
    // - `report` — generation report
    // - `json` — emit JSON when true
    //
    // Returns:
    // Formatted output string.
    //
    // Options:
    // None.
    //
    // Example:
    // let text = format_generation_report(&report, false);

    if json {
        return serde_json::to_string_pretty(report).unwrap_or_else(|error| error.to_string());
    }
    let mut lines = vec![
        format!("Generated {:?} scaffold ({:?} backend)", report.kind, report.backend),
        format!(
            "Validation: {}",
            if report.validated {
                "PASS (parse + typecheck)"
            } else {
                "FAIL"
            }
        ),
    ];
    if let Some(note) = &report.backend_note {
        lines.push(format!("Backend note: {note}"));
    }
    if let Some(error) = &report.validation_error {
        lines.push(format!("Validation error: {error}"));
    }
    lines.push(String::new());
    lines.push(report.source.clone());
    lines.join("\n")
}

fn finalize_report(
    kind: GenerateKind,
    backend: GenerateBackend,
    kind_label: &str,
    template: String,
) -> GenerationReport {
    let (source, backend_note) = apply_generation_backend(backend, kind_label, template);
    match validate_generated_source(&source) {
        Ok(()) => GenerationReport {
            kind,
            backend,
            backend_note,
            source,
            validated: true,
            validation_error: None,
        },
        Err(error) => GenerationReport {
            kind,
            backend,
            backend_note,
            source,
            validated: false,
            validation_error: Some(error),
        },
    }
}

fn apply_generation_backend(
    backend: GenerateBackend,
    kind_label: &str,
    template: String,
) -> (String, Option<String>) {
    if backend != GenerateBackend::Llm {
        return (template, None);
    }

    match refine_with_llm(kind_label, &template) {
        Ok(source) => (source, Some("refined via SPANDA_LLM_ENDPOINT".into())),
        Err(error) => (
            template,
            Some(format!("template fallback: {error}")),
        ),
    }
}

fn render_mission_program(options: &GenerateOptions) -> String {
    format!(
        r#"// Generated patrol mission scaffold (mock-first)
hardware {hardware} {{
  sensors [ GPS, Lidar ];
  actuators [ DifferentialDrive ];
}}

robot {robot} {{
  uses hardware {hardware};
  sensor gps: GPS;
  sensor lidar: Lidar;
  actuator wheels: DifferentialDrive;

  exposes capabilities [ gps_navigation, obstacle_avoidance ];

  mission {mission} {{
    requires capabilities [ gps_navigation, obstacle_avoidance ];
    {behavior};
  }}

  safety {{
    max_speed = 0.8 m/s;
    stop_if lidar.nearest_distance < 0.5 m;
  }}

  behavior {behavior}() {{
    loop every 100ms {{
      let scan = lidar.read();
      let _ = scan;
      wheels.drive(linear: 0.2 m/s, angular: 0.0 rad/s);
    }}
  }}
}}
"#,
        hardware = options.hardware_name,
        robot = options.robot_name,
        mission = options.mission_name,
        behavior = options.behavior_name,
    )
}

fn render_robot_program(options: &GenerateOptions) -> String {
    format!(
        r#"// Generated rover scaffold (mock-first)
hardware {hardware} {{
  sensors [ GPS, Lidar, Camera ];
  actuators [ DifferentialDrive ];
  connectivity [ WiFi, LTE ];
}}

robot {robot} {{
  uses hardware {hardware};
  sensor gps: GPS;
  sensor lidar: Lidar;
  sensor camera: Camera;
  actuator wheels: DifferentialDrive;

  exposes capabilities [
    gps_navigation,
    obstacle_avoidance,
    telemetry_streaming
  ];

  safety {{
    max_speed = 0.8 m/s;
    stop_if lidar.nearest_distance < 0.5 m;
  }}

  behavior idle() {{
    wheels.drive(linear: 0.0 m/s, angular: 0.0 rad/s);
  }}
}}
"#,
        hardware = options.hardware_name,
        robot = options.robot_name,
    )
}

fn render_health_policy_program(options: &GenerateOptions) -> String {
    format!(
        r#"// Generated health policy scaffold (mock-first)
hardware {hardware} {{
  sensors [ GPS, Lidar ];
  actuators [ DifferentialDrive ];
}}

robot {robot} {{
  uses hardware {hardware};
  sensor gps: GPS;
  sensor lidar: Lidar;
  actuator wheels: DifferentialDrive;

  safety {{
    max_speed = 0.8 m/s;
  }}

  mode caution_mode {{ wheels.drive(linear: 0.1 m/s, angular: 0.0 rad/s); }}

  behavior idle() {{
    wheels.drive(linear: 0.0 m/s, angular: 0.0 rad/s);
  }}
}}

health_check {health_check} for robot {robot} {{
  check gps.status == Healthy;
  check lidar.status == Healthy;
}}

health_policy {health_policy} {{
  on Degraded {{ enter caution_mode; }}
  on Critical {{ emergency_stop; }}
}}
"#,
        hardware = options.hardware_name,
        robot = options.robot_name,
        health_check = options.health_check_name,
        health_policy = options.health_policy_name,
    )
}
