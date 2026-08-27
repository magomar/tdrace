use std::path::Path;
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

use crate::profile::{PlayerProfile, ProfileCareerStats, RaceHistoryEntry};
use crate::render::color::CarColorScheme;

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

/// SQLite persistence manager for local Hall of Fame leaderboards, player profiles, and career race logs.
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
            CREATE INDEX IF NOT EXISTS idx_hof_track_time ON hall_of_fame(track_id, total_time ASC);

            CREATE TABLE IF NOT EXISTS player_profiles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                alias TEXT NOT NULL,
                country TEXT,
                primary_color TEXT NOT NULL,
                secondary_color TEXT NOT NULL,
                helmet_color TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS race_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id INTEGER NOT NULL,
                track_id TEXT NOT NULL,
                car_name TEXT NOT NULL,
                position INTEGER NOT NULL,
                total_cars INTEGER NOT NULL,
                total_time REAL NOT NULL,
                best_lap REAL,
                laps INTEGER NOT NULL,
                is_time_attack INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY(profile_id) REFERENCES player_profiles(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_race_history_profile ON race_history(profile_id, created_at DESC);",
        )?;
        Ok(())
    }

    // =========================================================================
    // Player Profile Management
    // =========================================================================

    /// Retrieves all player profiles ordered by active status descending, then creation date ascending.
    pub fn get_all_profiles(&self) -> Result<Vec<PlayerProfile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, alias, country, primary_color, secondary_color, helmet_color, is_active, created_at
             FROM player_profiles
             ORDER BY is_active DESC, id ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            let p_hex: String = row.get(4)?;
            let s_hex: String = row.get(5)?;
            let h_hex: String = row.get(6)?;
            let is_active_int: i32 = row.get(7)?;

            Ok(PlayerProfile {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                alias: row.get(2)?,
                country: row.get(3)?,
                color_scheme: CarColorScheme::from_hex_strings(&p_hex, &s_hex, &h_hex),
                is_active: is_active_int != 0,
                created_at: row.get(8)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Retrieves the currently active player profile or creates a default if none exists.
    pub fn get_active_profile(&self) -> Result<PlayerProfile> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, alias, country, primary_color, secondary_color, helmet_color, is_active, created_at
             FROM player_profiles
             WHERE is_active = 1
             LIMIT 1",
        )?;

        let mut rows = stmt.query_map([], |row| {
            let p_hex: String = row.get(4)?;
            let s_hex: String = row.get(5)?;
            let h_hex: String = row.get(6)?;

            Ok(PlayerProfile {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                alias: row.get(2)?,
                country: row.get(3)?,
                color_scheme: CarColorScheme::from_hex_strings(&p_hex, &s_hex, &h_hex),
                is_active: true,
                created_at: row.get(8)?,
            })
        })?;

        if let Some(first) = rows.next() {
            first
        } else {
            // If no active profile, seed default and return
            self.seed_default_profile_if_empty()
        }
    }

    /// Fetches a profile by ID.
    pub fn get_profile_by_id(&self, id: i64) -> Result<Option<PlayerProfile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, alias, country, primary_color, secondary_color, helmet_color, is_active, created_at
             FROM player_profiles
             WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(params![id], |row| {
            let p_hex: String = row.get(4)?;
            let s_hex: String = row.get(5)?;
            let h_hex: String = row.get(6)?;
            let is_active_int: i32 = row.get(7)?;

            Ok(PlayerProfile {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                alias: row.get(2)?,
                country: row.get(3)?,
                color_scheme: CarColorScheme::from_hex_strings(&p_hex, &s_hex, &h_hex),
                is_active: is_active_int != 0,
                created_at: row.get(8)?,
            })
        })?;

        if let Some(r) = rows.next() {
            Ok(Some(r?))
        } else {
            Ok(None)
        }
    }

    /// Inserts a new profile and optionally makes it active.
    pub fn create_profile(&self, profile: &PlayerProfile) -> Result<i64> {
        let (p_hex, s_hex, h_hex) = profile.color_scheme.to_hex_strings();
        let created_at = if profile.created_at.is_empty() {
            Utc::now().format("%Y-%m-%d %H:%M").to_string()
        } else {
            profile.created_at.clone()
        };

        if profile.is_active {
            // Unset other active profiles
            self.conn.execute("UPDATE player_profiles SET is_active = 0", [])?;
        }

        self.conn.execute(
            "INSERT INTO player_profiles (name, alias, country, primary_color, secondary_color, helmet_color, is_active, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                profile.name.trim(),
                profile.alias.trim(),
                profile.country.as_deref().map(|s| s.trim().to_uppercase()),
                p_hex,
                s_hex,
                h_hex,
                if profile.is_active { 1 } else { 0 },
                created_at,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Updates an existing profile.
    pub fn update_profile(&self, profile: &PlayerProfile) -> Result<()> {
        if let Some(id) = profile.id {
            let (p_hex, s_hex, h_hex) = profile.color_scheme.to_hex_strings();
            if profile.is_active {
                self.conn.execute("UPDATE player_profiles SET is_active = 0 WHERE id != ?1", params![id])?;
            }
            self.conn.execute(
                "UPDATE player_profiles
                 SET name = ?1, alias = ?2, country = ?3, primary_color = ?4, secondary_color = ?5, helmet_color = ?6, is_active = ?7
                 WHERE id = ?8",
                params![
                    profile.name.trim(),
                    profile.alias.trim(),
                    profile.country.as_deref().map(|s| s.trim().to_uppercase()),
                    p_hex,
                    s_hex,
                    h_hex,
                    if profile.is_active { 1 } else { 0 },
                    id,
                ],
            )?;
        }
        Ok(())
    }

    /// Sets a profile as the only active profile.
    pub fn set_active_profile(&self, profile_id: i64) -> Result<()> {
        self.conn.execute("UPDATE player_profiles SET is_active = 0", [])?;
        self.conn.execute("UPDATE player_profiles SET is_active = 1 WHERE id = ?1", params![profile_id])?;
        Ok(())
    }

    /// Deletes a profile by ID. If the deleted profile was active, activates the first available profile.
    pub fn delete_profile(&self, profile_id: i64) -> Result<()> {
        let is_active: bool = self.conn.query_row(
            "SELECT is_active FROM player_profiles WHERE id = ?1",
            params![profile_id],
            |row| Ok(row.get::<_, i32>(0)? != 0),
        ).unwrap_or(false);

        self.conn.execute("DELETE FROM player_profiles WHERE id = ?1", params![profile_id])?;
        self.conn.execute("DELETE FROM race_history WHERE profile_id = ?1", params![profile_id])?;

        if is_active {
            // Activate the first remaining profile
            let remaining_id: Option<i64> = self.conn.query_row(
                "SELECT id FROM player_profiles ORDER BY id ASC LIMIT 1",
                [],
                |row| row.get(0),
            ).ok();

            if let Some(rem_id) = remaining_id {
                let _ = self.set_active_profile(rem_id);
            } else {
                let _ = self.seed_default_profile_if_empty();
            }
        }
        Ok(())
    }

    /// Seeds a default driver profile if the table is empty.
    pub fn seed_default_profile_if_empty(&self) -> Result<PlayerProfile> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM player_profiles",
            [],
            |row| row.get(0),
        )?;

        if count == 0 {
            let default_profile = PlayerProfile {
                id: None,
                name: "Racer One".to_string(),
                alias: "Apex Legend".to_string(),
                country: Some("ESP".to_string()),
                color_scheme: CarColorScheme::from_index(0),
                is_active: true,
                created_at: Utc::now().format("%Y-%m-%d %H:%M").to_string(),
            };
            let new_id = self.create_profile(&default_profile)?;
            let mut seeded = default_profile;
            seeded.id = Some(new_id);
            Ok(seeded)
        } else {
            // Return first profile if none was active
            let mut all = self.get_all_profiles()?;
            if let Some(first) = all.first_mut() {
                if !first.is_active {
                    if let Some(id) = first.id {
                        let _ = self.set_active_profile(id);
                        first.is_active = true;
                    }
                }
                Ok(first.clone())
            } else {
                Ok(PlayerProfile::default())
            }
        }
    }

    // =========================================================================
    // Career Race History & Statistics
    // =========================================================================

    /// Inserts a completed race result into the history log.
    pub fn insert_race_history(&self, record: &RaceHistoryEntry) -> Result<i64> {
        let best_lap_f64 = record.best_lap.map(|v| v as f64);
        let created_at = if record.created_at.is_empty() {
            Utc::now().format("%Y-%m-%d %H:%M").to_string()
        } else {
            record.created_at.clone()
        };

        self.conn.execute(
            "INSERT INTO race_history (profile_id, track_id, car_name, position, total_cars, total_time, best_lap, laps, is_time_attack, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                record.profile_id,
                record.track_id,
                record.car_name,
                record.position as i64,
                record.total_cars as i64,
                record.total_time as f64,
                best_lap_f64,
                record.laps,
                if record.is_time_attack { 1 } else { 0 },
                created_at,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Fetches up to `limit` recent race history records for a profile.
    pub fn get_history_for_profile(&self, profile_id: i64, limit: usize) -> Result<Vec<RaceHistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, profile_id, track_id, car_name, position, total_cars, total_time, best_lap, laps, is_time_attack, created_at
             FROM race_history
             WHERE profile_id = ?1
             ORDER BY id DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![profile_id, limit as i64], |row| {
            let is_ta: i32 = row.get(9)?;
            Ok(RaceHistoryEntry {
                id: Some(row.get(0)?),
                profile_id: row.get(1)?,
                track_id: row.get(2)?,
                car_name: row.get(3)?,
                position: row.get::<_, i64>(4)? as usize,
                total_cars: row.get::<_, i64>(5)? as usize,
                total_time: row.get::<_, f64>(6)? as f32,
                best_lap: row.get::<_, Option<f64>>(7)?.map(|v| v as f32),
                laps: row.get(8)?,
                is_time_attack: is_ta != 0,
                created_at: row.get(10)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Computes aggregated career statistics for a profile.
    pub fn get_stats_for_profile(&self, profile_id: i64) -> Result<ProfileCareerStats> {
        let all_races = self.get_history_for_profile(profile_id, 1000)?;
        Ok(ProfileCareerStats::compute(&all_races))
    }

    // =========================================================================
    // Hall of Fame Leaderboards
    // =========================================================================

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
