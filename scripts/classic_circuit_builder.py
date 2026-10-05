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

# Density generation modes
MODE_ANGULAR_45 = "angular_45"        # Approach 1 (Adopted default): ~45° per curve knot, straight endpoints
MODE_RADIAL_30 = "radial_30"          # Approach 2 (Alternative): ~30° per curve knot, 60m straight subdivisions
MODE_APEX_CAD = "apex_cad"            # Approach 3 (Alternative): Canonical apex-centered CAD knots
MODE_CHORD_SAGITTA = "chord_sagitta"  # Alternative: Chord-sagitta polygon approximation (<= 0.08m)
MODE_LEGACY_UNIFORM = "legacy_uniform"# Alternative: Legacy uniform step spacing

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


TREE_TRUNK_RADII = {
    "pine": 0.35,
    "palm": 0.28,
    "oak": 0.48,
    "cypress": 0.24,
    "sakura": 0.35,
    "autumn_maple": 0.40,
    "bush": 0.0,
    "cactus": 0.22,
    "snow_pine": 0.35,
}

ROCK_RADII = {
    "granite": 1.2,
    "sandstone": 1.4,
    "slate": 1.1,
    "snow_capped": 1.3,
}


@dataclass(frozen=True)
class GrandstandProp:
    at: float
    side: str = "right"
    offset: float = None
    length: float = 30.0
    depth: float = 8.0
    style: str = "open_bleachers"
    tiers: int = 6
    seat_color: list = None
    elevation: float = None


@dataclass(frozen=True)
class BuildingProp:
    at: float
    side: str = "right"
    offset: float = None
    width: float = 24.0
    depth: float = 10.0
    style: str = "pit_garage"
    roof_color: list = None
    elevation: float = None
    angle: float = None


@dataclass(frozen=True)
class RockProp:
    at: float
    side: str = "right"
    offset: float = None
    rock_type: str = "granite"
    scale: float = 1.0
    rotation: float = 0.0
    elevation: float = None


@dataclass(frozen=True)
class TreeProp:
    at: float
    side: str = "right"
    offset: float = None
    tree_type: str = "pine"
    scale: float = 1.0
    rotation: float = 0.0
    elevation: float = None


def tree_row(start, end, count, side="right", offset=None, tree_type="pine", scale=1.0):
    if count <= 0:
        return []
    if count == 1:
        return [TreeProp(at=(start + end) * 0.5, side=side, offset=offset, tree_type=tree_type, scale=scale)]
    step = (end - start) / (count - 1)
    return [
        TreeProp(
            at=round(start + i * step, 2),
            side=side,
            offset=None if offset is None else round(offset + (0.4 if i % 2 == 1 else -0.3), 2),
            tree_type=tree_type,
            scale=round(scale * (0.9 + 0.2 * (i % 3 == 0)), 2),
            rotation=round(i * 0.73, 3),
        )
        for i in range(count)
    ]


def rock_cluster(at, count, side="right", offset=None, rock_type="granite", scale=1.0, span=14.0):
    if count <= 0:
        return []
    if count == 1:
        return [RockProp(at=at, side=side, offset=offset, rock_type=rock_type, scale=scale)]
    step = span / (count - 1)
    start = at - span * 0.5
    return [
        RockProp(
            at=round(start + i * step, 2),
            side=side,
            offset=None if offset is None else round(offset + (0.6 if i % 2 == 1 else -0.5), 2),
            rock_type=rock_type,
            scale=round(scale * (0.85 + 0.25 * (i % 2 == 0)), 2),
            rotation=round(i * 1.15, 3),
        )
        for i in range(count)
    ]


def flatten_features(features):
    flat = []
    for f in features:
        if isinstance(f, (list, tuple)):
            flat.extend(flatten_features(f))
        else:
            flat.append(f)
    return flat


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
    car_model_id: str = None


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
        min_radius = road.width * 0.5 + 1.0
        if radius < min_radius:
            raise ValueError(
                f"{circuit.id}: segment {k} (radius {radius:.1f} m) is below minimum centerline radius "
                f"w/2 + 1.0 m ({min_radius:.1f} m)"
            )
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


def trace(circuit, segments=None, density_mode=MODE_ANGULAR_45):
    """Walks the segments (default: `segments_of(circuit)`).

    Density generation modes:
      - MODE_ANGULAR_45 (default, Approach 1): Angular knot budget (~45° per curve knot, straight endpoints).
      - MODE_RADIAL_30 (Approach 2): Spline radial error budget (~30° per curve knot, 60m straight spans).
      - MODE_APEX_CAD (Approach 3): Canonical apex-centered CAD knots (entry + geometric apexes).
      - MODE_CHORD_SAGITTA: Chord sagitta error budget (<= 0.08m).
      - MODE_LEGACY_UNIFORM: Uniform arc-length stepping based on circuit.step.

    Returns (points, end pose). A point is (x, y, heading_rad, lap_m, road); it
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

    points = []
    if density_mode == MODE_LEGACY_UNIFORM:
        count = max(3, round(lap / circuit.step))
        if lap / count < MIN_WAYPOINT_GAP_M:
            count = max(3, math.floor(lap / MIN_WAYPOINT_GAP_M))
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
    else:
        profile_ts_per_piece = [[] for _ in pieces]
        if circuit.profile:
            for where, _elev in circuit.profile:
                if isinstance(where, tuple):
                    pk, pfrac, *pextra = where
                    if 0 <= pk < len(pieces) and 0.0 <= pfrac < 1.0:
                        profile_ts_per_piece[pk].append(float(pfrac))

        for k, (start, seg, pose, before, target) in enumerate(pieces):
            turn = math.radians(seg.turn_deg)
            varies = any(getattr(before, f) != getattr(target, f) for f in EASED)
            if not turn:
                # Straight segment
                if varies and seg.length > 25.0:
                    num = max(1, round(seg.length / 25.0))
                    step_ts = [i / num for i in range(num)]
                elif density_mode == MODE_RADIAL_30:
                    num = max(1, round(seg.length / 60.0))
                    step_ts = [i / num for i in range(num)]
                elif density_mode == MODE_CHORD_SAGITTA:
                    if seg.length > 50.0:
                        num = max(1, round(seg.length / 45.0))
                        step_ts = [i / num for i in range(num)]
                    else:
                        step_ts = [0.0]
                elif seg.length > 80.0:
                    num = max(1, round(seg.length / 80.0))
                    step_ts = [i / num for i in range(num)]
                else:  # MODE_ANGULAR_45 (default) and MODE_APEX_CAD
                    step_ts = [0.0]

                prev_seg = pieces[(k - 1) % len(pieces)][1]
                if abs(prev_seg.turn_deg) > 0.0 and seg.length > 35.0:
                    exit_t = 20.0 / seg.length
                    if exit_t < (step_ts[1] if len(step_ts) > 1 else 1.0) - 10.0 / seg.length:
                        step_ts.append(exit_t)

                next_seg = pieces[(k + 1) % len(pieces)][1]
                if abs(next_seg.turn_deg) > 0.0 and seg.length > 35.0:
                    approach_t = (seg.length - 20.0) / seg.length
                    if approach_t > step_ts[-1] + 10.0 / seg.length:
                        step_ts.append(approach_t)
            else:
                # Curved segment
                deg = abs(seg.turn_deg)
                side = "left" if seg.turn_deg > 0 else "right"
                wall_d = getattr(target, f"{side}_wall_distance")
                radius = seg.length / abs(turn)
                inner_r = radius - target.width * 0.5 - wall_d

                if density_mode == MODE_ANGULAR_45:
                    budget = 30.0 if (wall_d >= 12.0 or inner_r < 18.0) else 45.0
                    num = max(2 if deg >= 30.0 else 1, math.ceil(deg / budget))
                    num = max(num, math.ceil(seg.length / 50.0))
                    if seg.length / num < MIN_WAYPOINT_GAP_M:
                        num = max(1, math.floor(seg.length / MIN_WAYPOINT_GAP_M))
                    step_ts = [i / num for i in range(num)]
                elif density_mode == MODE_RADIAL_30:
                    num = max(1, math.ceil(deg / 30.0))
                    if seg.length / num < MIN_WAYPOINT_GAP_M:
                        num = max(1, math.floor(seg.length / MIN_WAYPOINT_GAP_M))
                    step_ts = [i / num for i in range(num)]
                elif density_mode == MODE_APEX_CAD:
                    if deg <= 90.0:
                        step_ts = [0.0, 0.5]
                    else:
                        step_ts = [0.0, 1.0 / 3.0, 2.0 / 3.0]
                elif density_mode == MODE_CHORD_SAGITTA:
                    num_sub = max(1, round(deg / 20.0))
                    d_theta = math.radians(deg) / num_sub
                    sagitta = radius * (1.0 - math.cos(d_theta / 2.0))
                    if sagitta > 0.08 and radius > 1.0:
                        max_d_theta = 2.0 * math.acos(max(-1.0, 1.0 - 0.08 / radius))
                        if max_d_theta > 1e-4:
                            num_sub = max(num_sub, math.ceil(math.radians(deg) / max_d_theta))
                    if seg.length / num_sub < MIN_WAYPOINT_GAP_M:
                        num_sub = max(1, math.floor(seg.length / MIN_WAYPOINT_GAP_M))
                    step_ts = [i / num_sub for i in range(num_sub)]

            if profile_ts_per_piece[k]:
                for pt in profile_ts_per_piece[k]:
                    if 0.0 <= pt < 1.0:
                        step_ts.append(pt)
            step_ts = sorted(list(set(step_ts)))

            # Filter step_ts to enforce MIN_WAYPOINT_GAP_M between steps
            valid_ts = [step_ts[0]]
            for t in step_ts[1:]:
                if (t - valid_ts[-1]) * seg.length >= MIN_WAYPOINT_GAP_M:
                    valid_ts.append(t)
            step_ts = valid_ts

            for t in step_ts:
                d = start + seg.length * t
                px, py, h = pose(t)
                e = smoothstep(t)
                eased = {
                    key: getattr(before, key)
                    + (getattr(target, key) - getattr(before, key)) * e
                    for key in EASED
                }
                points.append((px, py, h, d, replace(target, **eased)))

    return points, (x, y, heading)


def closure_gap(circuit, density_mode=MODE_ANGULAR_45):
    """(distance m, heading difference deg) between the end of the last segment and the start pose."""
    _points, (x, y, heading) = trace(circuit, density_mode=density_mode)
    sx, sy = circuit.start
    dh = math.degrees(heading) - circuit.heading_deg
    dh = (dh + 180.0) % 360.0 - 180.0
    return math.hypot(x - sx, y - sy), dh


def waypoints(circuit, density_mode=MODE_ANGULAR_45):
    """The Track JSON waypoints of a circuit. Raises ValueError when the lap does not close."""
    segments = segments_of(circuit)
    check_segments(circuit, segments)
    points, (x, y, _h) = trace(circuit, segments, density_mode=density_mode)
    gap, dh = closure_gap(circuit, density_mode=density_mode)
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
        target_lap = lap_of(circuit.finish_at) % total
        first = min(
            range(len(out)),
            key=lambda i: min(
                abs(points[i][3] - target_lap),
                total - abs(points[i][3] - target_lap),
            ),
        )
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


def source_track(circuit, density_mode=MODE_ANGULAR_45):
    """The circuit as a source file: metadata and waypoints; track_bake fills the rest."""
    track = {
        "name": circuit.name,
        "description": circuit.description,
        "category": "main",
        "kind": {"type": "circuit"},
        "spline": {
            "waypoints": waypoints(circuit, density_mode=density_mode),
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
            "rocks": [],
            "buildings": [],
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
    if circuit.car_model_id:
        track["car_model_id"] = circuit.car_model_id
    return track


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


def grandstand_json(stand_id, spline, prop):
    half_l = prop.length * 0.5
    samples_span = [spline.nearest(prop.at), spline.nearest(prop.at - half_l), spline.nearest(prop.at + half_l)]
    wall_d = max(s.get(f"{prop.side}_wall_distance") or 4.0 for s in samples_span)
    has_curb = any(s.get(f"{prop.side}_curb", False) for s in samples_span)
    min_front = wall_d + 1.2
    if has_curb:
        min_front = max(min_front, 1.4 + 1.2)
    front_offset = max(prop.offset, min_front) if prop.offset is not None else min_front

    point, tangent, normal, width, _s = spline.at(prop.at)
    theta = math.atan2(tangent[1], tangent[0])
    lateral_dist = width * 0.5 + front_offset + prop.depth * 0.5
    if prop.side == "left":
        center = [point[0] + normal[0] * lateral_dist, point[1] + normal[1] * lateral_dist]
        angle = theta
    elif prop.side == "right":
        center = [point[0] - normal[0] * lateral_dist, point[1] - normal[1] * lateral_dist]
        angle = theta + math.pi
    else:
        raise ValueError(f"side must be 'left' or 'right', got {prop.side!r}")

    sample = spline.nearest(prop.at)
    elevation = prop.elevation if prop.elevation is not None else sample["elevation"]
    data = {
        "id": stand_id,
        "center": r3(center),
        "length": round(prop.length, 2),
        "depth": round(prop.depth, 2),
        "angle": round(angle, 5),
        "tiers": prop.tiers,
        "style": prop.style,
        "elevation": round(elevation, 2),
    }
    if prop.seat_color is not None:
        data["seat_color"] = prop.seat_color
    return data


def building_json(bldg_id, spline, prop):
    half_w = prop.width * 0.5
    samples_span = [spline.nearest(prop.at), spline.nearest(prop.at - half_w), spline.nearest(prop.at + half_w)]
    wall_d = max(s.get(f"{prop.side}_wall_distance") or 4.0 for s in samples_span)
    has_curb = any(s.get(f"{prop.side}_curb", False) for s in samples_span)
    min_front = wall_d + 1.5
    if has_curb:
        min_front = max(min_front, 1.4 + 1.5)
    front_offset = max(prop.offset, min_front) if prop.offset is not None else min_front

    point, tangent, normal, width, _s = spline.at(prop.at)
    theta = math.atan2(tangent[1], tangent[0])
    lateral_dist = width * 0.5 + front_offset + prop.depth * 0.5
    if prop.side == "left":
        center = [point[0] + normal[0] * lateral_dist, point[1] + normal[1] * lateral_dist]
        angle = theta if prop.angle is None else prop.angle
    elif prop.side == "right":
        center = [point[0] - normal[0] * lateral_dist, point[1] - normal[1] * lateral_dist]
        angle = (theta + math.pi) if prop.angle is None else prop.angle
    else:
        raise ValueError(f"side must be 'left' or 'right', got {prop.side!r}")

    sample = spline.nearest(prop.at)
    elevation = prop.elevation if prop.elevation is not None else sample["elevation"]
    data = {
        "id": bldg_id,
        "center": r3(center),
        "size": [round(prop.width, 2), round(prop.depth, 2)],
        "angle": round(angle, 5),
        "style": prop.style,
        "elevation": round(elevation, 2),
    }
    if prop.roof_color is not None:
        data["roof_color"] = prop.roof_color
    return data


def rock_json(rock_id, spline, prop):
    sample = spline.nearest(prop.at)
    wall_d = sample.get(f"{prop.side}_wall_distance")
    if wall_d is None:
        wall_d = 4.0
    has_curb = sample.get(f"{prop.side}_curb", False)
    r = ROCK_RADII.get(prop.rock_type, 1.2) * prop.scale
    min_offset = wall_d + r + 0.8
    if has_curb:
        min_offset = max(min_offset, 1.4 + r + 0.8)
    actual_offset = max(prop.offset, min_offset) if prop.offset is not None else min_offset

    point, _t, normal, width, _s = spline.at(prop.at)
    lateral_dist = width * 0.5 + actual_offset
    if prop.side == "left":
        pos = [point[0] + normal[0] * lateral_dist, point[1] + normal[1] * lateral_dist]
    elif prop.side == "right":
        pos = [point[0] - normal[0] * lateral_dist, point[1] - normal[1] * lateral_dist]
    else:
        raise ValueError(f"side must be 'left' or 'right', got {prop.side!r}")

    elevation = prop.elevation if prop.elevation is not None else sample["elevation"]
    return {
        "id": rock_id,
        "position": r3(pos),
        "rock_type": prop.rock_type,
        "scale": round(prop.scale, 2),
        "rotation": round(prop.rotation, 5),
        "elevation": round(elevation, 2),
    }


def tree_json(tree_id, spline, prop):
    sample = spline.nearest(prop.at)
    wall_d = sample.get(f"{prop.side}_wall_distance")
    if wall_d is None:
        wall_d = 4.0
    has_curb = sample.get(f"{prop.side}_curb", False)
    r = TREE_TRUNK_RADII.get(prop.tree_type, 0.35) * prop.scale
    min_offset = wall_d + r + 0.8
    if has_curb:
        min_offset = max(min_offset, 1.4 + r + 0.8)
    actual_offset = max(prop.offset, min_offset) if prop.offset is not None else min_offset

    point, _t, normal, width, _s = spline.at(prop.at)
    lateral_dist = width * 0.5 + actual_offset
    if prop.side == "left":
        pos = [point[0] + normal[0] * lateral_dist, point[1] + normal[1] * lateral_dist]
    elif prop.side == "right":
        pos = [point[0] - normal[0] * lateral_dist, point[1] - normal[1] * lateral_dist]
    else:
        raise ValueError(f"side must be 'left' or 'right', got {prop.side!r}")

    elevation = prop.elevation if prop.elevation is not None else sample["elevation"]
    return {
        "id": tree_id,
        "position": r3(pos),
        "tree_type": prop.tree_type,
        "scale": round(prop.scale, 2),
        "rotation": round(prop.rotation, 5),
        "elevation": round(elevation, 2),
    }


def place_features(track, circuit):
    """Adds the circuit's ramps, zones, scenery and starting grid to a baked track (in place)."""
    spline = BakedSpline(track)
    ramps, zones = [], []
    grandstands, trees, rocks, buildings = [], [], [], []
    for f in flatten_features(circuit.features):
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
        elif isinstance(f, GrandstandProp):
            grandstands.append(grandstand_json(len(grandstands) + 1, spline, f))
        elif isinstance(f, BuildingProp):
            buildings.append(building_json(len(buildings) + 1, spline, f))
        elif isinstance(f, RockProp):
            rocks.append(rock_json(len(rocks) + 1, spline, f))
        elif isinstance(f, TreeProp):
            trees.append(tree_json(len(trees) + 1, spline, f))
        else:
            raise TypeError(f"{circuit.id}: unknown feature {f!r}")
    track["geometry"]["jump_ramps"] = ramps
    track["geometry"]["surface_zones"] = zones
    track["geometry"]["grandstands"] = grandstands
    track["geometry"]["trees"] = trees
    track["geometry"]["rocks"] = rocks
    track["geometry"]["buildings"] = buildings
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
        cmd.extend(["--rebuild", "--adaptive-checkpoints"])
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


