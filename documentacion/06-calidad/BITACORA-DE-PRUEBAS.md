# Bitácora de pruebas de funcionamiento

**Abierta el 14 de octubre de 2026**, el día que Dirección abrió ARLES por
primera vez como aplicación de verdad.

Aquí entra **todo defecto encontrado usando ARLES**: qué pasó, qué debería pasar,
cuánto estorba y en qué estado está. Un hallazgo no se arregla hasta que está
escrito aquí — así ninguno se pierde entre dos mensajes.

> **Esto no es el [registro de tropiezos](../07-entrega/REGISTRO-DE-TROPIEZOS.md).**
> Aquél es para **Dirección**: «me ha salido esto, qué hago». Éste es para
> **corregir el producto**: «esto está mal y hay que arreglarlo».

---

## El método de trabajo

Acordado el 14/10/2026. **Hay dos vías, y la decide una sola pregunta:**

### ¿Puedes seguir probando?

| | | Qué haces | Qué hago |
|---|---|---|---|
| **NO** · te bloquea | 🔴 | **Dímelo al momento** | Lo arreglo y publico un instalador nuevo. Paramos la prueba hasta tenerlo |
| **SÍ** · es feo, confuso o está mal escrito | 🟡 | **Sigue probando** y acumula | Los registro todos. Los arreglo **en tanda** cuando me digas «corrige» |

### Por qué así, y no de uno en uno

**Cada instalador tarda unos veinte minutos en compilarse.** Arreglar de uno en
uno significa veinte minutos de espera por cada defecto, y te rompe el ritmo de
la prueba justo cuando estás dentro del flujo.

En tanda es **un solo ciclo** para diez defectos. Pero un bloqueante no puede
esperar, porque sin él no hay prueba que seguir.

### Cómo me das un hallazgo

No hace falta que sea formal. Con esto me basta:

1. **En qué pantalla estabas**
2. **Qué hiciste** (el botón que pulsaste, lo que escribiste)
3. **Qué pasó**
4. **Qué esperabas que pasara**
5. **Una captura**, si se ve

Si falta algo te lo pregunto. **Nunca des por supuesto que es poco importante** —
en este proyecto, siete de cada diez defectos graves los ha encontrado alguien
mirando, no una comprobación automática.

### Qué pasa después

- Lo registro aquí con un número (**H-01**, **H-02**…) y su gravedad.
- Cuando lo arreglo, escribo **qué lo causaba** y la comprobación que impide que
  vuelva — porque en este proyecto la regla es que **una comprobación que nunca
  se ha visto fallar no está verificada** (§2).
- La fila queda aquí para siempre, con su estado.

---

## Gravedad

| | Qué significa |
|---|---|
| 🔴 **Bloqueante** | No se puede seguir. O hace perder datos. Se arregla ya |
| 🟠 **Grave** | Se puede seguir, pero el resultado es incorrecto o engañoso |
| 🟡 **Molesto** | Funciona, pero confunde, se ve mal o está mal escrito |
| 🔵 **Propuesta** | No está mal; podría estar mejor |

---

## Estado

| | |
|---|---|
| **Abierto** | Registrado, sin arreglar |
| **En curso** | Lo estoy arreglando |
| **Arreglado** | Corregido y verificado, con la versión en que entra |
| **Declarado** | No se arregla ahora, y aquí está el motivo y su cifra |
| **No era un defecto** | Se comprobó y el comportamiento es correcto. **Se queda escrito**, con por qué |

---

# Hallazgos

