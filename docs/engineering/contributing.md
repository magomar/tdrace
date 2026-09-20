---
type: Architecture Spec
title: "Contributing Guide & Cross-Platform Setup"
description: "Repository setup, symlink configuration for Windows and macOS, and development workflow."
status: active
category: engineering
tags: [contributing, symlinks, git, cross-platform, development]
---

# Contributing Guide & Cross-Platform Setup 🏎️🤝

This guide outlines setup requirements, development workflows, and cross-platform instructions for contributing to **TdRace**.

---

## 🪟 Cross-Platform Repository Setup: Symlinks on Windows & macOS

TdRace uses relative filesystem symlinks (softlinks) between the canonical documentation/asset catalogs and the static web documentation portals:
* `portals/option-a-starlight/src/content/docs` $\to$ `docs/`
* `portals/option-b-showroom/public/textures` $\to$ `assets/textures/`

This maintains **single-source-of-truth** integrity under Google OKF v0.2 with zero duplicate files and zero build overhead.

### macOS & Linux
Symlinks work natively out-of-the-box. No special settings are needed:
```bash
git clone https://github.com/magomar/tdrace.git
```

### Windows
Windows Git checkouts do not enable symlinks by default. Choose one of the following methods:

1. **Clone with Symlinks Enabled**:
   ```bash
   git clone -c core.symlinks=true https://github.com/magomar/tdrace.git
   ```
   Or set the global Git configuration:
   ```bash
   git config --global core.symlinks true
   ```
2. **Enable Windows Developer Mode**:
   Windows requires Developer Mode to create symlinks without administrative elevation:
   * **Settings** $\to$ **System** $\to$ **For developers** $\to$ toggle **Developer Mode** on.
3. **WSL2 (Recommended)**:
   Develop inside **WSL2 (Windows Subsystem for Linux)**, where symlinks operate natively in the Linux filesystem layer.

---

## 🛠️ Toolchains & Prerequisites

* **Rust (2021 Edition)**: Managed via [rustup](https://rustup.rs).
* **Python (3.10+)**: For Gymnasium environment bindings.
* **Bun (>= 1.0)**: For Astro web portals in `portals/`.

---

## 🧪 Development Workflow

```bash
# Rust engine & app
cargo check --workspace
cargo test --workspace

# Web portals
cd portals
bun install
bun run dev:starlight    # Engineering documentation
bun run dev:showroom     # Interactive showroom
```
