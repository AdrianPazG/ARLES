# ARLES RELAY I · Fase 6 · Informe para Dirección

**Campañas y mensajes** · 29 de septiembre de 2026

---

## En una página

**ARLES ya puede mandar una campaña entera.** Escribes el mensaje, eliges desde
qué cuenta sale y a qué ritmo, ARLES te dice a cuánta gente va a llegar y cuánto
va a tardar, te obliga a mandártelo a ti primero, y sólo entonces te deja
activar.

Hasta esta fase, el programa sabía enviar y no sabía **qué** enviar. Ahora sabe
las dos cosas.

**Tres cosas que no voy a disimular:**

1. **La lista de destinatarios no se puede filtrar todavía.** Llegan todos tus
   contactos con correo, menos los que pidieron no recibir. El asistente lo dice
   con todas las letras en el paso 2, en vez de enseñarte filtros que no filtran.
2. **No hay pantalla para escribir plantillas.** Se pueden guardar —el programa
   lo hace y está probado— pero el editor de texto no está construido.
3. **Sigue faltando WhatsApp**, que es de la Fase 5 y no está empezado. La
   decisión sobre cuándo construirlo sigue en tu mesa.

**El avance de la versión 1.2.0 pasa del 61 % al 73 %.**

---

## Lo que puedes hacer ahora, paso a paso

Son nueve pasos, y **tres de ellos no te piden nada: te enseñan algo**. Están
separados a propósito. En un formulario largo se bajan de un scroll sin verlos,
y son justo los que hacen parar a pensar.

| Paso | Qué haces |
|---|---|
| 1 · Información | Le pones nombre. Sólo lo ves tú |
| 2 · A quién | ARLES te dice a cuántos va a llegar |
| 3 · Desde qué cuenta | Eliges el remitente |
| 4 · Qué se manda | Eliges la plantilla y ves qué datos rellena |
| 5 · Ritmo y horario | Cuántos al día, y en qué horas |
| **6 · Cuánto tarda** | **Lees.** «Esta campaña enviará 1 342 correos…» |
| **7 · Comprobaciones** | **Lees.** Las ocho, una por una |
| 8 · Prueba | Te lo mandas a ti. **No se puede saltar** |
| 9 · Activar | Lees el resumen y decides |

### El paso 6 es, probablemente, lo más útil de todo ARLES

> Esta campaña enviará **1 342 correos**.
> Con tu ritmo, caben 50 al día dentro del horario, así que saldrán a lo largo
> de **27 días de envío**.
> A **34 de 1 342** contactos les falta algún dato de la plantilla.

Esa última línea es una cifra y no un sí/no a propósito. Con 34 de 1 342 se
puede seguir; con 900, conviene parar y revisar la lista. Un «hay datos sin
rellenar» te obliga a revisarla entera a mano para saber cuál de los dos casos
es.

### El paso 7 tiene tres respuestas, no dos

Cada comprobación puede salir **bien**, **mal**, o **sin comprobar**. La tercera
existe porque ARLES a veces no puede mirar —no hay internet, el servidor no
contesta— y decir que algo está bien sin haberlo visto sería mentir sobre tu
propia campaña.

**Un punto sin comprobar no deja activar**, igual que uno que falla. El texto es
distinto porque lo que hay que hacer es distinto: uno se arregla, el otro se
comprueba.

Y hay una que puse con cuidado: **un cable desenchufado nunca se enseña como «tu
contraseña no sirve»**. De las seis formas en que puede fallar la conexión, sólo
dos dicen algo sobre la contraseña. En las otras cuatro ARLES ni siquiera llegó
a intentarlo, y dice eso. Sin esta distinción, alguien se pasa la tarde
generando contraseñas de aplicación nuevas por un problema de red.

---

## El aviso de los 50 diarios

Si pones más de 50 correos al día desde una cuenta, aparece esto en la misma
pantalla —no en una ventana que se cierra sin leer—:

> ⚠ **Enviar más de 50 correos al día desde una cuenta tiene riesgo.**
> Los proveedores vigilan cuánto manda cada cuenta. Un volumen alto desde una
> cuenta sin historial puede acabar en rebotes, en la carpeta de no deseados, o
> con la cuenta suspendida.
>
> ARLES no te lo impide: la decisión es tuya.
>
> ☐ Entiendo el riesgo y quiero continuar con 120 al día.

**ARLES no bloquea el volumen.** Lo que no permite es crear la campaña sin que
conste que se avisó — y eso queda escrito en el registro del programa, con su
fecha, no en una casilla que se pueda cambiar después.

La diferencia importa el día que un proveedor marque una cuenta como spam: entre
«alguien decidió esto sabiendo el riesgo» y «no sabemos qué pasó».

