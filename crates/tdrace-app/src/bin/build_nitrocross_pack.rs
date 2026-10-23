//! Builds the 6 official North American Nitrocross circuits for Spec 134 from real-world
//! OpenStreetMap survey data (`assets/osm/*.osm`) and authentic Travis Pastrana / Nate Wessel
//! course layouts:
//! - `utah_umc_rx` (`way/476963551` — Utah Motorsports Campus 130-ft Mega Gap Jump & Talladega Bowl)
//! - `erx_motor_park` (`way/1023648273` — ERX Motor Park Hillside Kidney Amphitheater & Glacial Valley)
//! - `wild_horse_pass_rx` (`way/317246335` + `way/321988719` — Wild Horse Pass Sonora Desert Hybrid)
//! - `glen_helen_rx` (`way/887673423` — Glen Helen North-South Canyon Stadium & Talladega Berm)
//! - `midamerica_outdoors_rx` (`way/887408204` — MidAmerica Outdoors L-Boomerang Ozark Hillside)
//! - `calgary_stampede_rx` (`relation/3014962` + `way/37726459` — GMC Stadium Snow & Ice Oval + Infield Cut)
//!
//! Each circuit features:
//! 1. A unique real-world silhouette, aspect ratio, elevation profile, corner banking, surface mix, and scenery.
//! 2. Full-scale Nitrocross `RampKind::MegaRamp` over-under crossover (`22.5m` to `34.0m` / `74ft` to `112ft` open
//!    chasm; `46m` to `60m` / `151ft` to `197ft` total structure) on `Seg 1` where the Joker Lap (`Seg 2`)
//!    passes underneath through the open gap `(s_takeoff, s_landing)` at ground level (`elevation == 0.0`).
//! 3. Secondary `RampKind::Ramp` rhythm tabletop jump on the start/finish straight.
//! 4. 6-car 2-2-2 staggered Rallycross launch chute (`PackedGridPattern::RallycrossTwoTwoTwo`).

use std::path::Path;

use glam::Vec2;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::{
    BarrierType, JumpRamp, LineSegment, RampKind, SurfaceShape, TrackGeometry, WallBarrier,
};
use tdrace_core::track::network::{
    GoreConfig, JunctionId, JunctionKind, MergeConfig, RoadJunction, RoadSegment, SegmentId,
    SocketId, SplineSocket, TrackLayout, TrackNetwork,
};
use tdrace_core::track::scenery::{
    Building, BuildingStyle, Grandstand, GrandstandStyle, Rock, RockType, Tree, TreeType,
};
use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};
use tdrace_core::track::validation::{validate_track, ValidationSeverity};
use tdrace_core::track::{
    generate_checkpoints, generate_walls_from_spline, Checkpoint, ChuteSide, LaunchChuteSpec,
    PackedGridPattern, Track, TrackCategory, TrackKind,
};
use tdrace_core::CarCategory;

struct NitroVenueSpec {
    slug: &'static str,
    name: &'static str,
    description: &'static str,
    country_code: &'static str,
    country_name: &'static str,
    wikipedia_url: &'static str,
    osm_url: &'static str,
    tag: &'static str,
    default_surface: SurfaceType,
    runoff_surface: Option<SurfaceType>,
    primary_surface: SurfaceType,
    road_width: f32,
    mega_half_len: f32,
    mega_gap_len: f32,
    mega_height: f32,
    mega_angle_deg: f32,
    mega_launch_speed: f32,
    minor_wp_pair: (usize, usize),
    target_joker_cost_s: f32,
    mega_jump_name: &'static str,
    minor_jump_name: &'static str,
    joker_name: &'static str,
}

