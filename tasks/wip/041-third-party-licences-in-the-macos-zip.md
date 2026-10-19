---
size: S
origin: done/018-macos-build-bundle-and-ci.md
created: 2026-10-19
started: 2026-10-19
plan: C:\Users\peter\.claude\plans\n-zz-k-t-a-backlogot-toasty-dove.md
milestone: M4 macOS
---

# 041 — `THIRD-PARTY-LICENSES.html` in the macOS zip

## Context

What: the Windows zip carries the licence page that cargo-about renders (025, D-012); the macOS zip of
`scripts/bundle-macos.sh` does not. Run cargo-about over the two darwin targets in the `bundle-macos` job (the
macOS dependency graph adds objc2 and friends) and put the page into the zip next to `usage.md`.
Why: the licence page is part of what a release ships (D-012); the macOS zip must not be the exception.
Done when: the macOS zip holds a licence page that lists the darwin-only crates, and `cargo deny check` still
passes with the macOS targets.
User decision 2026-10-19: part of the backlog wave 039 + 041 + 020 in parallel task sessions; 041 lands before
the first release tag (028), which builds the macOS zip too.

## Steps

1. [x] how `scripts/release.ps1` and the `release` job render the page today (`about.toml`, the pinned
   `CARGO_ABOUT_VERSION`, the template), so the macOS side reuses the same config and version
2. [x] the `bundle-macos` job installs the same cargo-about and renders the page for `aarch64-apple-darwin` and
   `x86_64-apple-darwin`; `scripts/bundle-macos.sh` copies it into the zip root next to `usage.md` (deviation:
   a Linux job renders it, see below)
3. [x] `deny.toml` / `about.toml` cover the darwin targets; `cargo deny check` passes with them
4. [x] docs: `docs/development.md` (the macOS bundle), the header comment of `scripts/bundle-macos.sh`
5. [ ] local checks (CONTRIBUTING.md), `uvx zizmor .github`; a manual `bundle-macos` run on the branch shows the
   page in the zip with the darwin-only crates (objc2 …)

## Log

- 2026-10-19 — promoted on `task/041-third-party-licences-in-the-macos-zip`.
- 2026-10-19 — Felmérés: a `release.ps1` `cargo about generate --frozen --fail` hívással, az `about.toml`-lal
  (`targets = ["x86_64-pc-windows-msvc"]`) és a `scripts/third-party-licenses.hbs` sablonnal rendereli a
  lapot; a `release` job a 0.9.2-es cargo-about-ot `cargo install --locked`-kal építi. A cargo-about 0.9.2
  `generate --target` kapcsolója több targetet is elfogad és felülírja a config `targets` értékét, így a
  macOS-lap ugyanazzal az `about.toml`-lal készül, második config nélkül. A `deny.toml` `[graph] targets`
  már a 018 óta mindhárom targetet tartalmazza, ott nincs teendő.
- 2026-10-19 — Helyi próba Windowson: `cargo about generate --frozen --fail --target aarch64-apple-darwin
  --target x86_64-apple-darwin …` hibátlanul lefut, 272 crate (Windows: 278); csak a macOS-lapon: objc2,
  objc2-app-kit, objc2-foundation, objc2-web-kit, objc2-core-foundation, objc2-quartz-core,
  objc2-exception-helper, objc2-encode, block2, dispatch2, core-foundation(-sys), core-graphics(-types),
  foreign-types*, swift-rs, embed_plist, rustix, errno, base64 0.21; csak a Windows-lapon a `windows-*`,
  `webview2-com*`, `winapi*`, softbuffer, tracing*. A gráf tehát hosztfüggetlen.
- 2026-10-19 — Döntés: hol fut a cargo-about. A macOS runneren a `cargo install cargo-about` pár perces
  fordítás 10× áron (042), minden tagen és kézi futáson; a lap viszont csak a lock fájltól és a crate-forrásoktól
  függ, nem a hoszttól. Ezért egy új `licences-macos` job ubuntu-latest-en rendereli (`cargo fetch --locked`
  a két darwin targetre, majd `--frozen` offline generálás), egynapos artifactként adja át, a `bundle-macos`
  letölti és becsomagolja. A job a checkekkel párhuzamosan indul (nem `needs`), így a `bundle-macos` nem vár rá
  külön. A parancs egy helyen él: `scripts/bundle-macos.sh --licences-only <fájl>` (bármely OS-en, cargo és
  cargo-about kell hozzá), a csomagolás `--licences <fájl>`-lal veszi át; kapcsoló nélkül egy Macen a script maga
  rendereli. A `CARGO_ABOUT_VERSION` pin a workflow szintű `env`-be került, a `release` és a `licences-macos`
  job ugyanazt használja.
- 2026-10-19 — A sablon bevezetője eddig Windows-specifikus volt (`mboxer.exe`, „WebView2, part of Windows
  11”); platformsemleges lett (exe vagy app, WebView2 vagy WKWebView). A release notes „Verify the download”
  szövege (`scripts/changelog.py`) is mindkét zipre utal már.
- 2026-10-19 — Első kézi futás (run 39260328239): a `licences-macos` elbukott: `cargo metadata` exited with an
  error: failed to download `android_system_properties v0.1.6` … `--frozen was specified`. Ok: a cargo-about
  szűretlen `cargo metadata`-val olvassa a gráfot, ami minden platform crate-jeinek forrását kéri, a
  `cargo fetch --target …` (és egy build) viszont csak a saját targetjéét tölti le. Helyben azért ment, mert a
  `~/.cargo/registry` már mindent tartalmazott. Friss `CARGO_HOME`-mal reprodukálva: `cargo fetch --locked
  --target x86_64-pc-windows-msvc` után a Windows-lap ugyanígy bukik — vagyis a `release` job (025) is elbukott
  volna az első tagen tiszta Windows runneren (tagen még sosem futott, a 025 csak helyi dry runnal ellenőrzött).
  Javítás mindkét scriptben: `cargo fetch --locked` (minden target) közvetlenül a `--frozen` generálás előtt;
  friss `CARGO_HOME`-mal utána a macOS- és a Windows-lap is hibátlanul elkészül.

## Deviations from the plan

- 2. lépés: a cargo-about nem a `bundle-macos` jobban (macOS runner) fut, hanem egy Linux jobban, és a
  `bundle-macos` csak becsomagolja a lapot — ugyanaz az eredmény a macOS-percek nélkül (indoklás a Logban).
- A sablon és a release notes szövege is változott (a lap már nem csak az exe-ről szól).
- `scripts/release.ps1` is kapott egy `cargo fetch --locked`-ot a cargo-about elé: ugyanaz a hiba a Windows
  release jobot is eltörte volna az első tagen (Log), a 041 a 028 első release-e előtt landol.

## Verification

- 

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
