#!/usr/bin/env python3
"""
TdRace - Visual Archetype Feature Clustering & Gate 2 Review Generator (Spec 095)

Extracts multi-view wheel feature vectors across the 126-vehicle fleet,
crops high-resolution lateral and cenital wheel patches, organizes them
into 8 motorsport tyre archetypes, and renders an interactive HITL review gallery.
"""

import argparse
import json
from pathlib import Path

import numpy as np
from extract_multiview_wheel_anchors import (
    analyze_lateral_sprite,
)
from PIL import Image

ARCHETYPES = [
    {
        "id": "kart_slick_front",
        "name": "Kart Slick",
        "description": "Ultra-small diameter, wide slick tread, small exposed hub. For Cadet Karts & Superkarts.",
        "disciplines": "Karting",
    },
    {
        "id": "gt_slick_front",
        "name": "GT Competition Slick",
        "description": "Large diameter alloy wheel, ultra-low sidewall, slick tread. For GT4, GT3, GT2, GT1, Hypercars.",
        "disciplines": "GT & Road Racing",
    },
    {
        "id": "nascar_wheel_front",
        "name": "NASCAR Stock Wheel",
        "description": "Medium-large diameter, tall sidewall, deep-dish steel rim with yellow lettering. For Stock Cars & Trucks.",
        "disciplines": "NASCAR / Oval Racing",
    },
    {
        "id": "rally_wheel_front",
        "name": "Rally Gravel / Tarmac",
        "description": "Reinforced OZ multi-spoke alloy, grooved asymmetric gravel/tarmac tread. For Rallycross, RX Lites, Group B.",
        "disciplines": "Rallycross & Autocross Touring",
    },
    {
        "id": "buggy_allterrain_front",
        "name": "Buggy All-Terrain",
        "description": "Lightweight directional steer ribs or all-terrain knobby tread with beadlock rim. For Sand Rails, Cross Cars, Baja Buggies.",
        "disciplines": "Autocross & Sand Dunes",
    },
    {
        "id": "truck_allterrain_front",
        "name": "Trophy Truck All-Terrain",
        "description": "Heavy-duty 37-40\" aggressive knobby all-terrain tread, beadlock ring, heavy steel hub. For Trophy Trucks & Stadium Pickups.",
        "disciplines": "Desert & Stadium Offroad",
    },
    {
        "id": "monster_wheel_front",
        "name": "Monster Terra V-Tread",
        "description": "Massive 66\" Terra agricultural chevron V-tread, giant deep planetary hub. For Monster Trucks.",
        "disciplines": "Extreme Arena & Monster Jumps",
    },
    {
        "id": "mud_tractor_front",
        "name": "Mud Tractor Paddle",
        "description": "Deep-lug directional tractor paddle tread on military Rockwell pinion axles. For Heavy Mud Boggers.",
        "disciplines": "Mud Bogging Trenches",
    },
]


def extract_wheel_crops(
    model_id: str,
    mod: str,
    anchor_entry: dict,
    top_dir: Path,
    lat_dir: Path,
    out_dir: Path,
):
    """Crops high-resolution lateral wheel and cenital wheel patches."""
    top_path = top_dir / mod / f"{model_id}.png"
    lat_path = lat_dir / mod / f"{model_id}.png"

    if not top_path.exists() or not lat_path.exists():
        return None

    top_im = Image.open(top_path).convert("RGBA")
    lat_im = Image.open(lat_path).convert("RGBA")

    # 1. Lateral crop
    lat_info = analyze_lateral_sprite(lat_im)
    if not lat_info:
        return None

    hub_x, hub_y = lat_info["hub_front"]
    r_front = lat_info["r_front"]
    pad = int(r_front * 0.15)
    lat_box = (
        max(0, hub_x - r_front - pad),
        max(0, hub_y - r_front - pad),
        min(lat_im.width, hub_x + r_front + pad),
        min(lat_im.height, hub_y + r_front + pad),
    )
    lat_crop = lat_im.crop(lat_box).resize((128, 128), Image.Resampling.LANCZOS)

    # 2. Cenital (Top-down) crop (Front-Left wheel)
    axle_x = anchor_entry["axle_x_px"]
    track_w = anchor_entry["track_width_px"]
    tlen = anchor_entry["tire_len_px"]
    twid = anchor_entry["tire_wid_px"]

    # Top-down vehicle centerline is ~256
    cy = 256.0
    fl_cx = axle_x
    fl_cy = cy - track_w * 0.5

    top_box = (
        max(0, int(fl_cx - tlen * 0.65)),
        max(0, int(fl_cy - twid * 0.70)),
        min(top_im.width, int(fl_cx + tlen * 0.65)),
        min(top_im.height, int(fl_cy + twid * 0.70)),
    )
    top_crop = top_im.crop(top_box).resize((128, 128), Image.Resampling.LANCZOS)

    # Save crops
    arch = anchor_entry["archetype"]
    arch_dir = out_dir / arch
    arch_dir.mkdir(parents=True, exist_ok=True)

    lat_out = arch_dir / f"{model_id}_lat.png"
    top_out = arch_dir / f"{model_id}_top.png"

    lat_crop.save(lat_out)
    top_crop.save(top_out)

    diam_ratio = lat_info["diam_ratio"]
    aspect_ratio = twid / max(1.0, tlen)

    return {
        "model_id": model_id,
        "module": mod,
        "archetype": arch,
        "diam_ratio": diam_ratio,
        "aspect_ratio": aspect_ratio,
        "tire_len_px": tlen,
        "tire_wid_px": twid,
        "track_width_px": track_w,
        "layering": anchor_entry.get("layering", "UnderChassis"),
        "lat_rel_path": f"{arch}/{model_id}_lat.png",
        "top_rel_path": f"{arch}/{model_id}_top.png",
    }


