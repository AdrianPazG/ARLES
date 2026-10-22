import { describe, expect, it, vi } from 'vitest'

import { i18n } from '@/app/i18n'
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
  // Entrega W.1 · WhatsApp. Los tres se niegan a aproximar hacia lo seguro:
  // un modo desconocido no se lee como «seguimiento» y un estado desconocido
  // no se lee como «aprobada».
  'error.modo_de_canal_desconocido',
  'error.estado_de_plantilla_desconocido',
  'error.plantilla_no_aprobada',
  'error.origen_desconocido',
  'error.motivo_de_supresion_desconocido',
  'error.origen_de_supresion_desconocido',
  'error.alcance_de_supresion_desconocido',
  'error.supresion_sin_canales',
  'error.alcance_no_coincide',
  'error.no_es_un_respaldo',
  'error.respaldo_de_version_futura',
  'error.respaldo_corrupto',
  'error.etapa.sin_etapas',
  'error.etapa.demasiadas',
  'error.etapa.posiciones',
  'error.etapa.canal_repetido',
  'error.etapa.primera_no_espera',
  'error.etapa.espera_desmesurada',
  'error.etapa.condicion_desconocida',
  'error.id_invalido',
  'error.transicion_invalida',
  // Ronda del 17/10/2026 · R-04. Una firma no puede llevar variables de quien
  // recibe: el remitente firmaría con el nombre y la empresa del destinatario.
  'error.firma_con_variables_del_contacto',
  // arles-db
  'error.db.clave_incorrecta',
  'error.db.clave_mal_formada',
  'error.db.migracion',
  'error.db.sqlite',
  // Entrega 8.2b · el esquema tiene algo que un respaldo no sabe llevar. Es un
  // defecto nuestro, no del usuario, y por eso el texto no le pide arreglarlo.
  'error.db.esquema_no_se_deja_respaldar',
  'error.db.respaldo_no_encaja',
  // Entrega 8.3 bis · archivar el historial (D-8D). Son dos y no uno porque
  // mandan a hacer cosas distintas: uno a elegir otro archivo, el otro a
  // volver a archivar.
  'error.db.eso_es_un_archivo_de_historial',
  'error.db.el_historial_cambio',
  'error.app.respaldo_sin_revisar',
  // Entrega 8.3 bis b · no hay historial de ese periodo, así que no se
  // escribe un archivo vacío.
  'error.app.nada_que_archivar',
  // arles-app
  'error.app.llavero_no_disponible',
  'error.app.clave_maestra_perdida',
  'error.app.directorio_de_datos',
  // El freno (7.0): detener pide el nombre escrito de la campaña.
  'error.app.nombre_no_coincide',
  'error.app.sitio_no_abre',
  'error.app.tema_desconocido',
  'error.app.empresa_invalida',
  'error.app.contacto_invalido',
  'error.app.empresa_no_configurada',
  'error.app.sin_importacion_en_curso',
  'error.app.archivo_no_se_pudo_leer',
  // Entrega 8.2c · la webview pidió un alcance de respaldo que no existe. Es
  // un defecto nuestro: la pantalla sólo ofrece los dos de D-8B.
  'error.app.alcance_de_respaldo_desconocido',
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
  'error.db.supresion_no_existe',
  'error.db.supresion_no_se_levanta',
  // Entrega 4.4 · pausar, reanudar y detener.
  'error.db.campana_no_existe',
  'error.db.audiencia_vacia',
  // Las dos de la 8.6 E: el desenlace que el núcleo calculaba y nadie
  // preguntaba. El validador las exigió aquí en cuanto entraron en `DbError`,
  // que es por lo que esta lista a mano se puede tener.
  'error.db.la_campana_no_termina',
  'error.db.la_campana_no_avanza',
  // Entrega 6.6 · el aviso de volumen del §48. No es un límite: es la
  // constancia de que se informó antes de crear la campaña.
  'error.db.falta_aceptar_el_volumen',
  'error.db.limite_de_envios_en_cero',
  // Y los cuatro motivos de una ventana que no se abriría nunca. `DbError`
  // devuelve la clave del MOTIVO, no una suya, para que el texto diga qué
  // campo arreglar (§95).
  'ventana.sinDias',
  'ventana.horasAlReves',
  'ventana.horaFueraDeRango',
  'ventana.zonaDesconocida',
  'error.estado_de_campana_desconocido',
  // Entrega 5.3 · una cuenta guardada con un cifrado que no reconocemos.
  'error.app.cuenta_de_correo_invalida',
  // La fábrica del proveedor (entrega 5.4a): una cuenta guardada que en este
  // momento no puede enviar. Seis motivos, seis acciones distintas.
  'error.app.cuenta.no_existe',
  'error.app.cuenta.hay_que_configurarla',
  'error.app.cuenta.detenida_por_fallos',
  'error.app.cuenta.desactivada',
  'error.app.cuenta.sin_contrasena',
  'error.app.cuenta.estado_desconocido',
  'error.cifrado_desconocido',
  'error.db.cuenta_de_correo_no_existe',
  // Entrega 5.2 · la cuenta de correo. Son problemas de configuración, no de
  // un envío: se leen mientras alguien da de alta un remitente.
  'error.correo.no_se_pudo_cifrar',
  'error.correo.el_servidor_no_vale',
  'error.correo.remitente_no_vale',
  'error.correo.no_se_pudo_armar',
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

