use std::fs;
use std::path::Path;
use tdrace_app::render::{
    generate_curb_image, generate_edge_fringe_mask, generate_surface_image,
    generate_tire_rubber_image,
};
use tdrace_core::physics::surface::SurfaceType;

fn main() {
    let out_dirs = [
        Path::new("assets/textures/surfaces"),
        Path::new("../assets/textures/surfaces"),
        Path::new("../../assets/textures/surfaces"),
    ];

    let target_dir = out_dirs
        .iter()
        .find(|p| p.parent().map(|par| par.exists()).unwrap_or(false))
        .copied()
        .unwrap_or_else(|| Path::new("assets/textures/surfaces"));

    fs::create_dir_all(target_dir).expect("Failed to create assets/textures/surfaces directory");
    println!("Generating surface textures into: {:?}", target_dir);

    let surfaces = [
        (SurfaceType::Asphalt, "asphalt_diffuse.png", 512),
        (SurfaceType::Dirt, "dirt_compacted.png", 512),
        (SurfaceType::Grass, "grass_turf.png", 512),
        (SurfaceType::Gravel, "gravel_crushed.png", 512),
        (SurfaceType::PackedSand, "sand_compacted.png", 512),
        (SurfaceType::DeepSand, "sand_dune.png", 512),
        (SurfaceType::MudTrack, "mud_compacted.png", 512),
        (SurfaceType::DeepMud, "mud_viscous.png", 512),
        (SurfaceType::PackedSnow, "snow_packed.png", 512),
        (SurfaceType::DeepSnow, "snow_powder.png", 512),
        (SurfaceType::SheetIce, "ice_glazed.png", 512),
        (SurfaceType::Water, "water_caustic.png", 256),
        (SurfaceType::Oil, "oil_iridescent.png", 256),
        (SurfaceType::Concrete, "concrete_brushed.png", 512),
    ];

    for (surf, filename, dim) in surfaces {
        let img = generate_surface_image(surf, dim, dim);
        let path = target_dir.join(filename);
        let path_str = path.to_str().unwrap();
        img.export_png(path_str);
        println!("  ✓ Saved {} ({}x{})", filename, dim, dim);
    }

    // Curb teeth
    let curb_img = generate_curb_image(256, 128);
    let curb_path = target_dir.join("curb_teeth.png");
    curb_img.export_png(curb_path.to_str().unwrap());
    println!("  ✓ Saved curb_teeth.png (256x128)");

    // Edge fringe alpha mask
    let fringe_img = generate_edge_fringe_mask(256, 256);
    let fringe_path = target_dir.join("edge_fringe_mask.png");
    fringe_img.export_png(fringe_path.to_str().unwrap());
    println!("  ✓ Saved edge_fringe_mask.png (256x256)");

    // Dedicated realistic tire rubber contact texture
    let rubber_img = generate_tire_rubber_image(256, 512);
    let rubber_path = target_dir.join("tire_rubber.png");
    rubber_img.export_png(rubber_path.to_str().unwrap());
    println!("  ✓ Saved tire_rubber.png (256x512)");

    // Asphalt racing groove overlay (dark rubber streak)
    let mut groove_bytes = vec![0u8; 512 * 512 * 4];
    for y in 0..512 {
        for x in 0..512 {
            let idx = (y * 512 + x) * 4;
            // Central groove Gaussian profile
            let u_norm = (x as f32 / 512.0) - 0.5;
            let dist_sq = u_norm * u_norm;
            let groove_alpha = (-dist_sq / (2.0 * 0.15 * 0.15)).exp();
            let a = (groove_alpha * 160.0).clamp(0.0, 180.0) as u8;

            groove_bytes[idx] = 12; // deep rubber black
            groove_bytes[idx + 1] = 12;
            groove_bytes[idx + 2] = 14;
            groove_bytes[idx + 3] = a;
        }
    }
    let groove_img = macroquad::texture::Image {
        bytes: groove_bytes,
        width: 512,
        height: 512,
    };
    let groove_path = target_dir.join("asphalt_groove.png");
    groove_img.export_png(groove_path.to_str().unwrap());
    println!("  ✓ Saved asphalt_groove.png (512x512)");

    println!("All surface textures successfully generated!");
}
