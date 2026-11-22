#!/usr/bin/env bash
# Shortwave - Windows port: publish a GitHub Release for a version tag
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Usage: publish-release.sh v<version>   (run from the repository root)
# Expects the CI build's installer and zip in dist/ and the release notes in
# docs/releases/<version>.md. Set GH=echo for a dry run.
set -euo pipefail

tag=$1
version=${tag#v}
setup="dist/Shortwave-$version-windows-x64-setup.exe"
zip="dist/Shortwave-$version-windows-x64.zip"
notes="docs/releases/$version.md"

missing=0
for file in "$setup" "$zip" "$notes"; do
    if [ ! -f "$file" ]; then
        echo "::error::$file not found (does meson.build's version match $tag, and are the release notes written?)"
        missing=1
    fi
done
[ "$missing" -eq 0 ] || exit 1

${GH:-gh} release create "$tag" \
    --title "Shortwave for Windows $version" \
    --notes-file "$notes" \
    --latest \
    "$setup" "$zip"
