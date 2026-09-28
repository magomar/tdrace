//! NASCAR circuit scale rule (docs/circuits/index.md): real laps under 3 km are 1:1, Road America
//! is 0.5x, every other circuit is 0.75x. The lap length must match the declared scale.

use tdrace_app::module::{nascar::NascarGameModule, GameModule};

/// Official real lap length in meters of each NASCAR preset.
const OFFICIAL_LAP_M: [(&str, f32); 17] = [
    ("daytona_superspeedway", 4023.0),
    ("talladega_superspeedway", 4281.0),
    ("watkins_glen_nascar", 3943.0),
    ("bristol_motor_speedway", 858.0),
    ("martinsville_speedway", 847.0),
    ("darlington_raceway", 2198.0),
    ("charlotte_motor_speedway", 2414.0),
    ("indianapolis_motor_speedway", 4023.0),
    ("eldora_speedway", 805.0),
    ("iowa_speedway", 1408.0),
    ("road_america", 6515.0),
    ("chicago_street_course", 3541.0),
    ("bowman_gray_stadium", 402.0),
    ("lucas_oil_irp", 1104.0),
    ("north_wilkesboro_speedway", 1006.0),
    ("pocono_raceway", 4023.0),
    ("phoenix_raceway", 1645.0),
];

#[test]
fn test_nascar_tracks_follow_the_scale_rule() {
    for def in NascarGameModule::new().tracks() {
        let official = OFFICIAL_LAP_M
            .iter()
            .find(|(id, _)| *id == def.id)
            .unwrap_or_else(|| panic!("{} has no official lap length in this test", def.id))
            .1;
        let (label, factor) = match def.id {
            "road_america" => ("0.5x", 0.5),
            _ if official < 3000.0 => ("1:1", 1.0),
            _ => ("0.75x", 0.75),
        };

        let track = tdrace_core::catalog::official_track("nascar", def.id);
        assert_eq!(track.scale(), label, "{} ({} m real) must declare {}", def.id, official, label);

        // The spline rounds the OSM corners, so the lap is a little shorter than the target
        // (Road America, the oldest import, is 3.4% short).
        let target = official * factor;
        let lap = track.spline.total_length();
        assert!(
            (lap - target).abs() / target < 0.04,
            "{} lap is {:.0} m, expected {:.0} m ({} of {} m)",
            def.id,
            lap,
            target,
            label,
            official
        );
    }
}
