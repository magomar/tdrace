# Contributing to TDRace 🏎️🤝

Thank you for your interest in contributing to **TDRace**! This guide covers repository setup, cross-platform requirements, coding standards, and documentation guidelines.

---

## 🪟 Cross-Platform Repository Setup: Symlinks on Windows & macOS

TDRace uses relative symlinks (softlinks) between the documentation/asset catalogs and the static web documentation portals (`portals/option-a-starlight` and `portals/option-b-showroom`):
* `portals/option-a-starlight/src/content/docs` $\to$ `docs/`
* `portals/option-b-showroom/public/textures` $\to$ `assets/textures/`

This architecture ensures zero duplicate files and zero build step overhead while maintaining a single source of truth under Google OKF v0.2.

### macOS & Linux
Symlinks are supported natively out-of-the-box by the operating system and Git. No extra configuration is required:
```bash
git clone https://github.com/magomar/tdrace.git
```

### Windows
Windows requires explicit permission or configuration to checkout and create filesystem symlinks:

1. **Clone with Symlinks Enabled**:
   Pass `-c core.symlinks=true` when cloning the repository:
   ```bash
   git clone -c core.symlinks=true https://github.com/magomar/tdrace.git
   ```
   Or enable it globally/locally in your git configuration:
   ```bash
   git config --global core.symlinks true
   ```
2. **Enable Windows Developer Mode**:
   Symlink creation without elevated Administrator privileges requires Windows Developer Mode:
   * Go to **Settings** $\to$ **System** $\to$ **For developers** $\to$ turn on **Developer Mode**.
3. **WSL2 (Recommended for Windows)**:
   Alternatively, run development inside **WSL2 (Windows Subsystem for Linux)**, where symlinks work natively without special Windows privileges.

---

## 🛠️ Tech Stack & Prerequisites

* **Rust (2021 Edition)**: Install via [rustup.rs](https://rustup.rs) (`cargo`, `rustc >= 1.75`).
* **Python (3.10+)**: For Gymnasium reinforcement learning bindings (`uv` or `pip`).
* **Bun (>= 1.0)**: For the Astro documentation and showroom portals (`portals/`).

---

## 🧪 Development Workflow

### Rust Engine & App
```bash
# Check compilation across all workspace crates
cargo check --workspace

# Run Rust unit, integration, and physics tests
cargo test --workspace

# Run Macroquad arcade desktop application
cargo run -p tdrace-app
```

### Web Portals & Documentation
```bash
cd portals

# Install dependencies
bun install

# Run Starlight Engineering Manual
bun run dev:starlight

# Run Motorsport Showroom & Physics Lab
bun run dev:showroom

# Build both portals
bun run build
```

---

## 📐 Spec-Driven Development (SDD) Guidelines

TdRace follows Keel Spec-Driven Development methodology:
* **Specs are Contracts**: Review active specifications in `specs/` and documentation in `docs/` before implementing major architectural changes.
* **Surgical Changes**: Touch only what is required for your feature or fix.
* **Atomic Commits**: Follow Conventional Commits format (`feat(...)`, `fix(...)`, `docs(...)`).
