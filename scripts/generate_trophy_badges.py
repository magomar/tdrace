#!/usr/bin/env python3
"""
Generates 75 bespoke SVG vector master files and rendered 128px/256px RGBA PNGs
for championship podium trophies across 5 motorsport disciplines, 5 tiers, and 3 podium positions,
plus the locked trophy silhouette, adhering to Spec 027.
"""

import os
import subprocess
from pathlib import Path

OUTPUT_DIR = Path("assets/icons/trophies")
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

DISCIPLINES = ["gt", "kart", "rally", "nascar", "extreme_offroad"]
TIERS = [1, 2, 3, 4, 5]
METALS = ["gold", "silver", "bronze"]

ROMAN_NUMERALS = {1: "TIER I", 2: "TIER II", 3: "TIER III", 4: "TIER IV", 5: "TIER V"}

CHAMPIONSHIP_NAMES = {
    "gt": {
        1: ("CLUBMAN SPRINT", "TIER I • GT4", "GT4"),
        2: ("EURO CHALLENGE", "TIER II • GT3", "GT3"),
        3: ("POWER MASTERS", "TIER III • GT2", "GT2"),
        4: ("HERITAGE TROPHY", "TIER IV • GT1", "GT1"),
        5: ("WORLD GRAND PRIX", "TIER V • HYPERCAR", "HYPER"),
    },
    "kart": {
        1: ("WORLD CUP", "TIER I • CADET", "60CC"),
        2: ("NATIONAL CHAMP", "TIER II • JUNIOR", "OK-J"),
        3: ("CONTINENTAL CUP", "TIER III • KZ2", "KZ2"),
        4: ("EURO MASTERS", "TIER IV • SHIFTER", "MEAN"),
        5: ("SUPERKART SERIES", "TIER V • 250CC", "250"),
    },
    "rally": {
        1: ("GRASSROOTS CUP", "TIER I • CLUB", "R4"),
        2: ("WORLD RX CUP", "TIER II • SUPER1600", "WRX"),
        3: ("GROUP B MASTERS", "TIER III • GROUP B", "GP B"),
        4: ("DAKAR RAID", "TIER IV • RAID", "RAID"),
        5: ("SUPER TRUCKS", "TIER V • STADIUM", "SST"),
    },
    "nascar": {
        1: ("SHORT TRACK", "TIER I • SHORT TRACK", "LATE"),
        2: ("OVAL CHALLENGE", "TIER II • INTERMEDIATE", "ARCA"),
        3: ("NATIONAL TOUR", "TIER III • NATIONAL", "TRUCK"),
        4: ("PREMIER TROPHY", "TIER IV • PREMIER", "TA1"),
        5: ("CUP CHAMPION", "TIER V • APEX CUP", "CUP"),
    },
    "extreme_offroad": {
        1: ("DESERT SPRINT", "TIER I • SAND RAIL", "RAIL"),
        2: ("CANYON RAID", "TIER II • 4X4 RAID", "4X4"),
        3: ("OFFROAD CUP", "TIER III • ARCTIC ICE", "AT44"),
        4: ("MUD MASTERS", "TIER IV • PRO4", "PRO4"),
        5: ("ULTIMATE ARENA", "TIER V • MONSTER", "BIGF"),
    },
}

def generate_stars(tier: int, metal: str) -> str:
    # Star positions based on tier count
    # Stars float around y=102
    star_offsets = {
        1: [(0, 0, 1.0)],
        2: [(-28, 0, 1.0), (28, 0, 1.0)],
        3: [(-36, 4, 1.0), (0, -4, 1.18), (36, 4, 1.0)],
        4: [(-48, 6, 1.0), (-16, -2, 1.15), (16, -2, 1.15), (48, 6, 1.0)],
        5: [(-60, 8, 1.0), (-30, 2, 1.15), (0, -6, 1.35), (30, 2, 1.15), (60, 8, 1.0)],
    }[tier]

    star_poly_outer = "0,-16 4.7,-4.8 16,-4.8 7.3,1.8 10.6,12.8 0,6 -10.6,12.8 -7.3,1.8 -16,-4.8 -4.7,-4.8"
    star_poly_inner = "0,-11 3.2,-3.2 11,-3.2 5,1.2 7,8.6 0,4.2 -7,8.6 -5,1.2 -11,-3.2 -3.2,-3.2"

    inner_colors = {
        "gold": "#FFFDF0",
        "silver": "#FFFFFF",
        "bronze": "#FFE8DB",
    }
    inner_color = inner_colors[metal]

    stars_xml = ['<g transform="translate(256, 102)">']
    for x, y, scale in star_offsets:
        scale_attr = f' scale({scale})' if scale != 1.0 else ''
        stars_xml.append(f'  <g transform="translate({x}, {y}){scale_attr}">')
        stars_xml.append(f'    <polygon points="{star_poly_outer}" fill="url(#metal-rim)" stroke="#FFFFFF" stroke-width="1.5" />')
        stars_xml.append(f'    <polygon points="{star_poly_inner}" fill="{inner_color}" opacity="0.85" />')
        stars_xml.append('  </g>')
    stars_xml.append('</g>')
    return "\n".join(stars_xml)

