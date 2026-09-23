# Changelog

Todos los cambios notables de ARLES RELAY I se documentan aquí.

Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).
Versionado según [Versionado Semántico](https://semver.org/lang/es/) (§3).

> **Nomenclatura prohibida** en cualquier artefacto: FINAL, FINAL2, DEFINITIVO, NUEVO.
> Toda versión se identifica por su número semántico.

---

## [Sin publicar]

### Auditoría de cierre de la Fase 4 — cinco hallazgos

Antes de cerrar la fase se recorrió entera contra sus propios requisitos.
Ninguno de los cinco hallazgos era menor, y uno era grave.

**1 · La protección contra las ráfagas hacía justo lo que quería impedir.**

Si el equipo se suspende y despierta, la campaña no puede soltar de golpe los
envíos que «se ahorró»: eso es una ráfaga, y las ráfagas son lo que hace que un
proveedor marque a un remitente como spam.

Había código para eso. Tenía su prueba. La prueba se llamaba «ocho horas dormido
no producen una ráfaga», y pasaba. **Estaba mal.**

El caso que la tumba es una siesta **dentro del mismo día**: el equipo duerme de
10:00 a 18:00 con el cupo diario agotado, despierta, el contador vuelve a cero,
y esa cuenta manda otros cincuenta. **Cien envíos en un día con un tope de
cincuenta.**

El motivo es instructivo: esa regla se escribió cuando el cupo vivía en memoria.
Dos entregas después pasó a guardarse en disco con una etiqueta por hora y por
día, y el problema **se disolvió solo**. La regla vieja se quedó ahí, apuntando,
y su prueba estaba escrita para confirmar lo que el código hacía en vez de
comprobar lo que el producto necesita.

**2 · Atrasar el reloj del equipo regalaba el cupo diario.** Bastaba con mover la
hora un día hacia atrás para que el contador se reiniciara. No hace falta mala
fe: un ajuste automático de hora hace exactamente eso. Estaba enunciado en la
estrategia de pruebas desde hace meses y nadie lo había cubierto.

**3 · Faltaba el disyuntor por cuenta**, que la documentación describía y la base
de datos tenía preparado desde el primer día. Tras cinco fallos seguidos, esa
cuenta deja de insistir contra un servidor que no responde. La parte delicada:
que una dirección no exista **no dice nada malo del remitente**, así que los
rebotes no cuentan — si contaran, una lista con direcciones viejas pararía la
campaña cada pocos minutos.

**4 · El mismo suceso se contaba de dos maneras.** Actividad habría dicho «Se
volverá a intentar» y el detalle del mismo envío, a un clic, «Reintento 2 de 3».

**5 · El preflight ignoraba lo que la base ya sabía.** Con una contraseña
caducada ya detectada, seguía diciendo «sin comprobar».

**Fase 4 cerrada**: 36 de 36 comprobaciones, **0 omitidas**.


### «Terminará alrededor del 9 de febrero» — antes de activar, no después

Alguien importa cinco mil contactos, pone cincuenta envíos diarios de lunes a
viernes, y pulsa activar. Sin este cálculo se entera de que la campaña dura
**cuatro meses** cuando ya lleva tres semanas corriendo y el lanzamiento que
anunciaba fue el mes pasado.

Ahora lo sabe **antes**, que es el único momento en que todavía puede decidir
otra cosa: subir el límite, recortar la audiencia, ampliar el horario o partir
la campaña en dos.

**Y se dice cuál de los dos topes frena.** «Tu campaña tarda cuatro meses» deja
sin saber qué tocar. «Tarda cuatro meses **porque tu tope diario es 50**,
cuando tu horario daría para 90» dice exactamente dónde está el freno. Sin esto,
lo normal es ampliar el horario de 9-18 a 8-20 y no notar ninguna diferencia.

**Hay dos respuestas que no son una fecha**, y las dos importan más que la
fecha:

- **«Con este tope no saldría ni un mensaje.»** Con un límite mal puesto, la
  campaña quedaría marcada como enviando y no enviaría nada — sin fallar, sin
  avisar, y sin nada roto que mirar. Es el fallo más difícil de diagnosticar de
  todos.
- **«No termina en cinco años: 435 000 mensajes se quedarían fuera.»** Decir
  «terminará el 3 de marzo de 2041» sería correcto e inútil.

El cálculo **simula hora a hora** en vez de dividir, y por eso acierta en tres
casos que una división se salta: que hay dos topes y manda el más pequeño; que
el día en que se activa casi nunca está entero (quien activa a las 16:30 con
horario hasta las 18:00 tiene una hora, no nueve); y que los días del cambio de
horario duran veintitrés o veinticinco horas.

**Un fallo que encontró su propia prueba.** El contador de días se apoyaba en la
lista recortada de catorce días que se enseña en pantalla. A partir del día
quince esa lista deja de crecer, y cada hora de envío pasaba a contarse como un
día nuevo: la campaña de cinco mil correos decía **444 días** en vez de 100.
Nadie lo habría notado — 444 es un número perfectamente plausible para una
campaña larga, sólo que cuatro veces mayor que el real.

### Las ocho comprobaciones de antes de activar, con una tercera respuesta

Activar es irreversible en la práctica: la audiencia se congela y lo que sale no
vuelve. Los fallos que esto caza son todos del mismo tipo — cosas que **no
fallan al activar**, sino tres días después:

- la contraseña de la cuenta caducó y no sale nada;
- la audiencia se quedó vacía al aplicar las bajas;
- una variable de la plantilla no se rellena para 340 contactos, y salen 340
  correos que dicen «Hola {{nombre}}»;
- el horario configurado no alcanza.

**La parte importante es la tercera respuesta.** Cada punto puede salir «bien»,
«hay que arreglarlo» o **«sin comprobar»** — y sin comprobar **no es un
aprobado**. Varias de estas comprobaciones necesitan piezas que llegan después
(los proveedores reales son de la Fase 5, las plantillas de la Fase 6). Un
preflight que diera por bueno lo que todavía no sabe mirar sería peor que no
tenerlo.

Es la misma regla que ya aplicamos a las pruebas de navegador: **omitida no es
pasada, y se dice**. Un punto sin comprobar impide activar igual que uno que
falla, y las dos cifras se cuentan aparte porque lo que hay que hacer es
distinto: uno se arregla, el otro se comprueba.

**Siete de los ocho bloquean.** El único que no es el de los registros del
dominio: sin ellos los correos entregan peor, pero entregan, y bloquear por eso
dejaría fuera a quien no administra su propio DNS por algo que ARLES no le puede
arreglar.

### Pausar, Reanudar y Detener — y que el botón signifique lo que dice

Tres botones que parecen lo mismo y son dos cosas muy distintas:

| Botón | Qué le pasa a la cola | ¿Se puede deshacer? |
|---|---|---|
| **Pausar** | Se conserva **intacta** | Sí |
| **Reanudar** | Sigue donde estaba | — |
| **Detener** | Lo que no salió se cancela | **No** |

Pausar es «espera un momento». Detener es «esta campaña se acabó», y no se
deshace: volver a encolar lo cancelado significaría reconstruir una audiencia
que se congeló al activar y que quizá ya no exista igual.

Por eso detener **pide confirmación y dice cuántos mensajes se van a quedar sin
enviar**. «¿Seguro?» no es una confirmación informada; «esto cancela 4 312
correos que no han salido» sí. Y hay una prueba que comprueba que **la cifra que
se avisa es exactamente la que se cancela** — si miraran cosas distintas, la
pantalla prometería una cosa y ocurriría otra sobre algo irreversible.

**Lo que detener NO toca.** Los correos que ya salieron, y los que están
saliendo en ese preciso instante. Cancelar uno que ya se le pasó al proveedor
diría que no salió, y puede que sí. Esa fila la termina el trabajador que la
tiene en la mano; lo que impide que siga con la siguiente es el freno.

**Pausar ahora para el motor de verdad.** Hasta esta entrega, el freno del bucle
estaba puesto pero no había nada detrás: pausar habría cambiado el rótulo de la
pantalla mientras los correos seguían saliendo. Es la clase de mentira que este
producto no se puede permitir, y hay una prueba de punta a punta que la caza:
se manda uno, se pausa, y no sale ni uno más.

**La parada de emergencia sobrevive a cerrar el programa.** El caso concreto:
alguien se da cuenta de que la campaña que acaba de activar lleva la plantilla
equivocada, pulsa parar y cierra el programa para irse. Guardada sólo en
memoria, al día siguiente ARLES arrancaría sin parada y seguiría enviando la
plantilla equivocada a los cuatro mil que faltaban. Ahora se guarda en la base
(migración V6).

Se guarda **desde cuándo**, no un sí/no, para que la pantalla pueda decir
«detenido desde las 14:32» en vez de un «detenido» a secas que invita a
preguntarse si se quedó puesto de ayer. Volver a pulsar no reescribe esa hora.

**Y levantar la parada no reanuda nada por su cuenta.** Las campañas en pausa
siguen en pausa. Una parada de emergencia se pulsa porque algo iba mal; lo
último que debe hacer el botón de quitarla es poner a enviar todo a la vez.

**Ante la duda, se para.** Si la base no puede contestar si hay que parar —un
estado corrupto, una base tocada por fuera—, el motor se detiene. Los dos
errores posibles no se parecen: parar de más se ve, porque la campaña no avanza
y alguien pregunta; enviar de más no se ve hasta que los correos ya salieron.

### El bucle que junta todas las piezas, y un cerrojo para que no haya dos ARLES

Hasta ahora había una cola que sabe repartir trabajo sin duplicarlo, un
presupuesto que sobrevive a cerrar el programa, un horario con su zona, una
política de reintentos y un proveedor de mentira. **Cinco piezas que no se
hablaban.** Esto es lo que las junta.

El bucle hace, por cada correo y en este orden:

1. ¿Alguien pulsó parar? Se para.
2. ¿Estamos dentro del horario de la campaña? Si no, se espera — **sin tocar la
   base de datos siquiera**.
3. ¿Queda cupo en esta hora y en este día? Si no, se pausa y se avisa. **No se
   busca otra cuenta**: cambiar de remitente al agotar el cupo es lo que hace el
   software de spam, y es como se consigue que un proveedor cierre todas las
   cuentas de un dominio a la vez.
4. Se toma un envío de la cola, y **sólo uno**.
5. ¿Esta persona pidió la baja? Se consulta **ahora**, no al preparar la
   campaña. Con cincuenta correos diarios, una campaña de cinco mil dura cuatro
   meses; quien pidió la baja en octubre no puede seguir recibiendo en enero.
6. **¿Alguien pulsó parar, otra vez?** Entre el punto 1 y aquí ha habido dos
   consultas a la base: tiempo de sobra para que alguien pulse el botón. Y el
   botón significa «ningún correo más», no «un correo más y paro».
7. Se anota en la base que se está enviando, **antes** de llamar a nadie. Es lo
   único que permite reconocer después un envío del que no se sabe si salió.
8. Se envía, y se anota lo que pasó.

**Trece pruebas**, y tres de ellas se comprobaron quitando la garantía a
propósito para ver que fallaban: sin la comprobación del horario, un correo sale
de noche; sin la consulta de bajas, se le escribe a quien pidió no recibir; sin
la segunda comprobación de la parada, sale un correo después de pulsar parar.

**El cerrojo de instancia única.** Dos ARLES abiertos sobre la misma base no
producen correos duplicados —eso lo impide el esquema—, pero sí algo casi peor:
al arrancar, ARLES marca como «envío no confirmado» todo lo que quedó a medias,
porque sólo una caída deja filas así. Un segundo ARLES que arranque mientras el
primero envía marcaría como dudosos **los envíos que el otro está haciendo bien
en ese momento**, y el usuario acabaría con una lista que tiene que revisar uno
a uno para nada.

El cerrojo es un archivo con un **latido**: la hora, reescrita cada pocos
segundos por el propio bucle. Por eso llega ahora y no antes. Sin latido, un
ARLES que muere de golpe deja el archivo puesto y **nadie puede volver a abrir
el programa**, que es un fallo bastante peor que el que se quería evitar. Con
latido, el arranque distingue «hay alguien dentro» de «esto se quedó puesto».

**Tres cosas que aparecieron por el camino:**

- **Tres comprobaciones de la documentación del código llevaban fallando desde
  la entrega 4.2a**, y yo había reportado las pruebas en verde. Eran ejemplos de
  documentación que Rust intentaba compilar como si fueran programas. No
  afectaban al producto, pero sí a lo que yo estaba afirmando. Corregidas, y
  ahora entran en la comprobación de cada entrega.
- **La máquina de estados describía un programa distinto del que se ejecuta.**
  La cola toma directamente los envíos fallidos cuando llega su hora de
  reintento; la tabla de estados decía que ese paso no existía. Ninguna de las
  dos fallaba: simplemente no hablaban del mismo sistema. Hay una prueba nueva
  que las enfrenta.
- **Una credencial caducada gastaba reintentos que nadie había usado.** Si la
  contraseña de la cuenta deja de servir, ningún envío va a funcionar hasta que
  una persona la arregle; contarlo como intento fallido habría dejado a media
  audiencia sin reintentos contra una puerta cerrada. Ahora el intento se
  devuelve intacto y el motor se detiene diciendo por qué.

### Un proveedor de mentira al que se le puede ordenar que falle

Las decisiones más delicadas del motor son todas respuestas a un fallo: qué
hacer si se cae la red, si la contraseña caducó, si el servidor pide esperar, si
la dirección no existe.

**Y no se le puede pedir a Gmail que se caiga a voluntad.** Sin un proveedor al
que ordenarle que falle, esas partes del programa sólo se ejercitarían el día
que fallaran de verdad — en mitad de la campaña de un cliente, que es el peor
momento posible para descubrir que estaban mal escritas.

Ahora existe ese proveedor, con **seis formas de fallar a la orden**: se cayó la
red, tarda demasiado, «espera diez minutos», esa dirección no existe, la
contraseña no sirve, y contestar basura.

**Y deja constancia de lo que le pidieron.** Eso permite afirmar algo que de
otro modo habría que suponer: que a una persona dada de baja **no se la llamó ni
una vez**. Mirar la base de datos diría «no consta un segundo envío»; mirar lo
que recibió el proveedor dice «no se pidió», que es lo que de verdad importa —
lo que llega al proveedor ya no se puede retirar.

**Un defecto encontrado por su propia prueba.** El texto que devuelve un
servidor se limpia antes de guardarlo, porque hay caracteres que **dan la vuelta
al texto**: un servidor hostil puede hacer que un mensaje de error parezca decir
otra cosa. El filtro que escribí no los quitaba — resulta que ese carácter no es
técnicamente «de control», que es lo que el filtro miraba.

Ese mismo ataque ya se había resuelto en la importación de archivos, hace dos
semanas, con una lista de caracteres escrita allí. **Copiarla habría dejado dos
listas que se separan**: alguien añade un carácter a una, nadie se acuerda de la
otra, y el agujero reaparece en el sitio que no se tocó. Ahora la lista vive en
un solo sitio y la usan las dos.


### El presupuesto sobrevive a cerrar el programa

Cerrar ARLES y volver a abrirlo **no** devuelve a cero la cuenta de «cuántos
llevo hoy».

**Por qué importa.** Con un contador en memoria, gastar el límite diario varias
veces el mismo día sería trivial, y ni siquiera haría falta intención: basta
cerrar la ventana y volver a abrir. Usted no vería nada raro —la aplicación
diría que lleva cero— y el proveedor sí: tres veces el límite diario desde la
misma cuenta es exactamente el patrón que marca una cuenta como spam.

**Cómo se reinicia sin que nadie lo reinicie.** No hay ninguna tarea que borre
contadores a medianoche. Cada contador guarda **a qué hora y a qué día
pertenece**, y al sumar se compara con la actual: misma etiqueta, suma; etiqueta
distinta, empieza de nuevo.

Así el reinicio ocurre solo, sin relojes y sin importar cuánto tiempo estuvo el
programa cerrado. Un contador que dependiera de una tarea a las 00:00 fallaría
justo en el caso que más importa: con el equipo apagado.

**Y «hoy» significa hoy aquí.** El día se mide en la zona horaria de su empresa,
no en la del meridiano de Greenwich. A las 23:00 de Ciudad de México ya es el
día siguiente en UTC: un contador que usara UTC renovaría su límite diario a las
18:00 hora de México, y al día siguiente se quedaría corto. Nadie relacionaría
eso con un huso horario.

**Una distinción que quedó escrita.** El **horario de envío** es de la campaña
(D-10), porque decide a qué hora recibe el destinatario. El **día del
presupuesto** es de la empresa, porque el presupuesto cuelga de la cuenta
remitente y una cuenta puede servir a varias campañas. Si cada campaña midiera
su propio «hoy», la misma cuenta tendría dos días en curso a la vez y el límite
se podría gastar dos veces.


### La cola que no duplica · entrega 4.2

Es la pieza que concentra el riesgo de toda la Fase 4, y toda ella existe por un
solo motivo: **que nadie reciba un correo dos veces**.

**Por qué un duplicado es el peor fallo posible de ARLES.** Es invisible para
nosotros —nada falla, ningún registro lo delata—, irreversible para quien lo
recibe, y acumulativo para su dominio: los duplicados alimentan la tasa de queja,
y la reputación de un dominio se pierde en un día y se recupera en meses.

Un correo que **no** sale, en cambio, es visible y se arregla: usted lo ve y lo
reenvía. Esa asimetría gobierna todas las decisiones de esta entrega.

**El caso que hay que resolver.** ARLES está mandando un correo. Justo entre que
el proveedor lo acepta y que ARLES lo anota, se va la luz. Al volver a abrir, la
base dice «enviando». ¿Salió o no salió?

No hay forma de saberlo: averiguarlo exigiría leer su bandeja de enviados, un
permiso que ARLES no pide a propósito. Así que **no se reenvía nunca solo**. Se
marca como «envío no confirmado», con el nombre y la hora, y usted decide.

**Y lo que sí vuelve a la cola.** Si la caída ocurrió *antes* de llamar al
proveedor, no existe ningún correo que pueda estar duplicado: eso se reencola y
se envía con normalidad. Distinguir los dos casos es todo el trabajo — tratarlos
igual perdería envíos sin motivo, o los duplicaría.

**Tres capas, y ninguna sobra.** La base de datos rechaza por sí sola dos envíos
a la misma persona en la misma campaña, aunque el código falle. Encima, un
seguro impide que dos partes del programa se lleven el mismo envío a la vez. Y
cada correo lleva una marca única generada **antes** de mandarlo.

**Una prueba que no valía, descubierta poniéndole el cebo.** La del seguro
—la garantía más importante del módulo— **pasaba con el seguro quitado**. Estaba
comprobando algo distinto de lo que creía: el filtro que elige el candidato, en
vez del seguro que impide la carrera. Se sacó el seguro a su propia función para
poder probarlo de frente, y ahora sí falla cuando se rompe.


### Los reintentos, con las esperas que fijó Dirección (D-12)

Cuando un envío falla por algo pasajero —se cayó el internet, el proveedor está
saturado, se fue la luz— ARLES vuelve a intentarlo.

**La propuesta original era la de manual:** 30 segundos, 1, 2 y 4 minutos, cinco
intentos. Dirección la cambió con un argumento que es correcto: *un apagón o una
caída de proveedor demoran bastantito*.

Aquella tabla daba **siete minutos y medio** de margen. Los cinco intentos se
habrían gastado enteros dentro del mismo corte, y todos los envíos habrían
muerto por un problema que se resolvió solo veinte minutos después.

| | Espera |
|---|---|
| Reintento 1 | **5 minutos** |
| Reintento 2 | **15 minutos** |
| Reintento 3 | **30 minutos** |
| Después | Se rinde, con su motivo |

**Cincuenta minutos de margen**, y una prueba que lo comprueba para que nadie
recorte las esperas sin ver lo que pierde.

**Lo que hubo que recalibrar.** Cada reintento espera un rato ligeramente
distinto al azar, para que mil envíos fallidos no reintenten en el mismo
instante y tumben el servidor que acaba de volver. La receta habitual es esperar
«entre cero y el máximo» — y con estas cifras algunos reintentos saldrían a los
diez segundos, **anulando la decisión**. Así que el azar rodea el número en vez
de partir de cero: ±20 %. Cinco minutos pasa a ser entre cuatro y seis.

**Lo que no cambia.** Si el proveedor dice cuánto esperar, se respeta
literalmente, por encima de la tabla. Y un fallo definitivo no se reintenta ni
una vez: una dirección que no existe se da de baja en el acto, y una credencial
caducada para la cuenta entera. Insistir ahí no recupera nada y sí quema la
reputación del dominio.

**Un defecto propio, encontrado por su propia prueba.** El código recortaba el
número aleatorio a un rango seguro, y `clamp` **devuelve NaN si recibe NaN** —
con lo que la multiplicación siguiente reventaba el proceso. Era una defensa que
terminaba en pánico, que es peor que no defenderse: se llevaría la campaña por
delante en vez de enviar un correo un minuto antes.


### El horario de envío es de la campaña · y una dependencia que mentía sobre México

**La decisión (D-10).** Dirección lo planteó con una frase que reordena el
problema: *no importa a qué hora lo envíe el remitente, sino a qué hora lo
recibe el destinatario*. Una ventana de envío existe para que el correo caiga en
la bandeja de alguien a una hora en que esa persona lo lea.

Así que la ventana **pertenece a la campaña**, no a la empresa. Una campaña
«Clientes México · L-V 9-17 · hora de México» y otra «Clientes España · L-V 9-17
· hora de España» corren a la vez desde el mismo equipo, esté donde esté. Si el
portátil viaja, no cambia nada: es justo lo que la regla impide.

**Lo que la pregunta destapó.** La documentación del motor decía «la zona
horaria de la empresa», en singular. La base de datos soportaba una por ventana
desde el primer día. Lo equivocado era el texto, no el diseño.

**Y ahora lo importante: la dependencia que mentía.**

Para saber qué instante son «las 9:00 en Ciudad de México» hace falta una base
de datos de zonas horarias. Se eligió la opción obvia — la que extiende la
biblioteca de tiempo que el proyecto ya usaba. **La primera prueba que se le
puso la tumbó.**

Esa base **sigue aplicando horario de verano a México**. Daba UTC−5 en junio y
en septiembre de 2026. México abolió el horario de verano en **octubre de 2022**:
Ciudad de México es UTC−6 los doce meses del año desde entonces.

Con esa base, **todas las campañas mexicanas habrían enviado una hora fuera de
su horario durante medio año**. En silencio. Sin que fallara nada. Y México es
el mercado principal de ARLES, así que nadie lo habría notado hasta que un
cliente preguntara por qué sus correos llegan a las siete de la tarde.

Medido, y por eso se cambió de biblioteca:

| | enero | junio | septiembre |
|---|---|---|---|
| La que se descartó | UTC−6 | **UTC−5** | **UTC−5** |
| La que se usa | UTC−6 | UTC−6 | UTC−6 |

**Hay una prueba que fija esto para siempre**, mes a mes, los doce. No comprueba
nuestro código: comprueba **la dependencia**. El día que alguien suba su
versión, esa prueba falla si la nueva trae las reglas viejas.

La base va **incrustada en el programa**, no leída del sistema operativo. No es
sólo porque Windows no la tenga: es para que la misma campaña calcule igual en
Windows, macOS y Linux. Leyéndola del sistema, un equipo con la base vieja
reproduciría exactamente el fallo que acabamos de descartar.

**Y el cambio de horario de verano, probado.** España pasa a horario de invierno
el 25 de octubre de 2026. Una ventana de 9 a 18 sigue significando de 9 a 18
antes y después, sin hueco ni doble envío. Eso funciona porque se guarda el
**nombre** de la zona y no su desfase: un desfase cambia dos veces al año y un
nombre no.


### Fase 4 · arranca el motor, por el reloj

La Fase 4 es la que de verdad envía, y el roadmap la marca como la que
concentra el 70 % del riesgo del proyecto. Se empieza por donde no hay ni base
de datos ni red: el dominio puro.

**El problema que resuelve este primer trozo**, explicado con el caso real:

Usted programa ARLES para mandar 10 correos por hora. Cierra la tapa del
portátil y se va a dormir. Ocho horas después lo abre.

Un contador normal razonaría así: «han pasado ocho horas, me tocan ochenta
permisos» — y soltaría ochenta correos en un minuto. Eso es exactamente lo que
hace que un proveedor marque su cuenta como spam, y es irreversible: la
reputación de un dominio se pierde en un minuto y se recupera en meses.

**ARLES reinicia el presupuesto en vez de acumularlo.** La consecuencia hay que
decirla porque es real: **un correo que no salió ayer no se recupera hoy**. La
campaña dura más. Si el simulador dijo cien días y el equipo estuvo apagado
tres, dura ciento tres. Eso es lo esperado, no un fallo.

**Cómo se detecta que el equipo durmió.** Se comparan dos relojes: el que sigue
corriendo mientras el equipo duerme y el que no. La diferencia entre los dos
**es** lo que duró la suspensión. Hacen falta los dos: uno solo no distingue
«durmió ocho horas» de «alguien cambió la hora del sistema», y el otro no puede
ver la suspensión en absoluto, que es su definición.

Once pruebas nuevas, y las cuatro garantías se vieron fallar antes de darlas por
buenas: el cubo que acumula en vez de reiniciar, la resta que revienta cuando el
reloj se atrasa, la ventana de horario que deja de comprobarse, y el «mayor o
igual» cambiado por «mayor» — que dejaría enviar exactamente **uno de más** en
cada ventana, y ese uno es justo el que cruza el umbral que vigila el proveedor.


### La documentación se comprueba sola

Dirección pidió revisar toda la documentación de GitHub. Son **66 documentos**.
Leerlos de uno en uno no escala: para cuando terminas el último, el primero ya
cambió. Así que en vez de leerlos, ahora se miden.

`herramientas/validar/documentos.py` corre en cada validación y comprueba que
ningún enlace lleve a un archivo o a una sección que ya no existe, que ninguna
cifra afirmada haya caducado, que ningún documento prometa un anexo que nunca se
escribió, que el índice nombre todo, y que **ningún identificador de modelo**
acabe en el repositorio — una regla dura del proyecto que hasta hoy no vigilaba
nadie.

**Encontró cuatro cosas que ya no eran ciertas.** Tres documentos decían «las
ocho preguntas» para el abogado, cuando son dieciocho. Y la consulta jurídica
anunciaba «nueve preguntas nuevas: C-9 a C-18»: de C-9 a C-18 hay **diez**. Las
dos mitades de esa frase eran creíbles por separado, y contarlas a mano es
exactamente lo que nadie hace.

**Y diez documentos que el índice no nombraba**, entre ellos la propia consulta
jurídica y el informe de fase para Dirección. Un mapa que no nombra media
biblioteca es peor que no tener mapa: parece completo.

**Dos distinciones que la herramienta tuvo que aprender.** Una cifra **anclada a
un momento** —«133 pruebas al cerrar la entrega 3.1»— es un registro, no una
afirmación sobre hoy; reescribirla destruiría lo que un documento de fase sirve
para guardar. Y **citar un error no es cometerlo**: el informe que cuenta estos
hallazgos tiene que poder escribir la frase equivocada sin que salte la alarma,
o el remedio sería callarla.

**Y una de las comprobaciones nuevas no valía.** La que vigila que un rango de
preguntas cuadre con su recuento decía «todos correctos» sin haber mirado
ninguno: su expresión no sabía de negritas, así que contra la frase real no
casaba nunca. Se descubrió porque el error ya se conocía de antemano. Si no,
habría pasado por buena.


### Fase 3 cerrada · las cinco entregas

`validar.py --fase 3` da **77 de 77, con 0 omitidas**. «Omitida» no es «pasa»:
significa que una comprobación no se pudo correr, y el validador las cuenta
aparte precisamente para que nadie las confunda. Hoy no hay ninguna.

Por debajo: **386 pruebas** del motor, **102** de las pantallas y **9 sondas**
que abren un navegador de verdad, pulsan y miden.

**Lo que la fase deja listo.** Siete pantallas —Inicio, Configuración,
Contactos, Importar, Bajas, Derechos y Ajustes—, la base cifrada con cinco
migraciones, la lectura defensiva de archivos, y las reglas de validación de
correos y móviles mexicanos. Es todo lo que hay que tener **antes** de poder
mandar un solo mensaje: de qué empresa salen los envíos, a quién se le puede
escribir, cómo entran esos contactos, y a quién no se le vuelve a escribir.

**La puerta de salida que más costó.** Dar de baja a alguien, borrar su
contacto, reimportar la lista original seis meses después, y comprobar que
**sigue sin escribírsele**. Es el fallo más caro posible en este producto
porque no se nota hasta que alguien que pidió no recibir nada recibe algo — y
entonces ya es una queja ante el INAI, no un error interno.

**Y el informe para Dirección**, en llano, con su PDF: `FASE-03-PARA-DIRECCION`.
El validador comprueba que el PDF corresponde al texto que lo origina, porque un
PDF es binario y si se queda atrás nadie lo nota en una revisión.

**Sobre el diseño.** Dirección revisó la dirección visual el 22/09 y la
considera genérica. Queda registrado, con el motivo: lo que se hizo fue pulido
sobre lo que ya había, y lo genérico no es el color sino la composición, que no
se tocó. Decisión de Dirección: se retoma cuando tenga referencias. No bloquea
nada.


### Dirección visual · primer tramo: el armazón

Dirección pidió el 22/09/2026 una aplicación que se sienta cara y única, con
degradados, contraste y dinamismo entre pestañas — «super fluida y sin bugs».
Esa última parte es la que mandó sobre todas las decisiones.

**Lo que se ve.**

La barra lateral y las tarjetas ganan profundidad. El botón de acción deja de
ser un rectángulo amarillo y pasa a tener volumen, como una pieza de metal con
luz encima. Al pulsar cualquier botón, el botón cede bajo el dedo. Y al cambiar
de pantalla, la nueva llega subiendo, en vez de aparecer de golpe.

**La pieza de firma: el indicador de sección viaja.** Antes cada sección tenía
su propia barrita de oro, que se encendía y se apagaba. Ahora hay **una sola**,
que se desliza de una sección a la otra. Seis marcas que parpadean dicen «este
estado se apagó y aquel se encendió»; una que viaja dice «eres tú quien se ha
movido, y de aquí a allí».

**Cómo se consiguió que no cueste fluidez.** Todo lo que se mueve lo hace con
las dos únicas propiedades que la tarjeta gráfica compone sin rehacer la
página. Ninguna animación toca medidas ni posiciones, así que esto cuesta lo
mismo con una tabla de 500 000 contactos que con una pantalla vacía. Los
degradados van como una capa transparente **encima del color sólido**, que se
deja intacto: así el contraste se sigue midiendo contra un color real, y contra
el extremo más desfavorable.

**Y qué pasa si algo falla.** El indicador que viaja es la única pieza de la
barra que depende de medir la pantalla, y medir falla de maneras que no avisan.
Está escrito para que sea **un adorno encima de un estado que ya está dicho de
otras dos formas**: si la medida falla, el indicador no se dibuja y la barra
vuelve exactamente al dibujo anterior. Hay tres pruebas que lo comprueban, y
una de ellas corre en el modo degradado de verdad.

**Dos defectos encontrados por el camino, los dos propios.** Al meter el
indicador dentro de la lista se rompió un selector de una sonda, que se quedó
media hora esperando un elemento que existía y que ya no podía nombrar: nada
fallaba en la pantalla, lo que se rompió fue la capacidad de comprobarla. Y una
prueba del indicador pasaba con el defecto clásico de este patrón puesto a
mano —medir antes de que la pantalla se dibuje, y quedarse una sección por
detrás—, porque sólo miraba el primer montaje y el defecto sólo aparece al
navegar. Las dos cosas están corregidas y con su comprobación.

### Auditoría de las diez pantallas · primer tramo

Se recorrió el producto entero en un navegador de verdad, a tres anchos, y se
midió en vez de mirarlo a ojo. Tres hallazgos, y una herramienta nueva para
que no dependan de que alguien mire.

**1 · La vista previa mandaba a Ajustes.** El 18/09 se cambió «Configurar tu
empresa» para que llevara a la configuración y no a Ajustes. Se cambió en el
núcleo, y la aplicación real llevaba bien desde entonces. **El simulador de la
vista previa no se tocó**, así que seguía llevando a Ajustes — y la vista
previa es donde Dirección revisa. Ése es el motivo de que el mismo defecto se
reportara dos veces: la segunda vez ya no existía en el producto, sólo en la
copia que se estaba mirando.

Ahora hay una comprobación que compara las rutas del simulador con las del
núcleo y falla si vuelven a separarse. Se probó con los dos cebos: poniendo
otra vez `/ajustes`, y dándole al núcleo una ruta que el simulador no tuviera.

**2 · Un botón apagado parecía pulsable.** En el tema claro, «Preparar el
informe» conservaba su contorno de oro estando deshabilitado, porque la regla
que le pone ese contorno pesaba más que la que apaga los botones. Texto
apagado dentro de un marco de acción: a simple vista, un botón disponible que
no responde. Se corrigió, y el propio texto del componente ya decía lo
contrario de lo que el navegador hacía.

**3 · Lo que se midió y estaba bien.** Ningún error de consola en las diez
pantallas. Ningún texto por debajo de su mínimo de contraste, en claro y en
oscuro. Nada recortado ni fuera de la ventana a 1280, 1024 ni 900 px. Ochenta
y ocho paradas de tabulador, todas con marca de foco visible. Ningún botón sin
nombre, ningún salto de encabezado, ningún identificador repetido.

**La herramienta.** `npm run sonda:auditoria` repite todo eso en cada
validación. Sus nueve comprobaciones se rompieron a propósito una a una antes
de darlas por buenas, y dos no valían: la de «contenido que no cabe» preguntaba
algo que no podía ser cierto nunca, y su segunda versión señalaba el pie de la
barra plegada diez veces por recorrido sin que hubiera nada que arreglar.

### Ya se pueden borrar los datos de alguien · entrega 3.4 completa

El motor estaba construido desde ayer. Ahora tiene botón, y todo el trabajo
se fue en que ese botón **no se pueda pulsar por accidente** — es la única
acción de ARLES que no tiene vuelta atrás.

**Cómo funciona, en tres pasos.**

1. Escribes la dirección de quien pidió el borrado y pulsas **«Ver qué se
   borraría»**. No borra nada.
2. ARLES te enseña **de quién es esa dirección, por su nombre**, y dos cifras
   grandes: cuántos datos **desaparecen** y cuántos envíos **se conservan** sin
   su nombre. Más abajo dice si esa persona ya tenía baja o si se le creará una.
3. Para borrar hay que **escribir su dirección completa**. Entonces, y sólo
   entonces, se enciende el botón rojo.

**Por qué escribir la dirección y no marcar una casilla.** Una casilla de
«entiendo que es irreversible» se marca sin leerla — la prueba es que todo el
mundo las marca. Escribir la dirección obliga a **mirarla**, y si te
equivocaste de persona al teclear arriba, aquí lo ves. Ése es el accidente
concreto que esto existe para evitar: borrar a quien no era, sin deshacer.

**Y cambiar la dirección de arriba desarma la confirmación.** Sin eso, alguien
podría mirar lo de Ana, confirmar escribiendo su correo, cambiar de idea,
teclear otra dirección… y el botón seguiría armado, apuntando ya a otra
persona.

**«No consta nada» no impide borrar**, y es deliberado: la baja se crea igual,
que es justo lo que impide que esa dirección vuelva a entrar en una lista.

**Lo que se dice después no es «listo».** Es cuántos datos desaparecieron,
cuántos envíos se conservan y qué pasó con su baja. Quien atiende la solicitud
va a tener que responderle eso mismo a una persona, y «listo» no le sirve.

**Dos comprobaciones que decían lo contrario.** Una prueba y una comprobación
de la sonda exigían que la pantalla avisara de que el borrado «todavía no está
construido». Ya lo está, así que ahora exigen lo contrario: que se llegue a él
y que no deje borrar sin confirmar. Se deja escrita la afirmación vieja en vez
de borrarla — explica por qué ese texto llegó a existir.

Y otras dos comprobaciones eran **demasiado anchas**: buscaban «no hay nada
rojo en toda la página», y eso dejó de valer en cuanto la pantalla ganó un
aviso rojo legítimo. Ahora miran el elemento concreto. Una comprobación sobre
la página entera no distingue «esto está mal pintado» de «hay algo rojo en
otra parte».

### El borrado de verdad, por dentro (entrega 3.4, sin pantalla todavía)

Es la parte con más tensión del producto: la ley da derecho a un **borrado
físico real** —no a marcar una ficha como borrada— y a la vez exige una
bitácora de auditoría, que por definición no se borra.

La resolución separa **el hecho** de **la identidad**:

| Qué | Qué pasa |
|---|---|
| Su ficha, sus campos, sus correos y móviles, sus listas y etiquetas | **Se borran de verdad.** Desaparecen de la base |
| Los envíos que se le hicieron y la audiencia congelada de cada campaña | **El hecho se conserva, el nombre se corta.** Queda «se envió un correo el 3 de febrero y rebotó», sin a quién |
| La bitácora | Anota que hubo una cancelación, con la dirección **en huella** |
| Su baja | **Permanece. Y si no la había, se crea** |

**Por qué la baja se crea y no sólo se conserva.** Cancelación y oposición son
derechos distintos: alguien puede pedir que borres sus datos sin haber pedido
antes la baja. Borrarlo sin dejar la baja abre el agujero peor del producto —
su dirección en un CSV la semana que viene, y ARLES escribiéndole a alguien que
pidió desaparecer. Y no lo detectaría nada: el envío «funcionaría».

**Por qué la marca que sustituye a la dirección es aleatoria y no una huella.**
Hay que poner *algo* donde estaba el correo. Lo cómodo es guardar su huella, y
está mal: una dirección de correo vive en un espacio adivinable, y esa huella
se revierte con una lista de correos y un rato de cómputo. Sería conservar a la
persona con otro nombre y llamarlo borrado. La marca es aleatoria y **distinta
en cada fila**: ni siquiera ARLES puede saber después si dos envíos fueron de
la misma persona — que es justo lo que se ha pedido destruir.

Casi se cuela un fallo ahí: los identificadores de ARLES llevan la hora dentro,
y si la marca la hubiera usado, todas las de una misma cancelación quedarían
consecutivas al ordenarlas. El vínculo habría seguido a la vista.

**La puerta de salida.** Hay una prueba que, después de cancelar, **recorre el
esquema entero de la base** —tabla por tabla, columna por columna,
preguntándoselo a SQLite— y exige que la dirección no aparezca en claro en
ninguna parte salvo en la lista de bajas. Se comprobó rompiéndola: quitando la
anonimización de los envíos falla nombrando exactamente dónde se quedó.

Esa misma prueba es el guardián de un hueco que encontramos construyendo y que
**no es nuestro de decidir**: la tabla donde se registrará el consentimiento no
admite borrado por diseño, y guarda la dirección en claro. Hoy no la escribe
nadie, así que no hay nada que borrar; el día que se escriba, esa prueba
fallará. Queda abierto como **P-17**, para el abogado.

**Todavía no hay botón.** Esto es el motor: el núcleo, la capa de datos y sus
garantías. La pantalla desde la que se dispara va aparte, porque un borrado
irreversible necesita que la interfaz avise antes y no después.

### Ya se puede atender a quien pregunta qué datos tienes de él

La ley mexicana obliga a responder cuando alguien lo pide. La máquina que
arma esa respuesta estaba construida y probada desde hace días; **lo que no
existía era la pantalla desde la que se pide**. Ahora está, en
**CONTACTOS → Derechos**.

Se escribe una dirección —de correo o de WhatsApp—, se pulsa un botón, y
ARLES deja un archivo de texto con todo lo que consta de esa persona: su
ficha, de dónde salió, y los envíos que se le hicieron. Ese archivo se le
envía y asunto resuelto.

**Lo que más cuidado llevó no es el caso bueno, son los otros dos.**

- **Cuando no consta nada**, el informe se entrega igual y la pantalla
  **avisa de que va vacío**. Si se callara, quien atiende la solicitud
  mandaría un archivo en blanco creyendo que va lleno — y nada habría
  fallado por el camino. Pero tampoco se pinta en rojo: «no tenemos nada
  suyo» es una respuesta legítima, y muchas veces la que la persona quería
  oír.
- **Cuando ya no es un contacto pero queda rastro** —su baja, sus envíos
  viejos—, se dice así. Alguien que pidió que lo borraran sigue teniendo
  huella, y el informe la incluye porque también es información sobre él.
  Decir «no hay nada» sería mentir en la peor respuesta posible.

Y cerrar la ventana de guardado sin guardar **no es un error**: es una
decisión, y se cuenta como tal. Tratarla como fallo mandaría a revisar una
base de datos que está intacta.

**La palabra «ARCO» no aparece en ninguna parte.** Es la sigla legal de
Acceso, Rectificación, Cancelación y Oposición, y no se la sabe nadie que no
trabaje en protección de datos. Quien atiende una solicitud necesita entender
qué hace el botón, no aprobar un examen. Hay una comprobación que falla si
alguien la escribe.

**Se dice quién responde**, y no es ARLES: es el cliente. ARLES no habla con
esas personas ni guarda nada en ningún servidor — sólo reúne lo que hay en
esa computadora. Quien manda el archivo lo firma en nombre de su empresa, y
conviene que lo sepa antes de mandarlo.

**Y se dice lo que todavía no se puede hacer.** El borrado de datos aún no
está construido, y la pantalla lo dice ahí mismo, junto a los otros derechos,
en vez de enterrarlo en la documentación. Quien reciba esa petición se entera
antes de prometérsela a nadie.

**Ninguna ruta de tu disco llega a la pantalla.** La ventana de guardar la
abre la parte de Rust, y lo único que vuelve es el nombre del archivo. La
carpeta donde guardas tus cosas dice dónde vives, y no le hace falta a nadie.

### Una puerta por cuarto, y un candado en vez de la misma frase tres veces

Tres decisiones de Dirección del 22/09/2026, tomadas sobre la vista previa.

**La subpestaña se llama «Bajas».** El nombre largo —«Quién pidió no
recibir»— se partía en dos líneas dentro de la barra. El **título de la
pantalla no se toca**: la barra es una brújula y necesita nombres cortos; el
encabezado es una explicación y puede permitirse una frase.

**Contactos pierde su enlace a las bajas.** Había dos puertas al mismo cuarto:
la de la barra y un enlace arriba a la derecha. Ese enlace lo escribí cuando la
barra no enseñaba ese destino — igual que el puente de Ajustes que Dirección
mandó quitar el día antes, por el mismo motivo y con el mismo desenlace.

El criterio queda escrito para lo que venga: **la barra lateral lleva a los
sitios, los botones hacen cosas.** Por eso «Cargar una tabla» y «Agregar
contacto» sí se quedan: importar y dar de alta son cosas que se hacen, no
sitios a los que se va. Que importar tenga pantalla propia no lo convierte en
destino; el asistente es el camino de la acción.

Y la lección, que ya es la segunda vez: **la respuesta a «no se encuentra» es
arreglar la navegación, no añadir un atajo.**

**El candado.** La frase «Lo pidió ella misma, así que ARLES no lo deshace» se
repetía, en dos renglones, en cada fila bloqueada — tres de cinco. Con
doscientas bajas es una tabla tres veces más larga para decir lo mismo tres
veces. Ahora la fila lleva un candado pegado al motivo —«Pidió la baja 🔒»— y
la frase entera aparece **una sola vez encima de la tabla**, explicando el
símbolo antes de que aparezca.

Medido: la primera fila pasa de **63 px a 45**.

**El candado nunca va solo**, y eso importa más que el ahorro: lleva su texto
para lectores de pantalla, la frase completa al pasar el ratón, y la leyenda
de arriba. Un icono suelto es información codificada sólo en la forma, que es
el mismo error que codificarla sólo en el color. La leyenda además **sólo
aparece si hay alguna fila con candado**: explicar un símbolo que no está en
pantalla es ruido, y haría dudar de si falta algo.

### Auditoría del 22/09: la pantalla de bajas estaba sin estilos, y nadie podía verlo

Dirección pidió revisar y auditar. Lo más grave no estaba en la navegación.

**La hoja de estilos de «Quién pidió no recibir» estaba muerta entera.** Se
escribió con catorce nombres de token **inventados** —`--espacio-3`,
`--color-texto-suave`, `--texto-sm`— cuando los del proyecto son
`--arles-space-3`, `--arles-text-muted`, `--arles-font-size-small`. Ninguno de
los catorce existía. Ni uno.

Lo que Dirección habría visto al abrir esa pantalla: una tabla **sin relleno,
sin separadores y sin jerarquía**; la etiqueta del canal pegada a la dirección
—«CORREOana@empresa.mx»—; la columna «Por qué» desbordándose sobre la fila de
abajo, de forma que no se podía saber a quién pertenecía cada motivo; y el
texto reservado a los lectores de pantalla **a la vista**.

**Por qué no lo cazó nada, que es lo que importa:**

- Un CSS que usa una variable inexistente **no es un error**. No avisa el
  compilador, ni el linter, ni el navegador: la declaración se descarta en
  silencio y la propiedad se queda como estaba.
- La comprobación de «sin literales» busca colores y medidas sueltas. Éstos
  eran `var(…)` perfectamente formados.
- La sonda de esa pantalla mide **comportamiento**: que la lista cargue, que
  busque, que el botón de reactivar salga sólo donde toca. Todo eso
  funcionaba. Una pantalla puede hacer lo correcto y ser ilegible.

**Y había una comprobación que debía cazarlo y no podía.** Existe desde la
Fase 2 una que dice «todo token que se usa está definido». Miraba **sólo los
nombres que ya empiezan por `--arles-`**. Cazaba una errata dentro de la
familia y era ciega a una familia entera equivocada — es decir, a este fallo.
Llevaba meses en verde porque no tenía forma de fallar. Es el caso de manual de
la regla de la casa: *una comprobación que nunca se ha visto fallar no está
verificada*. Corregida y comprobada rompiéndola por los dos lados.

**Otro defecto de la misma pantalla, encontrado mirándola:** dos celdas de la
tabla tenían `display: flex` puesto sobre el `<td>`. Eso saca la celda del
algoritmo de tabla y sus líneas dejan de alinear con las de la fila — medido:
la primera columna cerraba a una altura y las otras a otra. Ahora las cinco
cierran en el mismo píxel.

**Y un hallazgo en lo que acababa de entregar.** El relleno que marca la
subsección activa da **1.76:1** en tema oscuro y **1.80:1** en claro sobre la
barra, por debajo del 3:1 mínimo. La sección madre aguanta ese relleno flojo
porque **además** lleva su barra de acento amarilla; la subsección no llevaba
nada, así que su estado vivía sólo en el color — justo lo que la regla §19
prohíbe. Ahora lleva su propia barra, más corta que la de la madre. Lo mide la
sonda leyendo la altura real en el navegador.

Escribí en el código que la línea vertical que une las subsecciones con su
sección servía de ancla sin color. **Medirla lo deshizo**: es un filete de un
píxel y no distingue cuál de las hijas está abierta. Dibuja la jerarquía, no el
estado. El comentario está corregido para decir lo que hace y no lo que yo
esperaba que hiciera.

### La barra lateral ya dice en qué sección y en qué subsección estás

Dirección entró en la configuración recién mudada y señaló lo siguiente: **la
barra lateral estaba entera apagada.** Sus palabras: «no tengo nada que me
indique en qué sección me encuentro; necesito un indicativo, tanto de la
pestaña como de la subpestaña».

**Y el fallo era más viejo y más ancho de lo que se veía.** La barra usaba el
marcado automático de la librería de navegación, que no compara rutas por
texto: comprueba si la pantalla actual *desciende* de la entrada de la barra. Y
las subpantallas están declaradas como hermanas, no como hijas. Medido:

```
/inicio                 → casa con  ['/inicio']
/inicio/configuracion   → casa con  ['/inicio/configuracion']
/contactos/importar     → casa con  ['/contactos/importar']
/contactos/supresion    → casa con  ['/contactos/supresion']
```

Es decir: **Importar contactos** y **Quién pidió no recibir** dejaban la barra
apagada **desde la entrega 3.2**. Se arreglan las tres, no sólo la que se vio.

**Qué se ve ahora**, en tres piezas que se sostienen entre ellas:

- La **sección** queda marcada, con relleno y barra de acento, también dentro de
  sus subpantallas. Sale de lo que cada ruta ya declara, así que el día que se
  añada «Campañas › Plantillas» se alumbra sola.
- Bajo la sección abierta —y **sólo** bajo ella— aparecen sus **subsecciones**
  indentadas y unidas por una línea, con la actual marcada. Las nueve a la vez
  convertirían la barra de brújula en índice.
- Sobre el título, **migas**: `Inicio › Configuración`, con la primera clicable.

Son dos indicadores y no uno por una razón medida: plegada, la barra sólo enseña
iconos, así que las sub-entradas no se dibujan. Sin las migas, plegar la barra
borraría el único indicativo de subsección. Al revés no se pierde nada.

**Y por eso Ajustes se quedó sin el enlace a la configuración**, que Dirección
mandó quitar el mismo día junto con su párrafo. Ayer ese bloque estaba
justificado como la respuesta a «cómo se encuentra la configuración dentro de
seis meses». Puestas las dos cosas juntas se ve que la justificación era falsa:
**el bloque existía porque la configuración no salía en ninguna parte de la
barra**. Era un parche a este fallo de navegación, y lo único que hacía era
devolverle a Ajustes el vocabulario del negocio — justo lo que la corrección de
ayer existía para impedir.

La forma de que una pantalla no necesite un atajo no es escribirle el atajo, es
que el sitio al que lleva se pueda encontrar solo.

**Detalles que no se ven pero sostienen lo anterior:**

- El separador `›` de las migas es decoración de estilos, no un carácter en el
  texto: escrito en el marcado, un lector de pantalla leería «Inicio, mayor que,
  Configuración».
- Las migas **no salen en las pantallas raíz**. Una miga sola —«Inicio»— ocupa
  una línea para no decir ni de dónde vienes ni a dónde volver.
- Las subsecciones no llevan icono propio: las seis secciones se reconocen por
  el suyo incluso plegadas, y darle uno a una hija la pondría al mismo nivel
  visual que su madre.

**El agujero de pruebas, que es la tercera vez que aparece.** Ayer se descubrió
que el enlace de mentira de las pruebas no guardaba el destino; se le puso. Pero
**tampoco calcula el estado activo**, así que esto seguía sin poder verse. Se
arregló de otra forma: hay un archivo de pruebas nuevo que monta la aplicación
con el **enrutador de verdad** —trece comprobaciones— y la sonda mide lo mismo
en un navegador real. Verificado rompiéndolo: devolviendo la barra al marcado
automático caen las tres subpantallas, cada una nombrando su ruta.

### Configurar la empresa ya no manda a Ajustes, y lo guardado no se toca sin querer

Dirección lo vio mirando la pantalla: pulsar «Configurar mi empresa» en INICIO
seleccionaba la pestaña **Ajustes**. Su corrección, textual: «Ajustes es sólo
para los ajustes del sistema o de la aplicación —modo de visualización, tamaño
de fuente, información de la aplicación, servidores—».

Tenía razón, y **el error era mío**: el 17/09 moví la lista de los seis pasos a
Ajustes razonando que «Ajustes es donde se configura». Esa frase junta dos
cosas distintas: configurar *la herramienta* —tema, densidad— es una
preferencia; configurar *el negocio* —nombre, país, zona horaria— decide a qué
hora sale cada correo y bajo qué marco legal. No es una preferencia. Y los seis
pasos no son ninguna de las dos: son una guía **con final**, que desaparece al
completarse. Un ajuste no desaparece nunca.

**Qué cambia para quien usa ARLES:**

- Hay una pantalla nueva, **INICIO → Configuración**, con el formulario de la
  empresa y los seis pasos. El botón de la portada lleva ahí, y la sección que
  queda marcada en la barra sigue siendo INICIO.
- **Ajustes** se queda con la apariencia y un enlace a la configuración. Un
  enlace, no una copia del formulario: dos formularios sobre la misma tabla
  divergen en cuanto alguien toca uno.
- Al configurar por primera vez todos los campos están abiertos. **Al guardar
  se cierran**, y para cambiarlos hay que pulsar «Editar» a propósito —lo pidió
  Dirección para que no se modifiquen por accidente—. «Cancelar» devuelve lo
  que había guardado, no lo que se estaba escribiendo.

**Por qué el fallo sobrevivió tres entregas, que es lo que de verdad importa
aquí.** El doble de `RouterLink` que usan las pruebas de pantalla era
`<a><slot /></a>`: sin `href`. Es decir, **ninguna prueba de este proyecto
podía comprobar a dónde lleva un enlace**, y por eso ninguna vio que el botón
llevaba a Ajustes. El doble ahora pinta `:href="to"`, y hay dos pruebas nuevas
que afirman el destino. Se comprobó rompiéndolas: con el doble viejo fallan las
dos con el mensaje correcto.

Se añaden además dos comprobaciones al validador —Ajustes no lleva los datos
del negocio, y el primer paso del alta no manda a Ajustes— y la sonda de
navegación pasa a exigir que pulsar ese botón **no cambie de sección**.

**Sólo lectura no es «deshabilitado», y la diferencia se cuidó.** Un campo
deshabilitado se atenúa, sale del orden de tabulación y no se puede copiar —
justo en la pantalla que existe para leer el dato, y donde alguien puede querer
copiarse su correo corporativo. Los campos guardados conservan el **contraste
normal**, se pueden enfocar y seleccionar. Y como el HTML no tiene «sólo
lectura» para un desplegable, en ese estado no se dibuja un desplegable: se
escribe el valor elegido. Un desplegable que no despliega miente sobre lo que
hace.

El error de navegación queda **escrito** en `UX_NAVEGACION.md` §3.3 junto a la
decisión vieja, en vez de sobrescribirla en silencio: la decisión equivocada
explica por qué existía la pantalla que Dirección vio.

### Ya se puede ver quién pidió no recibir (entrega 3.4, en curso)

Dirección la pidió con estas palabras: «tenemos que tener una lista de los
prospectos que seleccionaron la cancelación de correos o WhatsApp». Está en
**CONTACTOS → Quién pidió no recibir**.

Cada fila dice la dirección, de quién es, **por qué** dejó de recibir y desde
cuándo. Arriba, dos cifras: cuántas por correo y cuántas por WhatsApp.

**La regla que gobierna la pantalla:** lo que dedujo una máquina se puede
deshacer; lo que pidió una persona, no.

Un rebote o un número inválido salen de la lista con un botón — el servidor
pudo equivocarse, el buzón pudo volver. Una baja que alguien pidió, o un correo
que marcó como no deseado, **no tiene ese botón**, y la fila dice por qué en vez
de dejar un hueco: *«Lo pidió ella misma, así que ARLES no lo deshace»*.

No es sólo que el botón no se pinte: **si alguien llamara al comando de todas
formas, el núcleo diría que no**. La regla vive en Rust, no en la pantalla. Si
viviera en la pantalla habría dos copias, y el día que divergieran ésta
enseñaría un botón que el núcleo rechaza.

Se puede sortear editando la base a mano, y no se pretende otra cosa: el
objetivo no es impedirlo, es que no pase sin querer.

**Detalles que parecen menores y no lo son:**

- El desplegable de motivos **no ofrece los seis**. `rebote permanente`,
  `número inválido` y `fallo del canal` los deduce el sistema al enviar;
  ofrecerlos a mano dejaría marcar «rebote» sin que hubiera habido ninguno, y
  el registro dejaría de explicar nada.
- Buscar dice **sobre cuántas filas** está buscando: «1 de las 4 que se están
  viendo. La búsqueda mira esta página, no la lista completa». Quien busque en
  una lista de mil creería, si no, que buscó en las mil.
- Dar de baja a un contacto entero manda **sólo su identificador**. Las
  direcciones las lee Rust de la base: entre que la pantalla cargó la ficha y
  alguien pulsó el botón pudo añadirse un canal, y darlo de baja «todo» con una
  lista vieja dejaría ese canal abierto.

**La sonda `sonda:supresion`** abre un navegador de verdad y mide las seis
afirmaciones. Se verificó poniendo el botón de deshacer en todas las filas: la
sonda falla y **nombra las tres** que no deberían tenerlo.

### La importación se atacó con archivos hostiles, y tres la rompieron

La entrega 3.3 tenía una puerta de salida escrita desde la Fase 0: *«los
archivos maliciosos con los que se atacó, y su resultado»*. Se construyeron
**trece**, y encontraron tres fallos que ninguna prueba anterior veía. Informe
completo para Dirección en
[`documentacion/06-calidad/ATAQUES-A-LA-IMPORTACION.md`](documentacion/06-calidad/ATAQUES-A-LA-IMPORTACION.md).

**1 · La defensa contra la bomba de descompresión no defendía.** El modelo de
amenazas decía «lectura en streaming con `calamine`» desde la Fase 0. Era falso:
`calamine` construye la hoja entera en memoria antes de devolver la primera
fila, así que el tope de celdas —que se contaba recorriendo filas— se comprobaba
cuando la memoria ya estaba gastada.

| | Antes | Después |
|---|---|---|
| Archivo en disco | 47 MB | 47 MB |
| **Pico de memoria al leerlo** | **+1 586 MB** | **+0 MB** |
| Error devuelto | `DemasiadosDatos` | `DemasiadosDatos` |

Las dos columnas devuelven **el mismo error**. Por eso llevaba ahí sin verse:
cualquier prueba que sólo mirara el error habría pasado en las dos. Ahora el
archivo se descomprime primero contando y tirando los bytes, con un tope de
300 MB, y sólo entonces se abre.

**2 · Doscientas mil filas vacías por delante colgaban la lectura.** Las filas
vacías de cabecera se quitaban de una en una desde el principio de un vector, lo
que cuesta el cuadrado de su número. Es un archivo plausible —un Excel al que
alguien borró el contenido de arriba— y no daba error: se quedaba pensando.

**3 · Un nombre de archivo podía dibujarse al revés.** `U+202E` invierte el
texto que viene detrás y es legal en un nombre de archivo:
`factura⁦U+202E⁩gnp.exe` se lee en pantalla como `facturaexe.png`. El nombre se
enseña en el asistente y se guarda como prueba de qué se importó, así que ahora
se le quitan los caracteres de control y de formato, y se recorta a 120.

**Lo que no se arregló, a propósito:** una celda de 50 MB entra. Pasa por debajo
del tope de celdas —es una— y del de bytes. Queda anotado en el modelo de
amenazas para que sea una decisión y no un olvido.

**Y una comprobación nueva en el validador de fase:** que la medición de la
descompresión vaya **antes** de abrir la hoja. Invertir esas dos líneas no rompe
ningún test que mire el error; sólo el que mide la memoria, y sólo si alguien se
acuerda de correrlo. Se verificó invirtiéndolas: el validador las caza.

Además, el cálculo de la huella del archivo decía en su comentario que iba en
trozos y acumulaba los 200 MB enteros en memoria. Ahora va de verdad en trozos
(`arles_core::HuellaEnCurso`), con la misma huella de salida.

### Google Drive y Sheets: decidido, y diferido a la v1.3 (D-8 · P-16)

Dirección preguntó si ARLES puede leer una hoja directamente de Drive. Sí se
puede, y se hará **en la v1.3**: entrarlo ahora lo pagaría la entrega 3.4, y cae
sobre la dependencia de Google que D-1 decidió evitar para esta versión.

Queda decidido por adelantado, para no volver a razonarlo: alcance
**`drive.file`** y **Google Picker** —nunca Drive completo, que es alcance
restringido y exige auditoría de seguridad externa—, la excepción a la CSP
acotada a los dominios del Picker, y el `refresh_token` al llavero (ADR-0011).

TELEMETRY tiene correo de dominio propio, así que la pantalla de consentimiento
va **«Interna»** y Google no tiene que verificar nada. Eso saca el riesgo de
calendario de la ecuación.

Mientras tanto, Sheets ya exporta con *Archivo → Descargar → CSV*, y ARLES
importa ese archivo hoy con todas sus defensas.

### Ya se puede cargar una tabla de Excel o CSV (entrega 3.3, completa)

Cuatro pasos: elegir el archivo, revisar qué es cada columna, **ver qué va a
pasar**, y confirmar. Hasta el último, la base no se toca.

El tercero existe porque Dirección decidió que ARLES **enseñe los choques antes
de importar**. Sale con cifras —«Entran 2 · se quedan fuera 3 · de 5 filas»— y
cada fila que no entra lleva **su número de fila de Excel**, porque quien lea
eso va a abrir su archivo para corregirlo.

- **El mapeo se propone solo** y se corrige. Una columna que se pone en «No
  importar» se atenúa pero no desaparece: tiene que poder recuperarse.
- **Sin correo ni WhatsApp se avisa antes de analizar.** Dejar ver un informe de
  cero filas sin explicación es peor.
- **No se puede importar sin aceptar la declaración de origen**, y el texto
  dice que está pendiente de revisión jurídica. Es verdad, y callarlo aparentaría
  un rigor que no tiene (ADR-0013).
- Volver atrás no pierde el archivo.

**Dos fallos reales, y uno llevaba días dentro:**

**Los botones dentro de `EstadoVacio` no se pintaban.** El componente usa una
ranura **con nombre** (`accion`) y todo lo demás se descarta sin error ni aviso.
El botón «Agregar contacto» del estado vacío de CONTACTOS llevaba así desde la
entrega 3.2. `sonda:contactos` no lo vio porque buscaba el botón con `.first()`
y encontraba el del encabezado, que sí existe.

**`WhatsApp` viajaba por la IPC como `whatsApp`, y la interfaz mandaba
`whatsapp`.** En la aplicación de verdad la importación habría fallado al cruzar
la frontera — después de revisar los choques y aceptar la declaración. No lo vio
nadie más: el núcleo simulado de la vista previa es JavaScript y no comprueba
tipos, y las pruebas de Rust usaban el enum, no su JSON. Lo cazó **el validador**
al comparar las dos listas.

Arreglado con un `rename` para que haya **una sola grafía** en todo el sistema, y
fijado con tres pruebas que escriben el JSON esperado **a mano**: derivarlo de la
misma regla que lo produce habría hecho que la prueba se moviera con el fallo.

**La cadena de custodia queda en dos tramos:** las pruebas de Rust atan el enum
al JSON, y el validador ata el JSON a la interfaz. El cebo de quitar el `rename`
mata las tres pruebas de Rust y deja el validador en verde — que es correcto, y
por eso se comprobaron los dos lados por separado.

**Y dos comprobaciones del validador que no medían nada**, vistas al escribirlas:
una cortaba la lista de TypeScript en el `[]` del tipo y devolvía una lista
vacía; la otra buscaba el texto legal hasta un punto y coma que este proyecto no
usa. Las dos pasaban sin mirar nada.

**308 pruebas en Rust, 63 en la interfaz, fase 3 en 52/52**, seis sondas de
navegador. Avance de la v1.2.0: **34 % → 36 %**.


### La pantalla de contactos (entrega 3.2, completa)

Ya se ven. Se dan de alta a mano, se editan, se dan de baja, y la ficha enseña
todas sus formas de contacto.

- **La tabla** enseña el canal principal de cada tipo, con «+2» si hay más. Con
  diez canales por contacto, una tabla que los enseñara todos dejaría de serlo.
- **La ficha** sí los enseña todos, agrupados y con el principal arriba (L-14).
  Ese orden **lo decide la base**; la pantalla no reordena, y el validador lo
  comprueba.
- **El mismo formulario** para alta y edición. Separarlos daría dos sitios donde
  mantener las mismas reglas de canales.
- **La forma normalizada se enseña cuando no coincide con lo escrito.** Quien
  escribe «+52 1 81 1234 5678» ve debajo «Se usará +528112345678». Es el único
  sitio donde esa normalización es visible, y callarla sería cambiar el dato por
  detrás (§65).
- **El aviso de supresión dice explícitamente que no bloquea** (L-13). Sin esa
  frase, un aviso ámbar se lee como «esto no se puede usar».
- **La baja dice qué se conserva**: el contacto desaparece de la lista, y se
  conserva a qué dirección se le escribió y cuándo. El borrado definitivo es
  otra cosa y llega en la 3.4.

**`sonda:contactos`**, nueva: recorre la pantalla en un navegador de verdad,
desde `file://`. Encontró dos cosas que ningún test de unidad podía ver.

**Fallo real, y de los que no se ven mirando.** `AModal` no desmonta lo que
lleva dentro al cerrarse, y el `watch` sobre el contacto inicial no se dispara
cuando pasa de `null` a `null` —dos altas seguidas—. **El segundo formulario
abría con las formas de contacto del primero**, y al guardar chocaba con una
dirección que el usuario nunca había escrito. Lo delató la sonda: el error
nombraba un móvil que en ese formulario no existía. Arreglado forzando el
remontaje en cada apertura.

**Y un fallo en la propia sonda**, del mismo tipo que los del tramo anterior: la
comprobación del «1» del móvil leía el texto de la ficha buscando «+521», pero
ahí se enseña lo escrito —«+52 1 81…», con espacios—, así que el patrón no podía
coincidir **nunca**. Pasaba siempre. Ahora mide la forma normalizada, que es la
que existe precisamente porque se añadió a la ficha.

**Dos cosas más que cazaron las comprobaciones que ya había:**

- El texto `ana@empresa.mx` **no compilaba**: en vue-i18n la arroba abre un
  mensaje enlazado, y la pantalla que lo usara habría reventado. Lo cazó
  `es.spec.ts`, que existe justo para eso.
- El error de dirección repetida **no nombraba la dirección**, aunque el núcleo
  la mandaba. `resolverError` no interpolaba el detalle. Con diez canales en el
  formulario, eso obliga a repasarlos todos — que es lo que ese error existe
  para evitar.

Cinco cebos deliberados sobre la sonda y el validador, los cinco mortales.

**222 pruebas en Rust, 63 en la interfaz, fases 0–3 en verde (4 + 48 + 37 + 44),
cinco sondas de navegador.** Avance de la v1.2.0: **32 % → 34 %**.


### Contactos: la capa de datos y la frontera (entrega 3.2, tramos A y B)

ARLES ya sabe **meter contactos en su base y sacarlos**, y la pantalla ya tiene
a quién llamar. Lo que todavía no hay es la pantalla: al abrir la aplicación,
CONTACTOS sigue diciendo que aún no está. Se dice aquí porque se prometió antes
de empezar.

- **Repositorio `arles-db/src/contactos.rs`**: alta, edición, ficha, lista
  paginada, baja lógica y el aviso de supresión (L-13).
- El **orden de los canales** —correos primero, móviles después, el principal
  arriba— sale de la consulta y no de reordenar en Rust. La lista y la ficha no
  pueden ordenar distinto porque ordenan en el mismo sitio.
- Los canales retirados **se marcan, no se borran**. El índice único es parcial
  (`WHERE deleted_at IS NULL`), así que la dirección vuelve a estar libre y a la
  vez se conserva la respuesta a «¿a qué dirección se le escribió en marzo?».
- **Cinco comandos** en la frontera: listar, ficha, crear, editar y borrar.
  Ninguno recibe el identificador de la empresa desde la webview — se lee de la
  base. Que hoy sólo haya una empresa (D-4) no es una defensa, es una
  coincidencia que dejará de serlo en la v1.3. El validador lo comprueba.
- El **país con el que se completa un móvil sin prefijo** sale de la empresa
  configurada, no del formulario. Si lo eligiera la pantalla, «81 1234 5678»
  acabaría en dos países según quién lo mandara.

**Dos errores nuevos, porque «UNIQUE constraint failed» no le sirve a nadie:**
`DireccionEnUso` lleva dentro **la dirección en conflicto** —con hasta diez
canales en el formulario, sin ella habría que adivinar cuál se repitió—, y
`ContactoNoExiste` no distingue «nunca existió» de «era de otra empresa»,
porque decirlo filtraría la existencia de datos ajenos.

**Nueve cebos deliberados. Cuatro pruebas no medían nada** y sólo se supo al
intentar romperlas:

- Quitar el `CASE` que agrupa por tipo del `ORDER BY` **seguía en verde**: el
  núcleo ya ordena antes de insertar, así que `created_at` sola reproducía el
  resultado. Prueba nueva que escribe los canales intercalados a mano.
- Quitar el recorte de página **seguía en verde**: la prueba tenía un solo
  contacto, y pedir diez mil también devuelve uno. Ahora siembra más filas que
  el tope y exige que vuelvan exactamente mil, con el total real.
- El tope de página pasó de ser un `assert!` en un test a una **comprobación de
  compilación**: los dos valores son constantes, así que no puede depender de
  que alguien ejecute los tests. Lo señaló Clippy y tenía razón.
- Y una comprobación **del propio validador** que no se podía romper: miraba una
  sola línea encima de la estructura, y ahí está `#[serde(rename_all …)]`, no el
  `#[derive(…)]`. Pasaba siempre, dijera lo que dijera el derive.

**La capa de datos sigue sin conocer la IPC.** `arles-db` no depende de `serde`;
la conversión a lo que ve la pantalla vive en `arles-app`. Si `ContactoGuardado`
se serializara directo, renombrar una columna cambiaría el JSON del frontend.
Y un contacto validado **no puede volver a entrar**: ni él ni `CanalValidado`
deserializan, así que la webview no puede mandar canales «normalizados» a su
gusto y saltarse la deduplicación y la supresión, que comparan esa forma. Las
tres cosas las vigila ahora el validador, y las tres se probaron rompiéndolas.

**222 pruebas en Rust, 63 en la interfaz, fases 0–3 en verde (4 + 48 + 37 + 38).**
Avance de la v1.2.0: **31 % → 32 %**.


### Una vista previa en un solo archivo, que se abre con doble clic

- Dirección pidió «sólo preocuparme de descargar y probar». El instalador no
  puede salir de aquí —los bundles son de Windows y de macOS, y esto es Linux—
  y construirlo en GitHub exige etiquetas y veinte minutos. Así que llega antes
  otra cosa: **`ARLES-vista-previa.html`**, medio megabyte, sin nada al lado.
- Es **la interfaz real** —mismos componentes, tokens, temas y textos— sobre el
  **mismo núcleo simulado que usan las sondas**, guardando en la pestaña. Sirve
  para los bloques A a F de la checklist. **No** es la aplicación de escritorio:
  sin ventana nativa, sin base cifrada y sin llavero, y se dice en el propio
  archivo por si alguien lo reenvía.
- **Dos versiones salieron en blanco antes de que funcionara**, las dos sin dar
  un error al generarlas:
  - El enrutador carga cada pantalla con un `import()` dinámico, que produce un
    archivo por pantalla. Desde `file://` el navegador se niega a cargarlos.
    Arreglado forzando un solo bloque con una configuración de Vite aparte.
  - La CSP del producto declara `default-src 'self'`, que **prohíbe el script en
    línea** — y aquí todo va en línea. El navegador se negó a ejecutar nada, que
    es exactamente su trabajo. Se relaja **sólo en esa copia**, y es la razón
    más clara de por qué ese archivo no es el producto.
- **`sonda:vista-previa`** lo mide abriéndolo desde `file://`, como lo abrirá
  quien lo reciba: que pinte, que arranque en la portada de primera vez, que se
  guarde la empresa, que el tema cambie el píxel y que todo sobreviva a
  recargar. Servido por HTTP, los dos fallos de arriba desaparecen.
- Y un fallo de la propia sonda, del mismo tipo que ya me pasó: medía el tema
  con el navegador arrancado en claro, así que «Automático» ya era claro y
  elegir «Claro» no cambiaba nada. Acusaba al producto de un defecto suyo.
- La vista previa **sí lleva Mont** y el instalador no. No es contradicción: el
  instalador se **distribuye**, que es lo que la licencia no cubre; esto es una
  copia interna para quien tiene esa licencia. Se puede generar con
  `--sin-mont`.
- La checklist se actualiza para recorrerse sobre la vista previa: «cerrar y
  volver a abrir» pasa a ser **recargar la página**, y el bloque G gana que la
  vista previa **no cuenta** como la revisión de macOS.

### Entrega 3.2 · arranca por el núcleo, y una versión de prueba descargable

- **`arles_core::contacto`**: valida un contacto con **varios canales** y
  devuelve **todos** los errores, no el primero — un formulario que corrige de
  uno en uno se recorre tantas veces como errores tenga. Cada error dice **qué
  campo** y, si es un canal, **cuál** de ellos.
- **Un contacto sin ningún canal se rechaza.** No es inofensivo guardarlo:
  engorda la lista, cuenta en el total de la campaña y desaparece del envío sin
  explicación. Un contacto **sin nombre**, en cambio, es legítimo: muchas listas
  traen sólo el correo.
- **L-14 implementado en el núcleo.** Exactamente un principal por tipo —al
  importar no viene marcado ninguno, al editar a mano pueden venir dos— y el
  orden de la ficha: correos primero, móviles después, el principal arriba.
  **No el orden de importación**, que cambiaría al reimportar el mismo archivo
  ordenado de otra forma.
- La misma dirección dos veces dentro de un contacto se rechaza **con el índice
  del canal que sobra**. La base la rechazaría igual por su índice único, pero
  ese error no dice cuál de los dos quitar.
- 15 pruebas nuevas. Probadas rompiendo las dos reglas de L-14: sin agrupar por
  tipo y sin reducir a un solo principal, caen las dos que las vigilan.

- **Flujo «Entrega de prueba (sin firmar)»** en Actions: construye ARLES para
  Windows y macOS en los runners de GitHub y publica una **prerelease**. Aquí no
  se puede compilar —los instaladores son `.msi`, `.nsis` y `.dmg`, y esto es
  Linux—, así que lo construye quien sí puede.
- **Se lanza a mano y nunca sola.** Una versión sin firmar que se publica sola
  acaba instalada donde nadie la pidió, y sin firma quien la instale no puede
  comprobar de dónde salió.
- **Va sin Mont**, con `herramientas/marca/sin-mont.py`, que aplica el plan B de
  P-01 sobre la copia del runner. Lo que cambia es la forma de las letras; los
  tamaños, los pesos y el ritmo salen de los tokens y son los definitivos.

### La consulta jurídica, la sección de descargas y tres decisiones de contactos

- **`08-legal/CONSULTA-JURIDICA.md`**: ocho preguntas concretas para mandar tal
  cual a un despacho, cada una con por qué se pregunta y qué haremos con la
  respuesta. Lo que eran indicios está confirmado en fuentes secundarias —la
  LFPDPPP nueva es del **20/03/2025**, el INAI se extinguió, el **reglamento
  sigue pendiente**— y eso reduce la consulta a lo que de verdad hace falta.
- **Decisión D-7 · la revisión jurídica sale del camino crítico.** La entrega
  3.3 se construye entera salvo los textos, que viven como datos marcados como
  borrador. Con dos condiciones escritas, y con lo que **sigue** bloqueado dicho
  sin adornos: **no se envía a nadie de fuera de TELEMETRY** hasta que esté
  contestado si el correo en frío a fuentes públicas es lícito.
- **Sección «Descargar ARLES» en la portada.** No hay versión todavía y se dice
  por qué, con los tres trámites que faltan y quién los tiene. Cuando la haya irá
  a Releases, no a un archivo suelto en el repositorio.
- **L-12 · reconocimiento automático de columnas** al importar: se sube el
  archivo tal cual y ARLES propone qué es cada columna, **por la forma del
  contenido antes que por el nombre**. Siempre enseña lo que entendió, con
  ejemplos reales, antes de importar nada.
- **L-13 · alta y edición manual de contactos**, incluido cambiar el correo o el
  móvil — avisando de que la dirección nueva **no hereda** la baja de la vieja.
- **L-14 · la ficha enseña todos los canales, la tabla el principal**, con cuatro
  reglas de orden comprobables en vez de la intención de que esté ordenado.

### Avance de la versión visible, y una auditoría que encontró seis documentos mintiendo

- **El porcentaje de avance se ve en la portada del repositorio en GitHub.**
  Hoy, **31 %**.
- **Sólo ahí, no dentro de la aplicación.** La primera versión lo puso también
  en la pantalla de Inicio y Dirección lo retiró. Coincide con lo razonable: es
  un dato **del proyecto**, no del producto — a quien use ARLES no le sirve
  saber que está al 31 %, le sirve saber qué puede hacer hoy, que es lo que dice
  la lista de alta.
- **Un solo origen**, `documentacion/07-entrega/avance.json`; la insignia y el
  desglose de fases del README se **generan**. El validador recalcula y falla si
  el README y la fuente divergen — probado cambiando el número a mano. Escrito a
  mano, la insignia se actualiza y el desglose de debajo se queda con los valores
  del mes pasado, contradiciéndola.
- Cada fracción lleva **en qué se apoya**; los pesos se declaran como estimación
  en vez de presentarse como medición. Y el número **baja** cuando Dirección
  amplía el alcance, que es lo correcto: falta más porque hay más que hacer.
- **`CHECKLIST-VISUAL.md`**: lo que una persona tiene que mirar, en siete
  bloques, incluido el de lo que **no** se puede revisar todavía y por qué.

- !! **La auditoría encontró seis documentos que ya no decían la verdad.** No
  son fallos del código: son afirmaciones que dejaron de ser ciertas y nadie
  actualizó. Las seis corregidas, y ninguna borrada — se deja dicho qué decían
  antes y por qué cambió:
  - `FUERA_DE_ALCANCE.md` y `VISION_Y_ALCANCE.md` decían que WhatsApp **nunca**
    entraría. Dirección decidió lo contrario y el esquema ya está construido.
  - `MOTOR_DE_EJECUCION.md` llevaba **dos** versiones de retraso en la clave de
    idempotencia: decía `contact_id` cuando la Fase 1 ya lo había corregido a
    `contact_email`, y las V3 y V4 lo cambiaron otra vez.
  - `ADR-0004` iba una versión por detrás del mismo cambio.
  - `PREGUNTAS_ABIERTAS.md` seguía marcando P-13, P-14 y P-15 como abiertas
    cuando Dirección las contestó.
  - El `ROADMAP` decía «nada de esto está aprobado» sobre WhatsApp.
  - El `README` anunciaba la Fase 3 como «siguiente» y el producto como sólo de
    correo.
- Y un texto **de la propia pantalla**: Inicio decía «campañas de correo». Se
  quedó corto el mismo día en que WhatsApp entró en el alcance.

### Una campaña tiene etapas · migración V4 (L-1, L-6, L-11)

- **`campaign_stage`.** Una campaña tiene una o dos etapas, cada una de un solo
  canal, y cada una con **su** remitente, **su** plantilla, **su** ritmo, **su**
  ventana y **su propio estado**. El correo primero y el WhatsApp después, sobre
  la misma tabla de contactos.
- **El estado por etapa es L-6**: apagar WhatsApp sin apagar el correo. Con un
  solo `campaign.status`, «detener» sólo podía significar detenerlo todo, y el
  día que el número se ponga en rojo habría que elegir entre arriesgar el número
  o parar unos correos que no tienen nada que ver.
- El ritmo baja a la etapa porque WhatsApp empieza en 5 a 10 diarios y el correo
  en decenas. Un límite compartido obliga a frenar el correo o a quemar el número.
- **La condición de la segunda etapa admite dos valores y sólo dos**: `always` y
  `previous_not_failed`. «Sólo a quien no contestó» **no se puede evaluar** sin
  leer el buzón (§69), y una condición que no se evalúa no filtra nada mientras
  aparenta que sí.
- Cuatro invariantes nuevos en el esquema, no en el código: una etapa por canal,
  el remitente corresponde al canal, la firma sólo en correo, y la primera etapa
  ni espera ni depende.
- **`whatsapp_account`**, tabla aparte de `email_account`, con el modo (L-9), la
  calificación de Meta y su historial para la gráfica de ACTIVIDAD.
- **L-11 · Coexistencia**, decidida por Dirección hoy: al añadir el número hay
  que confirmar que está dado de alta con Coexistencia. Se guarda como **fecha
  con autor**, no como casilla. La explicación del producto lleva las **dos**
  razones y en este orden: sin ella el agente no puede responder desde su
  WhatsApp Business, y sólo después, que ARLES no podría contar las respuestas.
  Explicarlo sólo como «ayuda a la estadística» lo haría parecer analítica
  opcional, y es lo que sostiene la operación entera.
- De paso se corrige este documento: decía que un número conectado a la API
  **deja de funcionar en la app de WhatsApp**. Era cierto y ha dejado de serlo
  desde que Meta publicó Coexistencia.

- !! **La primera versión de la V4 borraba la audiencia congelada y el registro
  de envíos, sin dar un solo error.** Reconstruía `campaign` con `DROP TABLE` +
  `RENAME`; con `PRAGMA foreign_keys = ON`, soltar una tabla **padre** ejecuta un
  borrado implícito que cascadea a `campaign_audience` y `message_attempt`. La
  migración terminaba «bien» con las dos en cero filas: la instantánea de a
  quién se le escribió y la prueba de que se le escribió, perdidas. Lo destapó
  la prueba sobre base poblada — sobre una base vacía no habría pasado nada.
  Ahora las columnas se quitan en su sitio con `ALTER TABLE … DROP COLUMN`, y
  sólo se reconstruyen tablas de las que no cuelga nadie. Probado volviendo a
  poner el `DROP`: el test lo detecta. El validador lo vigila en las migraciones
  siguientes.

### Un contacto tiene canales · migración V3 (L-2, L-3, L-4)

- **`contact_channel`.** La dirección deja de ser dos columnas de `contact` y
  pasa a ser una fila por canal. Ése era el bloqueo de verdad para WhatsApp: un
  contacto **no tenía dónde guardar un móvil**.
- **La deduplicación es por canal**: `UNIQUE(company_id, channel,
  value_normalized)`. El mismo correo y el mismo teléfono conviven; dos
  contactos con el mismo móvil, no.
- **`consent_entry`, append-only** (L-3). El consentimiento deja de ser una
  casilla y pasa a ser un registro con base jurídica, prueba y fecha. Retirarlo
  es una entrada nueva: si se pudiera editar la anterior, cualquiera podría
  reescribir a posteriori con qué base se le escribió a alguien.
- **La supresión distingue canales** (L-4). «No me escribas por WhatsApp» ya no
  se guarda igual que «no me escribas nunca». La baja global se guarda como una
  fila por dirección unidas por `request_id`, porque una fila atada a la persona
  desaparecería al ejercerse el derecho de cancelación.
- `message_attempt` y `campaign_audience` llevan canal, y `contact_email` pasa a
  llamarse `contact_address`: una columna llamada «email» que guarda un teléfono
  es una trampa para quien lea la consulta dentro de un año.
- **El «1» mexicano.** `arles_core::PhoneNumber` guarda los móviles en E.164
  **sin** el `1` que WhatsApp arrastra de antes de 2019. Con las dos formas
  conviviendo, la misma persona entra dos veces y la deduplicación, la supresión
  y el «no le escribas dos veces» fallan a la vez.
- **La V3 se prueba sobre una base poblada de verdad**, parando el runner en la
  V2 e insertando contactos como los guardaba la V1. Sobre una base nueva el
  traslado no mueve ni una fila y la migración parece correcta sin haberse
  ejecutado sobre nada.

- !! **Un test tumbó el primer diseño de `consent_entry`.** Llevaba `contact_id`
  con `ON DELETE SET NULL`; `SET NULL` es un `UPDATE`, el disparador de
  append-only lo aborta, y el resultado era que **un contacto con consentimiento
  registrado ya no se podía borrar**. Registrar la prueba de que se le podía
  escribir a alguien impedía ejercer su derecho de cancelación. La columna se
  quitó: la entrada es sobre una dirección, no sobre un registro.

- !! **Corrección de una afirmación mía, no del código.** Escribí —aquí y en
  `LOGISTICA_DE_CAMPANAS.md`— que sin el canal en la clave única, el correo y el
  WhatsApp de la misma campaña chocarían entre sí. **Es falso:** la clave es la
  dirección, y un correo y un móvil son direcciones distintas. Se descubrió
  rompiendo el índice a propósito y ver que la prueba que lo vigilaba seguía
  pasando. Hay dos tests ahora, y el contraste entre ellos es el punto. El canal
  se queda en la clave por una razón más modesta, ya escrita donde toca.

### El tema ya se puede elegir (C-1, C-2)

- **La paleta clara llevaba dos pasos existiendo sin que hubiera forma de
  elegirla.** Estaba medida —72 contratos de contraste, los mismos en los dos
  temas— pero sólo se veía escribiendo `data-tema="claro"` a mano en el
  inspector. Ahora hay un desplegable en Ajustes › Apariencia.
- Tres opciones: **Automático · el de tu sistema**, **Oscuro** y **Claro**.
  «Automático» sigue a `prefers-color-scheme` **también en caliente**: macOS y
  Windows conmutan solos al anochecer, y sin eso ARLES se quedaría siendo la
  única ventana oscura del escritorio.
- **Se guarda la palabra elegida, no el tema resultante.** Guardar el resultado
  fundiría la elección del usuario y la respuesta del sistema en el mismo dato,
  y ya no habría forma de volver a seguir al sistema sin volver a elegir.
- La preferencia va a `ui_preference`, dentro de la base cifrada y del respaldo,
  no a `localStorage`. La lista de valores admitidos está **cerrada en Rust**:
  el desplegable tiene tres opciones, pero la frontera IPC admite cualquier
  cadena, y lo que se guarde acaba escrito tal cual en `data-tema`.
- Sin botón de guardar: el resultado se ve entero en la misma pantalla. Y fuera
  del formulario de empresa, que descarta su aviso de «Configuración guardada»
  en cada `input`.
- **Se mide que cambia el píxel, no que cambia el atributo.** Un atributo bien
  escrito con los tokens mal enlazados da el mismo atributo y la misma pantalla
  oscura. `sonda:tema` compara el fondo real del `<body>`.
- !! **La primera versión de esa sonda no comprobaba la persistencia.**
  Recargaba con `goto()` a `#/ajustes` estando ya en `#/ajustes`, que con
  enrutado por hash no recarga nada: medía el estado que seguía en memoria. Se
  descubrió al romper a propósito la llamada que guarda el tema — la sonda
  siguió diciendo que se recordaba. Corregida con `reload()` y un falso núcleo
  que guarda fuera del documento.
- Las tres comprobaciones probadas rompiéndolas: sin guardar, «no sobrevive a
  reabrir»; sin oyente de `matchMedia`, «el sistema cambió y no lo siguió»; con
  el atributo en `<body>` en vez de `<html>`, «aplicó null».
- De paso, el validador encontró que `error.app.tema_desconocido` se había
  quedado fuera de la lista de claves vigiladas.

### La sección activa se ve en los dos temas, y su barra crece

- **El relleno del activo era invisible en tema claro.** Reutilizaba
  `--arles-surface-raised`, que allí es blanco puro sobre una barra casi blanca:
  **1.06:1**. Las dos señales que marcan el estado se quedaban en una.
- Token propio `--arles-nav-activo`: `#045686` en oscuro, `#A1C0D6` en claro.
  Da **1.80:1** contra la barra —el oscuro da 1.76— y está en la misma familia
  de matiz, 205° frente a 202°: es la traducción del oscuro, no otro color.
- **El contrato que lo vigila no es un mínimo de WCAG.** La barra de acento es
  la que cumple el 1.4.11; éste es el suelo por debajo del cual el relleno deja
  de aportar nada, fijado en 1.5. Sin él, nada habría fallado el día que el
  relleno se volvió invisible.
- **La barra de acento crece desde el centro** al cambiar de sección, en 200 ms,
  mientras la anterior se recoge. El movimiento dice a dónde se fue el estado;
  sin él la barra aparece de golpe en otro sitio. Es un pseudoelemento y no una
  sombra interior porque una sombra no se puede animar por altura.
- **Y se comprueba que anima.** La sonda mide la altura a los 50 ms y falla si
  ya está en su valor final. Con `prefers-reduced-motion` exige lo contrario.
  Probada quitando la transición: la detecta.
- De paso, la propia sonda tenía un fallo: navegaba a Inicio y después pulsaba
  Inicio, así que no había cambio de sección que animar y acusaba al producto de
  un defecto suyo.

### El logotipo de TELEMETRY ya no desaparece al plegar

- Con la barra plegada el logotipo horizontal no entra en 64 px y **se quitaba
  entero**, dejando el pie sin ninguna marca. Ahora queda **el símbolo solo**,
  sin el texto, como pidió Dirección.
- Son **dos piezas distintas**, no una recortada con `overflow`: recortar
  dejaría la «T» partida asomando por el borde.
- **Dónde acaba el símbolo lo encuentra el script**, no una constante: busca el
  hueco vertical más ancho de la pieza —118 px entre símbolo y texto, frente a
  los 22 del espaciado entre letras— y lo busca sobre el original, antes de
  reducir, porque a 560 px ese hueco mediría 24 y se confundiría con el
  espaciado. Si un día no encuentra ninguno suficientemente ancho, **para y lo
  dice** en vez de entregar medio logotipo.

### Proporciones y espacio muerto

- **El problema no era que sobrara aire, era que estaba todo a un lado.** El
  contenido se pegaba al borde izquierdo y en un monitor de 1920 px la mitad
  derecha quedaba vacía. Toda pantalla se limita ahora a
  `--arles-ancho-pagina` y **se centra**; la regla vive en el armazón, porque
  una pantalla que se olvidara de ponerla volvería a pegarse al borde.
- **Ajustes reparte el ancho en vez de estirarse.** Lo fácil habría sido dejar
  crecer el formulario hasta llenar la ventana, pero un campo de 1200 px es
  peor de rellenar, no mejor. El formulario conserva su medida de 62 ch y lo
  que sobra pasa a llevar lo que antes estaba debajo: el aviso de la zona
  horaria y los seis pasos del alta. La segunda columna aparece **a partir de
  1240 px, no en cuanto cabe** — a 988 empujaría el umbral de plegado hasta
  casi el ancho mínimo de la ventana.
- **Los títulos eran pequeños y ahora no.** `h1` sube de 24 a **28** y
  `display` de 32 a **36**. La escala pasa a tener dos tramos a propósito: de
  `h2` para abajo sigue densa —base 14, razón 1.2, que es lo que quiere una
  aplicación llena de tablas— y los dos títulos de arriba van por libre. Con
  una sola razón, agrandar los títulos obligaba a engordar también el cuerpo.
- Una pantalla puede pedir todo el ancho con `--arles-ancho-pagina: none`.
  Está para lo que aún no existe: una tabla de 500 000 filas quiere cada píxel.

#### El logotipo del pie salía con la tinta del tema contrario

- Llevaba las dos piezas teñidas y elegía con una regla de CSS. **La regla se
  descartó al compilar** —un `:global()` dentro de estilos con ámbito— y el pie
  se quedaba con la tinta crema sobre papel claro, casi invisible. No falló
  nada: la regla simplemente no existía en el CSS final, y sólo se vio mirando
  el render en ancho.
- Ahora la interfaz usa **una sola pieza como máscara** y el color lo pone el
  token. No hay regla que descartar ni dos archivos entre los que elegir.

#### Lo que sigue vacío, y no se rellena

- En Inicio queda espacio **vertical** libre, consecuencia directa de no
  maquetar módulos que no pueden decir nada cierto. Rellenarlo con una gráfica
  decorativa o contadores sin contexto sería mentir sobre lo que la aplicación
  sabe hacer hoy.

### La barra lateral

- **Los iconos ya no saltan al plegar, y está medido.** Dirección lo señaló y
  era cierto: **40 px**. El logotipo desaparecía al plegar y arrastraba hacia
  arriba todo lo de abajo. La cabecera pasa a reservar su altura en los dos
  estados, y `sonda:cabecera` falla si la navegación se mueve más de un píxel.
  Probada rompiéndola: devuelve los 40 px originales.
- **Plegada va el isotipo**, la misma «A» con placa que el sistema operativo
  enseña en la barra de tareas. Sus dos colores **no se invierten con el tema**,
  y son los únicos del sistema que no lo hacen: el icono de la barra de tareas
  tampoco cambia cuando cambias el tema de Windows.
- **El botón de plegar vive siempre en el mismo sitio.** Antes cambiaba de
  alineación al plegar y había que buscarlo dos veces.
- **Los iconos de navegación heredaban el cuerpo del texto**, 14 px, que en una
  barra de 240 px se ve de juguete y plegada es lo único que hay. Pasan a 20.
- **«ARLES RELAY I» arriba a la izquierda.** El numeral no es parte del
  logotipo —§21— sino del nombre comercial, así que va en una propiedad aparte
  del componente. Sigue sin aparecer junto al número de versión (ADR-0010).
- **El pie lleva el logotipo de TELEMETRY**, enlazado a telemetrymx.com, con la
  versión en su misma línea de base. Antes la atribución y la versión iban
  apiladas y la versión colgaba sin alinearse con nada.
- **El comando que abre el sitio no recibe la URL.** Un plugin de shell con
  ámbito dejaría a la webview eligiendo el destino y la seguridad dependiendo de
  una expresión regular bien escrita. Aquí el destino es una constante compilada
  y el comando no tiene parámetros: no hay ámbito que validar ni dependencia
  nueva que auditar.

#### Lo que este cambio rompió, y por qué es bueno que lo rompiera

- **La sonda del teclado contaba 8 paradas en vez de 18.** Identificaba cada
  parada por su texto, y dos botones de sólo icono —el de plegar y el nuevo del
  logotipo— tienen el texto vacío: la segunda colisionaba con la primera y el
  recorrido se cortaba creyendo que había dado la vuelta. Ahora la identidad
  incluye el nombre accesible.
- **El umbral de plegado se quedó sin respaldo.** Al volverse panel, Inicio dejó
  de tener ancho de lectura, y la sonda leía precisamente el `max-width`. Cada
  pantalla declara ahora su suelo en `--arles-medida` —el ancho por debajo del
  cual deja de funcionar— y la sonda lee eso. **Se cambió el criterio porque
  cambió lo medido, no para que pasara:** con el criterio nuevo la sonda falló,
  el suelo resultó ser 988 px y el umbral se subió al medido.

### Inicio, modular

- **Inicio deja de ser una lista de configuración.** Dirección pidió un panel
  con acciones a mano, y que la primera vez guiara a configurar la empresa. Son
  dos necesidades opuestas, así que son **dos composiciones y no una con un
  `v-if` en medio**: quien abre ARLES por primera vez no tiene nada que mirar en
  un panel y necesita una sola cosa que hacer, sin competencia.
- **La regla que impide que un panel modular parezca roto:** un módulo sólo se
  dibuja cuando puede decir algo cierto. Los módulos de campañas, canales y
  actividad que pide la logística **no se maquetan vacíos** — enseñar cajas en
  espera anuncia capacidades que no existen. En su lugar hay un módulo que dice
  exactamente eso. Lo vigila una prueba.
- **La lista de los seis pasos se mudó a Ajustes**, que es donde se configura.
  Lo que no se mudó es su razón de ser: enseña los seis desde el primer día
  porque, con sólo los construidos, alguien la vería completa al terminar el
  primero y concluiría que ya puede enviar. La prueba que lo vigilaba **se
  repartió entre las dos pantallas en vez de borrarse**.
- **`AModulo`**, primitiva nueva. El módulo principal se distingue por el borde
  de acento y no por un fondo distinto: cambiar el fondo obligaría a volver a
  medir el contraste de todo lo que lleve dentro.
- **`ABoton` acepta una ruta y entonces se dibuja como enlace.** Un `<button>`
  con un `router.push` dentro pierde el menú contextual, el foco anunciado como
  enlace y la posibilidad de saber a dónde lleva antes de pulsarlo. El aspecto
  es el mismo; la semántica, no.
- **Rejilla áurea** (`--arles-aureo-fr`): el módulo principal y el secundario
  reparten el ancho en 1.618 : 1.

#### Dos defectos encontrados al mirar el render

- **`calc()` no acepta `fr`.** La rejilla se escribió como
  `calc(var(--arles-aureo) * 1fr)`, que es inválido: CSS descarta la declaración
  entera **sin decir nada** y la rejilla se cae a una columna, con aspecto de
  decisión de diseño. Por eso ahora hay un token con la unidad puesta.
- **Las sombras eran del tema oscuro.** Un 45 % de negro sobre papel no eleva,
  ensucia. En claro se aligeran, y el fondo del modal deja de ser casi opaco.

### Tema claro, medido en los mismos contratos que el oscuro

- **Paleta clara completa.** Cada token lleva ahora sus dos valores en
  `tokens.json`, y **los mismos contratos de contraste se miden en los dos
  temas**: 64 comprobaciones, 32 por tema. Un contrato que sólo se verificara en
  el oscuro no serviría de nada, porque el claro es precisamente donde el oro se
  vuelve ilegible y donde los semánticos se invierten.
- **El claro obligó a separar tres tokens que hacían dos trabajos.** El acento
  se usaba a la vez como texto, trazo, anillo de foco y relleno de botón; sobre
  fondo profundo las cuatro cosas funcionan, sobre papel el oro da 1.5:1. Ahora
  `--arles-accent` es sólo relleno y `--arles-accent-ink` es tinta y trazo.
  Siete de los diez usos del acento en los componentes eran tinta: se migraron.
- **`--arles-text-on-accent` era un alias de `--arles-bg-deep`** con el
  argumento de que «existe para que nadie ponga texto claro sobre relleno
  claro». En el tema claro `--arles-bg-deep` **es** claro, así que el alias se
  volvía la trampa que venía a evitar. Pasa a ser tres tokens explícitos, uno
  por relleno, porque los rellenos semánticos se invierten entre temas: en
  oscuro el peligro es un salmón claro que pide tinta oscura, en claro un rojo
  oscuro que pide tinta blanca.
- **En claro, el botón de acento lleva borde.** El oro sobre la página da
  1.32:1: se lee, pero como forma no existe, y el 1.4.11 pide 3:1 para lo que
  identifica un control.

#### El defecto que apareció al medir

- **El tema oscuro tenía los contornos de control casi invisibles dentro de una
  tarjeta**, y llevaba así desde la Fase 2. Al añadir el contrato «el borde de
  un control tiene que verse también dentro de una tarjeta», el oscuro falló:
  `--arles-border-strong` sobre `--arles-surface-raised` daba **1.76:1**. Sube a
  `#93D4EE`, que cumple 3:1 contra las cuatro superficies oscuras y no sólo
  contra la página.
- Es el argumento de por qué el segundo tema valía la pena: **medir la misma
  regla dos veces encuentra lo que medirla una vez esconde.**

#### Comprobado

- El bait-test del verificador: con un gris secundario demasiado claro, falla en
  **cuatro contratos y sólo en el tema claro**.
- `node app/pruebas/sondas/temas.mjs` renderiza Inicio, Ajustes y el catálogo en
  los dos temas, aplicando `data-tema` igual que lo hará la aplicación. Si
  dejara de pintar el claro, sería porque el mecanismo real está roto.
- La lámina de color enseña los dos temas por token y las dos matrices de
  contraste. De paso se corrigió su alto: al pasar de tres reglas aparecía una
  segunda fila y **el pie se salía de la imagen**.

### Logística de los dos canales, y el logotipo de TELEMETRY

- **`documentacion/01-producto/LOGISTICA_DE_CAMPANAS.md`**: el recorrido
  completo del usuario para correo y WhatsApp, en cuatro tiempos —lo que se
  configura una vez, lo que se prepara en cada campaña, lo que corre solo y lo
  que hay que atender—. Es el documento que Dirección pidió antes de rediseñar
  nada, y del que sale el rediseño.
- **Diez decisiones (L-1…L-10).** Las tres que cuestan dinero: una campaña tiene
  **una o dos etapas y cada etapa es de un canal** —lo que no se mezcla es el
  envío, no la campaña—; el contacto pasa a tener **canales** en lugar de ser un
  correo, lo que abre una migración del esquema de la Fase 1; y WhatsApp obliga
  a una sección **CONVERSACIONES** con el reloj de 24 horas, que aparece sólo
  cuando el canal existe.
- L-1 se escribió primero como «una campaña es de un solo canal», pensando en un
  envío mixto simultáneo, y **estaba mal**: lo que Dirección pidió es una
  secuencia sobre una misma tabla —sale el correo y quien pase los filtros
  recibe después el WhatsApp—, que no rompe nada. Corregida el 16/09/2026.
- **L-7: no se puede comprobar si un número está dado de alta en WhatsApp.** El
  endpoint que servía para eso era de la API On-Premises, apagada en octubre de
  2025, y antes de apagarla Meta ya lo había alterado para que devolviera
  «válido» siempre. Las alternativas de terceros manejan WhatsApp Web por detrás
  y exigen subir la lista del cliente a un extraño.
- **L-8: se rota entre plantillas escritas por el usuario y se mide cuál rinde.**
  No se genera ni muta texto para esquivar filtros: eso es evasión, y el §154 la
  prohíbe. Además no serviría — lo que Meta mide son bloqueos, no repeticiones.
- **L-9** (dos modos: «Seguimiento» y «Prospección Directa») y **L-10**
  (CONVERSACIONES es para trabajar; las estadísticas van en ACTIVIDAD, y queda
  escrito que Meta no dice quién te bloquea ni quién te reporta).
- **Tres riesgos nuevos** en la matriz: R-21 (Meta inhabilita el número, y
  puede llevarse la cuenta entera), R-22 (se pierde a quien respondió porque
  nadie contesta dentro de la ventana) y R-23 (migrar el esquema después de la
  Fase 4 obliga a reescribir el motor).
- **Cuatro preguntas para Dirección** (P-12…P-15), de las que las dos primeras
  bloquean el rediseño.
- **`herramientas/marca/derivar-logos.py`**: el logotipo horizontal de
  TELEMETRY, teñido con los colores de la paleta. Los dos originales llegaron en
  tintas que no están en el sistema —`#EFE7DC` se lee como blanco sucio junto
  al texto de la interfaz, y `#001638` es un azul de matiz 216° frente a los
  200° de ARLES, así que sobre el fondo tira a violeta mientras todo lo demás
  tira a cian—. La pieza es de un solo color sobre transparencia, de modo que
  el color no es parte del dibujo: se tiñe desde `tokens.json` y los originales
  de `/RECURSOS` no se tocan.
- El script **comprueba que los dos originales siguen siendo la misma pieza**
  antes de teñir uno solo; si divergieran, se producirían dos logotipos
  distintos sin que nadie se entere. Probado rompiéndolo: apuntado a otra
  imagen, para con «difieren en el 71,2 %».

### Resumen de WhatsApp para Dirección

- Versión de 4 páginas de la nota de WhatsApp, con la decisión de Dirección ya
  incorporada: prueba con número prescindible y número propio sólo para
  seguimiento.
- **Corrige un dato falso de la versión larga.** Decía «0 apelaciones que suelan
  prosperar»; Meta sí tiene proceso de apelación, lo que no tiene es plazo ni
  interlocutor. Se retira también «1 día puede bastar», que no está medido.
- Añade lo que faltaba: el castigo puede alcanzar a la **cuenta de empresa**, no
  sólo al número señalado —de ahí que los dos números no puedan colgar de la
  misma cuenta de Meta—, y la **ventana de 24 horas** obliga a tener a alguien
  contestando el mismo día.

### Entrega 3.1 — configuración de empresa y lista de alta

- **Pantalla de Ajustes con la configuración de empresa**: nombre, país, zona
  horaria, correo corporativo y sitio web. La validación es del núcleo, no del
  formulario, y devuelve **todos** los campos malos de una vez con el motivo de
  cada uno. Dos validaciones serían dos reglas que mantener iguales, y el día
  que divergen el formulario aprueba lo que el núcleo rechaza.
- **Las zonas horarias son una lista cerrada.** Comprobar sólo la forma dejaría
  entrar `America/Mexico` —que no existe— y el fallo aparecería meses después,
  al calcular una ventana de ejecución. El desplegable se llena con esa misma
  lista, y el validador comprueba que las dos copias no divergen.
- **Pantalla de Inicio con la lista de alta**, que **deriva** el estado de cada
  paso de los datos reales en vez de guardar un booleano. Con el booleano,
  borrar la única cuenta remitente dejaría ese paso en verde para siempre.
  Enseña los seis pasos desde el primer día, con la entrega en que llega cada
  uno: enseñar sólo lo construido haría que alguien viera la lista completa al
  terminar el primero y concluyera que ya puede enviar.
- **Barra lateral fija y plegable** (P-11): a mano y sola, en iconos sin texto,
  y el estado se recuerda. Seis iconos de sección nuevos.
- El **umbral del plegado automático está medido**: 984 px. Ver más abajo.
- Migración **V2** con `ui_preference`. La preferencia va en la base cifrada y
  no en `localStorage`, que vive en el perfil de la WebView: se borra con la
  caché del sistema y no entra en el respaldo `.arles`.
- Configurar la empresa deja constancia en `audit_log`, **en la misma
  transacción** que el cambio. Registrarla aparte dejaría cambios sin rastro
  ante un fallo entre las dos escrituras.

#### Medido, no elegido

- **La primera medición del umbral no encontró nada.** Buscando desbordamiento
  con la barra desplegada, ninguna pantalla desbordó hasta 600 px: las de esta
  entrega son fluidas. Con ese criterio el plegado automático no tenía
  justificación, y redondear a 1000 px habría sido repetir cómo se escribió el
  `min-width: 1120` que causó R-01. Lo que sí se pierde antes es la **medida de
  diseño**: la lista de alta deja de alcanzar sus 78 ch a 984 px con la barra
  desplegada, y plegar devuelve justo los 176 px que faltan.
- `sonda:plegado` falla **por los dos lados** —umbral corto y umbral inflado—,
  probada con 800 y con 1200.

#### Corregido tras auditar el funcionamiento

- **La pantalla de Ajustes reventaba al mostrar el error del correo.** El texto
  llevaba una arroba suelta, y en la gramática de `vue-i18n` `@` abre un enlace
  a otra clave: el mensaje no compila, la función de render falla y **la
  pantalla entera deja de pintarse**, en silencio. Quien escribiera mal su
  correo veía un botón que no hacía nada. Lo vigila una prueba que **compila
  todos los textos**, que encontró de paso otro igual esperando desde la Fase 1.
- Los textos de error se devolvían crudos: el usuario habría leído
  `nombre{'@'}dominio.com`. Ahora se compilan.
- La lista de claves de error que dice vigilar que ninguna se quede sin texto
  **está escrita a mano y no se había actualizado**. Ahora el validador la
  compara con las claves reales de Rust.
- «Configuración guardada» se quedaba puesto mientras se editaba el formulario.
- Al fallar el guardado, los campos malos se marcaban pero **no se anunciaban**:
  ahora el foco va al primero y hay un aviso con `role="alert"`.
- El enlace al catálogo desaparecía con la barra plegada, es decir **justo a
  partir del 200 % de escala**, que es la condición en la que hay que abrirlo.
- Un comentario afirmaba que WCAG 2.2 AA pide 40 px de objetivo. Pide 24 × 24.
- Los desplegables enseñaban **«MX»** y **«America/Mexico_City»**. Ahora se lee
  «México» y «Ciudad de México»; el valor que viaja al núcleo no cambia.
- En la lista de alta, los cinco pasos que **no** se pueden hacer llevaban
  insignia de relleno sólido y pesaban más en la pantalla que el único que sí.
- **Inicio enseñaba una lista que no sabía si era cierta.** Si la configuración
  no se podía leer, pintaba la de reserva —«0 de 6», todo pendiente— como si
  fuera el estado real, y alguien con su empresa ya configurada habría vuelto a
  configurarla. Ahora enseña el error con sus tres partes.

#### Comprobado

- **Las migraciones publicadas no cambian**, comprobado por sha256. Editar una
  ya aplicada deja dos instalaciones con el mismo número de versión y esquemas
  distintos, y el fallo aparece mucho después.
- La empresa se valida **también al leer**: una fila editada por fuera con una
  zona inexistente se nombra en vez de devolverse como buena.
- Validador: fase 3, 19 comprobaciones, 0 omitidas. Las cuatro nuevas, probadas
  rompiéndolas.

---

### Preparación de la Fase 3 — revisión visual

- **Flujo «Revisión visual»** que compila los instaladores de Windows y macOS
  y los publica como artefactos descargables. Quien revise la interfaz **no
  instala nada**: ni Rust, ni el CLI de Tauri, ni dependencias de plataforma.
  Sin firmar — los certificados son de la Fase 9 (R-14) —, así que la guía
  explica cómo saltarse las advertencias de SmartScreen y Gatekeeper.
- **`documentacion/06-calidad/REVISION_VISUAL.md`**: guía paso a paso de las
  cuatro comprobaciones que sólo una persona con una pantalla real puede
  hacer —ventana nativa, WKWebView, escalado de Windows y lector de pantalla—,
  con qué capturar y qué escuchar. Incluye lo que **ya está verificado**, para
  no gastar tiempo dos veces.
- El catálogo se enciende con `VITE_ARLES_CATALOGO=1` en vez de editando
  `router.ts` al vuelo. **La sonda de CSP lo parcheaba y lo restauraba**: si
  algo la interrumpía entre medias, dejaba el interruptor abierto en el árbol
  de trabajo. Una variable de entorno no deja residuo.
- En una ventana nativa no hay barra de direcciones, así que el catálogo
  existía pero no había forma de llegar a él. La navegación muestra el enlace
  **sólo cuando el catálogo está compilado dentro**.

#### Corregido

- **La comprobación del catálogo era una búsqueda de texto** en `router.ts`:
  habría pasado con el interruptor correcto y un segundo `rutas.push` cinco
  líneas más abajo. Ahora **inspecciona el bundle compilado**.
- Y encontró de inmediato un defecto recién introducido: con
  `import.meta.env['VITE_ARLES_CATALOGO']` **entre corchetes**, Vite no
  sustituye la variable en compilación, la rama sobrevive y **el catálogo
  entraba en el bundle de producción** como un trozo cargado bajo demanda. Con
  notación de punto, desaparece. Documentado en `app/src/entorno.d.ts`.

### Preparación de la Fase 3 — decisiones registradas

- **ADR-0013 · Origen de contactos, purificación y envío canario.** Acepta la
  delegación de responsabilidad por EULA más afirmación en dos momentos, con el
  origen concreto y la versión del texto guardadas. Corrige la purificación
  propuesta —MX **y si no hay, A/AAAA**, deduplicada por dominio, con topes y
  caché— y añade el **envío canario**, que es lo que de verdad evita las pausas:
  la comprobación de MX detecta dominios muertos, no buzones muertos.
- **ADR-0014 · Detección de rebotes sin VERP.** Corrige el plan de v1.3 de
  ADR-0009, que **no habría funcionado**: el VERP exige controlar el
  `MAIL FROM`, y la API de Gmail no lo expone mientras el SMTP de Gmail y
  Microsoft 365 lo reescriben. La detección asíncrona pasa a **reenvío a un
  buzón externo leído por IMAP**, correlacionando por `Message-Id` —el que ya
  generamos desde la clave de idempotencia—, sin ningún scope de Google.
- **ADR-0004 corregido**: seguía mostrando el índice único por `contact_id`, que
  la Fase 1 cambió a `contact_email` (hallazgo F1).
- Nueve documentos que describían el VERP como el plan quedan corregidos o
  anotados. `AUDITORIA_DISCOVERY.md` se **anota, no se reescribe**: es el
  entregable con fecha de la Fase 0.
- **R-20**: la Acceptable Use Policy de Google Workspace prohíbe facilitar
  correo masivo no solicitado, y una revocación de la verificación OAuth
  **apaga Gmail para todos los clientes a la vez**. Un EULA no protege de eso.
- **P-09** (marco legal vigente, **bloquea la entrega 3.3**) y **P-10**
  (cobertura real de correo en el DENUE).
- El roadmap parte la Fase 3 en **cinco entregas con puerta propia**.

### Fase 2 — Design System ✅ cerrada

**Validación: 18/18 comprobaciones de la fase, 0 omitidas** — `validar.py --fase 2`
(54/54 en la ejecución completa). 23 tests de componente y **tres sondas de
navegador**. Revisión adversaria: **10 hallazgos, todos corregidos** (§6 del
documento de fase); versión sin tecnicismos para Dirección en
[`FASE-02-PARA-DIRECCION.md`](documentacion/09-fases/FASE-02-PARA-DIRECCION.md).
Resumen en [`FASE-02-DESIGN-SYSTEM.md`](documentacion/09-fases/FASE-02-DESIGN-SYSTEM.md).

- **Once primitivas** en `app/src/design/componentes/`: botón, entrada,
  selector, insignia, aviso, modal, menú, pestañas, tabla virtualizada, icono y
  logotipo. Más los cuatro estados de pantalla del §97.
- **Mont incrustada** — Regular 400, SemiBold 600, Bold 700 y Black 900 en
  `.woff2`, 188 KB. Cada `@font-face` fija su peso explícitamente.
- **Catálogo del design system** en `/#/catalogo`, sólo en desarrollo: cada
  primitiva en sus cuatro estados, para revisarla en Windows y en macOS.
- **Tokens nuevos** con contrato de contraste verificado en CI:
  `--arles-accent-hover`, `--arles-danger-hover` y `--arles-text-disabled`;
  más alturas de control y de fila, anchos de modal y menú, y las dos sombras.
- **13 comprobaciones nuevas** en el validador, todas probadas rompiéndolas a
  propósito, tres de ellas sondas que manejan un navegador de verdad
  (`app/pruebas/sondas/`): virtualización real, teclado y foco, y CSP.

#### Cambiado

- **CI corre las tres sondas de navegador.** Sin Chromium instalado, el
  validador las omitía con su motivo y el job terminaba en verde: las tres
  comprobaciones que encontraron el defecto más grave de la fase no habrían
  protegido nada en el único sitio donde importa, que es el commit de otra
  persona. El job instala el navegador y, tras validar, **falla si queda una
  sola comprobación omitida**.
- La ruta del navegador dejó de estar escrita a mano en cada sonda: la busca
  `app/pruebas/sondas/navegador.mjs`, y el validador le pregunta a ella.

#### Seguridad

- **La CSP del producto deja de admitir `unsafe-inline`.** La Fase 1 lo había
  registrado como riesgo aceptado (F14) con el argumento de que los estilos
  *scoped* de Vue lo necesitan. Sólo era cierto en desarrollo: en el bundle,
  Vite los extrae a un `.css` y los `:style` de Vue se aplican con
  `element.style.setProperty()`, que la CSP no gobierna. Medido sirviendo el
  build bajo la política exacta de `tauri.conf.json` y recorriendo tabla, modal
  y menú: cero violaciones. `style-src` pasa a `'self'`.
- **Los campos ya no ofrecen autocompletado ni corrector.** WebView2 hereda el
  gestor de contraseñas de Edge y WKWebView el de Safari: un formulario SMTP
  con autorrelleno acabaría guardando la credencial del cliente en el almacén
  del navegador, que es lo que prohíbe el §30. Barandilla puesta antes de que
  exista el formulario (Fase 5).
- **Una búsqueda por clave alcanzaba `Object.prototype`.** `acciones[evento.key]`
  encontraba lo heredado: una tecla llamada `constructor` devolvía una función
  invocable, se llamaba a `preventDefault()` y la tecla quedaba tragada. No era
  explotable —ningún teclado produce ese valor—, pero ya no ocurre: el
  despacho pasa por `Object.hasOwn`.

#### Corregido

- **La tabla no virtualizaba: renderizaba las 5 001 filas.** `.tabla` no tenía
  altura, así que nada desbordaba y el virtualizador concluía que cabían todas.
  Invisible en una captura e invisible en jsdom, donde el contenedor mide 0 px
  y no se renderiza ninguna fila. Habría llegado a la Fase 4 y allí, con 500 000
  contactos (T-7), habría tirado la ventana. De **5 001 nodos a 18**.
- **Las fuentes daban 403 en desarrollo** y caían a la reserva en silencio: el
  alias apunta fuera de la raíz de Vite. Sólo fallaba donde se trabaja.
- El panel de pestañas era una parada de tabulación **sin anillo de foco**
  (§100), y cambiar la pestaña desde fuera **robaba el foco**.

- **El botón deshabilitado no se leía.** `opacity: 0.45` daba **1.89:1** en el
  primario y **3.56:1** en un secundario ocupado. WCAG exime a los controles
  deshabilitados, así que ninguna herramienta lo marcaba. Se pinta el estado en
  vez de atenuar el elemento: 3.64:1. Y «ocupado» deja de atenuarse — pasa a
  11.8:1, porque está trabajando, no deshabilitado.
- **`TIPOGRAFIA.md` pedía un corte Medium 500 que el kit no tiene.** Medidos los
  grosores reales, salta de Regular (87 por mil) a SemiBold (115). Las cifras
  usan Regular 400 con figuras tabulares.
- **El `usWeightClass` del kit está desplazado** un escalón: `Mont-Regular`
  declara 600. Documentado, y cada `@font-face` fija su peso para no depender
  del metadato.
- La comprobación de `prefers-reduced-motion` era una búsqueda de texto y pasaba
  con la propiedad mal escrita. Ahora exige la regla `@media` completa y que
  apague animación **y** transición.

### Fase 1 — Cimientos ✅ cerrada

**Validación: 32/32 comprobaciones de la fase, 0 omitidas** — `validar.py --fase 1`
(36/36 contando las 4 de la Fase 0 en la ejecución completa).
Revisión a fondo: **13 hallazgos, todos corregidos** (§6 del documento de fase).
Resumen completo en [`FASE-01-CIMIENTOS.md`](documentacion/09-fases/FASE-01-CIMIENTOS.md);
versión sin tecnicismos para Dirección en [`FASE-01-PARA-DIRECCION.md`](documentacion/09-fases/FASE-01-PARA-DIRECCION.md).


- **`arles-core`** — tipos de dominio sin I/O: ids tipados con UUID v7, `Secret<T>`
  con `Debug`/`Display` redactados, normalización de correo y máquina de estados
  de intento. 40 tests.
- **`arles-db`** — SQLite cifrado con SQLCipher, migraciones con `refinery` y el
  esquema inicial completo. Incluye los tests que verifican que el archivo
  en disco está realmente cifrado y que los invariantes del esquema se cumplen.
  29 tests.
- **`arles-app`** — shell de Tauri 2 con capabilities denegadas por defecto, CSP
  estricta, integración con el llavero del sistema e iconos generados desde los
  tokens. 19 tests, incluido el arranque en sus dos ramas.
- **`app/`** — andamiaje de Vue 3 con TypeScript estricto, Pinia, vue-router e
  i18n, consumiendo el CSS generado desde `tokens.json`. 14 tests.
- **`herramientas/design-tokens/`** — fuente única de color: genera el CSS, la
  lámina de la paleta y la verificación WCAG de CI.
- **`herramientas/iconos/`** — iconos de la aplicación desde los mismos tokens,
  con `.ico` e `.icns` empaquetados sin dependencias externas.
- **`herramientas/validar/`** — valida una fase completa: estructura, fronteras
  de arquitectura, compilación, tests y documentación. Emite informe JSON.
- **CI** — contraste, rustfmt, clippy con `-D warnings`, tests en Linux, Windows
  y macOS, `cargo-deny`, auditoría de npm con dos umbrales y validación de fase.

### Decisiones

- **D-5** — Se desarrolla con Mont; la puerta de la licencia pasa de la Fase 2 a
  **antes de la demo**. Cierra P-02: el logotipo es tipográfico por diseño (§21).
- **ADR-0002** — `refinery` 0.9, no 0.8: la 0.8 fija `rusqlite` ≤ 0.26 y ambos
  declaran `links = "sqlite3"`, así que no coexisten con el 0.37 que SQLCipher
  necesita.

### Fase 0 — Discovery y auditoría

#### Añadido
- Cuerpo documental completo de la Fase 0 en `documentacion/`:
  - Auditoría de discovery con el entregable A–U del §163
  - Inventario forense de assets con 7 hallazgos
  - Matriz de riesgos (R-01…R-18) y preguntas abiertas (P-01…P-08)
  - Arquitectura, modelo de datos, motor de ejecución y abstracción de proveedores
  - 12 registros de decisión de arquitectura (ADR-0001…ADR-0012)
    · ampliados a 14 al preparar la Fase 3 (ADR-0013 y ADR-0014)
  - Modelo de amenazas STRIDE, modelo de secretos y cumplimiento LFPDPPP
  - Sistema de color, design system, tipografía, UX y UX writing
  - Estrategia de QA y presupuesto de rendimiento
  - Distribución, firma y roadmap de 10 fases
  - Inventario de licencias de terceros

### Decisiones de Dirección
- **D-1** — SMTP en v1.2.0; verificación OAuth de Google en paralelo desde la Fase 1
- **D-2** — Extraer cian, oro y cremas de la referencia; construir por rampa los azules profundos ausentes
- **D-3** — Verificar o adquirir la App License de Mont antes de la Fase 2 (**bloqueante**)
- **D-4** — v1.2.0 es despliegue interno de TELEMETRY; lanzamiento comercial en v1.3

### Contradicciones del brief resueltas
- §37 frente a §69 (detección de rebotes) → ADR-0009: parcial en v1.2.0, declarada en la interfaz
- T-4 frente a T-6 (beta comercial sin licenciamiento) → D-4
- §16 frente a la paleta medida (0.73 % de azul profundo) → D-2 y ADR-0005

### Pendiente tras la Fase 0
- 🔴 **P-01** — Licencia de Mont sin verificar. Bloqueaba la Fase 2
- 🔴 **P-02** — No existe ningún activo de marca de ARLES. Bloqueaba la Fase 2

*Ambas resueltas en la Fase 1 por D-5: se desarrolla con Mont y la puerta de la
licencia pasa a antes de la demo; P-02 se cierra porque el logotipo es
tipográfico por diseño (§21).*

---

## Plantilla para futuras versiones

```markdown
## [1.2.0] — AAAA-MM-DD

**Build:** 2026.09.11 · **Commit:** abc1234

### Añadido
### Cambiado
### Obsoleto
### Eliminado
### Corregido
### Seguridad
### Migraciones
### Cambios incompatibles
```

Toda entrada de versión publicada debe incluir, como mínimo (§3): versión,
fecha, cambios, correcciones, actualizaciones de seguridad, migraciones,
cambios incompatibles, build y commit.
