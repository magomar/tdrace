//! Compact field dropdown and segmented control (tdrace spec 086).

use cabinet::ui::{segment_at, FieldDropdown, FieldDropdownEvent, FieldDropdownInput};

const FIELD: (f32, f32, f32, f32) = (60.0, 100.0, 150.0, 22.0);
const BOUNDS: (f32, f32, f32, f32) = (0.0, 0.0, 240.0, 600.0);
const ITEM_H: f32 = 22.0;
const ROWS: usize = 8;
const COUNT: usize = 14;

fn click(x: f32, y: f32) -> FieldDropdownInput {
    FieldDropdownInput { mouse: (x, y), clicked: true, ..Default::default() }
}

fn open_at(selected: Option<usize>) -> FieldDropdown {
    let mut dd = FieldDropdown::default();
    let ev = dd.handle_input(FIELD, COUNT, selected, ROWS, BOUNDS, ITEM_H, click(100.0, 110.0));
    assert_eq!(ev, FieldDropdownEvent::Opened);
    dd
}

#[test]
fn click_on_field_opens_and_click_elsewhere_while_closed_does_nothing() {
    let mut dd = FieldDropdown::default();
    assert_eq!(dd.handle_input(FIELD, COUNT, Some(0), ROWS, BOUNDS, ITEM_H, click(10.0, 10.0)), FieldDropdownEvent::None);
    assert!(!dd.is_open);
    let dd = open_at(Some(0));
    assert!(dd.is_open);
    assert_eq!(dd.hovered, Some(0));
}

#[test]
fn list_shows_at_most_max_rows_below_the_field() {
    let dd = open_at(Some(0));
    let layout = dd.popup_layout(FIELD, COUNT, ROWS, BOUNDS, ITEM_H);
    assert_eq!(layout.visible_rows, ROWS);
    assert_eq!(layout.y, FIELD.1 + FIELD.3, "opens below the field");
    assert_eq!(layout.first_row, 0);
}

#[test]
fn list_opens_above_when_there_is_no_room_below_and_stays_inside_bounds() {
    let field = (200.0, 560.0, 150.0, 22.0);
    let mut dd = FieldDropdown::default();
    dd.handle_input(field, COUNT, Some(0), ROWS, BOUNDS, ITEM_H, click(250.0, 570.0));
    let layout = dd.popup_layout(field, COUNT, ROWS, BOUNDS, ITEM_H);
    assert_eq!(layout.y + layout.h(), field.1, "opens above the field");
    assert!(layout.x >= BOUNDS.0 && layout.x + layout.w <= BOUNDS.0 + BOUNDS.2, "kept inside bounds");
}

#[test]
fn list_is_limited_by_a_short_bounds_rectangle() {
    let dd = FieldDropdown::default();
    let layout = dd.popup_layout(FIELD, COUNT, ROWS, (0.0, 0.0, 240.0, 100.0), ITEM_H);
    assert_eq!(layout.visible_rows, 4);
    assert!(layout.y >= 0.0 && layout.y + layout.h() <= 100.0);
}

#[test]
fn opening_scrolls_the_selected_option_into_view() {
    let dd = open_at(Some(12));
    let layout = dd.popup_layout(FIELD, COUNT, ROWS, BOUNDS, ITEM_H);
    assert!(layout.first_row <= 12 && 12 < layout.first_row + layout.visible_rows);
    assert_eq!(dd.hovered, Some(12));
}

#[test]
fn mixed_selection_opens_at_the_first_option() {
    let dd = open_at(None);
    assert_eq!(dd.hovered, Some(0));
    assert_eq!(dd.first_row, 0);
}

#[test]
fn clicking_a_row_picks_it_even_when_it_is_already_selected() {
    let mut dd = open_at(Some(2));
    let layout = dd.popup_layout(FIELD, COUNT, ROWS, BOUNDS, ITEM_H);
    let row_y = layout.y + (2 - layout.first_row) as f32 * ITEM_H + 5.0;
    let ev = dd.handle_input(FIELD, COUNT, Some(2), ROWS, BOUNDS, ITEM_H, click(100.0, row_y));
    assert_eq!(ev, FieldDropdownEvent::Picked(2), "re-picking applies the value to a mixed selection");
    assert!(!dd.is_open);
}

