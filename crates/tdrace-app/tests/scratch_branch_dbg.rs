use tdrace_core::track::bake::{bake, BakeOptions};
use tdrace_core::track::branch_kit::{junction_checks, BranchLayout};
use tdrace_core::track::geometry::{BarrierType, LineSegment};
use tdrace_core::track::junction_kit::{JunctionComponent, JunctionShape, Side};
use tdrace_core::track::spline::TrackWaypoint;

fn run(module: &str, id: &str, side: Side, length: f32, gap: f32) {
    let track = tdrace_core::catalog::official_track(module, id);
    let net = track.network.clone().unwrap();
    let split_idx = net.get_segment(tdrace_core::track::network::SegmentId(0)).unwrap().waypoints.len() - 1;
    let merge_idx = track.spline.waypoints.len() + 1 - net.get_segment(tdrace_core::track::network::SegmentId(3)).unwrap().waypoints.len();
    let main = track.spline.clone();
    let arc = |i: usize| main.project_point(main.waypoints[i].point).progress_distance;
    let (s_a, s_m) = (arc(split_idx), arc(merge_idx));
    let sigma = side.sign();
    let half = main.sample_at_distance(s_a).width * 0.5;
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
    let mut t = track.clone();
    t.network = None;
    t.checkpoints.retain(|c| !c.is_joker);
    t.branch_layout = Some(layout);
    bake(&mut t, &BakeOptions { rebuild: true, ..BakeOptions::default() }).unwrap();
    let checks = junction_checks(&t).unwrap();
    let walls: Vec<_> = t.geometry.all_walls().copied().collect();
    for region in &checks.regions {
        println!("{id} region {} reach {:.1} window {:?}", region.name, region.reach, region.window);
        for (i, (edge, out)) in region.rays.iter().enumerate() {
            let ray = LineSegment::new(*edge, *edge + *out * region.reach);
            let hit = walls.iter().any(|w| w.segment.intersect_segment(&ray).is_some() || [w.segment.start, w.segment.end].iter().any(|p| ray.distance_to_point(*p) < 0.02));
            if !hit && id == "rx_canyon_flyer" {
                for (wi, w) in walls.iter().enumerate() {
                    if w.segment.distance_to_point(*edge) < 6.0 {
                        println!("      wall #{} {:?} -> {:?} ({} walls total)", wi, w.segment.start, w.segment.end, walls.len());
                    }
                }
            }
            if !hit {
                let nearest = walls.iter().map(|w| w.segment.distance_to_point(*edge)).fold(f32::MAX, f32::min);
                println!("   miss #{i} edge {:?} out {:?} nearest wall {:.2} m", edge, out, nearest);
            }
        }
    }
}

#[test]
fn mettet_canyon_misses() {
    run("rally", "mettet_rx", Side::Right, 30.0, 4.0);
    run("classic", "rx_canyon_flyer", Side::Left, 30.0, 4.0);
}
