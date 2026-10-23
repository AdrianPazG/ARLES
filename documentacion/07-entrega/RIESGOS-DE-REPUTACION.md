# Cómo se quema un dominio (y un número de WhatsApp), y qué hacer

**Para Dirección · 22/10/2026 · ARLES RELAY I v1.2.0**

Este documento contesta una pregunta que Adrián hizo así: *«quiero un reporte de
aviso de cómo es que puedo quemar mi dominio y cuáles serían los planes de acción
de mitigación de riesgos»*.

No es una lista de buenas prácticas. Es un inventario de **las formas concretas
en que esto se rompe**, cada una con tres cosas al lado:

- **qué hace ARLES hoy** — medido, no supuesto;
- **qué queda en tus manos** — porque no todo es código;
- **qué habría que construir**, si decides que ARLES lo impida.

> **Lo primero, porque cambia cómo se lee todo lo demás.** Quemar un dominio no
> es un interruptor: es una **cuesta**. Google y Microsoft no te bloquean, te
> van bajando — primero la pestaña de Promociones, después no deseados, y al
> final el rechazo en el `RCPT TO`. **Bajar cuesta días; subir cuesta meses.**
> Por eso todo lo que sigue está ordenado por «cuánto daño hace», no por
> «cuánto cuesta arreglarlo».

---

## 0 · El riesgo número uno es nuestro, y está medido

**Hoy, dos campañas activas mandan el correo DOS VECES a cada contacto.**

Adrián preguntó: *«al colocar en dos campañas diferentes al mismo contacto y el
mismo remitente en ambas, ¿envía correo 2 veces? ¿qué es lo que sucede?»*.

Medido el 22/10/2026 con el mecanismo real, no leído:

| Qué se midió | Resultado |
|---|---|
| Correos en cola para una persona con dos campañas activas | **2** |
| Claves de idempotencia distintas | **2** — nada los une |

Y la parte que hace que esto sea más grave de lo que la pregunta suponía:
**ARLES no tiene forma de «colocar» un contacto en una campaña.** Al activar,
la audiencia se congela como *todos los contactos activos de la empresa, menos
las bajas*. No hay filtro (está declarado desde la Fase 6). Así que no es «si
coincide un contacto»:

> **Dos campañas activas = TODO tu padrón recibe dos correos. Tres campañas =
> tres.**

**Por qué es el riesgo número uno.** El duplicado no es una molestia estética:
es **la causa más directa de una queja de spam**. Quien recibe dos correos
iguales del mismo remitente no piensa «se equivocaron», pulsa *«marcar como
spam»* — y la tasa de quejas es la cifra que más pesa en tu reputación (§2).
Un padrón de 5 000 personas recibiendo doble puede generar en una tarde más
quejas de las que el dominio aguanta en un mes.

**Y una defensa que esta misma ronda quitó a propósito, porque no se puede
contar como defensa si ya no está.** Hasta el 22/10/2026 el cupo diario **no se
sumaba**: dos campañas de 200 sobre el mismo remitente mandaban 200 en total, no
400 — medido. Eso era un techo real para el dominio, **y escondía un plazo
roto**: el paso 6 le prometía al cliente los días calculados con su límite, y
activar una segunda campaña los duplicaba en silencio.

**La D-13 lo cambia: ahora se suma.** 200 y 200 son 400 al día, y cada campaña
cumple el plazo que prometió. El techo del agregado desaparece, y lo que ocupa
su sitio es **el semáforo de la D-14** en Remitentes: la suma real a la vista,
en verde hasta 60, ámbar hasta 150 y rojo por encima.

> **Esto sube el riesgo de este documento, y hay que leerlo así.** Cinco
> campañas de 200 son mil correos al día desde un buzón, y ARLES ya no lo
> impide: lo enseña en rojo y te deja decidir (§154). **Mirar el semáforo deja
> de ser opcional**, y mirar tu tasa de quejas en Postmaster, más todavía.

