//! Builds the Joker Lap branches of the 3 Classic RX circuits.
//!
//! Spec 081: Rallycross Joker Lap Segments for Classic and OpenStreetMap Circuits.
//!
//! Spec 102: each circuit's old joker (its hand-placed waypoints below) is fitted with a branch layout of two junction
//! components (`rx_joker_fit`), which compiles into the network. A circuit that cannot be fitted keeps the legacy
//! network already in its file, untouched. The run writes `docs/circuits/branch_junction_migration.md`.
//!
//! ```text
//! cargo run --release --bin build_classic_rx_joker -- [tracks/classic]
//! cargo run --release --bin track_bake -- tracks/classic/rx_*.json --rebuild   # run twice; the second changes nothing
//! ```

#[path = "rx_joker_fit/mod.rs"]
mod rx_joker_fit;

use std::path::Path;

use glam::Vec2;
use tdrace_core::track::spline::TrackWaypoint;
use tdrace_core::track::Track;
use tdrace_core::SurfaceType;

/// One circuit: its file, the joker's name, the main waypoints where the old joker splits and merges, and the old
/// joker between them.
struct ClassicJoker {
    id: &'static str,
    name: &'static str,
    split_idx: usize,
    merge_idx: usize,
    joker: Vec<TrackWaypoint>,
}

fn wp(x: f32, y: f32, surface: SurfaceType, elevation: f32) -> TrackWaypoint {
    TrackWaypoint::new(Vec2::new(x, y), 12.0).with_surface(surface).with_elevation(elevation)
}

fn classic_jokers() -> Vec<ClassicJoker> {
    vec![
        // Split at wp 21 (-8.3, -229.3), merge at wp 27 (-53.3, -128.8).
        ClassicJoker {
            id: "rx_quarry_sprint",
            name: "Excavated Basin Joker Detour",
            split_idx: 21,
            merge_idx: 27,
            joker: vec![
                wp(-8.3, -229.3, SurfaceType::Dirt, 0.0),
                wp(-30.0, -230.4, SurfaceType::Dirt, -1.2),
                wp(-68.5, -237.4, SurfaceType::Dirt, -2.5),
                wp(-72.2, -209.7, SurfaceType::Concrete, -0.5),
                wp(-66.8, -165.0, SurfaceType::Concrete, 1.8),
                wp(-58.7, -140.0, SurfaceType::Dirt, 0.5),
                wp(-53.3, -128.8, SurfaceType::Asphalt, 0.0),
            ],
        },
        // Split at wp 28 (10.8, -132.2), merge at wp 35 (-92.0, 0.9).
        ClassicJoker {
            id: "rx_hilltop_leap",
            name: "Perimeter Ridge Joker Detour",
            split_idx: 28,
            merge_idx: 35,
            joker: vec![
                wp(10.8, -132.2, SurfaceType::Dirt, 1.5),
                wp(-13.9, -143.2, SurfaceType::Dirt, 2.5).with_bank_angle(8.0),
                wp(-52.4, -132.8, SurfaceType::Dirt, 4.5).with_bank_angle(8.0),
                wp(-92.4, -93.2, SurfaceType::Dirt, 3.5).with_bank_angle(8.0),
                wp(-112.4, -48.9, SurfaceType::Dirt, 2.2).with_bank_angle(8.0),
                wp(-105.8, -14.6, SurfaceType::Dirt, 1.5),
                wp(-92.0, 0.9, SurfaceType::Asphalt, 1.0),
            ],
        },
        // Split at wp 14 (113.1, -188.6), merge at wp 25 (-139.1, -235.1).
        ClassicJoker {
            id: "rx_canyon_flyer",
            name: "Canyon Rim Hairpin Joker Detour",
            split_idx: 14,
            merge_idx: 25,
            joker: vec![
                wp(113.1, -188.6, SurfaceType::Dirt, 0.0),
                wp(75.0, -231.8, SurfaceType::Dirt, 0.0),
                wp(30.0, -276.2, SurfaceType::Dirt, 0.0),
                wp(-20.0, -303.5, SurfaceType::Concrete, 0.0),
                wp(-70.0, -308.5, SurfaceType::Concrete, 0.0),
                wp(-110.0, -284.0, SurfaceType::Dirt, 0.0),
                wp(-139.1, -235.1, SurfaceType::Dirt, 0.0),
            ],
        },
    ]
}

fn build(joker: &ClassicJoker, tracks_dir: &Path) -> rx_joker_fit::Row {
    let path = tracks_dir.join(format!("{}.json", joker.id));
    println!("Processing {:?}...", path);
    let mut track = Track::load_from_file(&path).unwrap_or_else(|e| panic!("failed to load {}: {:?}", joker.id, e));
    track.branch_layout = None;
    let outcome = rx_joker_fit::convert(&mut track, joker.id, joker.name, joker.split_idx, joker.merge_idx, &joker.joker);
    if let rx_joker_fit::Outcome::Converted(fit) = &outcome {
        println!("  {} -> converted: deviation {:.2} m, joker costs {:.2} s", joker.id, fit.deviation, fit.cost_s);
        track.save_to_file(&path).unwrap_or_else(|e| panic!("failed to save {}: {:?}", joker.id, e));
    }
    rx_joker_fit::Row { circuit: joker.id.to_string(), outcome }
}

fn main() {
    let tracks_dir = std::env::args().nth(1).map(std::path::PathBuf::from).unwrap_or_else(|| std::path::PathBuf::from("tracks/classic"));
    println!("Building Classic RX Joker track networks in {:?}...", tracks_dir);
    let rows: Vec<rx_joker_fit::Row> = classic_jokers().iter().map(|j| build(j, &tracks_dir)).collect();
    rx_joker_fit::write_report(Path::new("docs/circuits/branch_junction_migration.md"), &rows);
}
