---
type: validation_receipt
schema_version: "1.0"
spec: "specs/068_unified_ui_and_ux_consistency_across_tdrace_and_shared_platform_crate.md"
epic: "tdrace-hqy2"
candidate_commit: "28065e183bb64a43513e6996bcfb0d739c5f6ab0"
verifier: "local-user"
evaluated_at: "2026-09-30T08:43:02Z"
command: "cargo test -p cabinet && cargo test -p tdrace-app --test modality_flow_tests"
exit_code: 0
duration_ms: 6981
status: passed
---

# 🧾 Validation Receipt: Spec 068

- **Candidate Commit**: `28065e183bb64a43513e6996bcfb0d739c5f6ab0`
- **Spec**: `specs/068_unified_ui_and_ux_consistency_across_tdrace_and_shared_platform_crate.md`
- **Command**: `cargo test -p cabinet && cargo test -p tdrace-app --test modality_flow_tests`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:

running 120 tests
test audio::dsp::tests::test_adsr_envelope ... ok
test audio::backend::tests::test_amplitude_to_db ... ok
test audio::dsp::tests::test_biquad_bandpass_filtering_and_stability ... ok
test audio::dsp::tests::test_combustion_and_cylinder_oscillators ... ok
test audio::dsp::tests::test_biquad_lowpass_stability ... ok
test audio::dsp::tests::test_encode_wav_16bit_stereo_headers ... ok
test audio::dsp::tests::test_encode_wav_16bit_mono_headers ... ok
test audio::dsp::tests::test_waveshape_engine ... ok
test audio::sink::tests::test_mock_audio_sink ... ok
test audio::sfx::tests::test_procedural_ui_sounds ... ok
test fx::crt::tests::test_crt_overlay_defaults_and_activation ... ok
test fx::crt::tests::test_crt_overlay_roll_animation ... ok
test fx::crt::tests::test_scanline_mode_indices_and_opacities ... ok
test audio::dsp::tests::test_multicylinder_combustion_summation_constructive ... ok
test fx::floating_text::tests::test_capacity_capping ... ok
test fx::floating_text::tests::test_floating_text_spawning_and_lifecycle ... ok
test fx::shake::tests::test_screen_shake_decay ... ok
test fx::transition::tests::test_transition_lifecycle_and_coverage ... ok
test fx::transition::tests::test_transition_presets ... ok
test input::filter::tests::test_legacy_profile_names_deserialize ... ok
test input::filter::tests::test_presets_are_ordered_and_distinct ... ok
test input::filter::tests::test_raw_pedals_are_instant ... ok
test input::filter::tests::test_steering_reaches_full_in_steer_time ... ok
test input::filter::tests::test_slider_edit_becomes_custom ... ok
test input::gamepad::tests::test_binding_active_matching ... ok
test input::mapping::tests::test_input_map_rebinding ... ok
test input::mapping::tests::test_input_map_axis_vector_with_gamepad ... ok
test input::nav2d::tests::test_nav_grid_2d_creation_and_bounds ... ok
test input::mapping::tests::test_racing_presets_no_steering_overlap_with_down ... ok
test input::mapping::tests::test_input_map_defaults_and_serialization ... ok
test input::gamepad::tests::test_custom_gamepad_profile_loading ... ok
test input::mapping::tests::test_racing_presets_and_labels ... ok
test net::clock::tests::test_offset_uses_lowest_rtt_sample ... ok
test net::clock::tests::test_old_samples_leave_the_window ... ok
test net::beacon::tests::test_broadcaster_timer_interval ... ok
test net::clock::tests::test_rejects_negative_or_non_finite_samples ... ok
test net::interp::tests::test_angle_takes_the_short_way_round ... ok
test net::interp::tests::test_extrapolation_is_bounded_then_holds ... ok
test audio::sfx::tests::test_procedural_impacts_and_skid ... ok
test net::interp::tests::test_fresh_state_after_extrapolation_blends_without_snap ... ok
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
test net::protocol::tests::test_lobby_packet_roundtrips ... ok
test net::protocol::tests::test_reject_short_packet ... ok
test net::protocol::tests::test_control_message_roundtrips ... ok
test net::protocol::tests::test_reject_version_mismatch ... ok
test net::protocol::tests::test_sanitize_string ... ok
test net::reliable::tests::test_channel_fails_after_max_tries_without_ack ... ok
test net::reliable::tests::test_duplicates_are_acked_but_not_delivered_twice ... ok
test net::reliable::tests::test_rejects_malformed_fragment ... ok
test net::reliable::tests::test_sequence_wraps_around ... ok
test net::stats::tests::test_counting_transport_and_rates ... ok
test net::transport::tests::test_sim_delivers_after_delay_in_send_order ... ok
test net::ui::ip_keypad::tests::test_ip_keypad_button_activations ... ok
test net::ui::ip_keypad::tests::test_ip_keypad_char_appending_and_backspace ... ok
test net::ui::ip_keypad::tests::test_ip_keypad_initialization_and_truncation ... ok
test net::wire::tests::test_eight_car_world_state_fits_and_roundtrips ... ok
test net::transport::tests::test_sim_jitter_reorders ... ok
test net::ui::ip_keypad::tests::test_ip_keypad_nav_movement ... ok
test net::wire::tests::test_world_state_refuses_too_many_cars ... ok
test net::wire::tests::test_car_state_roundtrip_is_exact ... ok
test ui::card_grid::tests::test_card_grid_2d_orthogonal_navigation ... ok
test ui::accordion::tests::test_accordion_navigation_and_selection ... ok
test ui::card_grid::tests::test_card_grid_rect_math ... ok
test ui::checklist_modal::tests::test_checklist_modal_rects_and_navigation ... ok
test records::leaderboard::tests::test_hall_of_fame_ranking_lowest_time ... ok
test ui::card_grid::tests::test_card_grid_wrap_navigation ... ok
test ui::checklist_modal::tests::test_checklist_modal_toggle_and_batch_actions ... ok
test ui::data_table::tests::test_data_table_layout_and_rects ... ok
test ui::data_table::tests::test_data_table_navigation_and_boundary_exits ... ok
test ui::data_table::tests::test_rank_badge_colors ... ok
test ui::font::tests::fit_to_width_keeps_text_that_fits ... ok
test ui::filter_bar::tests::test_filter_bar_navigation_and_cycling ... ok
test net::wire::tests::test_decoders_reject_bad_input_without_panic ... ok
test ui::accordion::tests::test_accordion_layout_rects_and_scrolling ... ok
test ui::font::tests::fit_to_width_cuts_at_the_longest_prefix_and_adds_an_ellipsis ... ok
test ui::card_grid::tests::test_card_grid_col_row_and_indices ... ok
test ui::font::tests::fit_to_width_never_splits_a_multibyte_character ... ok
test ui::layout::tests::test_hstack_equal_distribution ... ok
test ui::filter_bar::tests::test_filter_bar_exit_signals ... ok
test ui::layout::tests::test_hstack_navigation_and_boundary ... ok
test ui::layout::tests::test_vstack_layout_and_navigation ... ok
test ui::layout::tests::test_vstack_windowed_scrolling ... ok
test ui::screen_footer::tests::test_footer_prompts_and_layout ... ok
test ui::screen_footer::tests::test_hero_button_position_and_hit_test ... ok
test ui::swatch_picker::tests::test_swatch_picker_layout_and_rects ... ok
test ui::swatch_picker::tests::test_swatch_picker_navigation_and_exits ... ok
test ui::symbols::tests::every_symbol_has_a_positive_advance_except_hidden ... ok
test ui::symbols::tests::plain_text_has_no_symbols ... ok
test ui::symbols::tests::symbols_split_out_of_text_runs ... ok
test ui::symbols::tests::variation_selectors_and_flag_letters_are_dropped ... ok
test ui::text_input::tests::test_text_input_backspace_and_delete ... ok
test net::reliable::tests::test_every_message_arrives_once_in_order_with_30_percent_loss ... ok
test ui::text_input::tests::test_text_input_boundary_exits ... ok
test ui::text_input::tests::test_text_input_char_filter ... ok
test ui::text_input::tests::test_text_input_character_insertion_and_limits ... ok
test ui::text_input::tests::test_text_input_cursor_movement_and_mid_insertion ... ok
test ui::toast::tests::test_toast_alpha_decay_lifecycle ... ok
test ui::toast::tests::test_toast_overlay_queue_and_culling ... ok
test ui::toast::tests::test_toast_rect_layout ... ok
test net::interp::tests::test_circle_with_loss_and_jitter_stays_within_one_metre ... ok
test ui::widgets::tests::test_dropdown_widget_cycling_and_popup ... ok
test ui::widgets::tests::test_slider_widget_math_and_clamping ... ok
test ui::widgets::tests::test_slider_widget_percentage_formatting ... ok
test ui::widgets::tests::test_tab_bar_cycling_and_selection ... ok
test net::transport::tests::test_sim_loss_is_deterministic_and_close_to_rate ... ok
test input::gamepad::tests::test_hardware_gamepad_presence ... ok
test net::beacon::tests::test_beacon_broadcaster_and_scanner_loopback ... ok
test audio::sfx::tests::test_procedural_countdown_and_chimes ... ok
test audio::sink::tests::test_cabinet_audio_player_initialization_and_cues ... ok
test audio::backend::tests::test_sound_data_decoding_and_active_handle ... ok

