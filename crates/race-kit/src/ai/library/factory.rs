//! Factory for instantiating executable bot controllers from library behaviors (Spec 137).

use super::{BotBehaviour, BotBehaviourId, BotBehaviourRegistry, ControllerPayload};
use crate::ai::BotAiDriver;

/// Factory for instantiating runnable [`BotAiDriver`] controllers from library behaviors.
pub struct DriverControllerFactory;

impl DriverControllerFactory {
    /// Instantiates a [`BotAiDriver`] configured with the behavioral payload from `behaviour`.
    pub fn create_bot_driver(behaviour: &BotBehaviour) -> BotAiDriver {
        let profile = behaviour.to_bot_profile();
        let mut driver = BotAiDriver::new(profile);

        match &behaviour.controller {
            ControllerPayload::ClassicalLookahead {
                lookahead_time,
                speed_factor: _,
                steering_kp: _,
                steering_kd: _,
                brake_margin: _,
                aggression: _,
                avoidance_distance: _,
                traits: _,
            } => {
                // Initialize target distance from lookahead time and baseline speed
                driver.current_target_dist = (*lookahead_time * 20.0).clamp(5.0, 30.0);
            }
            ControllerPayload::OptimalLineFollower {
                lookahead_distance_m,
                ..
            } => {
                driver.current_target_dist = *lookahead_distance_m;
            }
            ControllerPayload::QssMpcRollout {
                horizon_s,
                ..
            } => {
                driver.current_target_dist = (*horizon_s * 25.0).clamp(6.0, 35.0);
            }
        }

        driver
    }

    /// Looks up a behavior in `registry` by ID and instantiates the corresponding bot driver.
    pub fn create_bot_driver_by_id(
        registry: &BotBehaviourRegistry,
        id: &BotBehaviourId,
    ) -> Option<BotAiDriver> {
        registry.get(id).map(Self::create_bot_driver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{DriverTier, DrivingStyle};

    #[test]
    fn test_create_bot_driver_from_canonical_behaviour() {
        let b = BotBehaviour::canonical_baseline(DrivingStyle::Aggressive, DriverTier::Pro);
        let driver = DriverControllerFactory::create_bot_driver(&b);

        assert_eq!(driver.profile.aggression, b.to_bot_profile().aggression);
        assert_eq!(driver.profile.steering_kp, b.to_bot_profile().steering_kp);
        assert!(driver.current_target_dist > 0.0);
    }
}