def build(circuits, tracks_dir, density_mode=MODE_ANGULAR_45):
    """Builds the circuits into <tracks_dir>/classic/. Returns True when every circuit validated."""
    if not circuits:
        print("no circuits to build")
        return True
    os.makedirs(os.path.join(tracks_dir, MODULE), exist_ok=True)
    paths = [os.path.join(tracks_dir, MODULE, f"{c.id}.json") for c in circuits]
    for circuit, path in zip(circuits, paths):
        write_json(path, source_track(circuit, density_mode=density_mode))
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


def check(circuits, density_mode=MODE_ANGULAR_45):
    """Builds into a temp folder and compares with tracks/. Returns True when every file matches."""
    with tempfile.TemporaryDirectory() as tmp:
        shutil.copy(os.path.join(TRACKS_DIR, ".track_order.json"), tmp)
        if not build(circuits, tmp, density_mode=density_mode):
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

# ---------------------------------------------------------------------------------------------------------------
# Karting (spec 055, User Flow section 3)
# ---------------------------------------------------------------------------------------------------------------
# Standard outdoor karting circuits with asphalt ribbons, wide grass runoffs, tyre walls 3.0-3.5 m out,
# and FIA-style apex curbs. Flat or rolling terrain with 10-slot kart starting grid (5.5 m spacing, 1.8 m stagger).


def outdoor_kart_road(width=8.0, wall_distance=3.5):
    return Road(
        width=width,
        wall_type="TireWall",
        left_wall_distance=wall_distance,
        right_wall_distance=wall_distance,
        left_runoff="Grass",
        right_runoff="Grass",
    )


KART_GRID = (10, 5.5, 1.8)
KART_STEP = 3.3
KERB_L = {"left_curb": True}
KERB_R = {"right_curb": True}

PINE_GROVE = Circuit(
    id="kart_pine_grove",
    name="Pine Grove",
    description="High-speed outdoor kart sprint through pine woods with flowing sweepers, a sweeping hairpin, and a quick chicane.",
    tag="OUTDOOR SPRINT",
    category_label="Karting",
    car_category="kart",
    default_laps=8,
    default_surface="Grass",
    road=outdoor_kart_road(8.0, 3.5),
    grid=KART_GRID,
    step=KART_STEP,
    heading_deg=0.0,
    segments=[
        S(60.0),                   # 0: start/finish straight heading East (solved)
        A(16.0, -90, **KERB_R),    # 1: Turn 1 Curva del Bosco
        S(40.0),                   # 2: east straight heading South (solved)
        A(14.0, -75, **KERB_R),    # 3: Turn 2 fast sweeper
        S(35.0),                   # 4: short blast
        A(18.0, 45, **KERB_L),     # 5: Turn 3 chicane entry left
        A(18.0, -45, **KERB_R),    # 6: Turn 4 chicane exit right
        S(48.0),                   # 7: back straight heading South-West
        A(11.0, -180, **KERB_R),   # 8: Turn 5 Pine Hairpin (180 deg right)
        S(40.0),                   # 9: exit run heading North-East
        A(16.0, 75, **KERB_L),     # 10: Turn 6 uphill sweeper left
        S(45.0),                   # 11: west straight heading North
        A(14.0, -90, **KERB_R),    # 12: Turn 7 final right turn onto main straight
    ],
    close_with=(0, 2),
    features=[
        # Grandstands: main straight and back straight
        GrandstandProp(at=25.0, side="left", offset=6.0, length=24.0, depth=6.0, style="open_bleachers", tiers=5),
        GrandstandProp(at=200.0, side="right", offset=6.0, length=20.0, depth=6.0, style="hillside_bleachers", tiers=4),
        # Buildings: control tower and pit garages on main straight, paddock marquee on west straight
        BuildingProp(at=15.0, side="right", offset=7.0, width=8.0, depth=7.0, style="control_tower"),
        BuildingProp(at=32.0, side="right", offset=7.5, width=16.0, depth=7.0, style="pit_garage"),
        BuildingProp(at=295.0, side="left", offset=7.0, width=14.0, depth=7.0, style="paddock_tent"),
        # Trees: pine woods around the sprint (38 trees)
        tree_row(10.0, 80.0, 8, side="left", offset=7.0, tree_type="pine"),
        tree_row(90.0, 160.0, 10, side="right", offset=7.0, tree_type="pine"),
        tree_row(180.0, 240.0, 10, side="left", offset=7.0, tree_type="pine"),
        tree_row(280.0, 360.0, 10, side="right", offset=7.0, tree_type="bush"),
    ],
)

RIVERBEND_CIRCUIT = Circuit(
    id="kart_riverbend_circuit",
    name="Riverbend Circuit",
    description="Technical outdoor club circuit featuring double-apex sweepers, flowing esses, and wide hairpin complexes.",
    tag="TECHNICAL CLUB",
    category_label="Karting",
    car_category="kart",
    default_laps=7,
    default_surface="Grass",
    road=outdoor_kart_road(8.0, 3.0),
    grid=KART_GRID,
    step=KART_STEP,
    heading_deg=0.0,
    segments=[
        S(65.0),                   # 0: heading 0 (solved)
        A(18.0, 60, **KERB_L),     # 1: Turn 1 Riverbend entry
        S(20.0),                   # 2:
        A(16.0, 60, **KERB_L),     # 3: Turn 2 Riverbend apex
        S(35.0),                   # 4: (solved)
        A(18.0, -45, **KERB_R),    # 5: Turn 3 right flick
        S(6.0),                    # short transition
        A(16.0, 75, **KERB_L),     # 6: Turn 4 left hook
        S(38.0),                   # 7:
        A(13.0, 150, **KERB_L),    # 8: Turn 5 The Loop (hairpin left)
        S(25.0),                   # 9:
        A(18.0, -50, **KERB_R),    # 10: Turn 6 esses right
        S(6.0),                    # transition
        A(18.0, 60, **KERB_L),     # 11: Turn 7 esses left
        S(6.0),                    # transition
        A(18.0, -60, **KERB_R),    # 12: Turn 8 esses right
        S(40.0),                   # 13:
        A(18.0, 45, **KERB_L),     # 14: Turn 9 chicane left
        S(6.0),                    # transition
        A(18.0, -45, **KERB_R),    # 15: Turn 10 chicane right
        S(25.0),                   # 16:
        A(16.0, 110, **KERB_L),    # 17: Turn 11 final carousel left
    ],
    close_with=(0, 4),
    features=[
        # Grandstands: main straight and loop
        GrandstandProp(at=25.0, side="left", offset=5.0, length=25.0, depth=6.0, style="open_bleachers", tiers=5),
        GrandstandProp(at=190.0, side="right", offset=5.0, length=22.0, depth=6.0, style="open_bleachers", tiers=5),
        # Buildings: timing tower, pit garage, team tent
        BuildingProp(at=10.0, side="right", offset=5.5, width=8.0, depth=8.0, style="control_tower"),
        BuildingProp(at=32.0, side="right", offset=5.5, width=20.0, depth=8.0, style="pit_garage"),
        BuildingProp(at=56.0, side="right", offset=6.0, width=15.0, depth=8.0, style="paddock_tent"),
        # Riverbend water zone
        Zone(start=90.0, end=150.0, surface="Water", lateral=(12.0, 32.0), name="Riverbend Water"),
        # Trees: oaks, cypress, bushes (38 trees)
        tree_row(10.0, 80.0, 8, side="left", offset=6.5, tree_type="oak"),
        tree_row(90.0, 170.0, 10, side="left", offset=6.5, tree_type="cypress"),
        tree_row(200.0, 280.0, 10, side="right", offset=6.5, tree_type="oak"),
        tree_row(300.0, 380.0, 10, side="left", offset=6.5, tree_type="bush"),
    ],
)

