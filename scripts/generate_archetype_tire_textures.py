#!/usr/bin/env python3
"""
TdRace - Dedicated High-Fidelity Archetype Tire Texture Generator (Spec 095)

Generates missing high-fidelity 128x256 top-down steered wheel assets for:
- buggy_allterrain_front.png (Baja buggies, Sand rails, Cross cars)
- truck_allterrain_front.png (Trophy trucks, Stadium pickups)
- monster_wheel_front.png   (66" Terra chevron monster trucks)
- mud_tractor_front.png     (Deep-lug directional mud paddle boggers)
"""

import math
from pathlib import Path
import numpy as np
from PIL import Image, ImageDraw


def draw_circle(draw: ImageDraw.ImageDraw, x: float, y: float, r: float, **kwargs):
    draw.ellipse([x - r, y - r, x + r, y + r], **kwargs)


def clip_to_carcass(im: Image.Image, cx: int, cy: int, tw_half: int, th_half: int, corner_r: int):
    """Clips all drawn elements strictly inside the rounded-rectangle tire carcass boundary."""
    carcass_mask = Image.new("L", im.size, 0)
    d_mask = ImageDraw.Draw(carcass_mask)
    d_mask.rounded_rectangle([cx - tw_half, cy - th_half, cx + tw_half, cy + th_half], radius=corner_r, fill=255)

    alpha = np.array(im.getchannel("A"))
    mask_arr = np.array(carcass_mask)
    clipped_alpha = np.minimum(alpha, mask_arr)
    im.putalpha(Image.fromarray(clipped_alpha))


def generate_buggy_wheel(out_path: Path):
    ss = 2
    W, H = 128 * ss, 256 * ss
    im = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im)
    cx, cy = W // 2, H // 2
    tw_half, th_half = int(52 * ss), int(116 * ss)
    corner_r = int(22 * ss)

    # 1. Outer tire body (dark vulcanized rubber)
    draw.rounded_rectangle(
        [cx - tw_half, cy - th_half, cx + tw_half, cy + th_half],
        radius=corner_r,
        fill=(36, 40, 46, 255),
    )
    # 2. Main tread block base
    im_m = int(3 * ss)
    draw.rounded_rectangle(
        [cx - tw_half + im_m, cy - th_half + im_m, cx + tw_half - im_m, cy + th_half - im_m],
        radius=corner_r - int(2 * ss),
        fill=(22, 24, 28, 255),
    )

    # 3. Off-road directional knobby side lugs & center ribs
    for y_pos in range(cy - th_half + int(14 * ss), cy + th_half - int(16 * ss), int(16 * ss)):
        # Left shoulder lug
        draw.rounded_rectangle(
            [cx - tw_half + int(4 * ss), y_pos, cx - tw_half + int(16 * ss), y_pos + int(8 * ss)],
            radius=int(2 * ss),
            fill=(14, 16, 18, 255),
            outline=(45, 50, 58, 255),
            width=int(1 * ss),
        )
        # Right shoulder lug
        draw.rounded_rectangle(
            [cx + tw_half - int(16 * ss), y_pos + int(8 * ss), cx + tw_half - int(4 * ss), y_pos + int(16 * ss)],
            radius=int(2 * ss),
            fill=(14, 16, 18, 255),
            outline=(45, 50, 58, 255),
            width=int(1 * ss),
        )

    # Center steering groove
    draw.line([cx, cy - th_half + int(18 * ss), cx, cy + th_half - int(18 * ss)], fill=(12, 14, 16, 255), width=int(4 * ss))

    # 4. Beadlock alloy rim (anthracite with silver machined beadlock ring)
    rw_half, rh_half = int(22 * ss), int(60 * ss)
    draw.rounded_rectangle(
        [cx - rw_half, cy - rh_half, cx + rw_half, cy + rh_half],
        radius=int(8 * ss),
        fill=(195, 200, 210, 255),
        outline=(150, 155, 165, 255),
        width=int(2 * ss),
    )
    # Inner rim well
    draw.rounded_rectangle(
        [cx - rw_half + int(4 * ss), cy - rh_half + int(4 * ss), cx + rw_half - int(4 * ss), cy + rh_half - int(4 * ss)],
        radius=int(6 * ss),
        fill=(42, 46, 54, 255),
    )

    # Beadlock bolts along the outer rim edge
    for dy in range(-int(52 * ss), int(54 * ss), int(12 * ss)):
        draw_circle(draw, cx - rw_half + int(2 * ss), cy + dy, int(1.5 * ss), fill=(240, 245, 255, 255))
        draw_circle(draw, cx + rw_half - int(2 * ss), cy + dy, int(1.5 * ss), fill=(240, 245, 255, 255))

    # 5. Center spindle hub & 4-bolt pattern
    draw_circle(draw, cx, cy, int(9 * ss), fill=(30, 33, 40, 255), outline=(70, 75, 85, 255), width=int(1.5 * ss))
    draw_circle(draw, cx, cy, int(3.5 * ss), fill=(220, 175, 45, 255))  # Gold center nut

    for i in range(4):
        ang = i * (math.pi / 2) + (math.pi / 4)
        lx = cx + math.cos(ang) * (6.0 * ss)
        ly = cy + math.sin(ang) * (6.0 * ss)
        draw_circle(draw, lx, ly, int(2 * ss), fill=(225, 230, 240, 255))

    clip_to_carcass(im, cx, cy, tw_half, th_half, corner_r)
    out_img = im.resize((128, 256), Image.Resampling.LANCZOS)
    out_img.save(out_path)
    print(f"✓ Generated buggy_allterrain_front: {out_path}")


