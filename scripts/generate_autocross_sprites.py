#!/usr/bin/env python3
"""
Generate high-fidelity, fictional 2D lateral and top-down textures for all 15
Continental Autocross Championship vehicles across 5 tiers:

Tier 1: Cross Car Junior
  - autocross_ardennes_junior_t1 (Ardennes Junior Cross 600 T1)
  - autocross_iberian_furia_t1    (Iberian Furia Junior Cross T1)
  - autocross_cosmo_nova_t1       (Cosmo Nova Junior Cross T1)

Tier 2: Cross Car Senior
  - autocross_ardennes_pro_t2     (Ardennes Pro Cross 850 T2)
  - autocross_iberian_relampago_t2 (Iberian Relampago Cross T2)
  - autocross_lusitania_bravo_t2  (Lusitania Bravo Sport Cross T2)

Tier 3: Buggy 1600
  - autocross_petersen_buggy1600_t3 (Petersen Buggy 1600 T3)
  - autocross_bologna_buggy1600_t3  (Bologna Buggy 1600 T3)
  - autocross_rapid_buggy1600_t3    (Rapid Dynamics Buggy 1600 T3)

Tier 4: TouringAutocross
  - autocross_bohemia_veloce_t4     (Bohemia Veloce Touring AX T4)
  - autocross_shinano_tsunami_t4    (Shinano Tsunami Touring AX T4)
  - autocross_vortek_quattro_t4     (Vortek Quattro Touring AX T4)

Tier 5: SuperBuggy
  - autocross_petersen_superbuggy_t5 (Petersen SuperBuggy V8 T5)
  - autocross_bologna_superbuggy_t5  (Bologna SuperBuggy Twin-Turbo T5)
  - autocross_rapid_superbuggy_t5    (Rapid Dynamics SuperBuggy Biturbo T5)

Design & IP Removal Rules:
- Both lateral (1024x512) and top-down (512x512) views face RIGHT (+X).
- Real livery inspiration (color blocks, geometric stripes, racing contrasts).
- STRICT ZERO IP: No manufacturer badges, no sponsor logos, no proprietary decals.
- Lateral exports 1024x512 PNG + 256x128 thumbnail.
- Top-down exports 512x512 PNG.
"""

import argparse
import math
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
LATERAL_DIR = ROOT / "assets" / "textures" / "vehicles" / "laterals" / "autocross"
TOPDOWN_DIR = ROOT / "assets" / "textures" / "vehicles" / "topdown" / "autocross"

LATERAL_DIR.mkdir(parents=True, exist_ok=True)
TOPDOWN_DIR.mkdir(parents=True, exist_ok=True)


# ==============================================================================
# Procedural Drawing Helpers
# ==============================================================================

def draw_circle(draw, cx, cy, r, fill, outline=None, width=1):
    draw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=fill, outline=outline, width=width)


def draw_soft_shadow(draw, cx, cy, rx, ry, alpha=140):
    draw.ellipse([cx - rx, cy - ry, cx + rx, cy + ry], fill=(0, 0, 0, alpha))


def draw_knobby_wheel_lateral(draw, cx, cy, r, rim_col=(220, 220, 225), beadlock_col=(230, 160, 20), ss=2):
    """Draws an authentic off-road knobby wheel with outer tread lugs, beadlock ring, and brake rotor."""
    # 1. Outer knobby tread blocks (16 perimeter lugs)
    num_lugs = 16
    for i in range(num_lugs):
        ang = i * (2 * math.pi / num_lugs)
        lx = cx + math.cos(ang) * (r * 0.98)
        ly = cy + math.sin(ang) * (r * 0.98)
        draw_circle(draw, lx, ly, int(r * 0.12), fill=(18, 20, 22, 255))

    # 2. Main tire sidewall rubber
    draw_circle(draw, cx, cy, int(r * 0.92), fill=(26, 28, 32, 255), outline=(12, 13, 15, 255), width=int(2 * ss))
    draw_circle(draw, cx, cy, int(r * 0.82), fill=(34, 36, 40, 255))

    # 3. Anodized Beadlock Outer Ring
    r_bead = int(r * 0.70)
    draw_circle(draw, cx, cy, r_bead, fill=(40, 42, 48, 255), outline=beadlock_col, width=int(3 * ss))
    # Beadlock perimeter bolts
    for i in range(12):
        ang = i * (math.pi / 6)
        bx = cx + math.cos(ang) * (r_bead * 0.90)
        by = cy + math.sin(ang) * (r_bead * 0.90)
        draw_circle(draw, bx, by, int(1.5 * ss), fill=(210, 215, 225, 255))

    # 4. Brake Rotor & Red Racing Caliper
    r_rotor = int(r * 0.52)
    draw_circle(draw, cx, cy, r_rotor, fill=(75, 78, 86, 255), outline=(110, 115, 125, 255), width=int(1.5 * ss))
    # Ventilation drill holes
    for rot_ang in range(8):
        ang = rot_ang * (math.pi / 4)
        rx = cx + math.cos(ang) * (r_rotor * 0.65)
        ry = cy + math.sin(ang) * (r_rotor * 0.65)
        draw_circle(draw, rx, ry, int(1.5 * ss), fill=(30, 32, 36, 255))
    # Caliper (upper right)
    cal_ang = 0.75
    cal_x = cx + math.cos(cal_ang) * (r * 0.44)
    cal_y = cy - math.sin(cal_ang) * (r * 0.44)
    draw_circle(draw, cal_x, cal_y, int(r * 0.18), fill=(225, 35, 30, 255), outline=(140, 20, 15, 255), width=int(1.5 * ss))

    # 5. Wheel Center Hub & Spokes
    for i in range(5):
        ang = i * (2 * math.pi / 5)
        x2 = cx + math.cos(ang) * (r_bead * 0.78)
        y2 = cy + math.sin(ang) * (r_bead * 0.78)
        draw.line([cx, cy, x2, y2], fill=rim_col, width=int(4 * ss))
    draw_circle(draw, cx, cy, int(r * 0.16), fill=(20, 22, 26, 255), outline=rim_col, width=int(2 * ss))
    draw_circle(draw, cx, cy, int(r * 0.07), fill=(200, 205, 215, 255))


def draw_tubular_cage_lateral(draw, nodes, col=(40, 180, 240), ss=2):
    """Draws tubular spaceframe tubes with an upper cylindrical lighting highlight."""
    for p1, p2 in nodes:
        # Base structural pipe
        draw.line([p1[0], p1[1], p2[0], p2[1]], fill=col, width=int(7 * ss))
        # Cylindrical top highlight
        hl_col = tuple(min(255, int(c * 1.35) + 40) for c in col[:3])
        draw.line([p1[0], p1[1] - int(1.5 * ss), p2[0], p2[1] - int(1.5 * ss)], fill=hl_col, width=int(2 * ss))


def draw_driver_lateral(draw, cx, cy, helmet_col=(245, 245, 250), visor_col=(30, 35, 45), ss=2):
    """Draws seated racing driver silhouette facing right (+X)."""
    # Shoulder & torso
    draw.rectangle([cx - int(16 * ss), cy + int(4 * ss), cx + int(14 * ss), cy + int(40 * ss)], fill=(35, 38, 45))
    # Helmet dome
    r_helm = int(18 * ss)
    draw_circle(draw, cx, cy, r_helm, fill=helmet_col, outline=(30, 32, 38), width=int(2 * ss))
    # Tinted visor facing right (+X)
    draw.polygon([
        (cx + int(4 * ss), cy - int(6 * ss)),
        (cx + int(18 * ss), cy - int(4 * ss)),
        (cx + int(16 * ss), cy + int(6 * ss)),
        (cx + int(4 * ss), cy + int(4 * ss)),
    ], fill=visor_col)


