//! Racing numbers for the grid (spec 092): one number per car, no two the same, stable across rounds.

use super::RaceSession;
use crate::ai::allocate_rival_numbers;
use crate::profile::PlayerProfile;
use crate::ui::menu::GameMode;

/// Numbers for the cars of a grid. `0` means "no number" (valid numbers are `1..=99`).
pub(super) struct GridNumbers {
    /// Player 2 in split screen.
    pub player2: u8,
    /// One per entry of `opponent_drivers`.
    pub rivals: Vec<u8>,
    /// Numbers of saved roster drivers that changed and still have to be stored (see `store_roster_numbers`).
    pub assigned: Vec<(String, u8)>,
}

impl RaceSession {
    /// The racing number of the active profile, `DEFAULT_RACE_NUMBER` if it is out of range.
    pub fn player_race_number(&self) -> u8 {
        PlayerProfile::race_number_from_i64(i64::from(self.active_profile.race_number))
    }

    /// Numbers the grid that `rebuild_roster_participants` is about to build. A championship or career roster
    /// is numbered as a whole, so a rival keeps one number in every heat and round; the caller stores the
    /// changed numbers with `store_roster_numbers`. A casual grid is numbered from the grid seed and not saved.
    pub(super) fn grid_numbers(&self, seed: u64) -> GridNumbers {
        let split = self.is_split_screen();
        let count = self.opponent_drivers.len();
        let no_numbers = GridNumbers { player2: 0, rivals: vec![0; count], assigned: Vec::new() };
        if count == 0 && !split {
            return no_numbers;
        }

        // The roster: saved drivers first, then any opponent of this grid that is not in it.
        let mut roster: Vec<(String, Option<u8>)> = if let Some(champ) = &self.championship_session {
            champ.standings.iter().filter(|s| s.driver_id != "player").map(|s| (s.driver_id.clone(), s.race_number)).collect()
        } else if self.game_mode == GameMode::Career {
            self.active_career_progress.career_rivals.iter().map(|r| (r.driver_id.clone(), r.race_number)).collect()
        } else {
            Vec::new()
        };
        let saved = roster.len();
        let rival_slots: Vec<usize> = if self.championship_session.is_none() && self.game_mode != GameMode::Career {
            roster.extend(self.opponent_drivers.iter().map(|d| (d.id.to_string(), None)));
            (0..count).collect()
        } else {
            self.opponent_drivers
                .iter()
                .map(|d| match roster.iter().position(|(id, _)| id == d.id) {
                    Some(slot) => slot,
                    None => {
                        roster.push((d.id.to_string(), None));
                        roster.len() - 1
                    }
                })
                .collect()
        };
        let player2_slot = split.then(|| {
            roster.push(("player2".to_string(), None));
            roster.len() - 1
        });

        let stored: Vec<Option<u8>> = roster.iter().map(|(_, number)| *number).collect();
        let numbers = match allocate_rival_numbers(self.player_race_number(), &stored, seed.wrapping_add(0x5EED_0092)) {
            Ok(numbers) => numbers,
            Err(e) => {
                eprintln!("racing numbers not shown: {}", e);
                return no_numbers;
            }
        };

        let assigned = if (0..saved).any(|i| stored[i] != Some(numbers[i])) {
            (0..saved).map(|i| (roster[i].0.clone(), numbers[i])).collect()
        } else {
            Vec::new()
        };

        GridNumbers {
            player2: player2_slot.map(|slot| numbers[slot]).unwrap_or(0),
            rivals: rival_slots.iter().map(|&slot| numbers[slot]).collect(),
            assigned,
        }
    }

    /// Writes numbers into the live and saved championship standings and career rivals, then saves them.
    pub(super) fn store_roster_numbers(&mut self, assigned: &[(String, u8)]) {
        if assigned.is_empty() {
            return;
        }
        let set = |id: &str| assigned.iter().find(|(driver, _)| driver == id).map(|&(_, number)| number);
        if let Some(champ) = &mut self.championship_session {
            for s in champ.standings.iter_mut() {
                if let Some(number) = set(&s.driver_id) {
                    s.race_number = Some(number);
                }
            }
        }
        if let Some(champ) = &mut self.active_career_progress.active_championship {
            for s in champ.standings.iter_mut() {
                if let Some(number) = set(&s.driver_id) {
                    s.race_number = Some(number);
                }
            }
        }
        for r in self.active_career_progress.career_rivals.iter_mut() {
            if let Some(number) = set(&r.driver_id) {
                r.race_number = Some(number);
            }
        }
        if let Some(db) = &self.hof_db {
            let _ = db.save_module_progress(&self.active_career_progress);
        }
        self.profile_module_progress.insert(self.active_career_progress.module_id.clone(), self.active_career_progress.clone());
    }
}
