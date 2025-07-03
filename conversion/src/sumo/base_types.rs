use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Param {
    #[serde(rename = "@key")]
    pub key: String,
    #[serde(rename = "@value")]
    pub value: Option<String>,
}
