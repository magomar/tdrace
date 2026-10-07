// Codex data collections (specs/087_tdrace_codex_unified_game_encyclopedia_and_technical_reference_portal.md).
// The JSON files are written by `cargo run -p tdrace-app --bin export_codex`; do not edit them by hand.
// A file with another schema_version, or an item that does not match its schema, stops the build.
import { defineCollection, z } from 'astro:content';
import { file } from 'astro/loaders';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';

export const CODEX_SCHEMA_VERSION = 1;

// CODEX_DATA_DIR selects another export, e.g. the launch-scope build (`make build-codex-launch`).
const dataDir = process.env.CODEX_DATA_DIR
  ? resolve(process.env.CODEX_DATA_DIR)
  : fileURLToPath(new URL('../../shared/data/codex', import.meta.url));

function codexFile(name: string) {
  return file(resolve(dataDir, name), {
    parser: (text) => {
      const doc = JSON.parse(text);
      if (doc.schema_version !== CODEX_SCHEMA_VERSION) {
        throw new Error(`${name}: schema_version ${doc.schema_version}, expected ${CODEX_SCHEMA_VERSION}. Re-run export_codex.`);
      }
      return doc.items;
    },
  });
}

const surfaceId = z.string();

const modules = defineCollection({
  loader: codexFile('modules.json'),
  schema: z.object({
    id: z.string(),
    title: z.string(),
    subtitle: z.string(),
    launch: z.boolean(),
    cars: z.number().int(),
    circuits: z.number().int(),
  }),
});

const cars = defineCollection({
  loader: codexFile('cars.json'),
  schema: z.object({
    id: z.string(),
    name: z.string(),
    manufacturer: z.string(),
    year: z.number().int(),
    module: z.string(),
    category: z.string(),
    category_name: z.string(),
    tier: z.number().int(),
    tier_name: z.string(),
    bhp: z.number(),
    torque_nm: z.number(),
    weight_kg: z.number(),
    top_speed_kmh: z.number(),
    accel_0_100: z.number(),
    drivetrain: z.string(),
    engine_desc: z.string(),
    aero_downforce: z.string(),
    brakes_desc: z.string(),
    history_bio: z.string(),
    stats: z.object({
      speed: z.number(),
      acceleration: z.number(),
      grip: z.number(),
      agility: z.number(),
      braking: z.number(),
      aero: z.number(),
    }),
    visual_type: z.unknown(),
    base_car: z.string(),
    sound: z.string(),
    images: z.object({
      reference: z.string().nullable(),
      lateral: z.string().nullable(),
      thumb: z.string().nullable(),
      topdown: z.string().nullable(),
    }),
    // The game's CarConfig. Fields the pages read are listed; the rest pass through unchecked.
    physics: z
      .object({
        mass: z.number(),
        max_engine_force: z.number(),
        max_brake_force: z.number(),
        drive_bias: z.number(),
        top_speed_mps: z.number(),
        downforce_coefficient: z.number(),
        air_drag_coefficient: z.number(),
      })
      .passthrough(),
  }),
});

const circuits = defineCollection({
  loader: codexFile('circuits.json'),
  schema: z.object({
    id: z.string(),
    name: z.string(),
    module: z.string(),
    category: z.string(),
    tag: z.string(),
    category_label: z.string(),
    description: z.string(),
    kind: z.enum(['circuit', 'arena']),
    country_code: z.string().nullable(),
    country_name: z.string().nullable(),
    scale: z.string(),
    wikipedia_url: z.string().nullable(),
    osm_url: z.string().nullable(),
    is_inspired: z.boolean(),
    default_laps: z.number().int(),
    length_m: z.number(),
    min_width_m: z.number(),
    max_width_m: z.number(),
    surfaces: z.array(z.object({ surface: surfaceId, share_pct: z.number() })),
    has_jumps: z.boolean(),
    has_joker: z.boolean(),
    has_pit_lane: z.boolean(),
    has_junctions: z.boolean(),
    image: z.string().nullable(),
  }),
});

