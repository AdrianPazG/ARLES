# Hallazgos de diseño

**Qué es esto:** lo que se encuentra **mirando** la aplicación, no midiéndola.
**Última revisión:** 2026-10-28, las dieciocho pantallas a 1440×900.
**Dónde van los otros:** los defectos de funcionalidad en
[`REGISTRO-DE-REQUISITOS.md`](REGISTRO-DE-REQUISITOS.md); los que encuentra
Dirección usando ARLES en `06-calidad/BITACORA-DE-PRUEBAS.md`.

---

## 0 · Por qué este registro existe aparte

Las sondas miden contraste, foco, desbordes, textos encimados, tamaños de
objetivo y jerarquía de encabezados. **Las dieciocho pantallas pasan las seis
sondas.** Nada de eso caza una pantalla que dice algo falso, dos formatos del
mismo dato, o una rejilla que separa igual lo que va junto y lo que no.

La frase que lo resume está desde el 29/09/2026 en `CLAUDE.md`: **una pantalla
puede pasar todas sus comprobaciones y seguir estando mal.**

**Y la mitad del valor está en lo descartado.** Las auditorías B, C y D
aprendieron cada una por las malas que **un hallazgo falso cuesta más que uno
ausente**: manda a arreglar algo que funciona y gasta la credibilidad del resto.
Por eso el §3 de este archivo lista lo que se descartó, con por qué.

---

## 1 · Lo encontrado el 28/10/2026

Siete, ordenados por lo que le cuestan a quien usa ARLES.

| Id | Qué se ve | Dónde | Estado |
|---|---|---|---|
| **D-01** | Inicio dice que faltan por construir tres secciones que existen, y que no hay nada más que hacer — con **6 de 6** pasos hechos | `/inicio` | **Cerrado 28/10/2026** |
| **D-02** | El mismo número de WhatsApp se enseña de **dos formas**: `+52 81 1111 1111` y `+528100000000` | 3 pantallas | **Cerrado 28/10/2026** |
| **D-03** | El asistente aprieta sus campos **al doble** que el resto del producto: 8 px entre campos contra 16 px | `/campanas/nueva` | **Cerrado 28/10/2026** · y la causa era otra |
| **D-04** | «Continuar» sale apagado y **no dice por qué**, ni en pantalla ni para un lector | `/campanas/nueva` | **Cerrado 28/10/2026** · y había cuatro más |
| **D-05** | La columna «Correo» corta la dirección en **136 px** mientras «WhatsApp» tiene **179 px** y no los necesita | `/contactos` | **Descartado 28/10/2026** · medido |
| **D-06** | Dos botones de **1020 px** donde el resto del producto usa de 139 a 169 | `/remitentes/whatsapp` | **Cerrado 28/10/2026** |
| **D-07** | El título de la pantalla y el del primer módulo dicen los dos «Campañas» | `/campanas` | **Cerrado 28/10/2026** |

Y una observación que **no** es un defecto, anotada para que no se vuelva a
levantar: en Actividad, «Contestaron» cae sola en una segunda fila de la
rejilla de seis columnas. Es un salto de línea legítimo. Se deja dicho en el §2.

---

## 2 · Cada uno, con lo que se midió

### D-01 · Inicio dice que el producto no está terminado

**Lo que se ve.** La primera pantalla, con los seis pasos del alta en **6 de 6**:

> **Para poder enviar tu primera campaña** · 6 / 6
> Los siguientes pasos se construyen en las entregas indicadas. **No hay nada
> más que puedas hacer aquí todavía.**
>
> **Lo que falta por construir** · 1 paso
> Los módulos de campañas, canales y actividad aparecerán aquí cuando existan.

Campañas, Remitentes y Actividad **existen y tienen pantalla** desde la Fase 7.
Los seis pasos están hechos. Las dos frases son falsas, y están en la pantalla
que se abre primero.

**Por qué pasa, que no es lo que parece.** No es un texto que se quedó atrás —
eso también—, es una línea de `arles-core/src/onboarding.rs`:

```rust
disponible: paso.ruta().is_some(),
```

`disponible` se deriva de si el paso **tiene pantalla propia**. Y el paso de la
ventana de envío no la tiene, **a propósito y con razón**: la ventana se
configura dentro del asistente. Pero de ahí sale que:

- `siguienteAccionable` no puede ser nunca ese paso, así que cuando los otros
  cinco están hechos Inicio concluye que no queda nada que hacer;
- `porConstruir` vale **1 para siempre**, en cualquier instalación, así que el
  módulo «Lo que falta por construir» no se va nunca.

**Son dos preguntas distintas metidas en un campo:** «¿tiene pantalla propia?»
y «¿se puede hacer hoy?». La ventana contesta *no* a la primera y *sí* a la
segunda.

**Y hay una prueba que sujeta el defecto en su sitio.**
`disponible_y_ruta_no_pueden_divergir` afirma `p.disponible == p.ruta.is_some()`
para los seis pasos. Está bien escrita y verifica lo que dice; lo que dice es la
regla equivocada. Una comprobación puede estar verificada y garantizar lo que no
toca.

**Es el mismo defecto que este archivo ya documenta dos veces**, y las dos están
escritas en comentarios a tres líneas de la línea culpable: los accesos rápidos
que decían «Contactos · aún no» con la pantalla a un clic (29/09/2026), y
`ruta()` devolviendo `None` para pantallas construidas, en la 3.2 y otra vez en
la 6.4. **Tercera vez del mismo patrón en el mismo archivo.**

**Lo que cuesta.** Quien abre ARLES por primera vez lee, en la primera pantalla,
que el producto no está terminado. Es lo contrario de lo que el §9 pide de un
archivo que se lee al empezar.

#### Cerrado el 28/10/2026, y el arreglo no fue el que este registro proponía

El §4 de abajo decía: «separar `disponible` de `ruta()`: son dos preguntas».
**Lo correcto resultó ser lo contrario: juntarlas del todo.**

Separarlas habría creado un estado nuevo —un paso que se puede hacer y no lleva
a ningún sitio— que la interfaz tendría que pintar como un paso sin botón. Eso
es el H-06 del 22/10/2026, el defecto de dejar sin salida. Lo que de verdad
pasaba es más simple: **la ventana sí tiene a dónde llevar**, al paso 5 del
asistente, y `ruta()` llevaba seis semanas diciendo que no.

Así que `disponible` **se fue**: era `ruta.is_some()` dicho otra vez, y tener el
mismo dato con dos nombres es lo que invitó a leerlos como preguntas distintas.
Con un solo campo el estado inconsistente no existe — gana el tipo sobre el
validador, y el validador sobre la prueba.

Con eso, los tres textos falsos dejaron de ser alcanzables y **se borraron**:
«Lo que falta por construir», «Llega en la entrega {n}» y «No hay nada más que
puedas hacer aquí todavía». En su sitio, con los seis hechos, Inicio dice **«Ya
puedes enviar campañas · 6 de 6»**.

**Lo que se encontró arreglándolo**, y que el registro no había visto:

1. **Una cuarta copia de la misma lista**, en `stores/empresa.ts`: la reserva
   que se pinta sin núcleo detrás —`vite dev`, las sondas, las capturas de
   revisión— tenía **tres de seis filas con `ruta: null`**, con dos de esas
   pantallas construidas desde el 29/09. Nadie la comparaba con nada. El
   comentario de ese archivo avisaba del riesgo… de las zonas horarias, que sí
   se comparan. **Un aviso escrito no es una comprobación.**
2. **La misma cifra con dos formatos en el mismo archivo**: el bloque de primera
   vez de Inicio usa el catálogo y da «0 de 6»; el panel la armaba a mano y daba
   «6 / 6». Lo encontró escribir la prueba, no mirarla.
3. **El comentario del simulador nombraba un `comprobar_simulador.py`** que no
   ha existido nunca. La comparación sí existe, en la fase 3 del validador.

**Las cuatro rupturas**, las cuatro en rojo: devolver `None` en la ventana
(el defecto original), dejar `ruta: null` en el store, mandar el simulador a
otra pantalla, y devolver el módulo «Lo que falta por construir» a Inicio.