SUMMIT_INTERNATIONAL = Circuit(
    id="kart_summit_international",
    name="Summit International",
    description="Championship-grade international outdoor kart circuit featuring long drafting straights, multi-apex carousels, and high-commitment esses.",
    tag="GRAND PRIX",
    category_label="Karting",
    car_category="kart",
    default_laps=6,
    default_surface="Grass",
    road=outdoor_kart_road(8.0, 3.0),
    grid=KART_GRID,
    step=KART_STEP,
    heading_deg=0.0,
    segments=[
        S(60.0),                   # 0: start/finish straight heading East (solved)
        A(20.0, 45, **KERB_L),     # 1: Turn 1 Omega entry
        A(18.0, 45, **KERB_L),     # 2: Turn 2 Omega apex
        S(20.0),                   # 3: short straight heading North (solved)
        A(18.0, 45, **KERB_L),     # 4: Turn 3 left
        S(40.0),                   # 5: straight heading North-West
        A(14.0, -150, **KERB_R),   # 6: Turn 4 hairpin right
        S(20.0),                   # 7:
        A(18.0, 60, **KERB_L),     # 8: Turn 5 esses left
        S(6.0),                    # transition
        A(18.0, -60, **KERB_R),    # 9: Turn 6 esses right
        S(6.0),                    # transition
        A(18.0, 75, **KERB_L),     # 10: Turn 7 sweeper left
        S(45.0),                   # 11: back straight
        A(14.0, 135, **KERB_L),    # 12: Turn 8 summit hairpin left
        S(150.0),                  # 13: straight heading South-West
        A(18.0, -45, **KERB_R),    # 14: Turn 9 chicane right
        S(6.0),                    # transition
        A(18.0, 45, **KERB_L),     # 15: Turn 10 chicane left
        S(40.0),                   # 16:
        A(16.0, 105, **KERB_L),    # 17: Turn 11 carousel left
        S(120.0),                  # 18: return straight heading South-East
        A(16.0, 60, **KERB_L),     # 19: Turn 12 final turn left onto main straight
    ],
    close_with=(0, 3),
    features=[
        # Grandstands: covered main stadium, back straight stadium, return straight stand
        GrandstandProp(at=30.0, side="left", offset=6.0, length=30.0, depth=7.0, style="covered_stadium", tiers=6),
        GrandstandProp(at=430.0, side="right", offset=6.0, length=24.0, depth=6.0, style="open_bleachers", tiers=5),
        GrandstandProp(at=670.0, side="left", offset=6.0, length=24.0, depth=6.0, style="open_bleachers", tiers=5),
        # Buildings: control tower, pit garages, scrutineering marquee, back straight marshal post
        BuildingProp(at=12.0, side="right", offset=6.5, width=8.0, depth=8.0, style="control_tower"),
        BuildingProp(at=35.0, side="right", offset=6.5, width=22.0, depth=7.5, style="pit_garage"),
        BuildingProp(at=680.0, side="right", offset=7.0, width=16.0, depth=8.0, style="paddock_tent"),
        BuildingProp(at=460.0, side="left", offset=7.0, width=14.0, depth=7.0, style="paddock_tent"),
        # Trees: pines, cypress, bushes (46 trees)
        tree_row(10.0, 110.0, 10, side="left", offset=7.0, tree_type="pine"),
        tree_row(130.0, 240.0, 12, side="right", offset=7.0, tree_type="cypress"),
        tree_row(260.0, 390.0, 12, side="left", offset=7.0, tree_type="pine"),
        tree_row(410.0, 560.0, 12, side="right", offset=7.0, tree_type="bush"),
    ],
)

# ---------------------------------------------------------------------------------------------------------------
# Rallycross (spec 055, User Flow section 3)
# ---------------------------------------------------------------------------------------------------------------
# 30–60 % Asphalt, rest Gravel or Dirt; width 11–14 m; walls TireWall and Steel.
# At least 3, 4 and 6 jumps per lap (JumpRamp tabletops or elevation crests).

QUARRY_SPRINT = Circuit(
    id="rx_quarry_sprint",
    name="Quarry Sprint",
    description="Fast mixed-surface quarry circuit with sweeping dirt bends, asphalt straights and three tabletop jumps.",
    tag="QUARRY RX",
    category_label="Rallycross",
    car_category="rally",
    default_laps=6,
    default_surface="Grass",
    road=Road(width=12.0, left_wall_distance=3.5, right_wall_distance=3.5),
    segments=[
        S(120, surface="Asphalt", wall_type="Steel"),               # 0: Main straight (East, 0 deg)
        A(25, -90, surface="Asphalt", wall_type="TireWall"),        # 1: Turn 1 (to South, -90 deg)
        S(70, surface="PackedGravel", wall_type="TireWall"),              # 2: South (-90 deg)
        A(30, -60, surface="PackedGravel", wall_type="TireWall"),         # 3: Sweeper (to -150 deg)
        S(60, surface="Dirt", wall_type="Steel"),                   # 4: -150 deg
        A(25, 45, surface="Dirt", wall_type="TireWall"),            # 5: Kink (to -105 deg)
        S(50, surface="Dirt", wall_type="Steel"),                   # 6: -105 deg
        A(20, -75, surface="Dirt", wall_type="TireWall"),           # 7: Hairpin turn (to -180 deg, West)
        S(80, surface="PackedGravel", wall_type="Steel"),                 # 8: West (-180 deg)
        A(25, -90, surface="PackedGravel", wall_type="TireWall"),         # 9: Turn 4 (to North, +90 deg / -270 deg)
        S(60, surface="Asphalt", wall_type="Steel"),                # 10: North (+90 deg)
        A(25, -45, surface="Asphalt", wall_type="TireWall"),        # 11: (to +45 deg)
        S(40, surface="Asphalt", wall_type="Steel"),                # 12: +45 deg
        A(25, -45, surface="Asphalt", wall_type="TireWall"),        # 13: (to 0 deg, East)
    ],
    close_with=(0, 10),
    finish_at=(0, 0.85),
    features=[
        Ramp(at=90.0, length=14.0, height=2.0, launch_speed=4.0, surface="PackedGravel", name="Quarry Dirt Jump 1"),
        Ramp(at=350.0, length=14.0, height=2.0, launch_speed=4.0, surface="PackedGravel", name="Quarry Dirt Jump 2"),
        Ramp(at=500.0, length=14.0, height=2.2, launch_speed=4.2, surface="Asphalt", name="Quarry Asphalt Jump"),
        # Grandstands: hillside bleachers
        GrandstandProp(at=30.0, side="left", offset=6.0, length=28.0, depth=6.0, style="hillside_bleachers", tiers=5),
        GrandstandProp(at=320.0, side="right", offset=6.0, length=24.0, depth=6.0, style="hillside_bleachers", tiers=5),
        # Buildings: control tower and paddock tents
        BuildingProp(at=15.0, side="right", offset=7.0, width=8.0, depth=8.0, style="control_tower"),
        BuildingProp(at=40.0, side="right", offset=7.5, width=18.0, depth=8.0, style="paddock_tent"),
        BuildingProp(at=65.0, side="right", offset=7.5, width=16.0, depth=8.0, style="paddock_tent"),
        # Rocks: quarry granite and slate boulders (12 rocks)
        rock_cluster(at=160.0, count=6, side="left", offset=7.5, rock_type="granite", span=16.0),
        rock_cluster(at=260.0, count=6, side="right", offset=7.5, rock_type="slate", span=16.0),
        # Trees: quarry perimeter pines, oaks, bushes (36 trees)
        tree_row(10.0, 100.0, 8, side="left", offset=7.0, tree_type="pine"),
        tree_row(120.0, 220.0, 10, side="right", offset=7.0, tree_type="oak"),
        tree_row(240.0, 360.0, 10, side="left", offset=7.0, tree_type="pine"),
        tree_row(380.0, 480.0, 8, side="right", offset=7.0, tree_type="bush"),
    ],
)

HILLTOP_LEAP = Circuit(
    id="rx_hilltop_leap",
    name="Hilltop Leap",
    description="Rolling rallycross track with a start straight crest, hilltop jumps and a gravel hairpin.",
    tag="HILLTOP RX",
    category_label="Rallycross",
    car_category="rally",
    default_laps=5,
    default_surface="Grass",
    road=Road(width=12.0, left_wall_distance=3.5, right_wall_distance=3.5),
    segments=[
        S(160, surface="Asphalt", wall_type="Steel"),               # 0: Start straight (East, 0 deg)
        A(28, -90, surface="Asphalt", wall_type="TireWall"),        # 1: Turn 1 (to -90 deg / South)
        S(75, surface="PackedGravel", wall_type="Steel"),                 # 2: Downhill straight (-90 deg)
        A(25, -45, surface="PackedGravel", wall_type="TireWall"),         # 3: (to -135 deg)
        S(80, surface="Dirt", wall_type="Steel"),                   # 4: Dirt straight (-135 deg)
        A(25, 45, surface="Dirt", wall_type="TireWall"),            # 5: (to -90 deg)
        S(50, surface="Dirt", wall_type="TireWall"),                # 6: (-90 deg)
        A(20, -180, surface="PackedGravel", wall_type="TireWall", wall_distance=3.0), # 7: Gravel Hairpin! (to +90 deg)
        S(95, surface="PackedGravel", wall_type="Steel"),                 # 8: (+90 deg)
        A(25, 45, surface="PackedGravel", wall_type="TireWall"),          # 9: (to +135 deg / NW)
        S(135, surface="Dirt", wall_type="Steel"),                  # 10: Dirt straight (+135 deg)
        A(25, -45, surface="Asphalt", wall_type="TireWall"),        # 11: (to +90 deg)
        S(70, surface="Asphalt", wall_type="Steel"),                # 12: North (+90 deg)
        A(28, -90, surface="Asphalt", wall_type="TireWall"),        # 13: (to 0 deg / East)
        S(20, surface="Asphalt", wall_type="Steel"),                # 14: (East)
        A(22, -90, surface="Asphalt", wall_type="TireWall"),        # 15: (to -90 deg)
        S(25, surface="Asphalt", wall_type="Steel"),                # 16: (to South)
        A(22, 90, surface="Asphalt", wall_type="TireWall"),         # 17: (to East 0 deg)
    ],
    close_with=(0, 12),
    finish_at=(0, 0.85),
    profile=[
        ((0, 0.0), 0.0),
        ((0, 0.4), 2.5),    # Crest on start straight!
        ((0, 0.85), 0.0),   # Finish line
        ((2, 0.5), 0.0),
        ((8, 0.5), 0.0),
        ((10, 0.5), 3.0),   # Hilltop rise
        ((12, 0.5), 1.0),
        ((15, 0.0), 0.0),
    ],
    features=[
        Ramp(at=60.0, length=14.0, height=2.0, launch_speed=4.0, surface="PackedGravel", name="Hilltop Downhill Leap"),
        Ramp(at=190.0, length=14.0, height=2.0, launch_speed=4.0, surface="Dirt", name="Hilltop Infield Jump"),
        Ramp(at=350.0, length=14.0, height=2.0, launch_speed=4.0, surface="PackedGravel", name="Hilltop Hairpin Exit Jump"),
        Ramp(at=510.0, length=14.0, height=2.2, launch_speed=4.2, surface="Dirt", name="Hilltop Crest Jump"),
        # Grandstands: hillside bleachers
        GrandstandProp(at=40.0, side="left", offset=6.0, length=30.0, depth=6.0, style="hillside_bleachers", tiers=5),
        GrandstandProp(at=380.0, side="right", offset=6.0, length=24.0, depth=6.0, style="hillside_bleachers", tiers=5),
        # Buildings: control tower and paddock tents
        BuildingProp(at=15.0, side="right", offset=7.0, width=8.0, depth=8.0, style="control_tower"),
        BuildingProp(at=70.0, side="right", offset=7.5, width=18.0, depth=8.0, style="paddock_tent"),
        BuildingProp(at=95.0, side="right", offset=7.5, width=16.0, depth=8.0, style="paddock_tent"),
        # Rocks: hilltop granite boulders (12 rocks)
        rock_cluster(at=140.0, count=6, side="left", offset=7.5, rock_type="granite", span=16.0),
        rock_cluster(at=450.0, count=6, side="right", offset=7.5, rock_type="granite", span=16.0),
        # Trees: hillside pines, oaks, bushes (40 trees)
        tree_row(10.0, 130.0, 10, side="left", offset=7.0, tree_type="pine"),
        tree_row(160.0, 280.0, 10, side="right", offset=7.0, tree_type="oak"),
        tree_row(300.0, 430.0, 10, side="left", offset=7.0, tree_type="pine"),
        tree_row(460.0, 580.0, 10, side="right", offset=7.0, tree_type="bush"),
    ],
)

