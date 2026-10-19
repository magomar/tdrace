---
type: Feature Spec
template: feature
title: "FIA European Autocross 10-Car 3-2-3-2 Grids, Timed Qualifying Practice, 30-Driver Knockout Format, Two-Phase Scoring, and Expanded Season Calendars"
description: "Upgrades the FIA Autocross (AX) championship format to authentic FIA European Autocross rules: 10-car 3-2-3-2 staggered concrete launch chutes across all 20 AX circuits, a 30-driver field, Stage 0 Timed Qualifying Practice (1 out-lap + 1 flying lap from the chute), configurable Qualifying Heats (default 1 heat for arcade pace, up to 3 heats with Best 2-of-3 Intermediate Classification), Top 20 into two 10-car Semifinals with Top 5 position-only advancement to the 10-car Final (plus B-Final and Consolation Semifinal for Option A continuous racing), Two-Phase FIA Autocross points (Intermediate Top 10: 10..1 pts + Final Top 10: 25-18-15-12-10-8-6-4-2-1 pts), and expanded 5-to-10-round season calendars across all 5 career tiers."
status: implemented
receipt: "docs/receipts/spec-130-receipt.md"
verified: { by: "human:mario", at: "2026-10-19T15:11:05Z", hash: "b486242cdc18" }
created: 2026-10-19
generated: { by: agent/antigravity, at: 2026-10-19T15:00:00Z }
depends_on:
  - "050"
  - "103"
  - "104"
  - "106"
amends:
  - "050"
  - "103"
  - "104"
  - "106"
---

# Feature Spec: FIA European Autocross 10-Car 3-2-3-2 Grids, Timed Qualifying Practice, 30-Driver Knockout Format, Two-Phase Scoring, and Expanded Season Calendars 🏁🏆

Amends [Spec 050](050_fia_autocross_championship_and_vehicle_roster.md), [Spec 103](103_autocross_and_rallycross_launch_chutes_and_templated_track_components.md), [Spec 104](104_autocross_and_rallycross_tournament_sprint_weekend_format.md), and [Spec 106](106_classic_autocross_launch_chutes_and_tencar_rallycross_chutes.md) for **FIA Autocross (`autocross`)**, upgrading Autocross launch chutes, tournament weekends, scoring, and season calendars to authentic **FIA European Autocross Championship** rules:

1. **10-Car `3-2-3-2` Staggered Autocross Launch Chutes**: All 20 official Autocross circuits (17 European circuits in `tracks/autocross/*.json` and 3 Classic Autocross circuits in `tracks/classic/ax_*.json`) upgrade their concrete launch pads from the 8-car `5-3` layout to the authentic FIA European Autocross **10-car `3-2-3-2` staggered 4-row grid** (`PackedGridPattern::AutocrossThreeTwoThreeTwo`, `[3, 2, 3, 2]`), regenerating `launch_chute.grid_slots` and `track.grid_positions` in place without altering chute spline control points, merge junctions, pad dimensions, or barrier walls.
2. **30-Driver Field & 10-Car Grids**: All Autocross championship series (`series/autocross/*.toml`) field **30 drivers** (`total_drivers = 30`, `grid_size = 10`, `bot_count = 9`) across 3 Qualifying Heats of 10 cars per Qualifying Round, 2 Semifinals of 10 cars (Top 20 drivers), and a 10-car Final (plus a 10-car Consolation Semifinal and 10-car B-Final for the Option A Continuous Racing Guarantee).
3. **Stage 0 — Timed Qualifying Practice (1 Out-Lap + 1 Flying Lap from Chute)**: Every Autocross championship weekend opens with Timed Qualifying Practice (`TournamentStage::SuperPole`, displayed with Autocross contextual header `TIMED QUALIFYING PRACTICE • FLYING LAP`):
   - **Lap 1 (Out-Lap)**: Solo launch from Slot 1 of the concrete launch chute, merging onto the unsealed dirt/clay circuit and crossing the main start/finish line at full speed.
   - **Lap 2 (Flying Lap)**: Timed flying lap setting the driver's qualifying practice time (`laps = 2`, no Joker Lap).
   - The 29 AI competitors receive deterministic simulated flying lap times. All 30 drivers (`P1–P30`) are ranked by fastest flying lap and **snake-seeded** into the 3 Qualifying Heats of 10 cars each for Heat 1 (`Q1`).