def generate_gate2_html(clustered_data: dict, out_html: Path):
    """Generates interactive Gate 2 HTML review gallery."""
    html = """<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>TdRace Spec 095 — Gate 2: Tyre Archetype Cluster Review</title>
  <style>
    body { margin: 0; background: #0c0f14; color: #d0d7de; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, monospace; }
    header { background: #161b22; padding: 18px 28px; border-bottom: 1px solid #30363d; display: flex; justify-content: space-between; align-items: center; }
    h1 { margin: 0; font-size: 20px; color: #58a6ff; font-weight: 600; }
    .badge { background: #238636; color: #fff; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: bold; }
    .tabs { display: flex; flex-wrap: wrap; gap: 8px; padding: 14px 28px; background: #12161e; border-bottom: 1px solid #21262d; }
    .tab-btn { background: #21262d; color: #c9d1d9; border: 1px solid #30363d; border-radius: 6px; padding: 8px 16px; cursor: pointer; font-size: 13px; font-weight: 500; transition: all 0.15s; }
    .tab-btn:hover { background: #30363d; }
    .tab-btn.active { background: #1f6feb; border-color: #58a6ff; color: #fff; font-weight: bold; }
    .cluster-summary { padding: 16px 28px; background: #161c24; border-bottom: 1px solid #30363d; display: none; }
    .cluster-summary.active { display: block; }
    .cluster-summary h2 { margin: 0 0 6px 0; font-size: 17px; color: #f0883e; }
    .cluster-summary p { margin: 0 0 10px 0; font-size: 13px; color: #8b949e; }
    .stat-row { display: flex; gap: 24px; font-size: 12px; }
    .stat-val { color: #58a6ff; font-weight: bold; }
    .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(290px, 1fr)); gap: 14px; padding: 24px 28px; }
    .card { background: #161b22; border: 1px solid #30363d; border-radius: 8px; overflow: hidden; display: flex; flex-direction: column; transition: transform 0.1s; }
    .card:hover { border-color: #58a6ff; transform: translateY(-2px); }
    .card-images { display: flex; background: #0d1117; padding: 8px; gap: 8px; justify-content: center; }
    .crop-box { text-align: center; font-size: 11px; color: #8b949e; }
    .crop-box img { width: 128px; height: 128px; border-radius: 4px; border: 1px solid #21262d; background: #181c24; display: block; margin-bottom: 4px; }
    .card-meta { padding: 12px 14px; background: #161b22; border-top: 1px solid #21262d; font-size: 12px; }
    .card-title { font-weight: bold; color: #f0f6fc; margin-bottom: 6px; display: flex; justify-content: space-between; align-items: center; }
    .props { color: #8b949e; line-height: 1.5; }
    .props span { color: #79c0ff; }
  </style>
</head>
<body>
  <header>
    <h1>🔍 Spec 095 Gate 2: Tyre Archetype Feature Clustering Review</h1>
    <div>
      <span class="badge">HITL Gate 2 Ready</span>
    </div>
  </header>

  <div class="tabs">
    <button class="tab-btn active" onclick="filterArchetype('all')">ALL ARCHETYPES</button>
"""
    for arch in ARCHETYPES:
        aid = arch["id"]
        items = clustered_data.get(aid, [])
        html += f'    <button class="tab-btn" onclick="filterArchetype(\'{aid}\')">{arch["name"].upper()} ({len(items)})</button>\n'

    html += """  </div>\n\n"""

    for arch in ARCHETYPES:
        aid = arch["id"]
        items = clustered_data.get(aid, [])
        mean_diam = np.mean([it["diam_ratio"] for it in items]) if items else 0.0
        mean_aspect = np.mean([it["aspect_ratio"] for it in items]) if items else 0.0
        html += f"""  <div class="cluster-summary" id="summary-{aid}">
    <h2>{arch['name']} ({len(items)} vehicles)</h2>
    <p>{arch['description']} &mdash; Target: <em>{arch['disciplines']}</em></p>
    <div class="stat-row">
      <div>Vehicle Count: <span class="stat-val">{len(items)}</span></div>
      <div>Mean Wheel Diameter Ratio: <span class="stat-val">{mean_diam:.3f}</span></div>
      <div>Mean Width/Length Aspect: <span class="stat-val">{mean_aspect:.2f}</span></div>
    </div>
  </div>
"""

    html += """  <div class="grid" id="cluster-grid">\n"""

    for arch in ARCHETYPES:
        aid = arch["id"]
        items = clustered_data.get(aid, [])
        for it in items:
            mid = it["model_id"]
            html += f"""    <div class="card" data-archetype="{aid}">
      <div class="card-images">
        <div class="crop-box">
          <img src="{it['lat_rel_path']}" alt="{mid} lateral" loading="lazy" />
          <span>Lateral Hub</span>
        </div>
        <div class="crop-box">
          <img src="{it['top_rel_path']}" alt="{mid} topdown" loading="lazy" />
          <span>Top-Down FL</span>
        </div>
      </div>
      <div class="card-meta">
        <div class="card-title">
          <span>{mid}</span>
          <span style="color:#f0883e; font-size:11px;">{it['layering']}</span>
        </div>
        <div class="props">
          Module: <span>{it['module']}</span> | Arch: <span>{aid}</span><br/>
          Tire Size: <span>{it['tire_len_px']:.1f} × {it['tire_wid_px']:.1f} px</span> | Track: <span>{it['track_width_px']:.1f} px</span><br/>
          Diam Ratio: <span>{it['diam_ratio']:.3f}</span> | Aspect: <span>{it['aspect_ratio']:.2f}</span>
        </div>
      </div>
    </div>
"""

    html += """  </div>

  <script>
    function filterArchetype(aid) {
      document.querySelectorAll('.tab-btn').forEach(btn => btn.classList.remove('active'));
      event.target.classList.add('active');
      document.querySelectorAll('.cluster-summary').forEach(s => s.classList.remove('active'));
      if (aid !== 'all') {
        const sum = document.getElementById('summary-' + aid);
        if (sum) sum.classList.add('active');
      }
      document.querySelectorAll('.card').forEach(card => {
        if (aid === 'all' || card.getAttribute('data-archetype') === aid) {
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
    parser = argparse.ArgumentParser(description="Visual Archetype Feature Clustering & Gate 2 Review")
    parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    anchors_path = root / "artifacts" / "visual_wheel_anchors.json"

    if not anchors_path.exists():
        print(f"Error: {anchors_path} not found. Run extract_multiview_wheel_anchors.py first.")
        return

    with open(anchors_path, "r", encoding="utf-8") as f:
        anchors_data = json.load(f)

    top_dir = root / "assets" / "textures" / "vehicles" / "topdown"
    lat_dir = root / "assets" / "textures" / "vehicles" / "laterals"
    gate2_dir = root / "artifacts" / "hitl" / "gate2_clusters"
    gate2_dir.mkdir(parents=True, exist_ok=True)
    out_html = root / "artifacts" / "hitl" / "gate2_cluster_review.html"

    print("=" * 90)
    print("🔬 TdRace Spec 095: Visual Archetype Feature Extraction & Clustering (Gate 2)")
    print("=" * 90)

    clustered = {a["id"]: [] for a in ARCHETYPES}
    processed = 0

    for model_id, entry in sorted(anchors_data.items()):
        mod = entry["module"]
        res = extract_wheel_crops(model_id, mod, entry, top_dir, lat_dir, gate2_dir)
        if res:
            arch = res["archetype"]
            if arch in clustered:
                clustered[arch].append(res)
            processed += 1

    print(f"\nExtracted wheel crops and features for {processed} vehicles across 8 archetypes:")
    for arch in ARCHETYPES:
        aid = arch["id"]
        count = len(clustered[aid])
        print(f"  • {arch['name']:28s} ({aid:22s}): {count:2d} vehicles")

    generate_gate2_html(clustered, out_html)
    print(f"\n✅ Gate 2 HTML Review Gallery: {out_html}")


if __name__ == "__main__":
    main()