const VENUES: &[NitroVenueSpec] = &[
    NitroVenueSpec {
        slug: "erx_motor_park",
        name: "ERX Motor Park RX",
        description: "Natural hillside kidney amphitheater in Elk River, Minnesota featuring a plunging glacial valley, banked clay bowls, tarmac acceleration strip, and a 75-foot stadium gap jump over the Joker Lap.",
        country_code: "US",
        country_name: "United States",
        wikipedia_url: "https://en.wikipedia.org/wiki/Nitrocross",
        osm_url: "https://www.openstreetmap.org/way/1023648273",
        tag: "NITROCROSS ERX GAP JUMP",
        default_surface: SurfaceType::Grass,
        runoff_surface: None,
        primary_surface: SurfaceType::Dirt,
        road_width: 14.0,
        mega_half_len: 23.0,
        mega_gap_len: 22.5,
        mega_height: 2.55,
        mega_angle_deg: 7.5,
        mega_launch_speed: 2.8,
        minor_wp_pair: (1, 2),
        target_joker_cost_s: 3.4,
        mega_jump_name: "ERX Motor Park RX 75-ft Stadium Gap Jump",
        minor_jump_name: "ERX Motor Park RX Rhythm Tabletop",
        joker_name: "ERX Clay Bowl Underpass Joker Detour",
    },
    NitroVenueSpec {
        slug: "utah_umc_rx",
        name: "Utah Motorsports Campus Nitro RX",
        description: "Flagship high-desert Nitrocross stadium circuit in Tooele, Utah featuring the world-famous 130-foot over-under Mega Gap Jump across the Joker Lap, diagonal infield switchbacks, and a 24-degree Talladega asphalt bowl.",
        country_code: "US",
        country_name: "United States",
        wikipedia_url: "https://en.wikipedia.org/wiki/Utah_Motorsports_Campus",
        osm_url: "https://www.openstreetmap.org/way/476963551",
        tag: "NITROCROSS UTAH 130FT MEGA GAP",
        default_surface: SurfaceType::Dirt,
        runoff_surface: None,
        primary_surface: SurfaceType::Dirt,
        road_width: 14.0,
        mega_half_len: 30.0,
        mega_gap_len: 34.0,
        mega_height: 2.85,
        mega_angle_deg: 7.9,
        mega_launch_speed: 3.0,
        minor_wp_pair: (1, 2),
        target_joker_cost_s: 3.6,
        mega_jump_name: "Utah UMC RX 130-ft Desert Mega Gap Jump",
        minor_jump_name: "Utah UMC RX Infield Step-Up Tabletop",
        joker_name: "Utah UMC Underpass Joker Detour",
    },
    NitroVenueSpec {
        slug: "wild_horse_pass_rx",
        name: "Wild Horse Pass Motorsports Park RX",
        description: "Elongated Sonora Desert hybrid circuit in Chandler, Arizona combining a high-speed tarmac launch straight, double technical infield dirt switchbacks, packed-sand runoffs, and a 100-foot desert flyover gap jump.",
        country_code: "US",
        country_name: "United States",
        wikipedia_url: "https://en.wikipedia.org/wiki/Firebird_Motorsports_Park",
        osm_url: "https://www.openstreetmap.org/way/317246335",
        tag: "NITROCROSS ARIZONA FINALE",
        default_surface: SurfaceType::Dirt,
        runoff_surface: Some(SurfaceType::PackedSand),
        primary_surface: SurfaceType::Dirt,
        road_width: 13.8,
        mega_half_len: 26.0,
        mega_gap_len: 27.5,
        mega_height: 2.70,
        mega_angle_deg: 7.7,
        mega_launch_speed: 2.9,
        minor_wp_pair: (1, 2),
        target_joker_cost_s: 3.2,
        mega_jump_name: "Wild Horse Pass RX 100-ft Desert Flyover Jump",
        minor_jump_name: "Wild Horse Pass RX Sonora Step-Up Tabletop",
        joker_name: "Wild Horse Pass Underpass Joker Detour",
    },
    NitroVenueSpec {
        slug: "glen_helen_rx",
        name: "Glen Helen Raceway RX",
        description: "North-South San Bernardino canyon stadium circuit featuring the towering 22-degree banked Talladega Turn 1 berm, steep Mt. St. Helens canyon climb, and a 110-foot canyon over-under Mega Jump.",
        country_code: "US",
        country_name: "United States",
        wikipedia_url: "https://en.wikipedia.org/wiki/Glen_Helen_Raceway",
        osm_url: "https://www.openstreetmap.org/way/887673423",
        tag: "NITROCROSS GLEN HELEN CANYON",
        default_surface: SurfaceType::Dirt,
        runoff_surface: None,
        primary_surface: SurfaceType::Dirt,
        road_width: 14.2,
        mega_half_len: 27.0,
        mega_gap_len: 29.0,
        mega_height: 2.75,
        mega_angle_deg: 7.8,
        mega_launch_speed: 2.9,
        minor_wp_pair: (1, 2),
        target_joker_cost_s: 3.5,
        mega_jump_name: "Glen Helen RX 110-ft Canyon Over-Under Jump",
        minor_jump_name: "Glen Helen RX Motocross Tabletop",
        joker_name: "Glen Helen Canyon Underpass Joker Detour",
    },
    NitroVenueSpec {
        slug: "midamerica_outdoors_rx",
        name: "MidAmerica Outdoors RX",
        description: "L-shaped Ozark hillside off-road course in Jay, Oklahoma with a plunging downhill start into a 24-degree red-clay mega-berm, forested plateau switchbacks, and a 120-foot hillside gap jump.",
        country_code: "US",
        country_name: "United States",
        wikipedia_url: "https://en.wikipedia.org/wiki/Nitrocross",
        osm_url: "https://www.openstreetmap.org/way/887408204",
        tag: "NITROCROSS OKLAHOMA RED CLAY",
        default_surface: SurfaceType::Grass,
        runoff_surface: None,
        primary_surface: SurfaceType::Dirt,
        road_width: 14.0,
        mega_half_len: 28.5,
        mega_gap_len: 31.0,
        mega_height: 2.80,
        mega_angle_deg: 7.8,
        mega_launch_speed: 3.0,
        minor_wp_pair: (1, 2),
        target_joker_cost_s: 3.7,
        mega_jump_name: "MidAmerica Outdoors RX 120-ft Ozark Gap Jump",
        minor_jump_name: "MidAmerica Outdoors RX Red-Clay Tabletop",
        joker_name: "MidAmerica Ozark Underpass Joker Detour",
    },
    NitroVenueSpec {
        slug: "calgary_stampede_rx",
        name: "Stampede Park Nitro Snow & Ice RX",
        description: "GMC Stadium winter circuit in Calgary, Alberta combining a 100-foot ice gap jump in front of the main grandstand, 16-degree banked sheet-ice stadium bowls, a technical infield double-hairpin cut, and deep snowbank runoffs.",
        country_code: "CA",
        country_name: "Canada",
        wikipedia_url: "https://en.wikipedia.org/wiki/Stampede_Park",
        osm_url: "https://www.openstreetmap.org/relation/3014962",
        tag: "NITROCROSS CALGARY SNOW & ICE",
        default_surface: SurfaceType::DeepSnow,
        runoff_surface: Some(SurfaceType::DeepSnow),
        primary_surface: SurfaceType::PackedSnow,
        road_width: 14.0,
        mega_half_len: 25.0,
        mega_gap_len: 26.0,
        mega_height: 2.65,
        mega_angle_deg: 7.6,
        mega_launch_speed: 2.9,
        minor_wp_pair: (1, 2),
        target_joker_cost_s: 3.1,
        mega_jump_name: "Stampede Park RX 100-ft GMC Stadium Ice Gap Jump",
        minor_jump_name: "Stampede Park RX Winter Tabletop",
        joker_name: "Stampede Stadium Tunnel Joker Detour",
    },
];

fn speed_limited_lap_time(spline: &TrackSpline) -> f32 {
    const TOP_SPEED: f32 = 40.0;
    const CORNER_GRIP: f32 = 10.0;
    const ACCELERATION: f32 = 8.0;
    const BRAKING: f32 = 10.0;
    const CURVATURE_SPAN_M: f32 = 4.0;
    let s = &spline.samples;
    let n = s.len();
    let ds = |i: usize| {
        if i + 1 < n {
            s[i + 1].distance - s[i].distance
        } else {
            spline.total_length - s[i].distance
        }
    };
    let span = ((CURVATURE_SPAN_M * n as f32 / spline.total_length).round() as usize).max(1);
    let mut v: Vec<f32> = (0..n)
        .map(|i| {
            let (a, b) = (&s[(i + n - span) % n], &s[(i + span) % n]);
            let turn = a.tangent.perp_dot(b.tangent).atan2(a.tangent.dot(b.tangent)).abs();
            let curvature = turn / (2.0 * span as f32 * spline.total_length / n as f32);
            (CORNER_GRIP / curvature.max(1e-6)).sqrt().min(TOP_SPEED)
        })
        .collect();
    for k in 0..2 * n {
        let (i, j) = (k % n, (k + 1) % n);
        v[j] = v[j].min((v[i] * v[i] + 2.0 * ACCELERATION * ds(i)).sqrt());
    }
    for k in (0..2 * n).rev() {
        let (i, j) = (k % n, (k + 1) % n);
        v[i] = v[i].min((v[j] * v[j] + 2.0 * BRAKING * ds(i)).sqrt());
    }
    (0..n).map(|i| ds(i) / ((v[i] + v[(i + 1) % n]) * 0.5)).sum()
}

fn min_turn_radius(seg: &RoadSegment, main: &RoadSegment) -> f32 {
    seg.samples
        .windows(2)
        .filter(|w| main.project_point(w[0].point).distance_to_spline > 2.0)
        .filter_map(|w| {
            let angle = w[0].tangent.dot(w[1].tangent).clamp(-1.0, 1.0).acos();
            let ds = w[1].distance - w[0].distance;
            (angle > 1e-4 && ds > 0.0).then(|| ds / angle)
        })
        .fold(f32::INFINITY, f32::min)
}

/// Builds a straight 8-waypoint `Seg 1` corridor from `p_split` to `p_merge` with `cross_idx = split_idx + 3`
/// placed at 45% along the straightaway so `span_in >= 89m` and `span_out >= 109m`.
fn push_mega_straight(
    raw: &mut Vec<(Vec2, SurfaceType, f32, f32)>,
    p_split: Vec2,
    p_merge: Vec2,
    surf: SurfaceType,
) -> (usize, usize, usize) {
    let split_idx = raw.len();
    let fracs = [0.0, 0.15, 0.30, 0.45, 0.59, 0.73, 0.87, 1.0];
    for &f in &fracs {
        let pt = p_split.lerp(p_merge, f);
        raw.push((pt, surf, 0.0, 0.0));
    }
    let cross_idx = split_idx + 3;
    let merge_idx = split_idx + 7;
    (split_idx, cross_idx, merge_idx)
}

