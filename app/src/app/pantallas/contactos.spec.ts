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

/**
 * R-09 · las acciones, al final de la fila · 21/10/2026
 *
 * Estaban dentro de la ficha, que se abre con un doble clic, y el estado
 * vacío de la columna lo decía por escrito desde la 7.7. Dirección probó la
 * aplicación y no llegó a ellas:
 *
 *   «como no se visualiza que puedas tener esa opción, me gustaría que mejor
 *    se abriera esa opción con un icono al final de la fila del contacto»
 *
 * ───────────────────────────────────────────────────────────────────────────
 * LO QUE ESTA PRUEBA **NO** PUEDE COMPROBAR, Y DÓNDE SE COMPRUEBA
 *
 * `ATabla` es virtualizada: sólo pinta las filas que caben en su ventana, y en
 * jsdom todo mide **cero**. No se pinta ni una fila, así que el slot de las
 * acciones **no llega a existir** y los iconos no se pueden pulsar aquí.
 *
 * Se intentó fingir `clientHeight` y `getBoundingClientRect`, y no bastó: el
 * virtualizador mide por su cuenta. Seguir por ahí sería pelearse con la
 * biblioteca en vez de probar el código propio, y acabaría en una prueba que
 * pasa sin ejercitar nada — que es justo lo que este proyecto persigue.
 *
 * Así que aquí se comprueba **lo que sí se puede**: que la columna existe con
 * su título. Que los iconos se ven y se pulsan se verifica **en Chromium**,
 * con la vista previa, que es como este proyecto mira lo visual desde la
 * revisión del 29/09/2026.
 */
describe('la columna de acciones', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('la tabla tiene su columna de acciones al final', () => {
    const contactos = useContactosStore()
    contactos.cargada = true
    contactos.cargando = false
    contactos.contactos = [UN_CONTACTO]
    contactos.total = 1

    const p = mount(PantallaContactos, entorno)
    const tabla = p.findComponent({ name: 'ATabla' })
    expect(tabla.exists()).toBe(true)

    const columnas = tabla.props('columnas') as readonly { id: string; titulo: string }[]
    const ultima = columnas[columnas.length - 1]
    expect(ultima?.id, 'la columna de acciones va al final de la fila').toBe('acciones')
    expect(ultima?.titulo).toBe('Acciones')
  })
})

/**
 * El filtro de la lista · R-26, 03/11/2026
 *
 * ───────────────────────────────────────────────────────────────────────────
 * LO QUE SÓLO ESTA CAPA PUEDE EQUIVOCAR
 *
 * Que el `WHERE` recorte bien lo prueban las cinco pruebas de
 * `arles-db::contactos`, y lo que cuesta lo mide el banco. Aquí se vigilan las
 * tres cosas que ninguna de las dos puede ver, y que son las que el usuario
 * nota:
 *
 * 1. Que el texto **no busque al teclear**. No es cosmética: medido, un texto
 *    que todavía no existe cuesta 867 ms con 455 470 contactos, así que
 *    escribir «ana» serían tres consultas y dos y medio segundos, y las dos
 *    primeras tiradas. Si alguien «mejora» esto poniendo un `@input`, la
 *    pantalla se vuelve inusable con una lista de verdad y **ninguna otra
 *    prueba lo notaría**.
 * 2. Que buscar **vuelva a la primera página**. Quedarse en la página 4 de una
 *    lista que acaba de pasar de 455 470 a 3 deja la tabla vacía con un total
 *    que dice 3: se lee como «el filtro no encuentra nada».
 * 3. Que el vacío del filtro **no sea el vacío de «no tienes contactos»**. Es
 *    el defecto de la pantalla de recuperación del 07/10 —«no se ha perdido
 *    nada» con 48 MB en disco—: un estado correcto para otra situación, que se
 *    lee como una afirmación falsa sobre los datos.
 *
 * **Vistas en rojo** con tres rupturas: atando el texto a `@input`, llamando a
 * `cargar(desde)` en vez de `cargar(0)`, y quitando la rama del vacío con
 * filtro.
 */
