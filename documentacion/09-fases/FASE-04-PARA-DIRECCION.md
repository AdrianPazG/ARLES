# ARLES RELAY I · Fase 4 · Informe para Dirección

**El motor de ejecución** · 23 de septiembre de 2026

---

## En una página

ARLES ya sabe mandar. No ha mandado todavía ni un solo correo real, y eso es
a propósito.

Esta fase construyó la parte del programa que decide **a quién le toca, cuándo,
a qué ritmo, y qué hacer cuando algo sale mal**. Se construyó entera contra un
proveedor de mentira: uno al que se le puede ordenar que se caiga, que tarde
demasiado, que diga que la contraseña no sirve. A Gmail no se le puede pedir eso.

Las decisiones más delicadas de un motor de envío son todas respuestas a un
fallo. Sin un proveedor al que ordenarle que falle, esas partes del programa
sólo se ejercitarían el día que fallaran de verdad — en mitad de la campaña de
un cliente, que es el peor momento posible para descubrir que estaban mal
escritas.

**El avance de la versión 1.2.0 pasa del 42 % al 56 %.** La Fase 4 pesa el 14 %
de la versión, y queda cerrada entera.

---

## Lo que ARLES sabe hacer ahora

### Nadie recibe el mismo correo dos veces

Es la garantía que gobierna todo lo demás, y la razón de que esta fase se
adelantara en el calendario.

Un correo duplicado es el peor fallo que ARLES puede cometer, por tres motivos
que se suman:

- **Es invisible para nosotros.** Nada falla, ningún registro lo delata.
- **Es irreversible para quien lo recibe.** Un correo que salió, salió.
- **Es acumulativo para el dominio.** Los duplicados alimentan la tasa de queja,
  y la reputación de un dominio se pierde en un día y se recupera en meses.

Un correo que *no* sale, en cambio, se ve y se arregla. Esa asimetría decide
todas las decisiones difíciles del motor.

Hay **tres barreras**, y ninguna sobra. La más importante es la primera, porque
es la única que sigue funcionando cuando nuestro código no: la base de datos
rechaza físicamente un segundo intento para la misma persona en la misma
campaña. No es improbable — es imposible.

### Se puede apagar el equipo a mitad de una campaña

Se probó de la peor manera posible: matando el programa justo entre el momento
en que el proveedor acepta el correo y el momento en que lo anotamos.

Esa situación es **genuinamente ambigua**: el correo pudo salir o no, y no hay
forma de saberlo sin leer la bandeja de enviados — un permiso que esta versión
no pide y no quiere pedir.

ARLES no adivina. Marca esos envíos como **«envío no confirmado»**, se los
enseña a usted con su nombre y su hora, y **no los reenvía nunca por su cuenta**.
Preferimos que usted decida sobre una lista corta a que alguien reciba dos veces.

### El límite diario se respeta de verdad

El cupo de envíos se guarda en el disco, no en la memoria. Si se guardara en
memoria, cerrar y abrir ARLES lo reiniciaría, y una cuenta con tope de cincuenta
diarios podría mandar ciento cincuenta en un día sin que nada avisara.

**Y al agotarse, ARLES pausa y avisa. No busca otra cuenta.** Rotar remitentes
para superar un límite es evasión, viola las políticas de Google, y es el
mecanismo exacto por el que se queman dominios. Que ARLES no lo haga es una
decisión de producto, no una limitación.

### Los horarios son de la campaña, no del equipo

Una campaña a clientes de Madrid se envía en horario de Madrid aunque quien la
lance esté en Monterrey — y sigue siendo horario de Madrid si esa persona se
lleva el portátil a Tokio.

**Aquí evitamos un fallo caro por poco.** La biblioteca de zonas horarias que
era la elección natural **sigue aplicando el horario de verano a México**,
abolido en 2022. Con ella, **todas las campañas mexicanas habrían enviado una
hora fuera de su horario durante medio año**, en silencio y sin que nada fallara.
Se cambió de biblioteca.

### Pausar, Reanudar y Detener significan lo que dicen

Tres botones que parecen lo mismo y son dos cosas muy distintas:

| Botón | Qué le pasa a la cola | ¿Se deshace? |
|---|---|---|
| **Pausar** | Se conserva **intacta** | Sí |
| **Reanudar** | Sigue donde estaba | — |
| **Detener** | Lo que no salió se cancela | **No** |