> **Y una quinta que encontró un defecto en mi propia comprobación.** La que
> compara el store con el núcleo recorría **sólo** lo que declara el núcleo, así
> que un paso que el núcleo deja sin ruta y el store sí lleva no se comparaba
> con nada: la clave no estaba en el bucle. Salió en verde con el núcleo roto a
> propósito, mientras la del simulador salía en rojo por lo mismo. Ahora compara
> las claves de los dos lados.
>
> Es exactamente el defecto que yo le había atribuido —por error— a la
> comprobación del simulador, que sí miraba las dos direcciones desde el
> principio. **Acusar al instrumento de al lado de lo que tiene el propio.**

---

### D-02 · El número de WhatsApp se enseña de dos formas

Medido en las tres pantallas que lo pintan:

| Pantalla | Cómo sale |
|---|---|
| `/contactos` | `+52 81 1111 1111` |
| `/remitentes/whatsapp` | `+528100000000` |
| `/contactos/supresion` | `+528112345678` |

**Dos de tres lo enseñan crudo.** Un número de doce dígitos seguidos no se
puede comparar de un vistazo con el que alguien tiene en el teléfono, y
comprobar que el número es el correcto es justo lo que se va a hacer en
Remitentes antes de mandar nada.

El formato con espacios **ya existe** y se usa en la lista de contactos. Lo que
falta es que las otras dos lo usen.

> **Esto ya mordió una vez, en la dirección contraria.** `CLAUDE.md` cuenta una
> sonda que buscaba `+521` en un texto donde el número se enseña con espacios:
> el patrón no podía coincidir **nunca**. Dos formatos del mismo dato no sólo se
> leen peor — rompen lo que los busca.

---

### D-03 · El asistente aprieta sus campos al doble

Medido, hueco vertical entre el final de un campo y la etiqueta del siguiente:

| Pantalla | Dentro de un campo | **Entre dos campos** |
|---|---|---|
| `/inicio/configuracion` | 4 px | **16 px** |
| `/ajustes/respaldos` | 4 px | **16 px** |
| `/campanas/nueva` | 4 px | **8 px** |

En el asistente, separar dos preguntas distintas ocupa el doble que separar una
etiqueta de su casilla. A esa distancia, «Sólo lo ves tú. No sale en el correo.»
—que explica el **nombre**— parece el encabezado de «Por dónde sale».

No es cuestión de gusto: es la regla de proximidad, y es la única señal que dice
qué va con qué cuando no hay líneas ni cajas. El asistente es además la pantalla
con más campos del producto.

---

### D-04 · «Continuar» apagado y sin decir por qué

Medido en el paso 1: el botón está `disabled`, **sin `title`, sin
`aria-describedby`, y sin ningún texto cerca** que lo explique. Para quien usa
un lector de pantalla no hay absolutamente nada.

Es la cuarta vez de este defecto en el proyecto, y las tres anteriores se
arreglaron por la misma razón (§95: un control que no responde y no se explica
se lee como una avería):

| Dónde | Cuándo |
|---|---|
| «Repite la contraseña», en Respaldos | 8.2c |
| «Restaurar y reemplazar», en Restaurar | 8.3c |
| «Continuar» del paso 6, al rechazar el desenlace | 8.6 E |

La del 8.6 E es la misma pantalla y el mismo botón, en otro paso: ahí el aviso
va **pegado encima del botón** y funciona. Falta en el paso 1.

---

### D-05 · La columna estrecha es la que lleva el dato largo

Medido en `/contactos`, con los anchos que reparte la rejilla:

| Columna | Ancho | ¿Se corta? |
|---|---|---|
| Nombre | 148 px | no |
| Empresa | 155 px | no |
| **Correo** | **136 px** | **sí** — `ana@ejemplo.mx` sale `ana@ejemplo…` |
| WhatsApp | 179 px | no — `+52 81 1111 1111` cabe con sitio de sobra |

La columna más estrecha de las cuatro lleva el dato más largo, y la segunda más
ancha lleva uno de longitud fija que siempre cabe.

**Cortar un dato está permitido** y está dicho en `CLAUDE.md`: la dirección
entera se lee al abrir la ficha. Lo que no tiene defensa es cortarlo **mientras
la columna de al lado sobra**: un correo es lo que se mira para saber si ese
contacto es el que se busca.

---

### D-06 · Dos botones de 1020 px

Medido, ancho de todos los botones visibles de cuatro pantallas:

| Pantalla | Botones |
|---|---|
| `/ajustes/restaurar` | 169 |
| `/ajustes/respaldos` | 157 |
| `/ajustes/archivar` | 147 |
| `/remitentes/whatsapp` | 139 · 166 · **1020** · **1020** |

Los dos de 1020 px son «Confirmo que está en Coexistencia» y «Entiendo el riesgo
y lo acepto»: ocupan el ancho entero de su aviso. Son además las **dos
decisiones más serias de esa pantalla** —una acepta que el número puede quedar
inhabilitado por Meta, y eso no se recupera—, así que su tamaño no es sólo una
rareza visual: un botón que ocupa toda una banda se pulsa sin apuntar.

Es la familia del defecto de la 8.3c, donde el botón de elegir medía 540 px y
los de abajo 200 y 96. Aquí la diferencia es de siete veces.

---

### D-07 · El título y el módulo dicen lo mismo

En `/campanas`, el `h1` dice «Campañas» y el primer `h2`, sesenta píxeles más
abajo, dice «Campañas». Medido en las cinco pantallas con módulos: es la única
donde coinciden.

Es la familia de «Con sus campañas · 3 campañas», que la 8.3 bis b arregló.

---

### La observación que NO es un defecto

En Actividad, la tarjeta de cada campaña reparte siete cifras en una rejilla de
seis columnas (x = 297, 481, 664, 848, 1032, 1215), así que **«Contestaron» cae
sola en una segunda fila**. Parece accidental, y además es la cifra que la
entradilla de la pantalla explica (D-7A: cuánta gente contestó, nunca qué
dijeron).

**No se anota como defecto** porque un salto de línea en una rejilla es correcto
y cualquier otro reparto tiene su propio precio. Queda dicho para que la próxima
revisión no lo levante como hallazgo nuevo.

---

## 3 · Lo que se descartó, y por qué

Tres cosas se vieron, se comprobaron y resultaron falsas. **Ninguna llegó al
registro**, y están aquí porque el descarte es parte del trabajo:

| Lo que creí ver | Lo que había |
|---|---|
| «Todavia no puede enviar», sin tilde, en la insignia de WhatsApp | El catálogo dice `'Todavía no puede enviar'`. La tilde está: a 12 px y en semibold no la leí en la captura |
| Restaurar no mete su contenido en una tarjeta, al revés que el resto | **Respaldos y Archivar tampoco.** Las tres pantallas de esa subsección usan una columna sin marco. Es su patrón, no una excepción |
| En `/recuperar`, tres huecos distintos de título a cuerpo (24, 40 y 64 px) | Medidos: **43 px los dos**. Me los inventé leyendo píxeles |

Las tres tienen la misma causa: **leer una captura en vez de medirla.** Las dos
últimas las descartó una medición de treinta segundos.

> **Y un cuarto, que no llegó ni a candidato.** Para buscar tildes perdidas pasé
> una lista de palabras sospechosas por el catálogo de textos y salieron unos
> sesenta resultados. **Todos falsos**: casaban las *claves* —`numero:`,
> `direccion:`, `configuracion:`—, que van sin tilde porque son
> identificadores de TypeScript, no texto que alguien lea.
>
> Es la quinta vez que hace falta esta lección en el proyecto; la cuarta fue la
> medición de argumentos de la auditoría G, que dio **130 hallazgos, todos
> falsos**, porque la expresión capturaba hasta el `return {` del final del
> archivo. **Medir una cadena no es medir lo que significa.**

---

## 4 · Qué comprobación haría imposible cada uno

El §1 bis obliga a preguntarlo, y la respuesta honesta no es «todas»:

| Id | ¿Se puede vigilar? | Cómo |
|---|---|---|
| D-01 | **Hecho** | `todo_paso_lleva_a_su_pantalla` en el núcleo y dos comprobaciones de fase que comparan las rutas del núcleo con las del simulador y las del store. Lo que esta fila proponía —separar `disponible` de `ruta()`— **era el arreglo equivocado**, y está explicado arriba |
| D-02 | **Sí** | Un solo formateador para el número, y una comprobación de que ninguna pantalla lo pinta crudo |
| D-03 | **Sí** | Una sonda que mida el hueco entre campos y exija el mismo token en todos los formularios |
| D-04 | **Sí** | Una sonda que exija que todo botón `disabled` tenga `title`, `aria-describedby` o un aviso al lado |
| D-05 | A medias | Se puede medir «una columna se corta mientras otra de la misma tabla sobra», pero el reparto bueno es un juicio |
| D-06 | **Sí** | Un máximo de ancho de botón, o que ninguno pase del doble del más ancho de su pantalla |
| D-07 | **Sí** | Comparar el `h1` con el primer `h2`, que es lo que ya hace la medición de este informe |