def generate_truck_wheel(out_path: Path):
    ss = 2
    W, H = 128 * ss, 256 * ss
    im = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im)
    cx, cy = W // 2, H // 2
    tw_half, th_half = int(55 * ss), int(118 * ss)
    corner_r = int(24 * ss)

    # 1. Outer tire (rugged high-traction rubber compound)
    draw.rounded_rectangle(
        [cx - tw_half, cy - th_half, cx + tw_half, cy + th_half],
        radius=corner_r,
        fill=(38, 42, 48, 255),
    )
    # 2. Main tread base
    im_m = int(3 * ss)
    draw.rounded_rectangle(
        [cx - tw_half + im_m, cy - th_half + im_m, cx + tw_half - im_m, cy + th_half - im_m],
        radius=corner_r - int(2 * ss),
        fill=(20, 22, 26, 255),
    )

    # 3. Aggressive interlocking all-terrain tread blocks & shoulder sipes
    for y_pos in range(cy - th_half + int(14 * ss), cy + th_half - int(20 * ss), int(20 * ss)):
        # Heavy shoulder cleats (left & right)
        draw.rounded_rectangle(
            [cx - tw_half + int(3 * ss), y_pos, cx - tw_half + int(18 * ss), y_pos + int(12 * ss)],
            radius=int(3 * ss),
            fill=(15, 17, 20, 255),
            outline=(48, 54, 62, 255),
            width=int(1.5 * ss),
        )
        draw.rounded_rectangle(
            [cx + tw_half - int(18 * ss), y_pos + int(10 * ss), cx + tw_half - int(3 * ss), y_pos + int(22 * ss)],
            radius=int(3 * ss),
            fill=(15, 17, 20, 255),
            outline=(48, 54, 62, 255),
            width=int(1.5 * ss),
        )

        # Center interlocking tread blocks
        draw.polygon(
            [
                (cx - int(7 * ss), y_pos + int(4 * ss)),
                (cx + int(7 * ss), y_pos + int(8 * ss)),
                (cx + int(7 * ss), y_pos + int(14 * ss)),
                (cx - int(7 * ss), y_pos + int(10 * ss)),
            ],
            fill=(26, 29, 34, 255),
            outline=(12, 14, 16, 255),
        )

    # 4. Heavy forged beadlock rim (satin black with neon green / gold beadlock ring)
    rw_half, rh_half = int(24 * ss), int(64 * ss)
    draw.rounded_rectangle(
        [cx - rw_half, cy - rh_half, cx + rw_half, cy + rh_half],
        radius=int(10 * ss),
        fill=(45, 185, 80, 255),  # High-visibility trophy truck beadlock ring
        outline=(30, 130, 55, 255),
        width=int(2 * ss),
    )
    # Recessed rim face
    draw.rounded_rectangle(
        [cx - rw_half + int(4 * ss), cy - rh_half + int(4 * ss), cx + rw_half - int(4 * ss), cy + rh_half - int(4 * ss)],
        radius=int(7 * ss),
        fill=(28, 30, 36, 255),
    )

    # 16 Beadlock grade-8 perimeter bolts
    for dy in range(-int(56 * ss), int(58 * ss), int(10 * ss)):
        draw_circle(draw, cx - rw_half + int(2 * ss), cy + dy, int(1.5 * ss), fill=(240, 245, 255, 255))
        draw_circle(draw, cx + rw_half - int(2 * ss), cy + dy, int(1.5 * ss), fill=(240, 245, 255, 255))

    # 5. Heavy 6-lug truck hub
    draw_circle(draw, cx, cy, int(11 * ss), fill=(38, 42, 50, 255), outline=(75, 82, 95, 255), width=int(1.5 * ss))
    for i in range(6):
        ang = i * (math.pi / 3)
        lx = cx + math.cos(ang) * (7.0 * ss)
        ly = cy + math.sin(ang) * (7.0 * ss)
        draw_circle(draw, lx, ly, int(2.2 * ss), fill=(230, 235, 245, 255), outline=(100, 105, 115, 255), width=int(1 * ss))
    draw_circle(draw, cx, cy, int(4 * ss), fill=(20, 22, 26, 255))

    clip_to_carcass(im, cx, cy, tw_half, th_half, corner_r)
    out_img = im.resize((128, 256), Image.Resampling.LANCZOS)
    out_img.save(out_path)
    print(f"✓ Generated truck_allterrain_front: {out_path}")


