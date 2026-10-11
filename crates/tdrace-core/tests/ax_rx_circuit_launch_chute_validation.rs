//! Spec 103 (`specs/103_autocross_and_rallycross_launch_chutes_and_templated_track_components.md`): every official
//! Autocross and Rallycross circuit has a valid launch chute.

use tdrace_core::catalog;
use tdrace_core::track::network::JunctionKind;
use tdrace_core::track::{validate_track, PackedGridPattern, Track, ValidationSeverity};
use tdrace_core::{CarCategory, SurfaceType};

/// The 17 Autocross circuits, the 20 Rallycross circuits of the Rallycross module, and the 3 Classic Autocross and
/// 3 Classic Rallycross ones.
fn launch_circuits() -> Vec<(String, Track)> {
    let load = |module: &'static str, id: &str| {
        let c = catalog::find(id, Some(module)).unwrap_or_else(|| panic!("no circuit {module}/{id}"));
        (format!("{module}/{id}"), c.load().unwrap_or_else(|e| panic!("{module}/{id}: {e}")))
    };
    let mut circuits: Vec<(String, Track)> = catalog::module_circuits("autocross")
        .chain(catalog::module_circuits("rally"))
        .map(|c| load(c.module, c.id))
        .collect();
    circuits.extend(
        ["ax_clay_bowl", "ax_hillside_hammer", "ax_meadow_sprint", "rx_canyon_flyer", "rx_hilltop_leap", "rx_quarry_sprint"]
            .map(|id| load("classic", id)),
    );
    circuits
}

/// Scenario: Global AX and RX Circuit Validation
///
/// Given the 20 official Autocross circuits and 23 official Rallycross circuits
/// When `validate_track()` is executed on every circuit definition
/// Then all 43 circuits contain a valid `LaunchChuteConfig`
/// And zero boundary wall gaps or invalid spawn poses are detected
/// And an Autocross chute holds 8 cars, a Rallycross chute 10
#[test]
fn test_all_43_circuits_have_a_valid_launch_chute() {
    let circuits = launch_circuits();
    assert_eq!(circuits.len(), 43);
    assert_eq!(circuits.iter().filter(|(_, t)| t.car_category == CarCategory::Autocross && t.module_id.as_deref() == Some("autocross")).count(), 17);
    assert_eq!(circuits.iter().filter(|(_, t)| t.car_category == CarCategory::Autocross).count(), 20);
    assert_eq!(circuits.iter().filter(|(_, t)| t.car_category == CarCategory::Rally).count(), 23);

    let mut problems = Vec::new();
    for (name, track) in &circuits {
        let Some(chute) = track.launch_chute() else {
            problems.push(format!("{name}: no LaunchChuteConfig"));
            continue;
        };
        let errors: Vec<String> = validate_track(track)
            .into_iter()
            .filter(|d| d.severity == ValidationSeverity::Error)
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect();
        if !errors.is_empty() {
            problems.push(format!("{name}: {errors:?}"));
        }
        let cars = if track.car_category == CarCategory::Rally { 10 } else { 8 };
        if chute.grid_slots.len() != cars || track.grid_positions != chute.grid_slots {
            problems.push(format!("{name}: the grid is not the chute's {cars} slots"));
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
}

/// The chute of each circuit has the category's template: Autocross a concrete 5-3 pad, Rallycross an asphalt 3-2-3-2 pad.
#[test]
fn test_each_category_has_its_pad_and_grid() {
    for (name, track) in launch_circuits() {
        let chute = track.launch_chute().unwrap_or_else(|| panic!("{name}: no chute"));
        let (surface, pattern) = if track.car_category == CarCategory::Rally {
            (SurfaceType::Asphalt, PackedGridPattern::RallycrossThreeTwoThreeTwo)
        } else {
            (SurfaceType::Concrete, PackedGridPattern::AutocrossFiveThree)
        };
        assert_eq!(chute.surface, surface, "{name}");
        assert_eq!(PackedGridPattern::from_slots(&chute.grid_slots), Some(pattern), "{name}");
        assert!((14.0..=18.0).contains(&chute.pad_width), "{name}: pad {} m", chute.pad_width);
    }
}

/// The chute is the run-once entry of every layout and merges tangent to the circuit (the Merge junction).
#[test]
fn test_every_chute_is_an_entry_segment_merging_tangent_to_the_loop() {
    for (name, track) in launch_circuits() {
        let net = track.network.as_ref().unwrap_or_else(|| panic!("{name}: no network"));
        let chute = net.launch_chute.as_ref().unwrap();
        assert!(net.validate().is_ok(), "{name}: {:?}", net.validate());
        for layout in &net.layouts {
            assert_eq!(layout.entry_segment, Some(chute.segment_id), "{name}: layout {}", layout.id);
            assert!(!layout.segment_sequence.contains(&chute.segment_id), "{name}: layout {}", layout.id);
            assert!(net.entry_continuation_segment(layout).is_some(), "{name}: layout {}", layout.id);
            assert!(net.build_entry_spline_for_layout(&layout.id).is_some(), "{name}: layout {}", layout.id);
        }
        let JunctionKind::Merge { ingress_sockets, egress_socket, .. } = &net.get_junction(chute.merge_junction_id).unwrap().kind else {
            panic!("{name}: the chute does not end in a Merge junction");
        };
        assert_eq!(ingress_sockets.len(), 2, "{name}: the loop and the chute feed the merge");
        let end = net.get_segment(chute.segment_id).unwrap().samples.last().unwrap();
        assert!(end.point.distance(egress_socket.point) < 0.5, "{name}");
        assert!(end.tangent.dot(egress_socket.tangent) > 0.99, "{name}: tangent");
    }
}

/// The launch lives in the JSON: a circuit read back from the catalog and written out again keeps its chute.
#[test]
fn test_chute_circuits_round_trip() {
    for (name, track) in launch_circuits() {
        let back = Track::from_json(&track.to_json().unwrap()).unwrap();
        assert_eq!(back.launch_chute(), track.launch_chute(), "{name}");
        assert_eq!(back.grid_positions, track.grid_positions, "{name}");
        assert_eq!(back.geometry.network_walls.len(), track.geometry.network_walls.len(), "{name}");
    }
}
