"""Spec 083 Stage B: tune the driver quality rows for each tier with EvoRL CMA-ES on blaze.

See specs/083_evorl_bot_driving_style_and_performance_optimization.md, "What gets tuned".

Runs tier by tier outward from T4 Pro (T3, T2, T1, T5).
Each tier is tuned against the T4 Pro reference car (equipped with calibrated Stage A controller)
to hit its target pace delta band:
  - T3: +2.5% +- 1.0% (+1.5% to +3.5%)
  - T2: +7.0% +- 2.0% (+5.0% to +9.0%)
  - T1: +13.0% +- 3.0% (+10.0% to +16.0%)
  - T5: -2.25% +- 0.75% (-3.0% to -1.5%)

    python tools/bot_tuning/stage_b.py --workers 28 --generations 12 --out results/stage_b
"""

from __future__ import annotations

import argparse
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

TIER_CONFIGS = [
    {
        "tier": 3,
        "name": "Contender",
        "target_centre": 0.025,
        "target_half_width": 0.010,
        "defaults": {
            "pace_limit": 0.96,
            "brake_padding": 0.06,
            "consistency": 0.83,
            "composure": 0.78,
        },
        "specs": [
            ("pace_limit", 0.88, 1.04, 0.96),
            ("brake_padding", -0.05, 0.20, 0.06),
            ("consistency", 0.78, 0.91, 0.83),
            ("composure", 0.68, 0.88, 0.78),
        ],
    },
    {
        "tier": 2,
        "name": "Amateur",
        "target_centre": 0.070,
        "target_half_width": 0.020,
        "defaults": {
            "pace_limit": 0.90,
            "brake_padding": 0.12,
            "consistency": 0.72,
            "composure": 0.60,
        },
        "specs": [
            ("pace_limit", 0.82, 0.98, 0.90),
            ("brake_padding", 0.00, 0.25, 0.12),
            ("consistency", 0.65, 0.77, 0.72),
            ("composure", 0.52, 0.67, 0.60),
        ],
    },
    {
        "tier": 1,
        "name": "Rookie",
        "target_centre": 0.130,
        "target_half_width": 0.030,
        "defaults": {
            "pace_limit": 0.82,
            "brake_padding": 0.20,
            "consistency": 0.60,
            "composure": 0.48,
        },
        "specs": [
            ("pace_limit", 0.72, 0.92, 0.82),
            ("brake_padding", 0.05, 0.30, 0.20),
            ("consistency", 0.45, 0.64, 0.60),
            ("composure", 0.35, 0.51, 0.48),
        ],
    },
    {
        "tier": 5,
        "name": "Legend",
        "target_centre": -0.055,
        "target_half_width": 0.015,
        "defaults": {
            "pace_limit": 1.06,
            "brake_padding": -0.05,
            "consistency": 0.99,
            "composure": 0.98,
        },
        "specs": [
            ("pace_limit", 1.00, 1.15, 1.06),
            ("brake_padding", -0.15, 0.05, -0.05),
            ("consistency", 0.94, 1.00, 0.99),
            ("composure", 0.91, 1.00, 0.98),
        ],
    },
]


class IncumbentCMAES(CMAESEmitter):
    """`CMAESEmitter` whose search starts at the baseline defaults, not the box centre."""

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


