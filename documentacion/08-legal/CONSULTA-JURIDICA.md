# Consulta jurídica

> Dieciocho preguntas concretas sobre protección de datos en México

@@ PARA | El despacho o abogado que contrate TELEMETRY INSIGHT
@@ DE | El equipo de ARLES RELAY I
@@ DE QUÉ VA | Enviar correo y WhatsApp comercial en México, dentro de la ley
@@ VERSIÓN | 2 · 22 de septiembre de 2026
@@ QUÉ CAMBIÓ | Nueve preguntas nuevas · sección 12
@@ LA MÁS URGENTE | C-3 · sección 11

=== 1 · Léase esto primero

Este documento está escrito **para mandarse tal cual**.

No le pedimos un dictamen sobre protección de datos. Le pedimos **dieciocho
respuestas concretas**, y en cada una le decimos exactamente qué vamos a hacer
con lo que nos conteste. La mayoría cambian un texto o un plazo. **Cuatro
cambian el producto.**

Está escrito en lenguaje llano a propósito. Quien lo lee del lado de TELEMETRY
no es abogado, y quien lo lee del lado de usted no es programador. Hemos
intentado que las dos personas entiendan lo mismo.

!i **Las preguntas se numeran C-1 a C-18** —«C» de consulta—. Si ya vio la
versión 1 de este documento, las ocho primeras son las mismas y en el mismo
orden: la que antes era «P-3» es ahora **C-3**, y sigue siendo la más urgente.
La sección 12 dice exactamente qué se añadió.

=== 2 · Qué es ARLES, sin tecnicismos

ARLES es **un programa que se instala en una computadora**. No es una página
web ni un servicio en la nube.

Sirve para una cosa: **mandar correos —y mensajes de WhatsApp— a una lista de
contactos**, de forma ordenada y con límites de velocidad, para ofrecer un
producto o un servicio.

Cuatro cosas que importan para sus respuestas:

- **Nosotros no vemos ningún dato.** Todo vive en el disco de quien usa el
  programa, cifrado. ARLES no tiene servidores. Si un cliente carga diez mil
  contactos, esos diez mil contactos jamás salen de su computadora **hacia
  nosotros**. Sí salen hacia su proveedor de correo — ver **C-14**.
- **Los correos salen de la cuenta del propio cliente**, no de una nuestra.
  ARLES usa su correo corporativo, igual que lo haría Outlook.
- **Muchos de esos contactos nunca dieron permiso.** Es correo comercial en
  frío, dirigido a empresas. Ése es el punto que más nos preocupa: **C-3**.
- **Quien usa ARLES hoy es sólo TELEMETRY INSIGHT**, internamente. Venderlo a
  otras empresas llega en una versión posterior, y **entendemos que eso es otra
  consulta**.

=> EN UNA FRASE :: Somos el fabricante de la herramienta. **Entendemos que quien responde ante la ley por los datos es el cliente que la usa** — hoy, nosotros mismos. Si esa lectura es incorrecta, díganoslo: cambia todo lo demás.

=== 3 · Qué datos guarda ARLES, exactamente

Incluimos esta tabla porque suponemos que la necesita para contestar casi
cualquier pregunta. **Es la lista completa**, sacada de la estructura real del
programa, no un resumen.

| Qué se guarda | De cada persona | Notas |
|---|---|---|
| **Correo electrónico** | Tal como se escribió, y normalizado en minúsculas | Puede haber varios por persona |
| **Número de WhatsApp** | Tal como se escribió, y normalizado | Puede haber varios |
| **Nombre y apellido** | Si venían en la lista | Opcionales |
| **Empresa** | Texto libre | Opcional |
| **Campos personalizados** | **Cualquier columna** que el usuario decida importar, con el nombre que quiera | **Ver C-18**: aquí es donde podría colarse un dato sensible |
| **Origen de la lista** | De qué archivo salió, con qué nombre, qué día, y qué declaró quien la importó | Con una huella del archivo, para demostrar que no se alteró |
| **Permisos registrados** | Qué permiso consta, de qué tipo, cuándo y en qué se apoya | Ver **C-9** |
| **Bajas** | Su dirección, por qué dejó de recibir y desde cuándo | Ver **C-8** |
| **Envíos** | A qué dirección, qué día, en qué estado acabó y qué contestó el servidor | Ver **C-16** |
| **Bitácora** | Qué se hizo y cuándo. Las direcciones van **cifradas en una huella**, nunca en claro | No se puede borrar ni modificar |