| # | Gravedad | Pantalla | Qué pasa | Estado |
|---|---|---|---|---|
| **H-01** | 🟠 Grave | Remitentes | Dice «todavía no has configurado los datos de tu empresa» **después** de haberla configurado | ✅ **Arreglado** 14/10/2026 |
| **H-02** | 🟡 Molesto | Todas las que avisan | Las tres partes de un error salen **pegadas**: «…con el servidor.El servidor no ofrece…» | ✅ **Arreglado** 14/10/2026 |
| **H-03** | 🔴 Bloqueante | Remitentes | **No se puede dar de alta una cuenta de Gmail.** «No se pudo cifrar la conexión con el servidor» | 🔎 **Investigando** · faltan datos |
| **R-01** | 🔵 Requisito | Remitentes | Dar de alta un correo es **demasiado complicado**. Tiene que ser sencillo e intuitivo | ✅ **Hecho** 14/10/2026 |
| **R-02** | 🔵 Requisito | WhatsApp | Lo mismo con el alta de un número | ✅ **Hecho** 14/10/2026 · con un límite declarado |
| **A-01** | 🟡 Molesto | Contactos | La cabecera de la tabla se leía **«Correo princip…»** y **«WhatsApp pri…»**: no cabían en su columna | ✅ **Arreglado** 14/10/2026 |
| **A-05** | 🟠 Grave | Remitentes | La contraseña de aplicación de Gmail se mandaba **con los espacios** con que Google la enseña, y **nada avisaba** | ✅ **Arreglado** 15/10/2026 |

> Los hallazgos con **A** los encontré yo auditando, no Dirección probando. Van
> en la misma tabla a propósito: separarlos en dos listas haría que una de las
> dos se mirara menos, y un defecto no es más leve por haberlo encontrado quien
> lo escribió.

---

## H-01 · Un aviso que sobrevive a la recarga que lo resuelve

**Encontrado por Dirección el 14/10/2026**, usando ARLES en Windows.

### Qué pasó

1. Entró en **Remitentes antes** de configurar la empresa. La pantalla avisó:
   *«Todavía no has configurado los datos de tu empresa. Ve a Ajustes, completa
   los datos de la empresa y vuelve aquí. No se modificó nada.»* **Correcto.**
2. Fue a **Configuración** y guardó los datos de TELEMETRY. La pantalla confirmó
   *«Configuración guardada»*. **Correcto.**
3. Volvió a **Remitentes**. La carga **funcionó**… y el aviso viejo **seguía
   pintado**.

### Por qué importa más de lo que parece

La pantalla **afirmaba algo falso sobre el estado de su propia base de datos**, y
eso hizo dudar de si el guardado había servido. Es el **§95 al revés**: un error
que ya no existe sigue dando instrucciones, y las instrucciones mandan a repetir
algo ya hecho.

### La causa

Una línea. `cargar()` escribe el aviso en su `catch` y **nunca lo limpia al
empezar**, así que el error de un intento fallido sobrevive al intento que lo
resuelve.

### Y no era de Remitentes

| Store | Estado antes | Qué se hizo |
|---|---|---|
| `remitentes` | **El defecto** | Limpia antes del `try` |
| `empresa` | **El mismo defecto** | Limpia antes del `try` |
| `whatsapp` | Limpiaba, pero **pasada su guarda de núcleo** | Se movió, para que el patrón sea uno |
| `interfaz` | Sin defecto — su `catch` está vacío **a propósito** | Nada |
| `recuperacion` | Sin defecto — deja `quePaso` en nulo **a propósito** | Nada |

> **Los dos últimos salieron de un detector por heurística y son falsos
> positivos.** Se descartaron **leyendo el código**, no la cifra: una preferencia
> de interfaz ilegible no impide usar ARLES, y la pantalla de recuperación sigue
> siendo útil sin sus metadatos. Anotarlos habría mandado a arreglar algo que
> funciona — que en este proyecto cuesta más que un defecto ausente.

### Qué impide que vuelva

- **`avisos-al-recargar.spec.ts`** · tres pruebas, **vistas en rojo antes del
  arreglo**.
- **`ningun_aviso_sobrevive_a_la_recarga_que_lo_resuelve`** en `--fase 7` ·
  recorre **todos** los stores que haya, ahora y los que vengan, porque un store
  nuevo no estaría en la prueba y este patrón ya se escribió tres veces sin
  querer. Verificada con tres rupturas: quitando el limpiado, dejándolo sólo en
  un comentario, y poniéndolo **después** del `try` — que es el error sutil.

### Entra en

La siguiente versión descargable.

---

## Lo que ya sabemos que falta, y no hace falta reportar