def generate_handles(discipline: str) -> str:
    if discipline == "gt":
        return """
    <!-- Swept Sculpted GT Wing Handles with Endplates -->
    <path d="M 180,180 C 105,170 98,270 190,290"
          stroke="url(#metal-rim)" stroke-width="15" stroke-linecap="round" fill="none" />
    <path d="M 180,180 C 115,170 110,262 190,280"
          stroke="#FFFFFF" stroke-width="4" stroke-linecap="round" fill="none" opacity="0.9" />

    <path d="M 332,180 C 407,170 414,270 322,290"
          stroke="url(#metal-rim)" stroke-width="15" stroke-linecap="round" fill="none" />
    <path d="M 332,180 C 397,170 402,262 322,280"
          stroke="#FFFFFF" stroke-width="4" stroke-linecap="round" fill="none" opacity="0.9" />
        """
    elif discipline == "kart":
        return """
    <!-- Tuned 2-Stroke Exhaust Expansion Chambers Handles -->
    <path d="M 184,180 C 100,165 92,275 186,290"
          stroke="url(#metal-rim)" stroke-width="17" stroke-linecap="round" fill="none" />
    <path d="M 184,180 C 110,165 104,265 186,280"
          stroke="#FFFFFF" stroke-width="4" stroke-linecap="round" fill="none" opacity="0.95" />

    <path d="M 328,180 C 412,165 420,275 326,290"
          stroke="url(#metal-rim)" stroke-width="17" stroke-linecap="round" fill="none" />
    <path d="M 328,180 C 402,165 408,265 326,280"
          stroke="#FFFFFF" stroke-width="4" stroke-linecap="round" fill="none" opacity="0.95" />
        """
    elif discipline == "rally":
        return """
    <!-- Knobby Rubber All-Terrain Tire Tread Handles -->
    <g>
      <path d="M 180,180 C 95,170 88,272 186,290"
            stroke="url(#tread-rubber)" stroke-width="20" stroke-linecap="round" fill="none" />
      <path d="M 180,180 C 95,170 88,272 186,290"
            stroke="url(#metal-rim)" stroke-width="4" stroke-dasharray="8 10" stroke-linecap="round" fill="none" />
    </g>
    <g>
      <path d="M 332,180 C 417,170 424,272 326,290"
            stroke="url(#tread-rubber)" stroke-width="20" stroke-linecap="round" fill="none" />
      <path d="M 332,180 C 417,170 424,272 326,290"
            stroke="url(#metal-rim)" stroke-width="4" stroke-dasharray="8 10" stroke-linecap="round" fill="none" />
    </g>
        """
    elif discipline == "nascar":
        return """
    <!-- Swept Eagle Wing Handles -->
    <g>
      <path d="M 170,165 C 75,150 68,275 180,298"
            stroke="url(#metal-rim)" stroke-width="19" stroke-linecap="round" fill="none" />
      <path d="M 170,165 C 88,150 82,265 180,286"
            stroke="#FFFFFF" stroke-width="5" stroke-linecap="round" fill="none" opacity="0.95" />

      <path d="M 342,165 C 437,150 444,275 332,298"
            stroke="url(#metal-rim)" stroke-width="19" stroke-linecap="round" fill="none" />
      <path d="M 342,165 C 424,150 430,265 332,286"
            stroke="#FFFFFF" stroke-width="5" stroke-linecap="round" fill="none" opacity="0.95" />
    </g>
        """
    elif discipline == "extreme_offroad":
        return """
    <!-- 4130 Chromoly Roll-Cage Truss Tubing Handles -->
    <g stroke="url(#metal-rim)" stroke-width="8" stroke-linecap="round" fill="none">
      <path d="M 174,175 C 90,165 80,275 180,296" />
      <line x1="120" y1="180" x2="160" y2="230" stroke-width="4" stroke="#00FF66" />
      <line x1="105" y1="230" x2="168" y2="275" stroke-width="4" stroke="#00FF66" />
    </g>
    <g stroke="url(#metal-rim)" stroke-width="8" stroke-linecap="round" fill="none">
      <path d="M 338,175 C 422,165 432,275 332,296" />
      <line x1="392" y1="180" x2="352" y2="230" stroke-width="4" stroke="#00FF66" />
      <line x1="407" y1="230" x2="344" y2="275" stroke-width="4" stroke="#00FF66" />
    </g>
        """
    return ""

