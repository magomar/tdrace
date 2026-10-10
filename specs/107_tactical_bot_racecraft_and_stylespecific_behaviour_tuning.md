---
type: Architecture Spec
template: architecture
title: "Tactical Bot Racecraft and Style-Specific Behaviour Tuning"
description: "Add defend and slipstream-follow tactics to the bot driver and tune the per-style settings on the spec 083 blaze pipeline, so the six driving styles race visibly differently in a pack."
status: draft
created: 2026-10-11
generated: { by: agent/claude, at: 2026-10-11T00:00:00Z }
depends_on:
  - "083"
---

# Architecture Spec 107: Tactical Bot Racecraft and Style-Specific Behaviour Tuning 🏁

This spec was split out of the first draft of spec 083 on 2026-10-11. It uses the spec 083 pipeline
(`bot_eval_worker`, EvoRL campaigns on blaze) and adds the tactics that the first draft assumed already
existed.

---

## 🔍 Context

- **Styles differ in mistake mix, not in racecraft.** `bot_behaviour_report.md` shows clear mistake mixes
  per style. In a pack, all styles use the same reactive overtake and avoidance logic.
- **There is no defend logic.** No bot code reacts to a car behind. `defense` exists only as a UI stat in
  `DriverStats`.
- **Drafting exists in physics, not in tactics.** `Car::compute_draft_intensity`
  (`crates/wheelbase/src/car.rs`) gives a drag reduction in a 2–32 m wake cone. Bots follow a wake only
  by chance.
- **Telemetry that can be measured today:** `CarState.sideslip_angle` (body slip), per-wheel
  `WheelTelemetry` (slip angle, slip ratio, lateral and longitudinal force) and the draft intensity.
  Tyre scrub energy is not measured. It must be derived as force x slip velocity.

---

## 🗺️ Current vs. Proposed System Architecture

### 1. Tactical layer in `race-kit`
A new module `crates/race-kit/src/ai/tactics.rs` holds a small state machine that is evaluated each
tick inside `BotAiDriver`:

```rust
pub enum TacticalState {
    Nominal,
    FollowWake { target: usize },
    Defend { threat: usize },
    Pass { side: f32, commit_s: f32 },
}
```

- **Defend:** a car behind with time-to-contact below `defend_ttc_s` and an open inside lane biases the
  target line toward the inside by `defend_bias` of the free half-width.
- **Follow wake:** a car ahead inside the physics wake cone, on a straight, steers toward its wake with
  gain `wake_gain`. The bot pulls out when its closing speed exceeds `pass_delta_v`.
- **Pass:** the existing overtake logic, entered from `FollowWake` or directly.
- **Fixed size:** the state is a plain `Copy` enum and fixed-size fields. The tick allocates nothing.
- **Only legal driving:** no state may push another car off the road on purpose or block by weaving more
  than once per straight.

### 2. New per-style settings
| Setting | Meaning | Search box |
| :--- | :--- | :--- |
| `defend_ttc_s` | Time-to-contact that starts a defend | 0.8 – 4.0 s |
| `defend_bias` | Inside-line bias while defending | 0.0 – 0.8 |
| `wake_gain` | Steering gain toward a leader's wake | 0.0 – 0.3 |
| `pass_delta_v` | Closing speed that starts a pass from the wake | 0.5 – 5.0 m/s |

The per-style `aggression` and `avoidance` values, and the per-style offsets from the Balanced tuple
(left fixed by spec 083), join the search.

### 3. Style objectives
Each style is tuned against the T4 Balanced reference in 4-car pack races on the spec 083 training
circuits.
- **Smooth:** low derived tyre scrub energy and high exit speed.
- **Aggressive:** more completed passes on corner entry. At-fault contacts are penalized.
- **Tenacious:** more time defending and fewer passes conceded under pressure.
- **Calculating:** more time in a wake before a pass, and a high pass success rate.
- **Bold:** body slip in the 10–25 deg band in corners. Spins are penalized.
- **Balanced:** low lap-time spread. This is the reference.

Every style must keep its tier pace band from spec 083 within +-1 %. Racecraft must not change the tier
ladder.

