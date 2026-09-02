use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::records::leaderboard::HallOfFame;

/// Multi-category record database with JSON persistence.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RecordDatabase {
    pub categories: HashMap<String, HallOfFame>,
}

impl RecordDatabase {
    pub fn new() -> Self {
        Self {
            categories: HashMap::new(),
        }
    }

    /// Retrieves or initializes a Hall of Fame category.
    pub fn get_or_create(
        &mut self,
        category_id: &str,
        metric: crate::records::leaderboard::RecordMetric,
        max_entries: usize,
    ) -> &mut HallOfFame {
        self.categories
            .entry(category_id.to_string())
            .or_insert_with(|| HallOfFame::new(category_id, metric, max_entries))
    }

    /// Saves the database to disk at path.
    pub fn save_to_disk(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }

    /// Loads the database from disk, or creates an empty one.
    pub fn load_from_disk(path: &Path) -> Self {
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(path) {
                if let Ok(db) = serde_json::from_str::<Self>(&data) {
                    return db;
                }
            }
        }
        Self::new()
    }
}
