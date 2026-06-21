//! Signed OTA deploy artifact bundles for rollout integrity.

use crate::deploy_service::{DeployAssignment, DeployPlan};
use serde::{Deserialize, Serialize};
use spanda_audit::{public_key_from_material, sign, verify_signature};

/// Canonical deploy artifact manifest signed for remote OTA rollouts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeployArtifactBundle {
    pub version: String,
    pub program: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program_hash: Option<String>,
    pub assignments: Vec<DeployAssignment>,
    pub certifications: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct BundleCanonicalBody {
    version: String,
    program: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    program_hash: Option<String>,
    assignments: Vec<DeployAssignment>,
    certifications: Vec<String>,
}

impl From<&DeployPlan> for BundleCanonicalBody {
    fn from(plan: &DeployPlan) -> Self {
        Self {
            version: plan.version.clone(),
            program: plan.program.clone(),
            program_hash: plan.program_hash.clone(),
            assignments: plan.assignments.clone(),
            certifications: plan.certifications.clone(),
        }
    }
}

impl From<&DeployArtifactBundle> for BundleCanonicalBody {
    fn from(bundle: &DeployArtifactBundle) -> Self {
        Self {
            version: bundle.version.clone(),
            program: bundle.program.clone(),
            program_hash: bundle.program_hash.clone(),
            assignments: bundle.assignments.clone(),
            certifications: bundle.certifications.clone(),
        }
    }
}

/// Build an unsigned artifact bundle from a deployment plan.
pub fn build_deploy_bundle(plan: &DeployPlan) -> DeployArtifactBundle {
    // Materialize the rollout manifest fields from a deploy plan.
    //
    // Parameters:
    // - `plan` — parsed deployment plan
    //
    // Returns:
    // Unsigned artifact bundle ready for optional signing.
    //
    // Options:
    // None.
    //
    // Example:
    // let bundle = build_deploy_bundle(&plan);

    DeployArtifactBundle {
        version: plan.version.clone(),
        program: plan.program.clone(),
        program_hash: plan.program_hash.clone(),
        assignments: plan.assignments.clone(),
        certifications: plan.certifications.clone(),
        signature: None,
        public_key: None,
    }
}

/// Canonical JSON body used for Ed25519 signing and verification.
pub fn bundle_canonical_json(bundle: &DeployArtifactBundle) -> Result<String, String> {
    // Serialize the signed payload without signature metadata.
    let body = BundleCanonicalBody::from(bundle);
    serde_json::to_string(&body).map_err(|e| e.to_string())
}

/// Sign an artifact bundle with Ed25519 key material.
pub fn sign_deploy_bundle(bundle: &mut DeployArtifactBundle, key_material: &str) -> Result<(), String> {
    // Attach a signature and derived public key to the bundle.
    //
    // Parameters:
    // - `bundle` — artifact bundle to sign in place
    // - `key_material` — Ed25519 seed or signing passphrase
    //
    // Returns:
    // Ok when signing succeeds.
    //
    // Options:
    // None.
    //
    // Example:
    // sign_deploy_bundle(&mut bundle, "fleet-signing-key")?;

    let canonical = bundle_canonical_json(bundle)?;
    bundle.public_key = Some(public_key_from_material(key_material));
    bundle.signature = Some(sign(&canonical, key_material));
    Ok(())
}

/// Verify an artifact bundle signature against trusted key material.
pub fn verify_deploy_bundle(bundle: &DeployArtifactBundle, key_material: &str) -> bool {
    // Validate the bundle signature against trusted signing material.
    //
    // Parameters:
    // - `bundle` — signed artifact bundle
    // - `key_material` — trusted public key hex or signing material
    //
    // Returns:
    // true when the signature matches the canonical body.
    //
    // Options:
    // None.
    //
    // Example:
    // verify_deploy_bundle(&bundle, trusted_key);

    let Some(signature) = bundle.signature.as_deref() else {
        return false;
    };
    let Ok(canonical) = bundle_canonical_json(bundle) else {
        return false;
    };
    verify_signature(&canonical, signature, key_material)
}

/// Verify rollout fields sent to a deploy agent.
pub fn verify_rollout_artifact(
    version: &str,
    program: &str,
    program_hash: Option<&str>,
    signature: &str,
    key_material: &str,
    assignments: &[DeployAssignment],
    certifications: &[String],
) -> bool {
    // Validate a remote rollout payload against a trusted signing key.
    let body = BundleCanonicalBody {
        version: version.to_string(),
        program: program.to_string(),
        program_hash: program_hash.map(str::to_string),
        assignments: assignments.to_vec(),
        certifications: certifications.to_vec(),
    };
    let Ok(canonical) = serde_json::to_string(&body) else {
        return false;
    };
    verify_signature(&canonical, signature, key_material)
}
