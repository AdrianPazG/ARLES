# Logística de campañas · correo y WhatsApp

**Proyecto:** ARLES RELAY I · v1.2.0
**Estado:** propuesta completa · pendiente de aprobación de Dirección
**Gobierna:** el rediseño de pantallas, el esquema de datos y las Fases 1, 3, 4, 5, 6 y 7

---

## 0 · Para qué existe este documento

Dirección pidió, antes de rediseñar nada, **el escrito con todos los pasos y
configuraciones** de los dos canales, y que todo gire en torno a él.

Esto es ese escrito. Describe el recorrido completo de una persona dentro de
ARLES, desde que abre la aplicación por primera vez hasta que atiende a quien
le respondió. No describe pantallas bonitas: describe **el trabajo**. Las
pantallas se diseñan después, y su única obligación es hacer este recorrido
corto y difícil de equivocar.

### Cómo leerlo

El recorrido tiene **cuatro tiempos**, y confundirlos es lo que hace que los
productos de este tipo se sientan pesados:

| | Tiempo | Con qué frecuencia | Dónde vive |
|---|---|---|---|
| **A** | Se configura **una vez** | al empezar, y casi nunca más | AJUSTES · CANALES |
| **B** | Se prepara **cada campaña** | cada vez | CAMPAÑAS |
| **C** | **Corre solo** | días o semanas | ACTIVIDAD · INICIO |
| **D** | Se **atiende** lo que vuelve | a diario, mientras corre | CONVERSACIONES · CONTACTOS |

La aplicación de hoy sólo tiene el tiempo A a medias. Todo lo demás está por
construir, y por eso este documento importa: define contra qué se construye.

!! **Lo que cambia respecto a lo planeado.** El roadmap actual fue escrito para
un producto de un solo canal. WhatsApp no es «otro proveedor más» dentro del
mismo recorrido: introduce el **permiso previo**, la **conversación de vuelta**
y el **reloj de 24 horas**, y esos tres tocan el esquema de datos, el motor y la
navegación. La sección 9 dice exactamente qué fases se ven afectadas.

---

## 1 · El mapa, en una página

```
                         ┌──────────────────────────────┐
  TIEMPO A               │  1 Empresa                   │
  una vez                │  2 Preferencias y tema       │   AJUSTES
                         │  3 Aviso de privacidad       │
                         └──────────────┬───────────────┘
                                        │
                         ┌──────────────┴───────────────┐
                         │  4 Cuenta de correo          │
                         │  5 Salud del dominio         │   CANALES
                         │  6 Número de WhatsApp        │
                         │  7 Ventana de ejecución      │
                         └──────────────┬───────────────┘
                                        │
                         ┌──────────────┴───────────────┐
                         │  8 Contactos                 │   CONTACTOS
                         │  9 Supresiones               │
                         └──────────────┬───────────────┘
                                        │
  ─────────────────────────────────────────────────────────────────
                                        │
  TIEMPO B               ┌──────────────┴───────────────┐
  cada campaña           │  El asistente · 9 pasos      │   CAMPAÑAS
                         │  ...→ PREFLIGHT → PRUEBA     │
                         └──────────────┬───────────────┘
                                        │
                                   ┌────┴────┐
                                   │ ACTIVAR │  ← la única puerta irreversible
                                   └────┬────┘
  ─────────────────────────────────────────────────────────────────
                                        │
  TIEMPO C               ┌──────────────┴───────────────┐
  corre solo             │  Motor: cola, ritmo, ventana │   ACTIVIDAD
                         │  Semáforo · pausa · parada   │   INICIO
                         └──────────────┬───────────────┘
                                        │
  ─────────────────────────────────────────────────────────────────
                                        │
  TIEMPO D               ┌──────────────┴───────────────┐
  se atiende             │  Respuestas · reloj de 24 h  │   CONVERSACIONES
                         │  Bajas · ARCO · métricas     │   CONTACTOS
                         └──────────────────────────────┘
```

---

# TIEMPO A · Lo que se configura una vez

## Paso 1 · La empresa

**Dónde:** AJUSTES → Empresa · **Estado:** construido (entrega 3.1)

Nombre comercial, país, zona horaria, correo corporativo, sitio web.

