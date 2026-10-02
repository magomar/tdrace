#!/usr/bin/env python3
"""
High-fidelity generator for the new and upgraded Classic Arcade fantasy vehicles:
1. classic_ax_mudlark: Mudlark Cross Car (Single-Seat 850cc RWD Cross Car)
2. classic_ax_brawler: Brawler Touring AX (Retro 1980s Silhouette AWD Touring Car)
3. classic_kart_vintage: Comet 100 Classic (1970s Air-Cooled Direct-Drive Kart)
4. classic_gt_vintage: Corsica '73 RS (1973 Air-Cooled Classic GT Sports Coupe)
5. classic_stock_vintage: Cyclone '69 Fastback (1969 Grand National Muscle Stocker)
6. classic_rx_vintage: Firebolt RS 2000 (1970s RWD Twin-Cam Rally Coupe)
7. classic_at_safari: Ironclad 4x4 Safari (1980s Paris-Dakar Vintage 4WD Trail Rig)
"""

import sys
import math
from pathlib import Path
from PIL import Image, ImageDraw, ImageFilter
import numpy as np

ROOT = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else Path.cwd()
LATERAL_DIR = ROOT / "assets" / "textures" / "vehicles" / "laterals" / "classic"
TOPDOWN_DIR = ROOT / "assets" / "textures" / "vehicles" / "topdown" / "classic"
WHEELS_DIR = ROOT / "assets" / "textures" / "vehicles" / "topdown" / "wheels"

LATERAL_DIR.mkdir(parents=True, exist_ok=True)
TOPDOWN_DIR.mkdir(parents=True, exist_ok=True)
WHEELS_DIR.mkdir(parents=True, exist_ok=True)


def draw_circle(draw, cx, cy, r, fill, outline=None, width=1):
    draw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=fill, outline=outline, width=width)


def draw_soft_shadow(draw, cx, cy, rx, ry, alpha=150):
    draw.ellipse([cx - rx, cy - ry, cx + rx, cy + ry], fill=(0, 0, 0, alpha))


def draw_glass_panel(draw, poly, base_col=(60, 130, 200, 220), reflection_col=(255, 255, 255, 140), ss=2):
    draw.polygon(poly, fill=base_col, outline=(20, 24, 30, 255), width=int(2 * ss))
    if len(poly) >= 4:
        p0, p1, p2, p3 = poly[:4]
        mid_x0 = (p0[0] + p1[0]) // 2
        mid_y0 = (p0[1] + p1[1]) // 2
        mid_x1 = (p2[0] + p3[0]) // 2
        mid_y1 = (p2[1] + p3[1]) // 2
        draw.line([mid_x0, mid_y0, mid_x1, mid_y1], fill=reflection_col, width=int(4 * ss))


def draw_realistic_wheel_lateral(draw, cx, cy, r, style="alloy", rim_col=(220, 220, 225), ss=2):
    # 1. Tire Rubber
    draw_circle(draw, cx, cy, r, fill=(24, 25, 28, 255), outline=(12, 13, 15, 255), width=int(2 * ss))
    draw_circle(draw, cx, cy, int(r * 0.92), fill=(34, 36, 40, 255), outline=(20, 21, 24, 255), width=int(1.5 * ss))
    
    # 2. Outer Rim Lip
    r_rim = int(r * 0.72)
    draw_circle(draw, cx, cy, r_rim, fill=(50, 53, 60, 255), outline=(180, 185, 195, 255), width=int(2.5 * ss))
    
    # 3. Inner recessed rim well / brake rotor
    r_rotor = int(r * 0.58)
    draw_circle(draw, cx, cy, r_rotor, fill=(80, 84, 92, 255), outline=(110, 115, 125, 255), width=int(1.5 * ss))
    for rot_ang in np.linspace(0, 2 * math.pi, 8, endpoint=False):
        rx = cx + math.cos(rot_ang) * (r_rotor * 0.70)
        ry = cy + math.sin(rot_ang) * (r_rotor * 0.70)
        draw_circle(draw, rx, ry, int(1.5 * ss), fill=(30, 32, 36, 255))
        
    # 4. Brake Caliper
    cal_ang = 0.75
    cal_x = cx + math.cos(cal_ang) * (r * 0.50)
    cal_y = cy - math.sin(cal_ang) * (r * 0.50)
    draw_circle(draw, cal_x, cal_y, int(r * 0.20), fill=(225, 35, 30, 255), outline=(140, 20, 15, 255), width=int(1.5 * ss))
    
    # 5. Spokes
    if style == "alloy" or style == "fuchs":
        num_spokes = 5 if style == "fuchs" else 8
        spoke_w = int(5 * ss) if style == "fuchs" else int(3 * ss)
        for i in range(num_spokes):
            ang = i * (2 * math.pi / num_spokes)
            x2 = cx + math.cos(ang) * (r_rim * 0.90)
            y2 = cy + math.sin(ang) * (r_rim * 0.90)
            draw.line([cx, cy, x2, y2], fill=rim_col, width=spoke_w)
    elif style == "steel":
        draw_circle(draw, cx, cy, int(r_rim * 0.88), fill=(26, 28, 32, 255), outline=(70, 75, 85, 255), width=int(2 * ss))
        for i in range(10):
            ang = i * (math.pi / 5)
            x2 = cx + math.cos(ang) * (r_rim * 0.60)
            y2 = cy + math.sin(ang) * (r_rim * 0.60)
            draw_circle(draw, x2, y2, int(r * 0.08), fill=(12, 14, 16, 255), outline=(120, 125, 135, 255), width=int(1 * ss))
    elif style == "retro_rally":
        for i in range(4):
            ang = i * (math.pi / 2)
            x2 = cx + math.cos(ang) * (r_rim * 0.92)
            y2 = cy + math.sin(ang) * (r_rim * 0.92)
            draw.line([cx, cy, x2, y2], fill=(245, 245, 250, 255), width=int(8 * ss))
    elif style == "knobby":
        draw_circle(draw, cx, cy, r_rim, fill=(35, 38, 44, 255), outline=(235, 160, 30, 255), width=int(3 * ss))
        for i in range(6):
            ang = i * (math.pi / 3)
            x2 = cx + math.cos(ang) * (r_rim * 0.75)
            y2 = cy + math.sin(ang) * (r_rim * 0.75)
            draw.line([cx, cy, x2, y2], fill=(60, 65, 75, 255), width=int(5 * ss))
    elif style == "kart":
        draw_circle(draw, cx, cy, int(r * 0.58), fill=(200, 165, 45, 255), outline=(140, 110, 25, 255), width=int(2 * ss))
        for i in range(3):
            ang = i * (2 * math.pi / 3)
            x2 = cx + math.cos(ang) * (r * 0.48)
            y2 = cy + math.sin(ang) * (r * 0.48)
            draw_circle(draw, x2, y2, int(r * 0.10), fill=(25, 28, 32, 255))

    # Center Hub & Lug Nuts
    draw_circle(draw, cx, cy, int(r * 0.18), fill=rim_col, outline=(20, 22, 26, 255), width=int(1.5 * ss))
    draw_circle(draw, cx, cy, int(r * 0.08), fill=(25, 27, 30, 255))


def save_dual_views(name, im_lat, im_td):
    # Lateral 1024x512
    lat_1024 = im_lat.resize((1024, 512), Image.Resampling.LANCZOS)
    lat_1024.save(LATERAL_DIR / f"{name}.png")
    # Thumbnail 256x128
    lat_thumb = lat_1024.resize((256, 128), Image.Resampling.LANCZOS)
    lat_thumb.save(LATERAL_DIR / f"{name}_thumb.png")
    # Topdown 512x512
    td_512 = im_td.resize((512, 512), Image.Resampling.LANCZOS)
    td_512.save(TOPDOWN_DIR / f"{name}.png")
    print(f"✓ Saved canonical assets for {name}")


