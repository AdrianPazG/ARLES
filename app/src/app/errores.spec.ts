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
  'error.telefono_invalido',
  'error.canal_desconocido',
  'error.origen_desconocido',
  'error.etapa.sin_etapas',
  'error.etapa.demasiadas',
  'error.etapa.posiciones',
  'error.etapa.canal_repetido',
  'error.etapa.primera_no_espera',
  'error.etapa.espera_desmesurada',
  'error.etapa.condicion_desconocida',
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
  'error.app.sitio_no_abre',
  'error.app.tema_desconocido',
  'error.app.empresa_invalida',
  'error.app.contacto_invalido',
  'error.app.empresa_no_configurada',
  'error.app.sin_importacion_en_curso',
  'error.app.archivo_no_se_pudo_leer',
  // arles-import
  'error.import.formato_desconocido',
  'error.import.demasiado_grande',
  'error.import.demasiados_datos',
  'error.import.sin_encabezados',
  'error.import.sin_hojas',
  'error.import.no_se_pudo_leer',
  'error.db.dato_invalido',
  'error.db.direccion_en_uso',
  'error.db.contacto_no_existe',
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

  /**
   * Los textos se muestran **compilados**, no crudos.
   *
   * `vue-i18n` tiene gramática propia: una arroba literal se escribe `{'@'}`
   * porque `@` suelta abre un enlace a otra clave. Devolviendo el texto tal
   * como está en el archivo, el usuario leería `nombre{'@'}dominio.com`.
   */
  it('ningún texto llega con la sintaxis de vue-i18n sin resolver', () => {
    for (const clave of CLAVES_DEL_NUCLEO) {
      const e = resolverError({ clave, detalle: 'prueba' })
      const todo = `${e.que} ${e.como} ${e.salvo}`
      expect(todo, `${clave} muestra sintaxis sin compilar`).not.toMatch(/\{'/)
    }
    expect(resolverError({ clave: 'error.email_invalido', detalle: '' }).como)
      .toContain('nombre@dominio.com')
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