**La zona horaria no es un dato decorativo.** Es la que decide qué significa
«de 9 a 18»: el motor calcula la ventana de ejecución en la zona de la empresa,
no en la del equipo. Alguien que viaje no cambia el horario de sus envíos.

=> Puerta: sin empresa configurada no hay campaña posible. Es el primer paso de la lista de alta y hoy ya lo impone.

## Paso 2 · Preferencias y tema

**Dónde:** AJUSTES → Preferencias · **Estado:** por construir

Tema (sistema · claro · oscuro), idioma, densidad de tabla, barra lateral.

El tema arranca en **lo que diga el sistema operativo**, y un desplegable
permite fijarlo. Decidido por Dirección.

## Paso 3 · El aviso de privacidad

**Dónde:** AJUSTES → Privacidad · **Estado:** por construir · **bloqueado por P-09**

La URL del aviso de privacidad de la empresa y el texto de la nota de baja.
ARLES los inserta en cada envío.

!x **Esto no es opcional y no es un adorno.** Sin aviso de privacidad accesible
y sin mecanismo de baja, el envío en frío deja de ser defendible bajo la
LFPDPPP. Es la diferencia entre «usamos una fuente de acceso público», que se
sostiene, y «mandamos correo a quien nos dio la gana», que no.

## Paso 4 · La cuenta de correo remitente

**Dónde:** CANALES → Correo · **Estado:** por construir (Fase 5)

| Qué se pide | Por qué |
|---|---|
| Servidor, puerto, usuario | SMTP con TLS obligatorio |
| Contraseña de aplicación | Nunca la contraseña de la cuenta |
| Nombre y correo visibles | Lo que ve quien recibe |
| Límite diario y por hora | El freno que el usuario se pone a sí mismo |

La credencial va al **llavero del sistema operativo**, jamás a la base de datos
ni a un archivo. Si el llavero no está disponible, ARLES **se niega a arrancar**
antes que guardar nada en claro.

=> Puerta: la cuenta no queda guardada hasta que una **prueba de conexión real** tiene éxito. Guardar datos que no funcionan sólo traslada el fallo a la primera campaña, que es el peor momento para descubrirlo.

## Paso 5 · La salud del dominio

**Dónde:** CANALES → Correo → Salud · **Estado:** por construir (Fase 7)

ARLES comprueba **SPF, DKIM y DMARC** del dominio del remitente y lo dice en
castellano, no en jerga de DNS.

Un dominio sin estos tres registros manda al buzón de no deseados por mucho que
el contenido sea impecable. **Es la causa número uno de «no llegó» y no tiene
nada que ver con el programa que envía.**

=> No bloquea el envío: avisa. Bloquearlo dejaría fuera a quien todavía no puede tocar su DNS. Pero el aviso es visible y persistente, no un icono discreto.

## Paso 6 · El número de WhatsApp

**Dónde:** CANALES → WhatsApp · **Estado:** por construir · **el más largo de todos**

Este paso no se resuelve dentro de ARLES. La mayor parte ocurre en Meta y
**tarda semanas**:

| | Qué | Dónde | Cuánto |
|---|---|---|---|
| 6.1 | Cuenta de empresa en Meta, con la empresa verificada | Meta | semanas |
| 6.2 | Número dedicado, que **deja de funcionar en la app de WhatsApp** | Meta | inmediato, e irreversible en la práctica |
| 6.3 | Conectar el número a ARLES con sus credenciales | ARLES | minutos |
| 6.4 | Plantillas aprobadas, una por una | Meta | días por plantilla |
| 6.5 | Aceptar la advertencia de riesgo, con casilla | ARLES | una vez, y queda registrado |

!x **La condición que decide si la mitigación de Dirección funciona.** Se
acordó probar con un número prescindible y usar el número propio sólo para dar
seguimiento. Para que eso proteja algo, **los dos números no pueden colgar de la
misma cuenta de empresa en Meta**: una restricción sobre el número de prueba
puede alcanzar al de seguimiento. Cuenta aparte, datos aparte, método de pago
aparte. ARLES no puede comprobarlo —no ve dentro de Meta—, así que lo pregunta
de forma explícita en el alta y lo deja registrado.

