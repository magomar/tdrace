---
type: Feature Spec
template: feature
title: "Cross-Module AI Character Rosters and Dynamic Tier Assignment"
description: "Decouples experience tiers from character definitions, aligns 72 drivers to 6 styles, introduces uniform style sampling from the global pool, normal-distribution difficulty tiering, and persistent career rosters with 90% retention and probabilistic tier advancement."
status: implemented
created: 2026-09-23
generated: { by: agent/antigravity, at: 2026-09-23T23:25:00Z }
---
# Feature Spec 024: Cross-Module AI Character Rosters and Dynamic Tier Assignment 🏁🤖

A unified architectural specification that refactors the AI driver roster system in **TdRace**:
1. **Decouples level and experience from predefined character definitions**: Characters embody identity, tactical driving style, and minor personal parameter nuances—never baked-in performance tiers.
2. **Aligns all 72 handcrafted drivers** across all 6 motorsport modules with the canonical 6 driving styles defined in [`docs/engineering/ai_driving_archetypes_and_characters.md`](../docs/engineering/ai_driving_archetypes_and_characters.md) (exactly 2 drivers per style per module).
3. **Refactors casual race roster generation (Quick Race, Custom Race, Multiplayer bot fill)**: Removes legacy string fallbacks, allows player selection of difficulty tier (Rookie to Legend, 1..=5), samples from the global 72-driver pool with a uniform driving style distribution, and assigns racer tiers via a discrete normal bell curve centered at the target difficulty.
4. **Governs Career Mode roster persistence and tier unlock progression**: Retains 90% of rivals across tier unlocks while renewing 10%, subjecting retained rivals to a probabilistic skill progression check where drivers advance or hold their experience tier.

---

## 🎯 Objectives & Design Philosophy

1. **Pure Identity, Decoupled Quality**: A predefined driver character (`DriverCharacter`) represents *who* the racer is (lore, personality, livery, favorite cars, tactical driving style) and *how* they slightly differ from peers with the same style (minor parameter deltas). Experience, competence, and execution speed (`DriverTier` / `DriverQuality`) are assigned dynamically when configuring an event.
2. **Zero Technical Debt & Clean Removal of Legacy Fallbacks**: Completely remove obsolete string-based fallbacks (e.g. `ai_character = "pro"`, `"fast"`, `"club"`, `"brawler"`). Use direct orthogonal types (`DrivingStyle` and `DriverTier`).
3. **Uniform Style Diversity on Every Grid**: Rather than clustering around one or two styles or drawing only from a single discipline, casual races sample from all 72 drivers across all 6 modules such that each of the 6 driving styles has an equal probability of being chosen ($P(S_i) = \frac{1}{6}$), creating an authentic, heterogeneous racing field.
4. **Natural Field Spread (Bell-Curve Difficulty)**: Instead of all bots running at the exact same tier, bots are assigned tiers following a discrete normal distribution centered at the user-specified difficulty. A Tier 3 race features mostly Tier 3 contenders, accompanied by a few slightly faster Tier 4 pace-setters and slightly slower Tier 2 backmarkers.
5. **Durable Career Rivalries with Dynamic Churn**: Career mode maintains a persistent rival grid to foster narrative rivalries. When the player unlocks a higher tier, 90% of the grid advances together, with rivals passing probabilistic skill advancement checks, while 10% of the roster rotates out to introduce fresh competition.

