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
| **R-01** | 🔵 Requisito | Remitentes | Dar de alta un correo es **demasiado complicado**. Tiene que ser sencillo e intuitivo | 📋 **Abierto** |

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

### Estado

📋 **Abierto**, esperando decisión de Dirección sobre cuáles de las cinco.
