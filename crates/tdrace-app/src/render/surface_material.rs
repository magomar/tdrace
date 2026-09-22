use std::collections::HashMap;
use macroquad::color::Color;
use macroquad::texture::{FilterMode, Image, Texture2D};
use serde::{Deserialize, Serialize};
use tdrace_core::physics::surface::SurfaceType;

/// Graphics quality tier for surface textures and terrain material shaders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SurfaceTextureQuality {
    /// Classic flat vector fills without bitmap textures. Minimal CPU/GPU overhead.
    Off,
    /// Fast 256x256 linear-filtered textures without macro-modulation.
    Standard,
    /// Full 512x512 physical fidelity micro-textures with multi-frequency macro-modulation.
    #[default]
    High,
}

impl SurfaceTextureQuality {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Off => "Off (Flat)",
            Self::Standard => "Standard",
            Self::High => "High",
        }
    }
}

/// Visual material properties bound to a racing surface type.
#[derive(Clone)]
pub struct SurfaceMaterial {
    pub surface_type: SurfaceType,
    pub texture: Option<Texture2D>,
    pub tile_scale_meters: f32,
    pub base_tint: Color,
    pub roughness: f32,
    pub has_macro_modulation: bool,
}

impl SurfaceMaterial {
    pub fn new(surface_type: SurfaceType, texture: Option<Texture2D>) -> Self {
        let (tile_scale_meters, base_tint, roughness, has_macro_modulation) = match surface_type {
            SurfaceType::Asphalt => (4.0, Color::new(1.0, 1.0, 1.0, 1.0), 0.70, true),
            SurfaceType::Concrete => (5.0, Color::new(1.0, 1.0, 1.0, 1.0), 0.55, false),
            SurfaceType::Curb => (3.0, Color::new(1.0, 1.0, 1.0, 1.0), 0.60, false),
            SurfaceType::Dirt => (3.0, Color::new(1.0, 1.0, 1.0, 1.0), 0.85, true),
            SurfaceType::Gravel => (3.5, Color::new(1.0, 1.0, 1.0, 1.0), 0.90, true),
            SurfaceType::Grass => (6.0, Color::new(1.0, 1.0, 1.0, 1.0), 0.80, true),
            SurfaceType::Sand => (6.0, Color::new(1.0, 1.0, 1.0, 1.0), 0.85, true),
            SurfaceType::Mud => (3.0, Color::new(1.0, 1.0, 1.0, 1.0), 0.40, true),
            SurfaceType::Snow => (5.0, Color::new(1.0, 1.0, 1.0, 1.0), 0.75, true),
            SurfaceType::Ice => (4.0, Color::new(1.0, 1.0, 1.0, 0.90), 0.15, false),
            SurfaceType::Water => (4.0, Color::new(1.0, 1.0, 1.0, 0.85), 0.10, false),
            SurfaceType::Oil => (3.0, Color::new(1.0, 1.0, 1.0, 0.95), 0.05, false),
        };

        Self {
            surface_type,
            texture,
            tile_scale_meters,
            base_tint,
            roughness,
            has_macro_modulation,
        }
    }
}

/// Central registry caching GPU surface textures and materials.
pub struct SurfaceMaterialRegistry {
    materials: HashMap<SurfaceType, SurfaceMaterial>,
    curb_material: Option<SurfaceMaterial>,
    edge_fringe_texture: Option<Texture2D>,
    macro_noise_texture: Option<Texture2D>,
    quality: SurfaceTextureQuality,
}

impl Default for SurfaceMaterialRegistry {
    fn default() -> Self {
        Self::new(SurfaceTextureQuality::High)
    }
}

impl SurfaceMaterialRegistry {
    pub fn new(quality: SurfaceTextureQuality) -> Self {
        let mut registry = Self {
            materials: HashMap::new(),
            curb_material: None,
            edge_fringe_texture: None,
            macro_noise_texture: None,
            quality,
        };

        if quality != SurfaceTextureQuality::Off {
            registry.load_or_generate_all();
        }

        registry
    }

