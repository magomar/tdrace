# ==============================================================================
# 🏎️ TDRace Justfile Task Runner
# ==============================================================================

set shell := ["bash", "-c"]

default:
    @just --list

# ------------------------------------------------------------------------------
# 🌐 Documentation & Asset Portals
# ------------------------------------------------------------------------------

# Launch Option A: Astro + Starlight Technical Reference Manual (port 4321)
wiki:
    @echo "📚 Launching TdRace Wiki (Astro + Starlight) on http://localhost:4321..."
    cd portals/option-a-starlight && bun run dev -- --host 0.0.0.0 --port 4321

# Launch Option B: Custom Motorsport Showroom & Physics Lab (port 4322)
showroom:
    @echo "🏎️  Launching TdRace Showroom (Custom Astro + Tailwind) on http://localhost:4322..."
    cd portals/option-b-showroom && bun run dev -- --host 0.0.0.0 --port 4322

# Verify OKF v0.2 documentation compliance and relative cross-links
verify-okf:
    python3 scripts/verify_okf.py

# Re-ingest game tracks and vehicle specs into JSON datasets
ingest-assets:
    python3 scripts/generate_asset_data.py