const surfaces = defineCollection({
  loader: codexFile('surfaces.json'),
  schema: z.object({
    id: surfaceId,
    name: z.string(),
    friction: z.number(),
    rolling_resistance: z.number(),
    drag: z.number(),
    tire_smoke: z.boolean(),
    debris: z.boolean(),
    water_splash: z.boolean(),
    loose: z.boolean(),
    rigid_pavement: z.boolean(),
    on_track_hazard: z.boolean(),
    valid_off_track: z.boolean(),
  }),
});

const curve = z.array(z.tuple([z.number(), z.number()]));
const archetypeId = z.string();
const placementId = z.enum(['front_engine', 'mid_engine', 'rear_engine']);
// DifferentialType: "Open", "Spool" or { LimitedSlip: { power_lock, coast_lock, preload_nm } }.
const differential = z.union([
  z.enum(['Open', 'Spool']),
  z.object({
    LimitedSlip: z.object({ power_lock: z.number(), coast_lock: z.number(), preload_nm: z.number() }),
  }),
]);
const suspensionCorner = z.object({
  archetype: archetypeId,
  spring_rate: z.number(),
  bump_damping_ratio: z.number(),
  rebound_damping_ratio: z.number(),
  max_bump_travel: z.number(),
  max_rebound_travel: z.number(),
  static_camber: z.number(),
  camber_recovery: z.number(),
});
const suspensionSetup = z.object({
  front: suspensionCorner,
  rear: suspensionCorner,
  front_arb_rate: z.number(),
  rear_arb_rate: z.number(),
  front_roll_center_height: z.number(),
  rear_roll_center_height: z.number(),
  response_frequency_hz: z.number(),
});

const chassis = defineCollection({
  loader: codexFile('chassis.json'),
  schema: z.object({
    id: z.string(),
    title: z.string(),
    tag: z.string(),
    description: z.string(),
    modules: z.array(z.string()),
    cars: z.array(z.string()),
    wheelbase_m: z.number(),
    track_width_m: z.number(),
    cg_to_front_m: z.number(),
    cg_to_rear_m: z.number(),
    cg_height_m: z.number(),
    total_length_m: z.number(),
    skeleton: z.object({
      front_overhang: z.number(),
      rear_overhang: z.number(),
      body_width: z.number(),
      cabin_start_offset: z.number(),
      cabin_end_offset: z.number(),
      headlight_spread: z.number(),
      taillight_spread: z.number(),
      light_inset: z.number(),
    }),
    hull: z.object({ front_extent_m: z.number(), rear_extent_m: z.number(), half_width_m: z.number() }),
    wheels: z.array(
      z.object({
        position: z.enum(['FL', 'FR', 'RL', 'RR']),
        radius_m: z.number(),
        width_m: z.number(),
        inertia_kgm2: z.number(),
        compound: z.string(),
      }),
    ),
    suspension: suspensionSetup,
    front_differential: differential,
    rear_differential: differential,
    engine_placement: placementId,
    tire: z.object({
      peak_slip_angle_deg: z.number(),
      rear_peak_slip_angle_deg: z.number(),
      peak_slip_ratio: z.number(),
      slide_grip: z.number(),
      falloff: z.number(),
      load_sensitivity: z.number(),
      power_slide: z.number(),
      curve,
    }),
  }),
});

const suspension = defineCollection({
  loader: codexFile('suspension.json'),
  schema: z.object({
    id: archetypeId,
    robustness_factor: z.number(),
    part_cost_multiplier: z.number(),
    factory: suspensionSetup,
    platforms_front: z.array(z.string()),
    platforms_rear: z.array(z.string()),
  }),
});

const tyres = defineCollection({
  loader: codexFile('tyres.json'),
  schema: z.object({
    id: z.string(),
    name: z.string(),
    badge: z.string(),
    accent_rgba: z.array(z.number()).length(4),
    wear_rate: z.number(),
    optimal_temp_c: z.tuple([z.number(), z.number()]),
    overheat_temp_c: z.number(),
    affinity: z.array(z.object({ surface: surfaceId, multiplier: z.number() })),
    thermal_curve: curve,
    platforms: z.array(z.string()),
  }),
});

const drivetrain = defineCollection({
  loader: codexFile('drivetrain.json'),
  schema: z.object({
    id: placementId,
    repair_cost_multiplier: z.number(),
    platforms: z.array(z.string()),
  }),
});