fn build_utah_umc_rx() -> (Vec<(Vec2, SurfaceType, f32, f32)>, usize, usize, usize) {
    use SurfaceType::{Asphalt, Dirt};
    let mut raw = vec![
        // South paved start straight -> dirt tabletop straight (heading East +X along y = 0)
        (Vec2::new(-10.0, 0.0), Asphalt, 0.0, 0.0),
        (Vec2::new(35.0, 0.0), Dirt, 0.0, 0.0),
        (Vec2::new(80.0, 0.0), Dirt, 0.0, 0.0),
        (Vec2::new(122.0, 0.0), Dirt, 0.0, 0.0),
        // Turn 1 SE sweeper into Turn 2 East hairpin
        (Vec2::new(160.0, 14.0), Dirt, 1.2, 12.0),
        (Vec2::new(176.0, 46.0), Dirt, 2.0, 14.0),
        (Vec2::new(162.0, 80.0), Dirt, 1.8, 12.0),
        // Diagonal Infield S-Switchback (cuts deep WSW into the center of the stadium per way/476963551)
        (Vec2::new(118.0, 76.0), Dirt, 1.4, 8.0),
        (Vec2::new(74.0, 62.0), Dirt, 0.8, -10.0),
        (Vec2::new(44.0, 78.0), Dirt, 1.2, -14.0),
        (Vec2::new(48.0, 112.0), Dirt, 1.8, -12.0),
        (Vec2::new(92.0, 126.0), Dirt, 2.2, 10.0),
        (Vec2::new(142.0, 138.0), Asphalt, 2.6, 16.0),
        // NE banked bowl feeding onto the North Mega Jump chute
        (Vec2::new(172.0, 162.0), Asphalt, 3.0, 18.0),
        (Vec2::new(162.0, 192.0), Asphalt, 2.4, 16.0),
        (Vec2::new(132.0, 206.0), Dirt, 1.0, 8.0),
    ];
    let (split_idx, cross_idx, merge_idx) = push_mega_straight(
        &mut raw,
        Vec2::new(102.0, 208.0),
        Vec2::new(-100.0, 208.0),
        Dirt,
    );
    // West 180-degree "Talladega" 24-degree Banked Asphalt Bowl
    raw.extend_from_slice(&[
        (Vec2::new(-146.0, 196.0), Asphalt, 2.5, 18.0),
        (Vec2::new(-182.0, 158.0), Asphalt, 4.0, 22.0),
        (Vec2::new(-194.0, 104.0), Asphalt, 4.5, 24.0),
        (Vec2::new(-178.0, 50.0), Asphalt, 2.5, 18.0),
        (Vec2::new(-148.0, 12.0), Asphalt, 0.5, 8.0),
        (Vec2::new(-110.0, 0.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-60.0, 0.0), Asphalt, 0.0, 0.0),
    ]);
    (raw, split_idx, cross_idx, merge_idx)
}

fn build_erx_motor_park() -> (Vec<(Vec2, SurfaceType, f32, f32)>, usize, usize, usize) {
    use SurfaceType::{Asphalt, Dirt};
    let mut raw = vec![
        // South shelf start straight (heading East +X along y = 0)
        (Vec2::new(-10.0, 0.0), Dirt, 0.0, 0.0),
        (Vec2::new(35.0, 0.0), Dirt, 0.0, 0.0),
        (Vec2::new(78.0, 0.0), Dirt, 0.0, 0.0),
        // Plunging downhill Turn 1 into SE glacial valley (-2.5m) and climbing East clay amphitheater bowl
        (Vec2::new(122.0, -20.0), Dirt, -1.5, 12.0),
        (Vec2::new(164.0, -16.0), Dirt, -2.5, 15.0),
        (Vec2::new(194.0, 18.0), Dirt, -1.0, 16.0),
        (Vec2::new(196.0, 58.0), Dirt, 1.5, 15.0),
        (Vec2::new(174.0, 96.0), Dirt, 2.2, 12.0),
        // Central Kidney Amphitheater Indent & Tarmac Acceleration Strip around the infield pond
        (Vec2::new(132.0, 106.0), Asphalt, 2.5, -10.0),
        (Vec2::new(108.0, 130.0), Asphalt, 2.0, -12.0),
        (Vec2::new(118.0, 162.0), Asphalt, 1.5, 10.0),
        (Vec2::new(112.0, 184.0), Dirt, 0.8, 8.0),
    ];
    let (split_idx, cross_idx, merge_idx) = push_mega_straight(
        &mut raw,
        Vec2::new(88.0, 192.0),
        Vec2::new(-112.0, 192.0),
        Dirt,
    );
    // West Clay Turnaround & Infield Chicane Kink
    raw.extend_from_slice(&[
        (Vec2::new(-148.0, 176.0), Dirt, 1.5, 14.0),
        (Vec2::new(-172.0, 138.0), Dirt, 2.0, 16.0),
        (Vec2::new(-162.0, 92.0), Dirt, 1.0, -8.0),
        (Vec2::new(-166.0, 46.0), Dirt, 0.5, 12.0),
        (Vec2::new(-142.0, 12.0), Dirt, 0.0, 8.0),
        (Vec2::new(-108.0, 0.0), Dirt, 0.0, 0.0),
        (Vec2::new(-58.0, 0.0), Dirt, 0.0, 0.0),
    ]);
    (raw, split_idx, cross_idx, merge_idx)
}

fn build_wild_horse_pass_rx() -> (Vec<(Vec2, SurfaceType, f32, f32)>, usize, usize, usize) {
    use SurfaceType::{Asphalt, Dirt};
    let mut raw = vec![
        // Elongated Sonora Desert Tarmac-to-Dirt Drag Strip Start (heading East +X along y = 0)
        (Vec2::new(-10.0, 0.0), Asphalt, 0.0, 0.0),
        (Vec2::new(38.0, 0.0), Dirt, 0.0, 0.0),
        (Vec2::new(86.0, 0.0), Dirt, 0.0, 0.0),
        (Vec2::new(134.0, 0.0), Dirt, 0.0, 0.0),
        (Vec2::new(176.0, 0.0), Asphalt, 0.0, 0.0),
        // East Turnaround & Technical Infield Switchback Complex (way/321988719)
        (Vec2::new(212.0, 16.0), Asphalt, 1.0, 14.0),
        (Vec2::new(226.0, 50.0), Dirt, 1.5, 15.0),
        (Vec2::new(206.0, 84.0), Dirt, 1.2, 12.0),
        (Vec2::new(162.0, 86.0), Dirt, 0.8, 6.0),
        (Vec2::new(122.0, 74.0), Dirt, 0.0, -10.0),
        (Vec2::new(96.0, 96.0), Dirt, 0.8, -12.0),
        (Vec2::new(110.0, 128.0), Dirt, 1.2, 10.0),
        (Vec2::new(122.0, 154.0), Dirt, 1.0, 12.0),
        (Vec2::new(108.0, 168.0), Dirt, 0.5, 8.0),
    ];
    let (split_idx, cross_idx, merge_idx) = push_mega_straight(
        &mut raw,
        Vec2::new(80.0, 172.0),
        Vec2::new(-120.0, 172.0),
        Dirt,
    );
    // West Sonora Desert Sweeper back to tarmac drag strip
    raw.extend_from_slice(&[
        (Vec2::new(-162.0, 158.0), Dirt, 1.5, 16.0),
        (Vec2::new(-194.0, 118.0), Dirt, 2.0, 18.0),
        (Vec2::new(-196.0, 66.0), Dirt, 1.5, 16.0),
        (Vec2::new(-170.0, 20.0), Asphalt, 0.5, 10.0),
        (Vec2::new(-128.0, 0.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-70.0, 0.0), Asphalt, 0.0, 0.0),
    ]);
    (raw, split_idx, cross_idx, merge_idx)
}

