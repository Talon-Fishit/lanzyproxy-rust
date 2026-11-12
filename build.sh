#!/usr/bin/env bash
# Build lanzyproxy for the host and for Windows.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"

TARGET=x86_64-pc-windows-gnu

echo ">> native release"
cargo build --release

if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
  echo ">> mingw not found, skipping the Windows build"
  echo "   sudo apt-get install gcc-mingw-w64-x86-64 g++-mingw-w64-x86-64 cmake libclang-dev"
  exit 0
fi

if ! rustup target list --installed | grep -q "$TARGET"; then
  echo ">> adding the $TARGET target"
  rustup target add "$TARGET"
fi

# enet-sys runs bindgen, which needs libclang.
if [ -d /usr/lib/llvm-19/lib ] && [ -z "${LIBCLANG_PATH:-}" ]; then
  export LIBCLANG_PATH=/usr/lib/llvm-19/lib
fi

echo ">> windows release"
cargo build --release --target "$TARGET"

OUT="dist"
mkdir -p "$OUT"
cp "target/$TARGET/release/lanzyproxy.exe" "$OUT/lanzyproxy.exe"
cp "target/release/lanzyproxy" "$OUT/lanzyproxy"
cp README.md "$OUT/" 2>/dev/null || true

echo
echo "built:"
ls -la "$OUT"
file "$OUT/lanzyproxy.exe"
