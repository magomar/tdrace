#!/usr/bin/env python3
"""
Process raw generated lateral and top-down vehicle images:
- Chroma-key out pure magenta background (#FF00FF)
- Crop tightly to car bounding box
- Place in 1024x512 transparent canvas for lateral view (aligned to ground)
- Downscale to 256x85 for menu thumbnails
- Place in 512x512 transparent canvas for top-down racing view (centered)
"""

import argparse
from pathlib import Path
import numpy as np
from PIL import Image

def remove_magenta_bg(img: Image.Image, is_topdown: bool = False) -> Image.Image:
    arr = np.array(img.convert("RGBA"))
    r = arr[:, :, 0].astype(float)
    g = arr[:, :, 1].astype(float)
    b = arr[:, :, 2].astype(float)

    is_pink = (r > 160) & (b > 160) & (g < 100)
    is_fringe = (r > 120) & (b > 120) & ((r + b) > 2.2 * np.maximum(g, 1.0))
    arr[is_pink | is_fringe, 3] = 0

    alpha = arr[:, :, 3]
    ys, xs = np.where(alpha > 0)
    if len(ys) == 0 or len(xs) == 0:
        return Image.fromarray(arr)

    min_y, max_y = ys.min(), ys.max()
    min_x, max_x = xs.min(), xs.max()

    return Image.fromarray(arr[min_y:max_y+1, min_x:max_x+1])

def process_lateral(raw_path: Path, out_path: Path, thumb_path: Path):
    im = Image.open(raw_path)
    cropped = remove_magenta_bg(im, is_topdown=False)
    cw, ch = cropped.size

    canvas = Image.new("RGBA", (1024, 512), (0, 0, 0, 0))
    scale = min(1000 / cw, 440 / ch)
    nw, nh = int(cw * scale), int(ch * scale)
    scaled = cropped.resize((nw, nh), Image.Resampling.LANCZOS)
    canvas.paste(scaled, ((1024 - nw) // 2, 512 - nh - 12))
    
    out_path.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(out_path, format="PNG")

    thumb = canvas.resize((256, 85), Image.Resampling.LANCZOS)
    thumb_path.parent.mkdir(parents=True, exist_ok=True)
    thumb.save(thumb_path, format="PNG")
    print(f"✓ Lateral (1024x512) -> {out_path}")
    print(f"✓ Thumbnail (256x85) -> {thumb_path}")

def process_topdown(raw_path: Path, out_path: Path):
    im = Image.open(raw_path)
    cropped = remove_magenta_bg(im, is_topdown=True)
    cw, ch = cropped.size

    canvas = Image.new("RGBA", (512, 512), (0, 0, 0, 0))
    scale = min(440 / cw, 490 / ch)
    nw, nh = int(cw * scale), int(ch * scale)
    scaled = cropped.resize((nw, nh), Image.Resampling.LANCZOS)
    canvas.paste(scaled, ((512 - nw) // 2, (512 - nh) // 2))

    # Rotate 90 degrees clockwise (ROTATE_270 in PIL) so vehicle nose points to the RIGHT (+X, forward heading)
    canvas = canvas.transpose(Image.Transpose.ROTATE_270)

    out_path.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(out_path, format="PNG")
    print(f"✓ Top-down (512x512, facing right) -> {out_path}")

def main():
    parser = argparse.ArgumentParser(description="Process vehicle sprites")
    parser.add_argument("--module", required=True, help="Module ID (gt, nascar, rally, extreme_offroad, kart)")
    parser.add_argument("--vehicle-id", required=True, help="Vehicle ID")
    parser.add_argument("--lateral-raw", type=Path, help="Path to raw lateral image")
    parser.add_argument("--topdown-raw", type=Path, help="Path to raw topdown image")

    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent

    lat_out = root / "assets" / "textures" / "vehicles" / "laterals" / args.module / f"{args.vehicle_id}.png"
    thumb_out = root / "assets" / "textures" / "vehicles" / "laterals" / args.module / f"{args.vehicle_id}_thumb.png"
    top_out = root / "assets" / "textures" / "vehicles" / "topdown" / args.module / f"{args.vehicle_id}.png"

    if args.lateral_raw:
        process_lateral(args.lateral_raw, lat_out, thumb_out)
    if args.topdown_raw:
        process_topdown(args.topdown_raw, top_out)

if __name__ == "__main__":
    main()
