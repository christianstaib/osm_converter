use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct LabelEntry {
    pub vertex: u32,
    pub predecessor: Option<u32>,
    pub cost: u32,
}