**DECIDIDO POR DIRECCIÓN EL 22/10/2026**, y no como yo lo había propuesto. Las
dos decisiones están en
[`DECISIONES_DE_DIRECCION.md`](../01-producto/DECISIONES_DE_DIRECCION.md):

**D-15 · Cada campaña carga su propia lista.** Yo propuse excluir a quien ya
está en otra campaña. Dirección fue al problema de fondo: una campaña de
videovigilancia y otra de desarrollo de software **no deberían tener la misma
audiencia**, y hoy la tienen porque no hay filtro. Al cargar su tabla, cada
campaña escribe a quien le interesa — y la relevancia es una defensa de
reputación de las fuertes: quien recibe una oferta que no pidió no se da de
baja, marca spam.

**D-16 · Y si un contacto cae en dos campañas, recibe las dos — pero no en la
misma semana.** Con plantillas distintas los dos correos tienen sentido; lo que
hostiga es la coincidencia en el tiempo. **Ventana de descanso de 14 días por
defecto**, mínimo 7, leída del registro de envíos. El envío **espera, no se
cancela**, y Actividad lo dice con su cifra — un envío que desaparece en
silencio es el defecto que este proyecto lleva nueve entregas aprendiendo a no
repetir.

> **Las dos juntas son mejor que lo que yo propuse.** Excluir a quien ya está en
> otra campaña resolvía el duplicado y dejaba intacta la causa: que todas las
> campañas escriben a todo el mundo.

---

## 1 · Las cifras que deciden tu reputación

Estas son las cifras de los filtros, no nuestras. **Con su fuente**, porque
cambian y porque una cifra sin fuente se discute en vez de usarse.

| Cifra | Valor | Quién la pone |
|---|---|---|
| **Tasa de quejas de spam · objetivo** | **menos del 0.10 %** | Google, requisitos de remitentes masivos |
| **Tasa de quejas · techo que no se toca** | **0.30 %** | Google: «evita llegar nunca al 0.30 % o más» |
| **Tasa de quejas · Microsoft** | **0.10 %** | Comparativas de 2026 (no verificado en documentación propia de Microsoft) |
| **«Remitente masivo» para Google** | **más de 5 000 correos al día a direcciones de Gmail** | Google. **Y la clasificación es permanente**: una vez dentro, las reglas te aplican aunque luego bajes el volumen |
| **Recuperación tras pasar del 0.30 %** | la tasa tiene que quedarse por debajo **siete días seguidos** | Fuente secundaria, no confirmada en la documentación de Google |

**Qué significa el 0.10 % en tus términos.** Si mandas **1 000** correos, puedes
permitirte **una** queja. Si mandas **5 000**, cinco. No es un margen amplio: es
una persona de cada mil pulsando un botón.

> **La trampa de la cifra.** Esa tasa la mide **Google Postmaster Tools**, no
> ARLES. ARLES no puede saber quién te marcó como spam — y el §65 del proyecto
> dice justamente eso: *aceptado ≠ entregado*. Así que esta cifra la tienes que
> mirar tú, en `postmaster.google.com`, con tu dominio dado de alta. **Si no lo
> das de alta, estás volando a ciegas sobre la única cifra que importa.**

---

## 2 · Las once formas de quemar el dominio

Ordenadas por daño. La columna «ARLES» dice qué pasa hoy.

### R-1 · Correos duplicados — 🔴 ARLES lo permite

Lo del §0. **El de mayor daño y el único que es nuestro.**
→ *Mitigación: la opción A+B de arriba.*

### R-2 · Direcciones que no te dieron permiso — 🟡 depende de ti

Comprar o raspar una lista es la forma más rápida. No por moral: por **trampas
de spam** (*spam traps*), direcciones que existen sólo para cazar listas
compradas. Caer en una te mete en listas negras sin una sola queja humana.

**ARLES hoy:** importa lo que le des. No pregunta de dónde salió.
**Mitigación:** que la tabla venga de alguien que pidió tu contacto. Y si una
lista es de origen dudoso, **no la subas al dominio bueno**: para eso existe la
opción del §3.

### R-3 · Rebotes duros por encima del 2 % — 🟡 ARLES los registra, no los frena

