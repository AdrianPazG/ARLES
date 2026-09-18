import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'

import type { NombreDeIcono } from '@/design/componentes'

/**
 * Las seis secciones de UX_NAVEGACION.md §2.
 *
 * Se recortaron de las siete del §23 porque dos se solapaban: las plantillas
 * son un insumo de las campañas, no un destino propio, y «Actividad» duplicaba
 * la salud de envío.
 */
export const SECCIONES = [
  'inicio',
  'campanas',
  'contactos',
  'remitentes',
  'actividad',
  'ajustes',
] as const

export type Seccion = (typeof SECCIONES)[number]

/**
 * El icono de cada sección.
 *
 * `Record<Seccion, …>` y no un objeto suelto: añadir una séptima sección sin
 * darle icono **no compila**. Es la garantía que necesita la decisión de P-11
 * —«iconos sin texto» al plegar—, porque una sección sin icono queda plegada
 * como un hueco en blanco que no se puede pulsar con criterio.
 */
export const ICONO_DE_SECCION: Record<Seccion, NombreDeIcono> = {
  inicio: 'inicio',
  campanas: 'campanas',
  contactos: 'contactos',
  remitentes: 'remitentes',
  actividad: 'actividad',
  ajustes: 'ajustes',
}

const rutas: RouteRecordRaw[] = [
  { path: '/', redirect: '/inicio' },
  {
    path: '/inicio',
    name: 'inicio',
    component: () => import('@/app/pantallas/PantallaInicio.vue'),
    meta: { seccion: 'inicio' },
  },
  {
    path: '/contactos',
    name: 'contactos',
    component: () => import('@/app/pantallas/PantallaContactos.vue'),
    meta: { seccion: 'contactos' },
  },
  {
    path: '/ajustes',
    name: 'ajustes',
    component: () => import('@/app/pantallas/PantallaAjustes.vue'),
    meta: { seccion: 'ajustes' },
  },
  // Las demás siguen siendo andamio hasta su entrega del roadmap. La lista de
  // alta de Inicio dice en cuál llega cada una, así que la pantalla vacía no
  // es una sorpresa.
  ...SECCIONES.filter(
    (s) => s !== 'inicio' && s !== 'ajustes' && s !== 'contactos',
  ).map((s) => ({
    path: `/${s}`,
    name: s,
    component: () => import('@/app/PantallaPendiente.vue'),
    meta: { seccion: s },
  })),
]

/**
 * ¿Se registra el catálogo del design system?
 *
 * Siempre en desarrollo. Y además cuando se compila con
 * `VITE_ARLES_CATALOGO=1`, que es lo que usan **la sonda de CSP** y **la
 * compilación de revisión visual** — la que se abre en Windows y en macOS para
 * mirar las primitivas en su sistema real.
 *
 * Nunca en una compilación normal de producción: ahí sería superficie de
 * ataque sin contrapartida.
 *
 * Por qué una variable y no editar este archivo al vuelo: la sonda de CSP
 * **lo hacía**, y si algo la interrumpía entre la edición y la restauración,
 * dejaba el interruptor abierto en el árbol de trabajo. Una variable de
 * entorno no deja residuo.
 *
 * Y por qué con **notación de punto**: Vite sólo sustituye así. Con corchetes
 * la expresión queda dinámica, la rama sobrevive a la compilación y el
 * catálogo entra en el bundle. Ver `src/entorno.d.ts`.
 */
export const CATALOGO_VISIBLE =
  import.meta.env.DEV || import.meta.env.VITE_ARLES_CATALOGO === '1'

if (CATALOGO_VISIBLE) {
  rutas.push({
    path: '/catalogo',
    name: 'catalogo',
    component: () => import('@/design/CatalogoDelSistema.vue'),
  })
}

// Una ruta desconocida no muestra un error: lleva a Inicio. Va la última
// porque el comodín captura todo lo que llegue después de ella.
rutas.push({ path: '/:pathMatch(.*)*', redirect: '/inicio' })

export const router = createRouter({
  // Historial por hash, no por ruta. En una aplicación empaquetada los archivos
  // se sirven desde el sistema de archivos, sin servidor que reescriba rutas:
  // con `createWebHistory`, recargar en `/campanas` daría una ventana en blanco.
  // El modo de desarrollo lo enmascara porque Vite sí reescribe.
  history: createWebHashHistory(),
  routes: rutas,
})