Detener pide confirmación **con la cifra**: no «¿seguro?», sino «esto cancela
4 312 correos que no han salido». Y hay una prueba que comprueba que la cifra
avisada es exactamente la que se cancela.

**La parada de emergencia sobrevive a cerrar el programa.** El caso concreto:
usted ve que la campaña recién activada lleva la plantilla equivocada, pulsa
parar, y cierra el programa para irse. Si la parada viviera en memoria, al día
siguiente ARLES arrancaría sin ella y seguiría enviando la plantilla equivocada
a los cuatro mil que faltaban.

### Antes de activar, ARLES dice cuánto va a tardar

> «Esta campaña enviará 5 000 mensajes y terminará alrededor del 9 de febrero.»

Es probablemente la información más valiosa de todo el producto: es lo que hace
que alguien reconsidere su configuración **antes** de lanzar, no tres semanas
después.

**Y dice cuál de los dos topes es el freno.** «Tarda cuatro meses» deja sin saber
qué tocar. «Tarda cuatro meses **porque tu tope diario es 50**, cuando tu horario
daría para 90» dice exactamente dónde está el problema. Sin eso, lo normal es
ampliar el horario, no notar ninguna diferencia, y perder la tarde.

Hay dos respuestas que no son una fecha y valen más que la fecha:

- **«Con este tope no saldría ni un mensaje.»** Es el fallo más difícil de
  diagnosticar que existe: la campaña quedaría marcada como enviando sin enviar
  nada, sin fallar y sin nada roto que mirar.
- **«No termina en cinco años: 435 000 mensajes se quedarían fuera.»** Decir
  «terminará en 2041» sería correcto e inútil.

### Ocho comprobaciones antes de activar, con una tercera respuesta

Activar una campaña es irreversible en la práctica. Estas comprobaciones cazan
los fallos que **no fallan al activar**, sino tres días después: la contraseña
caducada, la audiencia que se quedó vacía, los 340 correos que dirían «Hola
{{nombre}}».

Lo importante es la tercera respuesta. Cada punto sale «bien», «hay que
arreglarlo» o **«sin comprobar»** — y sin comprobar **no es un aprobado**.

Varias comprobaciones necesitan piezas que llegan en fases posteriores. Un
preflight que diera por bueno lo que todavía no sabe mirar sería peor que no
tenerlo. Es la misma regla que aplicamos a las pruebas de navegador: **omitida no
es pasada, y se dice**.

---

## La auditoría de la fase: cinco hallazgos

Antes de cerrar, se recorrió la fase entera contra sus propios requisitos.
Aparecieron cinco cosas. Ninguna era menor, y una era grave.

### 1 · La protección contra las ráfagas hacía justo lo que quería impedir

**Este es el hallazgo importante del informe.**

ARLES tiene una regla: si el equipo se suspende ocho horas y despierta, la
campaña **no** puede soltar de golpe los envíos que «se ahorró». Eso es una
ráfaga, y las ráfagas son lo que hace que un proveedor de correo marque a un
remitente como spam.

Había código para protegerse de eso. Tenía su prueba. La prueba se llamaba
«ocho horas dormido no producen una ráfaga». Y pasaba.

**Estaba mal.** El caso que la tumba es una siesta **dentro del mismo día**: el
equipo duerme de 10:00 a 18:00 con el cupo diario ya agotado, despierta, el
contador vuelve a cero, y esa cuenta manda otros cincuenta. **Cien envíos en un
día con un tope de cincuenta** — exactamente la ráfaga que ese código existía
para impedir.

El motivo es instructivo. Esa regla se escribió en una entrega temprana, cuando
el cupo vivía en la memoria del programa. Dos entregas después, el cupo pasó a
guardarse en el disco con una etiqueta por hora y por día, y con ese cambio el
problema **se disolvió solo**: al despertar, la hora es otra y su contador ya
vale cero; si el día es el mismo, el contador del día sigue donde estaba, que es
lo correcto.

La regla vieja se quedó ahí, apuntando. Y su prueba estaba escrita para
confirmar lo que el código hacía, no para comprobar lo que el producto necesita.

Es el ejemplo más limpio de la regla de trabajo de este proyecto: *una
comprobación que nunca se ha visto fallar no está verificada*. Se quitó la regla
peligrosa y la garantía se mide ahora donde de verdad vive.

