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
import copy
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
MIN_INNER_WALL_RADIUS_M = 0.2
# No turn this close after a bridge ends (spec 055); a heading change above the limit counts as a turn.
BRIDGE_EXIT_STRAIGHT_M = 20.0
BRIDGE_EXIT_MAX_TURN_DEG = 5.0

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
    # (slots, spacing m, lateral stagger m) of the starting grid, laid out like
    # presets.rs::generate_grid_positions_at_distance. None: track_bake's module default.
    grid: tuple = None
    # [(where, elevation m), ...]: when set, elevation follows this profile (eased between points, wrapping
    # from the last point back to the first) instead of the segments' `elev`. `where` is a lap distance in m,
    # or (segment index, fraction of its length[, extra m]), which stays on that segment when lengths change.
    profile: list = None
    # (i, j): indices of two straights that are not parallel. Their lengths are solved so the lap closes;
    # the lengths written in `segments` are first guesses.
    close_with: tuple = None
    # (segment index, fraction of its length): where the finish line goes. None: at `start`. The lap is
    # still drawn from `start`; only the waypoint list is rotated, so `profile` and `close_with` keep
    # their meaning. Feature distances (Ramp.at, Zone.start, ...) count from the finish line.
    finish_at: tuple = None


# ---------------------------------------------------------------------------------------------------------------
# Waypoints
# ---------------------------------------------------------------------------------------------------------------


def smoothstep(t):
    return t * t * (3.0 - 2.0 * t)


def segments_of(circuit):
    """The circuit's segments, with the `close_with` straights solved so that the lap closes."""
    segments = [copy.copy(seg) for seg in circuit.segments]
    if not circuit.close_with:
        return segments
    i, j = circuit.close_with
    for k in (i, j):
        if segments[k].turn_deg:
            raise ValueError(f"{circuit.id}: close_with segment {k} is not a straight")
    heading = math.radians(circuit.heading_deg)
    headings = []
    for seg in segments:
        headings.append(heading)
        heading += math.radians(seg.turn_deg)
    ax, ay = math.cos(headings[i]), math.sin(headings[i])
    bx, by = math.cos(headings[j]), math.sin(headings[j])
    det = ax * by - ay * bx
    if abs(det) < 1e-6:
        raise ValueError(f"{circuit.id}: close_with straights {i} and {j} are parallel")
    # The end point moves linearly with the two lengths: solve end + da*a + db*b = start.
    _points, (x, y, _h) = trace(circuit, segments)
    gx, gy = x - circuit.start[0], y - circuit.start[1]
    segments[i].length += (-gx * by + gy * bx) / det
    segments[j].length += (-ax * gy + ay * gx) / det
    for k in (i, j):
        if segments[k].length <= 0.5:
            raise ValueError(
                f"{circuit.id}: closing the lap needs straight {k} to be {segments[k].length:.1f} m"
            )
    return segments


def check_segments(circuit, segments):
    """Stops on turns that break the wall generator.

    A turn is too tight when its inner wall line would fold over itself. The wall line is computed even
    where the wall is off, and presets.rs::untangle_polyline then drops the folded points, which shifts
    every later wall onto the wrong sample (wrong height, wrong on/off). So the inner wall line must keep
    a positive radius on every turn.
    """
    for k, seg in enumerate(segments):
        if not seg.turn_deg:
            continue
        road = replace(circuit.road, **seg.road)
        side = "left" if seg.turn_deg > 0 else "right"
        radius = seg.length / abs(math.radians(seg.turn_deg))
        inner = radius - road.width * 0.5 - getattr(road, f"{side}_wall_distance")
        if inner < MIN_INNER_WALL_RADIUS_M:
            raise ValueError(
                f"{circuit.id}: segment {k} (radius {radius:.1f} m) leaves {inner:.2f} m inside its "
                f"{side} wall line (at least {MIN_INNER_WALL_RADIUS_M} m); use a larger radius or a "
                f"smaller {side}_wall_distance"
            )
    check_inner_wall_steps(circuit, segments)


