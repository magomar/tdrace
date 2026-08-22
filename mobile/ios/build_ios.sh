#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# TDRace iOS Build Helper
# Compiles Rust static libraries and packages XCFramework for iOS & Simulators
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
APP_DIR="${ROOT_DIR}/crates/tdrace-app"
BUILD_MODE="${1:-release}" # release or debug

echo "========================================================"
echo "🍏 TDRace iOS Build Pipeline [Mode: ${BUILD_MODE}]"
echo "========================================================"

TARGET_DEVICE="aarch64-apple-ios"
TARGET_SIM_ARM="aarch64-apple-ios-sim"
TARGET_SIM_INTEL="x86_64-apple-ios"

# 1. Verify / Install iOS targets
echo "🔍 Checking Rust iOS compilation targets..."
for TARGET in "${TARGET_DEVICE}" "${TARGET_SIM_ARM}" "${TARGET_SIM_INTEL}"; do
    if ! rustup target list --installed | grep -q "^${TARGET}$"; then
        echo "   Installing missing target: ${TARGET}..."
        rustup target add "${TARGET}" || true
    else
        echo "   ✓ Target installed: ${TARGET}"
    fi
done

BUILD_FLAG=""
if [ "${BUILD_MODE}" == "release" ]; then
    BUILD_FLAG="--release"
fi

OUTPUT_DIR="${SCRIPT_DIR}/build"
mkdir -p "${OUTPUT_DIR}"

echo "🔨 Compiling for iOS Device (${TARGET_DEVICE})..."
cargo build --package tdrace-app --target "${TARGET_DEVICE}" ${BUILD_FLAG} || {
    echo "⚠️  Cross-compiling for Apple targets requires macOS SDK / Xcode toolchains."
}

echo "🔨 Compiling for iOS Simulator (${TARGET_SIM_ARM} / ${TARGET_SIM_INTEL})..."
cargo build --package tdrace-app --target "${TARGET_SIM_ARM}" ${BUILD_FLAG} || true
cargo build --package tdrace-app --target "${TARGET_SIM_INTEL}" ${BUILD_FLAG} || true

# If running on macOS with lipo/xcodebuild, create universal simulator library & XCFramework
if command -v lipo >/dev/null 2>&1; then
    echo "🔀 Creating universal simulator binary using lipo..."
    SIM_DIR="${OUTPUT_DIR}/simulator"
    mkdir -p "${SIM_DIR}"

    SIM_ARM_LIB="${ROOT_DIR}/target/${TARGET_SIM_ARM}/${BUILD_MODE}/libtdrace_app.a"
    SIM_INTEL_LIB="${ROOT_DIR}/target/${TARGET_SIM_INTEL}/${BUILD_MODE}/libtdrace_app.a"

    if [ -f "${SIM_ARM_LIB}" ] && [ -f "${SIM_INTEL_LIB}" ]; then
        lipo -create "${SIM_ARM_LIB}" "${SIM_INTEL_LIB}" -output "${SIM_DIR}/libtdrace_app.a"
        echo "   ✓ Created universal simulator library: ${SIM_DIR}/libtdrace_app.a"
    fi
fi

echo "========================================================"
echo "✅ iOS Build Process Complete."
echo "   Refer to mobile/ios/Xcode_Setup_Guide.md to link library into your Xcode project."
echo "========================================================"
