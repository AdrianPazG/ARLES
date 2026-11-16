# ARLES RELAY I · Fase 5 · Informe para Dirección

**Proveedores de envío** · 29 de septiembre de 2026

---

## En una página

**ARLES ya manda correos.** Por primera vez desde que empezó el proyecto, el
programa abre una conversación con un servidor de correo, se identifica, cifra
el canal, entrega un mensaje y lo deja anotado. Está ejercitado de punta a punta
en las pruebas.

Lo que esta fase construyó es la parte que habla con el mundo: entender lo que
contesta un servidor, guardar una cuenta **sin guardar su contraseña**, y mandar
un correo de prueba a una dirección propia antes de arriesgar una campaña.

**Dos cosas que no voy a disimular, porque cambian lo que se puede planificar:**

1. **Falta el canal de WhatsApp.** El plan pone su adaptador en esta misma fase,
   y no está empezado. Lo que se cierra es la parte de correo.
2. **ARLES todavía no ha hablado con un servidor de correo real.** Todo está
   probado contra un servidor de mentira construido para eso. Esa prueba final
   sólo se puede hacer en tu máquina, y ahora ya se puede hacer entera.

**El avance de la versión 1.2.0 pasa del 59 % al 61 %.**

---

## Lo que ARLES sabe hacer ahora

### Conectarse a una cuenta de correo, y decir qué pasó si no puede

Das de alta tu cuenta en **Remitentes** y ARLES la prueba antes de guardarla.
Si algo falla, no dice «error de conexión»: dice cuál de **seis** cosas pasó, y
qué hacer con cada una. No se llegó al servidor · tardó demasiado · el cifrado
no se pudo establecer · la contraseña no sirve · el servidor no está disponible
· todo correcto.

Un «no se pudo conectar» a secas deja a alguien probando puertos media hora.

### Mandar un correo de prueba y ver si llega

Es el botón que cierra la última duda antes de una campaña. Manda un correo real
a la dirección que escribas —normalmente la tuya— y te dice si el servidor lo
aceptó.

**Dice «el servidor aceptó», nunca «entregado».** ARLES no puede saber si llegó
a tu bandeja: puede saber que el servidor se hizo cargo. Es el punto de todo el
programa donde más tentador sería decir lo contrario, porque acabas de pulsar un
botón y quieres un sí.

### Enviar de verdad, aunque cierres la ventana

Al abrir ARLES hay ahora algo dentro que mira qué campañas están enviando y las
atiende. Funciona aparte de la ventana: **cerrarla no detiene una campaña**, y
un servidor lento no congela el programa.

Si abres ARLES dos veces, sólo envía el primero. El segundo abre igual —puedes
mirar contactos y configuración— y te lo dice en pantalla, aclarando que no se
ha perdido ni duplicado ningún correo.

---

## Las tres reglas que gobiernan esta fase

Están escritas varias veces en el programa a propósito, y hay comprobaciones
automáticas que fallan si alguien toca una.

### 1 · No existe ninguna forma de no cifrar

Ni un valor, ni una casilla, ni una opción avanzada. Si algún día un servidor no
cifra, ARLES no le habla — no hay plan B.

La regla está escrita en tres sitios del programa, incluida la propia base de
datos, que **no admite** guardar una conexión sin cifrar aunque el resto del
código lo dejara pasar. Parece exagerado hasta que se piensa en el día en que
alguien «arregle» uno de los tres para hacer funcionar un servidor mal
configurado de un cliente.

Y ARLES no se fía de la biblioteca que hace el cifrado: antes de mandar tu
contraseña, le pregunta a la conexión si está cifrada y se niega a seguir si no
lo está.

### 2 · La contraseña entra y no sale

Va al llavero de tu sistema —el Keychain de macOS, el Administrador de
credenciales de Windows— y en los archivos de ARLES sólo queda una referencia
que no sirve para entrar a ningún sitio.

Ninguna pantalla puede leerla. Ningún informe la incluye. No hay ni un camino
en el programa que la escriba en otro lado.

### 3 · Nada da de baja a nadie por accidente

Cuando un servidor rechaza un correo, ARLES tiene que distinguir **dos cosas que
se parecen mucho**: «esa dirección no existe» —esa persona deja de recibir para
siempre— y «tu cuenta no puede enviar desde esa dirección» —el problema es tuyo,
y el contacto es inocente—.

Las dos llegan con el mismo número. Por eso ARLES lleva la conversación con el
servidor paso a paso en vez de de una vez: es la única forma de saber a cuál de
las dos preguntas contestaba.

Y el correo de prueba **no puede dar de baja a nadie**, pase lo que pase. Si
pudiera, probar una cuenta mal configurada contra tu propia dirección te daría
de baja a ti — y una baja no se deshace.

---

## Lo que encontré por el camino

### El motor estaba construido y nadie lo encendía

