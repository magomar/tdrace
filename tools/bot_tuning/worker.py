"""Client for the `bot_eval_worker` Rust binary (spec 083).

See specs/083_evorl_bot_driving_style_and_performance_optimization.md.

One `Worker` owns one `bot_eval_worker` child process and sends it one JSON race request per
line. Each Python process keeps a single worker for its whole life (`shared_worker`).
"""

from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]

# Legacy 13-circuit launch subset kept for backwards-compatible smoke checks.
TRAINING_CIRCUITS: list[tuple[str, str]] = [
    ("classic", "gt_coastal_grand_prix"),
    ("classic", "stock_tri_oval_speedway"),
    ("classic", "gt_ridge_ring"),
    ("classic", "kart_pine_grove"),
    ("kart", "lonato"),
    ("kart", "genk"),
    ("kart", "whilton_mill"),
    ("autocross", "maggiora_ax"),
    ("autocross", "nova_paka_ax"),
    ("autocross", "st_junien_ax"),
    ("rally", "holjes_rx"),
    ("rally", "mettet_rx"),
    ("rally", "montalegre_rx"),
]

LAUNCH_MODULES = ("classic", "kart", "autocross", "rally")
ALL_MODALITIES = (
    "classic",
    "kart",
    "autocross",
    "rally",
    "gt",
    "nascar",
    "extreme_offroad",
)
STYLES = ("smooth", "aggressive", "tenacious", "calculating", "bold", "balanced")


def worker_binary() -> Path:
    """`BOT_EVAL_WORKER` if set, else the release build in this checkout."""
    path = Path(
        os.environ.get(
            "BOT_EVAL_WORKER", REPO / "target" / "release" / "bot_eval_worker"
        )
    )
    if not path.exists():
        raise FileNotFoundError(
            f"{path} not found; run `cargo build --release -p tdrace-app --bin bot_eval_worker`"
        )
    return path


def launch_circuits() -> list[tuple[str, str]]:
    """(module, circuit id) of every launch circuit (75 circuits), as `bot_eval_worker` lists them."""
    out = subprocess.run(
        [str(worker_binary()), "--list-launch-circuits"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    return [(c["module"], c["id"]) for c in json.loads(out)]


def all_circuits() -> list[tuple[str, str]]:
    """(module, circuit id) of every closed-loop circuit across all 7 modalities (127 circuits)."""
    out = subprocess.run(
        [str(worker_binary()), "--list-all-circuits"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    return [(c["module"], c["id"]) for c in json.loads(out)]


def modality_circuits(modality: str) -> list[tuple[str, str]]:
    """All closed-loop (module, circuit id) pairs belonging to `modality`."""
    return [(m, cid) for m, cid in all_circuits() if m == modality]


class Worker:
    """A running `bot_eval_worker`."""

    def __init__(self, binary: Path | None = None) -> None:
        self.proc = subprocess.Popen(
            [str(binary or worker_binary())],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
            bufsize=1,
        )

    def race(self, request: dict[str, Any]) -> dict[str, Any]:
        """Runs one race. A result with `error` set means the request did not run."""
        assert self.proc.stdin and self.proc.stdout
        self.proc.stdin.write(json.dumps(request, allow_nan=False) + "\n")
        self.proc.stdin.flush()
        line = self.proc.stdout.readline()
        if not line:
            raise RuntimeError(f"bot_eval_worker exited with code {self.proc.wait()}")
        return json.loads(line)

    def close(self) -> None:
        if self.proc.stdin:
            self.proc.stdin.close()
        self.proc.wait(timeout=10)


_SHARED: Worker | None = None


def shared_worker() -> Worker:
    """The worker of this process, started on first use."""
    global _SHARED
    if _SHARED is None or _SHARED.proc.poll() is not None:
        _SHARED = Worker()
    return _SHARED


def race(request: dict[str, Any]) -> dict[str, Any]:
    """Runs one race on this process's shared worker."""
    return shared_worker().race(request)
