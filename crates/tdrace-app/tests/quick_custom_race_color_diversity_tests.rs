use macroquad::color::Color;
use tdrace_app::catalog::{find_model_by_id, get_models_for_module};
use tdrace_app::game::RaceSession;
use tdrace_app::ui::menu::GameMode;

fn color_dist(c1: Color, c2: Color) -> f32 {
    let dr = c1.r - c2.r;
    let dg = c1.g - c2.g;
    let db = c1.b - c2.b;
    (dr * dr + dg * dg + db * db).sqrt()
}

fn is_factory_color(c: Color, factory: Color) -> bool {
    let dr = (c.r - factory.r).abs();
    let dg = (c.g - factory.g).abs();
    let db = (c.b - factory.b).abs();
    dr < 0.05 && dg < 0.05 && db < 0.05
}

#[test]
fn test_quick_race_gt_bots_color_masking_and_diversity() {
    let mut session = RaceSession::new();
    session.game_mode = GameMode::StandardRace;
    session.switch_to_gt();
    session.num_bots = 7;
    session.init_race();

    assert_eq!(session.game_mode, GameMode::StandardRace);
    assert_eq!(session.grid_participants.len(), 8);
    assert_eq!(session.cars.len(), 8);

    let player_participant = session
        .grid_participants
        .iter()
        .find(|p| p.is_player)
        .expect("Player participant must exist");
    let player_primary = player_participant.color_scheme.primary;

    let bot_participants: Vec<&tdrace_app::game::GridParticipant> = session
        .grid_participants
        .iter()
        .filter(|p| !p.is_player)
        .collect();
    assert_eq!(bot_participants.len(), 7);

    // 1. Verify all 7 bots in grid_participants
    for (idx, bot) in bot_participants.iter().enumerate() {
        let bot_scheme = bot.color_scheme;
        let bot_model_id = bot.model_id.expect("GT bot must have model ID");
        let bot_model = find_model_by_id(bot_model_id).expect("Bot model must exist in catalog");

        // Bot must NOT match factory livery (must trigger dynamic mask-based tinting)
        assert!(
            !is_factory_color(bot_scheme.primary, bot_model.primary_color),
            "Bot {} ('{}' in '{}') must not match factory livery; primary was {:?}, factory is {:?}",
            idx,
            bot.name,
            bot_model.name,
            bot_scheme.primary,
            bot_model.primary_color
        );

        // Bot primary color must be visually distinct from player (dist >= 0.20)
        let dist_to_player = color_dist(bot_scheme.primary, player_primary);
        assert!(
            dist_to_player >= 0.20,
            "Bot {} color ({:?}) is too close to player color ({:?}), dist = {:.3}",
            idx,
            bot_scheme.primary,
            player_primary,
            dist_to_player
        );

        // In-race session color_schemes array must match participant scheme
        let car_idx = bot.bot_index.map(|b| b + 1).unwrap_or(idx + 1);
        assert_eq!(
            session.color_schemes[car_idx], bot_scheme,
            "Session color scheme for car {} must match its participant scheme",
            car_idx
        );

        // Bots sharing the same vehicle model must have distinct primary colors (dist >= 0.20)
        for other_bot in &bot_participants[(idx + 1)..] {
            if other_bot.model_id == bot.model_id {
                let dist_between_same_model = color_dist(bot_scheme.primary, other_bot.color_scheme.primary);
                assert!(
                    dist_between_same_model >= 0.20,
                    "Bots '{}' and '{}' share model '{}' but have near-identical colors: dist = {:.3}",
                    bot.name,
                    other_bot.name,
                    bot_model.name,
                    dist_between_same_model
                );
            }
        }
    }

    // 2. Also directly verify all in-race bot cars in session.cars / session.color_schemes
    for car_idx in 1..session.cars.len() {
        let scheme = session.color_schemes[car_idx];
        let model_id = session.car_model_ids[car_idx].expect("Bot car must have model id");
        let model = find_model_by_id(model_id).expect("Bot model exists in catalog");

        assert!(
            !is_factory_color(scheme.primary, model.primary_color),
            "In-race car {} must not match factory livery",
            car_idx
        );
        assert!(
            color_dist(scheme.primary, player_primary) >= 0.20,
            "In-race car {} must be distinct from player",
            car_idx
        );
    }
}

