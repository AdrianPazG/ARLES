import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join } from 'node:path'

import { describe, expect, it } from 'vitest'

import { es } from './es'

/**
 * **Ninguna pantalla pide un texto que el catálogo no tenga.**
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ ESTO NO LO CAZABA NADA
 *
 * `vue-i18n` no falla cuando falta una clave: **devuelve la clave**. Así que
 * una pantalla con un `t('remitentes.cupo.titulo')` mal escrito no se rompe —
 * enseña `remitentes.cupo.titulo` en medio de la ficha, y eso sólo se ve
 * mirando. Ya pasó: la Fase 6 tenía una clave
 * —`plantilla.variable.miEmpresa`— que apuntaba a un texto inexistente.
 *
 * El §139 vigila el otro lado: que cada error de Rust tenga su texto. Lo que
 * faltaba era **del catálogo hacia la pantalla**, que es el lado que el usuario
 * ve.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * Y POR QUÉ ES UNA PRUEBA Y NO UNA COMPROBACIÓN DEL VALIDADOR
 *
 * Porque **el catálogo se importa, no se parsea**. La primera versión de esto
 * fue un script que reconstruía las rutas de `es.ts` por sangría, y dio
 * **veinte hallazgos falsos**: las veinte claves existían y lo que estaba mal
 * era mi lectura del anidamiento. Es la cuarta vez en un día que una medición
 * propia da falsos, y la lección ya está escrita en tres auditorías —
 * **comprobar antes de anotar**.
 *
 * Aquí no hay nada que interpretar: el objeto es el que la aplicación carga.
 */

const RAIZ = join(import.meta.dirname, '..', '..')

/** Todos los `.ts` y `.vue` de la aplicación, menos los catálogos. */
function archivos(dir: string): string[] {
  const salida: string[] = []
  for (const nombre of readdirSync(dir)) {
    const ruta = join(dir, nombre)
    if (statSync(ruta).isDirectory()) {
      salida.push(...archivos(ruta))
    } else if (/\.(ts|vue|rs)$/.test(nombre) && !/locales[/\\]/.test(ruta)) {
      salida.push(ruta)
    }
  }
  return salida
}

/**
 * Quita los comentarios antes de buscar claves.
 *
 * ── ESTA FUNCIÓN EXISTE PORQUE LA PRUEBA SE CAZÓ A SÍ MISMA ──────────────
 *
 * La primera versión dio dos hallazgos, `error.email_invalido` y
 * `error.db.sqlite`, y los dos estaban **en los comentarios de `errores.ts`**
 * que explican por qué esas claves apuntan a un objeto de tres partes y no a
 * una cadena. Código correcto, comentarios correctos, prueba en rojo.
 *
 * Es la **décima vez** que pasa en este proyecto (5.3, 8.5, dos en la 8.3 bis,
 * 8.6 A, 8.6 D, 8.6 E, 8.6 F y la auditoría H), y la conclusión ya está
 * escrita en el CLAUDE.md: **se quitan los comentarios siempre, por defecto, en
 * cualquier comprobación que busque una cadena en un archivo.** Saberlo no
 * basta; hay que hacerlo.
 */
function sinComentarios(texto: string): string {
  return texto
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .split('\n')
    .map((l) => (l.trimStart().startsWith('//') ? '' : l))
    .join('\n')
}

/**
 * ¿Existe esta ruta de puntos dentro del catálogo, con un texto usable al final?
 *
 * ── «USABLE» SON DOS FORMAS, Y EXIGIR UNA DABA 95 FALSOS ────────────────
 *
 * La primera versión exigía una cadena, y salió en rojo con 95 claves que
 * existen: **los errores son objetos de tres partes** —`que`, `como`, `salvo`—
 * porque el §95 pide que todo error diga qué pasó, cómo se arregla y qué está
 * a salvo. `errores.ts` los compila parte por parte, y por eso un
 * `t('error.db.sqlite')` a secas devolvería el objeto.
 *
 * Así que lo que hay que comprobar no es «es una cadena» sino «hay un texto que
 * alguien puede enseñar»: una cadena, o un error con sus tres partes.
 *
 * Fue la quinta medición propia con falsos en un día, y es la misma lección de
 * las auditorías B, C y D: **comprobar antes de anotar.**
 */
function existe(clave: string): boolean {
  let nodo: unknown = es
  for (const parte of clave.split('.')) {
    if (typeof nodo !== 'object' || nodo === null) return false
    nodo = (nodo as Record<string, unknown>)[parte]
    if (nodo === undefined) return false
  }
  if (typeof nodo === 'string') return true
  // Un error de tres partes también es un texto que la pantalla sabe enseñar.
  if (typeof nodo === 'object' && nodo !== null) {
    const o = nodo as Record<string, unknown>
    return (
      typeof o['que'] === 'string' &&
      typeof o['como'] === 'string' &&
      typeof o['salvo'] === 'string'
    )
  }
  return false
}

