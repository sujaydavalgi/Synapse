fn main() {
  inject_tauri_config_from_env();
  tauri_build::build();
}

fn inject_tauri_config_from_env() {
  let signing_private_key = std::env::var("TAURI_SIGNING_PRIVATE_KEY")
    .ok()
    .filter(|value| !value.trim().is_empty());
  let updater_pubkey = std::env::var("TAURI_UPDATER_PUBKEY")
    .ok()
    .filter(|value| !value.trim().is_empty());

  let create_updater_artifacts = signing_private_key.is_some();
  let mut merge = serde_json::Map::new();
  merge.insert(
    "bundle".into(),
    serde_json::json!({
        "createUpdaterArtifacts": create_updater_artifacts
    }),
  );

  if create_updater_artifacts {
    if let Some(pubkey) = updater_pubkey {
      let active = std::env::var("TAURI_UPDATER_ACTIVE")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(true);
      merge.insert(
        "plugins".into(),
        serde_json::json!({
            "updater": {
                "pubkey": pubkey,
                "active": active
            }
        }),
      );
      println!("cargo:rerun-if-env-changed=TAURI_UPDATER_PUBKEY");
      println!("cargo:rerun-if-env-changed=TAURI_UPDATER_ACTIVE");
    }
  }

  std::env::set_var("TAURI_CONFIG", serde_json::Value::Object(merge).to_string());
  println!("cargo:rerun-if-env-changed=TAURI_SIGNING_PRIVATE_KEY");
}
