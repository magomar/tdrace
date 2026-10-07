---
type: Feature Spec
template: feature
title: "GT Tyre Compound Choice in the Garage"
description: "Lets the player choose Soft, Medium or Hard slicks for a GT module car in the garage, keeps the choice per car in the profile, and shows stock cars' fixed series compound read-only. No other car setup is tunable."
status: draft
created: 2026-10-07
generated: { by: agent/claude-code, at: 2026-10-07T12:00:00Z }
depends_on:
  - "009"
  - "074"
  - "089"
---

# Feature Spec: GT Tyre Compound Choice in the Garage 🏎️🛞

The player picks the slick compound of a GT car in the garage: Soft, Medium or Hard. The choice is a trade-off between grip and wear. Stock cars do not get a choice: like the real series, they race on the compound the series sets. No other part of the car setup becomes tunable.

---

## 🔍 Context & Problem Analysis

Spec 074 added data-driven tyre compounds (`CompoundId`, `TireCompoundConfig::from_id` in `crates/wheelbase/src/tire.rs`). Three slicks exist:

| Compound | Base grip | Peak slip angle | Wear rate (per W of slide power) | Wear vs Medium |
| :--- | :---: | :---: | :---: | :---: |
| Soft Slick | 1.20 | 8.0° | 0.45 × 10⁻⁶ | 1.8× |
| Medium Slick | 1.00 | 10.0° | 0.25 × 10⁻⁶ | 1.0× |
| Hard Slick | 0.95 | 11.5° | 0.12 × 10⁻⁶ | 0.5× |

Today every car has one fixed compound from its base preset (`crates/wheelbase/src/config.rs`): GT cars use Medium (`default_wheel_assemblies_for`), stock cars use Hard (`stock_car_ta1`, `stock_car_truck`), karts use Soft. Nothing in `crates/tdrace-app` changes it, so the compound is invisible as a choice.

Since tdrace-6dl0 (2026-10-07) only a sideways slide past the peak slip angle wears a tyre, and a slide on loose ground wears 0.25 of a slide on pavement. Clean sprint driving barely wears any compound; drifting abuse does. This makes the compound a meaningful choice:
- **Soft**: the most grip, but it slides past its peak sooner (8°) and wears 1.8× faster, so a driver who drifts loses it quickly and needs a pit stop (`Car::service_tires` resets wear at a pit service, race-kit `world.rs`).
- **Hard**: the least grip, but forgiving (peak 11.5°) and slow to wear.
- **Medium**: today's behaviour.

### Real-world reference: stock cars
In stock car racing (NASCAR Cup) the tyre supplier brings one compound per track (sometimes different left and right sides), and every team must use it. Softer tyres at some tracks are still the same for everyone; a free "option tyre" has only been a trial at a few races. So stock cars get no choice in this spec.

### Design note: why the compound is the only setup choice (Mario, 2026-10-07)
Other tuning (brake bias, gearing, springs) is out of scope on purpose. Tuning would let a player make one model behave like another model of the same category, so the choice of car model would become secondary. Car identity comes from the model; the compound only trades grip for wear and never changes what the model is. Any future setup option must pass the same test: it must not let one model imitate another.

---

## 🗺️ User Flow & Interface Design

### 1. Garage (`crates/tdrace-app/src/ui/garage.rs`)
- For a car of the GT module (`module_id` `gt` / `gt_challenge`), the garage detail panel shows a **TYRES** row with three options: SOFT, MEDIUM, HARD. The selected one uses the compound accent colour (`CompoundId::accent_rgba`, spec 089).
- Under the row, one line states the trade-off of the selected compound, e.g. "More grip, wears faster when you slide."
- Input: the same left/right and mouse/touch pattern as the other garage rows. Changing the compound plays the UI move sound.
- For a stock car (`module_id` `nascar`), the row shows the series compound read-only, e.g. "TYRES: HARD (series rule)", with no selector.
- For all other cars (kart, rally, autocross, extreme off-road, Classic module), the row is not shown.

### 2. Race
- The chosen compound is fitted before the race starts. The cockpit compound badge and wheel borders (spec 089) show it, because they read `car.state.wheel_assemblies[i].config.compound.id`.
- The compound does not change during a race. A pit service restores tread (existing behaviour) on the same compound.

---

## 🧭 Visual Standards & Technical Conventions
- Compound names and colours come from `CompoundId::name` / `badge_code` / `accent_rgba`; no new palette.
- All new text is English and must pass the IP denylist (spec 047); compound names are generic.

---

## ⚙️ Backend Models & API Endpoints

