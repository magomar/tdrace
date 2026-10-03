//! Bot roster, careers and the headless harness. The driver AI itself lives in `race_kit::ai`
//! (spec 056) and is re-exported here, so `crate::ai::` paths keep working.

pub mod bot_harness;
pub mod career;
pub mod driver;

pub use career::{
    CareerRivalEntry, RetainedRivalReport, RosterEvolutionEngine, RosterEvolutionReport,
    SkillProgressionOutcome,
};
pub use driver::{DriverCharacter, DriverFavoriteCar, DriverPersonalityOffsets, DriverStats};
pub use race_kit::ai::humanize;
pub use race_kit::ai::{
    BotAiDriver, BotDrivingStats, BotProfile, BotRouteStrategy, DriverQuality, DriverTier, DrivingStyle, HumanDriver, HumanTraits,
    LcgRng, MistakeKind,
};
