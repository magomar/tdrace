use serde::{Deserialize, Serialize};

/// Type of metric ranked on the leaderboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordMetric {
    /// Lowest score is best (e.g. Lap Time, Speedrun time in seconds).
    LowestTime,
    /// Highest score is best (e.g. Points, Wave survived, Accuracy).
    HighestScore,
}

/// A single entry in the Hall of Fame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordEntry {
    pub player_name: String,
    pub player_alias: String,
    pub country: Option<String>,
    pub score: f64,
    pub detail: String,
    pub timestamp: String,
}

/// Category / Track / Level leaderboard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HallOfFame {
    pub category_id: String,
    pub metric: RecordMetric,
    pub max_entries: usize,
    pub entries: Vec<RecordEntry>,
}

impl HallOfFame {
    pub fn new(category_id: &str, metric: RecordMetric, max_entries: usize) -> Self {
        Self {
            category_id: category_id.to_string(),
            metric,
            max_entries: max_entries.max(1),
            entries: Vec::new(),
        }
    }

    /// Checks if a score qualifies for the leaderboard.
    pub fn qualifies(&self, score: f64) -> bool {
        if self.entries.len() < self.max_entries {
            return true;
        }
        match self.metric {
            RecordMetric::LowestTime => score < self.entries.last().unwrap().score,
            RecordMetric::HighestScore => score > self.entries.last().unwrap().score,
        }
    }

    /// Inserts a new record in sorted rank order.
    pub fn insert(&mut self, entry: RecordEntry) -> Option<usize> {
        let mut insert_pos = self.entries.len();
        for (i, existing) in self.entries.iter().enumerate() {
            let is_better = match self.metric {
                RecordMetric::LowestTime => entry.score < existing.score,
                RecordMetric::HighestScore => entry.score > existing.score,
            };
            if is_better {
                insert_pos = i;
                break;
            }
        }

        if insert_pos < self.max_entries {
            self.entries.insert(insert_pos, entry);
            if self.entries.len() > self.max_entries {
                self.entries.pop();
            }
            Some(insert_pos + 1) // 1-indexed rank (P1, P2, ...)
        } else {
            None
        }
    }

    /// Returns the top / best record entry if any exists.
    pub fn top_entry(&self) -> Option<&RecordEntry> {
        self.entries.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hall_of_fame_ranking_lowest_time() {
        let mut hof = HallOfFame::new("circuit_1", RecordMetric::LowestTime, 3);
        assert_eq!(hof.insert(RecordEntry {
            player_name: "Bob".to_string(),
            player_alias: "B".to_string(),
            country: Some("USA".to_string()),
            score: 52.4,
            detail: "Lap".to_string(),
            timestamp: "2026".to_string(),
        }), Some(1));

        assert_eq!(hof.insert(RecordEntry {
            player_name: "Alice".to_string(),
            player_alias: "A".to_string(),
            country: Some("ESP".to_string()),
            score: 50.1,
            detail: "Lap".to_string(),
            timestamp: "2026".to_string(),
        }), Some(1)); // Alice is faster, takes Rank 1

        assert_eq!(hof.entries[0].player_name, "Alice");
        assert_eq!(hof.entries[1].player_name, "Bob");
    }
}