### 1. Profile data (`crates/tdrace-app/src/profile/mod.rs`, `crates/tdrace-app/src/db/mod.rs`)
- `PlayerProfile` gains `car_compounds: HashMap<String, CompoundId>` (car model id → compound), `#[serde(default)]`.
- Stored as a JSON TEXT column on the profile table, added with the existing `ALTER TABLE ... ADD COLUMN` migration pattern. An old database loads with an empty map.
- A car with no entry uses its preset compound (GT: Medium).

### 2. Session and race build (`crates/tdrace-app/src/game/mod.rs`)
- `RaceSession` resolves the player's compound from the profile when the garage confirms a car (`update_garage`) and keeps it next to `selected_car_model_id`.
- `rebuild_roster_participants`: after the player's `base_config` is built (`pm.to_car_config()` and the assists), call `base_config.set_compound(id)` only when the car is a GT module car, before `Car::new`.
- Split-screen player 2 keeps its car's preset compound (it has no garage of its own); it must not inherit player 1's choice.
- LAN (`launch_lan_race_session`): the owner applies its own profile compound to its own car before `Car::new`. Cars are owner-authoritative (spec 044) and the HUD shows only the local car, so the lobby protocol does not change (`PROTOCOL_VERSION` stays 3).

### 3. Bots
- Bots keep their preset compound (GT: Medium). A bot compound strategy is out of scope.

### 4. Stock cars
- The garage reads the compound from the car's preset (`CarConfig::wheels[0].compound.id`) and shows it read-only. A per-track compound is out of scope.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)
- Local single-player data only. Profile JSON is parsed with serde; an unknown compound value falls back to the preset compound (fail closed to the default, never panic).

---

## 🧪 Verification & Acceptance Criteria

- **Scenario: A GT player chooses a compound in the garage**
  - [ ] **Given** the player opens the garage on a GT module car
  - [ ] **When** the player changes TYRES from MEDIUM to SOFT and confirms the car
  - [ ] **Then** the race starts with Soft Slick on all four wheels of the player's car
  - [ ] **And** the cockpit compound badge shows SOFT

- **Scenario: The choice is kept per car**
  - [ ] **Given** the player chose HARD for one GT car and left another GT car untouched
  - [ ] **When** the game restarts and the player opens the garage
  - [ ] **Then** the first car shows HARD and the second car shows MEDIUM

- **Scenario: Stock cars race on the series compound**
  - [ ] **Given** the player opens the garage on a stock car
  - [ ] **When** the detail panel is shown
  - [ ] **Then** it shows the series compound read-only and offers no selector

- **Scenario: Other categories are unchanged**
  - [ ] **Given** a kart, rally, autocross, extreme off-road or Classic module car
  - [ ] **When** the garage detail panel is shown
  - [ ] **Then** there is no TYRES row and the car races on its preset compound

- **Scenario: The compound trades grip for wear**
  - [ ] **Given** the GT3 car (`GtWorldChallengeModule::car_gt3_evo`) on Soft, Medium and Hard
  - [ ] **When** each runs the skidpad protocol and a 60 s drift on asphalt
  - [ ] **Then** peak lateral grip orders Soft > Medium > Hard
  - [ ] **And** drift tread wear orders Soft > Medium > Hard

- **Scenario: Player 2 and bots keep their preset compound**
  - [ ] **Given** player 1 chose SOFT for a GT car in a split-screen race with bots
  - [ ] **When** the race starts
  - [ ] **Then** player 2's car and every bot car use their preset compound

- **Scenario: An old profile loads**
  - [ ] **Given** a profile database written before this spec
  - [ ] **When** the game loads it
  - [ ] **Then** every car uses its preset compound and no error is shown

---

## 🔗 Traceability & Codebase Mapping

### Modified Files
- `crates/tdrace-app/src/ui/garage.rs`: TYRES row (selector for GT, read-only for stock cars).
- `crates/tdrace-app/src/game/mod.rs`: garage input, compound resolution, `rebuild_roster_participants`, `launch_lan_race_session`.
- `crates/tdrace-app/src/profile/mod.rs`, `crates/tdrace-app/src/db/mod.rs`: `car_compounds` field and migration.
- `crates/tdrace-app/tests/`: garage, profile persistence and race-build tests; a compound trade-off test.

### Out of Scope
- Brake bias, gearing, suspension or any other setup tuning (see the design note).
- Changing the compound at a pit stop; bot compound strategy; per-track stock car compounds; endurance races.
- The unused compounds and the Medium Slick dirt value (tdrace-md1u).
