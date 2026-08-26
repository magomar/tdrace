use std::path::Path;
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// Record entry stored in the Hall of Fame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HallOfFameEntry {
    pub id: Option<i64>,
    pub track_id: String,
    pub player_name: String,
    pub car_name: String,
    pub total_time: f32,
    pub best_lap: Option<f32>,
    pub laps: u32,
    pub created_at: String,
}

/// SQLite persistence manager for local Hall of Fame leaderboards.
pub struct HallOfFameDb {
    conn: Connection,
}

impl HallOfFameDb {
    /// Default database filename placed in the working directory.
    pub const DEFAULT_DB_PATH: &'static str = "tdrace_records.db";

    /// Opens or creates the default local database.
    pub fn open_default() -> Result<Self> {
        Self::open(Path::new(Self::DEFAULT_DB_PATH))
    }

    /// Opens or creates a database at the specified path.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Creates an in-memory database instance (ideal for automated unit tests).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Initializes tables and indexes if they do not already exist.
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS hall_of_fame (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                track_id TEXT NOT NULL,
                player_name TEXT NOT NULL,
                car_name TEXT NOT NULL,
                total_time REAL NOT NULL,
                best_lap REAL,
                laps INTEGER NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_hof_track_time ON hall_of_fame(track_id, total_time ASC);",
        )?;
        Ok(())
    }

    /// Retrieves up to the 10 best historical results for a specific track, sorted by total time ascending.
    pub fn get_top_10(&self, track_id: &str) -> Result<Vec<HallOfFameEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, track_id, player_name, car_name, total_time, best_lap, laps, created_at
             FROM hall_of_fame
             WHERE track_id = ?1
             ORDER BY total_time ASC
             LIMIT 10",
        )?;

        let rows = stmt.query_map(params![track_id], |row| {
            Ok(HallOfFameEntry {
                id: Some(row.get(0)?),
                track_id: row.get(1)?,
                player_name: row.get(2)?,
                car_name: row.get(3)?,
                total_time: row.get::<_, f64>(4)? as f32,
                best_lap: row.get::<_, Option<f64>>(5)?.map(|v| v as f32),
                laps: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        Ok(entries)
    }

    /// Checks whether a given total race time qualifies for the Top 10 on this track.
    pub fn is_top_10(&self, track_id: &str, total_time: f32) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM hall_of_fame WHERE track_id = ?1",
            params![track_id],
            |row| row.get(0),
        )?;

        if count < 10 {
            return Ok(true);
        }

        // Fetch the 10th best time
        let tenth_time: Option<f64> = self.conn.query_row(
            "SELECT total_time FROM hall_of_fame WHERE track_id = ?1 ORDER BY total_time ASC LIMIT 1 OFFSET 9",
            params![track_id],
            |row| row.get(0),
        ).ok();

        match tenth_time {
            Some(t) => Ok((total_time as f64) < t),
            None => Ok(true),
        }
    }

    /// Inserts a new record entry into the Hall of Fame table.
    pub fn insert_entry(&self, entry: &HallOfFameEntry) -> Result<i64> {
        let best_lap_f64 = entry.best_lap.map(|v| v as f64);
        let created_at = if entry.created_at.is_empty() {
            Utc::now().format("%Y-%m-%d %H:%M").to_string()
        } else {
            entry.created_at.clone()
        };

        self.conn.execute(
            "INSERT INTO hall_of_fame (track_id, player_name, car_name, total_time, best_lap, laps, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                entry.track_id,
                entry.player_name.trim(),
                entry.car_name,
                entry.total_time as f64,
                best_lap_f64,
                entry.laps,
                created_at,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Seeds benchmark AI driver records if a track currently has no historical entries.
    pub fn seed_defaults_if_empty(&self, track_id: &str) -> Result<()> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM hall_of_fame WHERE track_id = ?1",
            params![track_id],
            |row| row.get(0),
        )?;

        if count == 0 {
            let now = Utc::now().format("%Y-%m-%d %H:%M").to_string();
            let (base_total, base_lap) = match track_id {
                "classic_grand_prix" => (75.0, 24.5),
                "oval_speedway" => (42.0, 13.8),
                "drift_park" => (68.0, 22.0),
                "kart_arena" => (52.0, 17.0),
                _ => (70.0, 23.0),
            };

            let benchmarks = [
                ("Apex Tanaka", "GT Sports Coupe", base_total * 0.96, base_lap * 0.96),
                ("Thunder Rossi", "AWD Turbo Rally", base_total * 0.98, base_lap * 0.98),
                ("Phoenix Lin", "GT Sports Coupe", base_total * 1.00, base_lap * 1.00),
                ("Drift King Kenji", "Tuned Drift Spec", base_total * 1.02, base_lap * 1.02),
                ("Viper Frost", "GT Sports Coupe", base_total * 1.04, base_lap * 1.04),
                ("Oversteer Reed", "AWD Turbo Rally", base_total * 1.06, base_lap * 1.05),
                ("Pocket Rocket Leo", "125cc Shifter Kart", base_total * 1.08, base_lap * 1.07),
                ("The Wall Sterling", "GT Sports Coupe", base_total * 1.11, base_lap * 1.10),
            ];

            for (name, car, total, lap) in benchmarks {
                self.insert_entry(&HallOfFameEntry {
                    id: None,
                    track_id: track_id.to_string(),
                    player_name: name.to_string(),
                    car_name: car.to_string(),
                    total_time: total,
                    best_lap: Some(lap),
                    laps: 3,
                    created_at: now.clone(),
                })?;
            }

        }
        Ok(())
    }
}
