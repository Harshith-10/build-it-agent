#!/usr/bin/env bash
set -euo pipefail

# Build the Rust app for macOS in release mode. Optionally build universal binary when BOTH archs are available.

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT_DIR"

APP_NAME="build-it-agent"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found. Please install Rust toolchain from https://rustup.rs" >&2
  exit 1
fi

TARGET_ARCH=${TARGET_ARCH:-}

if [[ -z "${TARGET_ARCH}" ]]; then
  echo "Building $APP_NAME for host architecture..."
  cargo build --release
  echo "Built: target/release/$APP_NAME"
else
  case "$TARGET_ARCH" in
    universal)
      echo "Building universal binary (x86_64 + aarch64)..."
      # Ensure targets exist
      rustup target add x86_64-apple-darwin || true
      rustup target add aarch64-apple-darwin || true
      cargo build --release --target x86_64-apple-darwin
      cargo build --release --target aarch64-apple-darwin
      mkdir -p target/release
      lipo -create -output target/release/$APP_NAME \
        target/x86_64-apple-darwin/release/$APP_NAME \
        target/aarch64-apple-darwin/release/$APP_NAME
      echo "Built universal: target/release/$APP_NAME"
      ;;
    x86_64|aarch64)
      echo "Building for $TARGET_ARCH-apple-darwin..."
      rustup target add ${TARGET_ARCH}-apple-darwin || true
      cargo build --release --target ${TARGET_ARCH}-apple-darwin
      ;;
    *)
      echo "Unknown TARGET_ARCH: $TARGET_ARCH (use universal|x86_64|aarch64)" >&2
      exit 2
      ;;
  esac
fi