test result: ok. 120 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s


running 1 test
test profile_paths_follow_the_app_id ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 26 tests
test test_arcade_settings_modal_controls_subtabs_navigation_and_profile_presets ... ok
test test_arcade_settings_modal_controls_tab_widgets_and_rollback ... ok
test test_arcade_settings_modal_display_tab_integration ... ok
test test_arcade_settings_modal_helpers_tab_integration ... ok
test test_arcade_settings_modal_lifecycle_and_bindings ... ok
test test_audio_mixer_buses ... ok
test test_arcade_settings_modal_sync_gamepad_config_is_not_an_unsaved_edit ... ok
test test_cabinet_settings_widgets_interaction ... ok
test test_crt_scanlines_and_settings_integration ... ok
test test_digital_input_filter_progressive_ramp ... ok
test test_display_resolutions_and_window_modes ... ok
test test_floating_text_popups_and_decay ... ok
test test_arcade_settings_dirty_tracking_and_exit_modal ... ok
test test_juice_fx_mechanics ... ok
test test_modal_screen_stack ... ok
test test_nav_grid_2d_orthogonal_navigation ... ok
test test_cabinet_context_audio_wiring_and_tactile_feedback ... ok
test test_profile_manager_lifecycle ... ok
test test_input_mapping_action_system ... ok
test test_screen_stack_transitions_lifecycle ... ok
test test_leaderboard_modal_rendering_and_scrolling ... ok
test test_ui_scaler_responsive_math ... ok
test test_profile_select_modal_slot_and_customization ... ok
test test_universal_confirm_modal_lifecycle ... ok
test test_arcade_settings_modal_gameplay_subtab_navigation ... ok
test test_arcade_settings_modal_arrow_category_navigation ... ok

