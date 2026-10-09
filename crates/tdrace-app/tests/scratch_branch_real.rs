use glam::Vec2;
use tdrace_core::track::bake::{bake, BakeOptions};
use tdrace_core::track::branch_kit::BranchLayout;
use tdrace_core::track::geometry::BarrierType;
use tdrace_core::track::junction_kit::{JunctionComponent, JunctionShape, Side};
use tdrace_core::track::spline::TrackWaypoint;
use tdrace_core::track::{validate_track, ValidationSeverity};

const RX: [(&str, &str); 23] = [
    ("classic", "rx_quarry_sprint"),
    ("classic", "rx_hilltop_leap"),
    ("classic", "rx_canyon_flyer"),
    ("rally", "holjes_rx"),
    ("rally", "lydden_hill"),
    ("rally", "hell_rx"),
    ("rally", "loheac_rx"),
    ("rally", "estering_rx"),
    ("rally", "montalegre_rx"),
    ("rally", "nyirad_rx"),
    ("rally", "kouvola_rx"),
    ("rally", "catalunya_rx"),
    ("rally", "mettet_rx"),
    ("rally", "lavare_rx"),
    ("rally", "riga_rx"),
    ("rally", "killarney_rx"),
    ("rally", "lessay_rx"),
    ("rally", "essay_rx"),
    ("rally", "dreux_rx"),
    ("rally", "croft_rx"),
    ("rally", "spa_rx"),
    ("rally", "silverstone_rx"),
    ("rally", "erx_motor_park"),
];

#[test]
fn synthetic_layouts_on_real_main_splines() {
    let mut compiled = 0;
    for (module, id) in RX {
        let mut track = tdrace_core::catalog::official_track(module, id);
        let net = track.network.clone().unwrap();
        let split_idx = net.get_segment(tdrace_core::track::network::SegmentId(0)).unwrap().waypoints.len() - 1;
        let merge_idx = track.spline.waypoints.len() + 1 - net.get_segment(tdrace_core::track::network::SegmentId(3)).unwrap().waypoints.len();
        let main = track.spline.clone();
        let arc = |i: usize| main.project_point(main.waypoints[i].point).progress_distance;
        let (s_a, s_m) = (arc(split_idx), arc(merge_idx));
        let mut done = false;
        let mut last_err = String::new();
        'search: for side in [Side::Left, Side::Right] {
            for (length, gap) in [(30.0f32, 4.0f32), (20.0, 3.0), (40.0, 5.0)] {
                let sigma = side.sign();
                let half = main.sample_at_distance(s_a).width * 0.5;
                // Free road: the main line, offset sideways, from just after the split junction to just before the merge.
                let lateral = half + gap + 6.5;
                let mut road = Vec::new();
                let mut s = s_a + length + 10.0;
                while s < s_m - length - 10.0 {
                    let c = main.sample_at_distance(s);
                    let p = c.point + c.normal * sigma * lateral;
                    let mut w = TrackWaypoint::new(p, 13.0).with_surface(tdrace_core::SurfaceType::Dirt);
                    w.elevation = c.elevation;
                    road.push(w);
                    s += 25.0;
                }
                let layout = BranchLayout {
                    layout_id: "joker".into(),
                    name: "Synthetic".into(),
                    side,
                    split: JunctionComponent { s: s_a, kind: JunctionShape::Taper, length, divider_gap: gap },
                    merge: JunctionComponent { s: s_m, kind: JunctionShape::Taper, length, divider_gap: gap },
                    road_waypoints: road,
                    road_width: 13.0,
                    nose_barrier: BarrierType::TireWall,
                };
                match layout.compile(&track) {
                    Ok(_) => {}
                    Err(e) => {
                        last_err = format!("{e:?}");
                        continue;
                    }
                }
                let mut t = track.clone();
                t.network = None;
                t.branch_layout = Some(layout);
                bake(&mut t, &BakeOptions { rebuild: true, ..BakeOptions::default() }).unwrap();
                let diags = validate_track(&t);
                let errors: Vec<_> = diags.iter().filter(|d| d.severity == ValidationSeverity::Error).map(|d| d.code).collect();
                let junction: Vec<_> = diags.iter().filter(|d| d.code.contains("JUNCTION")).map(|d| format!("{}: {}", d.code, d.message)).collect();
                println!("{id}: side {side:?} L{length} gap{gap}: {} errors {:?} junction {junction:?}", errors.len(), errors.iter().take(2).collect::<Vec<_>>());
                for d in diags.iter().filter(|d| d.severity == ValidationSeverity::Error).take(3) {
                    println!("      {}: {}", d.code, d.message);
                }
                compiled += 1;
                done = true;
                track = t;
                break 'search;
            }
        }
        if !done {
            println!("{id}: no synthetic layout compiles (last: {last_err})");
        }
        let _ = Vec2::ZERO;
    }
    println!("{compiled} of 23 compiled");
}
