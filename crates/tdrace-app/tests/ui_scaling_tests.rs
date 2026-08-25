use tdrace_app::ui::scaler::UiScaler;
use tdrace_app::ui::menu::{CarChoice, TrackChoice};

#[test]
fn test_ui_scaler_reference_resolution() {
    let scaler = UiScaler::new(1280.0, 720.0);
    assert!((scaler.scale - 1.0).abs() < 1e-4);
    assert_eq!(scaler.s(100.0), 100.0);
    assert_eq!(scaler.font_s(20.0), 20.0);
    assert!(!scaler.is_mobile_aspect);
}

#[test]
fn test_ui_scaler_mobile_aspect_and_safe_padding() {
    // iPhone 14 / modern Android (e.g. 844 x 390, ~2.16 aspect ratio)
    let mobile_scaler = UiScaler::new(844.0, 390.0);
    assert!(mobile_scaler.is_mobile_aspect);
    assert!(mobile_scaler.safe_pad_x >= 16.0);
    assert!(mobile_scaler.safe_pad_y >= 14.0);

    // Font size should never shrink below legible threshold (11.0 minimum)
    assert!(mobile_scaler.font_s(8.0) >= 11.0);

    // Touch targets must satisfy minimum 44dp mobile standard
    assert!(mobile_scaler.touch_target(20.0) >= UiScaler::MIN_TOUCH_SIZE);
}

#[test]
fn test_ui_scaler_4k_and_large_displays() {
    // 4K UHD display (3840 x 2160)
    let scaler_4k = UiScaler::new(3840.0, 2160.0);
    assert!(scaler_4k.scale >= 2.0);
    assert!(scaler_4k.s(50.0) >= 100.0);
}

#[test]
fn test_car_and_track_choices_metadata() {
    for track in &TrackChoice::ALL {
        assert!(!track.title().is_empty());
        assert!(!track.tag().is_empty());
        assert!(!track.description().is_empty());
    }

    for car in &CarChoice::ALL {
        assert!(!car.title().is_empty());
        assert!(!car.tag().is_empty());
        assert!(!car.description().is_empty());

        let (spd, acc, grp, dft) = car.stats();
        assert!(spd > 0.0 && spd <= 1.0);
        assert!(acc > 0.0 && acc <= 1.0);
        assert!(grp > 0.0 && grp <= 1.0);
        assert!(dft > 0.0 && dft <= 1.0);
    }
}
