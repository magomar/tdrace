// Shared helpers for Codex pages: data access, module presentation and the site map.
import { getCollection, type CollectionEntry } from 'astro:content';

export type Car = CollectionEntry<'cars'>['data'];
export type Circuit = CollectionEntry<'circuits'>['data'];
export type Surface = CollectionEntry<'surfaces'>['data'];
export type Module = CollectionEntry<'modules'>['data'];
export type Platform = CollectionEntry<'chassis'>['data'];
export type SuspensionArchetype = CollectionEntry<'suspension'>['data'];
export type Compound = CollectionEntry<'tyres'>['data'];
export type DamageModel = CollectionEntry<'damage'>['data'];
export type ControlsModel = CollectionEntry<'controls'>['data'];
export type DrivingModel = CollectionEntry<'driving'>['data'];
export type HudModel = CollectionEntry<'hud'>['data'];
export type RacingModel = CollectionEntry<'racing'>['data'];
export type RivalsModel = CollectionEntry<'rivals'>['data'];

// Collections keep the export order (modules grouped in Codex order), so no sorting here.
export const getCars = async () => (await getCollection('cars')).map((e) => e.data);
export const getCircuits = async () => (await getCollection('circuits')).map((e) => e.data);
export const getSurfaces = async () => (await getCollection('surfaces')).map((e) => e.data);
export const getModules = async () => (await getCollection('modules')).map((e) => e.data);
export const getPlatforms = async () => (await getCollection('chassis')).map((e) => e.data);
export const getSuspensionArchetypes = async () => (await getCollection('suspension')).map((e) => e.data);
export const getCompounds = async () => (await getCollection('tyres')).map((e) => e.data);
export const getEnginePlacements = async () => (await getCollection('drivetrain')).map((e) => e.data);
export const getDamageModel = async () => (await getCollection('damage'))[0].data;
export const getControlsModel = async () => (await getCollection('controls'))[0].data;
export const getDrivingModel = async () => (await getCollection('driving'))[0].data;
export const getHudModel = async () => (await getCollection('hud'))[0].data;
export const getRacingModel = async () => (await getCollection('racing'))[0].data;
export const getRivalsModel = async () => (await getCollection('rivals'))[0].data;

// Presentation only: a short label and a badge colour per module id. Titles come from the export.
const MODULE_STYLE: Record<string, { label: string; badge: string }> = {
  classic: { label: 'Arcade', badge: 'bg-cyan-500/15 text-cyan-400 border-cyan-500/30' },
  kart: { label: 'Karting', badge: 'bg-purple-500/15 text-purple-400 border-purple-500/30' },
  autocross: { label: 'Autocross', badge: 'bg-orange-500/15 text-orange-400 border-orange-500/30' },
  rally: { label: 'Rallycross', badge: 'bg-emerald-500/15 text-emerald-400 border-emerald-500/30' },
  gt: { label: 'Gran Turismo', badge: 'bg-blue-500/15 text-blue-400 border-blue-500/30' },
  nascar: { label: 'Stock cars', badge: 'bg-red-500/15 text-red-400 border-red-500/30' },
  extreme_offroad: { label: 'All terrain', badge: 'bg-amber-500/15 text-amber-400 border-amber-500/30' },
};

// Car categories (CarCategory::id in Rust). Classic cars and circuits carry the category they imitate.
const CATEGORY_LABEL: Record<string, string> = {
  gt: 'Gran Turismo',
  nascar: 'Stock cars',
  rally: 'Rallycross',
  kart: 'Karting',
  off_road: 'All terrain',
  autocross: 'Autocross',
};

const NEUTRAL_BADGE = 'bg-slate-500/15 text-slate-400 border-slate-500/30';

export const moduleLabel = (id: string) => MODULE_STYLE[id]?.label ?? id;
export const moduleBadge = (id: string) => MODULE_STYLE[id]?.badge ?? NEUTRAL_BADGE;
export const categoryLabel = (id: string) => CATEGORY_LABEL[id] ?? id;

export const pct = (x: number) => Math.round(x * 100);

// Display names for Rust enum ids. Presentation only.
export const ARCHETYPE_LABEL: Record<string, string> = {
  rigid_kart: 'Rigid kart',
  solid_live_axle: 'Solid live axle',
  mac_pherson_strut: 'MacPherson strut',
  double_wishbone: 'Double wishbone',
  pushrod_inboard: 'Pushrod inboard',
  long_travel_off_road: 'Long-travel off-road',
};
export const PLACEMENT_LABEL: Record<string, string> = {
  front_engine: 'Front engine',
  mid_engine: 'Mid engine',
  rear_engine: 'Rear engine',
};
export const ZONE_LABEL: Record<string, string> = {
  front_nose: 'Front nose',
  rear_tail: 'Rear tail',
  flank_left: 'Left flank',
  flank_right: 'Right flank',
  corner_f_l: 'Front-left corner',
  corner_f_r: 'Front-right corner',
  corner_r_l: 'Rear-left corner',
  corner_r_r: 'Rear-right corner',
};

