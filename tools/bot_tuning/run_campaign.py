"""Spec 083 calibration runs: throughput, EvoRL smoke run, full all-modality audit.

See specs/083_evorl_bot_driving_style_and_performance_optimization.md.

    python tools/bot_tuning/run_campaign.py throughput --workers 28 --out tools/bot_tuning/runs/throughput.json
    python tools/bot_tuning/run_campaign.py evorl-smoke --workers 28 --db tools/bot_tuning/runs/smoke.sqlite
    python tools/bot_tuning/run_campaign.py audit --workers 28 --out tools/bot_tuning/runs/audit_baseline.json
"""

from __future__ import annotations

import argparse
import dataclasses
import itertools
import json
import math
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
    """Runs `rounds` 3-lap 2-car races on every closed-loop circuit across all 7 modalities."""
    circuits = worker.all_circuits()
    requests = [
        pair_request(m, c, seed)
        for seed in range(rounds)
        for m, c in circuits
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
        [(m, c) for _ in range(rounds) for m, c in circuits], results
    ):
        per_circuit.setdefault(f"{m}/{c}", []).append(r["wall_time_ms"])
    return {
        "host": platform.node(),
        "cpus": os.cpu_count(),
        "workers": workers,
        "circuits": len(circuits),
        "races": len(results),
        "errors": errors,
        "elapsed_s": elapsed,
        "races_per_s": len(results) / elapsed,
        "car_steps_per_s": car_steps / elapsed,
        "mean_race_ms": {k: sum(v) / len(v) for k, v in per_circuit.items()},
    }


def evorl_smoke(workers: int, db: Path) -> dict[str, Any]:
    """The default genome against the baseline through EvoRL's Evaluator and ledger, on 2 circuits."""
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
    circuits = [f"balanced@{m}/{c}" for m, c in worker.TRAINING_CIRCUITS[:2]]
    started = time.perf_counter()
    genome = Genome({})
    evaluator.evaluate_candidates("smoke", [genome], circuits, [1, 2])
    elapsed = time.perf_counter() - started
    evaluator.shutdown()
    fit = ledger.fitness("smoke", f"candidate@{genome.key}")
    return {"elapsed_s": elapsed, "fitness": dataclasses.asdict(fit)}


STYLES = ["balanced", "smooth", "aggressive", "tenacious", "calculating", "bold"]
TIERS = [1, 2, 3, 4, 5]
# Spec 083: a bot without progress for longer than this is stuck (classic_circuits_bot_tests).
STUCK_S = 10.0


def grid_request(
    module: str, circuit: str, tier: int, seed: int, cars: int
) -> dict[str, Any]:
    """A grid of `cars` bots at one tier, one per style (Balanced is the subject)."""
    drivers = [{"style": style, "tier": tier} for style in STYLES[:cars]]
    return {
        "circuit": f"{module}/{circuit}",
        "laps": 3,
        "seed": seed,
        "subject": drivers[0],
        "opponents": drivers[1:],
    }


def _grid_race(job: tuple[str, str, int, int]) -> dict[str, Any]:
    """One audit race; the grid shrinks when the circuit has fewer than 6 grid positions."""
    module, circuit, tier, seed = job
    for cars in (6, 4, 2, 1):
        result = worker.race(grid_request(module, circuit, tier, seed, cars))
        if not (result["error"] and "grid positions" in result["error"]):
            break
    return {
        "module": module,
        "circuit": circuit,
        "tier": tier,
        "seed": seed,
        "cars": cars,
        **result,
    }


def _mean_se(xs: list[float]) -> tuple[float | None, float | None]:
    if not xs:
        return None, None
    mean = sum(xs) / len(xs)
    if len(xs) < 2:
        return mean, None
    var = sum((x - mean) ** 2 for x in xs) / (len(xs) - 1)
    return mean, math.sqrt(var / len(xs))


