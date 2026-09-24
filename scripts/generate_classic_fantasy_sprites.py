#!/usr/bin/env python3
"""
Generate high-fidelity, made-up 2D lateral and top-down textures for the 4
Classic Arcade fantasy vehicles:
1. classic_gt: Apex Phantom GT (Arcade GT Coupe)
2. classic_nascar: Thunderbolt Stock V8 (Arcade Speedway Stock)
3. classic_offroad: Vortex Dune Crusher (Extreme Off-Road Buggy)
4. classic_kart: Turbo Dart 200cc (Arcade Sprint Kart)

Outputs:
- assets/textures/vehicles/laterals/classic/{id}.png (1024x512)
- assets/textures/vehicles/laterals/classic/{id}_thumb.png (256x128)
- assets/textures/vehicles/topdown/classic/{id}.png (512x512)
"""

import sys
from pathlib import Path
from PIL import Image, ImageDraw
import math

ROOT = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else Path.cwd()
LATERAL_DIR = ROOT / "assets" / "textures" / "vehicles" / "laterals" / "classic"
TOPDOWN_DIR = ROOT / "assets" / "textures" / "vehicles" / "topdown" / "classic"
WHEELS_DIR = ROOT / "assets" / "textures" / "vehicles" / "topdown" / "wheels"

LATERAL_DIR.mkdir(parents=True, exist_ok=True)
TOPDOWN_DIR.mkdir(parents=True, exist_ok=True)
WHEELS_DIR.mkdir(parents=True, exist_ok=True)


def draw_circle(draw, cx, cy, r, fill, outline=None, width=1):
    draw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=fill, outline=outline, width=width)


def draw_wheel_lateral(draw, cx, cy, r, rim_color=(200, 200, 210), style="alloy", ss=2):
    # Tire rubber
    draw_circle(draw, cx, cy, r, fill=(28, 30, 34), outline=(15, 16, 18), width=int(2 * ss))
    # Outer rim
    draw_circle(draw, cx, cy, r * 0.72, fill=(40, 42, 48), outline=(90, 95, 105), width=int(2 * ss))
    # Inner rim / brake rotor
    draw_circle(draw, cx, cy, r * 0.60, fill=(75, 78, 85), outline=(120, 125, 135), width=int(1 * ss))
    # Brake caliper (top right)
    caliper_angle = 0.8
    cal_x = cx + math.cos(caliper_angle) * (r * 0.52)
    cal_y = cy - math.sin(caliper_angle) * (r * 0.52)
    draw_circle(draw, cal_x, cal_y, r * 0.20, fill=(220, 40, 30), outline=(140, 20, 15), width=int(1 * ss))
    # Spokes or steel plate
    if style == "alloy":
        for i in range(8):
            ang = i * (math.pi / 4)
            x2 = cx + math.cos(ang) * (r * 0.68)
            y2 = cy + math.sin(ang) * (r * 0.68)
            draw.line([cx, cy, x2, y2], fill=rim_color, width=int(3 * ss))
    elif style == "steel":
        # NASCAR black steel rim with gold/yellow perimeter holes
        draw_circle(draw, cx, cy, r * 0.62, fill=(22, 24, 28), outline=(60, 64, 70), width=int(2 * ss))
        for i in range(10):
            ang = i * (math.pi / 5)
            x2 = cx + math.cos(ang) * (r * 0.48)
            y2 = cy + math.sin(ang) * (r * 0.48)
            draw_circle(draw, x2, y2, r * 0.08, fill=(10, 10, 12))
    elif style == "paddle":
        # Deep ribbed off-road rim
        draw_circle(draw, cx, cy, r * 0.62, fill=(210, 110, 20), outline=(140, 70, 10), width=int(2 * ss))
        for i in range(6):
            ang = i * (math.pi / 3)
            x2 = cx + math.cos(ang) * (r * 0.58)
            y2 = cy + math.sin(ang) * (r * 0.58)
            draw.line([cx, cy, x2, y2], fill=(20, 20, 24), width=int(4 * ss))
    elif style == "kart":
        # Ultra-light kart hub
        draw_circle(draw, cx, cy, r * 0.55, fill=(190, 160, 40), outline=(120, 100, 20), width=int(1 * ss))
        for i in range(3):
            ang = i * (2 * math.pi / 3)
            x2 = cx + math.cos(ang) * (r * 0.48)
            y2 = cy + math.sin(ang) * (r * 0.48)
            draw_circle(draw, x2, y2, r * 0.10, fill=(30, 30, 35))

    # Center cap
    draw_circle(draw, cx, cy, r * 0.18, fill=rim_color, outline=(20, 20, 24), width=int(1 * ss))