test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 10 tests
test test_d1_eight_car_world_state_fits_one_datagram ... ok
test test_cabinet_lan_host_and_join_screens_lifecycle ... ok
test test_v1_client_gets_a_v1_version_mismatch_reply ... ok
test test_client_graceful_disconnect ... ok
test test_host_and_client_loopback_handshake ... ok
test test_client_slot_customization_and_ready_check ... ok
test test_d5_lost_state_sync_does_not_change_the_client_roster ... ok
test test_lobby_pump_network_keeps_client_connected_and_syncs_car ... ok
test test_d4_client_in_slot_three_keeps_its_slot_after_launch ... ok
test test_launch_start_and_car_state_relay ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s


running 3 tests
test test_space_arena_game_simulation_and_juice ... ok
test test_space_arena_modal_stack_pause_resume ... ok
test test_space_arena_modals_integration ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 28 tests
test test_category_based_race_eligibility_enforcement ... ok
test test_modality_card_navigation_and_wrapping ... ok
test test_career_mode_initial_tier_1_championship_gating ... ok
test test_starting_grid_card_0_opens_garage ... ok
test test_garage_lifecycle_and_return ... ok
test test_modality_select_options_player_profile_flow ... ok
test test_modality_select_escape_opens_exit_confirm_modal ... ok
test test_menu_backward_transition_to_modality_select ... ok
test test_grand_hub_player_profile_selection_and_navigation ... ok
test test_menu_category_filter_cycling_and_direct_keys ... ok
test test_split_screen_modality_invariants ... ok
test test_modality_select_options_championship_editor_flow ... ok
test test_career_mode_opens_career_select ... ok
test test_custom_race_modality_selection_invariants ... ok
test test_modality_single_selected_menu_isolation ... ok
test test_in_development_lan_cloud_modals ... ok
test test_time_trial_and_free_ride_solo_invariants ... ok
test test_career_mode_unlocking_tier_expands_championships ... ok
test test_modality_select_options_settings_modal_flow ... ok
test test_career_mode_escape_returns_to_modality_select ... ok
test test_modality_select_column_3_options_garage_navigation ... ok
test test_modality_category_switching_and_wrapping ... ok
test test_quick_race_modality_selection_invariants ... ok
test test_grand_hub_to_modality_select_transition ... ok
test test_career_mode_completed_championship_retention_and_replay ... ok
test test_modality_select_options_track_editor_flow ... ok
test test_menu_track_confirmation_updates_active_module_and_car ... ok
test test_menu_category_filter_circuits_isolation ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.09s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running unittests src/lib.rs (target/debug/deps/cabinet-cedf2a01979adf9b)
     Running tests/app_id_tests.rs (target/debug/deps/app_id_tests-b35bfde29cf2de01)
     Running tests/cabinet_integration_tests.rs (target/debug/deps/cabinet_integration_tests-d5ac5d838e38a120)
     Running tests/net_tests.rs (target/debug/deps/net_tests-8d652a196a6e3372)
     Running tests/space_arena_tests.rs (target/debug/deps/space_arena_tests-a4364cbe7f6e4419)
   Doc-tests cabinet
warning: tdrace-core@0.1.0: embedded official circuits are 9.3 MB (target <= 8 MB)
   Compiling race-ui v0.1.0 (/home/mario/workspace/games/tdrace/crates/race-ui)
   Compiling tdrace-app v0.1.0 (/home/mario/workspace/games/tdrace/crates/tdrace-app)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.29s
     Running tests/modality_flow_tests.rs (target/debug/deps/modality_flow_tests-afee74192dd2f522)

```
