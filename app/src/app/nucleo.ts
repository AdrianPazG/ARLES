/**
 * La única puerta al núcleo.
 *
 * Toda llamada a un comando de Tauri pasa por aquí. Tres motivos:
 *
 * 1. **`vite dev` y las sondas corren sin núcleo.** Ahí no hay comandos, y una
 *    llamada directa lanzaría una excepción que dejaría la pantalla a medias.
 *    `hayNucleo()` permite que cada pantalla decida su comportamiento sin
 *    repetir la comprobación del objeto interno de Tauri en seis sitios.
 * 2. **Los errores del núcleo tienen forma.** Llegan como `ErrorIpc`, con clave
 *    de i18n y, en los formularios, la lista de campos malos. Sin un punto
 *    único, cada pantalla los trataría como `unknown` y acabaría mostrando el
 *    texto técnico, que es justo lo que el §95 prohíbe.
 * 3. El `import()` dinámico mantiene la API de Tauri fuera del bundle de la
 *    parte que no la usa.
 */

/** Un campo de formulario rechazado por el núcleo. */
export interface ErrorDeCampo {
  readonly campo: string
  /** Clave de i18n del motivo. El texto vive en el catálogo, no aquí. */
  readonly clave: string
}

/** Forma en que un error del núcleo cruza la frontera. */
export interface ErrorIpc {
  readonly clave: string
  readonly detalle: string
  /** Presente sólo en errores de formulario. */
  readonly campos?: readonly ErrorDeCampo[]
  /**
   * Lo mismo para el formulario de la cuenta de correo.
   *
   * Lista aparte y no mezclada con `campos`: los nombres de campo de los dos
   * formularios no tienen nada que ver, y fundirlos obligaría a cada pantalla
   * a filtrar los que no son suyos — que es como se acaba marcando en rojo una
   * casilla que no existe.
   */
  readonly camposDeCuenta?: readonly ErrorDeCampo[]
}

interface ConTauri {
  __TAURI_INTERNALS__?: unknown
}

/** ¿Hay un núcleo al otro lado? */
/**
 * `true` si ARLES arrancó **sin poder abrir su base** (8.4).
 *
 * La marca la pone el núcleo antes de que cargue la interfaz. Se lee en vez de
 * preguntarse con un comando porque en el modo de recuperación sólo existen
 * tres, y preguntar por uno que no está deja un error en la consola del
 * arranque normal — y hay sondas que exigen la consola limpia.
 */
export function enRecuperacion(): boolean {
  return (window as unknown as Record<string, unknown>).__ARLES_RECUPERACION__ === true
}

export function hayNucleo(): boolean {
  return '__TAURI_INTERNALS__' in (window as unknown as ConTauri)
}

/**
 * `true` si el valor tiene la forma de un `ErrorIpc`.
 *
 * Se comprueba la forma en vez de confiar en el tipo: lo que llega de la
 * frontera es `unknown`, y un `as ErrorIpc` sería una afirmación que nadie ha
 * verificado.
 */
export function esErrorIpc(e: unknown): e is ErrorIpc {
  return (
    typeof e === 'object' &&
    e !== null &&
    'clave' in e &&
    typeof (e as { clave: unknown }).clave === 'string'
  )
}

/**
 * Invoca un comando del núcleo.
 *
 * **La vista previa no pasa por aquí de otra forma**: inyecta su propio
 * `__TAURI_INTERNALS__` con un `invoke` simulado, así que para este archivo
 * hay núcleo y no hay nada que distinguir. El simulador vive en
 * `herramientas/vista-previa/simulador.js`, y es el único sitio donde se
 * añaden los comandos que la demostración tiene que saber contestar.
 *
 * @throws el `ErrorIpc` tal cual lo devolvió el núcleo, o el error original si
 * no tiene esa forma. **No se traduce aquí**: quien llama sabe qué hacer con
 * cada clave, y una traducción temprana pierde los campos.
 */
export async function invocar<T>(
  comando: string,
  argumentos?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core')
  return invoke<T>(comando, argumentos)
}

