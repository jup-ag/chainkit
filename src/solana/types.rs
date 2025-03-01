use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalAddress {
    pub recent_blockhash: String,
}