CANYON_FLYER = Circuit(
    id="rx_canyon_flyer",
    name="Canyon Flyer",
    description="Challenging canyon rallycross with a water gap jump, a whoops rhythm section and technical dirt esses.",
    tag="CANYON RX",
    category_label="Rallycross",
    car_category="rally",
    default_laps=4,
    default_surface="Grass",
    road=Road(width=12.0, left_wall_distance=3.5, right_wall_distance=3.5),
    segments=[
        S(160, surface="Asphalt", wall_type="Steel"),               # 0: Start straight (East, 0 deg)
        A(30, -90, surface="Asphalt", wall_type="TireWall"),        # 1: Turn 1 (to -90 deg / South)
        S(70, surface="Dirt", wall_type="Steel"),                   # 2: South (-90 deg)
        A(25, -45, surface="Dirt", wall_type="TireWall"),           # 3: (to -135 deg / SW)
        S(90, surface="Dirt", wall_type="Steel"),                   # 4: SW (-135 deg)
        A(25, -45, surface="Dirt", wall_type="TireWall"),           # 5: (to -180 deg / West)
        S(110, surface="Dirt", wall_type="Steel"),                  # 6: Gap Jump straight (-180 deg / West)
        A(25, 45, surface="PackedGravel", wall_type="TireWall"),          # 7: (to -135 deg)
        S(45, surface="PackedGravel", wall_type="TireWall"),              # 8: (-135 deg)
        A(25, -45, surface="PackedGravel", wall_type="TireWall"),         # 9: (to -180 deg)
        S(95, surface="PackedGravel", wall_type="Steel"),                 # 10: Whoops straight (-180 deg)
        A(20, -90, surface="PackedGravel", wall_type="TireWall", wall_distance=3.0), # 11: Turn North (-270 deg / +90 deg)
        S(80, surface="PackedGravel", wall_type="Steel"),                 # 12: (+90 deg / North)
        A(25, -45, surface="Dirt", wall_type="TireWall"),           # 13: (to +45 deg)
        S(60, surface="Dirt", wall_type="Steel"),                   # 14: (+45 deg)
        A(25, 45, surface="Asphalt", wall_type="TireWall"),         # 15: (to +90 deg / North)
        S(80, surface="Asphalt", wall_type="Steel"),                # 16: North (+90 deg)
        A(28, -90, surface="Asphalt", wall_type="TireWall"),        # 17: (to 0 deg / East)
        S(30, surface="Asphalt", wall_type="Steel"),                # 18: (East)
        A(22, -90, surface="Asphalt", wall_type="TireWall"),        # 19: Chicane (to -90 deg)
        S(20, surface="Asphalt", wall_type="Steel"),                # 20: (South)
        A(22, 90, surface="Asphalt", wall_type="TireWall"),         # 21: (to East 0 deg)
    ],
    close_with=(0, 16),
    finish_at=(0, 0.85),
    features=[
        Ramp(at=190.0, length=14.0, height=2.0, launch_speed=4.0, surface="Dirt", name="Canyon Infield Jump"),
        Ramp(at=310.0, length=16.0, height=2.4, launch_speed=4.5, surface="Dirt", name="Canyon Gap Jump"),
        Zone(start=326.0, end=344.0, surface="Water", lateral=(-6.0, 6.0), layer="above_track", name="Canyon Water Gap"),
        Spot(at=335.0, lateral=18.0, radius=12.0, surface="Water", name="Canyon Gap Pond"),
        Whoops(at=490.0, count=6, spacing=5.0, height=0.6, name="Canyon Whoops"),
        Ramp(at=800.0, length=14.0, height=2.0, launch_speed=4.0, surface="Asphalt", name="Canyon Asphalt Jump"),
        # Grandstands: hillside bleachers
        GrandstandProp(at=40.0, side="left", offset=6.0, length=32.0, depth=6.0, style="hillside_bleachers", tiers=5),
        GrandstandProp(at=400.0, side="right", offset=6.0, length=24.0, depth=6.0, style="hillside_bleachers", tiers=5),
        # Buildings: control tower and paddock tents
        BuildingProp(at=15.0, side="right", offset=7.0, width=8.0, depth=8.0, style="control_tower"),
        BuildingProp(at=70.0, side="right", offset=7.5, width=18.0, depth=8.0, style="paddock_tent"),
        BuildingProp(at=95.0, side="right", offset=7.5, width=16.0, depth=8.0, style="paddock_tent"),
        # Rocks: canyon granite and slate cliffs (14 rocks)
        rock_cluster(at=230.0, count=7, side="left", offset=7.5, rock_type="granite", span=18.0),
        rock_cluster(at=350.0, count=7, side="right", offset=8.0, rock_type="slate", span=18.0),
        # Trees: pines, oaks, bushes (40 trees)
        tree_row(10.0, 140.0, 10, side="left", offset=7.0, tree_type="pine"),
        tree_row(160.0, 290.0, 10, side="right", offset=7.0, tree_type="oak"),
        tree_row(420.0, 560.0, 10, side="left", offset=7.0, tree_type="pine"),
        tree_row(600.0, 750.0, 10, side="right", offset=7.0, tree_type="bush"),
    ],
)

# ---------------------------------------------------------------------------------------------------------------
# Autocross (spec 055, User Flow section 3)
# ---------------------------------------------------------------------------------------------------------------
# 100 % unpaved (Dirt, Gravel, PackedSand; optional Concrete launch pad <= 60 m); width 12–16 m.
# No JumpRamps (uses crests and banked berms 6–12°). Each circuit names its own car in car_model_id.

MEADOW_SPRINT = Circuit(
    id="ax_meadow_sprint",
    name="Meadow Sprint",
    description="Flowing countryside autocross track with sweeping clay turns, elevation crests and a banked berm.",
    tag="MEADOW AX",
    category_label="Autocross",
    car_category="autocross",
    car_model_id="classic_ax_mudlark",
    default_laps=6,
    default_surface="Grass",
    road=Road(width=13.0, left_wall_distance=3.5, right_wall_distance=3.5, surface="Dirt"),
    segments=[
        S(130, surface="Dirt", wall_type="TireWall"),               # 0: Start straight (East, 0 deg)
        A(35, 90, surface="Dirt", bank=8.0, wall_type="TireWall"),  # 1: Banked berm turn! (to +90 deg / North)
        S(60, surface="Dirt", wall_type="Steel"),                   # 2: (+90 deg)
        A(30, 45, surface="PackedSand", wall_type="TireWall"),      # 3: (to +135 deg / NW)
        S(180, surface="PackedSand", wall_type="Steel"),            # 4: (+135 deg)
        A(25, -45, surface="PackedSand", wall_type="TireWall"),     # 5: (to +90 deg)
        S(40, surface="PackedGravel", wall_type="Steel"),                 # 6: (+90 deg)
        A(20, 180, surface="PackedGravel", wall_type="TireWall", wall_distance=3.0), # 7: Hairpin! (to -90 deg / South)
        S(70, surface="PackedGravel", wall_type="Steel"),                 # 8: (-90 deg)
        A(30, 45, surface="Dirt", wall_type="TireWall"),            # 9: (to -45 deg / SE)
        S(40, surface="Dirt", wall_type="Steel"),                   # 10: (-45 deg)
        A(30, 45, surface="Dirt", wall_type="TireWall"),            # 11: (to 0 deg / East)
    ],
    close_with=(0, 8),
    finish_at=(0, 0.85),
    profile=[
        ((0, 0.0), 0.0),
        ((0, 0.85), 0.0),   # Finish line
        ((2, 0.5), 3.0),    # Crest 1
        ((4, 0.5), 0.5),
        ((6, 0.5), 3.2),    # Crest 2
        ((8, 0.5), 0.0),
        ((10, 0.5), 0.5),
    ],
    features=[
        # Grandstands: hillside bleachers
        GrandstandProp(at=35.0, side="left", offset=6.0, length=28.0, depth=6.0, style="hillside_bleachers", tiers=5),
        GrandstandProp(at=350.0, side="right", offset=6.0, length=24.0, depth=6.0, style="hillside_bleachers", tiers=5),
        # Buildings: control tower and paddock tents
        BuildingProp(at=15.0, side="right", offset=7.0, width=8.0, depth=8.0, style="control_tower"),
        BuildingProp(at=55.0, side="right", offset=7.5, width=18.0, depth=8.0, style="paddock_tent"),
        BuildingProp(at=80.0, side="right", offset=7.5, width=16.0, depth=8.0, style="paddock_tent"),
        # Rocks: sandstone and granite boulders (12 rocks)
        rock_cluster(at=160.0, count=6, side="left", offset=7.5, rock_type="sandstone", span=16.0),
        rock_cluster(at=450.0, count=6, side="right", offset=7.5, rock_type="granite", span=16.0),
        # Trees: oaks, pines, bushes (40 trees)
        tree_row(10.0, 110.0, 10, side="left", offset=7.0, tree_type="oak"),
        tree_row(140.0, 260.0, 10, side="right", offset=7.0, tree_type="pine"),
        tree_row(280.0, 420.0, 10, side="left", offset=7.0, tree_type="oak"),
        tree_row(440.0, 580.0, 10, side="right", offset=7.0, tree_type="bush"),
    ],
)