La Fase 4 dejó el motor de envío terminado y probado entero. Al ir a conectarlo
resultó que **el programa ni siquiera sabía que existía**: dos mitades
construidas y sin presentar.

Ninguna prueba lo detectaba, porque cada mitad pasaba la suya. Es el tipo de
fallo que sólo aparece cuando alguien va a usar las dos juntas — en este caso,
yo, tres semanas después.

### Un límite que llevaba días en el sitio equivocado

Al escribir la consulta que decide cuántos correos puede mandar una campaña,
busqué los límites donde el esquema original dice que están. Una migración de
hace semanas los había movido. La prueba contra una base de datos real lo cazó;
si me hubiera fiado del documento, la consulta habría leído siempre cero.

### Tres comentarios que afirmaban algo falso

La regla de este proyecto es que una comprobación que nunca se ha visto fallar
no está verificada: cada garantía se rompe a propósito para ver si algo la caza.

Esta fase encontró **tres comentarios míos que describían una rotura que no
funcionaba** — decían «si cambias esto, la prueba falla», y al probarlo no
fallaba. Los tres están corregidos, y cada uno dice ahora la rotura que sí
verifiqué, y lo que la prueba **no** caza.

### Y una brecha que no existía

Al auditar el cierre sospeché que una garantía importante —que una cuenta con la
contraseña caducada deja de intentarlo— sólo estaba probada a trozos. Escribí
una prueba nueva para cubrirla.

Al romper el código a propósito falló **también una prueba que ya existía**, y
con una comprobación más fuerte que la mía. **Borré la prueba nueva.** Una
prueba de más no es gratis: hace creer que hay dos garantías donde hay una.

---

## Lo que falta, con nombre y apellidos

### WhatsApp · lo que hay que decidir

El plan pone el adaptador de WhatsApp en esta fase. No está empezado.

La Fase 6 —campañas y mensajes— construye un asistente que ofrece **una o dos
etapas**: correo y WhatsApp. Mientras el adaptador no exista, la segunda etapa
se puede configurar y no puede enviar.

**Hay que elegir**, y es una decisión tuya:

- **Construir el adaptador de WhatsApp antes de la Fase 6**, y que el asistente
  nazca completo.
- **O entregar la Fase 6 ofreciendo sólo correo**, y añadir WhatsApp después.

Mi recomendación es la segunda, por una razón concreta: WhatsApp depende de algo
que no controlamos —**META**, la cuenta con Coexistencia activada, que sigue
pendiente de Operaciones—. Atar la Fase 6 a eso significa que un retraso ajeno
paraliza la parte que sí podemos terminar. Con la primera opción, el riesgo se
propaga; con la segunda, se aísla.

### R-24 · la prueba que sólo puedes hacer tú

ARLES no ha hablado nunca con un servidor de correo real. Está medido por qué no
se puede comprobar desde aquí: el entorno de desarrollo bloquea los tres puertos
de correo, y además ARLES exige el llavero del sistema, que un servidor Linux sin
escritorio no tiene.

**Lo que esta fase cambia es que ahora la prueba se puede hacer entera.** Antes
sólo se podía comprobar que la cuenta conectaba.

Los tres pasos, en tu máquina:

1. En **Remitentes**, das de alta una cuenta de Gmail con **verificación en dos
   pasos** y una **contraseña de aplicación**. Usa una **cuenta desechable**, no
   la de trabajo: mientras el producto no esté cerrado, una credencial que se
   pueda tirar sin consecuencias vale más que la comodidad.
2. Pulsas **«Mandar un correo de prueba»** con tu dirección.
3. Miras la bandeja de entrada **y la carpeta de correo no deseado**.

Si llega, R-24 queda cerrado. Si no llega, lo que diga la pantalla me sirve para
arreglarlo.

---

## Las cifras

| | |
|---|---|
| **Avance de la v1.2.0** | **61 %**, a 29/09/2026 |
| Comprobaciones de la fase 5 | **35 de 35**, ninguna omitida |
| Pruebas automáticas | **720** del programa · **120** de la interfaz |
| Entregas de la fase | 6 de 6 en la parte de correo |
| Migraciones de base de datos | V7 y V8, las dos probadas sobre bases **con datos** |

Las seis fases del validador pasan en verde. Ninguna dependencia con problemas
de licencia ni de seguridad conocidos.

---

## Lo que sigue

**Fase 6 · Campañas y mensajes.** Es la que convierte todo lo anterior en algo
que se puede usar: plantillas, personalización, el asistente de nueve pasos, la
simulación de duración y las ocho comprobaciones previas a activar.

Es también donde el envío de prueba encuentra su sitio definitivo. Hoy el
programa sabe mandarlo, y no hay ninguna pantalla desde donde pedirlo para una
campaña concreta — porque todavía no hay campañas.
