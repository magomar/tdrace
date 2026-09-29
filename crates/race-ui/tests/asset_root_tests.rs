//! Spec 057 (`specs/057_raceui_rendering_camera_effects_and_hud_primitives.md`): a game points
//! the surface textures at its own asset folder.

use race_ui::render::set_asset_root;
use race_ui::render::surface_material::SurfaceMaterialRegistry;

/// Scenario: A game points the textures at its own folder
///
/// Given a temporary folder with textures/surfaces/asphalt_diffuse.png in it
/// When set_asset_root names that folder and the surface texture file lookup runs for asphalt
/// Then it returns that file's bytes, and without set_asset_root the lookup still searches the
/// current paths
#[test]
fn asset_root_is_searched_first() {
    // Without an asset root: the repo's assets/ folder is found from the crate folder.
    let repo_copy = std::fs::read("../../assets/textures/surfaces/asphalt_diffuse.png").expect("repo texture");
    assert_eq!(SurfaceMaterialRegistry::find_surface_asset_file("asphalt_diffuse.png"), Some(repo_copy.clone()));

    let root = std::env::temp_dir().join(format!("race_ui_assets_{}", std::process::id()));
    let dir = root.join("textures/surfaces");
    std::fs::create_dir_all(&dir).unwrap();
    let own = b"not a real png, only this game's bytes".to_vec();
    assert_ne!(own, repo_copy);
    std::fs::write(dir.join("asphalt_diffuse.png"), &own).unwrap();

    set_asset_root(&root);
    assert_eq!(SurfaceMaterialRegistry::find_surface_asset_file("asphalt_diffuse.png"), Some(own));
    // A file the game does not have still falls back to the current paths.
    assert!(SurfaceMaterialRegistry::find_surface_asset_file("curb_teeth.png").is_some());
    let _ = std::fs::remove_dir_all(&root);
}
