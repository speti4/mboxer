---
status: idea
size: S
origin: wip/028-publication-day-repo-settings.md
created: 2026-10-19
---

# The usage guide in the zips in a format that opens with a double-click

What: `release.ps1` and `bundle-macos.sh` put `usage.md` into the zips; Windows has no default app for `.md` and
the tables read poorly as plain text. Render it to `usage.html` next to `THIRD-PARTY-LICENSES.html` (or ship
`.txt`), without a new runtime dependency.
Why: the guide is for people who only run the program.
Done when: both zips hold a guide that opens in the default browser or text editor and reads well.
