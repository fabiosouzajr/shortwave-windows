#!/usr/bin/env bash
# Tests for publish-release.sh (dry run: GH=echo). Run: bash build-aux/test-publish-release.sh
set -u
script="$(cd "$(dirname "$0")" && pwd)/publish-release.sh"
fail=0
check() { if eval "$2"; then echo "ok   $1"; else echo "FAIL $1"; fail=1; fi; }

setup() {
    dir=$(mktemp -d) && cd "$dir" && mkdir -p dist docs/releases
    touch dist/Shortwave-5.0.0-win.2-windows-x64-setup.exe dist/Shortwave-5.0.0-win.2-windows-x64.zip
    echo notes > docs/releases/5.0.0-win.2.md
}

setup; rm docs/releases/5.0.0-win.2.md
out=$(GH=echo bash "$script" v5.0.0-win.2 2>&1); code=$?
check "missing notes fails" '[ $code -ne 0 ]'
check "missing notes is named" '[[ $out == *docs/releases/5.0.0-win.2.md* ]]'

setup; rm dist/*
touch dist/Shortwave-5.0.0-win.1-windows-x64-setup.exe dist/Shortwave-5.0.0-win.1-windows-x64.zip
out=$(GH=echo bash "$script" v5.0.0-win.2 2>&1); code=$?
check "version mismatch fails" '[ $code -ne 0 ]'
check "mismatch names the setup file" '[[ $out == *Shortwave-5.0.0-win.2-windows-x64-setup.exe* ]]'
check "mismatch publishes nothing" '[[ $out != *"release create"* ]]'

setup
out=$(GH=echo bash "$script" v5.0.0-win.2 2>&1); code=$?
check "all present succeeds" '[ $code -eq 0 ]'
check "title" '[[ $out == *"Shortwave for Windows 5.0.0-win.2"* ]]'
check "latest + notes" '[[ $out == *--latest* && $out == *docs/releases/5.0.0-win.2.md* ]]'
check "both files attached" '[[ $out == *setup.exe* && $out == *x64.zip* ]]'

exit $fail