def score_tier(
    ledger: MatchLedger,
    campaign: str,
    genome: Genome,
    target_c: float,
    target_h: float,
) -> dict[str, Any]:
    subj_id = f"candidate@{genome.key}"
    records = [
        r for r in ledger.get_records(campaign=campaign) if r.subject == subj_id
    ]
    if not records:
        return {
            "key": genome.key,
            "params": dict(genome.params),
            "score": -999.0,
            "metrics": {},
        }

    by_circuit: dict[str, list[float]] = {}
    disqs = 0
    spins = 0
    stalls = 0
    dnfs = 0
    for r in records:
        m = r.metrics
        c = r.opponent.split("@", 1)[1]
        if m.get("delta") is not None:
            by_circuit.setdefault(c, []).append(m["delta"])
        if m.get("disqualified"):
            disqs += 1
        if m.get("spins", 0) > 0:
            spins += 1
        if m.get("longest_no_progress_s", 0) > 10.0:
            stalls += 1
        if m.get("dnf"):
            dnfs += 1

    total = len(records)
    disq_rate = disqs / total if total > 0 else 1.0

    circuit_errors = []
    circuit_deltas = {}
    circuits_in_band = 0
    for c, deltas in by_circuit.items():
        c_mean = float(np.mean(deltas))
        circuit_deltas[c] = c_mean
        # E_k = max(0, |Delta_k - c_k| - h_k)^2
        err = max(0.0, abs(c_mean - target_c) - target_h)
        circuit_errors.append((err * 100.0) ** 2)
        if err == 0.0:
            circuits_in_band += 1

    mean_circuit_err = float(np.mean(circuit_errors)) if circuit_errors else 100.0
    mean_delta = (
        float(np.mean(list(circuit_deltas.values()))) if circuit_deltas else 0.0
    )
    center_err = ((mean_delta - target_c) * 100.0) ** 2

    # Higher score is better
    score_val = -(mean_circuit_err + 0.1 * center_err) - 2.0 * disq_rate

    return {
        "key": genome.key,
        "params": dict(genome.params),
        "score": score_val,
        "metrics": {
            "mean_delta_pct": round(mean_delta * 100.0, 2),
            "target_delta_pct": round(target_c * 100.0, 2),
            "circuits_in_band": circuits_in_band,
            "total_circuits": len(circuit_deltas),
            "mean_band_error": round(mean_circuit_err, 4),
            "disq_rate": round(disq_rate, 4),
            "spins": spins,
            "stalls": stalls,
            "dnfs": dnfs,
            "circuit_deltas_pct": {
                c: round(d * 100.0, 2) for c, d in circuit_deltas.items()
            },
        },
    }


def tune_tier(
    tier_cfg: dict[str, Any],
    evaluator: Evaluator,
    ledger: MatchLedger,
    args: argparse.Namespace,
) -> dict[str, Any]:
    tier = tier_cfg["tier"]
    name = tier_cfg["name"]
    target_c = tier_cfg["target_centre"]
    target_h = tier_cfg["target_half_width"]
    print(
        f"\n=== Calibrating Tier {tier} ({name}) | Target: {target_c*100:+.2f}% +- {target_h*100:.2f}% ==="
    )

    space = ParameterSpace(
        [
            ParameterSpec(
                name=n, kind=ParameterKind.REAL, low=lo, high=hi, default=d
            )
            for n, lo, hi, d in tier_cfg["specs"]
        ]
    )
    incumbent = space.default_genome()
    emitter = IncumbentCMAES(
        space,
        incumbent,
        args.population,
        args.sigma0,
        args.seed + tier * 10_000,
    )
    opponents = [f"{tier}@{m}/{c}" for m, c in worker.TRAINING_CIRCUITS]

    history: list[dict[str, Any]] = []
    seen: dict[str, dict[str, Any]] = {}
    started = time.perf_counter()

    for gen in range(args.generations):
        candidates = emitter.ask()
        extra = [incumbent] if gen == 0 else []
        seeds = [
            args.seed * 1_000_000 + tier * 100_000 + gen * 100 + i
            for i in range(args.seeds)
        ]
        campaign = f"stage_b_t{tier}_g{gen:03d}"
        evaluator.evaluate_candidates(
            campaign, candidates + extra, opponents, seeds
        )

        scored = [
            score_tier(ledger, campaign, g, target_c, target_h)
            for g in candidates + extra
        ]
        emitter.tell([s["score"] for s in scored[: len(candidates)]])

        for s in scored:
            if s["key"] not in seen or s["score"] > seen[s["key"]]["score"]:
                seen[s["key"]] = {**s, "generation": gen}

        best = max(scored, key=lambda s: s["score"])
        history.append(
            {
                "generation": gen,
                "best_score": best["score"],
                "best_params": best["params"],
                "best_metrics": best["metrics"],
                "mean_score": float(np.mean([s["score"] for s in scored])),
                "elapsed_s": time.perf_counter() - started,
            }
        )
        print(
            f"T{tier} Gen {gen:02d} | Best score: {best['score']:.4f} | "
            f"Delta: {best['metrics']['mean_delta_pct']:+.2f}% | "
            f"In-band: {best['metrics']['circuits_in_band']}/{best['metrics']['total_circuits']} | "
            f"Disq: {best['metrics']['disq_rate']*100:.1f}% | Elapsed: {history[-1]['elapsed_s']:.1f}s",
            flush=True,
        )

    # Re-evaluate top finalists and incumbent on fresh seeds
    top = sorted(seen.values(), key=lambda s: s["score"], reverse=True)[
        : args.finalists
    ]
    finalists = [Genome(t["params"]) for t in top]
    if incumbent.key not in {g.key for g in finalists}:
        finalists.append(incumbent)

    final_seeds = [
        args.seed * 1_000_000 + tier * 100_000 + 900_000 + i
        for i in range(args.final_seeds)
    ]
    final_campaign = f"stage_b_t{tier}_final"
    evaluator.evaluate_candidates(
        final_campaign, finalists, opponents, final_seeds
    )

    final = sorted(
        (
            score_tier(ledger, final_campaign, g, target_c, target_h)
            for g in finalists
        ),
        key=lambda s: s["score"],
        reverse=True,
    )
    baseline = next(f for f in final if f["key"] == incumbent.key)
    best_final = final[0]

    print(
        f"T{tier} Finalist Best: Score {best_final['score']:.4f} | "
        f"Delta: {best_final['metrics']['mean_delta_pct']:+.2f}% (Target: {target_c*100:+.2f}%) | "
        f"In-band: {best_final['metrics']['circuits_in_band']}/{best_final['metrics']['total_circuits']}"
    )
    print(f"  Params: {best_final['params']}")

    return {
        "tier": tier,
        "name": name,
        "target_centre": target_c,
        "target_half_width": target_h,
        "best": best_final,
        "baseline": baseline,
        "history": history,
        "final": final,
    }


