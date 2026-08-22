#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# TDRace WebAssembly Build Helper
# Compiles Rust app to wasm32-unknown-unknown and packages web distribution
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
APP_DIR="${ROOT_DIR}/crates/tdrace-app"
DIST_DIR="${SCRIPT_DIR}/dist"
BUILD_MODE="${1:-release}" # release or debug

echo "========================================================"
echo "🌐 TDRace WebAssembly (WASM) Build Pipeline"
echo "   Mode: ${BUILD_MODE}"
echo "========================================================"

TARGET="wasm32-unknown-unknown"

# 1. Check Rust target
echo "🔍 Checking WASM compilation target (${TARGET})..."
if ! rustup target list --installed | grep -q "^${TARGET}$"; then
    echo "   Installing missing target: ${TARGET}..."
    rustup target add "${TARGET}"
else
    echo "   ✓ Target installed: ${TARGET}"
fi

# 2. Compile WASM binary
echo "🔨 Compiling tdrace-app for WebAssembly..."
BUILD_FLAG=""
if [ "${BUILD_MODE}" == "release" ]; then
    BUILD_FLAG="--release"
fi

cargo build --package tdrace-app --target "${TARGET}" ${BUILD_FLAG}

WASM_SRC="${ROOT_DIR}/target/${TARGET}/${BUILD_MODE}/tdrace_app.wasm"
if [ ! -f "${WASM_SRC}" ]; then
    WASM_SRC="${ROOT_DIR}/target/${TARGET}/${BUILD_MODE}/tdrace-app.wasm"
fi
if [ ! -f "${WASM_SRC}" ]; then
    echo "❌ Error: compiled WASM file not found at ${WASM_SRC}"
    exit 1
fi

# 3. Create dist bundle
mkdir -p "${DIST_DIR}"
cp "${SCRIPT_DIR}/index.html" "${DIST_DIR}/index.html"
cp "${WASM_SRC}" "${DIST_DIR}/tdrace_app.wasm"

# 4. Run wasm-opt if available
if command -v wasm-opt >/dev/null 2>&1 && [ "${BUILD_MODE}" == "release" ]; then
    echo "⚡ Optimizing WebAssembly payload with wasm-opt (-O3)..."
    wasm-opt -O3 "${DIST_DIR}/tdrace_app.wasm" -o "${DIST_DIR}/tdrace_app.wasm"
fi

WASM_SIZE=$(du -h "${DIST_DIR}/tdrace_app.wasm" | cut -f1)
echo "========================================================"
echo "✅ WebAssembly build complete! Size: ${WASM_SIZE}"
echo "   Distribution directory: web/dist/"
echo "   To test locally: (cd web/dist && python3 -m http.server 8080)"
echo "========================================================"