Esto **no son defectos**: son cosas no construidas, ya declaradas. Si te las
encuentras, no hace falta que las anotes.

| Qué | Por qué |
|---|---|
| Una campaña entera por WhatsApp | El motor sólo recorre correo. De WhatsApp sale el mensaje de prueba |
| Filtrar la audiencia de una campaña | No está construido |
| Editar una campaña ya creada | No está construido |
| La letra no es Mont | Las versiones descargables van sin ella mientras P-01 siga abierto |
| Windows dice «editor desconocido» | Falta el certificado de firma (Fase 9) |
| Activar una campaña grande congela la ventana | Medido: 73,5 s con 451 196 destinatarios; se nota desde unos 30 000. Está en la Fase 9 |
| Buscar texto en el registro tarda | Medido: 3 709 ms. Declarado como C-2 |

---

## H-02 · Las tres partes de un error salen pegadas

**Encontrado por Dirección el 14/10/2026**, al fallar una conexión SMTP.

### Qué se leía

> **No se pudo cifrar la conexión con el servidor.**El servidor no ofrece
> cifrado, o su certificado no corresponde al nombre que escribiste. Revisa el
> nombre y el puerto. ARLES no envía por una conexión sin cifrar y no hay forma
> de desactivarlo: la contraseña viajaría legible por la red.La contraseña no
> llegó a enviarse: se cortó antes. No se envió ningún correo y no se guardó
> nada.

Fíjate en `servidor.El` y en `red.La`: **sin un espacio**.

### La causa

El §95 exige que todo error diga tres cosas, y las pantallas las escriben como
tres elementos:

```html
<AAviso>
  <strong>{{ que }}</strong><span>{{ como }}</span><span>{{ salvo }}</span>
</AAviso>
```

`<strong>` y `<span>` son elementos **en línea**. El contenedor del slot en
`AAviso` no los separaba, así que el navegador los pega.

**El texto era correcto.** Lo que fallaba era la maquetación — y daba impresión
de producto sin terminar **justo en el momento en que algo acaba de fallar**,
que es cuando más se lee.

### Alcance

**Cinco avisos en tres pantallas** (Remitentes, Supresión, Derechos). El arreglo
va en el componente compartido, así que el siguiente aviso que se escriba hereda
la separación sin que nadie tenga que acordarse.

### Lo que esto enseñó sobre nuestras sondas

**Se intentó medirlo con la sonda de auditoría**, que recorre las dieciocho
pantallas en Chromium y mide píxeles. Parecía su sitio.

**Con el defecto puesto a propósito, la sonda no encontró nada.** La razón
importa: en la demostración **los avisos de error no aparecen** —el simulador no
falla— y el catálogo del design system está fuera de las rutas que se recorren.
Así que esa medida nunca se habría ejercitado.

Se retiró de la sonda **en vez de dejarla dando falsa seguridad**, y en su lugar
hay una comprobación en `--fase 2` que inspecciona el CSS. Es menos fiel que
medir, pero **se puede romper y ver en rojo** — y eso es lo que decide si una
comprobación sirve (§2).

> **Queda anotado como límite conocido:** las sondas no pueden ver ningún estado
> de error del producto, porque la demostración nunca falla. Todo lo que sólo se
> ve cuando algo va mal está fuera de su alcance.

### Qué impide que vuelva

`las_tres_partes_de_un_error_no_salen_pegadas` en `--fase 2`. **Verificada
viéndola en rojo con el defecto real** —el mismo que vio Dirección— y con una
segunda ruptura: quitando sólo la separación, porque dos párrafos seguidos sin
hueco se siguen leyendo como uno.

---

## H-03 · No se pudo dar de alta la cuenta de Gmail

**Encontrado por Dirección el 14/10/2026.** 🔴 **Bloqueante**: sin cuenta
remitente no hay envío de prueba, y sin envío de prueba no se cierra **R-24**.

### Qué dijo ARLES

> No se pudo cifrar la conexión con el servidor. El servidor no ofrece cifrado,
> o su certificado no corresponde al nombre que escribiste. Revisa el nombre y
> el puerto.

