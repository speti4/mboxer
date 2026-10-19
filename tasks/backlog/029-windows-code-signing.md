---
status: waiting
waits-for: SignPath Foundation accepts the project (D-013)
size: M
origin: session 2026-10-18
created: 2026-10-18
milestone: M5 Public release
---

# Authenticode signing of the release exe

What: wire SignPath Foundation signing into the release job (D-013): `release.ps1` split into build and pack, the
unsigned exe uploaded and signed through SignPath's GitHub action (pinned by SHA, `SIGNPATH_API_TOKEN` secret),
`Get-AuthenticodeSignature` checked for Valid, then pack, hash and attest; the code signing policy section in the
README and on the release page; the SmartScreen paragraph of `docs/usage.md` updated.
Why: an unsigned exe downloaded from the internet meets "Windows protected your PC" on every new version.
Done when: a tagged release's exe is Valid under `Get-AuthenticodeSignature`, and its zip still matches
`SHA256SUMS.txt` and its attestation.
