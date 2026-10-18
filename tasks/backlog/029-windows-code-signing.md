---
status: idea
size: M
origin: session 2026-10-18
created: 2026-10-18
milestone: M5 Public release
---

# Authenticode signing of the release exe

What: decide how the Windows exe is signed against the SmartScreen warning: SignPath Foundation (free for open
source, needs the public repo), Azure Trusted Signing (paid, monthly), or unsigned with a usage note; then wire
the chosen path into the release job.
Why: an unsigned exe downloaded from the internet meets "Windows protected your PC", the first impression for
every new user once the repo is public.
Done when: a decision record exists, and with signing chosen the release exe carries a valid Authenticode
signature (`Get-AuthenticodeSignature` → Valid).