4. **Round 1 — Configurable Qualifying Heats (`qualifying_rounds = 1..=3`, Default `1`) & Best-2-of-3 Intermediate Classification**:
   - `TournamentConfig` and `[championship]` TOML metadata support `qualifying_rounds: usize` (default `1` across existing series for brisk arcade pacing, configurable up to `3` heats `Heat 1, Heat 2, Heat 3`).
   - In each Qualifying Heat round `Q(k)`, 3 races of 10 cars contest a pure sprint (`heat_laps`, no Joker Lap).
   - When `qualifying_rounds > 1`, completing Heat `Q(k)` ranks all 30 drivers by their `Q(k)` finishing times and seeds them into the 3 races of `Q(k+1)` (matching FIA Autocross where Heat 1 times set Heat 2 grids and Heat 2 results set Heat 3 grids).
   - **Intermediate Classification**:
     - When `qualifying_rounds == 3` (full 3-heat FIA weekend), each driver's **best 2 heat results out of 3** (lowest sum of their 2 best heat times/ranks) are tallied to rank `1st` through `30th`.
     - When `qualifying_rounds == 1` or `2`, all completed heats are summed to rank `1st` through `30th`.
   - The **Top 20 drivers** in the Intermediate Classification advance to the Semifinals (`21st–30th` are eliminated from main contention; if the player places `21st–30th`, they enter a 10-car Consolation Semifinal under Option A).
5. **Round 2 — Odd/Even 10-Car Semifinals & Top-5 Position-Only Advancement**:
   - The Top 20 drivers from the Intermediate Classification split into **Semifinal 1** (`1st, 3rd, 5th, ..., 19th`) and **Semifinal 2** (`2nd, 4th, 6th, ..., 20th`), lining up in the `3-2-3-2` grid for a sprint race (`semi_laps`).
   - The **Top 5 finishers by on-track finishing position** from Semifinal 1 and the **Top 5 finishers** from Semifinal 2 advance to the 10-car Grand Final; finishers `6th–10th` from each Semifinal enter the 10-car Consolation B-Final.
6. **Round 3 — The 10-Car Final Showdown (`3-2-3-2` Grid)**:
   - The 10 advancing finalists (5 from Semifinal 1 + 5 from Semifinal 2) pair up by Semifinal finishing place (`S1-P1 & S2-P1`, `S1-P2 & S2-P2`, ..., `S1-P5 & S2-P5`, ordered within each pair by Intermediate Classification rank) to fill the 4-row `3-2-3-2` starting grid for the Grand Final (`final_laps`).
   - The first driver across the finish line wins the round.
7. **Two-Phase FIA Autocross Championship Points**:
   - Points are awarded at two phases of the weekend and summed for each driver's weekend total:
     1. **Intermediate Standings (Post-Heats, Top 10)**: `[10, 9, 8, 7, 6, 5, 4, 3, 2, 1]` pts (`0` pts for `11th–30th`), rewarding weekend-long speed and heat reliability.
     2. **Final Classification (Grand Final, Top 10)**: `[25, 18, 15, 12, 10, 8, 6, 4, 2, 1]` pts (`0` pts for B-Final and non-finalists).
   - Maximum weekend score is $10 + 25 = 35\text{ pts}$.
8. **Expanded Progressive European Season Calendars (`5 -> 6 -> 7 -> 8 -> 10` Rounds)**:
   - Expands the 5 Autocross career championship calendars in `series/autocross/*.toml` from 3–5 rounds to a progressive **5, 6, 7, 8, and 10 rounds** across the 17 European dirt venues, culminating in a full 10-round FIA SuperBuggy European Championship visiting **Seelow**, **Mūsa / Bauska**, **Vilkyčiai**, **Matschenberg**, **Nová Paka**, **Saint-Georges-de-Montaigu**, **Přerov**, **Maggiora**, **Saint-Junien**, and **Arteixo**.

---

## 🗺️ User Flow & Interface Design

### 1. Stage 0: Timed Qualifying Practice (Flying Lap)
- **Tournament Bracket Screen**:
  - When an Autocross weekend begins in `Stage 0`, the Bracket screen displays the stage badge `TIMED PRACTICE • SOLO FLYING LAP (1 OUT-LAP + 1 TIMED LAP)` in Autocross burnt orange styling.
