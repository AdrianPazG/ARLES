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

=> **Decisión L-2 · Un contacto tiene canales, y cada canal tiene su propio estado.**

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

**Coste honesto:** es una migración del esquema de la Fase 1, con todo lo que
toca detrás. Hacerlo ahora cuesta días. Hacerlo después de la Fase 6 cuesta
reescribir el motor.

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

=> **Decisión L-1 · Una campaña es de un solo canal.** Una campaña «mixta» suena cómoda y rompe cuatro cosas a la vez: el preflight no puede dar un número, las métricas mezclan peras con manzanas, el registro de permiso se vuelve ambiguo y la parada de emergencia no sabe qué parar. Quien quiera los dos, hace dos campañas — y la sección 8 explica por qué ése es además el orden correcto.

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

=> **Decisión L-6 · Cada canal tiene su propio semáforo, visible siempre mientras hay campaña activa.** Verde, amarillo, rojo, y qué hacer en cada caso. Un número en amarillo que nadie ve es un número rojo mañana.

**La parada de emergencia** detiene todo, de los dos canales, sin preguntar. Es
un botón que existe para el peor día.

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

| Se puede decir | No se puede decir |
|---|---|
| **aceptado por el proveedor** | «entregado» |
| **respondido** (WhatsApp) | «leído» en correo |
| **rebotado** (rechazo 5xx) | tasa de rebote completa — ADR-0009, llega en v1.3 |
| **dado de baja** | «interesado», salvo que una persona lo marque |

Es el §65, y no se negocia: una cifra optimista en un panel se convierte en una
promesa a un cliente.

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
| **L-a** | ¿Se acepta **L-2** —canales por contacto— y con ello tocar el esquema de la Fase 1 ahora? | Dirección + Ingeniería |
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