# ==============================================================================
# 1. APEX PHANTOM GT (classic_gt)
# ==============================================================================
def generate_classic_gt():
    ss = 2
    W, H = 1024 * ss, 512 * ss
    im_lat = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im_lat)

    ground_y = int(450 * ss)
    r_wheel = int(58 * ss)
    wf_x = int(760 * ss)
    wr_x = int(270 * ss)
    cy_wheel = ground_y - r_wheel

    # Shadow
    draw.ellipse([int(150 * ss), ground_y - int(10 * ss), int(880 * ss), ground_y + int(24 * ss)], fill=(0, 0, 0, 140))

    # Body contours (Aggressive GT Coupe)
    body_poly = [
        (int(150 * ss), ground_y - int(28 * ss)),  # Rear diffuser bottom
        (int(140 * ss), ground_y - int(80 * ss)),  # Rear bumper tip
        (int(155 * ss), ground_y - int(125 * ss)), # Rear deck start
        (int(280 * ss), ground_y - int(140 * ss)), # Rear deck to rear glass
        (int(400 * ss), ground_y - int(210 * ss)), # Roof peak
        (int(580 * ss), ground_y - int(210 * ss)), # Roof front
        (int(700 * ss), ground_y - int(135 * ss)), # Windshield base
        (int(830 * ss), ground_y - int(105 * ss)), # Hood front / headlight
        (int(890 * ss), ground_y - int(55 * ss)),  # Nose splitter top
        (int(895 * ss), ground_y - int(22 * ss)),  # Front splitter edge
        (int(840 * ss), ground_y - int(22 * ss)),  # Under front overhang
        (int(700 * ss), ground_y - int(22 * ss)),  # Side skirt front
        (int(330 * ss), ground_y - int(22 * ss)),  # Side skirt rear
        (int(210 * ss), ground_y - int(22 * ss)),  # Under rear overhang
    ]
    # Primary red coat
    draw.polygon(body_poly, fill=(225, 35, 30))

    # Dark metallic roof / canopy
    greenhouse_poly = [
        (int(390 * ss), ground_y - int(142 * ss)),
        (int(420 * ss), ground_y - int(202 * ss)),
        (int(575 * ss), ground_y - int(202 * ss)),
        (int(685 * ss), ground_y - int(138 * ss)),
        (int(560 * ss), ground_y - int(142 * ss)),
    ]
    draw.polygon(greenhouse_poly, fill=(28, 32, 40))

    # Side window glass
    window_poly = [
        (int(440 * ss), ground_y - int(195 * ss)),
        (int(565 * ss), ground_y - int(195 * ss)),
        (int(665 * ss), ground_y - int(145 * ss)),
        (int(425 * ss), ground_y - int(145 * ss)),
    ]
    draw.polygon(window_poly, fill=(80, 150, 210, 210), outline=(20, 22, 28), width=int(2 * ss))

    # Carbon side skirt & front splitter
    draw.polygon([
        (int(320 * ss), ground_y - int(26 * ss)),
        (int(710 * ss), ground_y - int(26 * ss)),
        (int(710 * ss), ground_y - int(14 * ss)),
        (int(320 * ss), ground_y - int(14 * ss)),
    ], fill=(24, 26, 30))

    draw.polygon([
        (int(820 * ss), ground_y - int(24 * ss)),
        (int(905 * ss), ground_y - int(24 * ss)),
        (int(905 * ss), ground_y - int(14 * ss)),
        (int(820 * ss), ground_y - int(14 * ss)),
    ], fill=(18, 20, 24))

    # Swan neck GT rear wing
    # Uprights
    draw.polygon([
        (int(200 * ss), ground_y - int(130 * ss)),
        (int(185 * ss), ground_y - int(220 * ss)),
        (int(195 * ss), ground_y - int(220 * ss)),
        (int(210 * ss), ground_y - int(130 * ss)),
    ], fill=(30, 32, 38))
    # Wing blade
    draw.polygon([
        (int(135 * ss), ground_y - int(230 * ss)),
        (int(255 * ss), ground_y - int(222 * ss)),
        (int(250 * ss), ground_y - int(210 * ss)),
        (int(130 * ss), ground_y - int(218 * ss)),
    ], fill=(18, 20, 24), outline=(230, 40, 35), width=int(2 * ss))

    # Racing decals & speed stripe
    draw.polygon([
        (int(310 * ss), ground_y - int(75 * ss)),
        (int(760 * ss), ground_y - int(65 * ss)),
        (int(755 * ss), ground_y - int(52 * ss)),
        (int(305 * ss), ground_y - int(62 * ss)),
    ], fill=(245, 245, 250))

    # Wheels
    draw_wheel_lateral(draw, wr_x, cy_wheel, r_wheel, rim_color=(220, 220, 230), style="alloy", ss=ss)
    draw_wheel_lateral(draw, wf_x, cy_wheel, r_wheel, rim_color=(220, 220, 230), style="alloy", ss=ss)

    # Headlight LED
    draw.polygon([
        (int(835 * ss), ground_y - int(100 * ss)),
        (int(885 * ss), ground_y - int(72 * ss)),
        (int(865 * ss), ground_y - int(68 * ss)),
    ], fill=(220, 245, 255), outline=(160, 210, 255), width=int(1 * ss))

    # Taillight
    draw.polygon([
        (int(145 * ss), ground_y - int(118 * ss)),
        (int(170 * ss), ground_y - int(118 * ss)),
        (int(165 * ss), ground_y - int(105 * ss)),
        (int(142 * ss), ground_y - int(105 * ss)),
    ], fill=(255, 30, 30))

    lat_img = im_lat.resize((1024, 512), Image.Resampling.LANCZOS)
    lat_img.save(LATERAL_DIR / "classic_gt.png")
    lat_thumb = lat_img.resize((256, 128), Image.Resampling.LANCZOS)
    lat_thumb.save(LATERAL_DIR / "classic_gt_thumb.png")

    # --- TOP-DOWN SPRITE (512x512) ---
    W_TD, H_TD = 512 * ss, 512 * ss
    im_td = Image.new("RGBA", (W_TD, H_TD), (0, 0, 0, 0))
    draw_td = ImageDraw.Draw(im_td)

    cx = 256 * ss
    cy = 256 * ss
    w_half = int(90 * ss)
    h_half = int(210 * ss)

    # Wheels top-down (recessed inside wheel wells)
    for wx, wy in [
        (cx - w_half + int(10 * ss), cy - int(120 * ss)),
        (cx + w_half - int(10 * ss), cy - int(120 * ss)),
        (cx - w_half + int(10 * ss), cy + int(115 * ss)),
        (cx + w_half - int(10 * ss), cy + int(115 * ss)),
    ]:
        draw_td.rounded_rectangle([wx - int(14 * ss), wy - int(34 * ss), wx + int(14 * ss), wy + int(34 * ss)], radius=int(6 * ss), fill=(22, 24, 28))

    # Main Body Outline (Facing UP)
    top_poly = [
        (cx - int(45 * ss), cy - h_half),                 # Front splitter nose L
        (cx + int(45 * ss), cy - h_half),                 # Front splitter nose R
        (cx + int(82 * ss), cy - h_half + int(26 * ss)),  # Front fender R
        (cx + int(96 * ss), cy - int(100 * ss)),          # Front wheel arch R
        (cx + int(84 * ss), cy - int(40 * ss)),           # Door R
        (cx + int(98 * ss), cy + int(90 * ss)),           # Rear flare R
        (cx + int(88 * ss), cy + h_half - int(25 * ss)),  # Rear bumper R
        (cx + int(60 * ss), cy + h_half),                 # Rear diffuser R
        (cx - int(60 * ss), cy + h_half),                 # Rear diffuser L
        (cx - int(88 * ss), cy + h_half - int(25 * ss)),  # Rear bumper L
        (cx - int(98 * ss), cy + int(90 * ss)),           # Rear flare L
        (cx - int(84 * ss), cy - int(40 * ss)),           # Door L
        (cx - int(96 * ss), cy - int(100 * ss)),          # Front wheel arch L
        (cx - int(82 * ss), cy - h_half + int(26 * ss)),  # Front fender L
    ]
    draw_td.polygon(top_poly, fill=(225, 35, 30), outline=(170, 20, 18), width=int(2 * ss))

    # White Racing Twin Stripes
    for sx in [-int(18 * ss), int(6 * ss)]:
        draw_td.rectangle([cx + sx, cy - h_half + int(12 * ss), cx + sx + int(12 * ss), cy + h_half - int(20 * ss)], fill=(245, 245, 250))

    # Cockpit Greenhouse
    glass_poly = [
        (cx - int(52 * ss), cy - int(60 * ss)),
        (cx + int(52 * ss), cy - int(60 * ss)),
        (cx + int(58 * ss), cy + int(55 * ss)),
        (cx - int(58 * ss), cy + int(55 * ss)),
    ]
    draw_td.polygon(glass_poly, fill=(35, 45, 60), outline=(20, 22, 28), width=int(2 * ss))
    # Roof panel
    draw_td.polygon([
        (cx - int(38 * ss), cy - int(25 * ss)),
        (cx + int(38 * ss), cy - int(25 * ss)),
        (cx + int(42 * ss), cy + int(38 * ss)),
        (cx - int(42 * ss), cy + int(38 * ss)),
    ], fill=(225, 35, 30))

    # Large GT Rear Wing
    draw_td.rectangle([cx - int(96 * ss), cy + h_half - int(42 * ss), cx + int(96 * ss), cy + h_half - int(22 * ss)], fill=(20, 22, 26), outline=(230, 40, 35), width=int(2 * ss))
    # Endplates
    draw_td.rectangle([cx - int(98 * ss), cy + h_half - int(46 * ss), cx - int(92 * ss), cy + h_half - int(18 * ss)], fill=(225, 35, 30))
    draw_td.rectangle([cx + int(92 * ss), cy + h_half - int(46 * ss), cx + int(98 * ss), cy + h_half - int(18 * ss)], fill=(225, 35, 30))

    # Headlights
    draw_td.polygon([(cx - int(72 * ss), cy - h_half + int(35 * ss)), (cx - int(55 * ss), cy - h_half + int(18 * ss)), (cx - int(45 * ss), cy - h_half + int(28 * ss))], fill=(220, 245, 255))
    draw_td.polygon([(cx + int(72 * ss), cy - h_half + int(35 * ss)), (cx + int(55 * ss), cy - h_half + int(18 * ss)), (cx + int(45 * ss), cy - h_half + int(28 * ss))], fill=(220, 245, 255))

    td_img = im_td.resize((512, 512), Image.Resampling.LANCZOS)
    td_img.save(TOPDOWN_DIR / "classic_gt.png")
    print("✓ Generated classic_gt assets")


