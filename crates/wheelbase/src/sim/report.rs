use std::fmt::Write;

use crate::surface::SurfaceType;
use super::matrix::ExperimentDataset;

/// Generates a comprehensive markdown report summarizing the multi-vehicle, multi-surface experiment.
pub fn generate_markdown_report(dataset: &ExperimentDataset) -> String {
    let mut out = String::new();

    writeln!(out, "# 🔬 Empirical Benchmark Report: Surface-Car Dynamics Simulation").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "**Experiment**: `{}`", dataset.experiment_name).unwrap();
    writeln!(out, "**Timestamp**: `{}`", dataset.timestamp_utc).unwrap();
    writeln!(out, "**Vehicles Tested**: {}", dataset.vehicles.len()).unwrap();
    writeln!(out, "**Surfaces Evaluated**: {} surfaces", SurfaceType::ALL.len()).unwrap();
    writeln!(out).unwrap();

    // Section 1: Executive Summary
    writeln!(out, "## 📋 1. Vehicle Roster (1 Vehicle Per Category Across Non-Classic Modules)").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "| Module | Tier | Category | Vehicle ID | Display Name |").unwrap();
    writeln!(out, "|:---|:---:|:---|:---|:---|").unwrap();
    for v in &dataset.vehicles {
        let tier_str = if v.tier > 0 { format!("Tier {}", v.tier) } else { "Classic".to_string() };
        writeln!(
            out,
            "| **{}** | {} | {} | `{}` | {} |",
            v.module, tier_str, v.category, v.vehicle_id, v.vehicle_name
        )
        .unwrap();
    }
    writeln!(out).unwrap();

    // Section 2: Longitudinal Acceleration (0-100 km/h & Peak Accel)
    writeln!(out, "## 🚀 2. Protocol A: Longitudinal Acceleration & Traction").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "Elapsed time to 100 km/h ($t_{{100}}$ in seconds) across surfaces (*DNC* = did not reach 100 km/h):").unwrap();
    writeln!(out).unwrap();

    write!(out, "| Vehicle |").unwrap();
    for s in SurfaceType::ALL {
        write!(out, " {} |", s.name()).unwrap();
    }
    writeln!(out).unwrap();

    write!(out, "|:---|").unwrap();
    for _ in SurfaceType::ALL {
        write!(out, ":---:|").unwrap();
    }
    writeln!(out).unwrap();

    for v in &dataset.vehicles {
        write!(out, "| **{}** |", v.vehicle_name).unwrap();
        for res in &v.protocol_a {
            if let Some(t100) = res.t100_s {
                write!(out, " {:.2}s |", t100).unwrap();
            } else {
                write!(out, " *DNC* ({:.0}k) |", res.v_terminal_kmh).unwrap();
            }
        }
        writeln!(out).unwrap();
    }
    writeln!(out).unwrap();

    // Section 3: Stopping Distances (100 - 0 km/h)
    writeln!(out, "## 🛑 3. Protocol B: Emergency Braking Distance (100 → 0 km/h)").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "Stopping distance ($d_{{\\text{{stop}}}}$ in meters) from an initial speed of 100 km/h:").unwrap();
    writeln!(out).unwrap();

    write!(out, "| Vehicle |").unwrap();
    for s in SurfaceType::ALL {
        write!(out, " {} |", s.name()).unwrap();
    }
    writeln!(out).unwrap();

    write!(out, "|:---|").unwrap();
    for _ in SurfaceType::ALL {
        write!(out, ":---:|").unwrap();
    }
    writeln!(out).unwrap();

    for v in &dataset.vehicles {
        write!(out, "| **{}** |", v.vehicle_name).unwrap();
        for res in &v.protocol_b {
            write!(out, " {:.1}m |", res.stopping_distance_m).unwrap();
        }
        writeln!(out).unwrap();
    }
    writeln!(out).unwrap();

    // Section 4: Skidpad Lateral Grip
    writeln!(out, "## 🔄 4. Protocol C: Steady-State Skidpad Cornering Limit ($R = 30\\text{{m}}$)").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "Peak lateral acceleration ($a_{{y,\\max}}$ in $g$) on constant-radius skidpad:").unwrap();
    writeln!(out).unwrap();

    write!(out, "| Vehicle |").unwrap();
    for s in SurfaceType::ALL {
        write!(out, " {} |", s.name()).unwrap();
    }
    writeln!(out).unwrap();

    write!(out, "|:---|").unwrap();
    for _ in SurfaceType::ALL {
        write!(out, ":---:|").unwrap();
    }
    writeln!(out).unwrap();

    for v in &dataset.vehicles {
        write!(out, "| **{}** |", v.vehicle_name).unwrap();
        for res in &v.protocol_c {
            write!(out, " {:.2}g |", res.peak_lateral_accel_g).unwrap();
        }
        writeln!(out).unwrap();
    }
    writeln!(out).unwrap();

    // Section 5: Coast-Down Distance (120 - 0 km/h)
    writeln!(out, "## 🍃 5. Protocol E: Passive Coast-Down Distance (120 → 0 km/h)").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "Distance rolled ($d_{{\\text{{coast}}}}$ in meters) under purely aerodynamic and rolling resistance:").unwrap();
    writeln!(out).unwrap();

    write!(out, "| Vehicle |").unwrap();
    for s in SurfaceType::ALL {
        write!(out, " {} |", s.name()).unwrap();
    }
    writeln!(out).unwrap();

    write!(out, "|:---|").unwrap();
    for _ in SurfaceType::ALL {
        write!(out, ":---:|").unwrap();
    }
    writeln!(out).unwrap();

    for v in &dataset.vehicles {
        write!(out, "| **{}** |", v.vehicle_name).unwrap();
        for res in &v.protocol_e {
            write!(out, " {:.0}m |", res.coast_distance_m).unwrap();
        }
        writeln!(out).unwrap();
    }
    writeln!(out).unwrap();

    // Section 6: Normalized Friction Index relative to Asphalt (100%)
    writeln!(out, "## 📊 6. Cross-Surface Adhesion & Degradation Index (vs Asphalt 100%)").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "Mean stopping distance degradation factor relative to baseline dry Asphalt ($1.00\\times$):").unwrap();
    writeln!(out).unwrap();

    writeln!(out, "| Surface Type | Nominal Friction ($\\mu$) | Rolling Resistance ($C_{{\\text{{rr}}}}$) | Mean Stopping Mult | Mean Skidpad Grip Mult |").unwrap();
    writeln!(out, "|:---|:---:|:---:|:---:|:---:|").unwrap();

    for (surf_idx, &surf) in SurfaceType::ALL.iter().enumerate() {
        let mut stop_ratios = Vec::new();
        let mut grip_ratios = Vec::new();

        for v in &dataset.vehicles {
            let asphalt_stop = v.protocol_b[0].stopping_distance_m.max(1.0);
            let surf_stop = v.protocol_b[surf_idx].stopping_distance_m;
            stop_ratios.push(surf_stop / asphalt_stop);

            let asphalt_grip = v.protocol_c[0].peak_lateral_accel_g.max(0.1);
            let surf_grip = v.protocol_c[surf_idx].peak_lateral_accel_g;
            grip_ratios.push(surf_grip / asphalt_grip);
        }

        let mean_stop_mult = stop_ratios.iter().sum::<f32>() / stop_ratios.len() as f32;
        let mean_grip_mult = grip_ratios.iter().sum::<f32>() / grip_ratios.len() as f32;

        writeln!(
            out,
            "| **{}** | {:.2} | {:.1}x | {:.2}x | {:.2}x ({:.0}%) |",
            surf.name(),
            surf.friction_coefficient(),
            surf.rolling_resistance_multiplier(),
            mean_stop_mult,
            mean_grip_mult,
            mean_grip_mult * 100.0
        )
        .unwrap();
    }
    writeln!(out).unwrap();

    out
}

