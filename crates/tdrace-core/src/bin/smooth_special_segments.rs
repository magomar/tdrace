//! Smooths special segments (joker laps, launch chutes, pit lanes) across AX, RX, and GT circuits.
//!
//! Applies centripetal Catmull-Rom arc-fan filleting (Spec 097 Pillar V / VI) to acute interior vertices
//! of special segments while strictly preserving:
//! 1. Endpoint positions and tangent headings (junction split/merge sockets and timing gates).
//! 2. Waypoint spacing invariants (distance >= 3.0m).
//! 3. Cross-sectional corridor attributes (width, elevation, banking, wall distances, surface, runoff).

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use glam::Vec2;
use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};
use tdrace_core::track::{validate_track, Track, ValidationSeverity};

/// Smooths acute vertices in an open polyline of waypoints using circular arc fans.
pub fn smooth_special_segment_waypoints(
    wps: &[TrackWaypoint],
    target_radius: f32,
    min_deflection_deg: f32,
) -> Vec<TrackWaypoint> {
    let n = wps.len();
    if n < 3 {
        return wps.to_vec();
    }

    // Step 1: Pre-clean redundant or jitter vertices closer than 3.0m.
    // Endpoints (0 and n-1) are preserved as anchor sockets.
    let mut cleaned: Vec<TrackWaypoint> = Vec::with_capacity(n);
    cleaned.push(wps[0].clone());
    for i in 1..n - 1 {
        let p_prev = cleaned.last().unwrap().point;
        let p_curr = wps[i].point;
        if p_curr.distance(p_prev) >= 2.95 {
            cleaned.push(wps[i].clone());
        }
    }
    let p_last = wps[n - 1].point;
    if cleaned.len() > 2 && cleaned.last().unwrap().point.distance(p_last) < 2.95 {
        cleaned.pop();
    }
    cleaned.push(wps[n - 1].clone());

    let n_clean = cleaned.len();
    if n_clean < 3 {
        return cleaned;
    }
    let pts: Vec<Vec2> = cleaned.iter().map(|w| w.point).collect();

    // Step 2: Identify acute interior vertices (1..n_clean - 1)
    struct Kink {
        idx: usize,
        defl_rad: f32,
        is_left: bool,
        u: Vec2,
        v: Vec2,
        la: f32,
        lb: f32,
    }
    let mut kinks = Vec::new();
    for i in 1..n_clean - 1 {
        let p_prev = pts[i - 1];
        let p_curr = pts[i];
        let p_next = pts[i + 1];

        let u_vec = p_curr - p_prev;
        let v_vec = p_next - p_curr;
        let la = u_vec.length();
        let lb = v_vec.length();
        if la < 3.0 || lb < 3.0 {
            continue;
        }
        let u = u_vec / la;
        let v = v_vec / lb;

        let dot = (u.dot(v)).clamp(-1.0, 1.0);
        let defl_rad = dot.acos();
        let deg = defl_rad.to_degrees();
        let cross = u.x * v.y - u.y * v.x;

        if deg >= min_deflection_deg && cross.abs() > 1e-4 {
            kinks.push(Kink {
                idx: i,
                defl_rad,
                is_left: cross > 0.0,
                u,
                v,
                la,
                lb,
            });
        }
    }

    if kinks.is_empty() {
        return cleaned;
    }

    // Step 3: Compute arc fan replacements
    let mut replacements: std::collections::HashMap<usize, Vec<TrackWaypoint>> = std::collections::HashMap::new();

    for kink in kinks {
        let half_defl = kink.defl_rad * 0.5;
        let tan_half = half_defl.tan().max(1e-4);
        let t_des = target_radius * tan_half;

        // Keep at least 2.95m clearance to adjacent waypoints, and don't take more than 45% of the leg
        let t_max = (kink.la - 2.95).min(kink.lb - 2.95).max(0.0).min(0.45 * kink.la).min(0.45 * kink.lb);
        let t = t_des.min(t_max);
        if t < 0.5 {
            continue;
        }

        let r_act = t / tan_half;
        let norm_in = if kink.is_left {
            Vec2::new(-kink.u.y, kink.u.x)
        } else {
            Vec2::new(kink.u.y, -kink.u.x)
        };

        let p_curr = pts[kink.idx];
        let p_entry = p_curr - kink.u * t;
        let p_exit = p_curr + kink.v * t;
        let center = p_entry + norm_in * r_act;

        let a_in = (p_entry.y - center.y).atan2(p_entry.x - center.x);
        let a_out = (p_exit.y - center.y).atan2(p_exit.x - center.x);
        let mut da = a_out - a_in;
        if kink.is_left && da < 0.0 {
            da += std::f32::consts::TAU;
        } else if !kink.is_left && da > 0.0 {
            da -= std::f32::consts::TAU;
        }

        let arc_len = (r_act * da).abs();
        let chord = p_entry.distance(p_exit);

        let w_curr = &cleaned[kink.idx];

        if arc_len >= 12.0 {
            // 5-point fan (4 segments)
            let mut fan = Vec::with_capacity(5);
            for s in 0..=4 {
                let frac = s as f32 / 4.0;
                let ang = a_in + frac * da;
                let pt = center + Vec2::new(ang.cos(), ang.sin()) * r_act;
                let mut wp = w_curr.clone();
                wp.point = Vec2::new((pt.x * 100.0).round() / 100.0, (pt.y * 100.0).round() / 100.0);
                wp.left_curb = false;
                wp.right_curb = false;
                fan.push(wp);
            }
            replacements.insert(kink.idx, fan);
        } else if arc_len >= 5.9 {
            // 3-point fan (Entry, Apex, Exit)
            let mut fan = Vec::with_capacity(3);
            for s in 0..=2 {
                let frac = s as f32 / 2.0;
                let ang = a_in + frac * da;
                let pt = center + Vec2::new(ang.cos(), ang.sin()) * r_act;
                let mut wp = w_curr.clone();
                wp.point = Vec2::new((pt.x * 100.0).round() / 100.0, (pt.y * 100.0).round() / 100.0);
                wp.left_curb = false;
                wp.right_curb = false;
                fan.push(wp);
            }
            replacements.insert(kink.idx, fan);
        } else if chord >= 2.95 {
            // Chamfer (Entry, Exit)
            let mut fan = Vec::with_capacity(2);
            for &pt in &[p_entry, p_exit] {
                let mut wp = w_curr.clone();
                wp.point = Vec2::new((pt.x * 100.0).round() / 100.0, (pt.y * 100.0).round() / 100.0);
                wp.left_curb = false;
                wp.right_curb = false;
                fan.push(wp);
            }
            replacements.insert(kink.idx, fan);
        } else {
            // Single smoothed apex point
            let ang = a_in + 0.5 * da;
            let pt = center + Vec2::new(ang.cos(), ang.sin()) * r_act;
            let mut wp = w_curr.clone();
            wp.point = Vec2::new((pt.x * 100.0).round() / 100.0, (pt.y * 100.0).round() / 100.0);
            wp.left_curb = false;
            wp.right_curb = false;
            replacements.insert(kink.idx, vec![wp]);
        }
    }

    let mut result = Vec::new();
    for i in 0..n_clean {
        if let Some(fan) = replacements.remove(&i) {
            result.extend(fan);
        } else {
            result.push(cleaned[i].clone());
        }
    }

    // Step 4: Final verification that all consecutive waypoints are >= 2.95m
    let mut final_wps = Vec::with_capacity(result.len());
    final_wps.push(result[0].clone());
    for i in 1..result.len() - 1 {
        let p_prev = final_wps.last().unwrap().point;
        let p_curr = result[i].point;
        if p_curr.distance(p_prev) >= 2.95 {
            final_wps.push(result[i].clone());
        }
    }
    let p_last = result.last().unwrap().point;
    if final_wps.len() > 2 && final_wps.last().unwrap().point.distance(p_last) < 2.95 {
        final_wps.pop();
    }
    final_wps.push(result.last().unwrap().clone());

    final_wps
}

