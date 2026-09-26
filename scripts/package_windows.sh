#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# TDRace Windows Build & Packaging Helper
# Cross-compiles Windows x86_64 binary and packages standalone release ZIP
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

TARGET="x86_64-pc-windows-gnu"
BUILD_MODE="release"
CHECK_ONLY=0

function print_usage() {
    echo -e "${BOLD}TDRace Windows Build & Packaging Helper${RESET}

Usage:
  ./scripts/package_windows.sh [options]

Options:
  --release        Build in optimized release mode (default)
  --debug          Build in debug mode
  --check          Verify build with cargo check only
  -h, --help       Show this help message

Examples:
  ./scripts/package_windows.sh
  ./scripts/package_windows.sh --release
  ./scripts/package_windows.sh --debug"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --release|release)
            BUILD_MODE="release"
            shift
            ;;
        --debug|debug)
            BUILD_MODE="debug"
            shift
            ;;
        --check|check)
            CHECK_ONLY=1
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
echo -e "${BOLD}${BLUE}🪟 TDRace Windows Build & Packaging Pipeline${RESET}"
echo -e "   Target: ${BOLD}${TARGET}${RESET}"
echo -e "   Mode:   ${BOLD}${BUILD_MODE}${RESET}"
echo -e "${BOLD}${BLUE}===================================================================${RESET}"

cd "${ROOT_DIR}"

# 1. Verify Rust cross-compilation target
echo -e "\n${BOLD}🔍 Checking Rust target (${TARGET})...${RESET}"
if ! rustup target list --installed | grep -q "^${TARGET}$"; then
    echo -e "   ${YELLOW}Installing missing target: ${TARGET}...${RESET}"
    rustup target add "${TARGET}"
else
    echo -e "   ${GREEN}✓ Target installed: ${TARGET}${RESET}"
fi

# 2. Check for MinGW GCC linker
if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
    echo -e "   ${YELLOW}⚠️  Warning: x86_64-w64-mingw32-gcc not found on PATH.${RESET}"
    echo -e "   Ensure MinGW cross-compiler is installed (e.g. mingw-w64)."
fi

# 3. Compile or check binary
if [ "${CHECK_ONLY}" -eq 1 ]; then
    echo -e "\n${BOLD}🧪 Running cargo check for ${TARGET}...${RESET}"
    cargo check --package tdrace-app --target "${TARGET}"
    echo -e "${GREEN}✓ Windows target check passed!${RESET}"
    exit 0
fi

echo -e "\n${BOLD}🔨 Compiling tdrace-app for Windows (${BUILD_MODE})...${RESET}"
FLAG=""
if [ "${BUILD_MODE}" == "release" ]; then
    FLAG="--release"
fi

cargo build --package tdrace-app --target "${TARGET}" ${FLAG}

EXE_SRC="${ROOT_DIR}/target/${TARGET}/${BUILD_MODE}/tdrace-app.exe"
if [ ! -f "${EXE_SRC}" ]; then
    echo -e "${RED}❌ Error: compiled executable not found at ${EXE_SRC}${RESET}"
    exit 1
fi

# 4. Assemble package folder
PACKAGE_NAME="tdrace-windows-x86_64"
DIST_DIR="${ROOT_DIR}/dist"
STAGE_DIR="${DIST_DIR}/${PACKAGE_NAME}"

echo -e "\n${BOLD}📦 Assembling package directory (${STAGE_DIR})...${RESET}"
rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}"

# Copy binary
cp "${EXE_SRC}" "${STAGE_DIR}/tdrace-app.exe"

# Copy assets
if [ -d "${ROOT_DIR}/assets" ]; then
    cp -r "${ROOT_DIR}/assets" "${STAGE_DIR}/assets"
fi

# Copy tracks (excluding any .git files or submodules)
if [ -d "${ROOT_DIR}/tracks" ]; then
    mkdir -p "${STAGE_DIR}/tracks"
    if command -v rsync >/dev/null 2>&1; then
        rsync -a --exclude='.git*' "${ROOT_DIR}/tracks/" "${STAGE_DIR}/tracks/"
    else
        cp -r "${ROOT_DIR}/tracks/"* "${STAGE_DIR}/tracks/"
        rm -rf "${STAGE_DIR}/tracks/.git"* 2>/dev/null || true
    fi
fi

# Copy series (championship seasons)
if [ -d "${ROOT_DIR}/series" ]; then
    cp -r "${ROOT_DIR}/series" "${STAGE_DIR}/series"
fi

# Copy config files
for cfg in config.toml config.*.toml; do
    if [ -f "${ROOT_DIR}/${cfg}" ]; then
        cp "${ROOT_DIR}/${cfg}" "${STAGE_DIR}/"
    fi
done

# Copy gamepad profile if present
if [ -f "${ROOT_DIR}/gamepad_profile.json" ]; then
    cp "${ROOT_DIR}/gamepad_profile.json" "${STAGE_DIR}/"
fi

# Generate launcher scripts
cat << 'EOF' > "${STAGE_DIR}/run.bat"
@echo off
cd /d "%~dp0"
start "" tdrace-app.exe %*
EOF

