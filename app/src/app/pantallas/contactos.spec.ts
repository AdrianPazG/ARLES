import { mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { nextTick } from 'vue'

import { i18n } from '@/app/i18n'
import { useContactosStore } from '@/app/stores/contactos'
import { useSupresionStore } from '@/app/stores/supresion'

import { AEntrada, ASelector } from '@/design/componentes'

import PantallaContactos from './PantallaContactos.vue'

/**
 * El diálogo de baja de un contacto · 17/10/2026
 *
 * ───────────────────────────────────────────────────────────────────────────
 * DAR DE BAJA Y NO VOLVER A ESCRIBIR SON DOS COSAS, Y SÓLO HABÍA UNA
 *
 * `suprimir_contacto` estaba registrado en la frontera, el almacén lo
 * envolvía con su motivo y sus notas, y **ninguna pantalla lo llamaba**. El
 * patrón del §9 por vigésima vez, encontrado al arreglar la comprobación que
 * existe para cazarlo — que vigilaba cero funciones de almacén.
 *
 * Lo que faltaba no es un botón más: es la única puerta a una supresión que no
 * venga del acuse de un correo. Quien pide por teléfono que no le escriban
 * necesita esto, y una baja **no** lo cumple: se deshace reimportando el mismo
 * Excel sin querer.
 *
 * ───────────────────────────────────────────────────────────────────────────
 * LO QUE SE VIGILA AQUÍ
 *
 * No se comprueba qué escribe la supresión en la base —eso lo prueban
 * `arles-core::supresion` y `arles-db::supresion`— sino las cuatro cosas que
 * **sólo esta pantalla puede equivocar**, y que ninguna prueba de las dos
 * capas puede ver:
 *
 * 1. Que la casilla llegue hasta el comando.
 * 2. Que **sin marcarla no se suprima a nadie**. Suprimir de más es lo que no
 *    se deshace: con el motivo «lo pidió la persona», ARLES no lo levanta.
 * 3. Que la supresión vaya **antes** de la baja.
 * 4. Que si la supresión falla, **no se dé de baja** — y se diga.
 *
 * **Vistas en rojo el 17/10/2026** con cinco rupturas: no suprimir nunca,
 * suprimir siempre ignorando la casilla, invertir el orden, dar de baja
 * aunque la supresión falle, y dejar el error de supresión sin enseñar.
 *
 * Y dos de la comprobación que encontró este hueco
 * (`lo_que_se_construye_se_usa`): quitar esta llamada —la caza— y romperle el
 * troceado —salta su guardarraíl de «sólo se leyeron N funciones»—, que es lo
 * que impide que vuelva a vigilar cero sin decirlo.
 */

const entorno = {
  global: {
    plugins: [i18n],
    stubs: {
      RouterLink: { props: ['to'], template: '<a :href="to"><slot /></a>' },
      Teleport: true,
    },
  },
}

const UN_CONTACTO = {
  id: 'c-1',
  nombre: 'Ana',
  apellido: 'Ruiz',
  empresa: 'Fábrica',
  canales: [
    {
      canal: 'email' as const,
      valorRaw: 'ana@empresa.mx',
      valorNormalizado: 'ana@empresa.mx',
      principal: true,
    },
  ],
  creadoEn: '2026-10-01T00:00:00Z',
  actualizadoEn: '2026-10-01T00:00:00Z',
}

/** Monta la pantalla con un contacto y abre su diálogo de baja. */
async function abrirBaja(): Promise<{
  pantalla: VueWrapper
  pedidos: string[]
}> {
  const contactos = useContactosStore()
  contactos.cargada = true
  contactos.cargando = false
  contactos.contactos = [UN_CONTACTO]
  contactos.total = 1

  const supresion = useSupresionStore()
  const pedidos: string[] = []

  contactos.borrar = async (id) => {
    pedidos.push(`baja:${id}`)
    return true
  }
  supresion.suprimirContacto = async (id, motivo, notas) => {
    pedidos.push(`suprimir:${id}:${motivo}:${notas}`)
    return { nuevas: 1, yaEstaban: 0 }
  }

  const pantalla = mount(PantallaContactos, entorno)

  // La ficha se abre al **activar** una fila, y la fila se emite desde
  // `ATabla`. No se pulsa aquí: la tabla es virtualizada y en jsdom el
  // contenedor mide cero, así que no pinta ni una fila — buscarla devolvería
  // `undefined` y la prueba pasaría sin ejercitar nada. Emitiendo el evento
  // del componente de verdad, lo que corre es el manejador de la pantalla.
  const tabla = pantalla.findComponent({ name: 'ATabla' })
  expect(tabla.exists(), 'la tabla de contactos').toBe(true)
  tabla.vm.$emit('activar', UN_CONTACTO)
  await nextTick()

  const baja = pantalla.findAll('button').find((b) => b.text() === 'Dar de baja')
  await baja?.trigger('click')
  return { pantalla, pedidos }
}

/** Pulsa el botón rojo del diálogo. */
async function confirmar(pantalla: VueWrapper): Promise<void> {
  const botones = pantalla.findAll('button').filter((b) => b.text() === 'Dar de baja')
  expect(botones.length, 'el de la ficha y el del diálogo').toBeGreaterThan(1)
  // El de la ficha abre; el del diálogo confirma, y es el último.
  await botones[botones.length - 1]?.trigger('click')
  await nextTick()
}

describe('dar de baja a un contacto', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('ofrece no volver a escribirle, y dice en qué se diferencia de la baja', async () => {
    const { pantalla } = await abrirBaja()
    const texto = pantalla.text()

    expect(texto).toContain('no volver a escribirle nunca')
    // La frase que importa: una baja se deshace reimportando sin querer.
    expect(texto).toContain('vuelves a importar')
  })

  /**
   * **Sin marcar la casilla no se suprime a nadie.**
   *
   * Es la mitad que más cara sale de equivocar: suprimir «de más» con el
   * motivo «lo pidió la persona» no lo deshace ARLES, así que esa dirección
   * se queda fuera de toda campaña para siempre.
   */
  it('sin marcar la casilla, sólo da de baja', async () => {
    const { pantalla, pedidos } = await abrirBaja()
    await confirmar(pantalla)

    expect(pedidos).toEqual(['baja:c-1'])
  })

  it('marcándola, suprime con su motivo y su nota, y luego da de baja', async () => {
    const { pantalla, pedidos } = await abrirBaja()

    await pantalla.find('input[type="checkbox"]').setValue(true)

    // Por etiqueta y no por `find('select')`: el formulario de alta también
    // está montado —los diálogos se renderizan los dos— y tiene su propio
    // desplegable, así que el primero que aparece no es éste. Con el
    // equivocado la prueba habría pasado sin cambiar el motivo.
    const motivo = pantalla
      .findAllComponents(ASelector)
      .find((s) => s.props('etiqueta') === '¿Por qué?')
    expect(motivo, 'el selector del motivo').toBeDefined()
    await motivo?.find('select').setValue('manual')

    const notas = pantalla.findAllComponents(AEntrada).find((e) => e.props('etiqueta') === 'Nota (opcional)')
    await notas?.find('input').setValue('lo pidió por teléfono')
    await confirmar(pantalla)

    // El orden es la garantía: la supresión primero. Al revés, un fallo al
    // suprimir deja el contacto fuera de la lista y su dirección sin
    // proteger, y la siguiente importación lo devuelve.
    expect(pedidos).toEqual(['suprimir:c-1:manual:lo pidió por teléfono', 'baja:c-1'])
  })

  /**
   * **Si la supresión falla, no se da de baja.** Y se dice.
   *
   * Dar de baja igualmente dejaría al contacto fuera de la lista con su
   * dirección sin apuntar: desaparecería de la vista el único sitio donde se
   * ve que eso quedó a medias.
   */
  it('si la supresión falla, el contacto se queda y el error se enseña', async () => {
    const { pantalla, pedidos } = await abrirBaja()
    const supresion = useSupresionStore()
    supresion.suprimirContacto = async () => {
      supresion.errorGeneral = 'error.db.sqlite'
      return null
    }

    await pantalla.find('input[type="checkbox"]').setValue(true)
    await confirmar(pantalla)

    expect(pedidos, 'no se dio de baja').toEqual([])
    expect(pantalla.text()).toContain('Ocurrió un error al acceder a los datos')
  })
})
