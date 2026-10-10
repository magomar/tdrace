"""EvoRL play function for spec 083 bot calibration.

See specs/083_evorl_bot_driving_style_and_performance_optimization.md.

EvoRL calls `stage_a_play_fn(subject, opponent, seed, seat, params_str)` in pool worker processes.
- `subject` is "candidate@<genome key>"; the genome is `json.loads(params_str)["params"]`.
- `opponent` is the circuit, "module/id". The opponent car is always the current T4 Balanced bot.
- `seat` is the subject's grid slot (0 = pole, 1 = second). Both seats use the same seed (common
  random numbers); `bot_eval_worker` keeps each driver's random stream across the swap.
"""

from __future__ import annotations

import json
from typing import Any

import worker
from evorl.core.ledger import Outcome

LAPS = 3
# Spec 083 Stage A disqualification: no-progress limit of classic_circuits_bot_tests.
MAX_NO_PROGRESS_S = 10.0
MAX_WALL_CONTACTS = 3


def stage_a_request(
    circuit: str, seed: int, seat: int, genome: dict[str, float]
) -> dict[str, Any]:
    t4 = {"style": "balanced", "tier": 4}
    return {
        "circuit": circuit,
        "laps": LAPS,
        "seed": seed,
        "subject": {**t4, "genome": genome},
        "opponents": [t4],
        "subject_slot": seat,
    }


def stage_a_outcome(result: dict[str, Any]) -> Outcome:
    """WIN/DRAW/LOSS by finish order; a DNF or more than 3 wall contacts is a LOSS."""
    if result["dnf"] or result["wall_contacts"] > MAX_WALL_CONTACTS:
        return Outcome.LOSS
    opponent_rank = result["opponents"][0]["rank"]
    if result["subject_rank"] < opponent_rank:
        return Outcome.WIN
    if result["subject_rank"] == opponent_rank:
        return Outcome.DRAW
    return Outcome.LOSS


def stage_a_play_fn(
    subject: str, opponent: str, seed: int, seat: int, params_str: str | None
) -> tuple[Outcome, int, str | None, float | None, dict[str, Any] | None]:
    try:
        genome = json.loads(params_str)["params"] if params_str else {}
        result = worker.race(stage_a_request(opponent, seed, seat, genome))
        if result["error"]:
            return Outcome.ERROR, 0, result["error"], None, {"error": result["error"]}
        disqualified = (
            result["spins"] > 0 or result["longest_no_progress_s"] > MAX_NO_PROGRESS_S
        )
        metrics = {
            "mean_flying_lap": result["mean_flying_lap"],
            "opponent_mean_flying_lap": result["opponents"][0]["mean_flying_lap"],
            "spins": result["spins"],
            "offs": result["offs"],
            "mistakes": result["mistakes"],
            "wall_contacts": result["wall_contacts"],
            "longest_no_progress_s": result["longest_no_progress_s"],
            "dnf": int(result["dnf"]),
            "disqualified": int(disqualified),
            "controls_hash": str(result["controls_hash"]),
        }
        score = None
        if result["mean_flying_lap"] and result["opponents"][0]["mean_flying_lap"]:
            # Positive when the subject is faster than the baseline bot.
            score = (
                result["opponents"][0]["mean_flying_lap"] / result["mean_flying_lap"]
                - 1.0
            )
        return stage_a_outcome(result), result["steps"], None, score, metrics
    except Exception as exc:  # noqa: BLE001 - a play function must return ERROR, never raise
        return Outcome.ERROR, 0, str(exc), None, {"error": str(exc)}
