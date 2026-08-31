#!/usr/bin/env python3
"""Deriva la rampa ARLES desde los anclas extraidos y VERIFICA contraste WCAG 2.2."""
import numpy as np
import pathlib, json

M1 = np.array([[0.4122214708,0.5363325363,0.0514459929],
               [0.2119034982,0.6806995451,0.1073969566],
               [0.0883024619,0.2817188376,0.6299787005]])
M2 = np.array([[0.2104542553,0.7936177850,-0.0040720468],
               [1.9779984951,-2.4285922050,0.4505937099],
               [0.0259040371,0.7827717662,-0.8086757660]])
iM1, iM2 = np.linalg.inv(M1), np.linalg.inv(M2)

def s2l(c): return np.where(c<=0.04045, c/12.92, ((c+0.055)/1.055)**2.4)
def l2s(c):
    c=np.clip(c,0,1); return np.where(c<=0.0031308, c*12.92, 1.055*(c**(1/2.4))-0.055)
def hex2rgb(h):
    h=h.lstrip('#'); return np.array([int(h[i:i+2],16) for i in (0,2,4)])/255
def rgb2hex(r):
    r=(np.clip(r,0,1)*255).round().astype(int); return "#%02X%02X%02X"%tuple(r)
def rgb2lab(rgb): return (np.cbrt(np.maximum(s2l(np.asarray(rgb))@M1.T,0)))@M2.T
def lab2rgb(lab): return l2s(((np.asarray(lab)@iM2.T)**3)@iM1.T)
def lab2lch(lab):
    L,a,b=lab; return np.array([L,np.hypot(a,b),(np.degrees(np.arctan2(b,a))+360)%360])
def lch2rgb(L,C,h):
    a,b=C*np.cos(np.radians(h)),C*np.sin(np.radians(h)); return lab2rgb([L,a,b])
def in_gamut(rgb, eps=1e-4): return bool(np.all(rgb>=-eps) and np.all(rgb<=1+eps))
def lch2hex(L,C,h):
    """Reduce croma hasta entrar en gamut sRGB (gamut mapping simple)."""
    for k in np.linspace(1,0,101):
        rgb=lch2rgb(L,C*k,h)
        if in_gamut(rgb): return rgb2hex(rgb), round(float(C*k),4)
    return rgb2hex(lch2rgb(L,0,h)), 0.0
def lum(h):
    lin=s2l(hex2rgb(h)); return float(0.2126*lin[0]+0.7152*lin[1]+0.0722*lin[2])
def ratio(a,b):
    l1,l2=lum(a),lum(b); hi,lo=max(l1,l2),min(l1,l2); return round((hi+0.05)/(lo+0.05),2)

# ---------- ANCLAS medidos en /REFERENCIA_DE_COLOR ----------
ANCLAS = {
 "cielo profundo":"#055480", "cielo dominante":"#0B6690", "cielo medio":"#0E739E",
 "azul claro":"#2586A9", "azul saturado":"#28A0CE", "cian palido":"#92C1C5",
 "nube crema":"#DFE2D3", "girasol":"#F7D42C", "oro":"#EAA11C",
 "ambar profundo":"#C77617", "tierra":"#61401D", "oscuro":"#182126",
}
print("=== ANCLAS -> OKLCH ===")
H = {}
for k,v in ANCLAS.items():
    L,C,h = lab2lch(rgb2lab(hex2rgb(v)))
    H[k]=(L,C,h)
    print(f"{k:16} {v}  L={L:.3f} C={C:.3f} h={h:6.1f}")

HUE_AZUL   = H["cielo dominante"][2]   # estructura
HUE_AZURE  = H["azul saturado"][2]     # interaccion
HUE_AMAR   = H["girasol"][2]           # acento
HUE_ORO    = H["oro"][2]               # advertencia
HUE_CREMA  = H["nube crema"][2]

print(f"\nhues rectores: azul={HUE_AZUL:.1f} azure={HUE_AZURE:.1f} amarillo={HUE_AMAR:.1f} oro={HUE_ORO:.1f}")

# ---------- RAMPA DE PROFUNDIDAD (dark-first, hue del cielo) ----------
# croma bajo y creciente ligeramente: superficies "azules", no grises ni negras
capas = [
 ("bg-deep",        0.155, 0.030),
 ("bg",             0.196, 0.034),
 ("surface",        0.240, 0.038),
 ("surface-raised", 0.286, 0.042),
 ("surface-inter",  0.335, 0.046),
 ("surface-select", 0.385, 0.060),
 ("border",         0.330, 0.030),
 ("border-strong",  0.430, 0.040),
]
TOK={}
print("\n=== RAMPA DE PROFUNDIDAD (hue azul del cielo) ===")
for name,L,C in capas:
    hx,cc = lch2hex(L,C,HUE_AZUL); TOK[name]=hx
    print(f"--arles-{name:16} {hx}  L={L:.3f} C={cc}")

