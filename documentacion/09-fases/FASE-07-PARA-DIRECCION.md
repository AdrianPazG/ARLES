# Fase 7 cerrada · Actividad, y siete cosas que la auditoría destapó

**Para Adrián Paz · 1 de octubre de 2026**

---

## Lo que ya se puede hacer

ARLES **cuenta lo que hace**. Antes enviaba y la única forma de saber qué estaba
pasando era abrir la base de datos a mano.

Ahora, en la pantalla de Actividad:

- Una tarjeta por campaña, con sus cifras: cuántos salieron, cuántos quedan,
  cuántos fallaron, cuántos se dieron de baja y **cuánta gente contestó** —
  nunca qué dijeron.
- **Por qué no sale nada**, que es la pregunta real cuando algo va mal: cuál de
  las seis razones es, con su cifra. «Fuera de horario», «se acabó el cupo del
  día: cabían 500»…
- El cupo de cada remitente y las campañas que se lo reparten.
- Un botón que **guarda el resumen de una campaña** en un archivo, con el
  método de cada cifra al lado.

Y en Campañas está el freno: pausar, reanudar, detener y la parada de
emergencia. **Esto es lo más importante de la fase**: hasta la 7.0 una campaña
activada no se podía parar desde el programa.

El registro tiene seis filtros y nunca enseña más de 500 líneas, con el total al
lado. El recorte bimestral dice **antes de tocar nada** cuánto se va y cuánto se
queda, y **un desenlace de envío no se borra jamás**.

---

## La auditoría de cierre encontró siete cosas, con todo en verde

Esto es lo que te quiero contar bien, porque es el mismo patrón otra vez.

### 1 · Desde ARLES no se podía mandar un WhatsApp de prueba

Te dije el 29 de septiembre que ya se podía. **Era verdad a medias y te lo
corrijo.**

El comando existe, está probado de punta a punta contra un servidor que imita a
Meta, y funciona. Lo que no existía era **el botón**. En la pantalla de WhatsApp
no había por dónde pulsarlo.

Ya está. Y al construirlo apareció una segunda capa: las seis frases que ese
comando devuelve —«Meta aceptó el mensaje», «Meta rechazó el token»…— **no
estaban escritas**. Aunque el botón hubiera existido, habrías visto
`whatsapp.prueba.aceptado` en pantalla.

### 2 · «Detener» era casi ilegible

El botón rojo de detener una campaña tenía **2.41 de contraste**, cuando el
mínimo accesible es 4.5. Texto oscuro sobre rojo oscuro.

Es el botón más destructivo del producto y el único freno de algo que está
escribiendo a personas reales. Ahora tiene 6.64.

Lo que más me molesta de este: **el color correcto ya existía** desde la Fase 2,
con una nota explicando exactamente este caso. Y las insignias lo usan bien, con
un comentario que cuenta este mismo defecto arreglado **en la insignia y no en
el botón**. La lección se aprendió en un sitio y no se aplicó en el de al lado.

### 3 · Tres pantallas de catorce no tenían título

Remitentes, Bajas y Derechos empezaban directamente con una tarjeta. Las otras
once se presentan con su nombre en grande.

No es sólo estética: para alguien que use un lector de pantalla, «llévame al
título de esta página» no llevaba a ninguna parte.

### 4 · Tres puertas abiertas que nadie usaba

Tres órdenes del programa estaban registradas y nadie las llamaba. Dos las
cerré; una la convertí en el botón del punto 1.

**No lo hice por limpieza.** Cada orden registrada es una puerta por la que se
puede entrar. Una que nadie usa es una puerta que nadie vigila.

### 5 · Un texto que apuntaba a un sitio que no existe

La variable «Mi empresa» de las plantillas tenía su nombre escrito de dos formas
distintas en dos archivos. Sólo una existe. No se veía porque las pantallas usan
la buena y la mala no la llamaba nadie.