### 4. Worker telemetry
`bot_eval_worker` results gain: `scrub_energy_j`, `mean_corner_slip_deg`, `time_in_wake_s`,
`time_defending_s`, `passes_made`, `passes_conceded` and `at_fault_contacts`.

---

## 🔑 Security, Compliance, & IAM Roles
- No secrets, keys or new network services. blaze is reached over the existing Tailscale SSH setup.
- The tuning tools run offline. Nothing in the shipped game reads their output at runtime.

---

## 🗄️ Database & Storage Migration Plan
- No schema or save-data change. New per-style settings are Rust constants in `race-kit`.
- Campaign ledgers stay on blaze, as in spec 083.

---

## 🛡️ Disaster Recovery, Monitoring, & Fallbacks
- No bot-only rescue (same rule as spec 083).
- When a bot's tactical state is `Nominal` for the whole race, its controls are identical to the
  pre-107 bot with the same seed (same `controls_hash`).

---

## 🧪 Verification & Acceptance Criteria

### Automated tests
```bash
cargo test -p race-kit ai::tactics
cargo test -p tdrace-app --release --test bot_style_racecraft_tests
cargo test -p tdrace-app --release --test bot_tier_ladder_tests
```

### Manual Acceptance Criteria (Pseudo-Gherkin)

#### Scenario: Tactics off gives the old bot
- [ ] **Given** a race where tactics are disabled for every bot
- [ ] **When** it runs with the same seed as before spec 107
- [ ] **Then** every bot's `controls_hash` matches the pre-107 value.

#### Scenario: Tenacious defends more than Balanced
- [ ] **Given** 4-car pack races at T4 on the spec 083 training circuits, 8 seeds each
- [ ] **When** a Tenacious bot leads with a faster car close behind
- [ ] **Then** its `time_defending_s` is at least 30 % higher than a Balanced bot in the same seat
- [ ] **And** it concedes fewer passes than the Balanced bot.

#### Scenario: Calculating uses the wake before passing
- [ ] **Given** the same pack races
- [ ] **When** a Calculating bot follows a slower car on a straight
- [ ] **Then** its `time_in_wake_s` before each pass is at least 40 % higher than Balanced
- [ ] **And** its pass success rate is not lower than Balanced.

#### Scenario: Smooth and Bold have opposite tyre use
- [ ] **Given** the same pack races
- [ ] **When** scrub energy and corner body slip are compared
- [ ] **Then** Smooth has at least 25 % lower `scrub_energy_j` than Bold
- [ ] **And** Bold's `mean_corner_slip_deg` is between 10 and 25 deg on circuits where its car can slide.

#### Scenario: Racecraft keeps the tier ladder
- [ ] **Given** the tuned style settings
- [ ] **When** `bot_tier_ladder_tests` runs
- [ ] **Then** it passes, and each style's tier deltas stay within +-1 % of the spec 083 bands.

#### Scenario: The tactical tick allocates nothing
- [ ] **Given** 8 bots with tactics on
- [ ] **When** 36,000 ticks run under a counting allocator in a test
- [ ] **Then** the tactical evaluation makes zero heap allocations.

---

## 🔗 Traceability & Codebase Mapping

### Created/Modified Files
- `[ ]` `crates/race-kit/src/ai/tactics.rs` -> Tactical state machine.
- `[ ]` `crates/race-kit/src/ai/mod.rs` -> Hook tactics into `BotAiDriver`; per-style settings.
- `[ ]` `crates/tdrace-app/src/ai/bot_harness.rs` -> New telemetry fields.
- `[ ]` `crates/tdrace-app/src/bin/bot_eval_worker.rs` -> New result fields.
- `[ ]` `crates/tdrace-app/tests/bot_style_racecraft_tests.rs` -> Style racecraft checks.
- `[ ]` `tools/bot_tuning/styles.py` -> Style objectives and campaigns.
- `[ ]` `reports/bot_style_racecraft_report.md` -> Before/after style telemetry.

### Verification Assertions
- `tactics.rs` references `specs/107_tactical_bot_racecraft_and_stylespecific_behaviour_tuning.md` in its header comment.
