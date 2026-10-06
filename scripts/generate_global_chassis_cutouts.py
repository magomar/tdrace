#!/usr/bin/env python3
# /// script
# dependencies = ["pillow", "numpy", "scipy"]
# ///
"""
Automated Global Vehicle Chassis Cutout and Inpainting Pipeline (Spec 095).

Generates `<model_id>_chassis.png` sprites across all motorsport modules in TdRace:
- GT, NASCAR, Rally, Autocross, Karting, Extreme Off-Road, and Classic.
- Consumes verified Computer Vision anchors from `artifacts/visual_wheel_anchors.json`.
- Open-Wheel Archetypes (OverChassis):
  Clears outboard front tire rubber while strictly preserving suspension wishbones,
  pushrods, tie rods, and front nosecone/bumper bodywork.
- Closed-Wheel Archetypes (UnderChassis):
  Hollows out outer fender apertures for underlying steered wheel visibility,
  and inpaints dark ambient cavity backing (#14181c, 100% opacity) across the inner
  wheel-well liner to prevent track surface see-through during dynamic suspension roll (+-18cm).
- Generates Gate 3 visual diffs and interactive inspection gallery.
"""

import argparse
import json
from pathlib import Path
import sys
import numpy as np
from PIL import Image, ImageDraw
from scipy.ndimage import binary_dilation

DARK_CAVITY_RGB = (20, 24, 28)
DARK_CAVITY_ALPHA = 255


def load_wheel_anchors(root: Path):
    """Loads verified visual wheel anchors catalog."""
    anchors_path = root / "artifacts" / "visual_wheel_anchors.json"
    if not anchors_path.exists():
        print(f"Error: {anchors_path} not found. Run extract_multiview_wheel_anchors.py first.")
        sys.exit(1)
    with open(anchors_path, "r", encoding="utf-8") as f:
        return json.load(f)


