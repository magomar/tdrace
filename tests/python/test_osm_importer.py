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
