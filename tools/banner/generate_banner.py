#!/usr/bin/env python3
"""k1s0 README バナー GIF を生成する。

タイムライン (約 7.5 秒 / 15 fps):
  0.0-0.5s  プロンプト + カーソル待機
  0.5-3.0s  "keep it simple, 0 vendor lock-in" を打鍵
  3.0-3.5s  ホールド
  3.5-4.5s  K / I / S / 0 をシアン化、他文字をフェードアウト
  4.5-5.0s  "kis0" にクロスフェード
  5.0-5.2s  i → 1 グリッチスワップ
  5.2-7.5s  "k1s0" ロゴ + サブタイトルでホールド (カーソル無し / ループ点)

出力: docs/assets/banner.gif
"""

from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

W, H = 1280, 320
BG = (13, 17, 23)
FG = (230, 237, 243)
ACCENT = (95, 207, 255)
DIM = (110, 118, 129)
GHOST = (40, 46, 53)

FONT_PATH = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf"
F_TYPE = ImageFont.truetype(FONT_PATH, 44)
F_LOGO = ImageFont.truetype(FONT_PATH, 110)
F_SUB = ImageFont.truetype(FONT_PATH, 24)

FPS = 15
FRAME_MS = int(1000 / FPS)

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs" / "assets" / "banner.gif"

PROMPT = "$ "
FULL = "keep it simple, 0 vendor lock-in"
KEEPERS = {0, 5, 8, 16}  # k(0) i(5) s(8) 0(16)
SUBTITLE = "Keep It Simple, 0 Vendor Lock-in"


def measure(d, text, font):
    bbox = d.textbbox((0, 0), text, font=font)
    return bbox[2] - bbox[0], bbox[3] - bbox[1]


def render_typing(text_with_colors, cursor_visible):
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    full_str = PROMPT + "".join(c for c, _ in text_with_colors)
    tw, th = measure(d, full_str, F_TYPE)
    x0 = (W - tw) // 2
    y = (H - th) // 2
    d.text((x0, y), PROMPT, fill=DIM, font=F_TYPE)
    pw, _ = measure(d, PROMPT, F_TYPE)
    cx = x0 + pw
    for ch, col in text_with_colors:
        d.text((cx, y), ch, fill=col, font=F_TYPE)
        cw, _ = measure(d, ch, F_TYPE)
        cx += cw
    if cursor_visible:
        d.rectangle([cx + 2, y + 4, cx + 16, y + th + 2], fill=ACCENT)
    return img


def render_logo(text="k1s0", subtitle=None):
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    tw, th = measure(d, text, F_LOGO)
    x = (W - tw) // 2
    y = (H - th) // 2 - (20 if subtitle else 0)
    cx = x
    for ch in text:
        col = ACCENT if ch in ("1", "0") else FG
        d.text((cx, y), ch, fill=col, font=F_LOGO)
        cw, _ = measure(d, ch, F_LOGO)
        cx += cw
    if subtitle:
        sw, sh = measure(d, subtitle, F_SUB)
        sx = (W - sw) // 2
        sy = y + th + 30
        d.text((sx, sy), subtitle, fill=DIM, font=F_SUB)
    return img, cx, y, th


def render_logo_frame(cursor_visible):
    img, cx, y, th = render_logo("k1s0", subtitle=SUBTITLE)
    if cursor_visible:
        d = ImageDraw.Draw(img)
        d.rectangle([cx + 8, y + 18, cx + 36, y + th], fill=ACCENT)
    return img


def lerp_color(a, b, t):
    return tuple(int(a[i] * (1 - t) + b[i] * t) for i in range(3))


