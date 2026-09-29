#!/usr/bin/env python3
"""
Builds the fictional Classic circuits (spec 055) from data into tracks/classic/<id>.json.

A circuit is a start pose and a list of segments, `Straight(length)` and `Arc(radius, degrees)`, drawn like
a turtle. Degrees > 0 turn left (towards the spline normal), < 0 turn right. Each segment can set the road
attributes of its waypoints; an attribute it leaves out takes the circuit's `Road` default. Width, elevation
and bank ease from the previous segment's end value to the segment's value; the other attributes apply to
every waypoint of the segment.

Ramps, whoops and surface zones are placed by lap distance (metres from the finish line) plus a lateral
offset, on the baked spline. So when a corner moves, they move with the road.

Two passes through track_bake:
  1. write the waypoints and metadata, then `track_bake --rebuild` (spline, walls, checkpoints, grid);
  2. place the features on the baked samples, then `track_bake` again, which validates and rewrites the file.
A file with validation errors is not written (track_bake exits 1).

    python3 scripts/classic_circuit_builder.py                 # build every circuit
    python3 scripts/classic_circuit_builder.py --only <id>     # build one (repeatable)
    python3 scripts/classic_circuit_builder.py --check         # build into a temp folder, compare with tracks/
"""

import argparse
import json
import math
import os
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field, fields, replace

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from osm_importer import REPO_ROOT, TRACKS_DIR

MODULE = "classic"
CLOSURE_TOLERANCE_M = 0.5
CLOSURE_TOLERANCE_DEG = 1.0
MIN_WAYPOINT_GAP_M = 3.2  # the validator errors below 3.0 m
MAX_ARC_STEP_DEG = 30.0

# ---------------------------------------------------------------------------------------------------------------
# Circuit data
# ---------------------------------------------------------------------------------------------------------------


@dataclass(frozen=True)
class Road:
    """Waypoint attributes. `Road` holds a circuit's defaults; a segment overrides some of them."""

    width: float = 12.0
    surface: str = "Asphalt"
    elevation: float = 0.0
    bank: float = 0.0  # degrees; > 0 raises the right side (a left-hand banked turn)
    left_curb: bool = False
    right_curb: bool = False
    left_wall: bool = True
    right_wall: bool = True
    left_wall_distance: float = 4.0  # road edge to wall, m
    right_wall_distance: float = 4.0
    wall_type: str = "Steel"
    left_runoff: str = "Grass"
    right_runoff: str = "Grass"


EASED = ("width", "elevation", "bank")
ROAD_FIELDS = tuple(f.name for f in fields(Road))


class Segment:
    def __init__(self, length, turn_deg, **road):
        unknown = (
            set(road)
            - set(ROAD_FIELDS)
            - {"curbs", "walls", "wall_distance", "runoff", "elev"}
        )
        if unknown:
            raise ValueError(
                f"unknown segment attribute(s): {', '.join(sorted(unknown))}"
            )
        # Shorthands that set both sides.
        for short, (left, right) in {
            "curbs": ("left_curb", "right_curb"),
            "walls": ("left_wall", "right_wall"),
            "wall_distance": ("left_wall_distance", "right_wall_distance"),
            "runoff": ("left_runoff", "right_runoff"),
        }.items():
            if short in road:
                value = road.pop(short)
                road.setdefault(left, value)
                road.setdefault(right, value)
        if "elev" in road:
            road["elevation"] = road.pop("elev")
        self.length = length
        self.turn_deg = turn_deg
        self.road = road


def Straight(length, **road):
    """A straight of `length` metres."""
    return Segment(float(length), 0.0, **road)


def Arc(radius, degrees, **road):
    """A constant-radius turn; degrees > 0 turn left, < 0 turn right."""
    return Segment(abs(math.radians(degrees)) * radius, float(degrees), **road)


@dataclass(frozen=True)
class Ramp:
    """A tabletop jump ramp whose entrance is at lap distance `at`."""

    at: float
    length: float = 12.0
    height: float = 2.0
    width: float = None  # default: road width there
    launch_speed: float = 4.0
    angle_deg: float = None  # default: from height and incline length
    surface: str = None  # default: road surface there
    name: str = ""