```mermaid
flowchart TD
    subgraph Predefined Characters [72 Predefined Drivers Across 6 Modules]
        P1[Identity: Name, Bio, Alias]
        P2[Tactical Style: 1 of 6 DrivingStyles]
        P3[Visuals: Colors, Preferred Car, 26 Favorite Models]
        P4[Personality Deltas: Minor lookahead, kp, kd, brake, agg, avoid offsets]
    end

    subgraph Event Configuration [Race / Championship Setup]
        E1[Casual: User Chooses Difficulty Tier 1..=5]
        E2[Career: Tier Unlocks 1..=5 with Roster Evolution]
    end

    subgraph Sampling Engine [Global Roster Sampler]
        S1[Sample from Global 72 Pool Across All Modules]
        S2[Uniform Style Distribution: Equal 1/6 Probability per Style]
        S3[Discrete Normal Bell-Curve Tier Assignment]
    end

    subgraph Runtime Composition [BotProfile & DriverStats Synthesis]
        R1[BotProfile::from_character_and_tier]
        R2[DriverStats::from_character_and_tier]
    end

    Predefined Characters --> Sampling Engine
    Event Configuration --> Sampling Engine
    Sampling Engine --> Runtime Composition
    Runtime Composition --> SIM[60 Hz Closed-Loop AI Simulation]
```

---

## 🗺️ User Flow & Interface Design

### 1. Casual Race Experience (Quick Race & Custom Race)
When configuring a casual race in the main menu:
- **Difficulty Tier Selector**:
  - In Quick Race and Custom Race options, players select the AI difficulty tier via a stepper or dropdown:
    - `TIER 1: ROOKIE` (Grassroots / Cadet pace)
    - `TIER 2: AMATEUR` (Clubman baseline)
    - `TIER 3: CONTENDER` (National semi-pro pace)
    - `TIER 4: PRO` (Factory veteran pace)
    - `TIER 5: LEGEND` (World champion alien pace)
- **Grid Roster & Dossier Badging**:
  - The starting grid and race results screens display each bot's synthesized dossier card (`driver_card.rs`) with:
    - **Assigned Tier Badge**: Displays the bot's dynamically assigned tier (e.g. `TIER 3 CONTENDER`).
    - **Driving Style Badge**: Displays the character's intrinsic tactical style (e.g. `CALCULATING`, `AGGRESSIVE`).
    - **Discipline / Module Emblem**: Displays the pilot's home motorsport background (e.g. GT, NASCAR, Rally, Kart).

### 2. Multiplayer Lobby Configuration
When hosting a multiplayer room:
- The lobby host configures the `Bot Difficulty` setting (Tier 1..=5).
- Empty grid slots are filled with AI bots sampled across the global cross-module roster using the uniform style distribution and bell-curve tier assignment.
- All remote clients receive the deterministic seed, ensuring synchronized bot names, liveries, styles, and performance profiles across the network.

### 3. Career Mode Hub & Tier Evolution Flow
In single-player Career Mode:
- **Career Season Inception**:
  - When the player starts a new career in any motorsport module, the initial rival roster (e.g. 7 or 11 AI competitors) is sampled from the global 72-driver pool with uniform style distribution and Tier 1 centering.
  - The roster is saved to the career save file (`ModuleCareerProgress`), persisting across all championship rounds in that tier.
- **Tier Unlock Transition Screen**:
  - Upon earning a championship podium and advancing to tier $T+1$:
    - The Career Hub displays the **Season Roster Transition** summary.
    - **Retained Rivals (90%)**: A list of returning competitors showing development results (e.g., *"Marco Rossi stepped up to Tier 2"*, *"Elena Frost held steady at Tier 1"*), with progression probabilities modulated by championship ranking.
    - **Renewed Rivals (10%)**: A showcase card introducing the incoming rival drafted from the global pool using a uniform driving style distribution ($P(S_i) = 1/6$) and selecting an unused character fitting that style.

```mermaid
sequenceDiagram
    autonumber
    actor Player
    participant Career as ModuleCareerProgress
    participant Champ as ChampionshipSession
    participant Evolution as RosterEvolutionEngine
    participant Registry as Global Driver Registry

    Player->>Career: advance_tier() (e.g. Tier 1 -> Tier 2)
    Career->>Champ: trigger_roster_evolution(new_tier: 2)
    Champ->>Evolution: evolve_roster(current_roster, new_tier, seed)
    
    loop Each Retained Rival (90% of field)
        Evolution->>Evolution: evaluate_skill_progression(old_tier, new_tier, rank_index, seed)
        Note over Evolution: Rank-weighted progression (max 80% advance for 1st place, down to 70% for last place; field mean 65% step, 25% hold, 10% standout)
    end

    loop Churn Allocation (10% of field, min 1)
        Evolution->>Registry: draw_replacement_character(pool, uniform_random_style)
        Evolution->>Evolution: assign_bell_curve_tier(new_tier)
    end

    Evolution-->>Champ: Persist updated roster with continuity & fresh rivals
```

