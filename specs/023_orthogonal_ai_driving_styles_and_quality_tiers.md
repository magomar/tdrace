---
type: Feature Spec
template: feature
title: "Orthogonal AI Driving Styles and Quality Tiers"
description: "Decouples AI driving style (tactical personality) from experience and quality (competence, consistency, execution) into orthogonal dimensions across 6 styles and 5 performance tiers."
status: implemented
created: 2026-09-23
generated: { by: agent/antigravity, at: 2026-09-23T19:15:00Z }
---

# Feature Spec 023: Orthogonal AI Driving Styles and Quality Tiers 🏁🤖

A comprehensive specification that decouples the AI driving model in **TdRace** into two strictly orthogonal dimensions:
1. **Driving Style (`DrivingStyle`)**: The driver's tactical philosophy, line preference, and risk tolerance (6 distinct styles).
2. **Experience & Quality (`DriverTier` / `DriverQuality`)**: The driver's execution competence, speed limit exploitation, braking margin calibration, and lap-to-lap consistency (5 performance tiers).

This eliminates the architectural conflation where skill/experience designations (such as "Rookie" or "Fast / Pro") were erroneously treated as lateral driving styles alongside "Smooth" or "Aggressive".

---

## 🎯 Objectives & Design Philosophy

1. **Orthogonal Separation of Concerns**: Clearly distinguish *how* a driver intends to drive (Style) from *how well* they execute that intention (Experience & Quality).
2. **$6 \times 5 = 30$ Behavioral Combinations**: Provide 30 unique, mathematically distinct AI profiles combining 6 pure driving styles and 5 experience tiers.
3. **1:1 Alignment with 5 Motorsport Tiers**: Directly align the 5 driver quality tiers (`Tier 1` Grassroots to `Tier 5` World Champion) with TdRace's established 5-tier vehicle performance architecture (`DriverFavoriteCar { tier: 1..=5, .. }`).
4. **Authentic Racing Emergence**: Enable archetypes such as the "Aggressive Rookie" (divebombs without braking room, prone to running wide) versus the "Aggressive World Champion" (surgical, high-percentage late-braking overtakes without collision).
5. **Backwards Compatibility**: Seamlessly maintain existing `ai_character` TOML strings and character presets by mapping legacy names (e.g., `"smooth"`, `"aggressive"`) to default Pro/Clubman tiers, while allowing explicit `ai_style` and `ai_tier` properties.

---

## 🗺️ User Flow & Interface Design

### 1. Driver Dossier & Roster Card Display (`driver_card.rs`)
When the player inspects driver profiles in the Motorsport Dossier:
- **Dual Badge Header**: The top-right of the driver profile card displays both badges side by side:
  - `TIER [1..5] [ROOKIE | AMATEUR | CONTENDER | PRO | LEGEND]` (Gold/Cyan badge)
  - `STYLE: [SMOOTH | AGGRESSIVE | TENACIOUS | CALCULATING | BOLD | BALANCED]` (Neon Green badge)
- **Dynamic Skill Bars (`DriverStats`)**:
  - `PACE & SPEED`: Scales monotonically with `DriverQuality.pace_limit` ($0.88\times \to 1.04\times$).
  - `OVERTAKE AGGRESSION`: Governed by `DrivingStyle.aggression` ($0.65 \to 0.95$).
  - `APEX PRECISION`: Derived from `DriverQuality.consistency` ($0.60 \to 0.99$) and `DrivingStyle.base_precision`.
  - `DEFENSIVE POSITION`: Governed by `DrivingStyle.base_defense` and `DriverQuality.composure`.

### 2. Championship Series Authoring & Experience Flow
When loading or editing championship series TOML files (`series/**/*.toml`):
- Championship creators and system presets can author AI opponents with specific skill tiers suited for the series tier:
  - Tier 1 Entry Series (e.g. Junior Karts, Street Stocks): Default opponents configured at `ai_tier = 1` (Rookie) and `ai_tier = 2` (Amateur).
  - Tier 5 Pinnacle Series (e.g. Superkarts, Hypercars, Trophy Trucks): Opponents configured at `ai_tier = 4` (Pro) and `ai_tier = 5` (Legend).

```mermaid
flowchart TD
    subgraph Driving Styles [Dimension 1: Driving Style (6 Styles)]
        S1[Smooth: Precision, minimal tire scrub, momentum lines]
        S2[Aggressive: Late braking, divebombs, inside line contest]
        S3[Tenacious: Door-closing defense, track position priority]
        S4[Calculating: Draft lock, patient slipstream, gap exploitation]
        S5[Bold: High slip angles, fast yaw rotation, curb hopping]
        S6[Balanced: Adaptable, versatile, all-rounder baseline]
    end

    subgraph Quality Tiers [Dimension 2: Experience & Quality (5 Tiers)]
        T1[Tier 1: Rookie / Novice - Grassroots, wide safety buffers, high variance]
        T2[Tier 2: Amateur / Clubman - Regional club, cautious, consistent baseline]
        T3[Tier 3: Contender / Semi-Pro - National series, assertive, solid racecraft]
        T4[Tier 4: Pro / Veteran - Factory pilot, threshold braking, tight pack tolerance]
        T5[Tier 5: Legend / Elite - World Champion, limit exploitation, zero unforced errors]
    end

    subgraph Composite Engine [Runtime Composition]
        M[BotProfile::from_style_and_quality]
        N[DriverStats::from_style_and_quality]
    end

    Driving Styles --> M
    Quality Tiers --> M
    Driving Styles --> N
    Quality Tiers --> N

    M --> P[BotProfile: 7 Physics Control Law Parameters]
    N --> Q[DriverStats: 4 UI Dossier & Result Stats]
```

