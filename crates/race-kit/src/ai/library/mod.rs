//! Bot behaviour library and provenance registry (Spec 137).
//!
//! Provides a persistent, queryable library of bot behaviors so algorithms (such as Spec 133
//! QSS+MPC, Spec 132 human imitation, or Spec 083 EvoRL sweeps) do not destructively replace
//! existing controllers, but instead coexist as selectable assets with full creation provenance,
//! tactical persona specifications, and multi-scenario performance summaries.

pub mod factory;
pub mod registry;

pub use factory::DriverControllerFactory;
pub use registry::{BotBehaviourRegistry, BotQuery, BotQueryBuilder};

use std::collections::HashSet;
use serde::{Deserialize, Serialize};

use super::{BotProfile, DriverQuality, DriverTier, DrivingStyle, HumanTraits};

/// Unique identifier for a bot behavior asset.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BotBehaviourId(pub String);

impl std::fmt::Display for BotBehaviourId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for BotBehaviourId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for BotBehaviourId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Lifecycle status of a behavior entry in the library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BehaviourStatus {
    /// Active behavior approved for general game selection and career grids.
    Active,
    /// Candidate behavior undergoing simulation matrix evaluation or tuning.
    Candidate,
    /// Frozen golden standard used as a benchmark reference across versions.
    Baseline,
    /// Preserved historical behavior retained for compatibility or niche scenarios.
    Archived,
    /// Marked as obsolete or replaced for production use, but kept for evaluation.
    Deprecated,
}

impl Default for BehaviourStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// Structured record of the engineering method(s) used to create this behavior.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CreationMethod {
    /// Handcrafted heuristic tuning of lookahead, PID gains, and braking margins.
    HandcraftedHeuristic {
        engineer: String,
        notes: String,
    },
    /// Evolutionary search (EvoRL / CMA-ES / Genetic Algorithm) over parameter spaces.
    EvolutionarySearch {
        pipeline: String,
        generation: u32,
        fitness_metric: String,
        best_fitness: f32,
        seed: u64,
        source_run_id: String,
    },
    /// Quasi-Steady-State friction-circle solver with short-horizon MPC rollouts (Spec 133).
    PhysicsExactQssMpc {
        solver_version: String,
        horizon_seconds: f32,
        rollout_candidates: usize,
        friction_circle_safety_margin: f32,
    },
    /// Behavior cloned or warm-started from human player telemetry replays (Spec 132).
    HumanImitationCloning {
        source_replay_path: String,
        driver_alias: String,
        trajectory_similarity_score: f32,
    },
    /// Deep Reinforcement Learning neural network policy (PPO / SAC / Gymnasium).
    ReinforcementLearningPolicy {
        framework: String,
        checkpoint_path: String,
        observation_space: String,
        action_space: String,
    },
    /// Multi-stage composite pipeline (e.g. EvoRL tuning applied on top of QSS solver).
    CompositeHybrid {
        primary_method: Box<CreationMethod>,
        secondary_stage: String,
        notes: String,
    },
}

impl Default for CreationMethod {
    fn default() -> Self {
        Self::HandcraftedHeuristic {
            engineer: "tdrace-core".to_string(),
            notes: "Canonical default driver tuning".to_string(),
        }
    }
}

/// Tactical philosophy, intended operational niche, and target difficulty tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrivingPersona {
    /// Target tactical philosophy (Spec 023).
    pub style: DrivingStyle,
    /// Target experience and performance ladder tier (Spec 023 / Spec 131).
    pub tier: DriverTier,
    /// Intended motorsport module specialization (e.g. "classic", "karting", "rallycross", "universal").
    pub modality_specialization: String,
    /// High-level description of behavioral nuances.
    pub tactical_notes: String,
}

impl Default for DrivingPersona {
    fn default() -> Self {
        Self {
            style: DrivingStyle::Balanced,
            tier: DriverTier::Pro,
            modality_specialization: "universal".to_string(),
            tactical_notes: "Balanced baseline persona".to_string(),
        }
    }
}

/// Multi-candidate MPC cost weights for short-horizon trajectory rollouts (Spec 133).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MpcCostWeights {
    pub progress_weight: f32,
    pub track_boundary_penalty: f32,
    pub vehicle_collision_penalty: f32,
    pub yaw_acceleration_penalty: f32,
}

impl Default for MpcCostWeights {
    fn default() -> Self {
        Self {
            progress_weight: 1.0,
            track_boundary_penalty: 50.0,
            vehicle_collision_penalty: 100.0,
            yaw_acceleration_penalty: 5.0,
        }
    }
}

