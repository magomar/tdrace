#!/usr/bin/env python3
"""Builds Extreme Off-Road circuits from official desert-race GPX course files (spec 046).

Desert races are not mapped in OpenStreetMap, so these circuits come from the course files that
the organisers publish (cached in assets/gpx/). The pipeline reuses the OSM importer helpers:

    GPX track -> local metres -> scale -> cut spurs -> round corners -> close loop -> resample

Usage:
    python3 scripts/gpx_importer.py [--track <id>] [--json]
    cargo run --bin track_bake -- tracks/extreme_offroad/<id>.json   (printed by --json)
"""

import argparse
import json
import math
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from osm_importer import (
    BAKE_COMMAND,
    REPO_ROOT,
    TRACKS_DIR,
    fillet_corners,
    heading_ahead,
    latlon_to_meters,
    polyline_length,
    resample_polyline,
    rotate_points,
)

GPX_DIR = os.path.join(REPO_ROOT, "assets", "gpx")
MODULE = "extreme_offroad"
MINT_400_SOURCE = "https://themint400.com/race-format/"

# Two parts of the lap that are closer than this many track widths (centre to centre) and not
# neighbours along the lap would put their walls on top of each other: the importer cuts the part
# between them.
SPUR_GAP_WIDTHS = 2.2
# Only parts that are at least this far apart along the lap count as "not neighbours".
SPUR_MIN_ARC_M = 150.0
# A spur is shorter than this fraction of the course; a longer "spur" is the rest of the lap.
SPUR_MAX_FRACTION = 0.4

GPX_TRACKS = {
    "mint400_short_course": {
        "name": "Mint 400 Short Course",
        "description": "The 2.3 km Youth 170/250 loop of the Mint 400 at Primm, Nevada: tight desert "
        "sweepers and a hairpin, from the official 2026 course file.",
        "file": "2026_M400_Y170_250_Course_-_Race_Ready.gpx",
        "segment": 0,
        "scale": 1.0,
        "width": 14.0,
        "spacing_m": 25.0,
        "min_radius_m": 14.0,
        "laps": 4,
        "tag": "MINT 400 SHORT COURSE",
    },
    "mint400_qualifying_loop": {
        "name": "Mint 400 Qualifying Loop",
        "description": "The 9 km Mint 400 qualifying loop at Primm, Nevada: long desert straights, "
        "rocky switchbacks and a wide sweeper, from the official 2026 course file.",
        "file": "2026_M400_Qualifying_-_Race_Ready.gpx",
        "segment": 0,
        "scale": 1.0,
        "width": 16.0,
        "spacing_m": 40.0,
        "min_radius_m": 18.0,
        "laps": 2,
        "tag": "MINT 400 QUALIFYING",
    },
    "mint400_grand_loop": {
        "name": "Mint 400 Grand Loop",
        "description": "The whole 148 km Car/Truck/UTV course of the Mint 400, shrunk to a fast desert "
        "lap: the real shape of the Great American Off-Road Race, from the official 2026 course file.",
        "file": "2026_M400_CTU_Course_-_Race_Ready.gpx",
        "segment": 0,
        "scale": 0.05,
        "width": 16.0,
        "spacing_m": 35.0,
        "min_radius_m": 18.0,
        "laps": 2,
        "tag": "MINT 400 GRAND LOOP",
    },
}


def read_gpx_segment(path, index):
    """The (lat, lon) points of one <trkseg> (regex: LeadNav files use an unbound XML prefix)."""
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()
    segments = re.findall(r"<trkseg>(.*?)</trkseg>", text, re.DOTALL)
    pts = re.findall(r'<trkpt lat="([-\d.]+)" lon="([-\d.]+)"', segments[index])
    return [(float(lat), float(lon)) for lat, lon in pts]


def cut_spurs(points, gap_m, min_arc_m=SPUR_MIN_ARC_M):
    """Removes out-and-back spurs and the open ends of the course, and closes the loop.

    For each point i, the importer finds the last point j that is closer than `gap_m` but at least
    `min_arc_m` (and less than SPUR_MAX_FRACTION of the course) further along the lap, and drops
    everything between them. The loop closes the same
    way: the start and the end of the course are joined where they first come close.
    """
    n = len(points)
    cum = [0.0]
    for i in range(1, n):
        cum.append(cum[-1] + math.dist(points[i - 1], points[i]))

    # Close the loop: the earliest start point that the end of the course comes back to.
    start, end = 0, n - 1
    best = None
    for i in range(n):
        if cum[i] > cum[-1] * 0.25:
            break
        for j in range(n - 1, i, -1):
            if cum[j] < cum[-1] * 0.75:
                break
            if math.dist(points[i], points[j]) < gap_m:
                best = (i, j)
                break
        if best:
            break
    if best:
        start, end = best

    kept = []
    i = start
    while i <= end:
        kept.append(points[i])
        jump = None
        for j in range(end, i, -1):
            if cum[j] - cum[i] < min_arc_m:
                break
            if cum[j] - cum[i] > cum[-1] * SPUR_MAX_FRACTION:
                continue
            if math.dist(points[i], points[j]) < gap_m:
                jump = j
                break
        i = jump if jump is not None else i + 1
    if math.dist(kept[0], kept[-1]) < 1.0:
        kept.pop()
    return kept


def min_clearance(points, min_arc_m=SPUR_MIN_ARC_M):
    """Smallest centre-to-centre distance between two parts of the closed lap that are not neighbours."""
    n = len(points)
    step = polyline_length(points, closed=True) / n
    skip = max(1, int(min_arc_m / step))
    best = float("inf")
    for i in range(n):
        for k in range(skip, n - skip):
            best = min(best, math.dist(points[i], points[(i + k) % n]))
    return best