---

## ⚙️ Backend Models & API Endpoints

### 1. Core Data Structures (`crates/tdrace-app/src/ai/mod.rs` & `driver.rs`)

```rust
/// Tactical philosophy and driving personality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrivingStyle {
    Smooth,
    Aggressive,
    Tenacious,
    Calculating,
    Bold,
    Balanced,
}

/// 5-tier experience and performance ladder matching vehicle tiers 1..=5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum DriverTier {
    Rookie = 1,
    Amateur = 2,
    Contender = 3,
    Pro = 4,
    Legend = 5,
}

impl DriverTier {
    pub const fn from_u8(val: u8) -> Self {
        match val {
            1 => Self::Rookie,
            2 => Self::Amateur,
            3 => Self::Contender,
            4 => Self::Pro,
            _ => Self::Legend,
        }
    }

    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::Rookie => "Tier 1: Rookie",
            Self::Amateur => "Tier 2: Amateur",
            Self::Contender => "Tier 3: Contender",
            Self::Pro => "Tier 4: Pro",
            Self::Legend => "Tier 5: Legend",
        }
    }
}

/// Operationalized performance attributes of a driver quality level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DriverQuality {
    pub tier: DriverTier,
    pub pace_limit: f32,
    pub brake_padding: f32,
    pub avoidance_padding: f32,
    pub consistency: f32,
    pub composure: f32,
}

impl DriverQuality {
    pub const fn for_tier(tier: DriverTier) -> Self {
        match tier {
            DriverTier::Rookie => Self {
                tier,
                pace_limit: 0.88,
                brake_padding: 0.22,
                avoidance_padding: 2.5,
                consistency: 0.60,
                composure: 0.50,
            },
            DriverTier::Amateur => Self {
                tier,
                pace_limit: 0.92,
                brake_padding: 0.14,
                avoidance_padding: 1.5,
                consistency: 0.72,
                composure: 0.65,
            },
            DriverTier::Contender => Self {
                tier,
                pace_limit: 0.96,
                brake_padding: 0.07,
                avoidance_padding: 0.8,
                consistency: 0.83,
                composure: 0.78,
            },
            DriverTier::Pro => Self {
                tier,
                pace_limit: 1.00,
                brake_padding: 0.00,
                avoidance_padding: 0.0,
                consistency: 0.93,
                composure: 0.90,
            },
            DriverTier::Legend => Self {
                tier,
                pace_limit: 1.04,
                brake_padding: -0.03,
                avoidance_padding: -0.5,
                consistency: 0.99,
                composure: 0.98,
            },
        }
    }
}
```

### 2. Composition into `BotProfile` Physics Laws

```rust
impl BotProfile {
    pub fn from_style_and_quality(style: DrivingStyle, quality: &DriverQuality) -> Self {
        let (base_lookahead, base_kp, base_kd, style_brake, style_aggression, style_avoidance, style_speed_mult) = match style {
            DrivingStyle::Smooth => (0.40, 2.3, 0.08, 1.02, 0.70, 6.5, 1.01),
            DrivingStyle::Aggressive => (0.32, 2.5, 0.05, 0.90, 0.95, 5.0, 1.02),
            DrivingStyle::Tenacious => (0.42, 2.2, 0.08, 1.05, 0.82, 6.0, 0.99),
            DrivingStyle::Calculating => (0.39, 2.4, 0.07, 1.00, 0.75, 6.5, 1.00),
            DrivingStyle::Bold => (0.31, 2.7, 0.04, 0.88, 0.92, 5.2, 1.01),
            DrivingStyle::Balanced => (0.38, 2.1, 0.07, 1.05, 0.65, 7.0, 0.98),
        };

        Self {
            name: "Composite Driver",
            lookahead_time: (base_lookahead * (0.80 + 0.20 * quality.consistency)).clamp(0.20, 0.55),
            speed_factor: (quality.pace_limit * style_speed_mult).clamp(0.80, 1.15),
            steering_kp: base_kp,
            steering_kd: base_kd * (0.75 + 0.25 * quality.consistency),
            brake_margin: (style_brake + quality.brake_padding).clamp(0.80, 1.45),
            aggression: style_aggression,
            avoidance_distance: (style_avoidance + quality.avoidance_padding).clamp(3.5, 12.0),
        }
    }
}
```

