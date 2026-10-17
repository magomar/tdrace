"""Spec 083 Stage A: tune per-modality, per-style controllers with EvoRL CMA-ES on blaze.

See specs/083_evorl_bot_driving_style_and_performance_optimization.md, "What gets tuned".

Trains all 6 DrivingStyle controllers (Smooth, Aggressive, Tenacious, Calculating, Bold, Balanced)
independently for each of the 7 game modalities (classic, kart, autocross, rally, gt, nascar,
extreme_offroad), evaluating every candidate across ALL closed-loop circuits of that modality.

    python tools/bot_tuning/stage_a.py --workers 28 --generations 8 --out tools/bot_tuning/runs
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
from evorl.core.ledger import MatchLedger, MatchRecord, Outcome
from evorl.evo.evaluator import Evaluator, _worker_execute_match
from evorl.evo.search import CMAESEmitter

# Per-style search boxes: (param_name, low, high, baseline_default).
# Preserves style tactical identity and guarantees all Rust profile/tier invariants.
STYLE_SPECS: dict[str, list[tuple[str, float, float, float]]] = {
    "smooth": [
        ("lookahead_time", 0.28, 0.46, 0.36),
        ("steering_kp", 1.80, 2.70, 2.20),
        ("steering_kd", 0.06, 0.115, 0.09),
        ("brake_margin", 0.99, 1.14, 1.05),
        ("speed_mult", 1.02, 1.08, 1.03),
    ],
    "aggressive": [
        ("lookahead_time", 0.20, 0.35, 0.28),
        ("steering_kp", 2.10, 3.20, 2.40),
        ("steering_kd", 0.04, 0.09, 0.06),
        ("brake_margin", 0.86, 0.98, 0.93),
        ("speed_mult", 1.01, 1.085, 1.04),
    ],
    "tenacious": [
        ("lookahead_time", 0.28, 0.48, 0.38),
        ("steering_kp", 1.75, 2.65, 2.10),
        ("steering_kd", 0.06, 0.115, 0.09),
        ("brake_margin", 1.00, 1.16, 1.08),
        ("speed_mult", 0.995, 1.065, 1.01),
    ],
    "calculating": [
        ("lookahead_time", 0.27, 0.45, 0.35),
        ("steering_kp", 1.85, 2.80, 2.30),
        ("steering_kd", 0.06, 0.115, 0.08),
        ("brake_margin", 0.98, 1.14, 1.03),
        ("speed_mult", 1.00, 1.075, 1.02),
    ],
    "bold": [
        ("lookahead_time", 0.20, 0.34, 0.27),
        ("steering_kp", 2.20, 3.35, 2.60),
        ("steering_kd", 0.04, 0.085, 0.05),
        ("brake_margin", 0.85, 0.96, 0.91),
        ("speed_mult", 1.005, 1.08, 1.03),
    ],
    "balanced": [
        ("lookahead_time", 0.26, 0.44, 0.34),
        ("steering_kp", 1.75, 2.65, 2.00),
        ("steering_kd", 0.05, 0.115, 0.08),
        ("brake_margin", 1.02, 1.16, 1.08),
        ("speed_mult", 0.995, 1.065, 1.00),
    ],
}

DISQUALIFY_WEIGHT = 1.2


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


class BatchEvaluator(Evaluator):
    """`Evaluator` subclass that dispatches all candidate matches of a generation in one parallel batch."""

    def evaluate_batches(
        self,
        batches: list[tuple[str, list[Genome], list[str], list[int]]],
    ) -> None:
        assert self._executor is not None and self.play_fn is not None
        futures = []
        for campaign, candidates, opponents, seeds in batches:
            for candidate in candidates:
                subj_id = f"candidate@{candidate.key}"
                params_json = candidate.to_json()
                for opponent in opponents:
                    for seed in seeds:
                        for seat in (0, 1):
                            fut = self._executor.submit(
                                _worker_execute_match,
                                self.play_fn,
                                campaign,
                                subj_id,
                                opponent,
                                seed,
                                seat,
                                params_json,
                            )
                            futures.append((campaign, subj_id, fut))
        records: list[MatchRecord] = []
        for campaign, subj_id, fut in futures:
            try:
                records.append(fut.result(timeout=self.timeout_per_match))
            except Exception as exc:
                records.append(
                    MatchRecord(
                        campaign=campaign,
                        subject=subj_id,
                        opponent="unknown",
                        seed=0,
                        seat=0,
                        outcome=Outcome.TIMEOUT,
                        metrics={"error": str(exc)},
                    )
                )
        self.ledger.record_matches(records)


def score(ledger: MatchLedger, campaign: str, genome: Genome) -> dict[str, Any]:
    fit = ledger.fitness(campaign, f"candidate@{genome.key}")
    disq = fit.metrics_mean.get("disqualified", 0.0)
    offs_rate = fit.metrics_mean.get("offs", 0.0) / 3.0
    wall_rate = fit.metrics_mean.get("wall_contacts", 0.0) / 3.0
    mean_gain = fit.mean_score if fit.mean_score is not None else 0.0
    composite = (
        fit.wilson_lower
        + 0.50 * mean_gain
        - DISQUALIFY_WEIGHT * disq
        - 0.04 * offs_rate
        - 0.03 * wall_rate
    )
    return {
        "key": genome.key,
        "params": dict(genome.params),
        "score": float(composite),
        "wilson_lower": float(fit.wilson_lower),
        "mean_speed_gain_pct": round(float(mean_gain) * 100.0, 3),
        "disq_rate": round(float(disq), 4),
        "fitness": dataclasses.asdict(fit),
    }


def tune_modality(
    modality: str,
    styles: list[str],
    circuits: list[str],
    evaluator: BatchEvaluator,
    ledger: MatchLedger,
    args: argparse.Namespace,
    mod_idx: int,
) -> dict[str, dict[str, Any]]:
    started = time.perf_counter()
    print(
        f"\n=== Stage A Modality: {modality.upper()} ({len(circuits)} closed-loop circuits, {len(styles)} styles, pop={args.population}, gens={args.generations}) ===",
        flush=True,
    )

    state: dict[str, dict[str, Any]] = {}
    for style_idx, style in enumerate(styles):
        specs = STYLE_SPECS[style]
        space = ParameterSpace(
            [
                ParameterSpec(name=n, kind=ParameterKind.REAL, low=lo, high=hi, default=d)
                for n, lo, hi, d in specs
            ]
        )
        incumbent = space.default_genome()
        sub_seed = args.seed + mod_idx * 1000 + style_idx * 100
        emitter = IncumbentCMAES(space, incumbent, args.population, args.sigma0, sub_seed)
        state[style] = {
            "style_idx": style_idx,
            "incumbent": incumbent,
            "emitter": emitter,
            "opponents": [f"{style}@{c}" for c in circuits],
            "history": [],
            "seen": {},
        }

    for gen in range(args.generations):
        gen_start = time.perf_counter()
        batches = []
        asked: dict[str, tuple[str, list[Genome], list[Genome]]] = {}
        for style in styles:
            st = state[style]
            style_idx = st["style_idx"]
            candidates = st["emitter"].ask()
            extra = [st["incumbent"]] if gen == 0 else []
            seeds = [
                args.seed * 1_000_000 + mod_idx * 100_000 + style_idx * 10_000 + gen * 100 + i
                for i in range(args.seeds)
            ]
            campaign = f"stage_a_{modality}_{style}_g{gen:03d}"
            asked[style] = (campaign, candidates, extra)
            batches.append((campaign, candidates + extra, st["opponents"], seeds))

        evaluator.evaluate_batches(batches)

        gen_summaries = []
        for style in styles:
            st = state[style]
            campaign, candidates, extra = asked[style]
            scored = [score(ledger, campaign, g) for g in candidates + extra]
            st["emitter"].tell([s["score"] for s in scored[: len(candidates)]])
            for s in scored:
                if s["key"] not in st["seen"] or s["score"] > st["seen"][s["key"]]["score"]:
                    st["seen"][s["key"]] = {**s, "generation": gen}
            best = max(scored, key=lambda s: s["score"])
            st["history"].append(
                {
                    "generation": gen,
                    "best_score": best["score"],
                    "best_wilson": best["wilson_lower"],
                    "best_speed_gain_pct": best["mean_speed_gain_pct"],
                    "best_disq_rate": best["disq_rate"],
                    "mean_score": float(np.mean([s["score"] for s in scored])),
                    "best_params": best["params"],
                    "elapsed_s": time.perf_counter() - started,
                }
            )
            gen_summaries.append(
                f"{style[:3]}={best['score']:.3f}({best['mean_speed_gain_pct']:+.1f}%)"
            )
        print(
            f"  [{modality}] gen {gen:02d} ({time.perf_counter() - gen_start:.1f}s) | "
            + " | ".join(gen_summaries),
            flush=True,
        )

    # Re-evaluate top finalists + incumbent on fresh seeds for all 6 styles of the modality in one batch
    final_batches = []
    final_meta: dict[str, tuple[str, list[Genome]]] = {}
    for style in styles:
        st = state[style]
        style_idx = st["style_idx"]
        incumbent = st["incumbent"]
        top = sorted(st["seen"].values(), key=lambda s: s["score"], reverse=True)[: args.finalists]
        finalists = [Genome(t["params"]) for t in top]
        if incumbent.key not in {g.key for g in finalists}:
            finalists.append(incumbent)
        final_seeds = [
            args.seed * 1_000_000 + mod_idx * 100_000 + style_idx * 10_000 + 9_000 + i
            for i in range(args.final_seeds)
        ]
        final_campaign = f"stage_a_{modality}_{style}_final"
        final_meta[style] = (final_campaign, finalists)
        final_batches.append((final_campaign, finalists, st["opponents"], final_seeds))

    evaluator.evaluate_batches(final_batches)

    results: dict[str, dict[str, Any]] = {}
    mod_elapsed = time.perf_counter() - started
    for style in styles:
        st = state[style]
        incumbent = st["incumbent"]
        final_campaign, finalists = final_meta[style]
        final = sorted(
            (score(ledger, final_campaign, g) for g in finalists),
            key=lambda s: s["score"],
            reverse=True,
        )
        baseline = next(f for f in final if f["key"] == incumbent.key)
        winner = final[0]
        for cand in final[1:]:
            if (
                cand["score"] > baseline["score"]
                and cand["disq_rate"] < winner["disq_rate"]
                and cand["fitness"]["metrics_mean"]["mean_flying_lap"]
                <= winner["fitness"]["metrics_mean"]["mean_flying_lap"]
            ):
                winner = cand
                break
        if winner["score"] < baseline["score"]:
            winner = baseline
        print(
            f"  => [{modality}/{style}] WINNER score={winner['score']:.4f} (base={baseline['score']:.4f}, "
            f"gain={winner['mean_speed_gain_pct']:+.2f}%, disq={winner['disq_rate']*100:.1f}%) params={winner['params']}",
            flush=True,
        )
        results[style] = {
            "modality": modality,
            "style": style,
            "circuits": circuits,
            "elapsed_s": mod_elapsed,
            "history": st["history"],
            "final": final,
            "best": winner,
            "baseline": baseline,
        }
    return results


def run(args: argparse.Namespace) -> dict[str, Any]:
    args.out.mkdir(parents=True, exist_ok=True)
    all_circs = worker.all_circuits()
    modalities = [args.modality] if args.modality else list(worker.ALL_MODALITIES)
    styles = [args.style] if args.style else list(worker.STYLES)

    out_file = args.out / "stage_a.json"
    existing_modalities: dict[str, Any] = {}
    if out_file.exists():
        try:
            prev = json.loads(out_file.read_text())
            existing_modalities = prev.get("modalities", {})
        except Exception:
            existing_modalities = {}

    ledger = MatchLedger(args.out / "stage_a_ledger.sqlite")
    evaluator = BatchEvaluator(
        ledger=ledger,
        play_fn=harness.stage_a_play_fn,
        max_workers=args.workers,
        timeout_per_match=300.0,
    )
    started = time.perf_counter()
    try:
        for mod_idx, modality in enumerate(modalities):
            mod_circuits = [f"{m}/{cid}" for m, cid in all_circs if m == modality]
            if not mod_circuits:
                raise RuntimeError(f"No closed-loop circuits found for modality '{modality}'")
            mod_entry = existing_modalities.setdefault(
                modality, {"circuits": mod_circuits, "styles": {}}
            )
            mod_entry["circuits"] = mod_circuits
            style_results = tune_modality(
                modality,
                styles,
                mod_circuits,
                evaluator,
                ledger,
                args,
                mod_idx,
            )
            mod_entry["styles"].update(style_results)
            classic_bal = (
                existing_modalities.get("classic", {})
                .get("styles", {})
                .get("balanced", next(iter(style_results.values())))
            )
            checkpoint = {
                "evorl_commit": args.evorl_commit,
                "tdrace_commit": args.tdrace_commit,
                "seed": args.seed,
                "generations": args.generations,
                "population": args.population,
                "seeds_per_generation": args.seeds,
                "final_seeds": args.final_seeds,
                "total_circuits": len(all_circs),
                "elapsed_s": time.perf_counter() - started,
                "modalities": existing_modalities,
                "best": classic_bal["best"],
                "baseline": classic_bal["baseline"],
            }
            out_file.write_text(json.dumps(checkpoint, indent=2) + "\n")
    finally:
        evaluator.shutdown()

    return json.loads(out_file.read_text())


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--workers", type=int, default=28)
    parser.add_argument("--generations", type=int, default=7)
    parser.add_argument("--population", type=int, default=7)
    parser.add_argument(
        "--seeds", type=int, default=1, help="seed pairs per circuit per generation"
    )
    parser.add_argument(
        "--sigma0",
        type=float,
        default=0.28,
        help="initial step in the normalized [-1, 1] box",
    )
    parser.add_argument("--finalists", type=int, default=3)
    parser.add_argument("--final-seeds", type=int, default=2)
    parser.add_argument("--seed", type=int, default=83)
    parser.add_argument("--modality", default=None, help="optional single modality filter")
    parser.add_argument("--style", default=None, help="optional single style filter")
    parser.add_argument("--evorl-commit", default="11b9ac3")
    parser.add_argument("--tdrace-commit", default="5b52e86a")
    parser.add_argument(
        "--out",
        type=Path,
        default=Path(__file__).resolve().parent / "runs",
    )
    report = run(parser.parse_args())
    print(f"\nStage A complete in {report['elapsed_s']:.1f}s across {len(report['modalities'])} modalities.")


if __name__ == "__main__":
    main()
