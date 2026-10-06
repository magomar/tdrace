#!/usr/bin/env python3
# /// script
# dependencies = ["pillow", "numpy"]
# ///
"""
Automated Global Vehicle Chassis Cutout and Inpainting Pipeline (Spec 091).

Generates `<model_id>_chassis.png` sprites across all motorsport modules in TdRace:
- GT, NASCAR, Rally, Autocross, Karting, and Extreme Off-Road.
- Uses the authentic Canvas-to-Axle Offset Equation based on physical wheelbase,
  overhangs, and track width from portals/shared/data/codex/cars.json.
- Open-Wheel Archetypes (OverChassis):
  Clears outboard front tire rubber while strictly preserving suspension wishbones,
  pushrods, tie rods, and front nosecone/bumper bodywork.
- Closed-Wheel Archetypes (UnderChassis):
  Hollows out outer fender apertures for underlying steered wheel visibility,
  and inpaints dark ambient cavity backing (#14181c, 100% opacity) across the inner
  wheel-well liner to prevent track surface see-through during dynamic suspension roll (+-18cm).
"""

import argparse
import json
from pathlib import Path
import sys
import numpy as np
from PIL import Image

# Open-wheel modality platform archetypes (Spec 091 / Spec 094)
OPEN_WHEEL_PLATFORMS = {
    "CrossCar",
    "SuperBuggy",
    "DuneBuggyBaja",
    "MonsterTruck",
    "Kart",
    "SuperkartGP",
    "SandRail",
}

# Fallbacks for bonus / vault archive vehicles not tracked in codex cars.json
FALLBACK_VEHICLES = {
    "vault_asahi_blade_runner": {
        "base_car": "Kart",
        "module": "kart",
        "physics": {
            "wheelbase": 1.30,
            "track_width": 0.62,
            "chassis": {"front_overhang": 0.22, "rear_overhang": 0.20},
            "wheels": [{"tire_radius": 0.20, "tire_width": 0.16}],
        },
    },
    "vault_greenfield_prairie_racer": {
        "base_car": "Kart",
        "module": "kart",
        "physics": {
            "wheelbase": 1.30,
            "track_width": 0.62,
            "chassis": {"front_overhang": 0.22, "rear_overhang": 0.20},
            "wheels": [{"tire_radius": 0.20, "tire_width": 0.16}],
        },
    },
    "vault_nordic_valhalla_tractor": {
        "base_car": "Kart",
        "module": "kart",
        "physics": {
            "wheelbase": 1.30,
            "track_width": 0.62,
            "chassis": {"front_overhang": 0.22, "rear_overhang": 0.20},
            "wheels": [{"tire_radius": 0.20, "tire_width": 0.16}],
        },
    },
    "classic_ax_talon": {
        "base_car": "CrossCar",
        "module": "classic",
        "physics": {
            "wheelbase": 2.15,
            "track_width": 1.55,
            "chassis": {"front_overhang": 0.22, "rear_overhang": 0.28},
            "wheels": [{"tire_radius": 0.28, "tire_width": 0.24}],
        },
    },
}

DARK_CAVITY_RGB = (20, 24, 28)
DARK_CAVITY_ALPHA = 255


def load_vehicle_catalog(root: Path):
    """Loads all car definitions from codex cars.json and merges fallback vehicles."""
    cars_json_path = root / "portals" / "shared" / "data" / "codex" / "cars.json"
    catalog = {}

    if cars_json_path.exists():
        with open(cars_json_path, "r", encoding="utf-8") as f:
            data = json.load(f)
            for item in data.get("items", []):
                catalog[item["id"]] = item

    for fallback_id, fallback_data in FALLBACK_VEHICLES.items():
        if fallback_id not in catalog:
            catalog[fallback_id] = {
                "id": fallback_id,
                **fallback_data,
                "images": {
                    "topdown": f"/textures/vehicles/topdown/{fallback_data['module']}/{fallback_id}.png"
                },
            }

    return catalog