CLAY_BOWL = Circuit(
    id="ax_clay_bowl",
    name="Clay Bowl",
    description="Technical autocross amphitheatre with 3 banked berms, a downhill hairpin and a concrete launch pad.",
    tag="CLAY BOWL",
    category_label="Autocross",
    car_category="autocross",
    car_model_id="classic_ax_brawler",
    default_laps=5,
    default_surface="Grass",
    road=Road(width=13.0, left_wall_distance=3.5, right_wall_distance=3.5, surface="Dirt"),
    segments=[
        S(45, surface="Concrete", wall_type="Steel"),               # 0: Launch pad (East, 0 deg)
        S(90, surface="Dirt", wall_type="TireWall"),                # 1: Start straight (East, 0 deg)
        A(35, -90, surface="Dirt", bank=-9.0, wall_type="TireWall"),# 2: Berm 1! (to -90 deg / South)
        S(80, surface="Dirt", wall_type="Steel"),                   # 3: South (-90 deg)
        A(30, -45, surface="PackedSand", wall_type="TireWall"),     # 4: (to -135 deg / SW)
        S(120, surface="PackedSand", wall_type="Steel"),            # 5: SW (-135 deg)
        A(35, -45, surface="PackedSand", bank=-8.5, wall_type="TireWall"), # 6: Berm 2! (to -180 deg / West)
        S(110, surface="Dirt", wall_type="Steel"),                  # 7: West (-180 deg)
        A(30, 45, surface="Dirt", wall_type="TireWall"),            # 8: (to -135 deg)
        S(90, surface="Dirt", wall_type="Steel"),                   # 9: Ridge straight at 6.0 m (Crest!)
        A(22, -180, surface="Dirt", bank=-9.0, wall_type="TireWall", wall_distance=3.0), # 10: Berm 3 & Downhill Hairpin! (to +45 deg / NE)
        S(80, surface="PackedGravel", wall_type="Steel"),                 # 11: Downhill gravel straight (+45 deg)
        A(30, -45, surface="PackedGravel", wall_type="TireWall"),         # 12: (to 0 deg / East)
        S(40, surface="PackedGravel", wall_type="Steel"),                 # 13: East (0 deg)
        A(30, 90, surface="Dirt", wall_type="TireWall"),            # 14: (to +90 deg / North)
        S(105, surface="Dirt", wall_type="Steel"),                  # 15: North (+90 deg)
        A(30, -90, surface="Dirt", wall_type="TireWall"),           # 16: (to 0 deg / East)
    ],
    close_with=(1, 3),
    finish_at=(1, 0.8),
    profile=[
        ((0, 0.0), 0.0),
        ((1, 0.8), 0.0),    # Finish line on start straight
        ((3, 0.5), 2.0),    # Climbing out of bowl
        ((5, 0.5), 4.0),
        ((7, 0.5), 5.2),
        ((9, 0.5), 6.0),    # Ridge crest!
        ((10, 0.5), 3.0),   # Downhill hairpin
        ((11, 0.5), 0.5),   # Back into bowl floor
        ((13, 0.5), 0.0),
        ((15, 0.5), 0.0),
    ],
    features=[
        # Grandstands: hillside amphitheatre seating
        GrandstandProp(at=50.0, side="left", offset=6.0, length=30.0, depth=6.0, style="hillside_bleachers", tiers=5),
        GrandstandProp(at=480.0, side="right", offset=6.0, length=24.0, depth=6.0, style="hillside_bleachers", tiers=5),
        # Buildings: control tower and paddock tents
        BuildingProp(at=15.0, side="right", offset=7.0, width=8.0, depth=8.0, style="control_tower"),
        BuildingProp(at=70.0, side="right", offset=7.5, width=18.0, depth=8.0, style="paddock_tent"),
        BuildingProp(at=95.0, side="right", offset=7.5, width=16.0, depth=8.0, style="paddock_tent"),
        # Rocks: amphitheatre sandstone and slate (12 rocks)
        rock_cluster(at=200.0, count=6, side="left", offset=7.5, rock_type="sandstone", span=16.0),
        rock_cluster(at=380.0, count=6, side="right", offset=7.5, rock_type="slate", span=16.0),
        # Trees: oaks, pines, bushes (42 trees)
        tree_row(10.0, 120.0, 10, side="left", offset=7.0, tree_type="oak"),
        tree_row(150.0, 300.0, 10, side="right", offset=7.0, tree_type="pine"),
        tree_row(320.0, 460.0, 10, side="left", offset=7.0, tree_type="oak"),
        tree_row(500.0, 680.0, 12, side="right", offset=7.0, tree_type="bush"),
    ],
)

HILLSIDE_HAMMER = Circuit(
    id="ax_hillside_hammer",
    name="Hillside Hammer",
    description="Brutal hillside autocross climb with off-camber turns, a summit hairpin and high-speed crests.",
    tag="HILLSIDE AX",
    category_label="Autocross",
    car_category="autocross",
    car_model_id="classic_ax_talon",
    default_laps=4,
    default_surface="Grass",
    road=Road(width=13.0, left_wall_distance=3.5, right_wall_distance=3.5, surface="Dirt"),
    segments=[
        S(160, surface="Dirt", wall_type="TireWall"),               # 0: Start straight (East, 0 deg)
        A(35, -90, surface="PackedGravel", wall_type="TireWall"),         # 1: Turn 1 (to -90 deg / South)
        S(90, surface="PackedGravel", wall_type="Steel"),                 # 2: South (-90 deg), climbing
        A(30, -45, surface="PackedGravel", bank=6.0, wall_type="TireWall"), # 3: Off-camber 1! (Right turn with bank > 0, to -135 deg)
        S(160, surface="PackedSand", wall_type="Steel"),            # 4: Climbing straight on PackedSand (-135 deg)
        A(30, 45, surface="PackedSand", wall_type="TireWall"),      # 5: (to -90 deg)
        S(140, surface="Dirt", wall_type="Steel"),                  # 6: Summit climb straight (to 8.5 m)
        A(20, -180, surface="Dirt", wall_type="TireWall", wall_distance=3.0), # 7: Summit Hairpin at 8.5 m! (to +90 deg / North)
        S(140, surface="Dirt", wall_type="Steel"),                  # 8: Downhill ridge straight (+90 deg)
        A(30, 60, surface="PackedGravel", bank=-6.0, wall_type="TireWall"), # 9: Off-camber 2! (Left turn with bank < 0, to +150 deg)
        S(140, surface="PackedGravel", wall_type="Steel"),                # 10: Downhill gravel straight (+150 deg)
        A(30, -60, surface="PackedGravel", wall_type="TireWall"),         # 11: (to +90 deg / North)
        S(120, surface="Dirt", wall_type="Steel"),                  # 12: Dirt straight (+90 deg)
        A(35, -90, surface="Dirt", wall_type="TireWall"),           # 13: (to 0 deg / East)
        S(60, surface="Dirt", wall_type="Steel"),                   # 14: East (0 deg)
        A(25, -45, surface="Dirt", wall_type="TireWall"),           # 15: Chicane entry (to -45 deg)
        S(40, surface="Dirt", wall_type="Steel"),                   # 16: (-45 deg)
        A(25, 45, surface="Dirt", wall_type="TireWall"),            # 17: (to 0 deg / East)
    ],
    close_with=(0, 2),
    finish_at=(0, 0.85),
    profile=[
        ((0, 0.0), 0.0),
        ((0, 0.85), 0.0),   # Finish line
        ((2, 0.5), 2.5),
        ((4, 0.5), 5.0),    # Ridge crest 1
        ((5, 0.5), 4.0),
        ((6, 0.9), 8.5),    # Summit climb
        ((7, 0.5), 8.5),    # Summit hairpin at 8.5 m!
        ((8, 0.5), 5.5),
        ((10, 0.5), 2.5),
        ((12, 0.5), 0.5),
        ((14, 0.5), 0.0),
    ],
    features=[
        # Grandstands: hillside bleachers
        GrandstandProp(at=50.0, side="left", offset=6.0, length=32.0, depth=6.0, style="hillside_bleachers", tiers=5),
        GrandstandProp(at=600.0, side="right", offset=6.0, length=24.0, depth=6.0, style="hillside_bleachers", tiers=5),
        # Buildings: control tower and paddock tents
        BuildingProp(at=15.0, side="right", offset=7.0, width=8.0, depth=8.0, style="control_tower"),
        BuildingProp(at=75.0, side="right", offset=7.5, width=18.0, depth=8.0, style="paddock_tent"),
        BuildingProp(at=100.0, side="right", offset=7.5, width=16.0, depth=8.0, style="paddock_tent"),
        # Rocks: summit granite boulders (14 rocks)
        rock_cluster(at=240.0, count=7, side="left", offset=7.5, rock_type="granite", span=18.0),
        rock_cluster(at=480.0, count=7, side="right", offset=7.5, rock_type="granite", span=18.0),
        # Trees: pines, oaks, bushes (46 trees)
        tree_row(10.0, 140.0, 10, side="left", offset=7.0, tree_type="pine"),
        tree_row(160.0, 320.0, 12, side="right", offset=7.0, tree_type="oak"),
        tree_row(340.0, 520.0, 12, side="left", offset=7.0, tree_type="pine"),
        tree_row(540.0, 780.0, 12, side="right", offset=7.0, tree_type="bush"),
    ],
)

# GT: high speed, braking and runoff (spec 055, User Flow section 3).
# Surface Asphalt, kerbs on apexes and chicanes, walls Steel with TireWall at the end of fast straights.
# Variable runoff 3-30 m per side (spec 089): straights Grass <= 8 m, corner traps DeepGravel <= 12 m,
# chicanes DeepGravel <= 6 m, no Asphalt run-off.

GT_ROAD = Road(
    width=13.0,
    surface="Asphalt",
    wall_type="Steel",
    left_wall_distance=6.0,
    right_wall_distance=6.0,
    left_runoff="Grass",
    right_runoff="Grass",
)

VELOCITY_PARK = Circuit(
    id="gt_velocity_park",
    name="Velocity Park",
    description="High-speed GT circuit featuring two long straights ending in heavy braking chicanes lined with deep gravel traps.",
    tag="POWER CIRCUIT",
    category_label="GT Circuit",
    car_category="gt",
    default_laps=4,
    road=GT_ROAD,
    start=(0.0, 0.0),
    heading_deg=0.0,
    segments=[
        # Main Straight (>= 300 m)
        S(240, wall_type="Steel", wall_distance=8.0, runoff="Grass"),                                       # 0
        S(80, wall_type="TireWall", wall_distance=8.0, runoff="Grass"),                                  # 1: Braking zone
        # Chicane 1 (Right-Left) - narrow DeepGravel traps on both sides
        A(50, -35, right_curb=True, wall_distance=6.0, runoff="DeepGravel", wall_type="TireWall"),          # 2
        S(25, left_curb=True, right_curb=True, wall_distance=6.0, runoff="DeepGravel", wall_type="TireWall"),# 3
        A(50, 35, left_curb=True, wall_distance=6.0, runoff="DeepGravel", wall_type="TireWall"),            # 4
        S(30, wall_distance=6.0, runoff="DeepGravel", wall_type="TireWall"),                             # 5: chicane exit
        # Turn 1: Sweeper to South (-90 deg)
        A(70, -90, right_curb=True, left_wall_distance=12.0, right_wall_distance=8.0, left_runoff="DeepGravel", right_runoff="Grass", wall_type="TireWall"), # 6
        S(60, wall_type="Steel", wall_distance=8.0, runoff="Grass"),                                        # 7
        # Turn 2: To West (-180 deg)
        A(60, -90, right_curb=True, left_wall_distance=12.0, right_wall_distance=8.0, left_runoff="DeepGravel", right_runoff="Grass", wall_type="TireWall"), # 8
        # Back Straight (>= 300 m)
        S(260, wall_type="Steel", wall_distance=8.0, runoff="Grass"),                                       # 9
        S(80, wall_type="TireWall", wall_distance=8.0, runoff="Grass"),                                  # 10: Braking zone
        # Chicane 2 (Left-Right)
        A(50, 35, left_curb=True, wall_distance=6.0, runoff="DeepGravel", wall_type="TireWall"),             # 11
        S(25, left_curb=True, right_curb=True, wall_distance=6.0, runoff="DeepGravel", wall_type="TireWall"),# 12
        A(50, -35, right_curb=True, wall_distance=6.0, runoff="DeepGravel", wall_type="TireWall"),          # 13
        S(30, wall_distance=6.0, runoff="DeepGravel", wall_type="TireWall"),                             # 14: chicane exit
        # Turn 3: To North (+90 deg)
        A(60, -90, right_curb=True, left_wall_distance=12.0, right_wall_distance=8.0, left_runoff="DeepGravel", right_runoff="Grass", wall_type="TireWall"), # 15
        S(60, wall_type="Steel", wall_distance=8.0, runoff="Grass"),                                        # 16
        # Turn 4: Final corner to East (0 deg)
        A(70, -90, right_curb=True, left_wall_distance=12.0, right_wall_distance=8.0, left_runoff="DeepGravel", right_runoff="Grass", wall_type="TireWall"), # 17
    ],
    close_with=(0, 16),
    finish_at=(0, 0.5),
    features=[
        # Grandstands: main straight covered stadium stands
        GrandstandProp(at=40.0, side="left", offset=10.0, length=45.0, depth=8.0, style="covered_stadium", tiers=8),
        GrandstandProp(at=90.0, side="left", offset=10.0, length=40.0, depth=8.0, style="covered_stadium", tiers=8),
        # Buildings: race control tower and pit garages
        BuildingProp(at=20.0, side="right", offset=11.0, width=10.0, depth=10.0, style="control_tower"),
        BuildingProp(at=55.0, side="right", offset=11.0, width=32.0, depth=9.0, style="pit_garage"),
        BuildingProp(at=100.0, side="right", offset=11.0, width=32.0, depth=9.0, style="pit_garage"),
        # Lake: infield lake
        Spot(at=350.0, lateral=40.0, radius=22.0, surface="Water", name="Velocity Park Infield Lake"),
        # Trees: cypress, oaks, bushes (48 trees)
        tree_row(20.0, 200.0, 12, side="left", offset=14.0, tree_type="cypress"),
        tree_row(240.0, 440.0, 12, side="right", offset=14.0, tree_type="oak"),
        tree_row(500.0, 700.0, 12, side="left", offset=14.0, tree_type="cypress"),
        tree_row(740.0, 920.0, 12, side="right", offset=14.0, tree_type="oak"),
    ],
)

