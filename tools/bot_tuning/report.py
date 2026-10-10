"""Markdown tables for reports/bot_calibration_report.md from `run_campaign.py` JSON (spec 083).

See specs/083_evorl_bot_driving_style_and_performance_optimization.md.

    python tools/bot_tuning/report.py audit tools/bot_tuning/runs/audit_baseline.json
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

# Spec 083 Stage B pace bands: tier -> (centre, half-width) of the delta against T4.
BANDS = {1: (0.13, 0.03), 2: (0.07, 0.02), 3: (0.025, 0.01), 5: (-0.0225, 0.0075)}
STUCK_S = 10.0


def in_band(tier: int, delta: float | None) -> bool:
    if delta is None:
        return False
    centre, half = BANDS[tier]
    return abs(delta - centre) <= half


def pct(x: float | None) -> str:
    return "—" if x is None else f"{x * 100:+.1f}%"


def mark(tier: int, delta: float | None) -> str:
    return pct(delta) if in_band(tier, delta) else f"**{pct(delta)}**"


def raceable(c: dict[str, Any]) -> bool:
    return bool(c["closed_loop"]) and not c["errors"]


def audit_markdown(data: dict[str, Any]) -> str:
    circuits: dict[str, dict[str, Any]] = data["circuits"]
    lines: list[str] = []
    ok = {k: c for k, c in circuits.items() if raceable(c)}
    excluded = {k: c for k, c in circuits.items() if not raceable(c)}

    lines.append(
        f"{len(circuits)} circuits, {data['races']} races ({data['seeds']} seeds x 5 tiers per circuit), "
        f"{data['elapsed_s'] / 60:.1f} min on {data['host']}. "
        "Each race is a 3-lap grid of one bot per style at one tier. "
        "Bold deltas are outside the spec 083 band.\n"
    )

    lines.append("### Summary by module\n")
    lines.append(
        "| Module | Circuits | Tier order held (all 4 steps) | T1 in band | T2 in band | T3 in band "
        "| T5 in band | Circuits with a stuck bot |"
    )
    lines.append("|---|---|---|---|---|---|---|---|")
    for module in ("classic", "kart", "autocross", "rally"):
        cs = [c for k, c in ok.items() if k.startswith(module + "/")]
        n = len(cs)

        def count(pred: Any, cs: list[dict[str, Any]] = cs) -> int:
            return sum(1 for c in cs if pred(c))

        held = count(lambda c: all(c["order_held"]))
        bands = [
            count(lambda c, t=t: in_band(t, c["tiers"][str(t)]["delta_vs_t4"]))
            for t in (1, 2, 3, 5)
        ]
        stuck = count(lambda c: any(t["stuck"] for t in c["tiers"].values()))
        lines.append(
            f"| {module} | {n} | {held} | "
            + " | ".join(str(b) for b in bands)
            + f" | {stuck} |"
        )
    lines.append("")

    if excluded:
        lines.append("### Excluded circuits\n")
        lines.append("| Circuit | Reason |")
        lines.append("|---|---|")
        for k, c in sorted(excluded.items()):
            reason = (
                "; ".join(c["errors"]) if c["errors"] else "not a closed spline loop"
            )
            lines.append(f"| {k} | {reason} |")
        lines.append("")

    lines.append("### Per circuit\n")
    lines.append(
        "| Circuit | Cars | T1 | T2 | T3 | T5 | Order T1>T2>T3>T4>T5 | DNF (all tiers) | Stuck bots "
        "| Max no-progress (s) | T4 spins/lap | T4 wall contacts/lap |"
    )
    lines.append("|---|---|---|---|---|---|---|---|---|---|---|---|")
    for k, c in sorted(ok.items()):
        t = c["tiers"]
        order = "".join(
            "✓" if h else ("✗" if h is False else "?") for h in c["order_held"]
        )
        dnf = sum(v["dnf"] for v in t.values())
        stuck = sum(v["stuck"] for v in t.values())
        max_np = max(v["max_no_progress_s"] for v in t.values())
        lines.append(
            f"| {k} | {c['grid_cars']} | "
            + " | ".join(
                mark(tier, t[str(tier)]["delta_vs_t4"]) for tier in (1, 2, 3, 5)
            )
            + f" | {order} | {dnf} | {stuck} | {max_np:.1f} | {t['4']['spins_per_lap']:.3f} "
            f"| {t['4']['wall_contacts_per_lap']:.2f} |"
        )
    lines.append("")
    return "\n".join(lines)


def stuck_circuits(data: dict[str, Any]) -> list[tuple[str, int, float]]:
    """(circuit, stuck bots, longest no-progress) for every circuit with a stuck bot."""
    out = []
    for k, c in sorted(data["circuits"].items()):
        stuck = sum(v["stuck"] for v in c["tiers"].values())
        if stuck:
            out.append(
                (k, stuck, max(v["max_no_progress_s"] for v in c["tiers"].values()))
            )
    return out


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("kind", choices=["audit", "stuck"])
    parser.add_argument("json", type=Path)
    args = parser.parse_args()
    data = json.loads(args.json.read_text())
    if args.kind == "audit":
        print(audit_markdown(data))
    else:
        for k, n, s in stuck_circuits(data):
            print(f"{k}\t{n}\t{s:.1f}")


if __name__ == "__main__":
    main()