**Cómo está protegido:** la base de datos está **cifrada en el disco**. Las
contraseñas de correo **no se guardan en la base**: van al llavero del sistema
operativo —Keychain en Mac, Credential Manager en Windows—. Si el llavero no
está disponible, **el programa no arranca**: nunca guarda una contraseña en
texto plano.

=== 4 · Lo que ya averiguamos

Esto lo dimos por bueno leyendo el Diario Oficial y análisis de despachos.
**No le pedimos que lo investigue de cero: le pedimos que nos lo confirme o
nos corrija.**

| | Lo que entendemos | De dónde lo sacamos |
|---|---|---|
| **a** | La ley de protección de datos de 2010 **ya no existe**. Rige una **ley nueva**, publicada el 20 de marzo de 2025 y en vigor desde el día siguiente | Diario Oficial, y análisis de Garrigues, EY e IDC |
| **b** | El **INAI se extinguió**. Ahora la autoridad es la **Secretaría Anticorrupción y Buen Gobierno** | Decreto constitucional del 20 de diciembre de 2024 |
| **c** | El **reglamento de la ley nueva todavía no se publica** | Análisis de despachos, a la fecha de este documento |
| **d** | Para defenderse ya no hay juicio de nulidad, sino **juicio de amparo** ante tribunales especializados | Análisis de despachos |

=> SI ALGO DE ESTO ESTÁ MAL :: Decírnoslo **ya es una respuesta valiosa**, y cambia lo que escribimos dentro del programa.

=== 5 · Bloque A · El marco: qué ley y a quién aplica

### C-1 · ¿Sigue sirviendo el reglamento viejo mientras no salga el nuevo?

**La pregunta, en una frase.** La ley nueva es de 2025 y su reglamento aún no
se publica. ¿Nos seguimos guiando por el reglamento de 2011, o ya no vale?

**Por qué la hacemos.** El reglamento de 2011 es el que dice, con detalle, **qué
tiene que decir un aviso de privacidad** y **en cuántos días hay que responder**
cuando alguien pide sus datos. Si sigue valiendo, tenemos una guía clara. Si no,
sólo tenemos el texto de la ley, que es mucho más escueto y deja más al criterio.

**Qué haremos con su respuesta.** Decide qué norma citamos en el aviso de
privacidad que sale en cada correo, y **qué plazos programamos** para que el
programa avise: «llevas 12 días sin responder a esta solicitud».

### C-2 · ¿Hay que darse de alta en algún registro antes de operar?

**La pregunta, en una frase.** ¿Tenemos que inscribirnos, notificar o pedir
permiso a la Secretaría Anticorrupción y Buen Gobierno antes de mandar el primer
correo?

**Por qué la hacemos.** Con el INAI no existía ningún registro previo para
empresas privadas. Queremos confirmar que con la autoridad nueva sigue sin
haberlo — y si lo hay, enterarnos ahora.

**Qué haremos con su respuesta.** Si hay trámite, **es lo primero que hay que
hacer**, antes de la primera campaña. Si nos enteramos después, llevaríamos
meses operando sin cumplir.

### C-10 · ¿El correo de una empresa es un dato personal?

!i **Pregunta nueva en la versión 2.**

**La pregunta, en una frase.** En una misma lista importada conviven dos cosas
muy distintas:

- `contacto@ferreteria.mx` — no identifica a ninguna persona concreta
- `juan.perez@ferreteria.mx` — identifica a Juan Pérez

**¿Las dos son datos personales, o sólo la segunda?** ¿Y cambia algo si el
destinatario es una persona física con actividad empresarial —un taller, un
consultorio— en vez de una sociedad?

**Por qué la hacemos.** Porque si los datos de personas morales quedan fuera
del alcance de la ley, **una parte de la lista de nuestros clientes no estaría
sujeta a estas obligaciones**, y la otra sí. Necesitamos saber si hay que
tratarlas distinto.

