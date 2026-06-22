//! SLAM package adapter hooks for external mapping/localization stacks.

use spanda_ast::nodes::ImportDecl;

/// Import paths that enable SLAM adapter behavior.
pub fn slam_import_paths() -> &'static [&'static str] {
    &["navigation.slam", "navigation.cartographer", "navigation.rtabmap"]
}

/// Return true when the program imports a SLAM-related module path.
pub fn program_uses_slam(imports: &[ImportDecl]) -> bool {
    imports.iter().any(|imp| {
        let ImportDecl::ImportDecl { path, .. } = imp;
        slam_import_paths().contains(&path.as_str())
    })
}
