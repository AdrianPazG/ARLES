#!/usr/bin/env python3
"""Extraccion cromatica de /REFERENCIA_DE_COLOR (solo lectura)."""
import numpy as np
import pathlib
from PIL import Image
import json, sys, pathlib

SRC = str(pathlib.Path(__file__).parent.parent / "REFERENCIA_DE_COLOR" / "farm-lifestyle-digital-art.jpg")
OUT = str(pathlib.Path(__file__).parent)

im = Image.open(SRC)
print(f"formato={im.format} size={im.size} mode={im.mode}")
im = im.convert("RGB")
W, H = im.size
arr = np.asarray(im, dtype=np.float64) / 255.0
flat = arr.reshape(-1, 3)
print(f"pixeles totales: {flat.shape[0]:,}")

# ---------- sRGB -> OKLab ----------
def srgb_to_linear(c):
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)

def linear_to_srgb(c):
    c = np.clip(c, 0, 1)
    return np.where(c <= 0.0031308, c * 12.92, 1.055 * (c ** (1 / 2.4)) - 0.055)

M1 = np.array([[0.4122214708, 0.5363325363, 0.0514459929],
               [0.2119034982, 0.6806995451, 0.1073969566],
               [0.0883024619, 0.2817188376, 0.6299787005]])
M2 = np.array([[0.2104542553, 0.7936177850, -0.0040720468],
               [1.9779984951, -2.4285922050, 0.4505937099],
               [0.0259040371, 0.7827717662, -0.8086757660]])

def rgb_to_oklab(rgb):
    lin = srgb_to_linear(rgb)
    lms = lin @ M1.T
    lms = np.cbrt(np.maximum(lms, 0))
    return lms @ M2.T

def oklab_to_rgb(lab):
    lms = lab @ np.linalg.inv(M2).T
    lms = lms ** 3
    lin = lms @ np.linalg.inv(M1).T
    return linear_to_srgb(lin)

def hexof(rgb01):
    r, g, b = (np.clip(np.asarray(rgb01), 0, 1) * 255).round().astype(int)
    return f"#{r:02X}{g:02X}{b:02X}"

# ---------- WCAG luminancia / contraste ----------
def rel_lum(rgb01):
    lin = srgb_to_linear(np.asarray(rgb01))
    return float(0.2126 * lin[0] + 0.7152 * lin[1] + 0.0722 * lin[2])

def contrast(c1, c2):
    l1, l2 = rel_lum(c1), rel_lum(c2)
    hi, lo = max(l1, l2), min(l1, l2)
    return (hi + 0.05) / (lo + 0.05)

# muestreo determinista
rng = np.random.default_rng(20260831)
N = 400_000
idx = rng.choice(flat.shape[0], size=min(N, flat.shape[0]), replace=False)
sample = flat[idx]
lab = rgb_to_oklab(sample)

# ---------- k-means en OKLab ----------
def kmeans(X, k, iters=60, seed=7):
    r = np.random.default_rng(seed)
    # k-means++
    centers = [X[r.integers(len(X))]]
    for _ in range(k - 1):
        d = np.min(((X[:, None, :] - np.array(centers)[None, :, :]) ** 2).sum(-1), axis=1)
        p = d / d.sum()
        centers.append(X[r.choice(len(X), p=p)])
    C = np.array(centers)
    for _ in range(iters):
        d = ((X[:, None, :] - C[None, :, :]) ** 2).sum(-1)
        lbl = d.argmin(1)
        newC = np.array([X[lbl == j].mean(0) if (lbl == j).any() else C[j] for j in range(k)])
        if np.allclose(newC, C, atol=1e-6):
            C = newC
            break
        C = newC
    d = ((X[:, None, :] - C[None, :, :]) ** 2).sum(-1)
    lbl = d.argmin(1)
    return C, lbl

K = 12
sub = lab[rng.choice(len(lab), size=60_000, replace=False)]
C, _ = kmeans(sub, K)
d = ((lab[:, None, :] - C[None, :, :]) ** 2).sum(-1)
lbl = d.argmin(1)

rows = []
for j in range(K):
    m = lbl == j
    share = m.mean() * 100
    center_lab = lab[m].mean(0)
    rgb = oklab_to_rgb(center_lab[None, :])[0]
    L, a, b = center_lab
    chroma = float(np.hypot(a, b))
    hue = float((np.degrees(np.arctan2(b, a)) + 360) % 360)
    rows.append(dict(hex=hexof(rgb), share=round(share, 2), L=round(float(L), 3),
                     C=round(chroma, 4), h=round(hue, 1), lum=round(rel_lum(rgb), 4)))