def process_vehicle_sprite(
    image_path: Path,
    car_data: dict,
    out_path: Path,
    dry_run: bool = False,
):
    """Processes a single topdown vehicle sprite into a chassis cutout."""
    im = Image.open(image_path).convert("RGBA")
    arr = np.array(im)

    alpha = arr[:, :, 3]
    ys, xs = np.where(alpha > 20)
    if len(xs) == 0:
        print(f"  [WARN] {image_path.name} contains no opaque pixels, skipping.")
        return False

    x_min, x_max = xs.min(), xs.max()
    y_min, y_max = ys.min(), ys.max()
    w_px = x_max - x_min + 1
    cy_px = (y_min + y_max) / 2.0

    p = car_data.get("physics", {})
    wb = p.get("wheelbase", 2.6)
    chassis = p.get("chassis", {})
    df = chassis.get("front_overhang", 0.8)
    dr = chassis.get("rear_overhang", 1.0)
    tw = p.get("track_width", 1.8)
    ltot = wb + df + dr

    # Uniform scale factor: pixels per physical meter
    S = w_px / ltot

    # Canonical Canvas-to-Axle Offset Equation (Spec 091 Section 1)
    axle_x = x_max - df * S
    y_fl = cy_px - (tw / 2.0) * S
    y_fr = cy_px + (tw / 2.0) * S

    wheels = p.get("wheels", [])
    w0 = wheels[0] if wheels else {}
    tire_r = w0.get("tire_radius", 0.32)
    tire_w = w0.get("tire_width", 0.24)

    base_car = car_data.get("base_car", "GT4Clubsport")
    is_open_wheel = base_car in OPEN_WHEEL_PLATFORMS

    out_arr = arr.copy()

    # Bounding box along X
    half_len = (2.0 * tire_r * S) * 0.5 * 1.25
    x0 = max(0, int(round(axle_x - half_len)))
    x1 = min(512, int(round(axle_x + half_len)))

    r, g, b = arr[:, :, 0].astype(int), arr[:, :, 1].astype(int), arr[:, :, 2].astype(int)
    brightness = np.maximum(r, np.maximum(g, b))
    color_var = np.maximum(np.abs(r - g), np.maximum(np.abs(r - b), np.abs(g - b)))
    # Dark rubber detection: low brightness and achromatic
    is_rubber = (brightness < 80) & (color_var < 35) & (alpha > 20)

    if is_open_wheel:
        # Open wheel (OverChassis):
        # Clear tire rubber in front wheel bounding boxes while preserving wishbones/fairings
        half_wid = (tire_w * S) * 0.5 * 1.30
        for y_center in [y_fl, y_fr]:
            y0 = max(0, int(round(y_center - half_wid)))
            y1 = min(512, int(round(y_center + half_wid)))
            box = np.zeros((512, 512), dtype=bool)
            box[y0:y1, x0:x1] = True
            out_arr[box & is_rubber, 3] = 0
    else:
        # Closed wheel (UnderChassis):
        # 1. Hollow out outer aperture where the tire is exposed
        # 2. Inpaint dark cavity backing across inner liner (+-18cm roll buffer)
        half_wid = (tire_w * S) * 0.5 * 1.25
        cavity_depth = int(round(0.18 * S))  # 18cm suspension roll buffer

        # Front-Left (top quadrant)
        y_out_fl = max(0, int(round(y_fl - half_wid * 1.25)))
        y_in_fl = min(int(cy_px - 20), int(round(y_fl + half_wid)))
        y_cav_fl = min(int(cy_px - 15), y_in_fl + cavity_depth)

        fl_aperture = np.zeros((512, 512), dtype=bool)
        fl_aperture[y_out_fl:y_in_fl, x0:x1] = True
        out_arr[fl_aperture & is_rubber, 3] = 0

        fl_cavity = np.zeros((512, 512), dtype=bool)
        fl_cavity[y_in_fl:y_cav_fl, x0:x1] = True
        apply_cav_fl = fl_cavity & ((out_arr[:, :, 3] == 0) | is_rubber)
        out_arr[apply_cav_fl, 0] = DARK_CAVITY_RGB[0]
        out_arr[apply_cav_fl, 1] = DARK_CAVITY_RGB[1]
        out_arr[apply_cav_fl, 2] = DARK_CAVITY_RGB[2]
        out_arr[apply_cav_fl, 3] = DARK_CAVITY_ALPHA

        # Front-Right (bottom quadrant)
        y_out_fr = min(512, int(round(y_fr + half_wid * 1.25)))
        y_in_fr = max(int(cy_px + 20), int(round(y_fr - half_wid)))
        y_cav_fr = max(int(cy_px + 15), y_in_fr - cavity_depth)

        fr_aperture = np.zeros((512, 512), dtype=bool)
        fr_aperture[y_in_fr:y_out_fr, x0:x1] = True
        out_arr[fr_aperture & is_rubber, 3] = 0

        fr_cavity = np.zeros((512, 512), dtype=bool)
        fr_cavity[y_cav_fr:y_in_fr, x0:x1] = True
        apply_cav_fr = fr_cavity & ((out_arr[:, :, 3] == 0) | is_rubber)
        out_arr[apply_cav_fr, 0] = DARK_CAVITY_RGB[0]
        out_arr[apply_cav_fr, 1] = DARK_CAVITY_RGB[1]
        out_arr[apply_cav_fr, 2] = DARK_CAVITY_RGB[2]
        out_arr[apply_cav_fr, 3] = DARK_CAVITY_ALPHA

    erased_px = int(np.sum((arr[:, :, 3] > 0) & (out_arr[:, :, 3] == 0)))
    cavity_px = int(
        np.sum(
            (out_arr[:, :, 0] == DARK_CAVITY_RGB[0])
            & (out_arr[:, :, 1] == DARK_CAVITY_RGB[1])
            & (out_arr[:, :, 2] == DARK_CAVITY_RGB[2])
            & (out_arr[:, :, 3] == DARK_CAVITY_ALPHA)
        )
    )

    if not dry_run:
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_img = Image.fromarray(out_arr)
        out_img.save(out_path, format="PNG")

    mode_label = "OverChassis (open)" if is_open_wheel else "UnderChassis (closed)"
    print(
        f"  ✓ {car_data['id']:<34} [{mode_label:<20}] Axle: X={axle_x:5.1f} | Erased: {erased_px:4d} px | Cavity: {cavity_px:4d} px"
    )
    return True


