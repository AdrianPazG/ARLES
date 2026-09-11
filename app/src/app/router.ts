import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'

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

const rutas: RouteRecordRaw[] = [
  { path: '/', redirect: '/inicio' },
  ...SECCIONES.map((s) => ({
    path: `/${s}`,
    name: s,
    component: () => import('@/app/PantallaPendiente.vue'),
    meta: { seccion: s },
  })),
]

// El catálogo del design system es una pantalla de desarrollo: sirve para
// revisar las primitivas con los ojos, en Windows y en macOS. No viaja en el
// bundle que se instala en la máquina del cliente, y por eso se registra
// aquí y no en SECCIONES —donde aparecería en la navegación.
if (import.meta.env.DEV) {
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
