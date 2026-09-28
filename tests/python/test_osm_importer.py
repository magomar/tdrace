"""
Unit tests for the shared OSM circuit importer (scripts/osm_importer.py).

They use small synthetic lat/lon data only: no network and no OSM cache needed.
"""

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "scripts"))

import osm_importer as imp

# ~100 m square near Monza: 0.0009 deg of latitude is ~100 m
D = 0.0009
SQUARE = {"a": (45.0, 9.0), "b": (45.0, 9.0 + D * 1.41), "c": (45.0 + D, 9.0 + D * 1.41), "d": (45.0 + D, 9.0)}


def test_stitch_ways_flips_reversed_way_and_closes_loop():
    ways = [["a", "b"], ["c", "b"], ["c", "d", "a"]]
    assert imp.stitch_ways("t", ways, SQUARE) == ["a", "b", "c", "d", "a"]


def test_stitch_ways_refuses_far_way(capsys):
    coords = dict(SQUARE, x=(45.01, 9.0), y=(45.011, 9.0))  # ~1.1 km away
    assert imp.stitch_ways("t", [["a", "b"], ["x", "y"]], coords) == ["a", "b"]
    assert "stitching stopped" in capsys.readouterr().err


def test_stitch_ways_joins_small_gap():
    coords = dict(SQUARE, b2=(45.0, 9.0 + D * 1.41 + 0.0001))  # ~8 m from b
    assert imp.stitch_ways("t", [["a", "b"], ["b2", "c", "d", "a"]], coords) == ["a", "b", "b2", "c", "d", "a"]


def test_check_loop_joins_accepts_osm_edges_of_any_length():
    edges = imp.way_edges([["a", "b", "c", "d", "a"]])
    assert imp.check_loop_joins("t", ["a", "b", "c", "d"], SQUARE, edges) == 0


def test_check_loop_joins_flags_long_non_edge_jump(capsys):
    edges = imp.way_edges([["a", "b"], ["c", "d"]])  # b-c and d-a are not OSM edges
    assert imp.check_loop_joins("t", ["a", "b", "c", "d"], SQUARE, edges) == 2
    err = capsys.readouterr().err
    assert "chain index 1" in err and "loop closure" in err


def test_check_length_ratio(capsys):
    imp.check_length_ratio("t", 1050.0, 1000.0)
    assert capsys.readouterr().err == ""
    imp.check_length_ratio("t", 750.0, 1000.0)
    assert "0.750x" in capsys.readouterr().err


def test_deflection_sine_sign_and_spacing_independence():
    left_turn = [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0)]
    assert abs(imp.deflection_sine(left_turn, 1) - 1.0) < 1e-9
    right_turn_small = [(0.0, 0.0), (1.0, 0.0), (1.0, -1.0)]
    assert abs(imp.deflection_sine(right_turn_small, 1) + 1.0) < 1e-9


def test_resample_polyline_keeps_segment_props():
    square = [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)]
    pts, props, total = imp.resample_polyline(square, 8, ["A", "B", "C", "D"])
    assert total == 400.0
    assert pts[1] == (50.0, 0.0) and pts[2] == (100.0, 0.0)
    # A point exactly on a corner still belongs to the segment that ends there.
    assert props == ["A", "A", "A", "B", "B", "C", "C", "D"]


def test_provenance_registry_lists_all_real_circuits():
    urls = imp.provenance_osm_urls()
    assert len(urls) == 71
    assert all(u.startswith("https://www.openstreetmap.org/") for u in urls.values())


def test_every_importer_config_can_be_downloaded():
    urls = imp.provenance_osm_urls()
    for specs in (imp.GT_CIRCUITS, imp.KART_TRACKS, imp.RALLY_TRACKS):
        missing = [tid for tid in specs if tid not in urls]
        assert not missing, f"configs without a provenance osm_url: {missing}"