    pub fn quality(&self) -> SurfaceTextureQuality {
        self.quality
    }

    pub fn set_quality(&mut self, quality: SurfaceTextureQuality) {
        if self.quality != quality {
            self.quality = quality;
            if quality == SurfaceTextureQuality::Off {
                self.materials.clear();
                self.curb_material = None;
                self.edge_fringe_texture = None;
                self.macro_noise_texture = None;
            } else {
                self.load_or_generate_all();
            }
        }
    }

    pub fn get_material(&self, surface: SurfaceType) -> Option<&SurfaceMaterial> {
        if self.quality == SurfaceTextureQuality::Off {
            None
        } else {
            self.materials.get(&surface)
        }
    }

    pub fn get_curb_material(&self) -> Option<&SurfaceMaterial> {
        if self.quality == SurfaceTextureQuality::Off {
            None
        } else {
            self.curb_material.as_ref()
        }
    }

    pub fn edge_fringe_texture(&self) -> Option<&Texture2D> {
        if self.quality == SurfaceTextureQuality::High {
            self.edge_fringe_texture.as_ref()
        } else {
            None
        }
    }

    pub fn macro_noise_texture(&self) -> Option<&Texture2D> {
        if self.quality == SurfaceTextureQuality::High {
            self.macro_noise_texture.as_ref()
        } else {
            None
        }
    }

    pub fn tile_scale(&self, surface: SurfaceType) -> f32 {
        self.materials
            .get(&surface)
            .map(|m| m.tile_scale_meters)
            .unwrap_or(4.0)
    }