def generate_monster_wheel(out_path: Path):
    ss = 2
    W, H = 128 * ss, 256 * ss
    im = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im)
    cx, cy = W // 2, H // 2
    tw_half, th_half = int(58 * ss), int(120 * ss)
    corner_r = int(26 * ss)

    # 1. Massive 66" Terra tire base
    draw.rounded_rectangle(
        [cx - tw_half, cy - th_half, cx + tw_half, cy + th_half],
        radius=corner_r,
        fill=(40, 44, 52, 255),
    )
    im_m = int(4 * ss)
    draw.rounded_rectangle(
        [cx - tw_half + im_m, cy - th_half + im_m, cx + tw_half - im_m, cy + th_half - im_m],
        radius=corner_r - int(2 * ss),
        fill=(22, 25, 30, 255),
    )

    # 2. Iconic 45° directional chevron V-lugs (agricultural high-traction cleats)
    for y_pos in range(cy - th_half + int(10 * ss), cy + th_half - int(26 * ss), int(26 * ss)):
        # Left chevron bar angled forward toward center
        pts_left = [
            (cx - tw_half + int(3 * ss), y_pos),
            (cx - int(2 * ss), y_pos + int(14 * ss)),
            (cx - int(2 * ss), y_pos + int(24 * ss)),
            (cx - tw_half + int(3 * ss), y_pos + int(10 * ss)),
        ]
        draw.polygon(pts_left, fill=(14, 16, 19, 255), outline=(52, 58, 68, 255))

        # Right chevron bar angled forward toward center (staggered slightly)
        pts_right = [
            (cx + tw_half - int(3 * ss), y_pos + int(12 * ss)),
            (cx + int(2 * ss), y_pos + int(26 * ss)),
            (cx + int(2 * ss), y_pos + int(36 * ss)),
            (cx + tw_half - int(3 * ss), y_pos + int(22 * ss)),
        ]
        draw.polygon(pts_right, fill=(14, 16, 19, 255), outline=(52, 58, 68, 255))

    # Center mud trench groove
    draw.line([cx, cy - th_half + int(12 * ss), cx, cy + th_half - int(12 * ss)], fill=(10, 12, 14, 255), width=int(5 * ss))

    # 3. Deep-dish steel rim (bright red/orange monster dish with heavy bead ring)
    rw_half, rh_half = int(26 * ss), int(68 * ss)
    draw.rounded_rectangle(
        [cx - rw_half, cy - rh_half, cx + rw_half, cy + rh_half],
        radius=int(12 * ss),
        fill=(215, 40, 30, 255),  # Fiery competition red rim
        outline=(150, 25, 20, 255),
        width=int(2.5 * ss),
    )
    # Recessed dark dish center
    draw.rounded_rectangle(
        [cx - rw_half + int(5 * ss), cy - rh_half + int(5 * ss), cx + rw_half - int(5 * ss), cy + rh_half - int(5 * ss)],
        radius=int(8 * ss),
        fill=(32, 35, 42, 255),
    )

    # 4. Heavy monster planetary hub
    draw_circle(draw, cx, cy, int(15 * ss), fill=(55, 60, 72, 255), outline=(95, 105, 120, 255), width=int(2 * ss))
    draw_circle(draw, cx, cy, int(7 * ss), fill=(195, 30, 25, 255), outline=(130, 20, 15, 255), width=int(1.5 * ss))
    draw_circle(draw, cx, cy, int(3 * ss), fill=(245, 245, 255, 255))

    # Heavy perimeter planetary studs (8 studs)
    for i in range(8):
        ang = i * (math.pi / 4)
        lx = cx + math.cos(ang) * (11.0 * ss)
        ly = cy + math.sin(ang) * (11.0 * ss)
        draw_circle(draw, lx, ly, int(2.2 * ss), fill=(225, 230, 240, 255))

    clip_to_carcass(im, cx, cy, tw_half, th_half, corner_r)
    out_img = im.resize((128, 256), Image.Resampling.LANCZOS)
    out_img.save(out_path)
    print(f"✓ Generated monster_wheel_front: {out_path}")