### 2 · Atrasar el reloj del equipo regalaba el cupo diario

Bastaba con mover la hora del sistema un día hacia atrás para que el contador se
reiniciara y la cuenta mandara su cupo entero otra vez.

No hace falta mala fe para que pase: un ajuste automático de hora tras una
deriva hace exactamente eso. Era un caso que nuestra propia estrategia de
pruebas tenía enunciado desde hace meses y que nadie había cubierto.

### 3 · Faltaba una pieza que la documentación daba por hecha

El **disyuntor por cuenta**: tras cinco fallos seguidos, esa cuenta deja de
enviar en vez de seguir insistiendo contra un servidor que no responde.

La documentación lo describía, la base de datos tenía las columnas preparadas
desde el primer día, y **ningún código las usaba**. Una promesa escrita que
nadie cumplía no falla: simplemente no protege de nada.

Se construyó. La parte delicada fue decidir de quién es la culpa de cada fallo:
que una dirección no exista **no dice nada malo del remitente**. Si los rebotes
contaran, una lista importada con direcciones viejas pararía la campaña cada
pocos minutos sin que usted entendiera por qué.

### 4 · El mismo suceso se contaba de dos maneras distintas

La pantalla de Actividad habría dicho «Se volverá a intentar», y el detalle del
mismo envío, a un clic de distancia, «Reintento 2 de 3». El mismo hecho con dos
redacciones, y una de las dos sin la cifra.

### 5 · Dos piezas construidas por separado no se hablaban

Si ARLES ya había detectado que la contraseña de una cuenta no sirve, las
comprobaciones previas seguían diciendo «sin comprobar» — cuando la respuesta
estaba escrita en la base de datos. Le habría hecho buscar el problema donde no
estaba.

---

## Cómo está verificado

| Qué | Cifra |
|---|---|
| Pruebas automáticas de Rust | **595** |
| Pruebas del frontend | **102** |
| Comprobaciones de la fase 4 | **36 de 36, 0 omitidas** |
| Formato, análisis estático y tipos | Sin una sola advertencia |
| Licencias y avisos de seguridad | En verde |

**Y una forma de verificar que no es habitual.** Cada garantía que este informe
afirma se **rompió a propósito** para ver si la prueba correspondiente la
cazaba. Si no la cazaba, la prueba no valía y se arregló antes de seguir.

Ese método es el que encontró los dos primeros hallazgos de la auditoría.

---

## Lo que esta fase **no** hace, y conviene tener claro

**No ha mandado ni un correo real.** Eso llega en la Fase 5, que sustituye el
proveedor de mentira por uno de verdad sin tocar nada de lo construido aquí.

**No tiene pantallas.** El motor funciona por debajo; las pantallas de Campañas
y Actividad son de las fases 6 y 7. Todo lo que este informe describe existe y
está probado, pero todavía no se puede pulsar.

**No detecta todos los rebotes.** Sólo los que el servidor comunica durante el
envío. Los que llegan después, como un correo de vuelta a su bandeja, necesitan
leer un buzón — un permiso que esta versión no pide, y que está diferido a la
v1.3.

---

## Lo que sigue bloqueado y no depende de nosotros

Ninguno impide construir. **Los cuatro impiden entregar la versión 1.2.0.**

| Qué | Quién | Qué bloquea |
|---|---|---|
| **P-01 · Licencia de la tipografía Mont** | Dirección | No se puede distribuir el instalador con la tipografía dentro |
| **P-09 · Revisión jurídica** | Abogado | Los textos legales que el usuario lee. Dieciocho preguntas preparadas |
| **META · Cuenta con Coexistencia** | Operaciones | WhatsApp no se puede probar contra nada real |
| **R-07 · Revisión en macOS** | Quien tenga un Mac | Nadie ha abierto ARLES en un Mac todavía |

---

## Lo siguiente

**Fase 5 · Proveedores de envío.** Es donde ARLES manda su primer correo de
verdad. El trabajo es traducir: implementar el envío por SMTP y convertir los
códigos que devuelve cada proveedor a los cinco fallos que el motor ya sabe
manejar.

El motor no cambia. Ésa era la idea de construirlo contra un simulador.

---

*ARLES RELAY I · versión 1.2.0 · TELEMETRY INSIGHT*