def check_inner_wall_steps(circuit, segments, reach_m=10.0, max_step_m=0.6):
    """Stops on an inner wall that steps in while the road keeps turning the same way.

    A kerbed turn keeps its walls 2.0 m out and cars drive on the kerb. When the next turn in the same
    direction, less than reach_m later, has its inner wall closer to the road, a car on the kerb line hits
    the step (bots got stuck there). Both turns need the same inner wall distance, or a straight between.
    """
    n = len(segments)
    for k, seg in enumerate(segments):
        if not seg.turn_deg:
            continue
        side = "left" if seg.turn_deg > 0 else "right"
        gap = getattr(replace(circuit.road, **seg.road), f"{side}_wall_distance")
        between = 0.0
        for j in range(1, n):
            nxt = segments[(k + j) % n]
            if not nxt.turn_deg:
                between += nxt.length
                if between >= reach_m:
                    break
                continue
            if (nxt.turn_deg > 0) != (seg.turn_deg > 0):
                break
            nxt_gap = getattr(
                replace(circuit.road, **nxt.road), f"{side}_wall_distance"
            )
            if gap - nxt_gap > max_step_m:
                raise ValueError(
                    f"{circuit.id}: the {side} (inner) wall steps in from {gap} m on segment {k} to {nxt_gap} m "
                    f"on segment {(k + j) % n}, {between:.1f} m later, while the road keeps turning; use the "
                    "same wall distance on both turns or a longer straight between"
                )
            break


def trace(circuit, segments=None):
    """Walks the segments (default: `segments_of(circuit)`).

    Returns (points, end pose). Points are spaced evenly along the whole lap (about `circuit.step`, at least
    3.2 m): track_bake draws a uniform Catmull-Rom spline through the waypoints, which kinks and turns
    tighter than the design wherever the spacing jumps. A point is (x, y, heading_rad, lap_m, road); it
    carries the settings of the segment it falls in (width, elevation and bank eased from the previous
    segment). The end pose (x, y, heading_rad) is the end of the last segment.
    """
    if segments is None:
        segments = segments_of(circuit)
    x, y = circuit.start
    heading = math.radians(circuit.heading_deg)
    prev = circuit.road
    pieces = []  # (lap start, segment, pose function, road before, road of the segment)
    lap = 0.0
    for seg in segments:
        turn = math.radians(seg.turn_deg)

        def pose(t, seg=seg, turn=turn, x0=x, y0=y, h0=heading):
            if not turn:
                return (
                    x0 + seg.length * t * math.cos(h0),
                    y0 + seg.length * t * math.sin(h0),
                    h0,
                )
            radius = seg.length / abs(turn)
            sign = 1.0 if turn > 0 else -1.0
            # Centre of the turn sits on the left (sign > 0) or right of the start pose.
            cx = x0 - sign * radius * math.sin(h0)
            cy = y0 + sign * radius * math.cos(h0)
            h = h0 + turn * t
            return cx + sign * radius * math.sin(h), cy - sign * radius * math.cos(h), h

        target = replace(circuit.road, **seg.road)
        pieces.append((lap, seg, pose, prev, target))
        x, y, heading = pose(1.0)
        lap += seg.length
        prev = target

    count = max(3, round(lap / circuit.step))
    if lap / count < MIN_WAYPOINT_GAP_M:
        count = max(3, math.floor(lap / MIN_WAYPOINT_GAP_M))
    points = []
    k = 0
    for i in range(count):
        d = lap * i / count
        while k + 1 < len(pieces) and pieces[k + 1][0] <= d:
            k += 1
        start, seg, pose, before, target = pieces[k]
        t = (d - start) / seg.length
        px, py, h = pose(t)
        e = smoothstep(t)
        eased = {
            key: getattr(before, key)
            + (getattr(target, key) - getattr(before, key)) * e
            for key in EASED
        }
        points.append((px, py, h, d, replace(target, **eased)))
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
    segments = segments_of(circuit)
    check_segments(circuit, segments)
    points, (x, y, _h) = trace(circuit, segments)
    gap, dh = closure_gap(circuit)
    if gap > CLOSURE_TOLERANCE_M or abs(dh) > CLOSURE_TOLERANCE_DEG:
        raise ValueError(
            f"{circuit.id}: the lap does not close: end is {gap:.2f} m and {dh:+.2f} deg from the start "
            f"(limits {CLOSURE_TOLERANCE_M} m, {CLOSURE_TOLERANCE_DEG} deg)"
        )
    # Spread the small closure gap back along the lap.
    total = sum(seg.length for seg in segments)
    if circuit.profile and any("elevation" in seg.road for seg in circuit.segments):
        raise ValueError(
            f"{circuit.id}: set elevation with `profile` or with segment `elev`, not both"
        )

    starts = [sum(seg.length for seg in segments[:k]) for k in range(len(segments))]

    def lap_of(where):
        if isinstance(where, tuple):
            k, frac, *extra = where
            return starts[k] + segments[k].length * frac + sum(extra)
        return where

    profile = [(lap_of(w), e) for w, e in circuit.profile] if circuit.profile else None

    def road_at(road, lap):
        if not profile:
            return road
        return replace(road, elevation=profile_at(profile, lap, total))

    ex, ey = x - circuit.start[0], y - circuit.start[1]
    out = []
    for px, py, _h, lap, road in points:
        f = lap / total
        out.append(waypoint(px - ex * f, py - ey * f, road_at(road, lap)))
    if circuit.finish_at:
        first = round(lap_of(circuit.finish_at) / total * len(out)) % len(out)
        out = out[first:] + out[:first]
    return out


