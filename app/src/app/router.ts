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

export function esSeccion(v: unknown): v is Seccion {
  return SECCIONES.includes(v as Seccion)
}

/**
 * Una pantalla que cuelga de una sección y se enseña **dentro** de ella.
 *
 * ───────────────────────────────────────────────────────────────────────────
 * POR QUÉ ESTO EXISTE, Y POR QUÉ NO SE USA `router-link-active`
 *
 * La clase `router-link-active` que pone vue-router **no compara rutas por
 * texto**: compara si el registro de la ruta actual desciende del registro del
 * enlace. Y aquí `/inicio/configuracion` está declarada como HERMANA de
 * `/inicio`, no como hija suya — no hay `children`, porque eso obligaría a
 * convertir `PantallaInicio` en un armazón con su propio `<RouterView>`.
 *
 * Resultado medido el 21/09/2026:
 *
 *     /inicio                 → casa con  ['/inicio']
 *     /inicio/configuracion   → casa con  ['/inicio/configuracion']
 *     /contactos/importar     → casa con  ['/contactos/importar']
 *     /contactos/supresion    → casa con  ['/contactos/supresion']
 *
 * Es decir: estando en cualquiera de esas tres pantallas, **la barra lateral
 * se quedaba entera apagada**. No es un fallo de la configuración: Importar y
 * Quién pidió no recibir lo arrastraban desde la entrega 3.2, y nadie lo vio
 * porque las pruebas de pantalla sustituyen `RouterLink` por un doble que no
 * calcula ningún estado activo.
 *
 * Lo encontró Dirección mirando la pantalla, igual que el fallo de ayer.
 *
 * La marca pasa a salir de `meta.seccion`, que es el dato que ya declara cada
 * ruta y el único que significa lo que hace falta: «a qué sección pertenece
 * esta pantalla». El día que se añada `/campanas/plantillas`, se alumbra sola.
 * ───────────────────────────────────────────────────────────────────────────
 */
export interface Subseccion {
  /** La ruta a la que lleva. */
  ruta: string
  /** Clave de i18n de su nombre (§139: claves, no texto). */
  clave: string
  /** Orden dentro de la sección. Explícito y no alfabético: el orden es una
   *  decisión de producto, no una casualidad de cómo se escriben las palabras. */
  orden: number
}

/**
 * Las subsecciones de cada sección, sacadas de las propias rutas.
 *
 * Derivado y no escrito a mano: una segunda lista a mano es una lista que se
 * queda vieja el día que alguien añade una ruta y no se acuerda de tocarla.
 * Aquí la ruta es la fuente, y si no declara `subseccion` no aparece.
 */
