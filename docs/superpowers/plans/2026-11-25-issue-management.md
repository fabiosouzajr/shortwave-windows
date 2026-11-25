# Issue Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the port a working issue process: structured issue forms, a
small label set, release milestones, a private security channel, and a
written triage workflow. Known issues in the docs get linked to issue numbers.

**Architecture:** Everything lives on GitHub (`fabiosouzajr/shortwave-windows`).
Repository files describe the process: issue forms and `config.yml` under
`.github/ISSUE_TEMPLATE/`, plus `CONTRIBUTING.md`, `SECURITY.md` and a README
section. `gh` commands create the GitHub-side state: labels, a milestone,
tracking issues and the vulnerability-reporting setting. The release routine
gains one milestone step.

**Tech Stack:** GitHub issue forms (YAML), `gh` CLI, Markdown. PyYAML (already
installed) for the local syntax check.

**Spec:** No separate spec. The decisions come from the issue-management
recommendations agreed in the session of 2026-11-25 and are listed under
"Decisions" below.

## Decisions

- **Where:** GitHub Issues on this repo are for Windows build problems.
  Bugs that also happen on Linux or in upstream logic belong on
  <https://gitlab.gnome.org/World/Shortwave/-/issues>; keep a short tracking
  issue here that links to the upstream one. Station data problems belong to
  radio-browser.info. Security reports go through GitHub private
  vulnerability reporting.
- **Labels:** keep GitHub's defaults (`bug`, `enhancement`, `question`,
  `documentation`, `duplicate`, `invalid`, `wontfix`, `good first issue`,
  `help wanted`, `accessibility`). Add the four status labels below. No
  priority labels, no `windows-only` label, no Projects board, no stale bot.
- **Lifecycle:** a new issue gets `needs-triage` from its form. After
  triage it gets exactly one of: `confirmed`, `needs-info`, `upstream`, or
  it is closed (`duplicate`, `invalid`, `wontfix`). `needs-info` issues with
  no reply after 30 days are closed by hand. `upstream` issues stay open
  until a release picks up the upstream fix. Fixes close issues with
  `Fixes #N` in the commit or PR.
- **Milestones:** one per release version, named exactly like the version
  (`5.0.0-win.2`). It is closed when the tag is published.
- **Known issues:** every actionable row in "Not working / not tested" in
  `docs/WINDOWS-PORT-STATUS.md` has an issue. Release notes can then cite
  `(#N)`.

## Global Constraints

- Repository is public. Creating labels, milestones and issues, changing
  repo settings, and pushing to `main` all need the user's go-ahead at
  execution time (CLAUDE.md, "Ask first").
- Version strings use the format `5.0.0-win.N`, and the app shows the version under About.
- Upstream issue tracker URL: `https://gitlab.gnome.org/World/Shortwave/-/issues`.
- Security advisory URL: `https://github.com/fabiosouzajr/shortwave-windows/security/advisories/new`.
- Logs: release builds have no console and write no log file. The only way to
  get logs is a development build (`.\build-aux\windows\build.ps1 -BuildProfile development`)
  run with `RUST_BACKTRACE=1` and `RUST_LOG=shortwave=debug`, as the README
  FAQ says. Forms must not require a log.
- Match the existing doc voice: short sentences, plain English, no marketing.
- Commits end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Review Focus

1. **A form label that doesn't exist yet.** GitHub silently drops labels in
   a form's `labels:` list that the repo lacks, so new issues would arrive
   without `needs-triage`. Task 1 creates the labels before Task 6 pushes the
   forms; Task 6 checks that a test issue from a form gets the label.
2. **YAML that parses but GitHub rejects** (unknown key, missing `id`, wrong
   `type`). The template chooser then shows an error and hides the form.
   Task 2's PyYAML check catches only syntax, so Task 6 checks the forms
   render at `/issues/new/choose`.
3. **A reporter on a release build is asked for logs they can't produce.**
   The log field is optional and its description says how to get logs (dev
   build). Task 2 checks this field is `required: false`.
4. **A plain question with no form that fits.** Blank issues are disabled. The
   chooser shows a "Questions" contact link to the README FAQ. The bug form's
   description says it may also be used for "it doesn't behave as I expect".
   Task 6 checks the link shows.
5. **A vulnerability posted as a public issue anyway.** CONTRIBUTING's triage
   steps say: remove the details from the issue (edit or delete it), ask the
   reporter to use the advisory link, and don't discuss it publicly. Task 3
   pins this text.

---

