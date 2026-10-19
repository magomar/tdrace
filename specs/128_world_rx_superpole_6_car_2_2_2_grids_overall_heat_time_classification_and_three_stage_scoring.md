---
type: Feature Spec
template: feature
title: "World RX SuperPole, 6-Car 2-2-2 Grids, Overall Heat Time Classification, and Three-Stage Scoring"
description: "Upgrades the Rallycross (RX) tournament weekend format to authentic FIA World RX rules: 6-car 2-2-2 staggered launch chutes across all 23 RX circuits, a 24-driver field, Stage 0 SuperPole (1 out-lap + 1 flying lap from the chute, bots simulated), configurable multi-round Classificatory Heats (qualifying_rounds = 1 by default across existing series, extensible to Q1–Q4) ranked by cumulative overall completion time across all heats, Odd/Even 6-car Semifinals with top-3 position-only advancement, Row-by-Row Final grid pairing, and Three-Stage cumulative championship points, while preserving the 32-driver 8-car Autocross format untouched."
status: implemented
receipt: "docs/receipts/spec-128-receipt.md"
created: 2026-10-19
generated: { by: agent/antigravity, at: 2026-10-19T14:40:22Z }
verified: { by: human:mario, at: 2026-10-19T14:50:16Z, hash: "2216a3e0e222" }
depends_on:
  - "104"
  - "106"
amends:
  - "104"
  - "106"
---

# Feature Spec: World RX SuperPole, 6-Car 2-2-2 Grids, Overall Heat Time Classification, and Three-Stage Scoring 🌟

Amends [Spec 104](104_autocross_and_rallycross_tournament_sprint_weekend_format.md) and [Spec 106](106_classic_autocross_launch_chutes_and_tencar_rallycross_chutes.md) for **Rallycross (`rally` / `rallycross`) only**, aligning Rallycross championship weekends and launch chutes with real-world FIA World Rallycross Championship rules while keeping Autocross (`autocross`) untouched on its existing 32-driver, 8-car (`5-3` concrete launch chute) format:

1. **6-Car `2-2-2` Staggered Rallycross Launch Chutes**: All 23 official Rallycross circuits (20 in `tracks/rally/*.json` and 3 in `tracks/classic/rx_*.json`) use `PackedGridPattern::RallycrossTwoTwoTwo` (`[2, 2, 2]`, 6 cars in 3 staggered rows of 2 on the existing asphalt launch pad without shifting chute merge geometry).
2. **24-Driver Field & 6-Car Grids**: All Rallycross championship series (`series/rally/*.toml`) field **24 drivers** (`total_drivers = 24`, `grid_size = 6`, `bot_count = 5`) across 4 Classificatory Heats of 6 cars per Qualifying Round, 2 Semifinals of 6 cars, and a 6-car Final (plus 6-car Consolation Semifinal and 6-car B-Final for the Option A Continuous Racing Guarantee).
3. **Stage 0 — SuperPole (1 Out-Lap + 1 Flying Lap from Chute)**: Every Rallycross weekend begins with a solo timed run starting from the launch chute: **Lap 1 is the out-lap** from the chute to the main loop finish line, and **Lap 2 is the timed flying lap** (`laps = 2`, Joker Lap disabled `mandatory = 0`). AI drivers receive deterministic simulated flying lap times. SuperPole ranks all 24 drivers (`P1–P24`) and **snake-seeds** them into the 4 Classificatory Heats of Qualifying Round 1 (`Q1`).
4. **Round 1 — Configurable Multi-Round Classificatory Heats (`qualifying_rounds = 1..=4`, Default `1`) Ranked by Overall Total Completion Time**:
   - `TournamentConfig` and `[championship]` TOML metadata expose `qualifying_rounds: usize` (default `1`, valid range `1..=4` corresponding to `Q1..=Q4`). All existing Rallycross and Autocross series keep `qualifying_rounds = 1` (a single Qualifying Round `Q1`).
   - In each Qualifying Round `Q(k)` (`k = 1..=qualifying_rounds`), 4 Heats of 6 cars run `heat_laps` (`3 laps`, `1 mandatory Joker Lap`, `+30.0s` penalty if missed).
   - When `qualifying_rounds > 1`, completing `Q(k)` ranks all 24 drivers by their `Q(k)` overall completion time and snake-seeds them into the 4 Heats of `Q(k+1)`.
   - After all configured `qualifying_rounds` conclude, all 24 drivers are ranked together in a single **Intermediate Classification** sorted by **cumulative overall total completion time across all completed Qualifying Rounds (`Q1..=Qn`)** (not in-heat finishing position). The **Top 12** advance to the Semifinals (`13th–24th` are eliminated from main contention; if the player places `13th–24th`, they enter a 6-car Consolation Semifinal).