### Lo que se ve en la captura

| Campo | Valor |
|---|---|
| Dirección desde la que se envía | `pruebas.telemetry@gmail.com` |
| Nombre que acompaña | `Telemetry Insight` |
| **Usuario** | **`Pruebas_1`** |
| Contraseña | (ocho caracteres ocultos) |

**Dos cosas llaman la atención, y las dos apuntan a lo mismo (R-01):**

1. **El usuario no es la dirección completa.** Para Gmail **tiene que serlo**:
   `pruebas.telemetry@gmail.com`, no `Pruebas_1`. La ayuda del campo dice «casi
   siempre es la misma dirección de correo» — es una **pista**, no una
   instrucción, y el formulario lo aceptó sin decir nada.
2. **La contraseña parece de ocho caracteres.** La de aplicación de Google son
   **dieciséis letras**. Si es la contraseña normal de la cuenta, Google la
   rechaza siempre.

### Lo que falta por saber

Lo que no se ve en la captura, y hace falta para cerrar esto:

- **Servidor**, **puerto** y **cifrado** (están más abajo en el formulario)
- Si la cuenta tiene **verificación en dos pasos** activada
- Si la contraseña es una **contraseña de aplicación** de dieciséis letras

### Por qué no lo doy por diagnosticado

El mensaje dice «no se pudo cifrar», que apunta al **servidor o al puerto**, no
al usuario ni a la contraseña — una credencial mala daría un error de
autenticación, no de cifrado. Así que lo más probable es que el puerto y el
cifrado no se correspondan: **587 va con STARTTLS** y **465 con TLS directo**,
y cruzarlos deja la conexión esperando a un servidor que no va a contestar.

**Dar por buena esa hipótesis sin los datos sería inventarse un diagnóstico**,
que en este proyecto ya ha costado caro dos veces.

### Qué cambió el 14/10/2026 con R-01, y qué NO

R-01 hace que el servidor, el puerto y el cifrado **ya no se escriban a mano**
cuando el dominio se reconoce: `@gmail.com` rellena `smtp.gmail.com`, `587` y
STARTTLS, y la garantía `el puerto y el cifrado nunca se cruzan` impide
justamente el par imposible que era la hipótesis.

Así que **si la causa era ésa, hoy ya no puede ocurrir**. Pero eso no cierra
H-03, y la diferencia importa:

- **No se ha probado contra Gmail.** La red de este contenedor bloquea los
  puertos 587, 465 y 25 —medido—, así que aquí no se puede. Es **R-24**.
- **Si la causa era la contraseña** —la de la cuenta en vez de la de
  aplicación— R-01 sólo añade un aviso cuando no mide dieciséis letras. Avisa,
  no impide.
- **Y el usuario** seguía siendo el otro sospechoso: ahora sigue a la dirección
  mientras nadie lo toque, que cierra el caso de la captura.

**H-03 se cierra cuando Dirección vuelva a intentarlo** con la versión nueva, no
antes. Un defecto que se marca arreglado porque «debería estarlo» es peor que
uno abierto: deja de buscarse.

---

## R-01 · Dar de alta un correo es demasiado complicado

**Pedido por Dirección el 14/10/2026**, y H-03 es su consecuencia directa: el
formulario dejó guardar un usuario que no podía funcionar, sin decir nada.

### Lo que hay hoy

Siete campos: dirección, nombre visible, usuario, contraseña, servidor, puerto y
cifrado. Tres se rellenan solos con los valores de Gmail; los otros cuatro los
escribe el usuario, y **nada comprueba que encajen entre sí** hasta que se pulsa
«Probar conexión».

### Lo que debería ser

**Que la mayoría de la gente sólo escriba dos cosas: su correo y su contraseña
de aplicación.** Todo lo demás se puede deducir o esconder.

Propuestas, a decidir:

| | Qué | Por qué |
|---|---|---|
| **1** | **Rellenar el usuario con la dirección** en cuanto se escriba, y dejarlo cambiar | Es lo correcto en Gmail, Outlook y casi todo. Hoy es un campo vacío que invita a inventarse algo |
| **2** | **Esconder servidor, puerto y cifrado** tras un «Configuración avanzada» plegado | Tres campos que nadie debería tocar ocupan la mitad del formulario |
| **3** | **Detectar el proveedor por el dominio** (`@gmail.com` → Gmail) y decirlo en pantalla | Quita la duda de si los valores automáticos son los correctos |
| **4** | **Avisar si la contraseña no tiene dieciséis caracteres** y el dominio es Gmail | Es el error más común y hoy sólo se descubre al fallar la conexión |
| **5** | **Que «Probar conexión» sea el camino principal**, no un botón más | Guardar una cuenta que no funciona no sirve de nada |

### Qué se hizo

**Lo que escribes ahora son dos campos: tu correo y tu contraseña.**

| | Qué | Cómo |
|---|---|---|
| **1** | **El servidor, el puerto y el cifrado se deducen del dominio** | `@gmail.com` → `smtp.gmail.com:587` con STARTTLS. También Outlook, Yahoo, Zoho e iCloud |
| **2** | **El usuario se rellena con tu dirección** | Es lo correcto en todos ellos. Sigue siendo editable, y deja de seguirla en cuanto lo tocas |
| **3** | **Los tres campos técnicos se esconden** tras «Configuración avanzada» | Ocupaban media pantalla y nadie debería tocarlos |
| **4** | **Se dice qué proveedor se reconoció** | «Reconocido como Gmail. El servidor, el puerto y el cifrado ya están puestos» — para no tener que abrir lo avanzado sólo por comprobar |
| **5** | **Avisa si la contraseña no tiene 16 caracteres** y el dominio es Gmail | Es el error más común y hoy sólo se descubría al fallar la conexión |

**Un dominio propio abre la configuración avanzada sola**, porque ahí sí hay que
escribirlos: ARLES no se inventa un servidor que no sabe.

### La garantía que pediste

> «Comprueba que el servidor, puerto y cifrado sean siempre correctos.»

El par **puerto + cifrado** sólo tiene dos combinaciones válidas en SMTP, y
cruzarlas es el error que no da un mensaje útil: la conexión se queda esperando.
Una prueba lo fija **para todos los proveedores, los de hoy y los que se
añadan** — 587 va con STARTTLS, 465 con TLS directo, y cualquier otro puerto
hace fallar la prueba con su nombre.

**Verificada en rojo** cruzando el par de Gmail a propósito: falla con
*«Gmail: 587 va con STARTTLS»*.

---

## R-02 · El alta de un número de WhatsApp

**Pedido por Dirección el 14/10/2026**, junto con R-01.

### El límite, dicho antes que nada

**Esto no se puede reducir a dos campos como el correo**, y conviene que quede
escrito por qué:

El correo **se deduce del dominio** — `@gmail.com` implica `smtp.gmail.com:587`.
Pero el **WABA ID**, el **Phone Number ID** y el **token** de Meta **sólo existen
en su panel**. No hay nada de lo que inferirlos: ni el teléfono los determina, ni
hay un patrón que seguir. Prometer otra cosa sería inventarse una función.

### Qué sí se hizo

| | Qué | Por qué |
|---|---|---|
| **1** | **El formulario se parte en tres bloques**: tu número · lo que se copia de Meta · cómo se va a usar | Antes eran seis campos seguidos, todos iguales, mezclando lo que se sabe de memoria con lo que hay que ir a buscar |
| **2** | **El nombre para identificarlo se rellena con el teléfono** | Es un campo «sólo para ti» que no sale en ningún mensaje: dejarlo vacío era pedir trabajo sin motivo |
| **3** | **Los tres datos de Meta se revisan al pegarlos**, antes de guardar | Es donde se pierde la gente |
| **4** | **Se dice en pantalla que esos tres sólo existen en Meta** | Para que nadie los busque en ARLES |

### Lo que la revisión caza

Los dos identificadores son números largos; el token empieza por `EAA`. Copiar
uno en el campo del otro da un error de Meta que **no dice cuál de los tres está
mal**. Ahora se avisa antes:

| Lo que pegas | Lo que dice |
|---|---|
| El token en un identificador | «Esto parece el token de acceso, no un identificador» |
| Un identificador en el token | «Esto parece un identificador, no el token» |
| Un identificador con espacios o guiones | «Son sólo dígitos, sin espacios ni guiones» |
| Un identificador cortado | «Parece corto: suelen tener quince dígitos o más» |
| Un token que no empieza por EAA | «Los tokens de Meta empiezan por EAA» |

**Avisa, no bloquea.** Meta podría cambiar el formato, y un formulario que se
niega a guardar algo correcto es peor que uno que señala algo dudoso.

**Nueve pruebas, cinco vistas en rojo** haciendo que la revisión deje pasar todo.

---

## A-01 · Dos cabeceras de columna que no cabían en su columna

**🟡 Molesto · Contactos · ✅ Arreglado 14/10/2026**

### Qué se leía

La tabla de contactos tiene cuatro columnas. Las dos últimas decían
«Correo principal» y «WhatsApp principal», y lo que se veía era:

```
   Nombre     Empresa     Correo princip…     WhatsApp pri…
```

Medido en Chromium, no mirado: **15 px de sobra a 1280 px de ancho y 51 px a
1024**, con el icono de ordenar comiéndose lo que quedaba. A 900 px no pasaba,
porque ahí la tabla se repliega a tarjetas.

### Por qué ninguna sonda lo veía

Y ésta es la parte que importa, porque el defecto de verdad no era la columna.

`.celda` recorta con `text-overflow: ellipsis`, que es la decisión correcta
**para un dato**: una dirección larga se corta con sus puntos y se lee entera al
abrir la ficha. El problema es que recortar **un rótulo** no es lo mismo: una
cabecera de columna es la única pista de qué estás mirando, y lo que se pierde
al cortarla es el final, que es justo donde está lo que la distingue de la de al
lado.

Y de ancho **cabe**: el recorte no ensancha nada, así que
`scrollWidth > clientWidth` —que es lo que mide `sonda:auditoria`— no se
dispara. Las dieciocho pantallas salían en verde.

### Qué se hizo

Se quita la palabra «principal». No se ensancha la columna porque el ancho que
haría falta se lo quita a la dirección, que es el dato; y lo que sobra es la
palabra: la celda ya enseña **cuántos más hay** con un `+2` al lado, que es
exactamente lo que «principal» venía a decir, y lo dice con una cifra (§94).

### Qué impide que vuelva

`sonda:encimado`, medida 4: ningún rótulo —`th`, `columnheader`, `label`,
`legend`, `caption`— puede estar recortado por los puntos suspensivos. Vista en
rojo estrechando la columna «Nombre» a 34 px: falla con «rótulo cortado:
«Nombre» necesita 46 px más».

---

## A-02 · Las sondas no medían el texto encimado

**🟠 Grave · Las dieciocho · ✅ Arreglado 14/10/2026**

No es un defecto del producto: es un defecto de las comprobaciones, y por eso
está aquí. Dirección escribió el 13/10/2026 *«el texto se ve extremadamente
desordenado y poco profesional»* con **las dos sondas de recorrido en verde**.

### Qué medía cada una, y qué no

| Sonda | Qué mide | Por qué no cazaba esto |
|---|---|---|
| `sonda:auditoria` | Desbordes con `scrollWidth > clientWidth` | Caza lo que no cabe **de ancho**. Dos párrafos montados uno sobre otro caben: su contenedor no se ensancha ni un píxel |
| `sonda:recorrido` | El borde izquierdo y el hueco vertical **del título** | Lo que pase cien píxeles más abajo no lo mira |

### Las cuatro medidas nuevas

`sonda:encimado` recorre las dieciocho pantallas en cuatro ventanas —**1 836
textos medidos**— y pregunta cuatro cosas geométricas:

| | Qué caza |
|---|---|
| **1** | Dos textos cuyos rectángulos se cruzan sin que uno contenga al otro |
| **2** | Texto que su contenedor recorta de alto: la última línea se queda a medias |
| **3** | Texto que asoma por debajo o por la derecha de su padre |
| **4** | Un **rótulo** cortado por los puntos suspensivos (ver A-01) |