def profile_at(profile, lap, total):
    """Elevation at a lap distance from [(lap m, elevation m), ...], eased between points."""
    pts = sorted(profile)
    lap = lap % total
    wrapped = (
        [(pts[-1][0] - total, pts[-1][1])] + pts + [(pts[0][0] + total, pts[0][1])]
    )
    for i in range(len(wrapped) - 1):
        (d0, e0), (d1, e1) = wrapped[i], wrapped[i + 1]
        if d0 <= lap <= d1:
            t = 0.0 if d1 == d0 else (lap - d0) / (d1 - d0)
            return e0 + (e1 - e0) * smoothstep(t)
    raise ValueError(f"lap distance {lap} outside the profile")


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

    def nearest(self, lap_m):
        """The baked sample nearest to a lap distance."""
        d = lap_m % self.total
        i = min(range(len(self.dist)), key=lambda k: abs(self.dist[k] - d))
        return self.samples[i]

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
    """Adds the circuit's ramps, zones and starting grid to a baked track (in place)."""
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
    if circuit.grid:
        track["grid_positions"] = grid_json(spline, *circuit.grid)
    track["checkpoints"] = fit_checkpoints(circuit, spline, track["checkpoints"])


GATE_REACH_M = 4.0  # how far generate_checkpoints puts a gate end past the road edge
GATE_SHIFTS_M = [0, 3, -3, 6, -6, 9, -9, 12, -12, 15, -15]


