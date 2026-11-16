/**
 * El desplegable de tema hace lo que dice (C-1, C-2).
 *
 *     node app/pruebas/sondas/tema.mjs
 *
 * ─────────────────────────────────────────────────────────────────────────
 * QUÉ SE MIDE Y POR QUÉ
 *
 * La paleta clara existía y estaba medida desde el paso 2 —72 contratos de
 * contraste—, pero **no había forma de elegirla**: sólo se veía escribiendo
 * `data-tema="claro"` a mano en el inspector. Para quien usa ARLES, eso es lo
 * mismo que si no existiera.
 *
 * Esta sonda comprueba las cuatro afirmaciones que hace el desplegable, y
 * ninguna de ellas se puede comprobar con un test de unidad:
 *
 *   1. Elegir «Claro» **cambia el color que se ve**. No que cambie un atributo:
 *      que cambie el píxel. Un atributo bien escrito con los tokens mal
 *      enlazados da exactamente el mismo atributo y la misma pantalla oscura.
 *   2. La elección **se recuerda**. Se recarga la página y sigue clara.
 *   3. «Automático» **sigue al sistema**, y sigue siguiéndolo cuando el sistema
 *      cambia con ARLES ya abierto — que es lo que hacen macOS y Windows al
 *      anochecer.
 *   4. Las tres opciones del desplegable son **las tres que el núcleo acepta**.
 *      Si divergieran, elegir una opción de la lista daría un error de IPC.
 * ─────────────────────────────────────────────────────────────────────────
 *
 * Lo que NO prueba: que en macOS sea igual. Ahí el motor es WKWebView (R-07).
 */
