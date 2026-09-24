use tdrace_app::game::RaceSession;
use tdrace_app::ui::menu::{CarChoice, TrackChoice};
use tdrace_core::physics::surface::SurfaceType;

#[test]
fn test_all_six_track_choices_selectable_and_initializable() {
    assert_eq!(TrackChoice::ALL.len(), 7);

    for choice in &TrackChoice::ALL {
        let mut session = RaceSession::new();
        session.track_choice = choice.clone();
        session.num_bots = 3;
        session.init_race();

        assert_eq!(session.cars.len(), 4);
        assert_eq!(session.trackers.len(), 4);
        assert!(!session.track.name.is_empty());
        assert!(!session.track.checkpoints.is_empty());
        assert_eq!(session.track_choice_id(), choice.track_id());
    }
}

#[test]
fn test_ramp_raceway_session_features() {
    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::RampRaceway;
    session.init_race();

    assert_eq!(session.track.name, "Ramp Raceway");
    assert!(!session.track.geometry.jump_ramps.is_empty());
}

#[test]
fn test_oasis_rally_session_features() {
    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::OasisRally;
    session.car_choice = CarChoice::RallyCar;
    session.init_race();

    assert_eq!(session.track.name, "Oasis Rally");
    assert_eq!(session.track.default_surface, SurfaceType::DeepSand);
    assert!(session.track.geometry.obstacles.is_empty());

    // Check that spline sample 0 has Dirt surface
    assert_eq!(session.track.spline.samples[0].surface, SurfaceType::Dirt);

    // Verify pure dirt circuit: NO red-white curbs anywhere on the track
    let has_any_curbs = session.track.spline.samples.iter().any(|s| s.left_curb || s.right_curb);
    assert!(!has_any_curbs, "Oasis Rally must not have red-white curbs");

    // Verify Oasis water hazard is present
    let has_water = session.track.geometry.surface_zones.iter().any(|z| z.surface == SurfaceType::Water);
    assert!(has_water, "Oasis Rally must feature Oasis water hazard zones");
}
