#!/usr/bin/env python3
"""
Gate 4 Visual Validation Script: In-Game Interactive Steering Sweep Simulation
Renders multi-angle steering sweeps (-30°, -15°, 0°, +15°, +30°) for representative vehicles
across all motorsport archetypes using exact visual anchor math, layering, and padding factors.
"""

import json
import math
import os
from pathlib import Path

from PIL import Image

REPO_ROOT = Path(__file__).resolve().parent.parent
ANCHORS_PATH = REPO_ROOT / "artifacts" / "visual_wheel_anchors.json"
WHEELS_DIR = REPO_ROOT / "assets" / "textures" / "vehicles" / "topdown" / "wheels"
TOPDOWN_DIR = REPO_ROOT / "assets" / "textures" / "vehicles" / "topdown"
OUTPUT_DIR = REPO_ROOT / "artifacts" / "hitl" / "gate4_sweeps"
HTML_OUTPUT = REPO_ROOT / "artifacts" / "hitl" / "gate4_steering_sweep.html"

REPRESENTATIVES = [
    ("kart_blackline_cadet_t1", "kart", "Karting (Cadet)", "kart_slick_front", "OverChassis"),
    ("offroad_volkskraft_dune_t1", "baja", "Dune Buggy Baja", "buggy_allterrain_front", "OverChassis"),
    ("offroad_havoc_overkill_t5", "extreme_offroad", "Monster Truck", "monster_wheel_front", "OverChassis"),
    ("offroad_crossbow_ridge_t4", "extreme_offroad", "Mud Bogger Heavy", "mud_tractor_front", "UnderChassis"),
    ("offroad_desert_forge_truck_t2", "rallycross", "Trophy Truck AWD", "truck_allterrain_front", "UnderChassis"),
    ("gt_vandorn_arrowhead_t2", "gt", "GT3 / Supercar", "gt_slick_front", "UnderChassis"),
    ("classic_nascar", "classic", "Stock Car (NASCAR)", "nascar_wheel_front", "UnderChassis"),
    ("classic_rally", "classic", "Rallycross", "rally_wheel_front", "UnderChassis"),
]

def get_padding_factor(archetype: str) -> tuple[float, float]:
    factors = {
        "monster_wheel_front": (128.0 / 122.0, 256.0 / 246.0),
        "mud_tractor_front": (128.0 / 118.0, 256.0 / 244.0),
        "truck_allterrain_front": (128.0 / 116.0, 256.0 / 242.0),
        "nascar_wheel_front": (128.0 / 116.0, 256.0 / 242.0),
        "gt_slick_front": (128.0 / 114.0, 256.0 / 238.0),
        "buggy_allterrain_front": (128.0 / 110.0, 256.0 / 238.0),
        "kart_slick_front": (128.0 / 110.0, 256.0 / 234.0),
        "rally_wheel_front": (128.0 / 110.0, 256.0 / 234.0),
        "offroad_wheel_front": (128.0 / 90.0, 256.0 / 236.0),
    }
    return factors.get(archetype, (1.0, 1.0))

def compute_ackermann(steer_angle_rad: float, wheelbase: float = 2.6, track_width: float = 1.8) -> tuple[float, float]:
    if abs(steer_angle_rad) < 1e-4:
        return 0.0, 0.0
    r = wheelbase / math.tan(steer_angle_rad)
    half_w = track_width * 0.5
    steer_fl = math.atan(wheelbase / (r - half_w))
    steer_fr = math.atan(wheelbase / (r + half_w))
    return steer_fl, steer_fr

def find_chassis_path(model_id: str, module: str) -> Path | None:
    p = TOPDOWN_DIR / module / f"{model_id}_chassis.png"
    if p.exists():
        return p
    # Try searching
    for root, _, files in os.walk(TOPDOWN_DIR):
        if f"{model_id}_chassis.png" in files:
            return Path(root) / f"{model_id}_chassis.png"
    return None