cat << 'EOF' > "${STAGE_DIR}/run-gt.bat"
@echo off
cd /d "%~dp0"
start "" tdrace-app.exe --gt %*
EOF

cat << 'EOF' > "${STAGE_DIR}/run-kart.bat"
@echo off
cd /d "%~dp0"
start "" tdrace-app.exe --kart %*
EOF

cat << 'EOF' > "${STAGE_DIR}/run-nascar.bat"
@echo off
cd /d "%~dp0"
start "" tdrace-app.exe --nascar %*
EOF

cat << 'EOF' > "${STAGE_DIR}/run-rally.bat"
@echo off
cd /d "%~dp0"
start "" tdrace-app.exe --rally %*
EOF

cat << 'EOF' > "${STAGE_DIR}/run-classic.bat"
@echo off
cd /d "%~dp0"
start "" tdrace-app.exe --classic %*
EOF

# Generate README.txt
cat << 'EOF' > "${STAGE_DIR}/README.txt"
==============================================================================
  TDRace - Top-Down 2D Arcade Racing Game (Windows x86_64 Release)
==============================================================================

HOW TO RUN:
-----------
Simply double-click "tdrace-app.exe" or "run.bat" to start the game!

You can also use the dedicated launchers to start directly in specific motorsport modules:
- run-gt.bat       : GT World Challenge
- run-kart.bat     : Karting Series
- run-nascar.bat   : NASCAR Cup Series
- run-rally.bat    : World Rally Championship
- run-classic.bat  : Classic Arcade Motorsport

Or run from Command Prompt / PowerShell:
  tdrace-app.exe [options]
  Options: --gt, --kart, --nascar, --rally, --classic, --module <name>


DEFAULT CONTROLS:
-----------------
Race Controls (Keyboard):
  Q              : Throttle (Accelerate)
  A              : Brake
  Space          : Handbrake (Drift / Power Slide)
  Z              : Reverse
  O              : Steer Left
  P              : Steer Right
  Arrow Keys     : Alternative driving controls (Up/Down/Left/Right)
  Tab / C        : Toggle Follow Camera / Full-Track Overview
  R              : Restart Session
  Esc            : Pause / Menu Back
  M              : Return to Main Menu
  E              : Open Track Studio (from Menu)
  F1 - F5        : Toggle Debug Visuals (LIDAR, Checkpoints, OBBs, AI Paths)

Gamepad (Xbox / DirectInput / XInput):
  RT             : Analog Throttle
  LT             : Analog Brake
  A / RB         : Handbrake / Confirm in menu
  Left Stick     : Proportional Steering
  X / LB         : Reverse
  B              : Back / Cancel
  Start          : Pause
  L3             : Toggle Camera
  R3 / Select    : Cycle Assist Profile (Arcade, Sport, Pro)

Track Studio Controls (Editor):
  1 - 0          : Tool selector (Spline, Surfaces, Ramps, Hazards, Gates)
  Left Click     : Place / Drag nodes and waypoints
  Right Drag     : Pan camera canvas
  Scroll Wheel   : Zoom in / out
  Ctrl+Z / Ctrl+Y: Undo / Redo
  Space / P      : Instant Test Drive
  Esc            : Exit Test Drive / Return to Studio
==============================================================================
EOF

# 5. Compress into ZIP archive
ZIP_OUT="${DIST_DIR}/${PACKAGE_NAME}.zip"
echo -e "\n${BOLD}🗜️  Compressing package into ${ZIP_OUT}...${RESET}"
rm -f "${ZIP_OUT}"

if command -v python3 >/dev/null 2>&1; then
    python3 -c "import shutil; shutil.make_archive('${DIST_DIR}/${PACKAGE_NAME}', 'zip', '${DIST_DIR}', '${PACKAGE_NAME}')"
elif command -v zip >/dev/null 2>&1; then
    (cd "${DIST_DIR}" && zip -rq "${PACKAGE_NAME}.zip" "${PACKAGE_NAME}")
elif command -v bsdtar >/dev/null 2>&1; then
    (cd "${DIST_DIR}" && bsdtar -a -cf "${PACKAGE_NAME}.zip" "${PACKAGE_NAME}")
else
    echo -e "${RED}❌ Error: Neither python3, zip, nor bsdtar found for archiving.${RESET}"
    exit 1
fi

ZIP_SIZE=$(du -h "${ZIP_OUT}" | cut -f1)
EXE_SIZE=$(du -h "${STAGE_DIR}/tdrace-app.exe" | cut -f1)

echo -e "\n${BOLD}${GREEN}===================================================================${RESET}"
echo -e "${BOLD}${GREEN}✓ Windows build and packaging completed successfully!${RESET}"
echo -e "   Executable: ${BOLD}${STAGE_DIR}/tdrace-app.exe${RESET} (${EXE_SIZE})"
echo -e "   Staging:    ${BOLD}${STAGE_DIR}/${RESET}"
echo -e "   Zip Bundle: ${BOLD}${ZIP_OUT}${RESET} (${ZIP_SIZE})"
echo -e "${BOLD}${GREEN}===================================================================${RESET}\n"
