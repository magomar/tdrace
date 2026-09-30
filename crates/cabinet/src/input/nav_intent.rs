use serde::{Deserialize, Serialize};

/// High-level normalized navigation action across keyboard, gamepad, and mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavAction {
    None,
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Cancel,
    BumperLeft,
    BumperRight,
    Number(usize),
}

/// Normalized navigation intent bundling action and repeated flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavIntent {
    pub action: NavAction,
    pub repeated: bool,
}

impl NavIntent {
    pub const NONE: Self = Self {
        action: NavAction::None,
        repeated: false,
    };

    pub fn new(action: NavAction, repeated: bool) -> Self {
        Self { action, repeated }
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        self.action == NavAction::None
    }

    #[inline]
    pub fn is_up(&self) -> bool {
        self.action == NavAction::Up
    }

    #[inline]
    pub fn is_down(&self) -> bool {
        self.action == NavAction::Down
    }

    #[inline]
    pub fn is_left(&self) -> bool {
        self.action == NavAction::Left
    }

    #[inline]
    pub fn is_right(&self) -> bool {
        self.action == NavAction::Right
    }

    #[inline]
    pub fn is_confirm(&self) -> bool {
        self.action == NavAction::Confirm
    }

    #[inline]
    pub fn is_cancel(&self) -> bool {
        self.action == NavAction::Cancel
    }

    #[inline]
    pub fn is_bumper_left(&self) -> bool {
        self.action == NavAction::BumperLeft
    }

    #[inline]
    pub fn is_bumper_right(&self) -> bool {
        self.action == NavAction::BumperRight
    }

    #[inline]
    pub fn number(&self) -> Option<usize> {
        if let NavAction::Number(n) = self.action {
            Some(n)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nav_intent_predicates() {
        let up = NavIntent::new(NavAction::Up, false);
        assert!(up.is_up());
        assert!(!up.is_down());
        assert!(!up.repeated);

        let rep_right = NavIntent::new(NavAction::Right, true);
        assert!(rep_right.is_right());
        assert!(rep_right.repeated);

        let num = NavIntent::new(NavAction::Number(4), false);
        assert_eq!(num.number(), Some(4));
    }
}
