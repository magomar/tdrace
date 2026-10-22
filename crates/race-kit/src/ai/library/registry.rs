//! In-memory Bot Behaviour catalog registry and query engine (Spec 137).

use std::collections::HashMap;
use super::{BehaviourStatus, BotBehaviour, BotBehaviourId};
use crate::ai::{DriverTier, DrivingStyle};

/// Query filters for discovering matching behaviors in the library.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BotQuery {
    pub style: Option<DrivingStyle>,
    pub tier: Option<DriverTier>,
    pub modality: Option<String>,
    pub min_status: Option<BehaviourStatus>,
    pub required_tags: Vec<String>,
    pub max_contacts_per_lap: Option<f32>,
    pub max_pace_delta_pct: Option<f32>,
    pub min_human_believability: Option<f32>,
}

impl BotQuery {
    pub fn builder() -> BotQueryBuilder {
        BotQueryBuilder::default()
    }
}

/// Fluent builder for constructing [`BotQuery`] instances.
#[derive(Debug, Clone, Default)]
pub struct BotQueryBuilder {
    query: BotQuery,
}

impl BotQueryBuilder {
    pub fn style(mut self, style: DrivingStyle) -> Self {
        self.query.style = Some(style);
        self
    }

    pub fn tier(mut self, tier: DriverTier) -> Self {
        self.query.tier = Some(tier);
        self
    }

    pub fn modality(mut self, modality: impl Into<String>) -> Self {
        self.query.modality = Some(modality.into());
        self
    }

    pub fn min_status(mut self, status: BehaviourStatus) -> Self {
        self.query.min_status = Some(status);
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.query.required_tags.push(tag.into());
        self
    }

    pub fn max_contacts_per_lap(mut self, max_contacts: f32) -> Self {
        self.query.max_contacts_per_lap = Some(max_contacts);
        self
    }

    pub fn max_pace_delta_pct(mut self, max_delta: f32) -> Self {
        self.query.max_pace_delta_pct = Some(max_delta);
        self
    }

    pub fn min_human_believability(mut self, min_score: f32) -> Self {
        self.query.min_human_believability = Some(min_score);
        self
    }

    pub fn build(self) -> BotQuery {
        self.query
    }
}

/// In-memory catalog of registered bot behaviors.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BotBehaviourRegistry {
    behaviours: HashMap<BotBehaviourId, BotBehaviour>,
}

