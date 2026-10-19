"""Generates the application icon, a flat envelope glyph: crates/mboxer-app/icons/icon.ico
for Windows, icon.png for the other targets and the PNG set of icon.iconset/ for macOS.

One-off helper (Pillow), not part of the build: tauri-build embeds the committed .ico as the
Windows application icon, tauri's code generation wants the .png as the window icon on every
other target, and scripts/bundle-macos.sh turns the committed iconset into the icon.icns of the
.app with iconutil (task 018). Re-run after changing the drawing.

    python -I scripts/make-icon.py
"""

from pathlib import Path

from PIL import Image, ImageDraw

ICONS = Path(__file__).resolve().parent.parent / "crates" / "mboxer-app" / "icons"
OUT = ICONS / "icon.ico"
PNG = ICONS / "icon.png"
ICONSET = ICONS / "icon.iconset"
SIZE = 256
PNG_SIZE = 512
# The point sizes an iconset names; each comes at 1x and at 2x (`icon_16x16@2x.png` is 32 px).
ICONSET_POINTS = (16, 32, 128, 256, 512)
BACKGROUND = (31, 94, 160, 255)
PAPER = (245, 247, 250, 255)
FOLD = (205, 215, 228, 255)


def draw_icon(size: int) -> Image.Image:
    image = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    radius = size * 0.22
    draw.rounded_rectangle((0, 0, size - 1, size - 1), radius=radius, fill=BACKGROUND)
    left, top = size * 0.17, size * 0.28
    right, bottom = size * 0.83, size * 0.72
    draw.rectangle((left, top, right, bottom), fill=PAPER)
    middle_x, middle_y = size * 0.5, size * 0.53
    draw.polygon([(left, top), (right, top), (middle_x, middle_y)], fill=FOLD)
    draw.line([(left, top), (middle_x, middle_y), (right, top)], fill=BACKGROUND, width=max(1, size // 32))
    return image


def write_iconset() -> None:
    """One PNG per iconset entry, each drawn at its own pixel size rather than scaled."""
    ICONSET.mkdir(parents=True, exist_ok=True)
    for points in ICONSET_POINTS:
        for scale in (1, 2):
            suffix = "@2x" if scale == 2 else ""
            path = ICONSET / f"icon_{points}x{points}{suffix}.png"
            draw_icon(points * scale).save(path, format="PNG", optimize=True)
            print(f"wrote {path}")


def main() -> None:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    base = draw_icon(SIZE)
    base.save(OUT, format="ICO", sizes=[(16, 16), (32, 32), (48, 48), (256, 256)])
    print(f"wrote {OUT}")
    draw_icon(PNG_SIZE).save(PNG, format="PNG", optimize=True)
    print(f"wrote {PNG}")
    write_iconset()


if __name__ == "__main__":
    main()
