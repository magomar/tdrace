#!/usr/bin/env python3
"""
TdRace - Dedicated Realistic 3D-Lit Top-Down Steered Wheel Sprite Generator
Generates pure, authentic 128x256 top-down tyre tread sprites with ZERO artificial
indicators, rims, or hub graphics:
- gt_slick_front.png
- nascar_wheel_front.png
- rally_wheel_front.png
- kart_slick_front.png
- buggy_allterrain_front.png
- truck_allterrain_front.png
- monster_wheel_front.png
- mud_tractor_front.png
- offroad_wheel_front.png
"""

import math
from pathlib import Path

import numpy as np
from PIL import Image
from scipy.ndimage import gaussian_filter


def create_base_carcass(W, H, tw_half, th_half, corner_r):
    """
    Returns:
    - mask: (H, W) float in [0, 1] for tire carcass boundary
    - z_cyl: (H, W) float height representing cylindrical curvature of the rolling tire
    - z_shoulder: (H, W) float height representing curved shoulder profile across width
    """
    y_coords, x_coords = np.mgrid[0:H, 0:W]
    cx, cy = W / 2.0, H / 2.0
    
    dx = np.abs(x_coords - cx)
    dy = np.abs(y_coords - cy)
    
    # Distance to rounded rectangle
    rx = np.maximum(0.0, dx - (tw_half - corner_r))
    ry = np.maximum(0.0, dy - (th_half - corner_r))
    dist = np.sqrt(rx * rx + ry * ry)
    
    qx = dx - (tw_half - corner_r)
    qy = dy - (th_half - corner_r)
    outside_dist = np.where((qx > 0) & (qy > 0), dist - corner_r, np.maximum(dx - tw_half, dy - th_half))
    
    # Smooth anti-aliased edge alpha over 2.0 pixels
    mask = np.clip(0.5 - outside_dist / 2.0, 0.0, 1.0)
    
    # Cylindrical curvature along Y (rolling direction)
    ny = np.clip(dy / th_half, 0.0, 1.0)
    z_cyl = np.maximum(0.0, np.cos(ny * (math.pi / 2.0))) ** 0.55
    
    # Transverse shoulder curvature along X
    nx = np.clip(dx / tw_half, 0.0, 1.0)
    z_shoulder = np.where(
        nx < 0.68,
        1.0,
        np.maximum(0.0, np.cos(np.clip((nx - 0.68) / 0.32, 0.0, 1.0) * (math.pi / 2.0))) ** 0.45
    )
    
    return mask, z_cyl * z_shoulder


def compute_normals_and_shading(z_map, mask, light_dir=(-0.38, -0.48, 0.79), ambient=0.38, spec_power=18.0):
    """
    Computes 3D surface normals from a heightmap and applies Phong/ambient lighting with high relief.
    """
    dz_dy, dz_dx = np.gradient(z_map)
    
    scale = 6.0
    nx = -dz_dx * scale
    ny = -dz_dy * scale
    nz = np.ones_like(z_map)
    
    norm = np.sqrt(nx * nx + ny * ny + nz * nz)
    nx /= norm
    ny /= norm
    nz /= norm
    
    lx, ly, lz = light_dir
    l_norm = math.sqrt(lx * lx + ly * ly + lz * lz)
    lx, ly, lz = lx / l_norm, ly / l_norm, lz / l_norm
    
    # Diffuse N . L
    n_dot_l = np.clip(nx * lx + ny * ly + nz * lz, 0.0, 1.0)
    
    # Ambient Occlusion: grooves and depressions get deep shadow
    d2z_dy2, _ = np.gradient(dz_dy)
    _, d2z_dx2 = np.gradient(dz_dx)
    laplacian = d2z_dx2 + d2z_dy2
    ao = np.clip(1.0 + laplacian * 6.5, 0.22, 1.15)
    
    # Blinn-Phong specular (View = (0, 0, 1))
    hx, hy, hz = lx, ly, lz + 1.0
    h_norm = math.sqrt(hx * hx + hy * hy + hz * hz)
    hx, hy, hz = hx / h_norm, hy / h_norm, hz / h_norm
    
    n_dot_h = np.clip(nx * hx + ny * hy + nz * hz, 0.0, 1.0)
    specular = (n_dot_h ** spec_power) * 0.36
    
    diffuse = ambient + (1.0 - ambient) * n_dot_l
    lighting = (diffuse * ao + specular)
    
    return np.clip(lighting, 0.10, 1.35) * mask


