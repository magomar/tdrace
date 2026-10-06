"""
Unit tests for the Classic circuit builder (scripts/classic_circuit_builder.py, spec 055).

The geometry tests are pure Python. The build test runs track_bake through cargo on a small circuit in a
temporary tracks folder; it is skipped when cargo is not installed.
"""

import json
import math
import os
import shutil
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "scripts"))

import classic_circuit_builder as ccb


def stadium(**overrides):
    """A 520 m stadium loop that starts in the middle of a straight."""
    args = {
        "id": "test_stadium",
        "name": "Test Stadium",
        "description": "Builder test loop.",
        "tag": "TEST",
        "category_label": "Test",
        "car_category": "rally",
        "default_laps": 3,
        "road": ccb.Road(width=12.0, surface="Dirt", wall_type="TireWall"),
        "segments": [
            ccb.Straight(75),
            ccb.Arc(35, 180, bank=6),
            ccb.Straight(150, elev=4),
            ccb.Arc(35, 180, curbs=True),
            ccb.Straight(75),
        ],
    }
    args.update(overrides)
    return ccb.Circuit(**args)


def gaps(wps):
    pts = [w["point"] for w in wps]
    return [math.dist(pts[i], pts[(i + 1) % len(pts)]) for i in range(len(pts))]


def test_left_arc_turns_towards_the_normal():
    c = stadium(segments=[ccb.Arc(50, 90)])
    _points, (x, y, heading) = ccb.trace(c)
    assert (round(x, 6), round(y, 6)) == (50.0, 50.0)
    assert math.degrees(heading) == pytest.approx(90.0)


def test_right_arc_turns_away_from_the_normal():
    c = stadium(segments=[ccb.Arc(50, -90)])
    _points, (x, y, _h) = ccb.trace(c)
    assert (round(x, 6), round(y, 6)) == (50.0, -50.0)


def test_closed_lap_gives_spaced_waypoints_starting_at_the_start():
    c = stadium()
    wps = ccb.waypoints(c)
    assert wps[0]["point"] == [0.0, 0.0]
    assert min(gaps(wps)) >= 3.0
    # Spec 071: a straight keeps only its end points (and ease anchors), so the longest gap is the 75 m straight.
    assert max(gaps(wps)) <= 75.0 + 0.01
    assert ccb.closure_gap(c)[0] == pytest.approx(0.0, abs=1e-6)


def test_open_lap_is_refused():
    c = stadium(
        segments=[
            ccb.Straight(75),
            ccb.Arc(35, 180),
            ccb.Straight(150),
            ccb.Arc(35, 170),
        ]
    )
    with pytest.raises(ValueError, match="does not close"):
        ccb.waypoints(c)


def test_small_closure_gap_is_spread_along_the_lap():
    c = stadium(
        segments=[
            ccb.Straight(75.3),
            ccb.Arc(35, 180),
            ccb.Straight(150),
            ccb.Arc(35, 180),
            ccb.Straight(75),
        ]
    )
    wps = ccb.waypoints(c)
    assert min(gaps(wps)) >= 3.0
    assert max(gaps(wps)) <= 75.3 + 0.05


def test_elevation_and_bank_ease_between_segments():
    wps = ccb.waypoints(stadium())
    elevations = [w["elevation"] for w in wps]
    top = elevations.index(max(elevations))
    assert max(elevations) == pytest.approx(4.0, abs=0.1)
    rising = elevations[: top + 1]
    assert rising == sorted(rising) and len(set(rising)) > 3
    assert max(w["bank_angle"] for w in wps) == pytest.approx(6.0, abs=0.1)


def test_attributes_left_out_take_the_circuit_default():
    wps = ccb.waypoints(stadium())
    curbed = [i for i, w in enumerate(wps) if w["left_curb"]]
    assert curbed and all(wps[i]["right_curb"] for i in curbed)
    assert not wps[curbed[-1] + 1]["left_curb"]
    assert {w["wall_type"] for w in wps} == {"TireWall"}
    assert {w["surface"] for w in wps} == {"Dirt"}


def test_unknown_segment_attribute_is_refused():
    with pytest.raises(ValueError, match="unknown segment attribute"):
        ccb.Straight(10, kerbs=True)


