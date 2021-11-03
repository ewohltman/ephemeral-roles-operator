use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Ephemeral Roles Version resource spec
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    kind = "Version",
    group = "ephemeral-roles.net",
    version = "v1",
    namespaced
)]
#[kube(status = "VersionStatus")]
pub struct VersionSpec {
    name: String,
    version: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct VersionStatus {
    is_deployed: bool,
}
