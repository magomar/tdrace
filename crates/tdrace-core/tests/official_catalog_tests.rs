//! Embedded official catalog tests (specs/042_jsononly_official_circuit_catalog_and_embedded_track_data.md).

use std::path::Path;

use tdrace_core::catalog;
use tdrace_core::track::{validate_track, Track, ValidationSeverity};

const MODULES: [&str; 8] = ["classic", "extreme_offroad", "gt", "kart", "nascar", "rally", "autocross", "vault"];

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
    let classic_count = catalog::module_circuits("classic").count();
    let vault_count = catalog::module_circuits("vault").count();
    assert_eq!(
        counts,
        vec![classic_count, 20, 18, 20, 17, 20, 17, vault_count],
        "circuits per module {:?}",
        MODULES
    );
    assert_eq!(catalog::circuits().len(), classic_count + 112 + vault_count);
}

#[test]
fn test_no_embedded_circuit_has_validation_errors() {
    let mut failures = Vec::new();
    for c in catalog::circuits() {
        let track = c.load().unwrap();
        let errors: Vec<_> = validate_track(&track)
            .into_iter()
            .filter(|d| d.severity == ValidationSeverity::Error)
            .collect();
        if !errors.is_empty() {
            eprintln!("FAILING TRACK: {}/{}", c.module, c.id);
            for err in &errors {
                eprintln!("   {:?}: {}", err.code, err.message);
            }
            failures.push((c.module, c.id, errors));
        }
    }
    assert!(failures.is_empty(), "{} circuits have validation errors: {:?}", failures.len(), failures.iter().map(|(m, id, _)| format!("{}/{}", m, id)).collect::<Vec<_>>());
}

#[test]
fn test_aliases_and_module_hint() {
    assert_eq!(catalog::canonical_id("daytona"), Some("daytona_superspeedway"));
    assert_eq!(catalog::canonical_id("monza"), Some("monza"));
    assert_eq!(catalog::canonical_id("no_such_circuit"), None);

    assert_eq!(catalog::find("dirt_figure_eight", Some("extreme_offroad")).unwrap().module, "extreme_offroad");
    assert_eq!(catalog::find("classic_grand_prix", None).unwrap().id, "gt_coastal_grand_prix");
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

/// A rebuild must be a fixed point: baking the rebuilt circuit again changes nothing. Guards against bake steps
/// that read the old geometry back into the new one (869b0b86 re-derived the wall offset from the previous walls
/// on every bake, so each re-import moved the walls again) and against rebuilds that drop network checkpoints.
#[test]
fn test_rebuild_is_idempotent_on_every_osm_circuit() {
    use tdrace_core::track::bake::{bake, BakeOptions};
    let opts = BakeOptions { rebuild: true, ..Default::default() };
    let mut failures = Vec::new();
    for c in catalog::circuits().iter().filter(|c| ["gt", "kart", "nascar", "rally", "autocross"].contains(&c.module)) {
        let mut once = c.load().unwrap();
        bake(&mut once, &opts).unwrap_or_else(|e| panic!("{}/{}: {}", c.module, c.id, e));
        let mut twice = once.clone();
        bake(&mut twice, &opts).unwrap_or_else(|e| panic!("{}/{}: {}", c.module, c.id, e));
        let (a, b) = (serde_json::to_value(&twice).unwrap(), serde_json::to_value(&once).unwrap());
        if let Err(e) = json_close(&a, &b, "") {
            failures.push(format!("{}/{}: {}", c.module, c.id, e));
        }
    }
    assert!(failures.is_empty(), "{:#?}", failures);
}

/// `track_bake --rebuild` must reproduce every OSM-built circuit from its waypoints and current setup, so a
/// re-import cannot silently move walls, checkpoints or the grid (spec 042 §2.7).
#[test]
fn test_rebuild_reproduces_every_osm_circuit() {
    use tdrace_core::track::bake::{bake, BakeOptions};
    let opts = BakeOptions { rebuild: true, ..Default::default() };
    let mut failures = Vec::new();
    for c in catalog::circuits().iter().filter(|c| ["gt", "kart", "nascar", "rally", "autocross"].contains(&c.module)) {
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
