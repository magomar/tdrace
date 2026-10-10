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

# Spec 083 training set: (module, circuit id).
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
    ("rally", "lydden_hill"),
    ("rally", "montalegre_rx"),
]

LAUNCH_MODULES = ("classic", "kart", "autocross", "rally")


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
