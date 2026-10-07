#!/usr/bin/env python3
# /// script
# dependencies = ["pillow", "numpy"]
# ///
"""
Multi-View Sprite Wheel Anchor Extraction & Diagnostic Overlay Pipeline (Spec 095).

Fuses Cenital Top-Down (512x512) and Lateral Side-Profile (1024x512) computer vision analysis:
1. Detects authentic wheel hubs, wheel diameters, and axle positions on both views.
2. Cross-calibrates front axle X and track width without relying on synthetic platform overhangs.
3. Classifies vehicles into 8 distinct motorsport tyre archetypes.
4. Produces visual diagnostic overlays for Human-in-the-Loop (HITL) Gate 1 review.
5. Generates `artifacts/visual_wheel_anchors.json` and `artifacts/hitl/gate1_anchor_review.html`.
"""

import argparse
import json
from pathlib import Path
import sys
import numpy as np
from PIL import Image, ImageDraw, ImageFont

# Fallbacks for bonus / vault archive vehicles not tracked in codex cars.json
FALLBACK_VEHICLES = {
    "vault_asahi_blade_runner": {"base_car": "Kart", "module": "kart", "archetype": "kart_slick_front", "layering": "OverChassis"},
    "vault_greenfield_prairie_racer": {"base_car": "Kart", "module": "kart", "archetype": "kart_slick_front", "layering": "OverChassis"},
    "vault_nordic_valhalla_tractor": {"base_car": "Kart", "module": "kart", "archetype": "kart_slick_front", "layering": "OverChassis"},
    "classic_ax_talon": {"base_car": "CrossCar", "module": "classic", "archetype": "buggy_allterrain_front", "layering": "OverChassis"},
    "classic_kart": {"base_car": "Kart", "module": "classic", "archetype": "kart_slick_front", "layering": "OverChassis"},
    "classic_kart_vintage": {"base_car": "Kart", "module": "classic", "archetype": "kart_slick_front", "layering": "OverChassis"},
    "classic_gt": {"base_car": "GT3Car", "module": "classic", "archetype": "gt_slick_front", "layering": "UnderChassis"},
    "classic_gt_vintage": {"base_car": "GT4Clubsport", "module": "classic", "archetype": "gt_slick_front", "layering": "UnderChassis"},
    "classic_nascar": {"base_car": "StockCar", "module": "classic", "archetype": "nascar_wheel_front", "layering": "UnderChassis"},
    "classic_stock_vintage": {"base_car": "StockCar", "module": "classic", "archetype": "nascar_wheel_front", "layering": "UnderChassis"},
    "classic_rally": {"base_car": "RallyCar", "module": "classic", "archetype": "rally_wheel_front", "layering": "UnderChassis"},
    "classic_rx_vintage": {"base_car": "RallyJuniorFWD", "module": "classic", "archetype": "rally_wheel_front", "layering": "UnderChassis"},
    "classic_ax_mudlark": {"base_car": "SuperBuggy", "module": "classic", "archetype": "buggy_allterrain_front", "layering": "OverChassis"},
    "classic_ax_brawler": {"base_car": "TouringAX", "module": "classic", "archetype": "rally_wheel_front", "layering": "UnderChassis"},
    "classic_offroad": {"base_car": "DuneBuggyBaja", "module": "classic", "archetype": "buggy_allterrain_front", "layering": "OverChassis"},
    "classic_at_safari": {"base_car": "TrophyTruckAWD", "module": "classic", "archetype": "truck_allterrain_front", "layering": "UnderChassis"},
}

OPEN_WHEEL_BASE_CARS = {
    "Kart",
    "SuperkartGP",
    "CrossCar",
    "SuperBuggy",
    "DuneBuggyBaja",
    "SandRail",
    "MonsterTruck",
    "MudBoggerHeavy",
}