# ---------- TEXTO ----------
texto = [("text-primary",0.945,0.012,HUE_CREMA),("text-secondary",0.780,0.020,HUE_AZUL),
         ("text-tertiary",0.660,0.024,HUE_AZUL),("text-disabled",0.520,0.020,HUE_AZUL),
         ("text-on-accent",0.200,0.030,HUE_AZUL)]
print("\n=== TEXTO ===")
for name,L,C,h in texto:
    hx,_=lch2hex(L,C,h); TOK[name]=hx; print(f"--arles-{name:16} {hx}")

# ---------- ACENTOS SEMANTICOS ----------
acentos = [
 ("blue-primary",  0.640, 0.130, HUE_AZURE),
 ("blue-light",    0.760, 0.105, HUE_AZURE),
 ("blue-deep",     0.480, 0.110, HUE_AZUL),
 ("yellow",        0.870, 0.170, HUE_AMAR),
 ("yellow-dim",    0.780, 0.150, HUE_AMAR),
 ("gold",          0.760, 0.150, HUE_ORO),
 ("cream",         0.905, 0.021, HUE_CREMA),
 ("info",          0.700, 0.120, HUE_AZURE),
 ("warning",       0.780, 0.150, HUE_ORO),
]
print("\n=== ACENTOS DERIVADOS DE LA REFERENCIA ===")
for name,L,C,h in acentos:
    hx,cc=lch2hex(L,C,h); TOK[name]=hx; print(f"--arles-{name:16} {hx}  L={L:.2f} C={cc}")

# ---------- EXTENSIONES (NO existen en la referencia: se declara) ----------
print("\n=== EXTENSIONES FUERA DE LA REFERENCIA (declaradas) ===")
ext=[("success",0.740,0.130,160.0),("danger",0.660,0.170,25.0),("danger-dim",0.560,0.150,25.0)]
for name,L,C,h in ext:
    hx,cc=lch2hex(L,C,h); TOK[name]=hx; print(f"--arles-{name:16} {hx}  L={L:.2f} C={cc} h={h}  [EXTENSION]")

# focus = amarillo girasol
TOK["focus"]=TOK["yellow"]

# ---------- VERIFICACION WCAG 2.2 ----------
print("\n=== VERIFICACION DE CONTRASTE (WCAG 2.2) ===")
def chk(fg,bg,need,label):
    r=ratio(TOK[fg],TOK[bg]); ok="OK " if r>=need else "FALLA"
    print(f"{ok} {r:5.2f}:1 (min {need})  {label}")
    return r>=need
fails=0
print("-- texto sobre fondos --")
for bg in ["bg-deep","bg","surface","surface-raised","surface-inter","surface-select"]:
    fails += not chk("text-primary",bg,4.5,f"text-primary / {bg}")
for bg in ["bg","surface","surface-raised"]:
    fails += not chk("text-secondary",bg,4.5,f"text-secondary / {bg}")
    fails += not chk("text-tertiary",bg,4.5,f"text-tertiary / {bg} (texto normal)")
print("-- acentos como TEXTO/ICONO sobre superficie --")
for t in ["blue-primary","blue-light","yellow","gold","success","danger","warning","info","cream"]:
    fails += not chk(t,"surface",4.5,f"{t} / surface")
print("-- texto oscuro sobre rellenos de acento (botones) --")
for fill in ["yellow","blue-primary","success","danger","gold"]:
    fails += not chk("text-on-accent",fill,4.5,f"text-on-accent / fill {fill}")
print("-- bordes / no-texto (min 3.0) --")
fails += not chk("border","surface",3.0,"border / surface  [componente]")
fails += not chk("border-strong","surface",3.0,"border-strong / surface")
fails += not chk("focus","bg",3.0,"focus ring / bg")
fails += not chk("focus","surface-raised",3.0,"focus ring / surface-raised")
print("-- separacion entre capas (percepcion de profundidad, no WCAG) --")
for a,b in [("bg-deep","bg"),("bg","surface"),("surface","surface-raised"),("surface-raised","surface-inter")]:
    print(f"     {ratio(TOK[a],TOK[b]):.2f}:1  {a} vs {b}")

print(f"\nRESULTADO: {'TODAS LAS COMPROBACIONES PASAN' if fails==0 else str(fails)+' FALLAS'}")
json.dump(TOK, open(str(pathlib.Path(__file__).parent/"tokens.json"),"w"), indent=1)
print("\n".join(f'  --arles-{k}: {v};' for k,v in TOK.items()))
