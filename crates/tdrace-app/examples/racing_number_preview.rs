//! Native visual smoke test for Spec 092: the racing number on a Classic kart, a Classic Autocross car and a
//! Classic Rallycross car in a real race at the default camera zoom, then the starting grid, driver card and
//! the number field of both profile editors. Saves `racing_number_<name>.png` to TDRACE_UI_PREVIEW_DIR
//! (default: current directory) and exits.
use macroquad::prelude::*;
use tdrace_app::db::HallOfFameDb;
use tdrace_app::game::{DriverCardsOrigin, GameState, RaceSession};
use tdrace_app::ui::menu::{CarChoice, TrackChoice};

fn window_conf() -> Conf {
    Conf { window_title: "Racing number preview".into(), window_width: 1600, window_height: 900, ..Default::default() }
}

/// A Classic race for one category: the car and the circuit slug (an official choice when `slug` is `None`).
fn classic_race(choice: CarChoice, model_id: &'static str, track: TrackChoice, slug: Option<&str>) -> RaceSession {
    let mut session = RaceSession::new();
    session.active_profile.race_number = 17;
    session.switch_to_classic();
    session.track_choice = track;
    session.car_choice = choice;
    session.selected_car_model_id = Some(model_id);
    if let Some(slug) = slug {
        session.track = session.track_manager.load_track_by_slug(slug).expect("circuit");
    }
    session.init_race();
    session.state = GameState::Racing;
    session
}

async fn shoot(session: &mut RaceSession, dir: &str, name: &str) {
    for frame in 0..3 {
        clear_background(BLACK);
        session.render();
        if frame == 2 {
            get_screen_data().export_png(&format!("{}/racing_number_{}.png", dir, name));
        }
        next_frame().await;
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let dir = std::env::var("TDRACE_UI_PREVIEW_DIR").unwrap_or_else(|_| ".".into());
    let ax = TrackChoice::Custom { id: "ax_clay_bowl".into(), title: "Clay Bowl".into(), description: String::new(), path: "tracks/classic/ax_clay_bowl.json".into() };
    let races = [
        ("kart", classic_race(CarChoice::Kart, "classic_kart", TrackChoice::KartArena, None)),
        ("autocross", classic_race(CarChoice::CrossCar, "classic_ax_mudlark", ax, Some("ax_clay_bowl"))),
        ("rallycross", classic_race(CarChoice::RallyCar, "classic_rally", TrackChoice::ClassicRallycross, None)),
    ];
    let mut screens = classic_race(CarChoice::RallyCar, "classic_rally", TrackChoice::ClassicRallycross, None);
    let db = HallOfFameDb::open_in_memory().expect("database");
    db.seed_default_profile_if_empty().expect("profile");
    screens.hof_db = Some(db);
    screens.refresh_profiles_and_stats();
    screens.active_profile.race_number = 17;
    screens.init_race();
    screens.state = GameState::StartingGrid;
    shoot(&mut screens, &dir, "grid").await;

    // The dossier lists the module's drivers: open it on the first one that is on this grid.
    let on_grid = screens.active_module_drivers().iter().position(|d| screens.driver_race_number(d.id) != 0);
    screens.driver_cards_idx = on_grid.unwrap_or(0);
    screens.state = GameState::DriverCards(DriverCardsOrigin::StartingGrid);
    shoot(&mut screens, &dir, "driver_card").await;

    screens.state = GameState::ProfileCreate {
        editing_id: None,
        field_idx: 4,
        input_name: "Mario".into(),
        input_alias: "Apex".into(),
        country_idx: 1,
        livery_idx: 2,
        race_number: 17,
        cursor_timer: 0.0,
    };
    shoot(&mut screens, &dir, "profile_create").await;

    screens.open_player_roster_manager(0);
    if let GameState::PlayerRosterManager { active_column, field_idx, race_number, .. } = &mut screens.state {
        *active_column = 1;
        *field_idx = 5;
        *race_number = 17;
    }
    shoot(&mut screens, &dir, "roster_manager").await;

    for (name, mut session) in races {
        for frame in 0..90 {
            clear_background(session.active_backdrop_color());
            session.update();
            session.render();
            if frame == 89 {
                get_screen_data().export_png(&format!("{}/racing_number_{}.png", dir, name));
            }
            next_frame().await;
        }
    }
}