describe('el catálogo de textos', () => {
  /**
   * Sólo las claves escritas como literal. Las que se componen —`t(clave)` con
   * una variable, o `` t(`${x}Detalle`) `` — no se pueden resolver sin ejecutar
   * la pantalla, y fingir que sí las cubre sería peor que no mirarlas: dejaría
   * esta prueba en verde sobre un hueco.
   *
   * Lo que sí se exige de ellas es que su parte fija exista, y eso lo
   * comprueban las pruebas de cada pantalla, que montan el componente con el
   * catálogo de verdad y miran el texto.
   */
  it('tiene todos los textos que las pantallas piden con una clave literal', () => {
    const faltan: string[] = []
    let pedidas = 0

    for (const ruta of archivos(RAIZ)) {
      if (ruta.endsWith('.spec.ts')) continue
      const texto = sinComentarios(readFileSync(ruta, 'utf-8'))
      for (const m of texto.matchAll(/\bte?\(\s*'([a-zA-Z][\w.]*)'/g)) {
        const clave = m[1]
        if (clave === undefined) continue
        pedidas += 1
        if (!existe(clave)) {
          faltan.push(`${clave} · ${ruta.replace(RAIZ, '')}`)
        }
      }
    }

    // Que se hayan encontrado claves es parte de lo que hay que comprobar: con
    // una expresión que no case nada, esto saldría en verde sin mirar una sola.
    // Umbral medido el 22/10/2026: 693.
    expect(pedidas).toBeGreaterThan(500)
    expect(faltan).toEqual([])
  })

  /**
   * **Y ningún texto lleva una `@` suelta ni asteriscos de Markdown.**
   *
   * Las dos tumban una pantalla o salen literales, y las dos son cadenas
   * válidas de TypeScript, así que ni `vue-tsc` ni ESLint las ven. Ya las caza
   * el validador en la fase 7; aquí se comprueban sobre el **objeto cargado**,
   * que es lo que de verdad llega a `vue-i18n` — el validador mira el archivo,
   * y un texto compuesto con `+` se le parte en dos trozos.
   */
  it('ningún texto lleva una arroba suelta ni asteriscos', () => {
    const malos: string[] = []
    const recorrer = (nodo: unknown, ruta: string): void => {
      if (typeof nodo === 'string') {
        // `@` escapada como {'@'} es válida; suelta rompe el componente entero.
        if (/(^|[^{'])@/.test(nodo) && !nodo.includes("{'@'}")) {
          malos.push(`${ruta}: arroba suelta`)
        }
        if (/\*\*/.test(nodo)) malos.push(`${ruta}: asteriscos de Markdown`)
        return
      }
      if (typeof nodo === 'object' && nodo !== null) {
        for (const [k, v] of Object.entries(nodo)) recorrer(v, ruta ? `${ruta}.${k}` : k)
      }
    }
    recorrer(es, '')
    expect(malos).toEqual([])
  })
  /**
   * **Toda clave que Rust emite resuelve en el catálogo, por su RUTA entera.**
   *
   * ── EL HUECO QUE ESTO CIERRA, MEDIDO EL 22/10/2026 ──────────────────────
   *
   * El §139 lo vigila en el validador, y ahí hay dos límites escritos:
   *
   * 1. sólo mira las claves que empiezan por `error.`;
   * 2. y compara **la hoja, no la ruta** — su propio comentario dice que
   *    escribir aquí un segundo intérprete de TypeScript sería peor que el
   *    problema, y que «la que decide es la del frontend».
   *
   * Esta es la del frontend. Y el hueco era real: el semáforo de Remitentes
   * (D-14) emite tres claves desde `RiesgoDelBuzon::clave_i18n()` que **no
   * empiezan por `error.`**, así que nadie las vigilaba. Medido renombrando
   * `remitentes.riesgo.tranquilo` en el catálogo: **las diez fases del
   * validador y las 721 pruebas de Rust pasaron en verde**, y en la ficha del
   * remitente el usuario habría leído la clave cruda.
   *
   * Las claves se leen **del código Rust**, no de una lista: una lista a mano
   * es el defecto que este validador persigue en todas sus formas, y la que
   * `errores.spec.ts` lleva ya hizo falta arreglarla una vez por eso.
   */
  it('resuelve todas las claves que el núcleo emite, por su ruta entera', () => {
    const CRATES = ['arles-core', 'arles-db', 'arles-app', 'arles-email']
    const claves = new Set<string>()

    for (const crate of CRATES) {
      const dir = join(RAIZ, '..', '..', 'crates', crate, 'src')
      for (const ruta of archivos(dir).filter((r) => r.endsWith('.rs'))) {
        const texto = sinComentarios(readFileSync(ruta, 'utf-8'))
        // Las ramas de un `match` que devuelven una clave: `=> "a.b.c"`.
        for (const m of texto.matchAll(/=>\s*"([a-z][a-zA-Z]*(?:\.[a-zA-Z_]+)+)"/g)) {
          const c = m[1]
          if (c !== undefined) claves.add(c)
        }
      }
    }

    // Umbral medido el 22/10/2026: 139 claves. Sin él, una expresión que no
    // case nada dejaría esto en verde sin mirar una sola.
    expect(claves.size).toBeGreaterThan(100)

    const sinTexto = [...claves].filter((c) => !existe(c)).sort()
    expect(sinTexto).toEqual([])
  })

})
