#!/usr/bin/env python3
"""
Extracción reproducible de la paleta de ARLES RELAY.

Lee REFERENCIA_DE_COLOR/ en su resolución original y reproduce las cifras
de docs/COLOR_SYSTEM.md. NO modifica ningún archivo.

    pip install pillow numpy
    python3 tools/extract_palette.py
"""
import glob
import os
import sys

import numpy as np
from PIL import Image

Image.MAX_IMAGE_PIXELS = None

REF_DIR = "REFERENCIA_DE_COLOR"
KMEANS_K = 12
KMEANS_STEP = 4          # submuestreo espacial
KMEANS_SEED = 7          # fijo, para que el resultado sea reproducible

REGIONS = {
    "Cielo cenital (0-8%)":       (0.00, 0.08, 0.30, 0.70),
    "Cielo profundo (12-22%)":    (0.12, 0.22, 0.35, 0.75),
    "Cielo medio (25-40%)":       (0.25, 0.40, 0.40, 0.80),
    "Cielo bajo / azur (45-55%)": (0.45, 0.55, 0.45, 0.85),
    "Nube brillante (50-58%)":    (0.50, 0.58, 0.82, 1.00),
    "Horizonte dorado (60-64%)":  (0.60, 0.64, 0.05, 0.95),
    "Campo medio (68-76%)":       (0.68, 0.76, 0.05, 0.95),
    "Sombras follaje (78-88%)":   (0.78, 0.88, 0.05, 0.95),
}

TOKENS = {
    "text-primary": "#F2F6F8", "text-secondary": "#AFC6D6", "text-tertiary": "#8AA6B8",
    "yellow": "#F5D21F", "gold": "#F0AC0E", "azure": "#2B9DC3",
    "blue-light": "#8CC5D4", "blue-primary": "#0F759E",
    "success": "#3DC98F", "danger": "#F0645C", "info": "#5FB8DC",
}
GROUNDS = {
    "bg-deep": "#04101B", "bg": "#071C2C", "surface": "#0B2739",
    "surface-raised": "#103247", "surface-inter": "#17405A",
}


def srgb_to_linear(c):
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)


def rgb_to_lab(rgb):
    """sRGB [0,1] -> CIELAB con blanco D65."""
    r, g, b = (srgb_to_linear(rgb[..., i]) for i in range(3))
    x = r * 0.4124564 + g * 0.3575761 + b * 0.1804375
    y = r * 0.2126729 + g * 0.7151522 + b * 0.0721750
    z = r * 0.0193339 + g * 0.1191920 + b * 0.9503041

    def f(t):
        d = 6 / 29
        return np.where(t > d ** 3, np.cbrt(t), t / (3 * d * d) + 4 / 29)

    fx, fy, fz = f(x / 0.95047), f(y / 1.0), f(z / 1.08883)
    return np.stack([116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz)], axis=-1)


def to_hex(rgb01):
    return "#%02X%02X%02X" % tuple(int(round(min(max(v, 0), 1) * 255)) for v in rgb01)


def from_hex(h):
    h = h.lstrip("#")
    return [int(h[i:i + 2], 16) / 255 for i in (0, 2, 4)]


def relative_luminance(rgb01):
    c = srgb_to_linear(np.asarray(rgb01, dtype=float))
    return float(0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2])


def contrast_ratio(fg_hex, bg_hex):
    a, b = relative_luminance(from_hex(fg_hex)), relative_luminance(from_hex(bg_hex))
    hi, lo = max(a, b), min(a, b)
    return (hi + 0.05) / (lo + 0.05)


def kmeans_lab(lab, k, seed):
    rng = np.random.default_rng(seed)
    centroids = lab[rng.choice(len(lab), k, replace=False)].copy()
    labels = np.zeros(len(lab), dtype=int)
    for _ in range(60):
        dist = ((lab[:, None, :] - centroids[None, :, :]) ** 2).sum(-1)
        labels = dist.argmin(1)
        updated = np.array([
            lab[labels == i].mean(0) if (labels == i).any() else centroids[i]
            for i in range(k)
        ])
        if np.allclose(updated, centroids, atol=1e-4):
            break
        centroids = updated
    return centroids, labels


