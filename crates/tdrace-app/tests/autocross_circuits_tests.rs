use tdrace_core::catalog;
use tdrace_core::surface::SurfaceType;
use tdrace_core::track::{validate_track, ValidationSeverity};

#[test]
fn test_seventeen_autocross_circuits_embedded_in_catalog() {
    let circuits: Vec<_> = catalog::module_circuits("autocross").collect();
    assert_eq!(
        circuits.len(),
        17,
        "Expected exactly 17 circuits embedded for 'autocross' module"
    );

    let expected_slugs = [
        "nova_paka_ax",
        "prerov_ax",
        "humpolec_ax",
        "matschenberg_ax",
        "seelow_ax",
        "schluechtern_ax",
        "uelzen_ax",
        "st_georges_ax",
        "faleyras_ax",
        "st_junien_ax",
        "bazaigues_ax",
        "maggiora_ax",
        "musa_ax",
        "vilkyciai_ax",
        "arteixo_ax",
        "carballo_ax",
        "castelo_branco_ax",
    ];

    for (c, expected_id) in circuits.iter().zip(expected_slugs) {
        assert_eq!(c.id, expected_id);
        assert_eq!(c.module, "autocross");
        assert_eq!(c.category_label, "FIA Autocross");
        assert!(!c.country_code.is_empty());
        assert!((4..=6).contains(&c.default_laps));

        let track = c.load().expect("Embedded circuit must deserialize cleanly");
        assert_eq!(track.car_category, tdrace_core::CarCategory::Autocross);
        assert_eq!(track.module_id.as_deref(), Some("autocross"));
        assert_eq!(track.scale, "1:1");
        assert_eq!(track.grid_positions.len(), 12, "Autocross grids must seat 12 vehicles");
        assert!(track.checkpoints.len() >= 4, "Must have valid checkpoints");

        // Verify dominant surface is Dirt
        let total_samples = track.spline.samples.len();
        let dirt_samples = track.spline.samples.iter().filter(|s| s.surface == SurfaceType::Dirt).count();
        assert!(
            dirt_samples as f32 / total_samples as f32 >= 0.90,
            "Circuit {} dominant surface must be pure Dirt (got {}/{} dirt samples)",
            c.id,
            dirt_samples,
            total_samples
        );

        // Verify OSM provenance URL
        assert!(
            track.osm_url.as_ref().is_some_and(|u| u.starts_with("https://www.openstreetmap.org/way/")),
            "Circuit {} missing valid OSM URL: {:?}",
            c.id,
            track.osm_url
        );

        // Verify zero validation errors
        let errors: Vec<_> = validate_track(&track)
            .into_iter()
            .filter(|d| d.severity == ValidationSeverity::Error)
            .collect();
        assert!(errors.is_empty(), "Circuit {} has validation errors: {:?}", c.id, errors);
    }
}