**Qué haremos con su respuesta.** Hoy ARLES **no distingue** entre las dos: las
guarda mezcladas y las trata igual. Si la distinción importa legalmente, hay que
construir algo que las separe —o al menos que lo advierta—, y eso es una
función nueva, no un texto.

=> POR QUÉ ES IMPORTANTE :: Es la pregunta que decide **cuánto de todo lo demás aplica**. Si la mitad de una lista B2B queda fuera, cambia el tamaño del problema.

=== 6 · Bloque B · De dónde salen los contactos

!! **Este bloque es el corazón del asunto.** Si las respuestas aquí son
negativas, no cambia un texto del programa: **cambia el producto**.

### C-3 · ¿Se puede escribir a alguien cuyo correo salió de un directorio público?

**La pregunta, en una frase.** El INEGI publica gratis un directorio de
empresas —el DENUE— con el correo y el teléfono de cada establecimiento.
**¿Podemos usar esos datos para mandar publicidad, sin haber pedido permiso
antes?**

**Por qué la hacemos.** Porque **es el supuesto central del producto**. La ley
anterior lo permitía expresamente cuando los datos venían de una fuente de
acceso público. Necesitamos saber tres cosas de la ley nueva:

1. ¿Mantiene esa excepción?
2. Si la mantiene, ¿con qué límites?
3. ¿Que un dato sea público lo hace utilizable **para publicidad**, o sólo para
   la finalidad por la que se publicó?

**Qué haremos con su respuesta.**

- Si es **sí**: seguimos como estamos.
- Si es **no**, o es **«sí, pero…»**: hay que construir una compuerta nueva en
  el programa que **impida lanzar una campaña sin prueba de consentimiento**.
  Eso es trabajo de semanas, no un texto.

=> POR ESO ÉSTA NO SE PUEDE APLAZAR :: Construir la parte de campañas asumiendo que se puede, y descubrir después que no, obliga a rehacerla entera. Ver la sección 11.

### C-11 · ¿Y si la lista se compró?

!i **Pregunta nueva en la versión 2.**

**La pregunta, en una frase.** ARLES deja elegir el origen de cada lista, y una
de las opciones es **«Otro origen»**. Ahí cabe una lista comprada a un tercero.
**¿Se puede usar? ¿Y qué expone a quien la use?**

**Por qué la hacemos.** Es un caso distinto del directorio público: aquí no hay
una fuente abierta, hay un intermediario que afirma haber obtenido los datos
lícitamente. Queremos saber si esa afirmación de un tercero **traslada la
responsabilidad** o si quien envía responde igual.

**Qué haremos con su respuesta.** Si no es defendible, **quitamos esa opción del
programa** o le ponemos una advertencia que haya que aceptar expresamente. Hoy
está ahí, sin fricción.

### C-12 · ¿Tiene algún valor que el usuario declare de dónde sacó la lista?

!i **Pregunta nueva en la versión 2.**

**La pregunta, en una frase.** Cada vez que alguien carga una lista, ARLES le
obliga a declarar de dónde salió y a aceptar un texto. Guardamos ese texto
íntegro, con una huella que demuestra que no se alteró, y la fecha.

**¿Eso tiene valor probatorio, o es puro teatro?**

**Por qué la hacemos.** Porque construimos ese mecanismo suponiendo que sirve
de algo, y **no lo hemos validado con nadie**. Si no sirve, preferimos saberlo
ahora que después de que un cliente lo use como defensa.

**Qué haremos con su respuesta.**

- Si sirve: **necesitamos que usted nos escriba el texto exacto** que el usuario
  debe aceptar. Hoy ese texto está marcado dentro del programa como *«todavía no
  lo ha revisado un abogado»*, visible para quien lo usa.
- Si no sirve: hay que decidir si se quita o se sustituye por otra cosa.

=== 7 · Bloque C · El envío

### C-4 · ¿Qué tiene que decir, palabra por palabra, el aviso de privacidad?

**La pregunta, en una frase.** Cuando le escribimos a alguien cuyos datos NO nos
dio él directamente, ¿qué texto exacto tiene que leer en ese correo?

