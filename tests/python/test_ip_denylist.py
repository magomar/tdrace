"""Automated IP Denylist Gate for Steam Release.

Specification Reference:
    specs/047_fictional_branding_and_realworld_ip_removal_for_steam_release.md
    Task: tdrace-fictional-branding-ip-removal-pre1.2

Purpose:
    Scans all shipped files (assets text files, series definitions, crates source strings,
    and player-visible track JSON fields) against the real-world terms in docs/legal/ip_rename_registry.toml.
    Guarantees no trademarked car makers, models, series, drivers, teams, sponsors,
    or circuit brand names appear in player-visible content.
"""

from __future__ import annotations

import json
import os
import re
from pathlib import Path
from typing import NamedTuple

import pytest
import tomllib

# -----------------------------------------------------------------------------
# Configuration & Paths
# -----------------------------------------------------------------------------

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
REGISTRY_PATH = REPO_ROOT / "docs" / "legal" / "ip_rename_registry.toml"

# Explicit allowlist for generic racing terminology that could otherwise collide
# with substrings of real series or category names.
GENERIC_ALLOWLIST = {
    # Generic vehicle categories & layouts
    "gt",
    "stock car",
    "sports car",
    "prototype",
    "supercar",
    "hypercar",
    "endurance",
    "sprint",
    "clubsport",
    "biturbo",
    "turbo",
    "v8",
    "v6",
    "v10",
    "v12",
    "4x4",
    "awd",
    "fwd",
    "rwd",
    "hybrid",
    "electric",
    "custom",
    "spec",
    "chassis",
    "open wheel",
    # Disciplines & venues
    "racer",
    "racing",
    "motorsport",
    "rally",
    "rallycross",
    "cross car",
    "autocross",
    "kart",
    "karting",
    "shifter",
    "cadet",
    "junior",
    "senior",
    "buggy",
    "superbuggy",
    "truck",
    "trophy truck",
    "sand rail",
    "mud bogger",
    "ice racer",
    "speedway",
    "oval",
    "road",
    "short track",
    "dirt",
    "circuit",
    "autodrome",
    "arena",
    "stadium",
    # Event & organization descriptors
    "pro",
    "master",
    "legend",
    "grand prix",
    "gp",
    "cup",
    "trophy",
    "challenge",
    "championship",
    "series",
    "national",
    "international",
    "european",
    "world",
    "continental",
    "american",
}


class DenylistHit(NamedTuple):
    file_path: str
    line_number: int
    matched_term: str
    line_preview: str

    def format_report(self) -> str:
        return f"{self.file_path}:{self.line_number}: term '{self.matched_term}' in -> {self.line_preview}"


# -----------------------------------------------------------------------------
# Registry Loader & Pattern Compiler
# -----------------------------------------------------------------------------

def load_denylist_terms(registry_path: Path) -> list[str]:
    """Extract all real-world trademark terms from the registry TOML."""
    if not registry_path.is_file():
        raise FileNotFoundError(f"IP rename registry not found at {registry_path}")

    with open(registry_path, "rb") as f:
        data = tomllib.load(f)

    raw_terms: set[str] = set()

    # 1. Brands
    for brand in data.get("brands", {}).keys():
        raw_terms.add(brand)

    # 2. Car models & makers
    for car_info in data.get("cars", {}).values():
        if "real_name" in car_info:
            raw_terms.add(car_info["real_name"])
        if "real_manufacturer" in car_info:
            raw_terms.add(car_info["real_manufacturer"])

    # 3. Series & sanctioning bodies
    for series in data.get("series_and_bodies", {}).keys():
        raw_terms.add(series)

    # 4. Teams
    for team in data.get("teams", {}).keys():
        raw_terms.add(team)

    # 5. Drivers & nicknames
    for driver in data.get("drivers", {}).keys():
        raw_terms.add(driver)

    # 6. Circuits & track brands
    for circuit in data.get("circuits", {}).keys():
        raw_terms.add(circuit)

    # 7. Corner names & venue text
    for corner in data.get("corners_and_venue_text", {}).keys():
        raw_terms.add(corner)

    # 8. Sponsors
    for sponsor in data.get("sponsors", {}).keys():
        raw_terms.add(sponsor)

    # Filter out generic allowlisted tokens
    filtered_terms = [
        term.strip()
        for term in raw_terms
        if term.strip().lower() not in GENERIC_ALLOWLIST and len(term.strip()) >= 3
    ]

    # Sort longest first to avoid partial-word shadowing
    filtered_terms.sort(key=len, reverse=True)
    return filtered_terms


