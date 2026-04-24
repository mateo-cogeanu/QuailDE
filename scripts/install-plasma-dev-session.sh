#!/usr/bin/env bash
set -euo pipefail

# Register the built-from-source Plasma dev session so a display manager can
# offer it on the login screen for a clean Wayland test session.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${BUILD_DIR:-$REPO_ROOT/build/plasma-workspace}"
INSTALL_SCRIPT="$BUILD_DIR/login-sessions/install-sessions.sh"

if [[ ! -x "$INSTALL_SCRIPT" ]]; then
    echo "Missing $INSTALL_SCRIPT" >&2
    echo "Run $REPO_ROOT/scripts/build-plasma-workspace.sh first." >&2
    exit 1
fi

"$INSTALL_SCRIPT"

echo
echo "Installed the Plasma dev session."
echo "Log out, then choose the dev Wayland Plasma session from your login screen."