def generate_stem(discipline: str) -> str:
    if discipline == "rally":
        return """
    <!-- Helical Spring Coilover Shock Stem -->
    <path d="M 238,318 L 274,318 L 270,356 L 242,356 Z"
          fill="url(#metal-body)" stroke="#FFFFFF" stroke-width="3" />
    <path d="M 240,324 C 272,328 272,336 240,340 C 272,344 272,352 240,356"
          fill="none" stroke="#DC2626" stroke-width="6" stroke-linecap="round" />
        """
    elif discipline == "extreme_offroad":
        return """
    <!-- Dual Remote Coilover Reservoir Stem -->
    <path d="M 238,318 L 274,318 L 270,356 L 242,356 Z"
          fill="url(#metal-body)" stroke="#00FF66" stroke-width="3" />
    <rect x="226" y="324" width="10" height="26" rx="3" fill="#1E293B" stroke="#00FF66" stroke-width="1.8" />
    <rect x="276" y="324" width="10" height="26" rx="3" fill="#1E293B" stroke="#00FF66" stroke-width="1.8" />
        """
    elif discipline == "kart":
        return """
    <!-- 3-Spoke Kart Steering Column Stem -->
    <path d="M 238,318 L 274,318 L 270,356 L 242,356 Z"
          fill="url(#metal-body)" stroke="#FFFFFF" stroke-width="3" />
    <ellipse cx="256" cy="336" rx="20" ry="6" fill="#CCFF00" opacity="0.9" />
    <ellipse cx="256" cy="348" rx="23" ry="6" fill="url(#metal-rim)" />
        """
    else:
        return """
    <!-- Fluted Diffuser Stem with Aero Rings -->
    <path d="M 238,318 L 274,318 L 270,356 L 242,356 Z"
          fill="url(#metal-body)" stroke="#FFFFFF" stroke-width="3" />
    <ellipse cx="256" cy="334" rx="22" ry="6" fill="#FFFFFF" opacity="0.85" />
    <ellipse cx="256" cy="348" rx="24" ry="6" fill="url(#metal-rim)" />
        """