### 6 y 7 · Dos cosas que sólo se vieron mirando la pantalla

«Meta aceptó el mensaje» salía escrito dos veces seguidas, y el botón de mandar
la prueba se estiraba de lado a lado hasta parecer una barra. Las pruebas pasaban
las dos: buscaban que la frase estuviera, y estaba — dos veces.

---

## Por qué no lo cazó nada, otra vez

Hay tres comprobaciones en el proyecto persiguiendo este patrón —una pieza
correcta que nadie usa—. Cada una mira un tramo distinto del cable que va del
núcleo a la pantalla.

**El tramo de en medio no lo miraba ninguna**, y es el único que tú pisas.

Lo mismo con las frases: la comprobación que exige que todo texto exista tenía
una lista escrita a mano de qué familias vigilar, y faltaban justo las dos de
WhatsApp. Y comparaba el nombre corto de la frase en vez de su dirección
completa, así que una frase guardada en el cajón equivocado pasaba.

Las dos comprobaciones están reescritas para que lean la lista **del propio
código** en vez de una copia a mano. Una lista a mano envejece, y envejecer en
silencio es exactamente el problema.

Y una cosa que me parece justo contarte: **escribir la comprobación nueva me
salió mal a la primera, con el mismo defecto que perseguía** — una lista a mano
de dónde mirar. Lo vi porque dio un falso positivo inmediato.

---

## Una cosa que aprendí y que cambia cómo cierro una fase

Arreglar el título de las tres pantallas me costó **tres intentos**. Las tres
veces la comprobación automática se puso verde, y las tres veces **la captura de
pantalla seguía enseñando el título pequeño**.

Medir no es mirar. Lo del 29 de septiembre no fue casualidad.

---

## Lo que decide Dirección

**1 · El canal de WhatsApp, ¿entero o no?**

Hoy manda mensajes de prueba y cuenta respuestas. **No puede lanzar una campaña
entera**, porque el motor que reparte los envíos con su ritmo y su horario sólo
recorre el correo.

- *Si sí*: vuelven el preflight del canal y la pantalla de plantillas aprobadas
  de Meta, que están construidas por dentro y sin pantalla.
- *Si no*: la v1.2.0 se entrega ofreciendo sólo correo, y WhatsApp queda como
  «envío de prueba y cuenta de respuestas», que es lo que de verdad hace.

Está planteado desde hace días en `FASE-05-PARA-DIRECCION.md` y es lo único que
bloquea cerrar el alcance.

**2 · El título dentro de la tarjeta.**

Las catorce pantallas tienen ya su título del mismo tamaño, pero en tres empieza
28 píxeles más a la derecha porque va dentro de la tarjeta. Se iguala sacándolo
fuera, como en las otras once. Es un cambio de diseño: lo decides tú.

**3 · R-24, la prueba con correo real.** Sigue siendo tuya, y sigue siendo lo
que más valor tiene de todo lo pendiente: mientras no se haga, todo lo de correo
está probado contra un servidor de mentira. No se puede hacer desde aquí — la
red de esta sesión bloquea los puertos de correo, medido, y ARLES exige el
llavero del sistema, que un Linux sin escritorio no tiene.

---

## Cómo quedó verificado

| Qué | Resultado |
|---|---|
| Formato y análisis de Rust | limpio, 0 advertencias |
| Pruebas de Rust | todas en verde |
| Pruebas de la interfaz | 187 |
| Tipos y estilo del frontend | limpios |
| Validador, **las nueve fases con sondas** | **298 comprobaciones, 0 fallos, 0 omitidas** |
| Las catorce pantallas, abiertas y miradas | de ahí salieron cuatro de los siete hallazgos |

**Avance del proyecto: 82 %** (al 01/10/2026). La Fase 7 queda al 92 %: lo único que le falta es
SPF/DKIM/DMARC, que tú moviste al final y que sólo se puede verificar de verdad
el día que se cierre R-24.