- **On-Track Execution**:
  - The player starts alone (`1 car`) in Slot 1 of the circuit's `3-2-3-2` concrete launch chute (`laps = 2`, `joker.mandatory = 0`).
  - **Lap 1** is the out-lap from the launch chute to the main start/finish line; **Lap 2** is the timed flying lap that records the player's qualifying practice time.
  - All 30 drivers (`P1–P30`) are ranked by fastest flying lap and snake-seeded across the 3 Qualifying Heats of 10 cars each:
    - **Heat 1**: `P1, P6, P7, P12, P13, P18, P19, P24, P25, P30`
    - **Heat 2**: `P2, P5, P8, P11, P14, P17, P20, P23, P26, P29`
    - **Heat 3**: `P3, P4, P9, P10, P15, P16, P21, P22, P27, P28`

### 2. Round 1: Qualifying Heats (`Q1..=Qn`) & Intermediate Classification
- Each of the 3 Heats in a Qualifying Round lines up **10 cars** in the `3-2-3-2` staggered formation on the concrete launch pad and runs a pure dirt sprint (`heat_laps`, no Joker Lap).
- When `qualifying_rounds == 1` (default), completing the player's heat and simulating the remaining 2 heats computes the 30-driver **Intermediate Classification**.
- When `qualifying_rounds == 3` (full 3-heat FIA Autocross mode), Heat 1 results seed Heat 2, Heat 2 results seed Heat 3, and the **Intermediate Classification** ranks all 30 drivers by their **best 2 heat results out of 3** (dropping each driver's worst heat).
- The Top 10 qualifiers in the Intermediate Classification earn `10–9–8–7–6–5–4–3–2–1` championship points immediately, and the **Top 20** (`Q1–Q20`) advance to the two 10-car Semifinals.

### 3. Round 2 & Round 3: Two 10-Car Semifinals & The 10-Car Final Showdown
- **Semifinals (2 × 10 Cars, `3-2-3-2` Grid)**:
  - **Semifinal 1**: Intermediate ranks `1, 3, 5, 7, 9, 11, 13, 15, 17, 19`.
  - **Semifinal 2**: Intermediate ranks `2, 4, 6, 8, 10, 12, 14, 16, 18, 20`.
  - **Consolation Semifinal (Option A)**: If the player placed `21st–30th` in the Intermediate Classification, they race a 10-car Consolation Semifinal against the remaining non-qualifying drivers (`21st–30th`).
- **Finals (10 Cars, `3-2-3-2` Grid)**:
  - **Grand Final**: The **Top 5 finishers by position** from Semifinal 1 and the **Top 5 finishers** from Semifinal 2 advance to the 10-car Grand Final, paired by Semifinal finish (`S1-P1 vs S2-P1`, ..., `S1-P5 vs S2-P5`, ordered by Intermediate Classification rank) and slotted into the `3-2-3-2` starting formation (`Row 1: Slots 1–3`, `Row 2: Slots 4–5`, `Row 3: Slots 6–8`, `Row 4: Slots 9–10`).
  - **Consolation B-Final**: Finishers `6th–10th` from Semifinal 1 and Semifinal 2 contest the 10-car B-Final (if the player came from the Consolation Semifinal, they take Slot 10 in the B-Final so they race all stages).
- **Two-Phase Points Breakdown**:
  - Grand Final finishers `1st–10th` earn `25–18–15–12–10–8–6–4–2–1` points, summed with their Intermediate Classification points (`10..=1` for `Q1..=Q10`) on the Tournament Bracket and Championship Standings screens.

---

## ⚙️ Backend Models & API Endpoints

### 1. Packed Grid Pattern (`crates/arcade-race-core/src/track/presets.rs`)
- Rename/alias the `3-2-3-2` 10-car staggered grid variant to `PackedGridPattern::AutocrossThreeTwoThreeTwo` (keeping `RallycrossThreeTwoThreeTwo` as a type/serde alias or compatibility constant alongside `PackedGridPattern::RallycrossTwoTwoTwo` `[2, 2, 2]` from Spec 128 and `AutocrossFiveThree` `[5, 3]` for legacy/custom circuits):
  - `AutocrossThreeTwoThreeTwo.row_counts()` returns `&[3, 2, 3, 2]` (10 cars across 4 rows).
  - `AutocrossThreeTwoThreeTwo.label()` returns `"3-2-3-2 (AX)"`.
- Update `crates/tdrace-app/src/bin/build_launch_chutes.rs` and refresh `launch_chute.grid_slots` and `track.grid_positions` in place across all 17 circuits in `tracks/autocross/*.json` and all 3 circuits in `tracks/classic/ax_*.json` to use the 10-car `3-2-3-2` (`[3, 2, 3, 2]`) pattern on their existing concrete pads without moving chute geometry.

### 2. FIA Autocross 30-Driver Tournament Engine (`crates/tdrace-app/src/series/tournament.rs`)
- Support `total_drivers = 30` (FIA Autocross 10-car format) in `TournamentConfig`:
  - `grid_size() = 10`, `advancers_per_group() = 5`, `heat_count() = 3`, `has_superpole() = true`.
  - `qualifying_rounds: usize` (`1..=4`, default `1`):
    - Snake seeding from Timed Qualifying Practice (`superpole_results`, 30 drivers) into the 3 Heats of 10 cars (`idx` in `0..30`, `row = idx / 3`, `heat = if row % 2 == 0 { idx % 3 } else { 2 - (idx % 3) }`).
    - When `qualifying_rounds > 1`, each subsequent Heat round `Q(k+1)` is seeded from `Q(k)`'s overall finishing time order.
    - **Best-2-of-3 Intermediate Classification**: When `total_drivers == 30` and `qualifying_rounds >= 3`, each driver's Intermediate Classification time/score is computed from their **best 2 heat results out of 3** (dropping their single worst/slowest heat time). When `qualifying_rounds < 3`, cumulative heat time across completed heats is used.
  - **Odd/Even 10-Car Semifinals (Top 20)**:
    - `Semifinal 1`: Intermediate ranks `1, 3, 5, 7, 9, 11, 13, 15, 17, 19` (indices `0, 2, 4, 6, 8, 10, 12, 14, 16, 18`).
    - `Semifinal 2`: Intermediate ranks `2, 4, 6, 8, 10, 12, 14, 16, 18, 20` (indices `1, 3, 5, 7, 9, 11, 13, 15, 17, 19`).
    - `Consolation Semifinal`: Intermediate ranks `21..=30` (10 cars) when the player places `21st–30th`.
  - **Top-5 Position-Only Semifinal Advancement & 10-Car Final Grid Pairing**:
    - For `pos_idx` in `0..5`, pair `semi_results[0][pos_idx]` and `semi_results[1][pos_idx]`, ordering each pair by higher Intermediate Classification rank to populate Slots `1..=10` of the Grand Final (`3-2-3-2` grid).
    - Finishers `6th–10th` (`pos_idx` in `5..10`) from Semifinal 1 and Semifinal 2 pair into the 10-car Consolation B-Final.
  - **Two-Phase FIA Autocross Scoring**:
    - `FIA_AX_INTERMEDIATE_POINTS_10: [u32; 10] = [10, 9, 8, 7, 6, 5, 4, 3, 2, 1]`
    - `FIA_AX_FINAL_POINTS_10: [u32; 10] = [25, 18, 15, 12, 10, 8, 6, 4, 2, 1]`
    - Total weekend points for each driver = `intermediate_points + final_points`.

### 3. Expanded Autocross Career Season Calendars (`series/autocross/*.toml`)
- Update all 5 Autocross career championship TOMLs to `total_drivers = 30`, `bot_count = 9`, `heat_laps = 4`, `semi_laps = 5`, `final_laps = 6` (with Tier 1 at `3, 4, 5`), and progressive season calendars:
  1. **Tier 1 — `autocross_crosscar_junior_trophy.toml` (5 Rounds)**: `seelow_ax`, `bazaigues_ax`, `uelzen_ax`, `vilkyciai_ax`, `musa_ax`.
  2. **Tier 2 — `autocross_crosscar_senior_challenge.toml` (6 Rounds)**: `seelow_ax`, `arteixo_ax`, `castelo_branco_ax`, `st_georges_ax`, `musa_ax`, `maggiora_ax`.
  3. **Tier 3 — `autocross_buggy1600_championship.toml` (7 Rounds)**: `seelow_ax`, `musa_ax`, `vilkyciai_ax`, `matschenberg_ax`, `nova_paka_ax`, `st_georges_ax`, `maggiora_ax`.
  4. **Tier 4 — `autocross_touring_masters.toml` (8 Rounds)**: `seelow_ax`, `humpolec_ax`, `schluechtern_ax`, `faleyras_ax`, `carballo_ax`, `matschenberg_ax`, `prerov_ax`, `nova_paka_ax`.
  5. **Tier 5 — `autocross_superbuggy_world_series.toml` (10 Rounds)**: `seelow_ax`, `musa_ax`, `vilkyciai_ax`, `matschenberg_ax`, `nova_paka_ax`, `st_georges_ax`, `prerov_ax`, `maggiora_ax`, `st_junien_ax`, `arteixo_ax`.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

- **Anti-Tamper Result Validation**: `TournamentWeekendState::submit_player_race` validates that every 10-car Autocross heat, semifinal, or final submission matches the exact 10 scheduled `driver_ids` of the player's active race without duplicates or omissions.
- **Backwards Compatibility**: Legacy `total_drivers = 16 | 32 | 64` configurations remain supported in `TournamentConfig` so custom series and historical unit tests continue to function without breakage.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- Launch chute & circuit catalog validation tests:
  - `cargo test -p arcade-race-core --test launch_chute_tests`
  - `cargo test -p tdrace-core --test ax_rx_circuit_launch_chute_validation`
  - `cargo test -p tdrace-app --test track_editor_launch_chute_tests`
- Autocross tournament engine, scoring & calendar tests:
  - `cargo test -p tdrace-app --test autocross_tournament_tests`
  - `cargo test -p tdrace-app --test tournament_bracket_ui_tests`

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: 20 Autocross Circuits Use 10-Car 3-2-3-2 Staggered Concrete Launch Chutes**
  - [x] **Given** the 17 European Autocross circuits in `tracks/autocross/*.json` and the 3 Classic Autocross circuits in `tracks/classic/ax_*.json`
  - [x] **When** `validate_track()` and `PackedGridPattern::from_slots()` inspect each circuit's `LaunchChuteConfig`
  - [x] **Then** every Autocross chute should contain exactly 10 starting grid slots arranged in `3-2-3-2` (`[3, 2, 3, 2]`) across 4 staggered rows on a concrete launch pad with zero validation errors

- **Scenario: Stage 0 Timed Qualifying Practice and 30-Driver Snake Seeding into 3 Heats of 10 Cars**
  - [x] **Given** a 30-driver FIA Autocross championship weekend begins
  - [x] **When** Stage 0 Timed Qualifying Practice (`TournamentStage::SuperPole`) concludes after the player's 2-lap chute out-lap + flying lap run and deterministic AI flying-lap simulation
  - [x] **Then** all 30 drivers should be ranked `P1` through `P30` by flying lap time and snake-seeded into 3 Qualifying Heats of 10 cars each (`Heat 1: P1, P6, P7, P12, P13, P18, P19, P24, P25, P30`, `Heat 2: P2, P5, P8, P11, P14, P17, P20, P23, P26, P29`, `Heat 3: P3, P4, P9, P10, P15, P16, P21, P22, P27, P28`)

- **Scenario: Configurable Qualifying Heats (1 Default, Up to 3 Heats with Best 2-of-3 Intermediate Classification)**
  - [x] **Given** a 30-driver Autocross weekend supports `qualifying_rounds = 1..=3` (defaulting to `1` across standard career series)
  - [x] **When** an Autocross weekend is configured with `qualifying_rounds = 3` (`Heat 1, Heat 2, Heat 3`) and a driver suffers one slow heat or DNF in Heat 2 but records fast times in Heat 1 and Heat 3
  - [x] **Then** Heat 2 grids should be seeded from Heat 1 times, Heat 3 grids from Heat 2 results, and the Intermediate Classification should rank all 30 drivers using each driver's best 2 heat results out of 3 (dropping their worst heat)

- **Scenario: Top 20 Odd/Even 10-Car Semifinals and Top-5 Advancement to the 10-Car Final**
  - [x] **Given** the 30-driver Intermediate Classification is finalized after the Qualifying Heats
  - [x] **When** Round 2 (Semifinals) and Round 3 (Finals) are scheduled and completed
  - [x] **Then** the Top 20 drivers should split into Semifinal 1 (`1st, 3rd, ..., 19th`, 10 cars) and Semifinal 2 (`2nd, 4th, ..., 20th`, 10 cars), and the Top 5 finishers by position from each Semifinal should pair into the 10-car Grand Final (`3-2-3-2` grid) ordered within each pair by Intermediate Classification rank

- **Scenario: Option A Continuous Player Racing Across a 30-Driver Autocross Weekend**
  - [x] **Given** the player finishes `21st–30th` in the Intermediate Classification or `6th–10th` in a Semifinal
  - [x] **When** subsequent stages of the Autocross weekend are scheduled
  - [x] **Then** the player should be placed into a 10-car `ConsolationSemifinal` (`21st–30th`) and/or 10-car `ConsolationBFinal` so they actively race in every stage of the weekend

- **Scenario: Two-Phase FIA Autocross Championship Points Allocation**
  - [x] **Given** a 30-driver Autocross weekend completes both the Qualifying Heats and the 10-car Grand Final
  - [x] **When** championship points are awarded for the round
  - [x] **Then** each driver's weekend score should equal the sum of their Intermediate Standings points (`10, 9, 8, 7, 6, 5, 4, 3, 2, 1` for Intermediate ranks `1..=10`) and their Grand Final points (`25, 18, 15, 12, 10, 8, 6, 4, 2, 1` for Grand Final places `1..=10`), up to a maximum of 35 points

- **Scenario: Expanded Progressive Season Calendars Across All 5 Autocross Career Tiers**
  - [x] **Given** the 5 Autocross championship definitions in `series/autocross/*.toml`
  - [x] **When** loaded by `SeriesDefinition::from_toml_str`
  - [x] **Then** all 5 series should specify `total_drivers = 30` and `bot_count = 9`, with progressive calendar lengths of 5 rounds (Tier 1 Junior), 6 rounds (Tier 2 Senior), 7 rounds (Tier 3 Buggy1600), 8 rounds (Tier 4 TouringAutocross), and 10 rounds (Tier 5 SuperBuggy visiting Seelow, Mūsa, Vilkyčiai, Matschenberg, Nová Paka, Saint-Georges, Přerov, Maggiora, Saint-Junien, and Arteixo)

---

## 🔗 Traceability & Codebase Mapping

### Created/Modified Files
- `[x]` `crates/arcade-race-core/src/track/presets.rs` -> Configures `AutocrossThreeTwoThreeTwo` (`[3, 2, 3, 2]`, 10 cars) for Autocross launch chutes.
- `[x]` `crates/tdrace-app/src/bin/build_launch_chutes.rs` -> Updates Autocross launch chute grid slots in place to `3-2-3-2` (10 cars).
- `[x]` `tracks/autocross/*.json` & `tracks/classic/ax_*.json` -> Updated 10-car `3-2-3-2` starting grids on all 20 Autocross circuits.
- `[x]` `crates/tdrace-app/src/series/tournament.rs` -> Implements 30-driver 10-car FIA Autocross tournament flow, Timed Qualifying Practice snake seeding into 3 heats, Best-2-of-3 Intermediate Classification when `qualifying_rounds == 3`, Top 20 Odd/Even Semifinals, Top-5 advancement to the 10-car Final, and Two-Phase FIA AX points (`10..1` + `25..1`).
- `[x]` `crates/tdrace-app/src/series/format.rs` & `crates/tdrace-app/src/series/mod.rs` -> Supports 30-driver Autocross tournament validation and Two-Phase point attribution.
- `[x]` `crates/tdrace-app/src/ui/tournament_bracket.rs` -> Renders Autocross `TIMED QUALIFYING PRACTICE` header, 3 × 10-car heat cards, 30-driver Intermediate Classification table with Top-20 cutoff and Top-10 points, 2 × 10-car Semifinal cards, and 10-car Final / B-Final cards.
- `[x]` `series/autocross/*.toml` -> Updates all 5 Autocross career series to `total_drivers = 30`, `bot_count = 9`, and expanded `5 -> 6 -> 7 -> 8 -> 10` round European calendars.
- `[x]` `crates/arcade-race-core/tests/launch_chute_tests.rs`, `crates/tdrace-core/tests/ax_rx_circuit_launch_chute_validation.rs`, `crates/tdrace-app/tests/autocross_tournament_tests.rs`, `crates/tdrace-app/tests/tournament_bracket_ui_tests.rs` -> Automated test coverage for all Pseudo-Gherkin scenarios.

### Verification Assertions
- All 20 Autocross circuits pass `validate_track()` with 10-car `3-2-3-2` concrete launch chutes.
- All 30-driver FIA Autocross tournament tests (both 1-heat default and 3-heat Best-2-of-3 mode) pass.
