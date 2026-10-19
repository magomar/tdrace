use macroquad::prelude::*;

use super::font::Fonts;
use super::scaler::UiScaler;
use crate::series::{DriverSubstitutionEvent, ReplacementDuration};

/// Renders a paddock bulletin modal overlay informing the player of pre-race driver substitutions.
pub fn render_substitution_modal(fonts: &Fonts, events: &[DriverSubstitutionEvent]) {
    if events.is_empty() {
        return;
    }

    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Dim backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.02, 0.03, 0.05, 0.86));

    let visible_events = &events[..events.len().min(3)];
    let card_h = scaler.s(148.0);
    let card_gap = scaler.s(14.0);
    let modal_w = scaler.s(780.0).min(sw * 0.90);
    let modal_h = (scaler.s(175.0)
        + visible_events.len() as f32 * card_h
        + visible_events.len().saturating_sub(1) as f32 * card_gap)
        .min(sh * 0.88);
    let modal_x = (sw - modal_w) * 0.5;
    let modal_y = (sh - modal_h) * 0.5;

    // Outer frame & dark panel
    draw_rectangle(
        modal_x - scaler.s(3.0),
        modal_y - scaler.s(3.0),
        modal_w + scaler.s(6.0),
        modal_h + scaler.s(6.0),
        Color::new(0.95, 0.62, 0.14, 0.88),
    );
    draw_rectangle(
        modal_x,
        modal_y,
        modal_w,
        modal_h,
        Color::new(0.06, 0.08, 0.12, 0.98),
    );

    // Header bar
    let header_h = scaler.s(62.0);
    draw_rectangle(
        modal_x,
        modal_y,
        modal_w,
        header_h,
        Color::new(0.11, 0.14, 0.21, 1.0),
    );
    draw_line(
        modal_x,
        modal_y + header_h,
        modal_x + modal_w,
        modal_y + header_h,
        scaler.s(2.0),
        Color::new(0.95, 0.62, 0.14, 0.70),
    );

    let title = if events.len() == 1 {
        "PADDOCK STEWARDS BULLETIN — DRIVER SUBSTITUTION"
    } else {
        "PADDOCK STEWARDS BULLETIN — DRIVER SUBSTITUTIONS"
    };
    fonts.draw_ui_bold(
        title,
        modal_x + scaler.s(24.0),
        modal_y + scaler.s(38.0),
        scaler.font_s(21.0),
        Color::new(1.0, 0.78, 0.24, 1.0),
    );

    // Event cards
    let mut cur_y = modal_y + header_h + scaler.s(18.0);
    let card_x = modal_x + scaler.s(24.0);
    let card_w = modal_w - scaler.s(48.0);

    for ev in visible_events {
        let is_temp = ev.duration == ReplacementDuration::OneRace;
        let accent = if is_temp {
            Color::new(0.22, 0.78, 0.96, 1.0)
        } else {
            Color::new(0.96, 0.42, 0.24, 1.0)
        };

        draw_rectangle(
            card_x,
            cur_y,
            card_w,
            card_h,
            Color::new(0.09, 0.12, 0.17, 0.98),
        );
        draw_rectangle(card_x, cur_y, scaler.s(5.0), card_h, accent);
        draw_rectangle_lines(
            card_x,
            cur_y,
            card_w,
            card_h,
            scaler.s(1.2),
            Color::new(accent.r, accent.g, accent.b, 0.45),
        );

        let badge_text = if is_temp {
            "[TEMPORARY • 1-RACE STAND-IN]"
        } else {
            "[PERMANENT SEAT CHANGE]"
        };
        fonts.draw_ui_bold(
            badge_text,
            card_x + scaler.s(16.0),
            cur_y + scaler.s(26.0),
            scaler.font_s(14.0),
            accent,
        );

        let headline_dim = fonts.measure_ui_bold(&ev.reason_headline, scaler.font_s(14.0));
        fonts.draw_ui_bold(
            &ev.reason_headline,
            card_x + card_w - headline_dim.width - scaler.s(16.0),
            cur_y + scaler.s(26.0),
            scaler.font_s(14.0),
            Color::new(0.82, 0.86, 0.92, 1.0),
        );

        let swap_line = format!(
            "{}  -->  {} (0 PTS)",
            ev.outgoing_driver_name.to_uppercase(),
            ev.incoming_driver_name.to_uppercase()
        );
        fonts.draw_ui_bold(
            &swap_line,
            card_x + scaler.s(16.0),
            cur_y + scaler.s(54.0),
            scaler.font_s(18.0),
            WHITE,
        );

        let team_car_line = format!("{}  •  {}", ev.team_name, ev.car_name);
        fonts.draw_ui_regular(
            &team_car_line,
            card_x + scaler.s(16.0),
            cur_y + scaler.s(76.0),
            scaler.font_s(14.0),
            Color::new(0.68, 0.74, 0.84, 1.0),
        );

        // Word-wrap the narrative explanation across up to 2 lines
        let max_text_w = card_w - scaler.s(32.0);
        let font_sz = scaler.font_s(14.0);
        let wrapped = wrap_modal_lines(fonts, &ev.reason_text, font_sz, max_text_w);
        for (idx, line) in wrapped.iter().take(2).enumerate() {
            fonts.draw_ui_regular(
                line,
                card_x + scaler.s(16.0),
                cur_y + scaler.s(104.0 + idx as f32 * 20.0),
                font_sz,
                Color::new(0.88, 0.90, 0.95, 1.0),
            );
        }

        cur_y += card_h + card_gap;
    }

    // Footer prompt
    let prompt = "[ENTER / SPACE / CLICK] ACKNOWLEDGE BULLETIN";
    let prompt_dim = fonts.measure_ui_bold(prompt, scaler.font_s(15.0));
    fonts.draw_ui_bold(
        prompt,
        modal_x + (modal_w - prompt_dim.width) * 0.5,
        modal_y + modal_h - scaler.s(22.0),
        scaler.font_s(15.0),
        Color::new(1.0, 0.82, 0.28, 1.0),
    );
}

fn wrap_modal_lines(fonts: &Fonts, text: &str, font_size: f32, max_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current, word)
        };
        if fonts.measure_ui_regular(&candidate, font_size).width <= max_width || current.is_empty() {
            current = candidate;
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}