**Ninguna está escrita todavía.** Este tramo es el registro; escribirlas y
arreglar lo que encontraron es el siguiente, y se pregunta antes.

---

## 5 · Lo que esta revisión NO cubre

Obligatorio decirlo, por la guía de documentos:

- **Un solo tamaño de ventana.** 1440×900. Las sondas recorren cuatro, ésta no:
  mirar dieciocho pantallas a cuatro ventanas son setenta y dos capturas y el
  rendimiento decreciente llega antes.
- **Un solo tema.** El claro, que es con el que abre la demostración. El oscuro
  es el tema de origen de ARLES y **no se ha mirado en esta pasada**.
- **Ningún estado intermedio.** Cada pantalla se vio como la deja el simulador:
  sin errores en pantalla, sin listas vacías, sin cargas a medias, sin
  formularios a medio rellenar. Los cuatro estados del §97 **no** se revisaron.
- **Ni Windows ni macOS.** Esto es Chromium en Linux. Sigue abierto **R-07**:
  nadie ha abierto ARLES en WKWebView.
- **No se pulsó nada.** Es una revisión de lo que se ve al llegar, no del
  recorrido.

---

## 6 · Cómo se cerraron los seis · 28/10/2026

**Cinco arreglados y uno descartado**, y tres de los cinco resultaron ser otra
cosa de la que este registro había anotado. Vale la pena el detalle, porque el
patrón se repite: **mirar una pantalla dice que algo está mal; sólo medir dice
qué.**

### D-02 · Un solo formateador, y el diagnóstico estaba a medias

Este registro decía que «el formato con espacios ya existe y se usa en la lista
de contactos, lo que falta es que las otras dos lo usen». **Falso.** Contactos
no formatea nada: enseña `valorRaw`, que es **lo que el usuario tecleó**. Así
que ARLES no tenía una forma de presentar un número — tenía dos orígenes.

Ahora hay una, `numeroLegible()` en `app/src/app/formato.ts`, con su límite
escrito: no es un formateador internacional —eso es una biblioteca entera—, es
una ayuda de lectura con una regla para México y otra genérica. **Lo que no
reconoce lo devuelve intacto**, porque la lista de bajas pinta con ella tanto
correos como números.

Y medir encontró una pantalla más de las tres anotadas: **Plantillas** también
lo pintaba crudo. Cinco apariciones, no cuatro.

### D-03 · La causa no era el asistente

El asistente separaba sus campos con 8 px, cierto. Lo que este registro no
sabía es **por qué**: `AModulo` tiene desde el 16/10/2026 una regla escrita
para el **H-09 de Dirección** —que leyó el paso 5 y dijo que el hueco era «muy
reducido»— y su comentario explica el cálculo: *«16 + 8 = 24 px entre campos»*,
los 16 del `gap` del módulo más estos 8.

**Esos 16 no existen.** El cuerpo del módulo es un bloque, así que no tiene
`gap` ninguno. Durante doce días esa regla entregó **8 px**, no 24 — el doble
de los 4 de dentro de un campo, no seis veces.

Así que el defecto no era del asistente: era una corrección de Dirección que
nunca llegó a su valor. Hoy son los 24 que pedía, en un solo sitio y sin
depender de un hueco que no está.

> **Y el primer arreglo fue peor que el defecto.** Puse el cuerpo del módulo
> como columna flex con `gap`. Las tres sondas pasaron… y una columna flex
> **estira a sus hijos**: el botón «Ver qué se puede liberar» del Registro pasó
> a medir **1 086 px**, que es exactamente D-06 dos pantallas más allá. Lo cazó
> volver a medir los anchos después del cambio, no ninguna sonda.
>
> El segundo intento —un margen entre hermanos— tampoco hacía nada: el CSS con
> `scoped` **no alcanza al contenido del slot**, que pertenece al ámbito de la
> pantalla. Y el tercero, ya con `:deep`, seguía perdiendo contra la regla del
> H-09, que es más específica.
>
> Tres intentos, y los tres medidos. El que valía era cambiar el valor donde la
> decisión ya estaba tomada.

