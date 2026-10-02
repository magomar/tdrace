# Assist Mode × Key Style × Preset × Car Report (Spec 072) ⌨️🏎️

Generated `2026-10-01T23:08:26.197719427+00:00` by `cargo run -p tdrace-app --bin keyboard_simulation_benchmark` in 14.62 s.
Physics: spec 043 slip-based tires, 120 Hz. Cars: 10 vehicles (Classic plus FWD/AWD anchors). Modes: Arcade/Sport/Pro. Presets: Smooth, Balanced, Sharp, Raw.
Surfaces: asphalt, dirt, packed sand, sheet ice. Intervention multipliers are averaged per fixed step.

## 1. Mode × input response matrix

`Help pref` is mode-resolved configured traction help. `Help cut`, lateral/longitudinal TCS cuts and ESC yaw torque are measured per fixed step, separately from physical tire-force limits.

| Vehicle | Mode | Input | Help pref | Help cut | Lat TCS cut | Long TCS cut | ESC Nm | Retention | Rear slip | Reversal ms | Catch R/D/S |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Apex Phantom GT | ARCADE | Smooth | 90.0% | 41.5% | 0.0% | 21.1% | 144.6 | 120.8% | 5.7° | 897 | 0 / 1 / 5 |
| Apex Phantom GT | ARCADE | Balanced | 70.0% | 37.1% | 1.0% | 26.9% | 319.5 | 117.5% | 9.0° | 399 | 0 / 0 / 6 |
| Apex Phantom GT | ARCADE | Sharp | 35.0% | 19.4% | 2.9% | 36.8% | 660.4 | 114.3% | 12.3° | 260 | 0 / 1 / 5 |
| Apex Phantom GT | ARCADE | Raw | 0.0% | 0.0% | 3.4% | 44.7% | 862.5 | 114.4% | 12.8° | 148 | 0 / 1 / 5 |
| Apex Phantom GT | SPORT | Smooth | 0.0% | 0.0% | 2.6% | 15.9% | 1191.7 | 100.9% | 34.7° | 1610 | 0 / 0 / 6 |
| Apex Phantom GT | SPORT | Balanced | 0.0% | 0.0% | 2.8% | 16.7% | 1238.4 | 94.4% | 39.1° | 1231 | 0 / 0 / 6 |
| Apex Phantom GT | SPORT | Sharp | 0.0% | 0.0% | 2.9% | 17.3% | 1179.6 | 89.9% | 41.3° | 1067 | 0 / 0 / 6 |
| Apex Phantom GT | SPORT | Raw | 0.0% | 0.0% | 2.9% | 17.3% | 1251.0 | 91.3% | 40.7° | 626 | 0 / 0 / 6 |
| Apex Phantom GT | PRO | Smooth | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 74.7% | 73.1° | 1972 | 0 / 0 / 6 |
| Apex Phantom GT | PRO | Balanced | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 70.9% | 75.8° | 1951 | 0 / 0 / 6 |
| Apex Phantom GT | PRO | Sharp | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 68.8% | 77.4° | 1926 | 0 / 0 / 6 |
| Apex Phantom GT | PRO | Raw | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 68.0% | 78.2° | 1832 | 0 / 0 / 6 |
| Thunderbolt Stock V8 | ARCADE | Smooth | 90.0% | 48.5% | 0.0% | 22.4% | 179.9 | 127.0% | 5.1° | 843 | 0 / 2 / 4 |
| Thunderbolt Stock V8 | ARCADE | Balanced | 70.0% | 42.0% | 1.1% | 30.0% | 390.2 | 123.2% | 9.4° | 370 | 1 / 0 / 5 |
| Thunderbolt Stock V8 | ARCADE | Sharp | 35.0% | 21.0% | 2.0% | 42.5% | 908.5 | 119.7% | 12.0° | 228 | 0 / 1 / 5 |
| Thunderbolt Stock V8 | ARCADE | Raw | 0.0% | 0.0% | 2.2% | 50.4% | 994.5 | 119.4% | 12.0° | 129 | 0 / 1 / 5 |
| Thunderbolt Stock V8 | SPORT | Smooth | 0.0% | 0.0% | 3.4% | 18.1% | 1790.1 | 98.3% | 39.1° | 2000 | 0 / 0 / 6 |
| Thunderbolt Stock V8 | SPORT | Balanced | 0.0% | 0.0% | 3.8% | 18.8% | 1692.1 | 91.5% | 42.7° | 1774 | 0 / 0 / 6 |
| Thunderbolt Stock V8 | SPORT | Sharp | 0.0% | 0.0% | 4.1% | 19.3% | 1560.0 | 86.2% | 45.2° | 1651 | 0 / 0 / 6 |
| Thunderbolt Stock V8 | SPORT | Raw | 0.0% | 0.0% | 4.0% | 19.4% | 1644.3 | 87.8% | 44.5° | 1186 | 0 / 0 / 6 |
| Thunderbolt Stock V8 | PRO | Smooth | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 71.6% | 76.1° | 1830 | 0 / 0 / 6 |
| Thunderbolt Stock V8 | PRO | Balanced | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 66.6% | 78.7° | 1714 | 0 / 0 / 6 |
| Thunderbolt Stock V8 | PRO | Sharp | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 64.8% | 80.0° | 1670 | 0 / 0 / 6 |
| Thunderbolt Stock V8 | PRO | Raw | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 63.9% | 80.5° | 1649 | 0 / 0 / 6 |
| Vortex Dune Crusher | ARCADE | Smooth | 90.0% | 37.4% | 0.0% | 10.2% | 47.4 | 139.3% | 5.5° | 737 | 1 / 2 / 3 |
| Vortex Dune Crusher | ARCADE | Balanced | 70.0% | 39.4% | 0.1% | 15.0% | 105.8 | 137.6% | 6.6° | 289 | 0 / 1 / 5 |
| Vortex Dune Crusher | ARCADE | Sharp | 35.0% | 20.2% | 1.4% | 24.3% | 270.0 | 138.2% | 10.6° | 201 | 0 / 1 / 5 |
| Vortex Dune Crusher | ARCADE | Raw | 0.0% | 0.0% | 1.3% | 31.4% | 395.3 | 141.2% | 11.7° | 118 | 0 / 0 / 6 |
| Vortex Dune Crusher | SPORT | Smooth | 0.0% | 0.0% | 2.2% | 10.8% | 417.0 | 126.5% | 32.9° | 1210 | 0 / 0 / 6 |
| Vortex Dune Crusher | SPORT | Balanced | 0.0% | 0.0% | 2.5% | 11.4% | 464.8 | 121.1% | 36.1° | 923 | 0 / 1 / 5 |
| Vortex Dune Crusher | SPORT | Sharp | 0.0% | 0.0% | 2.8% | 11.8% | 460.7 | 116.8% | 37.8° | 867 | 0 / 0 / 6 |
| Vortex Dune Crusher | SPORT | Raw | 0.0% | 0.0% | 2.8% | 11.9% | 490.2 | 117.6% | 37.4° | 534 | 0 / 0 / 6 |
| Vortex Dune Crusher | PRO | Smooth | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 97.4% | 69.4° | 1502 | 0 / 1 / 5 |
| Vortex Dune Crusher | PRO | Balanced | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 91.3% | 71.3° | 1477 | 0 / 0 / 6 |
| Vortex Dune Crusher | PRO | Sharp | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 87.6% | 71.8° | 1536 | 0 / 0 / 6 |
| Vortex Dune Crusher | PRO | Raw | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 85.7% | 71.8° | 1360 | 0 / 0 / 6 |
| Turbo Dart 200cc | ARCADE | Smooth | 90.0% | 15.8% | 0.0% | 12.9% | 2.7 | 117.0% | 2.9° | 487 | 0 / 2 / 4 |
| Turbo Dart 200cc | ARCADE | Balanced | 70.0% | 19.1% | 0.0% | 14.5% | 12.0 | 115.3% | 3.5° | 391 | 0 / 2 / 4 |
| Turbo Dart 200cc | ARCADE | Sharp | 35.0% | 12.1% | 0.0% | 15.5% | 25.8 | 117.6% | 3.9° | 235 | 0 / 0 / 6 |
| Turbo Dart 200cc | ARCADE | Raw | 0.0% | 0.0% | 0.2% | 18.8% | 39.4 | 117.7% | 5.8° | 141 | 0 / 1 / 5 |
| Turbo Dart 200cc | SPORT | Smooth | 0.0% | 0.0% | 0.1% | 5.7% | 0.3 | 119.5% | 10.4° | 576 | 0 / 1 / 5 |
| Turbo Dart 200cc | SPORT | Balanced | 0.0% | 0.0% | 0.3% | 6.0% | 1.3 | 117.1% | 12.5° | 540 | 0 / 2 / 4 |
| Turbo Dart 200cc | SPORT | Sharp | 0.0% | 0.0% | 0.5% | 6.2% | 2.6 | 115.6% | 13.8° | 516 | 0 / 0 / 6 |
| Turbo Dart 200cc | SPORT | Raw | 0.0% | 0.0% | 0.6% | 6.3% | 4.7 | 114.5% | 14.5° | 365 | 0 / 0 / 6 |
| Turbo Dart 200cc | PRO | Smooth | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 120.5% | 15.0° | 596 | 0 / 1 / 5 |
| Turbo Dart 200cc | PRO | Balanced | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 118.8% | 17.7° | 560 | 0 / 2 / 4 |
| Turbo Dart 200cc | PRO | Sharp | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 117.3% | 19.0° | 537 | 0 / 0 / 6 |
| Turbo Dart 200cc | PRO | Raw | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 115.8% | 19.7° | 485 | 0 / 0 / 6 |
| Trailfire Turbo 4WD | ARCADE | Smooth | 90.0% | 26.0% | 0.0% | 5.0% | 3.0 | 153.9% | 2.6° | 576 | 0 / 3 / 3 |
| Trailfire Turbo 4WD | ARCADE | Balanced | 70.0% | 26.5% | 0.0% | 5.7% | 20.9 | 154.0% | 4.5° | 230 | 2 / 1 / 3 |
| Trailfire Turbo 4WD | ARCADE | Sharp | 35.0% | 14.1% | 0.1% | 14.3% | 63.4 | 158.1% | 5.9° | 183 | 0 / 1 / 5 |
| Trailfire Turbo 4WD | ARCADE | Raw | 0.0% | 0.0% | 0.3% | 21.1% | 137.0 | 158.0% | 7.5° | 112 | 0 / 0 / 6 |
| Trailfire Turbo 4WD | SPORT | Smooth | 0.0% | 0.0% | 0.0% | 5.4% | 0.0 | 167.8% | 4.8° | 601 | 0 / 3 / 3 |
| Trailfire Turbo 4WD | SPORT | Balanced | 0.0% | 0.0% | 0.0% | 5.6% | 0.9 | 166.9% | 6.8° | 387 | 2 / 0 / 4 |
| Trailfire Turbo 4WD | SPORT | Sharp | 0.0% | 0.0% | 0.1% | 5.8% | 1.8 | 165.9% | 8.4° | 245 | 0 / 0 / 6 |
| Trailfire Turbo 4WD | SPORT | Raw | 0.0% | 0.0% | 0.1% | 6.1% | 4.2 | 164.4% | 9.9° | 169 | 0 / 0 / 6 |
| Trailfire Turbo 4WD | PRO | Smooth | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 169.4% | 4.4° | 658 | 0 / 0 / 6 |
| Trailfire Turbo 4WD | PRO | Balanced | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 169.1% | 6.8° | 392 | 0 / 1 / 5 |
| Trailfire Turbo 4WD | PRO | Sharp | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 168.7% | 8.7° | 281 | 0 / 1 / 5 |
| Trailfire Turbo 4WD | PRO | Raw | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 168.4% | 10.1° | 208 | 0 / 1 / 5 |
| Mudlark Cross Car | ARCADE | Smooth | 90.0% | 33.8% | 0.0% | 5.0% | 12.7 | 124.8% | 4.4° | 586 | 0 / 3 / 3 |
| Mudlark Cross Car | ARCADE | Balanced | 70.0% | 35.2% | 0.2% | 8.3% | 63.0 | 123.5% | 6.3° | 257 | 1 / 1 / 4 |
| Mudlark Cross Car | ARCADE | Sharp | 35.0% | 19.6% | 1.0% | 15.7% | 135.7 | 125.3% | 8.6° | 190 | 0 / 0 / 6 |
| Mudlark Cross Car | ARCADE | Raw | 0.0% | 0.0% | 1.4% | 20.9% | 217.8 | 127.8% | 10.6° | 112 | 0 / 0 / 6 |
| Mudlark Cross Car | SPORT | Smooth | 0.0% | 0.0% | 1.6% | 7.0% | 134.8 | 122.6% | 24.5° | 881 | 0 / 2 / 4 |
| Mudlark Cross Car | SPORT | Balanced | 0.0% | 0.0% | 1.8% | 7.4% | 159.6 | 119.0% | 27.0° | 704 | 1 / 0 / 5 |
| Mudlark Cross Car | SPORT | Sharp | 0.0% | 0.0% | 2.0% | 7.7% | 168.6 | 116.2% | 28.2° | 676 | 0 / 0 / 6 |
| Mudlark Cross Car | SPORT | Raw | 0.0% | 0.0% | 2.0% | 7.7% | 183.0 | 116.0% | 28.1° | 388 | 0 / 0 / 6 |
| Mudlark Cross Car | PRO | Smooth | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 112.8% | 49.4° | 1127 | 0 / 0 / 6 |
| Mudlark Cross Car | PRO | Balanced | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 107.4% | 51.7° | 1099 | 0 / 1 / 5 |
| Mudlark Cross Car | PRO | Sharp | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 103.9% | 53.2° | 1080 | 0 / 0 / 6 |
| Mudlark Cross Car | PRO | Raw | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 102.1% | 53.4° | 1035 | 0 / 0 / 6 |
| Brawler Touring AX | ARCADE | Smooth | 90.0% | 19.1% | 0.0% | 9.0% | 1.4 | 151.3% | 2.3° | 562 | 0 / 3 / 3 |
| Brawler Touring AX | ARCADE | Balanced | 70.0% | 17.0% | 0.0% | 11.8% | 6.6 | 154.0% | 2.6° | 349 | 2 / 1 / 3 |
| Brawler Touring AX | ARCADE | Sharp | 35.0% | 9.2% | 0.1% | 20.0% | 34.4 | 154.6% | 4.8° | 179 | 2 / 1 / 3 |
| Brawler Touring AX | ARCADE | Raw | 0.0% | 0.0% | 0.2% | 24.0% | 82.8 | 154.2% | 5.4° | 109 | 0 / 1 / 5 |
| Brawler Touring AX | SPORT | Smooth | 0.0% | 0.0% | 0.0% | 6.0% | 0.0 | 159.6% | 3.4° | 611 | 0 / 3 / 3 |
| Brawler Touring AX | SPORT | Balanced | 0.0% | 0.0% | 0.0% | 6.3% | 0.0 | 159.4% | 4.8° | 394 | 1 / 1 / 4 |
| Brawler Touring AX | SPORT | Sharp | 0.0% | 0.0% | 0.0% | 6.5% | 0.0 | 159.0% | 6.0° | 250 | 2 / 0 / 4 |
| Brawler Touring AX | SPORT | Raw | 0.0% | 0.0% | 0.0% | 6.8% | 0.0 | 158.8% | 6.9° | 175 | 0 / 0 / 6 |
| Brawler Touring AX | PRO | Smooth | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 158.6% | 2.2° | 693 | 1 / 2 / 3 |
| Brawler Touring AX | PRO | Balanced | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 158.7% | 3.0° | 382 | 0 / 4 / 2 |
| Brawler Touring AX | PRO | Sharp | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 158.6% | 3.5° | 214 | 2 / 2 / 2 |
| Brawler Touring AX | PRO | Raw | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 158.6% | 3.8° | 153 | 1 / 2 / 3 |
| Talon Super Buggy | ARCADE | Smooth | 90.0% | 20.8% | 0.0% | 30.2% | 0.8 | 178.4% | 2.2° | 594 | 0 / 3 / 3 |
| Talon Super Buggy | ARCADE | Balanced | 70.0% | 20.4% | 0.0% | 38.5% | 6.0 | 178.7% | 4.3° | 234 | 1 / 2 / 3 |
| Talon Super Buggy | ARCADE | Sharp | 35.0% | 11.7% | 0.1% | 44.2% | 18.9 | 178.7% | 5.4° | 186 | 2 / 1 / 3 |
| Talon Super Buggy | ARCADE | Raw | 0.0% | 0.0% | 0.2% | 48.6% | 57.6 | 177.6% | 6.9° | 117 | 2 / 1 / 3 |
| Talon Super Buggy | SPORT | Smooth | 0.0% | 0.0% | 0.0% | 14.1% | 0.0 | 185.6% | 3.7° | 932 | 0 / 3 / 3 |
| Talon Super Buggy | SPORT | Balanced | 0.0% | 0.0% | 0.0% | 14.5% | 0.0 | 185.7% | 5.4° | 408 | 0 / 3 / 3 |
| Talon Super Buggy | SPORT | Sharp | 0.0% | 0.0% | 0.0% | 14.8% | 0.0 | 185.6% | 6.7° | 262 | 2 / 0 / 4 |
| Talon Super Buggy | SPORT | Raw | 0.0% | 0.0% | 0.0% | 15.2% | 0.2 | 185.6% | 7.8° | 192 | 2 / 0 / 4 |
| Talon Super Buggy | PRO | Smooth | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 165.5% | 2.4° | 1040 | 1 / 2 / 3 |
| Talon Super Buggy | PRO | Balanced | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 165.2% | 3.6° | 447 | 2 / 2 / 2 |
| Talon Super Buggy | PRO | Sharp | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 164.9% | 4.5° | 280 | 2 / 3 / 1 |
| Talon Super Buggy | PRO | Raw | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 164.8% | 5.0° | 208 | 4 / 1 / 1 |
| Peugeot 208 Rally4 | ARCADE | Smooth | 90.0% | 11.3% | 0.0% | 12.0% | 6.6 | 118.8% | 2.4° | 687 | 2 / 1 / 3 |
| Peugeot 208 Rally4 | ARCADE | Balanced | 70.0% | 10.3% | 0.0% | 13.7% | 40.2 | 118.4% | 2.8° | 356 | 0 / 1 / 5 |
| Peugeot 208 Rally4 | ARCADE | Sharp | 35.0% | 6.4% | 0.0% | 16.2% | 111.8 | 117.4% | 3.3° | 185 | 0 / 1 / 5 |
| Peugeot 208 Rally4 | ARCADE | Raw | 0.0% | 0.0% | 0.0% | 19.1% | 186.2 | 116.7% | 3.6° | 116 | 0 / 1 / 5 |
| Peugeot 208 Rally4 | SPORT | Smooth | 0.0% | 0.0% | 0.0% | 5.3% | 0.0 | 123.3% | 2.1° | 824 | 0 / 3 / 3 |
| Peugeot 208 Rally4 | SPORT | Balanced | 0.0% | 0.0% | 0.0% | 5.4% | 0.0 | 122.4% | 2.5° | 499 | 0 / 1 / 5 |
| Peugeot 208 Rally4 | SPORT | Sharp | 0.0% | 0.0% | 0.0% | 5.5% | 0.1 | 121.5% | 2.9° | 477 | 0 / 0 / 6 |
| Peugeot 208 Rally4 | SPORT | Raw | 0.0% | 0.0% | 0.0% | 5.6% | 0.6 | 120.5% | 3.2° | 272 | 0 / 0 / 6 |
| Peugeot 208 Rally4 | PRO | Smooth | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 124.9% | 1.8° | 819 | 0 / 1 / 5 |
| Peugeot 208 Rally4 | PRO | Balanced | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 124.4% | 2.4° | 644 | 0 / 3 / 3 |
| Peugeot 208 Rally4 | PRO | Sharp | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 123.7% | 2.9° | 627 | 0 / 0 / 6 |
| Peugeot 208 Rally4 | PRO | Raw | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 123.2% | 3.1° | 422 | 0 / 0 / 6 |
| Olsbergs MSE Supercar Lites | ARCADE | Smooth | 90.0% | 23.5% | 0.0% | 3.2% | 2.6 | 133.3% | 2.7° | 424 | 0 / 3 / 3 |
| Olsbergs MSE Supercar Lites | ARCADE | Balanced | 70.0% | 23.9% | 0.0% | 2.6% | 33.4 | 133.0% | 4.4° | 233 | 2 / 1 / 3 |
| Olsbergs MSE Supercar Lites | ARCADE | Sharp | 35.0% | 14.1% | 0.1% | 8.9% | 112.8 | 135.4% | 5.3° | 183 | 0 / 1 / 5 |
| Olsbergs MSE Supercar Lites | ARCADE | Raw | 0.0% | 0.0% | 0.1% | 13.6% | 196.5 | 138.6% | 6.2° | 110 | 0 / 0 / 6 |
| Olsbergs MSE Supercar Lites | SPORT | Smooth | 0.0% | 0.0% | 0.0% | 3.5% | 0.0 | 144.6% | 4.0° | 466 | 0 / 3 / 3 |
| Olsbergs MSE Supercar Lites | SPORT | Balanced | 0.0% | 0.0% | 0.0% | 3.6% | 0.0 | 143.6% | 5.6° | 301 | 2 / 0 / 4 |
| Olsbergs MSE Supercar Lites | SPORT | Sharp | 0.0% | 0.0% | 0.0% | 3.7% | 0.1 | 142.6% | 6.9° | 246 | 0 / 0 / 6 |
| Olsbergs MSE Supercar Lites | SPORT | Raw | 0.0% | 0.0% | 0.0% | 3.8% | 0.9 | 141.8% | 8.1° | 168 | 0 / 0 / 6 |
| Olsbergs MSE Supercar Lites | PRO | Smooth | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 145.3% | 4.2° | 491 | 0 / 0 / 6 |
| Olsbergs MSE Supercar Lites | PRO | Balanced | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 144.5% | 6.1° | 422 | 0 / 1 / 5 |
| Olsbergs MSE Supercar Lites | PRO | Sharp | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 143.7% | 7.7° | 291 | 0 / 1 / 5 |
| Olsbergs MSE Supercar Lites | PRO | Raw | 0.0% | 0.0% | 0.0% | 0.0% | 0.0 | 143.0% | 8.8° | 216 | 0 / 1 / 5 |