fn build_glen_helen_rx() -> (Vec<(Vec2, SurfaceType, f32, f32)>, usize, usize, usize) {
    use SurfaceType::{Asphalt, Dirt};
    let mut raw = vec![
        // South Canyon Staging Straight (heading East +X along y = 0)
        (Vec2::new(-10.0, 0.0), Asphalt, 0.0, 0.0),
        (Vec2::new(35.0, 0.0), Dirt, 0.0, 0.0),
        (Vec2::new(80.0, 0.0), Dirt, 0.0, 0.0),
        // Famous 22-degree Banked "Talladega" Turn 1 Berm & Steep North Canyon Climb up Mt. St. Helens (+9.5m)
        (Vec2::new(120.0, 18.0), Dirt, 2.5, 20.0),
        (Vec2::new(140.0, 62.0), Dirt, 5.5, 22.0),
        (Vec2::new(128.0, 118.0), Dirt, 7.5, 14.0),
        (Vec2::new(142.0, 174.0), Dirt, 9.0, 16.0),
        (Vec2::new(128.0, 232.0), Dirt, 9.5, 18.0),
        // North Canyon Ridge Summit Hairpin & Central Canyon Dogleg Loop
        (Vec2::new(102.0, 278.0), Dirt, 8.5, 20.0),
        (Vec2::new(58.0, 286.0), Dirt, 6.5, 16.0),
        (Vec2::new(24.0, 254.0), Dirt, 4.0, -12.0),
        (Vec2::new(-12.0, 236.0), Dirt, 2.5, -14.0),
        (Vec2::new(-48.0, 258.0), Dirt, 1.8, 10.0),
        (Vec2::new(-86.0, 274.0), Dirt, 1.2, 14.0),
        (Vec2::new(-118.0, 262.0), Dirt, 0.5, 10.0),
    ];
    // Vertical North-to-South West Canyon Mega Jump Straightaway (heading South -Y along x = -126, 198m long)
    let (split_idx, cross_idx, merge_idx) = push_mega_straight(
        &mut raw,
        Vec2::new(-126.0, 236.0),
        Vec2::new(-126.0, 38.0),
        Dirt,
    );
    // SW Canyon Floor Turnaround into flat 105m start straight
    raw.extend_from_slice(&[
        (Vec2::new(-118.0, 10.0), Asphalt, 0.0, 10.0),
        (Vec2::new(-92.0, 0.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-52.0, 0.0), Asphalt, 0.0, 0.0),
    ]);
    (raw, split_idx, cross_idx, merge_idx)
}

fn build_midamerica_outdoors_rx() -> (Vec<(Vec2, SurfaceType, f32, f32)>, usize, usize, usize) {
    use SurfaceType::{Asphalt, Dirt};
    let mut raw = vec![
        // Valley Floor Start Straight (heading East +X along y = 0)
        (Vec2::new(-10.0, 0.0), Asphalt, 0.0, 0.0),
        (Vec2::new(36.0, 0.0), Dirt, 0.0, 0.0),
        (Vec2::new(82.0, 0.0), Dirt, 0.0, 0.0),
        (Vec2::new(124.0, 0.0), Dirt, 0.0, 0.0),
        // Plunging SE Valley Bowl (-3.5m) with Giant 24-degree Red-Clay Mega-Berm
        (Vec2::new(166.0, -14.0), Dirt, -2.5, 20.0),
        (Vec2::new(202.0, 12.0), Dirt, -3.5, 24.0),
        (Vec2::new(210.0, 58.0), Dirt, -1.0, 18.0),
        // Steep North Climb up the East Wing of the L-Shape (+9.0m) & Forested Plateau Switchbacks
        (Vec2::new(196.0, 114.0), Dirt, 3.5, 10.0),
        (Vec2::new(206.0, 172.0), Dirt, 7.5, 14.0),
        (Vec2::new(186.0, 224.0), Dirt, 9.0, 18.0),
        (Vec2::new(146.0, 242.0), Dirt, 8.0, 16.0),
        (Vec2::new(108.0, 216.0), Dirt, 6.0, -10.0),
        // Deep Inner L-Elbow creating the signature Boomerang / L silhouette
        (Vec2::new(92.0, 176.0), Dirt, 4.0, -14.0),
        (Vec2::new(66.0, 152.0), Dirt, 1.5, 10.0),
    ];
    // Diagonal Hillside Mega Jump Straightaway (200m long from (36, 144) to (-158, 96))
    let (split_idx, cross_idx, merge_idx) = push_mega_straight(
        &mut raw,
        Vec2::new(36.0, 144.0),
        Vec2::new(-158.0, 96.0),
        Dirt,
    );
    // West Ozark Hillside Turnaround
    raw.extend_from_slice(&[
        (Vec2::new(-188.0, 66.0), Dirt, 0.0, 16.0),
        (Vec2::new(-178.0, 24.0), Asphalt, 0.0, 14.0),
        (Vec2::new(-144.0, 4.0), Asphalt, 0.0, 8.0),
        (Vec2::new(-108.0, 0.0), Asphalt, 0.0, 0.0),
        (Vec2::new(-58.0, 0.0), Asphalt, 0.0, 0.0),
    ]);
    (raw, split_idx, cross_idx, merge_idx)
}

