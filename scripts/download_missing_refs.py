#!/usr/bin/env python3
import io
import json
import time
import urllib.parse
import urllib.request
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
REFS_DIR = ROOT / "assets" / "textures" / "vehicles" / "references"

HEADERS = {
    "User-Agent": "TdRace-ReferenceBot/1.0 (motorsport simulator research; magomar@gmail.com)"
}

MISSING = {
    "offroad_audi_quattro_ice": ("extreme_offroad", ["Audi Quattro rally", "Audi Sport Quattro rally", "Audi Quattro snow"]),
    "offroad_chevy_k30_mud_bogger": ("extreme_offroad", ["Chevrolet K-30", "Chevrolet K30", "Chevrolet K20", "Chevrolet C/K mud"]),
    "offroad_ford_f250_high_riser": ("extreme_offroad", ["Ford F-250 Super Duty", "Ford F-250 4x4", "Ford F-250"]),
    "kart_birel_art_kz2": ("kart", ["Kart racing", "Go-kart racing", "Karting"]),
    "kart_crg_road_rebel_kz": ("kart", ["Go-kart", "Karting race", "Kart chassis"]),
    "kart_tony_kart_racer_kz": ("kart", ["Karting competition", "Kart track", "Go-kart driver"]),
    "kart_honda_mean_mower": ("kart", ["Lawnmower race", "Lawn mower racing", "Racing mower"]),
}

def search_wikimedia(query):
    url = f"https://commons.wikimedia.org/w/api.php?action=query&generator=search&gsrsearch={urllib.parse.quote(query)}&gsrnamespace=6&gsrlimit=6&prop=imageinfo&iiprop=url|size|mime&format=json"
    req = urllib.request.Request(url, headers=HEADERS)
    try:
        with urllib.request.urlopen(req, timeout=12) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            pages = data.get("query", {}).get("pages", {})
            results = []
            for pid, p in pages.items():
                ii_list = p.get("imageinfo", [])
                if not ii_list:
                    continue
                ii = ii_list[0]
                mime = ii.get("mime", "")
                if "jpeg" not in mime and "png" not in mime and "jpg" not in mime:
                    continue
                w = ii.get("width", 0)
                h = ii.get("height", 0)
                u = ii.get("url", "")
                if w >= 600 and u:
                    results.append((w, h, u))
            results.sort(key=lambda x: x[0], reverse=True)
            return results
    except Exception as e:
        print(f"    [API Error] {e}")
        return []

def download_and_process(url, dest_path):
    download_url = url
    if "upload.wikimedia.org/wikipedia/commons/" in url and not "/thumb/" in url:
        parts = url.split("wikipedia/commons/")
        if len(parts) == 2:
            fname = parts[1].split("/")[-1]
            download_url = f"{parts[0]}wikipedia/commons/thumb/{parts[1]}/1280px-{fname}"

    req = urllib.request.Request(download_url, headers=HEADERS)
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            raw = resp.read()
    except Exception:
        try:
            req = urllib.request.Request(url, headers=HEADERS)
            with urllib.request.urlopen(req, timeout=20) as resp:
                raw = resp.read()
        except Exception as e:
            print(f"    [Download Error] {e}")
            return False

    try:
        im = Image.open(io.BytesIO(raw)).convert("RGB")
        w, h = im.size
        if w > 1280:
            new_h = int(h * (1280 / w))
            im = im.resize((1280, new_h), Image.Resampling.LANCZOS)
        dest_path.parent.mkdir(parents=True, exist_ok=True)
        im.save(dest_path, "JPEG", quality=88)
        return True
    except Exception as e:
        print(f"    [Image Process Error] {e}")
        return False

for cid, (mod_id, queries) in MISSING.items():
    dest = REFS_DIR / mod_id / f"{cid}.jpg"
    print(f"Searching for {cid}...")
    for q in queries:
        res = search_wikimedia(q)
        if res:
            w, h, u = res[0]
            if download_and_process(u, dest):
                print(f"  ✅ Downloaded {dest.name} ({w}x{h}) via '{q}'")
                break
        time.sleep(0.3)
