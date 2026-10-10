# Bot Calibration Report (spec 083)

Spec: [`specs/083_evorl_bot_driving_style_and_performance_optimization.md`](../specs/083_evorl_bot_driving_style_and_performance_optimization.md).
Raw data: [`tools/bot_tuning/runs/`](../tools/bot_tuning/runs/).

## 1. Throughput

Workload: 3-lap races, T4 Balanced against T4 Balanced, on the 13 training circuits, through
`bot_eval_worker` (release build). Command:

```bash
python tools/bot_tuning/run_campaign.py throughput --workers <N> --rounds <R> --out <file>
```

| Machine | Workers | Races | Races/s | Car-steps/s |
|---|---|---|---|---|
| Mac (MGomez, 16 CPUs) | 1 | 13 | 2.4 | 102,900 |
| blaze (Ryzen 9 7950X3D, 16 cores / 32 threads) | 1 | 13 | 2.1 | 94,700 |
| blaze | 28 | 416 | 29.6 | 1,297,100 |

- One race takes 0.11 s (`classic/kart_pine_grove`) to 0.84 s (`rally/montalegre_rx`) on one
  core. Rallycross races are the slowest.
- 28 workers give 14x one worker. blaze has 16 physical cores, so the second thread per core adds
  little.
- The spec's planning estimate (0.1–0.5 s per race, 60–280 races/s on 28 workers) was optimistic
  for the parallel case. The measured value is 30 races/s.

**Budget from the measurement.** One Stage A generation (population 14 x 13 circuits x 4 seeds x
2 seats = 1,456 races) takes about 50 s on blaze. 30 generations take about 25 min.

**EvoRL path.** `run_campaign.py evorl-smoke` ran the default genome against the baseline through
EvoRL's `Evaluator` and `MatchLedger` on 2 circuits, with 2 seeds and 2 seats each:
- 8 matches, 0 errors
- win rate 0.50; the bot raced a copy of itself, so this is the expected value
- mean score +0.0002

## 2. Baseline audit (pre-tuning constants)

Command: `python tools/bot_tuning/run_campaign.py audit --workers 28 --seeds 8` on blaze, tracks 0236e292. Raw: [`runs/audit_baseline.json`](../tools/bot_tuning/runs/audit_baseline.json). Tables: `python tools/bot_tuning/report.py audit <json>`.

75 circuits, 3000 races (8 seeds x 5 tiers per circuit), 6.5 min on BLAZE. Each race is a 3-lap grid of one bot per style at one tier. Bold deltas are outside the spec 083 band.

### Summary by module

| Module | Circuits | Tier order held (all 4 steps) | T1 in band | T2 in band | T3 in band | T5 in band | Circuits with a stuck bot |
|---|---|---|---|---|---|---|---|
| classic | 18 | 12 | 12 | 11 | 11 | 6 | 9 |
| kart | 20 | 6 | 9 | 6 | 8 | 5 | 4 |
| autocross | 17 | 9 | 9 | 10 | 8 | 12 | 3 |
| rally | 20 | 0 | 2 | 1 | 1 | 4 | 14 |

### Per circuit