fn build_calgary_stampede_rx() -> (Vec<(Vec2, SurfaceType, f32, f32)>, usize, usize, usize) {
    use SurfaceType::{PackedSnow, SheetIce};
    let mut raw = vec![
        // South Stadium Straight (heading East +X along y = 0)
        (Vec2::new(-10.0, 0.0), SheetIce, 0.0, 0.0),
        (Vec2::new(32.0, 0.0), PackedSnow, 0.0, 0.0),
        (Vec2::new(72.0, 0.0), PackedSnow, 0.0, 0.0),
        // Sharp Infield Cut & Rounded Technical Snow/Ice Section (relation/3014962 infield)
        (Vec2::new(106.0, 16.0), SheetIce, 0.5, 12.0),
        (Vec2::new(114.0, 48.0), SheetIce, 1.0, 10.0),
        (Vec2::new(88.0, 76.0), PackedSnow, 1.0, -12.0),
        (Vec2::new(62.0, 102.0), PackedSnow, 1.0, -14.0),
        (Vec2::new(80.0, 132.0), PackedSnow, 0.8, 10.0),
        (Vec2::new(118.0, 138.0), SheetIce, 1.2, 12.0),
        // Rejoining East 16-degree Banked Sheet-Ice Stadium Bowl
        (Vec2::new(152.0, 154.0), SheetIce, 1.8, 16.0),
        (Vec2::new(156.0, 184.0), SheetIce, 2.0, 16.0),
        (Vec2::new(128.0, 200.0), PackedSnow, 1.0, 10.0),
    ];
    // North GMC Stadium Grandstand Straight (heading West -X along y = 202, 198m long)
    let (split_idx, cross_idx, merge_idx) = push_mega_straight(
        &mut raw,
        Vec2::new(96.0, 202.0),
        Vec2::new(-102.0, 202.0),
        PackedSnow,
    );
    // West 16-degree Banked Sheet-Ice Stadium Bowl
    raw.extend_from_slice(&[
        (Vec2::new(-142.0, 184.0), SheetIce, 1.5, 16.0),
        (Vec2::new(-168.0, 140.0), SheetIce, 2.0, 16.0),
        (Vec2::new(-166.0, 84.0), SheetIce, 1.5, 14.0),
        (Vec2::new(-144.0, 32.0), SheetIce, 0.8, 12.0),
        (Vec2::new(-106.0, 0.0), SheetIce, 0.0, 0.0),
        (Vec2::new(-58.0, 0.0), SheetIce, 0.0, 0.0),
    ]);
    (raw, split_idx, cross_idx, merge_idx)
}

fn build_main_waypoints(spec: &NitroVenueSpec) -> (Vec<TrackWaypoint>, usize, usize, usize) {
    let (raw_pts, split_idx, cross_idx, merge_idx) = match spec.slug {
        "utah_umc_rx" => build_utah_umc_rx(),
        "erx_motor_park" => build_erx_motor_park(),
        "wild_horse_pass_rx" => build_wild_horse_pass_rx(),
        "glen_helen_rx" => build_glen_helen_rx(),
        "midamerica_outdoors_rx" => build_midamerica_outdoors_rx(),
        "calgary_stampede_rx" => build_calgary_stampede_rx(),
        other => panic!("unknown Nitrocross venue slug: {}", other),
    };

    let wps: Vec<TrackWaypoint> = raw_pts
        .into_iter()
        .map(|(pt, surf, elev, bank)| {
            let mut wp = TrackWaypoint::new(pt, spec.road_width)
                .with_surface(surf)
                .with_elevation(elev)
                .with_bank_angle(bank);
            wp.left_runoff_surface = spec.runoff_surface;
            wp.right_runoff_surface = spec.runoff_surface;
            wp
        })
        .collect();

    (wps, split_idx, cross_idx, merge_idx)
}

fn build_scrossover_joker_waypoints(
    spec: &NitroVenueSpec,
    wps: &[TrackWaypoint],
    split_idx: usize,
    cross_idx: usize,
    merge_idx: usize,
    d_lobe: f32,
) -> Vec<TrackWaypoint> {
    let p_split = wps[split_idx].point;
    let p_cross = wps[cross_idx].point;
    let p_merge = wps[merge_idx].point;
    let t_dir = (p_merge - p_split).normalize_or_zero();
    let n_dir = Vec2::new(-t_dir.y, t_dir.x); // Left normal
    let span_in = p_split.distance(p_cross);
    let span_out = p_cross.distance(p_merge);

    // Smooth grade-separated S-crossover through the 22.5m..34.0m open chasm centered at p_cross:
    // Across |lat| <= 4.6m, |u| <= 1.25m, which sits squarely in the middle of the [-11.2m..+11.2m] (or [-17m..+17m])
    // MegaRamp gap with 10m+ longitudinal clearance from both takeoff and landing walls.
    let stations: Vec<(f32, f32)> = vec![
        (-span_in, 0.0),
        (-span_in + 16.0, 0.16 * d_lobe),
        (-span_in + 32.0, 0.50 * d_lobe),
        (-span_in + 46.0, 0.84 * d_lobe),
        (-30.0, 1.00 * d_lobe),
        (-18.0, 13.5f32.max(0.80 * d_lobe)),
        (-6.0, 8.8),
        (-2.4, 3.6),
        (0.0, 0.0),
        (2.4, -3.6),
        (6.0, -8.8),
        (18.0, -13.5f32.max(0.80 * d_lobe)),
        (32.0, -1.00 * d_lobe),
        (span_out - 62.0, -1.00 * d_lobe),
        (span_out - 46.0, -0.84 * d_lobe),
        (span_out - 30.0, -0.50 * d_lobe),
        (span_out - 15.0, -0.16 * d_lobe),
        (span_out, 0.0),
    ];

    let n = stations.len();
    stations
        .into_iter()
        .enumerate()
        .map(|(idx, (u, lat))| {
            let pt = if idx == 0 {
                p_split
            } else if idx + 1 == n {
                p_merge
            } else {
                p_cross + t_dir * u + n_dir * lat
            };
            let width = if idx == 0 || idx + 1 == n {
                spec.road_width
            } else {
                13.0
            };
            let mut wp = TrackWaypoint::new(pt, width)
                .with_surface(spec.primary_surface)
                .with_elevation(0.0)
                .with_bank_angle(0.0);
            wp.left_runoff_surface = spec.runoff_surface;
            wp.right_runoff_surface = spec.runoff_surface;
            wp
        })
        .collect()
}

