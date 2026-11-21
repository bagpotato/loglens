use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Meta {
    pub hosts: Vec<String>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub by_severity: BTreeMap<String, u64>,
    pub by_source: BTreeMap<String, u64>,
}
