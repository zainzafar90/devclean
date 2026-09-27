#!/bin/bash
# Builds the universal devclean CLI, installs it into ~/.local/bin (or the directory given)
# and turns on the daily clean. DEVCLEAN_NO_SCHEDULE=1 skips the schedule.
set -euo pipefail
root="$(cd "$(dirname "$0")" && pwd)"
target="${1:-$HOME/.local/bin}"
"$root/scripts/build-cli-universal.sh"
mkdir -p "$target"
rm -f "$target/devclean"
install -m 755 "$root/target/cli-universal/devclean" "$target/devclean"
echo "installed $target/devclean ($("$target/devclean" version))"
case ":$PATH:" in *":$target:"*) ;; *) echo "add to your shell profile: export PATH=\"$target:\$PATH\"" ;; esac
if [ "${DEVCLEAN_NO_SCHEDULE:-}" != "1" ]; then "$target/devclean" schedule on; fi
