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
    assert_eq!(counts, vec![13, 20, 18, 17, 17, 17], "circuits per module {:?}", MODULES);
    assert_eq!(catalog::circuits().len(), 102);
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

/// Relative float tolerance for comparing a re-baked circuit with the original.
fn json_close(a: &serde_json::Value, b: &serde_json::Value, path: &str) -> Result<(), String> {
    use serde_json::Value;
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            for (k, va) in x {
                let vb = y.get(k).ok_or_else(|| format!("{}.{} missing", path, k))?;
                json_close(va, vb, &format!("{}.{}", path, k))?;
            }
            if y.len() != x.len() {
                return Err(format!("{}: key count {} vs {}", path, x.len(), y.len()));
            }
            Ok(())
        }
        (Value::Array(x), Value::Array(y)) => {
            if x.len() != y.len() {
                return Err(format!("{}: len {} vs {}", path, x.len(), y.len()));
            }
            x.iter().zip(y).enumerate().try_for_each(|(i, (va, vb))| json_close(va, vb, &format!("{}[{}]", path, i)))
        }
        (Value::Number(x), Value::Number(y)) => {
            let (x, y) = (x.as_f64().unwrap(), y.as_f64().unwrap());
            if (x - y).abs() > 1e-3 * x.abs().max(y.abs()).max(1.0) {
                return Err(format!("{}: {} vs {}", path, x, y));
            }
            Ok(())
        }
        _ if a == b => Ok(()),
        _ => Err(format!("{}: {} vs {}", path, a, b)),
    }
}

/// `track_bake --rebuild` must reproduce every OSM-built circuit from its waypoints and current setup, so a
/// re-import cannot silently move walls, checkpoints or the grid (spec 042 §2.7).
#[test]
fn test_rebuild_reproduces_every_osm_circuit() {
    use tdrace_core::track::bake::{bake, BakeOptions};
    let opts = BakeOptions { rebuild: true, ..Default::default() };
    let mut failures = Vec::new();
    for c in catalog::circuits().iter().filter(|c| ["gt", "kart", "nascar", "rally"].contains(&c.module)) {
        let original = c.load().unwrap();
        let mut rebuilt = original.clone();
        bake(&mut rebuilt, &opts).unwrap_or_else(|e| panic!("{}/{}: {}", c.module, c.id, e));
        let (a, b) = (serde_json::to_value(&rebuilt).unwrap(), serde_json::to_value(&original).unwrap());
        if let Err(e) = json_close(&a, &b, "") {
            failures.push(format!("{}/{}: {}", c.module, c.id, e));
        }
    }
    assert!(failures.is_empty(), "{:#?}", failures);
}
