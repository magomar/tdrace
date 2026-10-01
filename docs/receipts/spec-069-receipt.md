---
type: validation_receipt
schema_version: "1.0"
spec: "specs/069_legacy_ui_migration_and_reusable_platform_component_adoption.md"
epic: "tdrace-3r5p"
candidate_commit: "1e783662b856d38460e7f75668d3a28b356739ed"
verifier: "local-user"
evaluated_at: "2026-10-01T19:28:15Z"
command: "python3 scripts/verify_ui_migration.py"
exit_code: 0
duration_ms: 18604
status: passed
---

# 🧾 Validation Receipt: Spec 069

- **Candidate Commit**: `1e783662b856d38460e7f75668d3a28b356739ed`
- **Spec**: `specs/069_legacy_ui_migration_and_reusable_platform_component_adoption.md`
- **Command**: `python3 scripts/verify_ui_migration.py`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 142 tests
test audio::backend::tests::test_amplitude_to_db ... ok
test audio::dsp::tests::test_adsr_envelope ... ok
test audio::dsp::tests::test_biquad_lowpass_stability ... ok
test audio::dsp::tests::test_combustion_and_cylinder_oscillators ... ok
test audio::dsp::tests::test_biquad_bandpass_filtering_and_stability ... ok
test audio::dsp::tests::test_encode_wav_16bit_mono_headers ... ok
test audio::dsp::tests::test_encode_wav_16bit_stereo_headers ... ok
test audio::dsp::tests::test_waveshape_engine ... ok
test audio::sink::tests::test_mock_audio_sink ... ok
test fx::crt::tests::test_crt_overlay_roll_animation ... ok
test fx::crt::tests::test_scanline_mode_indices_and_opacities ... ok
test fx::floating_text::tests::test_capacity_capping ... ok
test fx::floating_text::tests::test_floating_text_spawning_and_lifecycle ... ok
test fx::crt::tests::test_crt_overlay_defaults_and_activation ... ok
test fx::shake::tests::test_screen_shake_decay ... ok
test fx::transition::tests::test_transition_lifecycle_and_coverage ... ok
test fx::transition::tests::test_transition_presets ... ok
test input::filter::tests::test_presets_are_ordered_and_distinct ... ok
test input::filter::tests::test_legacy_profile_names_deserialize ... ok
test audio::sfx::tests::test_procedural_ui_sounds ... ok
test input::filter::tests::test_raw_pedals_are_instant ... ok
test input::filter::tests::test_slider_edit_becomes_custom ... ok
test input::filter::tests::test_steering_reaches_full_in_steer_time ... ok
test input::gamepad::tests::test_binding_active_matching ... ok
test input::key_repeat::tests::test_key_repeat_lifecycle ... ok
test input::mapping::tests::test_input_map_axis_vector_with_gamepad ... ok
test audio::dsp::tests::test_multicylinder_combustion_summation_constructive ... ok
test input::mapping::tests::test_input_map_rebinding ... ok
test input::nav_intent::tests::test_nav_intent_predicates ... ok
test input::gamepad::tests::test_custom_gamepad_profile_loading ... ok
test input::mapping::tests::test_input_map_defaults_and_serialization ... ok
test net::clock::tests::test_offset_uses_lowest_rtt_sample ... ok
test net::clock::tests::test_old_samples_leave_the_window ... ok
test net::clock::tests::test_rejects_negative_or_non_finite_samples ... ok
test input::mapping::tests::test_racing_presets_and_labels ... ok
test net::interp::tests::test_angle_takes_the_short_way_round ... ok
test net::interp::tests::test_extrapolation_is_bounded_then_holds ... ok
test net::beacon::tests::test_broadcaster_timer_interval ... ok
test input::nav2d::tests::test_nav_grid_2d_creation_and_bounds ... ok
test net::interp::tests::test_fresh_state_after_extrapolation_blends_without_snap ... ok
test input::mapping::tests::test_racing_presets_no_steering_overlap_with_down ... ok
test net::interp::tests::test_old_and_duplicate_states_are_dropped ... ok
test net::ip::tests::test_format_address ... ok
test net::ip::tests::test_parse_address_invalid ... ok
test net::ip::tests::test_parse_address_with_port ... ok
test net::ip::tests::test_parse_address_without_port_uses_default ... ok
test net::ip::tests::test_resolve_local_ipv4_returns_valid_ip ... ok
test net::protocol::tests::test_constants_and_magic_header ... ok
test net::protocol::tests::test_packet_type_accessors_and_conversions ... ok
test net::protocol::tests::test_lan_beacon_roundtrip ... ok
test net::protocol::tests::test_reject_invalid_magic ... ok
test net::protocol::tests::test_reject_oversized_packet ... ok
test net::protocol::tests::test_reject_short_packet ... ok
test net::protocol::tests::test_reject_version_mismatch ... ok
test net::protocol::tests::test_lobby_packet_roundtrips ... ok
test net::protocol::tests::test_sanitize_string ... ok
test net::reliable::tests::test_channel_fails_after_max_tries_without_ack ... ok
test net::protocol::tests::test_control_message_roundtrips ... ok
test net::reliable::tests::test_duplicates_are_acked_but_not_delivered_twice ... ok
test net::reliable::tests::test_rejects_malformed_fragment ... ok
test net::reliable::tests::test_sequence_wraps_around ... ok
test net::stats::tests::test_counting_transport_and_rates ... ok
test net::transport::tests::test_sim_delivers_after_delay_in_send_order ... ok
test net::transport::tests::test_sim_jitter_reorders ... ok
test audio::sfx::tests::test_procedural_impacts_and_skid ... ok
test net::ui::ip_keypad::tests::test_ip_keypad_button_activations ... ok
test net::ui::ip_keypad::tests::test_ip_keypad_char_appending_and_backspace ... ok
test net::interp::tests::test_circle_with_loss_and_jitter_stays_within_one_metre ... ok
test net::ui::ip_keypad::tests::test_ip_keypad_initialization_and_truncation ... ok
test net::ui::ip_keypad::tests::test_ip_keypad_nav_movement ... ok
test net::ui::virtual_keypad::tests::test_virtual_keypad_alphanumeric_mode ... ok
test net::ui::virtual_keypad::tests::test_virtual_keypad_ip_mode ... ok
test net::wire::tests::test_car_state_roundtrip_is_exact ... ok
test net::wire::tests::test_decoders_reject_bad_input_without_panic ... ok
test net::wire::tests::test_world_state_refuses_too_many_cars ... ok
test net::wire::tests::test_eight_car_world_state_fits_and_roundtrips ... ok
test records::leaderboard::tests::test_hall_of_fame_ranking_lowest_time ... ok
test ui::accordion::tests::test_accordion_layout_rects_and_scrolling ... ok
test ui::card_grid::tests::test_card_grid_2d_orthogonal_navigation ... ok
test ui::accordion::tests::test_accordion_navigation_and_selection ... ok
test ui::card_grid::tests::test_card_grid_col_row_and_indices ... ok
test ui::card_grid::tests::test_card_grid_rect_math ... ok
test net::reliable::tests::test_every_message_arrives_once_in_order_with_30_percent_loss ... ok
test ui::card_grid::tests::test_card_grid_wrap_navigation ... ok
test ui::checklist_modal::tests::test_checklist_modal_rects_and_navigation ... ok
test ui::checklist_modal::tests::test_checklist_modal_toggle_and_batch_actions ... ok
test ui::countdown::tests::test_countdown_sequence_and_audio ... ok
test ui::data_table::tests::display_rows_fit_bounded_viewport_without_reordering ... ok
test ui::data_table::tests::test_data_table_layout_and_rects ... ok
test ui::data_table::tests::test_data_table_navigation_and_boundary_exits ... ok
test ui::data_table::tests::test_rank_badge_colors ... ok
test ui::filter_bar::tests::test_filter_bar_navigation_and_cycling ... ok
test ui::font::tests::fit_to_width_cuts_at_the_longest_prefix_and_adds_an_ellipsis ... ok
test ui::font::tests::fit_to_width_keeps_text_that_fits ... ok
test ui::font::tests::fit_to_width_never_splits_a_multibyte_character ... ok
test ui::layout::tests::test_flow_layout_wrap_traversal_and_exits ... ok
test ui::layout::tests::test_grid_layout_boundary_exits_and_wrapping ... ok
test ui::filter_bar::tests::test_filter_bar_exit_signals ... ok
test ui::layout::tests::test_hstack_equal_distribution ... ok
test ui::layout::tests::test_hstack_navigation_and_boundary ... ok
test ui::layout::tests::test_scroll_indicator_ratios_and_thumb ... ok
test ui::layout::tests::test_split_pane_focus_handoff_and_exits ... ok
test ui::layout::tests::test_vstack_layout_and_navigation ... ok
test ui::layout::tests::test_vstack_windowed_scrolling ... ok
test ui::metric::tests::test_kpi_tile_construction ... ok
test ui::metric::tests::test_metric_bar_clamping_and_styles ... ok
test ui::metric::tests::test_progress_bar_clamping ... ok
test ui::modal::tests::test_modal_container_open_close_and_content_rect ... ok
test ui::page_dots::tests::test_page_dots_navigation_and_bounds ... ok
test ui::screen_footer::tests::test_footer_prompts_and_layout ... ok
test ui::screen_footer::tests::test_hero_button_position_and_hit_test ... ok
test ui::swatch_picker::tests::test_swatch_picker_layout_and_rects ... ok
test ui::swatch_picker::tests::test_swatch_picker_navigation_and_exits ... ok
test ui::symbols::tests::every_symbol_has_a_positive_advance_except_hidden ... ok
test ui::symbols::tests::plain_text_has_no_symbols ... ok
test ui::symbols::tests::symbols_split_out_of_text_runs ... ok
test ui::symbols::tests::variation_selectors_and_flag_letters_are_dropped ... ok
test ui::text_input::tests::test_text_input_backspace_and_delete ... ok
test ui::text_input::tests::test_text_input_boundary_exits ... ok
test ui::text_input::tests::test_text_input_char_filter ... ok
test ui::text_input::tests::test_text_input_character_insertion_and_limits ... ok
test ui::text_input::tests::test_text_input_cursor_movement_and_mid_insertion ... ok
test ui::toast::tests::test_toast_alpha_decay_lifecycle ... ok
test ui::toast::tests::test_toast_overlay_queue_and_culling ... ok
test ui::toast::tests::test_toast_rect_layout ... ok
test ui::tooltip::tests::test_help_chip_creation ... ok
test ui::tooltip::tests::test_tooltip_properties ... ok
test ui::widgets::tests::test_counter_bounds_and_actions ... ok
test ui::widgets::tests::test_dropdown_widget_cycling_and_popup ... ok
test ui::widgets::tests::test_option_cycler_forward_backward_and_wrap ... ok
test ui::widgets::tests::test_radio_group_single_select_and_exits ... ok
test ui::widgets::tests::test_slider_widget_math_and_clamping ... ok
test ui::widgets::tests::test_slider_widget_percentage_formatting ... ok
test ui::widgets::tests::test_tab_bar_cycling_and_selection ... ok
test ui::widgets::tests::test_toggle_flip_and_locked_state ... ok
test ui::widgets::tests::test_value_stepper_step_up_down ... ok
test net::transport::tests::test_sim_loss_is_deterministic_and_close_to_rate ... ok
test input::gamepad::tests::test_hardware_gamepad_presence ... ok
test net::beacon::tests::test_beacon_broadcaster_and_scanner_loopback ... ok
test audio::sfx::tests::test_procedural_countdown_and_chimes ... ok
test audio::backend::tests::test_sound_data_decoding_and_active_handle ... ok
test audio::sink::tests::test_cabinet_audio_player_initialization_and_cues ... ok

