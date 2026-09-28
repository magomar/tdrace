# Key Style × Preset × Car Report (Spec 042) ⌨️🏎️

Generated `2026-09-28T00:45:05.032963+00:00` by `cargo run -p tdrace-app --bin keyboard_simulation_benchmark` in 0.21 s.  
Physics: spec 042 slip-based tires, 120 Hz. Cars: 5 classic arcade cars. Presets: Smooth, Balanced, Sharp, Raw.  
Surfaces: asphalt, dirt, packed sand, sheet ice. Default arcade driver aids of each car.

## 1. What was driven

| Key style | Keys |
|---|---|
| Sustained Hold (Full Lock) | steer key held, W held |
| Rapid Feathering (Staccato Taps) | steer 75 ms on / 75 ms off, W held |
| Cadence Pulse (Medium Taps) | steer 180 ms on / 120 ms off, W held |
| Tap-and-Coast (Flick Entry) | 250 ms tap, 200 ms coast, then 100/100 ms taps, W held |
| Lift-Off Turn-In (Weight Transfer) | steer held, W released for the first 600 ms |
| Snap Countersteer (Direction Reversal) | chicane: right 1 s, left 1 s, centre (scripted by the scenario) |

Scenarios: **sweeper** (3 s corner from 70 km/h, kart 55 km/h), **chicane** (full right then full left from 65 km/h, kart 50 km/h, styles: hold, feathering, cadence), **slide catch** (15° / 35°·s⁻¹ induced slide on dirt, sand and ice, styles: hold, feathering).

## 2. Balanced preset on asphalt: exit speed by key style (km/h)

| Car | Hold | Feathering | Cadence | Tap-and-coast | Lift-off | Hold vs feathering |
|---|---|---|---|---|---|---|
| Apex Phantom GT | 79.1 (CLEAN CARVE) | 109.5 (CLEAN CARVE) | 100.4 (CLEAN CARVE) | 106.6 (CLEAN CARVE) | 66.9 (CLEAN CARVE) | 72% |
| Thunderbolt Stock V8 | 87.4 (CLEAN CARVE) | 116.1 (CLEAN CARVE) | 104.4 (CLEAN CARVE) | 112.4 (CLEAN CARVE) | 73.8 (CLEAN CARVE) | 75% |
| Vortex Dune Crusher | 82.0 (CLEAN CARVE) | 130.2 (CLEAN CARVE) | 114.9 (CLEAN CARVE) | 124.8 (CLEAN CARVE) | 69.7 (CLEAN CARVE) | 63% |
| Turbo Dart 200cc | 53.6 (CLEAN CARVE) | 83.0 (CLEAN CARVE) | 79.1 (CLEAN CARVE) | 81.7 (CLEAN CARVE) | 43.3 (CLEAN CARVE) | 65% |
| Trailfire Turbo 4WD | 94.8 (CLEAN CARVE) | 139.7 (CLEAN CARVE) | 134.4 (CLEAN CARVE) | 137.4 (CLEAN CARVE) | 77.3 (CLEAN CARVE) | 68% |

## 3. Key Style Sensitivity (asphalt sweeper)

Spread = how much the exit speed changes between the best and the worst key style for the same car and preset. A small spread means *how* you press matters little; a large spread means technique matters.

| Car | Smooth | Balanced | Sharp | Raw |
|---|---|---|---|---|
| Apex Phantom GT | 47% (best feathering, worst lift_off) | 39% (best feathering, worst lift_off) | 41% (best feathering, worst lift_off) | 46% (best feathering, worst lift_off) |
| Thunderbolt Stock V8 | 46% (best feathering, worst lift_off) | 36% (best feathering, worst lift_off) | 50% (best feathering, worst lift_off) | 50% (best tap_coast, worst lift_off) |
| Vortex Dune Crusher | 52% (best feathering, worst lift_off) | 46% (best feathering, worst lift_off) | 29% (best feathering, worst lift_off) | 42% (best feathering, worst lift_off) |
| Turbo Dart 200cc | 55% (best feathering, worst lift_off) | 48% (best feathering, worst lift_off) | 34% (best feathering, worst lift_off) | 21% (best feathering, worst lift_off) |
| Trailfire Turbo 4WD | 48% (best feathering, worst lift_off) | 45% (best feathering, worst lift_off) | 35% (best feathering, worst lift_off) | 22% (best feathering, worst lift_off) |

Average spread over all cars, per surface:

| Surface | Smooth | Balanced | Sharp | Raw |
|---|---|---|---|---|
| Asphalt | 50% | 43% | 38% | 36% |
| Dirt | 43% | 41% | 35% | 34% |
| PackedSand | 40% | 39% | 33% | 40% |
| SheetIce | 11% | 11% | 10% | 9% |

## 4. Chicane reversal outcomes (all cars, surfaces and styles)

| Preset | Crisp | Mild pendulum | Snap oversteer | Spinout | Mean reversal latency |
|---|---|---|---|---|---|
| Smooth | 46 | 6 | 0 | 8 | 488 ms |
| Balanced | 47 | 8 | 0 | 5 | 388 ms |
| Sharp | 46 | 7 | 4 | 3 | 317 ms |
| Raw | 44 | 9 | 5 | 2 | 202 ms |

## 5. Slide catch outcomes (dirt, sand, ice)

| Preset | Recovered | Delayed | Spun out |
|---|---|---|---|
| Smooth | 0 | 11 | 19 |
| Balanced | 1 | 10 | 19 |
| Sharp | 1 | 2 | 27 |
| Raw | 1 | 1 | 28 |

## 6. Old physics vs spec 042 (Balanced, asphalt sweeper)

Old numbers come from `reports/keyboard_input_car_control_report_pre042.json` (the pre-042 benchmark output).

| Car | Hold old → new | Feathering old → new | Hold vs feathering old → new | Lift-off old → new |
|---|---|---|---|---|
| Apex Phantom GT | 55.5 → 79.1 km/h | 110.4 → 109.5 km/h | 50% → 72% | 47.5 (ScrubUndersteer) → 66.9 (CLEAN CARVE) |
| Thunderbolt Stock V8 | 63.4 → 87.4 km/h | 111.6 → 116.1 km/h | 57% → 75% | 53.5 (CleanCarve) → 73.8 (CLEAN CARVE) |
| Vortex Dune Crusher | 70.5 → 82.0 km/h | 125.4 → 130.2 km/h | 56% → 63% | 53.0 (CleanCarve) → 69.7 (CLEAN CARVE) |
| Turbo Dart 200cc | 8.9 → 53.6 km/h | 83.3 → 83.0 km/h | 11% → 65% | 4.0 (Spinout) → 43.3 (CLEAN CARVE) |
| Trailfire Turbo 4WD | 58.9 → 94.8 km/h | 135.4 → 139.7 km/h | 43% → 68% | 50.3 (CleanCarve) → 77.3 (CLEAN CARVE) |

## 7. Spec 042 gates on this run

- PASS Holding keeps >= 90% of its entry speed and carves cleanly on every car (Balanced, asphalt).
- PASS No chicane spinout on Smooth or Balanced on asphalt, dirt and packed sand (0 found).
- PASS Average asphalt key-style spread falls from Smooth to Raw (50% / 43% / 38% / 36%).

Reading the numbers: holding a key now carves the tightest line without scrubbing speed; tapping keeps a wider, faster line. Smooth filters taps into gentle steering, so on Smooth your technique changes the line the most; on Raw every tap is full input, so tapping and holding converge. For slides, tap the counter-steer: holding full opposite lock over-corrects into a slide the other way.