RIDGE_RING = Circuit(
    id="gt_ridge_ring",
    name="Ridge Ring",
    description="Undulating GT circuit climbing to an 8-meter ridge crest before plunging down through flowing esses and a technical hairpin.",
    tag="HILL CIRCUIT",
    category_label="GT Circuit",
    car_category="gt",
    default_laps=4,
    road=GT_ROAD,
    start=(0.0, 0.0),
    heading_deg=0.0,
    segments=[
        # 0: Main Straight (>= 300 m) East (0 deg)
        S(305, wall_type="Steel", wall_distance=7.0, runoff="Grass"),                                       # 0
        # Turn 1: Right turn to South (-90 deg)
        A(40, -90, right_curb=True, left_wall_distance=12.0, right_wall_distance=7.0, left_runoff="DeepGravel", wall_type="TireWall"), # 1
        # Ridge climb straight (climbing towards crest)
        S(65, wall_type="Steel", wall_distance=7.0, runoff="Grass"),                                        # 2
        # Esses on the ridge (climbing to 8 m)
        A(45, 45, left_curb=True, wall_distance=8.0, runoff="Grass", wall_type="TireWall"),                 # 3
        S(35, wall_type="Steel", wall_distance=8.0, runoff="Grass"),                                        # 4
        A(45, -45, right_curb=True, left_runoff="DeepGravel", left_wall_distance=10.0, right_wall_distance=8.0, runoff="Grass", wall_type="TireWall"), # 5: Crest
        # Downhill corner immediately after crest
        A(45, -45, right_curb=True, left_wall_distance=12.0, right_wall_distance=8.0, left_runoff="DeepGravel", wall_type="TireWall"),# 6: Downhill
        S(40, wall_type="TireWall", left_wall_distance=8.0, right_wall_distance=8.0, left_runoff="Grass"), # 7
        # Chicane
        A(40, 45, left_curb=True, wall_distance=6.0, runoff="DeepGravel", wall_type="TireWall"),             # 8
        S(20, left_curb=True, right_curb=True, wall_distance=6.0, runoff="DeepGravel", wall_type="TireWall"),# 9
        A(40, -45, right_curb=True, wall_distance=6.0, runoff="DeepGravel", wall_type="TireWall"),          # 10
        S(20, wall_type="Steel", wall_distance=8.0, runoff="Grass"),                                        # 11
        # Turn to West (-180 deg)
        A(45, -45, right_curb=True, left_wall_distance=10.0, right_wall_distance=8.0, left_runoff="DeepGravel", wall_type="TireWall"),# 12
        # Westward run along the southern edge
        S(100, wall_type="Steel", wall_distance=7.0, runoff="Grass"),                                       # 13
        # Hairpin at South-West corner (180 deg right turn to North) - radius 40m
        A(40, -90, right_curb=True, left_wall_distance=12.0, right_wall_distance=7.0, left_runoff="DeepGravel", wall_type="TireWall"), # 14: Hairpin
        S(60, wall_type="Steel", wall_distance=7.0, runoff="Grass"),                                        # 15
        # Final turn to East (0 deg)
        A(40, -90, right_curb=True, left_wall_distance=12.0, right_wall_distance=7.0, left_runoff="DeepGravel", wall_type="TireWall"), # 16
    ],
    close_with=(13, 15),
    finish_at=(0, 0.4),
    profile=[
        ((0, 0.0), 0.0),
        ((0, 0.4), 0.0),   # Finish line at 0.0 m
        ((1, 0.0), 0.0),   # Climb starts
        ((1, 1.0), 1.0),
        ((2, 0.5), 2.2),
        ((2, 1.0), 3.6),
        ((3, 1.0), 5.2),   # Esse 1
        ((4, 1.0), 6.8),
        ((5, 0.7), 8.0),   # Esse 2 Crest at 8.0 m!
        ((6, 0.5), 6.8),   # Downhill corner descending
        ((7, 0.5), 5.4),   # Downhill run
        ((8, 0.5), 4.2),   # Chicane entry
        ((9, 0.5), 3.2),
        ((10, 0.5), 2.2),  # Chicane exit
        ((11, 0.5), 1.3),
        ((12, 0.5), 0.6),
        ((12, 1.0), 0.0),  # Back to ground level
        ((13, 0.5), 0.0),
    ],
    features=[
        Zone(start=420.0, end=495.0, surface="DeepGravel", lateral=(2.5, 8.5), from_edge="left", name="Downhill Gravel Trap"),
        Zone(start=925.0, end=985.0, surface="DeepGravel", lateral=(2.5, 8.5), from_edge="left", name="Hairpin Gravel Trap"),
        # Grandstands: main straight covered stadium stands
        GrandstandProp(at=40.0, side="left", offset=9.0, length=45.0, depth=8.0, style="covered_stadium", tiers=8),
        GrandstandProp(at=95.0, side="left", offset=9.0, length=40.0, depth=8.0, style="open_bleachers", tiers=7),
        # Buildings: control tower and pit garages
        BuildingProp(at=20.0, side="right", offset=10.0, width=10.0, depth=10.0, style="control_tower"),
        BuildingProp(at=60.0, side="right", offset=10.0, width=32.0, depth=9.0, style="pit_garage"),
        BuildingProp(at=110.0, side="right", offset=10.0, width=30.0, depth=9.0, style="pit_garage"),
        # Lake: infield lake
        Spot(at=250.0, lateral=35.0, radius=20.0, surface="Water", name="Ridge Ring Lake"),
        # Trees: cypress and oaks (48 trees)
        tree_row(20.0, 180.0, 12, side="left", offset=14.0, tree_type="cypress"),
        tree_row(220.0, 420.0, 12, side="right", offset=14.0, tree_type="oak"),
        tree_row(500.0, 700.0, 12, side="left", offset=14.0, tree_type="cypress"),
        tree_row(750.0, 950.0, 12, side="right", offset=14.0, tree_type="oak"),
    ],
)

COASTAL_GRAND_PRIX = Circuit(
    id="gt_coastal_grand_prix",
    name="Coastal Grand Prix",
    description="Premier coastal Grand Prix circuit featuring a 410 m blast, an elevated 5-meter plateau, a sweeping 165-degree seaside carousel, and the notorious bus-stop chicane.",
    tag="GRAND PRIX",
    category_label="GT Circuit",
    car_category="gt",
    default_laps=3,
    road=GT_ROAD,
    start=(0.0, 0.0),
    heading_deg=0.0,
    segments=[
        # Main Straight (>= 400 m) East (0 deg)
        S(330, wall_type="Steel", wall_distance=7.0, runoff="Grass"),                                       # 0
        S(80, wall_type="TireWall", wall_distance=8.0, runoff="Grass"),                                  # 1: Braking zone (total straight = 410 m)
        # Turn 1: Right kink to South-East (-30 deg)
        A(60, -30, right_curb=True, left_wall_distance=12.0, right_wall_distance=8.0, left_runoff="DeepGravel", wall_type="TireWall"), # 2
        # Climb to plateau
        S(60, wall_type="Steel", wall_distance=8.0, runoff="Grass"),                                        # 3
        # Curve on plateau to East (0 deg)
        A(60, 30, left_curb=True, wall_distance=8.0, left_runoff="Grass", right_runoff="Grass", wall_type="TireWall"),              # 4
        # Plateau straight (~5 m)
        S(60, wall_type="Steel", wall_distance=8.0, runoff="Grass"),                                        # 5
        # The Carousel: long 165-degree sweeping turn (to -165 deg / West-South-West)
        A(50, -165, right_curb=True, left_wall_distance=12.0, right_wall_distance=8.0, left_runoff="DeepGravel", wall_type="TireWall"), # 6: Carousel (165 deg >= 150 deg!)
        # Fast sweeper bend along the coast (left kink then right kink)
        S(40, wall_type="TireWall", left_wall_distance=12.0, right_wall_distance=8.0, left_runoff="DeepGravel"),                    # 7
        A(70, 45, left_curb=True, right_wall_distance=12.0, left_wall_distance=8.0, right_runoff="DeepGravel", wall_type="TireWall"), # 8: Fast sweeper (to -120 deg)
        S(80, wall_type="Steel", wall_distance=8.0, runoff="Grass"),                                        # 9
        A(70, -45, right_curb=True, left_wall_distance=12.0, right_wall_distance=8.0, left_runoff="DeepGravel", wall_type="TireWall"),   # 10: (to -165 deg)
        # Coastal straight into Bus-Stop Chicane
        S(100, wall_type="Steel", wall_distance=7.0, runoff="Grass"),                                       # 11
        S(40, wall_type="TireWall", wall_distance=8.0, runoff="Grass"),                                  # 12: Braking zone
        # Bus-Stop Chicane
        A(40, -45, right_curb=True, wall_distance=6.0, runoff="DeepGravel", wall_type="TireWall"),          # 13
        S(15, left_curb=True, right_curb=True, wall_distance=6.0, runoff="DeepGravel", wall_type="TireWall"),# 14
        A(40, 90, left_curb=True, wall_distance=6.0, runoff="DeepGravel", wall_type="TireWall"),            # 15
        S(15, left_curb=True, right_curb=True, wall_distance=6.0, runoff="DeepGravel", wall_type="TireWall"),# 16
        A(40, -45, right_curb=True, wall_distance=6.0, runoff="DeepGravel", wall_type="TireWall"),          # 17
        S(30, wall_type="Steel", wall_distance=8.0, runoff="Grass"),                                        # 18
        # Final turns back to Main Straight
        A(50, -45, right_curb=True, left_wall_distance=12.0, right_wall_distance=8.0, left_runoff="DeepGravel", wall_type="TireWall"), # 19 (to -210 / +150 deg)
        S(100, wall_type="Steel", wall_distance=7.0, runoff="Grass"),                                       # 20 (West-North)
        A(50, -90, right_curb=True, left_wall_distance=12.0, right_wall_distance=8.0, left_runoff="DeepGravel", wall_type="TireWall"), # 21 (to +60 deg)
        S(60, wall_type="Steel", wall_distance=7.0, runoff="Grass"),                                        # 22 (solved)
        A(50, -60, right_curb=True, left_wall_distance=12.0, right_wall_distance=7.0, left_runoff="DeepGravel", wall_type="TireWall"), # 23 (to 0 deg / East)
    ],
    close_with=(20, 22),
    finish_at=(0, 0.4),
    profile=[
        ((0, 0.0), 0.0),
        ((0, 0.4), 0.0),   # Finish line at 0.0 m
        ((2, 0.0), 0.0),   # Climb starts
        ((2, 1.0), 1.5),
        ((3, 1.0), 3.5),
        ((4, 1.0), 5.0),   # Plateau reached at 5.0 m
        ((5, 1.0), 5.0),   # Plateau straight flat at 5.0 m!
        ((6, 0.5), 4.5),   # Carousel gentle descent
        ((6, 1.0), 3.5),
        ((7, 1.0), 2.2),
        ((8, 1.0), 1.0),   # Fast sweeper
        ((9, 1.0), 0.0),   # Back to ground level
        ((11, 0.5), 0.0),
    ],
    features=[
        # Grandstands: main straight stadium stands
        GrandstandProp(at=50.0, side="left", offset=9.0, length=50.0, depth=8.0, style="covered_stadium", tiers=8),
        GrandstandProp(at=110.0, side="left", offset=9.0, length=45.0, depth=8.0, style="covered_stadium", tiers=8),
        # Buildings: control tower and pit garages
        BuildingProp(at=25.0, side="right", offset=10.0, width=10.0, depth=10.0, style="control_tower"),
        BuildingProp(at=70.0, side="right", offset=10.0, width=35.0, depth=9.0, style="pit_garage"),
        BuildingProp(at=125.0, side="right", offset=10.0, width=35.0, depth=9.0, style="pit_garage"),
        # Lake: Coastal GP lagoon
        Spot(at=350.0, lateral=45.0, radius=25.0, surface="Water", name="Coastal GP Lagoon"),
        # Trees: cypress and oaks (54 trees)
        tree_row(20.0, 220.0, 12, side="left", offset=14.0, tree_type="cypress"),
        tree_row(260.0, 500.0, 14, side="right", offset=14.0, tree_type="oak"),
        tree_row(550.0, 800.0, 14, side="left", offset=14.0, tree_type="cypress"),
        tree_row(850.0, 1100.0, 14, side="right", offset=14.0, tree_type="oak"),
    ],
)

# Stock Cars: banked ovals (spec 055, User Flow section 3).
# Races run anticlockwise (left turns), outer wall is right, bank positive in turns.
# Infield default_surface Grass.

STOCK_ROAD = Road(
    width=16.0,
    surface="Concrete",
    wall_type="Concrete",
    left_wall_distance=6.0,
    right_wall_distance=1.2,
    left_runoff="Concrete",
    right_runoff="Grass",
    bank=5.0,
)