def process_vehicle_sprite(
    image_path: Path,
    anchor_entry: dict,
    out_path: Path,
    diff_out_path: Path | None = None,
    dry_run: bool = False,
):
    """Processes a single topdown vehicle sprite into a chassis cutout using verified anchors."""
    im = Image.open(image_path).convert("RGBA")
    arr = np.array(im)

    alpha = arr[:, :, 3]
    ys, xs = np.where(alpha > 20)
    if len(xs) == 0:
        print(f"  [WARN] {image_path.name} contains no opaque pixels, skipping.")
        return False

    cy_px = (ys.min() + ys.max()) / 2.0

    axle_x = anchor_entry["axle_x_px"]
    track_w = anchor_entry["track_width_px"]
    tlen = anchor_entry["tire_len_px"]
    twid = anchor_entry["tire_wid_px"]
    is_open_wheel = anchor_entry["layering"] == "OverChassis"

    out_arr = arr.copy()

    # Bounding box along X with 15% safety margin
    half_len = tlen * 0.5 * 1.15
    x0 = max(0, int(round(axle_x - half_len)))
    x1 = min(512, int(round(axle_x + half_len)))

    r, g, b = arr[:, :, 0].astype(int), arr[:, :, 1].astype(int), arr[:, :, 2].astype(int)
    brightness = np.maximum(r, np.maximum(g, b))
    sat = np.maximum(np.abs(r - g), np.maximum(np.abs(r - b), np.abs(g - b)))

    # Dark rubber detection (achromatic dark OR edge chromatic fringe)
    is_rubber = (((brightness < 95) & (sat < 50)) | ((g < 35) & (brightness < 95))) & (alpha > 10)

    half_wid = twid * 0.5 * 1.15
    y_fl = cy_px - track_w * 0.5
    y_fr = cy_px + track_w * 0.5

    if is_open_wheel:
        # Open wheel (OverChassis):
        # Clear tire rubber in front wheel bounding boxes while strictly preserving wishbones/nosecone
        for y_center in [y_fl, y_fr]:
            y0 = max(0, int(round(y_center - half_wid)))
            y1 = min(512, int(round(y_center + half_wid)))
            box = np.zeros((512, 512), dtype=bool)
            box[y0:y1, x0:x1] = True

            # Dilate rubber mask by 2px to eliminate residual edge fringes
            rubber_in_box = box & is_rubber
            dilated = binary_dilation(rubber_in_box, iterations=2) & box
            out_arr[dilated, 3] = 0

            # Clean residual sub-15 opacity stray dust inside the box
            out_arr[box & (out_arr[:, :, 3] < 15), 3] = 0
    else:
        # Closed wheel (UnderChassis):
        # 1. Hollow out outer aperture where the tire is exposed in the fender
        # 2. Inpaint dark cavity backing across inner liner (+-18cm roll buffer)
        cavity_depth = int(round(0.18 * 115.0))  # ~18cm suspension roll buffer (~20 px)

        # Front-Left (top quadrant)
        y_out_fl = max(0, int(round(y_fl - half_wid * 1.25)))
        y_in_fl = min(int(cy_px - 20), int(round(y_fl + half_wid)))
        y_cav_fl = min(int(cy_px - 15), y_in_fl + cavity_depth)

        fl_aperture = np.zeros((512, 512), dtype=bool)
        fl_aperture[y_out_fl:y_in_fl, x0:x1] = True
        out_arr[fl_aperture & is_rubber, 3] = 0

        fl_cavity = np.zeros((512, 512), dtype=bool)
        fl_cavity[y_in_fl:y_cav_fl, x0:x1] = True
        apply_cav_fl = fl_cavity & ((out_arr[:, :, 3] == 0) | is_rubber)
        out_arr[apply_cav_fl, 0] = DARK_CAVITY_RGB[0]
        out_arr[apply_cav_fl, 1] = DARK_CAVITY_RGB[1]
        out_arr[apply_cav_fl, 2] = DARK_CAVITY_RGB[2]
        out_arr[apply_cav_fl, 3] = DARK_CAVITY_ALPHA

        # Front-Right (bottom quadrant)
        y_out_fr = min(512, int(round(y_fr + half_wid * 1.25)))
        y_in_fr = max(int(cy_px + 20), int(round(y_fr - half_wid)))
        y_cav_fr = max(int(cy_px + 15), y_in_fr - cavity_depth)

        fr_aperture = np.zeros((512, 512), dtype=bool)
        fr_aperture[y_in_fr:y_out_fr, x0:x1] = True
        out_arr[fr_aperture & is_rubber, 3] = 0

        fr_cavity = np.zeros((512, 512), dtype=bool)
        fr_cavity[y_cav_fr:y_in_fr, x0:x1] = True
        apply_cav_fr = fr_cavity & ((out_arr[:, :, 3] == 0) | is_rubber)
        out_arr[apply_cav_fr, 0] = DARK_CAVITY_RGB[0]
        out_arr[apply_cav_fr, 1] = DARK_CAVITY_RGB[1]
        out_arr[apply_cav_fr, 2] = DARK_CAVITY_RGB[2]
        out_arr[apply_cav_fr, 3] = DARK_CAVITY_ALPHA

    erased_px = int(np.sum((arr[:, :, 3] > 0) & (out_arr[:, :, 3] == 0)))
    cavity_px = int(
        np.sum(
            (out_arr[:, :, 0] == DARK_CAVITY_RGB[0])
            & (out_arr[:, :, 1] == DARK_CAVITY_RGB[1])
            & (out_arr[:, :, 2] == DARK_CAVITY_RGB[2])
            & (out_arr[:, :, 3] == DARK_CAVITY_ALPHA)
        )
    )

    if not dry_run:
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_img = Image.fromarray(out_arr)
        out_img.save(out_path, format="PNG")

        if diff_out_path:
            diff_out_path.parent.mkdir(parents=True, exist_ok=True)
            # Create high-contrast side-by-side diagnostic diff
            comp_w = 512 * 3 + 20
            comp_h = 512
            comp = Image.new("RGBA", (comp_w, comp_h), (14, 18, 24, 255))

            # 1. Original
            comp.paste(im, (0, 0), im)

            # 2. Chassis Cutout
            comp.paste(out_img, (512 + 10, 0), out_img)

            # 3. Diff Overlay
            diff_arr = np.zeros((512, 512, 4), dtype=np.uint8)
            diff_arr[:, :] = [25, 30, 38, 255]  # Dark background
            # Show chassis in dim gray
            chassis_mask = out_arr[:, :, 3] > 20
            diff_arr[chassis_mask] = [90, 100, 115, 255]
            # Erased pixels in bright neon green
            erased_mask = (arr[:, :, 3] > 20) & (out_arr[:, :, 3] == 0)
            diff_arr[erased_mask] = [0, 255, 120, 255]
            # Inpainted cavity in bright cyan
            cavity_mask = (
                (out_arr[:, :, 0] == DARK_CAVITY_RGB[0])
                & (out_arr[:, :, 1] == DARK_CAVITY_RGB[1])
                & (out_arr[:, :, 2] == DARK_CAVITY_RGB[2])
                & (out_arr[:, :, 3] == DARK_CAVITY_ALPHA)
            )
            diff_arr[cavity_mask] = [0, 200, 255, 255]

            diff_img = Image.fromarray(diff_arr)
            draw_diff = ImageDraw.Draw(diff_img)
            # Draw axle line and wheel boxes
            draw_diff.line([(axle_x, 0), (axle_x, 512)], fill=(255, 230, 0, 180), width=1)
            comp.paste(diff_img, (1024 + 20, 0))

            comp.save(diff_out_path, format="PNG")

    mode_label = "OverChassis (open)" if is_open_wheel else "UnderChassis (closed)"
    print(
        f"  ✓ {anchor_entry['model_id']:<34} [{mode_label:<20}] Axle: X={axle_x:5.1f} | Erased: {erased_px:4d} px | Cavity: {cavity_px:4d} px"
    )
    return True