# ==============================================================================
# 2. THUNDERBOLT STOCK V8 (classic_nascar)
# ==============================================================================
def generate_classic_nascar():
    ss = 2
    W, H = 1024 * ss, 512 * ss
    im_lat = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im_lat)

    ground_y = int(450 * ss)
    r_wheel = int(60 * ss)
    wf_x = int(750 * ss)
    wr_x = int(270 * ss)
    cy_wheel = ground_y - r_wheel

    # Shadow
    draw.ellipse([int(140 * ss), ground_y - int(10 * ss), int(890 * ss), ground_y + int(24 * ss)], fill=(0, 0, 0, 140))

    # Muscular Stock Car Silhouette
    body_poly = [
        (int(140 * ss), ground_y - int(32 * ss)),  # Rear bumper bottom
        (int(135 * ss), ground_y - int(115 * ss)), # Rear bumper top
        (int(260 * ss), ground_y - int(135 * ss)), # Rear deck start
        (int(360 * ss), ground_y - int(218 * ss)), # Roof rear
        (int(560 * ss), ground_y - int(218 * ss)), # Roof front
        (int(680 * ss), ground_y - int(135 * ss)), # Windshield base
        (int(840 * ss), ground_y - int(115 * ss)), # Hood front
        (int(890 * ss), ground_y - int(65 * ss)),  # Nose tip
        (int(885 * ss), ground_y - int(24 * ss)),  # Front splitter
        (int(830 * ss), ground_y - int(24 * ss)),
        (int(700 * ss), ground_y - int(24 * ss)),
        (int(320 * ss), ground_y - int(24 * ss)),
        (int(210 * ss), ground_y - int(24 * ss)),
    ]
    # Deep cobalt blue
    draw.polygon(body_poly, fill=(20, 75, 185))

    # Yellow thunderbolt side stripe
    stripe_poly = [
        (int(140 * ss), ground_y - int(95 * ss)),
        (int(380 * ss), ground_y - int(90 * ss)),
        (int(460 * ss), ground_y - int(60 * ss)),
        (int(880 * ss), ground_y - int(55 * ss)),
        (int(880 * ss), ground_y - int(40 * ss)),
        (int(470 * ss), ground_y - int(45 * ss)),
        (int(390 * ss), ground_y - int(75 * ss)),
        (int(140 * ss), ground_y - int(80 * ss)),
    ]
    draw.polygon(stripe_poly, fill=(250, 195, 20))

    # Greenhouse / Cabin with roll cage lattice
    greenhouse_poly = [
        (int(355 * ss), ground_y - int(138 * ss)),
        (int(380 * ss), ground_y - int(210 * ss)),
        (int(550 * ss), ground_y - int(210 * ss)),
        (int(665 * ss), ground_y - int(138 * ss)),
    ]
    draw.polygon(greenhouse_poly, fill=(18, 20, 25))
    # Window Net on driver side
    window_poly = [
        (int(405 * ss), ground_y - int(202 * ss)),
        (int(540 * ss), ground_y - int(202 * ss)),
        (int(645 * ss), ground_y - int(145 * ss)),
        (int(385 * ss), ground_y - int(145 * ss)),
    ]
    draw.polygon(window_poly, fill=(45, 70, 95, 220), outline=(20, 20, 20), width=int(2 * ss))
    # Window net ribs
    for x_rib in range(int(410 * ss), int(480 * ss), int(12 * ss)):
        draw.line([x_rib, ground_y - int(200 * ss), x_rib + int(15 * ss), ground_y - int(148 * ss)], fill=(18, 18, 18), width=int(2 * ss))

    # NASCAR Tall Blade / Ducktail Spoiler
    draw.polygon([
        (int(130 * ss), ground_y - int(165 * ss)),
        (int(165 * ss), ground_y - int(135 * ss)),
        (int(150 * ss), ground_y - int(120 * ss)),
        (int(125 * ss), ground_y - int(150 * ss)),
    ], fill=(15, 16, 20), outline=(250, 195, 20), width=int(2 * ss))

    # Side exhaust exit & heat shield
    draw.polygon([
        (int(490 * ss), ground_y - int(38 * ss)),
        (int(540 * ss), ground_y - int(38 * ss)),
        (int(540 * ss), ground_y - int(24 * ss)),
        (int(490 * ss), ground_y - int(24 * ss)),
    ], fill=(180, 185, 195))
    draw_circle(draw, int(505 * ss), ground_y - int(31 * ss), int(6 * ss), fill=(20, 20, 22))
    draw_circle(draw, int(525 * ss), ground_y - int(31 * ss), int(6 * ss), fill=(20, 20, 22))

    # Wheels (NASCAR Black Steel Wheels with yellow Goodyear style perimeter)
    draw_wheel_lateral(draw, wr_x, cy_wheel, r_wheel, style="steel", ss=ss)
    draw_wheel_lateral(draw, wf_x, cy_wheel, r_wheel, style="steel", ss=ss)

    lat_img = im_lat.resize((1024, 512), Image.Resampling.LANCZOS)
    lat_img.save(LATERAL_DIR / "classic_nascar.png")
    lat_thumb = lat_img.resize((256, 128), Image.Resampling.LANCZOS)
    lat_thumb.save(LATERAL_DIR / "classic_nascar_thumb.png")

    # --- TOP-DOWN SPRITE (512x512) ---
    W_TD, H_TD = 512 * ss, 512 * ss
    im_td = Image.new("RGBA", (W_TD, H_TD), (0, 0, 0, 0))
    draw_td = ImageDraw.Draw(im_td)

    cx = 256 * ss
    cy = 256 * ss
    w_half = int(98 * ss)
    h_half = int(215 * ss)

    # Wide NASCAR tires
    for wx, wy in [
        (cx - w_half + int(12 * ss), cy - int(120 * ss)),
        (cx + w_half - int(12 * ss), cy - int(120 * ss)),
        (cx - w_half + int(12 * ss), cy + int(115 * ss)),
        (cx + w_half - int(12 * ss), cy + int(115 * ss)),
    ]:
        draw_td.rounded_rectangle([wx - int(16 * ss), wy - int(35 * ss), wx + int(16 * ss), wy + int(35 * ss)], radius=int(6 * ss), fill=(20, 22, 25))

    # Wide muscular stock body
    top_poly = [
        (cx - int(55 * ss), cy - h_half),                 # Front nose L
        (cx + int(55 * ss), cy - h_half),                 # Front nose R
        (cx + int(92 * ss), cy - h_half + int(30 * ss)),  # Front fender R
        (cx + int(98 * ss), cy - int(80 * ss)),           # Front side R
        (cx + int(95 * ss), cy + int(90 * ss)),           # Rear quarter R
        (cx + int(90 * ss), cy + h_half - int(15 * ss)),  # Rear corner R
        (cx - int(90 * ss), cy + h_half - int(15 * ss)),  # Rear corner L
        (cx - int(95 * ss), cy + int(90 * ss)),           # Rear quarter L
        (cx - int(98 * ss), cy - int(80 * ss)),           # Front side L
        (cx - int(92 * ss), cy - h_half + int(30 * ss)),  # Front fender L
    ]
    draw_td.polygon(top_poly, fill=(20, 75, 185), outline=(12, 45, 120), width=int(2 * ss))

    # Gold lightning hood graphics
    draw_td.polygon([
        (cx - int(30 * ss), cy - h_half + int(20 * ss)),
        (cx + int(30 * ss), cy - h_half + int(20 * ss)),
        (cx + int(15 * ss), cy - int(70 * ss)),
        (cx - int(15 * ss), cy - int(70 * ss)),
    ], fill=(250, 195, 20))

    # Roof & Glass
    draw_td.polygon([
        (cx - int(56 * ss), cy - int(65 * ss)),
        (cx + int(56 * ss), cy - int(65 * ss)),
        (cx + int(62 * ss), cy + int(60 * ss)),
        (cx - int(62 * ss), cy + int(60 * ss)),
    ], fill=(22, 26, 32))
    draw_td.polygon([
        (cx - int(45 * ss), cy - int(30 * ss)),
        (cx + int(45 * ss), cy - int(30 * ss)),
        (cx + int(48 * ss), cy + int(42 * ss)),
        (cx - int(48 * ss), cy + int(42 * ss)),
    ], fill=(20, 75, 185))

    # Roof Shark Fin (Center stabilizing blade)
    draw_td.line([cx, cy - int(25 * ss), cx, cy + int(50 * ss)], fill=(240, 245, 255), width=int(3 * ss))

    # Rear Blade Spoiler
    draw_td.rectangle([cx - int(86 * ss), cy + h_half - int(22 * ss), cx + int(86 * ss), cy + h_half - int(10 * ss)], fill=(16, 18, 22), outline=(250, 195, 20), width=int(2 * ss))

    td_img = im_td.resize((512, 512), Image.Resampling.LANCZOS)
    td_img.save(TOPDOWN_DIR / "classic_nascar.png")
    print("✓ Generated classic_nascar assets")