=> Puerta: WhatsApp **no aparece como canal en el asistente** hasta que hay un número conectado, al menos una plantilla aprobada y la advertencia aceptada. Un canal a medias que se ofrece en el desplegable es una campaña fallida en diferido.

### Los dos modos del canal

=> **Decisión L-9 · WhatsApp se ofrece en dos modos separados, no como una función con una casilla.**

| Modo | A quién escribe | Riesgo |
|---|---|---|
| **Seguimiento** | sólo a quien dio permiso o escribió primero | **ninguno** |
| **Prospección Directa** | en frío, con el número que el cliente ponga | el de la sección 3 |

Separarlos importa porque **la mayoría de los clientes sólo necesitan el
primero**, y hoy la única forma de ofrecérselo sería dentro de la misma función
que los expone al segundo.

**Sobre el nombre.** «Prospección Directa» es comercial y es exacto; «directa»
carga la connotación correcta. Lo que hay que evitar es un nombre que
**anestesie** —del tipo «Modo Turbo» o «Impulso»—, y no por escrúpulo: el día
que caiga un número, la pregunta será «¿le advirtieron?», y un nombre que
ocultaba el riesgo es prueba en contra. El nombre puede ser atractivo; lo que no
puede es mentir.

## Paso 7 · La ventana de ejecución

**Dónde:** CANALES → Ventana · **Estado:** por construir (Fase 4)

Días de la semana y franja horaria en que ARLES puede enviar, en la zona de la
empresa.

**Es una sola ventana para los dos canales**, y conviene decir por qué: un
mensaje a las 23:40 molesta igual venga por donde venga. Si más adelante hay
razón para separarlas, se separan; empezar con dos ventanas es complejidad sin
demanda.

---

## Paso 8 · Los contactos

**Dónde:** CONTACTOS · **Estado:** por construir (entregas 3.2 y 3.3)

Importación XLSX/CSV con todas las defensas ya documentadas, listas, etiquetas,
campos propios, tabla a 500 000 filas.

### Lo que cambia por WhatsApp — y es un cambio de fondo

Hoy el modelo de datos trata el **correo como la identidad del contacto**.
`message_attempt` supone una dirección de correo. Eso deja de valer.

=> **Decisión L-2 · Un contacto tiene canales, y cada canal tiene su propio estado.** ✅ **Aprobada por Dirección el 16/09/2026 · implementada el 17/09/2026 en la migración `V3__canales_de_contacto.sql`.**

```
contact           quién es          nombre, empresa, campos propios
  └── contact_channel                canal · dirección o teléfono
                                     · estado de permiso
                                     · de dónde salió
                                     · verificado / rebotado / suprimido
```

La misma persona puede tener correo utilizable y WhatsApp prohibido, o al revés.
Meterlo todo en una fila obliga a inventar reglas en el código para algo que el
esquema puede decir solo.

**Y la clave única de los envíos gana el canal:**

```
antes   UNIQUE (campaign_id, contact_address)
ahora   UNIQUE (campaign_id, channel, contact_address)
```

!! **Corrección, escrita al implementarlo.** Este apartado decía que sin la
tercera columna el correo y el WhatsApp de la misma campaña **serían dos filas
idénticas** y la base rechazaría la segunda. **Es falso**, y conviene dejarlo
dicho en vez de borrarlo: la clave nunca fue `contact_id`, era la **dirección**,
y un correo y un móvil son direcciones distintas — no habrían chocado nunca.

Se descubrió rompiendo el índice a propósito: la prueba que supuestamente
vigilaba esto siguió pasando.

Lo que de verdad impedía el doble canal era más simple y más grave: **un
contacto no tenía dónde guardar un móvil.** La dirección eran dos columnas de
`contact` y sólo cabía una. Eso es lo que `contact_channel` arregla.

El canal en la clave se mantiene, por una razón más modesta: hace que la clave
*diga* lo que significa, y sostiene la garantía si dos canales llegaran a
compartir la misma cadena. La protección contra duplicados no se debilita:
sigue siendo imposible mandar dos correos, y ahora también dos WhatsApp.

**Coste honesto:** es una migración del esquema de la Fase 1, con todo lo que
toca detrás. Hacerlo ahora cuesta días **y no hay ninguna campaña guardada que
convertir**. Hacerlo después de la Fase 4 obliga a reescribir el motor.