def run(args: argparse.Namespace) -> dict[str, Any]:
    args.out.mkdir(parents=True, exist_ok=True)
    # Ensure Stage A params are present
    harness.get_stage_a_params()

    ledger = MatchLedger(args.out / "ledger.sqlite")
    evaluator = Evaluator(
        ledger=ledger,
        play_fn=harness.stage_b_play_fn,
        max_workers=args.workers,
        timeout_per_match=300.0,
    )

    tier_results = {}
    started = time.perf_counter()
    configs = [c for c in TIER_CONFIGS if args.tier is None or c["tier"] == args.tier]

    try:
        for cfg in configs:
            tier_results[f"tier_{cfg['tier']}"] = tune_tier(
                cfg, evaluator, ledger, args
            )
    finally:
        evaluator.shutdown()

    summary = {
        "evorl_commit": args.evorl_commit,
        "tdrace_commit": args.tdrace_commit,
        "seed": args.seed,
        "generations_per_tier": args.generations,
        "population": args.population,
        "seeds_per_gen": args.seeds,
        "final_seeds": args.final_seeds,
        "elapsed_s": time.perf_counter() - started,
        "tiers": tier_results,
    }

    (args.out / "stage_b.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(f"\nStage B completed in {summary['elapsed_s']:.1f} s. Saved to {args.out / 'stage_b.json'}")
    return summary


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--workers", type=int, default=28)
    parser.add_argument("--generations", type=int, default=12)
    parser.add_argument("--population", type=int, default=10)
    parser.add_argument(
        "--seeds", type=int, default=4, help="seed pairs per circuit per generation"
    )
    parser.add_argument(
        "--sigma0",
        type=float,
        default=0.25,
        help="initial step in normalized [-1, 1] box",
    )
    parser.add_argument("--finalists", type=int, default=4)
    parser.add_argument("--final-seeds", type=int, default=16)
    parser.add_argument("--seed", type=int, default=83)
    parser.add_argument("--tier", type=int, default=None, help="only tune a single tier")
    parser.add_argument("--evorl-commit", default="11b9ac3")
    parser.add_argument("--tdrace-commit", default="61e334dd")
    parser.add_argument("--out", type=Path, default=Path("results/stage_b"))
    run(parser.parse_args())


if __name__ == "__main__":
    main()