## 2. What was driven

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
| Apex Phantom GT | 80.1 (CLEAN CARVE) | 119.4 (CLEAN CARVE) | 107.8 (CLEAN CARVE) | 115.3 (CLEAN CARVE) | 58.5 (CLEAN CARVE) | 67% |
| Thunderbolt Stock V8 | 88.1 (CLEAN CARVE) | 129.0 (CLEAN CARVE) | 115.0 (CLEAN CARVE) | 124.1 (CLEAN CARVE) | 71.0 (CLEAN CARVE) | 68% |
| Vortex Dune Crusher | 97.0 (CLEAN CARVE) | 146.3 (CLEAN CARVE) | 131.9 (CLEAN CARVE) | 140.4 (CLEAN CARVE) | 60.9 (CLEAN CARVE) | 66% |
| Turbo Dart 200cc | 64.4 (CLEAN CARVE) | 83.1 (CLEAN CARVE) | 81.6 (CLEAN CARVE) | 82.4 (CLEAN CARVE) | 45.1 (CLEAN CARVE) | 78% |
| Trailfire Turbo 4WD | 117.6 (CLEAN CARVE) | 139.7 (CLEAN CARVE) | 138.0 (CLEAN CARVE) | 138.7 (CLEAN CARVE) | 59.5 (CLEAN CARVE) | 84% |
| Mudlark Cross Car | 84.4 (CLEAN CARVE) | 118.2 (CLEAN CARVE) | 113.2 (CLEAN CARVE) | 116.4 (CLEAN CARVE) | 52.8 (CLEAN CARVE) | 71% |
| Brawler Touring AX | 124.5 (CLEAN CARVE) | 129.6 (CLEAN CARVE) | 128.2 (CLEAN CARVE) | 129.1 (CLEAN CARVE) | 75.5 (CLEAN CARVE) | 96% |
| Talon Super Buggy | 153.5 (CLEAN CARVE) | 161.9 (CLEAN CARVE) | 160.0 (CLEAN CARVE) | 161.1 (CLEAN CARVE) | 107.3 (CLEAN CARVE) | 95% |
| Peugeot 208 Rally4 | 90.0 (CLEAN CARVE) | 99.5 (CLEAN CARVE) | 97.4 (CLEAN CARVE) | 98.8 (CLEAN CARVE) | 52.6 (CLEAN CARVE) | 90% |
| Olsbergs MSE Supercar Lites | 98.6 (CLEAN CARVE) | 114.8 (CLEAN CARVE) | 112.8 (CLEAN CARVE) | 114.1 (CLEAN CARVE) | 56.3 (CLEAN CARVE) | 86% |

