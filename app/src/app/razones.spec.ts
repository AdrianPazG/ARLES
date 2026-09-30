import { readFileSync } from 'node:fs'
import { join } from 'node:path'

import { describe, expect, it } from 'vitest'

import { es } from '@/app/locales/es'
import {
  cifrasDeRazon,
  MARCADORES_DE_RAZON,
  RAZONES_EN_SEGUNDOS,
} from '@/app/razones'

/** El texto de una clave `grupo.nombre` del catálogo, ya resuelto. */
function texto(clave: string): string {
  const [grupo, nombre] = clave.split('.')
  const bloque = (es as Record<string, unknown>)[grupo!]
  expect(
    bloque,
    `el catálogo no tiene el grupo «${grupo}» de «${clave}»`,
  ).toBeTypeOf('object')
  const t = (bloque as Record<string, unknown>)[nombre!]
  expect(t, `el grupo «${grupo}» no tiene «${nombre}»`).toBeTypeOf('string')
  return t as string
}

/** Los nombres de los marcadores `{x}` de un texto, en orden de aparición. */
function marcadoresDe(t: string): string[] {
  return [...t.matchAll(/\{([a-zA-Z]+)\}/g)].map((m) => m[1]!)
}

describe('las razones por las que una campaña está quieta', () => {
  /**
   * **Cada razón declara exactamente los marcadores que su texto tiene.**
   *
   * ── EL HUECO QUE ESTO CAZA ──
   *
   * Es el del §9, entre dos capas que cada una pasa lo suyo. Rust comprueba que
   * el suceso traiga tantas cifras como marcadores hay; esto comprueba que se
   * llamen igual. Sin las dos, `numeroA` puede llegar bien y perderse porque
   * esta tabla lo bautizó `cabía` en vez de `cabian` — y el texto sale entero,
   * con el marcador literal, sin que falle nada.
   *
   * Verificada rompiéndola el 30/09/2026: cambiando `cabian` por `cuantos` en
   * `MARCADORES_DE_RAZON`, falla con «ritmo.topeDeLaHora: el texto pide cabian
   * y la tabla declara cuantos».
   */
  it('cada razón declara los marcadores de su texto', () => {
    for (const [clave, declarados] of Object.entries(MARCADORES_DE_RAZON)) {
      const pide = marcadoresDe(texto(clave))
      expect(
        declarados,
        `${clave}: el texto pide ${pide.join(', ') || '(ninguno)'} y la tabla declara ${
          declarados.join(', ') || '(ninguno)'
        }`,
      ).toEqual(pide)
    }
  })

  /**
   * **Y toda razón que el motor puede anotar está en la tabla.**
   *
   * Sin esto, una clave nueva del motor —o una que alguien renombre— se pinta
   * sin cifras y el texto sale con `{cabian}` en pantalla. Las claves se leen de
   * los fuentes de Rust, no de una lista a mano: una lista a mano se queda vieja
   * el día que hace falta que esto salte.
   *
   * Sólo se exigen las de bucle. Los desenlaces de un envío no pueden ser una
   * razón: `por_que_esta_quieta_cada_campana` los excluye en SQL.
   */
  it('la tabla cubre toda clave de bucle que el motor puede anotar', () => {
    const raiz = join(__dirname, '../../..')
    const bucle = readFileSync(
      join(raiz, 'crates/arles-motor/src/bucle.rs'),
      'utf8',
    )
    const ritmo = readFileSync(
      join(raiz, 'crates/arles-core/src/ritmo.rs'),
      'utf8',
    )

    // ── QUÉ VARIANTES SON DE BUCLE, LEÍDO DEL ENUM Y NO A MANO ────────────
    //
    // La primera versión filtraba por prefijo de grupo —`motor.*` y `ritmo.*`—
    // y exigía `motor.cancelado` y `motor.aceptadoPorElProveedor`, que son
    // DESENLACES de un envío y nunca pueden ser una razón: la consulta los
    // excluye con `is_outcome = 0`. Un filtro por grupo no sabe distinguirlos.
    //
    // El criterio de verdad es el mismo que usa `anotar_el_paso`: un paso es
    // desenlace si lleva un `intento`. Así que se lee del propio enum `Paso`
    // qué variantes lo llevan, y del `match` de `clave_i18n` qué clave da cada
    // variante. Si alguien añade un paso de bucle nuevo, sale de aquí solo.
    const cuerpoDelEnum = bucle.slice(bucle.indexOf('pub enum Paso {'))
    const conIntento = new Set(
      [...cuerpoDelEnum.matchAll(/^ {4}([A-Z][A-Za-z]*) \{([^}]*)\}/gm)]
        .filter((m) => m[2]!.includes('intento'))
        .map((m) => m[1]!),
    )
    expect(
      conIntento.size,
      'se leyeron las variantes con intento del enum Paso',
    ).toBeGreaterThan(3)

    // Variante → clave, del `match` de `clave_i18n`.
    const deVariante = [
      ...bucle.matchAll(/Self::([A-Za-z]+)[^=\n]*=> "([a-z]+\.[a-zA-Z]+)"/g),
    ]
    expect(deVariante.length, 'se leyó el match de clave_i18n').toBeGreaterThan(
      5,
    )

    const razones = new Set(
      deVariante
        .filter(([, variante]) => !conIntento.has(variante!))
        .map(([, , clave]) => clave!),
    )
    // `SinPresupuesto` delega en `NoSePuede::clave_i18n`, que vive en el núcleo:
    // sus tres claves no salen del `match` de arriba y hay que leerlas allí.
    //
    // **Sólo dentro de `impl NoSePuede`.** Leyendo el archivo entero entraban
    // `ritmo.sinNovedad` y `ritmo.huboSuspension`, que son del enum del reloj
    // —describen si el equipo estuvo suspendido— y no son un paso del motor: no
    // llegan nunca a `activity_event`. Otra comprobación que miraba el sitio
    // equivocado, cazada al estrenarla (§2).
    const implNoSePuede = ritmo.slice(ritmo.indexOf('impl NoSePuede {'))
    const finDelImpl = implNoSePuede.indexOf('\n}')
    expect(finDelImpl, 'se encontró el impl de NoSePuede').toBeGreaterThan(0)
    for (const m of implNoSePuede
      .slice(0, finDelImpl)
      .matchAll(/Self::[A-Za-z]+[^=\n]*=> "(ritmo\.[a-zA-Z]+)"/g)) {
      razones.add(m[1]!)
    }

    for (const clave of razones) {
      expect(
        Object.prototype.hasOwnProperty.call(MARCADORES_DE_RAZON, clave),
        `${clave}: el motor la puede anotar como razón y MARCADORES_DE_RAZON no la conoce`,
      ).toBe(true)
    }
    // Y al revés: la tabla no declara claves que el motor no anote nunca, que
    // serían filas muertas que nadie descubre.
    for (const clave of Object.keys(MARCADORES_DE_RAZON)) {
      expect(
        razones.has(clave),
        `${clave} está en la tabla y el motor no la anota nunca`,
      ).toBe(true)
    }
  })

  /**
   * **Los segundos se enseñan en minutos, redondeando hacia arriba.**
   *
   * Con 30 segundos restantes, «vuelve en 0 minutos» es falso y se lee como «ya
   * debería estar enviando» — la confusión que esta pantalla existe para
   * evitar. Y sin la conversión, «vuelve en 1 380 minutos» diría que la cuenta
   * está parada casi un día cuando son veintitrés minutos.
   */
  it('la pausa de la cuenta se cuenta en minutos y nunca en cero', () => {
    const cifras = cifrasDeRazon({
      clave: 'motor.laCuentaEstaEnPausa',
      numeroA: 1_380,
      numeroB: null,
      cuando: '2026-09-30T10:04:00Z',
    })
    expect(cifras).toEqual({ minutos: 23 })

    const casi = cifrasDeRazon({
      clave: 'motor.laCuentaEstaEnPausa',
      numeroA: 30,
      numeroB: null,
      cuando: '2026-09-30T10:04:00Z',
    })
    expect(casi['minutos'], '30 s no son «0 minutos»: todavía no vuelve').toBe(
      1,
    )
  })

  /** Una razón sin cifras no inventa ninguna. */
  it('una razón sin cifras no trae ninguna', () => {
    expect(
      cifrasDeRazon({
        clave: 'motor.fueraDeVentana',
        numeroA: null,
        numeroB: null,
        cuando: '2026-09-30T10:04:00Z',
      }),
    ).toEqual({})
  })

  /**
   * **Y una cifra que llega nula no se pinta como cero.**
   *
   * Un `0` en «el tope es de 0 envíos por hora» es una afirmación falsa sobre la
   * configuración de alguien. Si el dato no vino, el marcador se queda sin
   * rellenar y el texto lo enseña — que es visible, y por lo tanto reparable.
   */
  it('una cifra ausente no se convierte en cero', () => {
    expect(
      cifrasDeRazon({
        clave: 'ritmo.topeDeLaHora',
        numeroA: null,
        numeroB: null,
        cuando: '2026-09-30T10:04:00Z',
      }),
    ).toEqual({})
  })

  /** La lista de conversiones no nombra claves que no existan. */
  it('las razones en segundos son razones conocidas', () => {
    for (const clave of RAZONES_EN_SEGUNDOS) {
      expect(
        Object.prototype.hasOwnProperty.call(MARCADORES_DE_RAZON, clave),
        `${clave} está en RAZONES_EN_SEGUNDOS y no en MARCADORES_DE_RAZON`,
      ).toBe(true)
    }
  })
})