def test_expand_bbox_adds_margin_in_meters():
    s, w, n, e = imp.expand_bbox((45.0, 9.0, 45.0, 9.0), 111.32)
    assert abs((n - s) - 0.002) < 1e-9
    assert abs((e - w) - 0.002 / 0.70710678) < 1e-6


# --- JSON output (spec 042) ---

import json


def _mini_tracks(tmp_path):
    """A tracks/ folder with one GT circuit and the order/alias files."""
    (tmp_path / "gt").mkdir()
    existing = {
        "name": "Monza Autodromo Nazionale",
        "tag": "TEMPLE OF SPEED",
        "osm_url": "https://www.openstreetmap.org/way/1",
        "spline": {
            "waypoints": [{"point": [0, 0]}],
            "closed": True,
            "samples": [1],
            "total_length": 5.0,
        },
        "geometry": {"trees": [{"x": 1}]},
    }
    (tmp_path / "gt" / "monza.json").write_text(json.dumps(existing))
    (tmp_path / ".track_order.json").write_text(json.dumps({"gt": ["monza"]}))
    (tmp_path / ".aliases.json").write_text(json.dumps({"autodromo": "monza"}))
    return tmp_path


def _data(track_id):
    wps = [
        {
            "x": 1.04,
            "y": 2.0,
            "width": 12.0,
            "left_curb": True,
            "right_curb": False,
            "wall_dist": 3.25,
        }
    ]
    return {
        "id": track_id,
        "name": "New GT",
        "description": "d",
        "tag": "T",
        "default_laps": 4,
        "barrier": "BarrierType::Concrete",
        "barrier_offset": 2.5,
        "waypoints": wps,
    }


def test_write_source_json_updates_only_waypoints_of_existing_circuit(tmp_path):
    tracks = _mini_tracks(tmp_path)
    path, cmd = imp.write_source_json("gt", _data("autodromo"), str(tracks))
    assert path == str(tracks / "gt" / "monza.json")
    track = json.loads((tracks / "gt" / "monza.json").read_text())
    assert (
        track["name"] == "Monza Autodromo Nazionale"
        and track["tag"] == "TEMPLE OF SPEED"
    )
    assert track["geometry"]["trees"] == [{"x": 1}]
    assert track["spline"]["waypoints"] == [
        {
            "point": [1.0, 2.0],
            "width": 12.0,
            "left_curb": True,
            "right_curb": False,
            "surface": "Asphalt",
            "elevation": 0.0,
            "left_wall_distance": 3.2,
            "right_wall_distance": 3.2,
        }
    ]
    assert cmd.endswith("--rebuild") and "track_bake" in cmd


def test_write_source_json_creates_new_circuit_and_lists_it(tmp_path):
    tracks = _mini_tracks(tmp_path)
    _path, cmd = imp.write_source_json("gt", _data("imola"), str(tracks))
    track = json.loads((tracks / "gt" / "imola.json").read_text())
    assert (
        track["name"] == "New GT"
        and track["module_id"] == "gt"
        and track["scale"] == "0.5x"
    )
    assert track["checkpoints"] == [] and track["spline"]["samples"] == []
    assert json.loads((tracks / ".track_order.json").read_text()) == {
        "gt": ["monza", "imola"]
    }
    assert cmd.endswith("--barrier-offset 2.5 --barrier-type Concrete")


def test_provenance_urls_come_from_tracks_json(tmp_path):
    tracks = _mini_tracks(tmp_path)
    assert imp.provenance_osm_urls(str(tracks), str(tmp_path / "osm")) == {
        "monza": "https://www.openstreetmap.org/way/1"
    }
    (tmp_path / "osm").mkdir()
    (tmp_path / "osm" / "autodromo.osm").write_text("")
    assert imp.provenance_osm_urls(str(tracks), str(tmp_path / "osm")) == {
        "autodromo": "https://www.openstreetmap.org/way/1"
    }