    fn load_or_generate_all(&mut self) {
        let surfaces = [
            SurfaceType::Asphalt,
            SurfaceType::Dirt,
            SurfaceType::Curb,
            SurfaceType::Grass,
            SurfaceType::Sand,
            SurfaceType::Water,
            SurfaceType::Oil,
            SurfaceType::Ice,
            SurfaceType::Mud,
            SurfaceType::Snow,
            SurfaceType::Gravel,
            SurfaceType::Concrete,
        ];

        let tex_dim = match self.quality {
            SurfaceTextureQuality::High => 256,
            SurfaceTextureQuality::Standard => 128,
            SurfaceTextureQuality::Off => 64,
        };

        for &surf in &surfaces {
            let tex = Self::load_or_generate_surface_texture(surf, tex_dim);
            let mat = SurfaceMaterial::new(surf, tex);
            self.materials.insert(surf, mat);
        }

        // Curb dedicated material
        let curb_tex = Self::load_or_generate_curb_texture(tex_dim);
        self.curb_material = Some(SurfaceMaterial::new(SurfaceType::Curb, curb_tex));

        // Edge fringe and macro noise
        let fringe_tex = if let Some(bytes) = Self::find_surface_asset_file("edge_fringe_mask.png") {
            if let Ok(tex) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let t = Texture2D::from_file_with_format(&bytes, None);
                t.set_filter(FilterMode::Linear);
                Self::set_texture_repeat(&t);
                t
            })) {
                Some(tex)
            } else {
                let fringe_img = generate_edge_fringe_mask(128, 128);
                Self::upload_texture(&fringe_img)
            }
        } else {
            let fringe_img = generate_edge_fringe_mask(128, 128);
            Self::upload_texture(&fringe_img)
        };
        self.edge_fringe_texture = fringe_tex;

        let noise_img = generate_macro_noise_image(128, 128);
        self.macro_noise_texture = Self::upload_texture(&noise_img);
    }

    fn find_surface_asset_file(filename: &str) -> Option<Vec<u8>> {
        let candidates = [
            format!("assets/textures/surfaces/{}", filename),
            format!("../assets/textures/surfaces/{}", filename),
            format!("../../assets/textures/surfaces/{}", filename),
            format!("../../../assets/textures/surfaces/{}", filename),
            format!("../../../../assets/textures/surfaces/{}", filename),
        ];
        for c in &candidates {
            if let Ok(data) = std::fs::read(c) {
                return Some(data);
            }
        }
        None
    }

    fn load_or_generate_surface_texture(surf: SurfaceType, dim: u16) -> Option<Texture2D> {
        let filename = match surf {
            SurfaceType::Asphalt => "asphalt_diffuse.png",
            SurfaceType::Dirt => "dirt_compacted.png",
            SurfaceType::Curb => "curb_teeth.png",
            SurfaceType::Grass => "grass_turf.png",
            SurfaceType::Sand => "sand_dune.png",
            SurfaceType::Water => "water_caustic.png",
            SurfaceType::Oil => "oil_iridescent.png",
            SurfaceType::Ice => "ice_glazed.png",
            SurfaceType::Mud => "mud_viscous.png",
            SurfaceType::Snow => "snow_powder.png",
            SurfaceType::Gravel => "gravel_crushed.png",
            SurfaceType::Concrete => "concrete_brushed.png",
        };

        if let Some(bytes) = Self::find_surface_asset_file(filename) {
            if let Ok(tex) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let t = Texture2D::from_file_with_format(&bytes, None);
                t.set_filter(FilterMode::Linear);
                Self::set_texture_repeat(&t);
                t
            })) {
                return Some(tex);
            }
        }

        // Procedural synthetic fallback
        let img = generate_surface_image(surf, dim, dim);
        Self::upload_texture(&img)
    }

    fn load_or_generate_curb_texture(dim: u16) -> Option<Texture2D> {
        if let Some(bytes) = Self::find_surface_asset_file("curb_teeth.png") {
            if let Ok(tex) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let t = Texture2D::from_file_with_format(&bytes, None);
                t.set_filter(FilterMode::Linear);
                Self::set_texture_repeat(&t);
                t
            })) {
                return Some(tex);
            }
        }
        let img = generate_curb_image(dim, dim / 2);
        Self::upload_texture(&img)
    }

    fn set_texture_repeat(tex: &Texture2D) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            unsafe {
                let gl_ctx = macroquad::window::get_internal_gl();
                gl_ctx.quad_context.texture_set_wrap(
                    tex.raw_miniquad_id(),
                    macroquad::miniquad::TextureWrap::Repeat,
                    macroquad::miniquad::TextureWrap::Repeat,
                );
            }
        }));
    }

    fn upload_texture(img: &Image) -> Option<Texture2D> {
        // Safe upload: catches panic if running in headless environments without window context.
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let tex = Texture2D::from_image(img);
            tex.set_filter(FilterMode::Linear);
            Self::set_texture_repeat(&tex);
            tex
        })).ok()
    }
}

// ---------------------------------------------------------------------------
// Procedural Synthetic Fallback Generators (Physical Fidelity Micro-Textures)
// ---------------------------------------------------------------------------

#[inline]
fn pseudo_hash(x: u32, y: u32, seed: u32) -> u32 {
    let mut h = x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263) ^ seed.wrapping_mul(1274126177);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

#[inline]
fn pseudo_noise_f32(x: u32, y: u32, seed: u32) -> f32 {
    (pseudo_hash(x, y, seed) & 0xFFFF) as f32 / 65535.0
}

#[inline]
fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Evaluates a 2D periodic (toroidal) value noise field that wraps seamlessly at `grid_w` and `grid_h`.
fn sample_periodic_noise(x: f32, y: f32, grid_w: u32, grid_h: u32, seed: u32) -> f32 {
    let gw = grid_w as f32;
    let gh = grid_h as f32;
    let gx = (x % gw + gw) % gw;
    let gy = (y % gh + gh) % gh;
    let x0 = gx as u32;
    let y0 = gy as u32;
    let x1 = (x0 + 1) % grid_w;
    let y1 = (y0 + 1) % grid_h;
    let fx = smoothstep(gx - x0 as f32);
    let fy = smoothstep(gy - y0 as f32);

    let v00 = pseudo_noise_f32(x0, y0, seed);
    let v10 = pseudo_noise_f32(x1, y0, seed);
    let v01 = pseudo_noise_f32(x0, y1, seed);
    let v11 = pseudo_noise_f32(x1, y1, seed);

    let top = v00 * (1.0 - fx) + v10 * fx;
    let bot = v01 * (1.0 - fx) + v11 * fx;
    top * (1.0 - fy) + bot * fy
}