def summarize_circuit(races: list[dict[str, Any]]) -> dict[str, Any]:
    """Per-tier pace, mistakes and stalls of one circuit, and whether the tier order holds."""
    errors = sorted({r["error"] for r in races if r["error"]})
    ok = [r for r in races if not r["error"]]
    tiers: dict[int, dict[str, Any]] = {}
    for tier in TIERS:
        rs = [r for r in ok if r["tier"] == tier]
        per_car = [
            {
                k: r[k]
                for k in (
                    "finished",
                    "mean_flying_lap",
                    "spins",
                    "offs",
                    "mistakes",
                    "wall_contacts",
                    "longest_no_progress_s",
                )
            }
            for r in rs
        ] + [o for r in rs for o in r["opponents"]]
        # One sample per race: the mean flying lap of the grid, so seeds are independent.
        race_means = []
        for r in rs:
            laps = [
                c["mean_flying_lap"]
                for c in [r, *r["opponents"]]
                if c["mean_flying_lap"]
            ]
            if laps:
                race_means.append(sum(laps) / len(laps))
        mean, se = _mean_se(race_means)
        n_laps = max(1, 3 * len(per_car))  # scheduled laps
        tiers[tier] = {
            "races": len(rs),
            "cars": len(per_car),
            "mean_flying_lap": mean,
            "se": se,
            "dnf": sum(1 for c in per_car if not c["finished"]),
            "stuck": sum(1 for c in per_car if c["longest_no_progress_s"] > STUCK_S),
            "max_no_progress_s": max(
                (c["longest_no_progress_s"] for c in per_car), default=0.0
            ),
            "spins_per_lap": sum(c["spins"] for c in per_car) / n_laps,
            "offs_per_lap": sum(c["offs"] for c in per_car) / n_laps,
            "mistakes_per_lap": sum(c["mistakes"] for c in per_car) / n_laps,
            "wall_contacts_per_lap": sum(c["wall_contacts"] for c in per_car) / n_laps,
        }
    t4 = tiers[4]["mean_flying_lap"]
    for t in tiers.values():
        t["delta_vs_t4"] = (
            (t["mean_flying_lap"] / t4 - 1.0) if t4 and t["mean_flying_lap"] else None
        )
    order = []
    for slow, fast in itertools.pairwise(TIERS):
        a, b = tiers[slow], tiers[fast]
        held = None
        if (
            a["mean_flying_lap"]
            and b["mean_flying_lap"]
            and a["se"] is not None
            and b["se"] is not None
        ):
            held = a["mean_flying_lap"] - b["mean_flying_lap"] > 2.0 * math.hypot(
                a["se"], b["se"]
            )
        order.append(held)
    closed = all(r["closed_loop"] for r in ok) if ok else None
    return {
        "closed_loop": closed,
        "errors": errors,
        "grid_cars": max((r["cars"] for r in ok), default=0),
        "tiers": tiers,
        "order_held": order,
    }


def audit(
    workers: int, seeds: int, circuits: list[tuple[str, str]] | None
) -> dict[str, Any]:
    """Every tier on every closed-loop circuit across all 7 modalities, `seeds` 3-lap grid races each."""
    circuits = circuits or worker.all_circuits()
    jobs = [
        (m, c, tier, 1000 + seed)
        for m, c in circuits
        for tier in TIERS
        for seed in range(seeds)
    ]
    started = time.perf_counter()
    with ProcessPoolExecutor(max_workers=workers) as pool:
        races = list(pool.map(_grid_race, jobs, chunksize=1))
    by_circuit: dict[str, list[dict[str, Any]]] = {}
    for r in races:
        by_circuit.setdefault(f"{r['module']}/{r['circuit']}", []).append(r)
    return {
        "host": platform.node(),
        "cpus": os.cpu_count(),
        "workers": workers,
        "seeds": seeds,
        "races": len(races),
        "car_steps": sum(r["car_steps"] for r in races),
        "elapsed_s": time.perf_counter() - started,
        "circuits": {k: summarize_circuit(v) for k, v in by_circuit.items()},
    }


def write_json(path: Path | None, data: Any) -> None:
    text = json.dumps(data, indent=2)
    if path:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text + "\n")
    else:
        print(text)


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = parser.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("throughput")
    p.add_argument("--workers", type=int, default=28)
    p.add_argument("--rounds", type=int, default=2)
    p.add_argument("--out", type=Path)

    p = sub.add_parser("evorl-smoke")
    p.add_argument("--workers", type=int, default=28)
    p.add_argument("--db", type=Path, required=True)
    p.add_argument("--out", type=Path)

    p = sub.add_parser("audit")
    p.add_argument("--workers", type=int, default=28)
    p.add_argument("--seeds", type=int, default=4)
    p.add_argument(
        "--training-only",
        action="store_true",
        help="run only the 13 legacy training circuits",
    )
    p.add_argument(
        "--launch-only",
        action="store_true",
        help="run only the 75 launch module circuits",
    )
    p.add_argument("--out", type=Path)

    args = parser.parse_args()
    if args.cmd == "throughput":
        write_json(args.out, throughput(args.workers, args.rounds))
    elif args.cmd == "evorl-smoke":
        write_json(args.out, evorl_smoke(args.workers, args.db))
    elif args.cmd == "audit":
        if args.training_only:
            circuits = worker.TRAINING_CIRCUITS
        elif args.launch_only:
            circuits = worker.launch_circuits()
        else:
            circuits = worker.all_circuits()
        write_json(args.out, audit(args.workers, args.seeds, circuits))


if __name__ == "__main__":
    main()
