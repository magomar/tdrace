//! Embedded official catalog tests (specs/042_jsononly_official_circuit_catalog_and_embedded_track_data.md).

use std::path::Path;

use tdrace_core::catalog;
use tdrace_core::track::{validate_track, Track, ValidationSeverity};

const MODULES: [&str; 6] = ["classic", "extreme_offroad", "gt", "kart", "nascar", "rally"];

#[test]
fn test_every_embedded_circuit_equals_its_json_file() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tracks");
    for c in catalog::circuits() {
        let path = root.join(c.module).join(format!("{}.json", c.id));
        let from_disk = Track::load_from_file(&path).unwrap_or_else(|e| panic!("{}: {}", path.display(), e));
        let embedded = c.load().unwrap_or_else(|e| panic!("{}/{}: {}", c.module, c.id, e));
        assert!(embedded == from_disk, "embedded {}/{} differs from {}", c.module, c.id, path.display());
        assert_eq!(c.name, from_disk.name);
        assert_eq!(c.description, from_disk.description);
        assert_eq!(c.tag, from_disk.tag);
        assert_eq!(c.category_label, from_disk.category_label);
        assert_eq!(c.default_laps, from_disk.default_laps);
    }
}

#[test]
fn test_catalog_counts_per_module() {
    let counts: Vec<usize> = MODULES.iter().map(|m| catalog::module_circuits(m).count()).collect();
    assert_eq!(counts, vec![10, 17, 18, 17, 17, 17], "circuits per module {:?}", MODULES);
    assert_eq!(catalog::circuits().len(), 96);
}

#[test]
fn test_no_embedded_circuit_has_validation_errors() {
    for c in catalog::circuits() {
        let track = c.load().unwrap();
        let errors: Vec<_> = validate_track(&track)
            .into_iter()
            .filter(|d| d.severity == ValidationSeverity::Error)
            .collect();
        assert!(errors.is_empty(), "{}/{}: {:?}", c.module, c.id, errors);
    }
}

#[test]
fn test_aliases_and_module_hint() {
    assert_eq!(catalog::canonical_id("daytona"), Some("daytona_superspeedway"));
    assert_eq!(catalog::canonical_id("monza"), Some("monza"));
    assert_eq!(catalog::canonical_id("no_such_circuit"), None);

    assert_eq!(catalog::find("dirt_figure_eight", Some("extreme_offroad")).unwrap().module, "extreme_offroad");
    assert_eq!(catalog::find("dirt_figure_eight", Some("classic")).unwrap().module, "classic");
    assert_eq!(catalog::find("singapore", None).unwrap().id, "marina_bay");

    let first_gt: Vec<&str> = catalog::module_circuits("gt").take(2).map(|c| c.id).collect();
    assert_eq!(first_gt, vec!["monza", "red_bull_ring"]);
    assert_eq!(catalog::official_track("gt", "monza").name, "Monza Autodromo Nazionale");
}