/// Generates an in-memory physical micro-texture Image for a given surface type.
pub fn generate_surface_image(surface: SurfaceType, width: u16, height: u16) -> Image {
    let mut bytes = vec![0u8; width as usize * height as usize * 4];

    match surface {
        SurfaceType::Asphalt => {
            // Isotropic matte bitumen aggregate: smooth multi-scale matrix with subtle crushed stone specks
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let n_coarse = sample_periodic_noise(
                        x as f32 * (64.0 / width as f32),
                        y as f32 * (64.0 / height as f32),
                        64,
                        64,
                        101,
                    );
                    let n_fine = sample_periodic_noise(
                        x as f32 * (128.0 / width as f32),
                        y as f32 * (128.0 / height as f32),
                        128,
                        128,
                        202,
                    );
                    let jitter = (pseudo_noise_f32(x as u32, y as u32, 303) - 0.5) * 6.0;

                    let mut base = 33.0 + n_coarse * 8.0 + n_fine * 4.0 + jitter;
                    // Low-contrast crushed aggregate specks (soft +12 luminance jump, anti-aliased)
                    if pseudo_noise_f32(x as u32, y as u32, 404) > 0.96 {
                        base += 12.0;
                    }
                    let base_clamped = base.clamp(26.0, 52.0);

                    bytes[idx] = (base_clamped * 0.97) as u8;
                    bytes[idx + 1] = (base_clamped * 0.99) as u8;
                    bytes[idx + 2] = (base_clamped * 1.03) as u8;
                    bytes[idx + 3] = 255;
                }
            }
        }
        SurfaceType::Dirt => {
            // Compacted clay / loam with tangent-aligned rut striations and fine dust grain
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let rut_wave = ((x as f32 / width as f32) * std::f32::consts::PI * 8.0).sin();
                    let n = pseudo_noise_f32(x as u32, y as u32, 303);
                    let n_rut = pseudo_noise_f32(x as u32 / 3, y as u32, 404);

                    let base_lum = 0.55 + rut_wave * 0.08 + (n - 0.5) * 0.16 + (n_rut - 0.5) * 0.12;
                    let r = (145.0 * base_lum).clamp(60.0, 210.0) as u8;
                    let g = (100.0 * base_lum).clamp(40.0, 160.0) as u8;
                    let b = (62.0 * base_lum).clamp(24.0, 115.0) as u8;

                    bytes[idx] = r;
                    bytes[idx + 1] = g;
                    bytes[idx + 2] = b;
                    bytes[idx + 3] = 255;
                }
            }
        }
        SurfaceType::Grass => {
            // Fine seamless turf: periodic multi-scale organic lawn noise without square borders or seams
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let n1 = sample_periodic_noise(
                        x as f32 * (32.0 / width as f32),
                        y as f32 * (32.0 / height as f32),
                        32,
                        32,
                        505,
                    );
                    let n2 = sample_periodic_noise(
                        x as f32 * (64.0 / width as f32),
                        y as f32 * (64.0 / height as f32),
                        64,
                        64,
                        606,
                    );
                    let val = n1 * 0.70 + n2 * 0.30;

                    let r = (30.0 + val * 26.0).clamp(24.0, 70.0) as u8;
                    let g = (78.0 + val * 52.0).clamp(65.0, 150.0) as u8;
                    let b = (34.0 + val * 30.0).clamp(26.0, 80.0) as u8;

                    bytes[idx] = r;
                    bytes[idx + 1] = g;
                    bytes[idx + 2] = b;
                    bytes[idx + 3] = 255;
                }
            }
        }
        SurfaceType::Gravel => {
            // Crushed angular limestone & slate scree with cast micro-shadows
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let cell_x = (x / 4) as u32;
                    let cell_y = (y / 4) as u32;
                    let pebble_id = pseudo_hash(cell_x, cell_y, 707);
                    let pebble_hue = (pebble_id & 0xFF) as f32 / 255.0;

                    let in_cell_x = (x % 4) as f32;
                    let in_cell_y = (y % 4) as f32;
                    // Top-left highlight, bottom-right micro-shadow
                    let relief = (4.0 - (in_cell_x + in_cell_y)) * 0.12 - 0.24;

                    let base = 135.0 + (pebble_hue - 0.5) * 45.0 + relief * 50.0;
                    let r = base.clamp(60.0, 215.0) as u8;
                    let g = (base * 0.98).clamp(58.0, 210.0) as u8;
                    let b = (base * 0.94).clamp(55.0, 205.0) as u8;

                    bytes[idx] = r;
                    bytes[idx + 1] = g;
                    bytes[idx + 2] = b;
                    bytes[idx + 3] = 255;
                }
            }
        }
        SurfaceType::Concrete => {
            // Brushed industrial slabs with fine directional micro-grooves and expansion joint
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let is_joint = y == height / 2 || y == height / 2 + 1;
                    if is_joint {
                        bytes[idx] = 60;
                        bytes[idx + 1] = 62;
                        bytes[idx + 2] = 65;
                        bytes[idx + 3] = 255;
                    } else {
                        let brush = pseudo_noise_f32(x as u32, y as u32 / 2, 808);
                        let n = pseudo_noise_f32(x as u32, y as u32, 909);
                        let base = 155.0 + (brush - 0.5) * 22.0 + (n - 0.5) * 12.0;
                        let r = base.clamp(110.0, 210.0) as u8;
                        let g = (base * 1.01).clamp(112.0, 212.0) as u8;
                        let b = (base * 1.03).clamp(115.0, 215.0) as u8;
                        bytes[idx] = r;
                        bytes[idx + 1] = g;
                        bytes[idx + 2] = b;
                        bytes[idx + 3] = 255;
                    }
                }
            }
        }
        SurfaceType::Sand => {
            // Wind-rippled dune contours with loose fine grain
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let wave = ((y as f32 / height as f32) * std::f32::consts::PI * 6.0).sin();
                    let n = pseudo_noise_f32(x as u32, y as u32, 1010);
                    let base = 190.0 + wave * 18.0 + (n - 0.5) * 16.0;

                    bytes[idx] = base.clamp(140.0, 235.0) as u8;
                    bytes[idx + 1] = (base * 0.86).clamp(120.0, 205.0) as u8;
                    bytes[idx + 2] = (base * 0.60).clamp(80.0, 155.0) as u8;
                    bytes[idx + 3] = 255;
                }
            }
        }
        SurfaceType::Mud => {
            // Saturated chocolate churned earth with glossy wet highlight pools
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let n = pseudo_noise_f32(x as u32, y as u32, 1111);
                    let wet_pool = pseudo_noise_f32(x as u32 / 8, y as u32 / 8, 1212);
                    let is_wet = wet_pool > 0.68;

                    let (r, g, b) = if is_wet {
                        // Glossy dark specular sheen
                        let spec = (n * 35.0) as u8;
                        (68 + spec, 45 + spec, 30 + spec)
                    } else {
                        let base = 62.0 + (n - 0.5) * 18.0;
                        (base as u8, (base * 0.72) as u8, (base * 0.45) as u8)
                    };

                    bytes[idx] = r;
                    bytes[idx + 1] = g;
                    bytes[idx + 2] = b;
                    bytes[idx + 3] = 255;
                }
            }
        }
        SurfaceType::Snow => {
            // Crystalline powder snow with blue-sky ambient tint and subtle glints
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let n = pseudo_noise_f32(x as u32, y as u32, 1313);
                    let is_glint = n > 0.96;

                    let (r, g, b) = if is_glint {
                        (255, 255, 255)
                    } else {
                        let v = 228.0 + n * 20.0;
                        ((v * 0.96) as u8, (v * 0.98) as u8, v as u8)
                    };

                    bytes[idx] = r;
                    bytes[idx + 1] = g;
                    bytes[idx + 2] = b;
                    bytes[idx + 3] = 255;
                }
            }
        }
        SurfaceType::Ice => {
            // Glazed frozen surface with hairline fractures and translucent icy blue body
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let crack = ((x as i32 * 3 + y as i32 * 7) % 67).abs() == 0;
                    let n = pseudo_noise_f32(x as u32, y as u32, 1414);

                    let (r, g, b, a) = if crack {
                        (245, 252, 255, 250)
                    } else {
                        let base = 195.0 + n * 30.0;
                        ((base * 0.88) as u8, (base * 0.95) as u8, base as u8, 230)
                    };

                    bytes[idx] = r;
                    bytes[idx + 1] = g;
                    bytes[idx + 2] = b;
                    bytes[idx + 3] = a;
                }
            }
        }
        SurfaceType::Water => {
            // Clear reflective puddle with subtle wind ripple caustics
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let wave = (((x as f32 * 0.15).sin() + (y as f32 * 0.18).cos()) * 0.5 + 0.5) * 35.0;
                    bytes[idx] = (40.0 + wave * 0.6) as u8;
                    bytes[idx + 1] = (115.0 + wave) as u8;
                    bytes[idx + 2] = (175.0 + wave * 0.8) as u8;
                    bytes[idx + 3] = 220;
                }
            }
        }
        SurfaceType::Oil => {
            // Dark viscous puddle with iridescent thin-film rainbow fringes
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let dist_center = ((x as f32 - width as f32 * 0.5).powi(2)
                        + (y as f32 - height as f32 * 0.5).powi(2))
                    .sqrt();
                    let fringe = (dist_center * 0.25).sin() * 0.5 + 0.5;

                    let r = (20.0 + fringe * 45.0) as u8;
                    let g = (18.0 + (1.0 - fringe) * 35.0) as u8;
                    let b = (25.0 + (fringe * 0.5) * 55.0) as u8;

                    bytes[idx] = r;
                    bytes[idx + 1] = g;
                    bytes[idx + 2] = b;
                    bytes[idx + 3] = 245;
                }
            }
        }
        SurfaceType::Curb => {
            return generate_curb_image(width, height);
        }
    }

    Image {
        bytes,
        width,
        height,
    }
}

