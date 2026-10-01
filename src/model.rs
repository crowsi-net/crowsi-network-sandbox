use serde::{Deserialize, Serialize};

pub const ISOLATION_SCHEMA: &str = "crowsi://network/isolation-capability/v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// Independent flags keep the machine contract explicit about absent isolation layers.
#[allow(clippy::struct_excessive_bools)]
pub struct IsolationCapabilityV1 {
    pub schema: String,
    pub mechanism: String,
    pub platform: String,
    pub environment_cleared: bool,
    pub finite_timeout: bool,
    pub user_namespace: bool,
    pub network_namespace: bool,
    pub mount_namespace: bool,
    pub filesystem_isolation: bool,
    pub external_actions: bool,
}

impl Default for IsolationCapabilityV1 {
    fn default() -> Self {
        Self {
            schema: ISOLATION_SCHEMA.to_owned(),
            mechanism: "linux-user-network-namespace".to_owned(),
            platform: "linux".to_owned(),
            environment_cleared: true,
            finite_timeout: true,
            user_namespace: true,
            network_namespace: true,
            mount_namespace: false,
            filesystem_isolation: false,
            external_actions: false,
        }
    }
}
