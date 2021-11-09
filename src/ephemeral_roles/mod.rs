use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ERVersion spec
#[derive(CustomResource, Default, PartialEq, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[kube(
    group = "ephemeral-roles.net",
    version = "v1",
    kind = "ERVersion",
    plural = "erversions",
    shortname = "erv",
    derive = "PartialEq",
    derive = "Default",
    status = "ERVersionStatus"
)]
pub struct ERVersionSpec {
    pub components: Vec<Component>,
}

#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Component {
    pub name: String,
    pub version: String,
    pub files: Vec<String>,
}

/// ERVersion status
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ERVersionStatus {}