def generate_background_inlay(discipline: str) -> str:
    if discipline == "gt":
        return """
      <!-- Carbon Weave Matrix -->
      <g opacity="0.09" stroke="#FFFFFF" stroke-width="1.8">
        <line x1="40" y1="60" x2="472" y2="60" /><line x1="40" y1="120" x2="472" y2="120" />
        <line x1="40" y1="180" x2="472" y2="180" /><line x1="40" y1="240" x2="472" y2="240" />
        <line x1="40" y1="300" x2="472" y2="300" /><line x1="40" y1="360" x2="472" y2="360" />
      </g>
      <!-- Checkered Racing Wing Flags Inlay -->
      <g transform="translate(256, 70) skewX(-20)" opacity="0.12">
        <rect x="-140" y="0" width="40" height="35" fill="#FFFFFF" />
        <rect x="-60" y="0" width="40" height="35" fill="#FFFFFF" />
        <rect x="20" y="0" width="40" height="35" fill="#FFFFFF" />
        <rect x="100" y="0" width="40" height="35" fill="#FFFFFF" />
        <rect x="-100" y="35" width="40" height="35" fill="#FFFFFF" />
        <rect x="-20" y="35" width="40" height="35" fill="#FFFFFF" />
        <rect x="60" y="35" width="40" height="35" fill="#FFFFFF" />
        <rect x="140" y="35" width="40" height="35" fill="#FFFFFF" />
      </g>
        """
    elif discipline == "kart":
        return """
      <!-- Alternating Red / White Apex Rumble Curbs -->
      <g>
        <polygon points="70,90 95,95 85,130 62,125" fill="#EF4444" />
        <polygon points="62,125 85,130 76,165 55,160" fill="#FFFFFF" />
        <polygon points="55,160 76,165 68,200 48,195" fill="#EF4444" />
        <polygon points="48,195 68,200 62,235 44,230" fill="#FFFFFF" />
        <polygon points="44,230 62,235 58,270 42,265" fill="#EF4444" />
        <polygon points="42,265 58,270 56,305 42,300" fill="#FFFFFF" />
      </g>
      <g>
        <polygon points="442,90 417,95 427,130 450,125" fill="#EF4444" />
        <polygon points="450,125 427,130 436,165 457,160" fill="#FFFFFF" />
        <polygon points="457,160 436,165 444,200 464,195" fill="#EF4444" />
        <polygon points="464,195 444,200 450,235 468,230" fill="#FFFFFF" />
        <polygon points="468,230 450,235 454,270 470,265" fill="#EF4444" />
        <polygon points="470,265 454,270 456,305 470,300" fill="#FFFFFF" />
      </g>
      <g opacity="0.18">
        <polygon points="160,20 190,20 330,490 300,490" fill="#CCFF00" />
        <polygon points="205,20 225,20 365,490 345,490" fill="#FFFFFF" />
      </g>
        """
    elif discipline == "rally":
        return """
      <!-- Mud Roost Splatter Texture -->
      <g opacity="0.15" fill="#FF8800">
        <circle cx="90" cy="180" r="14" /><circle cx="110" cy="220" r="8" />
        <circle cx="80" cy="260" r="18" /><circle cx="100" cy="310" r="12" />
        <circle cx="420" cy="180" r="14" /><circle cx="400" cy="220" r="8" />
        <circle cx="430" cy="260" r="18" /><circle cx="410" cy="310" r="12" />
      </g>
      <g opacity="0.25">
        <polygon points="140,20 180,20 320,490 280,490" fill="#FF6B00" />
        <polygon points="195,20 215,20 355,490 335,490" fill="#FFFFFF" />
      </g>
        """
    elif discipline == "nascar":
        return """
      <!-- High-Banked Tri-Oval Curves -->
      <path d="M 256,60 C 420,60 460,200 460,260 C 460,340 400,430 256,460 C 112,430 52,340 52,260 C 52,200 92,60 256,60 Z"
            fill="none" stroke="#1E293B" stroke-width="28" opacity="0.6" />
      <path d="M 256,60 C 420,60 460,200 460,260 C 460,340 400,430 256,460 C 112,430 52,340 52,260 C 52,200 92,60 256,60 Z"
            fill="none" stroke="#FFD700" stroke-width="2.5" stroke-dasharray="14 10" opacity="0.4" />
      <g opacity="0.25">
        <polygon points="120,20 150,20 290,490 260,490" fill="#DC2626" />
        <polygon points="155,20 185,20 325,490 295,490" fill="#FFFFFF" />
        <polygon points="190,20 220,20 360,490 330,490" fill="#2563EB" />
      </g>
        """
    elif discipline == "extreme_offroad":
        return """
      <!-- Hazard Caution Chevrons -->
      <g opacity="0.16">
        <polygon points="60,20 100,20 240,490 200,490" fill="#00FF66" />
        <polygon points="120,20 160,20 300,490 260,490" fill="#F59E0B" />
        <polygon points="180,20 220,20 360,490 320,490" fill="#00FF66" />
        <polygon points="240,20 280,20 420,490 380,490" fill="#F59E0B" />
      </g>
      <g opacity="0.12" fill="#FFFFFF">
        <circle cx="90" cy="140" r="2.5" /><circle cx="130" cy="140" r="2.5" /><circle cx="170" cy="140" r="2.5" />
        <circle cx="110" cy="170" r="2.5" /><circle cx="150" cy="170" r="2.5" /><circle cx="190" cy="170" r="2.5" />
        <circle cx="330" cy="140" r="2.5" /><circle cx="370" cy="140" r="2.5" /><circle cx="410" cy="140" r="2.5" />
        <circle cx="350" cy="170" r="2.5" /><circle cx="390" cy="170" r="2.5" /><circle cx="430" cy="170" r="2.5" />
      </g>
        """
    return ""