impl BotBehaviourRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            behaviours: HashMap::new(),
        }
    }

    /// Registers a behavior into the catalog. Overwrites if an exact ID already exists.
    pub fn register(&mut self, behaviour: BotBehaviour) {
        self.behaviours.insert(behaviour.id.clone(), behaviour);
    }

    /// Retrieves a reference to a behavior by its ID.
    pub fn get(&self, id: &BotBehaviourId) -> Option<&BotBehaviour> {
        self.behaviours.get(id)
    }

    /// Checks if a behavior ID is present.
    pub fn contains(&self, id: &BotBehaviourId) -> bool {
        self.behaviours.contains_key(id)
    }

    /// Removes a behavior by ID.
    pub fn remove(&mut self, id: &BotBehaviourId) -> Option<BotBehaviour> {
        self.behaviours.remove(id)
    }

    /// Returns the total number of registered behaviors.
    pub fn len(&self) -> usize {
        self.behaviours.len()
    }

    /// Returns `true` if the registry contains no behaviors.
    pub fn is_empty(&self) -> bool {
        self.behaviours.is_empty()
    }

    /// Returns an iterator over all registered behaviors.
    pub fn all(&self) -> impl Iterator<Item = &BotBehaviour> {
        self.behaviours.values()
    }

    /// Searches for all behaviors that satisfy the query parameters.
    pub fn search(&self, query: &BotQuery) -> Vec<&BotBehaviour> {
        self.behaviours
            .values()
            .filter(|b| {
                if let Some(style) = query.style {
                    if b.persona.style != style {
                        return false;
                    }
                }
                if let Some(tier) = query.tier {
                    if b.persona.tier != tier {
                        return false;
                    }
                }
                if let Some(ref mod_spec) = query.modality {
                    if b.persona.modality_specialization != "universal"
                        && &b.persona.modality_specialization != mod_spec
                    {
                        return false;
                    }
                }
                if let Some(status) = query.min_status {
                    match status {
                        BehaviourStatus::Active => {
                            if !matches!(b.status, BehaviourStatus::Active | BehaviourStatus::Baseline) {
                                return false;
                            }
                        }
                        BehaviourStatus::Baseline => {
                            if b.status != BehaviourStatus::Baseline {
                                return false;
                            }
                        }
                        _ => {}
                    }
                }
                for req_tag in &query.required_tags {
                    if !b.tags.contains(req_tag) {
                        return false;
                    }
                }
                if let Some(max_contacts) = query.max_contacts_per_lap {
                    if let Some(ref perf) = b.performance_summary {
                        if let Some(c) = perf.contacts_per_lap {
                            if c > max_contacts {
                                return false;
                            }
                        }
                    }
                }
                if let Some(max_delta) = query.max_pace_delta_pct {
                    if let Some(ref perf) = b.performance_summary {
                        if let Some(d) = perf.pace_delta_to_optimal_pct {
                            if d > max_delta {
                                return false;
                            }
                        }
                    }
                }
                if let Some(min_believability) = query.min_human_believability {
                    if let Some(ref perf) = b.performance_summary {
                        if let Some(score) = perf.human_believability_score {
                            if score < min_believability {
                                return false;
                            }
                        }
                    }
                }
                true
            })
            .collect()
    }

    /// Filters behaviors matching a specific driving style.
    pub fn filter_by_style(&self, style: DrivingStyle) -> Vec<&BotBehaviour> {
        self.search(&BotQuery::builder().style(style).build())
    }

    /// Filters behaviors matching a specific driver tier.
    pub fn filter_by_tier(&self, tier: DriverTier) -> Vec<&BotBehaviour> {
        self.search(&BotQuery::builder().tier(tier).build())
    }

    /// Populates and returns a canonical registry containing standard baseline behaviors
    /// across all 6 driving styles and all 7 driver tier variants (Spec 023 / Spec 131).
    pub fn canonical_baseline_registry() -> Self {
        let mut reg = Self::new();
        for &style in &DrivingStyle::ALL {
            for &tier in &DriverTier::ALL {
                reg.register(BotBehaviour::canonical_baseline(style, tier));
            }
        }
        reg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::library::PerformanceSummaryMetrics;

    #[test]
    fn test_canonical_baseline_registry_count() {
        let reg = BotBehaviourRegistry::canonical_baseline_registry();
        // 6 styles x 7 tiers = 42 canonical behaviors
        assert_eq!(reg.len(), 42);

        let aggressive_t4 = reg.get(&BotBehaviourId::from("canonical_aggressive_t4"));
        assert!(aggressive_t4.is_some());
        let b = aggressive_t4.unwrap();
        assert_eq!(b.persona.style, DrivingStyle::Aggressive);
        assert_eq!(b.persona.tier, DriverTier::Pro);
    }

    #[test]
    fn test_query_filtering_by_style_and_tier() {
        let reg = BotBehaviourRegistry::canonical_baseline_registry();

        let smooth_t5 = reg.search(
            &BotQuery::builder()
                .style(DrivingStyle::Smooth)
                .tier(DriverTier::Legend)
                .build(),
        );
        assert_eq!(smooth_t5.len(), 1);
        assert_eq!(smooth_t5[0].id.0, "canonical_smooth_t5");

        let all_pro_plus = reg.filter_by_tier(DriverTier::ProPlus);
        assert_eq!(all_pro_plus.len(), 6);
    }

    #[test]
    fn test_query_filtering_by_performance_metrics() {
        let mut reg = BotBehaviourRegistry::new();
        let mut b1 = BotBehaviour::canonical_baseline(DrivingStyle::Aggressive, DriverTier::Pro);
        b1.performance_summary = Some(PerformanceSummaryMetrics {
            best_flying_lap_s: Some(45.0),
            mean_flying_lap_s: Some(45.5),
            pace_delta_to_optimal_pct: Some(0.8),
            contacts_per_lap: Some(0.25),
            clean_pass_ratio: Some(0.70),
            human_believability_score: Some(0.65),
        });

        let mut b2 = BotBehaviour::canonical_baseline(DrivingStyle::Smooth, DriverTier::Pro);
        b2.id = BotBehaviourId::from("smooth_safe_pro");
        b2.performance_summary = Some(PerformanceSummaryMetrics {
            best_flying_lap_s: Some(46.2),
            mean_flying_lap_s: Some(46.8),
            pace_delta_to_optimal_pct: Some(2.5),
            contacts_per_lap: Some(0.02), // very safe!
            clean_pass_ratio: Some(0.95),
            human_believability_score: Some(0.92),
        });

        reg.register(b1);
        reg.register(b2);

        // Filter for safe, low-contact racers (<= 0.05 contacts/lap)
        let safe_bots = reg.search(
            &BotQuery::builder()
                .max_contacts_per_lap(0.05)
                .min_human_believability(0.80)
                .build(),
        );
        assert_eq!(safe_bots.len(), 1);
        assert_eq!(safe_bots[0].id.0, "smooth_safe_pro");
    }
}
