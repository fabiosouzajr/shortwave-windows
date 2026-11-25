# Contributing

This is an unofficial Windows port of
[GNOME Shortwave](https://gitlab.gnome.org/World/Shortwave). Problems with the
Windows build are tracked here.

## Reporting a problem

Where to report:

- **Something in the Windows build doesn't work:** open an issue here with
  the "Bug report" form. Ideas go in the "Feature request" form.
- **It also happens in Shortwave on Linux:** report it
  [upstream](https://gitlab.gnome.org/World/Shortwave/-/issues). The shared
  code is maintained there.
- **Wrong or broken station data** (names, stream URLs, tags): that comes from
  [radio-browser.info](https://www.radio-browser.info/). Fix it there.
- **A security vulnerability:** report it privately, see
  [SECURITY.md](SECURITY.md). Don't open a public issue.

What helps: the Shortwave version (About dialog), your Windows version, how
you installed it (installer or portable zip), the steps that cause the
problem, and the station if it's about one. Logs are optional: release builds
don't write any. See the README's
[FAQ](README.md#faq) for how to get them from a development build.

## Triage (for maintainers)

1. A new issue arrives with `needs-triage` from its form.
2. Look at it and give it exactly one of:
   - `confirmed`: reproduced or accepted.
   - `needs-info`: ask the reporter for what's missing. With no reply in 30
     days, close it with a note that it can be reopened.
   - `upstream`: it also happens in upstream Shortwave. Link the GitLab
     issue, or file one there if none exists. Keep it open until a release
     includes the fix.
   - Or close it as `duplicate`, `invalid` or `wontfix`, with a short reason.
3. Remove `needs-triage`.

| Label | Meaning |
|---|---|
| `needs-triage` | New, not yet looked at |
| `needs-info` | Waiting for the reporter |
| `confirmed` | Reproduced or accepted |
| `upstream` | Also in upstream Shortwave; tracked on GNOME GitLab |

Type labels (`bug`, `enhancement`, `question`, `documentation`) and the other
GitHub defaults are used as usual.

### If someone posts a vulnerability publicly

Remove the details: edit the issue, or delete it if editing isn't enough.
Comment with the
[private reporting link](https://github.com/fabiosouzajr/shortwave-windows/security/advisories/new)
and ask the reporter to use it. Don't discuss the details publicly.

## Fixes and releases

- Close issues from the fix: put `Fixes #N` in the commit message or pull
  request.
- Give each fixed or planned issue the milestone of the next release (named
  like the version, e.g. `5.0.0-win.2`).
- Known issues in the release notes cite their issue as `(#N)`.

## Code and builds

Building on Windows: [docs/BUILDING-WINDOWS.md](docs/BUILDING-WINDOWS.md).
New to the project? Start with
[build-aux/windows/setup.ps1](build-aux/windows/setup.ps1).
