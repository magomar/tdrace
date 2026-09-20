#!/usr/bin/env python3
import io
import json
import time
import urllib.parse
import urllib.request
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
VEHICLES_JSON = ROOT / "portals" / "shared" / "data" / "vehicles.json"
REFS_DIR = ROOT / "assets" / "textures" / "vehicles" / "references"

HEADERS = {
    "User-Agent": "TdRace-ReferenceBot/1.0 (motorsport simulator research; magomar@gmail.com)"
}

MODULE_MAP = {
    "Gran Turismo & Endurance": "gt",
    "NASCAR Stock Car Racing": "nascar",
    "Rallycross & All-Terrain": "rally",
    "Extreme Off-Road & Arenas": "extreme_offroad",
    "Karting & Micro-Racers": "kart",
}

# Curated specific search queries for each vehicle
SEARCH_QUERIES = {
    # GT Module
    "gt_porsche_718_gt4": ["Porsche 718 Cayman GT4 RS Clubsport", "Porsche 718 Cayman GT4 Clubsport", "Porsche Cayman GT4"],
    "gt_bmw_m4_gt4": ["BMW M4 GT4", "BMW M4 G82 GT4", "BMW M4 GT4 racing"],
    "gt_aston_vantage_gt4": ["File:Volt Aston Martin GT4 WGI22.jpg", "Aston Martin Vantage AMR GT4", "Aston Martin Vantage GT4"],
    "gt_toyota_supra_gt4": ["Toyota GR Supra GT4", "Supra GT4 EVO", "Toyota Supra racing"],
    "gt_porsche_911_gt3r": ["Porsche 911 GT3 R", "992 GT3 R"],
    "gt_ferrari_296_gt3": ["File:2023 Ferrari 296 GT3 Daytona (cropped).jpg", "Ferrari 296 GT3", "Ferrari 296 racing"],
    "gt_amg_gt3_evo": ["File:Mercedes-AMG GT3 Evo of 2 Seas Motorsports (2025 British GT Donington Park).jpg", "Mercedes-AMG GT3 Evo", "Mercedes-AMG GT3"],
    "gt_audi_r8_gt3_evo2": ["Audi R8 LMS GT3 Evo", "Audi R8 LMS GT3", "Audi R8 LMS"],
    "gt_porsche_911_gt2_rs": ["Porsche 911 GT2 RS Clubsport", "Porsche 911 GT2 RS", "GT2 RS Clubsport"],
    "gt_brabham_bt62_gt2": ["Brabham BT62", "Brabham BT63 GT2", "Brabham BT62 racing"],
    "gt_maserati_mc20_gt2": ["File:Blue Maserati MC20 GT2 Stradale.jpg", "Maserati MC20 GT2", "Maserati MC20 racing"],
    "gt_audi_r8_gt2": ["Audi R8 LMS GT2", "Audi R8 GT2"],
    "gt_porsche_911_gt1_98": ["File:Porsche 911 GT1 '98 -25.jpg", "Porsche 911 GT1-98", "Porsche 911 GT1 Le Mans"],
    "gt_mclaren_f1_gtr_lt": ["McLaren F1 GTR Longtail", "McLaren F1 GTR", "McLaren F1 GTR Le Mans"],
    "gt_mercedes_clk_gtr": ["File:Mercedes-Benz CLK-GTR 1997 AMG FIA-GT Bernd Schneider RFront MBMuse 9June2013 (14983247202).jpg", "Mercedes-Benz CLK GTR", "Mercedes CLK GTR FIA GT"],
    "gt_nissan_r390_gt1": ["File:Nissan R390 GT1 No.32 in 1998 24 Hours of Le Mans.jpg", "Nissan R390 GT1", "Nissan R390 Le Mans"],
    "gt_ferrari_499p": ["Ferrari 499P", "Ferrari 499P Le Mans", "Ferrari Hypercar 499P"],
    "gt_porsche_963": ["Porsche 963", "Porsche 963 LMDh", "Porsche Penske 963"],
    "gt_toyota_gr010": ["Toyota GR010 Hybrid", "Toyota GR010 Le Mans", "Toyota GR010"],
    "gt_cadillac_v_series_r": ["File:2023 Cadillac V-LMDh Daytona (cropped).jpg", "Cadillac V-Series.R", "Cadillac V-LMDh"],

    # NASCAR Module
    "nascar_monte_carlo_ss": ["Chevrolet Monte Carlo NASCAR", "Monte Carlo stock car", "Chevrolet Monte Carlo 1986"],
    "nascar_mustang_street_stock": ["File:Ford Fox Body Mustang Pace Car (15386407392).jpg", "Ford Mustang Street Stock", "Mustang stock car racing"],
    "nascar_dodge_dart_street_stock": ["File:1973 Dodge Dart Swinger -- 05-28-2011.jpg", "Dodge Dart racing", "Dodge Dart stock car"],
    "nascar_super_late_model": ["File:2019 Slinger Super Late Model 51 Steve Apel Slinger Nationals.jpg", "Super Late Model Camaro", "Super Late Model stock car"],
    "nascar_mustang_super_late_model": ["File:Ford Mustang, NASCAR Whelen Euro Series (53444816388).jpg", "Super Late Model Mustang", "Ford Late Model stock car"],
    "nascar_late_model_stock_car": ["File:Chris Clyne CARS Tour West Late Model Las Vegas 2025.jpg", "Late Model Stock Car", "Late Model chassis"],
    "nascar_arca_chevy_ss": ["Chevrolet SS ARCA", "NASCAR ARCA Menards", "Chevrolet SS NASCAR"],
    "nascar_toyota_camry_arca": ["File:Jesse Love ARCA Sonoma 2021.jpg", "Toyota Camry ARCA", "Toyota Camry NASCAR"],
    "nascar_ford_fusion_arca": ["Ford Fusion ARCA", "Ford Fusion NASCAR", "Ford Fusion stock car"],
    "nascar_silverado_truck": ["Chevrolet Silverado NASCAR Truck", "Silverado Craftsman Truck", "Silverado racing"],
    "nascar_f150_truck": ["File:2025 LiUNA! 150 - Matt Crafton.jpg", "Ford F-150 NASCAR Truck", "F-150 Craftsman Truck"],
    "nascar_tundra_truck": ["File:Kyle Busch Miccosukee 51 Toyota Tundra - Brands Hatch 2026.jpg", "Toyota Tundra NASCAR Truck", "Tundra TRD Pro NASCAR"],
    "nascar_corvette_ta1": ["Trans-Am Corvette", "Corvette C7 TA1", "Corvette Trans Am racing"],
    "nascar_mustang_ta1": ["File:Bridgehaul 42 TA23 01.jpg", "Trans-Am Mustang", "Mustang TA1"],
    "nascar_challenger_ta1": ["File:Said Challenger 79 TA23.jpg", "Trans-Am Dodge Challenger", "Challenger TA1"],

    # Rally Module
    "rally_peugeot_208_rally4": ["Peugeot 208 Rally4", "Peugeot 208 R2", "Peugeot 208 rally"],
    "rally_fiesta_rally4": ["File:2020 Rally Bohemia - Rådström.jpg", "Ford Fiesta Rally4", "Ford Fiesta R2 rally"],
    "rally_clio_rally4": ["Renault Clio Rally4", "Renault Clio R3 rally", "Renault Clio Rally"],
    "rally_hyundai_i20_rx": ["File:Thierry Neuville - Hyundai i20 WRC.jpg", "Hyundai i20 RX", "Hyundai i20 WRX"],
    "rally_polo_rx": ["Volkswagen Polo RX", "VW Polo WRX", "Volkswagen Polo rallycross"],
    "rally_audi_s1_rx": ["File:Anton Marklund (Audi S1 EKS RX quattro) (23372879166).jpg", "Audi S1 EKS RX", "Audi S1 WRX"],
    "rally_audi_sport_quattro_s1": ["Audi Sport Quattro S1 E2", "Audi Sport Quattro S1", "Audi Quattro Group B"],
    "rally_peugeot_205_t16": ["Peugeot 205 T16", "Peugeot 205 Turbo 16", "Peugeot 205 Group B"],
    "rally_lancia_delta_s4": ["Lancia Delta S4", "Lancia Delta S4 Group B", "Delta S4 rally"],
    "rally_toyota_hilux_t1_plus": ["Toyota Hilux Dakar", "Toyota GR DKR Hilux", "Hilux T1+"],
    "rally_audi_rs_q_etron": ["Audi RS Q e-tron", "Audi RS Q e-tron Dakar", "Audi Dakar rally"],
    "rally_prodrive_hunter_t1": ["Prodrive Hunter Dakar", "Prodrive Hunter T1+", "Prodrive Hunter"],
    "rally_sst_super_truck": ["File:No 51 Ryan Beat jumping Continental Tire truck.jpg", "Stadium Super Trucks", "Stadium Super Truck"],
    "rally_sst_robby_gordon": ["Robby Gordon Stadium Super Truck", "Robby Gordon SST", "Stadium Super Trucks Gordon"],
    "rally_sst_traxxas_edition": ["Traxxas Stadium Super Truck", "SST Traxxas", "Stadium Super Truck jumping"],

    # Extreme Offroad Module
    "offroad_sand_rail_buggy": ["Can-Am Maverick R", "Can-Am Maverick buggy", "Can-Am Maverick X3"],
    "offroad_polaris_rzr_pro_r": ["Polaris RZR Pro R", "Polaris RZR buggy", "Polaris RZR racing"],
    "offroad_vw_sand_rail": ["Sand rail buggy", "VW Sand rail", "Dune buggy sand rail"],
    "offroad_baja_trophy_truck": ["File:2009TrophyTruck.jpg", "Baja 1000 Trophy Truck", "Geiser Bros Trophy Truck"],
    "offroad_bettantown_trophy_truck": ["Trophy Truck Baja", "Desert Trophy Truck", "Unlimited Trophy Truck"],
    "offroad_mason_awd_truck": ["File:JoshBaldwin.jpg", "Mason AWD Trophy Truck", "Mason Motorsport truck"],
    "offroad_subaru_ice_racer": ["Subaru WRX STI rally snow", "Subaru STI ice racing", "Subaru WRX rally"],
    "offroad_audi_quattro_ice": ["Audi Quattro ice racing", "Audi Sport Quattro snow", "Audi rally snow"],
    "offroad_lancer_evo_ice": ["Mitsubishi Lancer Evo snow rally", "Lancer Evolution ice racing", "Evo rally snow"],
    "offroad_mega_mud_truck": ["File:MudBoggingSeptember2007(cropped).jpg", "Mega Truck mud", "Mega Truck mud bogging"],
    "offroad_chevy_k30_mud_bogger": ["File:1978 Chevrolet K10 Scottsdale 6.5' Fleetside with Chevy Sport package, front right (2022 Ellwood City Fall Fest & Car Cruise).jpg", "Chevrolet K30 mud truck", "Chevy mud bogger"],
    "offroad_ford_f250_high_riser": ["File:Lifted White Ford F-250 on W Tropical Pkwy.jpg", "Ford F-250 mud truck", "Ford F250 monster"],
    "offroad_grave_crusher": ["Grave Digger Monster Truck", "Grave Digger monster jam", "Grave Digger"],
    "offroad_max_d_monster": ["Maximum Destruction Monster Truck", "Max-D monster jam", "Max-D"],
    "offroad_bigfoot_crusher": ["File:Bigfoot 19 Monster Truck 2013.jpg", "Bigfoot Monster Truck", "Bigfoot 4x4"],

    # Karting Module
    "kart_crg_hero_60": ["File:CRG Kali Go-Kart.jpg", "CRG kart", "CRG Cadet kart"],
    "kart_birel_c28": ["Birel ART kart", "Birel cadet kart", "Birel kart"],
    "kart_tony_kart_neos": ["Tony Kart Neos", "Tony Kart cadet", "Tony Kart racing"],
    "kart_tony_kart_racer_ok": ["File:Tony Kart Viper.jpg", "Tony Kart Racer 401", "Tony Kart OK"],
    "kart_crg_kt2_ok": ["CRG KT2 kart", "CRG OK kart", "CRG racing kart"],
    "kart_birel_ry30_ok": ["Birel ART RY30", "Birel ART racing kart", "Birel racing"],
    "kart_birel_art_kz2": ["Birel ART KZ2", "KZ2 shifter kart", "Birel shifter kart"],
    "kart_crg_road_rebel_kz": ["File:Klara Kowalczyk's karts under CRG Factory Karting Racing Team's tent.jpg", "CRG Road Rebel", "CRG KZ2 shifter"],
    "kart_tony_kart_racer_kz": ["File:MotorFest 2012 Kart Performance (7830258396).jpg", "Tony Kart KZ2", "Tony Kart shifter"],
    "kart_honda_mean_mower": ["File:Honda S Racing Lawn Mowers (25976437).jpeg", "Honda Mean Mower", "Honda Mean Mower V2"],
    "kart_john_deere_racing_mower": ["Lawnmower racing", "Racing lawnmower", "Custom racing mower"],
    "kart_viking_t6_tractor": ["File:2007 swifts creek lawnmower races04 edit.jpg", "Racing tractor lawnmower", "Lawn mower racing tractor"],
    "kart_anderson_cs250": ["Superkart 250", "Anderson Superkart", "250cc Superkart"],
    "kart_ms_superkart_250": ["MS Kart Superkart", "Superkart MS Kart 250", "Superkart racing"],
    "kart_viper_250_twin": ["File:Iya Harpas's Superkart at the 2025 Adelaide Grand Final.jpg", "Viper Superkart 250", "PVP Superkart 250"],
}