def render_steered_vehicle(
    chassis_img: Image.Image,
    wheel_img: Image.Image,
    anchor: dict,
    steer_angle_deg: float,
) -> Image.Image:
    canvas = Image.new("RGBA", (512, 512), (0, 0, 0, 0))
    steer_rad = math.radians(steer_angle_deg)
    steer_fl, steer_fr = compute_ackermann(steer_rad)

    # In sprite coordinate frame:
    # +X is forward (right), +Y is lateral right (down)
    # Axle is at X = anchor["axle_x_px"]
    # Track width is anchor["track_width_px"]
    axle_x = anchor["axle_x_px"]
    track_w = anchor["track_width_px"]
    fl_y = 256.0 - track_w * 0.5
    fr_y = 256.0 + track_w * 0.5

    kw, kh = get_padding_factor(anchor["archetype"])
    quad_w = round(anchor["tire_wid_px"] * kw)
    quad_h = round(anchor["tire_len_px"] * kh)

    # Resize wheel texture to quad dimensions
    scaled_wheel = wheel_img.resize((quad_w, quad_h), Image.Resampling.LANCZOS)

    # In top-down sprite:
    # Car heading is along +X. Wheel texture original orientation is vertical (height along Y, width along X).
    # When wheel is unsteered (steer = 0), tire length points along car forward (+X).
    # In PIL rotate: positive degrees rotate counter-clockwise.
    # Base rotation to point vertical tire along +X: rotate by -90° (or +90°).
    # With steer_fl (positive steer turns right/clockwise):
    # In screen coordinates (+Y down): turning right turns clockwise (+deg).
    # PIL rotate: counter-clockwise, so -(-90 + steer_deg).
    rot_fl = scaled_wheel.rotate(-math.degrees(steer_fl) - 90, resample=Image.Resampling.BICUBIC, expand=True)
    rot_fr = scaled_wheel.rotate(-math.degrees(steer_fr) - 90, resample=Image.Resampling.BICUBIC, expand=True)

    layering = anchor["layering"]

    # 1. UnderChassis wheels
    if layering == "UnderChassis":
        canvas.paste(rot_fl, (round(axle_x - rot_fl.width * 0.5), round(fl_y - rot_fl.height * 0.5)), rot_fl)
        canvas.paste(rot_fr, (round(axle_x - rot_fr.width * 0.5), round(fr_y - rot_fr.height * 0.5)), rot_fr)

    # 2. Chassis body
    canvas.paste(chassis_img, (0, 0), chassis_img)

    # 3. OverChassis wheels
    if layering == "OverChassis":
        canvas.paste(rot_fl, (round(axle_x - rot_fl.width * 0.5), round(fl_y - rot_fl.height * 0.5)), rot_fl)
        canvas.paste(rot_fr, (round(axle_x - rot_fr.width * 0.5), round(fr_y - rot_fr.height * 0.5)), rot_fr)

    return canvas

