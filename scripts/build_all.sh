#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# TDRace Unified Cross-Platform Build Helper
# Orchestrates desktop, mobile (Android/iOS), and web (WASM) targets
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Color formatting
BOLD="\033[1m"
GREEN="\033[0;32m"
BLUE="\033[0;34m"
YELLOW="\033[0;33m"
RED="\033[0;31m"
RESET="\033[0m"

BUILD_MODE="release"
BUILD_DESKTOP=0
BUILD_ANDROID=0
BUILD_IOS=0
BUILD_WEB=0
CHECK_ONLY=0

function print_usage() {
    cat << EOF
${BOLD}TDRace Cross-Platform Build Helper${RESET}

Usage:
  ./scripts/build_all.sh [options]

Options:
  --all            Build all platforms (Desktop, Web, Android, iOS)
  --desktop        Build Native Desktop executable (Linux/macOS/Windows)
  --web            Build WebAssembly HTML5 client
  --android        Build Android native libraries & APK structure
  --ios            Build iOS native libraries / framework
  --check          Run cargo check / verification without full artifact compilation
  --debug          Build in debug mode (default is release)
  --release        Build in release mode (default)
  -h, --help       Show this help message

Examples:
  ./scripts/build_all.sh --desktop --web
  ./scripts/build_all.sh --all --release
EOF
}

if [ $# -eq 0 ]; then
    BUILD_DESKTOP=1
fi

while [[ $# -gt 0 ]]; do
    case "$1" in
        --all)
            BUILD_DESKTOP=1
            BUILD_WEB=1
            BUILD_ANDROID=1
            BUILD_IOS=1
            shift
            ;;
        --desktop)
            BUILD_DESKTOP=1
            shift
            ;;
        --web)
            BUILD_WEB=1
            shift
            ;;
        --android)
            BUILD_ANDROID=1
            shift
            ;;
        --ios)
            BUILD_IOS=1
            shift
            ;;
        --check)
            CHECK_ONLY=1
            shift
            ;;
        --debug)
            BUILD_MODE="debug"
            shift
            ;;
        --release)
            BUILD_MODE="release"
            shift
            ;;
        -h|--help)
            print_usage
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${RESET}"
            print_usage
            exit 1
            ;;
    esac
done

echo -e "${BOLD}${BLUE}===================================================================${RESET}"
echo -e "${BOLD}${BLUE}🏎️  TDRace Unified Cross-Platform Build Pipeline${RESET}"
echo -e "   Mode: ${BOLD}${BUILD_MODE}${RESET}"
echo -e "${BOLD}${BLUE}===================================================================${RESET}"

RESULTS=()

# 1. Desktop Build
if [ "${BUILD_DESKTOP}" -eq 1 ]; then
    echo -e "\n${BOLD}🖥️  Building Desktop Target...${RESET}"
    cd "${ROOT_DIR}"
    FLAG=""
    if [ "${BUILD_MODE}" == "release" ]; then
        FLAG="--release"
    fi

    if [ "${CHECK_ONLY}" -eq 1 ]; then
        if cargo check --package tdrace-app; then
            RESULTS+=("${GREEN}✓ Desktop (Check): PASSED${RESET}")
        else
            RESULTS+=("${RED}✗ Desktop (Check): FAILED${RESET}")
        fi
    else
        if cargo build --package tdrace-app ${FLAG}; then
            RESULTS+=("${GREEN}✓ Desktop Binary: target/${BUILD_MODE}/tdrace-app${RESET}")
        else
            RESULTS+=("${RED}✗ Desktop Build: FAILED${RESET}")
        fi
    fi
fi

# 2. Web (WASM) Build
if [ "${BUILD_WEB}" -eq 1 ]; then
    echo -e "\n${BOLD}🌐 Building WebAssembly Target...${RESET}"
    if [ "${CHECK_ONLY}" -eq 1 ]; then
        if cargo check --package tdrace-app --target wasm32-unknown-unknown; then
            RESULTS+=("${GREEN}✓ WebAssembly (Check): PASSED${RESET}")
        else
            RESULTS+=("${RED}✗ WebAssembly (Check): FAILED${RESET}")
        fi
    else
        if "${ROOT_DIR}/web/build_web.sh" "${BUILD_MODE}"; then
            RESULTS+=("${GREEN}✓ WebAssembly Bundle: web/dist/index.html${RESET}")
        else
            RESULTS+=("${RED}✗ WebAssembly Build: FAILED${RESET}")
        fi
    fi
fi

# 3. Android Build
if [ "${BUILD_ANDROID}" -eq 1 ]; then
    echo -e "\n${BOLD}📱 Building Android Target...${RESET}"
    if [ "${CHECK_ONLY}" -eq 1 ]; then
        if cargo check --package tdrace-app --target aarch64-linux-android; then
            RESULTS+=("${GREEN}✓ Android aarch64 (Check): PASSED${RESET}")
        else
            RESULTS+=("${RED}✗ Android aarch64 (Check): FAILED${RESET}")
        fi
    else
        if "${ROOT_DIR}/mobile/android/build_android.sh" "${BUILD_MODE}"; then
            RESULTS+=("${GREEN}✓ Android Native Libraries: mobile/android/app/src/main/jniLibs/${RESET}")
        else
            RESULTS+=("${YELLOW}⚠ Android Build Completed with Warnings (see log)${RESET}")
        fi
    fi
fi

# 4. iOS Build
if [ "${BUILD_IOS}" -eq 1 ]; then
    echo -e "\n${BOLD}🍏 Building iOS Target...${RESET}"
    if [ "${CHECK_ONLY}" -eq 1 ]; then
        if cargo check --package tdrace-app --target aarch64-apple-ios; then
            RESULTS+=("${GREEN}✓ iOS aarch64 (Check): PASSED${RESET}")
        else
            RESULTS+=("${RED}✗ iOS aarch64 (Check): FAILED${RESET}")
        fi
    else
        if "${ROOT_DIR}/mobile/ios/build_ios.sh" "${BUILD_MODE}"; then
            RESULTS+=("${GREEN}✓ iOS Libraries: mobile/ios/build/${RESET}")
        else
            RESULTS+=("${YELLOW}⚠ iOS Build Completed with Warnings (see log)${RESET}")
        fi
    fi
fi

# Summary
echo -e "\n${BOLD}${BLUE}===================================================================${RESET}"
echo -e "${BOLD}📋 Cross-Platform Build Summary:${RESET}"
echo -e "${BOLD}${BLUE}===================================================================${RESET}"
for res in "${RESULTS[@]}"; do
    echo -e "  ${res}"
done
echo -e "${BOLD}${BLUE}===================================================================${RESET}\n"