export function subseccionesDe(seccion: Seccion): Subseccion[] {
  return router
    .getRoutes()
    .filter((r) => r.meta['seccion'] === seccion && r.meta['subseccion'])
    .map((r) => ({
      ruta: r.path,
      clave: `nav.sub.${String(r.meta['subseccion'])}`,
      orden: Number(r.meta['orden'] ?? 0),
    }))
    .sort((a, b) => a.orden - b.orden)
}

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
    // Cuelga de Inicio y no es una séptima sección: el producto son seis
    // (P-13, 17/09/2026), y una sección de «configuración inicial» es una
    // entrada en la barra que pasada la primera semana no se vuelve a usar.
    //
    // Y NO cuelga de Ajustes, que es lo que hacía antes: Ajustes es de la
    // aplicación —tema, densidad, versión—, no del negocio.
    path: '/inicio/configuracion',
    name: 'configuracion',
    component: () => import('@/app/pantallas/PantallaConfiguracion.vue'),
    meta: { seccion: 'inicio', subseccion: 'configuracion', orden: 1 },
  },
  {
    // Es una de las seis secciones desde el principio (P-13): una campaña no es
    // un paso que se hace una vez, es lo que se mira cada mañana mientras está
    // enviando.
    path: '/campanas',
    name: 'campanas',
    component: () => import('@/app/pantallas/PantallaCampanas.vue'),
    meta: { seccion: 'campanas' },
  },
  {
    // Cuelga de Campañas porque una plantilla es el texto de una campaña, y
    // quien busca «qué se manda» mira ahí. Otra sección rompería P-11, que
    // fija seis.
    //
    // Sí es una subsección permanente, a diferencia del asistente: escribir
    // plantillas es un sitio al que se vuelve —a añadir otra, a mirar cuál se
    // usó—, no un camino con principio y final.
    path: '/campanas/plantillas',
    name: 'plantillas',
    component: () => import('@/app/pantallas/PantallaPlantillas.vue'),
    meta: { seccion: 'campanas', subseccion: 'plantillas', orden: 1 },
  },
  {
    // El asistente cuelga de campañas y no es una sección: es un camino con
    // principio y final, como importar. Una entrada permanente en la barra
    // diría que crear campañas es un sitio al que se va, y es algo que se hace.
    path: '/campanas/nueva',
    name: 'campanaNueva',
    component: () => import('@/app/pantallas/AsistenteDeCampana.vue'),
    meta: { seccion: 'campanas', subseccion: 'nueva', orden: 1 },
  },
  {
    path: '/contactos',
    name: 'contactos',
    component: () => import('@/app/pantallas/PantallaContactos.vue'),
    meta: { seccion: 'contactos' },
  },
  {
    // Cuelga de contactos porque es lo que hace: mete contactos. Una sección
    // propia en la barra sugeriría que importar es un destino, y es un camino.
    path: '/contactos/importar',
    name: 'importar',
    component: () => import('@/app/pantallas/PantallaImportar.vue'),
    meta: { seccion: 'contactos', subseccion: 'importar', orden: 1 },
  },
  {
    // Cuelga de contactos por lo mismo que importar: es una vista de los
    // contactos, no un destino propio. Una séptima entrada en la barra diría
    // que las bajas son un área del producto, y son una propiedad de la gente.
    path: '/contactos/supresion',
    name: 'supresion',
    component: () => import('@/app/pantallas/PantallaSupresion.vue'),
    meta: { seccion: 'contactos', subseccion: 'supresion', orden: 2 },
  },
  {
    // Los derechos ARCO cuelgan de CONTACTOS por el mismo criterio que la
    // lista de bajas (UX_NAVEGACION.md §2): **son decisiones sobre personas**,
    // y quien las atiende las busca donde están las personas, no en un
    // apartado legal aparte.
    //
    // Tampoco es una sección propia: atender una solicitud de acceso es algo
    // que pasa de vez en cuando, y una séptima entrada permanente en la barra
    // para algo esporádico desplaza a las seis que se usan a diario.
    path: '/contactos/derechos',
    name: 'derechos',
    component: () => import('@/app/pantallas/PantallaDerechos.vue'),
    meta: { seccion: 'contactos', subseccion: 'derechos', orden: 3 },
  },
  {
    // Sí es una de las seis secciones (P-13): una cuenta remitente es un
    // insumo que se revisa —cuando deja de funcionar, cuando cambia la
    // contraseña de aplicación— y no un paso que se hace una vez.
    path: '/remitentes',
    name: 'remitentes',
    component: () => import('@/app/pantallas/PantallaRemitentes.vue'),
    meta: { seccion: 'remitentes' },
  },
  {
    // WhatsApp cuelga de Remitentes y no es sección propia: es el otro canal
    // desde el que se escribe, y quien busca «desde dónde sale esto» mira ahí.
    // Una séptima sección rompería además P-11, que fija seis.
    path: '/remitentes/whatsapp',
    name: 'whatsapp',
    component: () => import('@/app/pantallas/PantallaWhatsApp.vue'),
    meta: { seccion: 'remitentes', subseccion: 'whatsapp', orden: 1 },
  },
  {
    // Sí es una de las seis (P-13): «cómo va lo que mandé» es lo que se mira
    // cada mañana mientras hay una campaña enviando, y con tres a la vez
    // (D-7A) es el sitio donde se compara una con otra.
    path: '/actividad',
    name: 'actividad',
    component: () => import('@/app/pantallas/PantallaActividad.vue'),
    meta: { seccion: 'actividad' },
  },
  {
    // Cuelga de Actividad y no es sección propia: el registro es el detalle de
    // lo que Actividad resume, y quien busca «qué pasó con Ana» mira ahí. Una
    // séptima entrada en la barra rompería P-11, que fija seis.
    //
    // Sí es una subsección permanente, a diferencia del asistente de campaña:
    // es un sitio al que se vuelve —a buscar un envío, a revisar un rebote—, no
    // un camino con principio y final.
    path: '/actividad/registro',
    name: 'registro',
    component: () => import('@/app/pantallas/PantallaRegistro.vue'),
    meta: { seccion: 'actividad', subseccion: 'registro', orden: 1 },
  },
  {
    path: '/ajustes',
    name: 'ajustes',
    component: () => import('@/app/pantallas/PantallaAjustes.vue'),
    meta: { seccion: 'ajustes' },
  },
  {
    // Cuelga de Ajustes y no es una séptima sección (P-13). La cabecera de
    // `PantallaAjustes` ya decía qué pertenece ahí —«tema, densidad, versión,
    // respaldos»—: un respaldo es una propiedad de la herramienta, no del
    // negocio, porque no decide a quién se le escribe ni a qué hora.
    path: '/ajustes/respaldos',
    name: 'respaldos',
    component: () => import('@/app/pantallas/PantallaRespaldos.vue'),
    meta: { seccion: 'ajustes', subseccion: 'respaldos', orden: 1 },
  },
  {
    // Restaurar vive al lado de Respaldos y no dentro: son dos acciones
    // opuestas, y la segunda no se puede deshacer. Compartir pantalla con el
    // botón de crear un respaldo pondría «Crear» y «Reemplazar toda la base» a
    // dos centímetros el uno del otro.
    path: '/ajustes/restaurar',
    name: 'restaurar',
    component: () => import('@/app/pantallas/PantallaRestaurar.vue'),
    meta: { seccion: 'ajustes', subseccion: 'restaurar', orden: 2 },
  },
  {
    // Archivar va la tercera y no junto a Respaldos, por lo mismo que
    // Restaurar: borra. Y va **después** de Restaurar a propósito — quien
    // llega a Ajustes buscando «recuperar mis datos» tiene que encontrar
    // Restaurar antes de encontrar algo que quita historial.
    path: '/ajustes/archivar',
    name: 'archivar',
    component: () => import('@/app/pantallas/PantallaArchivar.vue'),
    meta: { seccion: 'ajustes', subseccion: 'archivar', orden: 3 },
  },
  {
    // ── SIN SECCIÓN, Y ES LO QUE LA SACA DEL ARMAZÓN ────────────────────
    //
    // En el modo de recuperación no hay base abierta: ni contactos, ni
    // campañas, ni ajustes. Enseñar la barra lateral sería ofrecer seis
    // puertas pintadas a quien acaba de perder el acceso a sus datos.
    path: '/recuperar',
    name: 'recuperar',
    component: () => import('@/app/pantallas/PantallaRecuperar.vue'),
    meta: { sinArmazon: true },
  },
  // ── YA NO HAY ANDAMIO ─────────────────────────────────────────────────
  //
  // Hasta la 7.3, las secciones sin pantalla se registraban en bloque contra
  // `PantallaPendiente.vue`. Con Actividad construida, **las seis tienen la
  // suya**, así que ese bloque dejó de casar con nada y se quitó junto con el
  // componente: un `filter` que excluye las seis de seis es un `map` sobre una
  // lista vacía, y el componente al que apuntaba habría quedado sin ruta — el
  // patrón del §9 justo al revés, y de los que sólo se ven leyendo.
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