describe('el filtro de la lista de contactos (R-26)', () => {
  beforeEach(() => setActivePinia(createPinia()))

  /** Monta la pantalla con la lista ya cargada y espía las peticiones. */
  function conLaLista(): { pantalla: VueWrapper; desdes: number[] } {
    const contactos = useContactosStore()
    contactos.cargada = true
    contactos.cargando = false
    contactos.contactos = [UN_CONTACTO]
    contactos.total = 1
    const desdes: number[] = []
    contactos.cargar = async (inicio = 0) => {
      desdes.push(inicio)
    }
    const pantalla = mount(PantallaContactos, entorno)

    // ── EL MONTAJE YA PIDE LA LISTA, Y POR ESO SE VACÍA AQUÍ ──
    //
    // `onMounted` llama a `cargar()`, así que sin esta línea `desdes` arranca
    // con un `0` dentro. La primera versión de la prueba del tecleo cayó por
    // eso —y las otras tres **habrían pasado por la llamada del montaje**, no
    // por lo que vinieron a comprobar—. Una prueba que no distingue las dos
    // llamadas no prueba nada, que es el §2 aplicado a sí misma.
    desdes.length = 0
    return { pantalla, desdes }
  }

  /** El campo de texto del filtro, por su etiqueta. */
  function campoDelTexto(pantalla: VueWrapper) {
    return pantalla
      .findAllComponents(AEntrada)
      .find((e) => e.props('etiqueta') === 'Nombre, apellido o empresa')
  }

  it('el texto NO busca al teclear: hay que pulsar Enter o el botón', async () => {
    const { pantalla, desdes } = conLaLista()
    const campo = campoDelTexto(pantalla)
    expect(campo?.exists(), 'el campo de texto del filtro').toBe(true)

    const dentro = campo?.find('input')
    // Tres letras, como quien escribe «ana».
    for (const v of ['a', 'an', 'ana']) {
      await dentro?.setValue(v)
    }
    await nextTick()
    expect(
      desdes,
      'teclear no puede pedir la lista: 867 ms por letra con medio millón de contactos',
    ).toEqual([])

    await dentro?.trigger('keyup.enter')
    await nextTick()
    expect(desdes, 'Enter sí busca').toEqual([0])
  })

  it('al buscar vuelve a la primera página', async () => {
    const contactos = useContactosStore()
    const { pantalla, desdes } = conLaLista()
    // Como si estuviéramos en la página 4.
    contactos.desde = 300
    contactos.filtroTexto = 'ana'
    await nextTick()

    const boton = pantalla.findAll('button').find((b) => b.text() === 'Buscar')
    expect(boton?.exists(), 'el botón de buscar').toBe(true)
    await boton?.trigger('click')
    await nextTick()
    expect(desdes, 'quedarse en la página 4 deja la tabla vacía con un total de 3').toEqual([0])
  })

  it('los desplegables sí buscan solos: cuestan 0.1 ms', async () => {
    const { pantalla, desdes } = conLaLista()
    const canal = pantalla
      .findAllComponents(ASelector)
      .find((s) => s.props('etiqueta') === 'Tiene canal de')
    expect(canal?.exists(), 'el desplegable de canal').toBe(true)
    canal?.vm.$emit('update:modelValue', 'whatsapp')
    await nextTick()
    expect(desdes).toEqual([0])
  })

  it('sin resultados con filtro dice que la lista sigue completa, no que no haya contactos', async () => {
    const contactos = useContactosStore()
    const { pantalla } = conLaLista()
    contactos.contactos = []
    contactos.total = 0
    contactos.filtroTexto = 'nadie'
    await nextTick()

    const texto = pantalla.text()
    expect(texto).toContain('Ningún contacto coincide con lo que buscas')
    expect(
      texto,
      'el vacío de «todavía no hay contactos» es de otra situación: con 455 470 ' +
        'contactos detrás, decirlo es afirmar algo falso sobre los datos',
    ).not.toContain('Todavía no has agregado')
  })
})