# ==============================================================================
# 1. MUDLARK CROSS CAR (classic_ax_mudlark) - High-Fidelity Pre-Rendered
# ==============================================================================
def generate_classic_ax_mudlark():
    ss = 2
    W, H = 1024 * ss, 512 * ss
    im_lat = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im_lat)
    ground_y = int(450 * ss)
    r_wheel = int(62 * ss)
    wf_x = int(750 * ss)
    wr_x = int(270 * ss)
    cy_wheel = ground_y - r_wheel - int(8 * ss)

    # Shadow
    draw_soft_shadow(draw, int(512 * ss), ground_y, int(370 * ss), int(18 * ss), 160)

    frame_col = (45, 175, 235)  # Cyan
    shadow_col = (20, 24, 30)

    # Tubular Spaceframe Lower Rails
    draw.line([int(210 * ss), ground_y - int(45 * ss), int(730 * ss), ground_y - int(45 * ss)], fill=frame_col, width=int(8 * ss))
    draw.line([int(200 * ss), ground_y - int(85 * ss), int(710 * ss), ground_y - int(80 * ss)], fill=frame_col, width=int(8 * ss))
    
    # Roll Cage Uprights & Roof
    draw.line([int(350 * ss), ground_y - int(85 * ss), int(420 * ss), ground_y - int(220 * ss)], fill=frame_col, width=int(8 * ss))
    draw.line([int(420 * ss), ground_y - int(220 * ss), int(560 * ss), ground_y - int(220 * ss)], fill=frame_col, width=int(8 * ss))
    draw.line([int(560 * ss), ground_y - int(220 * ss), int(640 * ss), ground_y - int(85 * ss)], fill=frame_col, width=int(8 * ss))
    draw.line([int(350 * ss), ground_y - int(85 * ss), int(560 * ss), ground_y - int(220 * ss)], fill=frame_col, width=int(5 * ss))
    draw.line([int(420 * ss), ground_y - int(220 * ss), int(640 * ss), ground_y - int(85 * ss)], fill=frame_col, width=int(5 * ss))

    # Metallic pipe highlights (cylindrical shading)
    draw.line([int(210 * ss), ground_y - int(47 * ss), int(730 * ss), ground_y - int(47 * ss)], fill=(120, 220, 255), width=int(2 * ss))
    draw.line([int(420 * ss), ground_y - int(222 * ss), int(560 * ss), ground_y - int(222 * ss)], fill=(120, 220, 255), width=int(2 * ss))

    # Single-seater cockpit seat & driver
    draw.polygon([
        (int(440 * ss), ground_y - int(85 * ss)),
        (int(465 * ss), ground_y - int(170 * ss)),
        (int(505 * ss), ground_y - int(170 * ss)),
        (int(485 * ss), ground_y - int(85 * ss)),
    ], fill=(25, 27, 32))
    # Driver helmet with dark visor & reflection
    draw_circle(draw, int(485 * ss), ground_y - int(185 * ss), int(22 * ss), fill=(245, 245, 250), outline=(20, 22, 28), width=int(2 * ss))
    draw.polygon([
        (int(492 * ss), ground_y - int(192 * ss)),
        (int(506 * ss), ground_y - int(192 * ss)),
        (int(504 * ss), ground_y - int(178 * ss)),
        (int(490 * ss), ground_y - int(178 * ss)),
    ], fill=(20, 25, 35))
    draw.line([int(494 * ss), ground_y - int(185 * ss), int(504 * ss), ground_y - int(185 * ss)], fill=(200, 220, 255), width=int(1.5 * ss))

    # Sculpted composite nosecone
    nose_poly = [
        (int(630 * ss), ground_y - int(90 * ss)),
        (int(745 * ss), ground_y - int(70 * ss)),
        (int(775 * ss), ground_y - int(48 * ss)),
        (int(710 * ss), ground_y - int(42 * ss)),
    ]
    draw.polygon(nose_poly, fill=(45, 175, 235), outline=(20, 80, 120), width=int(2 * ss))
    # Nosecone highlight ridge
    draw.line([(int(635 * ss), ground_y - int(85 * ss)), (int(765 * ss), ground_y - int(52 * ss))], fill=(130, 225, 255), width=int(3 * ss))

    # Rear motorcycle engine & exhaust
    draw.rectangle([int(180 * ss), ground_y - int(145 * ss), int(255 * ss), ground_y - int(115 * ss)], fill=(170, 175, 185), outline=(50, 55, 65), width=int(2 * ss))
    # Titanium exhaust with heat-blued gradient
    draw.line([int(255 * ss), ground_y - int(130 * ss), int(330 * ss), ground_y - int(85 * ss)], fill=(120, 130, 160), width=int(6 * ss))
    draw.line([int(180 * ss), ground_y - int(130 * ss), int(210 * ss), ground_y - int(130 * ss)], fill=(60, 100, 200), width=int(6 * ss))

    # Wheels
    draw_realistic_wheel_lateral(draw, wr_x, cy_wheel, int(r_wheel * 1.05), style="knobby", ss=ss)
    draw_realistic_wheel_lateral(draw, wf_x, cy_wheel, r_wheel, style="knobby", ss=ss)

    # Top-Down Sprite (512x512)
    W_TD, H_TD = 512 * ss, 512 * ss
    im_td = Image.new("RGBA", (W_TD, H_TD), (0, 0, 0, 0))
    draw_td = ImageDraw.Draw(im_td)
    cx, cy = 256 * ss, 256 * ss

    # Facing RIGHT (+X): nose at right (+X), rear at left (-X)
    # Suspension Wishbones & Tie Rods
    # Front FL (Y=175) & FR (Y=337)
    draw_td.line([int(350 * ss), cy - int(45 * ss), int(380 * ss), cy - int(95 * ss)], fill=(160, 165, 175), width=int(6 * ss))
    draw_td.line([int(390 * ss), cy - int(45 * ss), int(380 * ss), cy - int(95 * ss)], fill=(160, 165, 175), width=int(6 * ss))
    draw_td.line([int(350 * ss), cy + int(45 * ss), int(380 * ss), cy + int(95 * ss)], fill=(160, 165, 175), width=int(6 * ss))
    draw_td.line([int(390 * ss), cy + int(45 * ss), int(380 * ss), cy + int(95 * ss)], fill=(160, 165, 175), width=int(6 * ss))

    # Rear Wishbones RL & RR
    draw_td.line([int(150 * ss), cy - int(45 * ss), int(130 * ss), cy - int(95 * ss)], fill=(160, 165, 175), width=int(6 * ss))
    draw_td.line([int(190 * ss), cy - int(45 * ss), int(130 * ss), cy - int(95 * ss)], fill=(160, 165, 175), width=int(6 * ss))
    draw_td.line([int(150 * ss), cy + int(45 * ss), int(130 * ss), cy + int(95 * ss)], fill=(160, 165, 175), width=int(6 * ss))
    draw_td.line([int(190 * ss), cy + int(45 * ss), int(130 * ss), cy + int(95 * ss)], fill=(160, 165, 175), width=int(6 * ss))

    # Wheels (Static for showroom)
    # Front FL & FR (X: 340..420)
    draw_td.rounded_rectangle([int(345 * ss), cy - int(120 * ss), int(415 * ss), cy - int(70 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    draw_td.rounded_rectangle([int(345 * ss), cy + int(70 * ss), int(415 * ss), cy + int(120 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    # Rear RL & RR (X: 95..175)
    draw_td.rounded_rectangle([int(95 * ss), cy - int(125 * ss), int(175 * ss), cy - int(65 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    draw_td.rounded_rectangle([int(95 * ss), cy + int(65 * ss), int(175 * ss), cy + int(125 * ss)], radius=int(6 * ss), fill=(22, 24, 28))

    # Main Tubular Cage & Paneling
    draw_td.rectangle([int(190 * ss), cy - int(45 * ss), int(330 * ss), cy + int(45 * ss)], outline=(45, 175, 235), fill=(25, 28, 34), width=int(6 * ss))
    # Shaded Fiberglass Nosecone (+X: 330..445)
    nose_td = [
        (int(330 * ss), cy - int(40 * ss)),
        (int(440 * ss), cy - int(20 * ss)),
        (int(450 * ss), cy),
        (int(440 * ss), cy + int(20 * ss)),
        (int(330 * ss), cy + int(40 * ss)),
    ]
    draw_td.polygon(nose_td, fill=(45, 175, 235), outline=(20, 80, 120), width=int(2 * ss))
    draw_td.line([int(330 * ss), cy, int(445 * ss), cy], fill=(130, 225, 255), width=int(3 * ss))

    # Cockpit Driver in Center
    draw_td.rectangle([int(230 * ss), cy - int(22 * ss), int(290 * ss), cy + int(22 * ss)], fill=(18, 20, 24))
    draw_circle(draw_td, int(260 * ss), cy, int(18 * ss), fill=(245, 245, 250), outline=(20, 22, 28), width=int(2 * ss))
    draw_td.line([int(268 * ss), cy - int(10 * ss), int(268 * ss), cy + int(10 * ss)], fill=(20, 25, 35), width=int(4 * ss))

    # Rear Engine Block & Radiators
    draw_td.rectangle([int(130 * ss), cy - int(38 * ss), int(190 * ss), cy + int(38 * ss)], fill=(85, 90, 100), outline=(40, 44, 52), width=int(2 * ss))
    draw_td.line([int(130 * ss), cy + int(25 * ss), int(90 * ss), cy + int(25 * ss)], fill=(160, 165, 180), width=int(5 * ss))

    save_dual_views("classic_ax_mudlark", im_lat, im_td)


# ==============================================================================
# 2. BRAWLER TOURING AX (classic_ax_brawler) - High-Fidelity Pre-Rendered
# ==============================================================================
def generate_classic_ax_brawler():
    ss = 2
    W, H = 1024 * ss, 512 * ss
    im_lat = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im_lat)
    ground_y = int(450 * ss)
    r_wheel = int(62 * ss)
    wf_x = int(740 * ss)
    wr_x = int(275 * ss)
    cy_wheel = ground_y - r_wheel - int(6 * ss)

    # Shadow
    draw_soft_shadow(draw, int(512 * ss), ground_y, int(390 * ss), int(20 * ss), 170)

    # Red Mudflaps behind wheels
    draw.polygon([(wr_x - int(72 * ss), ground_y - int(10 * ss)), (wr_x - int(56 * ss), ground_y - int(10 * ss)), (wr_x - int(58 * ss), ground_y - int(70 * ss)), (wr_x - int(70 * ss), ground_y - int(70 * ss))], fill=(180, 25, 20))
    draw.polygon([(wf_x - int(72 * ss), ground_y - int(10 * ss)), (wf_x - int(56 * ss), ground_y - int(10 * ss)), (wf_x - int(58 * ss), ground_y - int(70 * ss)), (wf_x - int(70 * ss), ground_y - int(70 * ss))], fill=(180, 25, 20))

    # Main Bodywork (Muscular 1980s Touring Silhouette)
    body_poly = [
        (int(165 * ss), ground_y - int(45 * ss)),
        (int(150 * ss), ground_y - int(100 * ss)),
        (int(160 * ss), ground_y - int(155 * ss)),
        (int(240 * ss), ground_y - int(215 * ss)),
        (int(460 * ss), ground_y - int(218 * ss)),
        (int(630 * ss), ground_y - int(215 * ss)),
        (int(730 * ss), ground_y - int(140 * ss)),
        (int(855 * ss), ground_y - int(115 * ss)),
        (int(880 * ss), ground_y - int(70 * ss)),
        (int(870 * ss), ground_y - int(38 * ss)),
        (int(790 * ss), ground_y - int(38 * ss)),
        (int(680 * ss), ground_y - int(38 * ss)),
        (int(340 * ss), ground_y - int(38 * ss)),
        (int(220 * ss), ground_y - int(38 * ss)),
    ]
    draw.polygon(body_poly, fill=(215, 55, 45), outline=(140, 25, 20), width=int(2 * ss))

    # Box Wheel Flares (Deep inner well shadows)
    draw.arc([wr_x - int(76 * ss), cy_wheel - int(76 * ss), wr_x + int(76 * ss), cy_wheel + int(76 * ss)], start=180, end=360, fill=(35, 38, 44), width=int(12 * ss))
    draw.arc([wf_x - int(76 * ss), cy_wheel - int(76 * ss), wf_x + int(76 * ss), cy_wheel + int(76 * ss)], start=180, end=360, fill=(35, 38, 44), width=int(12 * ss))

    # Greenhouse & Windows
    window_poly = [
        (int(255 * ss), ground_y - int(205 * ss)),
        (int(620 * ss), ground_y - int(205 * ss)),
        (int(710 * ss), ground_y - int(145 * ss)),
        (int(465 * ss), ground_y - int(145 * ss)),
        (int(275 * ss), ground_y - int(145 * ss)),
    ]
    draw_glass_panel(draw, window_poly, base_col=(70, 140, 210, 220), ss=ss)

    # Massive Rear Touring Wing
    wing_poly = [
        (int(130 * ss), ground_y - int(255 * ss)),
        (int(220 * ss), ground_y - int(255 * ss)),
        (int(240 * ss), ground_y - int(215 * ss)),
        (int(190 * ss), ground_y - int(215 * ss)),
        (int(145 * ss), ground_y - int(235 * ss)),
    ]
    draw.polygon(wing_poly, fill=(28, 30, 36), outline=(215, 60, 50), width=int(2 * ss))

    # Wheels
    draw_realistic_wheel_lateral(draw, wr_x, cy_wheel, r_wheel, style="retro_rally", ss=ss)
    draw_realistic_wheel_lateral(draw, wf_x, cy_wheel, r_wheel, style="retro_rally", ss=ss)

    # Top-Down Sprite (512x512)
    W_TD, H_TD = 512 * ss, 512 * ss
    im_td = Image.new("RGBA", (W_TD, H_TD), (0, 0, 0, 0))
    draw_td = ImageDraw.Draw(im_td)
    cx, cy = 256 * ss, 256 * ss

    # Static wheels for showroom
    # Front FL & FR (X: 340..420)
    draw_td.rounded_rectangle([int(345 * ss), cy - int(115 * ss), int(415 * ss), cy - int(65 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    draw_td.rounded_rectangle([int(345 * ss), cy + int(65 * ss), int(415 * ss), cy + int(115 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    # Rear RL & RR (X: 95..175)
    draw_td.rounded_rectangle([int(95 * ss), cy - int(115 * ss), int(175 * ss), cy - int(65 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    draw_td.rounded_rectangle([int(95 * ss), cy + int(65 * ss), int(175 * ss), cy + int(115 * ss)], radius=int(6 * ss), fill=(22, 24, 28))

    # Widebody Top-Down Silhouette (Facing +X)
    top_poly = [
        (int(465 * ss), cy - int(50 * ss)),
        (int(475 * ss), cy),
        (int(465 * ss), cy + int(50 * ss)),
        (int(410 * ss), cy + int(98 * ss)),  # Front fender R
        (int(335 * ss), cy + int(98 * ss)),
        (int(290 * ss), cy + int(85 * ss)),  # Waist R
        (int(185 * ss), cy + int(100 * ss)), # Rear fender R
        (int(90 * ss), cy + int(95 * ss)),
        (int(60 * ss), cy + int(65 * ss)),   # Rear bumper R
        (int(50 * ss), cy),
        (int(60 * ss), cy - int(65 * ss)),   # Rear bumper L
        (int(90 * ss), cy - int(95 * ss)),
        (int(185 * ss), cy - int(100 * ss)), # Rear fender L
        (int(290 * ss), cy - int(85 * ss)),  # Waist L
        (int(335 * ss), cy - int(98 * ss)),
        (int(410 * ss), cy - int(98 * ss)),  # Front fender L
    ]
    draw_td.polygon(top_poly, fill=(215, 55, 45), outline=(140, 25, 20), width=int(2 * ss))

    # Cabin & Glass (Facing +X)
    greenhouse = [
        (int(340 * ss), cy - int(48 * ss)),
        (int(340 * ss), cy + int(48 * ss)),
        (int(180 * ss), cy + int(52 * ss)),
        (int(180 * ss), cy - int(52 * ss)),
    ]
    draw_td.polygon(greenhouse, fill=(25, 28, 34))
    # Windshield
    draw_td.polygon([(int(335 * ss), cy - int(42 * ss)), (int(335 * ss), cy + int(42 * ss)), (int(295 * ss), cy + int(45 * ss)), (int(295 * ss), cy - int(45 * ss))], fill=(90, 160, 220, 230))
    # Rear Window
    draw_td.polygon([(int(220 * ss), cy - int(45 * ss)), (int(220 * ss), cy + int(45 * ss)), (int(185 * ss), cy + int(45 * ss)), (int(185 * ss), cy - int(45 * ss))], fill=(65, 120, 180, 230))

    # Roof Air Scoop & Rear Wing
    draw_td.rectangle([int(275 * ss), cy - int(15 * ss), int(310 * ss), cy + int(15 * ss)], fill=(28, 30, 36), outline=(225, 230, 240), width=int(1 * ss))
    draw_td.rounded_rectangle([int(45 * ss), cy - int(85 * ss), int(75 * ss), cy + int(85 * ss)], radius=int(4 * ss), fill=(28, 30, 36), outline=(215, 55, 45), width=int(2 * ss))

    save_dual_views("classic_ax_brawler", im_lat, im_td)


# ==============================================================================
# 3. COMET 100 CLASSIC (classic_kart_vintage) - 1970s Air-Cooled Direct-Drive Kart
# ==============================================================================
def generate_classic_kart_vintage():
    ss = 2
    W, H = 1024 * ss, 512 * ss
    im_lat = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im_lat)
    ground_y = int(450 * ss)
    r_wheel = int(44 * ss)
    wf_x = int(730 * ss)
    wr_x = int(280 * ss)
    cy_wheel = ground_y - r_wheel

    # Shadow
    draw_soft_shadow(draw, int(512 * ss), ground_y, int(310 * ss), int(14 * ss), 150)

    # 1970s Low-Slung Chrome Frame Rails & Nerf Bars
    draw.line([int(230 * ss), ground_y - int(24 * ss), int(760 * ss), ground_y - int(24 * ss)], fill=(225, 230, 240), width=int(5 * ss))
    # Chrome side nerf bar
    draw.line([int(320 * ss), ground_y - int(45 * ss), int(670 * ss), ground_y - int(45 * ss)], fill=(225, 230, 240), width=int(4 * ss))
    draw.line([int(320 * ss), ground_y - int(24 * ss), int(320 * ss), ground_y - int(45 * ss)], fill=(225, 230, 240), width=int(4 * ss))
    draw.line([int(670 * ss), ground_y - int(24 * ss), int(670 * ss), ground_y - int(45 * ss)], fill=(225, 230, 240), width=int(4 * ss))

    # Vintage Molded Seat & Driver
    draw.polygon([(int(430 * ss), ground_y - int(28 * ss)), (int(450 * ss), ground_y - int(120 * ss)), (int(490 * ss), ground_y - int(120 * ss)), (int(475 * ss), ground_y - int(28 * ss))], fill=(240, 190, 50))
    # Driver helmet (classic 70s open-face with snap visor)
    draw_circle(draw, int(470 * ss), ground_y - int(150 * ss), int(22 * ss), fill=(245, 245, 250), outline=(25, 35, 60), width=int(2 * ss))
    draw.arc([int(455 * ss), ground_y - int(165 * ss), int(485 * ss), ground_y - int(135 * ss)], start=280, end=380, fill=(30, 45, 90), width=int(5 * ss))

    # Steering Column & Wheel
    draw.line([int(530 * ss), ground_y - int(35 * ss), int(590 * ss), ground_y - int(110 * ss)], fill=(180, 185, 195), width=int(4 * ss))
    draw.line([int(580 * ss), ground_y - int(120 * ss), int(605 * ss), ground_y - int(100 * ss)], fill=(25, 28, 34), width=int(6 * ss))

    # Air-cooled 100cc engine (right side / behind seat)
    draw.rectangle([int(340 * ss), ground_y - int(95 * ss), int(410 * ss), ground_y - int(35 * ss)], fill=(150, 155, 165), outline=(40, 45, 55), width=int(2 * ss))
    # Horizontal cooling fins
    for fy in range(ground_y - int(90 * ss), ground_y - int(40 * ss), int(8 * ss)):
        draw.line([int(335 * ss), fy, int(415 * ss), fy], fill=(210, 215, 225), width=int(2 * ss))
    # Expansion chamber exhaust
    draw.line([int(230 * ss), ground_y - int(35 * ss), int(340 * ss), ground_y - int(65 * ss)], fill=(80, 85, 95), width=int(8 * ss))

    # Wheels
    draw_realistic_wheel_lateral(draw, wr_x, cy_wheel, r_wheel, style="kart", ss=ss)
    draw_realistic_wheel_lateral(draw, wf_x, cy_wheel, r_wheel, style="kart", ss=ss)

    # Top-Down Sprite (512x512)
    W_TD, H_TD = 512 * ss, 512 * ss
    im_td = Image.new("RGBA", (W_TD, H_TD), (0, 0, 0, 0))
    draw_td = ImageDraw.Draw(im_td)
    cx, cy = 256 * ss, 256 * ss

    # Static wheels for showroom (+X is forward)
    # Front FL & FR (X: 320..385, Y: 155..205 and Y: 307..357)
    draw_td.rounded_rectangle([int(320 * ss), cy - int(105 * ss), int(385 * ss), cy - int(65 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    draw_td.rounded_rectangle([int(320 * ss), cy + int(65 * ss), int(385 * ss), cy + int(105 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    # Rear RL & RR (X: 110..185)
    draw_td.rounded_rectangle([int(110 * ss), cy - int(115 * ss), int(185 * ss), cy - int(60 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    draw_td.rounded_rectangle([int(110 * ss), cy + int(60 * ss), int(185 * ss), cy + int(115 * ss)], radius=int(6 * ss), fill=(22, 24, 28))

    # Chrome side nerf bars (+X forward)
    draw_td.line([int(200 * ss), cy - int(80 * ss), int(330 * ss), cy - int(80 * ss)], fill=(225, 230, 240), width=int(4 * ss))
    draw_td.line([int(200 * ss), cy + int(80 * ss), int(330 * ss), cy + int(80 * ss)], fill=(225, 230, 240), width=int(4 * ss))
    draw_td.line([int(200 * ss), cy - int(80 * ss), int(180 * ss), cy - int(45 * ss)], fill=(225, 230, 240), width=int(4 * ss))
    draw_td.line([int(200 * ss), cy + int(80 * ss), int(180 * ss), cy + int(45 * ss)], fill=(225, 230, 240), width=int(4 * ss))

    # Minimal Vintage Nosecone
    draw_td.polygon([(int(320 * ss), cy - int(35 * ss)), (int(410 * ss), cy - int(18 * ss)), (int(415 * ss), cy), (int(410 * ss), cy + int(18 * ss)), (int(320 * ss), cy + int(35 * ss))], fill=(240, 190, 50), outline=(180, 135, 20), width=int(2 * ss))
    # Number plate on nosecone
    draw_td.rectangle([int(350 * ss), cy - int(15 * ss), int(385 * ss), cy + int(15 * ss)], fill=(255, 255, 255), outline=(20, 24, 30), width=int(1 * ss))

    # Vintage Driver & Seat
    draw_circle(draw_td, int(250 * ss), cy, int(18 * ss), fill=(245, 245, 250), outline=(30, 45, 90), width=int(2 * ss))
    draw_td.rectangle([int(200 * ss), cy - int(24 * ss), int(240 * ss), cy + int(24 * ss)], fill=(240, 190, 50))

    # Right-Mounted Air-Cooled Engine (at bottom Y in topdown, Y > cy)
    draw_td.rectangle([int(180 * ss), cy + int(30 * ss), int(230 * ss), cy + int(60 * ss)], fill=(160, 165, 175), outline=(40, 45, 55), width=int(2 * ss))

    save_dual_views("classic_kart_vintage", im_lat, im_td)


# ==============================================================================
# 4. CORSICA '73 RS (classic_gt_vintage) - 1973 Air-Cooled Classic GT
# ==============================================================================
def generate_classic_gt_vintage():
    ss = 2
    W, H = 1024 * ss, 512 * ss
    im_lat = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im_lat)
    ground_y = int(450 * ss)
    r_wheel = int(58 * ss)
    wf_x = int(740 * ss)
    wr_x = int(280 * ss)
    cy_wheel = ground_y - r_wheel

    # Shadow
    draw_soft_shadow(draw, int(512 * ss), ground_y, int(390 * ss), int(20 * ss), 160)

    # 1973 Fastback Silhouette (Classic Sports Coupe)
    body_poly = [
        (int(150 * ss), ground_y - int(32 * ss)),
        (int(140 * ss), ground_y - int(85 * ss)),
        (int(155 * ss), ground_y - int(130 * ss)),  # Ducktail start
        (int(200 * ss), ground_y - int(155 * ss)),  # Ducktail tip
        (int(215 * ss), ground_y - int(140 * ss)),  # Ducktail base
        (int(310 * ss), ground_y - int(150 * ss)),  # Fastback slope
        (int(420 * ss), ground_y - int(205 * ss)),  # Roof peak
        (int(580 * ss), ground_y - int(205 * ss)),  # Roof front
        (int(690 * ss), ground_y - int(135 * ss)),  # Cowl
        (int(830 * ss), ground_y - int(105 * ss)),  # Front fender
        (int(890 * ss), ground_y - int(55 * ss)),   # Nosecone
        (int(885 * ss), ground_y - int(24 * ss)),   # Front chin
        (int(820 * ss), ground_y - int(24 * ss)),
        (int(680 * ss), ground_y - int(24 * ss)),
        (int(340 * ss), ground_y - int(24 * ss)),
        (int(210 * ss), ground_y - int(24 * ss)),
    ]
    draw.polygon(body_poly, fill=(35, 140, 75), outline=(20, 85, 45), width=int(2 * ss))

    # Chrome rocker strip & window trim
    draw.line([int(340 * ss), ground_y - int(26 * ss), int(680 * ss), ground_y - int(26 * ss)], fill=(235, 238, 245), width=int(3 * ss))

    # Classic Greenhouse & Side Windows
    window_poly = [
        (int(320 * ss), ground_y - int(145 * ss)),
        (int(425 * ss), ground_y - int(195 * ss)),
        (int(575 * ss), ground_y - int(195 * ss)),
        (int(675 * ss), ground_y - int(142 * ss)),
        (int(440 * ss), ground_y - int(145 * ss)),
    ]
    draw_glass_panel(draw, window_poly, base_col=(70, 140, 210, 210), ss=ss)

    # Round Chrome-Ringed Headlight
    draw_circle(draw, int(840 * ss), ground_y - int(95 * ss), int(15 * ss), fill=(255, 255, 235), outline=(220, 225, 235), width=int(3 * ss))

    # Wheels
    draw_realistic_wheel_lateral(draw, wr_x, cy_wheel, r_wheel, style="fuchs", rim_col=(235, 238, 245), ss=ss)
    draw_realistic_wheel_lateral(draw, wf_x, cy_wheel, r_wheel, style="fuchs", rim_col=(235, 238, 245), ss=ss)

    # Top-Down Sprite (512x512)
    W_TD, H_TD = 512 * ss, 512 * ss
    im_td = Image.new("RGBA", (W_TD, H_TD), (0, 0, 0, 0))
    draw_td = ImageDraw.Draw(im_td)
    cx, cy = 256 * ss, 256 * ss

    # Static wheels for showroom (+X forward)
    # Front FL & FR (X: 335..415)
    draw_td.rounded_rectangle([int(340 * ss), cy - int(110 * ss), int(410 * ss), cy - int(65 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    draw_td.rounded_rectangle([int(340 * ss), cy + int(65 * ss), int(410 * ss), cy + int(110 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    # Rear RL & RR (X: 95..175)
    draw_td.rounded_rectangle([int(95 * ss), cy - int(115 * ss), int(175 * ss), cy - int(65 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    draw_td.rounded_rectangle([int(95 * ss), cy + int(65 * ss), int(175 * ss), cy + int(115 * ss)], radius=int(6 * ss), fill=(22, 24, 28))

    # Classic Curvaceous Body (Facing +X)
    top_poly = [
        (int(465 * ss), cy - int(45 * ss)),
        (int(480 * ss), cy),
        (int(465 * ss), cy + int(45 * ss)),
        (int(410 * ss), cy + int(90 * ss)),
        (int(335 * ss), cy + int(90 * ss)),
        (int(280 * ss), cy + int(78 * ss)),  # Tapered waist
        (int(190 * ss), cy + int(96 * ss)),  # Wide flared rear hips
        (int(85 * ss), cy + int(90 * ss)),
        (int(50 * ss), cy + int(60 * ss)),   # Ducktail rear
        (int(45 * ss), cy),
        (int(50 * ss), cy - int(60 * ss)),
        (int(85 * ss), cy - int(90 * ss)),
        (int(190 * ss), cy - int(96 * ss)),
        (int(280 * ss), cy - int(78 * ss)),
        (int(335 * ss), cy - int(90 * ss)),
        (int(410 * ss), cy - int(90 * ss)),
    ]
    draw_td.polygon(top_poly, fill=(35, 140, 75), outline=(20, 85, 45), width=int(2 * ss))

    # Dual Cream Racing Stripes along Center
    draw_td.line([int(50 * ss), cy - int(8 * ss), int(475 * ss), cy - int(8 * ss)], fill=(245, 235, 215), width=int(6 * ss))
    draw_td.line([int(50 * ss), cy + int(8 * ss), int(475 * ss), cy + int(8 * ss)], fill=(245, 235, 215), width=int(6 * ss))

    # Windshield & Rear Glass
    draw_td.polygon([(int(340 * ss), cy - int(40 * ss)), (int(340 * ss), cy + int(40 * ss)), (int(290 * ss), cy + int(44 * ss)), (int(290 * ss), cy - int(44 * ss))], fill=(90, 160, 220, 230), outline=(225, 230, 240), width=int(2 * ss))
    draw_td.polygon([(int(205 * ss), cy - int(42 * ss)), (int(205 * ss), cy + int(42 * ss)), (int(150 * ss), cy + int(38 * ss)), (int(150 * ss), cy - int(38 * ss))], fill=(65, 120, 180, 230), outline=(225, 230, 240), width=int(2 * ss))

    save_dual_views("classic_gt_vintage", im_lat, im_td)


# ==============================================================================
# 5. CYCLONE '69 FASTBACK (classic_stock_vintage) - 1969 Grand National Muscle
# ==============================================================================
def generate_classic_stock_vintage():
    ss = 2
    W, H = 1024 * ss, 512 * ss
    im_lat = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im_lat)
    ground_y = int(450 * ss)
    r_wheel = int(64 * ss)
    wf_x = int(745 * ss)
    wr_x = int(270 * ss)
    cy_wheel = ground_y - r_wheel

    # Shadow
    draw_soft_shadow(draw, int(512 * ss), ground_y, int(410 * ss), int(22 * ss), 170)

    # 1969 Muscle Fastback Profile
    body_poly = [
        (int(135 * ss), ground_y - int(38 * ss)),
        (int(125 * ss), ground_y - int(95 * ss)),   # Rear chrome bumper
        (int(135 * ss), ground_y - int(135 * ss)),  # Trunk lip
        (int(250 * ss), ground_y - int(150 * ss)),  # Fastback glass
        (int(420 * ss), ground_y - int(210 * ss)),  # Roof
        (int(590 * ss), ground_y - int(210 * ss)),  # A-pillar
        (int(700 * ss), ground_y - int(140 * ss)),  # Cowl induction hood
        (int(865 * ss), ground_y - int(120 * ss)),  # Long hood
        (int(890 * ss), ground_y - int(75 * ss)),   # Vertical front grille
        (int(880 * ss), ground_y - int(35 * ss)),   # Front chrome bumper
        (int(820 * ss), ground_y - int(35 * ss)),
        (int(680 * ss), ground_y - int(35 * ss)),
        (int(340 * ss), ground_y - int(35 * ss)),
        (int(210 * ss), ground_y - int(35 * ss)),
    ]
    draw.polygon(body_poly, fill=(190, 48, 38), outline=(120, 25, 20), width=int(2 * ss))

    # Recessed Front Grille & Round Dual Headlights
    draw.rectangle([int(875 * ss), ground_y - int(110 * ss), int(885 * ss), ground_y - int(75 * ss)], fill=(25, 28, 34))
    draw_circle(draw, int(875 * ss), ground_y - int(92 * ss), int(10 * ss), fill=(255, 255, 220), outline=(225, 230, 240), width=int(2 * ss))

    # Fastback Window Glass
    window_poly = [
        (int(260 * ss), ground_y - int(155 * ss)),
        (int(430 * ss), ground_y - int(202 * ss)),
        (int(585 * ss), ground_y - int(202 * ss)),
        (int(685 * ss), ground_y - int(145 * ss)),
        (int(450 * ss), ground_y - int(145 * ss)),
    ]
    draw_glass_panel(draw, window_poly, base_col=(70, 140, 210, 220), ss=ss)

    # Wheels
    draw_realistic_wheel_lateral(draw, wr_x, cy_wheel, r_wheel, style="steel", ss=ss)
    draw_realistic_wheel_lateral(draw, wf_x, cy_wheel, r_wheel, style="steel", ss=ss)

    # Top-Down Sprite (512x512)
    W_TD, H_TD = 512 * ss, 512 * ss
    im_td = Image.new("RGBA", (W_TD, H_TD), (0, 0, 0, 0))
    draw_td = ImageDraw.Draw(im_td)
    cx, cy = 256 * ss, 256 * ss

    # Static wheels for showroom (+X forward)
    # Front FL & FR (X: 340..425)
    draw_td.rounded_rectangle([int(345 * ss), cy - int(118 * ss), int(420 * ss), cy - int(68 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    draw_td.rounded_rectangle([int(345 * ss), cy + int(68 * ss), int(420 * ss), cy + int(118 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    # Rear RL & RR (X: 90..175)
    draw_td.rounded_rectangle([int(95 * ss), cy - int(118 * ss), int(170 * ss), cy - int(68 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    draw_td.rounded_rectangle([int(95 * ss), cy + int(68 * ss), int(170 * ss), cy + int(118 * ss)], radius=int(6 * ss), fill=(22, 24, 28))

    # Broad Square Muscle Car Contour (Facing +X)
    top_poly = [
        (int(475 * ss), cy - int(55 * ss)),
        (int(485 * ss), cy),
        (int(475 * ss), cy + int(55 * ss)),
        (int(425 * ss), cy + int(98 * ss)),  # Front fender R
        (int(335 * ss), cy + int(98 * ss)),
        (int(280 * ss), cy + int(92 * ss)),  # Door
        (int(185 * ss), cy + int(100 * ss)), # Rear quarter R
        (int(80 * ss), cy + int(95 * ss)),
        (int(45 * ss), cy + int(60 * ss)),   # Rear trunk
        (int(40 * ss), cy),
        (int(45 * ss), cy - int(60 * ss)),
        (int(80 * ss), cy - int(95 * ss)),
        (int(185 * ss), cy - int(100 * ss)),
        (int(280 * ss), cy - int(92 * ss)),
        (int(335 * ss), cy - int(98 * ss)),
        (int(425 * ss), cy - int(98 * ss)),
    ]
    draw_td.polygon(top_poly, fill=(190, 48, 38), outline=(120, 25, 20), width=int(2 * ss))

    # Cowl-Induction Hood Scoop & Louvers (+X forward)
    draw_td.rectangle([int(320 * ss), cy - int(25 * ss), int(420 * ss), cy + int(25 * ss)], fill=(28, 30, 36), outline=(225, 230, 240), width=int(1.5 * ss))
    # Greenhouse Glass
    draw_td.polygon([(int(325 * ss), cy - int(48 * ss)), (int(325 * ss), cy + int(48 * ss)), (int(280 * ss), cy + int(50 * ss)), (int(280 * ss), cy - int(50 * ss))], fill=(90, 160, 220, 230))
    draw_td.polygon([(int(190 * ss), cy - int(48 * ss)), (int(190 * ss), cy + int(48 * ss)), (int(120 * ss), cy + int(42 * ss)), (int(120 * ss), cy - int(42 * ss))], fill=(65, 120, 180, 230))

    save_dual_views("classic_stock_vintage", im_lat, im_td)


# ==============================================================================
# 6. FIREBOLT RS 2000 (classic_rx_vintage) - 1970s RWD Twin-Cam Rally Legend
# ==============================================================================
def generate_classic_rx_vintage():
    ss = 2
    W, H = 1024 * ss, 512 * ss
    im_lat = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im_lat)
    ground_y = int(450 * ss)
    r_wheel = int(60 * ss)
    wf_x = int(740 * ss)
    wr_x = int(280 * ss)
    cy_wheel = ground_y - r_wheel - int(6 * ss)

    # Shadow
    draw_soft_shadow(draw, int(512 * ss), ground_y, int(380 * ss), int(20 * ss), 160)

    # Red Mudflaps behind wheels
    draw.polygon([(wr_x - int(72 * ss), ground_y - int(10 * ss)), (wr_x - int(56 * ss), ground_y - int(10 * ss)), (wr_x - int(58 * ss), ground_y - int(70 * ss)), (wr_x - int(70 * ss), ground_y - int(70 * ss))], fill=(220, 35, 30))
    draw.polygon([(wf_x - int(72 * ss), ground_y - int(10 * ss)), (wf_x - int(56 * ss), ground_y - int(10 * ss)), (wf_x - int(58 * ss), ground_y - int(70 * ss)), (wf_x - int(70 * ss), ground_y - int(70 * ss))], fill=(220, 35, 30))

    # 1970s 2-Door Rally Coupe Silhouette
    body_poly = [
        (int(165 * ss), ground_y - int(42 * ss)),
        (int(150 * ss), ground_y - int(95 * ss)),   # Rear bumper
        (int(160 * ss), ground_y - int(140 * ss)),  # Trunk crease
        (int(240 * ss), ground_y - int(148 * ss)),  # C-pillar base
        (int(360 * ss), ground_y - int(210 * ss)),  # Roof
        (int(580 * ss), ground_y - int(210 * ss)),  # Windshield top
        (int(690 * ss), ground_y - int(135 * ss)),  # Cowl
        (int(840 * ss), ground_y - int(115 * ss)),  # Front fender
        (int(875 * ss), ground_y - int(70 * ss)),   # Nosecone
        (int(865 * ss), ground_y - int(38 * ss)),
        (int(790 * ss), ground_y - int(38 * ss)),
        (int(680 * ss), ground_y - int(38 * ss)),
        (int(340 * ss), ground_y - int(38 * ss)),
        (int(220 * ss), ground_y - int(38 * ss)),
    ]
    draw.polygon(body_poly, fill=(245, 245, 250), outline=(180, 185, 195), width=int(2 * ss))

    # Blue Rally Side Stripe
    draw.polygon([(int(160 * ss), ground_y - int(95 * ss)), (int(875 * ss), ground_y - int(70 * ss)), (int(870 * ss), ground_y - int(55 * ss)), (int(162 * ss), ground_y - int(80 * ss))], fill=(35, 95, 215))

    # Quad Front Rally Spotlights on Nose
    for ly in [ground_y - int(85 * ss), ground_y - int(65 * ss)]:
        draw_circle(draw, int(890 * ss), ly, int(14 * ss), fill=(255, 255, 220), outline=(235, 240, 250), width=int(2.5 * ss))

    # Greenhouse Windows
    window_poly = [
        (int(255 * ss), ground_y - int(152 * ss)),
        (int(370 * ss), ground_y - int(202 * ss)),
        (int(575 * ss), ground_y - int(202 * ss)),
        (int(675 * ss), ground_y - int(140 * ss)),
        (int(460 * ss), ground_y - int(142 * ss)),
    ]
    draw_glass_panel(draw, window_poly, base_col=(70, 140, 210, 220), ss=ss)

    # Wheels
    draw_realistic_wheel_lateral(draw, wr_x, cy_wheel, r_wheel, style="retro_rally", ss=ss)
    draw_realistic_wheel_lateral(draw, wf_x, cy_wheel, r_wheel, style="retro_rally", ss=ss)

    # Top-Down Sprite (512x512)
    W_TD, H_TD = 512 * ss, 512 * ss
    im_td = Image.new("RGBA", (W_TD, H_TD), (0, 0, 0, 0))
    draw_td = ImageDraw.Draw(im_td)
    cx, cy = 256 * ss, 256 * ss

    # Static wheels for showroom (+X forward)
    # Front FL & FR (X: 335..415)
    draw_td.rounded_rectangle([int(340 * ss), cy - int(112 * ss), int(410 * ss), cy - int(65 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    draw_td.rounded_rectangle([int(340 * ss), cy + int(65 * ss), int(410 * ss), cy + int(112 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    # Rear RL & RR (X: 95..175)
    draw_td.rounded_rectangle([int(95 * ss), cy - int(112 * ss), int(175 * ss), cy - int(65 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    draw_td.rounded_rectangle([int(95 * ss), cy + int(65 * ss), int(175 * ss), cy + int(112 * ss)], radius=int(6 * ss), fill=(22, 24, 28))

    # Body Contours (Facing +X)
    top_poly = [
        (int(465 * ss), cy - int(48 * ss)),
        (int(475 * ss), cy),
        (int(465 * ss), cy + int(48 * ss)),
        (int(410 * ss), cy + int(92 * ss)),  # Bubble arch R
        (int(335 * ss), cy + int(92 * ss)),
        (int(280 * ss), cy + int(82 * ss)),  # Door
        (int(190 * ss), cy + int(94 * ss)),  # Bubble arch rear R
        (int(85 * ss), cy + int(90 * ss)),
        (int(55 * ss), cy + int(55 * ss)),   # Rear trunk
        (int(50 * ss), cy),
        (int(55 * ss), cy - int(55 * ss)),
        (int(85 * ss), cy - int(90 * ss)),
        (int(190 * ss), cy - int(94 * ss)),
        (int(280 * ss), cy - int(82 * ss)),
        (int(335 * ss), cy - int(92 * ss)),
        (int(410 * ss), cy - int(92 * ss)),
    ]
    draw_td.polygon(top_poly, fill=(245, 245, 250), outline=(180, 185, 195), width=int(2 * ss))

    # Twin Blue Rally Stripes
    draw_td.line([int(55 * ss), cy - int(8 * ss), int(470 * ss), cy - int(8 * ss)], fill=(35, 95, 215), width=int(6 * ss))
    draw_td.line([int(55 * ss), cy + int(8 * ss), int(470 * ss), cy + int(8 * ss)], fill=(35, 95, 215), width=int(6 * ss))

    # Quad Spotlights Pod (+X: 475..495)
    for ly in [-32, -11, 11, 32]:
        draw_circle(draw_td, int(485 * ss), cy + int(ly * ss), int(8 * ss), fill=(255, 255, 220), outline=(235, 240, 250), width=int(1.5 * ss))

    # Windshield & Rear Glass
    draw_td.polygon([(int(335 * ss), cy - int(40 * ss)), (int(335 * ss), cy + int(40 * ss)), (int(285 * ss), cy + int(44 * ss)), (int(285 * ss), cy - int(44 * ss))], fill=(90, 160, 220, 230))
    draw_td.polygon([(int(205 * ss), cy - int(42 * ss)), (int(205 * ss), cy + int(42 * ss)), (int(150 * ss), cy + int(38 * ss)), (int(150 * ss), cy - int(38 * ss))], fill=(65, 120, 180, 230))

    save_dual_views("classic_rx_vintage", im_lat, im_td)


# ==============================================================================
# 7. IRONCLAD 4x4 SAFARI (classic_at_safari) - 1980s Paris-Dakar 4WD Trail Rig
# ==============================================================================
def generate_classic_at_safari():
    ss = 2
    W, H = 1024 * ss, 512 * ss
    im_lat = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im_lat)
    ground_y = int(450 * ss)
    r_wheel = int(68 * ss)
    wf_x = int(740 * ss)
    wr_x = int(270 * ss)
    cy_wheel = ground_y - r_wheel - int(12 * ss)  # Lifted heavy-duty 4x4 suspension

    # Shadow
    draw_soft_shadow(draw, int(512 * ss), ground_y, int(410 * ss), int(22 * ss), 180)

    # Heavy Upright 1980s Dakar 4x4 Wagon Silhouette
    body_poly = [
        (int(145 * ss), ground_y - int(55 * ss)),
        (int(135 * ss), ground_y - int(120 * ss)),  # Rear door / spare tire mount
        (int(145 * ss), ground_y - int(240 * ss)),  # High roof rear
        (int(630 * ss), ground_y - int(240 * ss)),  # High roof front
        (int(710 * ss), ground_y - int(155 * ss)),  # Upright windshield cowl
        (int(865 * ss), ground_y - int(140 * ss)),  # Flat square hood
        (int(885 * ss), ground_y - int(85 * ss)),   # Grille & winch bumper
        (int(875 * ss), ground_y - int(50 * ss)),   # Steel skid plate
        (int(810 * ss), ground_y - int(50 * ss)),
        (int(680 * ss), ground_y - int(50 * ss)),
        (int(340 * ss), ground_y - int(50 * ss)),
        (int(210 * ss), ground_y - int(50 * ss)),
    ]
    draw.polygon(body_poly, fill=(215, 175, 115), outline=(140, 110, 70), width=int(2 * ss))

    # Expedition Roof Rack with Spare Tire & Sand Ladders
    draw.rectangle([int(155 * ss), ground_y - int(265 * ss), int(620 * ss), ground_y - int(242 * ss)], outline=(40, 44, 52), fill=(60, 65, 75), width=int(3 * ss))
    # Spare tire on roof
    draw_circle(draw, int(260 * ss), ground_y - int(275 * ss), int(22 * ss), fill=(24, 25, 28), outline=(12, 13, 15), width=int(2 * ss))

    # Snorkel Intake rising up A-Pillar
    draw.line([int(710 * ss), ground_y - int(150 * ss), int(640 * ss), ground_y - int(250 * ss)], fill=(28, 30, 36), width=int(8 * ss))
    draw_circle(draw, int(635 * ss), ground_y - int(252 * ss), int(10 * ss), fill=(28, 30, 36))

    # Windows (Upright Safari Side Windows)
    window_poly = [
        (int(165 * ss), ground_y - int(230 * ss)),
        (int(625 * ss), ground_y - int(230 * ss)),
        (int(695 * ss), ground_y - int(162 * ss)),
        (int(165 * ss), ground_y - int(162 * ss)),
    ]
    draw_glass_panel(draw, window_poly, base_col=(70, 140, 210, 220), ss=ss)

    # Wheels
    draw_realistic_wheel_lateral(draw, wr_x, cy_wheel, r_wheel, style="knobby", ss=ss)
    draw_realistic_wheel_lateral(draw, wf_x, cy_wheel, r_wheel, style="knobby", ss=ss)

    # Top-Down Sprite (512x512)
    W_TD, H_TD = 512 * ss, 512 * ss
    im_td = Image.new("RGBA", (W_TD, H_TD), (0, 0, 0, 0))
    draw_td = ImageDraw.Draw(im_td)
    cx, cy = 256 * ss, 256 * ss

    # Static wheels for showroom (+X forward)
    # Front FL & FR (X: 335..425)
    draw_td.rounded_rectangle([int(340 * ss), cy - int(122 * ss), int(420 * ss), cy - int(68 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    draw_td.rounded_rectangle([int(340 * ss), cy + int(68 * ss), int(420 * ss), cy + int(122 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    # Rear RL & RR (X: 90..180)
    draw_td.rounded_rectangle([int(95 * ss), cy - int(122 * ss), int(175 * ss), cy - int(68 * ss)], radius=int(6 * ss), fill=(22, 24, 28))
    draw_td.rounded_rectangle([int(95 * ss), cy + int(68 * ss), int(175 * ss), cy + int(122 * ss)], radius=int(6 * ss), fill=(22, 24, 28))

    # Boxy 4x4 Wagon Contour (Facing +X)
    top_poly = [
        (int(465 * ss), cy - int(55 * ss)),
        (int(475 * ss), cy),
        (int(465 * ss), cy + int(55 * ss)),
        (int(425 * ss), cy + int(98 * ss)),  # Front fender R
        (int(335 * ss), cy + int(98 * ss)),
        (int(290 * ss), cy + int(92 * ss)),  # Cabin body
        (int(185 * ss), cy + int(98 * ss)),  # Rear fender R
        (int(85 * ss), cy + int(95 * ss)),
        (int(50 * ss), cy + int(60 * ss)),   # Rear tailgate
        (int(45 * ss), cy),
        (int(50 * ss), cy - int(60 * ss)),
        (int(85 * ss), cy - int(95 * ss)),
        (int(185 * ss), cy - int(98 * ss)),
        (int(290 * ss), cy - int(92 * ss)),
        (int(335 * ss), cy - int(98 * ss)),
        (int(425 * ss), cy - int(98 * ss)),
    ]
    draw_td.polygon(top_poly, fill=(215, 175, 115), outline=(140, 110, 70), width=int(2 * ss))

    # Flat Square Hood & Winch Bumper (+X forward)
    draw_td.rectangle([int(465 * ss), cy - int(45 * ss), int(485 * ss), cy + int(45 * ss)], fill=(35, 38, 44), outline=(180, 185, 195), width=int(2 * ss))

    # Roof Rack with Gear & Spare Wheel
    draw_td.rectangle([int(90 * ss), cy - int(50 * ss), int(310 * ss), cy + int(50 * ss)], outline=(40, 44, 52), fill=(55, 60, 70), width=int(3 * ss))
    # Spare wheel on roof rack
    draw_circle(draw_td, int(150 * ss), cy, int(22 * ss), fill=(24, 25, 28), outline=(180, 185, 195), width=int(2 * ss))

    # Windshield (Facing +X)
    draw_td.polygon([(int(345 * ss), cy - int(45 * ss)), (int(345 * ss), cy + int(45 * ss)), (int(315 * ss), cy + int(48 * ss)), (int(315 * ss), cy - int(48 * ss))], fill=(90, 160, 220, 230))

    save_dual_views("classic_at_safari", im_lat, im_td)


# ==============================================================================
# CHASSIS CUTOUT GENERATOR (All 12 Classic Vehicles)
# ==============================================================================
def generate_all_12_chassis_sprites():
    chassis_configs = [
        # Karting
        ("classic_kart", 320, 420, 135, 205, 307, 377, True),
        ("classic_kart_vintage", 310, 400, 140, 210, 302, 372, True),

        # GT
        ("classic_gt", 340, 420, 145, 205, 307, 367, False),
        ("classic_gt_vintage", 335, 415, 140, 205, 307, 372, False),

        # Stock Cars
        ("classic_nascar", 315, 390, 155, 215, 297, 357, False),
        ("classic_stock_vintage", 340, 425, 135, 205, 307, 377, False),

        # Rallycross
        ("classic_rally", 340, 415, 145, 215, 295, 365, False),
        ("classic_rx_vintage", 335, 415, 140, 210, 302, 372, False),

        # Autocross
        ("classic_ax_mudlark", 340, 425, 130, 210, 302, 382, True),
        ("classic_ax_brawler", 340, 420, 135, 205, 307, 377, False),

        # All-Terrain
        ("classic_offroad", 370, 485, 100, 161, 344, 405, True),
        ("classic_at_safari", 335, 425, 130, 200, 312, 382, False),
    ]

    for name, x_min, x_max, y_fl_min, y_fl_max, y_fr_min, y_fr_max, is_open in chassis_configs:
        path = TOPDOWN_DIR / f"{name}.png"
        if not path.exists():
            print(f"⚠️ Warning: {path} does not exist, skipping chassis generation")
            continue
        im = Image.open(path)
        arr = np.array(im)

        if is_open:
            fl_mask = (np.arange(512)[None, :] >= x_min) & (np.arange(512)[None, :] <= x_max) & (np.arange(512)[:, None] >= y_fl_min) & (np.arange(512)[:, None] <= y_fl_max)
            fr_mask = (np.arange(512)[None, :] >= x_min) & (np.arange(512)[None, :] <= x_max) & (np.arange(512)[:, None] >= y_fr_min) & (np.arange(512)[:, None] <= y_fr_max)
            clear_mask = fl_mask | fr_mask
            arr[clear_mask] = [0, 0, 0, 0]
        else:
            # For closed wheel cars, clear the dark tire rubber
            is_dark = (arr[:, :, 0] < 60) & (arr[:, :, 1] < 60) & (arr[:, :, 2] < 60) & (arr[:, :, 3] > 0)
            fl_mask = is_dark & (np.arange(512)[None, :] >= x_min) & (np.arange(512)[None, :] <= x_max) & (np.arange(512)[:, None] >= y_fl_min) & (np.arange(512)[:, None] <= y_fl_max)
            fr_mask = is_dark & (np.arange(512)[None, :] >= x_min) & (np.arange(512)[None, :] <= x_max) & (np.arange(512)[:, None] >= y_fr_min) & (np.arange(512)[:, None] <= y_fr_max)
            clear_mask = fl_mask | fr_mask
            arr[clear_mask] = [0, 0, 0, 0]

        out_im = Image.fromarray(arr)
        out_im.save(TOPDOWN_DIR / f"{name}_chassis.png")
        print(f"✓ Generated {name}_chassis.png (cleared {np.sum(clear_mask)} wheel pixels)")


if __name__ == "__main__":
    generate_classic_ax_mudlark()
    generate_classic_ax_brawler()
    generate_classic_kart_vintage()
    generate_classic_gt_vintage()
    generate_classic_stock_vintage()
    generate_classic_rx_vintage()
    generate_classic_at_safari()
    generate_all_12_chassis_sprites()
    print("✨ Successfully generated all new vintage and realistic classic vehicle sprites!")
