# Auditoría D · UX y accesibilidad (§140)

**Fecha:** 07/10/2026 · **Entrega:** 8.6 D · **Estado al empezar:** las diez
fases del validador en verde.

Cuarta y última de las pasadas separadas. `ESTRATEGIA_QA.md` §5 le fija ocho
puntos, y **esta vez se podían medir de verdad**: el contenedor tiene Chromium,
así que no se auditó leyendo código sino abriendo las pantallas y midiendo.

**Límite declarado:** nada de esto se ha visto en **WKWebView**. La vista previa
corre sobre Chromium, que es el motor de WebView2 en Windows pero no el de
macOS. Eso es **R-07** y sigue abierto.

---

## 0 · El resultado, en una tabla

| | Punto del §5 | Veredicto |
|---|---|---|
| 1 | Contraste AA calculado en todos los pares | ❌ **se medía en 13 pantallas de 18** |
| 2 | Foco visible siempre | ❌ **igual: 13 de 18** · y en las 18 pasa (163 paradas) |
| 3 | Navegación completa por teclado | ✅ limpio |
| 4 | Ningún estado sólo por color | ❌ **no lo medía nada** · ahora sí, y pasa |
| 5 | `prefers-reduced-motion` | ✅ ya vigilado desde la Fase 2 |
| 6 | Objetivos ≥32 px al 200 % | ❌ **no se medía ni el tamaño ni el 200 %** · cinco enlaces en rojo |
| 7 | Los cuatro estados en cada pantalla (§97) | ⚠️ **la regla no se puede cumplir como está escrita** |
| 8 | Errores con las tres partes (§95) | ✅ limpio, y bien vigilado |

**Cuatro hallazgos, los cuatro arreglados.** Y tres cosas que estuve a punto de
anotar y no eran defectos.

---

## 1 · Los hallazgos

### D-1 · Cinco pantallas nunca se habían medido, y el aviso llevaba escrito ocho días

**Qué.** `auditoria.mjs` y `recorrido.mjs` son las dos sondas que recorren el
producto entero: la primera mide contraste real, foco, desbordes, nombres
accesibles, jerarquía de encabezados e identificadores repetidos; la segunda
busca callejones sin salida. Las dos llevaban **su propia lista de rutas escrita
a mano**, y las dos se habían quedado en las **trece** pantallas que existían el
29/09/2026.

El producto tiene **dieciocho**. Nunca se midieron:

- `/actividad/registro` — el registro con sus filtros (7.4)
- `/ajustes/respaldos` (8.2c)
- `/ajustes/restaurar` (8.3c)
- `/ajustes/archivar` (8.3 bis b)
- `/recuperar` (8.4)

Las cuatro últimas son **todo lo que construyó la Fase 8**.

**Y lo peor es que el aviso estaba escrito, en ese mismo archivo.** Lo puso la
revisión del 29/09/2026 el día que le añadió tres rutas que faltaban:

> *«Una auditoría que no recorre una pantalla no dice nada de esa pantalla,
> aunque termine en verde — que es peor que no tenerla, porque se cree.»*

Se escribió la lección y **volvió a pasar cinco veces**, porque la lección iba
en un comentario y la lista seguía siendo a mano. Una lista no se mantiene por
saber que hay que mantenerla.

Es el mismo defecto que la auditoría A encontró en el grafo de crates y la C en
los casos frontera. **La única defensa que ha funcionado las tres veces es leer
del código.**

**Arreglo.** `app/pruebas/sondas/rutas.mjs` lee las rutas del router. Las dos
sondas la usan, y `recorrido.mjs` —que necesita datos por pantalla— **revienta
si al router le sobra una fila**, para que añadir una pantalla obligue a decidir
qué se puede hacer en ella. Lo vigila
`las_sondas_recorren_todas_las_pantallas` (fase 8).

**Y la buena noticia, que también hay que decirla:** medidas las dieciocho, las
cinco pantallas **pasan**. Contraste, foco, desbordes, nombres, encabezados e
identificadores, en cuatro ventanas. El hueco estaba en la medición, no en las
pantallas.

---

### D-2 · Una pantalla sin landmark, y dos de las siete medidas la saltaban en silencio

**Qué.** `/recuperar` es la única pantalla que no pasa por el armazón de
`App.vue` —sin base de datos no hay secciones a las que ir, así que no lleva
barra lateral—, y su plantilla abría con un `<div>`.

**Dos consecuencias, y la segunda es la que lo encontró.**

La de accesibilidad: la pantalla se quedaba **sin ningún landmark**. Quien usa
un lector de pantalla no tiene «ir al contenido principal» justo el día en que
algo se ha roto y hay que leer con atención.

La de medición: **dos de las siete medidas de la sonda se acotan a `main`** —el
contraste y la jerarquía de encabezados—, así que en esa pantalla **medían cero
elementos** mientras la sonda la contaba como medida. La primera corrida sobre
las dieciocho la señaló diciendo «la pantalla no tiene ningún encabezado», y la
pantalla **tiene un `h1`** desde que se escribió.