# ==============================================================================
# 3. VORTEX DUNE CRUSHER (classic_offroad)
# ==============================================================================
def generate_classic_offroad():
    ss = 2
    W, H = 1024 * ss, 512 * ss
    im_lat = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im_lat)

    ground_y = int(450 * ss)
    r_wheel = int(68 * ss)
    wf_x = int(770 * ss)
    wr_x = int(250 * ss)
    cy_wheel = ground_y - r_wheel

    # Shadow
    draw.ellipse([int(160 * ss), ground_y - int(10 * ss), int(880 * ss), ground_y + int(24 * ss)], fill=(0, 0, 0, 140))

    # Chromoly Tubular Frame Lattice (Safety Orange)
    frame_color = (255, 115, 15)
    tube_w = int(7 * ss)

    # Main chassis rail
    draw.line([int(180 * ss), ground_y - int(75 * ss), int(760 * ss), ground_y - int(75 * ss)], fill=frame_color, width=tube_w)
    # Lower chassis rail
    draw.line([int(220 * ss), ground_y - int(35 * ss), int(720 * ss), ground_y - int(35 * ss)], fill=frame_color, width=tube_w)
    # Roll cage uprights & roof
    draw.line([int(340 * ss), ground_y - int(75 * ss), int(420 * ss), ground_y - int(225 * ss)], fill=frame_color, width=tube_w)
    draw.line([int(420 * ss), ground_y - int(225 * ss), int(580 * ss), ground_y - int(225 * ss)], fill=frame_color, width=tube_w)
    draw.line([int(580 * ss), ground_y - int(225 * ss), int(680 * ss), ground_y - int(75 * ss)], fill=frame_color, width=tube_w)
    # Diagonal braces
    draw.line([int(340 * ss), ground_y - int(75 * ss), int(580 * ss), ground_y - int(225 * ss)], fill=frame_color, width=int(4 * ss))
    draw.line([int(420 * ss), ground_y - int(225 * ss), int(680 * ss), ground_y - int(75 * ss)], fill=frame_color, width=int(4 * ss))
    draw.line([int(180 * ss), ground_y - int(75 * ss), int(420 * ss), ground_y - int(225 * ss)], fill=frame_color, width=int(4 * ss))

    # Long travel coilovers
    draw.line([wr_x, cy_wheel, int(330 * ss), ground_y - int(125 * ss)], fill=(220, 225, 235), width=int(6 * ss))
    draw.line([wf_x, cy_wheel, int(690 * ss), ground_y - int(125 * ss)], fill=(220, 225, 235), width=int(6 * ss))

    # Cockpit Bucket Seats & Driver Helmet
    draw.polygon([
        (int(440 * ss), ground_y - int(80 * ss)),
        (int(460 * ss), ground_y - int(155 * ss)),
        (int(490 * ss), ground_y - int(155 * ss)),
        (int(475 * ss), ground_y - int(80 * ss)),
    ], fill=(30, 32, 38))
    # Helmet
    draw_circle(draw, int(475 * ss), ground_y - int(175 * ss), int(22 * ss), fill=(245, 245, 250), outline=(20, 20, 24), width=int(2 * ss))
    # Visor
    draw.rectangle([int(482 * ss), ground_y - int(182 * ss), int(497 * ss), ground_y - int(170 * ss)], fill=(30, 32, 35))

    # Rear Exposed Flat-4 Boxer Turbo Engine Block
    draw.rectangle([int(180 * ss), ground_y - int(115 * ss), int(270 * ss), ground_y - int(55 * ss)], fill=(120, 125, 135), outline=(60, 65, 70), width=int(2 * ss))
    # Turbocharger & exhaust trumpet
    draw_circle(draw, int(190 * ss), ground_y - int(118 * ss), int(16 * ss), fill=(215, 130, 30), outline=(80, 45, 15), width=int(2 * ss))
    draw.line([int(190 * ss), ground_y - int(118 * ss), int(150 * ss), ground_y - int(145 * ss)], fill=(190, 195, 205), width=int(6 * ss))

    # 4-Pod Roof Lightbar
    for lx in range(int(440 * ss), int(560 * ss), int(30 * ss)):
        draw_circle(draw, lx, ground_y - int(238 * ss), int(11 * ss), fill=(250, 250, 210), outline=(40, 42, 45), width=int(2 * ss))

    # Tall whip antenna with neon orange safety pennant
    draw.line([int(220 * ss), ground_y - int(115 * ss), int(180 * ss), ground_y - int(330 * ss)], fill=(200, 205, 215), width=int(3 * ss))
    draw.polygon([
        (int(180 * ss), ground_y - int(330 * ss)),
        (int(120 * ss), ground_y - int(310 * ss)),
        (int(175 * ss), ground_y - int(295 * ss)),
    ], fill=(255, 80, 10))

    # Big Off-Road Paddle & Ribbed Wheels
    draw_wheel_lateral(draw, wr_x, cy_wheel, int(r_wheel * 1.12), style="paddle", ss=ss)
    draw_wheel_lateral(draw, wf_x, cy_wheel, r_wheel, style="paddle", ss=ss)

    lat_img = im_lat.resize((1024, 512), Image.Resampling.LANCZOS)
    lat_img.save(LATERAL_DIR / "classic_offroad.png")
    lat_thumb = lat_img.resize((256, 128), Image.Resampling.LANCZOS)
    lat_thumb.save(LATERAL_DIR / "classic_offroad_thumb.png")

    # --- TOP-DOWN SPRITE (512x512) ---
    W_TD, H_TD = 512 * ss, 512 * ss
    im_td = Image.new("RGBA", (W_TD, H_TD), (0, 0, 0, 0))
    draw_td = ImageDraw.Draw(im_td)

    cx = 256 * ss
    cy = 256 * ss
    w_half = int(105 * ss)
    h_half = int(210 * ss)

    # Exposed wide paddle sand tires
    # Rear paddle tires (extra wide)
    draw_td.rounded_rectangle([cx - w_half - int(24 * ss), cy + int(85 * ss), cx - w_half + int(14 * ss), cy + int(165 * ss)], radius=int(6 * ss), fill=(24, 26, 30))
    draw_td.rounded_rectangle([cx + w_half - int(14 * ss), cy + int(85 * ss), cx + w_half + int(24 * ss), cy + int(165 * ss)], radius=int(6 * ss), fill=(24, 26, 30))
    # Front steering tires
    draw_td.rounded_rectangle([cx - w_half - int(12 * ss), cy - int(155 * ss), cx - w_half + int(16 * ss), cy - int(85 * ss)], radius=int(6 * ss), fill=(24, 26, 30))
    draw_td.rounded_rectangle([cx + w_half - int(16 * ss), cy - int(155 * ss), cx + w_half + int(12 * ss), cy - int(85 * ss)], radius=int(6 * ss), fill=(24, 26, 30))

    # Long suspension wishbones
    draw_td.line([cx - int(45 * ss), cy - int(120 * ss), cx - w_half, cy - int(120 * ss)], fill=(180, 185, 195), width=int(5 * ss))
    draw_td.line([cx + int(45 * ss), cy - int(120 * ss), cx + w_half, cy - int(120 * ss)], fill=(180, 185, 195), width=int(5 * ss))
    draw_td.line([cx - int(50 * ss), cy + int(125 * ss), cx - w_half, cy + int(125 * ss)], fill=(180, 185, 195), width=int(6 * ss))
    draw_td.line([cx + int(50 * ss), cy + int(125 * ss), cx + w_half, cy + int(125 * ss)], fill=(180, 185, 195), width=int(6 * ss))

    # Tubular Chromoly Cockpit Frame
    draw_td.rectangle([cx - int(58 * ss), cy - int(80 * ss), cx + int(58 * ss), cy + int(70 * ss)], outline=frame_color, width=tube_w)
    # Cross brace X
    draw_td.line([cx - int(58 * ss), cy - int(80 * ss), cx + int(58 * ss), cy + int(70 * ss)], fill=frame_color, width=int(4 * ss))
    draw_td.line([cx + int(58 * ss), cy - int(80 * ss), cx - int(58 * ss), cy + int(70 * ss)], fill=frame_color, width=int(4 * ss))

    # Nosecone fairing
    draw_td.polygon([
        (cx - int(35 * ss), cy - int(80 * ss)),
        (cx + int(35 * ss), cy - int(80 * ss)),
        (cx + int(22 * ss), cy - int(165 * ss)),
        (cx - int(22 * ss), cy - int(165 * ss)),
    ], fill=(255, 115, 15), outline=(180, 70, 10), width=int(2 * ss))

    # Roof Lightbar
    for lx in range(cx - int(45 * ss), cx + int(46 * ss), int(26 * ss)):
        draw_circle(draw_td, lx, cy - int(85 * ss), int(10 * ss), fill=(250, 250, 210), outline=(30, 32, 35), width=int(2 * ss))

    # Twin bucket seats & driver helmet
    draw_td.rectangle([cx - int(42 * ss), cy - int(20 * ss), cx - int(10 * ss), cy + int(35 * ss)], fill=(32, 35, 42))
    draw_td.rectangle([cx + int(10 * ss), cy - int(20 * ss), cx + int(42 * ss), cy + int(35 * ss)], fill=(32, 35, 42))
    draw_circle(draw_td, cx - int(26 * ss), cy + int(5 * ss), int(16 * ss), fill=(245, 245, 250), outline=(20, 20, 24), width=int(2 * ss))

    # Rear engine block & exhausts
    draw_td.rectangle([cx - int(45 * ss), cy + int(85 * ss), cx + int(45 * ss), cy + int(155 * ss)], fill=(90, 95, 105), outline=(50, 52, 58), width=int(2 * ss))
    draw_td.ellipse([cx - int(20 * ss), cy + int(155 * ss), cx + int(20 * ss), cy + int(185 * ss)], fill=(200, 120, 20), outline=(50, 25, 10), width=int(2 * ss))

    td_img = im_td.resize((512, 512), Image.Resampling.LANCZOS)
    td_img.save(TOPDOWN_DIR / "classic_offroad.png")
    print("✓ Generated classic_offroad assets")


