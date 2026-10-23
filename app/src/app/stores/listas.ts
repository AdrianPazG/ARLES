import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { claveDelError, hayNucleo, invocar } from '@/app/nucleo'

/**
 * Las listas de contactos · D-15, 23/10/2026
 *
 * ───────────────────────────────────────────────────────────────────────────
 * LO QUE PIDIÓ DIRECCIÓN, Y POR QUÉ NO ES UNA COMODIDAD
 *
 *   «Cuando creen una campaña por ejemplo "Videovigilancia" y carguen todos los
 *   contactos donde podrían tener interés… pero si se crea otra campaña de
 *   "Desarrollo de aplicaciones" puedan cargar su tabla de contactos… y no
 *   mezclar contactos donde no sean de interés del público y así arriesgar el
 *   dominio de que le den spam.»
 *
 * Hasta el 23/10/2026 **toda campaña iba a todo el padrón**. Mandar una campaña
 * de videovigilancia a quien pidió una aplicación es la vía más rápida a que
 * marquen el correo como no deseado, y la tasa de quejas es la cifra que más
 * pesa en la reputación de un dominio.
 *
 * ───────────────────────────────────────────────────────────────────────────
 * POR QUÉ UN STORE PROPIO Y NO DENTRO DEL DE IMPORTACIÓN
 *
 * Porque lo usan dos pantallas que no se hablan: Importar, para decir a qué
 * lista entran los contactos del archivo, y el asistente de campaña, para
 * elegir a quién va dirigida. Meterlo en el de importación obligaría al
 * asistente a montar un flujo de importación que no va a usar.
 */

/** Una lista, tal y como llega de Rust. */
export interface ListaDeContactos {
  readonly id: string
  readonly nombre: string
  /**
   * Cuánta gente tiene dentro.
   *
   * **Va siempre con el nombre** (§94). Una lista llamada «Videovigilancia»
   * puede tener trescientos o doce, y elegirla sin esa cifra es elegir a
   * ciegas — justo en la decisión de a cuánta gente se le va a escribir.
   */
  readonly cuantos: number
  /** Cuántas campañas la usan ya. */
  readonly campanas: number
}

export const useListasStore = defineStore('listas', () => {
  const listas = ref<readonly ListaDeContactos[]>([])
  const cargando = ref(false)
  const errorGeneral = ref<string | null>(null)

  /** Las que tienen a alguien dentro. Una lista vacía no se puede enviar. */
  const conGente = computed(() => listas.value.filter((l) => l.cuantos > 0))

  async function cargar(): Promise<void> {
    cargando.value = true
    errorGeneral.value = null
    try {
      if (!hayNucleo()) {
        // Sin núcleo no hay listas, y **no es un error**: es la vista previa.
        // Pintar un aviso rojo aquí convertiría la demostración en una avería.
        listas.value = []
        return
      }
      listas.value = await invocar<ListaDeContactos[]>('listas_de_contactos')
    } catch (e) {
      errorGeneral.value = claveDelError(e)
    } finally {
      cargando.value = false
    }
  }

  return { listas, cargando, errorGeneral, conGente, cargar }
})
