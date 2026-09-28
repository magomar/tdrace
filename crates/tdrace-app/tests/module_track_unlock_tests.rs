use tdrace_app::db::HallOfFameDb;
use tdrace_app::game::RaceSession;
use tdrace_app::module::extreme_offroad::ExtremeOffRoadModule;
use tdrace_app::module::gt::GtWorldChallengeModule;
use tdrace_app::module::kart::KartGameModule;
use tdrace_app::module::nascar::NascarGameModule;
use tdrace_app::module::rally::RallyGameModule;
use tdrace_app::module::GameModule;
use tdrace_app::profile::ModuleCareerProgress;

fn modules() -> Vec<(&'static str, Box<dyn GameModule>)> {
    vec![
        ("gt", Box::new(GtWorldChallengeModule::new())),
        ("nascar", Box::new(NascarGameModule::new())),
        ("rally", Box::new(RallyGameModule::new())),
        ("kart", Box::new(KartGameModule::new())),
        ("extreme_offroad", Box::new(ExtremeOffRoadModule::new())),
    ]
}

#[test]
fn test_every_module_circuit_unlocks_by_max_level() {
    let mut never_unlock = Vec::new();
    for (module_id, module) in modules() {
        let mut progress = ModuleCareerProgress::default_for_module(1, module_id);
        progress.level = 5;
        progress.sync_unlocks_for_level();
        for track in module.tracks() {
            if !progress.is_track_unlocked(track.id, false) {
                never_unlock.push(format!("{module_id}/{}", track.id));
            }
        }
    }
    assert!(never_unlock.is_empty(), "circuits that never unlock: {never_unlock:?}");
}

#[test]
fn test_every_module_locks_circuits_above_tier_one() {
    for (module_id, module) in modules() {
        let progress = ModuleCareerProgress::default_for_module(1, module_id);
        let total = module.tracks().len();
        let unlocked = module.tracks().iter().filter(|t| progress.is_track_unlocked(t.id, false)).count();
        // Extreme Off-Road also opens the three Mint 400 circuits at Tier 1 (spec 048)
        let expected = if module_id == "extreme_offroad" { 8 } else { 5 };
        assert_eq!(unlocked, expected, "{module_id}: Tier 1 should open {expected} circuits");
        assert!(total > unlocked, "{module_id}: some circuits should stay locked at Tier 1");
    }
}

#[test]
fn test_session_locks_circuits_in_every_module_but_classic() {
    let mut session = RaceSession::new();
    let mem_db = HallOfFameDb::open_in_memory().unwrap();
    mem_db.seed_default_profile_if_empty().unwrap();
    session.hof_db = Some(mem_db);
    session.refresh_profiles_and_stats();

    session.switch_to_nascar();
    assert!(session.is_track_unlocked("martinsville_speedway"));
    assert!(!session.is_track_unlocked("daytona_superspeedway"));

    session.switch_to_rally();
    assert!(session.is_track_unlocked("holjes_rx"));
    assert!(!session.is_track_unlocked("essay_rx"));

    session.switch_to_kart();
    assert!(session.is_track_unlocked("lonato"));
    assert!(!session.is_track_unlocked("campillos"));

    session.switch_to_extreme_offroad();
    assert!(session.is_track_unlocked("sahara_dune_crossing"));
    assert!(!session.is_track_unlocked("monster_colosseum"));

    session.switch_to_classic();
    assert!(session.is_track_unlocked("oval_speedway"));
    assert!(session.is_track_unlocked("ramp_raceway"));
}

#[test]
fn test_gt_madring_unlocks_at_tier_five() {
    let mut progress = ModuleCareerProgress::default_for_module(1, "gt");
    progress.level = 4;
    progress.sync_unlocks_for_level();
    assert!(!progress.is_track_unlocked("madring", false));
    progress.level = 5;
    progress.sync_unlocks_for_level();
    assert!(progress.is_track_unlocked("madring", false));
}