const damage = defineCollection({
  loader: codexFile('damage.json'),
  schema: z.object({
    id: z.literal('damage'),
    energy_deadzone_j: z.number(),
    chassis_capacity_j: z.number(),
    engine_capacity_j: z.number(),
    suspension_capacity_j: z.number(),
    kerb_bottom_out_speed_mps: z.number(),
    kerb_bottom_out_capacity_j: z.number(),
    landing_speed_limit_mps: z.number(),
    landing_capacity_j: z.number(),
    pushrod_collapse_health: z.number(),
    pushrod_collapse_drag_multiplier: z.number(),
    zones: z.array(
      z.object({
        id: z.string(),
        weights: z.array(
          z.object({
            placement: placementId,
            chassis: z.number(),
            engine: z.number(),
            suspension: z.array(z.number()).length(4),
          }),
        ),
      }),
    ),
    engine_power_curve: curve,
    steering_pull_curve: curve,
    tire_wear_curve: curve,
    field_repair_caps: z.object({ chassis: z.number(), engine: z.number(), suspension: z.number() }),
    pit_stop_repair_amount: z.number(),
    purse_cap_share: z.number(),
    safety_net_credit_limit: z.number(),
    safety_net_health: z.number(),
    invoice_examples: z.array(
      z.object({
        tier: z.number(),
        base_purse: z.number(),
        health: z.number(),
        placement: placementId,
        archetype: archetypeId,
        invoice: z.object({
          chassis_cost: z.number(),
          engine_cost: z.number(),
          suspension_costs: z.array(z.number()).length(4),
          total_raw_damage_cost: z.number(),
          sponsor_subsidy: z.number(),
          net_deduction: z.number(),
          sponsor_safety_net_applied: z.boolean(),
        }),
      }),
    ),
  }),
});

const controls = defineCollection({
  loader: codexFile('controls.json'),
  schema: z.object({
    id: z.literal('controls'),
    presets: z.array(
      z.object({
        id: z.string(),
        name: z.string(),
        description: z.string(),
        bindings: z.array(
          z.object({
            action: z.string(),
            label: z.string(),
            keys: z.array(z.string()),
            gamepad: z.array(z.string()),
          }),
        ),
      }),
    ),
    hotkeys: z.array(
      z.object({
        key: z.string(),
        action: z.string(),
        context: z.string(),
        description: z.string(),
      }),
    ),
    gamepad: z.object({
      stick_deadzone: z.number(),
      trigger_deadzone: z.number(),
      steer_exponent: z.number(),
      steer_scale: z.number(),
      description: z.string(),
    }),
  }),
});

const driving = defineCollection({
  loader: codexFile('driving.json'),
  schema: z.object({
    id: z.literal('driving'),
    steering_profiles: z.array(
      z.object({
        id: z.string(),
        name: z.string(),
        display_name: z.string(),
        description: z.string(),
        steer_time_ms: z.number(),
        return_time_ms: z.number(),
        steer_authority: z.number(),
        center_precision: z.number(),
        pedal_time_ms: z.number(),
        traction_help: z.number(),
        recommended_for: z.string(),
        step_response: z.array(z.tuple([z.number(), z.number()])),
      }),
    ),
    assist_profiles: z.array(
      z.object({
        id: z.string(),
        name: z.string(),
        title: z.string(),
        short_name: z.string(),
        description: z.string(),
        tcs_enabled: z.boolean(),
        tcs_slip_threshold: z.number(),
        tcs_strength: z.number(),
        tcs_slip_angle_deg: z.number(),
        tcs_drift_bypass: z.boolean(),
        esc_enabled: z.boolean(),
        esc_yaw_threshold: z.number(),
        esc_strength: z.number(),
        esc_sideslip_limit_deg: z.number(),
        counter_steer_assist_enabled: z.boolean(),
        counter_steer_assist_strength: z.number(),
        abs_enabled: z.boolean(),
        abs_slip_threshold: z.number(),
        abs_strength: z.number(),
        handbrake_bypass: z.boolean(),
      }),
    ),
    aids: z.object({
      grip_aware_steering_description: z.string(),
      low_speed_authority_description: z.string(),
    }),
  }),
});

