#!/usr/bin/env python3
"""Build and bake the 3 new Rallycross circuits for Spec 051:
- silverstone_rx: restored from git history and baked
- spa_rx: Circuit de Spa-Francorchamps RX (Raidillon gravel climb & jump)
- erx_motor_park: ERX Motor Park RX (Nitrocross gap jump & banked clay bowl)
"""

import json
import math
import os
import subprocess

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TRACKS_DIR = os.path.join(REPO_ROOT, "tracks")
RALLY_DIR = os.path.join(TRACKS_DIR, "rally")
TRACK_BAKE = os.path.join(REPO_ROOT, "target", "debug", "track_bake")


def compute_curbs_and_widths(points, default_w=13.0, straight_w=14.0):
    n = len(points)
    waypoints = []
    for i in range(n):
        p_prev = points[(i - 1) % n]
        p_curr = points[i]
        p_next = points[(i + 1) % n]

        v1 = (p_curr[0] - p_prev[0], p_curr[1] - p_prev[1])
        v2 = (p_next[0] - p_curr[0], p_next[1] - p_curr[1])

        l1 = math.hypot(v1[0], v1[1])
        l2 = math.hypot(v2[0], v2[1])

        if l1 > 0 and l2 > 0:
            nv1 = (v1[0] / l1, v1[1] / l1)
            nv2 = (v2[0] / l2, v2[1] / l2)
            cross = nv1[0] * nv2[1] - nv1[1] * nv2[0]
            # Turn angle cross product: > 0.35 left, < -0.35 right
            left_curb = cross > 0.35
            right_curb = cross < -0.35
        else:
            left_curb = False
            right_curb = False

        # First 3 waypoints are start straight
        width = straight_w if i < 3 or i >= n - 1 else default_w

        waypoints.append({
            "point": [round(p_curr[0], 1), round(p_curr[1], 1)],
            "width": width,
            "left_curb": left_curb,
            "right_curb": right_curb,
            "surface": p_curr[2],  # surface string passed in tuple
            "elevation": round(p_curr[3], 1) if len(p_curr) > 3 else 0.0,
            "bank_angle": round(p_curr[4], 1) if len(p_curr) > 4 else 0.0,
            "left_wall": True,
            "right_wall": True,
            "left_wall_distance": None,
            "right_wall_distance": None,
            "left_runoff_surface": "Gravel",
            "right_runoff_surface": "Gravel",
        })
    return waypoints


def resample_polygon(points_with_surface, target_length, num_waypoints=28):
    """Resamples 2D polygon with surfaces to exact target length and uniform spacing."""
    # Compute perimeter
    cum_dist = [0.0]
    total = 0.0
    for i in range(len(points_with_surface)):
        p1 = points_with_surface[i]
        p2 = points_with_surface[(i + 1) % len(points_with_surface)]
        d = math.hypot(p2[0] - p1[0], p2[1] - p1[1])
        total += d
        cum_dist.append(total)

    scale_factor = target_length / total

    # Scale raw coordinates
    scaled_pts = [(p[0] * scale_factor, p[1] * scale_factor, p[2], p[3] if len(p) > 3 else 0.0) for p in points_with_surface]

    # Re-compute cumulative distance after scaling
    cum_dist = [0.0]
    total = 0.0
    for i in range(len(scaled_pts)):
        p1 = scaled_pts[i]
        p2 = scaled_pts[(i + 1) % len(scaled_pts)]
        d = math.hypot(p2[0] - p1[0], p2[1] - p1[1])
        total += d
        cum_dist.append(total)

    step = total / num_waypoints
    resampled = []

    for k in range(num_waypoints):
        target_d = k * step
        # Find segment
        idx = 0
        while idx < len(scaled_pts) and cum_dist[idx + 1] < target_d:
            idx += 1
        if idx >= len(scaled_pts):
            idx = len(scaled_pts) - 1

        seg_len = cum_dist[idx + 1] - cum_dist[idx]
        t = (target_d - cum_dist[idx]) / seg_len if seg_len > 1e-6 else 0.0
        p1 = scaled_pts[idx]
        p2 = scaled_pts[(idx + 1) % len(scaled_pts)]

        rx = p1[0] + t * (p2[0] - p1[0])
        ry = p1[1] + t * (p2[1] - p1[1])
        surf = p1[2] if t < 0.5 else p2[2]
        elev = p1[3] + t * (p2[3] - p1[3])
        resampled.append((rx, ry, surf, elev))

    # Rotate so that the start vector (point 0 to point 1) is aligned along +X
    dx = resampled[1][0] - resampled[0][0]
    dy = resampled[1][1] - resampled[0][1]
    angle = math.atan2(dy, dx)

    # Shift point 0 to (0, 0) and rotate by -angle
    ox, oy = resampled[0][0], resampled[0][1]
    final_pts = []
    cos_a = math.cos(-angle)
    sin_a = math.sin(-angle)

    for rx, ry, surf, elev in resampled:
        tx = rx - ox
        ty = ry - oy
        nx = tx * cos_a - ty * sin_a
        ny = tx * sin_a + ty * cos_a
        final_pts.append((nx, ny, surf, elev))

    return final_pts