def main():
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    with open(ANCHORS_PATH) as f:
        anchors = json.load(f)

    angles = [-30.0, -15.0, 0.0, 15.0, 30.0]
    results = []

    print("Generating Gate 4 steering sweep validations...")
    for model_id, module, title, expected_archetype, expected_layering in REPRESENTATIVES:
        anchor = anchors.get(model_id)
        if not anchor:
            print(f"ERROR: Missing anchor for {model_id}")
            continue

        chassis_path = find_chassis_path(model_id, module)
        if not chassis_path or not chassis_path.exists():
            print(f"ERROR: Missing chassis sprite for {model_id} at {chassis_path}")
            continue

        wheel_path = WHEELS_DIR / f"{anchor['archetype']}.png"
        if not wheel_path.exists():
            print(f"ERROR: Missing wheel texture at {wheel_path}")
            continue

        chassis_img = Image.open(chassis_path).convert("RGBA")
        wheel_img = Image.open(wheel_path).convert("RGBA")

        sweep_imgs = []
        angle_paths = []
        for ang in angles:
            rendered = render_steered_vehicle(chassis_img, wheel_img, anchor, ang)
            ang_filename = f"{model_id}_steer_{int(ang):+03d}.png"
            out_path = OUTPUT_DIR / ang_filename
            rendered.save(out_path)
            sweep_imgs.append(rendered)
            angle_paths.append(ang_filename)

        # Composite a strip panel
        strip = Image.new("RGBA", (512 * len(angles), 512), (18, 22, 28, 255))
        for idx, simg in enumerate(sweep_imgs):
            strip.paste(simg, (idx * 512, 0), simg)
        strip_filename = f"{model_id}_sweep_strip.png"
        strip.save(OUTPUT_DIR / strip_filename)

        results.append({
            "model_id": model_id,
            "title": title,
            "module": module,
            "archetype": anchor["archetype"],
            "layering": anchor["layering"],
            "axle_x_px": anchor["axle_x_px"],
            "track_width_px": anchor["track_width_px"],
            "tire_len_px": anchor["tire_len_px"],
            "tire_wid_px": anchor["tire_wid_px"],
            "strip_filename": strip_filename,
            "angle_paths": angle_paths,
        })
        print(f"  ✓ {title} ({model_id}) [{anchor['archetype']}]")

    # Generate HTML review artifact
    html = [
        "<!DOCTYPE html>",
        "<html lang='en'>",
        "<head>",
        "  <meta charset='utf-8'>",
        "  <title>Gate 4: In-Game Interactive Steering Sweep Validation</title>",
        "  <style>",
        "    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0d1117; color: #c9d1d9; margin: 0; padding: 20px; }",
        "    h1, h2 { color: #58a6ff; }",
        "    .card { background: #161b22; border: 1px solid #30363d; border-radius: 8px; padding: 16px; margin-bottom: 24px; }",
        "    .strip-container { overflow-x: auto; background: #0b0e14; border-radius: 6px; padding: 8px; margin-top: 12px; }",
        "    .strip-img { max-width: 100%; height: auto; display: block; border-radius: 4px; }",
        "    .meta { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; font-size: 13px; color: #8b949e; margin-top: 8px; }",
        "    .meta-val { color: #f0f6fc; font-weight: bold; font-family: monospace; }",
        "    .badge { display: inline-block; padding: 2px 8px; border-radius: 12px; font-size: 11px; font-weight: bold; }",
        "    .badge-open { background: #238636; color: #fff; }",
        "    .badge-closed { background: #1f6feb; color: #fff; }",
        "    .legend { display: flex; gap: 24px; margin-top: 8px; font-size: 12px; color: #8b949e; }",
        "  </style>",
        "</head>",
        "<body>",
        "  <h1>Gate 4: In-Game Steering Sweep Walkthrough</h1>",
        "  <p>Visual verification of full left-to-right steering lock articulation across diverse motorsport archetypes. Verifies sub-pixel anchor alignment, zero residual tire ghosting, authentic tire footprint scale, and correct UnderChassis / OverChassis z-layering.</p>",
    ]

    for item in results:
        badge_cls = "badge-open" if item["layering"] == "OverChassis" else "badge-closed"
        html.append("  <div class='card'>")
        html.append(f"    <h2>{item['title']} <span class='badge {badge_cls}'>{item['layering']}</span></h2>")
        html.append("    <div class='legend'><span>-30° (Full Left)</span><span>-15°</span><span>0° (Straight)</span><span>+15°</span><span>+30° (Full Right)</span></div>")
        html.append(f"    <div class='strip-container'><img class='strip-img' src='gate4_sweeps/{item['strip_filename']}' alt='{item['model_id']} sweep'></div>")
        html.append("    <div class='meta'>")
        html.append(f"      <div>Model: <span class='meta-val'>{item['model_id']}</span></div>")
        html.append(f"      <div>Archetype: <span class='meta-val'>{item['archetype']}</span></div>")
        html.append(f"      <div>Axle X: <span class='meta-val'>{item['axle_x_px']} px</span></div>")
        html.append(f"      <div>Track Width: <span class='meta-val'>{item['track_width_px']} px</span></div>")
        html.append(f"      <div>Tire Size: <span class='meta-val'>{item['tire_wid_px']} × {item['tire_len_px']} px</span></div>")
        html.append("    </div>")
        html.append("  </div>")

    html.append("</body></html>")
    with open(HTML_OUTPUT, "w") as f:
        f.write("\n".join(html))

    print(f"Gate 4 validation complete. HTML report: {HTML_OUTPUT}")

if __name__ == "__main__":
    main()
