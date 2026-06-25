//! Configuration layer merging and resolution engine.
//!
use crate::device_tree::DeviceTree;
use crate::error::{ConfigError, ConfigResult};
use crate::json::load_config_value;
use crate::layer::{ConfigGraph, ConfigGraphEdge, ConfigMergeStrategy};
use crate::manifest::{MergeStrategyHint, SpandaManifest, MANIFEST_FILENAME};
use crate::mapping::LogicalPhysicalMap;
use crate::resolved::ResolvedSystemConfig;
use crate::validation::{validate_device_tree, validate_logical_map, ConfigValidationReport};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Options controlling configuration resolution.
#[derive(Debug, Clone, Default)]
pub struct ResolverOptions {
    pub validate: bool,
    pub extra_layers: Vec<PathBuf>,
}

/// Resolves cascading TOML/JSON configuration into `ResolvedSystemConfig`.
#[derive(Debug, Clone, Default)]
pub struct ConfigResolver {
    pub options: ResolverOptions,
}

impl ConfigResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_validation(mut self, validate: bool) -> Self {
        self.options.validate = validate;
        self
    }

    pub fn resolve_from_dir(&self, dir: &Path) -> ConfigResult<ResolvedSystemConfig> {
        // Resolve configuration starting from a project directory.
        //
        // Parameters:
        // - `dir` — project root containing `spanda.toml`
        //
        // Returns:
        // Fully merged and validated system configuration.
        //
        // Options:
        // Uses `self.options` for validation and extra layers.
        //
        // Example:
        // let resolved = ConfigResolver::new().resolve_from_dir(&root)?;

        let manifest_path = dir.join(MANIFEST_FILENAME);
        let manifest = SpandaManifest::load(&manifest_path)?;
        self.resolve(dir, &manifest)
    }

    pub fn resolve(
        &self,
        project_root: &Path,
        manifest: &SpandaManifest,
    ) -> ConfigResult<ResolvedSystemConfig> {
        // Merge manifest layers and fragments into a resolved configuration.
        //
        // Parameters:
        // - `project_root` — project root directory
        // - `manifest` — parsed root manifest
        //
        // Returns:
        // Fully merged system configuration.
        //
        // Options:
        // Uses `self.options` for validation and extra layers.
        //
        // Example:
        // let resolved = resolver.resolve(&root, &manifest)?;

        let mut graph = ConfigGraph::default();
        let root_label = project_root
            .join(MANIFEST_FILENAME)
            .to_string_lossy()
            .into_owned();
        graph.nodes.push(root_label.clone());
        graph.merge_order.push(root_label.clone());

        let mut merged = toml::Value::Table(Default::default());
        let mut layers_applied = Vec::new();
        let mut fragments_loaded = Vec::new();
        let merge_hints = &manifest.merge;

        for (layer_name, rel) in manifest.layer_paths() {
            let layer_path = project_root.join(rel);
            let layer_value = self.load_layer_recursive(
                project_root,
                &layer_path,
                &mut graph,
                &mut HashSet::new(),
            )?;
            merged = merge_values(merged, layer_value, merge_hints, layer_name)?;
            layers_applied.push(format!("{layer_name}:{rel}"));
        }

        for (fragment_name, rel) in manifest.fragment_paths() {
            let fragment_path = project_root.join(rel);
            if !fragment_path.exists() {
                return Err(ConfigError::ConfigFileNotFound {
                    path: fragment_path,
                });
            }
            let fragment_value = load_config_value(&fragment_path)?;
            graph
                .nodes
                .push(fragment_path.to_string_lossy().into_owned());
            graph.edges.push(ConfigGraphEdge {
                from: root_label.clone(),
                to: fragment_path.to_string_lossy().into_owned(),
                layer_kind: fragment_name.into(),
            });
            merged = merge_values(merged, fragment_value, merge_hints, fragment_name)?;
            fragments_loaded.push(format!("{fragment_name}:{rel}"));
        }

        if let Some(ref project) = manifest.project {
            let mut project_table = toml::map::Map::new();
            project_table.insert("name".into(), toml::Value::String(project.name.clone()));
            project_table.insert(
                "version".into(),
                toml::Value::String(project.version.clone()),
            );
            if let Some(ref lang) = project.language {
                project_table.insert("language".into(), toml::Value::String(lang.clone()));
            }
            merge_table(&mut merged, "project", toml::Value::Table(project_table));
        }

        for extra in &self.options.extra_layers {
            let extra_value = load_config_value(extra)?;
            merged = merge_values(merged, extra_value, merge_hints, "extra")?;
            layers_applied.push(extra.to_string_lossy().into_owned());
        }

        let device_tree = DeviceTree::from_toml_value(&merged);
        let logical_map = LogicalPhysicalMap::from_device_tree(&device_tree);
        let providers = extract_providers(&merged);
        let packages = extract_packages(&merged);

        let mut validation = ConfigValidationReport {
            passed: true,
            findings: Vec::new(),
        };
        if self.options.validate {
            let device_report = validate_device_tree(&device_tree, &providers);
            let map_report = validate_logical_map(&logical_map);
            validation = merge_reports(device_report, map_report);
        }

        Ok(ResolvedSystemConfig {
            project_root: project_root.to_path_buf(),
            manifest: manifest.clone(),
            raw: merged,
            layers_applied,
            fragments_loaded,
            device_tree,
            logical_map,
            providers,
            packages,
            validation,
            graph,
        })
    }

    fn load_layer_recursive(
        &self,
        project_root: &Path,
        path: &Path,
        graph: &mut ConfigGraph,
        visited: &mut HashSet<PathBuf>,
    ) -> ConfigResult<toml::Value> {
        let canonical = path
            .canonicalize()
            .unwrap_or_else(|_| project_root.join(path));
        if !visited.insert(canonical.clone()) {
            return Err(ConfigError::CircularLayer {
                cycle: canonical.to_string_lossy().into_owned(),
            });
        }
        if !canonical.exists() {
            return Err(ConfigError::ConfigFileNotFound { path: canonical });
        }

        let label = canonical.to_string_lossy().into_owned();
        graph.nodes.push(label.clone());
        graph.merge_order.push(label.clone());

        let mut value = load_config_value(&canonical)?;
        if let Some(extends) = value.get("extends").and_then(|e| e.as_table().cloned()) {
            let mut base_merged = toml::Value::Table(Default::default());
            let ordered = ["base", "environment", "deployment", "robot"];
            for key in ordered {
                if let Some(rel) = extends.get(key).and_then(|v| v.as_str()) {
                    let layer_path = canonical.parent().unwrap_or(project_root).join(rel);
                    graph.edges.push(ConfigGraphEdge {
                        from: label.clone(),
                        to: layer_path.to_string_lossy().into_owned(),
                        layer_kind: key.into(),
                    });
                    let layer_val =
                        self.load_layer_recursive(project_root, &layer_path, graph, visited)?;
                    base_merged = merge_values(base_merged, layer_val, &HashMap::new(), key)?;
                }
            }
            value = merge_values(base_merged, value, &HashMap::new(), "layer")?;
        }
        visited.remove(&canonical);
        Ok(value)
    }
}

