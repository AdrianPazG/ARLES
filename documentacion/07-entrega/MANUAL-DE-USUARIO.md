# Manual de ARLES RELAY I

**Para quien va a usar ARLES, no para quien lo programa.**

Si buscas qué archivo dibuja cada pantalla o qué comando llama cada botón, eso
está en [`MAPA-FUNCIONAL.md`](../05-diseno/MAPA-FUNCIONAL.md). Esto de aquí
cuenta **qué puedes hacer con ARLES y en qué orden**, en español llano.

> **Fecha: 14/10/2026.** Lo que dice este manual se corresponde con lo que hay
> construido hoy. La última sección, «Lo que ARLES todavía no hace», no es una
> disculpa: es la parte más importante del manual, porque es lo que impide
> prometer algo que no se puede cumplir.

---

## 1 · Qué es ARLES, en dos frases

ARLES manda campañas de correo y de WhatsApp **desde tu computadora**, a tu
ritmo y con tus propias cuentas. No es una plataforma en la nube: no hay
servidor de nadie en medio, los contactos viven en tu disco cifrados, y nadie
más los ve.

**Qué no es:** no es un buzón ni un CRM. ARLES no lee tus correos, no recibe
respuestas y no te dice si alguien abrió un mensaje — y eso último es
deliberado, lo explica la sección 8.

---

## 2 · Lo primero que verás al abrirlo

ARLES abre en **Inicio**. Si es la primera vez, enseña «Empieza por aquí» y un
solo botón, porque sin los datos de tu empresa no se puede hacer nada más.

Una vez configurada, Inicio enseña **los 6 pasos** y cuántos llevas. Ése es el
orden del producto y no es decorativo: cada paso necesita el anterior.

```
   1. Los datos de tu empresa
   2. Tus contactos
   3. Un remitente (una cuenta de correo) o un número de WhatsApp
   4. Una plantilla: el mensaje que vas a mandar
   5. Una campaña: a quién, con qué, a qué ritmo
   6. Activarla
```

---

## 3 · Paso a paso, la primera campaña

### Paso 1 · Los datos de tu empresa

**Inicio → Configuración.** Nombre comercial, país, zona horaria, correo
corporativo y sitio web.

La **zona horaria** no es un detalle: decide a qué hora se mandan los mensajes.
Si dice Madrid y tú estás en Monterrey, la ventana de envío de las 9 de la
mañana sale a las 2 de la madrugada.

Se guarda en tu disco. Puedes cerrar ARLES y seguir mañana.

### Paso 2 · Tus contactos

**Contactos → Importar.** Arrastras un `.csv` o un `.xlsx`.

Tres cosas que conviene saber antes de soltar el archivo:

- **ARLES te enseña los repetidos antes de importar nada.** No los junta ni los
  salta por su cuenta: te los pone delante y tú decides. Es una decisión de
  Dirección del 18/09/2026, y la razón es que una importación de diez mil filas
  no se puede deshacer en la práctica.
- **El número de México va sin el «1».** `+52 81 1234 5678`, no `+52 1 81…`.
  ARLES lo corrige solo y te lo dice.
- **Un contacto puede tener varios correos y varios WhatsApp.** Se agrupan solos
  en su ficha; haz doble clic en una fila para verlos todos.

**No hace falta ordenar la tabla antes.** ARLES reconoce las columnas por su
nombre —«Nombre», «Razón social», «Correo», «Teléfono», «Celular»…— y deja
fuera las que no necesita, estén donde estén. Si la tabla del cliente trae
nueve columnas y ARLES usa tres, se cargan las tres.

> **Y si viene hecha un desastre, hay plantilla.** En ese mismo paso, al lado
> de «Elegir archivo», está **«Descargar la plantilla»**: un Excel con tres
> columnas —Nombre, Correo y Teléfono— donde pegar lo que sirva. No es
> obligatoria.

