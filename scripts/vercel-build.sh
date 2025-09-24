#!/usr/bin/env bash
set -euxo pipefail

# --- Install Rust toolchain if missing (non-interactive)
if ! command -v rustup >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  export PATH="$HOME/.cargo/bin:$PATH"
fi

# Make sure cargo is on PATH for this shell
if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck source=/dev/null
  source "$HOME/.cargo/env"
fi

# --- Targets / tools for wasm
rustup target add wasm32-unknown-unknown
# Install wasm-pack if missing
if ! command -v wasm-pack >/dev/null 2>&1; then
  cargo install wasm-pack --locked
fi

# (Optional) unify Cargo target dir to speed up incremental builds
export CARGO_TARGET_DIR="$PWD/target"

# --- Build all kernels (creates kernels/**/pkg)
pnpm wasm:build

# --- Next build
pnpm build
