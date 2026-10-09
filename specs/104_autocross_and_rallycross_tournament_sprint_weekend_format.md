---
type: Feature Spec
template: feature
title: "Autocross and Rallycross Tournament Sprint Weekend Format"
description: "Implements the authentic 3-stage tournament weekend format (4 Qualifying Heats -> 2 Semifinals -> Grand Final & Consolation B-Final) across a 32-driver field for Autocross and Rallycross championships, providing dynamic grid seeding, Option A continuous player racing, bracket visualization, and 64-driver 4-stage scalability."
status: implemented
receipt: "docs/receipts/spec-104-receipt.md"
verified: { by: human:mario, at: 2026-10-08T21:40:00Z, hash: "7bf904cb7151" }
created: 2026-10-08
generated: { by: agent/antigravity, at: 2026-10-08T23:01:00Z }
depends_on:
  - "017"
  - "050"
  - "051"
  - "103"
---

# Feature Spec 104: Autocross and Rallycross Tournament Sprint Weekend Format 🏆🏎️

Unlike circuit racing (GT World Challenge, NASCAR, Karting) where an entire field of 16–20 cars competes in a single monolithic feature race, **FIA Autocross** and **Rallycross (World RX)** are contested as high-stakes, short-course knockout sprint tournaments. Because dirt tracks are narrow, fast, and feature blinding roost/dust, grids are strictly capped at 8 cars per race.

A championship event unfolds as a multi-stage weekend:
- **32 competitors** (Player + 31 AI competitors) enter the weekend.
- **Stage 1 (Qualifying Heats)**: 4 heats of 8 cars contest 4-lap sprints. The top 4 from each heat (16 drivers total) advance to the Semifinals.
- **Stage 2 (Semifinals)**: 2 semifinals of 8 cars contest 5-lap sprints. The top 4 from each semi (8 drivers total) advance to the Grand Final.
- **Stage 3 (Finals)**: The top 8 drivers battle in the 6-lap **Grand Final** for the podium and maximum championship points. Drivers ranked 5th–8th from the semifinals battle in the 5-lap **Consolation B-Final** for 9th–16th position points.
- **Option A Continuous Racing Guarantee**: The player is never prematurely eliminated or relegated to a spectator screen; players failing to make a top-4 cut transition directly into the corresponding consolation race, ensuring active participation across all 3 weekend stages.
- **Scalability**: The binary bracket architecture ($2^N$) seamlessly expands to a **4-Stage, 64-driver format** (8 Heats $\to$ 4 Quarterfinals $\to$ 2 Semifinals $\to$ Final) for premier Tier 5 World Championships.

---

## 🗺️ User Flow & Interface Design

### 1. Tournament Weekend Lifecycle

```mermaid
flowchart TD
    Roster["32-Driver Weekend Field\n(Player + 31 Homologated AI Rivals)"] --> Stage1["STAGE 1: 4 QUALIFYING HEATS (4 Laps)\n• Heat 1, 2, 3, 4 (8 cars each) = 32 drivers\n• Top 4 from each Heat advance to Main Semifinals (16 drivers)\n• Pos 5–8 enter Consolation Bracket"]
    
    Stage1 -->|Top 4 from Each Heat (16 Drivers)| Stage2["STAGE 2: 2 SEMIFINALS (5 Laps)\n• Semi 1 (8 cars): Seeds from Heats 1 & 3\n• Semi 2 (8 cars): Seeds from Heats 2 & 4\n• Grid ordered by Heat lap times"]
    
    Stage2 -->|Top 4 from Semi 1 + Top 4 from Semi 2| FinalA["STAGE 3A: GRAND FINAL (6 Laps)\n• 8 Best Drivers Contest Overall Victory\n• Awarded Championship Points for Positions 1–8\n• Podium Ceremony (Gold, Silver, Bronze)"]
    
    Stage2 -->|Pos 5–8 from Semi 1 + Pos 5–8 from Semi 2| FinalB["STAGE 3B: B-FINAL (5 Laps)\n• Contests Championship Points for Positions 9–16\n• Option A Guarantee: Continuous Racing for Player"]
    
    Stage1 -.->|Player in Pos 5–8 of Heat| ConsolationSemi["Stage 2B: Consolation Semifinal (5 Laps)\n• Keeps Player in Active Race Rotation"]
    ConsolationSemi -.-> FinalB

    FinalA --> Standings["Round Summary & Championship Standings Update (1st–32nd)"]
    FinalB --> Standings
```

### 2. Stage Navigation & In-Race HUD
- **Tournament Bracket Screen (`GameState::TournamentBracket`)**:
  - Displays the 32-driver tree with interactive cards showing driver names, cars, teams, and finishing positions.
  - Highlights the player's active heat/semi slot with an amber glowing border.
  - Interactive `[ LAUNCH RACE ]` button proceeds into the 3D race session.
- **HUD Stage Badge**:
  - The top HUD bar displays the active stage: `HEAT [X]/4`, `SEMIFINAL [X]/2`, `GRAND FINAL`, or `B-FINAL`.
  - Dynamic lap counter reflects stage sprint distance (4 laps for Heats, 5 laps for Semis/B-Final, 6 laps for Grand Final).
- **Continuous Racing Guarantee (Option A)**:
  - If the player finishes 5th or lower in their Qualifying Heat, they are seeded into a Consolation Semifinal.
  - If the player finishes 5th or lower in their Semifinal, they are seeded into the Consolation B-Final.
  - The player is guaranteed to race in all 3 stages of every championship round.

---

## ⚙️ Backend Models & API Endpoints

