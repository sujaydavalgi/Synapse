//! Guardrails for the hosted registry index and tarballs in `registry/`.
//!
use spanda_package::registry_remote::{fetch_index_json, RemoteRegistryEntry};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const OFFICIAL_HOSTED: &[&str] = &[
    "spanda-ble",
    "spanda-cellular",
    "spanda-cloud",
    "spanda-dds",
    "spanda-fleet",
    "spanda-gazebo",
    "spanda-gps",
    "spanda-ledger",
    "spanda-maintenance",
    "spanda-moveit",
    "spanda-mqtt",
    "spanda-nav",
    "spanda-openai",
    "spanda-opencv",
    "spanda-ota",
    "spanda-ros2",
    "spanda-slam",
    "spanda-webots",
    "spanda-wifi",
    "spanda-yolo",
];

#[test]
fn hosted_registry_index_lists_twenty_official_packages() {
    let index_path = repo_root().join("registry/index.json");
    let body = fs::read_to_string(&index_path).expect("registry/index.json");
    let entries: Vec<RemoteRegistryEntry> =
        serde_json::from_str(&body).expect("parse hosted index.json");

    assert_eq!(
        entries.len(),
        OFFICIAL_HOSTED.len(),
        "hosted index should list all official packages"
    );

    for name in OFFICIAL_HOSTED {
        assert!(
            entries.iter().any(|entry| entry.name == *name),
            "missing hosted index entry for {name}"
        );
        assert!(
            spanda_package::is_official_package(name),
            "{name} should be in framework_packages()"
        );
    }
}

#[test]
fn hosted_registry_tarballs_exist_for_each_index_entry() {
    let index_path = repo_root().join("registry/index.json");
    let body = fs::read_to_string(&index_path).expect("registry/index.json");
    let entries: Vec<RemoteRegistryEntry> =
        serde_json::from_str(&body).expect("parse hosted index.json");

    for entry in entries {
        let version = entry
            .versions
            .first()
            .expect("each index entry should expose at least one version");
        let tarball = repo_root()
            .join("registry/packages")
            .join(&entry.name)
            .join(version);
        assert!(
            tarball.is_file(),
            "missing tarball at registry/packages/{}/{}",
            entry.name,
            version
        );
    }
}

#[test]
fn file_url_fetches_local_registry_index() {
    let index_path = repo_root().join("registry/index.json");
    let url = format!("file://{}", index_path.display());
    let body = fetch_index_json(&url).expect("fetch local registry index via file://");
    let entries: Vec<RemoteRegistryEntry> =
        serde_json::from_str(&body).expect("parse fetched index JSON");
    assert!(!entries.is_empty());
    assert!(entries.iter().any(|entry| entry.name == "spanda-ros2"));
}

#[test]
fn hosted_packages_match_registry_scaffolds() {
    let scaffolds: Vec<String> = fs::read_dir(repo_root().join("packages/registry"))
        .expect("packages/registry")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().join("spanda.toml").is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();

    assert!(
        scaffolds.len() >= OFFICIAL_HOSTED.len(),
        "packages/registry should contain at least one scaffold per hosted package (got {} scaffolds, {} hosted)",
        scaffolds.len(),
        OFFICIAL_HOSTED.len()
    );

    for name in OFFICIAL_HOSTED {
        let scaffold = repo_root()
            .join("packages/registry")
            .join(name)
            .join("spanda.toml");
        assert!(
            scaffold.is_file(),
            "missing scaffold at {}",
            scaffold.display()
        );
    }
}

#[test]
fn hosted_registry_tarballs_match_index_checksums() {
    let index_path = repo_root().join("registry/index.json");
    let body = fs::read_to_string(&index_path).expect("registry/index.json");
    let entries: Vec<RemoteRegistryEntry> =
        serde_json::from_str(&body).expect("parse hosted index.json");

    for entry in entries {
        for (version, expected) in &entry.version_checksums {
            let tarball = repo_root()
                .join("registry/packages")
                .join(&entry.name)
                .join(version);
            spanda_package::verify_sha256(&tarball, expected).unwrap_or_else(|err| {
                panic!("checksum mismatch for {}/{}: {err}", entry.name, version);
            });
        }
    }
}

#[test]
fn hosted_registry_index_carries_valid_signatures() {
    use spanda_package::registry_sign::verify_registry_signature;

    let index_path = repo_root().join("registry/index.json");
    let body = fs::read_to_string(&index_path).expect("registry/index.json");
    let entries: Vec<RemoteRegistryEntry> =
        serde_json::from_str(&body).expect("parse hosted index.json");
    let trust_key = fs::read_to_string(repo_root().join("registry/TRUST_KEY"))
        .expect("registry/TRUST_KEY")
        .trim()
        .to_string();

    for entry in entries {
        for version in &entry.versions {
            let digest = entry
                .version_checksums
                .get(version)
                .unwrap_or_else(|| panic!("missing checksum for {}/{}", entry.name, version));
            let signature = entry
                .version_signatures
                .get(version)
                .unwrap_or_else(|| panic!("missing signature for {}/{}", entry.name, version));
            assert_eq!(
                signature.public_key, trust_key,
                "unexpected signing key for {}/{}",
                entry.name, version
            );
            assert!(
                verify_registry_signature(&entry.name, version, digest, signature, &trust_key),
                "invalid signature for {}/{}",
                entry.name,
                version
            );
        }
    }
}
