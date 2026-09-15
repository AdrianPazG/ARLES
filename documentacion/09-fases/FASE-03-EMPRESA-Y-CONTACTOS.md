# Fase 3 · Empresa y contactos

**Estado:** 🟡 en curso — **entrega 3.1 cerrada** · **Fecha:** 2026-09-15
**Validación:** 23/23 comprobaciones de la fase 3, 0 omitidas — `validar.py --fase 3`
**Pruebas:** 133 de Rust · 55 de frontend · 6 sondas de navegador
**Auditoría de funcionamiento:** 10 hallazgos, los 10 corregidos — ver §3 bis

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
| **133 pruebas de Rust** | Validación campo a campo, lista de alta derivada, guardado idempotente, bitácora en la misma transacción, preferencias con clave cerrada, migración sobre base poblada |
| **55 pruebas de frontend** | P-11 escrito como pruebas: las dos formas de plegar, que el automático no pisa la preferencia, que sin núcleo no se finge un guardado |
| **`sonda:plegado`** | El umbral, por los dos lados, más los seis enlaces con nombre accesible estando plegados |
| **`sonda:ancho`** | Sigue vigilando lo de R-01: que a ninguna escala de Windows se corte nada |
| **`empresa.rs`, integración** | Arranca la aplicación de verdad, configura, cierra y reabre. CI lo corre **con llavero** en los tres sistemas |
| **23 comprobaciones de fase** | Ver abajo |

### Comprobaciones nuevas del validador, y qué atacan

| Comprobación | El fallo que impide |
|---|---|
| Las migraciones publicadas no cambian (sha256) | Editar una migración ya aplicada deja dos instalaciones con el mismo número de versión y esquemas distintos |
| Las listas cerradas del núcleo y del frontend coinciden | El desplegable ofrece una opción que el núcleo rechaza |
| Toda clave de error del núcleo tiene texto | Una clave sin texto enseña un mensaje genérico y se queda así para siempre |
| Cada sección tiene icono, y el mapa está tipado | Plegada, una sección sin icono es un hueco en blanco |
| Ninguna preferencia en `localStorage` | «Se recuerda» que no se recuerda |

| Toda clave de Rust está en la lista de `errores.spec.ts` | Esa lista afirma vigilar lo anterior y está escrita a mano: si no se actualiza, la afirmación es falsa |

**Todas se probaron rompiéndolas** antes de darlas por buenas: quitando una
zona de la copia del frontend, metiendo un `localStorage`, añadiendo un espacio
a la migración `V2`, renombrando un icono de sección y renombrando una clave de
error. Todas fallaron.

---

## 3 bis. La auditoría del 15 de septiembre

Dirección pidió auditar el funcionamiento antes de instalar nada. Se recorrieron
las dos pantallas en un navegador, con un núcleo simulado que devuelve lo mismo
que el Rust real, haciendo lo que hace una persona: enviar el formulario vacío,
corregirlo, guardar, volver a editar, plegar la barra, recargar y estrechar la
ventana.

**Diez hallazgos. Los diez corregidos.**

### A-1 · La pantalla de Ajustes reventaba al mostrar el error del correo · **grave**

El texto decía «Revisa que el correo tenga la forma nombre@dominio.com». En la
gramática de `vue-i18n` una **arroba suelta abre un enlace a otra clave**, así
que el mensaje no compila: lanza `SyntaxError`, la función de render falla y
**la pantalla entera deja de pintarse**.

Lo peor era cómo fallaba: **en silencio**. El formulario se quedaba con lo
último pintado —sin campos marcados, sin mensaje, sin nada— y el error sólo
existía en la consola del navegador, que en una ventana de Tauri no ve nadie.
Quien escribiera mal su correo vería un botón que no hace nada.

Ningún test lo veía porque los textos se comprobaban **como datos** —que
estuvieran, que tuvieran tres partes— y nunca **como textos**: nadie los pasaba
por `t()`, que es lo que hace la interfaz.

Corregido escribiendo la arroba como `{'@'}`. Y con una prueba nueva que
compila **todos** los textos del catálogo: encontró de paso otro igual,
`error.email_invalido.como`, que estaba ahí desde la Fase 1 esperando a la
primera pantalla que lo mostrara.

### A-2 · Los textos de error se mostraban crudos

Arreglar A-1 abrió el siguiente: `resolverError` devolvía el texto **tal como
está en el archivo**, así que el usuario habría leído literalmente
`nombre{'@'}dominio.com`. Ahora devuelve el texto compilado, y una prueba
comprueba que ninguna sintaxis de `vue-i18n` llega a la pantalla.

### A-3 · La lista que vigila las claves de error no vigilaba nada