Una dirección que no existe devuelve un rebote duro (`550`). Muchos seguidos le
dicen al filtro «éste no sabe a quién le escribe», que es la firma de una lista
comprada.

**ARLES hoy:** registra cada desenlace y **da de baja** en el rebote duro — la
supresión funciona, y *nada antes del `RCPT TO` puede dar de baja a nadie*,
que es una de las tres reglas del proyecto. Lo que **no** hace es **parar la
campaña** cuando los rebotes se disparan.
**Mitigación / qué construir:** el disyuntor del motor ya existe (Fase 4).
Falta engancharle un umbral de rebotes: *«5 % de rebotes duros en los primeros
200 envíos → se detiene y avisa»*. **Es media entrega, y es la red que falta.**

### R-4 · Arrancar en frío con mucho volumen — 🟡 ARLES avisa, no obliga

Un dominio que nunca ha mandado correo y de pronto manda 2 000 al día parece
comprometido. El calentamiento no es superstición: es cómo se construye
historial.

**ARLES hoy:** avisa por encima de 50 diarios (§48) y lo deja en la bitácora.
**Mitigación:** empieza en **20–50 al día** la primera semana y **dobla cada
semana** si las quejas siguen a cero. Con el límite por campaña que se construyó
el 22/10, esto ya se puede hacer a mano.
**Qué construir:** un calentamiento automático (la rampa como opción del
asistente). No es urgente si respetas la escalera.

### R-5 · Que el remitente no sea de tu dominio — ✅ resuelto el 22/10

Es lo que ya te pasó: desde `@gmail.com` con toda la autenticación en verde,
Outlook daba `SCL: 5`. El patrón «marca + gmail personal» es, para un filtro, la
firma de una suplantación.

**Está cerrado**: `enlace@telemetrymx.com` llega a bandeja principal en Outlook,
Gmail y un corporativo — tres de tres. **No vuelvas a mandar campañas desde una
dirección personal.** Está contado en
[`QUE-HACER-CON-EL-SPAM.md`](QUE-HACER-CON-EL-SPAM.md) §9.

### R-6 · SPF, DKIM o DMARC mal — ✅ los tuyos pasan

Los tres pasan hoy. **Ojo con una conclusión falsa**: pasaban también cuando el
correo caía en no deseados. **Tenerlos bien no te salva; tenerlos mal te
condena.** Son requisito, no ventaja.

**Qué construir:** la entrega **7.5** comprueba esos registros *desde ARLES* y
avisa si alguien los rompe. No es la solución al spam —nunca lo fue, y el
ROADMAP decía que sí—, es una alarma.

### R-7 · Sin baja visible — 🟡 media hecha, y la otra media importa

Para un remitente masivo, Google **exige** una salida fácil. Y hay algo peor que
no tenerla: tenerla donde nadie la ve. Quien no encuentra cómo darse de baja usa
el botón de spam, que es el que cuenta.

**Medido el 22/10/2026**, y salieron dos mitades distintas:

**✅ La cabecera `List-Unsubscribe` SÍ se manda**, y está bien hecha
(`arles-email/src/smtp.rs`). Su propio comentario dice que es *«lo que más mueve
la probabilidad de acabar en spam de todo lo que está en nuestra mano»*. Y tiene
tres detalles que ya se pensaron:

- va **sólo en las campañas**, no en el envío de prueba — decir «esto es correo
  de lista» en un mensaje que alguien se manda a sí mismo resta;
- **si no toca, se omite en vez de ir vacía** — una cabecera de baja sin valor es
  peor que ninguna, y lo cazó una prueba antes de entrar;
- es **`mailto:` y no la de un clic (RFC 8058)**, con su razón escrita: ARLES es
  una aplicación de escritorio y **no tiene servidor** que reciba el `POST`.
  Anunciar un botón de un clic que no existe lo comprueban los filtros.

**Dos cosas que esto cuesta, y son riesgo real:**

