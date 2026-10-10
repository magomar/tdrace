"""Spec 083 calibration runs: throughput measurement and an EvoRL smoke run (more stages follow).

See specs/083_evorl_bot_driving_style_and_performance_optimization.md.

    python tools/bot_tuning/run_campaign.py throughput --workers 28 --out results/throughput.json
    python tools/bot_tuning/run_campaign.py evorl-smoke --workers 28 --db results/smoke.sqlite
"""

from __future__ import annotations

import argparse
import dataclasses
import json
import os
import platform
import sys
import time
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))

import worker


def pair_request(module: str, circuit: str, seed: int, laps: int = 3) -> dict[str, Any]:
    """A T4 Balanced subject against a T4 Balanced opponent."""
    t4 = {"style": "balanced", "tier": 4}
    return {
        "circuit": f"{module}/{circuit}",
        "laps": laps,
        "seed": seed,
        "subject": t4,
        "opponents": [t4],
    }


def _race(request: dict[str, Any]) -> dict[str, Any]:
    return worker.race(request)


def throughput(workers: int, rounds: int) -> dict[str, Any]:
    """Runs `rounds` 3-lap 2-car races on every training circuit with `workers` processes."""
    requests = [
        pair_request(m, c, seed)
        for seed in range(rounds)
        for m, c in worker.TRAINING_CIRCUITS
    ]
    with ProcessPoolExecutor(max_workers=workers) as pool:
        list(
            pool.map(_race, requests[:workers], chunksize=1)
        )  # start every worker before timing
        started = time.perf_counter()
        results = list(pool.map(_race, requests, chunksize=1))
        elapsed = time.perf_counter() - started
    errors = [r["error"] for r in results if r["error"]]
    car_steps = sum(r["car_steps"] for r in results)
    per_circuit: dict[str, list[float]] = {}
    for (m, c), r in zip(
        [(m, c) for _ in range(rounds) for m, c in worker.TRAINING_CIRCUITS], results
    ):
        per_circuit.setdefault(f"{m}/{c}", []).append(r["wall_time_ms"])
    return {
        "host": platform.node(),
        "cpus": os.cpu_count(),
        "workers": workers,
        "races": len(results),
        "errors": errors,
        "elapsed_s": elapsed,
        "races_per_s": len(results) / elapsed,
        "car_steps_per_s": car_steps / elapsed,
        "mean_race_ms": {k: sum(v) / len(v) for k, v in per_circuit.items()},
    }


def evorl_smoke(workers: int, db: Path) -> dict[str, Any]:
    """The default genome against the baseline through EvoRL's Evaluator and ledger, on 2 circuits.

    Both drivers are the same bot, so the outcome mix shows only grid and seat effects.
    """
    import harness
    from evorl.core.genome import Genome
    from evorl.core.ledger import MatchLedger
    from evorl.evo.evaluator import Evaluator

    db.parent.mkdir(parents=True, exist_ok=True)
    ledger = MatchLedger(db)
    evaluator = Evaluator(
        ledger=ledger,
        play_fn=harness.stage_a_play_fn,
        max_workers=workers,
        timeout_per_match=120.0,
    )
    circuits = [f"{m}/{c}" for m, c in worker.TRAINING_CIRCUITS[:2]]
    started = time.perf_counter()
    genome = Genome({})
    evaluator.evaluate_candidates("smoke", [genome], circuits, [1, 2])
    elapsed = time.perf_counter() - started
    evaluator.shutdown()
    fit = ledger.fitness("smoke", f"candidate@{genome.key}")
    return {"elapsed_s": elapsed, "fitness": dataclasses.asdict(fit)}


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = parser.add_subparsers(dest="command", required=True)
    tp = sub.add_parser("throughput", help="measure races/s and car-steps/s")
    tp.add_argument("--workers", type=int, default=1)
    tp.add_argument("--rounds", type=int, default=2, help="races per training circuit")
    tp.add_argument("--out", type=Path)
    smoke = sub.add_parser("evorl-smoke", help="run the play function through EvoRL")
    smoke.add_argument("--workers", type=int, default=8)
    smoke.add_argument("--db", type=Path, default=Path("results/smoke.sqlite"))
    smoke.add_argument("--out", type=Path)
    args = parser.parse_args()

    if args.command == "throughput":
        report = throughput(args.workers, args.rounds)
    elif args.command == "evorl-smoke":
        report = evorl_smoke(args.workers, args.db)
    out = json.dumps(report, indent=2)
    print(out)
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(out + "\n")


if __name__ == "__main__":
    main()