# ==============================================================================
# 4. TURBO DART 200cc (classic_kart)
# ==============================================================================
def generate_classic_kart():
    ss = 2
    W, H = 1024 * ss, 512 * ss
    im_lat = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im_lat)

    ground_y = int(450 * ss)
    r_wheel = int(42 * ss)
    wf_x = int(720 * ss)
    wr_x = int(320 * ss)
    cy_wheel = ground_y - r_wheel

    # Shadow
    draw.ellipse([int(230 * ss), ground_y - int(8 * ss), int(810 * ss), ground_y + int(18 * ss)], fill=(0, 0, 0, 140))

    # Low-slung tubular chassis rails & bumpers
    draw.line([int(250 * ss), ground_y - int(22 * ss), int(790 * ss), ground_y - int(22 * ss)], fill=(40, 42, 48), width=int(5 * ss))
    # Front bumper curve
    draw.line([int(790 * ss), ground_y - int(22 * ss), int(810 * ss), ground_y - int(55 * ss)], fill=(40, 42, 48), width=int(5 * ss))
    # Rear bumper curve
    draw.line([int(250 * ss), ground_y - int(22 * ss), int(240 * ss), ground_y - int(55 * ss)], fill=(40, 42, 48), width=int(5 * ss))

    # Side Pod (Vibrant Lime Green with racing #7)
    pod_poly = [
        (int(360 * ss), ground_y - int(24 * ss)),
        (int(680 * ss), ground_y - int(24 * ss)),
        (int(660 * ss), ground_y - int(72 * ss)),
        (int(380 * ss), ground_y - int(72 * ss)),
    ]
    draw.polygon(pod_poly, fill=(50, 215, 65), outline=(25, 140, 35), width=int(2 * ss))
    # Racing black accent panel
    draw.polygon([
        (int(420 * ss), ground_y - int(30 * ss)),
        (int(620 * ss), ground_y - int(30 * ss)),
        (int(605 * ss), ground_y - int(65 * ss)),
        (int(435 * ss), ground_y - int(65 * ss)),
    ], fill=(22, 24, 28))

    # Front Fairing / Steering column support
    draw.polygon([
        (int(660 * ss), ground_y - int(26 * ss)),
        (int(780 * ss), ground_y - int(26 * ss)),
        (int(750 * ss), ground_y - int(95 * ss)),
        (int(670 * ss), ground_y - int(75 * ss)),
    ], fill=(50, 215, 65), outline=(25, 140, 35), width=int(2 * ss))

    # Driver Body & Helmet (Exposed seated posture)
    # Seat
    draw.polygon([
        (int(420 * ss), ground_y - int(30 * ss)),
        (int(440 * ss), ground_y - int(130 * ss)),
        (int(470 * ss), ground_y - int(130 * ss)),
        (int(480 * ss), ground_y - int(30 * ss)),
    ], fill=(24, 26, 30))
    # Torso (Racing suit)
    draw.polygon([
        (int(450 * ss), ground_y - int(45 * ss)),
        (int(475 * ss), ground_y - int(140 * ss)),
        (int(535 * ss), ground_y - int(130 * ss)),
        (int(520 * ss), ground_y - int(45 * ss)),
    ], fill=(30, 32, 38))
    # Arm reaching to steering wheel
    draw.line([int(515 * ss), ground_y - int(120 * ss), int(595 * ss), ground_y - int(110 * ss)], fill=(50, 215, 65), width=int(10 * ss))
    # Steering Wheel
    draw.line([int(590 * ss), ground_y - int(135 * ss), int(605 * ss), ground_y - int(85 * ss)], fill=(15, 16, 18), width=int(6 * ss))

    # Driver Helmet
    draw_circle(draw, int(495 * ss), ground_y - int(175 * ss), int(28 * ss), fill=(245, 245, 250), outline=(20, 20, 25), width=int(2 * ss))
    # Green helmet stripe
    draw.polygon([
        (int(475 * ss), ground_y - int(195 * ss)),
        (int(515 * ss), ground_y - int(195 * ss)),
        (int(520 * ss), ground_y - int(180 * ss)),
        (int(472 * ss), ground_y - int(180 * ss)),
    ], fill=(50, 215, 65))
    # Tinted Gold Visor
    draw.rectangle([int(505 * ss), ground_y - int(182 * ss), int(523 * ss), ground_y - int(168 * ss)], fill=(220, 180, 40))

    # 2-Stroke Engine & Chrome Expansion Pipe (Right side)
    draw.rectangle([int(370 * ss), ground_y - int(85 * ss), int(425 * ss), ground_y - int(35 * ss)], fill=(125, 130, 140))
    draw.line([int(370 * ss), ground_y - int(65 * ss), int(275 * ss), ground_y - int(65 * ss)], fill=(195, 200, 210), width=int(12 * ss))

    # Small Kart Wheels
    draw_wheel_lateral(draw, wr_x, cy_wheel, int(r_wheel * 1.15), style="kart", ss=ss)
    draw_wheel_lateral(draw, wf_x, cy_wheel, r_wheel, style="kart", ss=ss)

    lat_img = im_lat.resize((1024, 512), Image.Resampling.LANCZOS)
    lat_img.save(LATERAL_DIR / "classic_kart.png")
    lat_thumb = lat_img.resize((256, 128), Image.Resampling.LANCZOS)
    lat_thumb.save(LATERAL_DIR / "classic_kart_thumb.png")

    # --- TOP-DOWN SPRITE (512x512) ---
    W_TD, H_TD = 512 * ss, 512 * ss
    im_td = Image.new("RGBA", (W_TD, H_TD), (0, 0, 0, 0))
    draw_td = ImageDraw.Draw(im_td)

    cx = 256 * ss
    cy = 256 * ss
    w_half = int(85 * ss)
    h_half = int(185 * ss)

    # Exposed Kart Slick Wheels
    # Rear extra-wide slicks (fixed to chassis)
    draw_td.rounded_rectangle([cx - w_half - int(22 * ss), cy + int(70 * ss), cx - w_half + int(12 * ss), cy + int(145 * ss)], radius=int(6 * ss), fill=(24, 26, 30))
    draw_td.rounded_rectangle([cx + w_half - int(12 * ss), cy + int(70 * ss), cx + w_half + int(22 * ss), cy + int(145 * ss)], radius=int(6 * ss), fill=(24, 26, 30))
    # Front steer axle spindles / tie rods (front steer wheels decomposed into standalone kart_slick_front sprite)
    draw_td.line([cx - int(45 * ss), cy - int(105 * ss), cx - w_half + int(2 * ss), cy - int(105 * ss)], fill=(160, 165, 175), width=int(5 * ss))
    draw_td.line([cx + int(45 * ss), cy - int(105 * ss), cx + w_half - int(2 * ss), cy - int(105 * ss)], fill=(160, 165, 175), width=int(5 * ss))

    # Rear live axle bar
    draw_td.line([cx - w_half, cy + int(105 * ss), cx + w_half, cy + int(105 * ss)], fill=(190, 195, 205), width=int(8 * ss))

    # Side Pods (Left and Right)
    draw_td.rounded_rectangle([cx - int(76 * ss), cy - int(55 * ss), cx - int(45 * ss), cy + int(70 * ss)], radius=int(6 * ss), fill=(50, 215, 65), outline=(25, 140, 35), width=int(2 * ss))
    draw_td.rounded_rectangle([cx + int(45 * ss), cy - int(55 * ss), cx + int(76 * ss), cy + int(70 * ss)], radius=int(6 * ss), fill=(50, 215, 65), outline=(25, 140, 35), width=int(2 * ss))

    # Front Fairing & Nosecone
    draw_td.polygon([
        (cx - int(38 * ss), cy - h_half),
        (cx + int(38 * ss), cy - h_half),
        (cx + int(45 * ss), cy - int(85 * ss)),
        (cx - int(45 * ss), cy - int(85 * ss)),
    ], fill=(50, 215, 65), outline=(25, 140, 35), width=int(2 * ss))
    # #7 Number Plate on nose
    draw_td.polygon([
        (cx - int(18 * ss), cy - int(155 * ss)),
        (cx + int(18 * ss), cy - int(155 * ss)),
        (cx + int(14 * ss), cy - int(115 * ss)),
        (cx - int(14 * ss), cy - int(115 * ss)),
    ], fill=(245, 245, 250))

    # Driver Seat & Exposed Driver
    draw_td.rectangle([cx - int(35 * ss), cy - int(20 * ss), cx + int(35 * ss), cy + int(45 * ss)], fill=(28, 30, 36))
    # Steering column & wheel
    draw_td.line([cx, cy - int(75 * ss), cx, cy - int(42 * ss)], fill=(18, 20, 24), width=int(4 * ss))
    draw_td.ellipse([cx - int(24 * ss), cy - int(56 * ss), cx + int(24 * ss), cy - int(30 * ss)], fill=(18, 20, 24), outline=(50, 215, 65), width=int(2 * ss))

    # Driver Shoulders & Helmet
    draw_td.ellipse([cx - int(32 * ss), cy - int(8 * ss), cx + int(32 * ss), cy + int(32 * ss)], fill=(32, 35, 42))
    draw_circle(draw_td, cx, cy + int(10 * ss), int(20 * ss), fill=(245, 245, 250), outline=(20, 20, 24), width=int(2 * ss))
    # Visor facing up
    draw_td.rectangle([cx - int(14 * ss), cy - int(4 * ss), cx + int(14 * ss), cy + int(4 * ss)], fill=(220, 180, 40))

    # Engine (Right rear)
    draw_td.rectangle([cx + int(28 * ss), cy + int(45 * ss), cx + int(65 * ss), cy + int(98 * ss)], fill=(110, 115, 125), outline=(40, 42, 48), width=int(2 * ss))

    td_img = im_td.resize((512, 512), Image.Resampling.LANCZOS)
    td_img.save(TOPDOWN_DIR / "classic_kart.png")
    print("✓ Generated classic_kart assets")


