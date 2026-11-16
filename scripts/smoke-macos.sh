#!/usr/bin/env bash
# Start-up smoke test of the macOS bundle (task 073): opens mboxer.app the way the Finder does, waits for
# a window of its process on the screen, and stops the app. It answers one question, whether the packed
# app shows its window, whole and on the screen; what the webview renders is not checked. It runs on a
# Mac or on the macos-latest runner, in a logged-in session (the window server has to be there), never
# on Windows.
#
# The app starts through `open`, so LaunchServices launches it as a double click does, with a fresh
# instance and its output in files. A window counts when it belongs to the app's process, is on the
# screen, sits on the normal window layer and is at least 100 points wide and high, and it has to stand
# whole on the main display, as a window placed for a larger screen does not: the list comes from
# CGWindowListCopyWindowInfo, which gives the owner and the bounds of a window without the screen
# recording permission. The out folder keeps stdout.log, stderr.log, system.txt, a screenshot when the
# session allows one and, when no window came, a `sample` of the process, which shows where its main
# thread stands.
#
#   scripts/smoke-macos.sh /Applications/mboxer.app
#   scripts/smoke-macos.sh dist/unpacked/mboxer.app --out dist/smoke   # where the evidence goes
#   scripts/smoke-macos.sh mboxer.app --timeout 60                     # seconds to wait for the window
#
# Exit code: 0 a whole window appeared, 1 none did within the timeout, the app ended first or a part of
# the window lies off the display, 2 a usage or set-up error. Needs: the Xcode command line tools
# (swift), open, pgrep, sample, screencapture, sw_vers.
set -euo pipefail

app=""
out=""
timeout=30
while [ $# -gt 0 ]; do
    case "$1" in
        --out) out="$2"; shift ;;
        --timeout) timeout="$2"; shift ;;
        -h|--help) sed -n '2,23p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        -*) echo "unknown argument: $1" >&2; exit 2 ;;
        *) [ -z "$app" ] || { echo "one app only: $1" >&2; exit 2; }; app="$1" ;;
    esac
    shift
done
[ -n "$app" ] || { echo "usage: scripts/smoke-macos.sh <mboxer.app> [--out <folder>] [--timeout <seconds>]" >&2; exit 2; }
[ -d "$app" ] || { echo "missing: $app" >&2; exit 2; }
case "$timeout" in
    ''|*[!0-9]*) echo "--timeout takes whole seconds: $timeout" >&2; exit 2 ;;
esac
for tool in swift open pgrep sample screencapture sw_vers; do
    command -v "$tool" > /dev/null || { echo "$tool not found" >&2; exit 2; }
done

app="$(cd "$app" && pwd)"
binary="$app/Contents/MacOS/mboxer"
[ -x "$binary" ] || { echo "missing: $binary" >&2; exit 2; }

work="$(mktemp -d "${TMPDIR:-/tmp}/mboxer-smoke.XXXXXX")"
if [ -z "$out" ]; then
    out="$work/out"
fi
mkdir -p "$out"
out="$(cd "$out" && pwd)"

pid=""
cleanup() {
    if [ -n "$pid" ] && kill -0 "$pid" 2> /dev/null; then
        kill "$pid" 2> /dev/null || true
    fi
    rm -rf "$work/windows.swift"
}
trap cleanup EXIT

{
    sw_vers
    uname -m
} > "$out/system.txt"

# Waits for a window of the process: prints "<window id> <width>x<height> at <x>,<y>" and exits 0 when one
# stands whole on the main display, 3 when a part of it lies off the display, 1 at the timeout and 2 when the
# process is gone. The window is measured a second after it appeared, once the app has placed it.
cat > "$work/windows.swift" <<'SWIFT'
import CoreGraphics
import Foundation

let pid = Int32(CommandLine.arguments[1])!
let deadline = Date().addingTimeInterval(Double(CommandLine.arguments[2])!)

func mainWindow() -> (id: Int, frame: CGRect)? {
    let options: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
    let windows = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] ?? []
    for window in windows {
        guard let owner = window[kCGWindowOwnerPID as String] as? Int, owner == Int(pid),
              let layer = window[kCGWindowLayer as String] as? Int, layer == 0,
              let bounds = window[kCGWindowBounds as String] as? [String: Any],
              let x = bounds["X"] as? Double, let y = bounds["Y"] as? Double,
              let width = bounds["Width"] as? Double, let height = bounds["Height"] as? Double,
              width >= 100, height >= 100
        else { continue }
        let id = window[kCGWindowNumber as String] as? Int ?? 0
        return (id, CGRect(x: x, y: y, width: width, height: height))
    }
    return nil
}

while mainWindow() == nil {
    if kill(pid, 0) != 0 { exit(2) }
    if Date() >= deadline { exit(1) }
    Thread.sleep(forTimeInterval: 0.5)
}
Thread.sleep(forTimeInterval: 1.0)
guard let window = mainWindow() else { exit(kill(pid, 0) != 0 ? 2 : 1) }
let frame = window.frame
print("\(window.id) \(Int(frame.width))x\(Int(frame.height)) at \(Int(frame.minX)),\(Int(frame.minY))")
// Window bounds and display bounds share one space: points, the origin at the main display's top left.
let display = CGDisplayBounds(CGMainDisplayID())
exit(display.contains(frame) ? 0 : 3)
SWIFT

# -n: a fresh instance even if one runs; the output files are what a Finder start throws away.
open -n "$app" --stdout "$out/stdout.log" --stderr "$out/stderr.log"

# LaunchServices returns before the process is there.
for _ in $(seq 1 50); do
    pid="$(pgrep -n -f "$binary" || true)"
    [ -z "$pid" ] || break
    sleep 0.2
done
if [ -z "$pid" ]; then
    echo "smoke: the app did not start: no process of $binary" >&2
    [ ! -s "$out/stderr.log" ] || cat "$out/stderr.log" >&2
    exit 1
fi

status=0
found="$(swift "$work/windows.swift" "$pid" "$timeout")" || status=$?

# The whole screen, as the session shows it; a session without the permission just has no picture.
screencapture -x "$out/screen.png" 2> /dev/null || echo "smoke: no screenshot in this session" >&2

echo "system:  $(sw_vers -productVersion) ($(uname -m))"
echo "out:     $out"
case "$status" in
    0)
        echo "window:  ${found#* } (id ${found%% *}, pid $pid)"
        ;;
    3)
        echo "window:  ${found#* } (id ${found%% *}, pid $pid)"
        echo "smoke: a part of the window lies off the main display" >&2
        ;;
    1)
        # Where the main thread stands when the app runs without a window.
        sample "$pid" 3 -file "$out/sample.txt" > /dev/null 2>&1 || true
        echo "smoke: no window within $timeout s; the app still runs (pid $pid)" >&2
        ;;
    *)
        echo "smoke: the app ended before it showed a window" >&2
        ;;
esac
if [ "$status" -ne 0 ]; then
    if [ -s "$out/stderr.log" ]; then
        echo "--- stderr.log" >&2
        cat "$out/stderr.log" >&2
    fi
    exit 1
fi
