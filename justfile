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

# Launch the TDRace Codex: Custom Astro + Tailwind, spec 087 (port 4322)
codex:
    @echo "🏎️  Launching TDRace Codex (Custom Astro + Tailwind) on http://localhost:4322..."
    cd portals/codex && bun run dev -- --host 0.0.0.0 --port 4322

# Re-export game data for the Codex (portals/shared/data/codex)
export-codex:
    cargo run -q -p tdrace-app --bin export_codex

# Verify OKF v0.2 documentation compliance and relative cross-links
verify-okf:
    python3 scripts/verify_okf.py

# Re-ingest game tracks and vehicle specs into JSON datasets
ingest-assets:
    python3 scripts/generate_asset_data.py

# Rebuild both static web portals (Wiki + Codex) after re-ingesting assets
build-portals:
    @echo "🌐 Rebuilding static web portals (Wiki + Codex)..."
    cd portals && bun run build:all

# Rebuild static site for Option A (Astro + Starlight Wiki)
build-wiki:
    @echo "📚 Building static site for TdRace Wiki..."
    cd portals && bun run build:starlight

# Rebuild static site for the TDRace Codex
build-codex:
    @echo "🏎️  Building static site for the TDRace Codex..."
    cd portals && bun run build:codex

# Build the Codex with Steam v1 launch content only (Classic, Karting, Autocross, Rallycross)
build-codex-launch:
    cargo run -q -p tdrace-app --bin export_codex -- --scope launch --out portals/codex/.launch-data
    cd portals/codex && CODEX_DATA_DIR=.launch-data bun run build

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

# ------------------------------------------------------------------------------
# 📦 Windows Desktop Distribution
# ------------------------------------------------------------------------------

# Build Windows x86_64 executable and package into standalone zip with assets
build-windows MODE="release":
    @echo "🪟 Building and packaging Windows distribution ({{MODE}})..."
    ./scripts/package_windows.sh {{MODE}}

# Alias for build-windows
package-windows: (build-windows "release")