const hud = defineCollection({
  loader: codexFile('hud.json'),
  schema: z.object({
    id: z.literal('hud'),
    elements: z.array(
      z.object({
        id: z.string(),
        name: z.string(),
        screen_position: z.string(),
        hotkey: z.string().nullable().optional(),
        description: z.string(),
      }),
    ),
    hologram_modes: z.array(
      z.object({
        id: z.string(),
        name: z.string(),
        hotkey: z.string(),
        description: z.string(),
        features: z.array(z.string()),
      }),
    ),
    curve_helper: z.object({
      cycle_hotkey: z.string(),
      color_cycle_hotkey: z.string(),
      styles: z.array(
        z.object({
          id: z.string(),
          name: z.string(),
          description: z.string(),
        }),
      ),
      color_schemes: z.array(
        z.object({
          id: z.string(),
          name: z.string(),
          description: z.string(),
          palette: z.array(z.string()),
        }),
      ),
    }),
    locator_aids: z.array(
      z.object({
        id: z.string(),
        name: z.string(),
        description: z.string(),
      }),
    ),
    cameras: z.array(
      z.object({
        id: z.string(),
        name: z.string(),
        mode: z.string(),
        min_zoom: z.number(),
        max_zoom: z.number(),
        description: z.string(),
      }),
    ),
    camera_config: z.object({
      position_smoothing: z.number(),
      zoom_smoothing: z.number(),
      velocity_lookahead_time: z.number(),
      trauma_decay: z.number(),
      max_shake_offset: z.number(),
      max_car_screen_offset_frac: z.number(),
    }),
  }),
});

const racing = defineCollection({
  loader: codexFile('racing.json'),
  schema: z.object({
    id: z.literal('racing'),
    disciplines: z.array(
      z.object({
        id: z.string(),
        title: z.string(),
        subtitle: z.string(),
        description: z.string(),
        launch: z.boolean(),
        car_count: z.number().int(),
        circuit_count: z.number().int(),
        series_count: z.number().int(),
        tiers: z.array(
          z.object({
            tier: z.number().int(),
            name: z.string(),
            required_licence: z.string(),
            car_cost_xp: z.number().int(),
            base_purse: z.number().int(),
          }),
        ),
      }),
    ),
    formats: z.array(
      z.object({
        id: z.string(),
        name: z.string(),
        tag: z.string(),
        description: z.string(),
        rules: z.array(z.string()),
        scoring_summary: z.string(),
      }),
    ),
    joker_rule: z.object({
      mandatory_laps: z.number().int(),
      time_penalty_sec: z.number(),
      applies_to: z.string(),
      hud_indicator: z.string(),
      strategy_notes: z.string(),
    }),
    pit_service: z.object({
      repair_amount: z.number(),
      speed_limiter_kmh: z.number(),
      field_repair_chassis_cap: z.number(),
      field_repair_engine_cap: z.number(),
      field_repair_suspension_cap: z.number(),
      phases: z.array(
        z.object({
          id: z.string(),
          name: z.string(),
          description: z.string(),
        }),
      ),
    }),
    point_systems: z.array(
      z.object({
        id: z.string(),
        name: z.string(),
        description: z.string(),
        points_table: z.array(
          z.object({
            position: z.number().int(),
            points: z.number().int(),
          }),
        ),
        fastest_lap_bonus: z.string().nullable().optional(),
        stage_win_bonus: z.string().nullable().optional(),
      }),
    ),
    series: z.array(
      z.object({
        id: z.string(),
        name: z.string(),
        description: z.string(),
        module_id: z.string(),
        tier: z.number().int(),
        laps_per_round: z.number().int(),
        scoring_system: z.string(),
        fastest_lap_bonus: z.boolean(),
        stage_win_bonus: z.boolean(),
        round_count: z.number().int(),
        rounds: z.array(
          z.object({
            order: z.number().int(),
            track_id: z.string(),
            name: z.string().nullable().optional(),
            laps: z.number().int().nullable().optional(),
          }),
        ),
        driver_count: z.number().int(),
        drivers: z.array(
          z.object({
            id: z.string(),
            name: z.string(),
            team: z.string(),
            is_player: z.boolean(),
            car_model_id: z.string().nullable().optional(),
            country: z.string().nullable().optional(),
            ai_style: z.string().nullable().optional(),
            ai_tier: z.number().int().nullable().optional(),
          }),
        ),
      }),
    ),
    career: z.object({
      ladders: z.array(
        z.object({
          module_id: z.string(),
          module_name: z.string(),
          tiers: z.array(
            z.object({
              tier: z.number().int(),
              name: z.string(),
              promotion_criteria: z.string(),
              starter_car_id: z.string().nullable().optional(),
              starter_car_name: z.string().nullable().optional(),
              car_cost_xp: z.number().int(),
              first_time_exploration_xp: z.number().int(),
              round_base_purse: z.number().int(),
              unlocked_tracks: z.array(z.string()),
            }),
          ),
        }),
      ),
      xp_economy: z.object({
        distance_divisor: z.number(),
        completion_multiplier: z.number(),
        first_time_bonus_per_tier: z.number().int(),
        car_cost_per_tier: z.number().int(),
        formulas: z.array(z.string()),
      }),
      repair_economy: z.object({
        purse_cap_share: z.number(),
        safety_net_credit_limit: z.number().int(),
        safety_net_health: z.number(),
        formulas: z.array(z.string()),
      }),
    }),
    academy: z.object({
      curriculum: z.array(
        z.object({
          id: z.string(),
          index: z.number().int(),
          title: z.string(),
          description: z.string(),
          track_slug: z.string(),
          car_slug: z.string(),
          gold_time_sec: z.number(),
          silver_time_sec: z.number(),
          bronze_time_sec: z.number(),
          bronze_credit_bounty: z.number().int(),
          silver_credit_bounty: z.number().int(),
          gold_credit_bounty: z.number().int(),
          base_xp_reward: z.number().int(),
        }),
      ),
      max_permissible_impulse: z.number(),
      max_permissible_off_track_sec: z.number(),
      licence_grades: z.array(
        z.object({
          grade: z.string(),
          title: z.string(),
          badge: z.string(),
          description: z.string(),
          required_for: z.array(z.string()),
        }),
      ),
    }),
    multiplayer: z.object({
      min_players: z.number().int(),
      max_players: z.number().int(),
      simulation_rate_hz: z.number().int(),
      interpolation_delay_ms: z.number().int(),
      discovery: z.string(),
      collision_modes: z.array(
        z.object({
          id: z.string(),
          name: z.string(),
          description: z.string(),
        }),
      ),
      host_steps: z.array(z.string()),
      join_steps: z.array(z.string()),
    }),
  }),
});

