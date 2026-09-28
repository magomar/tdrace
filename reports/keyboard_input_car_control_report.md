# Key Style × Preset × Car Report (Spec 043) ⌨️🏎️

Generated `2026-09-28T11:07:45.398187+00:00` by `cargo run -p tdrace-app --bin keyboard_simulation_benchmark` in 1.33 s.  
Physics: spec 043 slip-based tires, 120 Hz. Cars: 5 classic arcade cars. Presets: Smooth, Balanced, Sharp, Raw.  
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
| Apex Phantom GT | 79.8 (CLEAN CARVE) | 119.6 (CLEAN CARVE) | 108.1 (CLEAN CARVE) | 115.4 (CLEAN CARVE) | 67.0 (CLEAN CARVE) | 67% |
| Thunderbolt Stock V8 | 87.9 (CLEAN CARVE) | 127.1 (CLEAN CARVE) | 112.3 (CLEAN CARVE) | 121.2 (CLEAN CARVE) | 73.8 (CLEAN CARVE) | 69% |
| Vortex Dune Crusher | 97.0 (CLEAN CARVE) | 146.3 (CLEAN CARVE) | 129.2 (CLEAN CARVE) | 139.7 (CLEAN CARVE) | 77.7 (CLEAN CARVE) | 66% |
| Turbo Dart 200cc | 64.3 (CLEAN CARVE) | 83.0 (CLEAN CARVE) | 81.5 (CLEAN CARVE) | 82.3 (CLEAN CARVE) | 43.3 (CLEAN CARVE) | 77% |
| Trailfire Turbo 4WD | 117.5 (CLEAN CARVE) | 139.7 (CLEAN CARVE) | 138.0 (CLEAN CARVE) | 138.7 (CLEAN CARVE) | 90.1 (CLEAN CARVE) | 84% |

## 3. Key Style Sensitivity (asphalt sweeper)

Spread = how much the exit speed changes between the best and the worst key style for the same car and preset. A small spread means *how* you press matters little; a large spread means technique matters.

| Car | Smooth | Balanced | Sharp | Raw |
|---|---|---|---|---|
| Apex Phantom GT | 49% (best feathering, worst lift_off) | 44% (best feathering, worst lift_off) | 43% (best feathering, worst lift_off) | 46% (best feathering, worst lift_off) |
| Thunderbolt Stock V8 | 52% (best feathering, worst lift_off) | 42% (best feathering, worst lift_off) | 50% (best feathering, worst lift_off) | 50% (best tap_coast, worst lift_off) |
| Vortex Dune Crusher | 46% (best feathering, worst lift_off) | 47% (best feathering, worst lift_off) | 34% (best feathering, worst lift_off) | 42% (best feathering, worst lift_off) |
| Turbo Dart 200cc | 55% (best feathering, worst lift_off) | 48% (best feathering, worst lift_off) | 34% (best feathering, worst lift_off) | 21% (best feathering, worst lift_off) |
| Trailfire Turbo 4WD | 34% (best feathering, worst lift_off) | 36% (best feathering, worst lift_off) | 30% (best feathering, worst lift_off) | 22% (best feathering, worst lift_off) |

Average spread over all cars, per surface:

| Surface | Smooth | Balanced | Sharp | Raw |
|---|---|---|---|---|
| Asphalt | 47% | 43% | 38% | 36% |
| Dirt | 38% | 40% | 36% | 34% |
| PackedSand | 38% | 38% | 34% | 40% |
| SheetIce | 13% | 11% | 9% | 9% |

## 4. Chicane reversal outcomes (all cars, surfaces and styles)

| Preset | Crisp | Mild pendulum | Snap oversteer | Spinout | Mean reversal latency |
|---|---|---|---|---|---|
| Smooth | 35 | 21 | 0 | 4 | 786 ms |
| Balanced | 45 | 12 | 0 | 3 | 444 ms |
| Sharp | 46 | 7 | 4 | 3 | 318 ms |
| Raw | 44 | 9 | 5 | 2 | 202 ms |

## 5. Slide catch outcomes (dirt, sand, ice)

| Preset | Recovered | Delayed | Spun out |
|---|---|---|---|
| Smooth | 0 | 8 | 22 |
| Balanced | 1 | 7 | 22 |
| Sharp | 1 | 2 | 27 |
| Raw | 1 | 1 | 28 |

## 6. Old physics vs spec 043 (Balanced, asphalt sweeper)

Old numbers come from `reports/keyboard_input_car_control_report_pre043.json` (the pre-043 benchmark output).

| Car | Hold old → new | Feathering old → new | Hold vs feathering old → new | Lift-off old → new |
|---|---|---|---|---|
| Apex Phantom GT | 55.5 → 79.8 km/h | 110.4 → 119.6 km/h | 50% → 67% | 47.5 (ScrubUndersteer) → 67.0 (CLEAN CARVE) |
| Thunderbolt Stock V8 | 63.4 → 87.9 km/h | 111.6 → 127.1 km/h | 57% → 69% | 53.5 (CleanCarve) → 73.8 (CLEAN CARVE) |
| Vortex Dune Crusher | 70.5 → 97.0 km/h | 125.4 → 146.3 km/h | 56% → 66% | 53.0 (CleanCarve) → 77.7 (CLEAN CARVE) |
| Turbo Dart 200cc | 8.9 → 64.3 km/h | 83.3 → 83.0 km/h | 11% → 77% | 4.0 (Spinout) → 43.3 (CLEAN CARVE) |
| Trailfire Turbo 4WD | 58.9 → 117.5 km/h | 135.4 → 139.7 km/h | 43% → 84% | 50.3 (CleanCarve) → 90.1 (CLEAN CARVE) |

## 7. Spec 043 gates on this run

- PASS Holding keeps >= 90% of its entry speed and carves cleanly on every car (Balanced, asphalt).
- PASS No chicane spinout on Smooth or Balanced on asphalt, dirt and packed sand (0 found).
- PASS Average asphalt key-style spread falls from Smooth to Raw (47% / 43% / 38% / 36%).

Reading the numbers: holding a key now carves the tightest line without scrubbing speed; tapping keeps a wider, faster line. Smooth filters taps into gentle steering, so on Smooth your technique changes the line the most; on Raw every tap is full input, so tapping and holding converge. For slides, tap the counter-steer: holding full opposite lock over-corrects into a slide the other way.
