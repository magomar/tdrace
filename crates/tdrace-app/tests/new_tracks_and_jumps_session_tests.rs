use tdrace_app::game::RaceSession;
use tdrace_app::ui::menu::{CarChoice, TrackChoice};
use tdrace_core::physics::surface::SurfaceType;

#[test]
fn test_all_seven_track_choices_selectable_and_initializable() {
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
        assert_eq!(session.track_choice_id(), match choice {
            TrackChoice::ClassicGrandPrix => "classic_grand_prix",
            TrackChoice::OvalSpeedway => "oval_speedway",
            TrackChoice::DriftPark => "drift_park",
            TrackChoice::KartArena => "kart_arena",
            TrackChoice::RampRaceway => "ramp_raceway",
            TrackChoice::OasisRally => "oasis_rally",
            TrackChoice::OutlawPass => "outlaw_pass",
            TrackChoice::Custom { id, .. } => id.as_str(),
        });
    }
}

#[test]
fn test_ramp_raceway_session_features() {
    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::RampRaceway;
    session.init_race();

    assert_eq!(session.track.name, "Ramp Raceway");
    assert_eq!(session.track.geometry.jump_ramps.len(), 3);
}

#[test]
fn test_oasis_rally_session_features() {
    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::OasisRally;
    session.car_choice = CarChoice::RallyCar;
    session.init_race();

    assert_eq!(session.track.name, "Oasis Rally");
    assert_eq!(session.track.default_surface, SurfaceType::Sand);
    assert!(!session.track.geometry.obstacles.is_empty());

    // Check that spline sample 0 has Dirt surface
    assert_eq!(session.track.spline.samples[0].surface, SurfaceType::Dirt);

    // Verify pure dirt circuit: NO red-white curbs anywhere on the track
    let has_any_curbs = session.track.spline.samples.iter().any(|s| s.left_curb || s.right_curb);
    assert!(!has_any_curbs, "Oasis Rally must not have red-white curbs");

    // Verify Oasis water hazard is present
    let has_water = session.track.geometry.surface_zones.iter().any(|z| z.surface == SurfaceType::Water);
    assert!(has_water, "Oasis Rally must feature Oasis water hazard zones");
}

#[test]
fn test_outlaw_pass_session_features() {
    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::OutlawPass;
    session.init_race();

    assert_eq!(session.track.name, "Outlaw Pass");
    assert!(session.track.geometry.jump_ramps.is_empty(), "No jump ramps in Outlaw Pass");
    assert_eq!(session.track.geometry.obstacles.len(), 4, "Must have 4 mountain cliff obstacles");
    assert!(
        !session.track.geometry.surface_zones.iter().any(|z| z.surface == SurfaceType::Water),
        "Outlaw Pass must have no water hazards"
    );

    // Verify narrow pass section exists on the track ribbon
    let has_narrow_pass = session.track.spline.samples.iter().any(|s| s.width <= 7.5);
    assert!(has_narrow_pass, "Outlaw Pass must feature a dedicated narrow pass section (width <= 7.5m)");
}