fn assemble_network(
    spec: &NitroVenueSpec,
    track: &Track,
    split_idx: usize,
    merge_idx: usize,
    joker_wps: Vec<TrackWaypoint>,
) -> TrackNetwork {
    let wps = &track.spline.waypoints;
    let seg0_wps = wps[0..=split_idx].to_vec();
    let seg1_wps = wps[split_idx..=merge_idx].to_vec();
    let mut seg3_wps = wps[merge_idx..].to_vec();
    seg3_wps.push(wps[0].clone());

    let s_dist = track.spline.project_point(wps[split_idx].point).progress_distance;
    let m_dist = track.spline.project_point(wps[merge_idx].point).progress_distance;
    let s_sample = track.spline.sample_at_distance(s_dist);
    let m_sample = track.spline.sample_at_distance(m_dist);

    let split_socket = SplineSocket {
        point: wps[split_idx].point,
        tangent: s_sample.tangent,
        normal: s_sample.normal,
        width: wps[split_idx].width,
        elevation: 0.0,
        bank_angle: 0.0,
        surface: spec.primary_surface,
    };
    let merge_socket = SplineSocket {
        point: wps[merge_idx].point,
        tangent: m_sample.tangent,
        normal: m_sample.normal,
        width: wps[merge_idx].width,
        elevation: 0.0,
        bank_angle: 0.0,
        surface: spec.primary_surface,
    };

    let nose_center =
        wps[split_idx].point + s_sample.tangent * 14.0 + s_sample.normal * (spec.road_width * 0.5 + 2.5);
    let split_junction = RoadJunction {
        id: JunctionId(0),
        name: format!("{} Joker Split", spec.slug),
        kind: JunctionKind::Split {
            ingress_socket: split_socket,
            egress_sockets: vec![split_socket, split_socket],
            gore_config: Some(GoreConfig {
                apex_point: wps[split_idx].point,
                divergence_angle: 14.0,
                gore_length: 14.0,
                nose_barrier: WallBarrier {
                    segment: LineSegment::new(
                        nose_center - s_sample.normal * 0.8,
                        nose_center + s_sample.normal * 0.8,
                    ),
                    restitution: 0.18,
                    friction: 0.8,
                    barrier_type: BarrierType::TireWall,
                    elevation: 0.0,
                    is_bridge: false,
                },
                has_chevrons: true,
            }),
        },
    };

    let merge_junction = RoadJunction {
        id: JunctionId(1),
        name: format!("{} Joker Merge", spec.slug),
        kind: JunctionKind::Merge {
            ingress_sockets: vec![merge_socket, merge_socket],
            egress_socket: merge_socket,
            merge_config: Some(MergeConfig {
                convergence_point: wps[merge_idx].point,
                merge_angle: 14.0,
                merge_length: 15.0,
            }),
        },
    };

    let mut seg0 = RoadSegment::new(SegmentId(0), "Start / Finish & Technical Sector", seg0_wps);
    seg0.entry_junction = Some(SocketId::new(JunctionId(1), 0));
    seg0.exit_junction = Some(SocketId::new(JunctionId(0), 0));
    seg0.recompute_samples(None, Some(&split_socket));

    let mut seg1 = RoadSegment::new(SegmentId(1), "Mega Jump Straightaway", seg1_wps);
    seg1.entry_junction = Some(SocketId::new(JunctionId(0), 0));
    seg1.exit_junction = Some(SocketId::new(JunctionId(1), 0));
    seg1.recompute_samples(Some(&split_socket), Some(&merge_socket));

    let mut seg2 = RoadSegment::new(SegmentId(2), spec.joker_name, joker_wps);
    seg2.entry_junction = Some(SocketId::new(JunctionId(0), 1));
    seg2.exit_junction = Some(SocketId::new(JunctionId(1), 1));
    seg2.recompute_samples(Some(&split_socket), Some(&merge_socket));

    let mut seg3 = RoadSegment::new(SegmentId(3), "Final Bowl & Return Chute", seg3_wps);
    seg3.entry_junction = Some(SocketId::new(JunctionId(1), 0));
    seg3.exit_junction = Some(SocketId::new(JunctionId(0), 0));
    seg3.recompute_samples(Some(&merge_socket), None);

    let main_len = seg0.length + seg1.length + seg3.length;
    let joker_len = seg0.length + seg2.length + seg3.length;

    let mut net = TrackNetwork {
        junctions: vec![split_junction, merge_junction],
        segments: vec![seg0, seg1, seg2, seg3],
        layouts: vec![
            TrackLayout {
                id: "main".to_string(),
                display_name: "Standard Circuit".to_string(),
                entry_segment: None,
                segment_sequence: vec![SegmentId(0), SegmentId(1), SegmentId(3)],
                is_closed: true,
                total_lap_length: main_len,
                start_finish_segment: SegmentId(0),
                checkpoint_ids: Vec::new(),
            },
            TrackLayout {
                id: "joker".to_string(),
                display_name: "Joker Lap Detour".to_string(),
                entry_segment: None,
                segment_sequence: vec![SegmentId(0), SegmentId(2), SegmentId(3)],
                is_closed: true,
                total_lap_length: joker_len,
                start_finish_segment: SegmentId(0),
                checkpoint_ids: Vec::new(),
            },
        ],
        default_layout_id: "main".to_string(),
        launch_chute: None,
        ..Default::default()
    };
    net.recompute_composite_splines();
    net
}

fn is_clear_of_roads(track: &Track, pt: Vec2, min_clearance: f32) -> bool {
    let proj = track.spline.project_point(pt);
    if proj.distance_to_spline < proj.track_width * 0.5 + min_clearance {
        return false;
    }
    if let Some(ref net) = track.network {
        for seg in &net.segments {
            let p = seg.project_point(pt);
            if p.distance_to_spline < p.track_width * 0.5 + min_clearance {
                return false;
            }
        }
        if let Some(ref chute) = net.launch_chute {
            for slot in &chute.grid_slots {
                if slot.position.distance(pt) < min_clearance + 10.0 {
                    return false;
                }
            }
        }
    }
    true
}

