# ==============================================================================
# 🏎️ TDRace - Top-Down 2D Arcade Racing Game & Gymnasium RL Environment
# ==============================================================================

SHELL := /bin/bash
.DEFAULT_GOAL := help

# Colors
CYAN  := \033[36m
GREEN := \033[32m
YELLOW:= \033[33m
RESET := \033[0m

# Python executable inside virtual environment
VENV_DIR := .venv
PYTHON   := $(VENV_DIR)/bin/python
MATURIN  := $(VENV_DIR)/bin/maturin
PYTEST   := $(VENV_DIR)/bin/pytest

.PHONY: help setup setup-python run run-dev play build build-release build-web serve-web build-android build-ios test test-rust test-python bench bench-rust bench-python clean

help: ## Display this help screen
	@echo -e "$(CYAN)🏎️  TDRace Make Commands$(RESET)"
	@echo -e "Usage: $(GREEN)make [target]$(RESET)\n"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  $(GREEN)%-16s$(RESET) %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ------------------------------------------------------------------------------
# 🛠️ Setup & Installation
# ------------------------------------------------------------------------------

setup: setup-python ## Full environment setup: Rust targets, Python venv, dependencies, and bindings
	@echo -e "$(GREEN)✓ Installing wasm32 compilation target...$(RESET)"
	@rustup target add wasm32-unknown-unknown 2>/dev/null || true
	@echo -e "$(GREEN)✓ TDRace environment setup complete!$(RESET)"
	@echo -e "Run '$(CYAN)make run$(RESET)' to launch the desktop game."

setup-python: ## Initialize Python virtual environment (.venv) and build editable wheel
	@echo -e "$(YELLOW)⚙️  Configuring Python virtual environment...$(RESET)"
	@if command -v uv >/dev/null 2>&1; then \
		uv venv $(VENV_DIR); \
		uv pip install --python $(PYTHON) maturin gymnasium pytest numpy tabulate; \
	else \
		python3 -m venv $(VENV_DIR); \
		$(PYTHON) -m pip install --upgrade pip maturin gymnasium pytest numpy tabulate; \
	fi
	@echo -e "$(YELLOW)🔨 Compiling PyO3 native extension with maturin...$(RESET)"
	@$(MATURIN) develop --release
	@echo -e "$(GREEN)✓ Python bindings installed in $(VENV_DIR)!$(RESET)"

# ------------------------------------------------------------------------------
# 🎮 Game Execution
# ------------------------------------------------------------------------------

run: ## Run the desktop arcade game (optimized release mode)
	@echo -e "$(GREEN)🚀 Launching TDRace Arcade Game (Release)...$(RESET)"
	cargo run --release -p tdrace-app

play: run ## Alias for 'make run'

run-dev: ## Run the desktop arcade game in debug mode
	@echo -e "$(YELLOW)🚀 Launching TDRace Arcade Game (Debug)...$(RESET)"
	cargo run -p tdrace-app

# ------------------------------------------------------------------------------
# 📦 Build & Cross-Platform Packaging
# ------------------------------------------------------------------------------

build: ## Build all Rust crates in workspace (debug mode)
	@echo -e "$(YELLOW)🔨 Compiling workspace crates (debug)...$(RESET)"
	cargo build --workspace

build-release: ## Build all Rust crates in workspace (optimized release mode)
	@echo -e "$(GREEN)🔨 Compiling workspace crates (release)...$(RESET)"
	cargo build --workspace --release

build-web: ## Build WebAssembly distribution for web browsers
	@echo -e "$(CYAN)🌐 Building WebAssembly distribution...$(RESET)"
	./web/build_web.sh

serve-web: build-web ## Start a local web server to play the WASM build in browser (port 8080)
	@echo -e "$(GREEN)🌐 Serving WebAssembly game at http://localhost:8080 (Ctrl+C to stop)...$(RESET)"
	@(cd web/dist && python3 -m http.server 8080)

build-android: ## Build Android APK / native library bundle
	@echo -e "$(CYAN)📱 Building Android package...$(RESET)"
	./mobile/android/build_android.sh

build-ios: ## Build iOS Xcode static framework
	@echo -e "$(CYAN)🍎 Building iOS package...$(RESET)"
	./mobile/ios/build_ios.sh

# ------------------------------------------------------------------------------
# 🧪 Testing & Verification
# ------------------------------------------------------------------------------

test: test-rust test-python ## Run all Rust and Python test suites

test-rust: ## Run Rust unit, integration, and physics tests
	@echo -e "$(GREEN)🧪 Running Rust test suite...$(RESET)"
	cargo test --workspace

test-python: ## Run Python Gymnasium compliance and adversarial tests
	@echo -e "$(GREEN)🧪 Running Python Gymnasium test suite...$(RESET)"
	@if [ -f "$(PYTEST)" ]; then \
		$(PYTEST) tests/python; \
	else \
		echo -e "$(YELLOW)Virtualenv not found. Running setup-python first...$(RESET)"; \
		$(MAKE) setup-python; \
		$(PYTEST) tests/python; \
	fi

# ------------------------------------------------------------------------------
# 📊 Benchmarks
# ------------------------------------------------------------------------------

bench: bench-rust bench-python ## Run both Rust physics and Python Gymnasium throughput benchmarks

bench-rust: ## Run Rust physics stepping and collision benchmarks
	@echo -e "$(CYAN)📊 Running Rust physics & collision benchmarks...$(RESET)"
	cargo bench -p tdrace-core

bench-python: ## Run Python Gymnasium throughput benchmark vs CarRacing-v3
	@echo -e "$(CYAN)📊 Running Gymnasium benchmark (TDRace vs CarRacing-v3)...$(RESET)"
	@$(PYTHON) benchmarks/gym_benchmark.py

# ------------------------------------------------------------------------------
# 🧹 Clean
# ------------------------------------------------------------------------------

clean: ## Remove build artifacts, target directories, and caches
	@echo -e "$(YELLOW)🧹 Cleaning build artifacts...$(RESET)"
	cargo clean
	rm -rf target/ dist/ build/ web/dist/ *.egg-info .pytest_cache/
	@echo -e "$(GREEN)✓ Clean complete!$(RESET)"