**Si una columna se llama «Teléfono» a secas, ARLES te avisa.** La va a usar
para WhatsApp, y ese nombre no dice si son celulares: si ahí hay teléfonos
fijos, WhatsApp no los alcanza y los intentos fallidos bajan la calificación de
tu número ante Meta. Lo que dice a las claras no ser un móvil —«Teléfono fijo»,
«Conmutador», «Lada», «Fax»— se deja fuera sin preguntar.

### Paso 3 · Tu cuenta de correo

**Remitentes → Agregar cuenta.** Desde el 14/10/2026 son **dos campos**: tu
correo y tu contraseña.

ARLES reconoce Gmail, Outlook, Yahoo, Zoho e iCloud por el dominio y rellena
solo el servidor, el puerto y el cifrado. Si usas otro proveedor, se abre sola
una sección de «Configuración avanzada» donde pedírtelos — porque ahí sí hacen
falta.

> **Con Gmail, Outlook y Yahoo la contraseña NO es la de tu cuenta.**
>
> Hay que crear una **contraseña de aplicación**, que es distinta y de un solo
> uso para ARLES. En Gmail son 16 letras sin espacios, y ARLES te avisa si lo
> que escribiste no las tiene.
>
> Se crea en la configuración de seguridad de tu cuenta de correo, y **hace
> falta tener activada la verificación en dos pasos** — sin eso, tu proveedor
> ni siquiera ofrece la opción.

**Tu contraseña no se guarda en ARLES.** Va al llavero del sistema —el de
Windows o el de macOS, el mismo donde el navegador guarda las tuyas—, y ningún
botón de ARLES puede volver a enseñarla. Es a propósito.

Antes de seguir, usa **«Mandar un correo de prueba»**: te manda uno a ti mismo.
Si llega, la cuenta está lista.

> **Si el correo te cae en «no deseados», lee esto antes de tocar nada.**
>
> Medido el 22/10/2026 con cabeceras reales: mandando desde una dirección
> `@gmail.com`, Outlook lo manda a no deseados **aunque toda la autenticación
> esté perfecta**. Lo que pesa es el patrón «nombre de tu marca + dirección
> personal de Gmail», que para un filtro es la firma de una suplantación.
>
> **No se arregla con configuración.** Se arregla mandando desde una dirección
> de tu propio dominio. El mismo día, desde `enlace@telemetrymx.com`, el correo
> llegó a la **bandeja principal** de Outlook, Gmail y un corporativo — tres de
> tres. Los pasos están en
> [`PASO-A-PASO-DEL-CORREO.md`](PASO-A-PASO-DEL-CORREO.md).

### Paso 4 · Tu mensaje

**Campañas → Plantillas y firmas.**

Puedes escribir `{{nombre}}`, `{{empresa}}` y las demás variables, y ARLES las
sustituye por los datos de cada contacto.

> **Una plantilla guardada no se edita: se crea una versión nueva.**
>
> Suena incómodo y es la única forma de que una campaña que ya salió siga
> diciendo lo que decía cuando salió. Si pudieras editarla, el día que alguien
> te pregunte «¿qué me mandaste?» no habría forma de saberlo.

### Paso 5 y 6 · La campaña

**Campañas → Nueva campaña.** Son nueve pasos y el asistente te lleva.

Tres cosas que te va a decir y conviene entender:

**El ritmo.** Cuántos mensajes al día y cuántos por hora. Los dos campos vienen
rellenos con lo que trae tu cuenta y **puedes cambiarlos**: el número que
escribas es el que ARLES va a respetar.

Si pones más de 50 al día, ARLES te hace aceptar un aviso: por encima de esa
cifra los proveedores de correo empiezan a tratarte como remitente masivo, y lo
que se pierde no es un mensaje, es la reputación de tu dominio. **Te avisa y no
te lo impide** — la decisión es tuya, y queda registrada con su fecha.