def draw_engine_exhaust_lateral(draw, x_engine, y_engine, tail_x, tail_y, ss=2):
    """Draws exposed dirt-track engine block and titanium exhaust with heat-tempered blue/gold gradient."""
    # Engine block & cylinder head
    draw.rectangle([x_engine - int(35 * ss), y_engine - int(25 * ss), x_engine + int(35 * ss), y_engine + int(25 * ss)], fill=(70, 75, 85), outline=(35, 38, 45), width=int(2 * ss))
    # Cooling fins
    for i in range(4):
        fy = y_engine - int(18 * ss) + int(i * 10 * ss)
        draw.line([x_engine - int(30 * ss), fy, x_engine + int(30 * ss), fy], fill=(130, 135, 145), width=int(2 * ss))
    # Titanium exhaust pipe curving out to tail
    draw.line([x_engine, y_engine, tail_x + int(40 * ss), tail_y], fill=(185, 145, 60), width=int(6 * ss))  # Gold heat
    draw.line([tail_x + int(40 * ss), tail_y, tail_x, tail_y], fill=(60, 110, 215), width=int(6 * ss))      # Blued titanium tip


def draw_wheel_topdown(draw_td, cx, cy, w, h, tread_col=(24, 26, 30), rim_col=(210, 215, 225), ss=2):
    """Draws top-down knobby tire (length w along X, width h along Y)."""
    # Outer tire
    draw_td.rounded_rectangle([cx - w // 2, cy - h // 2, cx + w // 2, cy + h // 2], radius=int(6 * ss), fill=tread_col, outline=(12, 13, 16), width=int(1.5 * ss))
    # Tread grooves
    num_grooves = 5
    for i in range(num_grooves):
        gx = cx - w // 2 + int((i + 1) * (w / (num_grooves + 1)))
        draw_td.line([gx, cy - h // 2 + int(3 * ss), gx, cy + h // 2 - int(3 * ss)], fill=(12, 14, 18), width=int(2 * ss))
    # Inner rim center
    draw_td.rounded_rectangle([cx - int(w * 0.35) // 2, cy - int(h * 0.40) // 2, cx + int(w * 0.35) // 2, cy + int(h * 0.40) // 2], radius=int(3 * ss), fill=rim_col)


def save_dual_views(name, im_lat, im_td):
    """Resamples with Lanczos filter and saves canonical 1024x512 lateral, 256x128 thumb, and 512x512 topdown."""
    # Lateral 1024x512
    lat_1024 = im_lat.resize((1024, 512), Image.Resampling.LANCZOS)
    lat_1024.save(LATERAL_DIR / f"{name}.png")
    # Thumbnail 256x128
    lat_thumb = lat_1024.resize((256, 128), Image.Resampling.LANCZOS)
    lat_thumb.save(LATERAL_DIR / f"{name}_thumb.png")
    # Top-Down 512x512
    td_512 = im_td.resize((512, 512), Image.Resampling.LANCZOS)
    td_512.save(TOPDOWN_DIR / f"{name}.png")
    print(f"  ✓ Saved canonical assets for {name}")


# ==============================================================================
# Fleet Generation Functions (15 Vehicles across 5 Tiers)
# ==============================================================================

def generate_crosscar(model_id, primary_col, accent_col, cage_col, beadlock_col, style="junior"):
    """
    Modular generator for Cross Car Junior (Tier 1) and Cross Car Senior (Tier 2).
    Junior: single-plane wing, 600cc compact chassis.
    Senior: bi-plane wing, roof air snorkel scoop, 850cc wide track.
    """
    ss = 2
    # -------------------------------------------------------------------------
    # Lateral View (1024x512 canvas -> ss=2 is 2048x1024)
    # -------------------------------------------------------------------------
    W, H = 1024 * ss, 512 * ss
    im_lat = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im_lat)
    ground_y = int(450 * ss)
    r_front = int(60 * ss)
    r_rear = int(66 * ss)
    wf_x = int(740 * ss)
    wr_x = int(280 * ss)
    cy_wheel = ground_y - r_rear - int(6 * ss)

    # 1. Ground contact shadow
    draw_soft_shadow(draw, int(512 * ss), ground_y, int(370 * ss), int(18 * ss), 150)

    # 2. Lower Body Paneling (Fibreglass Pods)
    # Nosecone (+X on right): 640..850
    nose_poly = [
        (int(580 * ss), ground_y - int(70 * ss)),
        (int(730 * ss), ground_y - int(75 * ss)),
        (int(850 * ss), ground_y - int(45 * ss)),  # Nose tip
        (int(830 * ss), ground_y - int(30 * ss)),
        (int(630 * ss), ground_y - int(38 * ss)),
    ]
    draw.polygon(nose_poly, fill=primary_col, outline=(25, 28, 35), width=int(2 * ss))
    # Livery Accent Stripe on Nose
    draw.polygon([
        (int(660 * ss), ground_y - int(72 * ss)),
        (int(780 * ss), ground_y - int(68 * ss)),
        (int(840 * ss), ground_y - int(45 * ss)),
        (int(740 * ss), ground_y - int(40 * ss)),
    ], fill=accent_col)

    # Side Pods: 340..600
    sidepod_poly = [
        (int(360 * ss), ground_y - int(105 * ss)),
        (int(580 * ss), ground_y - int(85 * ss)),
        (int(610 * ss), ground_y - int(40 * ss)),
        (int(350 * ss), ground_y - int(40 * ss)),
    ]
    draw.polygon(sidepod_poly, fill=primary_col, outline=(25, 28, 35), width=int(2 * ss))
    # Side Pod Air Duct
    draw.polygon([
        (int(490 * ss), ground_y - int(80 * ss)),
        (int(560 * ss), ground_y - int(75 * ss)),
        (int(570 * ss), ground_y - int(55 * ss)),
        (int(510 * ss), ground_y - int(58 * ss)),
    ], fill=(20, 22, 26))

    # 3. Cockpit & Driver (Centered around X=490)
    draw_driver_lateral(draw, int(480 * ss), ground_y - int(160 * ss), ss=ss)

    # 4. Tubular Roll Cage Structure
    cage_bars = [
        # Main hoop uprights
        ((int(370 * ss), ground_y - int(90 * ss)), (int(430 * ss), ground_y - int(230 * ss))),
        # Roof line
        ((int(430 * ss), ground_y - int(230 * ss)), (int(550 * ss), ground_y - int(230 * ss))),
        # Front A-pillars down to nose
        ((int(550 * ss), ground_y - int(230 * ss)), (int(640 * ss), ground_y - int(85 * ss))),
        # Diagonal cross-brace
        ((int(370 * ss), ground_y - int(90 * ss)), (int(550 * ss), ground_y - int(230 * ss))),
        # Lower side rails
        ((int(250 * ss), ground_y - int(50 * ss)), (int(710 * ss), ground_y - int(50 * ss))),
        ((int(260 * ss), ground_y - int(85 * ss)), (int(400 * ss), ground_y - int(90 * ss))),
    ]
    draw_tubular_cage_lateral(draw, cage_bars, col=cage_col, ss=ss)

    # 5. Roof Scoop / Snorkel (Senior spec gets aggressive air intake)
    if style == "senior":
        draw.polygon([
            (int(440 * ss), ground_y - int(232 * ss)),
            (int(490 * ss), ground_y - int(270 * ss)),
            (int(540 * ss), ground_y - int(270 * ss)),
            (int(545 * ss), ground_y - int(232 * ss)),
        ], fill=primary_col, outline=cage_col, width=int(2 * ss))
        draw.polygon([
            (int(520 * ss), ground_y - int(268 * ss)),
            (int(538 * ss), ground_y - int(268 * ss)),
            (int(542 * ss), ground_y - int(240 * ss)),
            (int(525 * ss), ground_y - int(240 * ss)),
        ], fill=(20, 22, 26))  # Intake mouth facing forward (+X)

    # 6. Rear Engine & Exhaust
    draw_engine_exhaust_lateral(draw, int(290 * ss), ground_y - int(105 * ss), int(200 * ss), ground_y - int(120 * ss), ss=ss)

    # 7. Aerodynamic Rear Wing
    wing_mount_x = int(280 * ss)
    draw.line([wing_mount_x, ground_y - int(120 * ss), wing_mount_x - int(30 * ss), ground_y - int(260 * ss)], fill=(40, 42, 50), width=int(5 * ss))
    # Main wing plane
    draw.polygon([
        (wing_mount_x - int(100 * ss), ground_y - int(275 * ss)),
        (wing_mount_x + int(20 * ss), ground_y - int(260 * ss)),
        (wing_mount_x + int(15 * ss), ground_y - int(245 * ss)),
        (wing_mount_x - int(105 * ss), ground_y - int(260 * ss)),
    ], fill=primary_col, outline=(30, 32, 38), width=int(2 * ss))
    # Endplate
    draw.polygon([
        (wing_mount_x - int(110 * ss), ground_y - int(290 * ss)),
        (wing_mount_x - int(20 * ss), ground_y - int(290 * ss)),
        (wing_mount_x - int(10 * ss), ground_y - int(240 * ss)),
        (wing_mount_x - int(100 * ss), ground_y - int(240 * ss)),
    ], fill=accent_col, outline=(20, 22, 26), width=int(2 * ss))
    if style == "senior":
        # Secondary lower wing plane
        draw.polygon([
            (wing_mount_x - int(80 * ss), ground_y - int(230 * ss)),
            (wing_mount_x + int(10 * ss), ground_y - int(218 * ss)),
            (wing_mount_x + int(8 * ss), ground_y - int(208 * ss)),
            (wing_mount_x - int(82 * ss), ground_y - int(220 * ss)),
        ], fill=primary_col)

    # 8. Wheels (Facing right: Rear at wr_x, Front at wf_x)
    draw_knobby_wheel_lateral(draw, wr_x, cy_wheel, r_rear, rim_col=(225, 225, 230), beadlock_col=beadlock_col, ss=ss)
    draw_knobby_wheel_lateral(draw, wf_x, cy_wheel, r_front, rim_col=(225, 225, 230), beadlock_col=beadlock_col, ss=ss)

    # -------------------------------------------------------------------------
    # Top-Down View (512x512 canvas -> ss=2 is 1024x1024)
    # Nose pointing RIGHT (+X)
    # -------------------------------------------------------------------------
    W_TD, H_TD = 512 * ss, 512 * ss
    im_td = Image.new("RGBA", (W_TD, H_TD), (0, 0, 0, 0))
    draw_td = ImageDraw.Draw(im_td)
    cy = int(256 * ss)

    # Suspension Wishbones & Steering Links (Front X: 360..400, Rear X: 110..150)
    # Front FL (upper) & FR (lower)
    draw_td.line([int(330 * ss), cy - int(45 * ss), int(380 * ss), cy - int(105 * ss)], fill=(160, 165, 175), width=int(5 * ss))
    draw_td.line([int(370 * ss), cy - int(45 * ss), int(380 * ss), cy - int(105 * ss)], fill=(160, 165, 175), width=int(5 * ss))
    draw_td.line([int(330 * ss), cy + int(45 * ss), int(380 * ss), cy + int(105 * ss)], fill=(160, 165, 175), width=int(5 * ss))
    draw_td.line([int(370 * ss), cy + int(45 * ss), int(380 * ss), cy + int(105 * ss)], fill=(160, 165, 175), width=int(5 * ss))

    # Rear RL & RR
    draw_td.line([int(150 * ss), cy - int(50 * ss), int(130 * ss), cy - int(110 * ss)], fill=(160, 165, 175), width=int(5 * ss))
    draw_td.line([int(190 * ss), cy - int(50 * ss), int(130 * ss), cy - int(110 * ss)], fill=(160, 165, 175), width=int(5 * ss))
    draw_td.line([int(150 * ss), cy + int(50 * ss), int(130 * ss), cy + int(110 * ss)], fill=(160, 165, 175), width=int(5 * ss))
    draw_td.line([int(190 * ss), cy + int(50 * ss), int(130 * ss), cy + int(110 * ss)], fill=(160, 165, 175), width=int(5 * ss))

    # Four Knobby Wheels (Front: X=380, Y=cy±105, Rear: X=130, Y=cy±110)
    wheel_w_front, wheel_h_front = int(76 * ss), int(38 * ss)
    wheel_w_rear, wheel_h_rear = int(82 * ss), int(44 * ss)
    draw_wheel_topdown(draw_td, int(380 * ss), cy - int(105 * ss), wheel_w_front, wheel_h_front, rim_col=beadlock_col, ss=ss)
    draw_wheel_topdown(draw_td, int(380 * ss), cy + int(105 * ss), wheel_w_front, wheel_h_front, rim_col=beadlock_col, ss=ss)
    draw_wheel_topdown(draw_td, int(130 * ss), cy - int(110 * ss), wheel_w_rear, wheel_h_rear, rim_col=beadlock_col, ss=ss)
    draw_wheel_topdown(draw_td, int(130 * ss), cy + int(110 * ss), wheel_w_rear, wheel_h_rear, rim_col=beadlock_col, ss=ss)

    # Main Body & Sidepods (X: 180..340, Y: cy±70)
    draw_td.polygon([
        (int(190 * ss), cy - int(70 * ss)),
        (int(320 * ss), cy - int(65 * ss)),
        (int(340 * ss), cy - int(45 * ss)),
        (int(340 * ss), cy + int(45 * ss)),
        (int(320 * ss), cy + int(65 * ss)),
        (int(190 * ss), cy + int(70 * ss)),
    ], fill=primary_col, outline=(25, 28, 35), width=int(2 * ss))

    # Shaded Fiberglass Nosecone (+X: 330..450)
    nose_td = [
        (int(335 * ss), cy - int(42 * ss)),
        (int(440 * ss), cy - int(18 * ss)),
        (int(455 * ss), cy),
        (int(440 * ss), cy + int(18 * ss)),
        (int(335 * ss), cy + int(42 * ss)),
    ]
    draw_td.polygon(nose_td, fill=primary_col, outline=(20, 24, 30), width=int(2 * ss))
    # Accent Livery Arrow pointing +X
    draw_td.polygon([
        (int(345 * ss), cy - int(25 * ss)),
        (int(435 * ss), cy),
        (int(345 * ss), cy + int(25 * ss)),
    ], fill=accent_col)

    # Cockpit Driver in Center (X=260)
    draw_td.rectangle([int(220 * ss), cy - int(26 * ss), int(300 * ss), cy + int(26 * ss)], fill=(18, 20, 24))
    draw_circle(draw_td, int(260 * ss), cy, int(20 * ss), fill=(245, 245, 250), outline=(20, 22, 28), width=int(2 * ss))
    # Visor pointing +X
    draw_td.line([int(270 * ss), cy - int(10 * ss), int(270 * ss), cy + int(10 * ss)], fill=(20, 25, 35), width=int(4 * ss))

    # Tubular Roll Cage Top Frame
    draw_td.rectangle([int(200 * ss), cy - int(38 * ss), int(320 * ss), cy + int(38 * ss)], outline=cage_col, fill=None, width=int(5 * ss))
    draw_td.line([int(200 * ss), cy - int(38 * ss), int(320 * ss), cy + int(38 * ss)], fill=cage_col, width=int(4 * ss))
    draw_td.line([int(200 * ss), cy + int(38 * ss), int(320 * ss), cy - int(38 * ss)], fill=cage_col, width=int(4 * ss))

    # Rear Wing (X: 75..115, Y: cy±75)
    draw_td.rectangle([int(80 * ss), cy - int(75 * ss), int(115 * ss), cy + int(75 * ss)], fill=primary_col, outline=(25, 28, 35), width=int(2 * ss))
    # Wing Endplates
    draw_td.rectangle([int(72 * ss), cy - int(80 * ss), int(120 * ss), cy - int(72 * ss)], fill=accent_col)
    draw_td.rectangle([int(72 * ss), cy + int(72 * ss), int(120 * ss), cy + int(80 * ss)], fill=accent_col)

    save_dual_views(model_id, im_lat, im_td)


def generate_buggy1600(model_id, primary_col, accent_col, cage_col, beadlock_col):
    """
    Tier 3: Buggy 1600 open-wheel single-seater buggies.
    Long wheelbase, exposed 1600cc engine, massive high-mount rear wing, long-travel coilovers.
    """
    ss = 2
    # Lateral (2048x1024)
    W, H = 1024 * ss, 512 * ss
    im_lat = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im_lat)
    ground_y = int(450 * ss)
    r_front = int(62 * ss)
    r_rear = int(72 * ss)
    wf_x = int(760 * ss)
    wr_x = int(260 * ss)
    cy_wheel = ground_y - r_rear - int(6 * ss)

    # Ground shadow
    draw_soft_shadow(draw, int(512 * ss), ground_y, int(400 * ss), int(20 * ss), 150)

    # Exposed suspension arms and long-travel coilovers
    draw.line([wf_x - int(80 * ss), cy_wheel - int(60 * ss), wf_x, cy_wheel], fill=(220, 40, 30), width=int(5 * ss))
    draw.line([wr_x + int(80 * ss), cy_wheel - int(70 * ss), wr_x, cy_wheel], fill=(220, 40, 30), width=int(6 * ss))

    # Aerodynamic Needle Nose Bodywork (Facing right: 570..860)
    nose_poly = [
        (int(540 * ss), ground_y - int(80 * ss)),
        (int(740 * ss), ground_y - int(75 * ss)),
        (int(865 * ss), ground_y - int(48 * ss)),
        (int(850 * ss), ground_y - int(32 * ss)),
        (int(630 * ss), ground_y - int(42 * ss)),
        (int(520 * ss), ground_y - int(42 * ss)),
    ]
    draw.polygon(nose_poly, fill=primary_col, outline=(25, 28, 35), width=int(2 * ss))
    # Geometric Livery Stripes
    draw.polygon([
        (int(610 * ss), ground_y - int(78 * ss)),
        (int(770 * ss), ground_y - int(72 * ss)),
        (int(860 * ss), ground_y - int(48 * ss)),
        (int(730 * ss), ground_y - int(45 * ss)),
    ], fill=accent_col)

    # Cockpit Driver (X=460)
    draw_driver_lateral(draw, int(460 * ss), ground_y - int(160 * ss), ss=ss)

    # Spaceframe Tubular Cage (Buggy 1600 pyramid geometry)
    cage_bars = [
        ((int(340 * ss), ground_y - int(85 * ss)), (int(410 * ss), ground_y - int(240 * ss))),
        ((int(410 * ss), ground_y - int(240 * ss)), (int(530 * ss), ground_y - int(240 * ss))),
        ((int(530 * ss), ground_y - int(240 * ss)), (int(640 * ss), ground_y - int(85 * ss))),
        ((int(340 * ss), ground_y - int(85 * ss)), (int(530 * ss), ground_y - int(240 * ss))),
        ((int(220 * ss), ground_y - int(55 * ss)), (int(730 * ss), ground_y - int(55 * ss))),
        ((int(210 * ss), ground_y - int(95 * ss)), (int(360 * ss), ground_y - int(95 * ss))),
    ]
    draw_tubular_cage_lateral(draw, cage_bars, col=cage_col, ss=ss)

    # 1600cc Atmospheric Engine & Exhaust
    draw_engine_exhaust_lateral(draw, int(290 * ss), ground_y - int(110 * ss), int(170 * ss), ground_y - int(130 * ss), ss=ss)

    # High-mount Buggy 1600 Rear Wing
    wing_x = int(240 * ss)
    draw.line([wing_x, ground_y - int(110 * ss), wing_x - int(30 * ss), ground_y - int(285 * ss)], fill=(40, 42, 50), width=int(6 * ss))
    draw.polygon([
        (wing_x - int(120 * ss), ground_y - int(300 * ss)),
        (wing_x + int(20 * ss), ground_y - int(285 * ss)),
        (wing_x + int(15 * ss), ground_y - int(265 * ss)),
        (wing_x - int(125 * ss), ground_y - int(280 * ss)),
    ], fill=primary_col, outline=(30, 32, 38), width=int(2 * ss))
    # Wing Endplates
    draw.polygon([
        (wing_x - int(130 * ss), ground_y - int(320 * ss)),
        (wing_x - int(30 * ss), ground_y - int(320 * ss)),
        (wing_x - int(15 * ss), ground_y - int(260 * ss)),
        (wing_x - int(115 * ss), ground_y - int(260 * ss)),
    ], fill=accent_col, outline=(20, 22, 26), width=int(2 * ss))

    # Wheels
    draw_knobby_wheel_lateral(draw, wr_x, cy_wheel, r_rear, rim_col=(230, 230, 235), beadlock_col=beadlock_col, ss=ss)
    draw_knobby_wheel_lateral(draw, wf_x, cy_wheel, r_front, rim_col=(230, 230, 235), beadlock_col=beadlock_col, ss=ss)

    # Top-Down (1024x1024)
    W_TD, H_TD = 512 * ss, 512 * ss
    im_td = Image.new("RGBA", (W_TD, H_TD), (0, 0, 0, 0))
    draw_td = ImageDraw.Draw(im_td)
    cy = int(256 * ss)

    # Suspension Wishbones
    draw_td.line([int(330 * ss), cy - int(40 * ss), int(390 * ss), cy - int(115 * ss)], fill=(170, 175, 185), width=int(5 * ss))
    draw_td.line([int(370 * ss), cy - int(40 * ss), int(390 * ss), cy - int(115 * ss)], fill=(170, 175, 185), width=int(5 * ss))
    draw_td.line([int(330 * ss), cy + int(40 * ss), int(390 * ss), cy + int(115 * ss)], fill=(170, 175, 185), width=int(5 * ss))
    draw_td.line([int(370 * ss), cy + int(40 * ss), int(390 * ss), cy + int(115 * ss)], fill=(170, 175, 185), width=int(5 * ss))

    draw_td.line([int(140 * ss), cy - int(45 * ss), int(120 * ss), cy - int(120 * ss)], fill=(170, 175, 185), width=int(5 * ss))
    draw_td.line([int(180 * ss), cy - int(45 * ss), int(120 * ss), cy - int(120 * ss)], fill=(170, 175, 185), width=int(5 * ss))
    draw_td.line([int(140 * ss), cy + int(45 * ss), int(120 * ss), cy + int(120 * ss)], fill=(170, 175, 185), width=int(5 * ss))
    draw_td.line([int(180 * ss), cy + int(45 * ss), int(120 * ss), cy + int(120 * ss)], fill=(170, 175, 185), width=int(5 * ss))

    # Wheels (Front X=390, Rear X=120)
    draw_wheel_topdown(draw_td, int(390 * ss), cy - int(115 * ss), int(78 * ss), int(38 * ss), rim_col=beadlock_col, ss=ss)
    draw_wheel_topdown(draw_td, int(390 * ss), cy + int(115 * ss), int(78 * ss), int(38 * ss), rim_col=beadlock_col, ss=ss)
    draw_wheel_topdown(draw_td, int(120 * ss), cy - int(120 * ss), int(88 * ss), int(46 * ss), rim_col=beadlock_col, ss=ss)
    draw_wheel_topdown(draw_td, int(120 * ss), cy + int(120 * ss), int(88 * ss), int(46 * ss), rim_col=beadlock_col, ss=ss)

    # Slender Chassis Pod & Needle Nose (+X: 320..465)
    nose_td = [
        (int(320 * ss), cy - int(38 * ss)),
        (int(450 * ss), cy - int(14 * ss)),
        (int(465 * ss), cy),
        (int(450 * ss), cy + int(14 * ss)),
        (int(320 * ss), cy + int(38 * ss)),
    ]
    draw_td.polygon(nose_td, fill=primary_col, outline=(20, 24, 30), width=int(2 * ss))
    draw_td.polygon([
        (int(335 * ss), cy - int(20 * ss)),
        (int(445 * ss), cy),
        (int(335 * ss), cy + int(20 * ss)),
    ], fill=accent_col)

    # Cockpit Driver (X=250)
    draw_td.rectangle([int(210 * ss), cy - int(24 * ss), int(290 * ss), cy + int(24 * ss)], fill=(18, 20, 24))
    draw_circle(draw_td, int(250 * ss), cy, int(20 * ss), fill=(245, 245, 250), outline=(20, 22, 28), width=int(2 * ss))
    draw_td.line([int(260 * ss), cy - int(10 * ss), int(260 * ss), cy + int(10 * ss)], fill=(20, 25, 35), width=int(4 * ss))

    # Cage
    draw_td.rectangle([int(190 * ss), cy - int(35 * ss), int(310 * ss), cy + int(35 * ss)], outline=cage_col, fill=None, width=int(5 * ss))
    draw_td.line([int(190 * ss), cy - int(35 * ss), int(310 * ss), cy + int(35 * ss)], fill=cage_col, width=int(4 * ss))
    draw_td.line([int(190 * ss), cy + int(35 * ss), int(310 * ss), cy - int(35 * ss)], fill=cage_col, width=int(4 * ss))

    # Massive Rear Wing (X: 70..110, Y: cy±85)
    draw_td.rectangle([int(70 * ss), cy - int(85 * ss), int(108 * ss), cy + int(85 * ss)], fill=primary_col, outline=(25, 28, 35), width=int(2 * ss))
    draw_td.rectangle([int(60 * ss), cy - int(92 * ss), int(118 * ss), cy - int(82 * ss)], fill=accent_col)
    draw_td.rectangle([int(60 * ss), cy + int(82 * ss), int(118 * ss), cy + int(92 * ss)], fill=accent_col)

    save_dual_views(model_id, im_lat, im_td)


def generate_touring_ax(model_id, primary_col, accent_col, secondary_col, body_style="hatch"):
    """
    Tier 4: TouringAutocross 600+ HP modified silhouette touring cars.
    body_style: 'hatch' (Bohemia Veloce), 'sedan' (Shinano Tsunami), 'coupe' (Vortek Quattro).
    Features wide box flares, vented hood, roof air scoop, giant rallycross rear wing.
    """
    ss = 2
    # Lateral (2048x1024)
    W, H = 1024 * ss, 512 * ss
    im_lat = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im_lat)
    ground_y = int(450 * ss)
    r_wheel = int(64 * ss)
    wf_x = int(740 * ss)
    wr_x = int(280 * ss)
    cy_wheel = ground_y - r_wheel - int(4 * ss)

    # Ground shadow
    draw_soft_shadow(draw, int(512 * ss), ground_y, int(410 * ss), int(22 * ss), 160)

    # Main Body Shell (Facing right: Nose at x=870, Tail at x=170)
    if body_style == "hatch":
        # Boxy rally hatchback silhouette
        body_poly = [
            (int(170 * ss), ground_y - int(85 * ss)),   # Rear bumper
            (int(170 * ss), ground_y - int(175 * ss)),  # Hatch corner
            (int(230 * ss), ground_y - int(240 * ss)),  # Roof rear
            (int(540 * ss), ground_y - int(240 * ss)),  # Roof front
            (int(670 * ss), ground_y - int(145 * ss)),  # Windshield base
            (int(850 * ss), ground_y - int(130 * ss)),  # Hood edge
            (int(875 * ss), ground_y - int(65 * ss)),   # Front bumper nose
            (int(840 * ss), ground_y - int(40 * ss)),   # Front splitter
            (int(190 * ss), ground_y - int(40 * ss)),   # Rear diffuser
        ]
    elif body_style == "sedan":
        # Aggressive notchback sedan silhouette
        body_poly = [
            (int(160 * ss), ground_y - int(85 * ss)),
            (int(175 * ss), ground_y - int(155 * ss)),  # Trunk lid
            (int(280 * ss), ground_y - int(155 * ss)),  # Rear window base
            (int(360 * ss), ground_y - int(235 * ss)),  # Roof rear
            (int(560 * ss), ground_y - int(235 * ss)),  # Roof front
            (int(680 * ss), ground_y - int(140 * ss)),  # Hood base
            (int(865 * ss), ground_y - int(125 * ss)),  # Hood tip
            (int(885 * ss), ground_y - int(65 * ss)),   # Nose
            (int(850 * ss), ground_y - int(40 * ss)),
            (int(180 * ss), ground_y - int(40 * ss)),
        ]
    else:  # coupe
        # Swept DTM-style coupe silhouette
        body_poly = [
            (int(165 * ss), ground_y - int(85 * ss)),
            (int(190 * ss), ground_y - int(160 * ss)),
            (int(300 * ss), ground_y - int(170 * ss)),
            (int(390 * ss), ground_y - int(225 * ss)),
            (int(580 * ss), ground_y - int(225 * ss)),
            (int(700 * ss), ground_y - int(135 * ss)),
            (int(870 * ss), ground_y - int(120 * ss)),
            (int(890 * ss), ground_y - int(65 * ss)),
            (int(855 * ss), ground_y - int(40 * ss)),
            (int(185 * ss), ground_y - int(40 * ss)),
        ]

    draw.polygon(body_poly, fill=primary_col, outline=(25, 28, 35), width=int(2 * ss))

    # Flared Fender Wheel Wells (Cutouts)
    draw_circle(draw, wf_x, cy_wheel, int(r_wheel * 1.25), fill=(20, 22, 26))
    draw_circle(draw, wr_x, cy_wheel, int(r_wheel * 1.25), fill=(20, 22, 26))
    # Box Fender Flare Over-Arches
    draw.arc([wf_x - int(r_wheel * 1.35), cy_wheel - int(r_wheel * 1.35), wf_x + int(r_wheel * 1.35), cy_wheel + int(r_wheel * 1.35)], start=180, end=0, fill=accent_col, width=int(8 * ss))
    draw.arc([wr_x - int(r_wheel * 1.35), cy_wheel - int(r_wheel * 1.35), wr_x + int(r_wheel * 1.35), cy_wheel + int(r_wheel * 1.35)], start=180, end=0, fill=accent_col, width=int(8 * ss))

    # Cabin Glass Windows (with white reflection slash)
    window_poly = [
        (int(380 * ss), ground_y - int(225 * ss)),
        (int(550 * ss), ground_y - int(225 * ss)),
        (int(650 * ss), ground_y - int(150 * ss)),
        (int(320 * ss), ground_y - int(150 * ss)),
    ]
    draw.polygon(window_poly, fill=(45, 110, 180, 230), outline=(20, 22, 28), width=int(2 * ss))
    # Window Pillar B-post
    draw.line([int(480 * ss), ground_y - int(225 * ss), int(470 * ss), ground_y - int(150 * ss)], fill=(20, 22, 28), width=int(6 * ss))

    # Livery Graphics: Bold contrast speed swishes and door triangles
    draw.polygon([
        (int(320 * ss), ground_y - int(140 * ss)),
        (int(630 * ss), ground_y - int(140 * ss)),
        (int(720 * ss), ground_y - int(60 * ss)),
        (int(410 * ss), ground_y - int(60 * ss)),
    ], fill=accent_col)
    draw.polygon([
        (int(420 * ss), ground_y - int(135 * ss)),
        (int(580 * ss), ground_y - int(135 * ss)),
        (int(640 * ss), ground_y - int(70 * ss)),
        (int(480 * ss), ground_y - int(70 * ss)),
    ], fill=secondary_col)

    # Front Splitter and Carbon Aero Dive Planes
    draw.polygon([
        (int(830 * ss), ground_y - int(45 * ss)),
        (int(895 * ss), ground_y - int(45 * ss)),
        (int(890 * ss), ground_y - int(35 * ss)),
        (int(825 * ss), ground_y - int(35 * ss)),
    ], fill=(25, 28, 32))

    # Giant Touring AX Rear Wing
    wing_x = int(180 * ss)
    draw.line([wing_x + int(40 * ss), ground_y - int(170 * ss), wing_x, ground_y - int(290 * ss)], fill=(30, 32, 38), width=int(6 * ss))
    draw.polygon([
        (wing_x - int(60 * ss), ground_y - int(305 * ss)),
        (wing_x + int(50 * ss), ground_y - int(290 * ss)),
        (wing_x + int(45 * ss), ground_y - int(275 * ss)),
        (wing_x - int(65 * ss), ground_y - int(290 * ss)),
    ], fill=secondary_col, outline=(25, 28, 35), width=int(2 * ss))
    # Wing Endplate
    draw.polygon([
        (wing_x - int(70 * ss), ground_y - int(325 * ss)),
        (wing_x + int(10 * ss), ground_y - int(325 * ss)),
        (wing_x + int(20 * ss), ground_y - int(260 * ss)),
        (wing_x - int(60 * ss), ground_y - int(260 * ss)),
    ], fill=accent_col)

    # Roof Air Scoop
    draw.polygon([
        (int(470 * ss), ground_y - int(242 * ss)),
        (int(520 * ss), ground_y - int(270 * ss)),
        (int(545 * ss), ground_y - int(270 * ss)),
        (int(550 * ss), ground_y - int(242 * ss)),
    ], fill=secondary_col, outline=(25, 28, 35), width=int(2 * ss))

    # Wheels
    draw_knobby_wheel_lateral(draw, wr_x, cy_wheel, r_wheel, rim_col=(225, 225, 230), beadlock_col=accent_col, ss=ss)
    draw_knobby_wheel_lateral(draw, wf_x, cy_wheel, r_wheel, rim_col=(225, 225, 230), beadlock_col=accent_col, ss=ss)

    # Top-Down (1024x1024)
    W_TD, H_TD = 512 * ss, 512 * ss
    im_td = Image.new("RGBA", (W_TD, H_TD), (0, 0, 0, 0))
    draw_td = ImageDraw.Draw(im_td)
    cy = int(256 * ss)

    # Four Wide Touring Wheels
    draw_wheel_topdown(draw_td, int(370 * ss), cy - int(100 * ss), int(80 * ss), int(42 * ss), rim_col=accent_col, ss=ss)
    draw_wheel_topdown(draw_td, int(370 * ss), cy + int(100 * ss), int(80 * ss), int(42 * ss), rim_col=accent_col, ss=ss)
    draw_wheel_topdown(draw_td, int(140 * ss), cy - int(100 * ss), int(80 * ss), int(42 * ss), rim_col=accent_col, ss=ss)
    draw_wheel_topdown(draw_td, int(140 * ss), cy + int(100 * ss), int(80 * ss), int(42 * ss), rim_col=accent_col, ss=ss)

    # Widebody Touring Silhouette (+X is nose on right)
    body_td = [
        (int(90 * ss), cy - int(85 * ss)),   # Rear left
        (int(180 * ss), cy - int(92 * ss)),  # Rear box flare
        (int(320 * ss), cy - int(88 * ss)),  # Front box flare
        (int(440 * ss), cy - int(78 * ss)),  # Front bumper corner
        (int(465 * ss), cy - int(45 * ss)),  # Nose front
        (int(465 * ss), cy + int(45 * ss)),
        (int(440 * ss), cy + int(78 * ss)),
        (int(320 * ss), cy + int(88 * ss)),
        (int(180 * ss), cy + int(92 * ss)),
        (int(90 * ss), cy + int(85 * ss)),
    ]
    draw_td.polygon(body_td, fill=primary_col, outline=(25, 28, 35), width=int(2 * ss))

    # Hood & Roof Livery Stripes (Dual center racing bands)
    draw_td.rectangle([int(95 * ss), cy - int(25 * ss), int(460 * ss), cy - int(8 * ss)], fill=accent_col)
    draw_td.rectangle([int(95 * ss), cy + int(8 * ss), int(460 * ss), cy + int(25 * ss)], fill=secondary_col)

    # Cabin Roof & Windshield Glass
    draw_td.polygon([
        (int(200 * ss), cy - int(65 * ss)),
        (int(330 * ss), cy - int(60 * ss)),
        (int(360 * ss), cy - int(50 * ss)),
        (int(360 * ss), cy + int(50 * ss)),
        (int(330 * ss), cy + int(60 * ss)),
        (int(200 * ss), cy + int(65 * ss)),
    ], fill=(35, 45, 60), outline=(20, 22, 28), width=int(2 * ss))

    # Roof Air Scoop
    draw_td.rectangle([int(260 * ss), cy - int(15 * ss), int(300 * ss), cy + int(15 * ss)], fill=secondary_col, outline=(20, 22, 28))

    # Wide Rear Wing
    draw_td.rectangle([int(75 * ss), cy - int(95 * ss), int(105 * ss), cy + int(95 * ss)], fill=secondary_col, outline=(25, 28, 35), width=int(2 * ss))
    draw_td.rectangle([int(68 * ss), cy - int(100 * ss), int(112 * ss), cy - int(90 * ss)], fill=accent_col)
    draw_td.rectangle([int(68 * ss), cy + int(90 * ss), int(112 * ss), cy + int(100 * ss)], fill=accent_col)

    save_dual_views(model_id, im_lat, im_td)


def generate_superbuggy(model_id, primary_col, accent_col, secondary_col, beadlock_col):
    """
    Tier 5: SuperBuggy 800+ HP unlimited open-wheel twin-engine / V8 beasts.
    Massive stance, extreme multi-element wing, exposed twin-turbo or V8 manifold, wide knobby tires.
    """
    ss = 2
    # Lateral (2048x1024)
    W, H = 1024 * ss, 512 * ss
    im_lat = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im_lat)
    ground_y = int(450 * ss)
    r_front = int(66 * ss)
    r_rear = int(78 * ss)
    wf_x = int(770 * ss)
    wr_x = int(250 * ss)
    cy_wheel = ground_y - r_rear - int(6 * ss)

    # Massive ground contact shadow
    draw_soft_shadow(draw, int(512 * ss), ground_y, int(430 * ss), int(24 * ss), 160)

    # Exposed Heavy-Duty 4WD Suspension & Shock Towers
    draw.line([wf_x - int(85 * ss), cy_wheel - int(65 * ss), wf_x, cy_wheel], fill=(230, 45, 35), width=int(6 * ss))
    draw.line([wr_x + int(85 * ss), cy_wheel - int(75 * ss), wr_x, cy_wheel], fill=(230, 45, 35), width=int(7 * ss))

    # Aggressive High-Rake Nosecone (+X on right: 540..880)
    nose_poly = [
        (int(520 * ss), ground_y - int(85 * ss)),
        (int(740 * ss), ground_y - int(80 * ss)),
        (int(880 * ss), ground_y - int(52 * ss)),  # Needle nose tip
        (int(865 * ss), ground_y - int(32 * ss)),
        (int(630 * ss), ground_y - int(45 * ss)),
        (int(500 * ss), ground_y - int(45 * ss)),
    ]
    draw.polygon(nose_poly, fill=primary_col, outline=(25, 28, 35), width=int(2 * ss))

    # Bold Tri-Tone Dynamic Livery Shards
    draw.polygon([
        (int(600 * ss), ground_y - int(82 * ss)),
        (int(760 * ss), ground_y - int(76 * ss)),
        (int(875 * ss), ground_y - int(52 * ss)),
        (int(720 * ss), ground_y - int(48 * ss)),
    ], fill=accent_col)
    draw.polygon([
        (int(650 * ss), ground_y - int(80 * ss)),
        (int(740 * ss), ground_y - int(75 * ss)),
        (int(820 * ss), ground_y - int(52 * ss)),
        (int(730 * ss), ground_y - int(50 * ss)),
    ], fill=secondary_col)

    # Cockpit Driver (Centered X=450)
    draw_driver_lateral(draw, int(450 * ss), ground_y - int(165 * ss), ss=ss)

    # Heavy Reinforced Tubular Spaceframe
    cage_bars = [
        ((int(320 * ss), ground_y - int(90 * ss)), (int(390 * ss), ground_y - int(250 * ss))),
        ((int(390 * ss), ground_y - int(250 * ss)), (int(520 * ss), ground_y - int(250 * ss))),
        ((int(520 * ss), ground_y - int(250 * ss)), (int(630 * ss), ground_y - int(85 * ss))),
        ((int(320 * ss), ground_y - int(90 * ss)), (int(520 * ss), ground_y - int(250 * ss))),
        ((int(190 * ss), ground_y - int(55 * ss)), (int(740 * ss), ground_y - int(55 * ss))),
        ((int(180 * ss), ground_y - int(105 * ss)), (int(340 * ss), ground_y - int(105 * ss))),
    ]
    draw_tubular_cage_lateral(draw, cage_bars, col=(240, 240, 245), ss=ss)

    # Monster V8 / Twin-Turbo Engine Block & Dual Titanium Exhausts
    draw_engine_exhaust_lateral(draw, int(270 * ss), ground_y - int(115 * ss), int(150 * ss), ground_y - int(140 * ss), ss=ss)
    draw_engine_exhaust_lateral(draw, int(270 * ss), ground_y - int(95 * ss), int(140 * ss), ground_y - int(115 * ss), ss=ss)

    # Extreme Multi-Element SuperBuggy Rear Wing
    wing_x = int(220 * ss)
    draw.line([wing_x + int(10 * ss), ground_y - int(115 * ss), wing_x - int(40 * ss), ground_y - int(310 * ss)], fill=(35, 38, 45), width=int(7 * ss))
    # Top main wing element
    draw.polygon([
        (wing_x - int(140 * ss), ground_y - int(325 * ss)),
        (wing_x + int(30 * ss), ground_y - int(305 * ss)),
        (wing_x + int(25 * ss), ground_y - int(285 * ss)),
        (wing_x - int(145 * ss), ground_y - int(305 * ss)),
    ], fill=primary_col, outline=(30, 32, 38), width=int(2 * ss))
    # Lower flap element
    draw.polygon([
        (wing_x - int(100 * ss), ground_y - int(275 * ss)),
        (wing_x + int(15 * ss), ground_y - int(260 * ss)),
        (wing_x + int(12 * ss), ground_y - int(248 * ss)),
        (wing_x - int(103 * ss), ground_y - int(263 * ss)),
    ], fill=secondary_col)
    # Giant Aerodynamic Endplate
    draw.polygon([
        (wing_x - int(155 * ss), ground_y - int(345 * ss)),
        (wing_x - int(40 * ss), ground_y - int(345 * ss)),
        (wing_x - int(20 * ss), ground_y - int(240 * ss)),
        (wing_x - int(135 * ss), ground_y - int(240 * ss)),
    ], fill=accent_col, outline=(20, 22, 26), width=int(2 * ss))

    # Wheels (Large high-offset knobby tires)
    draw_knobby_wheel_lateral(draw, wr_x, cy_wheel, r_rear, rim_col=(235, 235, 240), beadlock_col=beadlock_col, ss=ss)
    draw_knobby_wheel_lateral(draw, wf_x, cy_wheel, r_front, rim_col=(235, 235, 240), beadlock_col=beadlock_col, ss=ss)

    # Top-Down (1024x1024)
    W_TD, H_TD = 512 * ss, 512 * ss
    im_td = Image.new("RGBA", (W_TD, H_TD), (0, 0, 0, 0))
    draw_td = ImageDraw.Draw(im_td)
    cy = int(256 * ss)

    # Wide Track Wishbones
    draw_td.line([int(320 * ss), cy - int(45 * ss), int(400 * ss), cy - int(125 * ss)], fill=(180, 185, 195), width=int(6 * ss))
    draw_td.line([int(370 * ss), cy - int(45 * ss), int(400 * ss), cy - int(125 * ss)], fill=(180, 185, 195), width=int(6 * ss))
    draw_td.line([int(320 * ss), cy + int(45 * ss), int(400 * ss), cy + int(125 * ss)], fill=(180, 185, 195), width=int(6 * ss))
    draw_td.line([int(370 * ss), cy + int(45 * ss), int(400 * ss), cy + int(125 * ss)], fill=(180, 185, 195), width=int(6 * ss))

    draw_td.line([int(130 * ss), cy - int(50 * ss), int(115 * ss), cy - int(130 * ss)], fill=(180, 185, 195), width=int(6 * ss))
    draw_td.line([int(180 * ss), cy - int(50 * ss), int(115 * ss), cy - int(130 * ss)], fill=(180, 185, 195), width=int(6 * ss))
    draw_td.line([int(130 * ss), cy + int(50 * ss), int(115 * ss), cy + int(130 * ss)], fill=(180, 185, 195), width=int(6 * ss))
    draw_td.line([int(180 * ss), cy + int(50 * ss), int(115 * ss), cy + int(130 * ss)], fill=(180, 185, 195), width=int(6 * ss))

    # Four Massive Knobby Wheels (Front X=400, Rear X=115)
    draw_wheel_topdown(draw_td, int(400 * ss), cy - int(125 * ss), int(84 * ss), int(42 * ss), rim_col=beadlock_col, ss=ss)
    draw_wheel_topdown(draw_td, int(400 * ss), cy + int(125 * ss), int(84 * ss), int(42 * ss), rim_col=beadlock_col, ss=ss)
    draw_wheel_topdown(draw_td, int(115 * ss), cy - int(130 * ss), int(94 * ss), int(50 * ss), rim_col=beadlock_col, ss=ss)
    draw_wheel_topdown(draw_td, int(115 * ss), cy + int(130 * ss), int(94 * ss), int(50 * ss), rim_col=beadlock_col, ss=ss)

    # Wide Aero Nosecone (+X: 310..475)
    nose_td = [
        (int(310 * ss), cy - int(45 * ss)),
        (int(460 * ss), cy - int(16 * ss)),
        (int(475 * ss), cy),
        (int(460 * ss), cy + int(16 * ss)),
        (int(310 * ss), cy + int(45 * ss)),
    ]
    draw_td.polygon(nose_td, fill=primary_col, outline=(20, 24, 30), width=int(2 * ss))
    draw_td.polygon([
        (int(325 * ss), cy - int(24 * ss)),
        (int(455 * ss), cy),
        (int(325 * ss), cy + int(24 * ss)),
    ], fill=accent_col)

    # Cockpit Driver (X=240)
    draw_td.rectangle([int(200 * ss), cy - int(26 * ss), int(280 * ss), cy + int(26 * ss)], fill=(18, 20, 24))
    draw_circle(draw_td, int(240 * ss), cy, int(22 * ss), fill=(245, 245, 250), outline=(20, 22, 28), width=int(2 * ss))
    draw_td.line([int(252 * ss), cy - int(11 * ss), int(252 * ss), cy + int(11 * ss)], fill=(20, 25, 35), width=int(4 * ss))

    # Roll Cage
    draw_td.rectangle([int(180 * ss), cy - int(38 * ss), int(300 * ss), cy + int(38 * ss)], outline=(240, 240, 245), fill=None, width=int(5 * ss))
    draw_td.line([int(180 * ss), cy - int(38 * ss), int(300 * ss), cy + int(38 * ss)], fill=(240, 240, 245), width=int(4 * ss))
    draw_td.line([int(180 * ss), cy + int(38 * ss), int(300 * ss), cy - int(38 * ss)], fill=(240, 240, 245), width=int(4 * ss))

    # Extreme Rear Wing (X: 60..105, Y: cy±95)
    draw_td.rectangle([int(60 * ss), cy - int(95 * ss), int(105 * ss), cy + int(95 * ss)], fill=primary_col, outline=(25, 28, 35), width=int(2 * ss))
    draw_td.rectangle([int(50 * ss), cy - int(102 * ss), int(115 * ss), cy - int(90 * ss)], fill=accent_col)
    draw_td.rectangle([int(50 * ss), cy + int(90 * ss), int(115 * ss), cy + int(102 * ss)], fill=accent_col)

    save_dual_views(model_id, im_lat, im_td)


# ==============================================================================
# Master Dispatcher
# ==============================================================================

FLEET = {
    # Tier 1: Cross Car Junior
    "autocross_ardennes_junior_t1": lambda: generate_crosscar(
        "autocross_ardennes_junior_t1",
        primary_col=(20, 160, 220),      # Electric Cyan
        accent_col=(240, 220, 30),       # Neon Yellow
        cage_col=(40, 180, 240),
        beadlock_col=(240, 220, 30),
        style="junior"
    ),
    "autocross_iberian_furia_t1": lambda: generate_crosscar(
        "autocross_iberian_furia_t1",
        primary_col=(215, 35, 30),       # Fiery Crimson
        accent_col=(245, 190, 25),       # Sunburst Yellow
        cage_col=(220, 50, 40),
        beadlock_col=(245, 190, 25),
        style="junior"
    ),
    "autocross_cosmo_nova_t1": lambda: generate_crosscar(
        "autocross_cosmo_nova_t1",
        primary_col=(245, 248, 255),     # Pure White
        accent_col=(30, 95, 195),        # French Racing Navy
        cage_col=(50, 120, 220),
        beadlock_col=(30, 95, 195),
        style="junior"
    ),

    # Tier 2: Cross Car Senior
    "autocross_ardennes_pro_t2": lambda: generate_crosscar(
        "autocross_ardennes_pro_t2",
        primary_col=(60, 65, 75),        # Matte Gunmetal
        accent_col=(180, 240, 35),       # Acid Lime
        cage_col=(180, 240, 35),
        beadlock_col=(180, 240, 35),
        style="senior"
    ),
    "autocross_iberian_relampago_t2": lambda: generate_crosscar(
        "autocross_iberian_relampago_t2",
        primary_col=(235, 95, 20),       # Racing Orange
        accent_col=(30, 32, 38),         # Jet Black
        cage_col=(235, 95, 20),
        beadlock_col=(210, 215, 225),
        style="senior"
    ),
    "autocross_lusitania_bravo_t2": lambda: generate_crosscar(
        "autocross_lusitania_bravo_t2",
        primary_col=(25, 75, 185),       # Royal Blue
        accent_col=(245, 245, 250),      # Bright White
        cage_col=(210, 160, 35),         # Gold
        beadlock_col=(210, 160, 35),
        style="senior"
    ),

    # Tier 3: Buggy 1600
    "autocross_petersen_buggy1600_t3": lambda: generate_buggy1600(
        "autocross_petersen_buggy1600_t3",
        primary_col=(235, 110, 25),      # Dutch Orange
        accent_col=(245, 245, 250),      # Bright White
        cage_col=(235, 110, 25),
        beadlock_col=(210, 215, 225)
    ),
    "autocross_bologna_buggy1600_t3": lambda: generate_buggy1600(
        "autocross_bologna_buggy1600_t3",
        primary_col=(205, 30, 35),       # Rosso Corsa Red
        accent_col=(220, 175, 45),       # Gold
        cage_col=(205, 30, 35),
        beadlock_col=(220, 175, 45)
    ),
    "autocross_rapid_buggy1600_t3": lambda: generate_buggy1600(
        "autocross_rapid_buggy1600_t3",
        primary_col=(25, 90, 200),       # Cobalt Blue
        accent_col=(235, 230, 35),       # Fluorescent Yellow
        cage_col=(235, 230, 35),
        beadlock_col=(235, 230, 35)
    ),

    # Tier 4: TouringAutocross
    "autocross_bohemia_veloce_t4": lambda: generate_touring_ax(
        "autocross_bohemia_veloce_t4",
        primary_col=(35, 165, 75),       # Rally Green
        accent_col=(245, 245, 250),      # Pure White
        secondary_col=(45, 50, 60),      # Graphite
        body_style="hatch"
    ),
    "autocross_shinano_tsunami_t4": lambda: generate_touring_ax(
        "autocross_shinano_tsunami_t4",
        primary_col=(245, 248, 252),     # Pearl White
        accent_col=(215, 35, 35),        # Crimson Red
        secondary_col=(30, 32, 38),      # Carbon Black
        body_style="sedan"
    ),
    "autocross_vortek_quattro_t4": lambda: generate_touring_ax(
        "autocross_vortek_quattro_t4",
        primary_col=(70, 75, 85),        # Slate Grey
        accent_col=(225, 40, 35),        # Signal Red
        secondary_col=(25, 27, 32),      # Gloss Black
        body_style="coupe"
    ),

    # Tier 5: SuperBuggy
    "autocross_petersen_superbuggy_t5": lambda: generate_superbuggy(
        "autocross_petersen_superbuggy_t5",
        primary_col=(20, 60, 160),       # Midnight Blue
        accent_col=(240, 120, 25),       # Neon Orange
        secondary_col=(245, 245, 250),   # White
        beadlock_col=(240, 120, 25)
    ),
    "autocross_bologna_superbuggy_t5": lambda: generate_superbuggy(
        "autocross_bologna_superbuggy_t5",
        primary_col=(185, 25, 30),       # Deep Italian Crimson
        accent_col=(215, 220, 230),      # Metallic Silver
        secondary_col=(35, 38, 45),      # Carbon
        beadlock_col=(215, 220, 230)
    ),
    "autocross_rapid_superbuggy_t5": lambda: generate_superbuggy(
        "autocross_rapid_superbuggy_t5",
        primary_col=(235, 230, 30),      # Fluorescent Neon Yellow
        accent_col=(25, 28, 35),         # Stealth Black
        secondary_col=(25, 175, 205),    # Teal
        beadlock_col=(25, 175, 205)
    ),
}


def main():
    parser = argparse.ArgumentParser(description="Generate 2D vehicle sprites for the Autocross fleet.")
    parser.add_argument("--model", type=str, help="Specific model ID to generate")
    parser.add_argument("--tier", type=int, choices=[1, 2, 3, 4, 5], help="Generate only vehicles for this tier")
    parser.add_argument("--all", action="store_true", default=True, help="Generate all 15 vehicles (default)")
    args = parser.parse_args()

    targets = []
    if args.model:
        if args.model not in FLEET:
            print(f"Error: Unknown model ID '{args.model}'")
            return 1
        targets = [args.model]
    elif args.tier:
        tier_map = {
            1: ["autocross_ardennes_junior_t1", "autocross_iberian_furia_t1", "autocross_cosmo_nova_t1"],
            2: ["autocross_ardennes_pro_t2", "autocross_iberian_relampago_t2", "autocross_lusitania_bravo_t2"],
            3: ["autocross_petersen_buggy1600_t3", "autocross_bologna_buggy1600_t3", "autocross_rapid_buggy1600_t3"],
            4: ["autocross_bohemia_veloce_t4", "autocross_shinano_tsunami_t4", "autocross_vortek_quattro_t4"],
            5: ["autocross_petersen_superbuggy_t5", "autocross_bologna_superbuggy_t5", "autocross_rapid_superbuggy_t5"],
        }
        targets = tier_map[args.tier]
    else:
        targets = list(FLEET.keys())

    print(f"🎨 Generating 2D Sprites for {len(targets)} Autocross vehicle(s) (Facing RIGHT, Zero IP)...")
    for model_id in targets:
        FLEET[model_id]()

    print("\n✨ Done! All requested textures saved to assets/textures/vehicles/{laterals,topdown}/autocross/")
    return 0


if __name__ == "__main__":
    exit(main())