Las cuatro **vistas en rojo** rompiendo el código de verdad, con copia previa
del archivo y nunca `git checkout`.

### Los tres hallazgos falsos que dio al nacer, y por qué se comprobaron

Una sonda que grita de más se desactiva, y un hallazgo falso en una auditoría
cuesta más que uno ausente: manda a arreglar algo que funciona y gasta la
credibilidad del resto. Los tres se comprobaron **antes** de anotarlos:

1. **El contenido de un `<details>` cerrado sigue devolviendo rectángulos**, y
   todos sus hijos caen apilados en la posición del `<details>`. Nueve falsos
   por ventana en Respaldos. La pregunta correcta no es «¿tiene área?» sino
   `checkVisibility()`, que para ese mismo `<li>` devolvía `false`.
2. **Un padre con `text-overflow: ellipsis` no deja asomar nada**: corta y pone
   los puntos. Las tres columnas de Contactos salían como si se desbordaran
   sobre la de al lado. No lo hacían — el defecto era otro, y es A-01.
3. **Un `.sr-only` de 1×1 px tiene la forma exacta de un rótulo recortado.** Es
   el texto que sólo oye el lector de pantalla, y se lee entero por el único
   camino que lo usa.

### Y una ruptura mía mal elegida

La primera ruptura de la medida 1 fue un margen negativo en el aviso del
proveedor de Remitentes. **No dejó síntoma**, porque ese aviso sólo se pinta al
escribir una dirección y en la vista previa el campo está vacío. Una ruptura
sobre algo que no está en pantalla no prueba nada; se rehízo sobre los
subtítulos de Respaldos, que se pintan siempre.

---

## A-03 · La comprobación que vigila las sondas tenía el defecto que persigue

**🟡 Molesto · Herramientas · ✅ Arreglado 14/10/2026**

`las_sondas_recorren_todas_las_pantallas` existe desde el 07/10/2026 para que
ninguna sonda vuelva a llevar su propia lista de rutas — el defecto que dejó
cinco pantallas sin medir nunca.

Decía vigilar «las sondas» y vigilaba **dos nombres escritos a mano**:
`auditoria.mjs` y `recorrido.mjs`. Al escribir la tercera, `encimado.mjs`, esta
comprobación **no la habría mirado nunca**.

Ahora las sondas salen de la carpeta: son las que importan `rutas.mjs`. Y el
hueco obvio que eso deja —una sonda con su lista propia no importa
`rutas.mjs`— lo cierra un segundo criterio: **ninguna puede nombrar más de tres
rutas por su cuenta**. El umbral está medido, no elegido: las sondas de una
pantalla nombran 0 o 1, `vista-previa.mjs` nombra 3, y las que recorren el
producto nombran 0 porque las piden.

### Y una afirmación mía de verificación que era falsa

Anoté que renombrar `rutas.mjs` haría fallar la comprobación **dos veces** —por
no existir el módulo y porque entonces nadie lo importa—. Es falso: las tres
sondas siguen llevando la cadena `./rutas.mjs` en su `import`, así que la
segunda guarda no se dispara. Hubo que romperla aparte, apuntando los tres
imports a otro nombre.

Una afirmación de verificación es una afirmación como cualquier otra, y en este
proyecto llevan varias entregas siendo las que más veces resultan falsas.

---

## A-04 · Cuatro cosas del tablero, encontradas mirándolo

**🟡 Molesto · Tablero · ✅ Arreglado 14/10/2026**

El tablero que Dirección pidió el 14/10/2026 salió con sus cifras correctas y
sin un error de consola. Mirarlo encontró cuatro cosas:

