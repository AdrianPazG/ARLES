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
}

interface ConTauri {
  __TAURI_INTERNALS__?: unknown
}

/** ¿Hay un núcleo al otro lado? */
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
