use tdrace_core::track::validation::{validate_track, TrackValidationError, ValidationSeverity};
use tdrace_core::track::Track;


/// Serializes a track to formatted JSON string suitable for preset files or sharing.
pub fn export_track_to_json(track: &Track) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(track)
}

/// Validates all official circuits in the embedded catalog, returning any errors or warnings.
pub fn validate_all_official_presets() -> Vec<(String, Vec<TrackValidationError>)> {
    let mut results = Vec::new();
    for circuit in tdrace_core::catalog::circuits() {
        let track = match circuit.load() {
            Ok(track) => track,
            Err(e) => {
                results.push((
                    format!("{}:{}", circuit.module, circuit.id),
                    vec![TrackValidationError {
                        severity: ValidationSeverity::Error,
                        code: "LOAD_FAILED",
                        message: e.to_string(),
                        details: None,
                        entity_index: None,
                    }],
                ));
                continue;
            }
        };
        let errors_or_warnings: Vec<_> = validate_track(&track)
            .into_iter()
            .filter(|d| d.severity == ValidationSeverity::Error || d.severity == ValidationSeverity::Warning)
            .collect();
        if !errors_or_warnings.is_empty() {
            results.push((format!("{}:{}", circuit.module, circuit.id), errors_or_warnings));
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dev_tools_export_to_json() {
        let gp = tdrace_core::catalog::official_track("classic", "gt_coastal_grand_prix");
        let json = export_track_to_json(&gp).expect("Must serialize track to JSON");
        assert!(json.contains("Coastal Grand Prix"));
    }
}