### 3. Declarative Series TOML Schema Extension (`crates/tdrace-app/src/series/format.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverConfig {
    pub name: String,
    pub car_model: String,
    pub livery_index: Option<usize>,
    /// Modern orthogonal style specification ("smooth", "aggressive", etc.)
    pub ai_style: Option<String>,
    /// Modern orthogonal tier specification (1..=5 or "rookie".."legend")
    pub ai_tier: Option<u8>,
    /// Backwards-compatible legacy single-string specification
    pub ai_character: Option<String>,
}
```

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

### 1. Memory Safety & Zero-Allocation Simulation Loop
- `BotProfile::from_style_and_quality` calculates a fixed stack-allocated `BotProfile` struct during race setup.
- The 60 Hz physics update loop (`BotAiDriver::compute_controls`) retains zero dynamic heap allocation, avoiding garbage collection pauses or frame stuttering.

### 2. Deterministic Clamping & Boundary Guards
- Input tiers and styling strings from modded or authored TOML files are sanitized. Unrecognized styles fall back to `DrivingStyle::Balanced`, and out-of-range tiers are clamped to `[1, 5]`.
- All generated control parameters are clamped within physically stable bounds, preventing simulation divergence or vehicle catapulting:
  - `speed_factor` $\in [0.80, 1.15]$
  - `brake_margin` $\in [0.80, 1.45]$
  - `avoidance_distance` $\in [3.5, 12.0]$
  - `lookahead_time` $\in [0.20, 0.55]$

---

## 🧪 Verification & Acceptance Criteria

### Manual Acceptance Criteria (Pseudo-Gherkin)

### Scenario: 30 distinct behavioral combinations generated without panic
- **Given** the 6 `DrivingStyle` variants and 5 `DriverTier` variants
- **When** iterating through all 30 pairs and calling `BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(tier))`
- **Then** each resulting `BotProfile` contains valid, strictly positive physical values
- **And** no parameter results in `NaN` or `inf`

### Scenario: Monotonic pace progression across tiers for identical style
- **Given** any constant `DrivingStyle` (e.g., `DrivingStyle::Aggressive`)
- **When** evaluating `speed_factor` for tiers 1 through 5
- **Then** `speed_factor(T1) < speed_factor(T2) < speed_factor(T3) < speed_factor(T4) < speed_factor(T5)`
- **And** Tier 1 has the largest braking distance margin while Tier 5 has the tightest

### Scenario: Monotonic safety and avoidance padding across tiers
- **Given** any constant `DrivingStyle` (e.g., `DrivingStyle::Smooth`)
- **When** comparing `avoidance_distance` across all 5 tiers
- **Then** Tier 1 Rookie maintains the largest avoidance bubble ($\ge 9.0\,\text{m}$)
- **And** Tier 5 Legend maintains the tightest wheel-to-wheel proximity ($\le 6.0\,\text{m}$)

### Scenario: Backwards compatibility with legacy archetype strings
- **Given** legacy AI character strings `"rookie"`, `"fast"`, `"smooth"`, `"aggressive"`
- **When** invoking `BotProfile::from_archetype(name)`
- **Then** `"rookie"` resolves to `DrivingStyle::Balanced` with `DriverTier::Rookie`
- **And** `"fast"` resolves to `DriverTier::Legend` pace
- **And** legacy unit tests continue to pass without regression

### Scenario: Declarative series TOML deserialization of style and tier
- **Given** a series TOML entry containing `ai_style = "bold"` and `ai_tier = 2`
- **When** parsing the driver config into a `RaceSession`
- **Then** the driver participant's AI profile matches `DrivingStyle::Bold` composed with `DriverTier::Amateur`

---

## 🔗 Traceability & Codebase Mapping

### Target Files to Modify / Create
- `[x]` `crates/tdrace-app/src/ai/mod.rs` -> Define `DrivingStyle`, `DriverTier`, `DriverQuality`, and `BotProfile::from_style_and_quality`.
- `[x]` `crates/tdrace-app/src/ai/driver.rs` -> Update `DriverStats::from_style_and_quality`, backwards-compatible resolvers, and index round-robins.
- `[x]` `crates/tdrace-app/src/series/format.rs` -> Add optional `ai_style: Option<String>` and `ai_tier: Option<u8>` to `DriverConfig`.
- `[x]` `crates/tdrace-app/src/ui/driver_card.rs` -> Render dual Style and Tier badges on driver dossier cards.
- `[x]` `docs/engineering/ai_driving_archetypes_and_characters.md` -> Update engineering specification with the 6 styles and 5 tiers.
- `[x]` `specs/index.md` -> Register Spec 023.
- `[x]` `specs/constitution/ROADMAP.md` -> Add Spec 023 milestone item.
- `[x]` `crates/tdrace-app/tests/orthogonal_ai_styles_and_tiers_tests.rs` -> Dedicated integration test suite verifying the 30 combinations and backward compatibility.
