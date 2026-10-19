#!/usr/bin/env bash
# Builds the macOS app bundle and packs the release zip: dist/mboxer-<version>-macos.zip holding
# mboxer.app and usage.md, with SHA256SUMS.txt next to it (task 018, D-07). The macOS counterpart of
# scripts/release.ps1; it runs on a Mac or on the macos-latest runner, never on Windows.
#
# The version is the workspace Cargo.toml one, the only place it is written. The script builds the
# release profile for aarch64-apple-darwin and x86_64-apple-darwin, joins the two with lipo into one
# universal binary, lays out mboxer.app (Info.plist with the io.github.speti4.mboxer identifier and the
# .mbox document type, icon.icns from the committed iconset through iconutil), signs the bundle ad hoc
# (codesign -s -; no Developer ID, no notarization: the user allows it once under Privacy & Security,
# D-07) and zips it with ditto, which keeps the bundle's structure, permissions and extended attributes.
# No Tauri CLI (D-02): the bundle is these ~100 lines.
#
#   scripts/bundle-macos.sh                      # build, bundle, pack into dist/
#   scripts/bundle-macos.sh --skip-build         # pack what target/<target>/release already holds
#   scripts/bundle-macos.sh --out /tmp/dist      # where the zip and the checksum file go
#   scripts/bundle-macos.sh --tag v0.2.0         # CI: the tag must equal "v<version>"
#
# Needs: the Rust toolchain of rust-toolchain.toml with both darwin targets
# (rustup target add aarch64-apple-darwin x86_64-apple-darwin) and the Xcode command line tools
# (lipo, iconutil, codesign, plutil, ditto).
set -euo pipefail

skip_build=0
out=dist
tag=""
while [ $# -gt 0 ]; do
    case "$1" in
        --skip-build) skip_build=1 ;;
        --out) out="$2"; shift ;;
        --tag) tag="$2"; shift ;;
        -h|--help) sed -n '2,21p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
    shift
done

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

identifier=io.github.speti4.mboxer
targets=(aarch64-apple-darwin x86_64-apple-darwin)
iconset="$root/crates/mboxer-app/icons/icon.iconset"
guide="$root/docs/usage.md"

# The [workspace.package] version of Cargo.toml, as release.ps1 reads it.
version="$(sed -n '/^\[workspace\.package\]/,/^\[/{s/^version = "\(.*\)"$/\1/p;}' Cargo.toml)"
if [ -z "$version" ]; then
    echo "no [workspace.package] version in Cargo.toml" >&2
    exit 1
fi
if [ -n "$tag" ] && [ "$tag" != "v$version" ]; then
    echo "tag '$tag' does not match the Cargo.toml version 'v$version'" >&2
    exit 1
fi

# What can fail fast runs before the build: the tools, the targets and the inputs.
if ! command -v cargo > /dev/null && [ -x "$HOME/.cargo/bin/cargo" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi
for tool in cargo rustup lipo iconutil codesign plutil ditto shasum; do
    command -v "$tool" > /dev/null || { echo "$tool not found" >&2; exit 1; }
done
if [ "$skip_build" -eq 0 ]; then
    installed="$(rustup target list --installed)"
    for target in "${targets[@]}"; do
        grep -qx "$target" <<< "$installed" || {
            echo "target $target is not installed: rustup target add ${targets[*]}" >&2
            exit 1
        }
    done
fi
[ -d "$iconset" ] || { echo "missing: $iconset (python -I scripts/make-icon.py writes it)" >&2; exit 1; }
[ -f "$guide" ] || { echo "missing: $guide" >&2; exit 1; }

if [ "$skip_build" -eq 0 ]; then
    for target in "${targets[@]}"; do
        cargo build --release -p mboxer-app --target "$target"
    done
fi
binaries=()
for target in "${targets[@]}"; do
    binary="$root/target/$target/release/mboxer"
    [ -f "$binary" ] || { echo "missing: $binary" >&2; exit 1; }
    binaries+=("$binary")
done

stage="$(mktemp -d "${TMPDIR:-/tmp}/mboxer-release-$version.XXXXXX")"
trap 'rm -rf "$stage"' EXIT
app="$stage/mboxer.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

# One universal binary from the two builds; `lipo -archs` prints "x86_64 arm64" for it.
lipo -create "${binaries[@]}" -output "$app/Contents/MacOS/mboxer"
chmod 755 "$app/Contents/MacOS/mboxer"
archs="$(lipo -archs "$app/Contents/MacOS/mboxer")"

iconutil -c icns -o "$app/Contents/Resources/icon.icns" "$iconset"

# The bundle's metadata. LSMinimumSystemVersion is Tauri 2's default (10.13 High Sierra, the oldest
# WKWebView that wry supports); CFBundleDocumentTypes registers .mbox so Finder offers mboxer for the
# file (opening it from the Finder is task 019), declared as an imported type under public.data so the
# app never claims every data file. Keys that macOS reads from the executable's version resource on
# Windows (tauri-build) are all here instead.
cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>mboxer</string>
    <key>CFBundleExecutable</key>
    <string>mboxer</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>CFBundleIdentifier</key>
    <string>$identifier</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>mboxer</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$version</string>
    <key>CFBundleVersion</key>
    <string>$version</string>
    <key>CFBundleSupportedPlatforms</key>
    <array>
        <string>MacOSX</string>
    </array>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.productivity</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>MIT License</string>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>mbox mailbox</string>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSHandlerRank</key>
            <string>Alternate</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>$identifier.mbox</string>
            </array>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>mbox</string>
            </array>
        </dict>
    </array>
    <key>UTImportedTypeDeclarations</key>
    <array>
        <dict>
            <key>UTTypeIdentifier</key>
            <string>$identifier.mbox</string>
            <key>UTTypeDescription</key>
            <string>mbox mailbox</string>
            <key>UTTypeConformsTo</key>
            <array>
                <string>public.data</string>
            </array>
            <key>UTTypeTagSpecification</key>
            <dict>
                <key>public.filename-extension</key>
                <array>
                    <string>mbox</string>
                </array>
            </dict>
        </dict>
    </array>
</dict>
</plist>
PLIST
plutil -lint "$app/Contents/Info.plist"
printf 'APPL????' > "$app/Contents/PkgInfo"

# Ad hoc: a signature without an identity, which is what Apple Silicon needs to load the binary at all;
# Gatekeeper still asks the user once (D-07).
codesign --force --deep --sign - "$app"
codesign --verify --deep --strict "$app"

cp "$guide" "$stage/usage.md"

mkdir -p "$out"
out="$(cd "$out" && pwd)"
zip="$out/mboxer-$version-macos.zip"
rm -f "$zip"
# As Finder's Compress does: a zip whose root holds mboxer.app and usage.md, resource forks and extended
# attributes sequestered into __MACOSX, symlinks kept.
ditto -c -k --sequesterRsrc "$stage" "$zip"

# "<sha256>  <file name>" and a LF, the `sha256sum -c` / `shasum -a 256 -c` format, as release.ps1
# writes it for the Windows zip; the release job joins the two files.
sums="$out/SHA256SUMS.txt"
hash="$(shasum -a 256 "$zip" | cut -d' ' -f1)"
printf '%s  %s\n' "$hash" "mboxer-$version-macos.zip" > "$sums"

size_mb="$(du -m "$zip" | cut -f1)"
echo "version:  $version"
echo "binary:   $archs ($(du -m "$app/Contents/MacOS/mboxer" | cut -f1) MB)"
echo "zip:      $zip ($size_mb MB)"
echo "sha256:   $hash ($sums)"