rows.sort(key=lambda r: -r["share"])
print("\n=== CLUSTERS DOMINANTES (k=12, OKLab) ===")
print(f"{'hex':9} {'% area':>7} {'L':>6} {'croma':>7} {'hue':>6}  familia")
def familia(r):
    h, c, L = r["h"], r["C"], r["L"]
    if c < 0.03:
        return "neutro (crema/gris)" if L > 0.6 else ("neutro medio" if L > 0.3 else "neutro oscuro")
    if 200 <= h <= 290: return "AZUL"
    if 170 <= h < 200: return "CIAN/TURQUESA"
    if 290 < h <= 330: return "violeta"
    if 80 <= h < 170: return "verde"
    if 60 <= h < 80: return "amarillo-verde"
    if 45 <= h < 60: return "AMARILLO"
    if 20 <= h < 45: return "ORO/OCRE"
    return "rojo/naranja"
for r in rows:
    print(f"{r['hex']:9} {r['share']:6.2f}% {r['L']:6.3f} {r['C']:7.4f} {r['h']:6.1f}  {familia(r)}")

# ---------- mineria de acentos: alto croma aunque poca area ----------
print("\n=== ACENTOS (alto croma, por familia de tono) ===")
L_, a_, b_ = lab[:, 0], lab[:, 1], lab[:, 2]
chroma = np.hypot(a_, b_)
hue = (np.degrees(np.arctan2(b_, a_)) + 360) % 360
familias = {
    "AZUL PROFUNDO":   ((hue >= 230) & (hue <= 290) & (L_ < 0.45)),
    "AZUL MEDIO":      ((hue >= 220) & (hue <= 285) & (L_ >= 0.45) & (L_ < 0.68)),
    "AZUL CLARO/CIAN": ((hue >= 190) & (hue <= 240) & (L_ >= 0.62)),
    "CIAN/TURQUESA":   ((hue >= 165) & (hue < 200)),
    "AMARILLO":        ((hue >= 45) & (hue < 75) & (chroma > 0.06)),
    "ORO/OCRE":        ((hue >= 25) & (hue < 55) & (chroma > 0.05) & (L_ < 0.82)),
    "CREMA":           ((chroma <= 0.055) & (L_ > 0.78)),
    "OSCUROS":         (L_ < 0.28),
}
for name, mask in familias.items():
    n = int(mask.sum())
    if n < 50:
        print(f"{name:16} -> presencia despreciable ({n} px de muestra)")
        continue
    sel = lab[mask]
    selc = chroma[mask]
    # representante = percentil alto de croma (color "puro" de la familia), y mediana
    p90 = sel[selc >= np.percentile(selc, 90)].mean(0)
    med = np.median(sel, axis=0)
    rgb90 = oklab_to_rgb(p90[None, :])[0]
    rgbmed = oklab_to_rgb(med[None, :])[0]
    print(f"{name:16} {n/len(lab)*100:5.2f}% area | mediana {hexof(rgbmed)} (L={med[0]:.2f}) | saturado {hexof(rgb90)} (L={p90[0]:.2f})")

# ---------- bandas verticales: composicion de la imagen ----------
print("\n=== COMPOSICION POR BANDAS HORIZONTALES (de arriba a abajo) ===")
small = np.asarray(im.resize((160, 212), Image.LANCZOS), dtype=np.float64) / 255.0
for i in range(8):
    band = small[i * 212 // 8:(i + 1) * 212 // 8].reshape(-1, 3)
    m = band.mean(0)
    lb = rgb_to_oklab(m[None, :])[0]
    print(f"banda {i+1}/8: {hexof(m)}  L={lb[0]:.2f} croma={np.hypot(lb[1],lb[2]):.3f} hue={(np.degrees(np.arctan2(lb[2],lb[1]))+360)%360:.0f}")

# ---------- exportar miniatura + tira de paleta ----------
im.resize((720, int(720 * H / W)), Image.LANCZOS).save(f"{OUT}/referencia_720.png")
strip = Image.new("RGB", (K * 120, 160), "black")
from PIL import ImageDraw
dr = ImageDraw.Draw(strip)
for i, r in enumerate(rows):
    dr.rectangle([i * 120, 0, (i + 1) * 120, 160], fill=r["hex"])
strip.save(f"{OUT}/clusters.png")
json.dump(rows, open(f"{OUT}/clusters.json", "w"), indent=1)
print("\nOK -> referencia_720.png, clusters.png, clusters.json")