5. **Round 2 — Odd/Even Semifinals & Position-Only Advancement**: The Top 12 from the Intermediate Classification split into **Semifinal 1** (`1st, 3rd, 5th, 7th, 9th, 11th`) and **Semifinal 2** (`2nd, 4th, 6th, 8th, 10th, 12th`) for a `4-lap` sprint (`1 mandatory Joker Lap`). Elapsed time no longer governs advancement: the **Top 3 finishers by on-track position** in each Semifinal advance to the 6-car Grand Final; finishers `4th–6th` enter the 6-car B-Final.
6. **Round 3 — Row-by-Row Final Grid Pairing**: The 6-car Grand Final (`5 laps`, `1 mandatory Joker Lap`) pairs the Semifinalists row by row:
   - **Row 1 (Slots 1–2)**: Semifinal 1 Winner & Semifinal 2 Winner (higher Intermediate Classification rank takes Pole).
   - **Row 2 (Slots 3–4)**: Semifinal 1 2nd-place & Semifinal 2 2nd-place (ordered by Intermediate Classification rank).
   - **Row 3 (Slots 5–6)**: Semifinal 1 3rd-place & Semifinal 2 3rd-place (ordered by Intermediate Classification rank).
7. **Three-Stage Cumulative Championship Points**: Championship points are awarded and summed across all three competitive phases of the Rallycross weekend:
   - **Intermediate Classification (Top 12)**: `[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5]` pts (`0` pts for `13th–24th`).
   - **Semifinals (1st–6th in each of Semi 1 & Semi 2)**: `[6, 5, 4, 3, 2, 1]` pts (`0` pts for Consolation Semifinal).
   - **Grand Final (1st–6th)**: `[8, 5, 4, 3, 2, 1]` pts (`0` pts for B-Final).
   - Maximum weekend score is $16 + 6 + 8 = 30\text{ pts}$, rewarding both heat consistency and knockout racecraft.

---

## 🗺️ User Flow & Interface Design

### 1. Stage 0: SuperPole Solo Timed Run
- **Bracket Screen (`TournamentStage::SuperPole`)**:
  - When a Rallycross championship round opens, the Tournament Bracket screen highlights **Stage 0: SuperPole** ahead of Heats, Semifinals, and Finals.
  - The header badge displays `SUPERPOLE • SOLO FLYING LAP (1 OUT-LAP + 1 TIMED LAP)` and the action button prompts the player to launch their SuperPole run.
- **On-Track Execution**:
  - The player spawns alone (`1 car`) in Slot 1 of the circuit's `2-2-2` launch chute.
  - `total_laps = 2` and `joker.mandatory = 0` (no Joker detour required or penalized during SuperPole).
  - **Lap 1 (Out-Lap)**: Driving out of the launch chute, merging onto the main circuit, and completing the loop to cross the main finish line at full speed.
  - **Lap 2 (Flying Lap)**: The flying lap time (`last_lap_time` / `best_lap_time` on Lap 2, or total run time fallback if DNF) is recorded as the player's SuperPole qualifying time.
  - The 23 AI drivers receive deterministic simulated flying lap times based on the circuit's reference lap time and each AI driver's quality tier and driving style.
- **Snake Seeding into Qualifying Round 1 (`Q1`)**:
  - Sorting all 24 drivers by SuperPole flying lap time (`P1` fastest to `P24` slowest) deterministically populates the 4 Classificatory Heats of 6 cars each:
    - **Heat 1**: `P1, P8, P9, P16, P17, P24`
    - **Heat 2**: `P2, P7, P10, P15, P18, P23`
    - **Heat 3**: `P3, P6, P11, P14, P19, P22`
    - **Heat 4**: `P4, P5, P12, P13, P20, P21`