### D-04 · Había cuatro botones más

Al escribir la medida, **cinco** botones apagados en cuatro pantallas no decían
por qué: el «Continuar» del paso 1, el «Atrás» del mismo paso, «Apuntar la
baja» y los dos de Derechos.

Y uno de ellos enseña lo que la medida tiene que exigir: **Bajas sí tenía el
texto**, debajo del botón, desde que se escribió. Lo que no tenía era
`aria-describedby`, así que quien mira lo lee y quien usa un lector de pantalla
oye un botón apagado y nada más. Por eso la medida pide el enlace y no «un
texto cerca»: lo segundo se cumple a ojo y deja fuera a quien no ve.

### D-05 · Descartado, con la cifra

Los anchos de Contactos **están medidos**, y su comentario lo explica: lo que
no puede cortarse es el **rótulo**, no el dato, y «Correo» cede porque su
rótulo es corto y una dirección se lee entera al abrir la ficha.

Probado: dar a «Correo» 1.35fr y a «WhatsApp» 1.2fr **corta el rótulo
«WhatsApp» a 1024 px**, y la sonda lo dice con su cifra — *necesita 7 px más*.
No hay holgura.

Así que este hallazgo estaba mal planteado: miraba el dato y la restricción es
la cabecera. **Cuarto descarte de esta revisión.** Queda una mejora posible y
más cara —columnas que crezcan con la ventana, hoy los `fr` son fijos—, y no se
hace aquí.

### D-06 · El componente ya tenía el sitio

Los dos botones estaban en el cuerpo del aviso, que es una columna flex y los
estiraba. `AAviso` **ya tiene un slot `acciones`**, que es una fila y da el
ancho natural. No hacía falta tocar el componente: hacía falta usarlo.

> El primer intento sí lo tocó —`align-items: flex-start` en el cuerpo— y **no
> hacía nada**, porque el slot va dentro de otro contenedor. Revertido desde la
> copia: un comentario que describe un arreglo que no entrega es peor que no
> tener el comentario.

### D-07 · El módulo dice algo

«Campañas» dos veces pasa a «Lo que has creado», que además informa: lo que hay
dentro son las de esta empresa con su estado.

---

## 7 · Lo que vigila que no vuelvan

Dos medidas nuevas en `sonda:auditoria`, que recorre las dieciocho pantallas a
cuatro ventanas:

| Qué exige | Por qué ahí |
|---|---|
| **Todo botón apagado tiene `aria-describedby` o `title`** | El proyecto lo arregló cuatro veces a mano y nada lo vigilaba. Se exige el enlace y no un texto cerca, por lo del párrafo de D-04 |
| **Ningún texto visible lleva un número de corrido** | Se mide **lo que se ve**, no el código: el defecto es que el dato salga así, venga de donde venga |

Las dos **verificadas rompiéndolas**: devolviendo el número crudo a Remitentes
› WhatsApp y quitándole el `aria-describedby` al «Continuar» del asistente. Las
dos en rojo, nombrando la pantalla y el ancho.

> **Y las dos nacieron sin existir.** El script que las insertaba falló a mitad
> —después de preparar el texto y **antes de escribir el archivo**— y las
> primeras dos rupturas salieron en verde. La conclusión fácil habría sido «la
> medida no sirve»; la correcta, que no estaba. Se comprobó mirando el archivo.
>
> Es el §2 con una cara que no estaba en la lista: **una ruptura en verde
> significa que la comprobación no caza el defecto *o* que la comprobación no
> está.** Las dos se distinguen mirando, no deduciendo.

Más `formato.spec.ts`, con sus cuatro casos.

> **Uno de ellos nació siendo una prueba que no podía fallar**: comparaba
> `numeroLegible(x)` con `numeroLegible(x)` envuelto en un `replace`, o sea la
> función contra sí misma. Cazado antes de correrla; las cifras están ahora
> calculadas a mano.