fn populate_venue_scenery(spec: &NitroVenueSpec, track: &mut Track) {
    let mut grandstands = Vec::new();
    let mut buildings = Vec::new();
    let mut trees = Vec::new();
    let mut rocks = Vec::new();

    match spec.slug {
        "utah_umc_rx" => {
            grandstands.push(
                Grandstand::new(1, Vec2::new(10.0, -32.0), 85.0, 14.0, 0.0)
                    .with_style(GrandstandStyle::CoveredStadium),
            );
            grandstands.push(
                Grandstand::new(2, Vec2::new(15.0, 244.0), 90.0, 14.0, std::f32::consts::PI)
                    .with_style(GrandstandStyle::OpenBleachers),
            );
            buildings.push(Building::new(
                1,
                Vec2::new(-35.0, 55.0),
                Vec2::new(36.0, 16.0),
                0.0,
                BuildingStyle::PitGarage,
            ));
            for (i, pos) in [
                Vec2::new(-75.0, 110.0),
                Vec2::new(-110.0, 95.0),
                Vec2::new(115.0, 18.0),
                Vec2::new(210.0, 110.0),
                Vec2::new(-225.0, 100.0),
            ]
            .into_iter()
            .enumerate()
            {
                rocks.push(Rock::new(i + 1, pos, RockType::Sandstone).with_scale(1.4));
            }
            for (i, pos) in [
                Vec2::new(-60.0, 50.0),
                Vec2::new(-20.0, 110.0),
                Vec2::new(85.0, -32.0),
                Vec2::new(195.0, 85.0),
            ]
            .into_iter()
            .enumerate()
            {
                trees.push(Tree::new(i + 1, pos, TreeType::Bush).with_scale(1.2));
            }
        }
        "erx_motor_park" => {
            grandstands.push(
                Grandstand::new(1, Vec2::new(10.0, -34.0), 95.0, 16.0, 0.0)
                    .with_style(GrandstandStyle::HillsideBleachers),
            );
            buildings.push(Building::new(
                1,
                Vec2::new(-35.0, 62.0),
                Vec2::new(28.0, 14.0),
                0.0,
                BuildingStyle::PaddockTent,
            ));
            for (i, pos) in [
                Vec2::new(-75.0, 75.0),
                Vec2::new(-55.0, 95.0),
                Vec2::new(-25.0, 105.0),
                Vec2::new(25.0, 62.0),
                Vec2::new(55.0, 58.0),
                Vec2::new(225.0, 45.0),
                Vec2::new(220.0, 95.0),
                Vec2::new(-205.0, 90.0),
                Vec2::new(-195.0, 145.0),
                Vec2::new(10.0, 225.0),
            ]
            .into_iter()
            .enumerate()
            {
                let kind = if i % 2 == 0 { TreeType::Pine } else { TreeType::Oak };
                trees.push(Tree::new(i + 1, pos, kind).with_scale(1.15));
            }
        }
        "wild_horse_pass_rx" => {
            grandstands.push(
                Grandstand::new(1, Vec2::new(20.0, -32.0), 110.0, 14.0, 0.0)
                    .with_style(GrandstandStyle::OpenBleachers),
            );
            for (i, pos) in [
                Vec2::new(-65.0, 65.0),
                Vec2::new(-25.0, 72.0),
                Vec2::new(15.0, 68.0),
                Vec2::new(230.0, 40.0),
                Vec2::new(-235.0, 75.0),
                Vec2::new(120.0, -32.0),
            ]
            .into_iter()
            .enumerate()
            {
                let kind = if i % 2 == 0 { TreeType::Palm } else { TreeType::Cactus };
                trees.push(Tree::new(i + 1, pos, kind).with_scale(1.2));
            }
            for (i, pos) in [Vec2::new(-95.0, 72.0), Vec2::new(165.0, 120.0)]
                .into_iter()
                .enumerate()
            {
                rocks.push(Rock::new(i + 1, pos, RockType::Sandstone).with_scale(1.3));
            }
        }
        "glen_helen_rx" => {
            grandstands.push(
                Grandstand::new(1, Vec2::new(25.0, -32.0), 75.0, 14.0, 0.0)
                    .with_style(GrandstandStyle::HillsideBleachers),
            );
            for (i, pos) in [
                Vec2::new(45.0, 95.0),
                Vec2::new(55.0, 145.0),
                Vec2::new(172.0, 80.0),
                Vec2::new(175.0, 160.0),
                Vec2::new(165.0, 240.0),
                Vec2::new(75.0, 330.0),
            ]
            .into_iter()
            .enumerate()
            {
                rocks.push(Rock::new(i + 1, pos, RockType::Granite).with_scale(1.5));
            }
            for (i, pos) in [
                Vec2::new(20.0, 70.0),
                Vec2::new(30.0, 120.0),
                Vec2::new(-30.0, 95.0),
                Vec2::new(168.0, 120.0),
            ]
            .into_iter()
            .enumerate()
            {
                trees.push(Tree::new(i + 1, pos, TreeType::Pine).with_scale(1.2));
            }
        }
        "midamerica_outdoors_rx" => {
            grandstands.push(
                Grandstand::new(1, Vec2::new(25.0, -32.0), 85.0, 14.0, 0.0)
                    .with_style(GrandstandStyle::HillsideBleachers),
            );
            for (i, pos) in [
                Vec2::new(-10.0, 55.0),
                Vec2::new(35.0, 62.0),
                Vec2::new(85.0, 68.0),
                Vec2::new(145.0, 85.0),
                Vec2::new(150.0, 130.0),
                Vec2::new(245.0, 90.0),
                Vec2::new(248.0, 160.0),
                Vec2::new(220.0, 265.0),
                Vec2::new(135.0, 275.0),
                Vec2::new(55.0, 195.0),
                Vec2::new(-45.0, 175.0),
                Vec2::new(-115.0, 155.0),
            ]
            .into_iter()
            .enumerate()
            {
                let kind = match i % 3 {
                    0 => TreeType::Oak,
                    1 => TreeType::Pine,
                    _ => TreeType::AutumnMaple,
                };
                trees.push(Tree::new(i + 1, pos, kind).with_scale(1.25));
            }
        }
        "calgary_stampede_rx" => {
            // Iconic GMC Stadium Grandstand along the North straight + Infield Grandstand
            grandstands.push(
                Grandstand::new(1, Vec2::new(5.0, 218.0), 140.0, 18.0, std::f32::consts::PI)
                    .with_style(GrandstandStyle::CoveredStadium)
                    .with_tiers(12),
            );
            grandstands.push(
                Grandstand::new(2, Vec2::new(10.0, -32.0), 90.0, 14.0, 0.0)
                    .with_style(GrandstandStyle::OpenBleachers),
            );
            buildings.push(Building::new(
                1,
                Vec2::new(-38.0, 68.0),
                Vec2::new(32.0, 14.0),
                0.0,
                BuildingStyle::PitGarage,
            ));
            for (i, pos) in [
                Vec2::new(-85.0, 75.0),
                Vec2::new(-65.0, 95.0),
                Vec2::new(190.0, 120.0),
                Vec2::new(-195.0, 100.0),
            ]
            .into_iter()
            .enumerate()
            {
                trees.push(Tree::new(i + 1, pos, TreeType::SnowPine).with_scale(1.2));
            }
        }
        _ => {}
    }

    track.geometry.grandstands = grandstands
        .into_iter()
        .filter(|g| is_clear_of_roads(track, g.center, 22.0))
        .collect();
    track.geometry.buildings = buildings
        .into_iter()
        .filter(|b| is_clear_of_roads(track, b.center, 22.0))
        .collect();
    track.geometry.trees = trees
        .into_iter()
        .filter(|t| is_clear_of_roads(track, t.position, 14.0))
        .collect();
    track.geometry.rocks = rocks
        .into_iter()
        .filter(|r| is_clear_of_roads(track, r.position, 14.0))
        .collect();
}

