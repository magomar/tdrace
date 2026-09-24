use std::collections::HashMap;
use macroquad::color::Color;
use macroquad::texture::{Image, Texture2D};
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
            SurfaceType::PackedSand => (5.0, Color::new(1.0, 1.0, 1.0, 1.0), 0.75, true),
            SurfaceType::DeepSand => (6.0, Color::new(1.0, 1.0, 1.0, 1.0), 0.85, true),
            SurfaceType::MudTrack => (3.5, Color::new(1.0, 1.0, 1.0, 1.0), 0.50, true),
            SurfaceType::DeepMud => (3.0, Color::new(1.0, 1.0, 1.0, 1.0), 0.40, true),
            SurfaceType::PackedSnow => (4.5, Color::new(1.0, 1.0, 1.0, 1.0), 0.65, true),
            SurfaceType::DeepSnow => (5.5, Color::new(1.0, 1.0, 1.0, 1.0), 0.75, true),
            SurfaceType::SheetIce => (4.0, Color::new(1.0, 1.0, 1.0, 0.90), 0.15, false),
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
    tire_rubber_texture: Option<Texture2D>,
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
            tire_rubber_texture: None,
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
                self.tire_rubber_texture = None;
            } else {
                self.load_or_generate_all();
            }
        }
    }

    pub fn tire_rubber_texture(&self) -> Option<&Texture2D> {
        if self.quality == SurfaceTextureQuality::Off {
            None
        } else {
            self.tire_rubber_texture.as_ref()
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
            SurfaceType::PackedSand,
            SurfaceType::DeepSand,
            SurfaceType::Water,
            SurfaceType::Oil,
            SurfaceType::SheetIce,
            SurfaceType::MudTrack,
            SurfaceType::DeepMud,
            SurfaceType::PackedSnow,
            SurfaceType::DeepSnow,
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
                Self::set_texture_repeat_and_mipmaps(&t);
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

        // Tire rubber texture
        let rubber_tex = if let Some(bytes) = Self::find_surface_asset_file("tire_rubber.png") {
            if let Ok(tex) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let t = Texture2D::from_file_with_format(&bytes, None);
                Self::set_texture_repeat_and_mipmaps(&t);
                t
            })) {
                Some(tex)
            } else {
                let rubber_img = generate_tire_rubber_image(256, 512);
                Self::upload_texture(&rubber_img)
            }
        } else {
            let rubber_img = generate_tire_rubber_image(256, 512);
            Self::upload_texture(&rubber_img)
        };
        self.tire_rubber_texture = rubber_tex;
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
            SurfaceType::PackedSand => "sand_compacted.png",
            SurfaceType::DeepSand => "sand_dune.png",
            SurfaceType::Water => "water_caustic.png",
            SurfaceType::Oil => "oil_iridescent.png",
            SurfaceType::SheetIce => "ice_glazed.png",
            SurfaceType::MudTrack => "mud_compacted.png",
            SurfaceType::DeepMud => "mud_viscous.png",
            SurfaceType::PackedSnow => "snow_packed.png",
            SurfaceType::DeepSnow => "snow_powder.png",
            SurfaceType::Gravel => "gravel_crushed.png",
            SurfaceType::Concrete => "concrete_brushed.png",
        };

        if let Some(bytes) = Self::find_surface_asset_file(filename) {
            if let Ok(tex) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let t = Texture2D::from_file_with_format(&bytes, None);
                Self::set_texture_repeat_and_mipmaps(&t);
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
                Self::set_texture_repeat_and_mipmaps(&t);
                t
            })) {
                return Some(tex);
            }
        }
        let img = generate_curb_image(dim, dim / 2);
        Self::upload_texture(&img)
    }

    fn set_texture_repeat_and_mipmaps(tex: &Texture2D) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            unsafe {
                let gl_ctx = macroquad::window::get_internal_gl();
                let id = tex.raw_miniquad_id();
                gl_ctx.quad_context.texture_generate_mipmaps(id);
                gl_ctx.quad_context.texture_set_filter(
                    id,
                    macroquad::miniquad::FilterMode::Linear,
                    macroquad::miniquad::MipmapFilterMode::Linear,
                );
                gl_ctx.quad_context.texture_set_wrap(
                    id,
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
            Self::set_texture_repeat_and_mipmaps(&tex);
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
                    let jitter = (pseudo_noise_f32(x as u32, y as u32, 303) - 0.5) * 3.0;

                    let mut base = 35.0 + n_coarse * 6.0 + n_fine * 3.0 + jitter;
                    // Low-contrast crushed aggregate specks (soft +5 luminance jump, anti-aliased)
                    if pseudo_noise_f32(x as u32, y as u32, 404) > 0.97 {
                        base += 5.0;
                    }
                    let base_clamped = base.clamp(28.0, 48.0);

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
                    let n_coarse = sample_periodic_noise(
                        x as f32 * (32.0 / width as f32),
                        y as f32 * (32.0 / height as f32),
                        32,
                        32,
                        303,
                    );
                    let n_fine = sample_periodic_noise(
                        x as f32 * (64.0 / width as f32),
                        y as f32 * (64.0 / height as f32),
                        64,
                        64,
                        404,
                    );
                    let rut_wave = ((x as f32 / width as f32) * std::f32::consts::PI * 8.0).sin();

                    let base_lum = 0.58 + rut_wave * 0.04 + (n_coarse - 0.5) * 0.07 + (n_fine - 0.5) * 0.04;
                    let r = (145.0 * base_lum).clamp(80.0, 190.0) as u8;
                    let g = (100.0 * base_lum).clamp(55.0, 140.0) as u8;
                    let b = (62.0 * base_lum).clamp(32.0, 95.0) as u8;

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

                    let r = (32.0 + val * 16.0).clamp(28.0, 60.0) as u8;
                    let g = (86.0 + val * 30.0).clamp(76.0, 130.0) as u8;
                    let b = (36.0 + val * 18.0).clamp(30.0, 68.0) as u8;

                    bytes[idx] = r;
                    bytes[idx + 1] = g;
                    bytes[idx + 2] = b;
                    bytes[idx + 3] = 255;
                }
            }
        }
        SurfaceType::Gravel => {
            // Crushed angular limestone & slate scree: multi-scale seamless periodic pebbles
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let n_coarse = sample_periodic_noise(
                        x as f32 * (32.0 / width as f32),
                        y as f32 * (32.0 / height as f32),
                        32,
                        32,
                        707,
                    );
                    let n_pebbles = sample_periodic_noise(
                        x as f32 * (64.0 / width as f32),
                        y as f32 * (64.0 / height as f32),
                        64,
                        64,
                        808,
                    );
                    let n_speck = sample_periodic_noise(
                        x as f32 * (128.0 / width as f32),
                        y as f32 * (128.0 / height as f32),
                        128,
                        128,
                        909,
                    );

                    let base = 135.0 + (n_coarse - 0.5) * 14.0 + (n_pebbles - 0.5) * 12.0 + (n_speck - 0.5) * 8.0;
                    let r = base.clamp(100.0, 170.0) as u8;
                    let g = (base * 0.98).clamp(98.0, 168.0) as u8;
                    let b = (base * 0.94).clamp(94.0, 165.0) as u8;

                    bytes[idx] = r;
                    bytes[idx + 1] = g;
                    bytes[idx + 2] = b;
                    bytes[idx + 3] = 255;
                }
            }
        }
        SurfaceType::Concrete => {
            // Brushed industrial slabs: seamless directional micro-grooves and aggregate grain
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let n_brush = sample_periodic_noise(
                        x as f32 * (128.0 / width as f32),
                        y as f32 * (16.0 / height as f32),
                        128,
                        16,
                        1011,
                    );
                    let n_grain = sample_periodic_noise(
                        x as f32 * (64.0 / width as f32),
                        y as f32 * (64.0 / height as f32),
                        64,
                        64,
                        1112,
                    );
                    let base = 158.0 + (n_brush - 0.5) * 14.0 + (n_grain - 0.5) * 10.0;
                    let r = base.clamp(130.0, 190.0) as u8;
                    let g = (base * 1.01).clamp(132.0, 192.0) as u8;
                    let b = (base * 1.03).clamp(135.0, 195.0) as u8;

                    bytes[idx] = r;
                    bytes[idx + 1] = g;
                    bytes[idx + 2] = b;
                    bytes[idx + 3] = 255;
                }
            }
        }
        SurfaceType::PackedSand => {
            // Firmly compacted desert sand with smooth rolling grain and subtle grading
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let n_firm = sample_periodic_noise(
                        x as f32 * (32.0 / width as f32),
                        y as f32 * (32.0 / height as f32),
                        32,
                        32,
                        1200,
                    );
                    let n_fine = sample_periodic_noise(
                        x as f32 * (64.0 / width as f32),
                        y as f32 * (64.0 / height as f32),
                        64,
                        64,
                        1250,
                    );
                    let base = 205.0 + (n_firm - 0.5) * 14.0 + (n_fine - 0.5) * 6.0;

                    bytes[idx] = base.clamp(170.0, 235.0) as u8;
                    bytes[idx + 1] = (base * 0.88).clamp(150.0, 210.0) as u8;
                    bytes[idx + 2] = (base * 0.64).clamp(105.0, 155.0) as u8;
                    bytes[idx + 3] = 255;
                }
            }
        }
        SurfaceType::DeepSand => {
            // Wind-rippled dune contours with loose fine grain
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let wave = ((y as f32 / height as f32) * std::f32::consts::PI * 8.0).sin();
                    let n_dune = sample_periodic_noise(
                        x as f32 * (16.0 / width as f32),
                        y as f32 * (16.0 / height as f32),
                        16,
                        16,
                        1213,
                    );
                    let n_grain = sample_periodic_noise(
                        x as f32 * (64.0 / width as f32),
                        y as f32 * (64.0 / height as f32),
                        64,
                        64,
                        1314,
                    );
                    let base = 192.0 + wave * 9.0 + (n_dune - 0.5) * 12.0 + (n_grain - 0.5) * 8.0;

                    bytes[idx] = base.clamp(150.0, 225.0) as u8;
                    bytes[idx + 1] = (base * 0.86).clamp(130.0, 198.0) as u8;
                    bytes[idx + 2] = (base * 0.60).clamp(90.0, 145.0) as u8;
                    bytes[idx + 3] = 255;
                }
            }
        }
        SurfaceType::MudTrack => {
            // Graded, damp compacted mud with consistent earthy brown grain
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let n_earth = sample_periodic_noise(
                        x as f32 * (32.0 / width as f32),
                        y as f32 * (32.0 / height as f32),
                        32,
                        32,
                        1400,
                    );
                    let n_grain = sample_periodic_noise(
                        x as f32 * (64.0 / width as f32),
                        y as f32 * (64.0 / height as f32),
                        64,
                        64,
                        1450,
                    );
                    let base = 78.0 + (n_earth - 0.5) * 12.0 + (n_grain - 0.5) * 6.0;

                    bytes[idx] = base.clamp(55.0, 105.0) as u8;
                    bytes[idx + 1] = (base * 0.74).clamp(40.0, 80.0) as u8;
                    bytes[idx + 2] = (base * 0.48).clamp(25.0, 55.0) as u8;
                    bytes[idx + 3] = 255;
                }
            }
        }
        SurfaceType::DeepMud => {
            // Saturated chocolate churned earth with glossy wet specular sheen
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let n_silt = sample_periodic_noise(
                        x as f32 * (32.0 / width as f32),
                        y as f32 * (32.0 / height as f32),
                        32,
                        32,
                        1415,
                    );
                    let wet_pool = sample_periodic_noise(
                        x as f32 * (16.0 / width as f32),
                        y as f32 * (16.0 / height as f32),
                        16,
                        16,
                        1516,
                    );
                    let is_wet = wet_pool > 0.65;

                    let (r, g, b) = if is_wet {
                        let spec = (n_silt * 20.0) as u8;
                        (62 + spec, 42 + spec, 28 + spec)
                    } else {
                        let base = 62.0 + (n_silt - 0.5) * 10.0;
                        (base as u8, (base * 0.72) as u8, (base * 0.45) as u8)
                    };

                    bytes[idx] = r;
                    bytes[idx + 1] = g;
                    bytes[idx + 2] = b;
                    bytes[idx + 3] = 255;
                }
            }
        }
        SurfaceType::PackedSnow => {
            // Groomed, hard-packed snow trail with light track striations and cold blue cast
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let n_pack = sample_periodic_noise(
                        x as f32 * (32.0 / width as f32),
                        y as f32 * (32.0 / height as f32),
                        32,
                        32,
                        1600,
                    );
                    let n_fine = sample_periodic_noise(
                        x as f32 * (64.0 / width as f32),
                        y as f32 * (64.0 / height as f32),
                        64,
                        64,
                        1650,
                    );
                    let v = 225.0 + (n_pack - 0.5) * 10.0 + (n_fine - 0.5) * 5.0;

                    bytes[idx] = (v * 0.94).clamp(205.0, 245.0) as u8;
                    bytes[idx + 1] = (v * 0.97).clamp(210.0, 250.0) as u8;
                    bytes[idx + 2] = v.clamp(215.0, 255.0) as u8;
                    bytes[idx + 3] = 255;
                }
            }
        }
        SurfaceType::DeepSnow => {
            // Crystalline powder snow with blue-sky ambient tint and subtle glints
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let n_drift = sample_periodic_noise(
                        x as f32 * (16.0 / width as f32),
                        y as f32 * (16.0 / height as f32),
                        16,
                        16,
                        1617,
                    );
                    let n_sparkle = sample_periodic_noise(
                        x as f32 * (64.0 / width as f32),
                        y as f32 * (64.0 / height as f32),
                        64,
                        64,
                        1718,
                    );
                    let v = 232.0 + (n_drift - 0.5) * 14.0 + (n_sparkle - 0.5) * 6.0;

                    bytes[idx] = (v * 0.96).clamp(210.0, 255.0) as u8;
                    bytes[idx + 1] = (v * 0.98).clamp(215.0, 255.0) as u8;
                    bytes[idx + 2] = v.clamp(220.0, 255.0) as u8;
                    bytes[idx + 3] = 255;
                }
            }
        }
        SurfaceType::SheetIce => {
            // Glazed frozen surface with hairline fractures and translucent icy blue body
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let n_sheet = sample_periodic_noise(
                        x as f32 * (16.0 / width as f32),
                        y as f32 * (16.0 / height as f32),
                        16,
                        16,
                        1819,
                    );
                    let n_rime = sample_periodic_noise(
                        x as f32 * (64.0 / width as f32),
                        y as f32 * (64.0 / height as f32),
                        64,
                        64,
                        1920,
                    );
                    let base = 200.0 + (n_sheet - 0.5) * 16.0 + (n_rime - 0.5) * 8.0;

                    bytes[idx] = (base * 0.88).clamp(160.0, 230.0) as u8;
                    bytes[idx + 1] = (base * 0.95).clamp(175.0, 240.0) as u8;
                    bytes[idx + 2] = base.clamp(185.0, 250.0) as u8;
                    bytes[idx + 3] = 240;
                }
            }
        }
        SurfaceType::Water => {
            // Clear reflective puddle with subtle wind ripple caustics
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let wave1 = ((x as f32 / width as f32) * std::f32::consts::PI * 6.0).sin();
                    let wave2 = ((y as f32 / height as f32) * std::f32::consts::PI * 8.0).cos();
                    let n_caustic = sample_periodic_noise(
                        x as f32 * (32.0 / width as f32),
                        y as f32 * (32.0 / height as f32),
                        32,
                        32,
                        2021,
                    );
                    let wave = ((wave1 + wave2) * 0.5 + 0.5) * 18.0 + (n_caustic - 0.5) * 12.0;

                    bytes[idx] = (40.0 + wave * 0.5).clamp(30.0, 75.0) as u8;
                    bytes[idx + 1] = (115.0 + wave * 0.8).clamp(95.0, 150.0) as u8;
                    bytes[idx + 2] = (175.0 + wave).clamp(150.0, 215.0) as u8;
                    bytes[idx + 3] = 225;
                }
            }
        }
        SurfaceType::Oil => {
            // Dark viscous puddle with iridescent thin-film rainbow fringes
            for y in 0..height {
                for x in 0..width {
                    let idx = (y as usize * width as usize + x as usize) * 4;
                    let n_swirl1 = sample_periodic_noise(
                        x as f32 * (16.0 / width as f32),
                        y as f32 * (16.0 / height as f32),
                        16,
                        16,
                        2122,
                    );
                    let n_swirl2 = sample_periodic_noise(
                        x as f32 * (32.0 / width as f32),
                        y as f32 * (32.0 / height as f32),
                        32,
                        32,
                        2223,
                    );
                    let fringe = (n_swirl1 * 0.70 + n_swirl2 * 0.30).clamp(0.0, 1.0);

                    let r = (24.0 + fringe * 30.0) as u8;
                    let g = (22.0 + (1.0 - fringe) * 24.0) as u8;
                    let b = (28.0 + fringe * 36.0) as u8;

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
            let n = sample_periodic_noise(
                x as f32 * (32.0 / width as f32),
                y as f32 * (32.0 / height as f32),
                32,
                32,
                1515,
            );
            let alpha = ((falloff + (n - 0.5) * 0.28).clamp(0.0, 1.0) * 255.0) as u8;

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
            let n1 = sample_periodic_noise(
                x as f32 * (8.0 / width as f32),
                y as f32 * (8.0 / height as f32),
                8,
                8,
                1616,
            );
            let n2 = sample_periodic_noise(
                x as f32 * (16.0 / width as f32),
                y as f32 * (16.0 / height as f32),
                16,
                16,
                1717,
            );
            let val = ((0.5 + (n1 - 0.5) * 0.25 + (n2 - 0.5) * 0.25).clamp(0.0, 1.0) * 255.0) as u8;

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

/// Generates a realistic tire rubber contact texture with multi-rib tread striations,
/// feathered organic edges, rubber clumping, and slip chatter.
pub fn generate_tire_rubber_image(width: u16, height: u16) -> Image {
    let mut bytes = vec![0u8; width as usize * height as usize * 4];

    for y in 0..height {
        let v = y as f32 / height as f32;
        // Periodic slip chatter along longitudinal rolling direction
        let chatter = sample_periodic_noise(0.0, v * 16.0, 1, 16, 777);
        let clump = sample_periodic_noise(0.0, v * 32.0, 1, 32, 888);

        for x in 0..width {
            let idx = (y as usize * width as usize + x as usize) * 4;
            let u = x as f32 / width as f32;

            // Feathered soft outer contact edges (at u = 0.0 and u = 1.0)
            let edge_dist = (u * 6.0).min((1.0 - u) * 6.0).clamp(0.0, 1.0);
            let edge_feather = smoothstep(edge_dist);

            // Multi-rib tread contact striations across the contact patch
            let rib_wave = ((u * std::f32::consts::PI * 6.0).sin().abs()).powf(0.7);
            let tread_profile = 0.45 + rib_wave * 0.55;

            // Micro-grain and molten rubber clumping
            let grain = sample_periodic_noise(
                x as f32 * (64.0 / width as f32),
                y as f32 * (64.0 / height as f32),
                64,
                64,
                999,
            );
            let rubber_density = (tread_profile * (0.82 + chatter * 0.22 + clump * 0.16) + (grain - 0.5) * 0.12).clamp(0.0, 1.0);

            let alpha = (edge_feather * rubber_density * 255.0).clamp(0.0, 255.0) as u8;

            // Pure white RGB so vertex color multiplier renders exact rich tint
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
