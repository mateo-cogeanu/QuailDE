#!/usr/bin/env bash
set -euo pipefail

# Build the locally customized Plasma workspace checkout into a dedicated
# install prefix so QuailDE can be tried without touching the system package.
# If the upstream Plasma source tree is not present yet, bootstrap it from KDE
# and apply the tracked Quail patch automatically.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE_DIR="${SOURCE_DIR:-$REPO_ROOT/vendor/plasma-workspace}"
BUILD_DIR="${BUILD_DIR:-$REPO_ROOT/build/plasma-workspace}"
INSTALL_PREFIX="${INSTALL_PREFIX:-$HOME/.local/quail-plasma}"
BUILD_TYPE="${BUILD_TYPE:-Release}"
PATCH_FILE="${PATCH_FILE:-$REPO_ROOT/patches/plasma-workspace-quail.patch}"
PLASMA_REMOTE="${PLASMA_REMOTE:-https://invent.kde.org/plasma/plasma-workspace.git}"
PLASMA_REF="${PLASMA_REF:-52c5f44}"

GENERATOR_ARGS=()
if command -v ninja >/dev/null 2>&1; then
    GENERATOR_ARGS+=(-G Ninja)
fi

if [[ ! -d "$SOURCE_DIR/.git" ]]; then
    echo "Bootstrapping plasma-workspace into $SOURCE_DIR"
    mkdir -p "$(dirname "$SOURCE_DIR")"
    git clone "$PLASMA_REMOTE" "$SOURCE_DIR"
fi

if [[ -n "$PLASMA_REF" ]]; then
    CURRENT_REF="$(git -C "$SOURCE_DIR" rev-parse --short HEAD)"
    if [[ "$CURRENT_REF" != "$PLASMA_REF" ]]; then
        echo "Checking out plasma-workspace base $PLASMA_REF"
        git -C "$SOURCE_DIR" fetch --all --tags
        git -C "$SOURCE_DIR" checkout "$PLASMA_REF"
    fi
fi

if [[ ! -f "$PATCH_FILE" ]]; then
    echo "Missing patch file: $PATCH_FILE" >&2
    exit 1
fi

if git -C "$SOURCE_DIR" apply --check "$PATCH_FILE" >/dev/null 2>&1; then
    echo "Applying Quail Plasma patch"
    git -C "$SOURCE_DIR" apply "$PATCH_FILE"
elif git -C "$SOURCE_DIR" apply --reverse --check "$PATCH_FILE" >/dev/null 2>&1; then
    echo "Quail Plasma patch already present, continuing"
else
    echo "Patch $PATCH_FILE does not apply cleanly to $SOURCE_DIR" >&2
    echo "Check the plasma-workspace checkout state and base commit." >&2
    exit 1
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
