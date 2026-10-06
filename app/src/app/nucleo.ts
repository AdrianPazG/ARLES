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
