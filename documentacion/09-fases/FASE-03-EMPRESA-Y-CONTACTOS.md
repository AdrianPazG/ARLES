# Fase 3 · Empresa y contactos

**Estado:** 🟡 en curso — **entrega 3.1 cerrada** · **Fecha:** 2026-09-15
**Validación:** 19/19 comprobaciones de la fase 3, 0 omitidas — `validar.py --fase 3`
**Pruebas:** 128 de Rust · 49 de frontend · 6 sondas de navegador

> **Objetivo de la 3.1.** La primera pantalla real del producto: configurar la
> empresa y saber qué falta para poder enviar. Y de paso, las dos peticiones
> que Dirección dejó en la revisión visual del 14 de septiembre.

Las otras cuatro entregas de la fase siguen pendientes: 3.2 contactos, 3.3
importación, 3.4 supresión y derechos ARCO, 3.5 cierre.

---

## 1. Qué se construyó

### El núcleo decide, la pantalla pinta

| Módulo | Qué defiende |
|---|---|
| `arles_core::empresa` | Valida el formulario y devuelve **todos** los campos malos de una vez, cada uno con su clave de texto |
| `arles_core::onboarding` | **Deriva** la lista de alta de los datos reales |
| `arles_db::empresa` | Guarda, lee y **vuelve a validar al leer**; deja constancia en la bitácora en la misma transacción |
| `arles_db::preferencias` | Clave/valor de interfaz, con lista cerrada de claves |
| Migración `V2` | `ui_preference`, la tabla que hace posible el «se recuerda» de P-11 |

Cuatro comandos gruesos nuevos. `guardar_empresa` devuelve **la misma
configuración que se lee al abrir**, no un `()`: la lista de alta cambia al
guardar, y sin eso la pantalla tendría que volver a pedirla — otro cruce de la
frontera y un instante diciendo que el paso sigue pendiente.

### Cuatro decisiones que se ven en el código

**1 · La lista de alta se deriva, no se guarda.** Lo cómodo sería un booleano
por paso. Entonces basta que alguien borre su única cuenta remitente para que la
lista siga diciendo, para siempre, que ese paso está hecho. Aquí cada paso es
una consulta sobre los datos, así que **no puede mentir**.

**2 · Las zonas horarias son una lista cerrada.** Validar un identificador IANA
de verdad exige una base de datos de zonas. Comprobar sólo la forma dejaría
entrar `America/Mexico` —que no existe— y el fallo aparecería meses después, al
calcular una ventana de ejecución. La interfaz ofrece **exactamente** esa
lista, y el validador comprueba que las dos copias no divergen.

**3 · La preferencia de la barra vive en la base cifrada, no en
`localStorage`.** `localStorage` vive en el perfil de la WebView: se borra con
la caché del sistema, no entra en el respaldo `.arles` y en Windows depende del
directorio de WebView2. Una preferencia que se pierde al limpiar la caché no se
recuerda. El validador comprueba que no aparece `localStorage` en ninguna parte.

**4 · La empresa se valida también al leer.** Una fila editada por fuera con
una zona inexistente se nombra en vez de devolverse como buena. «Arreglarla»
aquí sería decidir por el usuario qué quiso escribir.

### La barra lateral (P-11)

Fija y plegable, a mano y sola, en iconos, recordada. El detalle está en
[UX_NAVEGACION.md §4.2](../05-diseno/UX_NAVEGACION.md). Lo que importa aquí:

- Hicieron falta **seis iconos nuevos**. `AIcono` tenía trece y ninguno era de
  sección.
- Plegada se quita el **texto**, no el **nombre accesible**.
- El plegado automático **no pisa** lo que eligió el usuario.

---

## 2. El umbral, y la sorpresa de medirlo

Documento completo: [UMBRAL_DE_PLEGADO.md](../06-calidad/UMBRAL_DE_PLEGADO.md).