def generate_crown(discipline: str) -> str:
    if discipline == "gt":
        return '<path d="M 180,60 L 256,42 L 332,60" stroke="#00E5FF" stroke-width="4.5" stroke-linecap="round" fill="none" opacity="0.95" />'
    elif discipline == "kart":
        return """
    <path d="M 180,64 L 256,44 L 332,64" stroke="#CCFF00" stroke-width="5" stroke-linecap="round" fill="none" />
    <path d="M 200,76 L 256,60 L 312,76" stroke="#EF4444" stroke-width="3" stroke-linecap="round" fill="none" opacity="0.9" />
        """
    elif discipline == "rally":
        return """
    <path d="M 180,64 L 256,44 L 332,64" stroke="#FF6B00" stroke-width="5" stroke-linecap="round" fill="none" />
    <path d="M 200,76 L 256,60 L 312,76" stroke="#FFFFFF" stroke-width="2.5" stroke-linecap="round" fill="none" opacity="0.9" />
        """
    elif discipline == "nascar":
        return """
    <path d="M 140,50 L 256,28 L 372,50" stroke="#FFD700" stroke-width="6" stroke-linecap="round" fill="none" />
    <path d="M 170,62 L 256,46 L 342,62" stroke="#EF4444" stroke-width="3.5" stroke-linecap="round" fill="none" opacity="0.95" />
        """
    elif discipline == "extreme_offroad":
        return """
    <path d="M 160,54 L 256,36 L 352,54" stroke="#00FF66" stroke-width="5.5" stroke-linecap="round" fill="none" opacity="0.98" />
    <path d="M 185,68 L 256,52 L 327,68" stroke="#F59E0B" stroke-width="3" stroke-linecap="round" fill="none" opacity="0.9" />
        """
    return ""

def generate_laurel_wreath(metal: str) -> str:
    # Gold has full Roman laurel wreath; Silver has sleek branch hints; Bronze has warm copper pinstripes
    opacity = "1.0" if metal == "gold" else ("0.75" if metal == "silver" else "0.55")
    return f"""
    <!-- Laurel Wreath -->
    <g stroke="url(#metal-rim)" stroke-width="3.5" fill="#141416" stroke-linecap="round" opacity="{opacity}">
      <!-- Left Laurel -->
      <path d="M 152,360 C 104,315 96,200 148,140" fill="none" stroke-width="4.5" />
      <path d="M 138,335 C 112,335 106,310 130,318 Z" />
      <path d="M 122,290 C 98,280 98,255 120,268 Z" />
      <path d="M 114,240 C 92,225 100,200 120,218 Z" />
      <path d="M 120,190 C 102,170 118,145 134,170 Z" />
      <path d="M 138,150 C 128,130 150,115 160,140 Z" />

      <!-- Right Laurel -->
      <path d="M 360,360 C 408,315 416,200 364,140" fill="none" stroke-width="4.5" />
      <path d="M 374,335 C 400,335 406,310 382,318 Z" />
      <path d="M 390,290 C 414,280 414,255 392,268 Z" />
      <path d="M 398,240 C 420,225 412,200 392,218 Z" />
      <path d="M 392,190 C 410,170 394,145 378,170 Z" />
      <path d="M 374,150 C 384,130 362,115 352,140 Z" />
    </g>
    """