1. **La baja llega a tu buzón y hoy la apunta una persona a mano**, y eso es
   lo que **R-25** viene a quitar: Dirección lo pidió el 23/10/2026 —«que ARLES
   lo dé de baja automáticamente»— y está planteado con su recomendación en
   [`DECISIONES_DE_DIRECCION.md`](../01-producto/DECISIONES_DE_DIRECCION.md).
   Mientras no esté, **vaciar ese buzón cada semana no es opcional**. ARLES no lee
   buzones (los rebotes asíncronos son de la v1.3). Si nadie vacía ese buzón,
   **estás escribiendo a gente que pidió que no le escribieras** — que es la
   forma más rápida de una queja, y además un problema legal con P-09 abierto.
2. **La de un clic no se puede dar hoy**, y es la que Google prefiere para
   remitentes masivos. Cumples la cabecera; no cumples el clic.

**❌ El cuerpo del correo NO lleva enlace de baja, y nada lo exige.** Medido: no
hay ninguna validación de plantilla que lo pida — `SiFalta` sólo mira variables
de contacto. Así que se puede crear, activar y mandar una campaña entera **sin
una sola línea que diga cómo dejar de recibirla**. La cabecera la ven los
filtros y los clientes que la soportan; **la persona que está leyendo el correo
busca en el cuerpo**.

**Mitigación inmediata, sin código:** escribe el pie de baja en tus plantillas
desde hoy, con una frase clara y la dirección a la que escribir.

**Qué construir:** que una plantilla **de campaña** no se pueda guardar sin su
pie de baja, y que el preflight lo compruebe. Es pequeño, y es el punto 5 del
plan.

### R-8 · Un asunto o un cuerpo que parecen publicidad barata — 🟡 depende de ti

Mayúsculas, signos de exclamación, «GRATIS», «URGENTE», acortadores de enlaces
(`bit.ly`) y un cuerpo que es sólo una imagen. Lo de la imagen está en el §4.

**Qué construir:** un revisor de asunto y cuerpo en el editor, que avise sin
bloquear. Es barato y evita el error de quien escribe con prisa.

### R-9 · Un dominio nuevo o recién comprado — 🟡 tiempo, no código

Un dominio registrado hace dos semanas tiene cero historial. No se arregla: se
espera y se calienta (R-4).

### R-10 · El mismo texto a miles de personas — 🟡 parcial

Mil correos idénticos byte a byte son más fáciles de agrupar que mil con el
nombre de cada quien.

**ARLES hoy:** tiene sustitución de variables (`{{nombre}}`, …) — no Turing
completa a propósito (ADR-0006). **Se usa si la plantilla la usa.**
**Mitigación:** escribe las plantillas con al menos el nombre y la empresa.

### R-11 · Mandar a las 3 de la mañana, o todo de golpe — ✅ ARLES lo controla

El motor tiene ventana horaria con zona IANA, ritmo y disyuntor desde la Fase 4.
No manda de golpe ni de madrugada. **Esto ya está bien.**

---

## 3 · La defensa estructural que no es código: el subdominio

Es la mitigación de mayor efecto y la que menos gente aplica.

**No mandes campañas desde `telemetrymx.com`. Manda desde un subdominio**, por
ejemplo `enlace.telemetrymx.com` o `correo.telemetrymx.com`, con su propio SPF y
su propio DKIM.

**Por qué.** La reputación se calcula en buena medida por dominio de envío. Si
quemas el subdominio, **tu correo de trabajo sigue llegando**: facturas,
contratos, respuestas a clientes. Si quemas el dominio raíz, se cae todo — y eso
es un problema de negocio, no de marketing.

Es una decisión de DNS, de una tarde, y hay que tomarla **antes** de la primera
campaña grande, no después.

---

## 4 · Imágenes (R-05): lo que preguntaste, con su riesgo

Adrián pidió imágenes en la firma **y** en la plantilla, con cabeceras. Y pidió
explícitamente saber si eso manda el correo a spam.

**La respuesta corta: una imagen pequeña y bien hecha no te manda a spam. Lo que
te manda a spam es el correo que ES una imagen.**

### Lo que dicen las fuentes (22/10/2026)