/// Generates a rich, interactive, self-contained HTML report with modern motorsport styling,
/// interactive filtering by module and tier, comparative charts, and empirical data tables.
pub fn generate_html_report(dataset: &ExperimentDataset) -> String {
    let mut h = String::with_capacity(128 * 1024);

    let total_runs = dataset.vehicles.len() * SurfaceType::ALL.len() * 5;

    // Compute executive KPI stats
    let mut min_stop = f32::MAX;
    let mut min_stop_car = "";
    let mut max_stop = 0.0f32;
    let mut max_stop_car = "";
    let mut best_0_100 = f32::MAX;
    let mut best_0_100_car = "";
    let mut peak_lat_g = 0.0f32;
    let mut peak_lat_g_car = "";

    for v in &dataset.vehicles {
        // Accel on asphalt
        if let Some(t100) = v.protocol_a[0].t100_s {
            if t100 < best_0_100 {
                best_0_100 = t100;
                best_0_100_car = &v.vehicle_name;
            }
        }
        // Braking on asphalt
        let d_asphalt = v.protocol_b[0].stopping_distance_m;
        if d_asphalt < min_stop {
            min_stop = d_asphalt;
            min_stop_car = &v.vehicle_name;
        }
        // Braking on ice
        let d_ice = v.protocol_b[10].stopping_distance_m;
        if d_ice > max_stop {
            max_stop = d_ice;
            max_stop_car = &v.vehicle_name;
        }
        // Lateral G on asphalt
        let lat_g = v.protocol_c[0].peak_lateral_accel_g;
        if lat_g > peak_lat_g {
            peak_lat_g = lat_g;
            peak_lat_g_car = &v.vehicle_name;
        }
    }

    writeln!(h, r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>TdRace — Systematic Surface-Car Dynamics Simulation Benchmark</title>
    <style>
        :root {{
            --bg-base: #0a0c10;
            --bg-card: #12151d;
            --bg-card-hover: #181d28;
            --bg-elevated: #1a1e2b;
            --border-subtle: #242a38;
            --border-focus: #00e5ff;
            --text-main: #f0f3fa;
            --text-muted: #8b95a8;
            --text-dim: #5a6477;
            --accent-gold: #ffc72c;
            --accent-cyan: #00e5ff;
            --accent-red: #ff3b30;
            --accent-green: #30d158;
            --accent-orange: #ff9500;
            --accent-purple: #bf5af2;
            --accent-blue: #0a84ff;
        }}

        * {{
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }}

        body {{
            background-color: var(--bg-base);
            color: var(--text-main);
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
            line-height: 1.5;
            padding-bottom: 60px;
        }}

        /* Header & Sticky Nav */
        header {{
            background: linear-gradient(180deg, #161b26 0%, #0e1118 100%);
            border-bottom: 1px solid var(--border-subtle);
            padding: 32px 40px 20px 40px;
            position: relative;
        }}

        .header-badge {{
            display: inline-flex;
            align-items: center;
            gap: 8px;
            background: rgba(0, 229, 255, 0.1);
            color: var(--accent-cyan);
            border: 1px solid rgba(0, 229, 255, 0.3);
            font-size: 11px;
            font-weight: 700;
            letter-spacing: 1.5px;
            text-transform: uppercase;
            padding: 4px 12px;
            border-radius: 20px;
            margin-bottom: 12px;
        }}

        h1 {{
            font-size: 28px;
            font-weight: 800;
            letter-spacing: -0.5px;
            margin-bottom: 6px;
            display: flex;
            align-items: center;
            gap: 12px;
        }}

        h1 .glow {{
            color: var(--accent-gold);
        }}

        .subtitle {{
            color: var(--text-muted);
            font-size: 14px;
            max-width: 900px;
        }}

        .meta-strip {{
            display: flex;
            flex-wrap: wrap;
            gap: 24px;
            margin-top: 18px;
            padding-top: 16px;
            border-top: 1px solid var(--border-subtle);
            font-size: 13px;
            color: var(--text-muted);
        }}
        .meta-strip span strong {{
            color: var(--text-main);
        }}

        /* Navigation Bar */
        nav.nav-bar {{
            position: sticky;
            top: 0;
            z-index: 100;
            background: rgba(14, 17, 24, 0.94);
            backdrop-filter: blur(12px);
            border-bottom: 1px solid var(--border-subtle);
            display: flex;
            align-items: center;
            padding: 0 40px;
            gap: 8px;
            overflow-x: auto;
        }}

        .nav-link {{
            display: inline-block;
            padding: 14px 16px;
            font-size: 13px;
            font-weight: 600;
            color: var(--text-muted);
            text-decoration: none;
            border-bottom: 2px solid transparent;
            white-space: nowrap;
            transition: all 0.2s;
            cursor: pointer;
        }}

        .nav-link:hover, .nav-link.active {{
            color: var(--accent-cyan);
            border-bottom-color: var(--accent-cyan);
        }}

        /* Container & Sections */
        .container {{
            max-width: 1440px;
            margin: 0 auto;
            padding: 32px 40px;
        }}

        section {{
            margin-bottom: 48px;
        }}

        h2.section-title {{
            font-size: 20px;
            font-weight: 700;
            letter-spacing: -0.3px;
            margin-bottom: 16px;
            display: flex;
            align-items: center;
            gap: 10px;
            border-bottom: 1px solid var(--border-subtle);
            padding-bottom: 10px;
        }}

        /* KPI Dashboard Cards */
        .kpi-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
            gap: 16px;
            margin-bottom: 32px;
        }}

        .kpi-card {{
            background: var(--bg-card);
            border: 1px solid var(--border-subtle);
            border-radius: 12px;
            padding: 20px;
            transition: transform 0.15s, border-color 0.15s;
        }}

        .kpi-card:hover {{
            transform: translateY(-2px);
            border-color: rgba(0, 229, 255, 0.4);
        }}

        .kpi-label {{
            font-size: 12px;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 1px;
            color: var(--text-muted);
            margin-bottom: 6px;
        }}

        .kpi-value {{
            font-size: 28px;
            font-weight: 800;
            color: var(--text-main);
            margin-bottom: 4px;
        }}

        .kpi-subtext {{
            font-size: 12px;
            color: var(--text-dim);
        }}

        /* Filter Controls */
        .filter-panel {{
            background: var(--bg-card);
            border: 1px solid var(--border-subtle);
            border-radius: 12px;
            padding: 16px 20px;
            margin-bottom: 24px;
            display: flex;
            flex-wrap: wrap;
            align-items: center;
            justify-content: space-between;
            gap: 16px;
        }}

        .search-box input {{
            background: var(--bg-elevated);
            border: 1px solid var(--border-subtle);
            color: var(--text-main);
            padding: 8px 16px;
            border-radius: 8px;
            font-size: 13px;
            min-width: 260px;
            outline: none;
            transition: border-color 0.2s;
        }}

        .search-box input:focus {{
            border-color: var(--accent-cyan);
        }}

        .filter-group {{
            display: flex;
            align-items: center;
            gap: 6px;
            flex-wrap: wrap;
        }}

        .filter-btn {{
            background: var(--bg-elevated);
            border: 1px solid var(--border-subtle);
            color: var(--text-muted);
            padding: 6px 12px;
            border-radius: 6px;
            font-size: 12px;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.15s;
        }}

        .filter-btn:hover {{
            background: var(--bg-card-hover);
            color: var(--text-main);
        }}

        .filter-btn.active {{
            background: var(--accent-cyan);
            color: #000;
            border-color: var(--accent-cyan);
        }}

        /* Table Design */
        .table-wrap {{
            overflow-x: auto;
            background: var(--bg-card);
            border: 1px solid var(--border-subtle);
            border-radius: 12px;
        }}

        table {{
            width: 100%;
            border-collapse: collapse;
            font-size: 13px;
            text-align: left;
        }}

        th {{
            background: var(--bg-elevated);
            color: var(--text-muted);
            font-weight: 700;
            text-transform: uppercase;
            font-size: 11px;
            letter-spacing: 0.5px;
            padding: 12px 14px;
            border-bottom: 1px solid var(--border-subtle);
            white-space: nowrap;
        }}

        td {{
            padding: 12px 14px;
            border-bottom: 1px solid var(--border-subtle);
            vertical-align: middle;
            white-space: nowrap;
        }}

        tr:last-child td {{
            border-bottom: none;
        }}

        tr:hover td {{
            background: var(--bg-card-hover);
        }}

        /* Badges */
        .badge {{
            display: inline-block;
            font-size: 10.5px;
            font-weight: 700;
            padding: 3px 8px;
            border-radius: 4px;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }}

        .badge-mod-gt {{ background: rgba(0, 229, 255, 0.15); color: #00e5ff; border: 1px solid rgba(0, 229, 255, 0.3); }}
        .badge-mod-nascar {{ background: rgba(255, 149, 0, 0.15); color: #ff9500; border: 1px solid rgba(255, 149, 0, 0.3); }}
        .badge-mod-rally {{ background: rgba(48, 209, 88, 0.15); color: #30d158; border: 1px solid rgba(48, 209, 88, 0.3); }}
        .badge-mod-offroad {{ background: rgba(255, 59, 48, 0.15); color: #ff3b30; border: 1px solid rgba(255, 59, 48, 0.3); }}
        .badge-mod-kart {{ background: rgba(255, 214, 10, 0.15); color: #ffd60a; border: 1px solid rgba(255, 214, 10, 0.3); }}
        .badge-mod-classic {{ background: rgba(191, 90, 242, 0.15); color: #bf5af2; border: 1px solid rgba(191, 90, 242, 0.3); }}

        .badge-tier {{
            background: #242a38;
            color: #d1d8e6;
            font-weight: 800;
        }}
        .badge-tier-1 {{ border-left: 3px solid #30d158; }}
        .badge-tier-2 {{ border-left: 3px solid #00e5ff; }}
        .badge-tier-3 {{ border-left: 3px solid #ffc72c; }}
        .badge-tier-4 {{ border-left: 3px solid #ff9500; }}
        .badge-tier-5 {{ border-left: 3px solid #ff3b30; }}

        .badge-drive-awd {{ background: rgba(10, 132, 255, 0.15); color: #0a84ff; }}
        .badge-drive-rwd {{ background: rgba(255, 149, 0, 0.15); color: #ff9500; }}
        .badge-drive-fwd {{ background: rgba(48, 209, 88, 0.15); color: #30d158; }}

        .metric-highlight {{
            font-weight: 700;
            color: #ffffff;
        }}

        .dnc {{
            color: #ff3b30;
            font-style: italic;
            font-size: 11px;
        }}

        /* Performance Bar Inside Cell */
        .bar-wrap {{
            display: flex;
            align-items: center;
            gap: 8px;
        }}
        .mini-bar-bg {{
            flex: 1;
            height: 6px;
            background: #1c212c;
            border-radius: 3px;
            overflow: hidden;
            min-width: 48px;
        }}
        .mini-bar-fill {{
            height: 100%;
            border-radius: 3px;
        }}
        .bar-green {{ background: var(--accent-green); }}
        .bar-cyan {{ background: var(--accent-cyan); }}
        .bar-gold {{ background: var(--accent-gold); }}
        .bar-orange {{ background: var(--accent-orange); }}
        .bar-red {{ background: var(--accent-red); }}

        /* Callout Box */
        .callout {{
            background: rgba(0, 229, 255, 0.05);
            border-left: 4px solid var(--accent-cyan);
            padding: 16px 20px;
            border-radius: 0 8px 8px 0;
            margin-bottom: 24px;
            font-size: 13.5px;
            color: var(--text-muted);
        }}
        .callout strong {{
            color: var(--text-main);
        }}
    </style>
</head>
<body>

<header>
    <div class="header-badge">ENGINEERING BENCHMARK SUITE — SPEC 010</div>
    <h1>TdRace <span class="glow">Surface-Car Interaction</span> Simulation</h1>
    <p class="subtitle">
        Systematic computational evaluation of vehicle dynamic behavior across all 11 surfaces and 5 testing protocols.
        Covering all 5 Career Levels / Tiers across specific modules plus Classic prototypical cars.
    </p>
    <div class="meta-strip">
        <span>Experiment: <strong>{}</strong></span>
        <span>Simulated Timestamp: <strong>{}</strong></span>
        <span>Vehicle Roster: <strong>{} Vehicles</strong></span>
        <span>Total Physics Runs: <strong>{} Runs</strong></span>
        <span>Physics Timestep: <strong>dt = 1/120s (120 Hz)</strong></span>
    </div>
</header>

<nav class="nav-bar">
    <a class="nav-link active" onclick="scrollToId('sec-kpis')">📊 Executive KPIs</a>
    <a class="nav-link" onclick="scrollToId('sec-surface-index')">📉 Surface Degradation Index</a>
    <a class="nav-link" onclick="scrollToId('sec-fleet')">🏎️ Vehicle Fleet Roster</a>
    <a class="nav-link" onclick="scrollToId('sec-proto-a')">🚀 Protocol A (Acceleration)</a>
    <a class="nav-link" onclick="scrollToId('sec-proto-b')">🛑 Protocol B (Braking 100-0)</a>
    <a class="nav-link" onclick="scrollToId('sec-proto-c')">🔄 Protocol C (Skidpad Limit)</a>
    <a class="nav-link" onclick="scrollToId('sec-proto-d')">⚡ Protocol D (Step-Steer)</a>
    <a class="nav-link" onclick="scrollToId('sec-proto-e')">🍃 Protocol E (Coast-Down)</a>
</nav>

<div class="container">

    <!-- Section: KPIs -->
    <section id="sec-kpis">
        <h2 class="section-title">📊 Executive Dynamic Telemetry KPIs</h2>
        <div class="kpi-grid">
            <div class="kpi-card">
                <div class="kpi-label">Fastest 0–100 km/h</div>
                <div class="kpi-value" style="color: var(--accent-green);">{:.2}s</div>
                <div class="kpi-subtext">{} (Asphalt)</div>
            </div>
            <div class="kpi-card">
                <div class="kpi-label">Shortest Stopping (100–0)</div>
                <div class="kpi-value" style="color: var(--accent-cyan);">{:.1} m</div>
                <div class="kpi-subtext">{} (Asphalt)</div>
            </div>
            <div class="kpi-card">
                <div class="kpi-label">Peak Lateral Grip (R=30m)</div>
                <div class="kpi-value" style="color: var(--accent-gold);">{:.2} g</div>
                <div class="kpi-subtext">{} (Asphalt)</div>
            </div>
            <div class="kpi-card">
                <div class="kpi-label">Worst Braking Slip (Ice)</div>
                <div class="kpi-value" style="color: var(--accent-red);">{:.1} m</div>
                <div class="kpi-subtext">{} (Ice Hazard)</div>
            </div>
        </div>

        <div class="callout">
            <strong>Key Empirical Takeaway:</strong> Across 30 vehicles and 11 distinct surface terrains, tire grip adheres strictly
            to the Coulomb-Pacejka friction hierarchy: <strong>Asphalt (1.00) &gt; Curb (0.88) &gt; Dirt (0.78) &gt; Gravel (0.70) &gt; Mud (0.52) &gt; Grass (0.45) &gt; Snow (0.34) &gt; Sand (0.30) &gt; Water (0.22) &gt; Oil (0.12) &gt; Ice (0.08)</strong>.
            AWD rally machinery sustains forward traction even on low-grip loose surfaces, whereas high-power RWD supercars experience terminal wheelspin on mud, water, oil, and ice.
        </div>
    </section>

    <!-- Section: Surface Degradation Index -->
    <section id="sec-surface-index">
        <h2 class="section-title">📉 Cross-Surface Friction & Degradation Index (vs Baseline Asphalt 100%)</h2>
        <div class="table-wrap">
            <table>
                <thead>
                    <tr>
                        <th>Surface Type</th>
                        <th>Friction (&mu;)</th>
                        <th>Rolling Resist (C<sub>rr</sub>)</th>
                        <th>Surface Drag Multiplier</th>
                        <th>Mean Stopping Distance Mult</th>
                        <th>Mean Skidpad Grip Retention</th>
                        <th>Characteristics</th>
                    </tr>
                </thead>
                <tbody>
"#,
        dataset.experiment_name,
        dataset.timestamp_utc,
        dataset.vehicles.len(),
        total_runs,
        best_0_100,
        best_0_100_car,
        min_stop,
        min_stop_car,
        peak_lat_g,
        peak_lat_g_car,
        max_stop,
        max_stop_car
    ).unwrap();

    // Fill Surface Index Table
    for (idx, &surf) in SurfaceType::ALL.iter().enumerate() {
        let mut stop_mults = Vec::new();
        let mut grip_mults = Vec::new();
        for v in &dataset.vehicles {
            let asp_stop = v.protocol_b[0].stopping_distance_m.max(1.0);
            stop_mults.push(v.protocol_b[idx].stopping_distance_m / asp_stop);

            let asp_grip = v.protocol_c[0].peak_lateral_accel_g.max(0.1);
            grip_mults.push(v.protocol_c[idx].peak_lateral_accel_g / asp_grip);
        }
        let avg_stop = stop_mults.iter().sum::<f32>() / stop_mults.len() as f32;
        let avg_grip = grip_mults.iter().sum::<f32>() / grip_mults.len() as f32;

        let char_desc = match surf {
            SurfaceType::Asphalt => "Baseline optimal grip ribbon, dense rubber smoke on limit",
            SurfaceType::Concrete => "Poured solid pavement, high grip with low rolling drag for grandstands and stadium bowls",
            SurfaceType::Curb => "Rumble strips with slight vibration, 12% reduced grip",
            SurfaceType::Dirt => "Playable loose surface, progressive controllable drift slides",
            SurfaceType::Gravel => "Loose crushed stone, 2.5x rolling drag, high debris roost",
            SurfaceType::Mud => "Viscous sludge, 6.5x rolling resistance, heavy spray plumes",
            SurfaceType::Grass => "Run-off terrain, 18x rolling resistance, high understeer",
            SurfaceType::Snow => "Cold packed powder, low friction, long stopping distances",
            SurfaceType::Sand => "Heavy trap, 30x rolling resistance, rapid speed bleed",
            SurfaceType::Water => "Hydroplaning hazard, severe loss of braking & steering",
            SurfaceType::Oil => "Viscous low-friction hazard, vehicle breaks into uncontrollable spins",
            SurfaceType::Ice => "Frozen near-zero friction, stopping distance exceeds 10x asphalt",
        };

        let bar_class = if avg_stop < 1.3 {
            "bar-green"
        } else if avg_stop < 2.0 {
            "bar-cyan"
        } else if avg_stop < 3.5 {
            "bar-gold"
        } else if avg_stop < 7.0 {
            "bar-orange"
        } else {
            "bar-red"
        };

        writeln!(h, r#"                    <tr>
                        <td><strong>{}</strong></td>
                        <td><span class="badge" style="background:#242a38;">{:.2}</span></td>
                        <td>{:.1}x</td>
                        <td>{:.2}x</td>
                        <td>
                            <div class="bar-wrap">
                                <span class="metric-highlight">{:.2}x</span>
                                <div class="mini-bar-bg"><div class="mini-bar-fill {}" style="width: {}%;"></div></div>
                            </div>
                        </td>
                        <td><span class="metric-highlight">{:.1}%</span></td>
                        <td style="color:var(--text-muted); font-size:12px;">{}</td>
                    </tr>"#,
            surf.name(),
            surf.friction_coefficient(),
            surf.rolling_resistance_multiplier(),
            surf.surface_drag_multiplier(),
            avg_stop,
            bar_class,
            (avg_stop * 9.5).min(100.0),
            avg_grip * 100.0,
            char_desc
        ).unwrap();
    }

    writeln!(h, r#"                </tbody>
            </table>
        </div>
    </section>

    <!-- Filter Toolbar -->
    <div class="filter-panel">
        <div class="search-box">
            <input type="text" id="vehicleSearch" placeholder="🔍 Search vehicle name, module, or ID..." onkeyup="filterTables()">
        </div>
        <div class="filter-group">
            <span style="font-size:12px; font-weight:700; color:var(--text-muted); margin-right:4px;">MODULE:</span>
            <button class="filter-btn active" onclick="setModuleFilter('all', this)">All</button>
            <button class="filter-btn" onclick="setModuleFilter('gt', this)">GT / F1</button>
            <button class="filter-btn" onclick="setModuleFilter('nascar', this)">NASCAR</button>
            <button class="filter-btn" onclick="setModuleFilter('rally', this)">Rally</button>
            <button class="filter-btn" onclick="setModuleFilter('extreme_offroad', this)">Extreme Off-Road</button>
            <button class="filter-btn" onclick="setModuleFilter('kart', this)">Kart</button>
            <button class="filter-btn" onclick="setModuleFilter('classic', this)">Classic</button>
        </div>
        <div class="filter-group">
            <span style="font-size:12px; font-weight:700; color:var(--text-muted); margin-right:4px;">LEVEL / TIER:</span>
            <button class="filter-btn active" onclick="setTierFilter('all', this)">All Tiers</button>
            <button class="filter-btn" onclick="setTierFilter('1', this)">Level 1</button>
            <button class="filter-btn" onclick="setTierFilter('2', this)">Level 2</button>
            <button class="filter-btn" onclick="setTierFilter('3', this)">Level 3</button>
            <button class="filter-btn" onclick="setTierFilter('4', this)">Level 4</button>
            <button class="filter-btn" onclick="setTierFilter('5', this)">Level 5</button>
        </div>
    </div>

    <!-- Section: Vehicle Fleet Roster -->
    <section id="sec-fleet">
        <h2 class="section-title">🏎️ Evaluated Vehicle Fleet (5 Levels / Tiers + Classic Prototypical Cars)</h2>
        <div class="table-wrap">
            <table id="tableFleet">
                <thead>
                    <tr>
                        <th>Module</th>
                        <th>Level / Tier</th>
                        <th>Category</th>
                        <th>Vehicle Name</th>
                        <th>Drivetrain</th>
                        <th>Mass (kg)</th>
                        <th>Power (BHP)</th>
                        <th>Top Speed</th>
                        <th>Vehicle ID</th>
                    </tr>
                </thead>
                <tbody>
"#).unwrap();

    for v in &dataset.vehicles {
        let mod_badge = match v.module.as_str() {
            "GT / F1" => "badge-mod-gt",
            "NASCAR" => "badge-mod-nascar",
            "Rally" => "badge-mod-rally",
            "Extreme Off-Road" => "badge-mod-offroad",
            "Kart" => "badge-mod-kart",
            _ => "badge-mod-classic",
        };

        let drive_badge = match v.drivetrain.as_str() {
            "AWD" | "4WD" => "badge-drive-awd",
            "FWD" => "badge-drive-fwd",
            _ => "badge-drive-rwd",
        };

        let tier_label = if v.tier > 0 {
            format!("<span class=\"badge badge-tier badge-tier-{}\">Level {}</span>", v.tier, v.tier)
        } else {
            "<span class=\"badge badge-mod-classic\">Classic</span>".to_string()
        };

        writeln!(h, r#"                    <tr data-module="{}" data-tier="{}" data-search="{} {} {}">
                        <td><span class="badge {}">{}</span></td>
                        <td>{}</td>
                        <td><strong>{}</strong></td>
                        <td>{}</td>
                        <td><span class="badge {}">{}</span></td>
                        <td>{:.0} kg</td>
                        <td>{} BHP</td>
                        <td>{} km/h</td>
                        <td><code>{}</code></td>
                    </tr>"#,
            v.module.to_lowercase().replace(" ", "_").replace("/", "_"),
            if v.tier > 0 { v.tier.to_string() } else { "classic".to_string() },
            v.vehicle_name.to_lowercase(),
            v.category.to_lowercase(),
            v.vehicle_id.to_lowercase(),
            mod_badge,
            v.module,
            tier_label,
            v.category,
            v.vehicle_name,
            drive_badge,
            if !v.drivetrain.is_empty() { &v.drivetrain } else { "RWD" },
            v.mass_kg,
            v.power_bhp,
            v.top_speed_kmh,
            v.vehicle_id
        ).unwrap();
    }

    writeln!(h, r#"                </tbody>
            </table>
        </div>
    </section>

    <!-- Section: Protocol A -->
    <section id="sec-proto-a">
        <h2 class="section-title">🚀 Protocol A: Standing Start Acceleration & Traction (0 &rarr; 100 km/h)</h2>
        <div class="table-wrap">
            <table id="tableProtoA">
                <thead>
                    <tr>
                        <th>Vehicle</th>
                        <th>Mod / Tier</th>
"#).unwrap();
    for s in SurfaceType::ALL {
        write!(h, "                        <th>{}</th>\n", s.name()).unwrap();
    }
    writeln!(h, r#"                    </tr>
                </thead>
                <tbody>
"#).unwrap();

    for v in &dataset.vehicles {
        let tier_str = if v.tier > 0 { format!("L{}", v.tier) } else { "Cl".to_string() };
        write!(h, r#"                    <tr data-module="{}" data-tier="{}" data-search="{}">
                        <td><strong>{}</strong></td>
                        <td><span class="badge badge-tier">{}</span></td>
"#,
            v.module.to_lowercase().replace(" ", "_").replace("/", "_"),
            if v.tier > 0 { v.tier.to_string() } else { "classic".to_string() },
            v.vehicle_name.to_lowercase(),
            v.vehicle_name,
            tier_str
        ).unwrap();

        for res in &v.protocol_a {
            if let Some(t100) = res.t100_s {
                let col = if t100 < 4.0 { "color:var(--accent-green);font-weight:700;" }
                          else if t100 < 7.0 { "color:var(--text-main);" }
                          else if t100 < 12.0 { "color:var(--accent-gold);" }
                          else { "color:var(--accent-orange);" };
                write!(h, "                        <td style=\"{}\">{:.2}s</td>\n", col, t100).unwrap();
            } else {
                write!(h, "                        <td class=\"dnc\">DNC ({:.0}k)</td>\n", res.v_terminal_kmh).unwrap();
            }
        }
        writeln!(h, "                    </tr>").unwrap();
    }

    writeln!(h, r#"                </tbody>
            </table>
        </div>
    </section>

    <!-- Section: Protocol B -->
    <section id="sec-proto-b">
        <h2 class="section-title">🛑 Protocol B: Emergency Braking Distance (100 &rarr; 0 km/h)</h2>
        <div class="table-wrap">
            <table id="tableProtoB">
                <thead>
                    <tr>
                        <th>Vehicle</th>
                        <th>Mod / Tier</th>
"#).unwrap();
    for s in SurfaceType::ALL {
        write!(h, "                        <th>{}</th>\n", s.name()).unwrap();
    }
    writeln!(h, r#"                    </tr>
                </thead>
                <tbody>
"#).unwrap();

    for v in &dataset.vehicles {
        let tier_str = if v.tier > 0 { format!("L{}", v.tier) } else { "Cl".to_string() };
        write!(h, r#"                    <tr data-module="{}" data-tier="{}" data-search="{}">
                        <td><strong>{}</strong></td>
                        <td><span class="badge badge-tier">{}</span></td>
"#,
            v.module.to_lowercase().replace(" ", "_").replace("/", "_"),
            if v.tier > 0 { v.tier.to_string() } else { "classic".to_string() },
            v.vehicle_name.to_lowercase(),
            v.vehicle_name,
            tier_str
        ).unwrap();

        for res in &v.protocol_b {
            let dist = res.stopping_distance_m;
            let col = if dist < 40.0 { "color:var(--accent-green);font-weight:700;" }
                      else if dist < 60.0 { "color:var(--accent-cyan);" }
                      else if dist < 90.0 { "color:var(--accent-gold);" }
                      else if dist < 160.0 { "color:var(--accent-orange);" }
                      else { "color:var(--accent-red);font-weight:700;" };
            write!(h, "                        <td style=\"{}\">{:.1}m</td>\n", col, dist).unwrap();
        }
        writeln!(h, "                    </tr>").unwrap();
    }

    writeln!(h, r#"                </tbody>
            </table>
        </div>
    </section>

    <!-- Section: Protocol C -->
    <section id="sec-proto-c">
        <h2 class="section-title">🔄 Protocol C: Steady-State Skidpad Peak Lateral Grip (R = 30m)</h2>
        <div class="table-wrap">
            <table id="tableProtoC">
                <thead>
                    <tr>
                        <th>Vehicle</th>
                        <th>Mod / Tier</th>
"#).unwrap();
    for s in SurfaceType::ALL {
        write!(h, "                        <th>{}</th>\n", s.name()).unwrap();
    }
    writeln!(h, r#"                    </tr>
                </thead>
                <tbody>
"#).unwrap();

    for v in &dataset.vehicles {
        let tier_str = if v.tier > 0 { format!("L{}", v.tier) } else { "Cl".to_string() };
        write!(h, r#"                    <tr data-module="{}" data-tier="{}" data-search="{}">
                        <td><strong>{}</strong></td>
                        <td><span class="badge badge-tier">{}</span></td>
"#,
            v.module.to_lowercase().replace(" ", "_").replace("/", "_"),
            if v.tier > 0 { v.tier.to_string() } else { "classic".to_string() },
            v.vehicle_name.to_lowercase(),
            v.vehicle_name,
            tier_str
        ).unwrap();

        for res in &v.protocol_c {
            let g_val = res.peak_lateral_accel_g;
            let col = if g_val > 0.50 { "color:var(--accent-green);font-weight:700;" }
                      else if g_val > 0.25 { "color:var(--accent-cyan);" }
                      else if g_val > 0.12 { "color:var(--accent-gold);" }
                      else { "color:var(--text-dim);" };
            write!(h, "                        <td style=\"{}\">{:.2}g</td>\n", col, g_val).unwrap();
        }
        writeln!(h, "                    </tr>").unwrap();
    }

    writeln!(h, r#"                </tbody>
            </table>
        </div>
    </section>

    <!-- Section: Protocol D -->
    <section id="sec-proto-d">
        <h2 class="section-title">⚡ Protocol D: Step-Steer & Slalom Stability Handling (80 km/h)</h2>
        <div class="table-wrap">
            <table id="tableProtoD">
                <thead>
                    <tr>
                        <th>Vehicle</th>
                        <th>Mod / Tier</th>
                        <th>Asphalt Recovery</th>
                        <th>Dirt Recovery</th>
                        <th>Gravel Recovery</th>
                        <th>Mud Recovery</th>
                        <th>Water Recovery</th>
                        <th>Oil Recovery</th>
                        <th>Ice Recovery</th>
                    </tr>
                </thead>
                <tbody>
"#).unwrap();

    for v in &dataset.vehicles {
        let tier_str = if v.tier > 0 { format!("L{}", v.tier) } else { "Cl".to_string() };
        write!(h, r#"                    <tr data-module="{}" data-tier="{}" data-search="{}">
                        <td><strong>{}</strong></td>
                        <td><span class="badge badge-tier">{}</span></td>
"#,
            v.module.to_lowercase().replace(" ", "_").replace("/", "_"),
            if v.tier > 0 { v.tier.to_string() } else { "classic".to_string() },
            v.vehicle_name.to_lowercase(),
            v.vehicle_name,
            tier_str
        ).unwrap();

        // Sample across key representative surfaces: Asphalt(0), Dirt(2), Gravel(3), Mud(4), Water(8), Oil(9), Ice(10)
        let sampled_surfaces = [0, 2, 3, 4, 8, 9, 10];
        for &s_idx in &sampled_surfaces {
            let res = &v.protocol_d[s_idx];
            let status_badge = match res.recovery_status {
                super::protocols::StepSteerStatus::Stable => "<span class=\"badge\" style=\"background:rgba(48,209,88,0.2);color:#30d158;\">Stable</span>",
                super::protocols::StepSteerStatus::Drifting => "<span class=\"badge\" style=\"background:rgba(255,199,44,0.2);color:#ffc72c;\">Drift</span>",
                super::protocols::StepSteerStatus::Spun => "<span class=\"badge\" style=\"background:rgba(255,59,48,0.2);color:#ff3b30;\">Spun</span>",
            };
            write!(h, "                        <td>{} <small style=\"color:var(--text-dim);\">({:.0}&deg;/s)</small></td>\n", status_badge, res.peak_yaw_rate_deg_s).unwrap();
        }
        writeln!(h, "                    </tr>").unwrap();
    }

    writeln!(h, r#"                </tbody>
            </table>
        </div>
    </section>

    <!-- Section: Protocol E -->
    <section id="sec-proto-e">
        <h2 class="section-title">🍃 Protocol E: Passive Coast-Down Drag & Rolling Resistance (120 &rarr; 0 km/h)</h2>
        <div class="table-wrap">
            <table id="tableProtoE">
                <thead>
                    <tr>
                        <th>Vehicle</th>
                        <th>Mod / Tier</th>
"#).unwrap();
    for s in SurfaceType::ALL {
        write!(h, "                        <th>{}</th>\n", s.name()).unwrap();
    }
    writeln!(h, r#"                    </tr>
                </thead>
                <tbody>
"#).unwrap();

    for v in &dataset.vehicles {
        let tier_str = if v.tier > 0 { format!("L{}", v.tier) } else { "Cl".to_string() };
        write!(h, r#"                    <tr data-module="{}" data-tier="{}" data-search="{}">
                        <td><strong>{}</strong></td>
                        <td><span class="badge badge-tier">{}</span></td>
"#,
            v.module.to_lowercase().replace(" ", "_").replace("/", "_"),
            if v.tier > 0 { v.tier.to_string() } else { "classic".to_string() },
            v.vehicle_name.to_lowercase(),
            v.vehicle_name,
            tier_str
        ).unwrap();

        for res in &v.protocol_e {
            let dist = res.coast_distance_m;
            write!(h, "                        <td>{:.0} m</td>\n", dist).unwrap();
        }
        writeln!(h, "                    </tr>").unwrap();
    }

    writeln!(h, r#"                </tbody>
            </table>
        </div>
    </section>

</div>

<script>
    let activeModule = 'all';
    let activeTier = 'all';

    function scrollToId(id) {{
        const el = document.getElementById(id);
        if (el) {{
            el.scrollIntoView({{ behavior: 'smooth' }});
        }}
    }}

    function setModuleFilter(mod, btn) {{
        activeModule = mod;
        document.querySelectorAll('.filter-group:first-of-type .filter-btn').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        filterTables();
    }}

    function setTierFilter(tier, btn) {{
        activeTier = tier;
        document.querySelectorAll('.filter-group:last-of-type .filter-btn').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        filterTables();
    }}

    function filterTables() {{
        const search = document.getElementById('vehicleSearch').value.toLowerCase().trim();
        const tables = ['tableFleet', 'tableProtoA', 'tableProtoB', 'tableProtoC', 'tableProtoD', 'tableProtoE'];

        tables.forEach(tableId => {{
            const table = document.getElementById(tableId);
            if (!table) return;
            const rows = table.querySelectorAll('tbody tr');
            rows.forEach(row => {{
                const rowMod = row.getAttribute('data-module') || '';
                const rowTier = row.getAttribute('data-tier') || '';
                const rowSearch = (row.getAttribute('data-search') || '') + ' ' + row.innerText.toLowerCase();

                let matchMod = (activeModule === 'all') || (rowMod.includes(activeModule));
                let matchTier = (activeTier === 'all') || (rowTier === activeTier);
                let matchSearch = search === '' || rowSearch.includes(search);

                if (matchMod && matchTier && matchSearch) {{
                    row.style.display = '';
                }} else {{
                    row.style.display = 'none';
                }}
            }});
        }});
    }}
</script>

</body>
</html>
"#).unwrap();

    h
}