def main():
    frames, durations = [], []

    # Phase 1: 空プロンプト + カーソル点滅 (8f)
    for i in range(8):
        cur = (i // 3) % 2 == 0
        frames.append(render_typing([], cur))
        durations.append(FRAME_MS)

    # Phase 2: タイピング (1 char / frame, カンマで小休止)
    typed = []
    for ch in FULL:
        typed.append((ch, FG))
        frames.append(render_typing(typed, True))
        durations.append(FRAME_MS)
        if ch == ",":
            for _ in range(2):
                frames.append(render_typing(typed, True))
                durations.append(FRAME_MS)

    # Phase 3: ホールド (8f)
    for i in range(8):
        cur = (i // 3) % 2 == 0
        frames.append(render_typing(typed, cur))
        durations.append(FRAME_MS)

    # Phase 4a: keeper をシアン化（1 つずつ、各 2f）
    for k_idx in [0, 5, 8, 16]:
        for _ in range(2):
            new = []
            for i, (ch, col) in enumerate(typed):
                if i in KEEPERS and i <= k_idx:
                    new.append((ch, ACCENT))
                else:
                    new.append((ch, col))
            typed = new
            frames.append(render_typing(typed, False))
            durations.append(FRAME_MS)

    # Phase 4b: 非 keeper を FG → GHOST にディゾルブ (4 ステップ)
    for step in range(4):
        t = (step + 1) / 4
        rendered = []
        for i, (ch, col) in enumerate(typed):
            if i in KEEPERS:
                rendered.append((ch, ACCENT))
            else:
                rendered.append((ch, lerp_color(FG, GHOST, t)))
        frames.append(render_typing(rendered, False))
        durations.append(FRAME_MS)

    # Phase 4c: 全文 (dimmed) → "kis0" センターへクロスフェード (6f)
    # kis0 はすべて keeper として cyan を維持
    src = render_typing(
        [(ch, ACCENT if i in KEEPERS else GHOST) for i, (ch, _) in enumerate(typed)],
        False,
    )
    tgt_kis0 = render_typing(
        [("k", ACCENT), ("i", ACCENT), ("s", ACCENT), ("0", ACCENT)], False
    )
    for step in range(6):
        a = (step + 1) / 6
        frames.append(Image.blend(src, tgt_kis0, a))
        durations.append(FRAME_MS)

    # kis0 ホールド (6f)
    for _ in range(6):
        frames.append(tgt_kis0)
        durations.append(FRAME_MS)

    # Phase 5: i → 1 グリッチスワップ (3f: |, 1, 1) — 全文字 cyan のまま
    for ch in ["|", "1", "1"]:
        f = render_typing(
            [("k", ACCENT), (ch, ACCENT), ("s", ACCENT), ("0", ACCENT)], False
        )
        frames.append(f)
        durations.append(FRAME_MS)

    # Phase 6: フォントサイズ補間で "k1s0" を拡大、k/s を白へ、サブタイトル淡入 (10f)
    for step in range(10):
        t = (step + 1) / 10
        size_pt = int(44 + (110 - 44) * t)
        f = ImageFont.truetype(FONT_PATH, size_pt)
        k_col = lerp_color(ACCENT, FG, t)
        s_col = lerp_color(ACCENT, FG, t)
        img = Image.new("RGB", (W, H), BG)
        d = ImageDraw.Draw(img)
        text = "k1s0"
        tw, th = measure(d, text, f)
        x = (W - tw) // 2
        # サブタイトル下に余白を確保するため上方シフトも補間
        y = (H - th) // 2 - int(20 * t)
        cx = x
        for i, ch in enumerate(text):
            if ch == "k":
                col = k_col
            elif ch == "s":
                col = s_col
            else:  # 1, 0
                col = ACCENT
            d.text((cx, y), ch, fill=col, font=f)
            cw, _ = measure(d, ch, f)
            cx += cw
        if t >= 0.4:
            sub_alpha = (t - 0.4) / 0.6
            sub_col = lerp_color(BG, DIM, sub_alpha)
            sw, _ = measure(d, SUBTITLE, F_SUB)
            d.text(((W - sw) // 2, y + th + 30), SUBTITLE, fill=sub_col, font=F_SUB)
        frames.append(img)
        durations.append(FRAME_MS)

    # Phase 7: 最終ホールド (ループ点) — カーソル無し (32f)
    final = render_logo_frame(False)
    for _ in range(32):
        frames.append(final)
        durations.append(FRAME_MS)

    # GIF 書き出し（パレット 64 色、optimize、disposal=2）
    OUT.parent.mkdir(parents=True, exist_ok=True)
    quantized = [
        f.convert("RGB").quantize(method=Image.MEDIANCUT, colors=64) for f in frames
    ]
    quantized[0].save(
        OUT,
        save_all=True,
        append_images=quantized[1:],
        duration=durations,
        loop=0,
        optimize=True,
        disposal=2,
    )
    total_ms = sum(durations)
    size_kb = OUT.stat().st_size / 1024
    print(
        f"saved {OUT.relative_to(ROOT)} — {len(frames)} frames / "
        f"{total_ms / 1000:.2f}s / {size_kb:.1f} KB"
    )


if __name__ == "__main__":
    main()