**Por qué la hacemos.** Nosotros no tratamos los datos — los trata el cliente.
Pero **ARLES es quien redacta el texto que sale en el correo**, y queremos que
salga correcto por defecto, sin que cada cliente tenga que pagarse un abogado.

**Qué haremos con su respuesta.** Será el texto que el programa propone por
defecto, editable por quien lo use. **Necesitamos que nos lo entregue
redactado**, listo para pegar.

!x **No tenemos borrador.** En la versión 1 de este documento ofrecíamos
enseñarle «el aviso de privacidad actual». Al revisarlo comprobamos que **no
existe**: el programa todavía no incluye ninguno. Lo corregimos aquí para no
hacerle perder el tiempo.

### C-13 · ¿Dónde tiene que estar disponible el aviso de privacidad?

!i **Pregunta nueva en la versión 2.**

**La pregunta, en una frase.** ¿Basta con incluir el texto completo dentro de
cada correo, o **tiene que estar publicado en una página web** a la que el
correo enlace?

**Por qué la hacemos.** ARLES es un programa de escritorio. **Un cliente puede
perfectamente no tener sitio web.** Si la ley exige una dirección pública
permanente, ese cliente no puede cumplir usando sólo ARLES, y tenemos que
decírselo antes de venderle nada.

**Qué haremos con su respuesta.** Si hace falta una página, el programa tendría
que **pedir esa dirección y no dejar enviar sin ella**. Es una compuerta nueva.

### C-5 · ¿El WhatsApp comercial tiene reglas distintas a las del correo?

**La pregunta, en una frase.** Mandar un WhatsApp a alguien que no lo pidió,
¿exige algo más en México que mandarle un correo?

**Por qué la hacemos.** WhatsApp es más invasivo: va a un número que suele ser
personal y suena en el bolsillo. Queremos saber si hay alguna obligación
adicional —de la ley de datos, de PROFECO o de donde sea— **más allá de las
reglas de Meta**, que ya conocemos y cumplimos.

**Qué haremos con su respuesta.** Decide si el modo de prospección por WhatsApp
necesita una advertencia distinta dentro del programa, o si hay que restringirlo.

!i **Contexto que quizá le sirva:** Meta prohíbe contractualmente el mensaje
comercial en frío. No es una cuestión legal sino de contrato, y ya lo tenemos
resuelto por nuestro lado. Le preguntamos sólo por la parte legal mexicana.

### C-6 · ¿Hay que consultar el REPEP antes de enviar?

**La pregunta, en una frase.** El REPEP es el registro donde la gente se apunta
para que no le llamen a vender. **¿Estamos obligados a consultarlo antes de cada
campaña?** ¿Y alcanza al correo y al WhatsApp, o sólo al teléfono?

**Por qué la hacemos.** Sabemos que cubre llamadas telefónicas. No sabemos si
llega al correo, ni si es exigible cuando el destinatario es una empresa y no
un particular.

**Qué haremos con su respuesta.** Si es exigible, **es una función nueva que no
existe**: habría que cruzar cada lista contra ese registro antes de enviar. Hay
que planearlo antes de cerrar el alcance de esta versión.

### C-14 · El proveedor de correo ve los datos. ¿Eso es una transferencia?

!i **Pregunta nueva en la versión 2.**

**La pregunta, en una frase.** ARLES envía **a través de la cuenta de correo del
propio cliente** — normalmente Gmail o un servidor de su proveedor. Eso
significa que **el nombre y el correo de cada destinatario pasan por los
servidores de ese proveedor**, que muchas veces están fuera de México.

¿Eso cuenta como una transferencia de datos que haya que declarar, contratar o
advertir?

**Por qué la hacemos.** Decimos con orgullo que «los datos no salen de tu
computadora», y es cierto respecto a **nosotros**. Pero **no es cierto respecto
al proveedor de correo**, y no queremos que esa frase se convierta en una
afirmación falsa en un aviso de privacidad.

**Qué haremos con su respuesta.**

- Si hay que declararlo: entra en el texto del aviso de privacidad (C-4).
- Si hace falta algún contrato con el proveedor: hay que decírselo al cliente
  **antes** de que conecte su cuenta, dentro del programa.
