//! Native visual smoke test for Spec 085: the unpainted joker split and pit entry, and the bifurcation pacenote
//! badge before them. Saves screenshots to TDRACE_UI_PREVIEW_DIR and exits.
use macroquad::prelude::*;
use tdrace_app::game::{joker_rule_for, GameState, RaceSession};
use tdrace_app::ui::menu::TrackChoice;
use tdrace_core::track::checkpoint::MultiRouteProgressTracker;
use tdrace_core::track::network::SegmentId;

fn window_conf() -> Conf {
    Conf { window_title: "Bifurcation preview".into(), window_width: 1600, window_height: 900, ..Default::default() }
}

async fn shoot(session: &mut RaceSession, dir: &str, name: &str, zoom: Option<f32>) {
    if let Some(zoom) = zoom {
        session.camera.current_zoom = zoom;
        session.camera.target_zoom = zoom;
    }
    for frame in 0..4 {
        clear_background(session.active_backdrop_color());
        session.render();
        if frame == 3 {
            get_screen_data().export_png(&format!("{}/{}.png", dir, name));
        }
        next_frame().await;
    }
}

fn start_race(path: &str) -> RaceSession {
    let mut session = RaceSession::new();
    let id = path.trim_end_matches(".json").rsplit('/').next().unwrap().to_string();
    session.track_choice = TrackChoice::Custom { id: id.clone(), title: id, description: String::new(), path: path.into() };
    session.init_race();
    session.state = GameState::Racing;
    session.world.rules.joker = joker_rule_for(&session.track);
    session
}

/// Puts the player car at `pos`, driving along `direction` at `speed`, and points the camera at it.
fn place_player(session: &mut RaceSession, pos: tdrace_core::Vec2, direction: tdrace_core::Vec2, speed: f32) {
    session.world.vehicles[0].state.position = pos;
    session.world.vehicles[0].state.angle = direction.y.atan2(direction.x);
    session.world.vehicles[0].state.velocity = direction * speed;
    session.world.vehicles[0].state.speed = speed;
    session.camera.target_pos = pos;
    session.camera.current_pos = pos;
}

/// Player on network segment `segment`, `meters` before its end.
fn place_on_segment(session: &mut RaceSession, segment: SegmentId, meters: f32, speed: f32) {
    let network = session.track.network.as_ref().unwrap();
    let seg = network.get_segment(segment).unwrap();
    let along = (seg.length - meters).max(0.0);
    let sample = seg.sample_at_distance(along);
    // The race creates the route tracker on its first step; make it here since the preview never steps.
    let mut multi = MultiRouteProgressTracker::from_network(network, None, 1);
    multi.current_segment_id = segment;
    multi.segment_progress_distance = along;
    session.world.trackers[0].multi_route = Some(multi);
    place_player(session, sample.point, sample.tangent, speed);
}

#[macroquad::main(window_conf)]
async fn main() {
    let dir = std::env::var("TDRACE_UI_PREVIEW_DIR").expect("Set TDRACE_UI_PREVIEW_DIR");

    // Joker split on a dirt circuit.
    let mut session = start_race("tracks/classic/rx_canyon_flyer.json");
    place_on_segment(&mut session, SegmentId(0), 70.0, 28.0);
    shoot(&mut session, &dir, "joker-dirt-approach", None).await;
    place_on_segment(&mut session, SegmentId(0), 25.0, 28.0);
    shoot(&mut session, &dir, "joker-dirt-split", Some(7.0)).await;

    // Joker split on a mixed asphalt and dirt circuit.
    let mut session = start_race("tracks/rally/holjes_rx.json");
    place_on_segment(&mut session, SegmentId(0), 25.0, 28.0);
    shoot(&mut session, &dir, "joker-mixed-split", Some(7.0)).await;

    // Pit lane entry.
    let mut session = start_race("tracks/gt/catalunya.json");
    let junctions = session.track.pit_lane_junctions.clone().or_else(|| session.track.compute_pit_lane_junctions()).unwrap();
    let apex = session.track.spline.project_point(junctions.p_apex);
    for (name, meters, zoom) in [("pit-entry-approach", 70.0, None), ("pit-entry-split", 25.0, Some(7.0))] {
        let s = (apex.progress_distance - meters).rem_euclid(session.track.spline.total_length());
        let sample = session.track.spline.sample_at_distance(s);
        session.world.trackers[0].progress_distance = s;
        place_player(&mut session, sample.point, sample.tangent, 28.0);
        shoot(&mut session, &dir, name, zoom).await;
    }
}