def search_wikimedia(query):
    if query.startswith("File:"):
        url = f"https://commons.wikimedia.org/w/api.php?action=query&titles={urllib.parse.quote(query)}&prop=imageinfo&iiprop=url|size|mime&format=json"
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
                    if u:
                        results.append((w, h, u))
                return results
        except Exception as e:
            print(f"    [API Error] {e}")
            return []

    url = f"https://commons.wikimedia.org/w/api.php?action=query&generator=search&gsrsearch={urllib.parse.quote(query)}&gsrnamespace=6&gsrlimit=8&prop=imageinfo&iiprop=url|size|mime&format=json"
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
                if w >= 800 and u:
                    results.append((w, h, u))
            results.sort(key=lambda x: x[0], reverse=True)
            return results
    except Exception as e:
        print(f"    [API Error] {e}")
        return []

def download_and_process(url, dest_path):
    # Try thumb url for fast download if large
    download_url = url
    if "upload.wikimedia.org/wikipedia/commons/" in url and not "/thumb/" in url:
        # e.g. https://upload.wikimedia.org/wikipedia/commons/a/b/filename.jpg -> thumb 1280px
        parts = url.split("wikipedia/commons/")
        if len(parts) == 2:
            fname = parts[1].split("/")[-1]
            download_url = f"{parts[0]}wikipedia/commons/thumb/{parts[1]}/1280px-{fname}"

    req = urllib.request.Request(download_url, headers=HEADERS)
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            raw = resp.read()
    except Exception:
        # Fallback to direct url
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