def process_gpx_track(track_id):
    spec = GPX_TRACKS[track_id]
    raw = read_gpx_segment(os.path.join(GPX_DIR, spec["file"]), spec["segment"])
    lat0 = sum(p[0] for p in raw) / len(raw)
    lon0 = sum(p[1] for p in raw) / len(raw)
    metric = [latlon_to_meters(lat, lon, lat0, lon0) for lat, lon in raw]
    raw_len = sum(math.dist(metric[i - 1], metric[i]) for i in range(1, len(metric)))

    s = spec["scale"]
    scaled = [(x * s, y * s) for x, y in metric]
    loop = cut_spurs(scaled, SPUR_GAP_WIDTHS * spec["width"])
    # GPS points are dense and noisy: resample to the waypoint spacing first, so that the fillet
    # rounds real corners and not GPS jitter.
    coarse = round(polyline_length(loop, closed=True) / spec["spacing_m"])
    loop, _, _ = resample_polyline(loop, coarse)
    loop, _ = fillet_corners(loop, ["PackedSand"] * len(loop), spec["min_radius_m"])

    heading = heading_ahead(loop)
    rotated = rotate_points(loop, heading)
    x0, y0 = rotated[0]
    aligned = [(x - x0, y - y0) for x, y in rotated]

    count = max(24, round(polyline_length(aligned, closed=True) / spec["spacing_m"]))
    resampled, _, final_len = resample_polyline(aligned, count)

    waypoints = [
        {
            "point": [round(x, 1), round(y, 1)],
            "width": spec["width"],
            "left_curb": False,
            "right_curb": False,
            "surface": "PackedSand",
            "elevation": 0.0,
            "left_runoff_surface": "DeepSand",
            "right_runoff_surface": "DeepSand",
        }
        for x, y in resampled
    ]
    return {
        "id": track_id,
        "spec": spec,
        "waypoints": waypoints,
        "raw_length": raw_len,
        "total_length": final_len,
        "clearance": min_clearance(resampled),
    }


def print_summary(res):
    spec = res["spec"]
    print(f"=== {spec['name']} ({res['id']}) ===")
    print(
        f"  GPX course: {res['raw_length'] / 1000:.2f} km, scale {spec['scale']:g}, "
        f"lap: {res['total_length']:.0f} m, {len(res['waypoints'])} waypoints"
    )
    print(
        f"  Smallest gap between two parts of the lap: {res['clearance']:.1f} m (width {spec['width']:g} m)"
    )
    if res["clearance"] < spec["width"] * 2.0:
        print(
            f"WARNING [{res['id']}]: two parts of the lap are closer than two track widths",
            file=sys.stderr,
        )


def write_track_json(res, tracks_dir=TRACKS_DIR):
    """Writes tracks/extreme_offroad/<id>.json (source waypoints only) and registers it in .track_order.json."""
    spec = res["spec"]
    path = os.path.join(tracks_dir, MODULE, f"{res['id']}.json")
    track = {
        "name": spec["name"],
        "description": spec["description"],
        "category": "main",
        "kind": {"type": "circuit"},
        "spline": {
            "waypoints": res["waypoints"],
            "closed": True,
            "samples": [],
            "total_length": 0.0,
            "curves": [],
        },
        "geometry": {
            "inner_walls": [],
            "outer_walls": [],
            "obstacles": [],
            "surface_zones": [],
            "jump_ramps": [],
            "left_boundary_polyline": [],
            "right_boundary_polyline": [],
        },
        "checkpoints": [],
        "grid_positions": [],
        "default_surface": "DeepSand",
        "pit_box_area": None,
        "default_laps": spec["laps"],
        "car_category": "off_road",
        "module_id": MODULE,
        "modules": [MODULE],
        "scale": "1:1" if spec["scale"] == 1.0 else f"{spec['scale']:g}x",
        "wikipedia_url": "https://en.wikipedia.org/wiki/Mint_400",
        "country_code": "US",
        "country_name": "United States",
        "is_inspired": False,
        "tag": spec["tag"],
        "category_label": "Desert Race",
    }
    with open(path, "w", encoding="utf-8") as f:
        json.dump(track, f, indent=2, ensure_ascii=False)

    order_path = os.path.join(tracks_dir, ".track_order.json")
    with open(order_path, "r", encoding="utf-8") as f:
        order = json.load(f)
    if res["id"] not in order.setdefault(MODULE, []):
        order[MODULE].append(res["id"])
        with open(order_path, "w", encoding="utf-8") as f:
            json.dump(order, f, indent=2, ensure_ascii=False)
    return (
        path,
        f"{BAKE_COMMAND} {os.path.relpath(path, REPO_ROOT)} --barrier-type TireWall",
    )


def main():
    parser = argparse.ArgumentParser(
        description=f"Build desert-race circuits from GPX files ({MINT_400_SOURCE})"
    )
    parser.add_argument(
        "--track",
        choices=list(GPX_TRACKS.keys()),
        help="Process one circuit (default: all)",
    )
    parser.add_argument(
        "--json", action="store_true", help="Write tracks/extreme_offroad/<id>.json"
    )
    args = parser.parse_args()
    for tid in [args.track] if args.track else list(GPX_TRACKS.keys()):
        res = process_gpx_track(tid)
        print_summary(res)
        if args.json:
            path, bake_cmd = write_track_json(res)
            print(f"  Wrote {path}")
            print(f"  Next: {bake_cmd}")


if __name__ == "__main__":
    main()
