---
name: osm-circuit-builder
description: >-
  Build, repair, scale, or calibrate realistic circuit and rallycross tracks from real-world
  OpenStreetMap (OSM) survey data, FIA homologation dimensions, mixed surfaces, apex curbs,
  and contained jump ramps.
---

# OpenStreetMap (OSM) Circuit Track Builder & Repairer

This skill guides the end-to-end engineering workflow to import, reconstruct, repair, or add real-world motorsport and rallycross circuits into `tdrace` using authentic OpenStreetMap (OSM) geographic survey data.

---

## 1. Locating the Circuit & Fetching OSM Data

### 1.1 Nominatim Geocoding
To find the bounding box of a real-world raceway:
```bash
curl -s "https://nominatim.openstreetmap.org/search?q=Estering+Buxtehude&format=json" \
  -H "User-Agent: tdrace-osm-tool/1.0" | jq '.[0] | {lat, lon, boundingbox}'
```

### 1.2 OpenStreetMap API vs Overpass
- **Overpass API** (`https://overpass-api.de/api/interpreter`): Great for querying specific tags (`way["highway"="raceway"](...)`), but frequently rate-limits (HTTP 429) or times out under load.
- **Direct OSM Map API** (`https://api.openstreetmap.org/api/0.6/map?bbox=minLon,minLat,maxLon,maxLat`): Extremely fast (<1s), reliable, and returns full node coordinates with tags in a single call.
- Always cache downloaded JSON files in `target/osm_cache/<track_id>.json`.

---

## 2. Assembling a Clean Closed-Loop Ribbon

Real-world raceways in OSM often consist of multiple contiguous or intersecting segments (`highway=raceway`, `surface=asphalt`, `surface=dirt` / `unpaved` / `gravel`).

### Steps for Assembly:
1. **Identify Start Straight**: Locate the main start/finish straight way and start index.
2. **Topological Slicing**: Match way junction nodes by node ID. For connecting way segments, ensure the end node of segment $A$ is identical to (or within $<5$m of) the start node of segment $B$.
3. **Loop Closure**: Verify that the distance from the final node back to the start node is contiguous ($<15$m).
4. **Surface Tagging**:
   - `surface in ["gravel", "dirt", "unpaved", "fine_gravel", "sand"]` $\to$ `SurfaceType::Dirt`.
   - `surface in ["asphalt", "paved", "concrete"]` $\to$ `SurfaceType::Asphalt`.

---

## 3. Projection, Alignment, and Resampling

### 3.1 Metric 2D Projection (Equirectangular / Tangent Plane)
```python
def latlon_to_meters(lat, lon, lat0, lon0):
    r = 6378137.0  # WGS84 earth radius
    x = (math.radians(lon) - math.radians(lon0)) * math.cos(math.radians(lat0)) * r
    y = (math.radians(lat) - math.radians(lat0)) * r
    return x, y
```

### 3.2 Heading Rotation along $+X$
Calculate the heading vector of the first 5–8 start straight points:
$$\theta = \text{atan2}(\Delta y, \Delta x)$$
Rotate all points by $-\theta$ so the starting grid faces along $+X$:
$$x' = x \cos(-\theta) - y \sin(-\theta)$$
$$y' = x \sin(-\theta) + y \cos(-\theta)$$

### 3.3 Scaling to Official FIA Homologation Length
Scale coordinates by the ratio of official FIA length to current perimeter:
$$s = \frac{L_{\text{FIA}}}{L_{\text{measured}}}$$
$$P_{\text{scaled}} = s \cdot P$$

### 3.4 Uniform Resampling
Resample the polygon to 26–32 uniform points using cumulative arc-length interpolation. This guarantees smooth Catmull-Rom spline curves without clustering.

---

## 4. Curvature, Curbs, and Corridor Tolerances

### 4.1 Apex Curb Detection
Calculate cross products between consecutive normalized segments:
$$\text{cross} = v_1.x \cdot v_2.y - v_1.y \cdot v_2.x$$
- $\text{cross} > 60.0$: sharp left turn $\to$ inside curb on left side (`with_curbs(true, false)`).
- $\text{cross} < -60.0$: sharp right turn $\to$ inside curb on right side (`with_curbs(false, true)`).

### 4.2 Hairpin Corridor Clearance
> [!WARNING]
> When two parallel track ribbons are within $< 20$m of each other (e.g. 180° hairpin turnarounds like Catalunya RX Turn 10), standard barrier generation with road half-width $W/2$ and wall offset $D_{\text{wall}}$ may cause wall barriers to cross the opposite track ribbon!
>
> **Fix**:
> 1. Narrow road width at the hairpin apex (e.g., $11.0\text{m} - 11.5\text{m}$).
> 2. Ensure apex centerline spacing $\ge W + 2 \cdot D_{\text{wall}} \approx 20\text{m}$.

---

## 5. Jump Ramp Calibration & Physical Containment

When adding rallycross jump ramps:
1. **Surface Type**: Always use `SurfaceType::Dirt` (`.with_surface(SurfaceType::Dirt)`).
2. **Dimensions**:
   - `half_extents: Vec2::new(3.8, 5.5)` (compact kicker/tabletop ramp).
   - `height: 1.2 – 1.3m`.
   - `ramp_angle_deg: 5.2° – 5.5°`.
   - `launch_speed: 2.2 m/s`.
3. **Alignment**: Orient tangent angle directly along the downstream spline tangent vector to prevent lateral kicks off the track.
4. **Containment Verification**: Cars jumping at $30\text{ m/s}$ ($108\text{ km/h}$) and $42\text{ m/s}$ ($151\text{ km/h}$) must land cleanly on track without breaching tire walls or fences.

---

## 6. Registration & Validation Checklist

Every new or modified circuit must be registered across the workspace:
1. `crates/tdrace-core/src/track/presets.rs`: Implement the constructor function and add docstrings.
2. `crates/tdrace-core/src/track/mod.rs`: Re-export the function in `pub use presets::{...}`.
3. `crates/tdrace-app/src/module/rally.rs`:
   - Add to `RallyGameModule::tracks()`.
   - Add to `supported_game_modes()` championships.
4. `crates/tdrace-app/src/track_manager.rs`:
   - Add to `load_track` fallback match.
   - Add to `preset_module` mapping.
   - Add to `canonical_preset_id` aliases.
5. `crates/tdrace-app/src/ui/menu.rs`:
   - Add slug to `TrackChoice::tag()` and `tag_for_module()`.
   - Add to `resolve_built_in_track()`.
6. Export canonical JSON tracks to git repository:
   ```bash
   cargo test -p tdrace-app --test track_manager_tests test_export_canonical_presets_to_git_repo
   ```
7. Verify all tests pass:
   ```bash
   cargo test --workspace --exclude tdrace-py
   ```