### 2. Round 1: Configurable Classificatory Heats (`Q1..=Qn`) & Intermediate Classification Table
- Each of the 4 Classificatory Heats in a Qualifying Round runs `heat_laps` (`3 laps`) with `6 cars` starting from the `2-2-2` launch chute and `1 mandatory Joker Lap` (`+30.0s` penalty if missed).
- When `qualifying_rounds == 1` (the default for all existing championships), completing the player's heat and simulating the other 3 heats immediately produces the final **Intermediate Classification** and transitions to the Semifinals.
- When `qualifying_rounds > 1` (`2..=4`), completing Qualifying Round `Q(k)` (`k < qualifying_rounds`) records the 4 heats of `Q(k)`, updates the running Intermediate Classification (sorted by cumulative completion time across `Q1..=Q(k)`), and snake-seeds the 24 drivers from `Q(k)`'s overall time ranking into the 4 heats of `Q(k+1)`. After the final configured Qualifying Round `Qn` finishes, the Top 12 in cumulative overall heat completion time advance to the Semifinals.
- The Tournament Bracket screen displays each driver's overall Intermediate Classification rank (`Q1–Q24`), cumulative heat time, Intermediate Classification points (`16..=5` for `Q1..=Q12`), and qualification cutoff indicator at `Q12`.

### 3. Round 2 & Round 3: Semifinals, Row-by-Row Final, and Three-Stage Points Summary
- **Semifinals (2 × 6 Cars, `4 laps`, `1 mandatory Joker Lap`)**:
  - **Semifinal 1**: Intermediate Classification ranks `1, 3, 5, 7, 9, 11`.
  - **Semifinal 2**: Intermediate Classification ranks `2, 4, 6, 8, 10, 12`.
  - **Consolation Semifinal (Option A)**: If the player finished `13th–24th` in the Intermediate Classification, they race a 6-car Consolation Semifinal against the 5 nearest non-qualifying drivers (`13th–18th`, with the player included in their relative slot).
- **Finals (6 Cars, `5 laps` Grand Final / `4 laps` B-Final, `1 mandatory Joker Lap`)**:
  - **Grand Final**: Top 3 finishers by position from Semifinal 1 and Top 3 from Semifinal 2, paired row by row (`Row 1: S1-P1 vs S2-P1`, `Row 2: S1-P2 vs S2-P2`, `Row 3: S1-P3 vs S2-P3`), with the driver holding the higher Intermediate Classification rank taking the inside/front slot of each row.
  - **Consolation B-Final**: Finishers `4th–6th` from Semifinal 1 and Semifinal 2 (paired row by row; if the player came from the Consolation Semifinal, they replace the lowest-seeded B-Final qualifier in Slot 6).
- **Weekend Classification & Cumulative Points**:
  - At the end of Round 3, each driver's weekend points equal `intermediate_points + semifinal_points + final_points`, and the Tournament Bracket displays the stage-by-stage breakdown alongside the updated championship standings.

---

## ⚙️ Backend Models & API Endpoints

### 1. Packed Grid Pattern (`crates/arcade-race-core/src/track/presets.rs`)
- Update the Rallycross packed launch chute pattern to `PackedGridPattern::RallycrossTwoTwoTwo` (with `#[serde(alias = "RallycrossThreeTwoThreeTwo")]` so any saved editor state deserializes cleanly):
  - `row_counts(&self) -> &'static [usize]` returns `&[2, 2, 2]` (6 grid slots across 3 rows of 2 cars).
  - `label(&self) -> &'static str` returns `"2-2-2 (RX)"`.
  - In `generate_packed_launch_grid`, when `pattern == PackedGridPattern::RallycrossTwoTwoTwo`, odd and even 2-car rows are laterally offset across the pad width so no car in Row 2 sits directly behind a car in Row 1 (`|y_row2 - y_row1| > 1.5 m`), while keeping both cars in a given row at the same longitudinal offset `s` so `PackedGridPattern::from_slots` reconstructs `[2, 2, 2]`.