pub fn merge_values(
    base: toml::Value,
    overlay: toml::Value,
    hints: &HashMap<String, MergeStrategyHint>,
    _section: &str,
) -> ConfigResult<toml::Value> {
    // Deep-merge two configuration values with per-section array strategy.
    //
    // Parameters:
    // - `base` — lower-priority value
    // - `overlay` — higher-priority value
    // - `hints` — `[merge]` strategy hints from manifest
    // - `section` — section name for strategy lookup
    //
    // Returns:
    // Merged value tree.
    //
    // Options:
    // None.
    //
    // Example:
    // let merged = merge_values(base, overlay, &hints, "devices")?;

    match (base, overlay) {
        (toml::Value::Table(mut base_table), toml::Value::Table(overlay_table)) => {
            for (key, overlay_val) in overlay_table {
                let strategy = hints
                    .get(&key)
                    .copied()
                    .unwrap_or_else(|| default_array_strategy(&key))
                    .into();
                let merged_entry = if let Some(base_val) = base_table.remove(&key) {
                    merge_entry(base_val, overlay_val, strategy, &key)?
                } else {
                    overlay_val
                };
                base_table.insert(key, merged_entry);
            }
            Ok(toml::Value::Table(base_table))
        }
        (_, overlay) => Ok(overlay),
    }
}