**Arreglo.** Un `<main>`, con el motivo escrito al lado.

> **Y aquí estuve a punto de anotar un defecto que no existe.** El primer
> impulso fue apuntar «la pantalla de recuperación no tiene `h1`», que es lo
> que decía la sonda. Lo tiene. Lo que no tenía era el landmark que la medida
> necesitaba para verlo.

---

### D-3 · Cinco enlaces de 20 px, y el 200 % no se había probado nunca

**Qué.** `ESTRATEGIA_QA.md` §5 pide objetivos de **32 px a escalado 200 %**
desde la Fase 0. No se medía **ninguna de las dos cosas**: ni el tamaño de los
objetivos, ni ese escalado.

Medidos, salieron cinco enlaces de navegación de **20 px de alto** —el alto de
su línea de texto, sin un píxel de relleno—:

| Pantalla | Enlace | Medida |
|---|---|---|
| Inicio | «Ver los 6 pasos» | 97×20 |
| Actividad | «Pausar o detener una campaña» | 226×20 |
| Actividad | «Las cuentas remitentes» | 164×20 |
| Actividad | «Quién pidió no recibir» | 152×20 |
| Registro | «Ver cómo va ahora» | 134×20 |

Tres de los cinco son **las salidas que la 7.3 añadió** a Actividad justo porque
la pantalla no tenía ninguna. Se añadieron, y se quedaron a dos tercios del
tamaño que el producto exige.

**A ojo un enlace corto no se ve pequeño**, que es exactamente por lo que hacía
falta medirlo.

**Y «32 px al 200 %» se puede leer mal.** El escalado del sistema operativo
**no encoge los píxeles CSS**: encoge la ventana medida en píxeles CSS. Un
portátil de 1920×1080 al 200 % le da al navegador **960×540**. Eso es lo que hay
que montar, y el alto importa tanto como el ancho. La sonda prueba ahora cuatro
ventanas, con ésa dentro.

**Arreglo.** Un token, `--arles-objetivo-minimo: 32px`, con su motivo escrito —
hasta hoy la regla **no vivía en ningún sitio**— aplicado a los cinco enlaces.

El suelo de ARLES (32) es más alto que el de WCAG 2.2 (24, del 2.5.8), y es una
decisión del §22: el público objetivo no tiene ratones de precisión.

---

### D-4 · Que ningún estado se distinga sólo por color no lo medía nada

**Qué.** Es el punto del §5 que no se puede mirar a ojo —quien mira ve los
colores— y no había ninguna comprobación. Una insignia que distingue «aceptado»
de «rebotó» **sólo** por el color de su fondo no dice nada a quien no distingue
esos dos colores, y en ARLES los estados de envío son justo lo que el usuario
mira para decidir si una campaña va bien.

**Arreglo.** Una medida más en la sonda: cada insignia tiene que llevar **texto**
o nombre accesible. Pasa en las dieciocho pantallas.

**No se exige icono, y es deliberado.** La revisión del 29/09/2026 encontró lo
contrario —ocho insignias con un reloj que no esperaba nada— y la lección fue
que un icono de adorno estorba. El texto es lo que no se puede confundir, y la
comprobación caza igual la insignia que lleva icono y no texto, que era el caso
dudoso.

---

## 2 · El punto que no se puede cumplir como está escrito

### §97 · «los cuatro estados en cada pantalla»

La regla pide cargando, vacío, error y éxito **en cada pantalla**. Medido, eso
no se puede cumplir y no debería:

- **Seis de las dieciocho no tienen ninguna colección**: Ajustes, Configuración,
  Derechos, Respaldos, Restaurar y Recuperar son formularios o pantallas de
  acción. Un «estado vacío» en un formulario no significa nada.
- **Ajustes no enseña errores, y es correcto.** Guardar el tema puede fallar y
  el fallo **se traga a propósito**, con el motivo escrito en el store desde la
  Fase 3: la pantalla ya cambió de color, y *«que no se recuerde al reabrir es
  peor que nada, pero mucho menos que un diálogo de error por haber elegido un
  color»*.

**Lo que sí es verificable, y se verificó:** toda pantalla que pinta una
colección **variable** tiene su estado vacío. Las nueve que la tienen —Actividad,
Campañas, Contactos, Importar, Plantillas, Registro, Remitentes, Bajas y
WhatsApp— lo tienen. Las cinco que recorren una lista **fija o posterior a una
acción** —Inicio con sus seis pasos, Configuración con sus campos, Archivar y
Restaurar con las cifras de su preflight, Respaldos con sus alcances— no la
necesitan.

Y diecisiete de dieciocho enseñan un error cuando lo hay.

**Queda declarado y no arreglado**, porque lo que haría falta es **reescribir la
regla**, no el producto: «los cuatro estados donde cada uno significa algo» es
lo que se cumple, y es lo que debería decir. Es una decisión sobre un documento
de la Fase 0, y va a la lista de la 8.7.

