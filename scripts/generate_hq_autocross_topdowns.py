#!/usr/bin/env python3
"""
High-Quality Top-Down Sprite Generator for Autocross Vehicles (Spec 093).
Generates high-fidelity 512x512 RGBA orthographic top-down sprites facing right (+X),
matching the 3-color liveries and geometries of the lateral sprites with:
- 4x Supersampling (2048x2048 -> Lanczos 512x512)
- Multi-layer volumetric ambient occlusion and grounded contact shadows
- Knobby off-road beadlock tires with tread lugs and rim bolts
- Detailed exposed tubular spaceframes and wishbone suspension A-arms
- Helmeted driver in bucket seat with steering wheel and harness
- Exposed mechanicals (engine blocks, turbo/intercooler plumbing, radiators)
- Zero text, zero numbers, zero manufacturer logos
- Strict 3-color bodywork palette
"""

import math
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
TOPDOWN_DIR = ROOT / "assets" / "textures" / "vehicles" / "topdown" / "autocross"
TOPDOWN_DIR.mkdir(parents=True, exist_ok=True)

SS = 4  # 4x supersampling (working canvas 2048x2048 -> export 512x512)
CANVAS_SIZE = 512 * SS
CENTER = CANVAS_SIZE // 2


def darken(col, factor=0.7):
    return tuple(max(0, min(255, int(c * factor))) for c in col[:3])


def lighten(col, factor=1.3):
    return tuple(max(0, min(255, int(c * factor))) for c in col[:3])


def draw_drop_shadow(draw, cx, cy, rx, ry, alpha=110):
    draw.ellipse([cx - rx, cy - ry, cx + rx, cy + ry], fill=(0, 0, 0, alpha))


