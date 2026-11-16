/**
 * Las órdenes externas que lanzan las sondas: compilar con Vite y correr
 * Python.
 *
 * Cada sonda llamaba a `spawnSync('npx', …)` y `spawnSync('python3', …)`, que
 * sólo funciona en Linux y macOS. En Windows `npx` es `npx.cmd` y Node se niega
 * a lanzar un `.cmd` sin shell; y `python3` suele ser el acceso directo de la
 * Microsoft Store, no un intérprete. Las sondas fallaban antes de abrir el
 * navegador, con `r.stderr` vacío, así que ni siquiera decían por qué.
 *
 * Vite se lanza con el mismo Node que corre la sonda, sin pasar por `npx`.
 * Si la orden no se puede lanzar, se lanza un error que lo dice.
 */
import { spawnSync } from 'node:child_process'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'

const APP = fileURLToPath(new URL('../..', import.meta.url))
const VITE = join(APP, 'node_modules', 'vite', 'bin', 'vite.js')

/** `ARLES_PYTHON` manda; si no, el nombre habitual de cada sistema. */
const PYTHON = process.env['ARLES_PYTHON'] ?? (process.platform === 'win32' ? 'python' : 'python3')

function lanzar(programa, argumentos, opciones) {
  const r = spawnSync(programa, argumentos, opciones)
  if (r.error) {
    throw new Error(`no se pudo lanzar ${programa}: ${r.error.message}`)
  }
  return r
}

/** `vite <argumentos>`, con el Node de esta sonda. */
export function vite(argumentos, opciones) {
  return lanzar(process.execPath, [VITE, ...argumentos], opciones)
}

/** `python <argumentos>`, con el intérprete de este sistema. */
export function python(argumentos, opciones) {
  return lanzar(PYTHON, argumentos, opciones)
}
