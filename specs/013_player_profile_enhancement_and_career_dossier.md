---
type: Feature Spec
template: feature
title: "Player Profile Enhancement & Career Dossier"
description: "Full-screen player profile dossier with integrated driver switcher, multi-level career hierarchy (Global -> Category -> Championship -> Race), stunt points, collision telemetry, and three selectable layout alternatives."
status: draft
created: 2026-09-20
generated: { by: agent/antigravity, at: 2026-09-20T21:45:00Z }
---

# Feature Spec: Player Profile Enhancement & Career Dossier 🌟

A comprehensive overhaul of the **Player Profile (`GameState::ProfileManager`)** in **TdRace**. This specification eliminates the legacy two-column constraint—which permanently allocated 36% of the screen width to a static driver roster list—and expands the player dossier across the entire viewport. The enhanced profile models a hierarchical motorsport career system (**Global Driver Lifetime → Discipline Categories [GT, NASCAR, Rally, Off-Road, Kart, Classic] → Multi-Round Championships → Race Telemetry**) with rich tracking of race podiums (P1, P2, P3), acrobatic stunt points, and collision incident metrics. Furthermore, this specification codifies **three distinct layout alternatives** (Tabbed Telemetry Dashboard, 3-Column Executive Widescreen, and Hierarchical Drill-Down Dossier) to be implemented and evaluated across dedicated Git worktrees.

---

## 🗺️ User Flow & Interface Design

### 1. Architectural Motivation & Viewport Expansion
Previously, `render_profile_manager_screen` rendered an inner box restricted to 90% screen width, internally divided into `col1_w` (left 36% roster) and `col2_w` (right 64% details). This severely cramped telemetry tables (allowing only 6 visible rows and 6 columns) and prevented displaying multi-category career trees.

The redesigned screen reclaims 100% of the active card area:
1. **Integrated Driver Switcher**: The active profile is displayed prominently in a top hero banner with hotkeys `[◄ Q]` / `[E ►]` (or Gamepad Bumpers `[LB]` / `[RB]`) to cycle drivers instantly, and `[TAB]` to summon a modal quick-picker if many drivers exist.
2. **Multi-Level Career Modeling**:
   - **Level 1 (Global Profile)**: Aggregate career statistics across all racing modes and categories.
   - **Level 2 (Discipline Careers)**: Category-specific progression (GT World Challenge, NASCAR Stock Car, WRC Rally, Extreme Off-Road, Karting, Classic), including Career Tiers (1–5), XP bars, and vehicle/circuit unlocks.
   - **Level 3 (Championships)**: Tournament standings, round-by-round results, point totals, and earned trophies (Gold, Silver, Bronze).
   - **Level 4 (Race Telemetry Logs)**: Deep chronological history with finishing positions, track names, vehicle models, total race times, personal best (PB) lap times, stunt scores, collision counts, and clean race badges.

---

