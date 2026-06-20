pub mod adapter;
pub mod category;
pub mod dependency;
pub mod error;
pub mod hardware_req;
pub mod import;
pub mod lockfile;
pub mod manifest;
pub mod project;
pub mod publish;
pub mod registry;
pub mod registry_fetch;
pub mod registry_remote;
pub mod resolver;
pub mod safety;
pub mod validation;
pub mod vendor;

pub use adapter::{framework_packages, AdapterMetadata, FrameworkPackage};
pub use category::PackageCategory;
pub use dependency::{
    parse_version, parse_version_req, version_satisfies, DependencyDetail, DependencySourceKind,
    DependencySpec, LockedDependency, LockedSource,
};
pub use error::{PackageError, PackageResult};
pub use hardware_req::{
    high_risk_capabilities, is_high_risk_capability, known_capabilities, validate_capability,
    CapabilityRequirements, HardwareRequirements,
};
pub use import::{all_registered_import_paths, resolve_package_import};
pub use lockfile::{Lockfile, LOCKFILE_FILENAME};
pub use manifest::{find_project_root, PackageManifest, PackageSection, MANIFEST_FILENAME};
pub use project::{add_dependency, collect_source_files, init_package, remove_dependency};
pub use publish::{bundle_package, publish_package, PublishReport};
pub use registry::{
    find_registry_entry, registry_info, registry_package_dir, search_registry,
    search_registry_merged, RegistryEntry, RegistryInfo, LOCAL_REGISTRY,
};
pub use registry_fetch::{
    cache_registry_tarball, fetch_registry_tarball, fetch_url_to_file, file_url_path,
    global_registry_cache_dir, registry_cache_dir, registry_tarball_url, resolve_local_tarball,
};
pub use registry_remote::{
    find_remote_entry, load_remote_registry, lookup_registry_entry, registry_base_url,
    search_remote_registry, RegistryEntryLookup, RemoteRegistryEntry,
};
pub use resolver::{resolve_dependencies, ResolveOptions, ResolveResult};
pub use safety::{SafetyLevel, SafetyMetadata};
pub use validation::{
    validate_package, ApplicationPermissions, ValidationIssue, ValidationReport, ValidationSeverity,
};
pub use vendor::{vendor_dependencies, VendorReport};
