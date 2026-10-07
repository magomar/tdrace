//! Track Studio inspector interaction state.
//! Governed by specs/086_compact_track_studio_inspector_with_unified_single_and_multiselection_editing.md.

use super::state::{EditorState, Selection};
use super::tools::ToolSettings;

/// Delay before a held step button starts repeating, in seconds.
pub const HOLD_REPEAT_DELAY: f64 = 0.4;
/// Interval between repeated steps while a step button is held, in seconds (10 Hz).
pub const HOLD_REPEAT_INTERVAL: f64 = 0.1;

/// Hold-to-repeat timer for step buttons: one step on press, then repeats after a delay.
#[derive(Debug, Clone, Default)]
pub struct HoldRepeat {
    held: Option<String>,
    next_step_at: f64,
}

impl HoldRepeat {
    /// Returns true when the button `id` must step this frame.
    /// `pressed` is the press edge, `down` the held state, `over` whether the pointer is on the button.
    pub fn poll(&mut self, id: &str, pressed: bool, down: bool, over: bool, now: f64) -> bool {
        if pressed && over {
            self.held = Some(id.to_string());
            self.next_step_at = now + HOLD_REPEAT_DELAY;
            return true;
        }
        if self.held.as_deref() != Some(id) {
            return false;
        }
        if !down {
            self.held = None;
            return false;
        }
        if over && now >= self.next_step_at {
            // Fixed schedule, so frame timing does not slow the rate; after a stall, never burst.
            self.next_step_at += HOLD_REPEAT_INTERVAL;
            if self.next_step_at <= now {
                self.next_step_at = now + HOLD_REPEAT_INTERVAL;
            }
            return true;
        }
        false
    }
}

/// Per-session interaction state of the inspector panel.
#[derive(Debug, Clone, Default)]
pub struct InspectorView {
    /// The pointer is over the inspector card this frame (and no modal is open).
    pub hovered: bool,
    /// Slider that owns the current mouse drag, from press to release.
    pub drag_capture: Option<String>,
    pub hold: HoldRepeat,
    /// Selection seen on the previous frame; a change clears control focus.
    pub last_selection: Option<Selection>,
}

/// Per-frame inspector bookkeeping, run before the inspector is drawn.
/// - One press-to-release that starts on the inspector is one undo step.
/// - A click clears control focus; the control under the click claims it again while drawing.
/// - A new selection clears focus and any inline value edit.
pub fn begin_inspector_frame(state: &mut EditorState, tools: &mut ToolSettings, hovered: bool, pressed: bool, down: bool) {
    tools.inspector.hovered = hovered;
    if !down {
        state.end_undo_gesture();
        tools.inspector.drag_capture = None;
    }
    if pressed {
        tools.clear_bar_selection();
        if hovered {
            state.begin_undo_gesture();
        }
    }
    if tools.inspector.last_selection.as_ref() != Some(&state.selection) {
        tools.clear_bar_selection();
        tools.stop_editing_bar();
        tools.inspector.last_selection = Some(state.selection.clone());
    }
}