`errores.spec.ts` lleva una lista escrita a mano con el comentario «si alguien
añade una variante sin texto, este test falla». **Era falso**: la lista es
manual, y las dos variantes nuevas de la 3.1 no estaban en ella.

Es el mismo patrón que dejó obsoleto el `Some(1)` del test de arranque: un dato
duplicado a mano que nadie compara. Ahora el validador extrae las claves del
Rust y las compara contra el catálogo **y** contra esa lista.

### A-4 · «Configuración guardada» se quedaba puesto mientras se editaba

El aviso de éxito seguía en pantalla al empezar a cambiar los datos, afirmando
algo que había dejado de ser cierto.

El primer arreglo —vigilar el objeto del formulario— **lo rompió al revés**: al
guardar, la respuesta del núcleo rellena el formulario con los valores
normalizados, y eso es una escritura, así que el aviso desaparecía en el mismo
instante en que aparecía. Se descubrió porque la auditoría se volvió a pasar
entera después de corregir. La versión buena escucha el evento `input`, que
sólo dispara una persona escribiendo.

### A-5 · Los campos malos no se anunciaban ni recibían el foco

Marcar el campo en rojo no le sirve a quien no ve la pantalla, ni a quien acaba
de pulsar «Guardar» con el teclado y sigue con el foco en el botón. Ahora, al
fallar, el foco va al primer campo malo —donde el lector de pantalla anuncia su
etiqueta y su mensaje— y hay un aviso con `role="alert"` que resume que el
formulario tiene campos que corregir.

### A-6 · El catálogo desaparecía justo cuando hay que abrirlo

El enlace al catálogo del sistema se escondía con la barra plegada. Como la
barra se pliega sola a partir del 200 % de escala, el enlace **desaparecía
exactamente en la condición en la que hay que abrir el catálogo para
revisarlo** — la misma en la que apareció R-01. Ahora se queda, con icono.

### A-7 · Un comentario afirmaba un requisito de accesibilidad falso

El token de la barra plegada decía que 64 px dejan el objetivo «por encima de
los 40 px que pide WCAG 2.2 AA (2.5.8)». Falso por partida doble: el mínimo de
AA son **24 × 24** px, y los 44 son del 2.5.5, que es AAA. Medido de verdad: el
enlace plegado queda en 47 × 30 y el botón en 32 × 32, los dos por encima del
mínimo real. Corregido el comentario con los números medidos.

### A-8 · Los desplegables enseñaban identificadores, no nombres

El país se leía **«MX»** y la zona **«America/Mexico_City»**. Son los valores
que el núcleo exige, pero elegir tu ciudad no debería obligarte a saber qué es
un identificador IANA. Ahora se lee «México» y «Ciudad de México»; el valor que
viaja no cambia. El validador comprueba que **ninguna opción del núcleo se
queda sin nombre**, porque si falta se enseña el identificador crudo y nadie se
entera.

### A-9 · Los pasos que no se pueden hacer pesaban más que el que sí

«Llega en la entrega 5» iba en insignia de relleno sólido. En la pantalla, los
**cinco pasos imposibles gritaban más que el único accionable** — al revés de
lo que la lista tiene que decir. Ahora es una nota al margen, apagada. La
insignia queda para estados que hay que atender.

### A-10 · Inicio enseñaba una lista que no sabía si era cierta

Si la configuración no se podía leer —base corrupta, un dato editado por
fuera—, Inicio pintaba **la lista de reserva**: «0 de 6», todo pendiente, como
si fuera el estado real. Alguien con su empresa ya configurada habría vuelto a
configurarla.

Ahora, si no se pudo leer, **no se enseña la lista**: se enseña el error con
sus tres partes. Hay prueba de pantalla que lo comprueba, y se probó
rompiéndola.

*(A-8, A-9 y A-10 salieron de mirar las pantallas, no de recorrerlas: son
defectos de lo que la pantalla **comunica**, y eso no lo detecta ninguna
aserción sobre el estado interno.)*

### Y lo que la auditoría dejó construido

Faltaba una prueba que recorriera el camino entero. Cada capa estaba probada
por su lado —el dominio valida, la base guarda, el comando conecta— y **nada
las recorría juntas**: una frontera mal puesta entre dos capas probadas pasa
desapercibida porque cada test dice que su lado funciona.

`crates/arles-app/tests/empresa.rs` arranca la aplicación de verdad —llavero,
base cifrada, migraciones—, configura la empresa, comprueba que el alta avanza,
cierra, vuelve a abrir y comprueba que sigue todo ahí. Lo mismo con la barra
plegada, que es la mitad de P-11 que no se ve en una captura. Y CI lo corre
**con llavero** en los tres sistemas, porque sin llavero esos tests se saltan
solos.

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