def load_catalog(root: Path):
    cars_json_path = root / "portals" / "shared" / "data" / "codex" / "cars.json"
    catalog = {}
    if cars_json_path.exists():
        with open(cars_json_path, "r", encoding="utf-8") as f:
            data = json.load(f)
            for item in data.get("items", []):
                catalog[item["id"]] = item

    for fallback_id, fallback_data in FALLBACK_VEHICLES.items():
        if fallback_id not in catalog:
            catalog[fallback_id] = {
                "id": fallback_id,
                "base_car": fallback_data["base_car"],
                "module": fallback_data["module"],
                "images": {
                    "topdown": f"/textures/vehicles/topdown/{fallback_data['module']}/{fallback_id}.png",
                    "lateral": f"/textures/vehicles/laterals/{fallback_data['module']}/{fallback_id}.png",
                },
            }
    return catalog


def analyze_lateral_sprite(im: Image.Image):
    """Detects vehicle bounding box, ground baseline, front/rear hubs, and wheel radius from lateral view."""
    arr = np.array(im.convert("RGBA"))
    alpha = arr[:, :, 3]
    ys, xs = np.where(alpha > 20)
    if len(xs) == 0:
        return None

    x_min, x_max = int(xs.min()), int(xs.max())
    y_min, y_max = int(ys.min()), int(ys.max())
    w_px = x_max - x_min + 1
    h_px = y_max - y_min + 1

    # 1. Front Wheel Contact Patch & Hub
    pts_f = []
    for x in range(int(x_min + 0.58 * w_px), int(x_max - 0.03 * w_px)):
        y_col = np.where(alpha[:, x] > 50)[0]
        if len(y_col):
            pts_f.append((x, y_col.max()))
    
    if len(pts_f) == 0:
        return None
    pts_f = np.array(pts_f)
    max_yf = pts_f[:, 1].max()
    hub_front_x = float(pts_f[pts_f[:, 1] >= max_yf - 3, 0].mean())

    # 2. Rear Wheel Contact Patch & Hub
    pts_r = []
    for x in range(int(x_min + 0.04 * w_px), int(x_min + 0.48 * w_px)):
        y_col = np.where(alpha[:, x] > 50)[0]
        if len(y_col):
            pts_r.append((x, y_col.max()))
    
    if len(pts_r) == 0:
        return None
    pts_r = np.array(pts_r)
    max_yr = pts_r[:, 1].max()
    hub_rear_x = float(pts_r[pts_r[:, 1] >= max_yr - 3, 0].mean())

    ground_y = max(max_yf, max_yr)

    # 3. Detect Wheel Radius (Circular annular fit)
    best_r_front = 45.0
    for r in range(25, min(140, int(h_px * 0.65))):
        angles = np.linspace(0, np.pi, 20)
        c_xs = np.clip(np.round(hub_front_x + r * np.cos(angles)).astype(int), 0, 1023)
        c_ys = np.clip(np.round((ground_y - r) - r * np.sin(angles)).astype(int), 0, 511)
        valid = (alpha[c_ys, c_xs] > 30).mean()
        if valid > 0.85:
            best_r_front = float(r)

    best_r_rear = best_r_front
    for r in range(25, min(140, int(h_px * 0.65))):
        angles = np.linspace(0, np.pi, 20)
        c_xs = np.clip(np.round(hub_rear_x + r * np.cos(angles)).astype(int), 0, 1023)
        c_ys = np.clip(np.round((ground_y - r) - r * np.sin(angles)).astype(int), 0, 511)
        valid = (alpha[c_ys, c_xs] > 30).mean()
        if valid > 0.85:
            best_r_rear = float(r)

    front_ratio = (hub_front_x - x_min) / float(w_px)
    diam_ratio = (2.0 * best_r_front) / float(w_px)

    return {
        "x_min": x_min,
        "x_max": x_max,
        "y_min": y_min,
        "y_max": y_max,
        "w_px": w_px,
        "h_px": h_px,
        "ground_y": ground_y,
        "hub_front": (hub_front_x, ground_y - best_r_front),
        "hub_rear": (hub_rear_x, ground_y - best_r_rear),
        "r_front": best_r_front,
        "r_rear": best_r_rear,
        "front_ratio": front_ratio,
        "diam_ratio": diam_ratio,
    }