**Lo que no cambia:** ni la importación, ni la tabla de contactos en pantalla,
ni el motor, ni las pantallas ya construidas. Es fontanería, no arquitectura.

### El teléfono no se normaliza como el correo

| | Correo | Teléfono |
|---|---|---|
| Forma canónica | minúsculas, sin puntos en Gmail | **E.164**: `+52` y diez dígitos |
| Trampa típica | alias con `+etiqueta` | el `1` de larga distancia, el `044`/`045` que ya no existe, el 55 de Ciudad de México partido en dos columnas |

Un teléfono mal normalizado no rebota como un correo: **le llega a otra
persona**. Es un fallo silencioso, y por eso la importación tiene que rechazar
lo que no pueda normalizar con certeza, en vez de adivinar.

## Paso 9 · Las supresiones

**Dónde:** CONTACTOS → Supresiones · **Estado:** por construir (entrega 3.4)

=> **Decisión L-4 · La supresión gana siempre, y tiene dos alcances.**

- **Por canal:** «no me escribas por WhatsApp» no significa «no me escribas».
- **Global:** «no me contacten» apaga los dos, y manda sobre cualquier lista,
  cualquier importación y cualquier campaña futura.

La prueba de que funciona no es un test: es el caso de la entrega 3.4 —borrar
un contacto, volver a importarlo y comprobar que **sigue sin escribírsele**.

---

# TIEMPO B · Lo que se prepara en cada campaña

**Dónde:** CAMPAÑAS → Nueva · **Estado:** por construir (Fase 6)

Nueve pasos. Los cinco primeros se pueden recorrer en cualquier orden y se
guardan como borrador; los cuatro últimos son una secuencia.

| | Paso | Qué decide | Qué cambia según el canal |
|---|---|---|---|
| **1** | **Nombre y objetivo** | cómo se llama esto dentro de seis meses | nada |
| **2** | **Canal** | correo o WhatsApp | **es la decisión que gobierna el resto** |
| **3** | **Audiencia** | a quién | en WhatsApp, sólo entran quienes tienen permiso registrado |
| **4** | **Mensaje** | qué se dice | correo: plantilla propia, libre. WhatsApp: **sólo plantillas ya aprobadas por Meta**, y no se pueden editar |
| **5** | **Remitente** | desde qué cuenta o número | cuenta de correo / número |
| **6** | **Ritmo y ventana** | cuánto al día | WhatsApp empieza mucho más lento |
| **7** | **Preflight** | qué va a pasar | distinto en los dos, ver abajo |
| **8** | **Envío de prueba** | verlo de verdad | obligatorio en los dos |
| **9** | **Activar** | soltarlo | la única puerta irreversible |

=> **Decisión L-1 · Una campaña tiene una o dos etapas, y cada etapa es de un solo canal.** ✅ **Corregida el 16/09/2026 a petición de Dirección.**

**La versión anterior de esta decisión decía «una campaña es de un solo canal», y
estaba mal.** Se escribió pensando en un envío mixto simultáneo. Lo que Dirección
pidió es otra cosa: una **secuencia** sobre una misma tabla de contactos con
columna de correo y columna de celular —sale el correo, y quien pase los filtros
recibe después el WhatsApp—. Eso es mejor producto y no rompe nada.

Lo que sigue sin mezclarse es el **envío**, no la campaña:

| | |
|---|---|
| El preflight da **un número por etapa** | «1 190 por correo» y «0 por WhatsApp, 847 sin permiso» son dos frases, no una |
| Las métricas **no se suman entre canales** | aceptado en correo y entregado en WhatsApp no son lo mismo |
| El permiso se registra **por canal** | es lo que pide Meta si alguien reclama |
| La parada de emergencia **sabe qué parar** | se puede apagar WhatsApp sin apagar el correo |

Un envío mixto simultáneo, en cambio, rompe las cuatro a la vez. Por eso la
regla se queda en la etapa.

## El filtro que no se puede construir

Dirección planteó, dentro de esa secuencia, un filtro razonable: mandar el
WhatsApp **sólo si el contacto existe en WhatsApp**. No se puede, y conviene que
conste por qué, porque es contraintuitivo.

=> **Decisión L-7 · ARLES no comprueba si un número está dado de alta en WhatsApp. No hay forma legítima de hacerlo.**