| Circuit | Cars | T1 | T2 | T3 | T5 | Order T1>T2>T3>T4>T5 | DNF (all tiers) | Stuck bots | Max no-progress (s) | T4 spins/lap | T4 wall contacts/lap |
|---|---|---|---|---|---|---|---|---|---|---|---|
| autocross/arteixo_ax | 6 | +11.5% | **+3.8%** | **+1.0%** | -2.2% | ✓✓✗✓ | 0 | 0 | 9.0 | 0.028 | 0.24 |
| autocross/bazaigues_ax | 6 | **+7.1%** | **+3.5%** | **+0.6%** | -2.2% | ✓✓✗✓ | 0 | 0 | 8.7 | 0.049 | 0.58 |
| autocross/carballo_ax | 6 | **+8.9%** | **+2.6%** | **+0.8%** | -1.9% | ✓✓✗✓ | 0 | 0 | 8.3 | 0.035 | 0.17 |
| autocross/castelo_branco_ax | 6 | +11.8% | +5.6% | +2.0% | -2.3% | ✓✓✓✓ | 0 | 0 | 8.4 | 0.035 | 0.15 |
| autocross/faleyras_ax | 6 | **+9.0%** | **+3.9%** | **+0.9%** | **-0.4%** | ✓✓✗✗ | 0 | 1 | 10.2 | 0.021 | 0.70 |
| autocross/humpolec_ax | 6 | **+8.7%** | **+4.1%** | **+0.3%** | **-0.9%** | ✓✓✗✗ | 0 | 1 | 10.6 | 0.035 | 0.11 |
| autocross/maggiora_ax | 6 | +11.7% | +5.0% | +2.2% | -2.1% | ✓✓✓✓ | 0 | 0 | 10.0 | 0.118 | 0.34 |
| autocross/matschenberg_ax | 6 | +12.6% | +7.1% | +2.1% | -2.5% | ✓✓✓✓ | 0 | 0 | 8.3 | 0.035 | 0.33 |
| autocross/musa_ax | 6 | **+9.9%** | +5.1% | **+0.7%** | -2.1% | ✓✓✗✓ | 0 | 0 | 8.6 | 0.021 | 0.67 |
| autocross/nova_paka_ax | 6 | **+9.9%** | +5.4% | **+0.6%** | -1.5% | ✓✓✗✓ | 0 | 0 | 8.2 | 0.021 | 1.94 |
| autocross/prerov_ax | 6 | +11.3% | +5.5% | +1.5% | -1.8% | ✓✓✓✓ | 0 | 0 | 7.6 | 0.000 | 0.21 |
| autocross/schluechtern_ax | 6 | +12.6% | +6.3% | +1.9% | -2.0% | ✓✓✓✓ | 0 | 0 | 7.8 | 0.028 | 0.08 |
| autocross/seelow_ax | 6 | **+9.8%** | +5.2% | +1.6% | **-1.5%** | ✓✓✓✓ | 0 | 0 | 9.3 | 0.188 | 0.15 |
| autocross/st_georges_ax | 6 | +12.0% | +6.1% | +2.5% | -1.8% | ✓✓✓✓ | 0 | 0 | 7.2 | 0.014 | 0.15 |
| autocross/st_junien_ax | 6 | +10.3% | +5.9% | **+1.5%** | **-1.4%** | ✓✓✓✓ | 0 | 0 | 9.9 | 0.042 | 0.30 |
| autocross/uelzen_ax | 6 | +14.6% | **+9.3%** | +2.9% | -1.7% | ✓✓✓✓ | 0 | 0 | 9.0 | 0.000 | 0.26 |
| autocross/vilkyciai_ax | 6 | **+3.3%** | **+1.5%** | **+0.4%** | **-0.6%** | ✓✗✗✗ | 0 | 4 | 11.3 | 0.090 | 0.40 |
| classic/at_dune_sea | 6 | +13.5% | +7.9% | +2.7% | **-1.2%** | ✓✓✓✓ | 0 | 3 | 28.8 | 0.000 | 0.03 |
| classic/at_frostbite_pass | 6 | **+3.3%** | **+2.0%** | **+1.0%** | **-1.0%** | ✓✓✓✓ | 0 | 0 | 10.0 | 0.000 | 2.81 |
| classic/at_mudbath_valley | 6 | +11.8% | +6.5% | +2.2% | **-0.0%** | ✓✓✓✗ | 0 | 1 | 10.9 | 0.000 | 0.22 |
| classic/ax_clay_bowl | 6 | +10.5% | **+4.3%** | **+1.4%** | **-1.3%** | ✓✓✓✓ | 0 | 0 | 7.3 | 0.000 | 0.03 |
| classic/ax_hillside_hammer | 6 | **+6.5%** | **+2.5%** | **+0.9%** | **+0.1%** | ✓✓✓✗ | 0 | 0 | 7.4 | 0.014 | 0.82 |
| classic/ax_meadow_sprint | 6 | **+1.0%** | **-0.0%** | **-0.9%** | **+0.5%** | ✗✗✗✗ | 0 | 3 | 33.5 | 0.042 | 1.00 |
| classic/gt_coastal_grand_prix | 6 | +14.1% | +5.4% | +2.3% | -2.6% | ✓✓✓✓ | 0 | 6 | 19.1 | 0.000 | 0.03 |
| classic/gt_ridge_ring | 6 | +14.1% | +6.0% | +2.1% | -2.2% | ✓✓✓✓ | 0 | 4 | 15.1 | 0.000 | 0.02 |
| classic/gt_velocity_park | 6 | +14.6% | +7.0% | +2.6% | -2.5% | ✓✓✓✓ | 0 | 5 | 29.7 | 0.007 | 0.01 |
| classic/kart_pine_grove | 6 | +14.5% | +8.1% | +1.9% | **-0.7%** | ✓✓✓✗ | 0 | 0 | 7.3 | 0.028 | 1.57 |
| classic/kart_riverbend_circuit | 6 | +13.3% | +6.8% | +2.1% | -1.7% | ✓✓✓✓ | 0 | 0 | 6.1 | 0.049 | 1.47 |
| classic/kart_summit_international | 6 | +13.1% | +6.5% | +2.7% | **-1.0%** | ✓✓✓✓ | 0 | 0 | 6.6 | 0.056 | 2.12 |
| classic/rx_canyon_flyer | 6 | **+6.2%** | **+2.5%** | **+0.6%** | **+0.9%** | ✓✓✗✗ | 0 | 0 | 5.2 | 0.000 | 0.85 |
| classic/rx_hilltop_leap | 6 | **+9.4%** | **+4.9%** | **+1.3%** | **+0.0%** | ✓✓✓✗ | 0 | 6 | 12.7 | 0.000 | 0.37 |
| classic/rx_quarry_sprint | 6 | **+6.7%** | **+3.4%** | **+0.9%** | **-0.8%** | ✓✓✓✓ | 0 | 0 | 7.5 | 0.000 | 0.72 |
| classic/stock_roval | 6 | +13.4% | +6.5% | +1.8% | **-1.4%** | ✓✓✓✓ | 0 | 1 | 10.4 | 0.007 | 0.02 |
| classic/stock_thunder_bowl | 6 | +14.7% | +6.7% | +2.6% | -2.5% | ✓✓✓✓ | 0 | 0 | 9.9 | 0.000 | 0.01 |
| classic/stock_tri_oval_speedway | 6 | +15.2% | +7.7% | +2.4% | -2.6% | ✓✓✓✓ | 0 | 2 | 10.2 | 0.007 | 0.03 |
| kart/adria_kart | 6 | +11.0% | **+4.2%** | **+1.2%** | -1.6% | ✓✓✓✓ | 0 | 0 | 5.9 | 0.104 | 0.14 |
| kart/ampfing | 6 | **+7.6%** | **+3.8%** | **+0.3%** | **-0.6%** | ✓✓✗✗ | 0 | 1 | 11.1 | 0.840 | 5.97 |
| kart/aunay_kart | 6 | +11.9% | +5.4% | +1.9% | -1.9% | ✓✓✓✓ | 0 | 0 | 6.8 | 0.021 | 0.00 |
| kart/campillos | 6 | +10.1% | **+4.6%** | **+1.3%** | **-0.9%** | ✓✓✓✗ | 0 | 0 | 9.9 | 1.632 | 4.72 |
| kart/franciacorta | 6 | +11.4% | **+4.2%** | **+1.4%** | **-1.0%** | ✓✓✓✓ | 0 | 0 | 7.0 | 0.389 | 2.47 |
| kart/genk | 6 | **+5.9%** | **+3.4%** | **+0.8%** | **-0.4%** | ✓✓✗✗ | 0 | 0 | 9.0 | 0.882 | 3.24 |
| kart/kristianstad | 6 | **+5.8%** | **+0.6%** | **-0.4%** | **-0.5%** | ✓✗✗✗ | 0 | 0 | 8.2 | 1.500 | 3.82 |
| kart/laval_kart | 6 | +14.6% | +7.5% | +2.2% | **+1.6%** | ✓✓✓✗ | 0 | 0 | 6.8 | 0.250 | 5.26 |
| kart/le_mans_kart | 6 | **+9.5%** | **+4.9%** | **+1.1%** | **-0.7%** | ✓✓✓✗ | 0 | 0 | 7.1 | 1.257 | 3.02 |
| kart/lonato | 6 | **+8.3%** | **+2.1%** | **+0.9%** | **-1.3%** | ✓✗✗✓ | 0 | 0 | 9.1 | 0.743 | 2.92 |
| kart/muelsen_kart | 6 | +11.4% | +5.2% | +1.5% | -1.6% | ✓✓✓✓ | 0 | 0 | 5.7 | 0.056 | 0.08 |
| kart/pfi | 6 | **+7.5%** | **+4.3%** | **+0.0%** | **-0.4%** | ✓✓✗✗ | 0 | 4 | 12.6 | 1.493 | 7.06 |
| kart/portimao_kart | 6 | **+8.7%** | **+3.8%** | +1.5% | **-0.5%** | ✓✓✓✗ | 0 | 0 | 6.6 | 0.062 | 0.70 |
| kart/sarno | 6 | **+9.8%** | **+4.8%** | +1.6% | **-1.0%** | ✓✓✓✗ | 0 | 1 | 11.4 | 0.611 | 4.81 |
| kart/seven_laghi | 6 | **+4.7%** | **+0.9%** | **+0.4%** | **+0.7%** | ✓✗✗✗ | 0 | 0 | 9.7 | 1.562 | 7.78 |
| kart/silverstone_national_kart | 6 | +12.6% | +5.6% | +2.4% | -2.6% | ✓✓✓✓ | 0 | 0 | 5.5 | 0.097 | 0.48 |
| kart/valencia_kart | 6 | +13.2% | +5.5% | +2.7% | **-1.4%** | ✓✓✓✓ | 0 | 0 | 8.4 | 0.257 | 1.94 |
| kart/wackersdorf | 6 | **+5.4%** | **+2.8%** | **+0.5%** | -1.9% | ✓✓✗✓ | 0 | 3 | 12.8 | 1.181 | 7.78 |
| kart/whilton_mill | 6 | +10.7% | +5.0% | +1.8% | **-0.8%** | ✓✓✓✗ | 0 | 0 | 7.6 | 1.167 | 4.49 |
| kart/zuera | 6 | **+9.5%** | **+4.2%** | **+1.4%** | **-0.1%** | ✓✓✓✗ | 0 | 0 | 8.0 | 1.049 | 4.40 |
| rally/catalunya_rx | 6 | **+6.3%** | **+3.4%** | **+0.4%** | **+0.0%** | ✓✓✗✗ | 0 | 1 | 10.1 | 0.083 | 3.03 |
| rally/croft_rx | 6 | **-0.5%** | **-1.4%** | **-0.2%** | **+2.5%** | ✗✗✗✗ | 0 | 14 | 113.5 | 0.042 | 4.15 |
| rally/dreux_rx | 6 | **+3.3%** | **+0.3%** | **+0.9%** | **+0.1%** | ✓✗✗✗ | 0 | 1 | 11.3 | 0.139 | 4.04 |
| rally/erx_motor_park | 6 | **-18.1%** | **-13.5%** | **-4.2%** | **-0.1%** | ✗✗✗✗ | 2 | 4 | 430.7 | 0.132 | 9.65 |
| rally/essay_rx | 6 | **+8.9%** | **+4.0%** | **+1.4%** | -2.1% | ✓✓✗✓ | 0 | 9 | 34.6 | 0.236 | 3.33 |
| rally/estering_rx | 6 | **+8.9%** | **+3.8%** | **+1.1%** | **-3.5%** | ✓✓✗✓ | 0 | 1 | 24.0 | 0.014 | 1.78 |
| rally/hell_rx | 6 | **-1.4%** | **-5.2%** | **+7.3%** | -2.3% | ✓✗✗✓ | 0 | 5 | 275.0 | 0.160 | 3.55 |
| rally/holjes_rx | 6 | +11.5% | +6.0% | +2.1% | **+1.7%** | ✓✓✓✗ | 0 | 0 | 6.8 | 0.028 | 1.69 |
| rally/killarney_rx | 6 | **+7.8%** | **+3.7%** | **+1.4%** | **+0.1%** | ✓✓✓✗ | 0 | 0 | 8.4 | 0.021 | 2.61 |
| rally/kouvola_rx | 6 | **+5.4%** | **+2.6%** | **+5.6%** | **+1.3%** | ✓✗✗✗ | 73 | 81 | 551.9 | 0.104 | 17.78 |
| rally/lavare_rx | 6 | **+1.2%** | **-0.2%** | **+1.1%** | **+1.2%** | ✗✗✓✗ | 0 | 0 | 9.0 | 0.056 | 3.40 |
| rally/lessay_rx | 6 | +14.8% | **+4.3%** | **+3.8%** | **+1.0%** | ✗✗✓✗ | 0 | 8 | 308.2 | 0.104 | 1.76 |
| rally/loheac_rx | 6 | **-2.1%** | **-0.6%** | **+0.7%** | **-0.8%** | ✗✗✗✗ | 0 | 0 | 8.4 | 0.243 | 4.65 |
| rally/lydden_hill | 6 | **+0.7%** | **+0.4%** | **-0.5%** | **-1.2%** | ✗✗✗✗ | 24 | 24 | 546.4 | 0.097 | 3.97 |
| rally/mettet_rx | 6 | **+6.2%** | **+2.4%** | **+0.5%** | **-0.3%** | ✓✓✗✗ | 0 | 0 | 8.8 | 0.111 | 1.42 |
| rally/montalegre_rx | 6 | **+2.8%** | **-0.2%** | **-0.0%** | -2.5% | ✓✗✗✓ | 0 | 0 | 9.4 | 0.444 | 3.81 |
| rally/nyirad_rx | 6 | **+9.4%** | **+4.5%** | **+0.9%** | -1.8% | ✓✓✗✓ | 10 | 12 | 504.3 | 0.042 | 1.58 |
| rally/riga_rx | 6 | **-29.3%** | **-29.6%** | **+0.9%** | **+12.0%** | ✗✗✗✗ | 75 | 134 | 504.3 | 0.326 | 57.34 |
| rally/silverstone_rx | 6 | **+4.7%** | **+1.8%** | **+0.0%** | **+0.6%** | ✓✗✗✗ | 6 | 8 | 560.2 | 0.097 | 8.17 |
| rally/spa_rx | 6 | **-3.0%** | **-3.8%** | **+0.5%** | **-3.5%** | ✗✗✗✗ | 0 | 7 | 177.3 | 0.069 | 2.94 |