export type Differential = Platform['front_differential'];
export const differentialLabel = (d: Differential) =>
  typeof d === 'string'
    ? d === 'Open'
      ? 'Open'
      : 'Spool (locked)'
    : `LSD ${pct(d.LimitedSlip.power_lock)}% / ${pct(d.LimitedSlip.coast_lock)}% · ${d.LimitedSlip.preload_nm} N·m`;

export const rgba = (c: number[]) =>
  `rgba(${Math.round(c[0] * 255)}, ${Math.round(c[1] * 255)}, ${Math.round(c[2] * 255)}, ${c[3]})`;
export const deg = (rad: number) => (rad * 180) / Math.PI;

// Site map. `ready: false` pages belong to later phases of spec 087 and are listed but not linked.
export interface CodexPage {
  label: string;
  href: string;
  blurb: string;
  ready: boolean;
}

export interface CodexSection {
  id: string;
  label: string;
  href: string;
  blurb: string;
  pages: CodexPage[];
}

export const SECTIONS: CodexSection[] = [
  {
    id: 'showroom',
    label: 'Showroom',
    href: '/showroom',
    blurb: 'Every car and circuit in the game, with the numbers the simulation uses.',
    pages: [
      { label: 'Cars', href: '/showroom/cars', blurb: 'The car catalogue by discipline and tier.', ready: true },
      { label: 'Circuits', href: '/showroom/circuits', blurb: 'Every official circuit with length, width and surfaces.', ready: true },
      { label: 'Compare', href: '/showroom/compare', blurb: 'Two to four cars side by side.', ready: true },
    ],
  },
  {
    id: 'technical',
    label: 'Technical',
    href: '/technical',
    blurb: 'How the simulation models a car: chassis, suspension, tyres, surfaces, drivetrain and damage.',
    pages: [
      { label: 'Physics lab', href: '/technical/physics-lab', blurb: 'Tyre curve, surface table and the main formulas.', ready: true },
      { label: 'Chassis', href: '/technical/chassis', blurb: 'Chassis skeletons, body types and collision hulls.', ready: true },
      { label: 'Suspension', href: '/technical/suspension', blurb: 'Six suspension types, their settings and how they fail.', ready: true },
      { label: 'Tyres', href: '/technical/tyres', blurb: 'Grip curve, compounds, heat and wear, wheel geometry.', ready: true },
      { label: 'Surfaces', href: '/technical/surfaces', blurb: 'Fifteen surfaces and how each compound grips on them.', ready: true },
      { label: 'Drivetrain', href: '/technical/drivetrain', blurb: 'Engine force, drive split, differentials, engine placement.', ready: true },
      { label: 'Damage & repair', href: '/technical/damage', blurb: 'Impact zones, power loss, field and garage repairs.', ready: true },
    ],
  },
  {
    id: 'driving',
    label: 'Driving',
    href: '/driving',
    blurb: 'How you drive: controls, steering profiles, assists, the HUD and cameras.',
    pages: [
      { label: 'Controls', href: '/driving/controls', blurb: 'Keyboard and gamepad presets and in-race hotkeys.', ready: true },
      { label: 'Steering', href: '/driving/steering', blurb: 'The four steering profiles and their step response.', ready: true },
      { label: 'Assists', href: '/driving/assists', blurb: 'Arcade, Sport and Pro: TCS, ESC, ABS and counter-steer.', ready: true },
      { label: 'HUD', href: '/driving/hud', blurb: 'Every HUD element, the cockpit hologram and locator aids.', ready: true },
      { label: 'Cameras', href: '/driving/cameras', blurb: 'Camera modes, look-ahead and speed zoom.', ready: true },
    ],
  },
  {
    id: 'racing',
    label: 'Racing',
    href: '/racing',
    blurb: 'How you race: disciplines, formats, championships, career, academy, rivals and LAN.',
    pages: [
      { label: 'Disciplines', href: '/racing/disciplines', blurb: 'Each discipline with its cars, circuits and tiers.', ready: true },
      { label: 'Formats', href: '/racing/formats', blurb: 'Laps, time attack, joker laps, pit stops, qualifying.', ready: true },
      { label: 'Championships', href: '/racing/championships', blurb: 'Point systems and series presets.', ready: true },
      { label: 'Career', href: '/racing/career', blurb: 'Tier ladders, promotion, credits and XP.', ready: true },
      { label: 'Academy', href: '/racing/academy', blurb: 'Lessons, medals and licence grades.', ready: true },
      { label: 'Rivals', href: '/racing/rivals', blurb: 'AI drivers, driving styles, skill tiers and mistakes.', ready: true },
      { label: 'Multiplayer', href: '/racing/multiplayer', blurb: 'LAN races for two to eight players.', ready: true },
    ],
  },
];
