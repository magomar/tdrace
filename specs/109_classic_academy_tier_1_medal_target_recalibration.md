---
type: Feature Spec
template: feature
title: "Classic Academy Tier 1 Medal Target Recalibration"
description: "Retargets the four Spec 060 Tier 1 lesson medal times from a headless reference run, because the shipped times (18.5 to 68 s) cannot be reached on the current circuits and nobody can graduate. Bounties, unlock rules and licences stay as they are."
status: draft
created: 2026-10-11
generated: { by: agent/unknown, at: 2026-10-11T15:32:53Z }
depends_on:
  - "060"
  - "064"
amends:
  - "060"
  - "064"
---


# Feature Spec: Classic Academy Tier 1 Medal Target Recalibration 🎯

[Spec 060](060_classic_academy_and_grassroots_career_onboarding.md) set the Tier 1 medal times (Gold, Silver, Bronze) for the four Class D lessons. [Spec 064](064_classic_module_multitier_academy_missions_and_degradation_curriculum.md) kept them unchanged. The calibration run of Spec 064 (`crates/tdrace-app/tests/academy_calibration_tests.rs`) found that no Tier 1 Bronze time can be reached on the circuits of today. The reference driver needs 59.4 s on `gt_velocity_park` (1,482 m), and the Bronze time is 22.5 s. Nobody can graduate, so Class D and Tiers 2 and 3 stay locked (bug `tdrace-tb8vi`).

This spec amends Spec 060 and Spec 064 in one way: **the Tier 1 medal times change, from the same reference run that set Tiers 2 and 3.** Nothing else in Tier 1 changes.

---

## 🎯 What Changes and What Does Not

**Changes**: `gold_time_sec`, `silver_time_sec` and `bronze_time_sec` of the four Tier 1 lessons (`AcademyLessonDef::default_curriculum`).

**Stays as it is**: lesson ids and slugs, circuits and cars, the bounties (14,500 Cr in total: Bronze 7,000, Silver 10,750, Gold 14,500 for all four), the XP, the sequential unlock, graduation on Lesson 4 at Bronze, the Class D grant, the clean-attempt rule, the 1,500 Cr stipend, and the session rules of Spec 064 (damage off, wear off).

### Reference run and targets

The reference driver is the one in the calibration test: a Contender balanced bot with no human mistakes, same car and rules as the lesson, from the grid. Gold is about 3% over the reference time, Silver about 12%, Bronze about 25%. The values below come from that run. The calibration task re-runs it and sets the final numbers (to 0.1 s).

| Lesson | Circuit and car | Reference | Gold | Silver | Bronze | Was (Gold / Silver / Bronze) |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| `lesson_1_apex_line` | `gt_velocity_park`, `classic_gt` | 59.4 s | 61.2 s | 66.5 s | 74.2 s | 18.5 / 20.0 / 22.5 |
| `lesson_2_braking_chicane` | `gt_ridge_ring`, `classic_gt` | 60.2 s | 62.0 s | 67.4 s | 75.2 s | 24.0 / 26.0 / 29.0 |
| `lesson_3_surface_transition` | `rx_quarry_sprint`, `classic_rally` | 56.5 s | 58.2 s | 63.3 s | 70.6 s | 32.0 / 35.0 / 39.0 |
| `lesson_4_graduation_sprint` | `ax_meadow_sprint`, `classic_ax_mudlark`, 2 laps, 1 bot | 101.8 s | 104.8 s | 114.0 s | 127.2 s | 68.0 / 73.0 / 80.0 |

Lessons 1 to 3 are scored on the lap time and Lesson 4 on the two-lap race time, as today.

---

## 🗺️ User Flow & Interface Design

No new screen. The Academy screen (`ui/academy_ui.rs`) and the in-race HUD read the targets from the lesson definitions, so the new times show without a UI change. The pace line of the HUD (`AHEAD OF GOLD`, `BEHIND BRONZE`) compares the race against the new targets.