> **Si ese remitente ya está enviando otra campaña, ARLES te lo dice ahí
> mismo.** El máximo al día es **de la cuenta de correo**, no de cada campaña:
> dos campañas de 50 sobre el mismo remitente sacan **25 y 25**, no 50 cada
> una, así que la tuya va a tardar más de lo que dice el plazo. **No salen más
> correos en total: salen repartidos.** Si necesitas que cada campaña vaya a su
> ritmo, usa un remitente distinto para cada una.

**La firma.** En el paso 4 eliges cuál va al pie. Si no eliges ninguna, se usa
la que marcaste por defecto en Plantillas y firmas.

**Si la campaña puede terminar.** En el paso 6, ARLES calcula cuánto va a
tardar con el ritmo que elegiste. Si no termina en cinco años, **no te deja
activarla** y te dice cuántas personas se quedarían sin recibir nada. Es la
cuenta que nadie hace a mano.

**Al activar, la audiencia se congela.** Los contactos que entren después no
reciben esa campaña. Así, si mañana importas mil contactos nuevos, no les llega
de golpe un mensaje que no esperaban.

---

## 4 · Mientras la campaña corre

**Actividad** enseña una tarjeta por campaña: cuántos se mandaron, cuántos
fallaron, cuánta gente contestó, y **por qué no sale nada** si no sale —que es
la pregunta de verdad—. Hay seis razones posibles y ARLES te dice cuál es, con
su cifra.

**El botón rojo de Detener** está en Campañas. Para todo, de todas las
campañas, al momento. Se vuelve a levantar cuando quieras.

**Registro de envíos** es la lista completa: cada intento, con su desenlace y su
fecha. Se puede filtrar por seis criterios y nunca enseña más de 500 filas de
golpe, con el total al lado.

---

## 5 · Lo que ARLES guarda, y dónde

| Qué | Dónde |
|---|---|
| Contactos, campañas, historial | Un archivo cifrado en tu computadora |
| Las contraseñas de tus cuentas | El **llavero del sistema**, nunca en el archivo |
| El resumen de una campaña | Donde tú lo exportes, cuando lo pidas |

La ruta del archivo, por si necesitas respaldarla a mano:

- **Windows:** `%APPDATA%\TelemetryInsight\ArlesRelay\data\arles.db`
- **macOS:** `~/Library/Application Support/mx.TelemetryInsight.ArlesRelay/arles.db`

> **Copiar ese archivo a otra computadora no sirve de nada.** Está cifrado con
> una clave que vive en el llavero de **esta**. Para mudarte, usa un respaldo.

---

## 6 · Respaldos: hazlos

**Ajustes → Respaldos.** Un archivo `.arles` con todo lo que elijas.

- **«Esencial»** trae contactos, bajas, campañas y plantillas. Es lo que querrás
  casi siempre.
- **«Completo»** trae además todo el historial de envíos.
- Puedes **ponerle contraseña**, y deberías.

> **La contraseña del respaldo no se guarda en ningún sitio.** Ni en ARLES, ni
> en el llavero, ni en el archivo. Si se te olvida, ese respaldo no se abre
> nunca más. No hay puerta de atrás, y eso es lo que lo hace seguro.

**Para restaurar:** Ajustes → Restaurar. ARLES te enseña **primero** qué trae el
archivo y qué se va a perder, y sólo entonces te deja confirmar.

> **Restaurar reemplaza, no mezcla.** Lo que había se va. La única excepción son
> las bajas: ésas se **suman**, nunca se pierden, porque alguien que pidió no
> recibir nada no puede volver a tu lista por un respaldo viejo.

---

## 7 · Si algo va mal

| Lo que ves | Qué hacer |
|---|---|
| **La ventana abre vacía** o dice «localhost refused to connect» | Se construyó mal. Ver [`COMO-ABRIR-ARLES.md`](COMO-ABRIR-ARLES.md) |
| **«Fallo al migrar el esquema»** | Tienes una base de una versión anterior. El mensaje te dice la ruta y qué renombrar |
| **«No se pudo cifrar la conexión con el servidor»** | El servidor, el puerto o el cifrado no cuadran. Abre «Configuración avanzada» en Remitentes |
| **Una pantalla de recuperación al abrir** | ARLES no encuentra su clave en el llavero. **No borres nada**: esa pantalla te deja restaurar un respaldo, y aparta la base vieja sin destruirla |