def find_reference():
    files = sorted(
        f for f in glob.glob(os.path.join(REF_DIR, "*"))
        if os.path.isfile(f) and not f.endswith(".md")
    )
    if not files:
        sys.exit(f"No se encontró ninguna imagen en {REF_DIR}/")
    # la de mayor resolución es la autoridad (§13)
    best, best_px = None, -1
    for f in files:
        try:
            with Image.open(f) as im:
                if im.width * im.height > best_px:
                    best, best_px = f, im.width * im.height
        except Exception:
            continue
    if best is None:
        sys.exit(f"Ningún archivo legible como imagen en {REF_DIR}/")
    return best


def main():
    path = find_reference()
    with Image.open(path) as raw:
        fmt, size = raw.format, raw.size
        source = (raw.info.get("XML:com.adobe.xmp") or raw.info.get("xmp") or b"")
        img = raw.convert("RGB")

    if isinstance(source, bytes):
        source = source.decode("utf-8", "replace")
    ai_generated = "trainedAlgorithmicMedia" in source

    arr = np.asarray(img, dtype=np.float64) / 255.0
    h, w, _ = arr.shape

    print(f"Archivo   : {path}")
    print(f"Formato   : {fmt}" + ("  (la extensión no coincide)" if fmt == "PNG"
                                  and path.lower().endswith((".jpg", ".jpeg")) else ""))
    print(f"Resolución: {size[0]} x {size[1]} = {size[0] * size[1]:,} px")
    print(f"Origen    : {'generada por IA (trainedAlgorithmicMedia)' if ai_generated else 'no declarado'}")

    print("\n=== A. ANÁLISIS POR REGIÓN (mediana) ===")
    for name, (y0, y1, x0, x1) in REGIONS.items():
        sub = arr[int(y0 * h):int(y1 * h), int(x0 * w):int(x1 * w)].reshape(-1, 3)
        med = np.median(sub, axis=0)
        lab = rgb_to_lab(med[None, None, :])[0, 0]
        print(f"  {name:30} {to_hex(med)}  L*={lab[0]:5.1f}")

    print(f"\n=== B. K-MEANS K={KMEANS_K} EN CIELAB (paso {KMEANS_STEP}px, semilla {KMEANS_SEED}) ===")
    small = arr[::KMEANS_STEP, ::KMEANS_STEP].reshape(-1, 3)
    lab = rgb_to_lab(small)
    centroids, labels = kmeans_lab(lab, KMEANS_K, KMEANS_SEED)
    counts = np.bincount(labels, minlength=KMEANS_K)
    print(f"  muestras: {len(small):,}")
    print(f"  {'%':>7}  {'HEX':9} {'L*':>6} {'a*':>6} {'b*':>6}  familia")
    for i in np.argsort(-counts):
        rep = small[labels == i].mean(0)
        L, A, B = centroids[i]
        chroma = (A * A + B * B) ** 0.5
        family = ("AZUL" if B < -12 else "AMARILLO/ORO" if B > 30
                  else "NEUTRO" if chroma < 10 else "mixto")
        print(f"  {100 * counts[i] / len(lab):6.2f}%  {to_hex(rep):9} "
              f"{L:6.1f} {A:6.1f} {B:6.1f}  {family}")

    print("\n=== C. CONTRASTE WCAG 2.2 DE LOS TOKENS ===")
    print(f"  {'token':16}{'hex':10}" + "".join(f"{g:>16}" for g in GROUNDS))
    failures = []
    for name, hexv in TOKENS.items():
        row = f"  {name:16}{hexv:10}"
        for gname, ghex in GROUNDS.items():
            r = contrast_ratio(hexv, ghex)
            mark = "" if r >= 4.5 else (" (grande)" if r >= 3.0 else " FALLA")
            if r < 3.0:
                failures.append((name, gname, r))
            row += f"{r:>8.2f}{mark:<8}"
        print(row)

    print("\n  AA: >=4.5 texto normal | >=3.0 texto grande y elementos no textuales")
    if failures:
        print("\n  Pares que NO alcanzan 3.0 (uso solo como relleno, nunca como tinta):")
        for name, ground, r in failures:
            print(f"    {name} sobre {ground}: {r:.2f}")

    print("\n=== D. TEXTO OSCURO SOBRE ACENTOS ===")
    dark = "#04101B"
    for name in ("yellow", "gold", "success", "danger", "azure"):
        print(f"  {dark} sobre {TOKENS[name]} ({name}): "
              f"{contrast_ratio(dark, TOKENS[name]):5.2f}   "
              f"vs texto claro #F2F6F8: {contrast_ratio('#F2F6F8', TOKENS[name]):5.2f}")


if __name__ == "__main__":
    main()