/// Generates a beveled red-and-white rumble curb teeth Image with 2.5D drop shadow relief.
pub fn generate_curb_image(width: u16, height: u16) -> Image {
    let mut bytes = vec![0u8; width as usize * height as usize * 4];
    let half_w = width / 2;

    for y in 0..height {
        let v_rel = y as f32 / height as f32; // 0 = inner track edge, 1 = outer edge
        // Bevel lighting: upper edge highlighted, lower outer edge shadowed
        let bevel_shade = 1.15 - v_rel * 0.35;

        for x in 0..width {
            let idx = (y as usize * width as usize + x as usize) * 4;
            let is_red = x < half_w;

            let (r, g, b) = if is_red {
                (
                    (215.0 * bevel_shade).clamp(0.0, 255.0) as u8,
                    (38.0 * bevel_shade).clamp(0.0, 255.0) as u8,
                    (38.0 * bevel_shade).clamp(0.0, 255.0) as u8,
                )
            } else {
                let w = (240.0 * bevel_shade).clamp(0.0, 255.0) as u8;
                (w, w, w)
            };

            bytes[idx] = r;
            bytes[idx + 1] = g;
            bytes[idx + 2] = b;
            bytes[idx + 3] = 255;
        }
    }

    Image {
        bytes,
        width,
        height,
    }
}