### 2. Three Layout Alternatives

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Alternative A: Tabbed Motorsport Telemetry Dashboard (F1 / Gran Turismo)     │
├──────────────────────────────────────────────────────────────────────────────┤
│ [FLAG] RACER ONE — "Apex Legend"        [◄ Q / E ► Switch Driver] [LIVERY]   │
│ Level 12 Global Driver • 48,250 Lifetime XP • ESP • 24 Trophies (12G/8S/4B)  │
├──────────────────────────────────────────────────────────────────────────────┤
│ [1] GLOBAL OVERVIEW  │ [2] CAREER DISCIPLINES │ [3] CHAMPIONSHIPS │ [4] LOGS │
├──────────────────────────────────────────────────────────────────────────────┤
│  • 6 KPI Cards: Races, Wins (P1), P2, P3, Podiums, Clean Race %, Laps        │
│  • Stunt Portfolio: Air Time, Drift Score, Longest Jump, Combos              │
│  • Incident Box: Total Collisions, Clean Races, Safety Rating                │
│  • Category Career Progress & Unlocks                                        │
│  • Granular 10-Column Race History with Personal Best Badges                 │
└──────────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────────┐
│ Alternative B: 3-Column Executive Widescreen (Grid / Dirt Rally)             │
├──────────────────────────────────────────────────────────────────────────────┤
│ [FLAG] RACER ONE — "Apex Legend"              [◄ Q / E ► Switch Driver]      │
├───────────────────┬───────────────────────────────┬──────────────────────────┤
│ CAREER CATEGORIES │ ACTIVE CHAMPIONSHIPS & ROUNDS │ LIFETIME METRICS & LOGS  │
│ [Up/Down]         │                               │                          │
│ ► GT World Cup    │ • Round 1: Monza (P1) [Gold]  │ • Total Races: 42        │
│   NASCAR Series   │ • Round 2: Red Bull Ring (P2) │ • Wins: 28 (66%)         │
│   WRC Rallycross  │ • Round 3: Nurburgring [Next] │ • P1: 28 | P2: 8 | P3: 4 │
│   Extreme Off-Road│                               │ • Stunt Pts: 14,850      │
│   World Karting   │ Standings: 1st (68 Pts)       │ • Collisions: 12         │
│   Classic GP      │ Next Car: Tier 3 (2,400 XP)   │ • Clean Races: 32 (76%)  │
└───────────────────┴───────────────────────────────┴──────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────────┐
│ Alternative C: Hierarchical Drill-Down Dossier (Forza Motorsport)            │
├──────────────────────────────────────────────────────────────────────────────┤
│ Breadcrumb: Driver Profile > GT World Challenge > Tier 2 FIA GT3             │
├──────────────────────────────────────────────────────────────────────────────┤
│ ┌───────────────────────────┐  ┌───────────────────────────┐                 │
│ │   LIFETIME MOTORSPORT     │  │   CAREER DISCIPLINES      │                 │
│ │   HALL OF FAME & RECORDS  │  │   6 Racing Categories     │                 │
│ │   [ENTER] Open Stats      │  │   [ENTER] Select Category │                 │
│ └───────────────────────────┘  └───────────────────────────┘                 │
│ ┌───────────────────────────┐  ┌───────────────────────────┐                 │
│ │   CHAMPIONSHIP TROPHIES   │  │   RECENT RACE TELEMETRY   │                 │
│ │   Gold / Silver / Bronze  │  │   Detailed Log Dossier    │                 │
│ │   [ENTER] View Showcase   │  │   [ENTER] View Records    │                 │
│ └───────────────────────────┘  └───────────────────────────┘                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

#### Alternative A: Tabbed Motorsport Telemetry Dashboard (Recommended)
- **Top Full-Width Hero**: Displays driver country banner, name, racing alias, nationality, team livery swatches, global driver level, and lifetime XP progress bar.
- **Header Switcher**: `[◄ Q] Driver Name [E ►]` allows instant profile switching without navigating away.
- **Widescreen Tabs**:
  - `[1] GLOBAL OVERVIEW`:
    - 6 KPI stat tiles: Total Races, Wins (P1), P2 Finishes, P3 Finishes, Podiums (P1–P3), Clean Race %.
    - Stunt Performance Showcase: Total Stunt Score, Total Drift Points, Max Drift Score, Total Air Time, Longest Jump, Max Combo.
    - Safety & Incident Record: Total Collisions (Wall + Vehicle), Clean Races Count, Incident Frequency.
    - Trophy Showcase: Total Gold, Silver, Bronze championship trophies.
  - `[2] CAREER DISCIPLINES`:
    - 6-discipline grid: **GT**, **NASCAR**, **Rally**, **Extreme Off-Road**, **Karting**, **Classic**.
    - Each discipline card displays current Tier (1–5), XP progress bar towards next tier car, unlocked vehicles (`X/Y`), explored tracks (`X/Y`), and category-specific win rate.
  - `[3] CHAMPIONSHIPS`:
    - Comprehensive championship registry with round status (Completed with Trophy, In Progress with Standings, or Locked).
  - `[4] RACE TELEMETRY & LOGS`:
    - Full-screen 10-column telemetry table with category filter pills (`ALL`, `GT`, `NASCAR`, `RALLY`, `OFF-ROAD`, `KART`, `CLASSIC`).
    - Columns: `POS`, `CAT`, `TRACK`, `CAR`, `TIME`, `BEST LAP` (with `[PB]` badge), `STUNTS`, `COLLISIONS`, `CLEAN?`, `DATE`.

