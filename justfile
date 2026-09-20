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

# Rebuild both static web portals (Wiki + Showroom) after re-ingesting assets
build-portals:
    @echo "🌐 Rebuilding static web portals (Wiki + Showroom)..."
    cd portals && bun run build:all

# Rebuild static site for Option A (Astro + Starlight Wiki)
build-wiki:
    @echo "📚 Building static site for TdRace Wiki..."
    cd portals && bun run build:starlight

# Rebuild static site for Option B (Motorsport Showroom)
build-showroom:
    @echo "🏎️  Building static site for Motorsport Showroom..."
    cd portals && bun run build:showroom

# ------------------------------------------------------------------------------
# 📦 WebAssembly Game Site
# ------------------------------------------------------------------------------

# Build WebAssembly distribution for web browsers
build-web:
    @echo "🌐 Building WebAssembly distribution..."
    ./web/build_web.sh

# Start local web server for WebAssembly game in browser (port 8080)
serve-web: build-web
    @echo "🌐 Serving WebAssembly game at http://localhost:8080 (Ctrl+C to stop)..."
    cd web/dist && python3 -m http.server 8080

