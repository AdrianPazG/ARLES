/**
 * Localiza el Chromium con el que corren las sondas.
 *
 * Las tres sondas llevaban la ruta escrita a mano
 * (`/opt/pw-browsers/chromium-1194/…`), que es la de un contenedor concreto.
 * Fuera de él no encontraban nada, y como el validador **omite** una sonda que
 * no puede correr, el resultado habría sido tres comprobaciones en gris que
 * nadie mira — precisamente las que encontraron el defecto más grave de la
 * fase.
 *
 * El orden de búsqueda va de lo explícito a lo adivinado, y cada paso se
 * puede explicar:
 *
 * 1. `ARLES_CHROMIUM` — quien lo pone sabe lo que hace y manda.
 * 2. `chromium.executablePath()` de Playwright, **si existe en disco**: es el
 *    camino normal tras `npx playwright-core install chromium`. Se comprueba
 *    la existencia porque el método devuelve la ruta esperada aunque no se
 *    haya descargado nada.
 * 3. Un rastreo de `PLAYWRIGHT_BROWSERS_PATH` (o `/opt/pw-browsers`), que es
 *    como quedan los contenedores que traen el navegador preinstalado con una
 *    versión distinta a la que espera la biblioteca.
 * 4. Un Chromium del sistema.
 *
 * Si no hay ninguno devuelve `null` **con el motivo**, para que la sonda diga
 * por qué no puede correr en vez de reventar con un rastreo de pila.
 */
import { existsSync, readdirSync } from 'node:fs'
import { join } from 'node:path'

import { chromium } from 'playwright-core'

const RAIZ_DE_NAVEGADORES = process.env['PLAYWRIGHT_BROWSERS_PATH'] ?? '/opt/pw-browsers'

const DEL_SISTEMA = [
  '/usr/bin/chromium',
  '/usr/bin/chromium-browser',
  '/usr/bin/google-chrome',
  '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
]

function rastrearInstalaciones() {
  if (!existsSync(RAIZ_DE_NAVEGADORES)) return null
  // Las carpetas llevan la revisión en el nombre (`chromium-1194`,
  // `chromium_headless_shell-1194`). Se ordenan al revés para preferir la más
  // reciente cuando hay varias.
  const carpetas = readdirSync(RAIZ_DE_NAVEGADORES)
    .filter((n) => n.startsWith('chromium'))
    .sort()
    .reverse()

  for (const carpeta of carpetas) {
    for (const interior of ['chrome-linux/chrome', 'chrome-linux64/chrome']) {
      const ruta = join(RAIZ_DE_NAVEGADORES, carpeta, interior)
      if (existsSync(ruta)) return ruta
    }
  }
  return null
}

/** @returns {{ruta: string} | {ruta: null, motivo: string}} */
export function buscarChromium() {
  const explicito = process.env['ARLES_CHROMIUM']
  if (explicito) {
    if (existsSync(explicito)) return { ruta: explicito }
    return { ruta: null, motivo: `ARLES_CHROMIUM apunta a ${explicito}, que no existe` }
  }

  try {
    const dePlaywright = chromium.executablePath()
    if (existsSync(dePlaywright)) return { ruta: dePlaywright }
  } catch {
    // `executablePath()` lanza si la biblioteca no sabe dónde buscar. No es
    // un fallo: quedan las otras vías.
  }

  const rastreado = rastrearInstalaciones()
  if (rastreado) return { ruta: rastreado }

  const delSistema = DEL_SISTEMA.find((r) => existsSync(r))
  if (delSistema) return { ruta: delSistema }

  return {
    ruta: null,
    motivo:
      'no se encontró Chromium. Instálalo con `npx playwright-core install ' +
      'chromium` dentro de app/, o apunta ARLES_CHROMIUM a uno existente',
  }
}

/**
 * Como `buscarChromium`, pero termina el proceso con un mensaje claro si no
 * hay navegador. Es lo que usan las sondas: sin navegador no hay nada que
 * medir, y fingir que sí lo hay sería peor.
 */
export function exigirChromium() {
  const encontrado = buscarChromium()
  if (encontrado.ruta) return encontrado.ruta
  console.error(`  – sonda omitida: ${encontrado.motivo}`)
  process.exit(2)
}
