pub mod career_hub;
pub mod series_editor;
pub use series_editor as championship_editor;
pub mod curve_indicator;
pub mod driver_card;
pub mod font;
pub mod garage;
pub mod hall_of_fame;
pub mod hud;
pub mod menu;
pub mod profile_ui;
pub mod race_stats;
pub mod scaler;
pub mod settings;
pub mod starting_grid;
pub mod track_manager_ui;
pub mod track_preview;

pub use settings::{cycle_surface_texture_quality, SurfaceTextureSettings};

pub use career_hub::{
    cycle_calendar_slot, gt_default_calendar, gt_eligible_previous_tracks,
    gt_mandatory_tracks, gt_tier_class, gt_tier_title, is_slot_mandatory,
    render_career_hub_screen, track_country, track_length_meters, track_title,
};

pub use curve_indicator::{
    compute_curve_arrow_position, compute_curve_colors, compute_indicator_alpha,
    compute_smart_curve_arrow_position, render_curve_indicator, CurveColorScheme,
};
pub use driver_card::render_driver_cards_screen;
pub use font::Fonts;
pub use garage::{
    garage_car_card_rect, garage_gallery_card_rect, garage_gallery_tab_rect,
    garage_select_button_rect, gallery_filter_to_module, module_to_gallery_filter,
    render_garage_screen, GALLERY_MODULES, GarageViewMode,
};
pub use hall_of_fame::{render_hall_of_fame_screen, render_name_input_modal, PlayerCongrats};
pub use hud::{format_lap_time, render_hud, PersonalBestNotification};
pub use race_stats::render_race_stats_screen;
pub use menu::{
    modality_card_rect, module_select_badge_rect, module_select_card_rect, pause_menu_layout,
    render_controls_screen, render_modality_select_screen, render_pause_menu,
    render_results_screen, render_track_select_menu, CarChoice, GameMode, GameModeChoice,
    MenuPanelFocus, ModalityCategory, ModalityItem, ModalityModal, PauseMenuButtonLayout,
    RaceResultEntry, TrackCatalogFilter, TrackChoice,
};
pub use profile_ui::{
    championship_visible_count, get_sorted_championships, render_profile_badge,
    render_profile_create_screen, render_profile_manager_screen, ProfileFocusArea,
    TELEMETRY_CATEGORY_FILTERS,
};
pub use scaler::UiScaler;
pub use starting_grid::{
    render_starting_grid_screen, starting_grid_footer_prompt, starting_grid_footer_prompt_with_mode,
    starting_grid_garage_button_rect, starting_grid_grid_button_rect, starting_grid_launch_button_rect,
    starting_grid_player_card_rect, StartingGridFocus,
};
pub use track_manager_ui::{
    render_track_manager_screen, TrackManagerAction, TrackManagerModal, TrackManagerTab,
};
pub use series_editor::{
    handle_championship_editor_input, render_championship_editor, ChampionshipEditorAction,
    ChampionshipEditorModal, ChampionshipEditorState, ChampionshipEditorTab,
};
pub use track_preview::{compute_track_bounds, render_track_detailed_preview, render_track_thumbnail};
pub use cabinet::state::{
    confirm_modal_layout,
    ArcadeSettingsModal, CabinetContext, CabinetScreen, ScreenAction, ScreenStack,
    UniversalConfirmModal, UniversalPauseModal,
};
pub use cabinet::CabinetTheme;