---

## 3 · Tres cosas que estuve a punto de anotar, y no eran defectos

Esto va escrito porque las pasadas B y C cayeron las dos en un hallazgo falso, y
**un hallazgo falso cuesta más que uno ausente**: manda a arreglar algo que
funciona y gasta la credibilidad del resto de la lista.

| Lo que parecía | Lo que era |
|---|---|
| «La pantalla de recuperación no tiene `h1`» | Lo tiene. Lo que faltaba era el `<main>` que la medida necesita (D-2) |
| «Ajustes no enseña errores» | Correcto a propósito, con su motivo escrito en el store |
| «Cinco pantallas pintan listas sin estado vacío» | Las cinco recorren datos fijos o posteriores a una acción. Ninguna puede estar vacía |

Las tres se descartaron **mirando el código entero** en vez de la cifra que
salía de un `grep`. Es la misma lección que la C escribió sobre citar media
frase de un comentario.

---

## 4 · Y un defecto de mi propia comprobación, el sexto de la serie

La comprobación nueva —`las_sondas_recorren_todas_las_pantallas`— tuvo **dos
defectos, los dos encontrados rompiéndola**:

1. **Su primera versión contaba rutas** y exigía como mucho una por archivo.
   Salió en rojo señalando `recorrido.mjs`, que **es correcto**: lleva una tabla
   de cuántas acciones mínimas ofrece cada pantalla, legítimamente indexada por
   ruta. Contar rutas era medir lo que se ve, no la propiedad que importa — que
   es **quién decide qué pantallas se recorren**.

2. **Y se cazaba a sí misma en un comentario.** La ruptura de «quitarle el
   mínimo a `rutas.mjs`» no la cazó: `MINIMO` sigue nombrado en la cabecera del
   módulo, que explica por qué existe. **Sexta vez en este proyecto** (5.3, 8.5,
   dos en la 8.3 bis, la 8.6 A, y ésta), y la primera **dentro de una
   comprobación escrita el mismo día para arreglar otra igual**.

Verificada al final con **cuatro rupturas**, las cuatro vistas en rojo: una
lista a mano devuelta a la sonda, `rutas.mjs` borrado, su guarda del mínimo
quitada entera, y el `throw` del recorrido fuera.

> **Y una ruptura mal elegida también enseña algo.** La tercera se intentó
> primero renombrando sólo la declaración de `MINIMO`, y no cazó nada: el
> identificador seguía en sus usos. Una ruptura que deja el síntoma en pie no
> prueba nada; hay que quitar la garantía, no su nombre.

---

## 5 · Lo que salió limpio, con lo que se midió

- **163 paradas del tabulador**, en las dieciocho pantallas, **todas con marca de
  foco visible**.
- **Contraste real** —el color que el navegador pinta, no el token— en las
  dieciocho pantallas y las cuatro ventanas, incluido el suelo propio de 3.0
  para controles deshabilitados.
- **Ningún desborde, ningún botón sin nombre accesible, ningún identificador
  repetido** en las dieciocho.
- **La jerarquía de encabezados** empieza en `h1` y no da saltos en las
  dieciocho — eran quince hasta hoy, y tres no tenían `h1` hasta la 7.7.
- **El indicador de sección** cae exactamente sobre la sección activa y viaja
  con ella.
- **`prefers-reduced-motion`**, vigilado desde la Fase 2 con su trampa cerrada
  (un `prefers-reduced-motionXX` mal escrito).
- **Los errores con sus tres partes (§95)**: la prueba recorre `es.error`
  **entero**, no una lista a mano, y exige que ninguna de las tres esté vacía.
  303 textos.
- **El recorrido sin callejones sin salida** en las dieciocho, con su mínimo de
  acciones declarado una por una.

---

## 6 · Qué queda de la serie de auditorías

**Las cuatro pasadas del §140 están hechas.** El recuento:

| Pasada | Hallazgos | Arreglados | Declarados |
|---|---|---|---|
| A · Arquitectura | 5 | 5 | — |
| B · Seguridad | 2 + 3 puntos del documento | 5 | — |
| C · QA y rendimiento | 6 | 4 | 2 |
| **D · UX y accesibilidad** | **4** | **4** | — |

**Diecisiete hallazgos. Quince arreglados, dos declarados con su cifra.**

Y un patrón que atraviesa las cuatro: **de los diecisiete, once no eran defectos
del producto sino de sus comprobaciones** — una lista a mano, un alcance
demasiado corto, un comentario que se cazaba a sí mismo. El producto aguantó
mejor que sus instrumentos.

**Lo que queda abierto:**

- **C-1 · el tope de activación**, esperando a Dirección, con sus cuatro
  alternativas y una recomendación.
- **C-2 · el filtro del registro** a los ~135 000 sucesos, declarado.
- **§97 · reescribir la regla** de los cuatro estados para que diga lo que se
  puede cumplir. Va a la 8.7.
- **R-07 · WKWebView.** Nada de esta pasada se ha visto en macOS.