def generate_svg(discipline: str, tier: int, metal: str) -> str:
    champ_title, tier_sub, medal_code = CHAMPIONSHIP_NAMES[discipline][tier]
    roman_tier = ROMAN_NUMERALS[tier]

    # Metallic color palettes
    if metal == "gold":
        glow_col = "#EAB308"
        body_stops = """
      <stop offset="0%" stop-color="#FFFDF0" />
      <stop offset="25%" stop-color="#FDE047" />
      <stop offset="60%" stop-color="#EAB308" />
      <stop offset="90%" stop-color="#A16207" />
      <stop offset="100%" stop-color="#543003" />"""
        rim_stops = """
      <stop offset="0%" stop-color="#FFFFFF" />
      <stop offset="40%" stop-color="#FACC15" />
      <stop offset="100%" stop-color="#854D0E" />"""
        plate_stops = """
      <stop offset="0%" stop-color="#CA8A04" />
      <stop offset="50%" stop-color="#FEF08A" />
      <stop offset="100%" stop-color="#A16207" />"""
        pedestal_stops = """
      <stop offset="0%" stop-color="#241B08" />
      <stop offset="50%" stop-color="#140F04" />
      <stop offset="100%" stop-color="#080602" />"""
        plate_text_primary = "#1C1202"
        plate_text_secondary = "#422006"
        bezel_color = "#EAB308"
        shield_rim = "#EAB308"
        crest_bg = "#1C1202"
    elif metal == "silver":
        glow_col = "#94A3B8"
        body_stops = """
      <stop offset="0%" stop-color="#FFFFFF" />
      <stop offset="25%" stop-color="#F1F5F9" />
      <stop offset="60%" stop-color="#CBD5E1" />
      <stop offset="90%" stop-color="#64748B" />
      <stop offset="100%" stop-color="#1E293B" />"""
        rim_stops = """
      <stop offset="0%" stop-color="#FFFFFF" />
      <stop offset="40%" stop-color="#E2E8F0" />
      <stop offset="100%" stop-color="#475569" />"""
        plate_stops = """
      <stop offset="0%" stop-color="#64748B" />
      <stop offset="50%" stop-color="#F8FAFC" />
      <stop offset="100%" stop-color="#334155" />"""
        pedestal_stops = """
      <stop offset="0%" stop-color="#1E293B" />
      <stop offset="50%" stop-color="#0F172A" />
      <stop offset="100%" stop-color="#020617" />"""
        plate_text_primary = "#090D16"
        plate_text_secondary = "#1E293B"
        bezel_color = "#94A3B8"
        shield_rim = "#94A3B8"
        crest_bg = "#0F172A"
    else:  # bronze
        glow_col = "#D4824C"
        body_stops = """
      <stop offset="0%" stop-color="#FFE8DB" />
      <stop offset="25%" stop-color="#F2B78E" />
      <stop offset="60%" stop-color="#D4824C" />
      <stop offset="90%" stop-color="#8C4318" />
      <stop offset="100%" stop-color="#3D1B07" />"""
        rim_stops = """
      <stop offset="0%" stop-color="#FFFFFF" />
      <stop offset="40%" stop-color="#F2B78E" />
      <stop offset="100%" stop-color="#7C2D12" />"""
        plate_stops = """
      <stop offset="0%" stop-color="#7C2D12" />
      <stop offset="50%" stop-color="#FFEDD5" />
      <stop offset="100%" stop-color="#431407" />"""
        pedestal_stops = """
      <stop offset="0%" stop-color="#2D1204" />
      <stop offset="50%" stop-color="#180A02" />
      <stop offset="100%" stop-color="#0A0401" />"""
        plate_text_primary = "#200802"
        plate_text_secondary = "#431407"
        bezel_color = "#D4824C"
        shield_rim = "#D4824C"
        crest_bg = "#1F0A02"

    stars_xml = generate_stars(tier, metal)
    handles_xml = generate_handles(discipline)
    stem_xml = generate_stem(discipline)
    bg_inlay_xml = generate_background_inlay(discipline)
    crown_xml = generate_crown(discipline)
    laurel_xml = generate_laurel_wreath(metal)

    return f"""<svg id="trophy-{discipline}-t{tier}-{metal}" viewBox="0 0 512 512" fill="none" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <!-- Drop Shadow and Metallic Glow -->
    <filter id="shadow-{discipline}-t{tier}-{metal}" x="-20%" y="-20%" width="140%" height="140%">
      <feDropShadow dx="0" dy="12" stdDeviation="16" flood-color="#000000" flood-opacity="0.9" />
      <feDropShadow dx="0" dy="0" stdDeviation="14" flood-color="{glow_col}" flood-opacity="0.45" />
    </filter>

    <linearGradient id="metal-body" x1="0%" y1="0%" x2="100%" y2="100%">{body_stops}
    </linearGradient>

    <linearGradient id="metal-rim" x1="0%" y1="0%" x2="0%" y2="100%">{rim_stops}
    </linearGradient>

    <linearGradient id="metal-plate" x1="0%" y1="0%" x2="100%" y2="0%">{plate_stops}
    </linearGradient>

    <linearGradient id="metal-pedestal" x1="0%" y1="0%" x2="100%" y2="100%">{pedestal_stops}
    </linearGradient>

    <!-- Tread Rubber Gradient for Rally Handles -->
    <linearGradient id="tread-rubber" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#334155" />
      <stop offset="50%" stop-color="#1E293B" />
      <stop offset="100%" stop-color="#0F172A" />
    </linearGradient>

    <clipPath id="shield-clip-{discipline}-{tier}">
      <path d="M 256,32 L 432,80 C 462,190 450,330 256,480 C 62,330 50,190 80,80 Z" />
    </clipPath>
  </defs>

  <g filter="url(#shadow-{discipline}-t{tier}-{metal})">
    <!-- Base Shield -->
    <path d="M 256,30 L 434,78 C 466,190 454,332 256,484 C 58,332 46,190 78,78 Z"
          fill="#0A0E14" stroke="{bezel_color}" stroke-width="6" />

    <!-- Inner Shield Clip Area -->
    <g clip-path="url(#shield-clip-{discipline}-{tier})">
      <rect x="40" y="20" width="432" height="470" fill="#06090E" />
{bg_inlay_xml}
    </g>

    <!-- Beveled Metal Rim Stroke -->
    <path d="M 256,40 L 420,84 C 448,185 438,315 256,466 C 74,315 64,185 92,84 Z"
          fill="none" stroke="url(#metal-rim)" stroke-width="4" opacity="0.95" />
    <path d="M 256,50 L 410,90 C 434,180 424,302 256,450 C 88,302 78,180 102,90 Z"
          fill="none" stroke="{shield_rim}" stroke-width="1.5" stroke-dasharray="10 8" opacity="0.6" />

{crown_xml}
{laurel_xml}

    <!-- ==================== DISCIPLINE HANDLES ==================== -->
{handles_xml}

    <!-- ==================== MAIN GOBLET CUP BODY ==================== -->
    <path d="M 176,145 L 336,145 C 336,245 300,305 256,318 C 212,305 176,245 176,145 Z"
          fill="url(#metal-body)" stroke="#FFFFFF" stroke-width="4" stroke-linejoin="round" />

    <!-- Specular Highlight -->
    <path d="M 216,147 C 216,228 238,295 256,316 C 274,295 296,228 296,147 Z"
          fill="#FFFFFF" opacity="0.3" />
    <line x1="256" y1="146" x2="256" y2="316" stroke="#FFFFFF" stroke-width="3" opacity="0.75" />

    <!-- Cup Rim Lip -->
    <ellipse cx="256" cy="145" rx="80" ry="18" fill="url(#metal-rim)" stroke="#FFFFFF" stroke-width="3.5" />
    <ellipse cx="256" cy="145" rx="64" ry="12" fill="#18181B" />

    <!-- Embossed Medallion -->
    <circle cx="256" cy="226" r="30" fill="#18181B" stroke="url(#metal-plate)" stroke-width="4" />
    <text x="256" y="234" fill="{bezel_color}" font-family="'Inter', 'Outfit', 'Segoe UI', sans-serif"
          font-weight="900" font-size="18" text-anchor="middle" letter-spacing="1">{medal_code}</text>

    <!-- ==================== STEM ==================== -->
{stem_xml}

    <!-- ==================== PEDESTAL BASE ==================== -->
    <polygon points="216,356 296,356 304,372 208,372"
             fill="url(#metal-rim)" stroke="#FFFFFF" stroke-width="2.5" />
    <rect x="176" y="372" width="160" height="46" rx="5"
          fill="url(#metal-pedestal)" stroke="{bezel_color}" stroke-width="3.5" />

    <!-- Nameplate: CHAMPIONSHIP TITLE -->
    <rect x="188" y="380" width="136" height="30" rx="3"
          fill="url(#metal-plate)" stroke="#FFFFFF" stroke-width="1.8" />
    <text x="256" y="393" fill="{plate_text_primary}" font-family="'Inter', 'Outfit', 'Segoe UI', sans-serif"
          font-weight="900" font-size="9" text-anchor="middle" letter-spacing="0.4">{champ_title}</text>
    <text x="256" y="405" fill="{plate_text_secondary}" font-family="'Inter', 'Outfit', 'Segoe UI', sans-serif"
          font-weight="800" font-size="7.5" text-anchor="middle" letter-spacing="1">{tier_sub}</text>

    <!-- Bottom Footing -->
    <rect x="152" y="418" width="208" height="14" rx="4"
          fill="#0D1117" stroke="{bezel_color}" stroke-width="2.5" />

    <!-- ==================== TIER STARS ==================== -->
{stars_xml}

    <!-- ==================== ROMAN NUMERAL BADGE ==================== -->
    <g transform="translate(256, 444)">
      <rect x="-28" y="-11" width="56" height="22" rx="3" fill="{crest_bg}" stroke="{bezel_color}" stroke-width="2" />
      <text x="0" y="5" fill="{bezel_color}" font-family="'Inter', 'Outfit', 'Segoe UI', sans-serif"
            font-weight="900" font-size="11" text-anchor="middle">{roman_tier}</text>
    </g>
  </g>
</svg>
"""

