#!/usr/bin/env python3
"""
Process raw photorealistic generated images into game-ready assets:
1. Chroma-key pure magenta background (#FF00FF)
2. Lateral: 1024x512 canvas (ground-aligned) + 256x128 thumbnail
3. Top-down: 512x512 canvas (centered, facing right +X)
4. Chassis cutout: front wheels cleanly erased for modular Ackermann steering
"""

import sys
from pathlib import Path

import numpy as np
from PIL import Image


def remove_magenta_bg(img: Image.Image, is_lateral: bool = False) -> Image.Image:
    arr = np.array(img.convert("RGBA"))
    r = arr[:, :, 0].astype(float)
    g = arr[:, :, 1].astype(float)
    b = arr[:, :, 2].astype(float)

    # Magenta chroma key: High R, High B, Low G
    is_pink = (r > 150) & (b > 150) & (g < 100)
    is_fringe = (r > 115) & (b > 115) & ((r + b) > 2.2 * np.maximum(g, 1.0))
    arr[is_pink | is_fringe, 3] = 0

    if is_lateral:
        # Detect flat ground shadow line in the bottom 30% of the image
        h = arr.shape[0]
        start_y = int(h * 0.70)
        row_counts = [np.sum(arr[y, :, 3] > 0) for y in range(start_y, h)]
        for i in range(1, len(row_counts)):
            # If row pixel count suddenly surges (> 40% jump) into a horizontal ground band
            if row_counts[i] > 650 and row_counts[i] > row_counts[i - 1] * 1.3:
                cut_y = start_y + i
                arr[cut_y:, :, 3] = 0
                break

    alpha = arr[:, :, 3]
    ys, xs = np.where(alpha > 0)
    if len(ys) == 0 or len(xs) == 0:
        return Image.fromarray(arr)

    # Neutralize any residual magenta bleed through glass windows into dark cockpit tint
    # Only pixels where green is significantly lower than both red and blue are magenta
    mag_bleed = (r > 80) & (b > 80) & (g < 0.7 * np.minimum(r, b)) & (arr[:, :, 3] > 0)
    arr[mag_bleed, 0] = np.clip(g[mag_bleed] * 0.9, 32, 60).astype(np.uint8)
    arr[mag_bleed, 1] = np.clip(g[mag_bleed] * 1.0, 36, 65).astype(np.uint8)
    arr[mag_bleed, 2] = np.clip(g[mag_bleed] * 1.1, 40, 75).astype(np.uint8)

    min_y, max_y = ys.min(), ys.max()
    min_x, max_x = xs.min(), xs.max()

    return Image.fromarray(arr[min_y:max_y+1, min_x:max_x+1])

def process_lateral(raw_path: Path, out_path: Path, thumb_path: Path):
    im = Image.open(raw_path)
    cropped = remove_magenta_bg(im, is_lateral=True)
    cw, ch = cropped.size

    canvas = Image.new("RGBA", (1024, 512), (0, 0, 0, 0))
    scale = min(960 / cw, 420 / ch)
    nw, nh = int(cw * scale), int(ch * scale)
    scaled = cropped.resize((nw, nh), Image.Resampling.LANCZOS)
    canvas.paste(scaled, ((1024 - nw) // 2, 512 - nh - 20))

    out_path.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(out_path, format="PNG")

    thumb = canvas.resize((256, 128), Image.Resampling.LANCZOS)
    thumb_path.parent.mkdir(parents=True, exist_ok=True)
    thumb.save(thumb_path, format="PNG")
    print(f"✓ Lateral (1024x512) -> {out_path}")
    print(f"✓ Thumbnail (256x128) -> {thumb_path}")

def process_topdown(raw_path: Path, out_topdown: Path, out_chassis: Path, is_open_wheel: bool = True, flip_h: bool = False):
    im = Image.open(raw_path)
    cropped = remove_magenta_bg(im)
    if flip_h:
        cropped = cropped.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
    cw, ch = cropped.size

    canvas = Image.new("RGBA", (512, 512), (0, 0, 0, 0))
    scale = min(460 / cw, 320 / ch)
    nw, nh = int(cw * scale), int(ch * scale)
    scaled = cropped.resize((nw, nh), Image.Resampling.LANCZOS)
    canvas.paste(scaled, ((512 - nw) // 2, (512 - nh) // 2))

    out_topdown.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(out_topdown, format="PNG")
    print(f"✓ Topdown (512x512) -> {out_topdown}")

    # Generate chassis with front wheels erased
    chassis_arr = np.array(canvas)
    # The car is facing right (+X forward).
    # Front wheels are in the right half of the image (x > 256).
    # Specifically for front wheels:
    # Top front wheel: y < 200, x > 300
    # Bottom front wheel: y > 312, x > 300
    if is_open_wheel:
        if model_id == "classic_kart_vintage":
            chassis_arr[108:180, 350:450, 3] = 0
            chassis_arr[332:404, 350:450, 3] = 0
        else: # classic_ax_mudlark
            chassis_arr[105:165, 365:485, 3] = 0
            chassis_arr[345:405, 365:485, 3] = 0
    else:
        # Closed wheel: hollow out front fender wells
        chassis_arr[40:170, 330:480, 3] = 0
        chassis_arr[342:472, 330:480, 3] = 0

    chassis_img = Image.fromarray(chassis_arr)
    chassis_img.save(out_chassis, format="PNG")
    print(f"✓ Chassis (512x512) -> {out_chassis}")

if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: process_classic_asset.py <model_id> <raw_lateral> <raw_topdown> [--open-wheel]")
        sys.exit(1)

    model_id = sys.argv[1]
    raw_lat = Path(sys.argv[2])
    raw_top = Path(sys.argv[3])
    is_open = "--open-wheel" in sys.argv
    flip_h = "--flip-h" in sys.argv

    root = Path(__file__).resolve().parent.parent
    lat_out = root / "assets" / "textures" / "vehicles" / "laterals" / "classic" / f"{model_id}.png"
    thumb_out = root / "assets" / "textures" / "vehicles" / "laterals" / "classic" / f"{model_id}_thumb.png"
    top_out = root / "assets" / "textures" / "vehicles" / "topdown" / "classic" / f"{model_id}.png"
    chassis_out = root / "assets" / "textures" / "vehicles" / "topdown" / "classic" / f"{model_id}_chassis.png"

    process_lateral(raw_lat, lat_out, thumb_out)
    process_topdown(raw_top, top_out, chassis_out, is_open_wheel=is_open, flip_h=flip_h)