- **El ratio imagen/texto fijo está desactualizado** como regla, aunque servidores
  B2B y proveedores pequeños todavía usan filtros viejos que lo miran.
- **Un correo que es una sola imagen sí dispara filtros.** Ése es el caso real
  de riesgo, y es justo lo que pasa cuando alguien «diseña» el correo en Canva y
  lo pega entero.
- **El peso importa más que el formato**: los mensajes grandes, sobre todo con
  imágenes, reciben más escrutinio.
- **Las fuentes se contradicen** sobre incrustar vs. alojar, y ninguna lo
  respalda con datos. Lo trato como lo que es: contradicción declarada.
- **Gmail no muestra imágenes en base64** dentro del HTML: las enseña como
  adjunto. Eso descarta el camino que parecía más simple.

### Las tres formas de meter una imagen, y la que recomiendo

| Forma | Cómo se ve | Riesgo |
|---|---|---|
| **base64 dentro del HTML** | **mal en Gmail**: sale como adjunto | Descartada por lo anterior |
| **Adjunta en línea (CID)** | bien en casi todos | Sube el peso de **cada** correo. No encontré fuente sobre si los filtros la tratan distinto, así que **no lo afirmo** |
| **Alojada en un servidor (URL)** | depende: muchos clientes **bloquean imágenes por defecto** | Menos peso y menos escrutinio. **Y aquí está el problema que no esperaba** |

### El problema que no es técnico, y es el que te toca decidir

El saneado de ARLES lleva esta frase escrita desde la Fase 6:

> *una `<img>` con un `src` remoto es un rastreador que delata cuándo se abrió el
> correo — lo que ARLES promete no hacer (§65: no se sabe si llegó)*

Una imagen alojada **le dice a tu servidor** que alguien abrió el correo, con su
IP y su hora. Eso es un píxel de rastreo, aunque nadie lo haya querido así. Y
ARLES tiene escrito, en la interfaz y en los textos legales provisionales, que
**no sabe si un correo llegó**.

**Problema** · Las imágenes alojadas son la mejor opción técnica y rompen una
promesa del producto.
**Riesgo** · Decir «no rastreamos» mientras el servidor registra aperturas. Con
la revisión jurídica abierta (P-09), es el tipo de frase que un abogado señala.
**Impacto** · No es deliverability: es que una afirmación del producto deja de
ser cierta, y en este proyecto eso es lo que más caro ha salido siempre.
**Alternativa** · Tres caminos: (a) sólo imágenes **adjuntas en línea**, que no
delatan nada; (b) alojadas, y se cambia lo que el producto promete; (c)
alojadas **en un servidor que no guarda bitácoras**, que es una promesa que no
podemos verificar.
**Trade-off** · (a) pesa más por correo y hay que poner un tope de tamaño; (b)
es más barato y cuesta una promesa; (c) suena bien y no se puede comprobar.
**Recomendación** · **(a), adjunta en línea, con tope duro de peso.** Y mi
propuesta de límites, para que la imagen no sea el problema:

- **cabecera: 600 px de ancho como máximo, y 40 KB**;
- **firma: 200 px de ancho, 20 KB** — un logo, no una tarjeta de presentación;
- **el correo no puede ser sólo imágenes**: se exige un mínimo de texto real;
- **`alt` obligatorio**, porque con imágenes bloqueadas es lo único que se lee;
- **nada de imágenes en el cuerpo más allá de la cabecera**, de entrada.

Y en el saneado: `<img>` entra en la lista blanca **sólo con `src` de adjunto en
línea**, `alt`, `width` y `height`. Un `src` que empiece por `http` **se cae**,
que es lo contrario de lo que haría una lista negra.

---

## 5 · WhatsApp: cómo se quema un número

Aquí el castigo es más rápido y más visible que en el correo: Meta te pone una
**calificación de calidad** por número y la enseña en su panel.

### Las cifras (22/10/2026, con una advertencia)

Las fuentes públicas **se contradicen** y Meta cambió el sistema en octubre de
2025. Lo que sigue es lo que coinciden varias, y digo cuándo no coinciden:

| Cifra | Valor | Nota |
|---|---|---|
| **Número nuevo o sin verificar** | **250 al día** | coinciden |
| **Tras verificar el negocio** | **2 000 al día** | varias fuentes; las guías viejas dicen 1 000, y describen el sistema anterior a octubre de 2025 |
| **Siguiente escalón** | **10 000**, luego **100 000** | coinciden |
| **Cómo se sube** | mandar a buena parte del cupo en **7 días** con calidad aceptable | coinciden |
| **Qué pasa si la calidad cae a rojo** | unas fuentes dicen que **ya no baja el escalón** y que Meta frena el ritmo en su lugar; otras dicen que sí baja | **contradicción declarada** |
| **Límites por cartera, revisados cada 6 horas** | reportado en 2026 | no verificado en documentación de Meta |

> **Lo que ARLES permite hoy son 50 al día.** Dos órdenes de magnitud por
> debajo del escalón más bajo de Meta. **El cupo de Meta no es tu problema**:
> tu problema es la calidad.

### Lo que baja la calidad de un número

1. **Que te bloqueen.** Es la señal más fuerte, y la cuenta Meta directamente.
2. **Que te reporten.** Peor que un bloqueo.
3. **Escribir primero a quien no te dio permiso.** En WhatsApp el permiso es más
   estricto que en el correo: es la casa de la gente, y lo tratan así.
4. **Plantillas rechazadas o con lenguaje promocional** donde no toca.
5. **Mandar fuera de la ventana de 24 horas** sin plantilla aprobada.
6. **Escribir a fijos.** Un número que no es móvil no recibe WhatsApp, y el
   intento cuenta en tu contra. **Esto ARLES ya lo cuida**: desde el 22/10 deja
   fuera «Teléfono fijo», «Conmutador», «Lada» y «Fax», y **avisa** cuando una
   columna dice «Teléfono» a secas, porque eso no dice si es móvil.

### Lo que ARLES hace hoy, medido

| Qué | Estado |
|---|---|
| Dos cifras de permiso antes de abrir el canal | ✅ la puerta del canal (W.5) |
| Coexistencia y aceptación de riesgo explícita | ✅ W.3 |
| Procesador de avisos de calidad de Meta | ✅ W.4 — los eventos se reciben y se guardan |
| Mandar un WhatsApp de prueba real | ✅ W.8 |
| **Lanzar una campaña entera por WhatsApp** | ❌ **no existe.** El motor sólo recorre correo, atado a `channel = 'email'` en **siete sitios** medidos |
| Contar respuestas | ❌ falta el «relay ciego» (decisión del 29/09/2026) |

**Lo que esto significa para el riesgo:** hoy **no puedes quemar tu número con
ARLES**, porque ARLES no manda campañas por WhatsApp. El riesgo llega con la
fase de WhatsApp, y es el momento de construir las defensas — no después.

### Las defensas que la fase de WhatsApp tiene que traer

1. **Opt-in registrado por contacto**, con fecha y de dónde salió. En WhatsApp
   esto no es cortesía: es la diferencia entre una campaña y una denuncia.
2. **Sólo plantillas aprobadas por Meta** para iniciar conversación. El registro
   de plantillas aprobadas ya existe (6.8b).
3. **Parar al primer síntoma.** El disyuntor existe; hay que engancharlo a los
   avisos de calidad que ya se reciben: *calidad a amarillo → se frena y avisa;
   a rojo → se detiene*.
4. **Un tope diario muy por debajo del de Meta** — los 50 de hoy están bien
   para empezar.
5. **Nunca a un número que no se sabe que es móvil**, salvo que alguien lo
   confirme a mano.
6. **Baja inmediata al primer «no me escribas»**, con la misma regla del correo:
   una baja no se pierde nunca.

---

## 6 · El plan, por orden de lo que más protege

