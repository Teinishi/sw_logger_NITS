use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub retention_period: u32,
    pub keep_values: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            retention_period: 3600,
            keep_values: false,
        }
    }
}

impl Settings {
    pub fn max_len(&self) -> usize {
        self.retention_period.try_into().unwrap()
    }
}
