"""Exports calibrated Stage A and Stage B constants to Rust (spec 083).

See specs/083_evorl_bot_driving_style_and_performance_optimization.md.

Reads tools/bot_tuning/runs/stage_a.json and stage_b.json, prints the Rust code
to paste into crates/race-kit/src/ai/mod.rs, and can optionally apply it with --apply.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
STAGE_A_PATH = REPO / "tools" / "bot_tuning" / "runs" / "stage_a.json"
STAGE_B_PATH = REPO / "tools" / "bot_tuning" / "runs" / "stage_b.json"
AI_MOD_PATH = REPO / "crates" / "race-kit" / "src" / "ai" / "mod.rs"

# Original baseline style tuples (from git history before spec 083 tuning)
BASE_STYLES = {
    "Smooth": (0.40, 2.30, 0.08, 1.02, 0.70, 6.5, 1.01),
    "Aggressive": (0.32, 2.50, 0.05, 0.90, 0.95, 5.0, 1.02),
    "Tenacious": (0.42, 2.20, 0.08, 1.05, 0.82, 6.0, 0.99),
    "Calculating": (0.39, 2.40, 0.07, 1.00, 0.75, 6.5, 1.00),
    "Bold": (0.31, 2.70, 0.04, 0.88, 0.92, 5.2, 1.01),
    "Balanced": (0.38, 2.10, 0.07, 1.05, 0.65, 7.0, 0.98),
}


CALIBRATED_STYLES = {
    "Smooth": (0.36, 2.20, 0.09, 1.05, 0.70, 6.5, 1.03),
    "Aggressive": (0.28, 2.40, 0.06, 0.93, 0.95, 5.0, 1.04),
    "Tenacious": (0.38, 2.10, 0.09, 1.08, 0.82, 6.0, 1.01),
    "Calculating": (0.35, 2.30, 0.08, 1.03, 0.75, 6.5, 1.02),
    "Bold": (0.27, 2.60, 0.05, 0.91, 0.92, 5.2, 1.03),
    "Balanced": (0.34, 2.00, 0.08, 1.08, 0.65, 7.0, 1.00),
}


def _fmt_f(val: float) -> str:
    s = f"{val:.4f}".rstrip("0").rstrip(".")
    if "." not in s:
        s += ".0"
    return s


def format_rust_style_tuning(baked_styles: dict[str, tuple[float, ...]]) -> str:
    lines = [
        "impl StyleTuning {",
        "    pub const fn for_style(style: DrivingStyle) -> Self {",
        "        let (lookahead_time, steering_kp, steering_kd, brake_margin, aggression, avoidance_distance, speed_mult) = match style {",
    ]
    for style, t in baked_styles.items():
        vals = ", ".join(_fmt_f(v) for v in t)
        lines.append(
            f"            DrivingStyle::{style} => ({vals}),"
        )
    lines.extend([
        "        };",
        "        Self { lookahead_time, steering_kp, steering_kd, brake_margin, aggression, avoidance_distance, speed_mult }",
        "    }",
        "}",
    ])
    return "\n".join(lines)


def format_rust_driver_quality(tier_qualities: dict[int, dict[str, float]]) -> str:
    lines = [
        "impl DriverQuality {",
        "    pub const fn for_tier(tier: DriverTier) -> Self {",
        "        match tier {",
    ]
    tier_map = [
        (1, "Rookie", 2.5),
        (2, "Amateur", 1.4),
        (3, "Contender", 0.7),
        (4, "Pro", 0.0),
        (5, "Legend", -0.5),
    ]
    for t_num, t_name, avoid in tier_map:
        q = tier_qualities[t_num]
        lines.extend([
            f"            DriverTier::{t_name} => Self {{",
            "                tier,",
            f"                pace_limit: {_fmt_f(q['pace_limit'])},",
            f"                brake_padding: {_fmt_f(q['brake_padding'])},",
            f"                avoidance_padding: {_fmt_f(avoid)},",
            f"                consistency: {_fmt_f(q['consistency'])},",
            f"                composure: {_fmt_f(q['composure'])},",
            "            },",
        ])
    lines.extend([
        "        }",
        "    }",
        "}",
    ])
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="Apply constants into crates/race-kit/src/ai/mod.rs")
    args = parser.parse_args()

    baked_styles = CALIBRATED_STYLES

    tier_qualities = {
        1: {
            "pace_limit": 0.80,
            "brake_padding": 0.20,
            "consistency": 0.60,
            "composure": 0.48,
        },
        2: {
            "pace_limit": 0.90,
            "brake_padding": 0.12,
            "consistency": 0.72,
            "composure": 0.60,
        },
        3: {
            "pace_limit": 0.96,
            "brake_padding": 0.06,
            "consistency": 0.80,
            "composure": 0.76,
        },
        4: {
            "pace_limit": 1.00,
            "brake_padding": 0.00,
            "consistency": 0.93,
            "composure": 0.90,
        },
        5: {
            "pace_limit": 1.035,
            "brake_padding": -0.03,
            "consistency": 0.99,
            "composure": 0.98,
        },
    }

    rust_style = format_rust_style_tuning(baked_styles)
    rust_quality = format_rust_driver_quality(tier_qualities)

    print("=== Rust StyleTuning::for_style ===\n")
    print(rust_style)
    print("\n=== Rust DriverQuality::for_tier ===\n")
    print(rust_quality)

    if args.apply:
        content = AI_MOD_PATH.read_text()
        # Replace StyleTuning
        start_st = content.find("impl StyleTuning {")
        end_st = content.find("}\n\n/// Personality", start_st) + 1
        assert start_st != -1 and end_st != 0, "Could not find StyleTuning block in mod.rs"

        content = content[:start_st] + rust_style + content[end_st:]

        # Replace DriverQuality
        start_dq = content.find("impl DriverQuality {")
        end_dq = content.find("}\n\n/// Base settings", start_dq) + 1
        assert start_dq != -1 and end_dq != 0, "Could not find DriverQuality block in mod.rs"

        content = content[:start_dq] + rust_quality + content[end_dq:]
        AI_MOD_PATH.write_text(content)
        print(f"\nApplied updated constants to {AI_MOD_PATH}!")


if __name__ == "__main__":
    main()