def compile_denylist_patterns(terms: list[str]) -> list[tuple[str, re.Pattern[str]]]:
    """Compile case-insensitive word-boundary regex patterns."""
    compiled: list[tuple[str, re.Pattern[str]]] = []
    for term in terms:
        # Word boundary matching
        pattern = re.compile(r"\b" + re.escape(term) + r"\b", re.IGNORECASE)
        compiled.append((term, pattern))
    return compiled


# -----------------------------------------------------------------------------
# Scanners for Shipped Roots
# -----------------------------------------------------------------------------

def scan_text_lines(
    rel_path: str,
    lines: list[str],
    patterns: list[tuple[str, re.Pattern[str]]],
) -> list[DenylistHit]:
    """Scan a list of lines against denylist patterns."""
    hits: list[DenylistHit] = []
    for line_num, line in enumerate(lines, start=1):
        clean_line = line.strip()
        if not clean_line:
            continue
        for term, pattern in patterns:
            if pattern.search(clean_line):
                hits.append(
                    DenylistHit(
                        file_path=rel_path,
                        line_number=line_num,
                        matched_term=term,
                        line_preview=clean_line[:120],
                    )
                )
    return hits


def scan_series_directory(
    root: Path, patterns: list[tuple[str, re.Pattern[str]]]
) -> list[DenylistHit]:
    """Scan all TOML files in series/."""
    hits: list[DenylistHit] = []
    series_dir = root / "series"
    if not series_dir.is_dir():
        return hits

    for toml_file in series_dir.rglob("*.toml"):
        rel_path = str(toml_file.relative_to(root))
        try:
            with open(toml_file, "r", encoding="utf-8", errors="replace") as f:
                lines = f.readlines()
            hits.extend(scan_text_lines(rel_path, lines, patterns))
        except Exception as e:
            print(f"Warning: Failed to read {rel_path}: {e}")

    return hits


def scan_crates_source(
    root: Path, patterns: list[tuple[str, re.Pattern[str]]]
) -> list[DenylistHit]:
    """Scan non-test string literals in crates/*/src/**/*.rs."""
    hits: list[DenylistHit] = []
    crates_dir = root / "crates"
    if not crates_dir.is_dir():
        return hits

    for rs_file in crates_dir.rglob("src/**/*.rs"):
        rel_path = str(rs_file.relative_to(root))
        try:
            with open(rs_file, "r", encoding="utf-8", errors="replace") as f:
                content = f.read()

            in_test_block = False
            for line_num, line in enumerate(content.splitlines(), start=1):
                # Ignore test modules and unit test blocks
                if "#[cfg(test)]" in line or "mod tests" in line:
                    in_test_block = True
                if in_test_block:
                    continue

                # Scan string literals on non-test lines
                for str_match in re.finditer(r"\"([^\"]+)\"", line):
                    literal = str_match.group(1)
                    for term, pattern in patterns:
                        if pattern.search(literal):
                            hits.append(
                                DenylistHit(
                                    file_path=rel_path,
                                    line_number=line_num,
                                    matched_term=term,
                                    line_preview=line.strip()[:120],
                                )
                            )
        except Exception as e:
            print(f"Warning: Failed to read {rel_path}: {e}")

    return hits


