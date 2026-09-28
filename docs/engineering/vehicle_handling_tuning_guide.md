---
type: Guide
title: "Vehicle Handling Tuning Guide"
description: "Designer and player handling knobs of the spec 043 vehicle model, what each one changes, and the gates that check them."
status: active
category: engineering
tags: [physics, tuning, tires, controls, presets]
---

# Vehicle Handling Tuning Guide

This guide covers the handling model from
[Spec 043](../../specs/043_vehicle_dynamics_rebuild_and_simplified_handling_settings.md). Each knob
is expressed in a unit that a driver can feel. After you tune, run the gates (see §4).

## 1. Car knobs (`wheelbase::CarConfig`)

| Knob | Typical range | Raise it to get | Notes |
|---|---|---|---|
| `tire.grip` | 0.6–1.6 | more cornering g, braking and traction | True friction scale. The catalog grip stat multiplies it. |
| `tire.peak_slip_angle_deg` | 6–15 | lazier turn-in (lower = sharper) | Also sets the cornering stiffness. |
| `tire.peak_slip_ratio` | 0.08–0.15 | more wheelspin before grip peaks | Also the TCS / ABS slip target at default assists. |
| `tire.slide_grip` | 0.6–1.0 | a more forgiving slide | Grip deep in a slide, relative to peak. |
| `tire.falloff` | 1–3 | a gentler loss of grip past the limit | Width of the fall after the peak. |
| `tire.load_sensitivity` | 0–0.25 | balance knobs that bite harder | Heavily loaded tires lose grip per newton. |
| `tire.power_slide` | 0.3–1 | more slide from wheelspin (1 = physical) | Lower values keep lateral grip under wheelspin (arcade). |
| `rear_axle.grip_scale` | 0.9–1.2 | more rear grip (stable) | Rear tire relative to the front tire. |
| `rear_axle.peak_slip_scale` | 0.75–1.0 | a looser rear (1.0 = neutral) | Default 0.85 is stable. The drift car uses 1.0. |
| `roll_balance` | 0.35–0.65 | understeer at the limit | Front share of lateral load transfer. |
| `weight_transfer_hz` | 2–10 | quicker, sharper weight shifts | Default 3 Hz. |
| `engine_brake_front_share` | 0.2–0.6 | less lift-off oversteer | Default `0.35 + 0.30 · drive_bias`. |
| `brake_bias` | 0.5–0.75 | a more stable stop (less rotation) | Minimum front share. EBD can only move it forward. |
| `rear_differential` | Open / LSD / Spool | Spool: understeer on power. LSD `power_lock`: exit balance | LSD lock acts on the real speed difference. |
| `caster_jacking_factor` | 0 (cars), ~1.25 (karts) | earlier inside-rear lift | Frees a spool in tight turns. |

**Balance recipe.** For stability in normal cornering, lower `rear_axle.peak_slip_scale`. For more
understeer at the limit, raise `roll_balance`. For more rotation on power, raise `power_lock`.
For more rotation on lift, lower `engine_brake_front_share`. For more rotation on the brakes,
lower `brake_bias`.

## 2. Player settings (keyboard)

| Setting | Range | What it does |
|---|---|---|
| Steering Speed | 30–300 ms | Time from center to full input. Centering takes 0.7× this time. |
| Steering Authority | 80–140 % | Where full input sits relative to the front grip limit. At 160 km/h, extra authority past the limit shrinks to 30 %. |
| Center Precision | 1.0–1.8 | Response curve. Higher = finer control near center. |
| Pedal Speed | 0–300 ms | Throttle and brake time to full. 0 = instant. |
| Traction Help | 0–100 % | Eases the throttle as the rear tires near their limit (85–100 % use) while you corner. It never eases straight-line drive. |

| Preset | Speed | Authority | Precision | Pedal | Traction Help |
|---|---|---|---|---|---|
| Smooth | 220 ms | 90 % | 1.5 | 220 ms | 90 % |
| Balanced | 140 ms | 100 % | 1.3 | 140 ms | 70 % |
| Sharp | 90 ms | 107 % | 1.1 | 80 ms | 35 % |
| Raw | 40 ms | 115 % | 1.0 | 0 ms | 0 % |

Steering Authority and Traction Help are car-side aids (`PlayerHandling`).
`RaceSession::apply_player_handling` applies them to the human cars. Bots keep linear steering,
capped at the grip limit with overslip 1.5.

## 3. Driver assists

Assists come from the assist profile (Arcade / Sport / Pro). TCS trims torque at the axle to the
tire's slip target. ABS holds the braking slip at its target, and the target shrinks while you
corner. The rear axle uses select-low and 0.6× the peak slip. ESC caps both the yaw error and the
body sideslip. Module defaults are: rally = Sport, all other modules (GT included) = Arcade.

## 4. Gates to run after tuning

```bash
cargo test -p wheelbase --test handling_calibration_tests
```

```bash
cargo test -p tdrace-app --test handling_presets_tests
```

```bash
cargo test -p tdrace-app --test keyboard_simulation_tests
```

```bash
cargo run -p tdrace-app --release --bin keyboard_simulation_benchmark
```

The first three commands must pass. The last one writes
`reports/keyboard_input_car_control_report.md`, which shows how each key style (hold, tap,
cadence, tap-and-coast, lift-off) performs on each preset and car.