const rivals = defineCollection({
  loader: codexFile('rivals.json'),
  schema: z.object({
    id: z.literal('rivals'),
    drivers: z.array(
      z.object({
        id: z.string(),
        name: z.string(),
        alias: z.string(),
        bio: z.string(),
        style: z.string(),
        preferred_car: z.string(),
        primary_color: z.array(z.number()),
        secondary_color: z.array(z.number()),
        accent_color: z.array(z.number()),
        stats: z.object({
          speed: z.number(),
          aggression: z.number(),
          precision: z.number(),
          defense: z.number(),
        }),
        favorite_cars: z.array(
          z.object({
            discipline: z.string(),
            tier: z.number().int(),
            model_id: z.string(),
          }),
        ),
      }),
    ),
    driving_styles: z.array(
      z.object({
        id: z.string(),
        name: z.string(),
        description: z.string(),
        speed_mult: z.number(),
        aggression: z.number(),
        precision: z.number(),
        defense: z.number(),
        tactical_traits: z.array(z.string()),
      }),
    ),
    skill_tiers: z.array(
      z.object({
        tier: z.number().int(),
        name: z.string(),
        short_name: z.string(),
        tag: z.string(),
        pace_limit: z.number(),
        consistency: z.number(),
        composure: z.number(),
        bell_curve_weights: z.array(z.number().int()),
      }),
    ),
    mistake_kinds: z.array(
      z.object({
        id: z.string(),
        name: z.string(),
        description: z.string(),
        trigger_condition: z.string(),
        gameplay_impact: z.string(),
        recovery_behavior: z.string(),
      }),
    ),
  }),
});

export const collections = {
  modules,
  cars,
  circuits,
  surfaces,
  chassis,
  suspension,
  tyres,
  drivetrain,
  damage,
  controls,
  driving,
  hud,
  racing,
  rivals,
};