def create_spa_rx():
    # Circuit de Spa-Francorchamps RX:
    # Starts below Eau Rouge on the straight, sweeps left through stadium hairpin,
    # charges up Raidillon crest, cuts right into the gravel bowl, jumps over tabletop,
    # loops around gravel bowl and descends back down to Eau Rouge.
    raw_outline = [
        # Start straight along +X (Asphalt)
        (0.0, 0.0, "Asphalt", 0.0),
        (50.0, 0.0, "Asphalt", 0.0),
        (100.0, -10.0, "Asphalt", 0.0),
        # T1/T2 Stadium loop below Eau Rouge (Asphalt)
        (130.0, -35.0, "Asphalt", 0.5),
        (135.0, -75.0, "Asphalt", 1.0),
        (105.0, -105.0, "Asphalt", 1.5),
        (65.0, -95.0, "Asphalt", 2.0),
        (45.0, -60.0, "Asphalt", 2.5),
        # Eau Rouge entry sweeping right and climbing (Asphalt)
        (40.0, -20.0, "Asphalt", 3.5),
        (65.0, 35.0, "Asphalt", 6.0),
        # Raidillon uphill sweep left-right (Asphalt, steep elevation rise)
        (90.0, 85.0, "Asphalt", 10.5),
        (120.0, 140.0, "Asphalt", 16.0),
        (145.0, 195.0, "Asphalt", 21.0),
        (160.0, 245.0, "Asphalt", 25.0),
        # Top of Raidillon crest -> sharp right cut into gravel bowl (Dirt)
        (150.0, 280.0, "Dirt", 26.5),
        (115.0, 295.0, "Dirt", 26.0),
        (75.0, 285.0, "Dirt", 24.5),
        # Arena jump crest on descent (Dirt)
        (40.0, 255.0, "Dirt", 21.5),
        (15.0, 220.0, "Dirt", 18.0),
        # Banked gravel carousel turnaround (Dirt)
        (0.0, 175.0, "Dirt", 14.5),
        (-10.0, 130.0, "Dirt", 11.0),
        (-5.0, 90.0, "Dirt", 8.0),
        # Fast descent transition to tarmac (Asphalt)
        (5.0, 55.0, "Asphalt", 5.0),
        (-15.0, 25.0, "Asphalt", 2.0),
        (-25.0, 5.0, "Asphalt", 0.5),
    ]

    pts = resample_polygon(raw_outline, target_length=1055.0, num_waypoints=28)
    waypoints = compute_curbs_and_widths(pts, default_w=13.0, straight_w=14.0)

    # Jump ramp at Raidillon descent: waypoint 17
    jp_idx = 17
    jp_p = waypoints[jp_idx]["point"]
    jp_next = waypoints[(jp_idx + 1) % 28]["point"]
    jdx = jp_next[0] - jp_p[0]
    jdy = jp_next[1] - jp_p[1]
    jlen = math.hypot(jdx, jdy)
    jdir = [round(jdx / jlen, 6), round(jdy / jlen, 6)]
    jangle = round(math.atan2(jdy, jdx), 2)

    jump_ramp = {
        "id": 1,
        "shape": {
            "OrientedBox": {
                "center": [round(jp_p[0] + jdir[0] * 3.0, 1), round(jp_p[1] + jdir[1] * 3.0, 1)],
                "half_extents": [3.8, 5.5],
                "angle": jangle,
            }
        },
        "direction": jdir,
        "launch_speed": 2.2,
        "ramp_angle_deg": 5.2,
        "height": 1.2,
        "name": "Raidillon Arena Dirt Jump",
        "surface": "Dirt",
    }

    track = {
        "name": "Spa-Francorchamps RX",
        "description": "World RX stadium circuit at Spa-Francorchamps charging backwards up the iconic Raidillon crest with banked gravel bowl and high-speed descent.",
        "category": "main",
        "kind": {"type": "circuit"},
        "default_surface": "Grass",
        "pit_box_area": None,
        "default_laps": 5,
        "car_category": "rally",
        "module_id": "rally",
        "modules": ["rally"],
        "scale": "1:1",
        "wikipedia_url": "https://en.wikipedia.org/wiki/Circuit_de_Spa-Francorchamps",
        "osm_url": "https://www.openstreetmap.org/way/234804574",
        "country_code": "BE",
        "country_name": "Belgium",
        "is_inspired": False,
        "tag": "WORLD RX SPA RAIDILLON",
        "category_label": "World RX",
        "spline": {
            "waypoints": waypoints,
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
            "jump_ramps": [jump_ramp],
            "left_boundary_polyline": [],
            "right_boundary_polyline": [],
        },
        "checkpoints": [],
        "grid_positions": [],
    }
    return track


