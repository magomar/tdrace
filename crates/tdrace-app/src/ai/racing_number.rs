//! Racing numbers (spec 092): the player picks one, every AI rival in a roster gets a different one.
//!
//! Valid numbers are `1..=99`. `0` is never a valid number; the grid and the renderer use it for
//! "no number".

use std::collections::HashSet;
use std::fmt;

use serde::{Deserialize, Deserializer};

use crate::ai::LcgRng;

/// Smallest racing number a driver can have.
pub const MIN_RACE_NUMBER: u8 = 1;
/// Largest racing number a driver can have.
pub const MAX_RACE_NUMBER: u8 = 99;

/// Most rivals one roster can hold: every number except the player's.
pub const MAX_RIVALS: usize = (MAX_RACE_NUMBER - MIN_RACE_NUMBER) as usize;

/// True when `number` is a number a driver may carry.
pub fn is_valid_race_number(number: u8) -> bool {
    (MIN_RACE_NUMBER..=MAX_RACE_NUMBER).contains(&number)
}

/// Serde reader for a rival's saved number: a missing, null or out-of-range value loads as `None`
/// ("no number yet") instead of failing the whole save.
pub fn deserialize_stored_race_number<'de, D>(deserializer: D) -> Result<Option<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<i64>::deserialize(deserializer)?;
    Ok(raw.and_then(|n| u8::try_from(n).ok()).filter(|&n| is_valid_race_number(n)))
}

/// The roster has more rivals than there are numbers, so no unique numbering exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RosterTooLarge {
    pub rivals: usize,
}

impl fmt::Display for RosterTooLarge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} rivals cannot all have a racing number (most is {})", self.rivals, MAX_RIVALS)
    }
}

impl std::error::Error for RosterTooLarge {}

