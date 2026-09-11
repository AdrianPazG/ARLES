# Informes de fase para Dirección

Convierte el Markdown de un informe de fase en un PDF con la identidad de ARLES.

```bash
pip install reportlab
python3 herramientas/informe-direccion/generar-pdf.py \
    documentacion/09-fases/FASE-01-PARA-DIRECCION.md
```

Deja dos archivos junto al `.md`:

| Archivo | Qué es |
|---|---|
| `FASE-01-PARA-DIRECCION.pdf` | El PDF que se entrega |
| `FASE-01-PARA-DIRECCION.pdf.sha256` | Huella del `.md` con el que se generó |

---

## Por qué existe la huella

Un PDF es un archivo binario: nadie ve en una revisión que se quedó atrás
respecto al texto del que salió. La huella lo convierte en algo comprobable —
el validador de fase compara el hash del `.md` actual con el registrado, y
**falla si el documento se editó y el PDF no se regeneró**.

Misma lógica que `dist/arles-tokens.css` frente a `tokens.json`: el artefacto
generado se versiona, pero nunca se edita a mano.

---

## Por qué Helvetica y no Mont

La licencia de Mont que hay hoy (**P-01**, sin resolver) no cubre incrustar la
fuente en artefactos que salen de la empresa, y un PDF para Dirección lo es.
Hasta que P-01 se cierre, el PDF usa una fuente base de PDF.

Cuando se resuelva, se registran las Mont con `pdfmetrics.registerFont` y se
sustituyen los nombres `Helvetica*` de `estilos()`. Un solo punto de cambio.

---

## Subconjunto de Markdown que entiende

Encabezados (`#`–`####`), párrafos, **negrita**, *cursiva*, `código`, enlaces
—se imprime el texto, no la URL—, listas con viñeta y numeradas, tablas,
citas, bloques de código y reglas horizontales.

No entiende imágenes ni listas anidadas. Si un informe las necesita, se añaden
aquí; **no se maqueta el PDF por fuera**, porque entonces deja de regenerarse.