def generate_locked_svg() -> str:
    return """<svg id="trophy-locked" viewBox="0 0 512 512" fill="none" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <filter id="shadow-locked" x="-20%" y="-20%" width="140%" height="140%">
      <feDropShadow dx="0" dy="12" stdDeviation="16" flood-color="#000000" flood-opacity="0.9" />
    </filter>

    <linearGradient id="locked-carbon" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#151C26" />
      <stop offset="50%" stop-color="#0C1017" />
      <stop offset="100%" stop-color="#06090E" />
    </linearGradient>

    <clipPath id="shield-clip-locked">
      <path d="M 256,32 L 432,80 C 462,190 450,330 256,480 C 62,330 50,190 80,80 Z" />
    </clipPath>
  </defs>

  <g filter="url(#shadow-locked)" opacity="0.6">
    <!-- Base Shield -->
    <path d="M 256,30 L 434,78 C 466,190 454,332 256,484 C 58,332 46,190 78,78 Z"
          fill="#080C12" stroke="#334155" stroke-width="4" stroke-dasharray="12 8" />

    <!-- Inner Shield Clip Area -->
    <g clip-path="url(#shield-clip-locked)">
      <rect x="40" y="20" width="432" height="470" fill="#04060A" />

      <!-- Carbon Grid Lines -->
      <g opacity="0.08" stroke="#FFFFFF" stroke-width="1.8">
        <line x1="40" y1="60" x2="472" y2="60" /><line x1="40" y1="120" x2="472" y2="120" />
        <line x1="40" y1="180" x2="472" y2="180" /><line x1="40" y1="240" x2="472" y2="240" />
        <line x1="40" y1="300" x2="472" y2="300" /><line x1="40" y1="360" x2="472" y2="360" />
      </g>
    </g>

    <!-- Dark Silhouette Goblet -->
    <path d="M 176,145 L 336,145 C 336,245 300,305 256,318 C 212,305 176,245 176,145 Z"
          fill="url(#locked-carbon)" stroke="#334155" stroke-width="3" />
    <path d="M 238,318 L 274,318 L 270,356 L 242,356 Z"
          fill="#0F172A" stroke="#334155" stroke-width="2" />
    <rect x="176" y="372" width="160" height="46" rx="5"
          fill="#0B0F17" stroke="#334155" stroke-width="2.5" />
    <rect x="152" y="418" width="208" height="14" rx="4"
          fill="#06090E" stroke="#1E293B" stroke-width="2" />

    <!-- Etched Golden Padlock -->
    <g transform="translate(256, 230)">
      <!-- Shackle -->
      <path d="M -16,-6 C -16,-24 16,-24 16,-6 L 16,6 L -16,6 Z"
            fill="none" stroke="#EAB308" stroke-width="5" stroke-linecap="round" />
      <!-- Lock Body -->
      <rect x="-24" y="6" width="48" height="38" rx="6" fill="#1C1917" stroke="#EAB308" stroke-width="3.5" />
      <!-- Keyhole -->
      <circle cx="0" cy="22" r="5" fill="#EAB308" />
      <polygon points="-3,22 3,22 2,34 -2,34" fill="#EAB308" />
    </g>

    <!-- Locked Banner Text -->
    <rect x="200" y="380" width="112" height="30" rx="3" fill="#1C1917" stroke="#44403C" stroke-width="1.8" />
    <text x="256" y="400" fill="#78716C" font-family="'Inter', 'Outfit', 'Segoe UI', sans-serif"
          font-weight="900" font-size="12" text-anchor="middle" letter-spacing="2">LOCKED</text>
  </g>
</svg>
"""