def create_erx_motor_park():
    # ERX Motor Park RX (Elk River, MN):
    # Wide dirt/clay stadium amphitheater with the signature 70ft gap jump,
    # banked clay bowl turns, switchback esses, and launch pad tarmac section.
    raw_outline = [
        # Main dirt start straight along +X (Dirt)
        (0.0, 0.0, "Dirt", 0.0),
        (60.0, 0.0, "Dirt", 0.0),
        (120.0, -10.0, "Dirt", 0.0),
        # Sweeping 90° right Turn 1 into stadium valley (Dirt)
        (180.0, -35.0, "Dirt", -1.0),
        (225.0, -80.0, "Dirt", -2.0),
        # Straight approaching the Gap Jump (Dirt)
        (240.0, -140.0, "Dirt", -1.5),
        (230.0, -200.0, "Dirt", 0.0),
        # Steep banked clay bowl hairpin Turn 2/3 (Dirt)
        (200.0, -260.0, "Dirt", 1.5),
        (150.0, -290.0, "Dirt", 2.0),
        (95.0, -280.0, "Dirt", 1.5),
        (60.0, -240.0, "Dirt", 0.5),
        # Technical switchback esses (Dirt)
        (50.0, -185.0, "Dirt", 0.0),
        (75.0, -145.0, "Dirt", -0.5),
        (65.0, -100.0, "Dirt", 0.0),
        # Tarmac launch pad acceleration strip (Asphalt, ~25% section)
        (35.0, -80.0, "Asphalt", 0.0),
        (-15.0, -90.0, "Asphalt", 0.0),
        (-60.0, -115.0, "Asphalt", 0.0),
        (-105.0, -145.0, "Asphalt", 0.0),
        # Banked clay bowl turnaround Turn 5 (Dirt)
        (-145.0, -170.0, "Dirt", 0.5),
        (-180.0, -150.0, "Dirt", 1.0),
        (-195.0, -105.0, "Dirt", 1.0),
        (-180.0, -60.0, "Dirt", 0.5),
        # Clay rhythm section and return sweeper (Dirt)
        (-140.0, -35.0, "Dirt", 0.0),
        (-95.0, -20.0, "Dirt", 0.0),
        (-50.0, -10.0, "Dirt", 0.0),
    ]

    pts = resample_polygon(raw_outline, target_length=1152.0, num_waypoints=28)
    waypoints = compute_curbs_and_widths(pts, default_w=13.5, straight_w=14.5)

    # Jump ramp at waypoint 6 (approaching Turn 2)
    jp_idx = 6
    jp_p = waypoints[jp_idx]["point"]
    jp_next = waypoints[(jp_idx + 1) % 28]["point"]
    jdx = jp_next[0] - jp_p[0]
    jdy = jp_next[1] - jp_p[1]
    jlen = math.hypot(jdx, jdy)
    jdir = [round(jdx / jlen, 6), round(jdy / jlen, 6)]
    jangle = round(math.atan2(jdy, jdx), 2)

    jump_ramp = {
        "id": 1,
        "shape": {
            "OrientedBox": {
                "center": [round(jp_p[0] + jdir[0] * 3.0, 1), round(jp_p[1] + jdir[1] * 3.0, 1)],
                "half_extents": [4.0, 6.0],
                "angle": jangle,
            }
        },
        "direction": jdir,
        "launch_speed": 2.4,
        "ramp_angle_deg": 5.5,
        "height": 1.3,
        "name": "Nitrocross Stadium Gap Jump",
        "surface": "Dirt",
    }

    track = {
        "name": "ERX Motor Park RX",
        "description": "Premier Nitrocross all-dirt amphitheater in Minnesota featuring the iconic 70-foot stadium gap jump, banked clay switchbacks and rolling rhythm rollers.",
        "category": "main",
        "kind": {"type": "circuit"},
        "default_surface": "Grass",
        "pit_box_area": None,
        "default_laps": 5,
        "car_category": "rally",
        "module_id": "rally",
        "modules": ["rally"],
        "scale": "1:1",
        "wikipedia_url": "https://en.wikipedia.org/wiki/Nitrocross",
        "osm_url": "https://www.openstreetmap.org/way/1023648273",
        "country_code": "US",
        "country_name": "United States",
        "is_inspired": False,
        "tag": "NITROCROSS ERX GAP JUMP",
        "category_label": "Nitrocross",
        "spline": {
            "waypoints": waypoints,
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
            "jump_ramps": [jump_ramp],
            "left_boundary_polyline": [],
            "right_boundary_polyline": [],
        },
        "checkpoints": [],
        "grid_positions": [],
    }
    return track


