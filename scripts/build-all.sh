#!/usr/bin/env bash
# Tests and packages every plugin under plugins/, then prints the sha256 and
# the size of each package.
#
#   scripts/build-all.sh            all plugins
#   scripts/build-all.sh kanban     only the ones named (directory names)
#
# The packages land in dist/<id>-<version>.zip at the repository root. On
# Windows, run it from Git Bash. The tool is `tessoa-plugin` from the PATH,
# or the executable named by TESSOA_PLUGIN.
#
# Stops at the first plugin whose tests or packaging fail; exit status 1.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
tool="${TESSOA_PLUGIN:-tessoa-plugin}"
if ! command -v "$tool" >/dev/null 2>&1; then
    echo "build-all: $tool not found; run scripts/setup.sh (or setup.ps1) or set TESSOA_PLUGIN" >&2
    exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
    sha256() { sha256sum "$1" | cut -d' ' -f1; }
else
    sha256() { shasum -a 256 "$1" | cut -d' ' -f1; }
fi

# The value of a key in the [plugin] table of a manifest.
plugin_key() {
    awk -v key="$2" '
        /^\[/ { in_plugin = ($0 == "[plugin]"); next }
        in_plugin && $1 == key { sub(/^[^=]*=[ \t]*"/, ""); sub(/".*$/, ""); print; exit }
    ' "$1"
}

if [ $# -gt 0 ]; then
    names=("$@")
else
    names=()
    for dir in "$root"/plugins/*/; do
        names+=("$(basename "$dir")")
    done
fi

mkdir -p "$root/dist"
packages=()
for name in "${names[@]}"; do
    dir="$root/plugins/$name"
    if [ ! -f "$dir/plugin.toml" ]; then
        echo "build-all: no plugin.toml in plugins/$name" >&2
        exit 1
    fi
    id="$(plugin_key "$dir/plugin.toml" id)"
    version="$(plugin_key "$dir/plugin.toml" version)"
    out="$root/dist/$id-$version.zip"

    echo "== $name ($id $version)"
    "$tool" test --dir "$dir"
    "$tool" package --dir "$dir" --out "$out"
    packages+=("$out")
done

echo
echo "sha256                                                            size  package"
for out in "${packages[@]}"; do
    size="$(wc -c < "$out" | tr -d ' ')"
    printf '%s  %8s  dist/%s\n' "$(sha256 "$out")" "$size" "$(basename "$out")"
done