**Y ARLES sigue sin repartir los envíos entre varias cuentas** para ayudar a
sortear ese límite. Es una decisión tuya de la primera reunión (T-1), y ahora se
dice en la pantalla donde se elige el remitente, que es donde a alguien se le
ocurriría preguntarlo.

---

## Lo que ARLES sigue sin decir, y no por descuido

**Nunca dirás «entregado» en ninguna pantalla de campañas.** El programa sabe que
el servidor de correo se hizo cargo del mensaje. No sabe si llegó a una bandeja,
ni si acabó en no deseados, ni si alguien lo abrió.

La columna dice **«Aceptados»**, y hay una comprobación automática que falla si
alguien escribe la otra palabra en cualquier texto de esta parte del programa.

Parece una distinción pequeña hasta que alguien decide algo con ella delante:
«entregamos 1 240 y no contestó nadie» lleva a conclusiones muy distintas de
«1 240 salieron, y no sabemos cuántos se vieron».

---

## Lo que revisé al cerrar, y lo que encontré

Recorrí la fase buscando lo que **no falla**: cosas que funcionan mal sin dar
ningún error. Cinco hallazgos.

**Dos eran fallos de verdad y están corregidos.** El asistente hablaba con el
servidor dos veces seguidas para calcular lo mismo —hasta un minuto de espera de
más—, y un paso podía quedarse un minuto con la palabra «Calculando» en
pantalla, que es indistinguible de una avería. Ahora explica qué está haciendo.

**Uno era un texto que faltaba.** Si el programa se encontrara un estado de
campaña que no reconoce, la lista habría enseñado un código interno.

**Y dos eran sustos que resultaron estar bien**, aunque ahora tienen prueba:

- La cifra que lees en el paso 6 y la que se congela al activar salen de **dos
  consultas distintas, escritas en dos entregas distintas**, y nadie las había
  comparado. Coinciden. Si un día dejaran de coincidir, ahora falla una prueba
  en vez de descubrirse contando correos tres días después.
- Un ritmo de cero al día crearía una campaña que se activa y no envía nunca,
  con la pantalla diciendo «enviando». Lo caza el paso 7, que no deja activarla.

Al comprobar el primero encontré además que **mi propia prueba no servía**:
pasaba igual con el programa roto, porque a los datos de prueba les faltaba el
caso que hacía falta. Lo digo porque es exactamente la regla de trabajo de este
proyecto: una comprobación que nunca se ha visto fallar no está verificada.

---

## Los números, con fecha

| Qué | Cuánto | Cuándo |
|---|---|---|
| Comprobaciones automáticas de esta fase | 20 de 20 | 29/09/2026 |
| Comprobaciones de todo el proyecto | 248, ninguna fallando | 29/09/2026 |
| Pruebas del motor (Rust) | 788 | 29/09/2026 |
| Pruebas de la interfaz | 133 | 29/09/2026 |
| Avance de la v1.2.0 | 73 % | 29/09/2026 |

Las comprobaciones que necesitan abrir un navegador de verdad se marcan como
**omitidas** cuando no hay Chromium, y eso no es lo mismo que pasadas. Se dice
así en la salida.

---

## Lo que necesito de ti

**1 · La decisión de WhatsApp sigue abierta.** Está planteada en el informe de
la Fase 5. Mi recomendación no ha cambiado: entregar la Fase 6 ofreciendo sólo
correo, y construir WhatsApp después, porque depende de una cuenta de Meta que
no está en nuestras manos.

**2 · R-24 · Probar contra un servidor de correo real.** Sigue sin poderse hacer
desde aquí: la red bloquea los puertos de correo y esta máquina no tiene llavero
del sistema. Se hace en tu equipo, con una cuenta desechable. **Ahora ya se puede
probar el camino entero**, no sólo que la cuenta conecte: crear una campaña de
prueba con dos o tres contactos tuyos y activarla.

**3 · R-07 · Que alguien abra ARLES en un Mac.** Esta fase le añade un motivo
propio: ARLES pide la contraseña al llavero cada vez que hace las comprobaciones
del paso 7, y en macOS eso puede sacar un diálogo del sistema. Son dos por
asistente, y **no se pueden evitar** sin guardarse la contraseña en memoria, que
es justo lo que decidimos no hacer nunca.

**4 · P-01 · La licencia de Mont** sigue siendo lo que impide entregar un
instalador. No bloquea construir; bloquea repartir.

---

## Qué viene después

La **Fase 7 · Actividad**: las dos pestañas donde se ve una campaña mientras
corre, la línea de tiempo de cada envío, y el detalle de qué pasó con cada
mensaje.

Y lo que la Fase 6 dejó fuera a propósito y habrá que decidir cuándo entra: el
editor de plantillas, y los filtros de audiencia.