def straight_baked_track(length=100.0, width=10.0):
    samples = [
        {
            "point": [d, 0.0],
            "tangent": [1.0, 0.0],
            "normal": [0.0, 1.0],
            "distance": d,
            "width": width,
            "surface": "Dirt",
        }
        for d in range(0, int(length), 5)
    ]
    return {
        "spline": {"samples": samples, "total_length": length},
        "geometry": {},
        "checkpoints": [],
    }


def test_features_are_placed_by_lap_distance_and_side():
    c = stadium(
        features=[
            ccb.Ramp(at=20, length=10, height=2.0),
            ccb.Whoops(at=50, count=3, spacing=5, height=0.5),
            ccb.Zone(
                start=10, end=30, surface="Gravel", lateral=(1, 6), from_edge="left"
            ),
            ccb.Zone(
                start=10, end=30, surface="Water", lateral=(-2, 2), layer="above_track"
            ),
            ccb.Spot(at=40, lateral=-12, radius=4, surface="Water"),
        ]
    )
    track = straight_baked_track()
    ccb.place_features(track, c)
    ramps = track["geometry"]["jump_ramps"]
    assert [r["id"] for r in ramps] == [1, 2, 3, 4]
    box = ramps[0]["shape"]["OrientedBox"]
    assert (
        box["center"] == [25.0, 0.0]
        and box["half_extents"] == [5.0, 5.0]
        and box["angle"] == 0.0
    )
    assert ramps[0]["surface"] == "Dirt" and ramps[1]["name"] == "Whoops #1"
    gravel, water, spot = track["geometry"]["surface_zones"]
    ys = [v[1] for v in gravel["shape"]["Polygon"]["vertices"]]
    assert min(ys) == pytest.approx(6.0) and max(ys) == pytest.approx(11.0)
    assert water["layer"] == "above_track"
    assert spot["shape"]["Circle"]["center"] == [40.0, -12.0]


@pytest.mark.skipif(
    shutil.which("cargo") is None, reason="needs cargo to run track_bake"
)
def bridge_then(turn_after_m):
    """Baked samples 1 m apart: 30 m of bridge heading east, then road that turns left 90 deg over 10 m."""
    samples, x, y, h = [], 0.0, 0.0, 0.0
    for i in range(120):
        on_bridge = i < 30
        turning = turn_after_m <= i - 30 < turn_after_m + 10
        if turning:
            h += math.radians(9)
        samples.append(
            {
                "point": [x, y],
                "tangent": [math.cos(h), math.sin(h)],
                "distance": float(i),
                "is_bridge": on_bridge,
            }
        )
        x, y = x + math.cos(h), y + math.sin(h)
    return {"spline": {"samples": samples}}


def test_a_turn_right_after_a_bridge_is_refused():
    exits = ccb.turns_after_bridges(bridge_then(turn_after_m=8))
    assert len(exits) == 1
    assert exits[0][0] == 29.0
    assert exits[0][1] > 45


def test_a_straight_after_a_bridge_is_accepted():
    assert ccb.turns_after_bridges(bridge_then(turn_after_m=22)) == []


def test_build_bakes_validates_and_is_reproducible(tmp_path):
    shutil.copy(os.path.join(ccb.TRACKS_DIR, ".track_order.json"), tmp_path)
    c = stadium(
        features=[
            ccb.Ramp(at=40, length=12, height=1.5),
            ccb.Zone(
                start=80,
                end=180,
                surface="DeepSand",
                lateral=(0.5, 3.0),
                from_edge="right",
            ),
        ]
    )
    assert ccb.build([c], str(tmp_path))
    path = tmp_path / "classic" / "test_stadium.json"
    first = path.read_bytes()
    track = json.loads(first)
    assert (
        track["spline"]["samples"] and track["checkpoints"] and track["grid_positions"]
    )
    assert (
        len(track["geometry"]["jump_ramps"]) == 1
        and len(track["geometry"]["surface_zones"]) == 1
    )
    designed = sum(seg.length for seg in c.segments)
    assert track["spline"]["total_length"] == pytest.approx(designed, rel=0.01)
    order = json.loads((tmp_path / ".track_order.json").read_text())
    assert "test_stadium" in order["classic"]

    assert ccb.build([c], str(tmp_path))
    assert path.read_bytes() == first