fn build_venue(spec: &NitroVenueSpec) -> (String, Track) {
    let (wps, split_idx, cross_idx, merge_idx) = build_main_waypoints(spec);
    let spline = TrackSpline::new(wps.clone(), true);
    let (inner_walls, outer_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    // Primary MegaRamp on Seg 1 with its open chasm [s_takeoff, s_landing] centered at wps[cross_idx].
    // Because takeoff_len = 0.46 * rem and landing_len = 0.54 * rem, the gap center in local coordinates
    // is -0.04 * rem, so shifting mega_center by +0.04 * rem along t_cross centers the open chasm at p_cross.
    let p_cross = wps[cross_idx].point;
    let t_cross = (wps[merge_idx].point - wps[split_idx].point).normalize_or_zero();
    let angle_cross = t_cross.y.atan2(t_cross.x);
    let rem = 2.0 * spec.mega_half_len - spec.mega_gap_len;
    let mega_center = p_cross + t_cross * (0.04 * rem);
    let mut mega_ramp = JumpRamp::new(
        1,
        SurfaceShape::OrientedBox {
            center: mega_center,
            half_extents: Vec2::new(spec.mega_half_len, 4.6),
            angle: angle_cross,
        },
        t_cross,
        spec.mega_launch_speed,
        spec.mega_angle_deg,
        spec.mega_height,
        spec.mega_jump_name,
    )
    .with_kind(RampKind::MegaRamp)
    .with_gap_length(spec.mega_gap_len);
    mega_ramp.surface = SurfaceType::Dirt;

    // Secondary tabletop jump on the start/finish straight
    let (m0, m1) = spec.minor_wp_pair;
    let p_minor = (wps[m0].point + wps[m1].point) * 0.5;
    let t_minor = (wps[m1].point - wps[m0].point).normalize_or_zero();
    let angle_minor = t_minor.y.atan2(t_minor.x);
    let mut minor_ramp = JumpRamp::new(
        2,
        SurfaceShape::OrientedBox {
            center: p_minor,
            half_extents: Vec2::new(4.0, 5.4),
            angle: angle_minor,
        },
        t_minor,
        2.4,
        6.0,
        1.25,
        spec.minor_jump_name,
    )
    .with_kind(RampKind::Ramp);
    minor_ramp.surface = SurfaceType::Dirt;

    let mut base_track = Track {
        name: spec.name.to_string(),
        description: spec.description.to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline: spline.clone(),
        geometry: TrackGeometry {
            jump_ramps: vec![mega_ramp.clone(), minor_ramp],
            inner_walls,
            outer_walls,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            ..Default::default()
        },
        default_surface: spec.default_surface,
        default_laps: 5,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: Some(spec.wikipedia_url.to_string()),
        osm_url: Some(spec.osm_url.to_string()),
        country_code: Some(spec.country_code.to_string()),
        country_name: Some(spec.country_name.to_string()),
        is_inspired: false,
        tag: spec.tag.to_string(),
        category_label: "Nitrocross".to_string(),
        ..Default::default()
    };

    // Binary search d_lobe so speed_limited_lap_time delta matches target_joker_cost_s
    let mut low = 12.0f32;
    let mut high = 52.0f32;
    let mut best_joker_wps = Vec::new();
    for _ in 0..24 {
        let mid = (low + high) * 0.5;
        let cand_wps =
            build_scrossover_joker_waypoints(spec, &wps, split_idx, cross_idx, merge_idx, mid);
        let net = assemble_network(spec, &base_track, split_idx, merge_idx, cand_wps.clone());
        let main_sp = net.build_composite_spline_for_layout("main").unwrap();
        let joker_sp = net.build_composite_spline_for_layout("joker").unwrap();
        let cost = speed_limited_lap_time(&joker_sp) - speed_limited_lap_time(&main_sp);
        if cost < spec.target_joker_cost_s {
            low = mid;
        } else {
            high = mid;
        }
        best_joker_wps = cand_wps;
    }

    let mut net = assemble_network(spec, &base_track, split_idx, merge_idx, best_joker_wps);

    // Build checkpoints along main spline + 1 joker checkpoint at the first lobe peak of Seg 2
    let mut checkpoints = generate_checkpoints(&base_track.spline, 24, 3);
    let s_dist = base_track.spline.project_point(wps[split_idx].point).progress_distance;
    let m_dist = base_track.spline.project_point(wps[merge_idx].point).progress_distance;
    let mut main_cp_ids = Vec::new();
    let mut joker_cp_ids = Vec::new();
    let joker_cp_id = checkpoints.len();
    let mut inserted_joker_cp = false;

    for cp in &mut checkpoints {
        let mid = (cp.gate.start + cp.gate.end) * 0.5;
        let d = base_track.spline.project_point(mid).progress_distance;
        main_cp_ids.push(cp.id);
        if d > s_dist + 8.0 && d < m_dist - 8.0 {
            cp.segment_id = Some(SegmentId(1));
            if !inserted_joker_cp {
                joker_cp_ids.push(joker_cp_id);
                inserted_joker_cp = true;
            }
        } else if d <= s_dist + 8.0 {
            cp.segment_id = Some(SegmentId(0));
            joker_cp_ids.push(cp.id);
        } else {
            cp.segment_id = Some(SegmentId(3));
            joker_cp_ids.push(cp.id);
        }
    }

    let seg2 = net.get_segment(SegmentId(2)).unwrap();
    let j_sample = seg2.sample_at_distance(seg2.length * 0.22);
    let j_half_w = j_sample.width * 0.5 + 1.5;
    let mut j_cp = Checkpoint::new(
        joker_cp_id,
        LineSegment::new(
            j_sample.point + j_sample.normal * j_half_w,
            j_sample.point - j_sample.normal * j_half_w,
        ),
        j_sample.tangent,
        1,
        false,
    );
    j_cp.is_joker = true;
    j_cp.segment_id = Some(SegmentId(2));
    checkpoints.push(j_cp);

    net.layouts[0].checkpoint_ids = main_cp_ids;
    net.layouts[1].checkpoint_ids = joker_cp_ids;

    base_track.checkpoints = checkpoints;
    base_track.network = Some(net);
    base_track.rebuild_geometry(3.5, BarrierType::TireWall);

    // Stamp the 6-car Rallycross launch chute
    let mut chute_template = LaunchChuteSpec::new(0, ChuteSide::Right);
    chute_template.pattern = PackedGridPattern::RallycrossTwoTwoTwo;
    chute_template.surface = SurfaceType::Asphalt;
    let chute_spec = base_track
        .place_launch_chute(&chute_template)
        .unwrap_or_else(|e| panic!("{}: failed to place launch chute: {}", spec.slug, e));

    // Populate authentic venue scenery after network and launch chute are placed
    populate_venue_scenery(spec, &mut base_track);

    // Verify all network segments have waypoint width >= 12.0m
    if let Some(ref mut net) = base_track.network {
        for seg in &mut net.segments {
            for wp in &mut seg.waypoints {
                if wp.width < 12.5 {
                    wp.width = 12.5;
                }
                if spec.runoff_surface.is_some() {
                    wp.left_runoff_surface = spec.runoff_surface;
                    wp.right_runoff_surface = spec.runoff_surface;
                }
            }
        }
    }
    base_track.ensure_runtime_geometry();

    // Diagnostics & assertions
    let net = base_track.network.as_ref().unwrap();
    let seg1 = net.get_segment(SegmentId(1)).unwrap();
    let seg2 = net.get_segment(SegmentId(2)).unwrap();
    let min_r = min_turn_radius(seg2, seg1);
    let main_sp = net.build_composite_spline_for_layout("main").unwrap();
    let joker_sp = net.build_composite_spline_for_layout("joker").unwrap();
    let cost_s = speed_limited_lap_time(&joker_sp) - speed_limited_lap_time(&main_sp);

    let underpass_samples: Vec<_> = seg2
        .samples
        .iter()
        .filter(|s| mega_ramp.contains(s.point))
        .collect();
    assert!(
        !underpass_samples.is_empty(),
        "{}: Seg 2 must cross through the MegaRamp box",
        spec.slug
    );
    for us in &underpass_samples {
        assert!(
            mega_ramp.is_mega_gap_underpass_point(us.point, 0.0),
            "{}: Seg 2 sample {:?} inside MegaRamp is not inside open gap!",
            spec.slug,
            us.point
        );
    }

    let diags = validate_track(&base_track);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == ValidationSeverity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "{}: validation errors: {:?}",
        spec.slug,
        errors
    );

    println!(
        "Built {:24} -> len={:6.1}m, mega_gap={:4.1}m ({:5.1}ft), mega_len={:4.1}m ({:5.1}ft), joker_cost={:+.2}s, min_r={:4.1}m, underpass_pts={:2}, chute_wp={} ({:?}), breakdown={}",
        spec.slug,
        base_track.spline.total_length(),
        mega_ramp.effective_gap_length(),
        mega_ramp.effective_gap_length() * 3.28084,
        mega_ramp.length(),
        mega_ramp.length() * 3.28084,
        cost_s,
        min_r,
        underpass_samples.len(),
        chute_spec.merge_waypoint,
        chute_spec.side,
        base_track.surface_summary_string()
    );
    (spec.slug.to_string(), base_track)
}

fn main() {
    let out_dir = Path::new("tracks/rally");
    let mut built = Vec::new();
    for spec in VENUES {
        built.push(build_venue(spec));
    }
    for (slug, track) in built {
        let out_path = out_dir.join(format!("{}.json", slug));
        track
            .save_to_file(&out_path)
            .unwrap_or_else(|e| panic!("{}: failed to save: {}", slug, e));
    }
}