- El endpoint `contacts` que servía para eso pertenecía a la **API On-Premises**,
  que **Meta apagó en octubre de 2025**.
- Antes de apagarla, Meta ya había cambiado su comportamiento: **devolvía
  «válido» y un identificador siempre**, existiera o no el número. No se rompió
  — lo inutilizaron a propósito, porque se usaba justo para esto.
- La **Cloud API actual no tiene equivalente**, y no es un olvido: enumerar
  números es lo que Meta quiere impedir.

!x Los servicios de terceros que lo ofrecen funcionan **manejando WhatsApp Web
por detrás** — la automatización no oficial, que es exactamente el camino que
lleva a que apaguen el número. Y obligan a **subir la lista de teléfonos del
cliente a una empresa desconocida**, lo que con la LFPDPPP encima no es un
detalle menor.

**Lo que ARLES filtra en su lugar:** formato normalizado con certeza, no
suprimido, y permiso registrado. Y el mejor filtro de todos es la etapa
anterior: **quien respondió al correo o pulsó el botón de WhatsApp acaba de
demostrar que está en WhatsApp.**

!i Verificado en septiembre de 2026 contra la documentación de Meta y de
integradores. Conviene reconfirmarlo antes de construir la Fase 5: es el tipo de
cosa que Meta cambia sin avisar.

## Las plantillas rotativas, y hasta dónde llegan

Dirección propuso varias plantillas rotando, y texto variable para no parecer
copiar y pegar. La respuesta es **distinta en cada canal**, y conviene no
mezclarlas:

| | Correo | WhatsApp |
|---|---|---|
| ¿Se puede rotar entre plantillas? | **sí** | **sí**, entre las ya aprobadas |
| ¿Se puede variar el texto libremente? | sí | **no.** Meta aprueba el texto exacto; cada variante es otra aprobación, de días |
| ¿Ayuda a que llegue? | **poco.** Pesan mucho más SPF/DKIM/DMARC, la reputación, la tasa de quejas y cómo se subió el volumen | **no.** Lo que Meta mide son bloqueos y reportes, no la repetición del texto |
| ¿Para qué sirve entonces? | **para medir cuál funciona** | igual, y para no cansar al mismo destinatario |

=> **Decisión L-8 · ARLES rota entre plantillas escritas por el usuario y mide cuál rinde. No genera ni muta texto para esquivar filtros.**

La diferencia no es de matiz. «Varias plantillas buenas que rotan y se miden» es
**A/B testing**, y es una virtud. «Un algoritmo que muta el texto para que no lo
detecten» es **evasión**: si funcionara, convertiría a ARLES en una herramienta
de spam, y el riesgo dejaría de ser del cliente para pasar a ser nuestro. El
§154 ya lo dice — informar sin dictar, pero **nunca facilitar la evasión**.

## El paso 7 · Preflight, que es el paso que salva

Antes de poder activar, ARLES enseña **lo que va a pasar, en números**:

```
Campaña «Distribuidores Bajío» · correo · cuenta ventas@…

  1 240  contactos en la audiencia
  −  38  suprimidos          no se les escribe
  −  12  sin correo válido   no se les escribe
  ─────
  1 190  recibirán el mensaje

  a 35 al día, dentro de tu ventana (L-V, 9:00–18:00)
  empieza el 17 de septiembre · termina el 12 de noviembre  ~34 días
```

**El dato que hace pensar a la gente es el último.** «1 190 contactos» no dice
nada; «termina en noviembre» hace que alguien reconsidere el ritmo, la
audiencia o las dos cosas. Es el mismo criterio del panel de INICIO.

En WhatsApp el preflight lleva **dos líneas más, y son las importantes**:

```
    847  contactos en la audiencia
  −  Ø   con permiso registrado            ← 0 de 847
  ─────
      0  pueden recibir el mensaje legítimamente
    847  recibirían sin permiso

  ⚠ Campaña en frío. Riesgo de que el número quede inhabilitado.
```

=> Si hay gente sin permiso, **el botón de activar cambia de sitio y de color** y pide escribir una palabra de confirmación. No se dispara por inercia, y quien lo hace no puede decir que no lo vio.

## El paso 8 · La prueba, obligatoria

