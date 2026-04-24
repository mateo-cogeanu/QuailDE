#!/usr/bin/env bash
set -euo pipefail

# Build the locally customized Plasma workspace checkout into a dedicated
# install prefix so QuailDE can be tried without touching the system package.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE_DIR="${SOURCE_DIR:-$REPO_ROOT/vendor/plasma-workspace}"
BUILD_DIR="${BUILD_DIR:-$REPO_ROOT/build/plasma-workspace}"
INSTALL_PREFIX="${INSTALL_PREFIX:-$HOME/.local/quail-plasma}"
BUILD_TYPE="${BUILD_TYPE:-Release}"

GENERATOR_ARGS=()
if command -v ninja >/dev/null 2>&1; then
    GENERATOR_ARGS+=(-G Ninja)
fi

mkdir -p "$BUILD_DIR"

cmake -S "$SOURCE_DIR" -B "$BUILD_DIR" \
    "${GENERATOR_ARGS[@]}" \
    -DCMAKE_BUILD_TYPE="$BUILD_TYPE" \
    -DCMAKE_INSTALL_PREFIX="$INSTALL_PREFIX"

cmake --build "$BUILD_DIR" -j"$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)"
cmake --install "$BUILD_DIR"

echo
echo "Built plasma-workspace into: $INSTALL_PREFIX"
echo "Next step: run $REPO_ROOT/scripts/install-plasma-dev-session.sh"
