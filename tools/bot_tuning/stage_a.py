"""Spec 083 Stage A: tune the shared (Balanced) controller with EvoRL CMA-ES on blaze.

See specs/083_evorl_bot_driving_style_and_performance_optimization.md, "What gets tuned".

The loop drives EvoRL's `CMAESEmitter`, `Evaluator` and `MatchLedger` directly. EvoRL's campaign
classes are not used: at commit 11b9ac3 their successive halving keeps unranked candidates and
replays seeds (see reports/bot_calibration_report.md).

    python tools/bot_tuning/stage_a.py --workers 28 --generations 25 --out results/stage_a
"""

from __future__ import annotations

import argparse
import dataclasses
import json
import sys
import time
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))

import cma
import harness
import numpy as np
import worker
from evorl.core.genome import Genome, ParameterKind, ParameterSpace, ParameterSpec
from evorl.core.ledger import MatchLedger
from evorl.evo.evaluator import Evaluator
from evorl.evo.search import CMAESEmitter

# (name, low, high, current Balanced constant). Bounds match GENOME_BOUNDS in
# crates/tdrace-app/src/ai/bot_harness.rs; constants match StyleTuning::for_style(Balanced) in
# crates/race-kit/src/ai/mod.rs. `check_incumbent` fails the run if the constants drift.
STAGE_A = [
    ("lookahead_time", 0.25, 0.55, 0.38),
    ("steering_kp", 1.50, 3.50, 2.10),
    ("steering_kd", 0.03, 0.12, 0.07),
    ("brake_margin", 0.85, 1.30, 1.05),
    ("speed_mult", 0.94, 1.06, 0.98),
]
# Share of a candidate's matches with a spin or a stall that costs the whole Wilson bound.
DISQUALIFY_WEIGHT = 1.0


class IncumbentCMAES(CMAESEmitter):
    """`CMAESEmitter` whose search starts at the current constants, not the box centre."""

    def __init__(
        self,
        space: ParameterSpace,
        x0: Genome,
        population: int,
        sigma0: float,
        seed: int,
    ) -> None:
        super().__init__(space, population=population, sigma0=sigma0, seed=seed)
        opts = {
            "seed": seed,
            "bounds": [-1.0, 1.0],
            "verbose": -9,
            "popsize": population,
        }
        self._es = cma.CMAEvolutionStrategy(space.normalize_vector(x0), sigma0, opts)


def check_incumbent(incumbent: Genome) -> None:
    """The incumbent genome must drive exactly like the game's Balanced bot."""
    module, circuit = worker.TRAINING_CIRCUITS[0]
    plain = worker.race(harness.stage_a_request(f"{module}/{circuit}", 7, 0, {}))
    same = worker.race(
        harness.stage_a_request(f"{module}/{circuit}", 7, 0, dict(incumbent.params))
    )
    if plain["error"] or plain["controls_hash"] != same["controls_hash"]:
        raise SystemExit(
            f"STAGE_A constants differ from the Rust Balanced tuning: {plain} vs {same}"
        )


def score(ledger: MatchLedger, campaign: str, genome: Genome) -> dict[str, Any]:
    fit = ledger.fitness(campaign, f"candidate@{genome.key}")
    disq = fit.metrics_mean.get("disqualified", 0.0)
    return {
        "key": genome.key,
        "params": dict(genome.params),
        "score": fit.wilson_lower - DISQUALIFY_WEIGHT * disq,
        "fitness": dataclasses.asdict(fit),
    }


