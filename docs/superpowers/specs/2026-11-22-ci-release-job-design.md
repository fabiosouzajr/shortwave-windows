# CI release job for version tags

## Intent
Make future releases "push a tag": CI builds, checks and publishes the GitHub
Release that 5.0.0-win.1 needed by hand.

## Goals / non-goals
- Goal: a `v*` tag push publishes a release with the installer and zip from
  the CI build, notes from `docs/releases/<version>.md`.
- Non-goals: code signing, draft/manual approval step, changelog generation.
- Anti-goals: never publish when tests fail, when the built files don't carry
  the tag's version, or when the notes file is missing.

## Approaches
- **Release job reusing the Windows job's artifact (chosen).** No second
  build; the published files are exactly the tested ones.
- Separate release workflow that rebuilds: duplicated 25-min build, two
  definitions of "the build" to keep in sync.
- Build and publish in the Windows job: mixes write permissions into the build
  job and can't wait for the Linux job.

## Design
- `on.push.tags: ['v*']` added to `.github/workflows/windows.yml` (tag pushes
  ignore `paths-ignore`).
- Job `release`: `needs: [windows, linux]`,
  `if: startsWith(github.ref, 'refs/tags/v')`, `ubuntu-latest`,
  `permissions: contents: write`. Steps: checkout, download artifact
  `shortwave-windows-x64` into `dist/`, then a bash step:
  `version=${GITHUB_REF_NAME#v}`; require
  `dist/Shortwave-$version-windows-x64-setup.exe`,
  `dist/Shortwave-$version-windows-x64.zip`,
  `docs/releases/$version.md` (each missing one → `::error::` + exit 1);
  `gh release create "$GITHUB_REF_NAME" --title "Shortwave for Windows
  $version" --notes-file … --latest <both files>`.
- Header comment: repository is public (Actions free); keep "ask before adding
  a schedule".
- README "Building" / BUILDING-WINDOWS gain the release routine (bump
  `meson.build`, notes file, CHANGELOG entry, push tag).