Nadie activa sin haberse mandado el mensaje a sí mismo y haberlo visto. En
correo, además, un **envío canario**: unos pocos primero, y pausa automática
para mirar antes de soltar el resto.

## El paso 9 · Activar

La única acción irreversible del producto. A partir de aquí:

- la **audiencia queda congelada** — quién estaba dentro cuando se activó, para
  que dentro de un año «¿a quién le llegó esto?» tenga respuesta;
- se crea **un intento por contacto**, con su clave de idempotencia;
- el motor empieza a trabajar.

---

# TIEMPO C · Lo que corre solo

**Dónde:** ACTIVIDAD e INICIO · **Estado:** por construir (Fase 4, el motor ya está diseñado)

El motor ya está especificado en `MOTOR_DE_EJECUCION.md` y no cambia por
WhatsApp: cola persistente, un intento por contacto garantizado por el esquema,
cubo de tokens, ventana horaria, reintentos con espera creciente, disyuntor por
cuenta.

Lo que sí cambia es **qué mira el semáforo**:

| | Correo | WhatsApp |
|---|---|---|
| Señal de alarma | rechazos 5xx, tasa de rebote duro | **bloqueos y reportes**, y la calificación que publica Meta |
| Qué se arriesga | la reputación del dominio | **el número, y la cuenta de empresa detrás** |
| Velocidad del daño | días | horas |
| Freno | disyuntor por cuenta | disyuntor **más agresivo**, y parada por umbral de bloqueos |

=> **Decisión L-6 · Cada canal tiene su propio semáforo, visible siempre mientras hay campaña activa, y su propia parada.** Un número en amarillo que nadie ve es un número rojo mañana.

### De qué se alimenta el semáforo de WhatsApp

Esto hay que fijarlo con cuidado, porque **Meta no dice quién te bloqueó ni
quién te reportó** (ver el tiempo D). Lo que llega es agregado y con retraso, así
que el freno se construye sobre lo que sí existe:

| Señal | De dónde llega | Qué hace ARLES |
|---|---|---|
| Baja la **calificación de calidad** del número | aviso automático de Meta (`phone_number_quality_update`) | 🟡 avisa y **baja el ritmo solo** |
| **Restricción del nivel de mensajería** | aviso automático de Meta | 🔴 **para el canal** |
| Suben los **no entregados** | conteo propio de ARLES | 🟡 avisa |
| **Nadie responde** en N envíos seguidos | conteo propio de ARLES | 🟡 avisa, porque precede a la caída |

**Parar WhatsApp no para el correo.** Son dos frenos independientes, y encima
de los dos está **la parada de emergencia**, que apaga todo sin preguntar. Es un
botón que existe para el peor día.

---

# TIEMPO D · Lo que vuelve

Y aquí está la diferencia que casi nadie anticipa.

## El correo es una salida. WhatsApp es una conversación

En correo, ARLES manda y la respuesta llega al buzón de siempre. El producto no
tiene que enterarse.

En WhatsApp **la respuesta entra por el mismo sitio por el que salió el
mensaje**, y arranca un reloj:

```
   sale la plantilla
        │
        ▼
   la persona responde  ────────►  se abre la ventana de 24 horas
        │                          se puede escribir libremente,
        │                          sin plantilla
        ▼
   pasan 24 horas  ─────────────►  se cierra
                                   sólo plantilla aprobada otra vez
```

=> **Decisión L-5 · Hace falta una sección CONVERSACIONES, y sólo aparece cuando hay WhatsApp configurado.**

Lista de conversaciones, el reloj de cada una bien visible, y responder. No es
un producto de mensajería: es lo mínimo para no perder a quien levantó la mano.

!x **Sin esto, la prueba que aprobó Dirección no sirve.** El objetivo declarado
es medir cuántos responden y quién se interesa. Si alguien responde y nadie lo
ve hasta pasado mañana, el interesado se perdió, la medición sale falseada y el
silencio además empeora la calificación del número.

=> La pregunta operativa que hay que responder antes de la primera campaña, y no es de programación: **¿quién contesta, y en qué horario?**

### Sobre la navegación

Eso lleva la aplicación de seis secciones a siete, y añadir una sección tiene
coste real: una más en la barra, un icono más, menos sitio para todo lo demás.