def run(args: argparse.Namespace) -> dict[str, Any]:
    args.out.mkdir(parents=True, exist_ok=True)
    space = ParameterSpace(
        [
            ParameterSpec(name=n, kind=ParameterKind.REAL, low=lo, high=hi, default=d)
            for n, lo, hi, d in STAGE_A
        ]
    )
    incumbent = space.default_genome()
    check_incumbent(incumbent)
    emitter = IncumbentCMAES(space, incumbent, args.population, args.sigma0, args.seed)
    ledger = MatchLedger(args.out / "ledger.sqlite")
    evaluator = Evaluator(
        ledger=ledger,
        play_fn=harness.stage_a_play_fn,
        max_workers=args.workers,
        timeout_per_match=300.0,
    )
    circuits = [f"{m}/{c}" for m, c in worker.TRAINING_CIRCUITS]
    history: list[dict[str, Any]] = []
    seen: dict[str, dict[str, Any]] = {}
    started = time.perf_counter()
    try:
        for gen in range(args.generations):
            candidates = emitter.ask()
            # The incumbent also races in generation 0, as a reference; CMA-ES is not told its score.
            extra = [incumbent] if gen == 0 else []
            seeds = [args.seed * 1_000_000 + gen * 100 + i for i in range(args.seeds)]
            campaign = f"stage_a_g{gen:03d}"
            evaluator.evaluate_candidates(campaign, candidates + extra, circuits, seeds)
            scored = [score(ledger, campaign, g) for g in candidates + extra]
            emitter.tell([s["score"] for s in scored[: len(candidates)]])
            for s in scored:
                if s["key"] not in seen or s["score"] > seen[s["key"]]["score"]:
                    seen[s["key"]] = {**s, "generation": gen}
            best = max(scored, key=lambda s: s["score"])
            history.append(
                {
                    "generation": gen,
                    "best_score": best["score"],
                    "mean_score": float(np.mean([s["score"] for s in scored])),
                    "best_params": best["params"],
                    "elapsed_s": time.perf_counter() - started,
                }
            )
            print(json.dumps(history[-1]), flush=True)

        # Winner's curse: re-evaluate the top candidates and the incumbent on fresh seeds.
        top = sorted(seen.values(), key=lambda s: s["score"], reverse=True)[
            : args.finalists
        ]
        finalists = [Genome(t["params"]) for t in top]
        if incumbent.key not in {g.key for g in finalists}:
            finalists.append(incumbent)
        seeds = [args.seed * 1_000_000 + 900_000 + i for i in range(args.final_seeds)]
        evaluator.evaluate_candidates("stage_a_final", finalists, circuits, seeds)
        final = sorted(
            (score(ledger, "stage_a_final", g) for g in finalists),
            key=lambda s: s["score"],
            reverse=True,
        )
    finally:
        evaluator.shutdown()
    baseline = next(f for f in final if f["key"] == incumbent.key)
    report = {
        "evorl_commit": args.evorl_commit,
        "tdrace_commit": args.tdrace_commit,
        "seed": args.seed,
        "generations": args.generations,
        "population": args.population,
        "seeds_per_generation": args.seeds,
        "circuits": circuits,
        "elapsed_s": time.perf_counter() - started,
        "history": history,
        "final": final,
        "best": final[0],
        "baseline": baseline,
    }
    (args.out / "stage_a.json").write_text(json.dumps(report, indent=2) + "\n")
    return report


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--workers", type=int, default=28)
    parser.add_argument("--generations", type=int, default=25)
    parser.add_argument("--population", type=int, default=14)
    parser.add_argument(
        "--seeds", type=int, default=4, help="seed pairs per circuit per generation"
    )
    parser.add_argument(
        "--sigma0",
        type=float,
        default=0.3,
        help="initial step in the normalized [-1, 1] box",
    )
    parser.add_argument("--finalists", type=int, default=5)
    parser.add_argument("--final-seeds", type=int, default=16)
    parser.add_argument("--seed", type=int, default=83)
    parser.add_argument("--evorl-commit", default="unknown")
    parser.add_argument("--tdrace-commit", default="unknown")
    parser.add_argument("--out", type=Path, default=Path("results/stage_a"))
    report = run(parser.parse_args())
    best, base = report["best"], report["baseline"]
    print(
        json.dumps(
            {
                "best": best["params"],
                "best_score": best["score"],
                "baseline_score": base["score"],
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