/// Generates an organic noise alpha mask for soft runoff edge feathering.
pub fn generate_edge_fringe_mask(width: u16, height: u16) -> Image {
    let mut bytes = vec![0u8; width as usize * height as usize * 4];

    for y in 0..height {
        let falloff = y as f32 / height as f32;
        for x in 0..width {
            let idx = (y as usize * width as usize + x as usize) * 4;
            let n = pseudo_noise_f32(x as u32, y as u32, 1515);
            let alpha = ((falloff + (n - 0.5) * 0.35).clamp(0.0, 1.0) * 255.0) as u8;

            bytes[idx] = 255;
            bytes[idx + 1] = 255;
            bytes[idx + 2] = 255;
            bytes[idx + 3] = alpha;
        }
    }

    Image {
        bytes,
        width,
        height,
    }
}

/// Generates low-frequency macro noise field for repetition breaking over long track straights.
pub fn generate_macro_noise_image(width: u16, height: u16) -> Image {
    let mut bytes = vec![0u8; width as usize * height as usize * 4];

    for y in 0..height {
        for x in 0..width {
            let idx = (y as usize * width as usize + x as usize) * 4;
            let n1 = pseudo_noise_f32(x as u32 / 8, y as u32 / 8, 1616);
            let n2 = pseudo_noise_f32(x as u32 / 16, y as u32 / 16, 1717);
            let val = ((0.5 + (n1 - 0.5) * 0.3 + (n2 - 0.5) * 0.4).clamp(0.0, 1.0) * 255.0) as u8;

            bytes[idx] = val;
            bytes[idx + 1] = val;
            bytes[idx + 2] = val;
            bytes[idx + 3] = 255;
        }
    }

    Image {
        bytes,
        width,
        height,
    }
}

