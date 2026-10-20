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
| **H-03** | 🔴 Bloqueante | Remitentes | **No se puede dar de alta una cuenta de Gmail.** «No se pudo cifrar la conexión con el servidor» | ✅ **Cerrado** 16/10/2026 · Dirección mandó un correo real; lo arreglaron A-05 y A-06 |
| **R-01** | 🔵 Requisito | Remitentes | Dar de alta un correo es **demasiado complicado**. Tiene que ser sencillo e intuitivo | ✅ **Hecho** 14/10/2026 |
| **R-02** | 🔵 Requisito | WhatsApp | Lo mismo con el alta de un número | ✅ **Hecho** 14/10/2026 · con un límite declarado |
| **A-01** | 🟡 Molesto | Contactos | La cabecera de la tabla se leía **«Correo princip…»** y **«WhatsApp pri…»**: no cabían en su columna | ✅ **Arreglado** 14/10/2026 |
| **A-05** | 🟠 Grave | Remitentes | La contraseña de aplicación de Gmail se mandaba **con los espacios** con que Google la enseña, y **nada avisaba** | ✅ **Arreglado** 15/10/2026 |
| **A-06** | 🟡 Molesto | Remitentes | La guía del alta decía **«busca Contraseñas de aplicaciones»** — un sitio donde buscar, no una instrucción | ✅ **Arreglado** 15/10/2026 |
| **H-04** | 🟡 Molesto | Configuración | No dejaba guardar `telemetrymx.com`: exigía escribir **`https://` a mano** | ✅ **Arreglado** 16/10/2026 |
| **H-05** | 🔴 Bloqueante | Contactos | «Ocurrió un error al acceder a los datos. Cierra ARLES y vuelve a abrirlo» al dar de alta un contacto | 🔎 **El mensaje, arreglado** 17/10 · falta la causa |
| **H-06** | 🔴 Bloqueante | Nueva campaña | **El horario de envío sale vacío** y no se puede avanzar del paso 5 | ✅ **Arreglado** 17/10/2026 |
| **H-07** | 🟠 Grave | Remitentes | La ficha del remitente **se parte en vertical** cuando hay un panel al lado: «Cifrado / (STARTTLS, / puerto / 587» una palabra por línea | ✅ **Arreglado** 17/10/2026 |
| **H-08** | 🟡 Molesto | Inicio | El paso «Conectar una cuenta remitente» **no se marca** aunque haya un remitente configurado | ✅ **Arreglado** 17/10/2026 |
| **H-09** | 🟡 Molesto | Nueva campaña | El hueco entre «Déjalo en blanco para usar el de la cuenta» y lo que sigue es **demasiado estrecho** | ✅ **Arreglado** 17/10/2026 |
| **H-10** | 🟠 Grave | Nueva campaña | No dice **cuál es el máximo diario por defecto**, así que se deja en blanco sin saber qué se está aceptando | ✅ **Arreglado** 17/10/2026 |
| **R-03** | 🔵 Requisito | Plantillas | Poder poner **negrita y cursiva** en el texto | ✅ **Arreglado 17/10/2026** |
| **R-04** | 🔴 Requisito | Firmas | **Una firma no puede ofrecer variables del contacto.** Que la firma del remitente lleve el nombre de a quien se escribe es una tragedia | ✅ **Arreglado 17/10/2026** |
| **R-05** | 🔵 Requisito | Firmas | Poder poner **una imagen** en la firma, como en Outlook | ⏸️ **Esperando decisión de Dirección** (ver H-15) |
| **R-06** | 🟠 Requisito | Plantillas | **No se pueden editar** las plantillas ni las firmas ya creadas | ✅ **Arreglado 17/10/2026** |
| **R-07** | 🔵 Requisito | Remitentes | El correo de prueba **llega a spam** — qué se puede hacer | 🔧 **En marcha** · paso 1 de 3 hecho 19/10/2026 |

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

---

## A-06 · La guía del alta mandaba a buscar en vez de decir dónde

**🟡 Molesto · Remitentes · ✅ Arreglado 15/10/2026**

### El síntoma fue una pregunta

Dirección preguntó: *«¿de dónde consigo la contraseña de 16 caracteres de
Google?»* — **con la guía de Gmail ya en la pantalla**, desplegable justo
debajo del campo.

Eso es lo que importa del hallazgo: **la guía estaba y no contestaba.** Su
paso 2 decía:

> *«En la configuración de seguridad de Google, busca "Contraseñas de
> aplicaciones" y crea una nueva.»*

Un sitio donde buscar no es una instrucción. Y buscar es donde la gente se
rinde: esa opción vive al final de una página a la que se llega por tres menús,
y **sólo aparece si la verificación en dos pasos está activada** — así que
quien no la tenga busca algo que para él no existe.

> Es el mismo defecto que el §95 persigue en los errores, aplicado a una
> ayuda: las tres partes estaban —qué pasa, cómo se arregla, qué está a
> salvo— y la de «cómo se arregla» mandaba a buscar.

### Qué se hizo

**La dirección exacta, dentro de la guía**, en un bloque seleccionable con un
botón para copiarla: `https://myaccount.google.com/apppasswords`.

Y tres cosas que faltaban:

| | Qué | Por qué |
|---|---|---|
| **Sólo se enseña una vez** | Quien cierra la ventana sin copiarla pierde la contraseña recién creada, y no lo sabe hasta que vuelve a buscarla |
| **Los cuatro casos** en que esa página no aparece | Enseñar sólo el primero —«activa la verificación en dos pasos»— manda a buscar durante horas a quien tiene una cuenta de empresa donde está prohibida. **Ese no lo arregla él** |
| **Los espacios no importan** | Va con A-05: ahora se pueden pegar las dieciséis letras tal como Google las enseña |

El error de contraseña rechazada lleva ahora la misma dirección: era la frase
que Dirección leyó el 13/10/2026 al no poder dar de alta su cuenta.

### El dato vive en el catálogo, no en un `if`

`dondeSeCreaLaContrasena` es un campo de `PROVEEDORES`. El día que se sepa la
de Outlook se escribe ahí y la pantalla no se toca. Un `if` sobre Gmail dentro
de la pantalla sería el mismo dato en dos sitios, que es como empiezan todos
los que se separan.

**Y `null` cuando no se sabe**, que es el caso de los otros cuatro: una
dirección inventada que lleva a ninguna parte es peor que no dar ninguna,
porque se sigue. Hoy sólo está comprobada la de Google.

### Por qué un botón de copiar y no un enlace que abra el navegador

Sería más cómodo, y **cuesta un permiso nuevo**. ARLES declara hoy
`core:default` y nada más, y la auditoría B del §140 lo recogió como uno de los
puntos que salieron limpios. Abrir el navegador desde la aplicación es un
plugin, una dependencia que pasar por `cargo deny` y un permiso que acotar.

**Queda como decisión escrita, no como olvido**: si Dirección considera que el
gesto de más lo merece, se añade con su scope acotado.

### Qué impide que vuelva

Cuatro pruebas en `proveedores.spec.ts`, verificadas rompiéndolas:

- **Gmail avisa del largo y no dice dónde** → falla con «Gmail avisa del largo:
  expected null not to be null». Es la garantía de que avisar de que algo está
  mal sin decir cómo arreglarlo no pasa (§95).
- **Una dirección inventada y a medias** → falla con «Outlook: expected
  'account.microsoft.com' to match /^https:\/\//».

### Y dos defectos míos en el mismo arreglo

1. **Me inventé un token de diseño**, `--arles-surface-alt`. No existe: los
   reales son `surface`, `surface-raised` y `surface-hover`. El navegador
   resuelve un `var()` inexistente **a nada, sin error**, así que el bloque se
   habría visto sin fondo y nadie lo habría notado. **Cuarta vez en el
   proyecto** — las otras tres están en la Fase 8. Lo encontré comprobando que
   el token existía antes de usarlo, y luego verifiqué que el validador
   también lo caza: lo nombra con archivo y token.

2. **Miré una captura de antes de mi cambio.** El guion de la captura no
   regenera la vista previa, así que medí el HTML anterior y di por hecho que
   el botón no había cambiado. Es la trampa de las herramientas que este
   proyecto ya tiene escrita: *«una sonda que no comprueba su código de salida
   mide otra versión del código»*. Al regenerar, el botón pasó de 170×40 a
   162×32 — había cambiado desde el principio.

---

## H-04 · El sitio web exigía escribir «https://» a mano

**🟡 Molesto · Configuración · ✅ Arreglado 16/10/2026**

### Qué pasó

Dirección escribió `telemetrymx.com` y ARLES se negó a guardar **toda la
configuración**, con el campo en rojo y:

> *«Escribe la dirección completa, empezando por https://.»*

Nadie escribe el esquema al dictar su propio sitio web, y exigirlo es hacer
teclear algo que se puede deducir — el mismo defecto que R-01 arregló en el
alta de correo.

> **Lo que sí estaba bien y conviene decirlo:** el campo **ya era opcional**.
> Se puede dejar vacío y guardar. Por eso no lleva asterisco, a diferencia del
> nombre comercial y el correo corporativo. Lo que bloqueaba no era que
> faltara: era que lo escrito no tenía la forma exigida.

### Qué se hizo

Si lo escrito **no puede llevar esquema**, se le pone `https://` solo:

| Lo que escribes | Lo que se guarda |
|---|---|
| `telemetrymx.com` | `https://telemetrymx.com` |
| `www.telemetrymx.com/contacto` | `https://www.telemetrymx.com/contacto` |
| `http://telemetrymx.com` | igual — completar no es reescribir |

### La condición no es «si no empieza por http», y la diferencia es de seguridad

Este valor acaba en un `href`, así que un esquema ejecutable ahí es XSS servido
por la propia configuración del usuario. Hay una prueba de seguridad que lo
cierra desde la Fase 3.

Anteponer `https://` a todo lo que no empiece por `http` convertiría
`javascript:alert(document.cookie)` en `https://javascript:alert(document.cookie)`,
que **pasa las dos comprobaciones** y entra en el `href`.

Así que sólo se completa lo que no lleva `:` ni empieza por `/`. El precio
declarado: un sitio con puerto —`ejemplo.com:8080`— lleva `:`, no se completa y
hay que escribirle el esquema. Un sitio corporativo con puerto no existe en la
práctica; un `javascript:` que entra, sí.

### Mi prueba nueva nació sin probar nada, y lo encontró romperla

Escribí `completar_el_esquema_no_cuela_uno_ejecutable` expresamente para ese
riesgo, y **pasó en verde con el código roto**.

El motivo: sus ejemplos —`javascript:alert(1)`, `file:///etc/passwd`,
`data:text/html,<script>`— **no llevan un punto**. Al completarlos caían en la
comprobación siguiente —«esto no parece un dominio»— y no en la del esquema. La
prueba decía cubrir una cosa y cubría otra.

Y el comentario que le puse afirmaba *«verificada poniendo esa condición
equivocada: falla nombrando javascript:alert(1)»*. **Era falso**: lo que fallaba
eran las otras dos pruebas, no la mía.

Los ejemplos de ahora llevan punto a propósito —`javascript:alert(document.cookie)`—
porque es lo que hace falta para llegar vivo hasta el `href`. Con esos, la
condición equivocada **sí** la caza, nombrando el payload.

> **Una afirmación de verificación es una afirmación como cualquier otra**, y
> en este proyecto llevan varias entregas siendo las que más veces resultan
> falsas. Ésta es la enésima.

### Lo que esto no arregla, y es la pregunta de Dirección

**El sitio web no se usa en ninguna parte.** Se guarda, se lee de vuelta al
formulario, y ahí acaba: no es variable de plantilla —las cuatro son `nombre`,
`apellido`, `empresa` y `mi_empresa`—, no sale en ninguna firma, no entra en el
preflight y no viaja en ningún correo.

Es un campo que pide trabajo en el primer minuto de uso y no devuelve nada. La
decisión de qué hacer con él está planteada a Dirección.

---

# La ronda del 16/10/2026

**Y lo primero, que no es un defecto: ARLES mandó un correo de verdad.** Con la
cuenta `pruebas.telemetry@gmail.com`, a una dirección real, y el servidor lo
aceptó. **H-03 queda cerrado** — era el bloqueante desde el 13/10, y con él
cerrado la Fase 5 deja de estar probada sólo contra un servidor de mentira.

---

## H-05 · «Ocurrió un error al acceder a los datos» al dar de alta un contacto

**🔴 Bloqueante · Contactos · 🔎 Investigando**

### Qué se vio

Al rellenar «Agregar contacto» —Adrian / Paz / Insight / `apazg@outlook.com`— y
pulsar **Dar de alta**:

> **Ocurrió un error al acceder a los datos.**
> Cierra ARLES y vuelve a abrirlo.
> Las operaciones incompletas se revirtieron por completo.

### Lo que encontré, y es un defecto aunque no sea la causa

**Ese mensaje es un cajón de sastre, y miente sobre lo que pasó.** En
`stores/contactos.ts`:

```ts
if (!esErrorIpc(e)) {
  errorGeneral.value = 'error.db.sqlite'
  return
}
```

Cualquier cosa que no sea un error tipado del núcleo —un fallo del propio
JavaScript, un problema al serializar, cualquier excepción de la webview— se
reporta como **«error al acceder a los datos»** y manda a **cerrar y reabrir
ARLES**, que si la causa es otra no arregla nada.

Es exactamente el defecto que el fallo de migración tenía el 13/10: una
instrucción que el producto no puede cumplir. Y aquí es peor, porque además
**afirma una causa que no conoce**.

### Lo que NO es

Reproduje el camino exacto de la captura en `arles-db`, contra una base nueva
con todas las migraciones: leer la lista vacía, validar ese borrador, crear el
contacto, recargar la lista y resolver las direcciones suprimidas. **Pasa
entero**, y un segundo alta seguido también.

Así que el alta en sí funciona. Las pruebas se quedan en el repositorio —
`reproduccion_h05`— porque cubren un camino que ninguna cubría de punta a
punta.

### Qué hace falta para cerrarlo

**El mensaje se traga la causa**, así que no puedo ir más allá sin verla. Lo
que se va a hacer: que ARLES **enseñe el detalle técnico** del error, plegado,
para poder pegarlo aquí. Un producto que pierde la causa de sus propios fallos
obliga a adivinar.

---

## H-06 · El horario de envío sale vacío y no se puede crear ninguna campaña

**🔴 Bloqueante · Nueva campaña · 🔎 Diagnosticado**

### Qué se vio

Paso 5 de 9, «Ritmo y horario». El desplegable **Horario de envío** no tiene
ninguna opción, y sin elegir uno **«Continuar» no deja pasar**. La campaña no
se puede crear.

### La causa, y es el patrón de este proyecto por decimoctava vez

Las opciones salen de las ventanas de envío guardadas. **Y no hay forma de
crear una desde la aplicación.**

| | |
|---|---|
| El comando `guardar_ventana` | **existe**, registrado en la frontera |
| El store `campanas.ts` | **lo envuelve**, en `guardarVentana()` |
| Alguna pantalla que lo llame | **ninguna** |

Es el hueco del §9 en su forma de siempre: las piezas correctas y el cable sin
conectar. La propia lista de Inicio lo dice —«Definir una ventana de envío ·
*Llega en la entrega 6*»— mientras el asistente **la exige para avanzar**.

### Por qué la comprobación que persigue esto no lo cazó

`ningun_comando_se_queda_sin_quien_lo_llame` compara los comandos de
`generate_handler!` con quien los invoca **en todo `app/src`**. Y el store lo
invoca, así que pasa en verde.

El tramo que falta es el siguiente: **store → pantalla**. Un comando que sólo
llega hasta el store es exactamente igual de inútil que uno que no llega a
ninguna parte, y hasta hoy nadie lo medía.

---

## H-07 · La ficha del remitente se parte en vertical

**🟠 Grave · Remitentes · 📋 Abierto**

Con el panel de «Mandar un correo de prueba» abierto al lado, la ficha de la
cuenta se estrecha y su línea de configuración se parte **una palabra por
línea**:

```
Cifrado
(STARTTLS,
puerto
587
Hasta
50
al
día
```

Cerrado el panel, la misma línea cabe entera. Es un defecto de maquetación: la
ficha no tiene ancho mínimo y el panel se lo come.

> **`sonda:encimado` no lo caza**, y es información: mide textos montados,
> cortados y asomados, y éste **cabe perfectamente** — está partido, no
> desbordado. Una palabra por línea es legible carácter a carácter y legible es
> lo que la sonda comprueba.

---

## H-08 · El paso del remitente no se marca

**🟡 Molesto · Inicio · 📋 Abierto**

Con una cuenta configurada y «Lista para enviar», la lista de Inicio sigue
enseñando **«1 de 6»** y «Conectar una cuenta remitente» sin su palomita,
mientras «Configurar tu empresa» sí la tiene.

Un contador que no sube cuando acabas de hacer el paso hace dudar de si de
verdad quedó hecho.

---

## H-09 · El hueco del ritmo es demasiado estrecho

**🟡 Molesto · Nueva campaña · 📋 Abierto**

Entre la ayuda «Déjalo en blanco para usar el de la cuenta» y la etiqueta
siguiente —«Máximo por hora»— casi no hay aire, y las dos líneas se leen como
una sola. Pedido: ampliarlo **sin pasarse**.

---

## H-10 · No dice cuál es el máximo diario por defecto

**🟠 Grave · Nueva campaña · 📋 Abierto**

«Máximo al día» se puede dejar en blanco, y la ayuda dice que entonces se usa
«el de la cuenta» — **sin decir cuál es**. Se acepta un número sin saberlo.

Y es un número que importa: por encima de 50 diarios ARLES exige aceptar un
aviso (§48), porque es donde los proveedores empiezan a tratarte como remitente
masivo.

Dirección pidió consejo sobre si hacerlo obligatorio. Está respondido más
abajo, en «Las tres decisiones».

---

## R-03 a R-07 · Lo que falta en plantillas, firmas y entregabilidad

| | Qué | Por qué importa |
|---|---|---|
| **R-03** | **Negrita y cursiva** en las plantillas | Hoy el texto sale plano |
| **R-04** | **Una firma NO puede ofrecer variables del contacto** | Es un error de diseño con consecuencias: una firma con `{{nombre}}` pondría el nombre de **quien recibe** donde va el de quien firma. Dirección lo llamó «una tragedia», y tiene razón: sale en cada correo de la campaña |
| **R-05** | **Una imagen en la firma**, como en Outlook | El logotipo es lo primero que se mira en una firma |
| **R-06** | **Editar** plantillas y firmas ya creadas | Hoy se crean y no se tocan. Chocaba con «las versiones no se editan», y hay que distinguir las dos cosas: una **versión** no se edita —eso es correcto y protege lo ya enviado—, pero **tiene que poder nacer otra** a partir de la anterior |
| **R-07** | **El correo de prueba llegó a spam** | Es la 7.5 (SPF, DKIM, DMARC), lo único que necesita red y que sólo se podía verificar el día que R-24 se cerrara. **Hoy se cerró** |

### R-04 · Arreglado el 17/10/2026 — en dos mitades, y las dos hacen falta

Dirección lo dijo así: *«sería una tragedia que apareciera en la firma el nombre
del contacto a quien se le está haciendo llegar el correo y su empresa, en lugar
de los datos del remitente»*. Y no era una posibilidad teórica: el editor
**ofrecía los cuatro botones de variables en la firma igual que en la
plantilla**, así que el camino más corto para firmar era pulsar el que decía
«Empresa» — que es la del contacto.

**Lo que no se puede hacer es sólo quitar los botones.** El cuerpo de la firma
es un campo de texto: quien escriba `{{nombre}}` a mano llega al mismo sitio, y
una pantalla que ayuda a no equivocarse no es una garantía. Así que el arreglo
tiene dos mitades, con responsabilidades distintas:

| Dónde | Qué hace | Qué pasa si falla |
|---|---|---|
| **La pantalla** (`PantallaPlantillas.vue`) | Ofrece sólo `{{mi_empresa}}` cuando se edita una firma, y avisa en tono de peligro de las que se escriban a mano, con «Guardar» bloqueado | Se puede intentar guardar una firma con `{{nombre}}` |
| **El núcleo** (`plantilla::revisar_firma`) | Rechaza el cuerpo antes de sanear y antes de tocar la base | **Nada**: la base no puede llegar a tener una firma así, venga de donde venga |

La pregunta que distingue las dos variables la contesta el propio núcleo:
`Variable::es_de_quien_recibe()` — `Nombre`, `Apellido` y `Empresa` son de quien
recibe; `MiEmpresa` es de quien firma. Y `de_quien_firma()` **se deriva de ella**
filtrando `TODAS`, en vez de ser una segunda lista a mano: una variable nueva
entra por un solo sitio, y si alguien añade una de quien recibe sin marcarla,
aparece en la firma. Hay una prueba de esa derivación.

**El error es uno nuevo y no un «dato inválido» genérico** —
`CoreError::FirmaConVariablesDelContacto { cuales }` — porque dice **cuáles**
encontró. Un error que sólo dijera «la firma no es válida» obligaría a buscar a
ojo en un HTML; éste nombra las tres si están las tres (§95: qué pasó, cómo se
arregla, qué está a salvo).

**Verificado rompiéndolo** (§2), que es lo que hace que estas nueve pruebas
valgan algo:

- Las cinco de `arles-core` (`firmas_sin_datos_ajenos`): se cambió
  `es_de_quien_recibe` para que `Empresa` contara como de quien firma. **Tres
  fallan** nombrando `{{empresa}}`, incluida la de la derivación.
- Las cuatro de la pantalla (`plantillas.spec.ts`): se devolvieron las cuatro
  variables al editor de firmas. **Falla** con «una firma no puede ofrecer
  «Nombre»».

**Lo que esto NO arregla:** R-05 (una imagen en la firma) sigue abierto.

### R-06 · Arreglado el 17/10/2026 — y la cabecera de la pantalla mentía

El código de `PantallaPlantillas.vue` decía esto:

> *«**No edita una versión existente.** Una versión de plantilla no se toca
> (6.3): guardar crea una nueva. Enseñar un botón de «editar» sería prometer
> algo que el modelo de datos impide a propósito.»*

**Era falso en los dos casos**, y no por poco:

| | Lo que había | Lo que faltaba |
|---|---|---|
| **Plantillas** | `nueva_version_de_plantilla` en `arles-db` **desde la 6.3**, con pruebas | No estaba registrada en la frontera ni la llamaba nadie |
| **Firmas** | Nada: una firma no tiene versiones | El `UPDATE` no estaba escrito |

Lo que el modelo de datos impide es **reescribir lo ya enviado**. Corregir una
plantilla no es eso: es hacer nacer la versión siguiente, y era exactamente lo
que Dirección necesitaba. Es el patrón del §9 por **decimonovena vez**, y aquí
con una agravante: el comentario que explicaba por qué no existía el botón
**describía una limitación que no existe**, así que leyéndolo se daba el hueco
por intencional.

**Consecuencia real, y es la que importa:** una firma con una errata —o con un
`{{nombre}}`, que es R-04— **no se podía arreglar nunca**.

**Los dos botones dicen cosas distintas a propósito.** Se llaman «Corregir» y
no «Editar»:

- En una **plantilla**, nace una versión nueva y un aviso lo dice con su cifra
  —«Esto va a crear la versión 3»— más la frase que importa: *las campañas que
  ya estén activas siguen enviando con la anterior; lo que está en cola no
  cambia*. Creer lo contrario es el error caro, y es el que la forma de los
  datos invita a cometer. El nombre sale en sólo lectura, porque versionar no
  lo cambia, y se dice por qué (§95, la lección de la 8.2c).
- En una **firma** se reescribe encima. No hay nada congelado que proteger: una
  firma no tiene versiones.

**Verificado rompiéndolo** (§2) — nueve pruebas, cinco rupturas:

- Cuatro en `arles-db` (`actualizar_firma`): el `UPDATE` sin escribir el
  cuerpo, sin desmarcar el por defecto anterior, sin la guarda de R-04, y sin
  comprobar de qué empresa es la firma. Las cuatro en rojo, cada una cazada
  sólo por la suya.
- Cinco de pantalla: el editor sin precargar, el aviso de la versión fuera, y
  el guardado mandando al comando de crear —en la plantilla y en la firma—.

**Y la quinta ruptura enseñó algo.** `idEditado` se limpia en **dos** sitios,
`abrir()` y `cerrar()`; quitar uno no deja ningún síntoma porque el otro lo
tapa, y la prueba seguía en verde. Hubo que quitar los dos para verla caer.
Está bien que el código lo haga dos veces; lo que no vale es dar la
verificación por hecha con una ruptura que la otra mitad tapa — es la lección
de la pasada D dicha de otra forma: **una ruptura que deja el síntoma en pie no
prueba nada.**

### R-03 · Arreglado el 17/10/2026 — lo que faltaba no era el permiso

El saneado del núcleo acepta `strong`, `b`, `em`, `i`, `u`, listas y enlaces
**desde la 6.2**. Lo que no había era **forma de ponerlos** sin escribir las
etiquetas a mano, que es pedirle HTML a quien no escribe HTML.

Dos botones que envuelven lo seleccionado, por el mismo motivo que los de las
variables: lo que acaba en el texto lo pone el programa.

**Se consideró un editor visual (`contenteditable`) y se descartó.** Produce el
HTML que decide el navegador —algunos escriben
`<span style="font-weight:700">`— y eso el saneado lo tira: el texto se vería
en negrita al escribirlo y saldría plano al guardarlo, sin un solo error. El
botón escribe `<strong>`, que es lo que la lista blanca acepta.

**Sin selección inserta el par y deja el cursor dentro**, que es el caso que se
olvida: quien quiere *empezar* a escribir en negrita pulsa antes de escribir, y
un botón que no responde se lee como una avería (§95). Las dos pruebas se
vieron en rojo.

### H-13 · La comprobación del §9 vigilaba **cero** funciones, y por eso pasó el H-06

Encontrado el 17/10/2026, arreglando lo de arriba. Es el hallazgo más grave de
esta ronda y no está en el producto: está en el instrumento.

`lo_que_se_construye_se_usa` existe desde la fase W para cazar el patrón del §9
en el tramo **almacén → pantalla**, y su primer tramo decía vigilar «cada
función del store que invoca un comando». Lo hacía con esta expresión regular:

```
async function (\w+)\(…\)[^{]*\{(?:[^{}]|\{[^{}]*\})*?invocar
```

Intenta llegar desde la llave de apertura hasta `invocar` admitiendo **un** nivel
de anidamiento. Y en este proyecto **todas** las funciones de almacén envuelven
la llamada en un `try { … }`, así que `invocar` está dos niveles dentro y el
patrón no llega nunca.

**Medido antes de tocar nada: 0 funciones vigiladas de 52.**

Por eso el H-06 pasó en verde durante tres entregas: `guardarVentana` envolvía
un comando probado, ninguna pantalla la llamaba, y lo encontró Dirección al
quedarse sin horario de envío en el paso 5 del asistente.

**Una expresión regular no puede contar llaves**, que es lo que esa intentaba.
Ahora el texto se trocea por `async function`, y el uso se busca como `.nombre`
—no como `nombre`— porque una función llamada `cargar` aparece en cualquier
pantalla por otros motivos. Se acepta sin paréntesis a propósito:
`@click="importacion.elegirArchivo"` pasa el manejador por referencia, y ése
fue el primer falso positivo al reescribirla.

**Y lleva un guardarraíl, que es lo que impide repetir hoy:** si el troceado
encuentra menos de 40 funciones, **falla diciéndolo**. Sin eso, el día que los
almacenes cambien de forma la comprobación volvería a vigilar cero y seguiría
saliendo en verde. Verificado rompiéndole el troceado: «sólo se leyeron 0
funciones de store».

### H-14 · No se podía dar de baja a nadie a mano (arreglado el 17/10/2026)

Lo encontró la comprobación de arriba el minuto en que empezó a mirar:
**`suprimirContacto()` no se llamaba desde ninguna pantalla.** Patrón del §9
por **vigésima vez**, y de los caros.

`suprimir_contacto` estaba registrado en la frontera, con sus pruebas, y el
almacén lo envolvía con motivo y notas. La pantalla de Bajas sólo **lista** y
**levanta**. Así que si alguien pedía por teléfono que no le escribieran más,
**desde ARLES no había dónde apuntarlo**.

Y hay un detalle que lo confirma: `MOTIVOS_A_MANO` —la lista de los tres
motivos que una persona puede elegir, con su explicación de por qué no son los
seis— llevaba escrita desde la 3.4 **sin que nadie la usara**. Estaba diseñado
el formulario que nunca se escribió.

**Dar de baja y no volver a escribir son dos cosas distintas**, y la diferencia
es la que había que decir en pantalla:

| | Qué hace | Qué pasa si se reimporta el Excel |
|---|---|---|
| **Dar de baja** | Quita el contacto de la lista | **Vuelve a entrar**, y se le escribe |
| **No volver a escribirle** | Apunta sus direcciones en la lista de supresión, que no depende del contacto | Sigue protegido |

Va en el mismo diálogo, separado por una línea, con el **motivo preguntado y no
supuesto**: marcar «lo pidió la persona» cuando lo decidimos nosotros falsea el
registro que existe justo para contestar una reclamación dentro de un año. Y
una nota opcional, para el «lo pidió por teléfono».

**La supresión va antes de la baja, y si falla no se da de baja.** Al revés, un
fallo al suprimir dejaría el contacto fuera de la lista con su dirección sin
proteger — y desaparecido de la vista el único sitio donde se ve que quedó a
medias. Hay una prueba de cada mitad, las dos vistas en rojo.

### R-07 · El spam, paso 1 de 3: la cabecera de baja (19/10/2026)

Dirección: *«el correo llega a spam, ¿hay alguna cosa que podamos hacer?»*. Hay
tres cosas, y ésta es la primera porque es **la que más mueve con menos
trabajo** y la única que está enteramente en nuestra mano.

**Lo primero que hay que decir, porque cambia el diagnóstico:** la cuenta de
pruebas es `@gmail.com`, así que **SPF, DKIM y DMARC no son el problema**. Esos
registros son de Google y están perfectos. Lo que penaliza un correo de Gmail
mandado en volumen es otra cosa, y la 7.5 —que es lo que el ROADMAP tiene
escrito para esto— **no lo arregla mientras el remitente sea Gmail**.

| | Qué | Estado |
|---|---|---|
| **1** | `List-Unsubscribe`: la salida de quien recibe | ✅ 19/10/2026 |
| **2a** | Correo con estructura: HTML **y** texto plano, en `multipart/alternative` | ✅ 20/10/2026 |
| **2b** | El cuerpo de la plantilla, por contacto, con sus variables y su firma | ✅ 20/10/2026 |
| **3** | La 7.5 (SPF, DKIM, DMARC), que sirve **el día que se mande desde `@telemetrymx.com`** | 📋 Abierto |

**Por qué la cabecera de baja es la primera.** Gmail y Outlook la **exigen**
desde febrero de 2024 a quien manda correo en volumen: sin ella el correo entra
con una penalización de salida que no compensa nada de lo demás. Y hay una
segunda razón que no es de entregabilidad: **es la salida de quien recibe**. Sin
ella, la única forma que tiene una persona de dejar de recibir es marcar el
correo como no deseado — que es justo lo que más daño hace a la reputación del
remitente. Dar una salida fácil es lo que evita la queja.

**Es un `mailto:` y no un `https:`, y es una limitación declarada.** La versión
de un clic (RFC 8058) necesita que alguien reciba un `POST` en una dirección
web, y ARLES es una aplicación de escritorio: **no tiene servidor**. Poner
`List-Unsubscribe-Post` sin ese `https` detrás sería anunciar una puerta que no
existe, y los filtros lo comprueban: afirmar de más ahí perjudica. Hay una
prueba dedicada a que **no** se prometa.

**La baja llega al buzón de quien envía y se apunta a mano.** ARLES no lee
buzones (ADR-0009/0014), así que esto no da de baja a nadie por su cuenta y la
regla de la Fase 5 sigue entera: nada antes del `RCPT TO` puede dar de baja a
nadie. Quien reciba el correo lo apunta en Contactos con «no volver a
escribirle» — que es exactamente lo que el H-14 construyó dos días antes, sin
saber que iba a hacer falta para esto.

**Se comprueba desde el lado del servidor, y eso obligó a cambiar una decisión
del servidor de mentira.** Tiraba el cuerpo entero a propósito —«no es un
buzón»— y ahora **se queda con las cabeceras**, porque mirar lo que produce la
función que arma el mensaje comprobaría que esa función está bien, no que el
envío la use: el patrón del §9. El texto que alguien escribió se sigue tirando.

Tres rupturas, las tres en rojo: sin la cabecera, sin los ángulos que pide el
RFC 2369, y prometiendo el clic único.

### R-07 · El spam, paso 2a: el correo bien formado (20/10/2026)

El correo salía **en una sola parte de texto plano**: `lettre` la declara
`text/plain`, así que un cuerpo con `<strong>` habría llegado **con las
etiquetas a la vista**. Y un correo de una sola parte es una de las señales que
miran los filtros de spam — además de que hay clientes y relojes que sólo
enseñan el texto.

Ahora sale en `multipart/alternative` con las dos versiones. **El orden lo fija
el RFC 2046**, de la menos preferida a la más: el texto primero, el HTML
después. Al revés, un cliente que respeta el orden enseñaría el texto plano a
todo el mundo y nadie vería la negrita.

**`Contenido` ya no deja mandar un correo sin su versión en texto.** Los campos
son privados y sólo se llenan por sus constructores, por la misma razón que no
existe forma de no cifrar (§29): si el texto plano fuera opcional, el camino
«sólo HTML» existiría, alguien lo tomaría, y el correo saldría peor sin que
nada fallara.

**El texto plano se deriva del HTML y no se pide.** Pedirlo invita a escribirlo
a mano, y dos versiones escritas a mano acaban diciendo cosas distintas — la
que nadie mira es la que se queda vieja. Lo deriva el mismo convertidor del
núcleo que usa `arles-db` al guardar una plantilla, así que para el mismo HTML
el texto que viaja y el que se guarda son el mismo.

**Y `Contenido::ninguno()` dice con su nombre lo que había escrito con dos
cadenas vacías:** que ese camino —la prueba de conexión, el arranque del
motor— no manda ningún correo. En el motor es además donde está el cable que
falta, y ahora se lee como una decisión pendiente en vez de como un olvido.

> **La prueba no cazaba una de sus tres rupturas, y la lección es la de
> siempre.** Comparaba **dónde aparece** cada `Content-Type`, y el constructor
> de `lettre` pone siempre la parte plana primero: ese orden no puede fallar.
> Pasándole los argumentos al revés —el HTML etiquetado como texto y el texto
> como HTML— la prueba seguía en verde sobre un correo roto. Lo que distingue
> las dos partes es **lo que llevan dentro**: la plana no puede llevar
> etiquetas y la de HTML tiene que llevarlas. Rehecha así, las tres rupturas en
> rojo.

### R-07 · El spam, paso 2b: el correo manda la plantilla (20/10/2026)

**Y con él se cierra el H-15: ARLES manda el texto que Dirección escribe.**

El contenido lo guardaba **el adaptador**, uno para toda la campaña. Y una
campaña no tiene un texto: tiene un texto por persona. Así que `arles-app` le
pasaba uno vacío, y **ninguna campaña mandaba el cuerpo de su plantilla**.

Ahora el contenido viaja con cada `Mensaje`, redactado para esa persona:

| Pieza | Qué hace | Dónde |
|---|---|---|
| `plantilla::redactar` | Pone las variables, escapa los datos, pega la firma | `arles-core` (puro) |
| `sanear::escapar_para_html` | Que un dato no se convierta en etiqueta | `arles-core` |
| `DatosParaLaPlantilla` | Nombre, apellido y empresa **vienen con el trabajo** de la cola | `arles-db` |
| `redaccion_de_la_campana` | La versión congelada, la firma por defecto y `{{mi_empresa}}` | `arles-db` |
| `Ajustes.redaccion` | Se lee **una vez por vuelta**, no por envío | `arles-motor` |

**La seguridad primero, porque la auditoría B la dejó escrita antes de que este
camino existiera.** La plantilla se sanea **al guardarla**, y la sustitución
ocurre **al enviar**: mete un dato de un XLSX ajeno dentro de un HTML que ya
pasó su revisión. Una empresa llamada `<script>…` habría entrado como etiqueta.
Los datos se escapan, y hay una prueba que lo mira.

**El asunto NO se escapa**, y también es a propósito: no es HTML, viaja en una
cabecera y se enseña tal cual. Escapándolo, `Pérez & Hijos` llegaría como
`Pérez &amp; Hijos` a la bandeja de quien lee.

**Los datos vienen con el trabajo de la cola, en la misma consulta.** Pedirlos
aparte abre un hueco —entre tomar el intento y leer el contacto, alguien puede
editarlo— y son dos idas a la base por envío. Y es un **`LEFT JOIN`**: un
intento puede sobrevivir a su contacto (`ON DELETE SET NULL` protege la prueba
de que se le escribió), y con un `JOIN` esa fila **desaparecería de la cola** —
un envío que no sale y que nada explica.

**La firma es la de por defecto, y es una limitación declarada.**
`campaign_stage.signature_id` sigue sin que nada lo escriba (H-12), así que se
usa la que la empresa marcó — que es literalmente lo que promete su casilla:
«usar esta firma cuando no se elija otra». Hoy nunca se elige otra.

**Si la base no contesta la redacción, se envía sin cuerpo en vez de no
enviar.** Es la decisión incómoda de las dos y es la correcta: la audiencia está
congelada y los permisos del día gastados, así que pararse ahí deja la campaña
muda sin que nada lo explique. Un correo sin cuerpo se ve; una campaña que no
avanza, no.

> **La prueba que este proyecto no podía escribir hasta hoy.** «El cuerpo de la
> plantilla llega al proveedor con los datos de esa persona» era imposible de
> comprobar: **el mensaje no tenía dónde llevarlo**, así que no había nada que
> mirar. Es el patrón del §9 en su versión más larga —siete piezas correctas y
> el cable sin conectar— y lo encontró medir, no leer.
>
> Nueve rupturas, todas en rojo: no escapar los datos, escapar el asunto, la
> firma sin rellenar, contar sólo lo que falta en el cuerpo, la raya de una
> firma vacía, el motor sin redactar, el `JOIN` en vez del `LEFT JOIN`, leer la
> versión vigente en vez de la congelada, y pegar una firma que no es la de por
> defecto.

**Lo que sigue sin existir:** que el asistente **pregunte** por la firma, y la
imagen en la firma (R-05), que ahora sí tiene dónde verse.

### H-15 · El camino «plantilla → correo» (17/10/2026) · ✅ cerrado el 20/10

Medido al preparar R-05, y es más grande que H-12. Tres cosas, las tres
comprobadas en el código y no supuestas:

1. **El mensaje se arma con un solo cuerpo de texto plano.**
   `ProveedorSmtp::armar` llama a `.body(cuerpo)` sin `multipart` y sin
   declarar `text/html`. Un cuerpo con `<strong>` llegaría **con las etiquetas
   a la vista**.
2. **El `Contenido` que llega al SMTP es fijo.** Los dos que existen son el del
   correo de prueba —escrito en `comandos.rs`— y uno vacío. **Ninguna campaña
   manda el texto de su plantilla.** La auditoría B ya lo había declarado; aquí
   queda medido otra vez con el nombre de las funciones.
3. **`<img>` no está en la lista blanca del saneado** (`sanear::ETIQUETAS`), ni
   `src`, ni `style`. Hoy una imagen en una plantilla o en una firma se tira al
   guardar.

**Qué significa para lo que se acaba de entregar:** R-03 (negrita y cursiva) y
R-04/R-06 están completos **en el editor y en la base**, que es donde Dirección
los necesitaba para poder escribir y corregir. Lo que no existe es el último
tramo, que es una entrega aparte y no un defecto de éstas: componer el correo
de una campaña con el cuerpo de la plantilla, su firma y su versión en texto
plano.

**Es el patrón del §9 en su versión más larga:** el editor, el saneado, el
versionado, el preflight, el motor, el adaptador SMTP y el envío de prueba
están construidos y probados — y **entre la plantilla y el correo falta el
cable**. Ninguna prueba de ninguna de esas piezas puede verlo.

### H-12 · La firma no llega a ningún correo (encontrado el 17/10/2026)

Mirando qué protege editar una firma salió esto: **`campaign_stage.signature_id`
existe desde la V4 y nada lo escribe.** Medido sobre los ocho crates — las
únicas apariciones de `signature` fuera del módulo de plantillas son el
manifiesto del respaldo y las migraciones.

Así que una firma se guarda, se lista, se corrige… y **no sale en ningún
mensaje**. Es el mismo estado en que está el cuerpo de la plantilla, que la
auditoría B ya declaró: *«el único `Contenido` que llega al SMTP es un texto
fijo escrito en el código»*.

**Qué cambia esto:**

- **No bloquea nada de hoy** y por eso no se inventó un aviso de «esta firma la
  usan N campañas»: hoy ese número es siempre 0, y un aviso que nunca se enseña
  es decorado. Se consideró y se descartó.
- **R-05 (una imagen en la firma) construye sobre algo desconectado**, y
  conviene saberlo antes de empezarlo.
- **El día que el contenido se conecte al envío hay que decidirlo otra vez**, y
  la decisión correcta será la de la plantilla: congelar en la etapa lo que se
  va a mandar. Si en su lugar la etapa apunta a la firma viva, editarla a media
  corrida cambia la firma de los que falten y nada protesta. Está escrito en el
  comentario de `actualizar_firma`, que es donde lo va a leer quien lo toque.

---

# Las tres decisiones que Dirección pidió que asesorara

## 1 · El máximo al día, ¿obligatorio o con un valor por defecto visible?

**Recomiendo: ni obligatorio ni en blanco — que salga relleno con el número de
la cuenta y se pueda cambiar.**

| | Qué pasa | Por qué no |
|---|---|---|
| **Dejarlo como está** | Se acepta un número sin saber cuál | Es H-10 |
| **Hacerlo obligatorio** | Un campo más que rellenar en un asistente de nueve pasos, y quien no sepa qué poner inventará un número | Obliga a decidir a quien no tiene con qué decidir |
| **Rellenarlo con el de la cuenta** ✅ | Se ve el número desde el principio, se puede cambiar, y quien no lo toque sabe qué está aceptando | — |

La cuenta ya dice «Hasta 50 al día · 10 por hora» en su ficha. Ese 50 es el que
debe aparecer en el campo, escrito, no sobreentendido. **Cifras, no adjetivos**
(§94): «el de la cuenta» es un adjetivo; «50» es una cifra.

## 2 · El sitio web de la empresa (de la ronda del 16/10)

**Recomiendo: darle uso como variable de firma**, no quitarlo. Está desarrollado
en H-04.

## 3 · Qué hacer con el spam

**No es un defecto de ARLES, y tampoco se arregla dentro de ARLES.** Un correo
de Gmail enviado a Gmail cae en spam cuando el dominio del remitente no tiene
los tres registros que dicen «este servidor puede mandar en mi nombre»:

| | Qué dice |
|---|---|
| **SPF** | Qué servidores pueden enviar con tu dominio |
| **DKIM** | Una firma que prueba que el correo no se alteró |
| **DMARC** | Qué hacer si los dos anteriores fallan |

Con una cuenta `@gmail.com` **esos registros son de Google, no tuyos**, y Google
marca como sospechoso un correo automatizado que sale de una cuenta personal.
Es lo que pasó.

**La salida real es mandar desde un dominio propio** —`@telemetrymx.com`— con
sus tres registros puestos. Eso es exactamente **la entrega 7.5**, que estaba
aplazada «hasta que se pueda verificar de verdad»: hoy ya se puede, porque R-24
se cerró con el primer correo real.

> **Lo que ARLES puede hacer y hoy no hace:** comprobar esos tres registros y
> decir cuáles faltan, antes de que la campaña salga. El preflight ya tiene el
> punto `registros_del_dominio` reservado desde la Fase 6 — y es uno de los tres
> que hoy se contestan desde `arles-app` sin mirar nada.

---

## H-06 y H-10 · arreglados el 17/10/2026

### El horario se crea desde donde hace falta

El paso 5 ya no deja a nadie sin salida. Si no hay ningún horario guardado,
ofrece **el de oficina a un botón** —lunes a viernes, 9:00 a 18:00— y debajo,
plegado, la forma de definir otro. El que se crea **queda elegido**: dejarlo
creado y sin elegir obligaría a desplegar y seleccionar el único que hay.

Se resuelve **dentro del asistente** y no en otra pantalla a propósito: mandar
a definir un horario a otra sección, a mitad de nueve pasos, es perder lo
tecleado o volver con el hilo roto.

La zona horaria es **la de la empresa**, nunca la del equipo — es la decisión
que la pantalla de Configuración explica en su propio aviso.

### Y el máximo diario dice su cifra

«Déjalo en blanco para usar el de la cuenta» pasa a «Déjalo en blanco y se usan
**50** al día, que es el tope de la cuenta que elegiste». Lo mismo por hora.

**Sin cuenta elegida no se inventa una cifra**: ahí se queda la frase de antes,
porque todavía no hay tope que nombrar. Hay una prueba de eso.

### La vista previa no podía enseñar este defecto

Su demo **trae una ventana ya guardada**, así que ahí el desplegable nunca
estuvo vacío. Es un hueco del decorado, no del producto: **un decorado con más
datos que la realidad esconde justo los caminos de quien empieza**, que son los
que más fallan.

Queda anotado como propuesta: que la vista previa tenga un modo «cuenta nueva»,
con la base como la encuentra alguien que acaba de instalar.

### Cuatro pruebas, dos vistas en rojo

Quitando la salida falla con «expected … to contain "Todavía no has definido
ningún horario"»; quitando la cifra, con «expected … to contain "se usan 50 al
día"».

> **Y un error mío en medio, el peor de la sesión.** Al restaurar la primera
> ruptura escribí un `git checkout -- AsistenteDeCampana.vue`, que **revirtió
> el archivo entero** y se llevó todo el arreglo de H-06 y H-10 — no sólo la
> ruptura.
>
> Es **exactamente** lo que el `CLAUDE.md` tiene escrito desde la 8.3 bis a:
> *«para restaurar una ruptura se usa una copia del archivo hecha antes, nunca
> `git checkout` sobre un archivo con trabajo sin confirmar»*. La lección estaba
> escrita y la repetí.
>
> El trabajo se rehízo entero y las rupturas siguientes ya fueron contra una
> copia. Lo que lo hizo recuperable fue que los textos y las pruebas estaban en
> otros archivos: si todo hubiera vivido en el mismo, se habría perdido también
> la forma de saber qué faltaba.

---

## H-07, H-08 y H-09 · arreglados el 17/10/2026

### H-07 · la ficha envuelve en vez de estrujarse

`.remitentes__quien` tenía `flex: 1` con `min-width: 0`, así que podía
encogerse hasta nada: con el panel del correo de prueba al lado, la línea de
configuración caía una palabra por renglón.

Ahora la fila lleva `flex-wrap: wrap` y el bloque de texto una base de 18rem.
Cuando no hay sitio, **los botones bajan a la línea siguiente** en vez de
robarle ancho al texto.

### Y la medida que faltaba, que nació sin medir nada

`sonda:encimado` no lo cazaba, y era información: sus cuatro medidas miran si
algo desborda, se corta, asoma o lo recortan unos puntos. Esto **cabe**. Está
estrujado, que es otra cosa.

La medida 5 cuenta **líneas contra palabras**: si son casi iguales, cada
palabra está cayendo en su renglón. Llegar a eso costó tres intentos, y los
tres fallos valen más que el resultado:

| | Qué hacía | Por qué estaba mal |
|---|---|---|
| **1ª** | Preguntaba si caben «dos palabras largas», medidas con un `canvas` | Dio **19 hallazgos en Bajas** y la tabla se lee bien. Era el umbral acusando a un texto correcto |
| **2ª** | Dividía `scrollHeight` entre el alto de línea | `scrollHeight` **incluye el relleno**, y en un `<td>` **el alto de la fila entera**. Tres líneas parecían cinco |
| **3ª** ✅ | Un `Range` sobre el contenido: **un rectángulo por línea pintada** | No depende del relleno ni de la tabla |

> **Y la medida no podía saltar nunca.** Escribí `lineas >= palabras * 0.8`, y
> `palabras` es un **array**: multiplicarlo da `NaN`, y `6 >= NaN` es `false`.
> Habría vivido en verde para siempre sin mirar un solo texto.
>
> **No lo encontró leer el código**: lo encontró romper la pantalla a propósito
> y ver que la sonda seguía en verde. Es el §2 en su forma literal.

> **Y una guarda mía se comía el caso peor.** La recogida de candidatos
> descartaba lo que tuviera `width < 1`, para saltar lo invisible. Un bloque sin
> ancho y con 120 px de alto —el síntoma exacto, seis palabras en seis líneas—
> entraba justo por ahí. Ahora se exige **alto**, no ancho.

Verificada reproduciendo la ficha como la vio Dirección: caza las tres líneas,
nombrando «cae en 6 líneas teniendo 6 palabras».

### H-08 · el contador no subía porque nadie volvía a preguntar

`empresa.cargar()` tiene un `if (cargada.value) return`, así que la lista de
los seis pasos se quedaba con **la foto del arranque**. El recuento es correcto
en la base —`recuento_de_alta` cuenta `email_account`— pero nadie lo volvía a
pedir.

Inicio llama ahora a `refrescar()`, que es la misma carga sin el atajo. Es la
pantalla a la que se vuelve **justo después** de completar un paso.

Es la hermana del H-01: allí un aviso sobrevivía a lo que lo resolvía; aquí un
contador no sube cuando acabas de hacer el paso, y eso hace dudar de si quedó
hecho.

### H-09 · el hueco entre campos, de 16 a 24

La ayuda de un campo y la etiqueta del siguiente son **las dos texto pequeño**,
y entre ellas había los 16 px del módulo mientras dentro del campo hay 4.
Cuatro veces más no basta cuando lo que separa y lo que junta se parecen tanto.

Se añaden 8 px **sólo entre campos consecutivos**, no al `gap` del módulo:
subir ése alejaría también el título de su primer campo. 24 contra 4 es seis
veces, y el título se queda donde estaba — que es el «equilibrado» que Dirección
pidió.

Comprobado mirando Configuración y Remitentes, y `sonda:recorrido` sigue en
verde: el cambio no desalineó ninguna de las dieciocho pantallas.