# ==============================================================================
# 5. TRAILFIRE TURBO 4WD (classic_rally)
# ==============================================================================
def generate_classic_rally():
    ss = 2
    W, H = 1024 * ss, 512 * ss
    im_lat = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im_lat)

    ground_y = int(450 * ss)
    r_wheel = int(60 * ss)
    wf_x = int(740 * ss)
    wr_x = int(280 * ss)
    cy_wheel = ground_y - r_wheel - int(8 * ss)  # Raised rally suspension travel

    # Ground Shadow
    draw.ellipse(
        [int(140 * ss), ground_y - int(12 * ss), int(880 * ss), ground_y + int(24 * ss)],
        fill=(0, 0, 0, 140),
    )

    # Rally Mudflaps (Durable red mudflaps behind wheels)
    draw.polygon(
        [
            (wr_x - int(72 * ss), ground_y - int(10 * ss)),
            (wr_x - int(56 * ss), ground_y - int(10 * ss)),
            (wr_x - int(58 * ss), ground_y - int(70 * ss)),
            (wr_x - int(70 * ss), ground_y - int(70 * ss)),
        ],
        fill=(220, 35, 30),
    )
    draw.polygon(
        [
            (wf_x - int(72 * ss), ground_y - int(10 * ss)),
            (wf_x - int(56 * ss), ground_y - int(10 * ss)),
            (wf_x - int(58 * ss), ground_y - int(70 * ss)),
            (wf_x - int(70 * ss), ground_y - int(70 * ss)),
        ],
        fill=(220, 35, 30),
    )

    # Main Body Contours (Group B / WRC Turbo Hatchback)
    body_poly = [
        (int(165 * ss), ground_y - int(45 * ss)),   # Rear skidplate bottom
        (int(150 * ss), ground_y - int(95 * ss)),   # Rear bumper tip
        (int(160 * ss), ground_y - int(145 * ss)),  # Hatch crease
        (int(230 * ss), ground_y - int(210 * ss)),  # Hatch upper glass to roof
        (int(240 * ss), ground_y - int(215 * ss)),  # Roof spoiler mount
        (int(460 * ss), ground_y - int(218 * ss)),  # Roof mid
        (int(630 * ss), ground_y - int(215 * ss)),  # Roof front / A-pillar
        (int(730 * ss), ground_y - int(140 * ss)),  # Hood base / cowl
        (int(850 * ss), ground_y - int(115 * ss)),  # Hood front / light pod
        (int(875 * ss), ground_y - int(70 * ss)),   # Front bumper nose
        (int(865 * ss), ground_y - int(38 * ss)),   # Front skidplate
        (int(790 * ss), ground_y - int(38 * ss)),   # Under front overhang
        (int(680 * ss), ground_y - int(38 * ss)),   # Front sill
        (int(340 * ss), ground_y - int(38 * ss)),   # Rear sill
        (int(220 * ss), ground_y - int(38 * ss)),   # Under rear overhang
    ]
    # Primary Rally Sunburst Yellow coat
    draw.polygon(body_poly, fill=(242, 209, 25))

    # Box Flares / Wheel Arches (Shadow undercuts)
    draw.arc(
        [wr_x - int(75 * ss), cy_wheel - int(75 * ss), wr_x + int(75 * ss), cy_wheel + int(75 * ss)],
        start=180, end=360, fill=(40, 42, 48), width=int(8 * ss)
    )
    draw.arc(
        [wf_x - int(75 * ss), cy_wheel - int(75 * ss), wf_x + int(75 * ss), cy_wheel + int(75 * ss)],
        start=180, end=360, fill=(40, 42, 48), width=int(8 * ss)
    )

    # Dark Rally Greenhouse & Side Windows
    window_poly = [
        (int(250 * ss), ground_y - int(202 * ss)),  # Rear quarter glass top
        (int(620 * ss), ground_y - int(205 * ss)),  # Windshield top
        (int(705 * ss), ground_y - int(145 * ss)),  # Windshield bottom
        (int(470 * ss), ground_y - int(145 * ss)),  # B-pillar bottom
        (int(280 * ss), ground_y - int(145 * ss)),  # C-pillar bottom
    ]
    draw.polygon(window_poly, fill=(28, 32, 40))

    # Front door window glass
    draw.polygon(
        [
            (int(485 * ss), ground_y - int(198 * ss)),
            (int(605 * ss), ground_y - int(198 * ss)),
            (int(685 * ss), ground_y - int(148 * ss)),
            (int(485 * ss), ground_y - int(148 * ss)),
        ],
        fill=(90, 160, 220, 220),
        outline=(20, 22, 28),
        width=int(2 * ss),
    )
    # Rear quarter window glass
    draw.polygon(
        [
            (int(275 * ss), ground_y - int(195 * ss)),
            (int(465 * ss), ground_y - int(195 * ss)),
            (int(465 * ss), ground_y - int(148 * ss)),
            (int(315 * ss), ground_y - int(148 * ss)),
        ],
        fill=(70, 130, 190, 220),
        outline=(20, 22, 28),
        width=int(2 * ss),
    )

    # Roof Air Scoop (High-flow rally ventilation)
    draw.polygon(
        [
            (int(520 * ss), ground_y - int(216 * ss)),
            (int(590 * ss), ground_y - int(216 * ss)),
            (int(580 * ss), ground_y - int(238 * ss)),
            (int(535 * ss), ground_y - int(238 * ss)),
        ],
        fill=(28, 30, 36),
        outline=(242, 209, 25),
        width=int(1 * ss),
    )
    draw.polygon(
        [
            (int(580 * ss), ground_y - int(238 * ss)),
            (int(590 * ss), ground_y - int(216 * ss)),
            (int(598 * ss), ground_y - int(228 * ss)),
        ],
        fill=(15, 16, 20),
    )

    # Massive Group B Dual-Tier Rear Wing
    wing_poly = [
        (int(130 * ss), ground_y - int(250 * ss)),
        (int(220 * ss), ground_y - int(250 * ss)),
        (int(240 * ss), ground_y - int(215 * ss)),
        (int(190 * ss), ground_y - int(215 * ss)),
        (int(145 * ss), ground_y - int(235 * ss)),
    ]
    draw.polygon(wing_poly, fill=(28, 32, 40), outline=(242, 209, 25), width=int(2 * ss))
    # Wing endplate detail
    draw.polygon(
        [
            (int(125 * ss), ground_y - int(255 * ss)),
            (int(155 * ss), ground_y - int(255 * ss)),
            (int(170 * ss), ground_y - int(220 * ss)),
            (int(135 * ss), ground_y - int(220 * ss)),
        ],
        fill=(220, 35, 30),
    )

    # Front Quad Rally Fog Light Pod
    draw.polygon(
        [
            (int(830 * ss), ground_y - int(118 * ss)),
            (int(885 * ss), ground_y - int(112 * ss)),
            (int(890 * ss), ground_y - int(88 * ss)),
            (int(835 * ss), ground_y - int(92 * ss)),
        ],
        fill=(245, 245, 250),
        outline=(30, 32, 38),
        width=int(2 * ss),
    )
    draw_circle(draw, int(848 * ss), ground_y - int(105 * ss), int(12 * ss), fill=(255, 255, 220), outline=(180, 180, 200), width=int(1 * ss))
    draw_circle(draw, int(872 * ss), ground_y - int(102 * ss), int(12 * ss), fill=(255, 255, 220), outline=(180, 180, 200), width=int(1 * ss))

    # Bold Charcoal & White Rally Racing Livery Graphics
    draw.polygon(
        [
            (int(320 * ss), ground_y - int(38 * ss)),
            (int(460 * ss), ground_y - int(140 * ss)),
            (int(530 * ss), ground_y - int(140 * ss)),
            (int(390 * ss), ground_y - int(38 * ss)),
        ],
        fill=(28, 30, 36),
    )
    draw.polygon(
        [
            (int(405 * ss), ground_y - int(38 * ss)),
            (int(545 * ss), ground_y - int(140 * ss)),
            (int(570 * ss), ground_y - int(140 * ss)),
            (int(430 * ss), ground_y - int(38 * ss)),
        ],
        fill=(245, 245, 250),
    )

    # Rally Competition Number Decal #4
    draw.rectangle(
        [int(440 * ss), ground_y - int(115 * ss), int(500 * ss), ground_y - int(65 * ss)],
        fill=(245, 245, 250),
        outline=(28, 30, 36),
        width=int(2 * ss),
    )
    draw.line([int(475 * ss), ground_y - int(110 * ss), int(455 * ss), ground_y - int(80 * ss)], fill=(28, 30, 36), width=int(4 * ss))
    draw.line([int(450 * ss), ground_y - int(80 * ss), int(485 * ss), ground_y - int(80 * ss)], fill=(28, 30, 36), width=int(4 * ss))
    draw.line([int(475 * ss), ground_y - int(110 * ss), int(475 * ss), ground_y - int(70 * ss)], fill=(28, 30, 36), width=int(4 * ss))

    # Wheels (OZ-style white multi-spoke rally alloys)
    draw_wheel_lateral(draw, wr_x, cy_wheel, r_wheel, rim_color=(245, 245, 250), style="alloy", ss=ss)
    draw_wheel_lateral(draw, wf_x, cy_wheel, r_wheel, rim_color=(245, 245, 250), style="alloy", ss=ss)

    lat_img = im_lat.resize((1024, 512), Image.Resampling.LANCZOS)
    lat_img.save(LATERAL_DIR / "classic_rally.png")
    lat_thumb = lat_img.resize((256, 128), Image.Resampling.LANCZOS)
    lat_thumb.save(LATERAL_DIR / "classic_rally_thumb.png")

    # --- TOP-DOWN SPRITE (512x512) ---
    W_TD, H_TD = 512 * ss, 512 * ss
    im_td = Image.new("RGBA", (W_TD, H_TD), (0, 0, 0, 0))
    draw_td = ImageDraw.Draw(im_td)

    cx = 256 * ss
    cy = 256 * ss
    w_half = int(92 * ss)
    h_half = int(205 * ss)

    # Mudflaps top-down (protruding slightly behind rear wheels)
    draw_td.rectangle([cx - w_half - int(6 * ss), cy + int(140 * ss), cx - w_half + int(12 * ss), cy + int(148 * ss)], fill=(220, 35, 30))
    draw_td.rectangle([cx + w_half - int(12 * ss), cy + int(140 * ss), cx + w_half + int(6 * ss), cy + int(148 * ss)], fill=(220, 35, 30))

    # Wheels top-down
    for wx, wy in [
        (cx - w_half + int(12 * ss), cy - int(110 * ss)),
        (cx + w_half - int(12 * ss), cy - int(110 * ss)),
        (cx - w_half + int(12 * ss), cy + int(115 * ss)),
        (cx + w_half - int(12 * ss), cy + int(115 * ss)),
    ]:
        draw_td.rounded_rectangle([wx - int(15 * ss), wy - int(34 * ss), wx + int(15 * ss), wy + int(34 * ss)], radius=int(6 * ss), fill=(22, 24, 28))

    # Main Body Outline (Facing UP)
    top_poly = [
        (cx - int(48 * ss), cy - h_half),                 # Front bumper nose L
        (cx + int(48 * ss), cy - h_half),                 # Front bumper nose R
        (cx + int(80 * ss), cy - h_half + int(25 * ss)),  # Front light pod R
        (cx + int(94 * ss), cy - int(95 * ss)),           # Front box flare R
        (cx + int(84 * ss), cy - int(30 * ss)),           # Door waist R
        (cx + int(96 * ss), cy + int(95 * ss)),           # Rear box flare R
        (cx + int(90 * ss), cy + h_half - int(15 * ss)),  # Rear bumper R
        (cx + int(65 * ss), cy + h_half),                 # Rear hatch R
        (cx - int(65 * ss), cy + h_half),                 # Rear hatch L
        (cx - int(90 * ss), cy + h_half - int(15 * ss)),  # Rear bumper L
        (cx - int(96 * ss), cy + int(95 * ss)),           # Rear box flare L
        (cx - int(84 * ss), cy - int(30 * ss)),           # Door waist L
        (cx - int(94 * ss), cy - int(95 * ss)),           # Front box flare L
        (cx - int(80 * ss), cy - h_half + int(25 * ss)),  # Front light pod L
    ]
    draw_td.polygon(top_poly, fill=(242, 209, 25), outline=(190, 160, 20), width=int(2 * ss))

    # Quad Fog Lamp Pod on Nosecone
    draw_td.rounded_rectangle([cx - int(45 * ss), cy - h_half - int(8 * ss), cx + int(45 * ss), cy - h_half + int(18 * ss)], radius=int(4 * ss), fill=(245, 245, 250), outline=(28, 30, 36), width=int(2 * ss))
    for lx in [-32, -11, 11, 32]:
        draw_circle(draw_td, cx + int(lx * ss), cy - h_half + int(5 * ss), int(8 * ss), fill=(255, 255, 220), outline=(140, 140, 160), width=int(1 * ss))

    # Hood Louvers / Vents
    draw_td.rectangle([cx - int(28 * ss), cy - int(120 * ss), cx - int(8 * ss), cy - int(90 * ss)], fill=(28, 30, 36))
    draw_td.rectangle([cx + int(8 * ss), cy - int(120 * ss), cx + int(28 * ss), cy - int(90 * ss)], fill=(28, 30, 36))

    # Greenhouse (Cabin & Windows)
    greenhouse = [
        (cx - int(48 * ss), cy - int(65 * ss)),
        (cx + int(48 * ss), cy - int(65 * ss)),
        (cx + int(56 * ss), cy + int(85 * ss)),
        (cx - int(56 * ss), cy + int(85 * ss)),
    ]
    draw_td.polygon(greenhouse, fill=(26, 28, 34))

    # Windshield (Facing Up)
    draw_td.polygon(
        [
            (cx - int(44 * ss), cy - int(60 * ss)),
            (cx + int(44 * ss), cy - int(60 * ss)),
            (cx + int(48 * ss), cy - int(20 * ss)),
            (cx - int(48 * ss), cy - int(20 * ss)),
        ],
        fill=(90, 160, 220, 230),
    )
    # Rear Hatch Window
    draw_td.polygon(
        [
            (cx - int(48 * ss), cy + int(45 * ss)),
            (cx + int(48 * ss), cy + int(45 * ss)),
            (cx + int(52 * ss), cy + int(80 * ss)),
            (cx - int(52 * ss), cy + int(80 * ss)),
        ],
        fill=(65, 120, 180, 230),
    )

    # Roof Air Scoop (Topdown)
    draw_td.rectangle([cx - int(18 * ss), cy - int(18 * ss), cx + int(18 * ss), cy + int(15 * ss)], fill=(28, 30, 36), outline=(245, 245, 250), width=int(1 * ss))

    # Rear High-Downforce Rally Wing
    draw_td.rounded_rectangle([cx - int(82 * ss), cy + h_half - int(8 * ss), cx + int(82 * ss), cy + h_half + int(18 * ss)], radius=int(4 * ss), fill=(28, 32, 40), outline=(242, 209, 25), width=int(2 * ss))
    # Red Wing Endplates
    draw_td.rectangle([cx - int(85 * ss), cy + h_half - int(10 * ss), cx - int(78 * ss), cy + h_half + int(20 * ss)], fill=(220, 35, 30))
    draw_td.rectangle([cx + int(78 * ss), cy + h_half - int(10 * ss), cx + int(85 * ss), cy + h_half + int(20 * ss)], fill=(220, 35, 30))

    td_img = im_td.resize((512, 512), Image.Resampling.LANCZOS)
    td_img.save(TOPDOWN_DIR / "classic_rally.png")
    print("✓ Generated classic_rally assets")