/// Underlying runtime controller implementation family and configuration payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "controller_kind", rename_all = "snake_case")]
pub enum ControllerPayload {
    /// Classical single-point lookahead PD controller with heuristic braking scan.
    ClassicalLookahead {
        lookahead_time: f32,
        speed_factor: f32,
        steering_kp: f32,
        steering_kd: f32,
        brake_margin: f32,
        aggression: f32,
        avoidance_distance: f32,
        #[serde(default)]
        traits: HumanTraits,
    },
    /// Optimal racing line follower with unchoked corner exit (Spec 131).
    OptimalLineFollower {
        speed_factor: f32,
        steering_kp: f32,
        steering_kd: f32,
        brake_margin: f32,
        aggression: f32,
        avoidance_distance: f32,
        lookahead_distance_m: f32,
        #[serde(default)]
        traits: HumanTraits,
    },
    /// Physics-exact forward-backward QSS solver with online grip memory & MPC rollout (Spec 133).
    QssMpcRollout {
        speed_factor: f32,
        steering_kp: f32,
        steering_kd: f32,
        brake_margin: f32,
        aggression: f32,
        avoidance_distance: f32,
        horizon_s: f32,
        candidate_count: usize,
        #[serde(default)]
        cost_weights: MpcCostWeights,
        #[serde(default)]
        traits: HumanTraits,
    },
}

impl Default for ControllerPayload {
    fn default() -> Self {
        let p = BotProfile::pro();
        Self::ClassicalLookahead {
            lookahead_time: p.lookahead_time,
            speed_factor: p.speed_factor,
            steering_kp: p.steering_kp,
            steering_kd: p.steering_kd,
            brake_margin: p.brake_margin,
            aggression: p.aggression,
            avoidance_distance: p.avoidance_distance,
            traits: p.traits,
        }
    }
}

/// Summary metrics aggregated across evaluations.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct PerformanceSummaryMetrics {
    /// Fastest single flying lap time across benchmark runs (s).
    pub best_flying_lap_s: Option<f32>,
    /// Mean flying lap time across benchmark runs (s).
    pub mean_flying_lap_s: Option<f32>,
    /// Percentage delta to theoretical optimal lap limit.
    pub pace_delta_to_optimal_pct: Option<f32>,
    /// Average vehicle-to-vehicle contacts per lap in multi-car simulations.
    pub contacts_per_lap: Option<f32>,
    /// Ratio of clean overtakes to total passing attempts [0.0..1.0].
    pub clean_pass_ratio: Option<f32>,
    /// Perceived human believability rating [0.0..1.0].
    pub human_believability_score: Option<f32>,
}

fn default_behaviour_version() -> String {
    "1.0.0".to_string()
}

/// The complete, self-documenting Bot Behaviour asset entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BotBehaviour {
    /// Unique behavioral identifier.
    pub id: BotBehaviourId,
    /// User-facing display title.
    pub name: String,
    /// Semantic version or revision string (e.g. "1.0.0").
    #[serde(default = "default_behaviour_version")]
    pub version: String,
    /// Lifecycle status.
    pub status: BehaviourStatus,
    /// Creation timestamp (ISO 8601 UTC) or release date.
    #[serde(alias = "date")]
    pub created_at: String,
    /// Optional timestamp when this behavior was updated, calibrated, or retuned.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Author, engineer, or automated system that produced this behavior.
    pub author: String,
    /// Searchable categorical tags.
    #[serde(default)]
    pub tags: HashSet<String>,
    /// Summary narrative of behavioral traits and intended use cases.
    pub description: String,
    /// Structured creation provenance.
    pub provenance: CreationMethod,
    /// Behavioral persona and target driving style.
    pub persona: DrivingPersona,
    /// Executable controller configuration and weights.
    pub controller: ControllerPayload,
    /// Cached aggregate driving performance summary.
    #[serde(default)]
    pub performance_summary: Option<PerformanceSummaryMetrics>,
}

impl BotBehaviour {
    /// Effective date of this behavior (latest of updated_at or created_at).
    pub fn date(&self) -> &str {
        self.updated_at.as_deref().unwrap_or(&self.created_at)
    }

    /// Sets the semantic version or revision string.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Sets the creation timestamp or date string.
    pub fn with_created_at(mut self, date: impl Into<String>) -> Self {
        self.created_at = date.into();
        self
    }

    /// Sets the updated timestamp or date string.
    pub fn with_updated_at(mut self, date: impl Into<String>) -> Self {
        self.updated_at = Some(date.into());
        self
    }

    /// Sets the effective date (convenience alias for with_created_at).
    pub fn with_date(self, date: impl Into<String>) -> Self {
        self.with_created_at(date)
    }