**La alternativa que descarto:** meter las conversaciones dentro de CONTACTOS.
Un contacto es un directorio, se consulta; una conversación es trabajo ordenado
por tiempo, con algo que vence. Esconder lo segundo dentro de lo primero
garantiza que nadie lo abra a tiempo.

**Se aparece sola.** Quien use sólo correo sigue viendo seis secciones. La
séptima existe cuando el canal existe.

## Las bajas

Quien pide salir, sale. En correo, por el enlace de baja. En WhatsApp, porque
lo dice o porque bloquea.

=> Una palabra de baja escrita en una conversación —«baja», «no me escriban», «stop»— vale tanto como pulsar un enlace. ARLES la detecta y la propone; **la confirma una persona**, porque un falso positivo aquí borra a un cliente real.

## Las métricas, y lo que no se puede decir

=> **Decisión L-10 · CONVERSACIONES es para trabajar; las estadísticas de los dos canales van juntas en ACTIVIDAD.**

Con doscientas conversaciones abiertas, meterle estadísticas encima a la bandeja
la vuelve inservible. Y las cifras del correo y las de WhatsApp se comparan
mejor una al lado de la otra que en dos sitios distintos.

### Qué se puede medir de verdad, canal por canal

| | Correo | WhatsApp |
|---|---|---|
| Enviado / aceptado | ✅ | ✅ |
| **Entregado de verdad** | ❌ nunca | ✅ **sí** |
| **Leído** | ❌ sólo con píxel, y Apple y Gmail ya lo falsean | ✅ si el destinatario no lo desactivó |
| Respondido | ✅ | ✅ |
| Sin contestar | ✅ | ✅ |
| **Quién bloqueó** | — | ❌ **no** |
| **Quién reportó** | — | ❌ **no** |
| Rebotado / rechazado | ✅ parcial (ADR-0009) | ✅ |
| Dado de baja | ✅ | ✅ |

!x **Las dos casillas rojas son las que más se piden y no existen.** Meta no
identifica a quien te bloquea o te reporta: lo que publica es una
**calificación de calidad del número entero**, agregada y con retraso. Por eso
el freno de L-6 se construye sobre esa calificación y no sobre bloqueos
individuales, que nunca vamos a ver.

!i **Y hay una asimetría que juega a favor.** WhatsApp sí dice si el mensaje
llegó y si lo leyeron; el correo no. Es el mejor termómetro que vamos a tener de
si el mensaje interesa — justo lo que la prueba aprobada por Dirección quiere
medir.

### Lo que la interfaz no puede decir

Es el §65, y no se negocia: una cifra optimista en un panel se convierte en una
promesa a un cliente.

- «**entregado**» en correo, nunca. Se dice «aceptado por el proveedor».
- «**leído**» en correo, nunca en v1.2.0.
- «**interesado**», salvo que una persona lo haya marcado a mano.

---

# 8 · Dónde se separan los dos canales, en una tabla

| | Correo | WhatsApp |
|---|---|---|
| **Permiso previo** | no exigido; sí aviso de privacidad y baja | **obligatorio**, es contrato con Meta |
| **Quién manda las reglas** | la ley mexicana, defendible | **Meta**, sin apelación práctica |
| **Alta del canal** | minutos | **semanas**, y depende de terceros |
| **Contenido** | libre | **plantilla aprobada**, una por una |
| **Coste** | la cuenta que ya se tiene | **por mensaje** |
| **Ritmo inicial** | decenas al día | mucho menos, y subiendo despacio |
| **Vuelta** | al buzón de siempre | **al producto**, con reloj de 24 h |
| **Si sale mal** | reputación del dominio, recuperable | **el número, y puede que la cuenta** |
| **Fuente pública ayuda** | sí, LFPDPPP art. 10 II | **no. Público no es permiso** |

---

# 9 · El puente: el correo consigue el permiso

Esto no es un extra. Es lo que convierte el problema en producto, y es la razón
de que L-1 obligue a hacer dos campañas en vez de una mixta.

```
   CAMPAÑA 1 · correo frío                    defendible, ya funciona
        │
        │  dentro del mensaje:
        │  «Prefiero que me escriban por WhatsApp»
        ▼
   la persona pulsa  ──────────►  se registra el permiso
                                  contacto · canal · cuándo · de dónde
        │
        ▼
   CAMPAÑA 2 · WhatsApp                       sobre gente que levantó la mano
```

