import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { hayNucleo, invocar } from '@/app/nucleo'

/**
 * Ancho de viewport por debajo del cual la barra lateral se pliega sola.
 *
 * **Medido, no elegido a ojo**, con `npm --prefix app run sonda:plegado`. Es el
 * ancho por debajo del cual la pantalla más exigente —la lista de alta, 78 ch—
 * deja de caber a su medida de diseño con la barra desplegada: 240 px de barra
 * + 64 px de márgenes + 679 px de medida = 983, y el barrido lo confirmó en
 * 988. Plegar devuelve 176 px (240 → 64), justo lo que hace falta.
 *
 * El número se remidió al rediseñar Inicio como panel: la pantalla dejó de
 * declarar un ancho de lectura y pasó a declarar el ancho por debajo del cual
 * la rejilla áurea se apila —684 px—, cuatro más de lo que pedía la lista de
 * alta anterior. Sigue saliendo de `sonda:plegado`, no de una estimación.
 *
 * **No es un umbral de desbordamiento**, y conviene saberlo: la sonda buscó
 * primero dónde desborda y no encontró nada hasta 600 px, porque estas
 * pantallas son fluidas. Ese resultado se documentó en vez de disfrazarse de
 * número redondo. Ver `documentacion/06-calidad/UMBRAL_DE_PLEGADO.md`.
 *
 * El número **cambia cuando cambie la pantalla más ancha**, y la 3.2 trae la
 * tabla de contactos. La sonda lo detectará: falla igual si el umbral se queda
 * corto que si se infla.
 */
export const UMBRAL_MEDIDO_PX = 988

/**
 * El umbral en uso.
 *
 * `VITE_ARLES_UMBRAL_PLEGADO` **sólo lo usa la sonda que mide**: poniéndolo a 1
 * la barra no se pliega nunca sola, y entonces se puede recorrer el eje de
 * anchos con la barra desplegada para encontrar dónde deja de caber la medida de
 * diseño. Sin ese interruptor no hay forma de medir el umbral, porque el propio
 * umbral impide llegar a los anchos donde se mediría.
 *
 * En una compilación normal la variable no existe, Vite sustituye la expresión
 * por `undefined`, `Number(undefined)` es `NaN` y queda el valor medido. Con
 * notación de punto, que es la única que Vite sustituye (ver `router.ts`).
 */
export const UMBRAL_DE_PLEGADO_PX =
  Number(import.meta.env.VITE_ARLES_UMBRAL_PLEGADO) || UMBRAL_MEDIDO_PX

/**
 * Estado de la interfaz que sobrevive al cierre de la aplicación.
 *
 * P-11, decidido por Dirección el 15 de septiembre de 2026:
 *
 * | Pregunta | Decisión |
 * |---|---|
 * | ¿Cómo se pliega? | A mano **y** sola |
 * | ¿Qué queda plegada? | Iconos sin texto |
 * | ¿Se recuerda al reabrir? | Sí |
 *
 * Las dos formas de plegar no compiten: `plegada` es la unión. Lo que el
 * usuario eligió **no se pisa** cuando la ventana se estrecha — si lo
 * hiciéramos, al volver a agrandarla habría perdido su preferencia sin haber
 * tocado nada.
 */
/**
 * Los tres temas que ofrece el desplegable, en el orden en que se ofrecen.
 *
 * **`auto` no es un tema**: es «el que pida el sistema operativo». Lo que se
 * guarda es la palabra, no el resultado de resolverla — guardar el resultado
 * haría que quien cambia su sistema a claro por la noche reabriera ARLES en
 * oscuro sin entender por qué.
 *
 * La lista tiene que ser la misma que `arles_app::comandos::TEMAS`. No se puede
 * importar de Rust, así que la vigila la sonda: si divergen, el núcleo rechaza
 * lo que el desplegable ofrece.
 */
export const TEMAS = ['auto', 'oscuro', 'claro'] as const

export type Tema = (typeof TEMAS)[number]

/** Lo que de verdad se escribe en `data-tema`. `auto` ya está resuelto. */
export type TemaAplicado = 'oscuro' | 'claro'