    /// Constructs a standard behavior from a [`BotProfile`], target style, and tier.
    pub fn from_profile(
        id: impl Into<BotBehaviourId>,
        name: impl Into<String>,
        persona: DrivingPersona,
        profile: BotProfile,
        author: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let mut tags = HashSet::new();
        tags.insert(persona.style.as_str().to_string());
        tags.insert(persona.tier.tag().to_lowercase());
        tags.insert(persona.modality_specialization.clone());

        Self {
            id: id.into(),
            name: name.into(),
            version: "1.0.0".to_string(),
            status: BehaviourStatus::Active,
            created_at: "2026-10-22T00:00:00Z".to_string(),
            updated_at: None,
            author: author.into(),
            tags,
            description: description.into(),
            provenance: CreationMethod::HandcraftedHeuristic {
                engineer: "tdrace-calibration".to_string(),
                notes: "Heuristic profile tuning".to_string(),
            },
            persona,
            controller: ControllerPayload::ClassicalLookahead {
                lookahead_time: profile.lookahead_time,
                speed_factor: profile.speed_factor,
                steering_kp: profile.steering_kp,
                steering_kd: profile.steering_kd,
                brake_margin: profile.brake_margin,
                aggression: profile.aggression,
                avoidance_distance: profile.avoidance_distance,
                traits: profile.traits,
            },
            performance_summary: None,
        }
    }

    /// Constructs a canonical baseline behavior for a given style and tier.
    pub fn canonical_baseline(style: DrivingStyle, tier: DriverTier) -> Self {
        let quality = DriverQuality::for_tier(tier);
        let profile = BotProfile::from_style_and_quality(style, &quality);
        let id_str = format!("canonical_{}_{}", style.as_str(), tier.tag().to_lowercase().replace('+', "_plus"));
        let name_str = format!("Canonical {} {}", style.display_name(), tier.short_name());
        let desc = format!("Canonical {} driver behavior calibrated for {}", style.display_name(), tier.title());

        let persona = DrivingPersona {
            style,
            tier,
            modality_specialization: "universal".to_string(),
            tactical_notes: format!("Canonical {} archetype", style.display_name()),
        };

        let mut b = Self::from_profile(id_str, name_str, persona, profile, "tdrace-system", desc);
        b.status = BehaviourStatus::Baseline;
        b.tags.insert("canonical".to_string());
        b.tags.insert("baseline".to_string());
        b
    }

    /// Extracts the corresponding [`BotProfile`] representation.
    pub fn to_bot_profile(&self) -> BotProfile {
        match &self.controller {
            ControllerPayload::ClassicalLookahead {
                lookahead_time,
                speed_factor,
                steering_kp,
                steering_kd,
                brake_margin,
                aggression,
                avoidance_distance,
                traits,
            } => BotProfile {
                name: "Library Bot",
                lookahead_time: *lookahead_time,
                speed_factor: *speed_factor,
                steering_kp: *steering_kp,
                steering_kd: *steering_kd,
                brake_margin: *brake_margin,
                aggression: *aggression,
                avoidance_distance: *avoidance_distance,
                traits: *traits,
            },
            ControllerPayload::OptimalLineFollower {
                speed_factor,
                steering_kp,
                steering_kd,
                brake_margin,
                aggression,
                avoidance_distance,
                lookahead_distance_m: _,
                traits,
            } => BotProfile {
                name: "Optimal Line Bot",
                lookahead_time: 0.35,
                speed_factor: *speed_factor,
                steering_kp: *steering_kp,
                steering_kd: *steering_kd,
                brake_margin: *brake_margin,
                aggression: *aggression,
                avoidance_distance: *avoidance_distance,
                traits: *traits,
            },
            ControllerPayload::QssMpcRollout {
                speed_factor,
                steering_kp,
                steering_kd,
                brake_margin,
                aggression,
                avoidance_distance,
                horizon_s: _,
                candidate_count: _,
                cost_weights: _,
                traits,
            } => BotProfile {
                name: "QSS MPC Bot",
                lookahead_time: 0.32,
                speed_factor: *speed_factor,
                steering_kp: *steering_kp,
                steering_kd: *steering_kd,
                brake_margin: *brake_margin,
                aggression: *aggression,
                avoidance_distance: *avoidance_distance,
                traits: *traits,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_baseline_generation() {
        let b = BotBehaviour::canonical_baseline(DrivingStyle::Aggressive, DriverTier::Pro);
        assert_eq!(b.id.0, "canonical_aggressive_t4");
        assert_eq!(b.persona.style, DrivingStyle::Aggressive);
        assert_eq!(b.persona.tier, DriverTier::Pro);
        assert_eq!(b.status, BehaviourStatus::Baseline);
        assert!(b.tags.contains("aggressive"));
        assert!(b.tags.contains("t4"));
    }

    #[test]
    fn test_behaviour_serialization_roundtrip() {
        let b = BotBehaviour::canonical_baseline(DrivingStyle::Smooth, DriverTier::LegendPlus);
        let json = serde_json::to_string_pretty(&b).expect("serialize behaviour");
        let decoded: BotBehaviour = serde_json::from_str(&json).expect("deserialize behaviour");
        assert_eq!(b.id, decoded.id);
        assert_eq!(b.persona.style, decoded.persona.style);
        assert_eq!(b.persona.tier, decoded.persona.tier);
        assert_eq!(b.controller, decoded.controller);
    }
}
