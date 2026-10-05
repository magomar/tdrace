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

export const collections = { modules, cars, circuits, surfaces, chassis, suspension, tyres, drivetrain, damage };