THUNDER_BOWL = Circuit(
    id="stock_thunder_bowl",
    name="Thunder Bowl",
    description="High-banked short-track colosseum featuring 25-degree banked concrete turns and thunderous pack racing.",
    tag="SHORT TRACK",
    category_label="Stock Oval",
    car_category="nascar",
    default_laps=10,
    default_surface="Grass",
    road=STOCK_ROAD,
    start=(0.0, 0.0),
    heading_deg=0.0,
    segments=[
        S(140.0, bank=5.0),
        A(35.0, 180.0, bank=25.0, left_wall_distance=6.0, right_wall_distance=1.2, left_runoff="Concrete"),
        S(140.0, bank=5.0),
        A(35.0, 180.0, bank=25.0, left_wall_distance=6.0, right_wall_distance=1.2, left_runoff="Concrete"),
    ],
    finish_at=(0, 0.5),
    features=[
        # Grandstands: main straight and back straight stadium stands
        GrandstandProp(at=470.0, side="right", offset=3.5, length=45.0, depth=8.0, style="covered_stadium", tiers=8),
        GrandstandProp(at=35.0, side="right", offset=3.5, length=45.0, depth=8.0, style="covered_stadium", tiers=8),
        GrandstandProp(at=250.0, side="right", offset=3.5, length=55.0, depth=8.0, style="open_bleachers", tiers=8),
        # Buildings: infield control tower and pit garages
        BuildingProp(at=15.0, side="left", offset=8.5, width=10.0, depth=10.0, style="control_tower"),
        BuildingProp(at=45.0, side="left", offset=8.5, width=28.0, depth=9.0, style="pit_garage"),
        BuildingProp(at=250.0, side="left", offset=8.5, width=32.0, depth=9.0, style="pit_garage"),
        # Lake: infield lake centered at (70, 35)
        Spot(at=0.0, lateral=35.0, radius=16.0, surface="Water", name="Thunder Bowl Infield Lake"),
        # Trees: oaks, cypress (40 trees)
        tree_row(190.0, 310.0, 12, side="right", offset=6.0, tree_type="oak"),
        tree_row(440.0, 500.0, 6, side="right", offset=16.0, tree_type="cypress"),
        tree_row(0.0, 60.0, 6, side="right", offset=16.0, tree_type="cypress"),
        tree_row(80.0, 160.0, 8, side="right", offset=6.0, tree_type="oak"),
        tree_row(340.0, 420.0, 8, side="right", offset=6.0, tree_type="oak"),
    ],
)

TRI_OVAL_ROAD = Road(
    width=16.0,
    surface="Asphalt",
    wall_type="Concrete",
    left_wall_distance=6.0,
    right_wall_distance=1.2,
    left_runoff="Asphalt",
    right_runoff="Grass",
    bank=5.0,
)

TRI_OVAL = Circuit(
    id="stock_tri_oval_speedway",
    name="Tri-Oval Speedway",
    description="Superspeedway tri-oval with 20-degree banked turns, an 8-degree front-stretch dogleg and high-speed drafting.",
    tag="TRI-OVAL",
    category_label="Stock Oval",
    car_category="nascar",
    default_laps=6,
    default_surface="Grass",
    road=TRI_OVAL_ROAD,
    start=(0.0, 0.0),
    heading_deg=180.0,
    segments=[
        S(322.72, bank=5.0),        # 0: Back straight (180 deg)
        A(65.0, 150.0, bank=20.0),  # 1: Turn 3 & 4 (to -30 deg)
        S(166.11, bank=8.0),        # 2: Front stretch 1 (-30 deg)
        A(100.0, 60.0, bank=8.0),   # 3: Dogleg (to +30 deg)
        S(166.11, bank=8.0),        # 4: Front stretch 2 (+30 deg)
        A(65.0, 150.0, bank=20.0),  # 5: Turn 1 & 2 (to 180 deg)
    ],
    finish_at=(3, 0.5),  # finish at apex of dogleg
    features=[
        # Grandstands: front stretch covered stadium and bleachers
        GrandstandProp(at=530.0, side="right", offset=3.5, length=60.0, depth=8.0, style="covered_stadium", tiers=8),
        GrandstandProp(at=600.0, side="right", offset=3.5, length=55.0, depth=8.0, style="covered_stadium", tiers=8),
        GrandstandProp(at=800.0, side="right", offset=3.5, length=60.0, depth=8.0, style="covered_stadium", tiers=8),
        GrandstandProp(at=870.0, side="right", offset=3.5, length=55.0, depth=8.0, style="open_bleachers", tiers=8),
        # Buildings: infield control tower and pit garages
        BuildingProp(at=520.0, side="left", offset=9.0, width=10.0, depth=10.0, style="control_tower"),
        BuildingProp(at=560.0, side="left", offset=9.0, width=35.0, depth=9.0, style="pit_garage"),
        BuildingProp(at=610.0, side="left", offset=9.0, width=35.0, depth=9.0, style="pit_garage"),
        # Lake: infield lake
        Spot(at=160.0, lateral=60.0, radius=30.0, surface="Water", name="Tri-Oval Infield Lake"),
        # Trees: oaks, cypress, pines (48 trees)
        tree_row(30.0, 290.0, 16, side="right", offset=6.0, tree_type="oak"),
        tree_row(520.0, 640.0, 10, side="right", offset=16.0, tree_type="cypress"),
        tree_row(790.0, 910.0, 10, side="right", offset=16.0, tree_type="cypress"),
        tree_row(30.0, 290.0, 12, side="left", offset=25.0, tree_type="pine"),
    ],
)

STOCK_ROVAL_ROAD = Road(
    width=14.0,
    surface="Asphalt",
    wall_type="Concrete",
    left_wall_distance=5.0,
    right_wall_distance=1.2,
    bank=5.0,
)

ROVAL = Circuit(
    id="stock_roval",
    name="Roval",
    description="Hybrid oval and road course combining high-banked 18-degree oval turns with a technical infield hairpin and chicane.",
    tag="ROVAL",
    category_label="Stock Roval",
    car_category="nascar",
    default_laps=5,
    default_surface="Grass",
    road=STOCK_ROVAL_ROAD,
    start=(0.0, 0.0),
    heading_deg=0.0,
    segments=[
        S(260.0, bank=5.0),                                                                            # 0: Front straight
        A(45.0, 60.0, left_curb=True, bank=0.0, wall_type="TireWall", left_wall_distance=5.0, right_wall_distance=5.0), # 1: Turn into infield
        S(50.0, bank=0.0, wall_type="TireWall", left_wall_distance=5.0, right_wall_distance=5.0),       # 2
        A(40.0, -45.0, right_curb=True, bank=0.0, wall_type="TireWall", left_wall_distance=5.0, right_wall_distance=5.0), # 3: Chicane entry
        S(25.0, left_curb=True, right_curb=True, bank=0.0, wall_type="TireWall", left_wall_distance=5.0, right_wall_distance=5.0), # 4: Chicane mid
        A(40.0, 45.0, left_curb=True, bank=0.0, wall_type="TireWall", left_wall_distance=5.0, right_wall_distance=5.0),  # 5: Chicane exit
        S(80.0, bank=0.0, wall_type="TireWall", left_wall_distance=5.0, right_wall_distance=5.0),       # 6
        A(32.0, 165.0, left_curb=True, bank=0.0, wall_type="TireWall", left_wall_distance=5.0, right_wall_distance=5.0), # 7: Hairpin
        S(70.0, bank=0.0, wall_type="TireWall", left_wall_distance=5.0, right_wall_distance=5.0),       # 8: Hairpin exit (solved)
        A(45.0, -45.0, right_curb=True, bank=0.0, wall_type="TireWall", left_wall_distance=5.0, right_wall_distance=5.0),# 9: Rejoin oval
        S(250.0, bank=5.0, wall_type="Concrete", left_wall_distance=5.0, right_wall_distance=1.2),     # 10: Back straight (solved)
        A(60.0, 180.0, bank=18.0, wall_type="Concrete", left_wall_distance=5.0, right_wall_distance=1.2),# 11: Turn 3 & 4
        S(80.0, bank=5.0, wall_type="Concrete", left_wall_distance=5.0, right_wall_distance=1.2),      # 12: Lead back to start
    ],
    close_with=(8, 10),
    finish_at=(0, 0.4),
    features=[
        # Grandstands: front straight covered stadium stands and bleachers
        GrandstandProp(at=40.0, side="right", offset=3.5, length=50.0, depth=8.0, style="covered_stadium", tiers=8),
        GrandstandProp(at=100.0, side="right", offset=3.5, length=50.0, depth=8.0, style="covered_stadium", tiers=8),
        GrandstandProp(at=160.0, side="right", offset=3.5, length=45.0, depth=8.0, style="open_bleachers", tiers=8),
        # Buildings: infield control tower and pit garages
        BuildingProp(at=20.0, side="left", offset=8.0, width=10.0, depth=10.0, style="control_tower"),
        BuildingProp(at=55.0, side="left", offset=8.0, width=32.0, depth=9.0, style="pit_garage"),
        BuildingProp(at=95.0, side="left", offset=8.0, width=32.0, depth=9.0, style="pit_garage"),
        # Lake: infield lake
        Spot(at=50.0, lateral=55.0, radius=22.0, surface="Water", name="Roval Infield Lake"),
        # Trees: oaks, pines, cypress (46 trees)
        tree_row(760.0, 980.0, 14, side="right", offset=6.0, tree_type="oak"),
        tree_row(320.0, 480.0, 10, side="right", offset=8.0, tree_type="pine"),
        tree_row(520.0, 680.0, 10, side="left", offset=8.0, tree_type="oak"),
        tree_row(30.0, 200.0, 12, side="right", offset=15.0, tree_type="cypress"),
    ],
)

CREST_HEIGHTS = [
    3.8, 4.2, 4.0, 4.5, 5.0, 5.6, 4.8, 4.2, 4.6, 4.0,
    4.5, 3.8, 4.2, 3.8, 4.0, 3.5, 4.2, 3.8, 4.0, 3.8,
]
TROUGH_HEIGHTS = [
    0.5, 1.2, 1.0, 1.5, 1.8, 2.0, 1.5, 1.2, 1.6, 1.0,
    1.4, 0.8, 1.2, 0.2, 0.8, 0.5, 1.0, 0.8, 1.2, 0.5,
]

DUNE_PROFILE = []
for i in range(20):
    trough_d = i * 60.0
    crest_d = trough_d + 30.0
    DUNE_PROFILE.append((round(trough_d, 1), TROUGH_HEIGHTS[i]))
    DUNE_PROFILE.append((round(crest_d, 1), CREST_HEIGHTS[i]))

DUNE_SEA = Circuit(
    id="at_dune_sea",
    name="Dune Sea",
    description="Rolling desert all-terrain circuit with sweeping sand dunes, dune crest leaps, tabletop ramps, and a desert oasis.",
    tag="SAND DUNES",
    category_label="All-Terrain",
    car_category="offroad",
    default_laps=4,
    default_surface="DeepSand",
    road=Road(width=15.0, surface="PackedSand", left_runoff="DeepSand", right_runoff="DeepSand", wall_type="TireWall", left_wall_distance=3.5, right_wall_distance=3.5),
    start=(0.0, 0.0),
    heading_deg=0.0,
    segments=[
        S(140.0),                  # 0: Start straight (0 deg, East)
        A(50.0, -90.0),            # 1: Turn 1 (to South, -90 deg)
        S(80.0),                   # 2:
        A(50.0, -45.0),            # 3: (-135 deg)
        S(90.0),                   # 4:
        A(50.0, 45.0),             # 5: Kink left (-90 deg)
        S(70.0),                   # 6:
        A(45.0, -90.0),            # 7: (-180 deg, West)
        S(120.0),                  # 8:
        A(50.0, -90.0),            # 9: (+90 deg, North)
        S(90.0),                   # 10:
        A(50.0, -45.0),            # 11: (+45 deg)
        S(60.0),                   # 12:
        A(50.0, -45.0),            # 13: (0 deg, East)
    ],
    close_with=(0, 10),
    profile=DUNE_PROFILE,
    features=[
        Ramp(at=390.0, length=14.0, height=2.0, launch_speed=4.0, surface="PackedSand", name="Dune Tabletop 1"),
        Ramp(at=930.0, length=14.0, height=2.0, launch_speed=4.0, surface="PackedSand", name="Dune Tabletop 2"),
        Zone(start=660.0, end=720.0, surface="Water", lateral=(7.5, 18.0), from_edge="left", layer="below_track", name="Oasis Shore"),
        Spot(at=690.0, lateral=26.0, radius=12.0, surface="Water", name="Dune Oasis Pool"),
        # Grandstands: open bleachers along start straight
        GrandstandProp(at=40.0, side="left", offset=6.5, length=32.0, depth=6.0, style="open_bleachers", tiers=6),
        GrandstandProp(at=90.0, side="left", offset=6.5, length=32.0, depth=6.0, style="open_bleachers", tiers=6),
        # Buildings: control tower and desert paddock tents
        BuildingProp(at=15.0, side="right", offset=7.5, width=8.0, depth=8.0, style="control_tower"),
        BuildingProp(at=50.0, side="right", offset=7.5, width=18.0, depth=8.0, style="paddock_tent"),
        BuildingProp(at=75.0, side="right", offset=7.5, width=18.0, depth=8.0, style="paddock_tent"),
        BuildingProp(at=100.0, side="right", offset=7.5, width=18.0, depth=8.0, style="paddock_tent"),
        # Rocks: desert sandstone rocks (12 rocks)
        rock_cluster(at=200.0, count=6, side="left", offset=7.5, rock_type="sandstone", span=16.0),
        rock_cluster(at=550.0, count=6, side="right", offset=7.5, rock_type="sandstone", span=16.0),
        # Trees/plants: oasis palms and desert cacti (40 plants)
        tree_row(640.0, 740.0, 10, side="left", offset=8.0, tree_type="palm"),
        tree_row(160.0, 320.0, 10, side="right", offset=7.5, tree_type="cactus"),
        tree_row(360.0, 520.0, 10, side="left", offset=7.5, tree_type="cactus"),
        tree_row(780.0, 960.0, 10, side="right", offset=7.5, tree_type="palm"),
    ],
)

