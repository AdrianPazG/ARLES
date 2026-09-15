import { i18n } from '@/app/i18n'

/** Forma en que un error cruza la frontera IPC. Coincide con `arles_app::ErrorIpc`. */
export interface ErrorIpc {
  /** Clave de i18n, p. ej. `error.db.clave_incorrecta`. */
  readonly clave: string
  /** Texto técnico para diagnóstico. No se muestra tal cual. */
  readonly detalle: string
}

/** Un error listo para mostrarse, con las tres partes que exige el §95. */
export interface ErrorMostrable {
  readonly que: string
  readonly como: string
  readonly salvo: string
}

const GENERICO: ErrorMostrable = {
  que: 'Ocurrió un error inesperado.',
  como: 'Cierra ARLES y vuelve a abrirlo.',
  salvo: 'Las operaciones incompletas se revirtieron por completo.',
}

function esMostrable(v: unknown): v is ErrorMostrable {
  return (
    typeof v === 'object' &&
    v !== null &&
    typeof (v as ErrorMostrable).que === 'string' &&
    typeof (v as ErrorMostrable).como === 'string' &&
    typeof (v as ErrorMostrable).salvo === 'string'
  )
}

/**
 * Resuelve la clave que emite el núcleo a su texto de tres partes.
 *
 * Existe porque las claves de Rust son planas (`error.email_invalido`) y en el
 * archivo de textos cada error es un objeto de tres campos: llamar a
 * `$t('error.email_invalido')` devolvía **el objeto**, no una cadena, y la
 * interfaz habría mostrado `[object Object]`. Nada lo detectaba porque los dos
 * lados nunca se probaron juntos.
 */
export function resolverError(e: ErrorIpc): ErrorMostrable {
  // Se recorre el mensaje del locale activo en vez de usar `t`/`tm`: esas
  // devuelven cadenas o envoltorios reactivos, y aquí lo que se necesita es el
  // objeto de tres campos tal cual.
  const raiz = i18n.global.getLocaleMessage(i18n.global.locale.value)

  let nodo: unknown = raiz
  for (const parte of e.clave.split('.')) {
    if (typeof nodo !== 'object' || nodo === null) {
      nodo = undefined
      break
    }
    nodo = (nodo as Record<string, unknown>)[parte]
  }

  if (esMostrable(nodo)) {
    // Se devuelve el texto **compilado**, no el del objeto. La diferencia
    // importa: vue-i18n tiene gramática propia, y un texto con una arroba
    // literal se escribe `{'@'}` porque `@` suelta abre un enlace a otra
    // clave. Devolviendo el crudo, el usuario leería `nombre{'@'}dominio.com`.
    //
    // Compilar por partes —`clave.que`— y no la clave entera es lo que evita
    // el fallo original: `t('error.db.sqlite')` apunta a un objeto y devolvía
    // «[object Object]».
    return {
      que: i18n.global.t(`${e.clave}.que`),
      como: i18n.global.t(`${e.clave}.como`),
      salvo: i18n.global.t(`${e.clave}.salvo`),
    }
  }

  // Una clave que el núcleo emite y los textos no tienen es un bug, no algo
  // que deba pasar en silencio.
  console.error(`Falta el texto del error «${e.clave}»`, e.detalle)
  return GENERICO
}
