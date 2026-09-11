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
  // Una ruta desconocida no muestra un error: lleva a Inicio.
  { path: '/:pathMatch(.*)*', redirect: '/inicio' },
]

export const router = createRouter({
  // Historial por hash, no por ruta. En una aplicación empaquetada los archivos
  // se sirven desde el sistema de archivos, sin servidor que reescriba rutas:
  // con `createWebHistory`, recargar en `/campanas` daría una ventana en blanco.
  // El modo de desarrollo lo enmascara porque Vite sí reescribe.
  history: createWebHashHistory(),
  routes: rutas,
})