def analyze_topdown_sprite(im: Image.Image, lat_info: dict | None, is_open_wheel: bool, archetype: str = "gt_slick_front"):
    """Detects vehicle bounding box, front axle X, track width, and tire dimensions from cenital top-down view."""
    arr = np.array(im.convert("RGBA"))
    alpha = arr[:, :, 3]
    ys, xs = np.where(alpha > 20)
    if len(xs) == 0:
        return None

    x_min, x_max = int(xs.min()), int(xs.max())
    y_min, y_max = int(ys.min()), int(ys.max())
    w_px = x_max - x_min + 1
    h_px = y_max - y_min + 1
    cy = (y_min + y_max) / 2.0

    # Expected front axle and tire diameter from lateral view fusion
    lat_front_ratio = lat_info["front_ratio"] if lat_info else (0.75 if is_open_wheel else 0.77)
    expected_axle_x = x_min + lat_front_ratio * w_px
    lat_diam_px = (lat_info["diam_ratio"] * w_px) if lat_info else (w_px * 0.20)
    tire_len_px = lat_diam_px

    # Archetype aspect ratio for tire width
    wid_ratio = {
        "kart_slick_front": 0.48,
        "gt_slick_front": 0.42,
        "nascar_wheel_front": 0.42,
        "rally_wheel_front": 0.38,
        "buggy_allterrain_front": 0.44,
        "truck_allterrain_front": 0.44,
        "monster_wheel_front": 0.58,
        "mud_tractor_front": 0.52,
    }.get(archetype, 0.42)
    tire_wid_px = float(np.clip(tire_len_px * wid_ratio, 28.0, 95.0))

    if is_open_wheel:
        # Rubber mask on top-down view
        r, g, b = arr[:, :, 0].astype(int), arr[:, :, 1].astype(int), arr[:, :, 2].astype(int)
        bright = np.maximum(r, np.maximum(g, b))
        sat = np.maximum(np.abs(r - g), np.maximum(np.abs(r - b), np.abs(g - b)))
        rubber = (bright < 80) & (sat < 35) & (alpha > 40)

        # Search window around expected front axle (+- 50 px)
        win_x0 = max(x_min, int(expected_axle_x - 50))
        win_x1 = min(x_max, int(expected_axle_x + 50))

        fl_mask = rubber.copy()
        fl_mask[:, :win_x0] = False
        fl_mask[:, win_x1:] = False
        fl_mask[int(cy - 6):, :] = False

        fr_mask = rubber.copy()
        fr_mask[:, :win_x0] = False
        fr_mask[:, win_x1:] = False
        fr_mask[:int(cy + 6), :] = False

        y_fl, x_fl = np.where(fl_mask)
        y_fr, x_fr = np.where(fr_mask)

        if len(x_fl) >= 20 and len(x_fr) >= 20:
            axle_x = (float(np.median(x_fl)) + float(np.median(x_fr))) / 2.0
            fl_outer = float(np.percentile(y_fl, 5))
            fr_outer = float(np.percentile(y_fr, 95))
            fl_center = fl_outer + tire_wid_px * 0.5
            fr_center = fr_outer - tire_wid_px * 0.5
            track_width_px = fr_center - fl_center
        else:
            axle_x = expected_axle_x
            track_width_px = h_px - tire_wid_px - 4.0
    else:
        # Closed-wheel: Axle X strictly from lateral hub; track width anchored to fender outer arch
        axle_x = expected_axle_x
        axle_int = int(np.clip(axle_x, x_min, x_max))
        ys_axle = np.where(alpha[:, max(0, axle_int - 4):min(512, axle_int + 5)] > 20)[0]
        if len(ys_axle) > 0:
            fender_top = ys_axle.min()
            fender_bot = ys_axle.max()
            body_span = fender_bot - fender_top + 1
            track_width_px = body_span - tire_wid_px - 4.0
        else:
            track_width_px = h_px - tire_wid_px - 4.0

    # Sanity bounds
    tire_len_px = float(np.clip(tire_len_px, 35.0, 165.0))
    tire_wid_px = float(np.clip(tire_wid_px, 25.0, 95.0))
    track_width_px = float(np.clip(track_width_px, 120.0, 290.0))

    return {
        "x_min": x_min,
        "x_max": x_max,
        "y_min": y_min,
        "y_max": y_max,
        "w_px": w_px,
        "h_px": h_px,
        "cy": cy,
        "axle_x": axle_x,
        "track_width_px": track_width_px,
        "tire_len_px": tire_len_px,
        "tire_wid_px": tire_wid_px,
    }