=> **Decisión L-3 · El permiso es un registro con prueba, no una casilla.** Quién, qué canal, cuándo, de dónde vino y qué texto aceptó. Es lo primero que pide Meta si alguien reclama, y un booleano no responde a ninguna de esas preguntas.

**Por qué además es mejor negocio:** una lista de gente que levantó la mano
convierte mucho más que una comprada, el embudo se puede medir de punta a punta
—correos, permisos, conversaciones, ventas— y ARLES deja de ser «otro programa
que manda mensajes» para ser el que **construye** la lista buena.

!i Lo que **no** se puede prometer son cifras de conversión. No tenemos datos
propios. Se sabrán con la primera campaña real — que es justamente lo que la
prueba aprobada por Dirección va a medir.

---

# 10 · Qué hay que construir, y qué fases se mueven

| Fase | Estado | Qué le hace WhatsApp |
|---|---|---|
| **0** · Discovery | cerrada | **Se reabre para anotar** la decisión de canal y la prueba del número prescindible. Documental, no bloquea |
| **1** · Cimientos | cerrada | **Cambio real.** `contact_channel`, `consent_entry`, supresión por canal, `message_attempt` deja de suponer un correo. Migración |
| **2** · Design System | cerrada | **Nada.** Los tokens y las primitivas sirven igual. Lo único nuevo es la paleta de tema claro, que ya estaba pedida |
| **3** · Contactos | en curso | Importación de teléfonos en E.164, permiso visible por contacto, supresión con dos alcances |
| **4** · Motor | por hacer | Semáforo por canal, umbral de bloqueos, disyuntor más agresivo. El núcleo del motor **no cambia** |
| **5** · Proveedores | por hacer | Se le suma un `WhatsAppProvider` bajo la misma abstracción |
| **6** · Campañas | por hacer | El canal entra como paso 2 del asistente y gobierna los demás. Plantillas aprobadas, que no se editan |
| **7** · Actividad | por hacer | **CONVERSACIONES**, el reloj de 24 h y las métricas del canal |

=> **La Fase 1 es la que decide el coste.** Es esquema, y el esquema se paga una vez o se paga tres. Conviene hacerlo antes de la Fase 4, no después.

!! **Lo que esto no incluye.** Este documento describe el recorrido completo,
no promete que WhatsApp entre en la v1.2.0. Cabe perfectamente que la v1.2.0
salga sólo con correo y el esquema ya preparado, y que WhatsApp llegue en la
v1.2.x cuando Meta termine la verificación — que es, de todos modos, lo que más
tarda. Es una decisión de Dirección y está en la lista de abajo.

---

# 11 · Lo que hace falta decidir

Las cuatro primeras bloquean el rediseño.

| | Decisión | Quién |
|---|---|---|
| ~~**L-a**~~ | ~~¿Se acepta **L-2** —canales por contacto—?~~ ✅ **Autorizada el 16/09/2026** | — |
| **L-b** | ¿Se acepta **L-5** —la séptima sección, CONVERSACIONES, que aparece sola? | Dirección |
| **L-c** | ¿Quién contesta a los interesados, y en qué horario? La ventana de 24 h no espera | Dirección |
| **L-d** | ¿WhatsApp entra en la v1.2.0, o el esquema se prepara y el canal llega en v1.2.x? | Dirección |
| **L-e** | Confirmar que el número de prueba irá en **cuenta de Meta aparte** de la de TELEMETRY | Dirección |
| **L-f** | Cerrar **P-09**: revisión jurídica del correo frío y de los textos legales | Abogado |
| **L-g** | Abrir la cuenta de Meta y arrancar la verificación — es el camino crítico más largo | Operaciones |

---

## Con esto decidido

El rediseño ya tiene contra qué diseñarse: INICIO modular con los accesos de los
cuatro tiempos, la barra lateral con su séptima sección condicional, el tema
claro, las proporciones y el logotipo de TELEMETRY — que ya está en la paleta
(`documentacion/05-diseno/MARCA_TELEMETRY.md`).

Ese es el siguiente documento, y sale de éste.