- Y corregimos nuestro material comercial, que hoy se apoya en esa frase.

=== 8 · Bloque D · Los derechos de quien recibe el correo

!i **Nota de vocabulario.** A los cuatro derechos de una persona sobre sus datos
se les llama **derechos ARCO**: **A**cceso —saber qué tienes de mí—,
**R**ectificación —corrígelo—, **C**ancelación —bórralo— y **O**posición —deja
de usarlo—. Los mencionamos así en las cuatro preguntas que siguen.

### C-7 · ¿Cuántos días tenemos para responder, y en qué forma?

**La pregunta, en una frase.** Con la ley nueva, ¿en cuánto tiempo hay que
contestar a cada uno de los cuatro derechos, y hay una forma obligatoria de
hacerlo?

**Por qué la hacemos.** El programa ya lleva esos plazos programados, pero
contra los de la ley anterior. Si cambiaron, **cambia el código**, no sólo un
documento.

**Qué haremos con su respuesta.** Ajustar los avisos automáticos que el programa
enseña: «esta solicitud lleva 12 días abierta, te quedan 8».

### C-8 · Al borrar a alguien, ¿podemos guardar su correo para no volver a escribirle?

**La pregunta, en una frase.** Alguien nos pide que borremos sus datos. Si
borramos su correo **del todo**, la próxima vez que importemos una lista
volverá a entrar y le volveremos a escribir. **¿Podemos guardar su dirección —y
nada más— en una lista de bloqueo, precisamente para respetar su voluntad?**

**Por qué la hacemos.** Es una contradicción real, y no la inventamos nosotros:
para **cumplir** con «no me vuelvas a escribir» hay que **conservar** el dato que
permite reconocerlo.

**Qué haremos con su respuesta.** Ya está construido así: se guarda **sólo la
dirección**, sin nombre, sin empresa, sin nada más, y marcada como «esto lo
pidió la persona». Necesitamos que nos confirme que es la lectura correcta, y si
hay que dejarlo documentado de alguna forma concreta.

### C-9 · Cuando alguien se va, ¿su consentimiento es dato suyo o prueba nuestra?

!i **Pregunta nueva en la versión 2.** Apareció mientras construíamos el
borrado.

**La situación, con un ejemplo.**

Ana se apunta en un formulario de la web y da permiso para recibir correos. El
programa anota una línea:

> *«ana@empresa.mx — dio su permiso el 3 de marzo de 2026 — en un formulario»*

Esa línea **está sellada a propósito**: nadie puede borrarla ni modificarla,
nunca. Si se pudiera editar, no probaría nada — cualquiera podría escribirla
después de recibir una queja. Una prueba que se puede fabricar no es una prueba.

Meses después, **Ana pide que borremos todos sus datos**. Y lo hacemos: su
ficha, su nombre, sus etiquetas, su rastro en los envíos. Todo.

Todo menos esa línea. Porque está sellada.

**La pregunta, en una frase.**

> Cuando Ana se va, ¿esa línea es **un dato de Ana** —y se va con ella— o es
> **la prueba del cliente** de que ese envío era legítimo —y se queda—?

**Las consecuencias de cada respuesta:**

| Si es… | Entonces… | Y el problema es… |
|---|---|---|
| **Dato de Ana** | Se borra con todo lo demás | El cliente **pierde la prueba** de que ese envío era legítimo, justo de la persona con más motivos para quejarse |
| **Prueba del cliente** | Se queda | Le dijimos a Ana «borramos todo» **y no era cierto**: su correo sigue en una tabla que nadie puede tocar |

**Lo que proponemos, si le parece defendible.** Una salida intermedia: **borrar
el nombre y conservar el hecho.** Que la línea quede así:

> *«—borrado— dio su permiso el 3 de marzo de 2026 — en un formulario»*

El cliente conserva lo que un auditor querría ver —que hubo consentimiento, de
qué tipo y cuándo— y Ana deja de estar ahí. Es exactamente lo mismo que ya
hacemos con el registro de envíos.