test result: ok. 142 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s


running 2 tests
test pause_focus_rows_and_setting_actions_preserve_paused_state ... ok
test results_columns_preserve_order_and_racing_formatting ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s


running 18 tests
test test_arcade_settings_modal_integration_and_bindings ... ok
test test_championship_and_profile_audio_toggle_preserves_state ... ok
test test_controls_help_game_state_transitions ... ok
test test_finished_state_audio_toggle_preserves_finished_state_and_hof ... ok
test test_gamepad_connection_status_snapshot ... ok
test test_menu_and_module_select_settings_x_shortcut_and_unsaved_flow ... ok
test test_menu_state_settings_modal_integration ... ok
test test_module_select_state_settings_modal_integration ... ok
test test_pause_menu_layout_and_buttons ... ok
test test_pause_menu_nav_grid_2d_navigation ... ok
test test_pause_state_audio_toggle_preserves_paused_state ... ok
test test_player_starts_in_arcade_and_preserves_last_used_mode_across_new_races ... ok
test test_profile_switch_restores_last_used_mode ... ok
test test_screen_stack_and_cabinet_screen_architecture ... ok
test test_split_screen_players_independent_mode_preservation ... ok
test test_starting_grid_audio_toggle_preserves_starting_grid_state ... ok
test test_starting_grid_driver_count_bounds ... ok
test test_starting_grid_scaling_across_discipline_capacities ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.80s