def main():
    generated_svgs = []

    # 1. Generate 75 bespoke SVGs
    for discipline in DISCIPLINES:
        for tier in TIERS:
            for metal in METALS:
                svg_content = generate_svg(discipline, tier, metal)
                filename = f"{discipline}_t{tier}_{metal}.svg"
                filepath = OUTPUT_DIR / filename
                filepath.write_text(svg_content)
                generated_svgs.append(filepath)

    # 2. Generate locked silhouette SVG
    locked_svg_content = generate_locked_svg()
    locked_filepath = OUTPUT_DIR / "trophy_locked.svg"
    locked_filepath.write_text(locked_svg_content)
    generated_svgs.append(locked_filepath)

    print(f"Generated {len(generated_svgs)} master SVGs in {OUTPUT_DIR}")

    # 3. Render 128px and 256px RGBA PNGs for all generated SVGs
    print("Rendering 128px and 256px PNGs with rsvg-convert...")
    for svg_path in generated_svgs:
        stem = svg_path.stem
        png_128 = OUTPUT_DIR / f"{stem}-128.png"
        png_256 = OUTPUT_DIR / f"{stem}-256.png"

        subprocess.run(["rsvg-convert", "-w", "128", "-h", "128", "-o", str(png_128), str(svg_path)], check=True)
        subprocess.run(["rsvg-convert", "-w", "256", "-h", "256", "-o", str(png_256), str(svg_path)], check=True)

    print(f"Successfully generated all 75 bespoke trophy badges + locked trophy in 128px and 256px PNG!")

if __name__ == "__main__":
    main()