def add_rubber_microtexture(H, W, scale=0.025):
    """Generates fine-grained vulcanized rubber micro-roughness."""
    np.random.seed(1337)
    noise = np.random.normal(0.0, 1.0, (H, W)).astype(np.float32)
    n_arr = gaussian_filter(noise, sigma=1.2)
    n_arr = (n_arr - n_arr.mean()) / (n_arr.std() + 1e-5)
    return n_arr * scale


def create_rubber_image(lighting, mask, base_rgb=(38.0, 42.0, 48.0)):
    r = np.clip(lighting * base_rgb[0], 0, 255).astype(np.uint8)
    g = np.clip(lighting * base_rgb[1], 0, 255).astype(np.uint8)
    b = np.clip(lighting * base_rgb[2], 0, 255).astype(np.uint8)
    a = (mask * 255.0).astype(np.uint8)
    return Image.fromarray(np.stack([r, g, b, a], axis=-1), mode="RGBA")


# ==============================================================================
# Pure Tyre Builders (Zero Rims, Zero Hubs, Zero Artificial Indicators)
# ==============================================================================

def build_gt_slick(ss=4):
    """GT Motorsport Slick: pure vulcanized racing rubber with subtle directional heat grain and twin rain sipes."""
    W, H = 128 * ss, 256 * ss
    cx, cy = W // 2, H // 2
    tw_half, th_half = int(57 * ss), int(119 * ss)
    corner_r = int(22 * ss)
    
    mask, z_base = create_base_carcass(W, H, tw_half, th_half, corner_r)
    
    _y_coords, x_coords = np.mgrid[0:H, 0:W]
    dx = np.abs(x_coords - cx)
    
    tread_h = np.zeros((H, W), dtype=np.float32)
    
    # Subtle longitudinal rain/expansion grooves at dx = 18*ss
    g_dist = np.abs(dx - 18 * ss)
    groove = np.clip(1.0 - g_dist / (2.2 * ss), 0.0, 1.0)
    tread_h -= groove * 0.16
    
    # Fine directional heat/marbles micro-grain
    tread_h += add_rubber_microtexture(H, W, 0.02)
    
    z_total = np.maximum(0.0, z_base + tread_h)
    lighting = compute_normals_and_shading(z_total, mask, ambient=0.42, spec_power=20.0)
    
    im = create_rubber_image(lighting, mask, base_rgb=(36.0, 40.0, 46.0))
    return im.resize((128, 256), Image.Resampling.LANCZOS)


def build_kart_slick(ss=4):
    """Sprint Kart Slick: pure squat, wide competition slick rubber with smooth cylindrical crown."""
    W, H = 128 * ss, 256 * ss
    tw_half, th_half = int(55 * ss), int(117 * ss)
    corner_r = int(22 * ss)
    
    mask, z_base = create_base_carcass(W, H, tw_half, th_half, corner_r)
    
    tread_h = add_rubber_microtexture(H, W, 0.02)
    z_total = np.maximum(0.0, z_base + tread_h)
    lighting = compute_normals_and_shading(z_total, mask, ambient=0.45, spec_power=22.0)
    
    im = create_rubber_image(lighting, mask, base_rgb=(35.0, 39.0, 45.0))
    return im.resize((128, 256), Image.Resampling.LANCZOS)


def build_nascar_wheel(ss=4):
    """Stock Car Slick: wide, thick Goodyear-style oval racing slick with uniform rubber surface."""
    W, H = 128 * ss, 256 * ss
    tw_half, th_half = int(58 * ss), int(121 * ss)
    corner_r = int(24 * ss)
    
    mask, z_base = create_base_carcass(W, H, tw_half, th_half, corner_r)
    
    tread_h = add_rubber_microtexture(H, W, 0.025)
    z_total = np.maximum(0.0, z_base + tread_h)
    lighting = compute_normals_and_shading(z_total, mask, ambient=0.43, spec_power=18.0)
    
    im = create_rubber_image(lighting, mask, base_rgb=(34.0, 38.0, 44.0))
    return im.resize((128, 256), Image.Resampling.LANCZOS)