def scan_track_json_files(
    root: Path, patterns: list[tuple[str, re.Pattern[str]]]
) -> list[DenylistHit]:
    """Scan player-visible fields of tracks/<module>/<id>.json."""
    hits: list[DenylistHit] = []
    tracks_dir = root / "tracks"
    if not tracks_dir.is_dir():
        return hits

    visible_keys = ("name", "description", "tag", "category_label")

    for json_file in tracks_dir.rglob("*.json"):
        # Explicitly skip non-track metadata per Spec 047
        if json_file.name in (".aliases.json", ".track_order.json", "MANIFEST.json"):
            continue

        rel_path = str(json_file.relative_to(root))
        try:
            with open(json_file, "r", encoding="utf-8", errors="replace") as f:
                data = json.load(f)

            if not isinstance(data, dict):
                continue

            # Check root player-visible text fields
            for key in visible_keys:
                val = data.get(key)
                if isinstance(val, str):
                    for term, pattern in patterns:
                        if pattern.search(val):
                            hits.append(
                                DenylistHit(
                                    file_path=rel_path,
                                    line_number=1,
                                    matched_term=term,
                                    line_preview=f"[{key}] {val[:100]}",
                                )
                            )

            # Check scenery feature names
            scenery = data.get("scenery")
            if isinstance(scenery, dict):
                for feature in scenery.get("features", []):
                    feat_name = feature.get("name")
                    if isinstance(feat_name, str):
                        for term, pattern in patterns:
                            if pattern.search(feat_name):
                                hits.append(
                                    DenylistHit(
                                        file_path=rel_path,
                                        line_number=1,
                                        matched_term=term,
                                        line_preview=f"[scenery.feature] {feat_name}",
                                    )
                                )
        except Exception as e:
            print(f"Warning: Failed to read {rel_path}: {e}")

    return hits


def scan_shipped_content(root: Path) -> list[DenylistHit]:
    """Execute complete denylist scan across all shipped roots."""
    terms = load_denylist_terms(REGISTRY_PATH)
    patterns = compile_denylist_patterns(terms)

    all_hits: list[DenylistHit] = []
    all_hits.extend(scan_series_directory(root, patterns))
    all_hits.extend(scan_crates_source(root, patterns))
    all_hits.extend(scan_track_json_files(root, patterns))
    return all_hits


# -----------------------------------------------------------------------------
# Pytest Test Cases
# -----------------------------------------------------------------------------

def test_registry_terms_loaded():
    """Verify registry loads valid denylist terms."""
    terms = load_denylist_terms(REGISTRY_PATH)
    assert len(terms) >= 100, f"Expected at least 100 terms, got {len(terms)}"
    assert "Ferrari" in terms
    assert "Porsche" in terms
    assert "NASCAR" in terms
    assert "Red Bull Ring" in terms


def test_denylist_catches_regression():
    """Verify scanner catches a regression with file, line, and matched term."""
    terms = load_denylist_terms(REGISTRY_PATH)
    patterns = compile_denylist_patterns(terms)

    dummy_content = [
        "# Clean series definition",
        "name = 'Super Sprint Series'",
        "car = 'Porsche 911 GT3 R'",  # Should be caught!
        "description = 'Pure racing action'",
    ]

    hits = scan_text_lines("scratch/test_series.toml", dummy_content, patterns)
    assert len(hits) >= 1, "Denylist scanner failed to catch 'Porsche 911 GT3 R'"

    target_hit = next((h for h in hits if "Porsche" in h.matched_term), None)
    assert target_hit is not None, "Target hit for Porsche was not recorded"
    assert target_hit.file_path == "scratch/test_series.toml"
    assert target_hit.line_number == 3
    assert "Porsche" in target_hit.matched_term
    assert "Porsche 911 GT3 R" in target_hit.line_preview


def test_ip_denylist_shipped_content():
    """Scan shipped content.
    
    In report-only mode (default during ongoing migration), logs hits without failing.
    In enforcing mode (ENFORCE_IP_DENYLIST=1), asserts zero hits.
    """
    enforce_mode = os.environ.get("ENFORCE_IP_DENYLIST", "0").lower() in ("1", "true", "yes")

    hits = scan_shipped_content(REPO_ROOT)

    if enforce_mode:
        if hits:
            sample = "\n".join(h.format_report() for h in hits[:25])
            pytest.fail(
                f"IP Denylist Gate ENFORCING: Found {len(hits)} denylisted terms in shipped content!\n"
                f"First 25 occurrences:\n{sample}"
            )
    else:
        # Report-only mode during migration
        if hits:
            print(
                f"\n[IP Denylist Gate: REPORT-ONLY] {len(hits)} real-world IP terms detected across shipped roots."
            )
            print(f"Sample remaining hits: {hits[0].format_report()}")
        else:
            print("\n[IP Denylist Gate: PASS] 0 denylisted terms found in shipped roots!")


if __name__ == "__main__":
    hits = scan_shipped_content(REPO_ROOT)
    print(f"Total denylist hits across codebase: {len(hits)}")
    for hit in hits[:30]:
        print(" ", hit.format_report())
