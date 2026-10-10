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