import { readFile, rm } from 'node:fs/promises'
import http from 'node:http'
import { extname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

import { chromium } from 'playwright-core'

import { exigirChromium } from './navegador.mjs'
import { vite } from './ordenes.mjs'

const APP = fileURLToPath(new URL('../..', import.meta.url))
const SALIDA = join(APP, 'dist-tema')
const PUERTO = 4179

/** Las tres palabras que acepta `arles_app::comandos::TEMAS`, en su orden. */
const TEMAS = ['auto', 'oscuro', 'claro']

const EJECUTABLE = exigirChromium()

/**
 * Núcleo simulado **con memoria que sobrevive a recargar**.
 *
 * Aquí la memoria no es un detalle de la simulación: es la mitad de lo que se
 * mide. Y tiene que estar **fuera del documento**, porque lo que se comprueba
 * es precisamente qué pasa cuando el documento se vuelve a cargar: una variable
 * de este cierre se reinicia en cada recarga igual que se reiniciaría la
 * aplicación, y entonces la sonda no distinguiría «no se guardó» de «no se
 * volvió a leer».
 *
 * `sessionStorage` es lo más parecido a `ui_preference` que hay aquí: vive en
 * la pestaña, no en el documento. La aplicación **no** lo usa —las preferencias
 * van a la base cifrada, ver `V2__preferencias_de_interfaz.sql`—; es el falso
 * núcleo el que lo usa para hacer de base.
 */
function nucleoSimulado() {
  const leer = (clave, siNo) => {
    try {
      return window.sessionStorage.getItem(`sonda:${clave}`) ?? siNo
    } catch {
      return siNo
    }
  }
  const escribir = (clave, valor) => {
    try {
      window.sessionStorage.setItem(`sonda:${clave}`, String(valor))
    } catch {
      /* si no se puede, la comprobación de persistencia fallará y lo dirá */
    }
  }
  const estado = {
    get plegada() {
      return leer('plegada', '0') === '1'
    },
    set plegada(v) {
      escribir('plegada', v ? '1' : '0')
    },
    get tema() {
      return leer('tema', 'auto')
    },
    set tema(v) {
      escribir('tema', v)
    },
  }
  window.__TAURI_INTERNALS__ = {
    invoke(cmd, args) {
      if (cmd === 'info_app') {
        return Promise.resolve({
          nombre: 'ARLES RELAY',
          nombreComercial: 'ARLES RELAY I',
          version: '1.2.0',
          atribucion: 'Software desarrollado por TELEMETRY INSIGHT',
        })
      }
      if (cmd === 'configuracion_de_empresa') {
        return Promise.resolve({
          empresa: null,
          zonas: ['America/Mexico_City'],
          paises: ['MX'],
          onboarding: { pasos: [], completados: 0, total: 6, siguiente: 'empresa' },
        })
      }
      if (cmd === 'preferencias_de_interfaz') {
        return Promise.resolve({
          barraLateralPlegada: estado.plegada,
          tema: estado.tema,
        })
      }
      if (cmd === 'guardar_barra_plegada') {
        estado.plegada = args.plegada
        return Promise.resolve()
      }
      if (cmd === 'guardar_tema') {
        // La lista cerrada del núcleo, simulada: si la interfaz mandara algo
        // que Rust rechaza, aquí se ve igual que se vería en producción.
        if (!['auto', 'oscuro', 'claro'].includes(args.tema)) {
          return Promise.reject({ clave: 'error.app.tema_desconocido', detalle: args.tema })
        }
        estado.tema = args.tema
        return Promise.resolve()
      }
      return Promise.reject({ clave: 'error.db.sqlite', detalle: `desconocido: ${cmd}` })
    },
  }
}

let servidor
let navegador
let codigo = 0

function fallo(texto) {
  console.error(`\n✗ ${texto}`)
  codigo = 1
}

try {
  const build = vite(['build', '--outDir', 'dist-tema', '--emptyOutDir'], {
    cwd: APP,
    encoding: 'utf8',
  })
  if (build.status !== 0) {
    console.error(build.stderr || build.stdout)
    process.exit(1)
  }

  const TIPOS = {
    '.html': 'text/html',
    '.js': 'text/javascript',
    '.css': 'text/css',
    '.woff2': 'font/woff2',
    '.png': 'image/png',
  }
  servidor = http.createServer(async (req, res) => {
    const ruta = (req.url ?? '/').split('?')[0]
    const archivo = join(SALIDA, ruta === '/' ? 'index.html' : ruta)
    try {
      const cuerpo = await readFile(archivo)
      res.writeHead(200, { 'Content-Type': TIPOS[extname(archivo)] ?? 'application/octet-stream' })
      res.end(cuerpo)
    } catch {
      res.writeHead(404)
      res.end()
    }
  })
  await new Promise((r) => servidor.listen(PUERTO, r))

  navegador = await chromium.launch({ executablePath: EJECUTABLE })
  // El navegador arranca declarando tema oscuro: el estado de partida tiene que
  // ser conocido, porque «Automático» depende de él.
  const pagina = await navegador.newPage({
    viewport: { width: 1360, height: 900 },
    colorScheme: 'dark',
  })
  await pagina.addInitScript(nucleoSimulado)

  const irAAjustes = async () => {
    await pagina.goto(`http://localhost:${PUERTO}/#/ajustes`, { waitUntil: 'networkidle' })
    await pagina.waitForTimeout(400)
  }

  /** Lo que se ve, no lo que se declara: el fondo real del `<body>`. */
  const estado = () =>
    pagina.evaluate(() => ({
      atributo: document.documentElement.getAttribute('data-tema'),
      fondo: getComputedStyle(document.body).backgroundColor,
    }))

  const selector = '.apariencia select'

  await irAAjustes()

  // ── 4 · Las opciones son las que el núcleo acepta ──
  const opciones = await pagina.$$eval(`${selector} option`, (o) => o.map((x) => x.value))
  console.log(`  opciones del desplegable: ${opciones.join(', ')}`)
  if (opciones.join(',') !== TEMAS.join(',')) {
    fallo(
      `El desplegable ofrece [${opciones.join(', ')}] y el núcleo acepta ` +
        `[${TEMAS.join(', ')}].\n` +
        '  Elegir una opción que el núcleo rechaza da un error de IPC al\n' +
        '  guardar, y el usuario ve cambiar la pantalla sin que se recuerde.',
    )
  } else {
    console.log('✓ Las tres opciones son las que acepta el núcleo.')
  }

  // ── 1 · Elegir «Claro» cambia el color que se ve ──
  const antes = await estado()
  await pagina.selectOption(selector, 'claro')
  await pagina.waitForTimeout(300)
  const claro = await estado()
  console.log(`\n  oscuro: ${antes.atributo} ${antes.fondo}`)
  console.log(`  claro:  ${claro.atributo} ${claro.fondo}`)

  if (claro.atributo !== 'claro') {
    fallo(`Elegir «Claro» dejó data-tema en «${claro.atributo}».`)
  } else if (claro.fondo === antes.fondo) {
    fallo(
      `El atributo cambió pero el color NO: el fondo sigue en ${claro.fondo}.\n` +
        '  Un tema que sólo cambia un atributo es un tema que no existe: los\n' +
        '  tokens del tema claro no están llegando a la pantalla.',
    )
  } else {
    console.log('✓ Elegir «Claro» cambia el color, no sólo el atributo.')
  }

  // ── 2 · Se recuerda ──
  //
  // `reload()`, no `goto()` a la misma dirección: con enrutado por hash, ir a
  // `#/ajustes` estando ya en `#/ajustes` es navegación **dentro del mismo
  // documento** —el navegador no recarga nada—, así que esta comprobación
  // medía el estado que seguía en memoria. Comprobado: quitando la llamada que
  // guarda el tema, la sonda seguía diciendo que se recordaba.
  await pagina.reload({ waitUntil: 'networkidle' })
  await pagina.waitForTimeout(400)
  const trasRecargar = await estado()
  const elegidoTrasRecargar = await pagina.inputValue(selector)
  console.log(`\n  tras recargar: data-tema=${trasRecargar.atributo}, ` +
              `desplegable=${elegidoTrasRecargar}`)
  if (trasRecargar.atributo !== 'claro' || elegidoTrasRecargar !== 'claro') {
    fallo(
      'La elección no sobrevive a reabrir la aplicación.\n' +
        '  El tema se guarda en `ui_preference`, dentro de la base cifrada.\n' +
        '  Si no vuelve, o no se guardó o no se está leyendo al arrancar.',
    )
  } else {
    console.log('✓ La elección se recuerda al reabrir.')
  }

  // ── 3 · «Automático» sigue al sistema, y lo sigue en vivo ──
  await pagina.selectOption(selector, 'auto')
  await pagina.waitForTimeout(300)
  const autoEnOscuro = await estado()

  await pagina.emulateMedia({ colorScheme: 'light' })
  await pagina.waitForTimeout(300)
  const autoEnClaro = await estado()

  console.log(`\n  automático · sistema oscuro → ${autoEnOscuro.atributo}`)
  console.log(`  automático · sistema claro  → ${autoEnClaro.atributo}`)

  if (autoEnOscuro.atributo !== 'oscuro') {
    fallo(
      `Con el sistema en oscuro, «Automático» aplicó «${autoEnOscuro.atributo}».`,
    )
  } else if (autoEnClaro.atributo !== 'claro') {
    fallo(
      'El sistema cambió a claro con ARLES abierto y «Automático» no lo siguió.\n' +
        '  No es un caso raro: macOS y Windows conmutan solos al anochecer, y\n' +
        '  entonces ARLES se queda siendo la única ventana oscura del escritorio.',
    )
  } else {
    console.log('✓ «Automático» sigue al sistema, también en caliente.')
  }

  // Y elegir a mano vuelve a mandar sobre el sistema.
  await pagina.selectOption(selector, 'oscuro')
  await pagina.waitForTimeout(300)
  const manualGana = await estado()
  if (manualGana.atributo !== 'oscuro') {
    fallo(
      'Con el sistema en claro, elegir «Oscuro» a mano no se respeta.\n' +
        '  Elegir tiene que ganar al sistema: si no, la elección dura hasta\n' +
        '  el siguiente amanecer y nadie entiende por qué se deshizo.',
    )
  } else {
    console.log('✓ Elegir a mano gana al sistema.')
  }
} finally {
  await navegador?.close()
  servidor?.close()
  await rm(SALIDA, { recursive: true, force: true })
}

process.exit(codigo)