#[test]
fn test_custom_race_single_make_grid_color_diversity() {
    let mut session = RaceSession::new();
    session.game_mode = GameMode::ExperimentalRace;
    session.switch_to_gt();
    session.num_bots = 7;
    session.init_race();

    assert_eq!(session.game_mode, GameMode::ExperimentalRace);
    assert_eq!(session.grid_participants.len(), 8);

    // Get a specific model (e.g. BMW M4 GT4) and force ALL bots to use that exact model
    let gt_models = get_models_for_module("gt");
    assert!(!gt_models.is_empty());
    let target_model = gt_models[0];

    // Simulate single-make grid by cycling/setting bot models
    for roster_idx in 0..session.grid_participants.len() {
        if session.grid_participants[roster_idx].is_player {
            continue;
        }
        session.starting_grid_roster_idx = roster_idx;
        // Cycle until the model matches target_model.id
        for _ in 0..gt_models.len() {
            if session.grid_participants[roster_idx].model_id == Some(target_model.id) {
                break;
            }
            session.cycle_starting_grid_participant_model(true);
        }
        assert_eq!(
            session.grid_participants[roster_idx].model_id,
            Some(target_model.id),
            "Failed to configure bot roster slot {} to single-make model {}",
            roster_idx,
            target_model.id
        );
    }

    let player_participant = session
        .grid_participants
        .iter()
        .find(|p| p.is_player)
        .expect("Player participant must exist");
    let player_primary = player_participant.color_scheme.primary;

    let bot_participants: Vec<&tdrace_app::game::GridParticipant> = session
        .grid_participants
        .iter()
        .filter(|p| !p.is_player)
        .collect();
    assert_eq!(bot_participants.len(), 7);

    // Verify all 7 bots on this single-make grid have diverse, non-factory colors
    for (idx, bot) in bot_participants.iter().enumerate() {
        let bot_scheme = bot.color_scheme;

        // 1. None of the bots use factory livery
        assert!(
            !is_factory_color(bot_scheme.primary, target_model.primary_color),
            "Bot {} on single-make grid must not match factory livery",
            idx
        );

        // 2. All bots distinct from player
        let dist_to_player = color_dist(bot_scheme.primary, player_primary);
        assert!(
            dist_to_player >= 0.20,
            "Bot {} on single-make grid must be distinct from player, dist = {:.3}",
            idx,
            dist_to_player
        );

        // 3. Every pair of bots must be visually distinct from each other (dist >= 0.20)
        for other_bot in &bot_participants[(idx + 1)..] {
            let dist_between_bots = color_dist(bot_scheme.primary, other_bot.color_scheme.primary);
            assert!(
                dist_between_bots >= 0.20,
                "Bots '{}' and '{}' on single-make grid have clashing colors: {:?} vs {:?}, dist = {:.3}",
                bot.name,
                other_bot.name,
                bot_scheme.primary,
                other_bot.color_scheme.primary,
                dist_between_bots
            );
        }
    }
}

