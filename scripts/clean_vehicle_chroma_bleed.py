#!/usr/bin/env python3
"""
clean_vehicle_chroma_bleed.py

Programmatic removal of residual #FF00FF magenta chroma-key bleed artifacts across
all vehicle sprites (laterals, thumbnails, top-down racing views, and cutout chassis).

Mechanics:
- Because assets were chroma-keyed against pure magenta (#FF00FF, R=255, G=0, B=255),
  the Green channel (G) contains zero background light.
- Any pixel where R and B symmetrically exceed G is chroma bleed:
    excess = min(R - G, B - G) > 10
    |R - B| < 35
- Despill: sets R = G and B = G.
    - Side windows become natural neutral dark cockpit tint ([40..60, 40..60, 40..60]).
    - Under-car contact shadows (where G ≈ 0) naturally become solid black ([0, 0, 0]).
    - Top-down roof flanks and wing cutouts turn into dark tinted glass/shadow.
- Faint edge fringes (G < 12, excess > 20, alpha < 100) are cleared to alpha = 0.
"""

import argparse
import sys
from pathlib import Path

import numpy as np
from PIL import Image


def clean_image(img: Image.Image) -> tuple[Image.Image, int, int]:
    """
    Cleans magenta chroma bleed from an RGBA image.
    Returns (cleaned_image, despilled_pixel_count, cleared_fringe_count).
    """
    arr = np.array(img.convert("RGBA"))
    r = arr[:, :, 0].astype(float)
    g = arr[:, :, 1].astype(float)
    b = arr[:, :, 2].astype(float)
    a = arr[:, :, 3].astype(float)

    excess = np.minimum(r - g, b - g)
    rb_diff = np.abs(r - b)

    # Magenta chroma bleed condition:
    # 1. R and B both noticeably exceed G
    # 2. R and B are fairly balanced (|R - B| < 35, distinct from pure red or blue)
    # 3. Pixel has non-trivial opacity
    is_magenta = (excess > 10) & (rb_diff < 35) & (a > 8)
    despilled_count = int(np.sum(is_magenta))

    if despilled_count == 0:
        return img, 0, 0

    clean_arr = arr.copy()

    # 1. Despill: Neutralize excess magenta by clamping R and B to G
    clean_arr[is_magenta, 0] = g[is_magenta].astype(np.uint8)
    clean_arr[is_magenta, 2] = g[is_magenta].astype(np.uint8)

    # 2. Outer silhouette fringe cleanup:
    # If a pixel has near-zero green (pure background light) and low alpha (< 100),
    # it was an anti-aliased edge halo on the #FF00FF boundary.
    is_pure_fringe = is_magenta & (g < 12) & (excess > 20) & (a < 100)
    cleared_fringe_count = int(np.sum(is_pure_fringe))
    clean_arr[is_pure_fringe, 3] = 0

    return Image.fromarray(clean_arr), despilled_count, cleared_fringe_count


def process_directory(target_dir: Path, dry_run: bool = False, verbose: bool = False):
    png_files = sorted(target_dir.rglob("*.png"))
    total_files = len(png_files)
    modified_files = 0
    total_despilled = 0
    total_fringes = 0

    print(f"Scanning {total_files} PNG sprites in {target_dir}...")
    if dry_run:
        print("🔍 DRY-RUN MODE: No files will be overwritten.\n")

    for idx, p in enumerate(png_files, 1):
        try:
            with Image.open(p) as img:
                clean_img, despilled, fringes = clean_image(img)

            if despilled > 0:
                modified_files += 1
                total_despilled += despilled
                total_fringes += fringes

                if verbose or despilled > 5000:
                    rel_path = p.relative_to(target_dir.parent.parent) if len(p.parts) > 3 else p.name
                    print(f"  [{modified_files:3d}] {rel_path}: {despilled:,} despilled, {fringes:,} fringes")

                if not dry_run:
                    clean_img.save(p, format="PNG")

        except Exception as e:  # noqa: BLE001 - deliberate catch-all: log and carry on
            print(f"❌ Error processing {p}: {e}", file=sys.stderr)

    print("\n" + "=" * 60)
    print("Summary:")
    print(f"  Total sprites scanned:    {total_files:,}")
    print(f"  Sprites modified:         {modified_files:,}")
    print(f"  Total pixels despilled:   {total_despilled:,}")
    print(f"  Edge fringes cleared:     {total_fringes:,}")
    print("=" * 60)


def main():
    parser = argparse.ArgumentParser(description="Clean magenta chroma-key bleed from vehicle sprites.")
    parser.add_argument(
        "--target-dir",
        type=Path,
        default=Path("assets/textures/vehicles"),
        help="Path to vehicle textures directory (default: assets/textures/vehicles)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Simulate cleaning and report statistics without writing changes",
    )
    parser.add_argument(
        "--file",
        type=Path,
        help="Process a single PNG file",
    )
    parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="Print details for every modified file",
    )

    args = parser.parse_args()

    if args.file:
        if not args.file.exists():
            print(f"Error: file {args.file} does not exist.", file=sys.stderr)
            return 1
        with Image.open(args.file) as img:
            cleaned, despilled, fringes = clean_image(img)
        print(f"File {args.file}: {despilled:,} pixels despilled, {fringes:,} fringes cleared.")
        if not args.dry_run:
            cleaned.save(args.file, format="PNG")
            print("✓ Saved cleaned image.")
        return 0

    if not args.target_dir.exists():
        print(f"Error: target directory {args.target_dir} does not exist.", file=sys.stderr)
        return 1

    process_directory(args.target_dir, dry_run=args.dry_run, verbose=args.verbose)
    return 0


if __name__ == "__main__":
    sys.exit(main())