### Findings

- **No circuit is excluded.** All 75 are closed spline loops with 6 grid slots, and no request failed.
- **Rallycross is broken on 11 of 20 circuits.**
  - On some circuits bots do not finish, or stall for 100–550 s.
  - Some lap times are impossible. On `riga_rx`, T1 is 29 % faster than T4, and on `erx_motor_park` 18 % faster. Lap counting or the route probably has a fault.
  - These are circuit faults, not bot settings, so the tuning must not adjust to them.
  - Beads: `tdrace-699t6` (new), `tdrace-xg1x` (kouvola), `tdrace-bppc` (essay), `tdrace-4nmh3` (spa). The probable fix is spec 105, which redraws the jokers (`tdrace-dic8l`).
- **There are short stalls (10–35 s) on 16 Classic, Karting and Autocross circuits.** Most are at T1–T3 in traffic. Beads: `tdrace-t8biy` (new) and `tdrace-le75`.
- **Kart bots spin often.** At T4 they spin 0.8–1.6 times per lap on 11 Karting circuits, and hit walls 3–8 times per lap. Beads: `tdrace-4dyav` (new). The Classic kart circuits are clean.
- **The tier ladder.**
  - The order holds on all 4 steps on 12 of 18 Classic, 9 of 17 Autocross and 6 of 20 Karting circuits.
  - T3 is often too close to T4 (+0.3 to +1.0 %), mainly in Autocross.
  - T5 is often not faster than T4 in Karting.
  - Stage B tunes these numbers.