---

## ⚙️ Backend Models & API Endpoints

### 1. Canonical Alignment of 72 Characters to 6 Driving Styles
Every driver character across all 6 motorsport modules maps to one of the 6 pure driving styles (`Smooth`, `Aggressive`, `Tenacious`, `Calculating`, `Bold`, `Balanced`), exactly 2 drivers per style in each module:

| Module | Smooth (2) | Aggressive (2) | Tenacious (2) | Calculating (2) | Bold (2) | Balanced (2) | Total |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :---: |
| **Classic Heritage** | Silvia Tanaka, Maya Lin | Marco Rossi, Jax Reed | Viktor Sterling, Damon Clark | Elena Frost, Chloe Laurent | Kenji Sato, Zane Holland | Leo Bianchi, Hiroshi Takahashi | 12 |
| **GT World Challenge** | Charles Laurent, Takumi Sato | Max Hunter, Carlos Wolf | Fernando Toro, Lewis Vance | Sophia Becker, Liam Vance | Oscar Rocket, Pierre Gascon | George Speed, Lando Vance | 12 |
| **NASCAR Stock Car** | Richard Pettyfield, Bill Elliott | Dale Vance, Rowdy Busch | Bobby Allison, Rusty Wallace | Chase Gordon, Jimmie Johnson | Tony Stewart, Cale Yarborough | Bubba Wallace, Joey Logano | 12 |
| **Rally & Rallycross** | Timo Scheider, Sebastien Loebfield | Mattias Storm, Andreas Bakkerud | Kevin Hansenfield, Anton Mark | Johan Vance, Timmy Hansenfield | Petter Solbergfield, Ken Blaster | Niclas Gron, Reinis Nitissfield | 12 |
| **Karting Championship**| Sofia Lind, Kenzo Yamamoto | Marco Armani, Dante Moretti | Finn Korhonen, Marta Santos | Lucas Vance, Leo Dupont | Alex Rossi, Mateo Silva | Liam Callaghan, Charlie Webb | 12 |
| **Extreme Off-Road** | Astrid Lindholm, Elise Roux | Cruz Morales, Dakota Black | Bubba Beauregard, Sven Lindqvist | Roxie Vance, Diego Valdez | Wyatt Cole, Travis McGrath | Jaxson Rivera, Colton Haze | 12 |
| **Global Pool** | **12 Drivers** | **12 Drivers** | **12 Drivers** | **12 Drivers** | **12 Drivers** | **12 Drivers** | **72** |

> [!IMPORTANT]
> In existing code (`crates/tdrace-app/src/ai/driver.rs`), Maya Lin, Damon Clark, Chloe Laurent, and Hiroshi Takahashi must be corrected to match the canonical table above.

### 2. Decoupled Data Model: `DriverPersonalityOffsets`
Predefined characters no longer store fixed `BotProfile` or `DriverStats` with hardcoded Pro/Legend stats. Instead, they define zero-centered micro-variance parameter offsets (`DriverPersonalityOffsets`):