/**
 * Escucha un evento del núcleo y devuelve cómo dejar de escucharlo.
 *
 * Hace falta para el avance del respaldo (8.2c): con medio millón de filas,
 * exportar tarda, y una barra parada veinte segundos es indistinguible de un
 * programa colgado. Un valor devuelto al final no sirve — llega cuando ya no
 * hace falta—, así que el núcleo emite cifras mientras escribe.
 *
 * **Quien llama tiene que llamar a lo que devuelve.** Un oyente que no se
 * quita sigue vivo al salir de la pantalla, y la segunda visita deja dos
 * sumando sobre el mismo contador.
 *
 * En la vista previa no llega ningún evento, y es correcto: ahí no hay núcleo
 * que los emita. La pantalla tiene que verse bien sin ellos, que es la misma
 * regla que `hayNucleo()` para todo lo demás.
 */
export async function escuchar<T>(
  evento: string,
  alLlegar: (dato: T) => void,
): Promise<() => void> {
  if (!hayNucleo()) return () => {}
  try {
    const { listen } = await import('@tauri-apps/api/event')
    const quitar = await listen<T>(evento, (e) => alLlegar(e.payload))
    return () => {
      void quitar()
    }
  } catch {
    // Que no se pueda escuchar el avance no es motivo para no exportar.
    return () => {}
  }
}

// ───────────────────────────────────────────────────────────────────────────
// EL DETALLE TÉCNICO DE UN ERROR QUE NADIE ESPERABA
//
// ── POR QUÉ ESTO EXISTE (H-05, 16/10/2026) ────────────────────────────────
//
// Dirección no pudo dar de alta un contacto y ARLES contestó «Ocurrió un error
// al acceder a los datos. Cierra ARLES y vuelve a abrirlo». No se pudo
// diagnosticar, y la razón es que **ese mensaje se traga la causa**: nueve
// stores escribían
//
//     errorGeneral.value = esErrorIpc(e) ? e.clave : 'error.db.sqlite'
//
// y ese `: 'error.db.sqlite'` convierte CUALQUIER cosa no tipada —un fallo del
// propio JavaScript, un problema al serializar, cualquier excepción de la
// webview— en «falló la base de datos». Dos defectos de una vez:
//
//   1. **Afirma una causa que no conoce.** Es el pecado que este proyecto
//      persigue en todas partes: un diagnóstico inventado manda a arreglar lo
//      que no está roto.
//   2. **Pierde la causa de verdad**, así que nadie puede arreglarla después.
//      Un producto que no sabe decir por qué falló obliga a adivinar, y
//      adivinar en este proyecto ya ha costado caro dos veces.
//
// La salida no es más texto: es **no perder el dato**. El detalle técnico se
// guarda aquí y el error lo enseña plegado, para que se pueda copiar y pegar.
// ───────────────────────────────────────────────────────────────────────────

/** Lo último que falló sin ser un error tipado del núcleo. */
let ultimoDetalle: string | null = null

/**
 * Describe en una línea algo que se lanzó y no es un `ErrorIpc`.
 *
 * Se recorren las formas que de verdad llegan: un `Error` con su mensaje, una
 * cadena suelta, o un objeto que hay que serializar. **Nunca revienta**: esto
 * corre dentro de un `catch`, y una excepción aquí taparía la original.
 */
function describir(e: unknown): string {
  try {
    if (e instanceof Error) {
      return e.stack ? `${e.name}: ${e.message}\n${e.stack}` : `${e.name}: ${e.message}`
    }
    if (typeof e === 'string') return e
    return JSON.stringify(e) ?? String(e)
  } catch {
    return String(e)
  }
}

/**
 * La clave de texto que le corresponde a lo que se capturó.
 *
 * **Sustituye al patrón `esErrorIpc(e) ? e.clave : <una causa inventada>`.**
 * Lo que no viene tipado del núcleo es `error.inesperado`, que dice justo eso
 * —que ARLES no esperaba esto— en vez de culpar a la base.
 *
 * De paso guarda el detalle técnico, que es lo que permite diagnosticarlo
 * después.
 */
export function claveDelError(e: unknown): string {
  if (esErrorIpc(e)) {
    ultimoDetalle = e.detalle && e.detalle.length > 0 ? e.detalle : null
    return e.clave
  }
  ultimoDetalle = describir(e)
  // Y a la consola también: en una sesión de desarrollo es donde se mira
  // primero, y ahí va entero sin recortar.
  console.error('[ARLES] error inesperado:', e)
  return 'error.inesperado'
}

/** El detalle de lo último que falló, para enseñarlo plegado. */
export function ultimoDetalleTecnico(): string | null {
  return ultimoDetalle
}

/** Se olvida al empezar una operación nueva, igual que los avisos (H-01). */
export function olvidarDetalleTecnico(): void {
  ultimoDetalle = null
}
