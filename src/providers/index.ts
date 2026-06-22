/**
 * Lean-core provider contracts mirrored for TypeScript tests and CLI fallback.
 * @module
 */

export {
  bootstrapDefaultProviders,
  bootstrapProvidersForPackages,
  officialPackageForTransport,
  syncCommBusForOfficialPackages,
} from "./bootstrap.js";
export { ProviderRegistry, transportRegistryKey, type ProviderId, type TransportProvider } from "./registry.js";

/** Where a module or feature belongs in the lean-core architecture. */
export enum ModuleOwnership {
  Core = "core",
  StandardLibrary = "stdlib",
  OfficialPackage = "official-package",
  ExperimentalPackage = "experimental-package",
  CompatibilityShim = "compatibility-shim",
  Deprecated = "deprecated",
}

export type ModuleClassification = {
  module: string;
  ownership: ModuleOwnership;
  targetPackage: string | null;
  notes: string;
};

/** Official first-party package names recognized by the lean-core model. */
export const OFFICIAL_PACKAGE_NAMES: readonly string[] = [
  "spanda-gps",
  "spanda-wifi",
  "spanda-ble",
  "spanda-cellular",
  "spanda-mqtt",
  "spanda-dds",
  "spanda-ros2",
  "spanda-slam",
  "spanda-nav",
  "spanda-opencv",
  "spanda-yolo",
  "spanda-moveit",
  "spanda-gazebo",
  "spanda-webots",
  "spanda-fleet",
  "spanda-ota",
  "spanda-maintenance",
  "spanda-ledger",
  "spanda-cloud",
  "spanda-openai",
] as const;

/** Static audit table aligned with Rust `spanda-runtime/src/classification.rs`. */
export const MODULE_CLASSIFICATIONS: readonly ModuleClassification[] = [
  { module: "lexer", ownership: ModuleOwnership.Core, targetPackage: null, notes: "Compiler front-end" },
  { module: "parser", ownership: ModuleOwnership.Core, targetPackage: null, notes: "Compiler front-end" },
  { module: "type_system", ownership: ModuleOwnership.Core, targetPackage: null, notes: "Type checker and std namespace registry" },
  { module: "safety", ownership: ModuleOwnership.Core, targetPackage: null, notes: "ActionProposal / SafeAction gate" },
  { module: "scheduler", ownership: ModuleOwnership.Core, targetPackage: null, notes: "Task and trigger scheduling interfaces" },
  { module: "providers", ownership: ModuleOwnership.Core, targetPackage: null, notes: "Extension trait contracts for packages" },
  {
    module: "connectivity_positioning",
    ownership: ModuleOwnership.CompatibilityShim,
    targetPackage: "spanda-gps / spanda-wifi / spanda-ble / spanda-cellular",
    notes: "Type names stay in core; drivers move to connectivity packages",
  },
  {
    module: "transport_mqtt",
    ownership: ModuleOwnership.Deprecated,
    targetPackage: "spanda-mqtt",
    notes: "Removed from spanda-core; use spanda-transport-mqtt or spanda-transport-routing",
  },
  {
    module: "transport_rclrs",
    ownership: ModuleOwnership.CompatibilityShim,
    targetPackage: "spanda-ros2",
    notes: "ROS2 transport; use spanda-ros2 package",
  },
  {
    module: "transport_dds",
    ownership: ModuleOwnership.Deprecated,
    targetPackage: "spanda-dds",
    notes: "Removed from spanda-core; use spanda-transport-dds or spanda-transport-routing",
  },
  {
    module: "transport_websocket",
    ownership: ModuleOwnership.Deprecated,
    targetPackage: "spanda-mqtt",
    notes: "Removed from spanda-core; use spanda-transport-websocket or spanda-transport-routing",
  },
  {
    module: "transport_live",
    ownership: ModuleOwnership.Deprecated,
    targetPackage: "spanda-transport-routing",
    notes: "Removed from spanda-core; use spanda_transport_routing::transport_live",
  },
  {
    module: "nav2_adapter",
    ownership: ModuleOwnership.CompatibilityShim,
    targetPackage: "spanda-nav",
    notes: "Nav2 bridge subprocess adapter",
  },
  {
    module: "slam_adapter",
    ownership: ModuleOwnership.CompatibilityShim,
    targetPackage: "spanda-slam",
    notes: "SLAM bridge subprocess adapter",
  },
  {
    module: "ai",
    ownership: ModuleOwnership.CompatibilityShim,
    targetPackage: "spanda-opencv / spanda-yolo / spanda-openai",
    notes: "AiProvider trait stays; vendor registries move to packages",
  },
  {
    module: "fleet_orchestrator",
    ownership: ModuleOwnership.CompatibilityShim,
    targetPackage: "spanda-fleet",
    notes: "Fleet orchestration CLI remains; heavy logic moves to package",
  },
  {
    module: "deploy_service",
    ownership: ModuleOwnership.CompatibilityShim,
    targetPackage: "spanda-ota",
    notes: "OTA deploy/rollout moves to spanda-ota",
  },
  {
    module: "simulator",
    ownership: ModuleOwnership.Core,
    targetPackage: null,
    notes: "Default in-memory sim; Gazebo/Webots via simulation packages",
  },
] as const;