running 28 tests
test test_career_mode_completed_championship_retention_and_replay ... ok
test test_career_mode_escape_returns_to_modality_select ... ok
test test_career_mode_initial_tier_1_championship_gating ... ok
test test_career_mode_opens_career_select ... ok
test test_career_mode_unlocking_tier_expands_championships ... ok
test test_category_based_race_eligibility_enforcement ... ok
test test_custom_race_modality_selection_invariants ... ok
test test_garage_lifecycle_and_return ... ok
test test_grand_hub_player_profile_selection_and_navigation ... ok
test test_grand_hub_to_modality_select_transition ... ok
test test_in_development_lan_cloud_modals ... ok
test test_menu_backward_transition_to_modality_select ... ok
test test_menu_category_filter_circuits_isolation ... ok
test test_menu_category_filter_cycling_and_direct_keys ... ok
test test_menu_track_confirmation_updates_active_module_and_car ... ok
test test_modality_card_navigation_and_wrapping ... ok
test test_modality_category_switching_and_wrapping ... ok
test test_modality_select_column_3_options_garage_navigation ... ok
test test_modality_select_escape_opens_exit_confirm_modal ... ok
test test_modality_select_options_championship_editor_flow ... ok
test test_modality_select_options_player_profile_flow ... ok
test test_modality_select_options_settings_modal_flow ... ok
test test_modality_select_options_track_editor_flow ... ok
test test_modality_single_selected_menu_isolation ... ok
test test_quick_race_modality_selection_invariants ... ok
test test_split_screen_modality_invariants ... ok
test test_starting_grid_card_0_opens_garage ... ok
test test_time_trial_and_free_ride_solo_invariants ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.11s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running unittests src/lib.rs (target/debug/deps/cabinet-73ea433c323397f1)
warning: tdrace-core@0.1.0: embedded official circuits are 10.3 MB (target <= 8 MB)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s

```