```rust
/// Minor scalar offsets applied to the composite BotProfile so that drivers sharing
/// the same style feel subtly unique without baking in a skill tier.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DriverPersonalityOffsets {
    pub delta_lookahead: f32,    // -0.02..+0.02 s
    pub delta_steering_kp: f32,  // -0.2..+0.2
    pub delta_steering_kd: f32,  // -0.01..+0.01
    pub delta_brake_margin: f32, // -0.03..+0.03
    pub delta_aggression: f32,   // -0.04..+0.04
    pub delta_avoidance: f32,    // -0.4..+0.4 m
    pub delta_speed_factor: f32, // -0.01..+0.015
}

impl DriverPersonalityOffsets {
    pub const ZERO: Self = Self {
        delta_lookahead: 0.0,
        delta_steering_kp: 0.0,
        delta_steering_kd: 0.0,
        delta_brake_margin: 0.0,
        delta_aggression: 0.0,
        delta_avoidance: 0.0,
        delta_speed_factor: 0.0,
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct DriverCharacter {
    pub id: &'static str,
    pub name: &'static str,
    pub alias: &'static str,
    pub bio: &'static str,
    pub style: DrivingStyle,
    pub preferred_car: CarChoice,
    pub color_scheme: CarColorScheme,
    pub offsets: DriverPersonalityOffsets,
    pub favorite_cars: &'static [DriverFavoriteCar],
}

impl DriverCharacter {
    /// Dynamically constructs the physics BotProfile for this character at the specified tier.
    pub fn resolve_profile(&self, tier: DriverTier) -> BotProfile {
        let quality = DriverQuality::for_tier(tier);
        let mut profile = BotProfile::from_style_and_quality(self.style, &quality);
        profile.name = self.name;
        profile.lookahead_time = (profile.lookahead_time + self.offsets.delta_lookahead).clamp(0.20, 0.55);
        profile.steering_kp = (profile.steering_kp + self.offsets.delta_steering_kp).clamp(1.5, 3.5);
        profile.steering_kd = (profile.steering_kd + self.offsets.delta_steering_kd).clamp(0.03, 0.12);
        profile.brake_margin = (profile.brake_margin + self.offsets.delta_brake_margin).clamp(0.80, 1.45);
        profile.aggression = (profile.aggression + self.offsets.delta_aggression).clamp(0.20, 1.00);
        profile.avoidance_distance = (profile.avoidance_distance + self.offsets.delta_avoidance).clamp(3.5, 12.0);
        profile.speed_factor = (profile.speed_factor + self.offsets.delta_speed_factor).clamp(0.80, 1.15);
        profile
    }

    /// Dynamically constructs the UI DriverStats for this character at the specified tier.
    pub fn resolve_stats(&self, tier: DriverTier) -> DriverStats {
        let quality = DriverQuality::for_tier(tier);
        let mut stats = DriverStats::from_style_and_quality(self.style, &quality);
        stats.speed = (stats.speed + self.offsets.delta_speed_factor * 1.5).clamp(0.60, 0.99);
        stats.aggression = (stats.aggression + self.offsets.delta_aggression).clamp(0.40, 0.99);
        stats.precision = (stats.precision + self.offsets.delta_steering_kd * 2.0).clamp(0.50, 0.99);
        stats.defense = (stats.defense - self.offsets.delta_avoidance * 0.05).clamp(0.50, 0.99);
        stats
    }
}
```

### 3. Uniform Driving Style Roster Sampling & Global Module Pool
When building an opponent field of size $N$ from the global pool of 72 characters:
1. The 6 driving styles are assigned balanced integer quotas:
   $$\text{base\_quota} = \lfloor N / 6 \rfloor, \quad \text{remainder} = N \pmod 6$$
   The remainder is distributed randomly across distinct styles without replacement.
2. For each style $S_k$, exactly $\text{quota}(S_k)$ characters are drawn without replacement from the 12 global characters possessing that style.
3. This guarantees that:
   - For $N = 12$, exactly 2 drivers of each style are present.
   - For $N = 8$, two styles have 2 drivers and four styles have 1 driver (selected randomly).
   - All 6 styles have identical expected selection frequency: $\mathbb{E}[N_S] = \frac{N}{6}$.

### 4. Discrete Normal Bell-Curve Difficulty Model
Rather than assigning all bots the exact same tier, each racer receives an individual tier drawn from a discrete normal distribution centered at the user-specified target tier $\mu = T_{\text{target}}$ with standard deviation $\sigma = 0.70$:

$$P(T = t \mid T_{\text{target}}) = \frac{\exp\left(-\frac{(t - T_{\text{target}})^2}{2\sigma^2}\right)}{\sum_{k=1}^5 \exp\left(-\frac{(k - T_{\text{target}})^2}{2\sigma^2}\right)}, \quad t \in \{1, 2, 3, 4, 5\}$$

#### Probability Matrix Across Tiers:
| Target Tier ($T_{\text{target}}$) | $P(T=1)$ | $P(T=2)$ | $P(T=3)$ | $P(T=4)$ | $P(T=5)$ | Field Character |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **Tier 1 (Rookie)** | **$74.3\%$** | $24.4\%$ | $1.3\%$ | $0.0\%$ | $0.0\%$ | Mostly Grassroots beginners; a few competent amateurs. |
| **Tier 2 (Amateur)** | $20.4\%$ | **$58.7\%$** | $20.4\%$ | $0.5\%$ | $0.0\%$ | Regional club racers; small mix of novices and contenders. |
| **Tier 3 (Contender)** | $1.1\%$ | $20.2\%$ | **$57.4\%$** | $20.2\%$ | $1.1\%$ | National semi-pros with balanced bell curve above and below. |
| **Tier 4 (Pro)** | $0.0\%$ | $0.5\%$ | $20.4\%$ | **$58.7\%$** | $20.4\%$ | Factory veterans; a few top contenders and world-class aliens. |
| **Tier 5 (Legend)** | $0.0\%$ | $0.0\%$ | $1.3\%$ | $24.4\%$ | **$74.3\%$** | World Champions; top factory pros pushing limits. |

### 5. Career Mode Persistence & Roster Evolution Engine
When the player advances to a newly unlocked career tier ($T \to T+1$):
- **Step 1: Rank-Dependent Probabilistic Rival Skill Advancement**:
  Skill progression is modulated by the participant's championship ranking while remaining predominantly stochastic.
  For a rival at finish order index $r_i \in [0, N-1]$ ($0$ is 1st place / champion, $N-1$ is last place):
  - Normalized performance score: $x_i = 1.0 - \frac{r_i}{N - 1}$ (for $N > 1$, or $0.5$ if $N \le 1$).
  - Performance delta: $d_i = x_i - 0.5 \in [-0.5, +0.5]$.
  - Outcome probabilities:
    - Breakout standout: $P_{\text{leap}}(d_i) = 0.10 + 0.05 \times d_i \quad (7.5\% \dots 12.5\%)$
    - Plateau retention: $P_{\text{hold}}(d_i) = 0.25 - 0.10 \times d_i \quad (20.0\% \dots 30.0\%)$
    - Regular progression: $P_{\text{step}}(d_i) = 1.0 - P_{\text{leap}}(d_i) - P_{\text{hold}}(d_i) = 0.65 + 0.05 \times d_i \quad (62.5\% \dots 67.5\%)$
  - Total advancement rate ($P_{\text{adv}} = P_{\text{step}} + P_{\text{leap}} = 1.0 - P_{\text{hold}}$):
    - **1st Place ($d_i = +0.5$)**: $80.0\%$ advance (capped at max 80%), $20.0\%$ hold steady ($67.5\%$ StepUp, $12.5\%$ StandoutLeap).
    - **Median ($d_i = 0.0$)**: $75.0\%$ advance, $25.0\%$ hold steady ($65.0\%$ StepUp, $10.0\%$ StandoutLeap).
    - **Last Place ($d_i = -0.5$)**: $70.0\%$ advance, $30.0\%$ hold steady ($62.5\%$ StepUp, $7.5\%$ StandoutLeap).
    - **Field Average**: Across symmetric ranks, exactly matches the macro baseline ($65\%$ StepUp, $25\%$ HoldSteady, $10\%$ StandoutLeap).
  - Tier assignments:
    - Regular progression: $T_{\text{rival}} \to T_{\text{new}}$
    - Plateau retention: $T_{\text{rival}} \to T_{\text{current}}$
    - Breakout standout: $T_{\text{rival}} \to \min(5, T_{\text{new}} + 1)$