def fit_checkpoints(circuit, spline, checkpoints):
    """Checkpoint gates that end at the walls and touch no other part of the lap.

    Crossing a gate is tested in 2D only (checkpoint.rs), so a gate that reaches a close parallel run, or the
    road under or over a bridge, gives a false "wrong way" or a skipped checkpoint. A gate that touches another
    part slides along the lap (up to 15 m). The finish line does not move; if it touches, the build stops.
    """
    fitted = []
    for cp in checkpoints:
        shifts = [0] if cp["is_finish_line"] else GATE_SHIFTS_M
        for shift in shifts:
            lap = (cp["target_distance"] + shift) % spline.total
            gate = gate_at(spline, lap)
            if gate_is_clear(spline, lap, gate):
                break
        else:
            where = (
                "the finish line" if cp["is_finish_line"] else f"checkpoint {cp['id']}"
            )
            raise ValueError(
                f"{circuit.id}: {where} at {cp['target_distance']:.0f} m touches another part of the lap"
            )
        _p, tangent, _n, _w, _s = spline.at(lap)
        fitted.append(
            dict(
                cp,
                gate={"start": r3(gate[0]), "end": r3(gate[1])},
                direction=[round(tangent[0], 6), round(tangent[1], 6)],
                target_distance=round(lap, 3),
                elevation=spline.nearest(lap)["elevation"],
            )
        )
    return fitted


def gate_at(spline, lap):
    """(left end, right end) of a gate across the road at a lap distance, stopping at the walls."""
    s = spline.nearest(lap)
    reach = []
    for side in ("left", "right"):
        dist = s.get(f"{side}_wall_distance")
        wall = s.get(f"{side}_wall", True) and dist is not None
        reach.append(min(GATE_REACH_M, dist - 0.05) if wall else GATE_REACH_M)
    _p, _t, _n, width, _s = spline.at(lap)
    left = spline.offset_point(lap, width * 0.5 + reach[0])
    right = spline.offset_point(lap, -(width * 0.5 + reach[1]))
    return left, right


def gate_is_clear(spline, lap, gate, keep_out=0.3, own_part_m=10.0):
    """True when no point of the gate lies on the road of a part of the lap more than `own_part_m` away."""
    (ax, ay), (bx, by) = gate
    points = [(ax + (bx - ax) * i / 20, ay + (by - ay) * i / 20) for i in range(21)]
    for s in spline.samples:
        gap = abs(s["distance"] - lap)
        if min(gap, spline.total - gap) <= own_part_m:
            continue
        reach = s["width"] * 0.5 + keep_out
        sx, sy = s["point"]
        if any((px - sx) ** 2 + (py - sy) ** 2 < reach * reach for px, py in points):
            return False
    return True


def grid_json(spline, slots, spacing, stagger):
    """Grid slots behind the finish line (lap distance 0), as generate_grid_positions_at_distance lays them out."""
    grid = []
    for i in range(slots):
        point, tangent, _n, _w, _s = spline.at(-15.0 - i * spacing)
        side = -stagger if i % 2 == 0 else stagger
        right = [tangent[1], -tangent[0]]
        grid.append(
            {
                "position": r3(
                    [point[0] + right[0] * side, point[1] + right[1] * side]
                ),
                "angle": round(math.atan2(tangent[1], tangent[0]), 5),
                "grid_slot": i,
            }
        )
    return grid


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


def segments_cross(a, b, c, d):
    def side(p, q, r):
        return (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])

    return side(c, d, a) * side(c, d, b) < 0 and side(a, b, c) * side(a, b, d) < 0


def wall_gap(sample, side):
    """Distance from the road edge to the wall line of a baked sample, as presets.rs places it (pulled in on
    bridges)."""
    lift = min(max(sample["elevation"] / 3.0, 0.0), 1.0) if sample["is_bridge"] else 0.0
    bridge_gap = (1.35 if sample["left_curb"] or sample["right_curb"] else 0.75) + 0.5
    return sample[f"{side}_wall_distance"] * (1.0 - lift) + bridge_gap * lift


