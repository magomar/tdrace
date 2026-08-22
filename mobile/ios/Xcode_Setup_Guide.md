# TDRace iOS Xcode Integration & Setup Guide

This guide details how to build and deploy **TDRace** to iPhone and iPad devices using Apple Xcode and Rust.

---

## 1. Prerequisites

- **macOS** with Xcode 15+ installed.
- **Rust toolchain** with iOS compilation targets:
  ```bash
  rustup target add aarch64-apple-ios
  rustup target add aarch64-apple-ios-sim
  rustup target add x86_64-apple-ios
  ```
- Optional: `cargo-bundle` or `cargo-dinghy` for automated bundling.

---

## 2. Compiling the Native Library

Run the automated iOS compilation script:
```bash
./mobile/ios/build_ios.sh release
```

This compiles:
- `target/aarch64-apple-ios/release/libtdrace_app.a` (Physical iOS Devices)
- `target/aarch64-apple-ios-sim/release/libtdrace_app.a` (Apple Silicon Simulator)
- `target/x86_64-apple-ios/release/libtdrace_app.a` (Intel Simulator)

---

## 3. Creating the Xcode Project

1. Open Xcode and select **Create New Xcode Project** -> **iOS App** (or **Game**).
2. Choose **Storyboard** (or SwiftUI) with **Swift** or **Objective-C**.
3. Product Name: `TDRace`, Organization Identifier: `com.tdrace`.
4. Copy `mobile/ios/Info.plist` into your project target.

### Link Binary With Libraries
In Xcode project settings under **Target -> Build Phases -> Link Binary With Libraries**:
- Add `libtdrace_app.a`
- Add system frameworks:
  - `Metal.framework`
  - `MetalKit.framework`
  - `OpenGLES.framework`
  - `QuartzCore.framework`
  - `UIKit.framework`
  - `AVFoundation.framework`
  - `AudioToolbox.framework`

### Library Search Paths
Under **Build Settings -> Library Search Paths**:
```text
$(PROJECT_DIR)/../../target/$(PLATFORM_NAME)/release
```

---

## 4. Multi-Touch and Orientation Configuration

In `Info.plist` (or Target General settings):
- **Device Orientation**: Check only `Landscape Left` and `Landscape Right`.
- **Status Bar**: Set `Status bar is initially hidden` = `YES`.
- **Requires Full Screen**: `YES`.
- **CADisableMinimumFrameDurationOnPhone**: `YES` (enables 60fps / 120fps ProMotion display on iPhone Pro models).

---

## 5. Running on Physical Device

1. Connect your iPhone or iPad via USB.
2. Select your Development Team under **Signing & Capabilities**.
3. Select your device from the Xcode target dropdown.
4. Press **Cmd + R** to build, deploy, and race!