def build_rally_wheel(ss=4):
    """Rally Mixed-Surface Tire: continuous directional diagonal siping and asymmetric tread blocks across the full contact patch."""
    W, H = 128 * ss, 256 * ss
    cx, cy = W // 2, H // 2
    tw_half, th_half = int(55 * ss), int(117 * ss)
    corner_r = int(22 * ss)
    
    mask, z_base = create_base_carcass(W, H, tw_half, th_half, corner_r)
    
    tread_h = np.zeros((H, W), dtype=np.float32)
    y_coords, x_coords = np.mgrid[0:H, 0:W]
    dx = x_coords - cx
    
    # Continuous interlocking diagonal sipes from top to bottom
    for dy in range(-int(th_half + 20 * ss), int(th_half + 20 * ss), int(16 * ss)):
        # Left angled grooves
        dist_l = np.abs((y_coords - cy - dy) - (dx * 0.40))
        groove_l = np.clip(1.0 - dist_l / (2.2 * ss), 0.0, 1.0) * (dx <= 2 * ss)
        # Right angled grooves (staggered)
        dist_r = np.abs((y_coords - cy - dy - 8 * ss) + (dx * 0.40))
        groove_r = np.clip(1.0 - dist_r / (2.2 * ss), 0.0, 1.0) * (dx >= -2 * ss)
        tread_h -= (groove_l + groove_r) * 0.28
    
    # Longitudinal dividing channel
    g_mid = np.abs(dx)
    groove_mid = np.clip(1.0 - g_mid / (2.0 * ss), 0.0, 1.0)
    tread_h -= groove_mid * 0.20
    
    tread_h = gaussian_filter(tread_h, sigma=1.4)
    tread_h += add_rubber_microtexture(H, W, 0.02)
    
    z_total = np.maximum(0.0, z_base + tread_h)
    lighting = compute_normals_and_shading(z_total, mask, ambient=0.40, spec_power=16.0)
    
    im = create_rubber_image(lighting, mask, base_rgb=(38.0, 42.0, 48.0))
    return im.resize((128, 256), Image.Resampling.LANCZOS)