### 1. Tournament Series Definition & Runtime Session
In [`crates/tdrace-app/src/series/mod.rs`](../crates/tdrace-app/src/series/mod.rs):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeekendFormat {
    /// Traditional single race per circuit round (GT, NASCAR, Karting).
    StandardSingleRace,
    /// 3-Stage Tournament Weekend (Autocross, Rallycross).
    TournamentSprint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TournamentStage {
    QualifyingHeat,
    Semifinal,
    ConsolationBFinal,
    GrandFinal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TournamentWeekendState {
    pub active_stage: TournamentStage,
    /// 4 Qualifying Heats (8 drivers each = 32 drivers total).
    pub heat_results: Vec<Vec<RaceDriverResult>>,
    /// 2 Semifinals (8 drivers each = 16 drivers total).
    pub semi_results: Vec<Vec<RaceDriverResult>>,
    /// Consolation B-Final (positions 9–16).
    pub b_final_results: Option<Vec<RaceDriverResult>>,
    /// Grand Final (positions 1–8).
    pub grand_final_results: Option<Vec<RaceDriverResult>>,
}
```

### 2. Declarative TOML Series Integration
In `series/autocross/*.toml` and `series/rally/*.toml`:

```toml
[championship]
id = "autocross_superbuggy_world_series"
name = "World SuperBuggy Series"
weekend_format = "tournament_sprint"
total_drivers = 32
heat_laps = 4
semi_laps = 5
final_laps = 6
```

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

1. **Determinism & Seeding Integrity**:
   - Driver heat allocation and dynamic grid slot seeding must execute with deterministic sorting (seeded PRNG for initial heat draws, elapsed race time comparisons for semifinal and final seeding).
   - Guarantees bit-identical tournament brackets and replay rollouts across platforms.
2. **Anti-Tamper Point Distribution**:
   - Points are strictly allocated from a verified 32-position scoring matrix (e.g. 1st = 25 pts down to 32nd = 1 pt).
   - Incomplete stages or aborted heats cannot claim final points; DNF drivers receive last place in their respective heat.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- `cargo test -p race-kit --test tournament_weekend_tests`
- `cargo test -p tdrace-app --test autocross_tournament_tests`
- `cargo test -p tdrace-app --test tournament_bracket_ui_tests`

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: 32-Driver Tournament Field Initialization**
  - [x] **Given** an Autocross or Rallycross championship configured with `weekend_format = "tournament_sprint"`
  - [x] **When** Round 1 begins
  - [x] **Then** a 32-driver roster (Player + 31 AI drivers) should be assembled
  - [x] **And** drivers should be partitioned into 4 Qualifying Heats of 8 cars each

- **Scenario: Qualifying Heat Execution and Semifinal Seeding**
  - [x] **Given** Stage 1 (Qualifying Heats) commences
  - [x] **When** all 4 heats conclude 4-lap sprint races
  - [x] **Then** the top 4 drivers from each of the 4 heats (16 drivers total) should advance to the Semifinals
  - [x] **And** drivers should receive seeds for Semifinal 1 and Semifinal 2 based on elapsed heat finish times

- **Scenario: Semifinal to Grand Final Progression (Top 4 Advance)**
  - [x] **Given** the 16 advancing drivers split into Semifinal 1 (8 cars) and Semifinal 2 (8 cars)
  - [x] **When** Stage 2 concludes 5-lap sprint races
  - [x] **Then** the top 4 drivers from Semifinal 1 and the top 4 drivers from Semifinal 2 (8 drivers total) should advance to the **Grand Final**
  - [x] **And** drivers finishing 5th through 8th in either semifinal should advance to the **Consolation B-Final**

- **Scenario: Continuous Player Racing (Option A Guarantee)**
  - [x] **Given** the player finishes in position 5, 6, 7, or 8 in their Stage 1 Heat or Stage 2 Semifinal
  - [x] **When** subsequent stages load
  - [x] **Then** the player should be assigned to an active Consolation race (Consolation Semi or B-Final)
  - [x] **And** the player should race all 3 stages of the weekend without encountering an elimination screen or spectate-only lock

- **Scenario: Championship Points Allocation Across 32 Drivers**
  - [x] **Given** Stage 3 concludes with the completion of the Grand Final and B-Final
  - [x] **When** round results are tabulated
  - [x] **Then** all 32 drivers should receive designated championship points according to their final finishing rank (1st through 32nd)
  - [x] **And** the player profile and standings table should update atomically

---

## 🔗 Traceability & Codebase Mapping

### Headless Simulation & Timing Layer
- [`crates/race-kit/src/world.rs`](../crates/race-kit/src/world.rs): Heat and stage completion hooks, finish line timing.

### Series & Tournament Engine Layer
- [`crates/tdrace-app/src/series/mod.rs`](../crates/tdrace-app/src/series/mod.rs): `TournamentWeekendState`, `WeekendFormat::TournamentSprint`, dynamic grid seeding.
- [`crates/tdrace-app/src/series/format.rs`](../crates/tdrace-app/src/series/format.rs): Declarative TOML tournament weekend parsing.

### UI & Presentation Layer
- [`crates/tdrace-app/src/ui/mod.rs`](../crates/tdrace-app/src/ui/mod.rs): In-race HUD stage badges (`HEAT 1/4`, `SEMIFINAL`, `GRAND FINAL`).
- [`series/autocross/*.toml`](../series/autocross/): Declaring `weekend_format = "tournament_sprint"`.
- [`series/rally/*.toml`](../series/rally/): Declaring `weekend_format = "tournament_sprint"`.