@dataclass(frozen=True)
class Whoops:
    """A row of small ramps (like presets.rs::generate_whoops_array), the first one starting at `at`."""

    at: float
    count: int = 6
    spacing: float = 5.0
    height: float = 0.6
    width: float = None
    surface: str = None
    name: str = "Whoops"


@dataclass(frozen=True)
class Zone:
    """A surface strip from lap distance `start` to `end`, between two lateral offsets.

    `from_edge=None`: offsets from the centre line, > 0 to the left. `from_edge="left"` or `"right"`: offsets
    outward from that road edge (0 = on the edge), so the strip follows width changes.
    """

    start: float
    end: float
    surface: str
    lateral: tuple
    from_edge: str = None
    layer: str = "below_track"
    name: str = ""


@dataclass(frozen=True)
class Spot:
    """A round surface zone centred at lap distance `at` and lateral offset `lateral` (> 0 left of centre)."""

    at: float
    lateral: float
    radius: float
    surface: str
    layer: str = "below_track"
    name: str = ""


@dataclass
class Circuit:
    id: str
    name: str
    description: str
    tag: str
    category_label: str
    car_category: str
    default_laps: int
    segments: list
    road: Road = field(default_factory=Road)
    features: list = field(default_factory=list)
    default_surface: str = "Grass"
    start: tuple = (0.0, 0.0)
    heading_deg: float = 0.0
    step: float = 10.0  # max waypoint spacing, m


# ---------------------------------------------------------------------------------------------------------------
# Waypoints
# ---------------------------------------------------------------------------------------------------------------


def smoothstep(t):
    return t * t * (3.0 - 2.0 * t)


def segment_steps(seg, step):
    """How many waypoints a segment gets (its end point included)."""
    n = math.ceil(seg.length / step)
    if seg.turn_deg:
        n = max(n, math.ceil(abs(seg.turn_deg) / MAX_ARC_STEP_DEG))
    n = min(n, max(1, math.floor(seg.length / MIN_WAYPOINT_GAP_M)))
    return max(1, n)


def trace(circuit):
    """Walks the segments. Returns (points [(x, y, heading_rad, lap_m, road)], end pose (x, y, heading_rad))."""
    x, y = circuit.start
    heading = math.radians(circuit.heading_deg)
    prev = circuit.road
    points = []
    lap = 0.0
    for seg in circuit.segments:
        target = replace(circuit.road, **seg.road)
        n = segment_steps(seg, circuit.step)
        turn = math.radians(seg.turn_deg)
        x0, y0, h0 = x, y, heading
        for i in range(1, n + 1):
            t = i / n
            if turn:
                radius = seg.length / abs(turn)
                sign = 1.0 if turn > 0 else -1.0
                # Centre of the turn sits on the left (sign > 0) or right of the start pose.
                cx = x0 - sign * radius * math.sin(h0)
                cy = y0 + sign * radius * math.cos(h0)
                h = h0 + turn * t
                px = cx + sign * radius * math.sin(h)
                py = cy - sign * radius * math.cos(h)
            else:
                h = h0
                px = x0 + seg.length * t * math.cos(h0)
                py = y0 + seg.length * t * math.sin(h0)
            e = smoothstep(t)
            eased = {
                k: getattr(prev, k) + (getattr(target, k) - getattr(prev, k)) * e
                for k in EASED
            }
            points.append((px, py, h, lap + seg.length * t, replace(target, **eased)))
        x, y, heading = points[-1][0], points[-1][1], points[-1][2]
        lap += seg.length
        prev = target
    return points, (x, y, heading)


def closure_gap(circuit):
    """(distance m, heading difference deg) between the end of the last segment and the start pose."""
    _points, (x, y, heading) = trace(circuit)
    sx, sy = circuit.start
    dh = math.degrees(heading) - circuit.heading_deg
    dh = (dh + 180.0) % 360.0 - 180.0
    return math.hypot(x - sx, y - sy), dh