/// Continuous multi-octave trigonometric value field for macro-modulation repetition breaking.
/// Operates at 32m - 64m spatial wavelength and returns a normalized offset in [-1.0, 1.0].
#[inline]
pub fn evaluate_macro_modulation(x: f32, y: f32) -> f32 {
    let w1 = (x * 0.13 + y * 0.09).sin() * (x * 0.07 - y * 0.11).cos();
    let w2 = 0.5 * (x * 0.19 - y * 0.15 + 1.2).sin();
    let w3 = 0.25 * (x * 0.08 + y * 0.14 - 0.7).cos();
    (w1 + w2 + w3) / 1.75
}

/// Dynamic track surface wear state (Phase 2 dynamic evolution hook).
/// Tracks cumulative rubber deposition and marble accumulation per track segment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrackWearState {
    /// Normalized dynamic rubber accumulation factor per segment [0.0, 1.0].
    pub segment_rubber: Vec<f32>,
    /// Off-line tire marble debris accumulation factor per segment [0.0, 1.0].
    pub segment_marbles: Vec<f32>,
}

impl TrackWearState {
    pub fn new(segment_count: usize) -> Self {
        Self {
            segment_rubber: vec![0.0; segment_count],
            segment_marbles: vec![0.0; segment_count],
        }
    }

    /// Records dynamic tire friction work onto a track segment.
    pub fn deposit_rubber(&mut self, segment_index: usize, slip_work: f32) {
        if let Some(r) = self.segment_rubber.get_mut(segment_index) {
            *r = (*r + slip_work * 0.001).clamp(0.0, 1.0);
        }
    }

    /// Accumulates loose marbles off-line on a track segment.
    pub fn accumulate_marbles(&mut self, segment_index: usize, debris: f32) {
        if let Some(m) = self.segment_marbles.get_mut(segment_index) {
            *m = (*m + debris * 0.001).clamp(0.0, 1.0);
        }
    }

    pub fn get_rubber(&self, segment_index: usize) -> f32 {
        self.segment_rubber.get(segment_index).copied().unwrap_or(0.0)
    }

    pub fn get_marbles(&self, segment_index: usize) -> f32 {
        self.segment_marbles.get(segment_index).copied().unwrap_or(0.0)
    }
}
