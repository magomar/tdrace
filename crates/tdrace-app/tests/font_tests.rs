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

#[test]
fn test_font_text_wrapping_multiline() {
    let fonts = Fonts::load_embedded();

    // 1. Empty string returns empty
    let empty_lines = fonts.wrap_text("", 12.0, 300.0);
    assert!(empty_lines.is_empty());

    // 2. Short string fits on single line
    let short_text = "Apex Tanaka";
    let short_lines = fonts.wrap_text(short_text, 12.0, 300.0);
    assert_eq!(short_lines.len(), 1);
    assert_eq!(short_lines[0], short_text);

    // 3. Long driver bio wraps across multiple lines
    let bio = "Former open-wheel champion whose surgical precision and textbook racing lines carve through chicanes like a scalpel.";
    let max_width = 250.0;
    let font_size = 11.0;
    let bio_lines = fonts.wrap_text(bio, font_size, max_width);

    assert!(bio_lines.len() >= 2, "Long bio should wrap into multiple lines, got {}", bio_lines.len());

    // Verify all words are preserved
    let joined = bio_lines.join(" ");
    assert_eq!(joined, bio);

    // Verify no line exceeds max_width (with small tolerance for single long words if any)
    for line in &bio_lines {
        let dim = fonts.measure_ui_regular(line, font_size);
        assert!(
            dim.width <= max_width + 10.0,
            "Line '{}' width {} exceeded max_width {}",
            line,
            dim.width,
            max_width
        );
    }
}