/**
 * Las claves del envío de prueba, atadas a las que emite Rust.
 *
 * ── POR QUÉ ESTO EXISTE, Y NO ES UNA COPIA DE LO DE ARRIBA ──
 *
 * Éstas no son errores de `AppError`: son el campo `problema` del resultado del
 * envío de prueba, y por eso la lista de arriba no las cubre.
 *
 * Se escriben aquí porque **una clave que no coincide con la que pide el código
 * no da ningún error**. La interfaz enseña la clave en crudo —
 * `correo.prueba.envio.pasajero`— y eso sólo se descubre abriendo esa pantalla,
 * en ese caso concreto, que es el que casi nunca se prueba a mano.
 *
 * Ya pasó en esta fase: los cinco textos se escribieron primero bajo
 * `remitentes.envioDePrueba.*` mientras el comando pedía
 * `correo.prueba.envio.*`. Lo cazó esta prueba al escribirla.
 *
 * La lista viva está en `por_que_no_salio`, en `arles_app::comandos`.
 */
const CLAVES_DEL_ENVIO_DE_PRUEBA = [
  'correo.prueba.envio.pasajero',
  'correo.prueba.envio.elServidorPideEsperar',
  'correo.prueba.envio.esaDireccionNoExiste',
  'correo.prueba.envio.laCuentaNoAutentica',
  'correo.prueba.envio.pidioLaBaja',
]

describe('el resultado de un envío de prueba', () => {
  it('toda clave que emite Rust tiene su texto de tres partes', () => {
    const faltan = CLAVES_DEL_ENVIO_DE_PRUEBA.filter((clave) => {
      const t = i18n.global.tm(clave) as Record<string, unknown>
      return !t || typeof t !== 'object' || !t.que || !t.como || !t.salvo
    })
    expect(faltan).toEqual([])
  })

  /**
   * **Y el de la dirección que no existe dice que no se dio de baja a nadie.**
   *
   * Es la frase que no puede faltar. En una campaña, ese mismo rechazo del
   * servidor **sí** suprime; quien acaba de mandarse una prueba a sí mismo y lee
   * «esa dirección no existe» tiene que poder ver, sin preguntar, que no se ha
   * dado de baja.
   */
  it('el rechazo del destinatario aclara que no se dio de baja a nadie', () => {
    const t = i18n.global.tm('correo.prueba.envio.esaDireccionNoExiste') as Record<
      string,
      string
    >
    expect(String(t.salvo)).toContain('No se dio de baja a nadie')
  })
})

/**
 * Las claves del preflight, atadas a las que emite Rust.
 *
 * ── POR QUÉ ESTO EXISTE ──
 *
 * Lo mismo que con el envío de prueba: **una clave que no coincide no da
 * ningún error**. La pantalla enseñaría `preflight.laCuentaAutentica` en
 * crudo, y eso sólo se descubre abriendo el preflight de una campaña — que es
 * la pantalla a la que se llega menos veces y en el peor momento.
 *
 * Desde la 6.5 estas claves ya no son decorativas: el comando `revisar_campana`
 * devuelve los ocho puntos con su veredicto, y la 6.6 los pinta.
 *
 * Las listas vivas están en `Punto::clave_i18n`, `Veredicto::clave_i18n` y
 * `Resultado::clave_i18n`, en `arles_core::preflight`.
 */
const CLAVES_DEL_PREFLIGHT = [
  'preflight.laCuentaAutentica',
  'preflight.laCredencialSirve',
  'preflight.laAudienciaNoEstaVacia',
  'preflight.lasVariablesSeResuelven',
  'preflight.elHorarioAlcanza',
  'preflight.hayConexion',
  'preflight.seHizoElEnvioDePrueba',
  'preflight.losRegistrosDelDominio',
  'preflight.pasa',
  'preflight.falla',
  'preflight.sinComprobar',
  'preflight.lista',
  'preflight.bloqueada',
]

describe('los ocho puntos del preflight', () => {
  it('toda clave que emite Rust tiene texto, y no vuelve como la clave', () => {
    for (const clave of CLAVES_DEL_PREFLIGHT) {
      const texto = String(i18n.global.t(clave))
      expect(texto, `${clave} no tiene texto`).not.toBe(clave)
      expect(texto.length).toBeGreaterThan(0)
    }
  })

  /**
   * **«Sin comprobar» no puede leerse como un aprobado.**
   *
   * Es la regla que gobierna el archivo entero en Rust: un punto sin comprobar
   * bloquea igual que uno que falla. Si el texto dijera «pendiente» o «se
   * comprobará luego», la pantalla contaría otra cosa que el botón apagado, y
   * quien lo lea buscará el fallo donde no está.
   */
  it('el texto de «sin comprobar» dice que no se da por bueno', () => {
    const detalle = String(i18n.global.t('preflight.sinComprobarDetalle'))
    expect(detalle).toContain('no lo da por bueno')
    expect(detalle).toContain('impide activar')
  })
})
