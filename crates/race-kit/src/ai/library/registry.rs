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
    pub version: Option<String>,
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

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.query.version = Some(version.into());
        self
    }

    pub fn build(self) -> BotQuery {
        self.query
    }
}

fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let parse_parts = |s: &str| -> Vec<u32> {
        s.trim_start_matches('v')
            .split(['.', '-', '_'])
            .filter_map(|p| p.parse::<u32>().ok())
            .collect()
    };
    let parts_a = parse_parts(a);
    let parts_b = parse_parts(b);
    if !parts_a.is_empty() && !parts_b.is_empty() {
        parts_a.cmp(&parts_b)
    } else {
        a.cmp(b)
    }
}

/// In-memory catalog of registered bot behaviors supporting multiple concurrent versions.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BotBehaviourRegistry {
    entries: HashMap<(BotBehaviourId, String), BotBehaviour>,
    versions_by_id: HashMap<BotBehaviourId, Vec<String>>,
}

impl BotBehaviourRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            versions_by_id: HashMap::new(),
        }
    }

    /// Registers a behavior into the catalog.
    ///
    /// Preserves existing versions when a new version of the same ID is registered.
    /// If an exact (ID, version) combination already exists, it is updated in place.
    pub fn register(&mut self, behaviour: BotBehaviour) {
        let key = (behaviour.id.clone(), behaviour.version.clone());
        let id = behaviour.id.clone();
        let version = behaviour.version.clone();
        self.entries.insert(key, behaviour);
        let list = self.versions_by_id.entry(id).or_default();
        if !list.contains(&version) {
            list.push(version);
            list.sort_by(|a, b| compare_versions(a, b));
        }
    }

    /// Retrieves the preferred active or latest version of a behavior by its ID.
    pub fn get(&self, id: &BotBehaviourId) -> Option<&BotBehaviour> {
        let versions = self.versions_by_id.get(id)?;
        // Prefer an Active or Baseline status behavior among versions; otherwise return the latest
        for v in versions.iter().rev() {
            if let Some(b) = self.entries.get(&(id.clone(), v.clone())) {
                if matches!(b.status, BehaviourStatus::Active | BehaviourStatus::Baseline) {
                    return Some(b);
                }
            }
        }
        let latest_ver = versions.last()?;
        self.entries.get(&(id.clone(), latest_ver.clone()))
    }

    /// Retrieves an exact version of a behavior by its ID and semantic version string.
    pub fn get_version(&self, id: &BotBehaviourId, version: &str) -> Option<&BotBehaviour> {
        self.entries.get(&(id.clone(), version.to_string()))
    }

    /// Returns all registered versions of a given behavior ID.
    pub fn versions_of(&self, id: &BotBehaviourId) -> Vec<&BotBehaviour> {
        if let Some(versions) = self.versions_by_id.get(id) {
            versions
                .iter()
                .filter_map(|v| self.entries.get(&(id.clone(), v.clone())))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Checks if a behavior ID is present in any version.
    pub fn contains(&self, id: &BotBehaviourId) -> bool {
        self.versions_by_id.contains_key(id)
    }

    /// Checks if a specific (ID, version) exists.
    pub fn contains_version(&self, id: &BotBehaviourId, version: &str) -> bool {
        self.entries.contains_key(&(id.clone(), version.to_string()))
    }

    /// Removes all versions of a behavior by ID, returning the most recent one if present.
    pub fn remove(&mut self, id: &BotBehaviourId) -> Option<BotBehaviour> {
        let versions = self.versions_by_id.remove(id)?;
        let mut last_removed = None;
        for v in versions {
            if let Some(b) = self.entries.remove(&(id.clone(), v)) {
                last_removed = Some(b);
            }
        }
        last_removed
    }

    /// Removes a specific version of a behavior by ID.
    pub fn remove_version(&mut self, id: &BotBehaviourId, version: &str) -> Option<BotBehaviour> {
        if let Some(list) = self.versions_by_id.get_mut(id) {
            list.retain(|v| v != version);
            if list.is_empty() {
                self.versions_by_id.remove(id);
            }
        }
        self.entries.remove(&(id.clone(), version.to_string()))
    }

    /// Returns the total number of registered behavior versions across all IDs.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns the number of distinct behavior IDs registered.
    pub fn unique_id_count(&self) -> usize {
        self.versions_by_id.len()
    }

    /// Returns `true` if the registry contains no behaviors.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns an iterator over all registered behaviors across all versions.
    pub fn all(&self) -> impl Iterator<Item = &BotBehaviour> {
        self.entries.values()
    }

    /// Searches for all behaviors that satisfy the query parameters.
    pub fn search(&self, query: &BotQuery) -> Vec<&BotBehaviour> {
        self.entries
            .values()
            .filter(|b| {
                if let Some(ref ver) = query.version {
                    if &b.version != ver {
                        return false;
                    }
                }
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

    #[test]
    fn test_behaviour_versioning_and_date_retention() {
        let mut reg = BotBehaviourRegistry::new();
        let bot_id = BotBehaviourId::from("qss_mpc_legend");

        let mut v1 = BotBehaviour::canonical_baseline(DrivingStyle::Calculating, DriverTier::Legend);
        v1.id = bot_id.clone();
        v1.version = "1.0.0".to_string();
        v1.created_at = "2026-10-22T10:00:00Z".to_string();

        let mut v2 = v1.clone();
        v2.version = "1.1.0".to_string();
        v2.created_at = "2026-10-23T15:30:00Z".to_string();
        v2 = v2.with_updated_at("2026-10-23T16:00:00Z");

        reg.register(v1);
        reg.register(v2);

        // Both versions are preserved concurrently
        assert_eq!(reg.len(), 2);
        assert_eq!(reg.unique_id_count(), 1);

        // get() returns the preferred / latest version
        let active = reg.get(&bot_id).expect("bot should exist");
        assert_eq!(active.version, "1.1.0");
        assert_eq!(active.date(), "2026-10-23T16:00:00Z");

        // get_version() retrieves the exact historical version
        let historical = reg.get_version(&bot_id, "1.0.0").expect("v1.0.0 must exist");
        assert_eq!(historical.version, "1.0.0");
        assert_eq!(historical.date(), "2026-10-22T10:00:00Z");

        // versions_of() returns all versions
        let all_vers = reg.versions_of(&bot_id);
        assert_eq!(all_vers.len(), 2);

        // Query filtering by exact version
        let query_v1 = reg.search(&BotQuery::builder().version("1.0.0").build());
        assert_eq!(query_v1.len(), 1);
        assert_eq!(query_v1[0].version, "1.0.0");
    }
}
