use tdrace_app::ui::font::{
    Fonts, FONT_DISPLAY_BYTES, FONT_UI_BOLD_BYTES, FONT_UI_MEDIUM_BYTES,
};
use tdrace_app::ui::hud::format_lap_time;

#[test]
fn test_embedded_font_bytes_integrity() {
    assert!(FONT_DISPLAY_BYTES.len() > 10000, "Display font should be embedded");
    assert!(FONT_UI_BOLD_BYTES.len() > 10000, "UI bold font should be embedded");
    assert!(FONT_UI_MEDIUM_BYTES.len() > 10000, "UI medium font should be embedded");

    // Check TTF / OpenType magic headers (0x00010000, 'OTTO', or 'true')
    let is_valid_ttf = |bytes: &[u8]| {
        bytes.starts_with(&[0x00, 0x01, 0x00, 0x00])
            || bytes.starts_with(b"OTTO")
            || bytes.starts_with(b"true")
            || bytes.starts_with(b"typ1")
    };

    assert!(is_valid_ttf(FONT_DISPLAY_BYTES), "Rajdhani must have valid TTF/OTF magic header");
    assert!(is_valid_ttf(FONT_UI_BOLD_BYTES), "Barlow bold must have valid TTF/OTF magic header");
    assert!(is_valid_ttf(FONT_UI_MEDIUM_BYTES), "Barlow medium must have valid TTF/OTF magic header");
}

#[test]
fn test_fonts_load_embedded_headless_resilience() {
    // In headless test environments, load_embedded() must not panic
    let fonts = Fonts::load_embedded();
    let sample = "01:23.45";

    // Text measurement must operate without panicking
    let _dim_display = fonts.measure_display(sample, 32.0);
    let _dim_ui = fonts.measure_ui_bold("RACE RESULTS", 20.0);
    let _dim_reg = fonts.measure_ui_regular("Subtext description", 14.0);
}

#[test]
fn test_format_lap_time_precision() {
    assert_eq!(format_lap_time(0.0), "--:--.--");
    assert_eq!(format_lap_time(-5.0), "--:--.--");
    assert_eq!(format_lap_time(f32::NAN), "--:--.--");
    assert_eq!(format_lap_time(f32::INFINITY), "--:--.--");

    assert_eq!(format_lap_time(65.42), "01:05.42");
    assert_eq!(format_lap_time(12.05), "00:12.05");
    assert_eq!(format_lap_time(184.99), "03:04.99");
}