#[test]
fn click_outside_or_escape_closes_without_picking() {
    let mut dd = open_at(Some(0));
    assert_eq!(dd.handle_input(FIELD, COUNT, Some(0), ROWS, BOUNDS, ITEM_H, click(5.0, 590.0)), FieldDropdownEvent::Closed);
    let mut dd = open_at(Some(0));
    let esc = FieldDropdownInput { cancel: true, ..Default::default() };
    assert_eq!(dd.handle_input(FIELD, COUNT, Some(0), ROWS, BOUNDS, ITEM_H, esc), FieldDropdownEvent::Closed);
    assert!(!dd.is_open);
}

#[test]
fn arrow_keys_move_the_highlight_and_scroll_and_enter_picks() {
    let mut dd = open_at(Some(0));
    let down = FieldDropdownInput { down: true, mouse: (-1.0, -1.0), ..Default::default() };
    for _ in 0..9 {
        assert_eq!(dd.handle_input(FIELD, COUNT, Some(0), ROWS, BOUNDS, ITEM_H, down), FieldDropdownEvent::None);
    }
    assert_eq!(dd.hovered, Some(9));
    assert_eq!(dd.first_row, 2, "the highlighted row stays in view");
    for _ in 0..20 {
        dd.handle_input(FIELD, COUNT, Some(0), ROWS, BOUNDS, ITEM_H, down);
    }
    assert_eq!(dd.hovered, Some(COUNT - 1), "no wrap past the last option");
    let enter = FieldDropdownInput { confirm: true, mouse: (-1.0, -1.0), ..Default::default() };
    assert_eq!(dd.handle_input(FIELD, COUNT, Some(0), ROWS, BOUNDS, ITEM_H, enter), FieldDropdownEvent::Picked(COUNT - 1));
}

#[test]
fn wheel_over_the_list_scrolls_within_limits() {
    let mut dd = open_at(Some(0));
    let layout = dd.popup_layout(FIELD, COUNT, ROWS, BOUNDS, ITEM_H);
    let over_list = (100.0, layout.y + 5.0);
    let wheel_down = FieldDropdownInput { mouse: over_list, wheel: -1.0, ..Default::default() };
    for _ in 0..20 {
        dd.handle_input(FIELD, COUNT, Some(0), ROWS, BOUNDS, ITEM_H, wheel_down);
    }
    assert_eq!(dd.first_row, COUNT - ROWS);
    let wheel_up = FieldDropdownInput { mouse: over_list, wheel: 1.0, ..Default::default() };
    dd.handle_input(FIELD, COUNT, Some(0), ROWS, BOUNDS, ITEM_H, wheel_up);
    assert_eq!(dd.first_row, COUNT - ROWS - 1);
    // The row under the pointer after scrolling is the one that gets picked.
    let ev = dd.handle_input(FIELD, COUNT, Some(0), ROWS, BOUNDS, ITEM_H, click(over_list.0, over_list.1));
    assert_eq!(ev, FieldDropdownEvent::Picked(COUNT - ROWS - 1));
}

#[test]
fn segment_at_maps_the_pointer_to_equal_segments() {
    assert_eq!(segment_at(0.0, 0.0, 150.0, 22.0, 3, (10.0, 10.0)), Some(0));
    assert_eq!(segment_at(0.0, 0.0, 150.0, 22.0, 3, (75.0, 10.0)), Some(1));
    assert_eq!(segment_at(0.0, 0.0, 150.0, 22.0, 3, (150.0, 10.0)), Some(2));
    assert_eq!(segment_at(0.0, 0.0, 150.0, 22.0, 3, (75.0, 30.0)), None);
    assert_eq!(segment_at(0.0, 0.0, 150.0, 22.0, 0, (75.0, 10.0)), None);
}
