# El umbral de plegado de la barra lateral

**Entrega 3.1 · 15 de septiembre de 2026**
**Sonda:** `npm --prefix app run sonda:plegado`
**Resultado:** **984 px** de ancho de ventana

---

## Por qué hay un documento para un número

P-11 decidió que la barra lateral se pliega «a mano **y** sola». «Sola»
necesita un umbral, y un umbral elegido a ojo es exactamente lo que produjo el
defecto **R-01**: aquel `min-width: 1120px` salió de un razonamiento que
parecía sólido y no se comprobó nunca. Dirección lo encontró revisando a 200 %
de escala.

Así que este número se midió. Y lo primero que dio la medición fue una
sorpresa.

---

## La primera medición no encontró nada

La sonda buscó, como hace `sonda:ancho`, **el ancho al que la aplicación
empieza a desbordar** con la barra desplegada. Recorrió de 1400 px a 600 px, en
las dos pantallas de la 3.1 y también en el catálogo.

**No desbordó ninguna, en ningún ancho.**

No es un error de la sonda: es que las pantallas de esta entrega son fluidas.
El formulario y la lista de alta se estrechan, el texto se reparte en más
líneas, y nada se corta. El `min-width` que provocaba R-01 ya no está.

Conclusión incómoda y que conviene decir tal cual: **con el criterio del
desbordamiento, el plegado automático no tenía ninguna justificación.** Se
podría haber redondeado a 1000 px y nadie lo habría notado. Eso es precisamente
cómo se escribió el 1120 de R-01.

---

## Lo que sí se rompe, y es medible

Antes de cortarse, lo que se pierde es la **medida de diseño**: el ancho máximo
de lectura que cada pantalla declara.

| Pantalla | Medida declarada | En píxeles |
|---|---|---|
| Ajustes · configuración de empresa | 62 ch | 540 px |
| Inicio · lista de alta | 78 ch | 679 px |

Por debajo de cierto ancho de ventana, la pantalla ya no alcanza esa medida: el
contenido se aprieta. Y **plegar la barra devuelve exactamente lo que falta**,
porque pasa de 240 px a 64 px — 176 px recuperados.

De ahí sale el criterio, que es el que se comprueba en cada compilación:

> **El umbral es el ancho por debajo del cual la pantalla más exigente ya no
> cabe a su medida de diseño con la barra desplegada.**

---

## La medición

Barrido de 1400 px a 600 px, de 4 en 4, con el plegado automático desactivado
(`VITE_ARLES_UMBRAL_PLEGADO=1`; sin ese interruptor el propio umbral impide
llegar a los anchos donde se mide).

| Pantalla | Mantiene su medida hasta | Se aprieta a partir de |
|---|---|---|
| `/ajustes` | 844 px | 840 px |
| **`/inicio`** | **984 px** | 980 px |

Cuadra con la aritmética, que es la forma barata de saber que la sonda mide lo
que cree medir: 240 px de barra + 32 px de margen a cada lado + 679 px de
medida = **983**, y el barrido lo encontró en 984.

**El umbral es 984 px**, el de la pantalla más exigente. No se redondea: un
número redondo invita a preguntar de dónde salió.

### Una consecuencia que conviene tener presente

El mínimo de la ventana son **1120 px lógicos**, y el umbral son 984. Así que
**al 100 % de escala la barra no se pliega sola nunca**: la ventana no puede
llegar a ser tan estrecha. Según la tabla de abajo, el plegado automático
**empieza a actuar al 200 %** de escala de Windows en una pantalla de 1920 —a
175 % quedan 1097 px, todavía por encima del umbral—.

No es un defecto, es el orden correcto: a escala normal manda el botón, que es
lo que Dirección pidió; lo automático es la red debajo.

### Qué pasa por debajo

Plegada, la barra ocupa 64 px, así que la medida de Inicio se conserva hasta
**808 px**. Por debajo de eso la lista se aprieta aunque esté plegada, y no hay
nada más que plegar. En los anchos que produce Windows a escala alta:

| Escala | Viewport | Barra | ¿Cabe la medida? |
|---|---|---|---|
| 100 % | 1920 px | desplegada | sí |
| 125 % | 1536 px | desplegada | sí |
| 150 % | 1280 px | desplegada | sí |
| 175 % | 1097 px | desplegada | sí |
| 200 % | 960 px | **plegada sola** | sí |
| 250 % | 768 px | **plegada sola** | no — se aprieta 40 px |

El último caso se acepta: a 250 % de escala en una pantalla de 1920 quedan 768
px lógicos, y ninguna distribución de barra lateral devuelve los 679 px de
medida. Se aprieta, no se corta. **`sonda:ancho` sigue vigilando que no se
corte.**

---

## Cómo se comprueba que el número sigue siendo cierto

`sonda:plegado` falla **por los dos lados**, y las dos mitades hacen falta:

| Si el umbral… | La sonda dice |
|---|---|
| se queda **corto** (800 px) | «está por debajo del suelo medido (984 px): entre los dos la barra sigue desplegada y la pantalla no alcanza su medida» |
| se **infla** (1200 px) | «excede el suelo medido en más de 40 px: la barra se plegaría sola en anchos donde la pantalla cabe holgada» |

Sin la segunda mitad, el umbral se podría subir a 4000 px: la barra quedaría
plegada siempre y la comprobación seguiría en verde — lo contrario de lo que
P-11 decidió. **Las dos se probaron rompiéndolas**, con esos dos valores, antes
de dar la sonda por buena.

La sonda comprueba además, en los anchos de escala alta y con la compilación
normal, que la barra se pliega sola, que no desborda, que se ven los seis
enlaces y que **ninguno se queda sin nombre accesible** — que es lo que se
pierde al quitarles el texto si se hace con `display: none`.

La holgura admitida es de **40 px**: menos de los 176 que ahorra plegar, así
que un umbral inflado no cabe dentro de ella, y suficiente para que un ajuste
tipográfico de unos píxeles no obligue a retocar el número cada semana.

---

## Lo que este número tiene de provisional

**El umbral es de la pantalla más ancha que exista, y esa pantalla va a
cambiar.** La entrega 3.2 trae la tabla de contactos, que sí tendrá columnas de
ancho fijo y un mínimo real — probablemente mayor que 679 px, y con
desbordamiento de verdad cuando no quepa.

No hay que acordarse: **la sonda lo detecta**. Si la tabla sube el suelo por
encima de 984 + 40, falla pidiendo que se suba el umbral. Lo único que hay que
hacer entonces es medir otra vez y actualizar el número aquí y en
`app/src/app/stores/interfaz.ts`.

---

## Lo que no se midió

| | Por qué |
|---|---|
| El umbral en **macOS** | R-07 sigue abierto: no hay equipo. WKWebView aplica la misma CSS, pero la tipografía del sistema y las barras de desplazamiento cambian algunos píxeles. Se revisará con el resto de la revisión de Mac |
| Si el umbral **le parece bien a alguien que lo usa** | Es una pregunta de uso, no de medida. Va en la próxima revisión visual con Dirección |