## 3. Key Style Sensitivity (asphalt sweeper)

Spread = how much the exit speed changes between the best and the worst key style for the same car and preset. A small spread means *how* you press matters little; a large spread means technique matters.

| Car | Smooth | Balanced | Sharp | Raw |
|---|---|---|---|---|
| Apex Phantom GT | 56% (best feathering, worst lift_off) | 51% (best feathering, worst lift_off) | 56% (best feathering, worst lift_off) | 65% (best feathering, worst lift_off) |
| Thunderbolt Stock V8 | 56% (best feathering, worst lift_off) | 45% (best feathering, worst lift_off) | 59% (best feathering, worst lift_off) | 60% (best feathering, worst lift_off) |
| Vortex Dune Crusher | 65% (best feathering, worst lift_off) | 58% (best feathering, worst lift_off) | 36% (best feathering, worst lift_off) | 42% (best feathering, worst lift_off) |
| Turbo Dart 200cc | 42% (best feathering, worst lift_off) | 46% (best feathering, worst lift_off) | 34% (best feathering, worst lift_off) | 21% (best feathering, worst lift_off) |
| Trailfire Turbo 4WD | 59% (best feathering, worst lift_off) | 57% (best feathering, worst lift_off) | 38% (best feathering, worst lift_off) | 31% (best feathering, worst lift_off) |
| Mudlark Cross Car | 57% (best feathering, worst lift_off) | 55% (best feathering, worst lift_off) | 38% (best feathering, worst lift_off) | 25% (best feathering, worst lift_off) |
| Brawler Touring AX | 58% (best feathering, worst lift_off) | 42% (best feathering, worst lift_off) | 29% (best feathering, worst lift_off) | 26% (best feathering, worst lift_off) |
| Talon Super Buggy | 42% (best feathering, worst lift_off) | 34% (best feathering, worst lift_off) | 29% (best feathering, worst lift_off) | 26% (best feathering, worst lift_off) |
| Peugeot 208 Rally4 | 47% (best feathering, worst lift_off) | 47% (best feathering, worst lift_off) | 46% (best feathering, worst lift_off) | 46% (best feathering, worst lift_off) |
| Olsbergs MSE Supercar Lites | 52% (best feathering, worst lift_off) | 51% (best feathering, worst lift_off) | 48% (best feathering, worst lift_off) | 31% (best feathering, worst lift_off) |