---

## ⚙️ Backend Models & API Endpoints

1. `crates/arcade-race-core/src/profile.rs`: the three times of each Tier 1 lesson in `default_curriculum()`. No type, field or SQL change.
2. The Codex export (`crates/tdrace-app/src/codex/mod.rs`) lists the Tier 1 lessons, so `portals/shared/data/codex/racing.json` is re-exported with the new times (`cargo run -p tdrace-app --bin export_codex`).
3. `crates/tdrace-app/tests/academy_calibration_tests.rs` covers Tier 1 as well as Tiers 2 and 3.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

- **Save compatibility**: medals, best times and claimed-bounty flags in saved profiles are untouched. A medal earned under the old times (only possible with a hand-edited save) stays. No bounty is paid twice.
- **Economy**: the bounty split and the 14,500 Cr total do not change, so the car price calibration of Spec 060 holds.
- **Licences and gating**: unchanged. Graduation (Bronze or better on Lesson 4) still grants Class D and opens Tier 2.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- `cargo test -p tdrace-app --test academy_calibration_tests` (the reference run earns Gold on every Tier 1 lesson, and the targets follow the reference run)
- `cargo test -p arcade-race-core --test profile_tests` and `cargo test -p tdrace-app --test career_onboarding_tests` (the Tier 1 times, bounties and unlocks)
- `cargo test -p tdrace-app --test codex_export_tests` (the committed Codex data is fresh)

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: Every Tier 1 Bronze is reachable cleanly**
  - [ ] **Given** each Tier 1 lesson with its circuit, car and rules
  - [ ] **When** the headless reference driver runs it
  - [ ] **Then** the reference finishes, the attempt is clean, and it earns Gold
  - [ ] **And** the Bronze time is about 25% over the reference time

- **Scenario: A player can graduate**
  - [ ] **Given** a rookie profile and the four Tier 1 lessons run in order at the reference pace
  - [ ] **When** Lesson 4 ends at Bronze or better
  - [ ] **Then** the profile holds Class D and Tier 2 is unlocked

- **Scenario: Bounties and unlocks do not change**
  - [ ] **Given** the Tier 1 curriculum after this spec
  - [ ] **When** the player earns all Bronze, all Silver, or all Gold
  - [ ] **Then** the purse is 7,000, 10,750 or 14,500 Cr
  - [ ] **And** each lesson still needs Bronze on the one before it

- **Scenario: Old saves keep their Tier 1 progress**
  - [ ] **Given** a profile saved before this spec with a Tier 1 medal and its claimed bounties
  - [ ] **When** the profile loads
  - [ ] **Then** the medal and the claimed flags are the same, and no bounty is paid again

- **Scenario: The Academy screen shows the new targets**
  - [ ] **Given** the Academy screen on the Tier 1 tab
  - [ ] **When** the player looks at Lesson 1
  - [ ] **Then** the card shows Gold 61.2 s, Silver 66.5 s and Bronze 74.2 s (or the final calibrated times)

---

## 🔗 Traceability & Codebase Mapping

### Modified Files
- `[ ]` `crates/arcade-race-core/src/profile.rs` -> the Tier 1 medal times.
- `[ ]` `crates/tdrace-app/tests/academy_calibration_tests.rs` -> the reference run covers Tier 1.
- `[ ]` `crates/arcade-race-core/tests/profile_tests.rs`, `crates/tdrace-app/tests/career_onboarding_tests.rs`, `crates/tdrace-app/tests/codex_export_tests.rs` -> the expected Tier 1 times, and the graduation scenario.
- `[ ]` `portals/shared/data/codex/racing.json` -> re-exported Codex data.
- `[ ]` `specs/constitution/ROADMAP.md` -> roadmap entry.

### Verification Assertions
- Bug `tdrace-tb8vi` is closed by this spec.