Los siete tropiezos que ya nos han pasado de verdad, con su solución exacta,
están en [`REGISTRO-DE-TROPIEZOS.md`](REGISTRO-DE-TROPIEZOS.md).

**Todo error de ARLES dice tres cosas**: qué pasó, cómo se arregla, y **qué está
a salvo**. Si lees uno que no dice las tres, eso es un defecto nuestro y
conviene reportarlo.

---

## 8 · Lo que ARLES todavía no hace

Esta sección es la más importante del manual, y está aquí para que no prometas
nada que no se pueda cumplir.

| Qué | Por qué |
|---|---|
| **No dice si un correo llegó** | Nadie puede saberlo. ARLES dice «aceptado» —el servidor lo tomó— y nunca «entregado», porque sería mentira |
| **No dice si alguien abrió un mensaje** | Eso exige un píxel espía. Se decidió no ponerlo |
| **No se puede lanzar una campaña entera por WhatsApp** | Sólo el envío de prueba. El motor todavía recorre únicamente el correo |
| **No se puede filtrar la audiencia** de una campaña | Entra la lista entera. El filtro del **registro** sí existe, con seis criterios |
| **Una campaña creada no se puede editar** | Hay que crear otra |
| **No se puede restaurar un respaldo de una versión anterior** | Se rechaza con un error claro en vez de restaurar algo a medias |
| **Nadie ha abierto ARLES en un Mac** | Está construido para macOS y **no se ha probado ahí**. Hasta que alguien lo haga, no se puede afirmar que funcione |
| **Una firma no puede llevar una imagen** | Sólo texto, con negrita y cursiva. Está pendiente de decidir cómo: embebida engorda cada correo y dispara filtros de spam; alojada hay que alojarla en algún sitio |
| **El número de WhatsApp nunca ha hablado con Meta** | Todo lo de WhatsApp está probado contra un servidor de mentira. Falta una cuenta de empresa verificada con Coexistencia |

> **Y una que dejó de ser cierta el 22/10/2026, porque conviene saberlo:**
> hasta ese día ARLES **nunca había hablado con un servidor de correo real**.
> Ahora sí: manda desde una cuenta de Titan por SMTP con TLS, y el correo llega
> a la bandeja principal de Outlook, Gmail y un corporativo.

---

## 9 · Dos cosas que ARLES no te va a dejar hacer

Y conviene saber por qué, porque van a parecer tozudez:

1. **No arranca si no encuentra el llavero del sistema.** Podría arrancar
   guardando las contraseñas en un archivo de texto; eso es exactamente lo que
   no va a hacer nunca.

2. **Nada antes de que el servidor acepte al destinatario puede dar de baja a
   nadie.** Un error de red no es una baja. Si lo fuera, un problema de tu
   conexión borraría gente de tu lista para siempre.

---

## 10 · Dónde seguir

| Para | Mira |
|---|---|
| Abrir ARLES por primera vez | [`COMO-ABRIR-ARLES.md`](COMO-ABRIR-ARLES.md) |
| Probarlo sin compilar nada | [`COMO-PROBAR-ARLES-SIN-COMPILAR.md`](COMO-PROBAR-ARLES-SIN-COMPILAR.md) |
| Conectar correo y WhatsApp de verdad | [`GUIA-DE-CONEXION-REAL.md`](GUIA-DE-CONEXION-REAL.md) |
| Qué hace cada pantalla, por dentro | [`MAPA-FUNCIONAL.md`](../05-diseno/MAPA-FUNCIONAL.md) |
| El estado del proyecto, en gráficos | `tablero/ARLES-tablero.html` (doble clic) |
| Un error que encontraste | [`BITACORA-DE-PRUEBAS.md`](../06-calidad/BITACORA-DE-PRUEBAS.md) |