MUDBATH_VALLEY = Circuit(
    id="at_mudbath_valley",
    name="Mudbath Valley",
    description="Treacherous off-road mud arena through sunken river valleys, deep mud bogs, standing water puddles, a whoops section and an 8-meter hill.",
    tag="MUD BATH",
    category_label="All-Terrain",
    car_category="offroad",
    default_laps=4,
    default_surface="DeepMud",
    road=Road(width=15.0, surface="MudTrack", left_runoff="DeepMud", right_runoff="DeepMud", wall_type="TireWall", left_wall_distance=3.5, right_wall_distance=3.5),
    start=(0.0, 0.0),
    heading_deg=0.0,
    segments=[
        S(70.0),                   # 0: Start straight (East, 0 deg)
        A(40.0, -90.0),            # 1: Turn 1 (South, -90 deg)
        S(50.0),                   # 2: Mud straight
        A(35.0, -45.0),            # 3: (-135 deg)
        S(70.0),                   # 4: Hill climb
        A(35.0, 45.0),             # 5: Crest kink (-90 deg)
        S(50.0),                   # 6: Hill descent
        A(40.0, -90.0),            # 7: (-180 deg, West)
        S(80.0),                   # 8: Valley floor / whoops
        A(35.0, -45.0),            # 9: (+135 deg)
        S(50.0),                   # 10:
        A(40.0, -45.0),            # 11: (+90 deg, North)
        S(50.0),                   # 12: Return straight
        A(40.0, -90.0),            # 13: (0 deg, East)
    ],
    close_with=(0, 12),
    profile=[
        ((0, 0.0), 0.0),
        ((0, 0.4), 0.0),
        ((3, 1.0), 0.5),
        ((4, 0.5), 4.5),
        ((4, 1.0), 7.5),
        ((5, 0.5), 8.0),
        ((5, 1.0), 7.8),
        ((6, 0.5), 4.0),
        ((6, 1.0), 0.5),
        ((7, 0.5), 0.0),
        ((12, 1.0), 0.0),
    ],
    finish_at=(0, 0.4),
    features=[
        Ramp(at=60.0, length=14.0, height=2.0, launch_speed=4.0, surface="MudTrack", name="Valley Tabletop 1"),
        Ramp(at=780.0, length=14.0, height=2.0, launch_speed=4.0, surface="MudTrack", name="Valley Tabletop 2"),
        Whoops(at=480.0, count=6, spacing=5.0, height=0.6, name="Valley Whoops", surface="MudTrack"),
        Zone(start=180.0, end=210.0, surface="DeepMud", lateral=(-5.0, 5.0), layer="above_track", name="Deep Mud Bog 1"),
        Zone(start=570.0, end=595.0, surface="DeepMud", lateral=(-4.5, 4.5), layer="above_track", name="Deep Mud Bog 2"),
        Zone(start=510.0, end=530.0, surface="Water", lateral=(-4.5, 4.5), layer="above_track", name="Mud Puddle 1"),
        Zone(start=680.0, end=700.0, surface="Water", lateral=(-4.5, 4.5), layer="above_track", name="Mud Puddle 2"),
        # Grandstands: hillside bleachers
        GrandstandProp(at=920.0, side="left", offset=6.5, length=28.0, depth=6.0, style="hillside_bleachers", tiers=5),
        GrandstandProp(at=20.0, side="left", offset=6.5, length=28.0, depth=6.0, style="hillside_bleachers", tiers=5),
        GrandstandProp(at=730.0, side="right", offset=6.5, length=30.0, depth=6.0, style="hillside_bleachers", tiers=5),
        # Buildings: control tower and paddock tents
        BuildingProp(at=910.0, side="right", offset=7.5, width=8.0, depth=8.0, style="control_tower"),
        BuildingProp(at=935.0, side="right", offset=7.5, width=16.0, depth=8.0, style="paddock_tent"),
        BuildingProp(at=15.0, side="right", offset=7.5, width=18.0, depth=8.0, style="paddock_tent"),
        # Rocks: slate rock clusters (12 rocks)
        rock_cluster(at=220.0, count=6, side="left", offset=7.5, rock_type="slate", span=16.0),
        rock_cluster(at=600.0, count=6, side="right", offset=7.5, rock_type="slate", span=16.0),
        # Trees/plants: oaks and mud bushes (40 plants)
        tree_row(140.0, 260.0, 10, side="right", offset=7.5, tree_type="oak"),
        tree_row(320.0, 440.0, 10, side="left", offset=7.5, tree_type="bush"),
        tree_row(500.0, 640.0, 10, side="left", offset=7.5, tree_type="oak"),
        tree_row(680.0, 820.0, 10, side="right", offset=7.5, tree_type="bush"),
    ],
)

FROSTBITE_PASS = Circuit(
    id="at_frostbite_pass",
    name="Frostbite Pass",
    description="Sub-zero mountain circuit climbing over a treacherous 10-meter alpine snow pass before sweeping across a frozen sheet-ice glacial lake.",
    tag="SNOW PASS",
    category_label="All-Terrain",
    car_category="offroad",
    default_laps=4,
    default_surface="DeepSnow",
    road=Road(width=15.0, surface="PackedSnow", left_runoff="PackedSnow", right_runoff="PackedSnow", wall_type="TireWall", left_wall_distance=3.5, right_wall_distance=3.5),
    start=(0.0, 0.0),
    heading_deg=0.0,
    segments=[
        S(90.0),                                                           # 0: Start straight (East, 0 deg)
        A(60.0, -90.0),                                                    # 1: Turn 1 (South, -90 deg)
        S(60.0),                                                           # 2: Mountain pass approach
        A(50.0, -45.0),                                                    # 3: (-135 deg)
        S(70.0, left_runoff="DeepSnow", right_runoff="DeepSnow"),          # 4: Pass climb 1
        A(45.0, 45.0),                                                     # 5: Kink (-90 deg)
        S(70.0, left_runoff="DeepSnow", right_runoff="DeepSnow"),          # 6: Pass climb 2 (crest)
        A(55.0, -90.0),                                                    # 7: Crest turn (-180 deg, West)
        S(80.0, left_runoff="DeepSnow", right_runoff="DeepSnow"),          # 8: Glacial descent
        A(50.0, -45.0),                                                    # 9: (-225 / +135 deg)
        S(60.0, left_runoff="DeepSnow", right_runoff="DeepSnow"),          # 10: Lake approach
        A(45.0, -45.0),                                                    # 11: (+90 deg, North)
        S(120.0, surface="SheetIce", left_runoff="DeepSnow", right_runoff="DeepSnow"), # 12: Frozen lake sheet ice straight!
        S(60.0),                                                           # 13: Snow braking straight!
        A(55.0, -90.0),                                                    # 14: (0 deg, East)
    ],
    close_with=(0, 13),
    profile=[
        ((0, 0.0), 0.0),
        ((0, 0.4), 0.0),
        ((2, 0.5), 0.5),
        ((3, 1.0), 2.5),
        ((4, 0.5), 5.5),
        ((4, 1.0), 8.0),
        ((5, 1.0), 9.2),
        ((6, 0.5), 10.0),
        ((6, 1.0), 9.8),
        ((7, 1.0), 7.5),
        ((8, 0.5), 3.5),
        ((8, 1.0), 0.5),
        ((9, 1.0), 0.0),
        ((13, 1.0), 0.0),
    ],
    finish_at=(0, 0.4),
    features=[
        Ramp(at=55.0, length=14.0, height=2.0, launch_speed=4.0, surface="PackedSnow", name="Pass Tabletop Jump"),
        # Grandstands: hillside bleachers
        GrandstandProp(at=1120.0, side="left", offset=6.5, length=30.0, depth=6.0, style="hillside_bleachers", tiers=5),
        GrandstandProp(at=25.0, side="left", offset=6.5, length=30.0, depth=6.0, style="hillside_bleachers", tiers=5),
        # Buildings: alpine control tower and tents
        BuildingProp(at=1115.0, side="right", offset=7.5, width=8.0, depth=8.0, style="control_tower"),
        BuildingProp(at=1140.0, side="right", offset=7.5, width=18.0, depth=8.0, style="paddock_tent"),
        BuildingProp(at=20.0, side="right", offset=7.5, width=18.0, depth=8.0, style="paddock_tent"),
        # Rocks: snow-capped rock clusters (12 rocks)
        rock_cluster(at=320.0, count=6, side="left", offset=7.5, rock_type="snow_capped", span=16.0),
        rock_cluster(at=600.0, count=6, side="right", offset=7.5, rock_type="snow_capped", span=16.0),
        # Trees/plants: snow pines and mountain bushes (42 plants)
        tree_row(140.0, 280.0, 10, side="right", offset=7.5, tree_type="snow_pine"),
        tree_row(360.0, 520.0, 10, side="left", offset=7.5, tree_type="snow_pine"),
        tree_row(640.0, 780.0, 10, side="right", offset=7.5, tree_type="bush"),
        tree_row(820.0, 1020.0, 12, side="left", offset=8.0, tree_type="snow_pine"),
    ],
)


CIRCUITS = [
    PINE_GROVE,
    RIVERBEND_CIRCUIT,
    SUMMIT_INTERNATIONAL,
    QUARRY_SPRINT,
    HILLTOP_LEAP,
    CANYON_FLYER,
    MEADOW_SPRINT,
    CLAY_BOWL,
    HILLSIDE_HAMMER,
    VELOCITY_PARK,
    RIDGE_RING,
    COASTAL_GRAND_PRIX,
    THUNDER_BOWL,
    TRI_OVAL,
    ROVAL,
    DUNE_SEA,
    MUDBATH_VALLEY,
    FROSTBITE_PASS,
]





def main(argv=None):
    parser = argparse.ArgumentParser(
        description="Build the Classic circuits of spec 055 into tracks/classic/"
    )
    parser.add_argument(
        "--only", action="append", metavar="ID", help="Build one circuit (repeatable)"
    )
    parser.add_argument(
        "--density-mode",
        choices=[
            MODE_ANGULAR_45,
            MODE_RADIAL_30,
            MODE_APEX_CAD,
            MODE_CHORD_SAGITTA,
            MODE_LEGACY_UNIFORM,
        ],
        default=MODE_ANGULAR_45,
        help="Waypoint density strategy: 'angular_45' (adopted default, Approach 1: ~45° per knot), "
             "'radial_30' (Approach 2: ~30° per knot, 60m straight spans), "
             "'apex_cad' (Approach 3: canonical apex-centered CAD knots), "
             "'chord_sagitta' (chord-sagitta budget <= 0.08m), "
             "'legacy_uniform' (uniform step spacing)",
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
        gap, dh = closure_gap(c, density_mode=args.density_mode)
        length = sum(s.length for s in segments_of(c))
        print(
            f"{c.id}: {length:.0f} m of segments, closure gap {gap:.2f} m / {dh:+.2f} deg"
        )

    ok = check(circuits, density_mode=args.density_mode) if args.check else build(circuits, TRACKS_DIR, density_mode=args.density_mode)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