### Task 1: Labels, milestone, and triage of issue #1 (GitHub state)

**Files:** none. This task only changes GitHub state, so get the user's go-ahead first.

**Interfaces:**
- Produces: labels `needs-triage`, `needs-info`, `confirmed`, `upstream`
  (Tasks 2, 3, 5 use these exact names); milestone `5.0.0-win.2` (Tasks 4, 5).

- [ ] **Step 1: Check what exists**

Run: `gh label list --limit 100` and `gh api repos/fabiosouzajr/shortwave-windows/milestones --jq '.[].title'`
Expected: the 10 default labels, none of the four below; no milestones.

- [ ] **Step 2: Create the labels**

| Name | Color | Description |
|---|---|---|
| `needs-triage` | `ededed` | New, not yet looked at |
| `needs-info` | `fbca04` | Waiting for the reporter |
| `confirmed` | `0e8a16` | Reproduced or accepted |
| `upstream` | `5319e7` | Also in upstream Shortwave; tracked on GNOME GitLab |

Run, per row: `gh label create <name> --color <color> --description "<description>"`

- [ ] **Step 3: Create the milestone**

Run: `gh api repos/fabiosouzajr/shortwave-windows/milestones -f title=5.0.0-win.2 -f description="Next Windows port release"`
Expected: JSON with `"title": "5.0.0-win.2"`, `"state": "open"`.

- [ ] **Step 4: Triage issue #1**

Run: `gh issue edit 1 --add-label confirmed --milestone 5.0.0-win.2`
(The issue already includes a root cause and a reproduction.)

- [ ] **Step 5: Verify**

Run: `gh label list --limit 100` and `gh issue view 1 --json labels,milestone`
Expected: 14 labels; issue #1 has `bug`, `confirmed`, milestone `5.0.0-win.2`.

---

### Task 2: Issue forms and chooser config

**Files:**
- Create: `.github/ISSUE_TEMPLATE/bug_report.yml`
- Create: `.github/ISSUE_TEMPLATE/feature_request.yml`
- Create: `.github/ISSUE_TEMPLATE/config.yml`

**Interfaces:**
- Consumes: label names from Task 1.
- Produces: form names "Bug report" and "Feature request" (Task 3's
  CONTRIBUTING refers to them by these names).

- [ ] **Step 1: Write `bug_report.yml`**

Top level: `name: Bug report`, `description: Something in the Windows build doesn't work or doesn't behave as you expect`, `labels: [bug, needs-triage]`.
Start with a `markdown` block saying: this tracker is for the Windows port;
problems that also happen in Shortwave on Linux go to upstream (link); search
existing issues first. Then these fields, in order:

| id | type | label | required | notes |
|---|---|---|---|---|
| `version` | input | Shortwave version | yes | description: "Shown in the About dialog (menu → About Shortwave)"; placeholder `5.0.0-win.1` |
| `windows` | input | Windows version | yes | placeholder `Windows 11 24H2` |
| `install` | dropdown | How did you install it? | yes | options: `Installer (just for me)`, `Installer (all users)`, `Portable zip`, `Built from source` |
| `linux` | dropdown | Does it also happen in Shortwave on Linux? | yes | options: `Don't know`, `No, only on Windows`, `Yes, also on Linux` |
| `what` | textarea | What happened? | yes | |
| `steps` | textarea | Steps to reproduce | yes | placeholder `1. …\n2. …\n3. …` |
| `expected` | textarea | What did you expect? | yes | |
| `station` | input | Station (if it's about one) | no | description: "Name as shown in Shortwave, or the stream URL" |
| `logs` | textarea | Logs | no | `render: text`; description: release builds don't write logs; to get them, build the development profile and run with `RUST_LOG=shortwave=debug` (link README FAQ "How I can get debug information?") |

- [ ] **Step 2: Write `feature_request.yml`**

`name: Feature request`, `description: Suggest something new or a change`, `labels: [enhancement, needs-triage]`.
Fields: `problem` (textarea, "What problem would this solve?", required),
`solution` (textarea, "What would you like to happen?", required),
`scope` (dropdown, "Is this about Windows specifically?", required, options
`Yes, Windows only`, `No, it applies to Shortwave everywhere`). The
description of `scope` says that general feature ideas are better proposed upstream (link).

- [ ] **Step 3: Write `config.yml`**

`blank_issues_enabled: false` and `contact_links`, in this order:

| name | url | about |
|---|---|---|
| Problem also in Shortwave on Linux | `https://gitlab.gnome.org/World/Shortwave/-/issues` | Report it upstream, where the shared code is maintained |
| Security vulnerability | `https://github.com/fabiosouzajr/shortwave-windows/security/advisories/new` | Report privately, not as a public issue |
| Wrong or broken station data | `https://www.radio-browser.info/` | Station names, URLs and tags come from radio-browser.info |
| Questions | `https://github.com/fabiosouzajr/shortwave-windows#faq` | Check the FAQ first |

Check that `#faq` is the README heading anchor. If it isn't, use the actual anchor.

- [ ] **Step 4: Check syntax and Review Focus 3**

Run: `python -I -c "import sys,yaml; d=[yaml.safe_load(open(f,encoding='utf-8')) for f in sys.argv[1:]]; logs=[b for b in d[0]['body'] if b.get('id')=='logs'][0]; assert logs['validations']['required'] is False; print('ok')" .github/ISSUE_TEMPLATE/bug_report.yml .github/ISSUE_TEMPLATE/feature_request.yml .github/ISSUE_TEMPLATE/config.yml`
Expected: `ok`. (If `logs` omits `validations`, give it `validations: {required: false}` explicitly so the check has something to assert.)

- [ ] **Step 5: Commit**

```bash
git add .github/ISSUE_TEMPLATE
git commit -m "chore(issues): add bug and feature issue forms"
```

---

### Task 3: CONTRIBUTING, SECURITY, README section

**Files:**
- Create: `CONTRIBUTING.md`
- Create: `SECURITY.md`
- Modify: `README.md` (new `## Reporting issues` section after the FAQ, before `## Translations`)
- Modify: `CHANGELOG.md` (`## [Unreleased]` → `### Added`)

**Interfaces:**
- Consumes: form names (Task 2), label names (Task 1).

- [ ] **Step 1: Write `CONTRIBUTING.md`**

Sections:
1. **Reporting a problem.** Where it goes (the four routes from Decisions)
   and what helps: version, Windows version, install type, steps, and
   station if relevant.
2. **Triage (for maintainers).** The lifecycle from Decisions as a short
   list, followed by the label table from Task 1 Step 2. Include the two rules:
   "`needs-info` with no reply in 30 days: close with a note that it can be
   reopened" and "`upstream`: link the GitLab issue, file one there if none
   exists, keep open until a release includes the fix".
3. **If someone posts a vulnerability publicly.** Remove the details
   (edit the issue, or delete it if editing isn't enough). Comment with the
   advisory link. Don't discuss it publicly.
4. **Fixes and releases.** Reference `Fixes #N` in commits or PRs. Assign
   the milestone of the next release. Release notes cite known issues as `(#N)`.
5. **Code and builds.** One line linking `docs/BUILDING-WINDOWS.md` and
   `build-aux/windows/setup.ps1`.

- [ ] **Step 2: Write `SECURITY.md`**

Cover three points:
- Supported version: the latest release only.
- How to report: the advisory link. Don't open a public issue.
- What to expect: an acknowledgement, a fix in the next `-win.N` release.

Also note that vulnerabilities in upstream code or in GStreamer/GTK go to those projects.
Keep it under 20 lines.

- [ ] **Step 3: Add the README section**

`## Reporting issues`: three or four sentences. Report Windows problems in
this repo's Issues using the forms. Report problems that also happen on
Linux upstream. Report security issues privately (link SECURITY.md).
Link CONTRIBUTING.md for details. Don't repeat the README's existing line 8
pointer. This section expands on it.

- [ ] **Step 4: Changelog entry**

Under `## [Unreleased]` → `### Added`: "Issue forms for bug reports and
feature requests, a security policy and contributing notes. See
[CONTRIBUTING.md](CONTRIBUTING.md)."

- [ ] **Step 5: Check links**

Run: `grep -n -o -E '\]\([^)]+\)' CONTRIBUTING.md SECURITY.md` and open each
relative path to confirm it exists.
Expected: every relative link resolves. Absolute links match Global Constraints.

- [ ] **Step 6: Commit**

```bash
git add CONTRIBUTING.md SECURITY.md README.md CHANGELOG.md
git commit -m "docs: add contributing notes, security policy, reporting section"
```

---

### Task 4: Milestone step in the release routine

**Files:**
- Modify: `CLAUDE.md` ("Project: releasing the Windows port")
- Modify: `README.md` ("## Releasing")
- Modify: `docs/BUILDING-WINDOWS.md` ("## Releasing", line 108 onward)

**Interfaces:**
- Consumes: milestone naming from Decisions.

- [ ] **Step 1: Add the steps in all three places, worded to match each file**

- Before step 2 (release notes): "Check the `<version>` milestone. Move
  open issues that won't make it to the next milestone. Known issues in the
  notes cite their issue as `(#N)`."
- After the tag's release is published: "Close the milestone:
  `gh api -X PATCH repos/fabiosouzajr/shortwave-windows/milestones/<number> -f state=closed`
  and create the next one."

In CLAUDE.md, extend "Docs to keep honest" with: "each row of the 'Not
working / not tested' table that needs work links its issue".

- [ ] **Step 2: Verify the three files agree**

Read the three Releasing sections side by side.
Expected: same steps in the same order. Only the level of detail differs.

- [ ] **Step 3: Commit**

```bash
git add CLAUDE.md README.md docs/BUILDING-WINDOWS.md
git commit -m "docs(release): add milestone steps to the release routine"
```

---

### Task 5: Tracking issues for known gaps (public, ask first)

**Files:**
- Modify: `docs/WINDOWS-PORT-STATUS.md:54-69` (add an `Issue` column to "Not working / not tested")

**Interfaces:**
- Consumes: labels (Task 1), milestone (Task 1).

- [ ] **Step 1: Confirm the list with the user**

Proposed issues, one per row. "Labels" are added on top of what the row needs:

| Row | Title | Labels |
|---|---|---|
| Station map | Station map is hidden on Windows (libshumate requests no tiles) | `bug`, `confirmed` |
| Toast buttons | Verify Save Track / Don't Record toast buttons | `bug`, `needs-triage` |
| Chromecast end-to-end | Test Chromecast casting end to end | `help wanted` |
| Per-machine install + firewall rule | Test per-machine install and firewall rule | `confirmed` |
| Hardware media keys | Test physical media keys | `good first issue` |
| Features added 2026-11-21/22 | Check new search, heart, flag and language features in the running app | `confirmed`, milestone `5.0.0-win.2` |
| Both translation rows | Untranslated port strings (filters, heart, tray, Location) | `enhancement` |
| Dark theme switch | Verify dark mode follows Windows | `needs-triage` |
| System proxy | Use Windows proxy settings | `enhancement` |

No issue for "Windows Sandbox / VM clean test" (agreed as not needed) or
"CI release job" (it gets verified on the next tag and the row then goes away).
Each issue body has three parts: the row's status text, what "done" means,
and a link to the status doc.

- [ ] **Step 2: Create the approved issues**

Run per issue: `gh issue create --title "<title>" --body-file <tmp file in scratchpad> --label <labels>`
Record each number.

- [ ] **Step 3: Add the `Issue` column**

Column values: `#N` links (`[#N](https://github.com/fabiosouzajr/shortwave-windows/issues/N)`),
`—` for the two rows without one.

- [ ] **Step 4: Verify**

Run: `gh issue list --state open --limit 50`
Expected: every `#N` in the table is open and its title matches its row.

- [ ] **Step 5: Commit**

```bash
git add docs/WINDOWS-PORT-STATUS.md
git commit -m "docs(status): link known gaps to their issues"
```

---

### Task 6: Publish and verify on GitHub

**Files:** none.

- [ ] **Step 1: Push (ask first)**

Run: `git push origin main`

- [ ] **Step 2: Enable private vulnerability reporting (ask first)**

Run: `gh api -X PUT repos/fabiosouzajr/shortwave-windows/private-vulnerability-reporting`
then `gh api repos/fabiosouzajr/shortwave-windows/private-vulnerability-reporting`
Expected: `{"enabled":true}`.

- [ ] **Step 3: Check the chooser (Review Focus 2, 4)**

Open `https://github.com/fabiosouzajr/shortwave-windows/issues/new/choose`.
Expected: "Bug report" and "Feature request" forms, the four contact links,
no template error banner, no "Open a blank issue" link.

- [ ] **Step 4: Check form labels (Review Focus 1)**

Ask the user whether a test issue is acceptable. If yes, open one from
"Bug report" titled `Test: issue form`. Check it has `bug` and `needs-triage`,
then close it as `invalid` and delete it (`gh issue delete <N> --yes`).
If not, check the form page's sidebar preview lists both labels.

- [ ] **Step 5: Check the community profile**

Run: `gh api repos/fabiosouzajr/shortwave-windows/community/profile --jq '.files | {contributing: (.contributing != null), issue_template: (.issue_template != null)}'`
Expected: both `true`. SECURITY.md shows under the repo's Security tab →
Policy.
