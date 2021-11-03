use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Ephemeral Roles Version resource spec
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(kind = "ERVersion", group = "ephemeral-roles.net", version = "v1")]
pub struct ERVersionSpec {
    enabled: bool,
}