def waypoints(circuit):
    """The Track JSON waypoints of a circuit. Raises ValueError when the lap does not close."""
    points, (x, y, _h) = trace(circuit)
    gap, dh = closure_gap(circuit)
    if gap > CLOSURE_TOLERANCE_M or abs(dh) > CLOSURE_TOLERANCE_DEG:
        raise ValueError(
            f"{circuit.id}: the lap does not close: end is {gap:.2f} m and {dh:+.2f} deg from the start "
            f"(limits {CLOSURE_TOLERANCE_M} m, {CLOSURE_TOLERANCE_DEG} deg)"
        )
    # The last point is the start again: drop it, and spread the small gap back along the lap.
    total = points[-1][3]
    ex, ey = x - circuit.start[0], y - circuit.start[1]
    out = []
    for px, py, _h, lap, road in points[:-1]:
        f = lap / total
        out.append(waypoint(px - ex * f, py - ey * f, road))
    sx, sy = circuit.start
    return [waypoint(sx, sy, points[-1][4])] + out


def waypoint(x, y, road):
    return {
        "point": [round(x, 2), round(y, 2)],
        "width": round(road.width, 2),
        "left_curb": road.left_curb,
        "right_curb": road.right_curb,
        "surface": road.surface,
        "elevation": round(road.elevation, 2),
        "bank_angle": round(road.bank, 2),
        "left_wall": road.left_wall,
        "right_wall": road.right_wall,
        "left_wall_distance": road.left_wall_distance,
        "right_wall_distance": road.right_wall_distance,
        "wall_type": road.wall_type,
        "left_runoff_surface": road.left_runoff,
        "right_runoff_surface": road.right_runoff,
    }


def source_track(circuit):
    """The circuit as a source file: metadata and waypoints; track_bake fills the rest."""
    return {
        "name": circuit.name,
        "description": circuit.description,
        "category": "main",
        "kind": {"type": "circuit"},
        "spline": {
            "waypoints": waypoints(circuit),
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
            "grandstands": [],
            "trees": [],
        },
        "checkpoints": [],
        "grid_positions": [],
        "default_surface": circuit.default_surface,
        "pit_box_area": None,
        "default_laps": circuit.default_laps,
        "car_category": circuit.car_category,
        "module_id": MODULE,
        "modules": [MODULE],
        "scale": "1:1",
        "is_inspired": False,
        "tag": circuit.tag,
        "category_label": circuit.category_label,
    }


# ---------------------------------------------------------------------------------------------------------------
# Features on the baked spline
# ---------------------------------------------------------------------------------------------------------------


class BakedSpline:
    """Looks up the baked samples by lap distance."""

    def __init__(self, track):
        self.samples = track["spline"]["samples"]
        self.total = track["spline"]["total_length"]
        self.dist = [s["distance"] for s in self.samples]

    def at(self, lap_m):
        """(point, tangent, normal, width, surface) at a lap distance, interpolated between samples."""
        d = lap_m % self.total
        lo, hi = 0, len(self.dist) - 1
        while lo < hi:  # last sample with distance <= d
            mid = (lo + hi + 1) // 2
            if self.dist[mid] <= d:
                lo = mid
            else:
                hi = mid - 1
        a = self.samples[lo]
        b = self.samples[(lo + 1) % len(self.samples)]
        span = (b["distance"] if lo + 1 < len(self.samples) else self.total) - a[
            "distance"
        ]
        t = 0.0 if span <= 0 else (d - a["distance"]) / span

        def lerp(key):
            return [
                a[key][0] + (b[key][0] - a[key][0]) * t,
                a[key][1] + (b[key][1] - a[key][1]) * t,
            ]

        tangent = unit(lerp("tangent"))
        normal = [-tangent[1], tangent[0]]  # left, as in spline.rs
        width = a["width"] + (b["width"] - a["width"]) * t
        return lerp("point"), tangent, normal, width, a["surface"] or "Asphalt"

    def offset_point(self, lap_m, lateral, from_edge=None):
        point, _t, normal, width, _s = self.at(lap_m)
        if from_edge == "left":
            lateral = width * 0.5 + lateral
        elif from_edge == "right":
            lateral = -(width * 0.5 + lateral)
        elif from_edge is not None:
            raise ValueError(
                f"from_edge must be None, 'left' or 'right', not {from_edge!r}"
            )
        return [point[0] + normal[0] * lateral, point[1] + normal[1] * lateral]