def main():
    parser = argparse.ArgumentParser(
        description="Automated Global Vehicle Chassis Cutout and Inpainting Pipeline (Spec 091)"
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Process all vehicles across all motorsport modules",
    )
    parser.add_argument(
        "--module",
        type=str,
        help="Target module (gt, nascar, rally, autocross, kart, extreme_offroad, classic)",
    )
    parser.add_argument(
        "--model",
        type=str,
        help="Target single vehicle model ID (e.g. gt_albion_victor_t1)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Compute geometry and stats without writing files to disk",
    )
    parser.add_argument(
        "--include-classic",
        action="store_true",
        help="Also re-generate cutouts for classic fantasy fleet (defaults to skipped)",
    )

    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent

    catalog = load_vehicle_catalog(root)
    topdown_dir = root / "assets" / "textures" / "vehicles" / "topdown"

    modules = ["gt", "nascar", "rally", "autocross", "kart", "extreme_offroad"]
    if args.include_classic:
        modules.append("classic")

    if args.module:
        target_mod = args.module.lower()
        if target_mod not in modules and target_mod != "classic":
            print(f"Unknown module '{args.module}'. Available: {modules + ['classic']}")
            sys.exit(1)
        modules = [target_mod]

    processed_count = 0
    skipped_count = 0

    print("=" * 95)
    print("🏎️  TdRace Spec 091 Global Chassis Cutout & Inpainting Generator")
    print("=" * 95)

    for mod in modules:
        mod_dir = topdown_dir / mod
        if not mod_dir.exists():
            continue

        sprite_files = sorted(mod_dir.glob("*.png"))
        valid_sprites = [p for p in sprite_files if not p.stem.endswith("_chassis")]

        print(f"\n📂 Module: [{mod}] ({len(valid_sprites)} vehicles)")

        for img_p in valid_sprites:
            model_id = img_p.stem
            if args.model and model_id != args.model:
                continue

            car_data = catalog.get(model_id)
            if not car_data:
                print(f"  [WARN] Model '{model_id}' not found in catalog, skipping.")
                skipped_count += 1
                continue

            out_chassis_p = mod_dir / f"{model_id}_chassis.png"

            if mod == "classic" and not args.include_classic and not args.model:
                print(f"  [SKIP] Classic fleet vehicle '{model_id}' preserved.")
                continue

            ok = process_vehicle_sprite(
                image_path=img_p,
                car_data=car_data,
                out_path=out_chassis_p,
                dry_run=args.dry_run,
            )
            if ok:
                processed_count += 1
            else:
                skipped_count += 1

    print("\n" + "=" * 95)
    print(f"🏁 Cutout generation complete! Processed: {processed_count}, Skipped: {skipped_count}")
    print("=" * 95)


if __name__ == "__main__":
    main()
