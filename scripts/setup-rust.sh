#!/usr/bin/env bash
# Installs rustup and the pinned toolchain (rust-toolchain.toml) into .cache/, for this repo only.
# Idempotent. Never touches ~/.rustup, ~/.cargo, or shell profiles (--no-modify-path).
#
#   source scripts/env.sh && bash scripts/setup-rust.sh
#
# CI runners already ship rustup; there, env.sh's RUSTUP_HOME makes the pinned toolchain
# install into .cache/rustup the first time cargo runs, so CI does not need this script.
set -euo pipefail

if [ -z "${LJ_CACHE:-}" ] || [ -z "${RUSTUP_HOME:-}" ] || [ -z "${CARGO_HOME:-}" ]; then
  echo "error: source scripts/env.sh first" >&2
  exit 1
fi

if [ ! -x "$CARGO_HOME/bin/rustup" ]; then
  case "$(uname -s)-$(uname -m)" in
    Darwin-arm64) triple=aarch64-apple-darwin ;;
    Darwin-x86_64) triple=x86_64-apple-darwin ;;
    Linux-x86_64) triple=x86_64-unknown-linux-gnu ;;
    Linux-aarch64) triple=aarch64-unknown-linux-gnu ;;
    *) echo "error: unsupported host $(uname -s)-$(uname -m); install rustup by hand" >&2; exit 1 ;;
  esac
  dl="$LJ_CACHE/tmp/rustup-init-$$"
  mkdir -p "$dl"
  base="https://static.rust-lang.org/rustup/dist/$triple/rustup-init"
  curl --proto '=https' --tlsv1.2 -sSf "$base" -o "$dl/rustup-init"
  curl --proto '=https' --tlsv1.2 -sSf "$base.sha256" -o "$dl/rustup-init.sha256"
  expected="$(awk '{print $1}' "$dl/rustup-init.sha256")"
  actual="$(shasum -a 256 "$dl/rustup-init" | awk '{print $1}')"
  if [ "$expected" != "$actual" ]; then
    echo "error: rustup-init checksum mismatch" >&2
    rm -rf "$dl"
    exit 1
  fi
  chmod +x "$dl/rustup-init"
  "$dl/rustup-init" -y --no-modify-path --profile minimal --default-toolchain none
  rm -rf "$dl"
fi

# rust-toolchain.toml pins the version, the minimal profile, and rustfmt + clippy.
cd "$LJ_ROOT"
rustup toolchain install
rustc --version
cargo --version