- **Step 2: Roster Churn / Renewal ($90\%$ Retention, $10\%$ Turnover)**:
  - $\text{retained\_count} = \lfloor 0.90 \times N \rfloor$
  - $\text{churn\_count} = \max(1, N - \text{retained\_count})$
  - Churned rivals are chosen uniformly at random among non-podium finishers (podium finishers $0..2$ are protected).
  - For each departing rival, a replacement driving style is selected uniformly at random across all 6 driving styles ($P(S_i) = 1/6$), and an unused character fitting that style is drafted from the global 72-driver pool (falling back to any unused character if that style's pool is exhausted).
  - The incoming rival is assigned a tier sampled from the normal distribution of $T_{\text{new}}$.

### 6. Elimination of Legacy Fallback Paths
- All legacy string parsing methods (`BotProfile::from_archetype`, `DriverStats::from_archetype`) with branches for `"pro"`, `"fast"`, `"club"`, `"brawler"`, `"defender"`, `"hotlap"`, `"cautious"` are removed.
- `RaceSession` and `SeriesSession` store and deserialize strongly typed `DrivingStyle` and `DriverTier` exclusively.

---

## 🛡️ Security & Role-Based Access Controls (RBAC)

### 1. Zero-Allocation 60 Hz Simulation Loop
- Roster sampling and tier assignment occur exclusively during event/round transitions.
- The 60 Hz physics update loop (`BotAiDriver::compute_controls`) retains zero dynamic heap allocation, avoiding garbage collection pauses or frame stuttering.

### 2. Deterministic Seed Reproducibility & Multiplayer Anti-Desync
- Using an explicit 64-bit seed ensures identical grid rosters, starting tier spreads, and driver liveries when restarting or replaying a seed, eliminating desyncs in local split-screen and networked multiplayer.

### 3. Parameter Clamping & Safety Envelope Verification
- All generated control parameters are strictly clamped within physical stability envelopes:
  - `speed_factor` $\in [0.80, 1.15]$
  - `brake_margin` $\in [0.80, 1.45]$
  - `avoidance_distance` $\in [3.5, 12.0]$
  - `lookahead_time` $\in [0.20, 0.55]$
  - `steering_kp` $\in [1.5, 3.5]$
  - `steering_kd` $\in [0.03, 0.12]$

---

## 🧪 Verification & Acceptance Criteria

### Automated Tests
- Command to run test suite: `cargo test --test dynamic_roster_and_tier_tests`

### Manual Acceptance Criteria (Pseudo-Gherkin)

- **Scenario: 72 characters properly aligned across 6 styles without baked-in tiers**
  - [x] **Given** the global roster of 72 predefined driver characters across all 6 motorsport modules
  - [x] **When** counting characters assigned to each of the 6 `DrivingStyle` variants
  - [x] **Then** exactly 12 characters are assigned to each style globally
  - [x] **And** each individual module contains exactly 2 characters for each style
  - [x] **And** no `DriverCharacter` struct contains a hardcoded `DriverTier` or `DriverQuality`

- **Scenario: Uniform driving style distribution in casual race sampling**
  - [x] **Given** the global 72-driver pool across all registered modules
  - [x] **When** sampling 12-driver casual race rosters over 1,000 distinct seeds
  - [x] **Then** the mean count of each driving style across all generated rosters is $2.0 \pm 0.1$
  - [x] **And** no style deviates from equal probability ($P = 1/6$) beyond statistical tolerance

- **Scenario: Normal bell-curve tier distribution centered on user-selected difficulty**
  - [x] **Given** a user-specified difficulty setting of Tier 3 (Contender)
  - [x] **When** generating 100 casual race grids of 12 opponents
  - [x] **Then** approximately 55–60% of opponents are Tier 3
  - [x] **And** approximately 18–22% are Tier 2, and 18–22% are Tier 4
  - [x] **And** Tier 1 and Tier 5 represent rare boundary outliers (< 3% each)

- **Scenario: Boundary tier distributions for Tier 1 Rookie and Tier 5 Legend**
  - [x] **Given** a user-specified difficulty setting of Tier 1 (Rookie)
  - [x] **When** generating casual race grids
  - [x] **Then** no opponent is assigned a tier below Tier 1
  - [x] **And** at least 70% of opponents are Tier 1, with the remainder at Tier 2
  - [x] **Given** a user-specified difficulty setting of Tier 5 (Legend)
  - [x] **When** generating casual race grids
  - [x] **Then** no opponent is assigned a tier above Tier 5
  - [x] **And** at least 70% of opponents are Tier 5, with the remainder at Tier 4

- **Scenario: Career mode 90% retention and 10% churn on tier unlock**
  - [x] **Given** an active 10-car career championship grid at Tier 1
  - [x] **When** the player advances career progress to Tier 2
  - [x] **Then** exactly 9 rivals are retained from the previous tier and 1 rival is renewed
  - [x] **And** the incoming rival is selected via uniform driving style distribution from the global pool
  - [x] **And** the retained rivals evaluate rank-weighted probabilistic skill checks where higher-ranking rivals have a higher advancement probability (up to 80%) than lower-ranking rivals

- **Scenario: Total elimination of legacy single-string ai_character fallbacks**
  - [x] **Given** race initialization code paths for quick race, custom race, and multiplayer
  - [x] **When** verifying driver configuration and bot instantiation
  - [x] **Then** no code path falls back to legacy string parsing (`"pro"`, `"fast"`, `"club"`, `"brawler"`)
  - [x] **And** all AI opponents are configured strictly through `DrivingStyle` and `DriverTier`

---

## 🔗 Traceability & Codebase Mapping

### Target Files to Modify / Create
- `[x]` `crates/tdrace-app/src/ai/driver.rs` -> Decouple `DriverCharacter`, add `DriverPersonalityOffsets`, implement `resolve_profile` / `resolve_stats`, correct Classic module styles, and implement global uniform style roster sampler.
- `[x]` `crates/tdrace-app/src/module/gt.rs` -> Decouple GT driver definitions and align offsets.
- `[x]` `crates/tdrace-app/src/module/nascar.rs` -> Decouple NASCAR driver definitions and align offsets.
- `[x]` `crates/tdrace-app/src/module/rally.rs` -> Decouple Rally driver definitions and align offsets.
- `[x]` `crates/tdrace-app/src/module/kart.rs` -> Decouple Kart driver definitions and align offsets.
- `[x]` `crates/tdrace-app/src/module/extreme_offroad.rs` -> Decouple Extreme Off-Road driver definitions and align offsets.
- `[x]` `crates/tdrace-app/src/game/mod.rs` -> Refactor casual race (quick race, custom race, multiplayer) bot generation to use dynamic difficulty tier and global style-uniform sampling; remove legacy fallbacks.
- `[x]` `crates/tdrace-app/src/series/session.rs` / `manager.rs` -> Implement career roster persistence, 90% retention / 10% turnover, and probabilistic skill progression on tier advancement.
- `[x]` `crates/tdrace-app/src/ui/menu.rs` -> Add difficulty tier selector (Rookie..Legend) to casual race configuration options.
- `[x]` `specs/constitution/ROADMAP.md` -> Register Spec 024 in living roadmap.
- `[x]` `specs/index.md` -> Register Spec 024 in OKF Progressive Disclosure Index.
- `[x]` `crates/tdrace-app/tests/dynamic_roster_and_tier_tests.rs` -> Test suite covering 72-driver style alignment, uniform sampling, bell curve distribution, and career churn.