def main():
    print("Building silverstone_rx.json from git history...")
    cmd = ["git", "-C", TRACKS_DIR, "show", "5a44422~1:rally/silverstone_rx.json"]
    proc = subprocess.run(cmd, capture_output=True, text=True, check=True)
    silverstone_data = json.loads(proc.stdout)
    silverstone_path = os.path.join(RALLY_DIR, "silverstone_rx.json")
    with open(silverstone_path, "w") as f:
        json.dump(silverstone_data, f, indent=2)
    print(f"  Wrote {silverstone_path}")

    print("Building spa_rx.json...")
    spa_data = create_spa_rx()
    spa_path = os.path.join(RALLY_DIR, "spa_rx.json")
    with open(spa_path, "w") as f:
        json.dump(spa_data, f, indent=2)
    print(f"  Wrote {spa_path}")

    print("Building erx_motor_park.json...")
    erx_data = create_erx_motor_park()
    erx_path = os.path.join(RALLY_DIR, "erx_motor_park.json")
    with open(erx_path, "w") as f:
        json.dump(erx_data, f, indent=2)
    print(f"  Wrote {erx_path}")

    # Run track_bake --rebuild on each file
    for p in [silverstone_path, spa_path, erx_path]:
        print(f"Baking {p}...")
        bake_cmd = [
            TRACK_BAKE,
            p,
            "--rebuild",
            "--barrier-offset", "3.5",
            "--barrier-type", "TireWall",
            "--checkpoints", "24",
        ]
        subprocess.run(bake_cmd, check=True)

    print("All 3 tracks built and baked successfully!")


if __name__ == "__main__":
    main()