**Qué haremos con su respuesta.** Decide si esa línea se borra, se anonimiza o
se conserva. **Las tres son construibles**; lo que no podemos es elegir por
usted, porque la respuesta depende de a quién pertenece esa anotación.

### C-15 · ¿Quién tiene que atender las solicitudes, y por qué vía?

!i **Pregunta nueva en la versión 2.**

**La pregunta, en una frase.** ¿Hay obligación de **designar a una persona o un
departamento** de datos personales, y de publicar una dirección concreta donde
la gente pueda ejercer sus derechos? ¿Tiene que ir en cada correo?

**Por qué la hacemos.** Hoy ARLES da por hecho que la solicitud llega «por
correo, a alguien de la empresa». No pide ni guarda quién es ese alguien.

**Qué haremos con su respuesta.** Si hay que designarlo, el programa debería
**pedir ese dato al configurarse** y meterlo automáticamente en el pie de cada
correo. Es una función pequeña, pero hay que construirla.

=== 9 · Bloque E · Conservación y accidentes

!i **Bloque entero nuevo en la versión 2.** Ninguna de estas tres estaba en la
versión anterior, y las tres afectan a decisiones que ya hay que tomar.

### C-16 · ¿Cuánto tiempo se pueden conservar los datos?

**La pregunta, en una frase.** Hoy ARLES **guarda los contactos
indefinidamente**: nadie ha definido un plazo. ¿Hay un límite legal? ¿Depende
de dónde salieron los datos, o de si la persona contestó alguna vez?

**Por qué la hacemos.** Es una pregunta con número. Si hay un plazo, el programa
puede vigilarlo solo y avisar. Si no lo hay, también queremos saberlo — pero
«indefinidamente porque nadie lo pensó» no es una política, es un descuido.

**Qué haremos con su respuesta.** Programar el aviso —«estos 340 contactos
llevan X meses sin actividad»— y, si procede, un borrado automático.

=> OJO CON ESTO :: Afecta también al **registro de envíos**, que guarda direcciones y hoy tampoco caduca. Si hay plazo, aplica a las dos cosas.

### C-17 · Si roban la computadora, ¿hay que avisar a alguien?

**La pregunta, en una frase.** La base de datos está cifrada en el disco del
cliente. Si le roban la laptop, **¿hay obligación de notificar?** ¿A quién, en
cuánto tiempo, y **cambia algo el hecho de que estuviera cifrada**?

**Por qué la hacemos.** No tenemos ningún procedimiento escrito para esto. Y es
el escenario más probable de todos los que hemos pensado: no hace falta un
ataque sofisticado, basta un descuido en un aeropuerto.

**Qué haremos con su respuesta.** Escribir un procedimiento de una página que se
entregue con el programa, y valorar si ARLES debe ayudar a cumplirlo — por
ejemplo, sabiendo decir cuántas personas había en esa base.

### C-18 · ¿Y si en una lista importada viene un dato sensible sin querer?

**La pregunta, en una frase.** ARLES deja importar **cualquier columna** de un
archivo. Si alguien carga un Excel con una columna «observaciones» que dice
*«es diabético»* o *«asiste a la parroquia de…»*, ese dato entra sin más.

**¿Tenemos obligación de impedirlo, de detectarlo o de advertirlo?**

**Por qué la hacemos.** No es un caso rebuscado: pasa continuamente con listas
que vienen de un CRM viejo o de una hoja de cálculo que alguien fue anotando.

**Qué haremos con su respuesta.** Si hay obligación, ARLES puede **avisar al
importar** —«esta columna podría contener datos sensibles, ¿seguro?»— o
directamente no dejar mapearla. Hoy no hace ninguna de las dos.

=== 10 · Qué le podemos entregar, y qué le pedimos

**Lo que SÍ tenemos listo para mandárselo hoy:**

- La **lista completa de qué datos guardamos**, sacada de la estructura real del
  programa. La sección 3 es un resumen; el detalle completo se lo mandamos.
- El **detalle técnico del cifrado** y de cómo se guardan las contraseñas.
- Una **demostración del programa**, presencial o por videollamada.
- El **análisis de riesgos** que hicimos por nuestra cuenta.

!x **Lo que NO tenemos, y le pedimos a usted.** En la versión 1 de este
documento ofrecíamos dos cosas que al revisarlas **resultó que no existen**:

- El **aviso de privacidad**. No hay borrador. Es lo que pedimos en **C-4**.
- El **texto de la declaración de origen** que el usuario acepta al importar.
  Dentro del programa hay hoy un texto de trabajo, marcado visiblemente como
  *«todavía no lo ha revisado un abogado»*. Es lo que pedimos en **C-12**.

**Para acotar el encargo, NO le estamos pidiendo:**

- Revisar el contrato de licencia de ARLES con sus futuros clientes. Eso es otra
  consulta y llega con la versión comercial.
- Revisar los términos de Meta ni de Google. Los conocemos y los cumplimos; son
  contractuales, no legales.
- Una auditoría de seguridad informática.

=== 11 · En qué orden las necesitamos

**Cuatro preguntas cambian el producto**, no un texto: **C-3**, **C-10**,
**C-13** y **C-6**. Si la respuesta a cualquiera de ellas es la que no
esperamos, hay que construir algo que hoy no existe. Ésas son las que más
prisa tienen, y no por urgencia legal sino porque **construir encima de una
suposición equivocada obliga a rehacer**.

| Prioridad | Preguntas | Por qué |
|---|---|---|
| **1 · Urgente** | **C-3** | Si la respuesta es «no», cambia el producto entero |
| **2 · Pronto** | **C-10**, **C-6**, **C-13** | Las tres pueden obligar a construir una función nueva |
| **3 · Antes de campañas** | C-4, C-12, C-14, C-8, **C-9** | Textos, permisos y reglas de borrado que se construyen ahora |
| **4 · Antes de entregar** | C-15, C-16, C-17, C-18 | Conservación, procedimientos y avisos |
| **5 · Cuando pueda** | C-1, C-2, C-5, C-7 | Plazos, citas normativas y trámites |

**Decisión de Dirección del 18 de septiembre de 2026.** Esta consulta se puede
aplazar **salvo C-3**, y conviene mandar esa suelta: es una pregunta concreta
sobre un artículo concreto, mucho más rápida y barata que la consulta entera.

=> FECHA LÍMITE DE C-3 :: **Antes de empezar a construir las campañas.** No es «antes de publicar»: para entonces ya sería tarde y habría que rehacer trabajo.

!x **Lo que hacemos mientras tanto.** El programa se construye con los textos
legales marcados **visiblemente como provisionales**, para que nadie los
confunda con algo revisado. La estructura guarda el texto que sea, así que
sustituirlo después no pierde nada de lo ya firmado por ningún usuario.

=== 12 · Historial de versiones

Este documento **va a seguir creciendo**. Las preguntas legales aparecen
construyendo, no planificando: C-9 apareció el mismo día que se escribió el
borrado de datos, y las de la versión 2 salieron de revisar el producto entero
con esta pregunta en la mano — *«¿qué necesitaría saber un abogado que no le
hemos contado?»*.

| Versión | Fecha | Preguntas | Qué cambió |
|---|---|---|---|
| **1** | 17 sep 2026 | 8 · P-1 a P-8 | Versión inicial |
| **2** | 22 sep 2026 | **18 · C-1 a C-18** | Ver abajo |

**Qué añade la versión 2:**

- **Nueve preguntas nuevas:** C-9 a C-18, todas marcadas dentro del documento.
- **Se renumeran** de «P-x» a «C-x» para no confundirlas con la numeración
  interna de nuestro registro de decisiones. **El orden y el contenido de las
  ocho primeras no cambia:** la antigua P-3 es la actual C-3.
- **Un bloque nuevo entero**, el E, sobre conservación y accidentes.
- **La sección 3**, con la lista completa de qué datos guarda el programa.
- **Corrección honesta:** la versión 1 ofrecía dos anexos —el aviso de
  privacidad y el texto de la declaración de origen— que **no existen**. Se
  corrige en la sección 10 y se convierten en dos peticiones, C-4 y C-12.

=> SI RECIBE UNA VERSIÓN 3 :: Las preguntas nuevas irán marcadas igual que las de ésta, y esta tabla dirá qué cambió. No hace falta releer lo ya contestado.