def generate_gate3_html(processed_entries: list, out_html: Path):
    """Generates Gate 3 HTML inspection report."""
    modules = sorted(list(set(e["module"] for e in processed_entries)))

    html = """<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>TdRace Spec 095 — Gate 3: Cutout Cleanliness & Cavity Inspection</title>
  <style>
    body { margin: 0; background: #0c0f14; color: #d0d7de; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, monospace; }
    header { background: #161b22; padding: 18px 28px; border-bottom: 1px solid #30363d; display: flex; justify-content: space-between; align-items: center; }
    h1 { margin: 0; font-size: 20px; color: #58a6ff; font-weight: 600; }
    .badge { background: #238636; color: #fff; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: bold; }
    .tabs { display: flex; flex-wrap: wrap; gap: 8px; padding: 14px 28px; background: #12161e; border-bottom: 1px solid #21262d; }
    .tab-btn { background: #21262d; color: #c9d1d9; border: 1px solid #30363d; border-radius: 6px; padding: 8px 16px; cursor: pointer; font-size: 13px; font-weight: 500; transition: all 0.15s; }
    .tab-btn:hover { background: #30363d; }
    .tab-btn.active { background: #1f6feb; border-color: #58a6ff; color: #fff; font-weight: bold; }
    .legend { padding: 10px 28px; background: #161b22; font-size: 12px; display: flex; gap: 24px; border-bottom: 1px solid #21262d; }
    .legend-item { display: flex; align-items: center; gap: 8px; }
    .dot { width: 12px; height: 12px; border-radius: 2px; }
    .grid { display: flex; flex-direction: column; gap: 20px; padding: 24px 28px; }
    .card { background: #161b22; border: 1px solid #30363d; border-radius: 8px; overflow: hidden; display: flex; flex-direction: column; }
    .card img { width: 100%; max-width: 1556px; height: auto; display: block; background: #0d1117; }
    .card-meta { padding: 12px 16px; display: flex; justify-content: space-between; align-items: center; background: #161b22; border-top: 1px solid #21262d; font-size: 13px; }
    .card-title { font-weight: bold; color: #f0f6fc; }
    .props { color: #8b949e; }
    .props span { color: #79c0ff; }
  </style>
</head>
<body>
  <header>
    <h1>✂️ Spec 095 Gate 3: Cutout Cleanliness & Cavity Inspection</h1>
    <div>
      <span class="badge">HITL Gate 3 Ready</span>
    </div>
  </header>

  <div class="legend">
    <div class="legend-item"><div class="dot" style="background:#fff"></div> Panel 1: Original Sprite</div>
    <div class="legend-item"><div class="dot" style="background:#58a6ff"></div> Panel 2: Inpainted Chassis Cutout</div>
    <div class="legend-item"><div class="dot" style="background:#00ff78"></div> Green: Erased Tire Rubber</div>
    <div class="legend-item"><div class="dot" style="background:#00c8ff"></div> Cyan: Ambient Cavity Liner (#14181c)</div>
  </div>

  <div class="tabs">
    <button class="tab-btn active" onclick="filterModule('all')">ALL ({len(processed_entries)})</button>
"""
    for m in modules:
        count = sum(1 for e in processed_entries if e["module"] == m)
        html += f'    <button class="tab-btn" onclick="filterModule(\'{m}\')">{m.upper()} ({count})</button>\n'

    html += """  </div>

  <div class="grid" id="diff-grid">
"""
    for e in processed_entries:
        mid = e["model_id"]
        mod = e["module"]
        html += f"""    <div class="card" data-module="{mod}">
      <img src="gate3_cutouts/{mid}_diff.png" alt="{mid} cutout diff" loading="lazy" />
      <div class="card-meta">
        <div class="card-title">
          <span>{mid}</span>
          <span style="color:#f0883e; margin-left:12px; font-weight:normal;">{e['archetype']} &mdash; {e['layering']}</span>
        </div>
        <div class="props">
          Module: <span>{mod}</span> | Axle X: <span>{e['axle_x_px']:.1f} px</span> | Track: <span>{e['track_width_px']:.1f} px</span>
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
    with open(out_html, "w", encoding="utf-8") as f:
        f.write(html)


def main():
    parser = argparse.ArgumentParser(
        description="Automated Global Vehicle Chassis Cutout and Inpainting Pipeline (Spec 095)"
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Process all vehicles across all motorsport modules",
    )
    parser.add_argument(
        "--module",
        type=str,
        help="Target module (gt, nascar, rally, autocross, kart, extreme_offroad, classic)",
    )
    parser.add_argument(
        "--model",
        type=str,
        help="Target single vehicle model ID (e.g. gt_albion_victor_t1)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Compute geometry and stats without writing files to disk",
    )

    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent

    anchors = load_wheel_anchors(root)
    topdown_dir = root / "assets" / "textures" / "vehicles" / "topdown"
    diff_dir = root / "artifacts" / "hitl" / "gate3_cutouts"
    out_html = root / "artifacts" / "hitl" / "gate3_cutout_diff.html"

    modules = ["gt", "nascar", "rally", "autocross", "kart", "extreme_offroad", "classic"]
    if args.module:
        target_mod = args.module.lower()
        if target_mod not in modules:
            print(f"Unknown module '{args.module}'. Available: {modules}")
            sys.exit(1)
        modules = [target_mod]

    processed = []
    skipped_count = 0

    print("=" * 95)
    print("🏎️  TdRace Spec 095 Global Precision Chassis Cutout & Inpainting Generator (Gate 3)")
    print("=" * 95)

    for mod in modules:
        mod_dir = topdown_dir / mod
        if not mod_dir.exists():
            continue

        sprite_files = sorted(mod_dir.glob("*.png"))
        valid_sprites = [p for p in sprite_files if not p.stem.endswith("_chassis")]

        print(f"\n📂 Module: [{mod}] ({len(valid_sprites)} vehicles)")

        for img_p in valid_sprites:
            model_id = img_p.stem
            if args.model and model_id != args.model:
                continue

            anchor_entry = anchors.get(model_id)
            if not anchor_entry:
                print(f"  [WARN] Model '{model_id}' not found in anchors, skipping.")
                skipped_count += 1
                continue

            out_chassis_p = mod_dir / f"{model_id}_chassis.png"
            diff_p = diff_dir / f"{model_id}_diff.png"

            ok = process_vehicle_sprite(
                image_path=img_p,
                anchor_entry=anchor_entry,
                out_path=out_chassis_p,
                diff_out_path=diff_p,
                dry_run=args.dry_run,
            )
            if ok:
                processed.append(anchor_entry)
            else:
                skipped_count += 1

    if not args.dry_run and processed:
        generate_gate3_html(processed, out_html)
        print(f"\n✅ Gate 3 HTML Review Gallery: {out_html}")

    print("\n" + "=" * 95)
    print(f"🏁 Cutout generation complete! Processed: {len(processed)}, Skipped: {skipped_count}")
    print("=" * 95)


if __name__ == "__main__":
    main()
