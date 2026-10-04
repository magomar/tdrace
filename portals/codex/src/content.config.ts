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

export const collections = { modules, cars, circuits, surfaces };
