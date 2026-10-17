"""Generates crates/mboxer-app/icons/icon.ico: a flat envelope glyph.

One-off helper (Pillow), not part of the build: tauri-build embeds the committed
.ico as the Windows application icon. Re-run after changing the drawing.

    python scripts/make-icon.py
"""

from pathlib import Path

from PIL import Image, ImageDraw

OUT = Path(__file__).resolve().parent.parent / "crates" / "mboxer-app" / "icons" / "icon.ico"
SIZE = 256
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


def main() -> None:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    base = draw_icon(SIZE)
    base.save(OUT, format="ICO", sizes=[(16, 16), (32, 32), (48, 48), (256, 256)])
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