- Refresh `launch_chute.grid_slots` and `track.grid_positions` in place across the 20 circuits in `tracks/rally/*.json` and the 3 circuits in `tracks/classic/rx_*.json` without altering chute spline control points, merge junctions, pad widths, or barrier walls.

### 2. Tournament Configuration & State (`crates/tdrace-app/src/series/tournament.rs`)
- Extend `TournamentStage` with `SuperPole` ahead of `QualifyingHeat`.
- Extend `TournamentConfig` (with backwards-compatible serde defaults for existing Autocross/Rallycross TOMLs and career saves):
  - `qualifying_rounds: usize` (default `1` via `#[serde(default = "default_qualifying_rounds")]`, validated to `1..=4`).
  - **Autocross (`total_drivers = 32` or `16` or `64`, `grid_size = 8`)**: Preserves Spec 104 behavior bit-for-bit (`has_superpole = false`, `qualifying_rounds = 1`, heat top-4 per heat advance, 8-car Semifinals and Finals, single-table `TOURNAMENT_POINTS_32`).
  - **Rallycross (`total_drivers = 24`, `grid_size = 6`, `has_superpole = true`)**:
    - `grid_size = 6`, `advancers_per_group = 3`, `heat_count() = 4`, `qualifying_rounds = 1` (by default across all existing series, configurable up to `4`).
    - `superpole_results: Vec<RoundDriverResult>` stores the 24 SuperPole flying-lap classifications (`P1–P24`).
    - `current_qualifying_round: usize` (`0..qualifying_rounds`) and `qualifying_round_results: Vec<Vec<Vec<RoundDriverResult>>>` track each completed Q round (`Q1..=Qn`), while `heat_results` continues to expose the latest completed Q round's 4 heats.
    - `intermediate_standings: Vec<RoundDriverResult>` stores the 24-driver overall heat classification ranked by cumulative total heat completion time across all completed Q rounds (`Q1..=Qn`).
    - Snake seeding from `superpole_results` into `Q1` (and from `Q(k)`'s overall time ranking into `Q(k+1)` when `qualifying_rounds > 1`): for rank index `idx` in `0..24`, `row = idx / 4`, `heat = if row % 2 == 0 { idx % 4 } else { 3 - (idx % 4) }`.
    - Odd/Even Semifinal seeding from `intermediate_standings[0..12]`: `Semi 1 = [0, 2, 4, 6, 8, 10]`, `Semi 2 = [1, 3, 5, 7, 9, 11]`.
    - Row-by-row Grand Final seeding from `semi_results[0][0..3]` and `semi_results[1][0..3]`: for `pos_idx` in `0..3`, pair `semi_results[0][pos_idx]` and `semi_results[1][pos_idx]`, placing the driver with the higher Intermediate Classification rank first.
    - Three-stage cumulative points calculation:
      - `WORLD_RX_INTERMEDIATE_POINTS_12: [u32; 12] = [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5]`
      - `WORLD_RX_SEMI_POINTS_6: [u32; 6] = [6, 5, 4, 3, 2, 1]`
      - `WORLD_RX_FINAL_POINTS_6: [u32; 6] = [8, 5, 4, 3, 2, 1]`

### 3. In-Race SuperPole Hook (`crates/tdrace-app/src/game/mod.rs`)
- When launching a tournament race in `TournamentStage::SuperPole`:
  - Spawn only the player car (`1 participant`, `0 AI bots` on track) in Slot 1 of the circuit's launch chute.
  - Set `laps = 2` (Lap 1 = Out-Lap from chute, Lap 2 = Timed Flying Lap).
  - Override `world.rules.joker.mandatory = 0` so no Joker Lap is required or penalized during SuperPole.
  - Upon completion, record the player's Lap 2 flying lap time (`best_lap_time` / `last_lap_time`) as `total_time` in `RoundDriverResult` for SuperPole submission.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

- **Anti-Tamper Result Validation**: `TournamentWeekendState::submit_player_race` validates that a `SuperPole` submission contains the player entry (or full 24-driver list if pre-merged), that `qualifying_rounds` is within `1..=4`, and that every 6-car heat, semifinal, or final submission matches the exact 6 scheduled `driver_ids` of the player's active race without duplicates or omissions.
- **Save Compatibility**: Existing `SeriesSession` JSON career saves and `series/autocross/*.toml` / `series/rally/*.toml` definitions deserialize without error via `#[serde(default)]` fields on `TournamentConfig` and `TournamentWeekendState`.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- Launch chute & catalog validation tests:
  - `cargo test -p arcade-race-core --test launch_chute_tests`
  - `cargo test -p tdrace-core --test ax_rx_circuit_launch_chute_validation`
  - `cargo test -p tdrace-app --test track_editor_launch_chute_tests`
- Tournament engine & bracket UI tests:
  - `cargo test -p tdrace-app --test autocross_tournament_tests`
  - `cargo test -p tdrace-app --test tournament_bracket_ui_tests`

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: 23 Rallycross Circuits Use 6-Car 2-2-2 Staggered Launch Chutes**
  - [x] **Given** the 20 official Rallycross circuits in `tracks/rally/*.json` and the 3 Classic Rallycross circuits in `tracks/classic/rx_*.json`
  - [x] **When** `validate_track()` and `PackedGridPattern::from_slots()` inspect each circuit's `LaunchChuteConfig`
  - [x] **Then** every Rallycross chute should contain exactly 6 starting grid slots arranged in `PackedGridPattern::RallycrossTwoTwoTwo` (`[2, 2, 2]`) with lateral staggering between adjacent rows (`|y_row2 - y_row1| > 1.5 m`) and zero validation errors, while all 20 Autocross circuits retain their 8-car `AutocrossFiveThree` (`[5, 3]`) concrete chutes

- **Scenario: Stage 0 SuperPole Solo Run and Snake Seeding into 4 Classificatory Heats**
  - [x] **Given** a 24-driver Rallycross championship weekend begins
  - [x] **When** Stage 0 (`TournamentStage::SuperPole`) starts and concludes after the player's 2-lap chute out-lap + flying lap run (with `joker.mandatory = 0`) and deterministic AI flying-lap simulation
  - [x] **Then** all 24 drivers should be ranked `P1` through `P24` by flying lap time and snake-seeded into 4 Classificatory Heats of 6 cars each (`Heat 1: P1, P8, P9, P16, P17, P24`, `Heat 2: P2, P7, P10, P15, P18, P23`, `Heat 3: P3, P6, P11, P14, P19, P22`, `Heat 4: P4, P5, P12, P13, P20, P21`)

- **Scenario: Overall Total Time Intermediate Classification and Odd/Even Semifinal Seeding**
  - [x] **Given** all 4 Classificatory Heats of 6 cars conclude their 3-lap sprint races (including any +30s missed-Joker penalties)
  - [x] **When** the Intermediate Classification is computed across all 24 drivers
  - [x] **Then** drivers should be ranked `1st` through `24th` strictly by overall total race completion time across all heats (so a fast 4th-place finisher in a rapid heat outranks a slower 2nd-place finisher in a slow heat), and the Top 12 should be seeded into Semifinal 1 (`1st, 3rd, 5th, 7th, 9th, 11th`) and Semifinal 2 (`2nd, 4th, 6th, 8th, 10th, 12th`)

- **Scenario: Configurable Multi-Round Qualifying Heats (Q1–Q4) with Default Single Qualifying Round**
  - [x] **Given** `TournamentConfig` supports a configurable `qualifying_rounds` setting (`1..=4`, defaulting to `1` across all existing championship series)
  - [x] **When** a Rallycross weekend is configured with `qualifying_rounds = 3` (`Q1, Q2, Q3`)
  - [x] **Then** the weekend should run 3 sequential Qualifying Heat rounds of 4 heats each (re-seeding each subsequent Q round from the previous Q round's overall time ranking) before computing the Intermediate Classification by cumulative total time across all 3 Q rounds and advancing the Top 12 to the Semifinals

- **Scenario: Position-Only Semifinal Advancement and Row-by-Row Final Grid Pairing**
  - [x] **Given** Semifinal 1 (6 cars) and Semifinal 2 (6 cars) conclude their 4-lap sprint races
  - [x] **When** the Grand Final grid is assembled
  - [x] **Then** the Top 3 finishers by on-track position from each Semifinal should advance to the 6-car Grand Final paired row by row (`Row 1: Semi 1 P1 & Semi 2 P1`, `Row 2: Semi 1 P2 & Semi 2 P2`, `Row 3: Semi 1 P3 & Semi 2 P3`), with the driver holding the higher Intermediate Classification rank taking the leading slot in each row

- **Scenario: Option A Continuous Player Racing Across a 24-Driver RX Weekend**
  - [x] **Given** the player finishes `13th–24th` in the Intermediate Classification or `4th–6th` in a Semifinal
  - [x] **When** subsequent stages of the Rallycross weekend are scheduled
  - [x] **Then** the player should be placed into a 6-car `ConsolationSemifinal` and/or 6-car `ConsolationBFinal` so they drive every scheduled stage of the weekend without an elimination lock

- **Scenario: Three-Stage Cumulative Championship Points Allocation**
  - [x] **Given** a 24-driver Rallycross weekend completes the Grand Final and B-Final
  - [x] **When** championship points are awarded for the round
  - [x] **Then** each driver's weekend points should equal the sum of their Intermediate Classification points (`16..=5` for ranks `1..=12`), Semifinal points (`6..=1` for places `1..=6` in Semi 1 and Semi 2), and Grand Final points (`8, 5, 4, 3, 2, 1` for places `1..=6` in the Grand Final), up to a maximum of 30 points

---

## 🔗 Traceability & Codebase Mapping

### Created/Modified Files
- `[x]` `crates/arcade-race-core/src/track/presets.rs` -> Replaces `RallycrossThreeTwoThreeTwo` with staggered 6-car `RallycrossTwoTwoTwo` (`[2, 2, 2]`).
- `[x]` `crates/tdrace-app/src/bin/build_launch_chutes.rs` -> Updates RX launch chute grid slots in place to `RallycrossTwoTwoTwo` without moving chute geometry.
- `[x]` `tracks/rally/*.json` & `tracks/classic/rx_*.json` -> Updated 6-car `2-2-2` starting grids on all 23 Rallycross circuits.
- `[x]` `crates/tdrace-app/src/series/tournament.rs` -> Implements `TournamentStage::SuperPole`, configurable `qualifying_rounds` (`1..=4`, default `1`), 24-driver 6-car World RX tournament flow, snake heat seeding, overall heat time Intermediate Classification, Odd/Even Semifinals, Row-by-Row Final grid pairing, and 3-stage cumulative points.
- `[x]` `crates/tdrace-app/src/series/format.rs` & `crates/tdrace-app/src/series/mod.rs` -> Supports `qualifying_rounds` config, 24-driver World RX tournament series validation, point attribution, and session progression.
- `[x]` `crates/tdrace-app/src/ui/tournament_bracket.rs` -> Renders SuperPole stage, Q-round progress (`Q1` or `Q1..Qn`), 24-driver Intermediate Classification table, 6-car heat/semi/final cards, and 3-stage cumulative points.
- `[x]` `crates/tdrace-app/src/game/mod.rs` -> Configures solo 2-lap SuperPole race from the launch chute with `joker.mandatory = 0` and records Lap 2 flying lap time.
- `[x]` `series/rally/*.toml` -> Updates all 7 Rallycross championship definitions to `total_drivers = 24` and `bot_count = 5` (retaining `qualifying_rounds = 1` default).
- `[x]` `crates/arcade-race-core/tests/launch_chute_tests.rs`, `crates/tdrace-core/tests/ax_rx_circuit_launch_chute_validation.rs`, `crates/tdrace-app/tests/track_editor_launch_chute_tests.rs`, `crates/tdrace-app/tests/autocross_tournament_tests.rs`, `crates/tdrace-app/tests/tournament_bracket_ui_tests.rs` -> Automated test coverage for all Pseudo-Gherkin scenarios.

### Verification Assertions
- All 43 launch-chute circuits pass `validate_track()` with 8-car `5-3` grids on the 20 Autocross circuits and 6-car `2-2-2` grids on the 23 Rallycross circuits.
- All Autocross (`32-driver`) and Rallycross (`24-driver`, both single-Q default and multi-Q `Q1..=Q4`) tournament tests pass.