fn merge_entry(
    base: toml::Value,
    overlay: toml::Value,
    strategy: ConfigMergeStrategy,
    key: &str,
) -> ConfigResult<toml::Value> {
    match (&base, &overlay, strategy) {
        (toml::Value::Table(_), toml::Value::Table(_), _) => {
            merge_values(base, overlay, &HashMap::new(), key)
        }
        (
            toml::Value::Array(base_arr),
            toml::Value::Array(overlay_arr),
            ConfigMergeStrategy::Append,
        ) => {
            let mut out = base_arr.clone();
            out.extend(overlay_arr.clone());
            Ok(toml::Value::Array(out))
        }
        (
            toml::Value::Array(base_arr),
            toml::Value::Array(overlay_arr),
            ConfigMergeStrategy::MergeById,
        ) => Ok(toml::Value::Array(merge_arrays_by_id(
            base_arr,
            overlay_arr,
        ))),
        (toml::Value::Array(_), toml::Value::Array(overlay_arr), ConfigMergeStrategy::Replace) => {
            Ok(toml::Value::Array(overlay_arr.clone()))
        }
        (_, overlay, _) => Ok(overlay.clone()),
    }
}

fn default_array_strategy(key: &str) -> MergeStrategyHint {
    match key {
        "robots" | "devices" => MergeStrategyHint::MergeById,
        _ => MergeStrategyHint::Replace,
    }
}

fn merge_arrays_by_id(base: &[toml::Value], overlay: &[toml::Value]) -> Vec<toml::Value> {
    let mut by_id: HashMap<String, toml::Value> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    for item in base {
        if let Some(id) = table_id(item) {
            order.push(id.clone());
            by_id.insert(id, item.clone());
        }
    }
    for item in overlay {
        if let Some(id) = table_id(item) {
            if !by_id.contains_key(&id) {
                order.push(id.clone());
            }
            if let Some(existing) = by_id.remove(&id) {
                let merged = merge_values(existing, item.clone(), &HashMap::new(), &id)
                    .unwrap_or_else(|_| item.clone());
                by_id.insert(id, merged);
            } else {
                by_id.insert(id, item.clone());
            }
        }
    }
    order
        .into_iter()
        .filter_map(|id| by_id.remove(&id))
        .collect()
}

fn table_id(value: &toml::Value) -> Option<String> {
    value
        .as_table()
        .and_then(|t| t.get("id"))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
}

fn merge_table(merged: &mut toml::Value, key: &str, value: toml::Value) {
    if let toml::Value::Table(ref mut table) = merged {
        table.insert(key.into(), value);
    }
}

fn extract_providers(merged: &toml::Value) -> Vec<String> {
    let mut providers = Vec::new();
    if let Some(table) = merged.get("providers").and_then(|v| v.as_table()) {
        for (name, _) in table {
            providers.push(name.clone());
        }
    }
    if let Some(list) = merged.get("providers").and_then(|v| v.as_array()) {
        for item in list {
            if let Some(name) = item.as_str() {
                providers.push(name.into());
            } else if let Some(id) = item.get("name").and_then(|v| v.as_str()) {
                providers.push(id.into());
            }
        }
    }
    providers.sort();
    providers.dedup();
    providers
}

fn extract_packages(merged: &toml::Value) -> Vec<String> {
    let mut packages = Vec::new();
    if let Some(deps) = merged.get("dependencies").and_then(|v| v.as_table()) {
        packages.extend(deps.keys().cloned());
    }
    packages.sort();
    packages
}

fn merge_reports(a: ConfigValidationReport, b: ConfigValidationReport) -> ConfigValidationReport {
    let mut findings = a.findings;
    findings.extend(b.findings);
    let passed = findings
        .iter()
        .all(|f| f.severity != crate::validation::ValidationSeverity::Error);
    ConfigValidationReport { findings, passed }
}

pub fn diff_configs(left: &toml::Value, right: &toml::Value) -> Vec<String> {
    // Produce human-readable diff lines between two config values.
    //
    // Parameters:
    // - `left` — baseline configuration
    // - `right` — comparison configuration
    //
    // Returns:
    // Lines describing added, removed, and changed keys.
    //
    // Options:
    // None.
    //
    // Example:
    // let lines = diff_configs(&base, &prod);

    let mut lines = Vec::new();
    diff_values("", left, right, &mut lines);
    lines
}

fn diff_values(prefix: &str, left: &toml::Value, right: &toml::Value, lines: &mut Vec<String>) {
    match (left, right) {
        (toml::Value::Table(l), toml::Value::Table(r)) => {
            for (key, lval) in l {
                let path = format!("{prefix}{key}");
                if let Some(rval) = r.get(key) {
                    diff_values(&format!("{path}."), lval, rval, lines);
                } else {
                    lines.push(format!("- {path}"));
                }
            }
            for (key, _) in r {
                if !l.contains_key(key) {
                    lines.push(format!("+ {prefix}{key}"));
                }
            }
        }
        (l, r) if l != r => {
            lines.push(format!("~ {prefix}: {l:?} -> {r:?}"));
        }
        _ => {}
    }
}