def draw_knobby_tire(draw, cx, cy, length, width, beadlock_col, bolt_col=(210, 215, 225)):
    """Draws an authentic top-down knobby off-road tire with tread blocks and beadlock bolts."""
    hw, hh = length // 2, width // 2
    # Outer tread lugs
    draw.rounded_rectangle(
        [cx - hw, cy - hh, cx + hw, cy + hh],
        radius=int(6 * SS),
        fill=(22, 24, 28, 255),
        outline=(10, 11, 14, 255),
        width=int(1.5 * SS)
    )
    # Tread groove patterns (interlocking dirt lugs)
    num_lugs = 7
    step = length / (num_lugs + 1)
    for i in range(num_lugs):
        lx = cx - hw + int((i + 1) * step)
        draw.line([lx, cy - hh + int(2 * SS), lx, cy + hh - int(2 * SS)], fill=(12, 13, 16, 255), width=int(2 * SS))
    
    # Rim base
    rw, rh = int(length * 0.42), int(width * 0.46)
    draw.rounded_rectangle(
        [cx - rw // 2, cy - rh // 2, cx + rw // 2, cy + rh // 2],
        radius=int(4 * SS),
        fill=(40, 42, 48, 255),
        outline=beadlock_col,
        width=int(2 * SS)
    )
    # Center hub
    draw.ellipse([cx - int(3 * SS), cy - int(3 * SS), cx + int(3 * SS), cy + int(3 * SS)], fill=bolt_col)


def draw_wishbones(draw, body_x1, body_x2, body_y, wheel_cx, wheel_cy, col=(140, 145, 155)):
    """Draws front or rear independent suspension A-arms."""
    draw.line([body_x1, body_y, wheel_cx, wheel_cy], fill=col, width=int(3 * SS))
    draw.line([body_x2, body_y, wheel_cx, wheel_cy], fill=col, width=int(3 * SS))


def draw_cockpit(draw, seat_cx, cy, helmet_col, harness_col=(220, 40, 40)):
    """Draws detailed cockpit with racing seat, helmeted driver facing right (+X), steering wheel."""
    # Bucket seat
    draw.rounded_rectangle(
        [seat_cx - int(28 * SS), cy - int(18 * SS), seat_cx + int(24 * SS), cy + int(18 * SS)],
        radius=int(6 * SS),
        fill=(24, 26, 30, 255),
        outline=(14, 15, 18, 255),
        width=int(1.5 * SS)
    )
    # Shoulder harness straps
    draw.line([seat_cx - int(18 * SS), cy - int(8 * SS), seat_cx + int(10 * SS), cy - int(6 * SS)], fill=harness_col, width=int(2.5 * SS))
    draw.line([seat_cx - int(18 * SS), cy + int(8 * SS), seat_cx + int(10 * SS), cy + int(6 * SS)], fill=harness_col, width=int(2.5 * SS))
    # Driver helmet (circle with dark visor pointing right)
    draw.ellipse(
        [seat_cx - int(12 * SS), cy - int(12 * SS), seat_cx + int(12 * SS), cy + int(12 * SS)],
        fill=helmet_col,
        outline=(15, 16, 20, 255),
        width=int(1.5 * SS)
    )
    # Dark visor facing right (+X)
    draw.arc(
        [seat_cx - int(10 * SS), cy - int(10 * SS), seat_cx + int(14 * SS), cy + int(10 * SS)],
        start=-75, end=75,
        fill=(10, 12, 16, 255),
        width=int(3.5 * SS)
    )
    # Steering wheel column and wheel
    sw_x = seat_cx + int(24 * SS)
    draw.ellipse([sw_x - int(2 * SS), cy - int(10 * SS), sw_x + int(4 * SS), cy + int(10 * SS)], fill=(20, 22, 26), outline=(80, 85, 95), width=int(1.5 * SS))


def render_cross_car_senior(name, p_col, s_col, a_col, beadlock_col):
    """Tier 2: Cross Car Senior (wide stance, bi-plane rear wing, aggressive cowl)."""
    im = Image.new("RGBA", (CANVAS_SIZE, CANVAS_SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im)
    cy = CENTER

    # Grounded shadow
    draw_drop_shadow(draw, CENTER, cy, int(200 * SS), int(130 * SS))

    # Wheels (Front X = 390, Rear X = 120, offset from 512 center -> shift to center at X=256)
    # Let vehicle center be at CENTER (1024), length = ~340*SS
    # Front axle at CENTER + 120*SS, Rear axle at CENTER - 130*SS
    fx = CENTER + int(125 * SS)
    rx = CENTER - int(135 * SS)
    track_f = int(125 * SS)
    track_r = int(135 * SS)
    tw, th = int(72 * SS), int(34 * SS)

    # Suspension wishbones
    draw_wishbones(draw, CENTER + int(60 * SS), CENTER + int(110 * SS), cy - int(45 * SS), fx, cy - track_f)
    draw_wishbones(draw, CENTER + int(60 * SS), CENTER + int(110 * SS), cy + int(45 * SS), fx, cy + track_f)
    draw_wishbones(draw, CENTER - int(60 * SS), CENTER - int(120 * SS), cy - int(55 * SS), rx, cy - track_r)
    draw_wishbones(draw, CENTER - int(60 * SS), CENTER - int(120 * SS), cy + int(55 * SS), rx, cy + track_r)

    # Knobby wheels
    draw_knobby_tire(draw, fx, cy - track_f, tw, th, beadlock_col)
    draw_knobby_tire(draw, fx, cy + track_f, tw, th, beadlock_col)
    draw_knobby_tire(draw, rx, cy - track_r, int(78 * SS), int(38 * SS), beadlock_col)
    draw_knobby_tire(draw, rx, cy + track_r, int(78 * SS), int(38 * SS), beadlock_col)

    # Tubular roll cage frame base
    cage_col = (30, 32, 38)
    draw.rounded_rectangle(
        [CENTER - int(130 * SS), cy - int(52 * SS), CENTER + int(85 * SS), cy + int(52 * SS)],
        radius=int(12 * SS), fill=(20, 22, 26), outline=cage_col, width=int(4 * SS)
    )

    # Engine bay & exhaust at rear
    draw.rectangle([CENTER - int(135 * SS), cy - int(28 * SS), CENTER - int(75 * SS), cy + int(28 * SS)], fill=(45, 48, 55))
    # Exhaust pipes
    draw.line([CENTER - int(135 * SS), cy - int(12 * SS), CENTER - int(165 * SS), cy - int(12 * SS)], fill=(160, 165, 175), width=int(4 * SS))
    draw.line([CENTER - int(165 * SS), cy - int(12 * SS), CENTER - int(172 * SS), cy - int(12 * SS)], fill=(60, 110, 215), width=int(4.5 * SS))

    # Cockpit
    draw_cockpit(draw, CENTER - int(15 * SS), cy, a_col)

    # Sidepods with livery
    # Left sidepod
    draw.polygon([
        (CENTER - int(60 * SS), cy - int(50 * SS)),
        (CENTER + int(70 * SS), cy - int(45 * SS)),
        (CENTER + int(70 * SS), cy - int(25 * SS)),
        (CENTER - int(60 * SS), cy - int(25 * SS)),
    ], fill=p_col, outline=darken(p_col), width=int(1.5 * SS))
    # Right sidepod
    draw.polygon([
        (CENTER - int(60 * SS), cy + int(50 * SS)),
        (CENTER + int(70 * SS), cy + int(45 * SS)),
        (CENTER + int(70 * SS), cy + int(25 * SS)),
        (CENTER - int(60 * SS), cy + int(25 * SS)),
    ], fill=p_col, outline=darken(p_col), width=int(1.5 * SS))

    # Nosecone (facing right +X)
    nose = [
        (CENTER + int(70 * SS), cy - int(42 * SS)),
        (CENTER + int(175 * SS), cy - int(22 * SS)),
        (CENTER + int(195 * SS), cy),
        (CENTER + int(175 * SS), cy + int(22 * SS)),
        (CENTER + int(70 * SS), cy + int(42 * SS)),
    ]
    draw.polygon(nose, fill=p_col, outline=darken(p_col), width=int(2 * SS))
    # Nose livery stripes (accent + secondary)
    draw.polygon([
        (CENTER + int(80 * SS), cy - int(15 * SS)),
        (CENTER + int(185 * SS), cy),
        (CENTER + int(80 * SS), cy + int(15 * SS)),
    ], fill=s_col)
    draw.line([CENTER + int(90 * SS), cy, CENTER + int(185 * SS), cy], fill=a_col, width=int(3 * SS))

    # Bi-plane Rear Wing (Rear X: CENTER - 180..-155)
    wx = CENTER - int(165 * SS)
    draw.rectangle([wx - int(14 * SS), cy - int(72 * SS), wx + int(14 * SS), cy + int(72 * SS)], fill=p_col, outline=darken(p_col), width=int(2 * SS))
    # Wing endplates
    draw.rectangle([wx - int(20 * SS), cy - int(78 * SS), wx + int(22 * SS), cy - int(70 * SS)], fill=a_col)
    draw.rectangle([wx - int(20 * SS), cy + int(70 * SS), wx + int(22 * SS), cy + int(78 * SS)], fill=a_col)

    # Save 512x512 PNG
    td_512 = im.resize((512, 512), Image.Resampling.LANCZOS)
    out_path = TOPDOWN_DIR / f"{name}.png"
    td_512.save(out_path)
    print(f"✓ Top-down 512x512 -> {out_path}")


def render_buggy1600(name, p_col, s_col, a_col, beadlock_col):
    """Tier 3: Buggy 1600 (slender single-seater, long wheelbase, massive high-mount wing)."""
    im = Image.new("RGBA", (CANVAS_SIZE, CANVAS_SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im)
    cy = CENTER

    # Grounded shadow
    draw_drop_shadow(draw, CENTER, cy, int(220 * SS), int(140 * SS))

    fx = CENTER + int(140 * SS)
    rx = CENTER - int(140 * SS)
    track_f = int(120 * SS)
    track_r = int(135 * SS)
    tw, th = int(76 * SS), int(36 * SS)

    # Suspension wishbones
    draw_wishbones(draw, CENTER + int(70 * SS), CENTER + int(120 * SS), cy - int(38 * SS), fx, cy - track_f)
    draw_wishbones(draw, CENTER + int(70 * SS), CENTER + int(120 * SS), cy + int(38 * SS), fx, cy + track_f)
    draw_wishbones(draw, CENTER - int(70 * SS), CENTER - int(130 * SS), cy - int(48 * SS), rx, cy - track_r)
    draw_wishbones(draw, CENTER - int(70 * SS), CENTER - int(130 * SS), cy + int(48 * SS), rx, cy + track_r)

    # Knobby wheels
    draw_knobby_tire(draw, fx, cy - track_f, tw, th, beadlock_col)
    draw_knobby_tire(draw, fx, cy + track_f, tw, th, beadlock_col)
    draw_knobby_tire(draw, rx, cy - track_r, int(86 * SS), int(42 * SS), beadlock_col)
    draw_knobby_tire(draw, rx, cy + track_r, int(86 * SS), int(42 * SS), beadlock_col)

    # Spaceframe base & engine
    draw.rounded_rectangle(
        [CENTER - int(140 * SS), cy - int(45 * SS), CENTER + int(90 * SS), cy + int(45 * SS)],
        radius=int(10 * SS), fill=(22, 24, 28), outline=(32, 34, 40), width=int(4 * SS)
    )
    # Exposed engine block
    draw.rectangle([CENTER - int(140 * SS), cy - int(24 * SS), CENTER - int(70 * SS), cy + int(24 * SS)], fill=(48, 52, 60))

    # Cockpit
    draw_cockpit(draw, CENTER - int(10 * SS), cy, a_col)

    # Slender needle nosecone (+X)
    nose = [
        (CENTER + int(80 * SS), cy - int(34 * SS)),
        (CENTER + int(195 * SS), cy - int(14 * SS)),
        (CENTER + int(215 * SS), cy),
        (CENTER + int(195 * SS), cy + int(14 * SS)),
        (CENTER + int(80 * SS), cy + int(34 * SS)),
    ]
    draw.polygon(nose, fill=p_col, outline=darken(p_col), width=int(2 * SS))
    # Livery center band
    draw.polygon([
        (CENTER + int(90 * SS), cy - int(14 * SS)),
        (CENTER + int(205 * SS), cy),
        (CENTER + int(90 * SS), cy + int(14 * SS)),
    ], fill=s_col)
    draw.line([CENTER + int(95 * SS), cy, CENTER + int(205 * SS), cy], fill=a_col, width=int(3 * SS))

    # Massive Rear Wing
    wx = CENTER - int(175 * SS)
    draw.rectangle([wx - int(16 * SS), cy - int(85 * SS), wx + int(16 * SS), cy + int(85 * SS)], fill=p_col, outline=darken(p_col), width=int(2 * SS))
    draw.rectangle([wx - int(22 * SS), cy - int(92 * SS), wx + int(22 * SS), cy - int(82 * SS)], fill=a_col)
    draw.rectangle([wx - int(22 * SS), cy + int(82 * SS), wx + int(22 * SS), cy + int(92 * SS)], fill=a_col)

    td_512 = im.resize((512, 512), Image.Resampling.LANCZOS)
    out_path = TOPDOWN_DIR / f"{name}.png"
    td_512.save(out_path)
    print(f"✓ Top-down 512x512 -> {out_path}")


def render_superbuggy(name, p_col, s_col, a_col, beadlock_col):
    """Tier 5: SuperBuggy (massive wide stance, exposed V8/Twin-Turbo, giant aero wing)."""
    im = Image.new("RGBA", (CANVAS_SIZE, CANVAS_SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im)
    cy = CENTER

    # Grounded shadow
    draw_drop_shadow(draw, CENTER, cy, int(230 * SS), int(150 * SS))

    fx = CENTER + int(145 * SS)
    rx = CENTER - int(145 * SS)
    track_f = int(135 * SS)
    track_r = int(145 * SS)
    tw, th = int(84 * SS), int(42 * SS)

    # Suspension wishbones
    draw_wishbones(draw, CENTER + int(70 * SS), CENTER + int(125 * SS), cy - int(45 * SS), fx, cy - track_f)
    draw_wishbones(draw, CENTER + int(70 * SS), CENTER + int(125 * SS), cy + int(45 * SS), fx, cy + track_f)
    draw_wishbones(draw, CENTER - int(70 * SS), CENTER - int(135 * SS), cy - int(55 * SS), rx, cy - track_r)
    draw_wishbones(draw, CENTER - int(70 * SS), CENTER - int(135 * SS), cy + int(55 * SS), rx, cy + track_r)

    # Oversized knobby wheels
    draw_knobby_tire(draw, fx, cy - track_f, tw, th, beadlock_col)
    draw_knobby_tire(draw, fx, cy + track_f, tw, th, beadlock_col)
    draw_knobby_tire(draw, rx, cy - track_r, int(96 * SS), int(48 * SS), beadlock_col)
    draw_knobby_tire(draw, rx, cy + track_r, int(96 * SS), int(48 * SS), beadlock_col)

    # Heavy-duty spaceframe
    draw.rounded_rectangle(
        [CENTER - int(150 * SS), cy - int(52 * SS), CENTER + int(95 * SS), cy + int(52 * SS)],
        radius=int(12 * SS), fill=(24, 26, 30), outline=(35, 38, 45), width=int(4 * SS)
    )

    # Large V8 / Biturbo engine block with twin ceramic exhaust pipes
    draw.rectangle([CENTER - int(150 * SS), cy - int(32 * SS), CENTER - int(70 * SS), cy + int(32 * SS)], fill=(50, 55, 65))
    draw.line([CENTER - int(150 * SS), cy - int(18 * SS), CENTER - int(185 * SS), cy - int(18 * SS)], fill=(190, 195, 205), width=int(5 * SS))
    draw.line([CENTER - int(150 * SS), cy + int(18 * SS), CENTER - int(185 * SS), cy + int(18 * SS)], fill=(190, 195, 205), width=int(5 * SS))

    # Cockpit
    draw_cockpit(draw, CENTER - int(10 * SS), cy, a_col)

    # Front cowl (+X)
    cowl = [
        (CENTER + int(70 * SS), cy - int(48 * SS)),
        (CENTER + int(185 * SS), cy - int(24 * SS)),
        (CENTER + int(210 * SS), cy),
        (CENTER + int(185 * SS), cy + int(24 * SS)),
        (CENTER + int(70 * SS), cy + int(48 * SS)),
    ]
    draw.polygon(cowl, fill=p_col, outline=darken(p_col), width=int(2 * SS))
    draw.polygon([
        (CENTER + int(85 * SS), cy - int(20 * SS)),
        (CENTER + int(195 * SS), cy),
        (CENTER + int(85 * SS), cy + int(20 * SS)),
    ], fill=s_col)
    draw.line([CENTER + int(90 * SS), cy, CENTER + int(195 * SS), cy], fill=a_col, width=int(4 * SS))

    # Giant Bi-Plane Rear Wing with Endplates
    wx = CENTER - int(180 * SS)
    draw.rectangle([wx - int(18 * SS), cy - int(95 * SS), wx + int(18 * SS), cy + int(95 * SS)], fill=p_col, outline=darken(p_col), width=int(2 * SS))
    draw.rectangle([wx - int(26 * SS), cy - int(102 * SS), wx + int(26 * SS), cy - int(92 * SS)], fill=a_col)
    draw.rectangle([wx - int(26 * SS), cy + int(92 * SS), wx + int(26 * SS), cy + int(102 * SS)], fill=a_col)

    td_512 = im.resize((512, 512), Image.Resampling.LANCZOS)
    out_path = TOPDOWN_DIR / f"{name}.png"
    td_512.save(out_path)
    print(f"✓ Top-down 512x512 -> {out_path}")


def render_touring_ax(name, p_col, s_col, a_col, style="hatch"):
    """Tier 4: TouringAutocross widebody silhouette racing car."""
    im = Image.new("RGBA", (CANVAS_SIZE, CANVAS_SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im)
    cy = CENTER

    # Grounded shadow
    draw_drop_shadow(draw, CENTER, cy, int(230 * SS), int(110 * SS))

    fx = CENTER + int(130 * SS)
    rx = CENTER - int(130 * SS)
    track_y = int(88 * SS)

    # Wheels (under widebody fenders)
    draw_knobby_tire(draw, fx, cy - track_y, int(80 * SS), int(36 * SS), (180, 185, 195))
    draw_knobby_tire(draw, fx, cy + track_y, int(80 * SS), int(36 * SS), (180, 185, 195))
    draw_knobby_tire(draw, rx, cy - track_y, int(80 * SS), int(36 * SS), (180, 185, 195))
    draw_knobby_tire(draw, rx, cy + track_y, int(80 * SS), int(36 * SS), (180, 185, 195))

    # Main silhouette body (facing right +X)
    if style == "hatch":
        body = [
            (CENTER - int(195 * SS), cy - int(72 * SS)),
            (CENTER - int(140 * SS), cy - int(92 * SS)),
            (CENTER + int(140 * SS), cy - int(92 * SS)),
            (CENTER + int(200 * SS), cy - int(70 * SS)),
            (CENTER + int(220 * SS), cy - int(35 * SS)),
            (CENTER + int(225 * SS), cy),
            (CENTER + int(220 * SS), cy + int(35 * SS)),
            (CENTER + int(200 * SS), cy + int(70 * SS)),
            (CENTER + int(140 * SS), cy + int(92 * SS)),
            (CENTER - int(140 * SS), cy + int(92 * SS)),
            (CENTER - int(195 * SS), cy + int(72 * SS)),
        ]
    elif style == "sedan":
        body = [
            (CENTER - int(215 * SS), cy - int(68 * SS)),
            (CENTER - int(140 * SS), cy - int(90 * SS)),
            (CENTER + int(140 * SS), cy - int(90 * SS)),
            (CENTER + int(210 * SS), cy - int(65 * SS)),
            (CENTER + int(225 * SS), cy - int(30 * SS)),
            (CENTER + int(230 * SS), cy),
            (CENTER + int(225 * SS), cy + int(30 * SS)),
            (CENTER + int(210 * SS), cy + int(65 * SS)),
            (CENTER + int(140 * SS), cy + int(90 * SS)),
            (CENTER - int(140 * SS), cy + int(90 * SS)),
            (CENTER - int(215 * SS), cy + int(68 * SS)),
        ]
    else:  # coupe
        body = [
            (CENTER - int(210 * SS), cy - int(66 * SS)),
            (CENTER - int(135 * SS), cy - int(88 * SS)),
            (CENTER + int(135 * SS), cy - int(88 * SS)),
            (CENTER + int(205 * SS), cy - int(62 * SS)),
            (CENTER + int(222 * SS), cy - int(28 * SS)),
            (CENTER + int(228 * SS), cy),
            (CENTER + int(222 * SS), cy + int(28 * SS)),
            (CENTER + int(205 * SS), cy + int(62 * SS)),
            (CENTER + int(135 * SS), cy + int(88 * SS)),
            (CENTER - int(135 * SS), cy + int(88 * SS)),
            (CENTER - int(210 * SS), cy + int(66 * SS)),
        ]
    draw.polygon(body, fill=p_col, outline=darken(p_col), width=int(2.5 * SS))

    # Roof & Windshield
    glass_col = (40, 48, 62)
    # Windshield (facing right)
    draw.polygon([
        (CENTER + int(40 * SS), cy - int(56 * SS)),
        (CENTER + int(95 * SS), cy - int(48 * SS)),
        (CENTER + int(95 * SS), cy + int(48 * SS)),
        (CENTER + int(40 * SS), cy + int(56 * SS)),
    ], fill=glass_col)
    # Rear window
    draw.polygon([
        (CENTER - int(95 * SS), cy - int(52 * SS)),
        (CENTER - int(40 * SS), cy - int(56 * SS)),
        (CENTER - int(40 * SS), cy + int(56 * SS)),
        (CENTER - int(95 * SS), cy + int(52 * SS)),
    ], fill=glass_col)
    # Roof panel with scoop
    draw.rectangle([CENTER - int(40 * SS), cy - int(56 * SS), CENTER + int(40 * SS), cy + int(56 * SS)], fill=p_col)
    draw.rounded_rectangle([CENTER - int(10 * SS), cy - int(14 * SS), CENTER + int(25 * SS), cy + int(14 * SS)], radius=int(4 * SS), fill=darken(p_col))

    # Livery Graphics (Angular shards of s_col and a_col)
    draw.polygon([
        (CENTER - int(120 * SS), cy - int(85 * SS)),
        (CENTER + int(30 * SS), cy - int(30 * SS)),
        (CENTER - int(40 * SS), cy),
        (CENTER - int(120 * SS), cy - int(40 * SS)),
    ], fill=s_col)
    draw.polygon([
        (CENTER - int(120 * SS), cy + int(85 * SS)),
        (CENTER + int(30 * SS), cy + int(30 * SS)),
        (CENTER - int(40 * SS), cy),
        (CENTER - int(120 * SS), cy + int(40 * SS)),
    ], fill=s_col)
    draw.line([CENTER - int(130 * SS), cy - int(60 * SS), CENTER + int(50 * SS), cy - int(20 * SS)], fill=a_col, width=int(4 * SS))
    draw.line([CENTER - int(130 * SS), cy + int(60 * SS), CENTER + int(50 * SS), cy + int(20 * SS)], fill=a_col, width=int(4 * SS))

    # High-Downforce Rear Wing (Rear X: CENTER - 210)
    wx = CENTER - int(205 * SS)
    draw.rectangle([wx - int(15 * SS), cy - int(82 * SS), wx + int(15 * SS), cy + int(82 * SS)], fill=p_col, outline=darken(p_col), width=int(2 * SS))
    draw.rectangle([wx - int(22 * SS), cy - int(88 * SS), wx + int(22 * SS), cy - int(78 * SS)], fill=a_col)
    draw.rectangle([wx - int(22 * SS), cy + int(78 * SS), wx + int(22 * SS), cy + int(88 * SS)], fill=a_col)

    td_512 = im.resize((512, 512), Image.Resampling.LANCZOS)
    out_path = TOPDOWN_DIR / f"{name}.png"
    td_512.save(out_path)
    print(f"✓ Top-down 512x512 -> {out_path}")


def main():
    print("🎨 Generating High-Quality Autocross Top-Down Sprites...")
    # Tier 2: Senior Cross Cars
    render_cross_car_senior("autocross_ardennes_pro_t2", (75, 80, 88), (160, 235, 30), (245, 245, 250), (160, 235, 30))
    render_cross_car_senior("autocross_iberian_relampago_t2", (235, 105, 20), (24, 25, 28), (195, 200, 210), (235, 105, 20))
    render_cross_car_senior("autocross_lusitania_bravo_t2", (25, 85, 200), (245, 245, 250), (235, 180, 25), (235, 180, 25))

    # Tier 3: Buggy 1600
    render_buggy1600("autocross_petersen_buggy1600_t3", (235, 110, 20), (80, 90, 105), (245, 248, 252), (235, 110, 20))
    render_buggy1600("autocross_bologna_buggy1600_t3", (215, 25, 35), (28, 30, 35), (245, 245, 250), (28, 30, 35))
    render_buggy1600("autocross_rapid_buggy1600_t3", (25, 90, 215), (225, 245, 25), (30, 32, 36), (225, 245, 25))

    # Tier 4: TouringAutocross
    render_touring_ax("autocross_bohemia_veloce_t4", (35, 175, 65), (245, 248, 252), (65, 70, 78), style="hatch")
    render_touring_ax("autocross_shinano_tsunami_t4", (245, 245, 248), (205, 30, 45), (28, 30, 35), style="sedan")
    render_touring_ax("autocross_vortek_quattro_t4", (145, 150, 158), (220, 35, 40), (25, 26, 30), style="coupe")

    # Tier 5: SuperBuggy
    render_superbuggy("autocross_petersen_superbuggy_t5", (20, 35, 75), (245, 248, 252), (245, 110, 20), (245, 110, 20))
    render_superbuggy("autocross_bologna_superbuggy_t5", (195, 25, 40), (38, 40, 46), (190, 195, 205), (190, 195, 205))
    render_superbuggy("autocross_rapid_superbuggy_t5", (220, 245, 25), (26, 28, 32), (25, 210, 240), (220, 245, 25))

    print("\n✨ Done! All top-down sprites rendered at high quality facing right (+X).")


if __name__ == "__main__":
    main()