#[test]
fn test_multi_discipline_quick_races_color_masking_and_diversity() {
    let disciplines = ["nascar", "rallycross", "extreme_offroad", "kart"];

    for disc in disciplines {
        let mut session = RaceSession::new();
        session.game_mode = GameMode::StandardRace;
        session.apply_module_config(disc);
        session.num_bots = 7;
        session.init_race();

        assert_eq!(
            session.grid_participants.len(),
            8,
            "Discipline '{}' must spawn 8 participants",
            disc
        );

        let player_participant = session
            .grid_participants
            .iter()
            .find(|p| p.is_player)
            .expect("Player participant must exist");
        let player_primary = player_participant.color_scheme.primary;

        let bot_participants: Vec<&tdrace_app::game::GridParticipant> = session
            .grid_participants
            .iter()
            .filter(|p| !p.is_player)
            .collect();

        for (idx, bot) in bot_participants.iter().enumerate() {
            let bot_scheme = bot.color_scheme;

            // If bot has a catalog model, check factory livery avoidance
            if let Some(bot_model_id) = bot.model_id {
                if let Some(bot_model) = find_model_by_id(bot_model_id) {
                    assert!(
                        !is_factory_color(bot_scheme.primary, bot_model.primary_color),
                        "Discipline '{}', Bot {} must not match factory livery",
                        disc,
                        idx
                    );
                }
            }

            // Must be distinct from player
            let dist_to_player = color_dist(bot_scheme.primary, player_primary);
            assert!(
                dist_to_player >= 0.20,
                "Discipline '{}', Bot {} color ({:?}) is too close to player ({:?}), dist = {:.3}",
                disc,
                idx,
                bot_scheme.primary,
                player_primary,
                dist_to_player
            );

            // Bots sharing the same vehicle model must have distinct colors
            if let Some(bot_mid) = bot.model_id {
                for other_bot in &bot_participants[(idx + 1)..] {
                    if other_bot.model_id == Some(bot_mid) {
                        let dist_same = color_dist(bot_scheme.primary, other_bot.color_scheme.primary);
                        assert!(
                            dist_same >= 0.20,
                            "Discipline '{}', Bots '{}' and '{}' have same model but clashing colors, dist = {:.3}",
                            disc,
                            bot.name,
                            other_bot.name,
                            dist_same
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_custom_race_starting_grid_vehicle_cycling_reresolves_color_scheme() {
    let mut session = RaceSession::new();
    session.game_mode = GameMode::ExperimentalRace;
    session.switch_to_gt();
    session.num_bots = 7;
    session.init_race();

    // Find a bot slot on the Starting Grid
    let bot_roster_idx = session
        .grid_participants
        .iter()
        .position(|p| !p.is_player)
        .expect("Must have bot participant");
    session.starting_grid_roster_idx = bot_roster_idx;

    let initial_bot_model_id = session.grid_participants[bot_roster_idx].model_id.expect("Model id required");
    let bot_idx_opt = session.grid_participants[bot_roster_idx].bot_index;
    let car_idx = bot_idx_opt.map(|b| b + 1).unwrap_or(bot_roster_idx);

    // Cycle forward
    session.cycle_starting_grid_participant_model(true);
    let new_bot_model_id = session.grid_participants[bot_roster_idx].model_id.expect("Model id required");
    assert_ne!(
        initial_bot_model_id, new_bot_model_id,
        "Cycling vehicle must change model ID"
    );

    let new_scheme = session.grid_participants[bot_roster_idx].color_scheme;
    let new_model = find_model_by_id(new_bot_model_id).expect("Catalog model exists");

    // Color scheme must not match new model's factory livery
    assert!(
        !is_factory_color(new_scheme.primary, new_model.primary_color),
        "Bot must not match new model's factory livery after cycling"
    );

    // Color scheme must be synchronized to session.color_schemes[car_idx]
    assert_eq!(
        session.color_schemes[car_idx], new_scheme,
        "session.color_schemes must reflect re-resolved scheme"
    );

    // Player color distance check
    let player_participant = session
        .grid_participants
        .iter()
        .find(|p| p.is_player)
        .expect("Player participant must exist");
    let player_primary = player_participant.color_scheme.primary;
    let dist_to_player = color_dist(new_scheme.primary, player_primary);
    assert!(
        dist_to_player >= 0.20,
        "Re-resolved scheme must remain distinct from player"
    );

    // Cycle backward returns to initial model
    session.cycle_starting_grid_participant_model(false);
    assert_eq!(
        session.grid_participants[bot_roster_idx].model_id,
        Some(initial_bot_model_id),
        "Cycling backward must restore previous vehicle model"
    );
}
