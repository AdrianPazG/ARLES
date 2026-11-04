#!/usr/bin/env python3
"""Convierte la guía imprimible de WhatsApp en el PDF que lee Dirección.

── POR QUÉ EXISTE ──────────────────────────────────────────────────────────

Dirección pidió el PDF el 04/11/2026, al no poder dar de alta su número con el
instalador: «dame un pdf para vincular whatsapp con Arles». El contenido vive
en `documentacion/07-entrega/guia-whatsapp/guia.html` y el PDF se genera de
ahí.

**El PDF no se escribe a mano y el HTML sí.** Si se editaran los dos, el día
que alguien corrija una frase en uno, el otro seguiría diciendo lo de antes —
y el que Dirección abre es el PDF. Un documento que se cree es peor que uno
que falta.

── POR QUÉ CHROMIUM Y NO UNA BIBLIOTECA DE PDF ─────────────────────────────

Porque ya está en el proyecto: lo usan las seis sondas. Una dependencia nueva
para esto tendría que pasar `cargo deny`/`npm audit` y mantenerse, y lo que
hace falta es imprimir una página que ya sabemos maquetar con CSS.

── CÓMO SE USA ─────────────────────────────────────────────────────────────

    python3 herramientas/guia-pdf/generar.py

Deja `documentacion/07-entrega/guia-whatsapp/ARLES-conectar-WhatsApp.pdf`.
Sin Chromium **no se inventa un PDF viejo**: aborta con su motivo y código de
salida 2, igual que las sondas. Un PDF de ayer presentado como el de hoy es la
trampa 2 de las herramientas (`CLAUDE.md`), que ya costó una sonda midiendo
otra versión del código.
"""

import os
import subprocess
import sys

RAIZ = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CARPETA = os.path.join(RAIZ, "documentacion", "07-entrega", "guia-whatsapp")
FUENTE = os.path.join(CARPETA, "guia.html")
DESTINO = os.path.join(CARPETA, "ARLES-conectar-WhatsApp.pdf")

# El pie va dentro del PDF y no en el CSS porque el número de página sólo lo
# sabe el motor de impresión.
PIE = (
    '<div style="width:100%;font-size:8pt;color:#46585f;padding:0 16mm;'
    'font-family:Segoe UI,Arial,sans-serif;display:flex;'
    'justify-content:space-between">'
    "<span>ARLES RELAY I · Conectar WhatsApp</span>"
    '<span class="pageNumber"></span></div>'
)

GUION = """
import { chromium } from 'playwright-core'
import { exigirChromium } from './pruebas/sondas/navegador.mjs'

const [fuente, destino, pie] = process.argv.slice(2)
const nav = await chromium.launch({
  executablePath: exigirChromium(),
  args: ['--no-sandbox'],
})
const p = await nav.newPage()
await p.goto(`file://${fuente}`, { waitUntil: 'load' })
// Las fuentes del sistema pueden tardar en resolverse, y un PDF impreso antes
// de que carguen sale con otra tipografía y otra paginación.
await p.waitForTimeout(400)
await p.pdf({
  path: destino,
  format: 'A4',
  printBackground: true,
  displayHeaderFooter: true,
  headerTemplate: '<div></div>',
  footerTemplate: pie,
  margin: { top: '18mm', bottom: '16mm', left: '16mm', right: '16mm' },
})
await nav.close()
"""


def main() -> int:
    if not os.path.exists(FUENTE):
        print(f"No está la fuente: {FUENTE}", file=sys.stderr)
        return 2

    # El guion se escribe DENTRO de `app/`, porque `playwright-core` y el
    # buscador de Chromium se resuelven desde ahí. Se borra siempre, también
    # si la impresión falla: un archivo suelto en `app/` entra en el lint.
    guion = os.path.join(RAIZ, "app", ".guia-pdf.mjs")
    with open(guion, "w", encoding="utf-8") as f:
        f.write(GUION)
    try:
        r = subprocess.run(
            ["node", ".guia-pdf.mjs", FUENTE, DESTINO, PIE],
            cwd=os.path.join(RAIZ, "app"),
            capture_output=True,
            text=True,
        )
    finally:
        os.remove(guion)

    if r.returncode != 0:
        # Código 2 es el que usa `exigirChromium` cuando no hay navegador: se
        # distingue de un fallo de verdad, igual que en las sondas.
        print(r.stderr.strip() or "No se pudo imprimir el PDF.", file=sys.stderr)
        return 2

    tam = os.path.getsize(DESTINO)
    print(f"✓ {os.path.relpath(DESTINO, RAIZ)}  ·  {tam // 1024} kB")
    return 0


if __name__ == "__main__":
    sys.exit(main())
