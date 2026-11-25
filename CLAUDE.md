# CLAUDE.md

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

---

## Project: releasing the Windows port

Releases are published by CI (`.github/workflows/windows.yml`, job `release`)
when a `v<version>` tag is pushed. Don't build release files locally or run
`gh release create` by hand; that was only needed for `5.0.0-win.1`.

- **Version format:** upstream version + `-win.N` (e.g. `5.0.0-win.2`); tag is
  `v` + version. The version lives only in `meson.build` (`Cargo.toml` stays
  `0.0.0`); it flows to the About dialog, the exe's version info (`build.rs`
  makes `5.0.0.2`) and the installer/zip names.
- **Steps:**
  1. Set the version in `meson.build`.
  2. Check the `<version>` milestone; move open issues that won't make it to
     the next milestone.
  3. Write `docs/releases/<version>.md` (the GitHub Release text, for people
     downloading the app; known issues cite their issue as `(#N)`) and move
     the `## [Unreleased]` entries in `CHANGELOG.md` under
     `## [<version>] - <date>`; update the link footer.
  4. Commit and push to `main`.
  5. `git tag -a v<version> -m "Shortwave for Windows <version>"` and
     `git push origin v<version>`.
  6. Watch the tag's CI run (`gh run list`, `gh run watch`); the `release` job
     runs only after the Windows and Linux jobs pass.
  7. Once published, close the milestone
     (`gh api -X PATCH repos/fabiosouzajr/shortwave-windows/milestones/<number> -f state=closed`)
     and create the next one.
- **Guard rails:** `build-aux/publish-release.sh` refuses to publish if the
  installer/zip don't carry the tag's version or the notes file is missing.
  Test it with `bash build-aux/test-publish-release.sh`. If a tag run fails,
  fix it and ask the user before deleting and re-pushing the tag.
- **Docs to keep honest:** known issues in the release notes must match the
  "Not working / not tested" table in `docs/WINDOWS-PORT-STATUS.md`; features
  not yet checked in the running app are known issues, not highlights. Each
  row of that table that needs work links its issue.
- **Ask first:** pushing a tag publishes a public release (the repository is
  public). Never move or force-push an already pushed tag.
- Full routine for people: README "Releasing" and
  `docs/BUILDING-WINDOWS.md#releasing`.