export const useInterfazStore = defineStore('interfaz', () => {
  /** Lo que el usuario eligió la última vez. Es lo que se recuerda. */
  const preferenciaPlegada = ref(false)
  const anchoVentana = ref(typeof window === 'undefined' ? 0 : window.innerWidth)
  const cargada = ref(false)

  /**
   * El tema elegido (C-1). `auto` mientras el usuario no elija otra cosa.
   */
  const tema = ref<Tema>('auto')

  /**
   * Lo que pide el sistema operativo ahora mismo (C-2).
   *
   * Es un dato que **cambia mientras la aplicación está abierta**: macOS y
   * Windows conmutan a claro y a oscuro por hora del día. Por eso se guarda en
   * un `ref` que el armazón refresca, y no se consulta al vuelo dentro del
   * `computed` — `matchMedia` no es reactivo, así que leerlo ahí daría un valor
   * que Vue nunca sabría que ha caducado.
   *
   * Por defecto oscuro: ARLES es dark-first (§18), así que cuando no se puede
   * preguntar —un entorno sin `matchMedia`— se queda en el tema de la casa.
   */
  const temaDelSistema = ref<TemaAplicado>('oscuro')

  /**
   * El tema que se pinta. Es lo que acaba en `data-tema` del `<html>`.
   *
   * `auto` se resuelve **aquí y no al guardar**: si lo resolviéramos al
   * guardar, la elección del usuario y la respuesta del sistema quedarían
   * fundidas en el mismo dato y ya no habría forma de volver a seguir al
   * sistema sin volver a elegir.
   */
  const temaAplicado = computed<TemaAplicado>(() =>
    tema.value === 'auto' ? temaDelSistema.value : tema.value,
  )

  /** La ventana no da para la barra desplegada sin apretar el contenido. */
  const estrecha = computed(() => anchoVentana.value < UMBRAL_DE_PLEGADO_PX)

  /**
   * ¿Está plegada ahora mismo?
   *
   * A mano **o** sola. Con la ventana estrecha se pliega aunque el usuario la
   * quisiera abierta: ahí la barra se está comiendo 240 px que el contenido
   * necesita para leerse a su medida.
   */
  const plegada = computed(() => estrecha.value || preferenciaPlegada.value)

  /**
   * ¿Puede el usuario cambiarla ahora?
   *
   * Con la ventana estrecha, no: el botón queda deshabilitado y explica por
   * qué. La alternativa —dejarlo pulsable sin efecto visible— es peor: un
   * control que no hace nada al pulsarlo se lee como una avería.
   */
  const alternableAhora = computed(() => !estrecha.value)

  async function cargar(): Promise<void> {
    if (cargada.value) return
    cargada.value = true
    if (!hayNucleo()) return
    try {
      const p = await invocar<{ barraLateralPlegada: boolean; tema: string }>(
        'preferencias_de_interfaz',
      )
      preferenciaPlegada.value = p.barraLateralPlegada
      // El núcleo ya descarta los valores que no reconoce, pero esta capa no
      // puede darlo por hecho: lo que llega por IPC se comprueba en el lado que
      // lo va a usar. Un tema inventado acabaría escrito tal cual en
      // `data-tema` y la aplicación se quedaría sin ningún tema aplicado.
      if (esTema(p.tema)) tema.value = p.tema
    } catch {
      // Una preferencia de interfaz que no se puede leer **no** impide usar la
      // aplicación: se queda desplegada, que es el valor por defecto.
    }
  }

  async function alternar(): Promise<void> {
    if (!alternableAhora.value) return
    preferenciaPlegada.value = !preferenciaPlegada.value
    if (!hayNucleo()) return
    try {
      await invocar('guardar_barra_plegada', { plegada: preferenciaPlegada.value })
    } catch {
      // Igual que arriba: la barra ya se plegó en pantalla. Que no se recuerde
      // al reabrir es peor que nada, pero mucho menos que un error modal por
      // una casilla plegada.
    }
  }

  /**
   * El usuario eligió un tema en Ajustes.
   *
   * El cambio se ve **antes** de guardarse, y se ve igual aunque guardar falle:
   * un desplegable que no cambia la pantalla hasta que la base responda se lee
   * como que no funciona.
   */
  async function elegirTema(nuevo: Tema): Promise<void> {
    if (!esTema(nuevo) || nuevo === tema.value) return
    tema.value = nuevo
    if (!hayNucleo()) return
    try {
      await invocar('guardar_tema', { tema: nuevo })
    } catch {
      // Igual que con la barra: la pantalla ya cambió. Que no se recuerde al
      // reabrir es peor que nada, pero mucho menos que un diálogo de error por
      // haber elegido un color.
    }
  }

  /** La ventana cambió de tamaño. Lo llama el armazón. */
  function anotarAncho(ancho: number): void {
    anchoVentana.value = ancho
  }

  /** El sistema operativo cambió de tema. Lo llama el armazón. */
  function anotarTemaDelSistema(claro: boolean): void {
    temaDelSistema.value = claro ? 'claro' : 'oscuro'
  }

  return {
    preferenciaPlegada,
    anchoVentana,
    cargada,
    estrecha,
    plegada,
    alternableAhora,
    tema,
    temaDelSistema,
    temaAplicado,
    cargar,
    alternar,
    elegirTema,
    anotarAncho,
    anotarTemaDelSistema,
  }
})

/** ¿Es una de las tres palabras? Escrito una vez y usado en los dos bordes. */
export function esTema(valor: unknown): valor is Tema {
  return typeof valor === 'string' && (TEMAS as readonly string[]).includes(valor)
}
