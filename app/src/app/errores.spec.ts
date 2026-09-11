import { describe, expect, it, vi } from 'vitest'

import { resolverError } from '@/app/errores'

/**
 * Las claves que el núcleo puede emitir.
 *
 * Duplicadas a propósito desde Rust: la lista viva está en los `clave_i18n()`
 * de `arles_core::CoreError`, `arles_db::DbError` y `arles_app::AppError`. Si
 * alguien añade una variante allí sin texto aquí, este test falla y el error
 * se descubre ahora y no delante de un usuario.
 */
const CLAVES_DEL_NUCLEO = [
  // arles-core
  'error.email_invalido',
  'error.id_invalido',
  'error.transicion_invalida',
  // arles-db
  'error.db.clave_incorrecta',
  'error.db.clave_mal_formada',
  'error.db.migracion',
  'error.db.sqlite',
  // arles-app
  'error.app.llavero_no_disponible',
  'error.app.clave_maestra_perdida',
  'error.app.directorio_de_datos',
] as const

describe('resolución de errores del núcleo', () => {
  it('toda clave que emite Rust tiene su propio texto de tres partes', () => {
    // Estricto a propósito: comprobar solo que hay tres partes pasaría también
    // con el texto genérico, y entonces el test no detectaría nada. Lo que se
    // exige es que NINGUNA clave caiga en el genérico.
    const espia = vi.spyOn(console, 'error').mockImplementation(() => {})

    for (const clave of CLAVES_DEL_NUCLEO) {
      const e = resolverError({ clave, detalle: 'prueba' })
      expect(e.que.length, `${clave}.que`).toBeGreaterThan(0)
      expect(e.como.length, `${clave}.como`).toBeGreaterThan(0)
      expect(e.salvo.length, `${clave}.salvo`).toBeGreaterThan(0)
    }

    const sinTexto = espia.mock.calls.map((c) => String(c[0]))
    espia.mockRestore()
    expect(sinTexto, 'estas claves del núcleo no tienen texto').toEqual([])
  })

  it('ninguna clave devuelve un objeto sin resolver', () => {
    // El bug original: `$t('error.db.sqlite')` devolvía el objeto y la interfaz
    // habría mostrado «[object Object]».
    for (const clave of CLAVES_DEL_NUCLEO) {
      const e = resolverError({ clave, detalle: 'prueba' })
      expect(String(e.que)).not.toContain('[object')
    }
  })

  it('una clave desconocida cae en el genérico y lo reporta', () => {
    const espia = vi.spyOn(console, 'error').mockImplementation(() => {})
    const e = resolverError({ clave: 'error.inventado', detalle: 'x' })

    expect(e.que.length).toBeGreaterThan(0)
    expect(e.salvo.length).toBeGreaterThan(0)
    expect(espia).toHaveBeenCalled()
    espia.mockRestore()
  })
})