def wall_line_folds(track, max_span=40):
    """Where the raw wall lines of a baked track fold over themselves: [(side, lap m), ...].

    Mirrors presets.rs: the wall line of each sample sits `width / 2 + wall distance` from the centre (pulled
    in on bridges), and untangle_polyline looks for crossings up to 40 samples apart. It removes those
    points, which shifts every later wall onto the wrong sample (bug tdrace-classic-circuits-revamp-dh6k.21),
    so a circuit must not have any.
    """
    samples = track["spline"]["samples"]
    n = len(samples)
    folds = []
    for side, sign in (("left", 1.0), ("right", -1.0)):
        pts = []
        for s in samples:
            half = s["width"] * 0.5 + wall_gap(s, side)
            pts.append(
                (
                    s["point"][0] + sign * s["normal"][0] * half,
                    s["point"][1] + sign * s["normal"][1] * half,
                )
            )
        span_max = min(max_span, n // 2)
        last = None
        for i in range(n):
            a, b = pts[i], pts[(i + 1) % n]
            for span in range(2, span_max + 1):
                j = (i + span) % n
                if (j + 1) % n == i:
                    continue
                if segments_cross(a, b, pts[j], pts[(j + 1) % n]):
                    lap = samples[i]["distance"]
                    if last is None or lap - last > 5.0:
                        folds.append((side, lap))
                    last = lap
                    break
    return folds


def turns_after_bridges(track):
    """Bridges that end too close to a turn: [(lap m where the bridge ends, heading change deg), ...].

    A bridge is a run of baked `is_bridge` samples (spline.rs flags the raised run while it is 1.2 m or
    more above the ground). Within BRIDGE_EXIT_STRAIGHT_M after its last sample, the road heading must not
    change by more than BRIDGE_EXIT_MAX_TURN_DEG.
    """
    samples = track["spline"]["samples"]
    n = len(samples)

    def heading(s):
        return math.atan2(s["tangent"][1], s["tangent"][0])

    found = []
    for i in range(n):
        if not samples[i]["is_bridge"] or samples[(i + 1) % n]["is_bridge"]:
            continue
        h0, k, walked, worst = heading(samples[i]), i, 0.0, 0.0
        while True:
            nxt = (k + 1) % n
            walked += math.dist(samples[k]["point"], samples[nxt]["point"])
            if walked > BRIDGE_EXIT_STRAIGHT_M:
                break
            k = nxt
            turn = abs((heading(samples[k]) - h0 + math.pi) % (2 * math.pi) - math.pi)
            worst = max(worst, math.degrees(turn))
        if worst > BRIDGE_EXIT_MAX_TURN_DEG:
            found.append((samples[i]["distance"], worst))
    return found


def heading_of(sample):
    return math.atan2(sample["tangent"][1], sample["tangent"][0])


def heading_change_deg(a, b):
    return math.degrees(
        abs((heading_of(b) - heading_of(a) + math.pi) % (2 * math.pi) - math.pi)
    )


def turns_under_bridges(track):
    """Places where the lower road turns under a bridge: [(lap m, heading change deg), ...].

    A lower sample is under a bridge when a bridge sample 2.5 m or more above it is closer than the two half
    widths plus 1 m. Along each such run, the lower road heading must not change by more than
    BRIDGE_EXIT_MAX_TURN_DEG: a car under a deck is hidden, so it must not have to turn there.
    """
    samples = track["spline"]["samples"]
    cell = 6.0
    grid = {}
    for s in samples:
        if s["is_bridge"]:
            key = (int(s["point"][0] // cell), int(s["point"][1] // cell))
            grid.setdefault(key, []).append(s)

    def under(s):
        cx, cy = int(s["point"][0] // cell), int(s["point"][1] // cell)
        for dx in (-1, 0, 1):
            for dy in (-1, 0, 1):
                for b in grid.get((cx + dx, cy + dy), ()):
                    if (
                        b["elevation"] - s["elevation"] >= 2.5
                        and math.dist(b["point"], s["point"])
                        < (b["width"] + s["width"]) * 0.5 + 1.0
                    ):
                        return True
        return False

    flags = [not s["is_bridge"] and under(s) for s in samples]
    found, i, n = [], 0, len(samples)
    while i < n:
        if not flags[i]:
            i += 1
            continue
        j = i
        while j + 1 < n and flags[j + 1]:
            j += 1
        worst = max(heading_change_deg(samples[i], samples[k]) for k in range(i, j + 1))
        if worst > BRIDGE_EXIT_MAX_TURN_DEG:
            found.append((samples[i]["distance"], worst))
        i = j + 1
    return found


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
    ok = True
    for circuit, path in zip(circuits, paths):
        with open(path, "r", encoding="utf-8") as f:
            track = json.load(f)
        folds = wall_line_folds(track)
        if folds:
            where = ", ".join(f"{side} at {lap:.0f} m" for side, lap in folds)
            print(
                f"{circuit.id}: wall line folds ({where}); widen those turns",
                file=sys.stderr,
            )
            ok = False
            continue
        problems = []
        for lap, deg in turns_under_bridges(track):
            problems.append(
                f"the road turns {deg:.0f} deg under a bridge at {lap:.0f} m"
            )
        if problems:
            print(f"{circuit.id}: " + "; ".join(problems), file=sys.stderr)
            ok = False
            continue
        exits = turns_after_bridges(track)
        if exits:
            where = ", ".join(f"at {lap:.0f} m ({deg:.0f} deg)" for lap, deg in exits)
            print(
                f"{circuit.id}: a bridge ends less than {BRIDGE_EXIT_STRAIGHT_M:.0f} m before a turn ({where}); "
                "end the raised run on a straight",
                file=sys.stderr,
            )
            ok = False
            continue
        place_features(track, circuit)
        write_json(path, track)
    if not ok:
        return False
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

S, A = Straight, Arc

# Karting: packed indoor halls with bridges (spec 055, User Flow section 3). Concrete hall floor, tyre walls
# 0.6 m from the road, kart grid (10 slots, 5.5 m apart, 1.8 m stagger). Bridge decks sit at 4.4 m.


def indoor_kart_road(width):
    return Road(
        width=width,
        wall_type="TireWall",
        left_wall_distance=0.6,
        right_wall_distance=0.6,
        left_runoff="Concrete",
        right_runoff="Concrete",
    )


KART_GRID = (10, 5.5, 1.8)
KART_STEP = 3.3  # even waypoint spacing keeps the tight turns true to the design
# A kerb widens the road the validator checks by 1.35 m on both sides, so kerbed pieces keep their walls
# 2.0 m out (still 1.3 m where the blend meets a 0.6 m piece). Kerbed turns need a radius of about
# 1.5 x (half width + 2.0 m) so their inner wall line does not fold.
KERB_L = {"left_curb": True, "wall_distance": 2.0}
KERB_R = {"right_curb": True, "wall_distance": 2.0}
# Turn radii. track_bake's uniform Catmull-Rom spline, with waypoints at least 3 m apart, bakes a tight turn
# about 30 % tighter than its design radius, and the inner wall line must not fold (wall_line_folds;
# bug tdrace-classic-circuits-revamp-dh6k.21). So: hairpins and U-turns 6.5 m with the inner wall 0.3 m
# out, plain turns 7.5 m, kerbed turns 9 m on an 8 m road and 8 m on a 7 m road, chicanes 12 m.
TIP_L = {"left_wall_distance": 0.3}
TIP_R = {"right_wall_distance": 0.3}

HANGAR_SPRINT = Circuit(
    id="kart_hangar_sprint",
    name="Hangar Sprint",
    description="Indoor figure-eight with one bridge, a tight hairpin pocket and a chicane.",
    tag="INDOOR SPRINT",
    category_label="Indoor Kart",
    car_category="kart",
    default_laps=8,
    default_surface="Concrete",
    road=indoor_kart_road(8.0),
    grid=KART_GRID,
    step=KART_STEP,
    heading_deg=90.0,
    segments=[
        S(21.5),  # 0: pocket, up leg, heading north; finish at its start
        A(7.5, 90),
        S(1),
        A(9, 60, **KERB_L),
        S(50),  # 4: diagonal under the bridge (solved)
        # West loop, clockwise.
        A(9, -60),  # 5: no kerb: the right wall would step in on turn 7
        S(2.5),
        A(7.5, -90, **TIP_R),
        S(3.5),
        A(13, -25, **KERB_R),  # chicane
        A(13, 50, **KERB_L),  # 10
        A(13, -25),  # no kerb: the right wall would step in on turn 13
        S(3.5),
        A(9, -90),  # on the ramp: bridge walls sit 1.25 m out here
        S(1),
        A(9, -60),  # 15: onto the bridge, no kerb on the deck
        S(33),  # 16: the bridge (solved)
        # Down into the east loop.
        A(9, 60),  # off the bridge
        S(5),
        A(10, 20),  # kink
        A(10, -40),  # 20
        A(10, 20),
        S(5),
        A(9, 90, **KERB_L),
        S(29),  # start straight, heading north
        A(6.5, 90, **TIP_L),  # 25: U-turn into the pocket
        A(6.5, 90, **TIP_L),
        S(19.5),  # pocket, down leg
        A(6.5, -180, **TIP_R),  # 28: pocket hairpin
    ],
    close_with=(4, 16),
    # Deck at 4.4 m over the middle of straight 16; up over about 60 m. Down to 1.5 m through the east
    # loop, then to the ground on the first 20 m of the start straight, so the bridge ends 20 m or more
    # before the U-turn (grade < 12 %). The grid sits on the flat pocket and start straight.
    profile=[
        (0, 0),
        ((16, 0.5, -75), 0),
        ((16, 0.5, -12), 4.4),
        ((16, 0.5, 12), 4.4),
        ((24, 0), 1.5),
        ((24, 0, 20), 0),
    ],
)

WAREHOUSE_TWISTER = Circuit(
    id="kart_warehouse_twister",
    name="Warehouse Twister",
    description="Three hairpin pockets; the long one dives twice under the raised back straight.",
    tag="TWIN BRIDGE",
    category_label="Indoor Kart",
    car_category="kart",
    default_laps=7,
    default_surface="Concrete",
    road=indoor_kart_road(7.0),
    grid=KART_GRID,
    step=KART_STEP,
    heading_deg=-90.0,
    segments=[
        S(31),  # 0: long pocket, down leg, heading south
        A(6.5, -180, **TIP_R),  # below the raised bottom side
        S(50),
        A(6.5, 90, **TIP_L),
        A(6.5, 90, **TIP_L),  # 4-8: short pocket
        S(20),
        A(6.5, -180, **TIP_R),
        S(20),
        A(6.5, 90, **TIP_L),
        A(6.5, 90, **TIP_L),  # 9-13: short pocket; the ramp starts on its up leg (12)
        S(20),
        A(6.5, -180, **TIP_R),
        S(20),
        A(6.5, 90, **TIP_L),
        A(6.5, 90),
        S(4),  # 15: west side (solved), ramp up
        A(12, 25, **KERB_L),  # chicane
        A(12, -50, **KERB_R),
        A(12, 25),  # no kerb: the left wall would step in on turn 20
        S(4),
        A(7.5, 90),  # 20
        S(62),  # 21: bottom side, raised (solved)
        A(7.5, 90),
        S(4),  # east side, ramp down
        A(12, 25, **KERB_L),  # chicane, bulging in (keeps the box)
        A(12, -50, **KERB_R),  # 25
        A(12, 25),  # no kerb: the left wall would step in on turn 28
        S(4),
        A(6.5, 90),  # 28
        S(1),  # top side
        A(6.5, 90, **TIP_L),  # 30: into the long pocket
        S(19),  # 31: long pocket, down leg
    ],
    close_with=(15, 21),
    finish_at=(
        10,
        0,
    ),  # start of the last short pocket: the grid sits on the flat first one
    # Bottom side at 4.4 m. Up from the middle of the last short pocket's up leg (about 60 m). Down to 2.0 m
    # along the east side and the top side, then to the ground on the first 12 m of the long pocket's down
    # leg, before it dives under the bottom side. So the bridge ends on the down leg, 20 m or more before
    # its hairpin (grade < 12 %).
    profile=[
        ((0, 0.4), 0),
        ((12, 0.5), 0),
        ((21, 0), 4.4),
        ((21, 1), 4.4),
        ((31, 0), 2.0),
    ],
)

TOWER_LABYRINTH = Circuit(
    id="kart_tower_labyrinth",
    name="Tower Labyrinth",
    description="A raised run bridges a crossover and both legs of a deep pocket; double hairpin, tightening turn.",
    tag="TRIPLE DECK",
    category_label="Indoor Kart",
    car_category="kart",
    default_laps=6,
    default_surface="Concrete",
    road=indoor_kart_road(6.5),
    grid=KART_GRID,
    step=KART_STEP,
    heading_deg=-90.0,
    segments=[
        S(35),  # 0: long pocket, down leg, heading south
        A(6.5, -180, **TIP_R),  # below the raised bottom side
        S(47),  # 2: up leg
        # Double hairpin: two short pockets in a row.
        A(6.5, 90, **TIP_L),
        A(6.5, 90, **TIP_L),
        S(22),  # 5
        A(6.5, -180, **TIP_R),
        S(22),
        A(6.5, 90, **TIP_L),
        A(6.5, 90, **TIP_L),
        S(22),  # 10
        A(6.5, -180, **TIP_R),
        S(22),
        A(6.5, 90),
        S(0.5),
        A(8, 70, **KERB_L),  # 15
        S(46),  # 16: crossover, ground
        # West loop, clockwise.
        A(8, -70),  # no kerb: the right wall would step in on turn 19
        S(0.5),
        A(7.5, -90),
        S(14),  # 20: ramp starts here
        A(12, -25, **KERB_R),  # chicane
        A(12, 50, **KERB_L),
        A(12, -25, **KERB_R),
        S(14),  # 24 (solved)
        A(7.5, -90),  # 25
        S(0.5),
        A(8, -70),  # onto the raised run: no kerb
        S(46),  # 28: crossover, bridge
        A(8, 70),
        S(40),  # 30: bottom side, raised, over the long pocket (solved)
        A(10, 45),  # tightening corner
        A(6.5, 45),
        S(8),  # east side, ramp down
        A(10, 20),  # kink
        A(10, -40),
        A(10, 20),
        S(8),
        A(6.5, 90, **TIP_L),  # U-turn into the long pocket
        A(6.5, 90, **TIP_L),  # 39
        S(12),  # 40: long pocket, down leg
    ],
    close_with=(24, 30),
    finish_at=(
        10,
        0,
    ),  # start of the last short pocket: the grid sits on the flat first one
    # One raised run at 4.4 m from the crossover to the tightening corner. Up over about 60 m. Down over
    # about 90 m, through the U-turn and 20 m into the long pocket, so the bridge ends on the pocket's down
    # leg, 20 m or more before its hairpin (grade < 12 %).
    profile=[
        ((0, 0, 20), 0),
        ((20, 0), 0),
        ((28, 0.5, -12), 4.4),
        ((31, 0.5), 4.4),
    ],
)

CIRCUITS = [HANGAR_SPRINT, WAREHOUSE_TWISTER, TOWER_LABYRINTH]


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
        length = sum(s.length for s in segments_of(c))
        print(
            f"{c.id}: {length:.0f} m of segments, closure gap {gap:.2f} m / {dh:+.2f} deg"
        )

    ok = check(circuits) if args.check else build(circuits, TRACKS_DIR)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