/// Fills in missing numbers and fixes collisions. Returns one number per rival, in order.
/// `stored[i]` is the number already saved for rival `i`, if any.
///
/// A stored number is kept when it is valid, differs from `player` and has not appeared earlier in
/// `stored`. Every other rival takes the next number from the unused numbers, shuffled with
/// `LcgRng::new(seed)`. Pure and deterministic: the same inputs give the same output.
pub fn allocate_rival_numbers(player: u8, stored: &[Option<u8>], seed: u64) -> Result<Vec<u8>, RosterTooLarge> {
    if stored.len() > MAX_RIVALS {
        return Err(RosterTooLarge { rivals: stored.len() });
    }

    let mut used: HashSet<u8> = HashSet::new();
    if is_valid_race_number(player) {
        used.insert(player);
    }
    let kept: Vec<Option<u8>> = stored
        .iter()
        .map(|slot| slot.filter(|&n| is_valid_race_number(n) && used.insert(n)))
        .collect();

    let mut pool: Vec<u8> = (MIN_RACE_NUMBER..=MAX_RACE_NUMBER).filter(|n| !used.contains(n)).collect();
    LcgRng::new(seed).shuffle(&mut pool);
    let mut pool = pool.into_iter();

    // `stored.len() <= MAX_RIVALS`, so the pool always covers every missing slot.
    kept.into_iter()
        .map(|slot| slot.or_else(|| pool.next()).ok_or(RosterTooLarge { rivals: stored.len() }))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_unique(numbers: &[u8]) -> bool {
        numbers.iter().collect::<HashSet<_>>().len() == numbers.len()
    }

    #[test]
    fn valid_range_is_one_to_ninety_nine() {
        assert!(!is_valid_race_number(0));
        assert!(is_valid_race_number(1));
        assert!(is_valid_race_number(99));
        assert!(!is_valid_race_number(100));
        assert!(!is_valid_race_number(255));
    }

    #[test]
    fn old_save_without_numbers_gets_unique_numbers_different_from_the_player() {
        let numbers = allocate_rival_numbers(17, &[None; 7], 1234).unwrap();
        assert_eq!(numbers.len(), 7);
        assert!(all_unique(&numbers));
        assert!(numbers.iter().all(|&n| is_valid_race_number(n) && n != 17));
    }

    #[test]
    fn stored_numbers_are_kept() {
        let stored = [Some(5), Some(42), None, Some(99)];
        let numbers = allocate_rival_numbers(7, &stored, 9).unwrap();
        assert_eq!((numbers[0], numbers[1], numbers[3]), (5, 42, 99));
        assert!(all_unique(&numbers));
        assert!(![5, 42, 99, 7].contains(&numbers[2]));
    }

    #[test]
    fn only_the_rival_that_collides_with_the_player_changes() {
        let stored = [Some(3), Some(42), Some(8)];
        let numbers = allocate_rival_numbers(42, &stored, 77).unwrap();
        assert_eq!(numbers[0], 3);
        assert_eq!(numbers[2], 8);
        assert_ne!(numbers[1], 42);
        assert!(is_valid_race_number(numbers[1]));
        assert!(all_unique(&numbers));
    }

    #[test]
    fn earlier_duplicate_wins() {
        let stored = [Some(12), Some(30), Some(12), Some(30)];
        let numbers = allocate_rival_numbers(1, &stored, 5).unwrap();
        assert_eq!((numbers[0], numbers[1]), (12, 30));
        assert!(![12, 30, 1].contains(&numbers[2]));
        assert!(![12, 30, 1].contains(&numbers[3]));
        assert!(all_unique(&numbers));
    }

    #[test]
    fn out_of_range_stored_numbers_count_as_missing() {
        let stored = [Some(0), Some(100), Some(200), Some(10)];
        let numbers = allocate_rival_numbers(1, &stored, 5).unwrap();
        assert_eq!(numbers[3], 10);
        assert!(numbers.iter().all(|&n| is_valid_race_number(n)));
        assert!(all_unique(&numbers));
    }

    #[test]
    fn an_invalid_player_number_reserves_nothing() {
        let err = allocate_rival_numbers(0, &[None; 99], 5).unwrap_err();
        assert_eq!(err.rivals, 99);
        let numbers = allocate_rival_numbers(0, &[None; 5], 5).unwrap();
        assert!(numbers.iter().all(|&n| is_valid_race_number(n)));
        assert!(all_unique(&numbers));
    }

    #[test]
    fn same_inputs_give_the_same_numbers() {
        let stored = [None, Some(14), None, None, Some(14)];
        let first = allocate_rival_numbers(23, &stored, 2024).unwrap();
        let second = allocate_rival_numbers(23, &stored, 2024).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn a_different_seed_gives_a_different_order() {
        let a = allocate_rival_numbers(1, &[None; 31], 1).unwrap();
        let b = allocate_rival_numbers(1, &[None; 31], 2).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn a_full_tournament_roster_is_unique() {
        for size in [7, 31, 63] {
            let numbers = allocate_rival_numbers(17, &vec![None; size], 4242).unwrap();
            assert_eq!(numbers.len(), size);
            assert!(all_unique(&numbers), "{size} rivals share a number");
            assert!(numbers.iter().all(|&n| is_valid_race_number(n) && n != 17));
        }
    }

    #[test]
    fn ninety_eight_rivals_use_every_number_but_the_players() {
        let numbers = allocate_rival_numbers(50, &[None; 98], 3).unwrap();
        assert!(all_unique(&numbers));
        assert!(!numbers.contains(&50));
        assert_eq!(numbers.len(), 98);
    }

    #[test]
    fn ninety_nine_rivals_are_too_many_and_do_not_panic() {
        let err = allocate_rival_numbers(50, &[None; 99], 3).unwrap_err();
        assert_eq!(err, RosterTooLarge { rivals: 99 });
        // Stored numbers in an oversized roster do not turn it into a numbered one with duplicates.
        let stored: Vec<Option<u8>> = (0..120).map(|i| Some((i % 99) as u8 + 1)).collect();
        assert!(allocate_rival_numbers(1, &stored, 3).is_err());
    }

    #[test]
    fn an_empty_roster_is_fine() {
        assert_eq!(allocate_rival_numbers(7, &[], 1).unwrap(), Vec::<u8>::new());
    }
}