| # | Qué | Quién | Esfuerzo |
|---|---|---|---|
| 1 | **Que dos campañas no manden dos veces** (§0, opción A+B) | ARLES | media entrega |
| 2 | **Subdominio de envío** con su SPF y DKIM (§3) | Dirección + DNS | una tarde |
| 3 | **Dar de alta el dominio en Google Postmaster Tools** | Dirección | 15 minutos, y es la única forma de ver tu tasa de quejas |
| 4 | **Umbral de rebotes que detiene la campaña** (R-3) | ARLES | media entrega |
| 5 | **Que una plantilla de campaña no se guarde sin pie de baja** (R-7) | ARLES | pequeña |
| 5 bis | **Vaciar el buzón de bajas** que llegan por `mailto:` y apuntarlas | Dirección | cada semana, y no es opcional |
| 6 | **Escalera de calentamiento**: 20–50 al día y doblar por semana (R-4) | Dirección | disciplina |
| 7 | **Imágenes con topes duros y adjuntas en línea** (§4) | ARLES | una entrega |
| 8 | **La fase de WhatsApp con sus seis defensas** (§5) | ARLES | una fase |
| 9 | **Revisor de asunto y cuerpo** que avise sin bloquear (R-8) | ARLES | pequeña |
| 10 | **7.5 · comprobar SPF/DKIM/DMARC desde ARLES** (R-6) | ARLES | ya planificada |

---

## 7 · Lo que este documento NO afirma

Por la regla del proyecto: *una comprobación que nunca se ha visto fallar no está
verificada*, y **lo mismo vale para una afirmación**.

- **Lo de la baja sí está medido** (R-7), y por eso cambió de «no lo sé» a dos
  mitades: la cabecera se manda y está bien pensada; el cuerpo no la lleva y nada
  lo exige. Esa frase decía «falta medirlo» hasta que se midió, una hora después
  de escribirla.
- **Las cifras de Meta se contradicen entre fuentes** y el sistema cambió en
  octubre de 2025. Las que doy son las que coinciden; las que no, lo dicen.
- **El 0.10 % de Microsoft** sale de comparativas, no de documentación de
  Microsoft.
- **La contradicción sobre incrustar vs. alojar imágenes** no la resuelvo: las
  fuentes no la respaldan con datos, y la recomendación del §4 se apoya en el
  conflicto con el §65, que sí es un hecho de nuestro producto.
- **Nada de esto se ha visto en una campaña real.** La primera campaña de verdad
  es el único instrumento que mide esto, y conviene que sea pequeña.

---

## Fuentes

- [Google bulk sender requirements — resumen](https://redsift.com/guides/bulk-email-sender-requirements)
- [Gmail bulk sender guidelines](https://www.gmass.co/blog/gmail-bulk-sender-guidelines/)
- [Umbrales de volumen y cumplimiento en Postmaster](https://www.suped.com/learn/email-deliverability/how-does-google-postmaster-compliance-work-and-what-are-the-volume-thresholds-for-bulk-senders)
- [Tasa de spam por encima del 0.3 %](https://www.devcommx.com/blogs/fix-gmail-postmaster-spam-rate-over-0-3-percent)
- [Guía de entregabilidad 2026](https://www.balistro.com/blog/the-2026-email-deliverability-guide)
- [¿Sigue importando el ratio imagen/texto?](https://www.suped.com/knowledge/email-deliverability/content/is-image-to-text-ratio-still-important-for-email-deliverability)
- [Incrustar vs. alojar imágenes de firma](https://www.suped.com/knowledge/email-deliverability/technical/what-are-the-deliverability-and-security-implications-of-embedding-versus-hosting-email-signatur)
- [Imágenes base64 en Gmail](https://labnol.org/gmail-base64-images-231026)
- [Límites y escalones de WhatsApp en 2026](https://m.aisensy.com/blog/whatsapp-message-limits-guide/)
- [Calificación de calidad y límites de mensajes](https://learn.rasayel.io/en/books/whatsapp/whatsapp-business-platform/msg-limits-quality-rating)
- [Escalones y qué pasa al tocar el techo](https://www.uptail.ai/blog/whatsapp-business-message-limits-2026-broadcast-caps-tier-progression-what-happens-when-you-hit-the-ceiling/copy)
