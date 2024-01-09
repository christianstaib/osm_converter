use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct LabelEntry {
    pub id: u32,
    pub cost: u32,
}