#### Alternative B: 3-Column Executive Widescreen
- **Left Pane (26% width)**: Vertical discipline career list with real-time Tier badges and XP bars.
- **Center Pane (44% width)**: Active championships, round results, and vehicle garage unlocks for the highlighted discipline.
- **Right Pane (30% width)**: Global lifetime driver stats, acrobatic stunt points, collision counters, and recent telemetry feed.

#### Alternative C: Hierarchical Drill-Down Dossier
- **Breadcrumb Navigation**: `Profile > Category > Championship > Round`.
- **Level 0 Dashboard**: 4 interactive visual portal cards (Lifetime Stats, Career Disciplines, Trophy Showcase, Telemetry Dossier).
- **Interactive Drill-Down**: Selecting any card enters a focused full-screen view. `[ESC]` steps back up.

---

## ⚙️ Backend Models & API Endpoints

### 1. SQLite Data Schema Extensions
In [`crates/tdrace-app/src/db/mod.rs`](../crates/tdrace-app/src/db/mod.rs), extend the `race_history` table schema:

```sql
ALTER TABLE race_history ADD COLUMN category TEXT NOT NULL DEFAULT 'gt';
ALTER TABLE race_history ADD COLUMN championship_name TEXT;
ALTER TABLE race_history ADD COLUMN stunt_score INTEGER NOT NULL DEFAULT 0;
ALTER TABLE race_history ADD COLUMN collisions INTEGER NOT NULL DEFAULT 0;
```

### 2. Rust Domain Models
In [`crates/tdrace-app/src/profile/mod.rs`](../crates/tdrace-app/src/profile/mod.rs):

```rust
/// Individual race completion entry logged in the history database.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RaceHistoryEntry {
    pub id: Option<i64>,
    pub profile_id: i64,
    pub track_id: String,
    pub car_name: String,
    pub position: usize,
    pub total_cars: usize,
    pub total_time: f32,
    pub best_lap: Option<f32>,
    pub laps: u32,
    pub is_time_attack: bool,
    pub created_at: String,
    // Enhanced Multi-Level Telemetry
    pub category: String,
    pub championship_name: Option<String>,
    pub stunt_score: u32,
    pub collisions: u32,
}

/// Comprehensive aggregated career statistics computed across all history logs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GlobalProfileStats {
    pub total_races: u32,
    pub wins: u32,       // P1
    pub p2_count: u32,   // P2
    pub p3_count: u32,   // P3
    pub podiums: u32,    // P1 + P2 + P3
    pub total_laps: u32,
    pub win_rate: f32,
    pub podium_rate: f32,
    // Acrobatic Stunt Aggregations
    pub total_stunt_score: u32,
    pub total_drift_points: u32,
    pub max_drift_score: f32,
    pub total_air_time: f32,
    pub longest_jump_time: f32,
    pub max_combo: u32,
    // Collision & Incident Aggregations
    pub total_collisions: u32,
    pub clean_races: u32,
    pub clean_race_rate: f32,
    // Category Breakdown
    pub category_stats: BTreeMap<String, CategoryCareerStats>,
    pub best_times: BTreeMap<String, f32>,
    pub best_circuit_times: BTreeMap<String, f32>,
}
```

### 3. Simulation Incident & Stunt Tracking
In [`crates/tdrace-app/src/game/mod.rs`](../crates/tdrace-app/src/game/mod.rs):
- Increment `self.player_race_stats.collision_count` during wall impacts ($> 2.2\,\text{m/s}$) and vehicle collisions ($> 2.0\,\text{m/s}$).
- Upon race completion, record `category`, `championship_name`, `stunt_score`, and `collisions` into `insert_race_history`.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