Average spread over all cars, per surface:

| Surface | Smooth | Balanced | Sharp | Raw |
|---|---|---|---|---|
| Asphalt | 53% | 49% | 41% | 37% |
| Dirt | 43% | 42% | 39% | 39% |
| PackedSand | 42% | 43% | 44% | 48% |
| SheetIce | 18% | 15% | 15% | 15% |

## 4. Chicane reversal outcomes (all cars, surfaces and styles)

| Preset | Crisp | Mild pendulum | Snap oversteer | Spinout | Mean reversal latency |
|---|---|---|---|---|---|
| Smooth | 76 | 44 | 0 | 0 | 639 ms |
| Balanced | 102 | 18 | 0 | 0 | 311 ms |
| Sharp | 113 | 7 | 0 | 0 | 203 ms |
| Raw | 117 | 3 | 0 | 0 | 121 ms |

## 5. Slide catch outcomes (dirt, sand, ice)

| Preset | Recovered | Delayed | Spun out |
|---|---|---|---|
| Smooth | 3 | 23 | 34 |
| Balanced | 9 | 10 | 41 |
| Sharp | 4 | 8 | 48 |
| Raw | 2 | 6 | 52 |

## 6. Old physics vs spec 043 (Balanced, asphalt sweeper)

Old numbers come from `reports/keyboard_input_car_control_report_pre043.json` (the pre-043 benchmark output).