def build_buggy_allterrain(ss=4):
    """Autocross Buggy All-Terrain: continuous aggressive staggered knobby dirt blocks across the full contact patch."""
    W, H = 128 * ss, 256 * ss
    cx, cy = W // 2, H // 2
    tw_half, th_half = int(55 * ss), int(119 * ss)
    corner_r = int(22 * ss)
    
    mask, z_base = create_base_carcass(W, H, tw_half, th_half, corner_r)
    
    tread_h = np.zeros((H, W), dtype=np.float32)
    y_coords, x_coords = np.mgrid[0:H, 0:W]
    dx = x_coords - cx
    
    # Continuous knobby tread blocks: left, right, and center rows from top to bottom
    spacing = int(18 * ss)
    for y_pos in range(-int(th_half + 10 * ss), int(th_half + 10 * ss), spacing):
        # Left outer shoulder lug
        lug_l = (np.abs(y_coords - cy - y_pos) < 5.5 * ss) & (dx < -18 * ss) & (dx > -50 * ss)
        # Right outer shoulder lug (staggered)
        lug_r = (np.abs(y_coords - cy - y_pos - spacing // 2) < 5.5 * ss) & (dx > 18 * ss) & (dx < 50 * ss)
        # Center dual knobby blocks
        mid_l = (np.abs(y_coords - cy - y_pos - spacing // 4) < 5.0 * ss) & (dx >= -15 * ss) & (dx <= -2 * ss)
        mid_r = (np.abs(y_coords - cy - y_pos - (3 * spacing) // 4) < 5.0 * ss) & (dx >= 2 * ss) & (dx <= 15 * ss)
        tread_h += np.where(lug_l | lug_r | mid_l | mid_r, 0.40, 0.0)
    
    tread_h = gaussian_filter(tread_h, sigma=1.6)
    tread_h += add_rubber_microtexture(H, W, 0.02)
    
    z_total = np.maximum(0.0, z_base + tread_h)
    lighting = compute_normals_and_shading(z_total, mask, ambient=0.38, spec_power=16.0)
    
    im = create_rubber_image(lighting, mask, base_rgb=(38.0, 42.0, 48.0))
    return im.resize((128, 256), Image.Resampling.LANCZOS)


def build_truck_allterrain(ss=4):
    """Trophy Truck All-Terrain: heavy interlocking Baja T/A style polygonal blocks covering the entire contact patch."""
    W, H = 128 * ss, 256 * ss
    cx, cy = W // 2, H // 2
    tw_half, th_half = int(58 * ss), int(121 * ss)
    corner_r = int(24 * ss)
    
    mask, z_base = create_base_carcass(W, H, tw_half, th_half, corner_r)
    
    tread_h = np.zeros((H, W), dtype=np.float32)
    y_coords, x_coords = np.mgrid[0:H, 0:W]
    dx = x_coords - cx
    
    spacing = int(20 * ss)
    for y_pos in range(-int(th_half + 10 * ss), int(th_half + 10 * ss), spacing):
        # Heavy shoulder cleats
        cleat_l = (np.abs(y_coords - cy - y_pos) < 6.5 * ss) & (dx < -20 * ss) & (dx > -54 * ss)
        cleat_r = (np.abs(y_coords - cy - y_pos - spacing // 2) < 6.5 * ss) & (dx > 20 * ss) & (dx < 54 * ss)
        # Interlocking center tread blocks
        mid_l = (np.abs(y_coords - cy - y_pos - 4 * ss) < 5.5 * ss) & (dx >= -18 * ss) & (dx <= 0)
        mid_r = (np.abs(y_coords - cy - y_pos - spacing // 2 - 4 * ss) < 5.5 * ss) & (dx >= 0) & (dx <= 18 * ss)
        tread_h += np.where(cleat_l | cleat_r | mid_l | mid_r, 0.42, 0.0)
    
    tread_h = gaussian_filter(tread_h, sigma=1.8)
    tread_h += add_rubber_microtexture(H, W, 0.02)
    
    z_total = np.maximum(0.0, z_base + tread_h)
    lighting = compute_normals_and_shading(z_total, mask, ambient=0.38, spec_power=16.0)
    
    im = create_rubber_image(lighting, mask, base_rgb=(38.0, 42.0, 48.0))
    return im.resize((128, 256), Image.Resampling.LANCZOS)


def build_monster_wheel(ss=4):
    """Monster Truck 66-inch Terra Tire: iconic continuous 45-degree chevron V-lugs spanning the entire contact patch."""
    W, H = 128 * ss, 256 * ss
    cx, cy = W // 2, H // 2
    tw_half, th_half = int(61 * ss), int(123 * ss)
    corner_r = int(26 * ss)
    
    mask, z_base = create_base_carcass(W, H, tw_half, th_half, corner_r)
    
    tread_h = np.zeros((H, W), dtype=np.float32)
    y_coords, x_coords = np.mgrid[0:H, 0:W]
    dx = x_coords - cx
    
    spacing = int(24 * ss)
    for y_pos in range(-int(th_half + 20 * ss), int(th_half + 20 * ss), spacing):
        # Left chevron bar angled forward toward center
        dist_l = np.abs((y_coords - cy - y_pos) - (-dx * 0.42))
        bar_l = (dist_l < 7.5 * ss) & (dx < -3 * ss)
        # Right chevron bar (staggered)
        dist_r = np.abs((y_coords - cy - y_pos - spacing // 2) - (dx * 0.42))
        bar_r = (dist_r < 7.5 * ss) & (dx > 3 * ss)
        tread_h += np.where(bar_l | bar_r, 0.50, 0.0)
    
    # Deep center mud trench
    g_dist = np.abs(dx)
    groove = np.clip(1.0 - g_dist / (3.5 * ss), 0.0, 1.0)
    tread_h -= groove * 0.22
    
    tread_h = gaussian_filter(tread_h, sigma=2.0)
    tread_h += add_rubber_microtexture(H, W, 0.02)
    
    z_total = np.maximum(0.0, z_base + tread_h)
    lighting = compute_normals_and_shading(z_total, mask, ambient=0.36, spec_power=14.0)
    
    im = create_rubber_image(lighting, mask, base_rgb=(38.0, 42.0, 48.0))
    return im.resize((128, 256), Image.Resampling.LANCZOS)


def build_mud_tractor(ss=4):
    """Mud Bogger Paddle Tire: deep directional paddle blades spanning the entire contact patch."""
    W, H = 128 * ss, 256 * ss
    cx, cy = W // 2, H // 2
    tw_half, th_half = int(59 * ss), int(122 * ss)
    corner_r = int(24 * ss)
    
    mask, z_base = create_base_carcass(W, H, tw_half, th_half, corner_r)
    
    tread_h = np.zeros((H, W), dtype=np.float32)
    y_coords, x_coords = np.mgrid[0:H, 0:W]
    dx = x_coords - cx
    
    spacing = int(22 * ss)
    for y_pos in range(-int(th_half + 20 * ss), int(th_half + 20 * ss), spacing):
        dist_l = np.abs((y_coords - cy - y_pos) - (-dx * 0.36))
        blade_l = (dist_l < 7.5 * ss) & (dx < -4 * ss)
        dist_r = np.abs((y_coords - cy - y_pos - spacing // 2) - (dx * 0.36))
        blade_r = (dist_r < 7.5 * ss) & (dx > 4 * ss)
        tread_h += np.where(blade_l | blade_r, 0.52, 0.0)
    
    # Deep mud channel
    g_dist = np.abs(dx)
    groove = np.clip(1.0 - g_dist / (4.0 * ss), 0.0, 1.0)
    tread_h -= groove * 0.24
    
    tread_h = gaussian_filter(tread_h, sigma=1.8)
    tread_h += add_rubber_microtexture(H, W, 0.02)
    
    z_total = np.maximum(0.0, z_base + tread_h)
    lighting = compute_normals_and_shading(z_total, mask, ambient=0.36, spec_power=14.0)
    
    im = create_rubber_image(lighting, mask, base_rgb=(38.0, 42.0, 48.0))
    return im.resize((128, 256), Image.Resampling.LANCZOS)


def build_offroad_wheel(ss=4):
    """High-Quality Off-Road / Sand Rail Tire: pure, razor-sharp chevron knobby tread (clean replacement for offroad_wheel_front)."""
    W, H = 128 * ss, 256 * ss
    cx, cy = W // 2, H // 2
    tw_half, th_half = int(45 * ss), int(118 * ss)
    corner_r = int(20 * ss)
    
    mask, z_base = create_base_carcass(W, H, tw_half, th_half, corner_r)
    
    tread_h = np.zeros((H, W), dtype=np.float32)
    y_coords, x_coords = np.mgrid[0:H, 0:W]
    dx = x_coords - cx
    
    # Continuous chevron dirt lugs running seamlessly from top to bottom
    spacing = int(15 * ss)
    for y_pos in range(-int(th_half + 15 * ss), int(th_half + 15 * ss), spacing):
        dist_l = np.abs((y_coords - cy - y_pos) - (-dx * 0.44))
        lug_l = (dist_l < 4.8 * ss) & (dx < -2 * ss) & (dx > -42 * ss)
        dist_r = np.abs((y_coords - cy - y_pos - spacing // 2) - (dx * 0.44))
        lug_r = (dist_r < 4.8 * ss) & (dx > 2 * ss) & (dx < 42 * ss)
        ridge = (np.abs(y_coords - cy - y_pos - spacing // 4) < 3.2 * ss) & (np.abs(dx) <= 4 * ss)
        tread_h += np.where(lug_l | lug_r | ridge, 0.48, 0.0)
    
    tread_h = gaussian_filter(tread_h, sigma=1.4)
    tread_h += add_rubber_microtexture(H, W, 0.02)
    
    z_total = np.maximum(0.0, z_base + tread_h)
    lighting = compute_normals_and_shading(z_total, mask, ambient=0.36, spec_power=16.0)
    
    im = create_rubber_image(lighting, mask, base_rgb=(38.0, 42.0, 48.0))
    return im.resize((128, 256), Image.Resampling.LANCZOS)


def generate_all_realistic_wheels(out_dir: Path):
    out_dir.mkdir(parents=True, exist_ok=True)
    generators = {
        "buggy_allterrain_front.png": build_buggy_allterrain,
        "gt_slick_front.png": build_gt_slick,
        "kart_slick_front.png": build_kart_slick,
        "monster_wheel_front.png": build_monster_wheel,
        "mud_tractor_front.png": build_mud_tractor,
        "nascar_wheel_front.png": build_nascar_wheel,
        "offroad_wheel_front.png": build_offroad_wheel,
        "rally_wheel_front.png": build_rally_wheel,
        "truck_allterrain_front.png": build_truck_allterrain,
    }
    
    print("=" * 70)
    print("🎨 Generating Pure Realistic Top-Down Tyre Tread Sprites (No Indicators)")
    print("=" * 70)
    
    results = {}
    for filename, gen_fn in sorted(generators.items()):
        print(f"Rendering {filename} (pure tread)...")
        img = gen_fn(ss=4)
        out_path = out_dir / filename
        img.save(out_path)
        results[filename] = img
        print(f"✓ Saved {out_path} ({img.size[0]}x{img.size[1]})")
        
    return results


if __name__ == "__main__":
    test_dir = Path("artifacts/test_wheels")
    generate_all_realistic_wheels(test_dir)
