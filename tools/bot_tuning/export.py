"""Exports calibrated Stage A (7 modalities x 6 styles) and Stage B (5 tiers) constants to Rust (spec 083).

See specs/083_evorl_bot_driving_style_and_performance_optimization.md.

Reads tools/bot_tuning/runs/stage_a.json and stage_b.json, prints the Rust code
for crates/race-kit/src/ai/mod.rs, and applies it with --apply.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
STAGE_A_PATH = REPO / "tools" / "bot_tuning" / "runs" / "stage_a.json"
STAGE_B_PATH = REPO / "tools" / "bot_tuning" / "runs" / "stage_b.json"
AI_MOD_PATH = REPO / "crates" / "race-kit" / "src" / "ai" / "mod.rs"

# (lookahead_time, steering_kp, steering_kd, brake_margin, aggression, avoidance_distance, speed_mult)
DEFAULT_STYLES: dict[str, tuple[float, float, float, float, float, float, float]] = {
    "Smooth": (0.36, 2.20, 0.09, 1.05, 0.70, 6.5, 1.03),
    "Aggressive": (0.28, 2.40, 0.06, 0.93, 0.95, 5.0, 1.04),
    "Tenacious": (0.38, 2.10, 0.09, 1.08, 0.82, 6.0, 1.01),
    "Calculating": (0.35, 2.30, 0.08, 1.03, 0.75, 6.5, 1.02),
    "Bold": (0.27, 2.60, 0.05, 0.91, 0.92, 5.2, 1.03),
    "Balanced": (0.34, 2.00, 0.08, 1.08, 0.65, 7.0, 1.00),
}

DEFAULT_TIERS: dict[int, dict[str, float]] = {
    1: {"pace_limit": 0.81, "brake_padding": 0.20, "consistency": 0.60, "composure": 0.48},
    2: {"pace_limit": 0.90, "brake_padding": 0.12, "consistency": 0.72, "composure": 0.60},
    3: {"pace_limit": 0.96, "brake_padding": 0.06, "consistency": 0.80, "composure": 0.76},
    4: {"pace_limit": 1.00, "brake_padding": 0.00, "consistency": 0.93, "composure": 0.90},
    5: {"pace_limit": 1.035, "brake_padding": -0.03, "consistency": 0.99, "composure": 0.98},
}

MODALITY_ARMS: list[tuple[str, str]] = [
    ("kart", 'b"kart" | b"karting"'),
    ("autocross", 'b"autocross" | b"ax"'),
    ("rally", 'b"rally" | b"rallycross" | b"rx"'),
    ("gt", 'b"gt" | b"gt_challenge" | b"gt_world_challenge"'),
    ("nascar", 'b"nascar" | b"stock_car" | b"trans_am"'),
    ("extreme_offroad", 'b"extreme_offroad" | b"off_road" | b"offroad"'),
    ("classic", "_"),
]


def _fmt_f(val: float) -> str:
    s = f"{val:.4f}".rstrip("0").rstrip(".")
    if "." not in s:
        s += ".0"
    return s


def load_modality_styles(stage_a_path: Path) -> dict[str, dict[str, tuple[float, ...]]]:
    out: dict[str, dict[str, tuple[float, ...]]] = {
        m: dict(DEFAULT_STYLES) for m, _ in MODALITY_ARMS
    }
    if not stage_a_path.exists():
        return out
    data = json.loads(stage_a_path.read_text())
    modalities_data = data.get("modalities", {})
    for mod_key, _ in MODALITY_ARMS:
        mod_entry = modalities_data.get(mod_key, {})
        styles_entry = mod_entry.get("styles", {})
        for style_name, base_tuple in DEFAULT_STYLES.items():
            s_lower = style_name.lower()
            if s_lower in styles_entry:
                sd = styles_entry[s_lower]
                chosen = sd["best"]
                base_score = sd.get("baseline", {}).get("score", float("-inf"))
                for cand in sd.get("final", []):
                    if (
                        cand.get("score", float("-inf")) > base_score
                        and cand.get("disq_rate", 1.0) < chosen.get("disq_rate", 1.0)
                        and cand.get("fitness", {}).get("metrics_mean", {}).get("mean_flying_lap", float("inf"))
                        <= chosen.get("fitness", {}).get("metrics_mean", {}).get("mean_flying_lap", float("inf"))
                    ):
                        chosen = cand
                        break
                p = chosen["params"]
                _, _, _, _, agg, avoid, _ = base_tuple
                out[mod_key][style_name] = (
                    round(float(p["lookahead_time"]), 4),
                    round(float(p["steering_kp"]), 4),
                    round(float(p["steering_kd"]), 4),
                    round(float(p["brake_margin"]), 4),
                    agg,
                    avoid,
                    round(float(p["speed_mult"]), 4),
                )
    return out


def load_tier_qualities(stage_b_path: Path) -> dict[int, dict[str, float]]:
    out = {t: dict(q) for t, q in DEFAULT_TIERS.items()}
    if not stage_b_path.exists():
        return out
    data = json.loads(stage_b_path.read_text())
    tiers_data: dict[str, Any] = data.get("tiers", {})
    for t in (1, 2, 3, 5):
        key = f"tier_{t}"
        if key in tiers_data:
            td = tiers_data[key]
            chosen = td["best"]
            base_score = td.get("baseline", {}).get("score", float("-inf"))
            for cand in td.get("final", []):
                if (
                    cand.get("score", float("-inf")) >= base_score
                    and cand.get("metrics", {}).get("disq_rate", 1.0)
                    < chosen.get("metrics", {}).get("disq_rate", 1.0)
                    and cand.get("metrics", {}).get("circuits_in_band", 0)
                    >= chosen.get("metrics", {}).get("circuits_in_band", 0)
                ):
                    chosen = cand
                    break
            p = chosen["params"]
            out[t] = {
                "pace_limit": round(float(p["pace_limit"]), 4),
                "brake_padding": round(float(p["brake_padding"]), 4),
                "consistency": round(float(p["consistency"]), 4),
                "composure": round(float(p["composure"]), 4),
            }
    return out


def format_rust_style_tuning(
    modality_styles: dict[str, dict[str, tuple[float, ...]]]
) -> str:
    lines = [
        "impl StyleTuning {",
        "    #[allow(clippy::approx_constant)]",
        "    pub const fn for_modality_and_style(modality: &str, style: DrivingStyle) -> Self {",
        "        let (lookahead_time, steering_kp, steering_kd, brake_margin, aggression, avoidance_distance, speed_mult) =",
        "            match modality.as_bytes() {",
    ]
    for mod_key, arm_pattern in MODALITY_ARMS:
        lines.append(f"                {arm_pattern} => match style {{")
        for style_name, vals_tuple in modality_styles[mod_key].items():
            vals = ", ".join(_fmt_f(v) for v in vals_tuple)
            lines.append(f"                    DrivingStyle::{style_name} => ({vals}),")
        lines.append("                },")
    lines.extend(
        [
            "            };",
            "        Self { lookahead_time, steering_kp, steering_kd, brake_margin, aggression, avoidance_distance, speed_mult }",
            "    }",
            "",
            "    pub const fn for_style(style: DrivingStyle) -> Self {",
            '        Self::for_modality_and_style("classic", style)',
            "    }",
            "}",
        ]
    )
    return "\n".join(lines)


def format_rust_driver_quality(tier_qualities: dict[int, dict[str, float]]) -> str:
    lines = [
        "impl DriverQuality {",
        "    #[allow(clippy::approx_constant)]",
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
        lines.extend(
            [
                f"            DriverTier::{t_name} => Self {{",
                "                tier,",
                f"                pace_limit: {_fmt_f(q['pace_limit'])},",
                f"                brake_padding: {_fmt_f(q['brake_padding'])},",
                f"                avoidance_padding: {_fmt_f(avoid)},",
                f"                consistency: {_fmt_f(q['consistency'])},",
                f"                composure: {_fmt_f(q['composure'])},",
            ]
        )
        lines.append("            },")
    lines.extend(
        [
            "        }",
            "    }",
            "}",
        ]
    )
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--stage-a", type=Path, default=STAGE_A_PATH, help="Path to stage_a.json"
    )
    parser.add_argument(
        "--stage-b", type=Path, default=STAGE_B_PATH, help="Path to stage_b.json"
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="Apply constants into crates/race-kit/src/ai/mod.rs",
    )
    args = parser.parse_args()

    modality_styles = load_modality_styles(args.stage_a)
    tier_qualities = load_tier_qualities(args.stage_b)

    rust_style = format_rust_style_tuning(modality_styles)
    rust_quality = format_rust_driver_quality(tier_qualities)

    print("=== Rust StyleTuning::for_modality_and_style ===\n")
    print(rust_style)
    print("\n=== Rust DriverQuality::for_tier ===\n")
    print(rust_quality)

    if args.apply:
        content = AI_MOD_PATH.read_text()
        start_st = content.find("impl StyleTuning {")
        end_st = content.find("}\n\n/// Personality", start_st) + 1
        assert start_st != -1 and end_st != 0, "Could not find StyleTuning block in mod.rs"
        content = content[:start_st] + rust_style + content[end_st:]

        start_dq = content.find("impl DriverQuality {")
        end_dq = content.find("}\n\n/// Base settings", start_dq) + 1
        assert start_dq != -1 and end_dq != 0, "Could not find DriverQuality block in mod.rs"
        content = content[:start_dq] + rust_quality + content[end_dq:]

        AI_MOD_PATH.write_text(content)
        print(f"\nApplied updated constants to {AI_MOD_PATH}!")


if __name__ == "__main__":
    main()