def determine_archetype(base_car: str, model_id: str, module: str):
    """Classifies the vehicle into one of the 8 motorsport tyre archetypes."""
    # 1. Karts
    if module == "kart" or "kart" in model_id or base_car in ["Kart", "SuperkartGP"]:
        return "kart_slick_front"

    # 2. Monster Trucks
    if base_car == "MonsterTruck" or "overkill" in model_id or "titan" in model_id or "tomb_raider" in model_id:
        return "monster_wheel_front"

    # 3. Mud Boggers
    if base_car == "MudBoggerHeavy" or "mammoth" in model_id or "mud_slinger" in model_id or "ridge" in model_id:
        return "mud_tractor_front"

    # 4. Trophy Trucks & Stadium Pickups
    if base_car in ["TrophyTruckAWD"] or "trophy" in model_id or "desert" in model_id or "stadium" in model_id:
        return "truck_allterrain_front"

    # 5. Open-Wheel Buggies & Cross Cars
    if base_car in ["SandRail", "CrossCar", "SuperBuggy", "DuneBuggyBaja"] or module == "autocross" or "dune" in model_id or "nomad" in model_id or "razor" in model_id:
        return "buggy_allterrain_front"

    # 6. NASCAR Stock Cars & Trucks
    if module == "nascar" or base_car in ["StockCar", "StockCarTruck"] or "nascar" in model_id:
        return "nascar_wheel_front"

    # 7. Rallycross & Autocross Touring
    if module == "rally" or base_car in ["RallyJuniorFWD", "RallyCar", "RallyGroupB", "RallyElectricRX", "TouringAX"] or "rally" in model_id:
        return "rally_wheel_front"

    # 8. GT & Road Racing
    return "gt_slick_front"


