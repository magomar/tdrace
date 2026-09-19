import { z } from 'zod';

export const okfFrontmatterSchema = z.object({
  type: z.enum([
    'Index',
    'Project Constitution',
    'Tech Stack',
    'Product Roadmap',
    'Feature Spec',
    'Architecture Spec',
    'Asset Catalog',
    'Physics Reference',
  ]),
  title: z.string(),
  description: z.string(),
  status: z.enum(['draft', 'active', 'implemented', 'deprecated']).default('active'),
  okf_version: z.string().optional(),
  category: z.string().optional(),
  tags: z.array(z.string()).default([]),
});

export type OkfFrontmatter = z.infer<typeof okfFrontmatterSchema>;

export const vehicleSchema = z.object({
  id: z.string(),
  name: z.string(),
  manufacturer: z.string().optional(),
  year: z.number().optional(),
  module: z.string(),
  tier: z.number(),
  category: z.string(),
  class_badge: z.string(),
  mass: z.number(),
  power_bhp: z.number(),
  torque_nm: z.number().optional(),
  engine_force: z.number(),
  top_speed_kmh: z.number(),
  accel_0_100: z.number().optional(),
  drive_bias: z.number(),
  drivetrain: z.string(),
  engine_desc: z.string().optional(),
  downforce: z.number(),
  drag: z.number(),
  brakes_desc: z.string().optional(),
  brakes_kn: z.number(),
  stats: z.object({
    speed: z.number(),
    acceleration: z.number(),
    grip: z.number(),
    agility: z.number(),
    downforce: z.number(),
  }),
  summary: z.string(),
});

export type Vehicle = z.infer<typeof vehicleSchema>;

export const circuitSchema = z.object({
  id: z.string(),
  name: z.string(),
  category: z.string(),
  modality: z.string(),
  description: z.string(),
  length_meters: z.number(),
  turns_count: z.number(),
  surfaces: z.array(z.string()),
  has_jumps: z.boolean(),
  rel_path: z.string(),
});

export type Circuit = z.infer<typeof circuitSchema>;

export const surfaceSchema = z.object({
  name: z.string(),
  friction: z.number(),
  rolling_resistance: z.number(),
  surface_drag: z.number(),
  smoke: z.boolean(),
  roost: z.boolean(),
  splash: z.boolean(),
  layer: z.string(),
  description: z.string(),
});

export type Surface = z.infer<typeof surfaceSchema>;
