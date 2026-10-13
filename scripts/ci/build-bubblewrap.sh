#!/usr/bin/env bash
set -euo pipefail
# CI-only OS prerequisite. It does not alter a developer's host during source tests.
test "${CI:-}" = true
test -n "${RUNNER_TEMP:-}"
bwrap_source=719a4fd474d44b26906bcf2b1b0fb6eddd8d56d0
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends meson ninja-build pkg-config libcap-dev python3
git init "$RUNNER_TEMP/bubblewrap-source"
git -C "$RUNNER_TEMP/bubblewrap-source" fetch --depth=1 https://github.com/containers/bubblewrap "$bwrap_source"
git -C "$RUNNER_TEMP/bubblewrap-source" checkout --detach FETCH_HEAD
test "$(git -C "$RUNNER_TEMP/bubblewrap-source" rev-parse HEAD)" = "$bwrap_source"
meson setup "$RUNNER_TEMP/bubblewrap-build" "$RUNNER_TEMP/bubblewrap-source" -Dman=disabled -Dtests=false -Dbash_completion=disabled -Dzsh_completion=disabled
meson compile -C "$RUNNER_TEMP/bubblewrap-build"
sudo install -m 0755 "$RUNNER_TEMP/bubblewrap-build/bwrap" /usr/bin/bwrap
/usr/bin/bwrap --version
