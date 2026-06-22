//! Robot program setup and declaration wiring.
//!

use super::{
    Environment, HalBlockExt, IntoSpandaError, Interpreter, RobotBackend, RuntimeError,
    RuntimeValue, SafetyBlockExt, SocDeclExt,
};
use crate::ai::create_ai_model;
use spanda_ast::nodes::{
    AgentDecl, RobotDecl, SafetyRule, TopicDecl,
};
use spanda_ast::foundations::{StateMachineDecl, TwinDecl};
use spanda_audit::{AuditRuntime, DeviceIdentity, MockLedgerBackend};
use spanda_error::SpandaError;
use spanda_runtime::events::EventBus;
use spanda_hal::hal::{hal_member_from_decl, HalBackend};
use spanda_hal::HardwareMonitor;
use spanda_safety::{create_safety_config_from_robot, SafetyMonitor};
use spanda_security::{RobotIdentity, SecretHandle, SecretSource, SecurityContext, TrustLevel};
use spanda_hal::get_soc_profile;
use spanda_runtime::state_machine::StateMachineRuntime;
use spanda_transport_routing::RoutingCommBus;
use spanda_transport::TransportConfig;
use spanda_runtime::triggers::{ConditionTriggerState, TriggerRegistry, TriggerTimerSchedule};
use spanda_runtime::twin::TwinRuntime;
use std::collections::HashMap;
use std::rc::Rc;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn load_robotics_platform_metadata(
        &mut self,
        fleets: &[spanda_ast::robotics_decl::FleetDecl],
        program_safety_zones: &[spanda_ast::robotics_decl::ProgramSafetyZoneDecl],
        certifications: &[spanda_ast::robotics_decl::CertifyDecl],
    ) {
        // Load fleet groupings, safety zone policies, and certification metadata.
        use spanda_ast::robotics_decl::{CertifyDecl, FleetDecl, ProgramSafetyZoneDecl};
        self.fleets = spanda_runtime::robotics::FleetRegistry::default();
        self.program_safety_zones = spanda_runtime::robotics::ProgramSafetyZoneRegistry::default();

        // Register each declared fleet and expose it through the fleet runtime object.
        for fleet in fleets {
            let FleetDecl::FleetDecl { name, members, .. } = fleet;
            self.fleets.register(name, members.clone());
            self.log(format!("fleet '{name}': {} member(s)", members.len()));
        }
        self.env.define(
            "fleet",
            RuntimeValue::FleetControl {
                registry: self.fleets.clone(),
            },
        );

        // Register program-level safety zone speed caps.
        for zone in program_safety_zones {
            let ProgramSafetyZoneDecl::ProgramSafetyZoneDecl {
                name,
                max_speed_mps,
                ..
            } = zone;
            if let Some(max_speed) = max_speed_mps {
                self.program_safety_zones.register(name, *max_speed);
                self.log(format!(
                    "safety_zone '{name}': max_speed {:.2} m/s",
                    max_speed
                ));
            }
        }

        // Log declared certification standards (metadata only).
        for cert in certifications {
            let CertifyDecl::CertifyDecl {
                standard, level, ..
            } = cert;
            let level_suffix = level
                .as_deref()
                .map(|l| format!(" level {l}"))
                .unwrap_or_default();
            self.log(format!(
                "certify {}{level_suffix}: metadata recorded (verify-only)",
                standard.as_str()
            ));
        }
    }

    pub(super) fn setup_robot(&mut self, robot: &RobotDecl) -> Result<(), SpandaError> {
        // Setup robot.
        //
        // Parameters:
        // - `self` — method receiver
        // - `robot` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.setup_robot(robot);

        // Compute RobotDecl for the following logic.
        let RobotDecl::RobotDecl {
            name: robot_name,
            soc,
            hal,
            topics,
            services,
            actions,
            sensors,
            actuators,
            safety,
            ai_models,
            agents,
            state_machines,
            events,
            event_handlers,
            trigger_handlers,
            twin,
            verify,
            observe,
            identity,
            audit,
            provenance,
            signed_records,
            secrets,
            trust,
            permissions,
            trait_impls,
            buses,
            peer_robots,
            devices,
            twin_sync,
            agent_channels,
            secure_comm,
            trust_boundaries,
            mission,
            ..
        } = robot;
        self.env = Environment::new();
        if self.fleets.names().next().is_some() {
            self.env.define(
                "fleet",
                RuntimeValue::FleetControl {
                    registry: self.fleets.clone(),
                },
            );
        }
        let registry = Rc::clone(&self.provider_registry);
        self.comm_bus = RoutingCommBus::new();
        self.comm_bus.attach_provider_registry(registry);
        self.zones.clear();
        self.stop_if_conditions.clear();
        self.event_bus = EventBus::new();
        self.trigger_registry = TriggerRegistry::new();
        self.trigger_timers.clear();
        self.condition_trigger_state = ConditionTriggerState::default();
        self.declared_event_names.clear();
        self.declared_topic_names.clear();
        self.triggers_dispatched_this_tick = 0;
        self.twin = None;
        self.topic_qos.clear();
        self.topic_last_publish_ms.clear();
        self.topic_deadline_misses.clear();
        self.state_machines.clear();
        self.agent_capabilities.clear();
        self.agent_trait_impls.clear();
        self.verify_rules.clear();
        self.verify_warning_rules.clear();
        self.fusion_sensors.clear();
        self.hardware_monitor = HardwareMonitor::default();
        self.topic_path_to_name.clear();
        self.topic_path_to_message_type.clear();
        self.ai_confidence_low_active = false;
        self.twin_faults_dispatched.clear();
        self.audit_runtime = None;
        self.mock_ledger = MockLedgerBackend::new();
        self.security = SecurityContext::new();
        self.geofences.clear();
        self.geofence_active.clear();
        self.connectivity_events_seen.clear();
        self.gps_available = true;
        if let Some(soc_decl) = soc {
            let profile_name = soc_decl.profile();

            // Emit output when get soc profile provides a profile.
            if let Some(profile) = get_soc_profile(profile_name) {
                self.log(format!("SoC: {} ({})", profile.name, profile.architecture));
            } else {
                self.log(format!("SoC: {profile_name} (unknown)"));
            }
        }

        // Emit output when get hal provides a hal backend.
        if let Some(hal_backend) = self.backend.get_hal() {
            let _ = hal_backend;
        }

        // Emit output when hal provides a hal block.
        if let Some(hal_block) = hal {
            let members: Vec<_> = hal_block
                .members()
                .iter()
                .map(hal_member_from_decl)
                .collect();
            self.hal.configure(&members);
            self.log(format!("HAL configured: {} bus(es)/pin(s)", members.len()));
        }

        // Register declared trust boundaries for runtime enforcement.
        for tb in trust_boundaries {
            let spanda_ast::foundations::TrustBoundaryDecl::TrustBoundaryDecl { name, .. } = tb;
            if let Ok(kind) = name.parse::<spanda_security::TrustBoundaryKind>() {
                self.security.trust_boundaries.declare(kind);
            }
        }

        // Process each bus.
        for bus in buses {
            let spanda_ast::comm_decl::BusDecl::BusDecl {
                transport,
                encryption,
                authentication,
                integrity,
                broker_url,
                ..
            } = bus;
            self.default_transport = *transport;
            let mut bus_security =
                spanda_transport::TransportSecurityConfig::from_bus_fields(
                    encryption.as_deref(),
                    authentication.as_deref(),
                    integrity.as_deref(),
                )
                .unwrap_or_default();
            if let Some(sc) = secure_comm {
                let spanda_ast::foundations::SecureCommPolicyDecl::SecureCommPolicyDecl {
                    encryption,
                    authentication,
                    integrity,
                    ..
                } = sc;
                let robot_policy = spanda_security::SecureCommPolicy {
                    encryption: encryption
                        .as_deref()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_default(),
                    authentication: authentication
                        .as_deref()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_default(),
                    integrity: integrity
                        .as_deref()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_default(),
                };
                bus_security = spanda_transport::effective_transport_policy(
                    &robot_policy,
                    &bus_security,
                );
            }
            for secret_decl in secrets {
                let spanda_ast::foundations::SecretDecl::SecretDecl { name, source, .. } = secret_decl;
                if name.contains("cert") {
                    if let spanda_ast::foundations::SecretSourceDecl::File { path } = source {
                        bus_security.cert_path = Some(path.clone());
                    }
                }
                if name.contains("key") {
                    bus_security.key_secret = Some(name.clone());
                    if let spanda_ast::foundations::SecretSourceDecl::File { path } = source {
                        bus_security.key_path = Some(path.clone());
                    }
                }
            }
            if let Err(e) = bus_security.validate(transport.as_str()) {
                return Err(RuntimeError::new(format!("bus security: {e}"), 1).into_spanda());
            }
            self.security.configure_wire_session(
                bus_security.cert_path.clone(),
                bus_security.key_secret.clone(),
            );
            let transport_boundary =
                spanda_security::boundary_for_transport_name(transport.as_str());
            self.security.set_transport_context(
                transport_boundary,
                bus_security.encryption,
                bus_security.authentication,
                bus_security.integrity,
            );
            let resolved_broker =
                spanda_transport::TransportSecurityConfig::resolve_broker_url(
                    broker_url.as_deref(),
                );
            self.comm_bus
                .configure(TransportConfig {
                    node_name: Some(robot_name.clone()),
                    security: bus_security.clone(),
                    broker_url: resolved_broker,
                    ..Default::default()
                })
                .map_err(|e| {
                    RuntimeError::new(format!("transport configure: {e}"), 1).into_spanda()
                })?;
            self.log(format!(
                "bus transport: {} (encryption: {:?})",
                transport.as_str(),
                bus_security.encryption
            ));
        }

        // Process each peer robot.
        for peer in peer_robots {
            let spanda_ast::comm_decl::PeerRobotDecl::PeerRobotDecl { name, .. } = peer;
            self.comm_bus.register_robot(name);
        }

        // Process each device.
        for device in devices {
            let spanda_ast::comm_decl::DeviceDecl::DeviceDecl { name, .. } = device;
            self.comm_bus.register_device(name);
            self.env.define(
                name.clone(),
                RuntimeValue::Object {
                    type_name: "Device".into(),
                    fields: HashMap::new(),
                },
            );
        }

        // Process each topic.
        for topic in topics {
            let TopicDecl::TopicDecl { name, .. } = topic;
            self.declared_topic_names.insert(name.clone());
            self.define_topic(topic);
        }

        // Process each service.
        for service in services {
            self.define_service(service);
        }

        // Process each action.
        for action in actions {
            self.define_action(action);
        }

        // Process each sensor.
        for sensor in sensors {
            self.define_sensor(sensor);
        }

        // Process each actuator.
        for actuator in actuators {
            self.define_actuator(actuator);
        }
        self.ai_models.clear();
        self.agents.clear();

        // Process each ai model.
        for model_decl in ai_models {
            let model = create_ai_model(model_decl);
            let name = model.name.clone();
            self.env.define(name.clone(), model.to_runtime_value());
            self.log(format!(
                "AI model '{}': {} ({}/{})",
                name, model.model_type, model.config.provider, model.config.model
            ));
            self.ai_models.insert(name, model);
        }

        // Process each agent.
        for agent_decl in agents {
            self.setup_agent(agent_decl);
        }

        // Process each agent channel.
        for channel in agent_channels {
            let spanda_ast::comm_decl::AgentChannelDecl::AgentChannelDecl {
                from_agent,
                to_agent,
                message_type,
                ..
            } = channel;
            self.concurrency
                .register_agent_route(from_agent, to_agent, message_type);
            self.log(format!(
                "agent channel: {from_agent} -> {to_agent}{}",
                // Skip further work when message type is empty.
                if message_type.is_empty() {
                    String::new()
                } else {
                    format!(" ({message_type})")
                }
            ));
        }

        // Process each trait impl.
        for trait_impl in trait_impls {
            use spanda_ast::foundations::TraitImplDecl;
            let TraitImplDecl::TraitImplDecl {
                agent_name,
                methods,
                ..
            } = trait_impl;
            let agent_methods = self
                .agent_trait_impls
                .entry(agent_name.clone())
                .or_default();

            // Process each method.
            for method in methods {
                agent_methods.insert(
                    method.name.clone(),
                    (method.params.clone(), method.body.clone()),
                );
            }
        }

        // Process each event.
        for event in events {
            let spanda_ast::foundations::EventDecl::EventDecl { name, .. } = event;
            self.declared_event_names.insert(name.clone());
            self.log(format!("event declared: {name}"));
        }

        // Invoke each registered handler.
        for handler in event_handlers {
            let spanda_ast::foundations::EventHandlerDecl::EventHandlerDecl {
                event_name, body, ..
            } = handler;
            self.event_bus.register(event_name.clone(), body.clone());
            self.trigger_registry
                .register_legacy_event(event_name.clone(), body.clone());
            self.log(format!("handler registered for {event_name}"));
        }

        // Evaluate each trigger definition.
        for trigger in trigger_handlers {
            self.register_trigger_decl(trigger, None);
        }

        // Process each agent.
        for agent in agents {
            let AgentDecl::AgentDecl {
                name: agent_name,
                trigger_handlers: agent_triggers,
                ..
            } = agent;

            // Evaluate each trigger definition.
            for trigger in agent_triggers {
                self.register_trigger_decl(trigger, Some(agent_name.clone()));
            }
        }
        self.trigger_timers = self
            .trigger_registry
            .timer_handlers()
            .iter()
            .filter_map(|h| TriggerTimerSchedule::from_handler(h))
            .collect();

        // Emit output when twin provides a twin decl.
        if let Some(twin_decl) = twin {
            let TwinDecl::TwinDecl {
                name,
                mirrors,
                replay,
                ..
            } = twin_decl;
            let mut runtime = TwinRuntime::new(name.clone(), mirrors.clone(), *replay);

            // Emit output when twin sync provides a sync.
            if let Some(sync) = twin_sync {
                let spanda_ast::comm_decl::TwinSyncDecl::TwinSyncDecl {
                    telemetry,
                    replay: sync_replay,
                    faults,
                    events,
                    ..
                } = sync;
                runtime = runtime.with_sync(*telemetry, *sync_replay, *faults, *events);
            }
            self.twin = Some(runtime);
            self.env
                .define(name.clone(), RuntimeValue::Twin { name: name.clone() });
            self.log(format!(
                "twin {name}: mirrors [{}], replay={replay}",
                mirrors.join(", ")
            ));
        } else if let Some(sync) = twin_sync {
            let spanda_ast::comm_decl::TwinSyncDecl::TwinSyncDecl {
                telemetry,
                replay,
                faults,
                events,
                ..
            } = sync;
            let name = format!("{robot_name}Twin");
            let runtime = TwinRuntime::new(name.clone(), Vec::new(), *replay)
                .with_sync(*telemetry, *replay, *faults, *events);
            self.twin = Some(runtime);
            self.env
                .define(name.clone(), RuntimeValue::Twin { name: name.clone() });
            self.log(format!(
                "twin sync for {robot_name}: telemetry={telemetry}, replay={replay}, faults={faults}, events={events}"
            ));
        }

        // Emit output when verify provides a verify decl.
        if let Some(verify_decl) = verify {
            let spanda_ast::foundations::VerifyDecl::VerifyDecl {
                rules, warnings, ..
            } = verify_decl;
            self.verify_rules = rules.clone();
            self.verify_warning_rules = warnings.clone();
            self.log(format!(
                "verify: {} rule(s), {} warning(s) registered",
                rules.len(),
                warnings.len()
            ));
        }

        // Emit output when observe provides a observe decl.
        if let Some(observe_decl) = observe {
            let spanda_ast::foundations::ObserveDecl::ObserveDecl { sensors, .. } = observe_decl;
            self.fusion_sensors = sensors.clone();
            self.env.define(
                "fusion",
                RuntimeValue::SensorFusion {
                    sensors: sensors.clone(),
                },
            );
            self.log(format!(
                "observe: fusing {} sensor(s) [{}]",
                sensors.len(),
                sensors.join(", ")
            ));
        }

        // Initialize mission controller and navigation helpers when declared.
        if let Some(mission_decl) = mission {
            use spanda_ast::foundations::MissionDecl;
            let MissionDecl::MissionDecl {
                name,
                duration_hours,
                steps,
                ..
            } = mission_decl;
            let runtime = spanda_runtime::robotics::MissionRuntime::new(
                name.clone(),
                steps.clone(),
                *duration_hours,
            );
            self.env
                .define("mission", RuntimeValue::MissionControl { runtime });
            self.env
                .define("navigation", RuntimeValue::NavigationControl { goal: None });
            let label = name.as_deref().unwrap_or("mission");
            self.log(format!(
                "mission '{label}': {} step(s), duration={:?} h",
                steps.len(),
                duration_hours
            ));
        }

        if self.slam_enabled {
            self.env.define("slam", RuntimeValue::SlamControl);
            self.log("slam: adapter enabled (stub localize/map hooks)".into());
        }

        // Emit output when permissions provides a perm decl.
        if let Some(perm_decl) = permissions {
            let spanda_ast::foundations::PermissionsDecl::PermissionsDecl { capabilities, .. } =
                perm_decl;
            self.security.enable_strict_permissions();
            self.security.capabilities.grant_all(capabilities);
            self.log(format!(
                "permissions: strict mode, granted {} capability(ies)",
                self.security.capabilities.granted().count()
            ));
        }

        // Emit output when trust provides a trust decl.
        if let Some(trust_decl) = trust {
            let spanda_ast::foundations::TrustDecl::TrustDecl { level, .. } = trust_decl;

            // Handle the success value from <TrustLevel>.
            if let Ok(t) = level.parse::<TrustLevel>() {
                self.security.trust = t;
                self.log(format!("trust: level set to {}", t.as_str()));
            }
        }

        // Process each secret.
        for secret_decl in secrets {
            let spanda_ast::foundations::SecretDecl::SecretDecl { name, source, .. } = secret_decl;
            let src = match source {
                spanda_ast::foundations::SecretSourceDecl::Env { var } => {
                    SecretSource::Env { var: var.clone() }
                }
                spanda_ast::foundations::SecretSourceDecl::File { path } => {
                    SecretSource::File { path: path.clone() }
                }
                spanda_ast::foundations::SecretSourceDecl::Literal { value } => SecretSource::Literal {
                    value: value.clone(),
                },
            };
            self.security.grant_if_not_strict("secret.read");
            self.security.secrets.register(SecretHandle {
                name: name.clone(),
                source: src,
            });
            self.env
                .define(name.clone(), RuntimeValue::Secret { name: name.clone() });
            self.log(format!("secret '{name}': registered"));
        }

        // Emit output when identity provides a identity decl.
        if let Some(identity_decl) = identity {
            let spanda_ast::foundations::IdentityDecl::IdentityDecl { fields, .. } = identity_decl;
            let id = fields
                .iter()
                .find(|(k, _)| k == "id")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "unknown".into());
            let public_key = fields
                .iter()
                .find(|(k, _)| k == "public_key")
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            let robot_id =
                RobotIdentity::new(id.clone(), public_key.clone()).with_trust(self.security.trust);
            self.env.define(
                String::from("identity"),
                RuntimeValue::Identity {
                    id: id.clone(),
                    public_key: public_key.clone(),
                },
            );

            // Emit output when as mut provides a rt.
            if let Some(rt) = self.audit_runtime.as_mut() {
                rt.identity = Some(DeviceIdentity::new(id.clone(), public_key));
            }
            self.security.set_identity(robot_id);
            self.security.grant_if_not_strict("identity.sign");
            self.security.grant_if_not_strict("identity.verify");
            self.log(format!("identity: device '{id}' registered"));
        }

        // Emit output when audit provides a audit decl.
        if let Some(audit_decl) = audit {
            let spanda_ast::foundations::AuditDecl::AuditDecl { name, records, .. } = audit_decl;
            let watched: Vec<String> = records.iter().map(|e| Self::expr_path_string(e)).collect();
            let mut rt = AuditRuntime::new(name.clone(), watched.clone());

            // Emit output when identity provides a identity decl.
            if let Some(identity_decl) = identity {
                let spanda_ast::foundations::IdentityDecl::IdentityDecl { fields, .. } = identity_decl;
                let id = fields
                    .iter()
                    .find(|(k, _)| k == "id")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| "unknown".into());
                let public_key = fields
                    .iter()
                    .find(|(k, _)| k == "public_key")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default();
                rt = rt.with_identity(DeviceIdentity::new(id, public_key));
            }

            // Emit output when provenance provides a provenance decl.
            if let Some(provenance_decl) = provenance {
                let spanda_ast::foundations::ProvenanceDecl::ProvenanceDecl {
                    hash_algo,
                    signed_by,
                    ..
                } = provenance_decl;
                rt = rt.with_provenance(hash_algo.clone(), signed_by.clone());
            }
            self.env.define("audit", RuntimeValue::AuditCtx);
            self.audit_runtime = Some(rt);
            self.security.grant_if_not_strict("audit.write");
            self.security.grant_if_not_strict("audit.read");
            self.log(format!(
                "audit {name}: recording {} field(s) [{}]",
                watched.len(),
                watched.join(", ")
            ));
        }

        // Emit output when provenance provides a provenance decl.
        if let Some(provenance_decl) = provenance {
            let spanda_ast::foundations::ProvenanceDecl::ProvenanceDecl { name, .. } = provenance_decl;
            self.log(format!("provenance {name}: sha256 signing enabled"));
        }

        // Process each signed record.
        for signed in signed_records {
            let spanda_ast::foundations::SignedRecordDecl::SignedRecordDecl {
                event_name,
                signed_by,
                ..
            } = signed;

            // Emit output when as mut provides a rt.
            if let Some(rt) = self.audit_runtime.as_mut() {
                let _ = rt.record_event(event_name, &format!("signed_by={signed_by}"));
            }
            self.log(format!(
                "signed record stream: {event_name} (signed_by {signed_by})"
            ));
        }
        self.env.define("mock_ledger", RuntimeValue::LedgerCtx);
        self.security.grant_if_not_strict("ledger.anchor");

        // Process each state machine.
        for sm in state_machines {
            let StateMachineDecl::StateMachineDecl {
                name,
                states,
                transitions,
                ..
            } = sm;
            let pairs: Vec<(String, String)> = transitions
                .iter()
                .map(|t| (t.from.clone(), t.to.clone()))
                .collect();
            let runtime = StateMachineRuntime::new(name.clone(), states.clone(), pairs);
            self.log(format!(
                "state_machine {name}: initial state {}",
                runtime.current
            ));
            self.state_machines.insert(name.clone(), runtime);
        }

        // Proceed only when is some is available.
        if safety.is_some() {
            self.env.define("safety", RuntimeValue::SafetyCtx);
        }
        self.env.define("robot", RuntimeValue::Robot);
        let mut max_speed = f64::INFINITY;

        // Emit output when safety provides a safety block.
        if let Some(safety_block) = safety {
            // Process each rule.
            for rule in safety_block.rules() {
                // Match on rule and handle each case.
                match rule {
                    SafetyRule::MaxSpeedRule { value, .. } => {
                        let val = self.eval_expr(value)?;

                        // Take this path when let RuntimeValue::Number { value, .. } = val.
                        if let RuntimeValue::Number { value, .. } = val {
                            max_speed = value;
                        }
                    }
                    SafetyRule::StopIfRule { condition, .. } => {
                        self.stop_if_conditions.push(condition.clone());
                    }
                }
            }

            // Process each zone.
            for zone in safety_block.zones() {
                let evaluated = self.eval_safety_zone(zone)?;
                self.zones.push(evaluated);
            }
        }
        self.safety_monitor = Some(SafetyMonitor::new(create_safety_config_from_robot(
            max_speed,
            vec![],
            self.zones.clone(),
            self.program_safety_zones.speed_caps().clone(),
        )));
        self.load_reliability_config(robot)?;
        Ok(())
    }

}
