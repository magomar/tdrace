#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# TDRace Android Build Helper
# Cross-compiles native Rust binaries and packages Android APK
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
APP_DIR="${ROOT_DIR}/crates/tdrace-app"
BUILD_MODE="${1:-release}" # release or debug

echo "========================================================"
echo "🏎️  TDRace Android Build Pipeline [Mode: ${BUILD_MODE}]"
echo "========================================================"

# Target architectures
ARCHS=(
    "aarch64-linux-android:arm64-v8a"
    "armv7-linux-androideabi:armeabi-v7a"
    "x86_64-linux-android:x86_64"
)

# 1. Check Rust toolchains
echo "🔍 Checking Rust Android cross-compilation targets..."
for pair in "${ARCHS[@]}"; do
    TARGET="${pair%%:*}"
    if ! rustup target list --installed | grep -q "^${TARGET}$"; then
        echo "   Installing missing target: ${TARGET}..."
        rustup target add "${TARGET}" || true
    else
        echo "   ✓ Target installed: ${TARGET}"
    fi
done

# 2. Check Android NDK environment
if [ -z "${ANDROID_NDK_HOME:-}" ] && [ -z "${ANDROID_NDK_ROOT:-}" ] && [ -z "${NDK_HOME:-}" ]; then
    echo "⚠️  Note: ANDROID_NDK_HOME is not set in environment."
    echo "   If building full APK via cargo-ndk/cargo-apk, ensure NDK is installed (API 24+)."
fi

# 3. Check for cargo-apk or cargo-ndk
USE_CARGO_APK=0
if command -v cargo-apk >/dev/null 2>&1; then
    USE_CARGO_APK=1
fi

if [ "${USE_CARGO_APK}" -eq 1 ]; then
    echo "📦 Building APK directly via cargo-apk..."
    cd "${APP_DIR}"
    if [ "${BUILD_MODE}" == "release" ]; then
        cargo apk build --release --manifest-path "${SCRIPT_DIR}/AndroidManifest.xml"
    else
        cargo apk build --manifest-path "${SCRIPT_DIR}/AndroidManifest.xml"
    fi
    echo "✅ APK build successful! Output in target/debug/apk or target/release/apk"
else
    echo "🔨 Compiling native cdylib libraries for all Android architectures..."
    JNI_LIBS_DIR="${SCRIPT_DIR}/app/src/main/jniLibs"
    mkdir -p "${JNI_LIBS_DIR}"

    for pair in "${ARCHS[@]}"; do
        TARGET="${pair%%:*}"
        ABI="${pair##*:}"
        ABI_OUT_DIR="${JNI_LIBS_DIR}/${ABI}"
        mkdir -p "${ABI_OUT_DIR}"

        echo "🚀 Building for ${TARGET} (${ABI})..."

        BUILD_FLAG=""
        if [ "${BUILD_MODE}" == "release" ]; then
            BUILD_FLAG="--release"
        fi

        if command -v cargo-ndk >/dev/null 2>&1; then
            cargo ndk --target "${TARGET}" --platform 24 build --package tdrace-app ${BUILD_FLAG}
        else
            cargo build --package tdrace-app --target "${TARGET}" ${BUILD_FLAG} || {
                echo "⚠️  Direct cross-compilation for ${TARGET} requires Android NDK clang linker configuration."
                echo "   Install cargo-ndk ('cargo install cargo-ndk') or set CC_${TARGET//-/_} in environment."
            }
        fi

        SO_FILE="${ROOT_DIR}/target/${TARGET}/${BUILD_MODE}/libtdrace_app.so"
        if [ -f "${SO_FILE}" ]; then
            cp "${SO_FILE}" "${ABI_OUT_DIR}/libtdrace_app.so"
            echo "   ✓ Copied ${SO_FILE} -> ${ABI_OUT_DIR}/libtdrace_app.so"
        fi
    done

    echo "✅ Android libraries prepared."
    echo "   To assemble APK with Gradle: cd mobile/android && ./gradlew assemble${BUILD_MODE^}"
fi
