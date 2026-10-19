---
status: decided
date: 2026-10-19
cost: cheap
sources: [backlog/029-windows-code-signing.md, done/028-publication-day-repo-settings.md, D-007-macos-distribution-adhoc-signed-app-zip.md]
---

# D-013 — The Windows exe is signed through SignPath Foundation once it accepts the project; until then it ships unsigned

## Context

The release exe is unsigned, so SmartScreen warns on the first start of every version. Since the switch the repo
is public, which free open-source signing programmes require. The options as of 2026-10-19: SignPath Foundation
(free for OSI-licensed public projects; official GitHub Actions integration; the key stays in its HSM; the
publisher shown is "SignPath Foundation"; acceptance is at its discretion and each release needs a manual
approval), Azure Artifact Signing (about $120 a year; open to individuals only in the USA and Canada), Certum's
open-source cloud certificate (€49, but headless signing only through an unofficial tool), an OV certificate
(about €200–330 a year, cloud HSM, the maintainer's legal name and city in the certificate, SmartScreen
reputation built from zero) and an EV certificate (no longer bypasses SmartScreen, so no benefit for its price).

## Decision

Apply to SignPath Foundation. Until the project is accepted, the exe ships unsigned with the SmartScreen note in
the usage guide, as today. Once accepted, the release job builds the exe, uploads it for signing, checks
`Get-AuthenticodeSignature` for Valid, and only then packs, hashes and attests, so `SHA256SUMS.txt` and the build
provenance cover the signed exe; the README and the release page carry the code signing policy SignPath asks
for. If the application is refused, the exe stays unsigned and the project applies again later; a paid OV
certificate is reconsidered only when the number of users justifies its cost. Decided on 2026-10-19, in line
with D-007 (no yearly fee for a free tool whose users can allow it once).

## Not chosen

- Azure Artifact Signing: not available to an individual maintainer in the EU.
- OV certificate: a yearly fee and a published legal name for a warning that still shows until reputation builds.
- EV certificate: Microsoft states that EV certificates no longer bypass SmartScreen.
- Certum open-source certificate: cheap, but signing in CI depends on an unofficial tool and a stored OTP seed.
