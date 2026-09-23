use std::path::Path;
use chrono::Utc;
#[cfg(not(target_arch = "wasm32"))]
use rusqlite::{params, Connection, Result};
#[cfg(target_arch = "wasm32")]
pub type Result<T> = std::result::Result<T, String>;
use serde::{Deserialize, Serialize};

use crate::profile::{ModuleCareerProgress, PlayerProfile, ProfileCareerStats, RaceHistoryEntry};
use crate::render::color::CarColorScheme;
use tdrace_core::physics::config::AssistProfile;

#[cfg(not(target_arch = "wasm32"))]
fn mode_to_str(mode: AssistProfile) -> &'static str {
    match mode {
        AssistProfile::Arcade => "arcade",
        AssistProfile::Sport => "sport",
        AssistProfile::Pro => "pro",
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn mode_from_str(s: &str) -> AssistProfile {
    match s.to_lowercase().as_str() {
        "sport" => AssistProfile::Sport,
        "pro" => AssistProfile::Pro,
        _ => AssistProfile::Arcade,
    }
}

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
#[cfg(not(target_arch = "wasm32"))]
pub struct HallOfFameDb {
    conn: Connection,
}

#[cfg(not(target_arch = "wasm32"))]
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
                created_at TEXT NOT NULL,
                last_mode TEXT NOT NULL DEFAULT 'arcade'
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
                category TEXT NOT NULL DEFAULT 'gt',
                championship_name TEXT,
                stunt_score INTEGER NOT NULL DEFAULT 0,
                collisions INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY(profile_id) REFERENCES player_profiles(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_race_history_profile ON race_history(profile_id, created_at DESC);

            CREATE TABLE IF NOT EXISTS profile_module_progress (
                profile_id INTEGER NOT NULL,
                module_id TEXT NOT NULL,
                xp INTEGER NOT NULL DEFAULT 0,
                lifetime_xp INTEGER NOT NULL DEFAULT 0,
                level INTEGER NOT NULL DEFAULT 1,
                unlocked_cars TEXT NOT NULL DEFAULT '[]',
                unlocked_tracks TEXT NOT NULL DEFAULT '[]',
                visited_tracks TEXT NOT NULL DEFAULT '[]',
                completed_events TEXT NOT NULL DEFAULT '[]',
                trophies_gold INTEGER NOT NULL DEFAULT 0,
                trophies_silver INTEGER NOT NULL DEFAULT 0,
                trophies_bronze INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (profile_id, module_id),
                FOREIGN KEY(profile_id) REFERENCES player_profiles(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_module_progress_profile ON profile_module_progress(profile_id, module_id);",
        )?;

        // Ensure backward-compatibility migration for pre-existing player_profiles tables
        let _ = self.conn.execute(
            "ALTER TABLE player_profiles ADD COLUMN last_mode TEXT NOT NULL DEFAULT 'arcade'",
            [],
        );
        let _ = self.conn.execute(
            "ALTER TABLE profile_module_progress ADD COLUMN lifetime_xp INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = self.conn.execute(
            "ALTER TABLE profile_module_progress ADD COLUMN visited_tracks TEXT NOT NULL DEFAULT '[]'",
            [],
        );
        let _ = self.conn.execute(
            "ALTER TABLE race_history ADD COLUMN category TEXT NOT NULL DEFAULT 'gt'",
            [],
        );
        let _ = self.conn.execute(
            "ALTER TABLE race_history ADD COLUMN championship_name TEXT",
            [],
        );
        let _ = self.conn.execute(
            "ALTER TABLE race_history ADD COLUMN stunt_score INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = self.conn.execute(
            "ALTER TABLE race_history ADD COLUMN collisions INTEGER NOT NULL DEFAULT 0",
            [],
        );

        Ok(())
    }

    // =========================================================================
    // Player Profile Management
    // =========================================================================

    /// Retrieves all player profiles ordered by active status descending, then creation date ascending.
    pub fn get_all_profiles(&self) -> Result<Vec<PlayerProfile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, alias, country, primary_color, secondary_color, helmet_color, is_active, created_at, COALESCE(last_mode, 'arcade')
             FROM player_profiles
             ORDER BY is_active DESC, id ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            let p_hex: String = row.get(4)?;
            let s_hex: String = row.get(5)?;
            let h_hex: String = row.get(6)?;
            let is_active_int: i32 = row.get(7)?;
            let mode_str: String = row.get(9)?;

            Ok(PlayerProfile {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                alias: row.get(2)?,
                country: row.get(3)?,
                color_scheme: CarColorScheme::from_hex_strings(&p_hex, &s_hex, &h_hex),
                is_active: is_active_int != 0,
                created_at: row.get(8)?,
                last_mode: mode_from_str(&mode_str),
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
            "SELECT id, name, alias, country, primary_color, secondary_color, helmet_color, is_active, created_at, COALESCE(last_mode, 'arcade')
             FROM player_profiles
             WHERE is_active = 1
             LIMIT 1",
        )?;

        let mut rows = stmt.query_map([], |row| {
            let p_hex: String = row.get(4)?;
            let s_hex: String = row.get(5)?;
            let h_hex: String = row.get(6)?;
            let mode_str: String = row.get(9)?;

            Ok(PlayerProfile {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                alias: row.get(2)?,
                country: row.get(3)?,
                color_scheme: CarColorScheme::from_hex_strings(&p_hex, &s_hex, &h_hex),
                is_active: true,
                created_at: row.get(8)?,
                last_mode: mode_from_str(&mode_str),
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
            "SELECT id, name, alias, country, primary_color, secondary_color, helmet_color, is_active, created_at, COALESCE(last_mode, 'arcade')
             FROM player_profiles
             WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(params![id], |row| {
            let p_hex: String = row.get(4)?;
            let s_hex: String = row.get(5)?;
            let h_hex: String = row.get(6)?;
            let is_active_int: i32 = row.get(7)?;
            let mode_str: String = row.get(9)?;

            Ok(PlayerProfile {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                alias: row.get(2)?,
                country: row.get(3)?,
                color_scheme: CarColorScheme::from_hex_strings(&p_hex, &s_hex, &h_hex),
                is_active: is_active_int != 0,
                created_at: row.get(8)?,
                last_mode: mode_from_str(&mode_str),
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
            "INSERT INTO player_profiles (name, alias, country, primary_color, secondary_color, helmet_color, is_active, created_at, last_mode)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                profile.name.trim(),
                profile.alias.trim(),
                profile.country.as_deref().map(|s| s.trim().to_uppercase()),
                p_hex,
                s_hex,
                h_hex,
                if profile.is_active { 1 } else { 0 },
                created_at,
                mode_to_str(profile.last_mode),
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
                 SET name = ?1, alias = ?2, country = ?3, primary_color = ?4, secondary_color = ?5, helmet_color = ?6, is_active = ?7, last_mode = ?8
                 WHERE id = ?9",
                params![
                    profile.name.trim(),
                    profile.alias.trim(),
                    profile.country.as_deref().map(|s| s.trim().to_uppercase()),
                    p_hex,
                    s_hex,
                    h_hex,
                    if profile.is_active { 1 } else { 0 },
                    mode_to_str(profile.last_mode),
                    id,
                ],
            )?;
        }
        Ok(())
    }

    /// Updates only the last used mode for an existing profile.
    pub fn update_profile_last_mode(&self, profile_id: i64, mode: AssistProfile) -> Result<()> {
        self.conn.execute(
            "UPDATE player_profiles SET last_mode = ?1 WHERE id = ?2",
            params![mode_to_str(mode), profile_id],
        )?;
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
                last_mode: AssistProfile::Arcade,
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
            "INSERT INTO race_history (profile_id, track_id, car_name, position, total_cars, total_time, best_lap, laps, is_time_attack, created_at, category, championship_name, stunt_score, collisions)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
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
                record.category,
                record.championship_name,
                record.stunt_score as i64,
                record.collisions as i64,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Fetches up to `limit` recent race history records for a profile.
    pub fn get_history_for_profile(&self, profile_id: i64, limit: usize) -> Result<Vec<RaceHistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, profile_id, track_id, car_name, position, total_cars, total_time, best_lap, laps, is_time_attack, created_at,
                    COALESCE(category, 'gt'), championship_name, COALESCE(stunt_score, 0), COALESCE(collisions, 0)
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
                category: row.get(11)?,
                championship_name: row.get(12)?,
                stunt_score: row.get::<_, i64>(13)? as u32,
                collisions: row.get::<_, i64>(14)? as u32,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Fetches race history filtered optionally by category discipline.
    pub fn get_history_for_profile_filtered(
        &self,
        profile_id: i64,
        category: Option<&str>,
        limit: usize,
    ) -> Result<Vec<RaceHistoryEntry>> {
        if let Some(cat) = category {
            let mut stmt = self.conn.prepare(
                "SELECT id, profile_id, track_id, car_name, position, total_cars, total_time, best_lap, laps, is_time_attack, created_at,
                        COALESCE(category, 'gt'), championship_name, COALESCE(stunt_score, 0), COALESCE(collisions, 0)
                 FROM race_history
                 WHERE profile_id = ?1 AND (category = ?2 OR (?2 = 'gt' AND category IS NULL))
                 ORDER BY id DESC
                 LIMIT ?3",
            )?;

            let rows = stmt.query_map(params![profile_id, cat, limit as i64], |row| {
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
                    category: row.get(11)?,
                    championship_name: row.get(12)?,
                    stunt_score: row.get::<_, i64>(13)? as u32,
                    collisions: row.get::<_, i64>(14)? as u32,
                })
            })?;

            let mut list = Vec::new();
            for r in rows {
                list.push(r?);
            }
            Ok(list)
        } else {
            self.get_history_for_profile(profile_id, limit)
        }
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

    /// Clears all entries from the Hall of Fame table.
    pub fn clear_hall_of_fame(&self) -> Result<()> {
        self.conn.execute("DELETE FROM hall_of_fame", [])?;
        Ok(())
    }

    /// Clears all Hall of Fame leaderboard entries for a specific track.
    pub fn clear_hall_of_fame_for_track(&self, track_id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM hall_of_fame WHERE track_id = ?1", params![track_id])?;
        Ok(())
    }

    /// Clears all race history logs for a specific track across all player profiles.
    pub fn clear_race_history_for_track(&self, track_id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM race_history WHERE track_id = ?1", params![track_id])?;
        Ok(())
    }

    /// Clears all race history logs for a specific championship across all player profiles.
    pub fn clear_race_history_for_championship(&self, championship_name: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM race_history WHERE championship_name = ?1 COLLATE NOCASE",
            params![championship_name],
        )?;
        Ok(())
    }

    /// Deletes the latest race history entry for a specific championship.
    pub fn delete_latest_race_history_entry_for_championship(&self, championship_name: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM race_history WHERE id IN (SELECT id FROM race_history WHERE championship_name = ?1 COLLATE NOCASE ORDER BY id DESC LIMIT 1)",
            params![championship_name],
        )?;
        Ok(())
    }

    /// Clears all Hall of Fame records and race history logs for a specific track.
    pub fn clear_track_history(&self, track_id: &str) -> Result<()> {
        self.clear_hall_of_fame_for_track(track_id)?;
        self.clear_race_history_for_track(track_id)?;
        Ok(())
    }

    /// Clears all race history logs for a specific player profile.
    pub fn clear_history_for_profile(&self, profile_id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM race_history WHERE profile_id = ?1", params![profile_id])?;
        Ok(())
    }

    /// Clears all Hall of Fame entries recorded by a driver alias (including split-screen P1 suffix).
    pub fn clear_hall_of_fame_for_driver(&self, driver_alias: &str) -> Result<()> {
        let p1_suffix = format!("{} (P1)", driver_alias);
        self.conn.execute(
            "DELETE FROM hall_of_fame WHERE player_name = ?1 OR player_name = ?2",
            params![driver_alias, p1_suffix],
        )?;
        Ok(())
    }

    /// Clears all historical data (race history logs and Hall of Fame entries) for a specific player profile.
    pub fn clear_profile_historical_data(&self, profile_id: i64, driver_alias: &str) -> Result<()> {
        self.clear_history_for_profile(profile_id)?;
        self.clear_hall_of_fame_for_driver(driver_alias)?;
        Ok(())
    }

    /// Clears all Hall of Fame leaderboard records generated by AI bot drivers.
    /// Preserves any records belonging to human player profiles.
    pub fn clear_bot_hall_of_fame(&self) -> Result<()> {
        self.conn.execute(
            "DELETE FROM hall_of_fame
             WHERE player_name NOT IN (
                 SELECT name FROM player_profiles
                 UNION
                 SELECT alias FROM player_profiles
                 UNION
                 SELECT alias || ' (P1)' FROM player_profiles
                 UNION
                 SELECT 'Player 2 (P2)'
             )",
            [],
        )?;
        Ok(())
    }

    /// Seeds default records if needed (currently a clean no-op, preserving real race records).
    pub fn seed_defaults_if_empty(&self, _track_id: &str) -> Result<()> {
        // Real race results are logged dynamically on session completion.
        Ok(())
    }

    // =========================================================================
    // Module Career Progress Management
    // =========================================================================

    /// Retrieves module career progress for a given player profile and module ID.
    pub fn get_module_progress(&self, profile_id: i64, module_id: &str) -> Result<Option<ModuleCareerProgress>> {
        let mut stmt = self.conn.prepare(
            "SELECT profile_id, module_id, xp, level, unlocked_cars, unlocked_tracks, completed_events,
                    trophies_gold, trophies_silver, trophies_bronze, updated_at,
                    COALESCE(lifetime_xp, xp), COALESCE(visited_tracks, '[]')
             FROM profile_module_progress
             WHERE profile_id = ?1 AND module_id = ?2",
        )?;

        let mut rows = stmt.query_map(params![profile_id, module_id], |row| {
            let cars_json: String = row.get(4)?;
            let tracks_json: String = row.get(5)?;
            let events_json: String = row.get(6)?;
            let lifetime_xp: i64 = row.get(11)?;
            let visited_json: String = row.get(12)?;

            let unlocked_cars: Vec<String> = serde_json::from_str(&cars_json).unwrap_or_default();
            let unlocked_tracks: Vec<String> = serde_json::from_str(&tracks_json).unwrap_or_default();
            let visited_tracks: Vec<String> = serde_json::from_str(&visited_json).unwrap_or_default();
            let completed_events: Vec<String> = serde_json::from_str(&events_json).unwrap_or_default();

            Ok(ModuleCareerProgress {
                profile_id: row.get(0)?,
                module_id: row.get(1)?,
                xp: row.get::<_, i64>(2)? as u64,
                lifetime_xp: lifetime_xp as u64,
                level: row.get::<_, i64>(3)? as u32,
                unlocked_cars,
                unlocked_tracks,
                visited_tracks,
                completed_events,
                trophies_gold: row.get::<_, i64>(7)? as u32,
                trophies_silver: row.get::<_, i64>(8)? as u32,
                trophies_bronze: row.get::<_, i64>(9)? as u32,
                updated_at: row.get(10)?,
            })
        })?;

        if let Some(res) = rows.next() {
            Ok(Some(res?))
        } else {
            Ok(None)
        }
    }

    /// Saves or updates module career progress for a player profile.
    pub fn save_module_progress(&self, progress: &ModuleCareerProgress) -> Result<()> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let cars_json = serde_json::to_string(&progress.unlocked_cars).unwrap_or_else(|_| "[]".to_string());
        let tracks_json = serde_json::to_string(&progress.unlocked_tracks).unwrap_or_else(|_| "[]".to_string());
        let visited_json = serde_json::to_string(&progress.visited_tracks).unwrap_or_else(|_| "[]".to_string());
        let events_json = serde_json::to_string(&progress.completed_events).unwrap_or_else(|_| "[]".to_string());

        self.conn.execute(
            "INSERT INTO profile_module_progress (
                profile_id, module_id, xp, lifetime_xp, level, unlocked_cars, unlocked_tracks, visited_tracks, completed_events,
                trophies_gold, trophies_silver, trophies_bronze, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(profile_id, module_id) DO UPDATE SET
                xp = excluded.xp,
                lifetime_xp = excluded.lifetime_xp,
                level = excluded.level,
                unlocked_cars = excluded.unlocked_cars,
                unlocked_tracks = excluded.unlocked_tracks,
                visited_tracks = excluded.visited_tracks,
                completed_events = excluded.completed_events,
                trophies_gold = excluded.trophies_gold,
                trophies_silver = excluded.trophies_silver,
                trophies_bronze = excluded.trophies_bronze,
                updated_at = excluded.updated_at",
            params![
                progress.profile_id,
                progress.module_id,
                progress.xp as i64,
                progress.lifetime_xp as i64,
                progress.level as i64,
                cars_json,
                tracks_json,
                visited_json,
                events_json,
                progress.trophies_gold as i64,
                progress.trophies_silver as i64,
                progress.trophies_bronze as i64,
                now
            ],
        )?;
        Ok(())
    }

    /// Returns existing career progress for profile and module, or creates and persists default starter progress.
    pub fn get_or_create_module_progress(&self, profile_id: i64, module_id: &str) -> Result<ModuleCareerProgress> {
        if let Some(mut p) = self.get_module_progress(profile_id, module_id)? {
            p.sync_unlocks_for_level();
            Ok(p)
        } else {
            let def = ModuleCareerProgress::default_for_module(profile_id, module_id);
            self.save_module_progress(&def)?;
            Ok(def)
        }
    }
}

/// In-memory fallback persistence manager for WebAssembly targets.
#[cfg(target_arch = "wasm32")]
pub struct HallOfFameDb {
    profiles: std::sync::Mutex<Vec<PlayerProfile>>,
    history: std::sync::Mutex<Vec<RaceHistoryEntry>>,
    hof: std::sync::Mutex<Vec<HallOfFameEntry>>,
    progress: std::sync::Mutex<Vec<ModuleCareerProgress>>,
}

#[cfg(target_arch = "wasm32")]
impl HallOfFameDb {
    pub const DEFAULT_DB_PATH: &'static str = "tdrace_records.db";

    pub fn open_default() -> Result<Self> {
        Ok(Self::new_in_memory())
    }

    pub fn open(_path: &Path) -> Result<Self> {
        Ok(Self::new_in_memory())
    }

    pub fn open_in_memory() -> Result<Self> {
        Ok(Self::new_in_memory())
    }

    fn new_in_memory() -> Self {
        Self {
            profiles: std::sync::Mutex::new(Vec::new()),
            history: std::sync::Mutex::new(Vec::new()),
            hof: std::sync::Mutex::new(Vec::new()),
            progress: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn get_all_profiles(&self) -> Result<Vec<PlayerProfile>> {
        let list = self.profiles.lock().unwrap().clone();
        Ok(list)
    }

    pub fn get_active_profile(&self) -> Result<PlayerProfile> {
        let guard = self.profiles.lock().unwrap();
        if let Some(p) = guard.iter().find(|p| p.is_active) {
            Ok(p.clone())
        } else {
            drop(guard);
            self.seed_default_profile_if_empty()
        }
    }

    pub fn get_profile_by_id(&self, id: i64) -> Result<Option<PlayerProfile>> {
        let guard = self.profiles.lock().unwrap();
        Ok(guard.iter().find(|p| p.id == Some(id)).cloned())
    }

    pub fn create_profile(&self, profile: &PlayerProfile) -> Result<i64> {
        let mut guard = self.profiles.lock().unwrap();
        let new_id = (guard.len() as i64) + 1;
        if profile.is_active {
            for p in guard.iter_mut() {
                p.is_active = false;
            }
        }
        let mut p = profile.clone();
        p.id = Some(new_id);
        guard.push(p);
        Ok(new_id)
    }

    pub fn update_profile(&self, profile: &PlayerProfile) -> Result<()> {
        if let Some(id) = profile.id {
            let mut guard = self.profiles.lock().unwrap();
            if profile.is_active {
                for p in guard.iter_mut() {
                    if p.id != Some(id) {
                        p.is_active = false;
                    }
                }
            }
            if let Some(p) = guard.iter_mut().find(|p| p.id == Some(id)) {
                *p = profile.clone();
            }
        }
        Ok(())
    }

    pub fn set_active_profile(&self, profile_id: i64) -> Result<()> {
        let mut guard = self.profiles.lock().unwrap();
        for p in guard.iter_mut() {
            p.is_active = p.id == Some(profile_id);
        }
        Ok(())
    }

    pub fn delete_profile(&self, profile_id: i64) -> Result<()> {
        let mut guard = self.profiles.lock().unwrap();
        let was_active = guard.iter().find(|p| p.id == Some(profile_id)).map(|p| p.is_active).unwrap_or(false);
        guard.retain(|p| p.id != Some(profile_id));
        if was_active {
            if let Some(first) = guard.first_mut() {
                first.is_active = true;
            }
        }
        Ok(())
    }

    pub fn update_profile_last_mode(&self, profile_id: i64, mode: AssistProfile) -> Result<()> {
        let mut guard = self.profiles.lock().unwrap();
        if let Some(p) = guard.iter_mut().find(|p| p.id == Some(profile_id)) {
            p.last_mode = mode;
        }
        Ok(())
    }

    pub fn seed_default_profile_if_empty(&self) -> Result<PlayerProfile> {
        let mut guard = self.profiles.lock().unwrap();
        if guard.is_empty() {
            let default_profile = PlayerProfile {
                id: Some(1),
                name: "Racer One".to_string(),
                alias: "Apex Legend".to_string(),
                country: Some("ESP".to_string()),
                color_scheme: CarColorScheme::from_index(0),
                is_active: true,
                created_at: Utc::now().format("%Y-%m-%d %H:%M").to_string(),
                last_mode: AssistProfile::Arcade,
            };
            guard.push(default_profile.clone());
            Ok(default_profile)
        } else {
            Ok(guard.first().cloned().unwrap_or_default())
        }
    }

    pub fn insert_race_history(&self, record: &RaceHistoryEntry) -> Result<i64> {
        let mut guard = self.history.lock().unwrap();
        let new_id = (guard.len() as i64) + 1;
        let mut r = record.clone();
        r.id = Some(new_id);
        guard.push(r);
        Ok(new_id)
    }

    pub fn get_history_for_profile(&self, profile_id: i64, limit: usize) -> Result<Vec<RaceHistoryEntry>> {
        self.get_history_for_profile_filtered(profile_id, None, limit)
    }

    pub fn get_history_for_profile_filtered(
        &self,
        profile_id: i64,
        category: Option<&str>,
        limit: usize,
    ) -> Result<Vec<RaceHistoryEntry>> {
        let guard = self.history.lock().unwrap();
        let items: Vec<RaceHistoryEntry> = guard
            .iter()
            .filter(|r| {
                r.profile_id == profile_id
                    && category.map_or(true, |c| r.category == c || (c == "gt" && r.category.is_empty()))
            })
            .rev()
            .take(limit)
            .cloned()
            .collect();
        Ok(items)
    }

    pub fn get_stats_for_profile(&self, profile_id: i64) -> Result<ProfileCareerStats> {
        let races = self.get_history_for_profile(profile_id, 1000)?;
        Ok(ProfileCareerStats::compute(&races))
    }

    pub fn get_top_10(&self, track_id: &str) -> Result<Vec<HallOfFameEntry>> {
        let guard = self.hof.lock().unwrap();
        let mut matching: Vec<HallOfFameEntry> = guard.iter().filter(|e| e.track_id == track_id).cloned().collect();
        matching.sort_by(|a, b| a.total_time.partial_cmp(&b.total_time).unwrap_or(std::cmp::Ordering::Equal));
        matching.truncate(10);
        Ok(matching)
    }

    pub fn is_top_10(&self, track_id: &str, total_time: f32) -> Result<bool> {
        let top = self.get_top_10(track_id)?;
        if top.len() < 10 {
            Ok(true)
        } else {
            Ok(total_time < top[9].total_time)
        }
    }

    pub fn insert_entry(&self, entry: &HallOfFameEntry) -> Result<i64> {
        let mut guard = self.hof.lock().unwrap();
        let new_id = (guard.len() as i64) + 1;
        let mut e = entry.clone();
        e.id = Some(new_id);
        guard.push(e);
        Ok(new_id)
    }

    pub fn clear_hall_of_fame(&self) -> Result<()> {
        let mut guard = self.hof.lock().unwrap();
        guard.clear();
        Ok(())
    }

    pub fn clear_hall_of_fame_for_track(&self, track_id: &str) -> Result<()> {
        let mut guard = self.hof.lock().unwrap();
        guard.retain(|e| e.track_id != track_id);
        Ok(())
    }

    pub fn clear_race_history_for_track(&self, track_id: &str) -> Result<()> {
        let mut guard = self.history.lock().unwrap();
        guard.retain(|r| r.track_id != track_id);
        Ok(())
    }

    pub fn clear_race_history_for_championship(&self, championship_name: &str) -> Result<()> {
        let mut guard = self.history.lock().unwrap();
        guard.retain(|r| {
            !r.championship_name
                .as_deref()
                .is_some_and(|n| n.eq_ignore_ascii_case(championship_name))
        });
        Ok(())
    }

    pub fn clear_track_history(&self, track_id: &str) -> Result<()> {
        self.clear_hall_of_fame_for_track(track_id)?;
        self.clear_race_history_for_track(track_id)?;
        Ok(())
    }

    pub fn clear_history_for_profile(&self, profile_id: i64) -> Result<()> {
        let mut guard = self.history.lock().unwrap();
        guard.retain(|r| r.profile_id != profile_id);
        Ok(())
    }

    pub fn clear_hall_of_fame_for_driver(&self, driver_alias: &str) -> Result<()> {
        let mut guard = self.hof.lock().unwrap();
        let p1_suffix = format!("{} (P1)", driver_alias);
        guard.retain(|e| e.player_name != driver_alias && e.player_name != p1_suffix);
        Ok(())
    }

    pub fn clear_profile_historical_data(&self, profile_id: i64, driver_alias: &str) -> Result<()> {
        self.clear_history_for_profile(profile_id)?;
        self.clear_hall_of_fame_for_driver(driver_alias)?;
        Ok(())
    }

    pub fn clear_bot_hall_of_fame(&self) -> Result<()> {
        let profiles_guard = self.profiles.lock().unwrap();
        let mut human_names = std::collections::HashSet::new();
        for p in profiles_guard.iter() {
            human_names.insert(p.name.to_lowercase());
            human_names.insert(p.alias.to_lowercase());
            human_names.insert(format!("{} (p1)", p.alias).to_lowercase());
        }
        human_names.insert("player 2 (p2)".to_string());
        drop(profiles_guard);

        let mut guard = self.hof.lock().unwrap();
        guard.retain(|e| human_names.contains(&e.player_name.to_lowercase()));
        Ok(())
    }

    pub fn seed_defaults_if_empty(&self, _track_id: &str) -> Result<()> {
        Ok(())
    }

    pub fn get_module_progress(&self, profile_id: i64, module_id: &str) -> Result<Option<ModuleCareerProgress>> {
        let guard = self.progress.lock().unwrap();
        let res = guard
            .iter()
            .find(|p| p.profile_id == profile_id && p.module_id == module_id)
            .cloned();
        Ok(res)
    }

    pub fn save_module_progress(&self, progress: &ModuleCareerProgress) -> Result<()> {
        let mut guard = self.progress.lock().unwrap();
        if let Some(pos) = guard.iter().position(|p| p.profile_id == progress.profile_id && p.module_id == progress.module_id) {
            guard[pos] = progress.clone();
        } else {
            guard.push(progress.clone());
        }
        Ok(())
    }

    pub fn get_or_create_module_progress(&self, profile_id: i64, module_id: &str) -> Result<ModuleCareerProgress> {
        if let Some(mut p) = self.get_module_progress(profile_id, module_id)? {
            p.sync_unlocks_for_level();
            Ok(p)
        } else {
            let def = ModuleCareerProgress::default_for_module(profile_id, module_id);
            self.save_module_progress(&def)?;
            Ok(def)
        }
    }
}