def main():
    vehicles = json.loads(VEHICLES_JSON.read_text(encoding="utf-8"))
    print(f"🏎️ Processing references for {len(vehicles)} vehicles...\n")

    success_count = 0
    skip_count = 0
    fail_count = 0

    for idx, v in enumerate(vehicles, 1):
        car_id = v["id"]
        mod_name = v["module"]
        mod_id = MODULE_MAP.get(mod_name, "gt")
        dest_file = REFS_DIR / mod_id / f"{car_id}.jpg"

        if dest_file.exists() and dest_file.stat().st_size > 10000:
            print(f"[{idx}/80] ✅ {car_id}: Already exists ({dest_file.stat().st_size // 1024} KB)")
            skip_count += 1
            continue

        print(f"[{idx}/80] 🔍 Searching reference for {car_id} ({v['name']})...")
        queries = SEARCH_QUERIES.get(car_id, [v["name"]])
        downloaded = False

        for q in queries:
            results = search_wikimedia(q)
            if results:
                for w, h, u in results[:3]:
                    if download_and_process(u, dest_file):
                        print(f"    ✨ Downloaded {dest_file.name} ({w}x{h}) via '{q}'")
                        downloaded = True
                        break
            if downloaded:
                break
            time.sleep(0.3)

        if downloaded:
            success_count += 1
        else:
            print(f"    ❌ Failed to find suitable reference photo for {car_id}")
            fail_count += 1

        time.sleep(0.2)

    print(f"\n🏁 Finished Reference Download:")
    print(f"   Existing/Skipped: {skip_count}")
    print(f"   Newly Downloaded: {success_count}")
    print(f"   Failed / Pending: {fail_count}")

if __name__ == "__main__":
    main()
