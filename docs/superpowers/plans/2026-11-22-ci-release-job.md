# CI release job Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** A `v*` tag push publishes the GitHub Release from the CI build.
**Spec:** `docs/superpowers/specs/2026-11-22-ci-release-job-design.md`

## Global Constraints
- Title `Shortwave for Windows <version>`, `--latest`, notes `docs/releases/<version>.md`.
- Publish only after `windows` and `linux` jobs pass; only on `refs/tags/v*`.

## Review Focus
1. Tag `v5.0.0-win.2` but `meson.build` still `5.0.0-win.1` → job fails naming the missing file, publishes nothing.
2. Notes file missing → fails, publishes nothing.
3. Branch pushes and PRs → release job skipped.

### Task 1: Release script with checks (testable locally)
- Create `build-aux/publish-release.sh TAG` (bash): `version=${TAG#v}`; check both `dist/` files and the notes file, `::error::` + exit 1 per missing item; then run `${GH:-gh} release create …`. `GH=echo` gives a dry run.
- Test `build-aux/test-publish-release.sh`: in a temp dir, (a) notes missing → exit 1 and message names the notes file; (b) version mismatch → exit 1 naming the setup file; (c) all present → exit 0 and the echoed command contains the title, `--latest` and both files. RED before the script exists, GREEN after.

### Task 2: Workflow + docs
- `windows.yml`: `tags: ['v*']`, job `release` (needs both, `if` on tag, ubuntu, `contents: write`, checkout, download-artifact into `dist`, run the script with `GH_TOKEN`). Fix header comment.
- `docs/BUILDING-WINDOWS.md`: "Releasing" section with the routine.
- Verify: `node` YAML parse is unavailable without deps → push and confirm the CI run on `main` starts and the `release` job shows as skipped.