Resumen honesto: **la primera medición no encontró nada.** Buscando
desbordamiento con la barra desplegada, no desbordó ninguna pantalla hasta 600
px — las de esta entrega son fluidas y se estrechan en vez de cortarse. Con ese
criterio, el plegado automático no tenía ninguna justificación, y redondear a
1000 px habría sido repetir exactamente cómo se escribió el `min-width: 1120`
que causó R-01.

Lo que sí se pierde antes es la **medida de diseño**. La lista de alta declara
78 ch y deja de alcanzarlos a **984 px** de ventana con la barra desplegada.
Plegar devuelve 176 px, que es justo lo que falta. Ese es el umbral.

`sonda:plegado` falla por los dos lados —umbral corto y umbral inflado—, y las
dos mitades se probaron rompiéndolas.

---

## 3. Cómo se comprobó

| | Qué cubre |
|---|---|
| **128 pruebas de Rust** | Validación campo a campo, lista de alta derivada, guardado idempotente, bitácora en la misma transacción, preferencias con clave cerrada, migración sobre base poblada |
| **49 pruebas de frontend** | P-11 escrito como pruebas: las dos formas de plegar, que el automático no pisa la preferencia, que sin núcleo no se finge un guardado |
| **`sonda:plegado`** | El umbral, por los dos lados, más los seis enlaces con nombre accesible estando plegados |
| **`sonda:ancho`** | Sigue vigilando lo de R-01: que a ninguna escala de Windows se corte nada |
| **19 comprobaciones de fase** | Ver abajo |

### Comprobaciones nuevas del validador, y qué atacan

| Comprobación | El fallo que impide |
|---|---|
| Las migraciones publicadas no cambian (sha256) | Editar una migración ya aplicada deja dos instalaciones con el mismo número de versión y esquemas distintos |
| Las listas cerradas del núcleo y del frontend coinciden | El desplegable ofrece una opción que el núcleo rechaza |
| Toda clave de error del núcleo tiene texto | Una clave sin texto enseña un mensaje genérico y se queda así para siempre |
| Cada sección tiene icono, y el mapa está tipado | Plegada, una sección sin icono es un hueco en blanco |
| Ninguna preferencia en `localStorage` | «Se recuerda» que no se recuerda |

**Las cuatro se probaron rompiéndolas** antes de darlas por buenas: quitando una
zona de la copia del frontend, metiendo un `localStorage`, añadiendo un espacio
a la migración `V2` y renombrando un icono de sección. Las cuatro fallaron.

---

## 4. Lo que quedó pendiente

| | Estado |
|---|---|
| **Revisión visual en macOS (R-07)** | Sigue abierta. Riesgo aceptado con fecha límite: antes de la Fase 6 |
| **R-02 · SmartScreen** | No apareció en el equipo del revisor. Se repite en el siguiente equipo Windows |
| **El umbral en macOS** | WKWebView aplica la misma CSS, pero tipografía y barras de desplazamiento mueven algunos píxeles |
| **El umbral volverá a moverse en la 3.2** | La tabla de contactos tendrá un mínimo real. `sonda:plegado` lo detectará y pedirá volver a medir |
| **Ninguna pantalla de la 3.1 se ha visto en una ventana nativa** | El instalador de revisión visual se compila desde `.github/revision-visual/PEDIDO.md`. Hasta entonces, lo comprobado es Chromium, que es el motor de WebView2 pero no la ventana |

---

## 5. Lo que esta entrega **no** hace

| | Por qué |
|---|---|
| Validar en el formulario además de en el núcleo | Dos reglas que mantener iguales. El día que divergen, el formulario aprueba lo que el núcleo rechaza |
| Ofrecer más países | v1.2.0 es despliegue interno en México (D-4), y los textos legales que la aplicación cita son la LFPDPPP. Una lista de doscientos países sugeriría un soporte legal que no existe |
| Bloquear pasos del alta entre sí | La lista sugiere un orden; no impone un embudo (UX_NAVEGACION §4) |
| Guardar el logotipo de la empresa | La columna existe en el esquema desde la V1. Subir un archivo abre la superficie de sistema de archivos, y eso se diseña entero en su fase, no de paso |