def unit(v):
    n = math.hypot(v[0], v[1]) or 1.0
    return [v[0] / n, v[1] / n]


def r3(v):
    return [round(v[0], 3), round(v[1], 3)]


def ramp_json(
    ramp_id, spline, at, length, height, width, launch_speed, angle_deg, surface, name
):
    point, tangent, _n, road_width, road_surface = spline.at(at + length * 0.5)
    angle = (
        angle_deg
        if angle_deg is not None
        else math.degrees(math.atan2(height, 0.85 * length))
    )
    return {
        "id": ramp_id,
        "shape": {
            "OrientedBox": {
                "center": r3(point),
                "half_extents": [
                    round(length * 0.5, 3),
                    round((width or road_width) * 0.5, 3),
                ],
                "angle": round(math.atan2(tangent[1], tangent[0]), 5),
            }
        },
        "direction": [round(tangent[0], 6), round(tangent[1], 6)],
        "launch_speed": launch_speed,
        "ramp_angle_deg": round(angle, 2),
        "height": height,
        "name": name,
        "surface": surface or road_surface,
    }


def place_features(track, circuit):
    """Adds the circuit's ramps and zones to a baked track (in place)."""
    spline = BakedSpline(track)
    ramps, zones = [], []
    for f in circuit.features:
        if isinstance(f, Ramp):
            name = f.name or f"Jump {len(ramps) + 1}"
            ramps.append(
                ramp_json(
                    len(ramps) + 1,
                    spline,
                    f.at,
                    f.length,
                    f.height,
                    f.width,
                    f.launch_speed,
                    f.angle_deg,
                    f.surface,
                    name,
                )
            )
        elif isinstance(f, Whoops):
            for i in range(f.count):
                ramps.append(
                    ramp_json(
                        len(ramps) + 1,
                        spline,
                        f.at + i * f.spacing,
                        f.spacing * 0.9,
                        f.height,
                        f.width,
                        3.5,
                        14.0,
                        f.surface,
                        f"{f.name} #{i + 1}",
                    )
                )
        elif isinstance(f, Zone):
            if f.end <= f.start:
                raise ValueError(f"{circuit.id}: zone {f.name!r} ends before it starts")
            n = max(2, math.ceil((f.end - f.start) / 2.0) + 1)
            ds = [f.start + (f.end - f.start) * i / (n - 1) for i in range(n)]
            near = [spline.offset_point(d, f.lateral[0], f.from_edge) for d in ds]
            far = [
                spline.offset_point(d, f.lateral[1], f.from_edge) for d in reversed(ds)
            ]
            zones.append(
                {
                    "shape": {"Polygon": {"vertices": [r3(p) for p in near + far]}},
                    "surface": f.surface,
                    "name": f.name or f"{f.surface} {f.start:.0f}-{f.end:.0f} m",
                    "layer": f.layer,
                }
            )
        elif isinstance(f, Spot):
            zones.append(
                {
                    "shape": {
                        "Circle": {
                            "center": r3(spline.offset_point(f.at, f.lateral)),
                            "radius": f.radius,
                        }
                    },
                    "surface": f.surface,
                    "name": f.name or f"{f.surface} at {f.at:.0f} m",
                    "layer": f.layer,
                }
            )
        else:
            raise TypeError(f"{circuit.id}: unknown feature {f!r}")
    track["geometry"]["jump_ramps"] = ramps
    track["geometry"]["surface_zones"] = zones


# ---------------------------------------------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------------------------------------------


def track_bake(paths, rebuild):
    cmd = ["cargo", "run", "--quiet", "--bin", "track_bake", "--", *paths]
    if rebuild:
        cmd.append("--rebuild")
    return subprocess.run(cmd, cwd=REPO_ROOT, check=False).returncode == 0