| Car | Hold old → new | Feathering old → new | Hold vs feathering old → new | Lift-off old → new |
|---|---|---|---|---|
| Apex Phantom GT | 55.5 → 80.1 km/h | 110.4 → 119.4 km/h | 50% → 67% | 47.5 (ScrubUndersteer) → 58.5 (CLEAN CARVE) |
| Thunderbolt Stock V8 | 63.4 → 88.1 km/h | 111.6 → 129.0 km/h | 57% → 68% | 53.5 (CleanCarve) → 71.0 (CLEAN CARVE) |
| Vortex Dune Crusher | 70.5 → 97.0 km/h | 125.4 → 146.3 km/h | 56% → 66% | 53.0 (CleanCarve) → 60.9 (CLEAN CARVE) |
| Turbo Dart 200cc | 8.9 → 64.4 km/h | 83.3 → 83.1 km/h | 11% → 78% | 4.0 (Spinout) → 45.1 (CLEAN CARVE) |
| Trailfire Turbo 4WD | 58.9 → 117.6 km/h | 135.4 → 139.7 km/h | 43% → 84% | 50.3 (CleanCarve) → 59.5 (CLEAN CARVE) |
| Mudlark Cross Car | n/a | n/a | n/a | n/a |
| Brawler Touring AX | n/a | n/a | n/a | n/a |
| Talon Super Buggy | n/a | n/a | n/a | n/a |
| Peugeot 208 Rally4 | n/a | n/a | n/a | n/a |
| Olsbergs MSE Supercar Lites | n/a | n/a | n/a | n/a |

## 7. Spec 043 gates on this run

- PASS Holding keeps >= 90% of its entry speed and carves cleanly on every car (Balanced, asphalt).
- PASS Arcade/Balanced sustained-key retention is >= 90% for all vehicles (0 failures).
- PASS No chicane spinout on Smooth or Balanced on asphalt, dirt and packed sand (0 found).
- PASS Average asphalt key-style spread falls from Smooth to Raw (53% / 49% / 41% / 37%).

Reading the numbers: holding a key now carves the tightest line without scrubbing speed; tapping keeps a wider, faster line. Smooth filters taps into gentle steering, so on Smooth your technique changes the line the most; on Raw every tap is full input, so tapping and holding converge. For slides, tap the counter-steer: holding full opposite lock over-corrects into a slide the other way.