def generate_mud_tractor_wheel(out_path: Path):
    ss = 2
    W, H = 128 * ss, 256 * ss
    im = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im)
    cx, cy = W // 2, H // 2
    tw_half, th_half = int(56 * ss), int(119 * ss)
    corner_r = int(24 * ss)

    # 1. Mud tractor carcass
    draw.rounded_rectangle(
        [cx - tw_half, cy - th_half, cx + tw_half, cy + th_half],
        radius=corner_r,
        fill=(38, 42, 48, 255),
    )
    im_m = int(3 * ss)
    draw.rounded_rectangle(
        [cx - tw_half + im_m, cy - th_half + im_m, cx + tw_half - im_m, cy + th_half - im_m],
        radius=corner_r - int(2 * ss),
        fill=(20, 22, 26, 255),
    )

    # 2. Deep-lug directional tractor paddle blades (churns trenches)
    for y_pos in range(cy - th_half + int(8 * ss), cy + th_half - int(24 * ss), int(24 * ss)):
        # Deep angled paddle bar extending outward past shoulder
        pts_left = [
            (cx - tw_half - int(2 * ss), y_pos),
            (cx - int(4 * ss), y_pos + int(10 * ss)),
            (cx - int(4 * ss), y_pos + int(18 * ss)),
            (cx - tw_half - int(2 * ss), y_pos + int(8 * ss)),
        ]
        draw.polygon(pts_left, fill=(12, 14, 16, 255), outline=(48, 54, 62, 255))

        pts_right = [
            (cx + tw_half + int(2 * ss), y_pos + int(12 * ss)),
            (cx + int(4 * ss), y_pos + int(22 * ss)),
            (cx + int(4 * ss), y_pos + int(30 * ss)),
            (cx + tw_half + int(2 * ss), y_pos + int(20 * ss)),
        ]
        draw.polygon(pts_right, fill=(12, 14, 16, 255), outline=(48, 54, 62, 255))

    # Center mud channel
    draw.line([cx, cy - th_half + int(14 * ss), cx, cy + th_half - int(14 * ss)], fill=(8, 10, 12, 255), width=int(6 * ss))

    # 3. Military Rockwell steel wheel rim (industrial yellow / olive drab)
    rw_half, rh_half = int(24 * ss), int(64 * ss)
    draw.rounded_rectangle(
        [cx - rw_half, cy - rh_half, cx + rw_half, cy + rh_half],
        radius=int(10 * ss),
        fill=(205, 165, 30, 255),  # Industrial caution yellow
        outline=(140, 110, 20, 255),
        width=int(2 * ss),
    )
    # Recessed steel well
    draw.rounded_rectangle(
        [cx - rw_half + int(4 * ss), cy - rh_half + int(4 * ss), cx + rw_half - int(4 * ss), cy + rh_half - int(4 * ss)],
        radius=int(7 * ss),
        fill=(30, 32, 38, 255),
    )

    # 4. Military 10-lug heavy pinion hub
    draw_circle(draw, cx, cy, int(13 * ss), fill=(48, 52, 60, 255), outline=(80, 88, 100, 255), width=int(1.5 * ss))
    draw_circle(draw, cx, cy, int(5 * ss), fill=(18, 20, 24, 255))

    for i in range(8):
        ang = i * (math.pi / 4)
        lx = cx + math.cos(ang) * (9.0 * ss)
        ly = cy + math.sin(ang) * (9.0 * ss)
        draw_circle(draw, lx, ly, int(2 * ss), fill=(240, 242, 250, 255))

    clip_to_carcass(im, cx, cy, tw_half, th_half, corner_r)
    out_img = im.resize((128, 256), Image.Resampling.LANCZOS)
    out_img.save(out_path)
    print(f"✓ Generated mud_tractor_front: {out_path}")


def main():
    root = Path(__file__).resolve().parent.parent
    wheels_dir = root / "assets" / "textures" / "vehicles" / "topdown" / "wheels"
    wheels_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 70)
    print("🎨 TdRace Spec 095: Generating Dedicated Archetype Tire Textures")
    print("=" * 70)

    generate_buggy_wheel(wheels_dir / "buggy_allterrain_front.png")
    generate_truck_wheel(wheels_dir / "truck_allterrain_front.png")
    generate_monster_wheel(wheels_dir / "monster_wheel_front.png")
    generate_mud_tractor_wheel(wheels_dir / "mud_tractor_front.png")

    print("\n✅ All 4 dedicated archetype tire textures generated successfully.")


if __name__ == "__main__":
    main()
