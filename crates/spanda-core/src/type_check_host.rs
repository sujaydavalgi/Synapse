//! Core-backed [`TypeCheckHost`] hooks for the extracted program type checker.
//!
use crate::ai::resolve_ai_import;
use crate::ffi_registry;
use crate::hal::hal_member_from_decl;
use crate::lib_registry::{all_library_sensor_types, resolve_import};
use crate::robotics_platform::{
    validate_fleet_members, validate_mission_decl, validate_swarm_fleet,
};
use crate::soc::{get_soc_profile, validate_hal_against_soc};
use crate::stdlib::resolve_std_import;
use crate::reliability;
use crate::slam_adapter;
use spanda_ast::nodes::{HalMemberDecl, Span, SpandaType};
use spanda_typecheck::{Diagnostic, TypeCheckHost};
use std::collections::HashMap;

/// Default host wiring domain registries from `spanda-core` into `spanda-typecheck`.
pub struct CoreTypeCheckHost;

impl TypeCheckHost for CoreTypeCheckHost {
    fn import_path_known(&self, path: &str, module_registry_has_export: bool) -> bool {
        resolve_import(path).is_some()
            || resolve_ai_import(path).is_some()
            || resolve_std_import(path)
            || ffi_registry::resolve_ffi_import(path)
            || module_registry_has_export
    }

    fn slam_import_known(&self, path: &str) -> bool {
        slam_adapter::slam_import_paths().contains(&path)
    }

    fn library_exports_sensor(&self, library: &str, sensor_type: &str) -> Option<bool> {
        resolve_import(library).map(|module| module.sensors.contains_key(sensor_type))
    }

    fn library_sensor_type_known(&self, sensor_type: &str) -> bool {
        all_library_sensor_types().contains_key(sensor_type)
    }

    fn library_sensor_robo_types(&self) -> HashMap<String, SpandaType> {
        all_library_sensor_types()
            .into_iter()
            .map(|(name, info)| (name, info.robo_type))
            .collect()
    }

    fn library_for_sensor_type(&self, sensor_type: &str) -> Option<String> {
        all_library_sensor_types()
            .get(sensor_type)
            .map(|info| info.library.clone())
    }

    fn soc_profile_known(&self, profile: &str) -> bool {
        get_soc_profile(profile).is_some()
    }

    fn validate_hal_against_soc(&self, profile: &str, members: &[HalMemberDecl]) -> Vec<String> {
        let Some(soc) = get_soc_profile(profile) else {
            return Vec::new();
        };
        let hal_members: Vec<_> = members.iter().map(hal_member_from_decl).collect();
        validate_hal_against_soc(&soc, &hal_members)
            .into_iter()
            .map(|d| d.message)
            .collect()
    }

    fn validate_fleet_members(
        &self,
        fleet_name: &str,
        members: &[String],
        robot_names: &[String],
    ) -> Option<String> {
        validate_fleet_members(fleet_name, members, robot_names)
    }

    fn validate_swarm_fleet(
        &self,
        swarm_name: &str,
        fleet_name: &str,
        fleet_names: &[String],
    ) -> Option<String> {
        validate_swarm_fleet(swarm_name, fleet_name, fleet_names)
    }

    fn validate_mission_decl(
        &self,
        name: &Option<String>,
        duration_hours: Option<f64>,
        steps: &[String],
    ) -> Option<String> {
        validate_mission_decl(name, duration_hours, steps)
    }

    fn security_capability_known(&self, capability: &str) -> bool {
        spanda_security::is_known_capability(capability)
    }

    fn validate_task_timing(&self, task: &spanda_ast::foundations::TaskDecl) -> Vec<Diagnostic> {
        reliability::validate_task_timing(task)
    }

    fn validate_task_priority(&self, task: &spanda_ast::foundations::TaskDecl) -> Vec<Diagnostic> {
        reliability::validate_task_priority(task)
    }

    fn validate_resource_budget(
        &self,
        budget: &spanda_ast::foundations::ResourceBudgetDecl,
        span: Span,
    ) -> Vec<Diagnostic> {
        reliability::validate_resource_budget(budget, span)
    }
}

/// Shared core host instance for type-check entry points.
pub fn core_type_check_host() -> &'static CoreTypeCheckHost {
    &CoreTypeCheckHost
}