def generate_kart_slick_wheel():
    ss = 2
    W, H = 128 * ss, 256 * ss
    im = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im)

    cx = W // 2
    cy = H // 2
    tw_half = int(52 * ss)  # 104px wide at 128x256 (leaves 12px transparent margin left/right)
    th_half = int(114 * ss) # 228px tall at 128x256 (leaves 14px transparent margin top/bottom)
    corner_r = int(22 * ss)

    # 1. Feathered outer bevel / edge
    draw.rounded_rectangle(
        [cx - tw_half, cy - th_half, cx + tw_half, cy + th_half],
        radius=corner_r,
        fill=(36, 40, 46, 255),
    )

    # 2. Main vulcanized black rubber tread
    inner_margin = int(3 * ss)
    draw.rounded_rectangle(
        [cx - tw_half + inner_margin, cy - th_half + inner_margin, cx + tw_half - inner_margin, cy + th_half - inner_margin],
        radius=corner_r - int(2 * ss),
        fill=(22, 24, 28, 255),
    )

    # 3. Subtle central tire contact patch line
    draw.line(
        [cx, cy - th_half + int(15 * ss), cx, cy + th_half - int(15 * ss)],
        fill=(16, 18, 20, 255),
        width=int(6 * ss),
    )

    # 4. Dark graphite alloy hub / rim center
    rw_half = int(18 * ss)
    rh_half = int(55 * ss)
    draw.rounded_rectangle(
        [cx - rw_half, cy - rh_half, cx + rw_half, cy + rh_half],
        radius=int(8 * ss),
        fill=(48, 52, 60, 255),
        outline=(75, 80, 92, 255),
        width=int(2 * ss),
    )

    # Rim inner structure
    draw.rounded_rectangle(
        [cx - rw_half + int(3 * ss), cy - rh_half + int(4 * ss), cx + rw_half - int(3 * ss), cy + rh_half - int(4 * ss)],
        radius=int(5 * ss),
        fill=(38, 42, 48, 255),
    )

    # 5. Anodized gold center hub nut / cap
    draw_circle(draw, cx, cy, int(10 * ss), fill=(215, 175, 45, 255), outline=(160, 125, 25, 255), width=int(1.5 * ss))
    draw_circle(draw, cx, cy, int(4 * ss), fill=(245, 210, 80, 255))

    out_img = im.resize((128, 256), Image.Resampling.LANCZOS)
    out_img.save(WHEELS_DIR / "kart_slick_front.png")
    print("✓ Generated kart_slick_front wheel asset (128x256)")


if __name__ == "__main__":
    generate_classic_gt()
    generate_classic_nascar()
    generate_classic_offroad()
    generate_classic_kart()
    generate_classic_rally()
    generate_kart_slick_wheel()
    print("✨ All 5 classic fantasy vehicle asset sets generated successfully!")