### 1. Local Persistence Gating & Profile Ownership
- All profile and telemetry logs are restricted to local SQLite execution (`tdrace_records.db`).
- Foreign key cascading deletes (`ON DELETE CASCADE`) guarantee that deleting a driver profile cleanly purges associated race history records without leaving orphaned telemetry data.
- Input validation sanitizes driver names and aliases, preventing SQL injection and layout overflow.

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- Unit test suite in `crates/tdrace-app/tests/profile_tests.rs`:
  - `test_global_profile_stats_computation`: Validates P1, P2, P3 counts, podium rate, stunt score sum, and collision counting.
  - `test_database_schema_backward_compatibility`: Validates existing database schema upgrades seamlessly.
  - `test_driver_switching_navigation`: Validates cycling between driver profiles.

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: Full-Screen Viewport Utilization Without Left Roster**
  - [ ] **Given** the user navigates to the Player Profile screen (`GameState::ProfileManager`)
  - [ ] **When** the screen is rendered on any display resolution
  - [ ] **Then** the UI card must occupy the full available width ($94\%\text{--}98\%$ of screen width)
  - [ ] **And** no static left-hand driver roster column shall be rendered
  - [ ] **And** the active driver's national flag, alias, livery colors, and level shall span the top hero banner.

- **Scenario: Inline Driver Profile Cycling**
  - [ ] **Given** multiple driver profiles exist in the database
  - [ ] **When** the user presses `[Q]` or `[Left Arrow]` or `[Gamepad LB]`
  - [ ] **Then** the active profile switches to the previous driver profile
  - [ ] **And** all career statistics, telemetry logs, and category unlocks immediately refresh to reflect that driver.
  - [ ] **When** the user presses `[E]` or `[Right Arrow]` or `[Gamepad RB]`
  - [ ] **Then** the active profile switches to the next driver profile.

- **Scenario: Granular Race Outcomes (P1, P2, P3) & Podium Rate Computation**
  - [ ] **Given** a driver has finished races in positions 1, 2, 3, and 5
  - [ ] **When** `GlobalProfileStats::compute` is evaluated
  - [ ] **Then** `wins` (P1) shall equal 1
  - [ ] **And** `p2_count` shall equal 1
  - [ ] **And** `p3_count` shall equal 1
  - [ ] **And** `podiums` shall equal 3
  - [ ] **And** `win_rate` shall equal $25.0\%$ and `podium_rate` shall equal $75.0\%$.

- **Scenario: Acrobatic Stunt Points & Collision Incident Telemetry**
  - [ ] **Given** a race where the player accumulates $2,400$ drift points, performs 3 jumps with $4.5\,\text{s}$ air time, and impacts 2 walls
  - [ ] **When** the race completes and telemetry is logged
  - [ ] **Then** the `RaceHistoryEntry` shall record `stunt_score = 2400` and `collisions = 2`
  - [ ] **And** the profile overview shall reflect the accumulated stunt points and collision count
  - [ ] **And** races with zero collisions shall be tagged as `CLEAN` and increment `clean_races`.

- **Scenario: Multi-Level Category Progression (GT, NASCAR, Rally, Off-Road, Kart, Classic)**
  - [ ] **Given** the driver has participated in both GT and Extreme Off-Road events
  - [ ] **When** viewing the Career Disciplines tab or category breakdown
  - [ ] **Then** separate progress cards shall display the independent Career Tier, spendable XP, vehicle unlocks, and win rates for each discipline.

- **Scenario: Layout Alternatives Validation Across Worktrees**
  - [ ] **Given** the three layout alternatives:
    - Alternative A: Tabbed Motorsport Telemetry Dashboard
    - Alternative B: 3-Column Executive Widescreen
    - Alternative C: Hierarchical Drill-Down Dossier
  - [ ] **When** each option is checked out and built in its respective worktree
  - [ ] **Then** `cargo test` shall pass with zero errors
  - [ ] **And** the visual presentation shall strictly adhere to that layout's architectural contract.

---

## 🔗 Traceability & Codebase Mapping

### Created/Modified Crates
- [`crates/tdrace-app`](../crates/tdrace-app) - UI layout, database schema, game state navigation, and simulation collision hooks.

### Beads Epic Mapping
- Epic: `tdrace-spec-013-player-profile-mg2q` (Fulfill Spec 013: Player Profile Enhancement & Career Dossier)