def generate_diagnostic_overlay(
    top_img: Image.Image,
    lat_img: Image.Image,
    top_info: dict,
    lat_info: dict | None,
    anchor_entry: dict,
    out_path: Path,
):
    """Renders high-contrast diagnostic overlay marking detected wheels across both views."""
    comp_w = 512 + 768 + 20
    comp_h = 512
    comp = Image.new("RGBA", (comp_w, comp_h), (18, 22, 28, 255))

    # 1. Annotate Top-Down View
    top_ann = top_img.copy().convert("RGBA")
    draw_top = ImageDraw.Draw(top_ann)

    axle_x = top_info["axle_x"]
    cy = top_info["cy"]
    hw = top_info["track_width_px"] * 0.5
    tlen = top_info["tire_len_px"]
    twid = top_info["tire_wid_px"]

    # Vehicle Centerline
    draw_top.line([(0, cy), (512, cy)], fill=(255, 60, 60, 180), width=1)
    # Axle Line
    draw_top.line([(axle_x, 0), (axle_x, 512)], fill=(255, 230, 0, 200), width=2)

    # Front-Left Wheel Box (top)
    fl_x0, fl_x1 = axle_x - tlen * 0.5, axle_x + tlen * 0.5
    fl_y0, fl_y1 = (cy - hw) - twid * 0.5, (cy - hw) + twid * 0.5
    draw_top.rectangle([fl_x0, fl_y0, fl_x1, fl_y1], outline=(0, 255, 120, 255), width=3)
    draw_top.ellipse([axle_x - 3, cy - hw - 3, axle_x + 3, cy - hw + 3], fill=(0, 255, 255, 255))

    # Front-Right Wheel Box (bottom)
    fr_y0, fr_y1 = (cy + hw) - twid * 0.5, (cy + hw) + twid * 0.5
    draw_top.rectangle([fl_x0, fr_y0, fl_x1, fr_y1], outline=(0, 255, 120, 255), width=3)
    draw_top.ellipse([axle_x - 3, cy + hw - 3, axle_x + 3, cy + hw + 3], fill=(0, 255, 255, 255))

    # Track width connector line
    draw_top.line([(axle_x, cy - hw), (axle_x, cy + hw)], fill=(0, 200, 255, 220), width=2)

    comp.paste(top_ann, (0, 0))

    # 2. Annotate Lateral View
    lat_ann = lat_img.copy().convert("RGBA")
    draw_lat = ImageDraw.Draw(lat_ann)

    if lat_info:
        fx, fy = lat_info["hub_front"]
        rf = lat_info["r_front"]
        rx, ry = lat_info["hub_rear"]
        rr = lat_info["r_rear"]

        # Front wheel circle and hub
        draw_lat.ellipse([fx - rf, fy - rf, fx + rf, fy + rf], outline=(255, 230, 0, 255), width=4)
        draw_lat.ellipse([fx - 5, fy - 5, fx + 5, fy + 5], fill=(0, 255, 255, 255))

        # Rear wheel circle and hub
        draw_lat.ellipse([rx - rr, ry - rr, rx + rr, ry + rr], outline=(255, 80, 200, 255), width=4)
        draw_lat.ellipse([rx - 5, ry - 5, rx + 5, ry + 5], fill=(0, 255, 255, 255))

        # Ground contact baseline
        draw_lat.line([(0, lat_info["ground_y"]), (1024, lat_info["ground_y"])], fill=(120, 140, 160, 200), width=2)

    # Resize lateral to 768x384 and paste centered vertically
    lat_resized = lat_ann.resize((768, 384), Image.Resampling.LANCZOS)
    comp.paste(lat_resized, (512 + 20, (512 - 384) // 2))

    # Add text overlay header
    draw_comp = ImageDraw.Draw(comp)
    txt_header = (
        f"{anchor_entry['model_id']} | Archetype: {anchor_entry['archetype']} | "
        f"Axle X: {anchor_entry['axle_x_px']:.1f} px | Track: {anchor_entry['track_width_px']:.1f} px | "
        f"Tire: {anchor_entry['tire_len_px']:.1f} x {anchor_entry['tire_wid_px']:.1f} px"
    )
    draw_comp.text((16, 12), txt_header, fill=(255, 255, 255, 255))

    out_path.parent.mkdir(parents=True, exist_ok=True)
    comp.save(out_path, format="PNG")


def generate_html_review_gallery(anchors_data: list, out_path: Path):
    """Generates the interactive Gate 1 Human-in-the-Loop review HTML gallery."""
    modules = sorted(list(set(item["module"] for item in anchors_data)))
    
    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Spec 095: Multi-View Visual Anchor Extraction Review (Gate 1 HITL)</title>
  <style>
    body {{
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background: #0f1216;
      color: #e2e8f0;
      margin: 0;
      padding: 24px;
    }}
    h1 {{ color: #38bdf8; margin-bottom: 8px; }}
    .subtitle {{ color: #94a3b8; font-size: 15px; margin-bottom: 24px; }}
    .filter-tabs {{
      display: flex;
      gap: 10px;
      margin-bottom: 20px;
      flex-wrap: wrap;
    }}
    .tab-btn {{
      background: #1e293b;
      color: #94a3b8;
      border: 1px solid #334155;
      padding: 8px 16px;
      border-radius: 6px;
      cursor: pointer;
      font-weight: 600;
      transition: all 0.2s;
    }}
    .tab-btn:hover, .tab-btn.active {{
      background: #38bdf8;
      color: #0f172a;
      border-color: #38bdf8;
    }}
    .grid {{
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(620px, 1fr));
      gap: 20px;
    }}
    .card {{
      background: #18202c;
      border: 1px solid #283548;
      border-radius: 8px;
      overflow: hidden;
      display: flex;
      flex-direction: column;
    }}
    .card img {{
      width: 100%;
      height: auto;
      background: #0b0e14;
      display: block;
    }}
    .card-meta {{
      padding: 12px 16px;
    }}
    .card-title {{
      font-weight: bold;
      font-size: 16px;
      color: #f1f5f9;
      margin-bottom: 6px;
      display: flex;
      justify-content: space-between;
    }}
    .badge {{
      font-size: 11px;
      padding: 3px 8px;
      border-radius: 12px;
      background: #0284c7;
      color: #fff;
    }}
    .props {{
      font-size: 13px;
      color: #94a3b8;
      line-height: 1.5;
    }}
    .props span {{
      color: #38bdf8;
      font-family: monospace;
    }}
  </style>
</head>
<body>
  <h1>🏎️ Spec 095: Multi-View Visual Wheel Anchor Review (Gate 1 HITL)</h1>
  <div class="subtitle">Reviewing {len(anchors_data)} vehicles across {len(modules)} motorsport modules. Green box: Cenital front tires | Yellow line: Steered axle | Yellow/Magenta circle: Lateral wheels</div>

  <div class="filter-tabs">
    <button class="tab-btn active" onclick="filterModule('all')">All Modules ({len(anchors_data)})</button>
"""
    for m in modules:
        count = sum(1 for item in anchors_data if item["module"] == m)
        html += f'    <button class="tab-btn" onclick="filterModule(\'{m}\')">{m.upper()} ({count})</button>\n'

    html += """  </div>

  <div class="grid" id="vehicle-grid">
"""
    for item in anchors_data:
        mid = item["model_id"]
        mod = item["module"]
        html += f"""    <div class="card" data-module="{mod}">
      <img src="gate1_anchor_overlays/{mid}.png" alt="{mid}" loading="lazy" />
      <div class="card-meta">
        <div class="card-title">
          <span>{mid}</span>
          <span class="badge">{item['archetype']}</span>
        </div>
        <div class="props">
          Module: <span>{mod}</span> | Layering: <span>{item['layering']}</span><br/>
          Axle X: <span>{item['axle_x_px']:.1f} px</span> | Track Width: <span>{item['track_width_px']:.1f} px</span> | Tire: <span>{item['tire_len_px']:.1f} × {item['tire_wid_px']:.1f} px</span>
        </div>
      </div>
    </div>
"""

    html += """  </div>

  <script>
    function filterModule(mod) {
      document.querySelectorAll('.tab-btn').forEach(btn => btn.classList.remove('active'));
      event.target.classList.add('active');
      document.querySelectorAll('.card').forEach(card => {
        if (mod === 'all' || card.getAttribute('data-module') === mod) {
          card.style.display = 'flex';
        } else {
          card.style.display = 'none';
        }
      });
    }
  </script>
</body>
</html>
"""
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(html)


def main():
    parser = argparse.ArgumentParser(description="Multi-View Sprite Wheel Anchor Extraction (Spec 095)")
    parser.add_argument("--all", action="store_true", help="Process all vehicles")
    parser.add_argument("--module", type=str, help="Target specific module")
    parser.add_argument("--model", type=str, help="Target single model ID")
    parser.add_argument("--dry-run", action="store_true", help="Skip writing images and JSON")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    catalog = load_catalog(root)

    topdown_dir = root / "assets" / "textures" / "vehicles" / "topdown"
    lateral_dir = root / "assets" / "textures" / "vehicles" / "laterals"
    overlays_dir = root / "artifacts" / "hitl" / "gate1_anchor_overlays"
    html_review_path = root / "artifacts" / "hitl" / "gate1_anchor_review.html"
    json_out_path = root / "artifacts" / "visual_wheel_anchors.json"

    modules = ["gt", "nascar", "rally", "autocross", "kart", "extreme_offroad", "classic"]
    if args.module:
        modules = [args.module.lower()]

    anchors_list = []
    processed = 0

    print("=" * 95)
    print("🔍 TdRace Spec 095: Multi-View Computer Vision Wheel Anchor Extraction (Gate 1)")
    print("=" * 95)

    for mod in modules:
        mod_top_dir = topdown_dir / mod
        if not mod_top_dir.exists():
            continue

        sprites = sorted(mod_top_dir.glob("*.png"))
        valid_sprites = [p for p in sprites if not p.stem.endswith("_chassis")]

        print(f"\n📂 Module: [{mod}] ({len(valid_sprites)} vehicles)")

        for p_top in valid_sprites:
            model_id = p_top.stem
            if args.model and model_id != args.model:
                continue

            car_entry = catalog.get(model_id, {})
            base_car = car_entry.get("base_car", "GT3Car")
            is_open_wheel = (base_car in OPEN_WHEEL_BASE_CARS) or (mod == "kart") or ("kart" in model_id)

            # Determine Archetype & Layering
            archetype = determine_archetype(base_car, model_id, mod)
            layering = "OverChassis" if is_open_wheel else "UnderChassis"

            # Load Top-Down Sprite
            top_im = Image.open(p_top).convert("RGBA")

            # Load Lateral Sprite
            lat_p = lateral_dir / mod / f"{model_id}.png"
            lat_im = Image.open(lat_p).convert("RGBA") if lat_p.exists() else None

            # 1. Analyze Lateral View
            lat_info = analyze_lateral_sprite(lat_im) if lat_im else None

            # 2. Analyze Top-Down View
            top_info = analyze_topdown_sprite(top_im, lat_info, is_open_wheel, archetype)
            if not top_info:
                print(f"  [WARN] Failed to analyze top-down sprite for {model_id}")
                continue

            anchor_entry = {
                "model_id": model_id,
                "module": mod,
                "base_car": base_car,
                "layering": layering,
                "archetype": archetype,
                "axle_x_px": round(top_info["axle_x"], 1),
                "track_width_px": round(top_info["track_width_px"], 1),
                "tire_len_px": round(top_info["tire_len_px"], 1),
                "tire_wid_px": round(top_info["tire_wid_px"], 1),
            }
            anchors_list.append(anchor_entry)

            # 4. Generate Diagnostic Overlay
            if not args.dry_run and lat_im:
                out_overlay = overlays_dir / f"{model_id}.png"
                generate_diagnostic_overlay(top_im, lat_im, top_info, lat_info, anchor_entry, out_overlay)

            processed += 1
            print(
                f"  ✓ {model_id:<34} [{archetype:<20}] Axle X: {anchor_entry['axle_x_px']:5.1f} | "
                f"Track: {anchor_entry['track_width_px']:5.1f} | Tire: {anchor_entry['tire_len_px']:4.1f}x{anchor_entry['tire_wid_px']:4.1f} px"
            )

    if not args.dry_run:
        # Write JSON Catalog
        json_out_path.parent.mkdir(parents=True, exist_ok=True)
        anchors_dict = {item["model_id"]: item for item in anchors_list}
        with open(json_out_path, "w", encoding="utf-8") as f:
            json.dump(anchors_dict, f, indent=2)

        # Generate HTML Review Report
        generate_html_review_gallery(anchors_list, html_review_path)

        print("\n" + "=" * 95)
        print(f"🎉 Processed {processed} vehicles successfully!")
        print(f"📁 JSON Catalog: {json_out_path}")
        print(f"🖼️ Diagnostic Overlays: {overlays_dir}/")
        print(f"🌐 Gate 1 HTML Review: {html_review_path}")
        print("=" * 95)


if __name__ == "__main__":
    main()
