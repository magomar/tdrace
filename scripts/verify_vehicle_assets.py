#!/usr/bin/env python3
import json
import sys
from pathlib import Path


def main():
    root = Path(__file__).resolve().parent.parent
    vehicles_json = root / "portals" / "shared" / "data" / "codex" / "cars.json"
    if not vehicles_json.exists():
        vehicles_json = root / "portals" / "shared" / "data" / "vehicles.json"
    if not vehicles_json.exists():
        print(f"Error: {vehicles_json} not found. Run cargo run -p tdrace-app --bin export_codex first.")
        return 1

    raw = json.loads(vehicles_json.read_text(encoding="utf-8"))
    vehicles = raw.get("items", raw)
    print(f"🔍 Auditing 2D Assets across {len(vehicles)} authentic motorsport vehicles...\n")

    stats = {}
    for v in vehicles:
        m = v["module"]
        mod_id = None
        for k, name in [("gt", "Gran Turismo & Endurance"), ("nascar", "NASCAR Stock Car Racing"), ("rally", "Rallycross & All-Terrain"), ("extreme_offroad", "Extreme Off-Road & Arenas"), ("kart", "Karting & Micro-Racers")]:
            if name == m:
                mod_id = k
                break
        if not mod_id:
            mod_id = m

        if m not in stats:
            stats[m] = {"total": 0, "lateral": 0, "thumb": 0, "topdown": 0}

        stats[m]["total"] += 1

        lat_file = root / "assets" / "textures" / "vehicles" / "laterals" / mod_id / f"{v['id']}.png"
        thumb_file = root / "assets" / "textures" / "vehicles" / "laterals" / mod_id / f"{v['id']}_thumb.png"
        top_file = root / "assets" / "textures" / "vehicles" / "topdown" / mod_id / f"{v['id']}.png"

        if lat_file.exists():
            stats[m]["lateral"] += 1
        if thumb_file.exists():
            stats[m]["thumb"] += 1
        if top_file.exists():
            stats[m]["topdown"] += 1

    print(f"{'Module Name':<32} | {'Cars':<5} | {'2D Lateral':<10} | {'Thumb':<6} | {'Top-Down':<8}")
    print("-" * 72)

    tot_cars = tot_lat = tot_thumb = tot_top = 0
    for m, s in stats.items():
        print(f"{m:<32} | {s['total']:<5} | {s['lateral']:<10} | {s['thumb']:<6} | {s['topdown']:<8}")
        tot_cars += s["total"]
        tot_lat += s["lateral"]
        tot_thumb += s["thumb"]
        tot_top += s["topdown"]

    print("-" * 72)
    print(f"{'TOTALS':<32} | {tot_cars:<5} | {tot_lat:<10} | {tot_thumb:<6} | {tot_top:<8}\n")

    return 0

if __name__ == "__main__":
    sys.exit(main())