fn process_circuit_file(path: &Path, dry_run: bool) -> Result<bool, String> {
    let mut track = Track::load_from_file(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    let mut modified = false;

    // 1. Process Network Segments (Launch Chutes and Joker Laps)
    if let Some(ref mut net) = track.network {
        let seg_ids: Vec<_> = net.segments.iter().map(|s| s.id).collect();
        for seg_id in seg_ids {
            let (is_chute, is_joker, seg_name) = {
                let seg = net.get_segment(seg_id).unwrap();
                let name_lower = seg.name.to_lowercase();
                let is_c = name_lower.contains("chute") || name_lower.contains("launch");
                let is_j = name_lower.contains("joker");
                (is_c, is_j, seg.name.clone())
            };

            if is_chute || is_joker {
                let (target_r, min_deg) = if is_joker {
                    (12.0, 18.0) // Joker Detour
                } else {
                    (14.0, 18.0) // Launch Chute
                };

                let old_wps = net.get_segment(seg_id).unwrap().waypoints.clone();
                let new_wps = smooth_special_segment_waypoints(&old_wps, target_r, min_deg);

                if new_wps != old_wps {
                    println!(
                        "  [{}] smoothed segment '{}': {} -> {} waypoints",
                        path.file_name().unwrap().to_string_lossy(),
                        seg_name,
                        old_wps.len(),
                        new_wps.len()
                    );
                    net.get_segment_mut(seg_id).unwrap().waypoints = new_wps;
                    net.rebuild_segment_samples(seg_id);
                    modified = true;
                }
            }
        }

        if modified {
            net.recompute_composite_splines();
        }
    }

    // 2. Process GT Pit Lanes (Free-form pit lanes)
    if let Some(ref mut lane) = track.pit_lane {
        if track.pit_lane_layout.is_none() {
            let old_wps = lane.spline.waypoints.clone();
            let new_wps = smooth_special_segment_waypoints(&old_wps, 16.0, 15.0);

            if new_wps != old_wps {
                println!(
                    "  [{}] smoothed pit_lane: {} -> {} waypoints",
                    path.file_name().unwrap().to_string_lossy(),
                    old_wps.len(),
                    new_wps.len()
                );
                lane.spline = TrackSpline::new(new_wps, false);
                modified = true;
            }
        }
    }

    if !modified {
        return Ok(false);
    }

    // Ensure runtime geometry and validate
    track.ensure_runtime_geometry();

    let errors: Vec<_> = validate_track(&track)
        .into_iter()
        .filter(|d| d.severity == ValidationSeverity::Error)
        .collect();

    if !errors.is_empty() {
        let err_strs: Vec<_> = errors.iter().map(|e| format!("{}: {}", e.code, e.message)).collect();
        return Err(format!(
            "Validation errors on {}:\n  {}",
            path.display(),
            err_strs.join("\n  ")
        ));
    }

    if !dry_run {
        track.save_to_file(path).map_err(|e| format!("{}: save failed: {}", path.display(), e))?;
    }

    Ok(true)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run");

    let tracks_dir = args
        .iter()
        .position(|a| a == "--tracks-dir")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("tracks"));

    println!(
        "=== Spline Smart Smoothing on Special Segments (dry_run: {}) ===",
        dry_run
    );

    let subdirs = ["autocross", "rally", "gt", "classic"];

    let mut total_files = 0;
    let mut modified_files = 0;
    let mut failed = false;

    for sub in &subdirs {
        let dir = tracks_dir.join(sub);
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let mut paths = Vec::new();
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().map_or(false, |ext| ext == "json") {
                paths.push(p);
            }
        }
        paths.sort();

        for path in paths {
            total_files += 1;
            match process_circuit_file(&path, dry_run) {
                Ok(true) => modified_files += 1,
                Ok(false) => {},
                Err(e) => {
                    eprintln!("ERROR: {}", e);
                    failed = true;
                }
            }
        }
    }

    println!(
        "\nComplete: {} files scanned, {} modified. Status: {}",
        total_files,
        modified_files,
        if failed { "FAILED" } else { "SUCCESS" }
    );

    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