def write_json(path, data):
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)


def read_bytes(path):
    with open(path, "rb") as f:
        return f.read()


def register(circuits, tracks_dir):
    """Appends missing ids to the classic list of .track_order.json (build.rs needs every file listed)."""
    path = os.path.join(tracks_dir, ".track_order.json")
    with open(path, "r", encoding="utf-8") as f:
        order = json.load(f)
    listed = order.setdefault(MODULE, [])
    missing = [c.id for c in circuits if c.id not in listed]
    if missing:
        listed.extend(missing)
        write_json(path, order)


def build(circuits, tracks_dir):
    """Builds the circuits into <tracks_dir>/classic/. Returns True when every circuit validated."""
    if not circuits:
        print("no circuits to build")
        return True
    os.makedirs(os.path.join(tracks_dir, MODULE), exist_ok=True)
    paths = [os.path.join(tracks_dir, MODULE, f"{c.id}.json") for c in circuits]
    for circuit, path in zip(circuits, paths):
        write_json(path, source_track(circuit))
    register(circuits, tracks_dir)

    if not track_bake(paths, rebuild=True):
        print("pass 1 (track_bake --rebuild) failed", file=sys.stderr)
        return False
    for circuit, path in zip(circuits, paths):
        with open(path, "r", encoding="utf-8") as f:
            track = json.load(f)
        place_features(track, circuit)
        write_json(path, track)
    if not track_bake(paths, rebuild=False):
        print("pass 2 (track_bake) failed", file=sys.stderr)
        return False
    return True


def check(circuits):
    """Builds into a temp folder and compares with tracks/. Returns True when every file matches."""
    with tempfile.TemporaryDirectory() as tmp:
        shutil.copy(os.path.join(TRACKS_DIR, ".track_order.json"), tmp)
        if not build(circuits, tmp):
            return False
        ok = True
        for c in circuits:
            committed = os.path.join(TRACKS_DIR, MODULE, f"{c.id}.json")
            built = os.path.join(tmp, MODULE, f"{c.id}.json")
            if not os.path.exists(committed):
                print(f"{c.id}: missing in tracks/{MODULE}/", file=sys.stderr)
                ok = False
            elif read_bytes(committed) != read_bytes(built):
                print(
                    f"{c.id}: tracks/{MODULE}/{c.id}.json differs from the builder output",
                    file=sys.stderr,
                )
                ok = False
        with open(
            os.path.join(TRACKS_DIR, ".track_order.json"), "r", encoding="utf-8"
        ) as f:
            listed = json.load(f).get(MODULE, [])
        for c in circuits:
            if c.id not in listed:
                print(
                    f"{c.id}: not listed in tracks/.track_order.json", file=sys.stderr
                )
                ok = False
    return ok


# ---------------------------------------------------------------------------------------------------------------
# Circuits (added by the spec 055 circuit tasks)
# ---------------------------------------------------------------------------------------------------------------

CIRCUITS = []


def main(argv=None):
    parser = argparse.ArgumentParser(
        description="Build the Classic circuits of spec 055 into tracks/classic/"
    )
    parser.add_argument(
        "--only", action="append", metavar="ID", help="Build one circuit (repeatable)"
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Build into a temp folder and compare with tracks/",
    )
    args = parser.parse_args(argv)

    known = {c.id: c for c in CIRCUITS}
    if args.only:
        unknown = [i for i in args.only if i not in known]
        if unknown:
            parser.error(f"unknown circuit id(s): {', '.join(unknown)}")
        circuits = [known[i] for i in args.only]
    else:
        circuits = list(CIRCUITS)
    for c in circuits:
        gap, dh = closure_gap(c)
        length = sum(s.length for s in c.segments)
        print(
            f"{c.id}: {length:.0f} m of segments, closure gap {gap:.2f} m / {dh:+.2f} deg"
        )

    ok = check(circuits) if args.check else build(circuits, TRACKS_DIR)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