| | Qué | Por qué importa |
|---|---|---|
| **1** | **«1 hallazgos abiertos»** y **«1 bloqueante(s)»** | El mismo defecto que la 8.3c corrigió en «2 cuenta(s)». Un tablero que Dirección enseña a alguien no puede escribir así |
| **2** | Un **punto verde** delante de cada tropiezo | Verde quiere decir «esto está bien» y lo que decía era «éste es el número 1». Es el defecto de las ocho insignias con un reloj que no esperaba nada |
| **3** | **«Sin medir» pintado en rojo** | Rojo quiere decir que algo falla. Lo que pasaba es que no se sabe, y son dos cosas distintas — la misma razón por la que las omitidas van en su columna |
| **4** | Una fase al **0 %** con el canal vacío | Un canal vacío se lee igual que «no hay dato» |

Ninguna la caza una comprobación: las cuatro son dibujos correctos que dicen
algo que no es. **Antes de cerrar una entrega, se abre y se mira.**

---

## A-05 · La contraseña de Gmail viajaba con los espacios del dibujo

**🟠 Grave · Remitentes · ✅ Arreglado 15/10/2026**

### Lo encontró una pregunta, no una prueba

Dirección preguntó: *«¿de dónde consigo la contraseña de 16 caracteres de
Google?»*. Al escribir la respuesta hubo que mirar cómo la entrega Google, y
ahí estaba:

```
abcd efgh ijkl mnop
```

**Cuatro grupos de cuatro.** Los espacios son del dibujo —están para que se
pueda leer— y lo que vale son las dieciséis letras seguidas. Quien la copia, la
copia entera, y ARLES la mandaba al servidor **tal cual**, espacios incluidos.
Gmail la rechaza.

### Por qué era grave, y no sólo molesto

**Nada avisaba.** El aviso del largo ya descontaba los espacios para contar, así
que con `abcd efgh ijkl mnop` veía dieciséis, daba la contraseña por buena y se
callaba. En pantalla todo estaba correcto.

El fallo llegaba después, del servidor, como «no se pudo autenticar» — un
mensaje que parece decir *te equivocaste al escribirla*. Y no se había
equivocado: había copiado exactamente lo que Google le enseñó.

> Es el patrón que este proyecto ya conoce con otra cara: **dos piezas
> correctas y el agujero en la junta.** El aviso limpiaba para contar y el
> envío no limpiaba para mandar, y cada uno por separado hacía lo suyo bien.

### Qué se hizo

`contrasenaParaEnviar(proveedor, escrita)`, y las **dos** salidas de la
contraseña pasan por ella: probar la conexión y guardar. Que una limpie y la
otra no daría una prueba en verde y una cuenta que no envía.

**No se quitan los espacios siempre**, y la distinción es el arreglo:

| Lo que se escribe | Qué sale | Por qué |
|---|---|---|
| `abcd efgh ijkl mnop` en Gmail | `abcdefghijklmnop` | Sin espacios da 16, el largo que Gmail declara |
| `mi clave secreta` en Gmail | igual | Sin espacios da 14, no 16: no es una de aplicación mal pegada |
| `con espacios dentro` en Outlook | igual | Outlook no declara largo. Ahí una contraseña con espacios **es** una contraseña con espacios |
| Cualquier cosa, dominio no reconocido | igual | ARLES no se inventa reglas de un proveedor que no conoce |

Limpiar siempre sería destruir una contraseña válida y hacer fallar un alta
correcta.

### Qué impide que vuelva

Cinco pruebas en `proveedores.spec.ts`, verificadas **en las dos direcciones**,
que es lo que esta garantía necesita porque se puede romper por los dos lados:

- **No limpiar nunca** —el comportamiento de ayer—: falla con
  «expected 'abcd efgh ijkl mnop' to be 'abcdefghijklmnop'».
- **Limpiar siempre**: fallan tres, entre ellas
  «expected 'miclavesecreta' to be 'mi clave secreta'».

### Y la guía estaba vaga justo en ese paso

`GUIA-DE-CONEXION-REAL.md` decía «entra en la configuración de seguridad de
Google y busca Contraseñas de aplicaciones». Ahora lleva la dirección directa,
el camino por menús, el aviso de que **sólo se enseña una vez**, y los **cuatro
casos** en que la opción no aparece —sin verificación en dos pasos, cuenta de
Workspace, Protección Avanzada, o bloqueo de la organización— que son la razón
de que R-24 pida una cuenta desechable y no la corporativa.
