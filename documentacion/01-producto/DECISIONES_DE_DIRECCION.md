# Decisiones de Dirección

**Proyecto:** ARLES RELAY I · v1.2.0
**Última actualización:** 2026-09-11

> Registro autoritativo. Una decisión aquí se considera firme hasta que Dirección la revoque por escrito en este mismo documento. Las decisiones técnicas derivadas viven en `03-arquitectura/adr/`.

---

## Serie T — Decisiones previas al discovery

Tomadas por Dirección antes de esta auditoría, comunicadas en el brief maestro.

### T-1 · Sin rotación automática de remitentes
**Firme.** Los usuarios pueden registrar varias cuentas, pero al alcanzarse un límite **la cola se pausa** con un aviso en la interfaz: «La cuenta [correo] alcanzó su límite…». No se salta automáticamente a otra cuenta.

*Valoración de la auditoría:* **decisión correcta y hay que defenderla.** La mayoría de herramientas del sector venden la rotación como funcionalidad estrella; es evasión de límites, viola las políticas de Google y es el mecanismo exacto por el que se queman dominios. Cuando un cliente la pida —y la pedirá—, la respuesta es no. Ver ADR-0008.

### T-2 · Tipografía Mont
**Firme con salvedad.** Dirección declara poseer licencia comercial de Mont para uso de texto.

*Valoración de la auditoría:* **insuficiente para lo que ARLES necesita.** Las licencias Desktop y Web de Fontfabric no cubren incrustar el binario de la fuente en una aplicación distribuida; eso exige una App License separada. No hay comprobante en el repositorio. Ver hallazgo A-07, riesgo R-01 y decisión **D-3**, que es bloqueante.

### T-3 · Cifrado de la base de datos en reposo
**Firme.** SQLCipher en v1.2.0 para proteger datos personales bajo la legislación mexicana.

*Valoración:* correcta y no negociable. Implica que la clave maestra vive en el llavero del sistema operativo y que **la aplicación se niega a arrancar si el llavero no está disponible** — sin degradación a texto plano. Ver ADR-0011 y riesgo R-10.

### T-4 · Versionado
**Firme con corrección.** El primer lanzamiento público es v1.2.0.

*Corrección:* T-4 lo describía como «beta comercial» mientras T-6 excluía el enforcement de licencias — sin enforcement no hay forma de limitar instalaciones ni cobrar. Resuelto por **D-4**: v1.2.0 es despliegue interno de TELEMETRY.

### T-5 · El numeral «I»
**Firme con matiz de presentación.** Se retiene «I» como identificador comercial.

*Matiz de la auditoría:* «ARLES RELAY I / v1.2.0» pone dos significantes de generación en la misma línea y hace pensar que «I» es la versión 1. Se retiene el numeral, pero **no se muestra adyacente al número de versión**. Ver ADR-0010.

### T-6 · Diferimientos de alcance
**Firme.** Fuera de v1.2.0: integración con Microsoft, cifrado de respaldos y enforcement de licencias. La arquitectura base debe soportarlos después.

*Nota:* el cifrado de respaldos diferido tiene una consecuencia que hay que declarar en la interfaz — en v1.2.0 el respaldo es el único camino de recuperación **y no está cifrado**.

### T-7 · Escala
**Firme.** Objetivo base 200 000 contactos; la arquitectura debe soportar cómodamente un techo de 500 000.

*Nota:* es un techo **arquitectónico**. El volumen **operativo** real está pendiente (P-06) y cambia decisiones de producto, no de arquitectura.

### T-8 · Cumplimiento
**Firme.** Foco estricto en la normativa mexicana de privacidad para v1.2.0 (LFPDPPP). GDPR y normativa estadounidense diferidos.

### T-9 · Activos
**Firme en la intención, no verificable en el repositorio.** Dirección declara poseer todos los derechos sobre marcas, logotipos (flecha, escudo) y paletas.

*Valoración de la auditoría:* **no hay ningún activo de marca en el repositorio** — ni logotipo, ni flecha, ni escudo (hallazgo A-06, riesgo R-08). Y la referencia cromática es stock generado por IA de un tercero, no un activo propio (hallazgo A-03). Los colores no son protegibles, así que extraer una paleta es limpio; distribuir el archivo o presentarlo como activo de marca, no.

### T-10 · Equipo Rust
**Firme.** El equipo son expertos en Rust. Se confía a Rust la complejidad del backend y el motor de ejecución.

*Valoración:* respalda la elección de Tauri (ADR-0001) y permite poner toda la lógica autoritativa en el núcleo, dejando la webview como pura superficie de presentación.

---

## Serie D — Decisiones tomadas en el discovery

Tomadas por Dirección el 2026-09-11, tras la presentación de los hallazgos de auditoría.

### D-1 · Híbrido: SMTP en v1.2.0, verificación OAuth en paralelo
**Decisión.** v1.2.0 envía por **SMTP**. La verificación OAuth de Google arranca en la Fase 1, en paralelo al desarrollo, para que `GoogleProvider` aterrice en v1.2.x.

**Contexto.** `gmail.send` es un scope **sensible**: evita la auditoría CASA Tier 2 —que sí exigirían `gmail.modify` o `mail.google.com`, con auditor externo y renovación anual— pero **exige verificación OAuth**: política de privacidad pública, dominio verificado, vídeo demostrativo y semanas de revisión. El roadmap del brief no contemplaba esta dependencia y era el camino crítico más largo del proyecto.

**Consecuencias.**
- El release **no depende del calendario de Google**.
- Gmail admite SMTP con contraseña de aplicación, sin verificación alguna: el caso de uso principal está cubierto desde el día uno.
- La abstracción `EmailProvider` se diseña **completa** desde el principio, con `GoogleProvider` como segundo adaptador, no como añadido posterior.
- Requiere resolver P-05 (dominio y aviso de privacidad) pronto.

**Alternativas descartadas.** Un único client ID de TELEMETRY verificado (concentra el riesgo: un cliente abusivo tumba el acceso de todos) · cada organización registra su propio proyecto de Google Cloud (inviable para un cliente no técnico) · diferir Gmail entero a v1.3 (pierde el gancho comercial).

**Ver:** ADR-0003, riesgo R-02.

---

### D-2 · Extraer el cian, el oro y los cremas; construir los azules profundos
**Decisión.** Se anclan la marca y el sistema en los colores que la referencia **sí** tiene, y se **derivan por rampa** los azules profundos que faltan. La imagen de referencia queda como material **interno** de estudio: nunca se distribuye ni se presenta como activo de marca.

**Contexto.** La medición del PNG completo arrojó: 54.35 % cian-azur (H 185–210°), 19.48 % ocre, 13.44 % amarillo, y **0.73 % de azul profundo** (H 210–250°). No es una paleta de *La noche estrellada*. El §16 asigna tres roles a tres azules distintos, pero el 54 % de la imagen cabe en ±12°: extraerlos literalmente los colapsa en una interfaz monocromática cian sin jerarquía. Y sólo el 1 % de los píxeles baja de L 10, así que el fondo profundo que exige el dark-first no está en la imagen.

**Consecuencias.**
- El sistema tiene jerarquía real entre estructura, superficie e interacción.
- `#2CA4D4` queda restringido a trazo, borde y foco: falla WCAG como superficie contra todo texto, blanco incluido (2.85:1).
- El amarillo `#FCCC0C` se usa como **luz sobre oscuro** (11.41:1 sobre `#041C2C`), nunca como relleno de botón con texto blanco (1.56:1).
- Legalmente limpio: los colores no son protegibles.

**Ver:** ADR-0005, hallazgos A-03/A-04/A-05, `05-diseno/COLOR_SYSTEM.md`.

---

### D-3 · Verificar o adquirir la App License de Fontfabric
**Decisión.** Dirección confirma el alcance exacto de la licencia de Mont que posee y, si no cubre app embedding, la adquiere. El comprobante se archiva en `documentacion/08-legal/`.

> ⚠️ **Revisada por D-5 (2026-09-11).** La puerta ya no está en la Fase 2, sino antes de la demo. Lee D-5 para saber qué aplica hoy.

**Contexto.** El kit del repositorio fue generado con Transfonter y contiene `.eot` —formato muerto desde IE11—, perfil característico de un paquete de agregador y no de una entrega comercial de Fontfabric. La tabla `name` identifica la fuente como Mont de Fontfabric (Svetoslav Simov, Mirela Belova). No hay licencia, factura ni comprobante en el repositorio. Un bundle de Tauri lleva la fuente dentro del binario instalado en la máquina del cliente: eso es exactamente el supuesto de la App License, que las licencias Desktop y Web no cubren.

El §21 del propio brief ordena detenerse ante esto.

**Plan B, listo para activarse.** Si al inicio de la Fase 2 no hay comprobante: Mont queda para marketing y material comercial · el logotipo ARLES RELAY se entrega como **SVG con contornos** (uso legítimo de una licencia de escritorio, porque ya no hay fuente que incrustar) · la interfaz usa una geométrica de licencia libre con métricas similares.

**Ver:** hallazgo A-07, riesgo R-01, pregunta P-01, `05-diseno/TIPOGRAFIA.md`.

---

### D-4 · v1.2.0 es despliegue interno de TELEMETRY; comercial en v1.3
**Decisión.** v1.2.0 se instala únicamente en máquinas de TELEMETRY INSIGHT. El lanzamiento comercial es v1.3.

**Contexto.** T-4 describía v1.2.0 como «beta comercial» mientras T-6 excluía el enforcement de licencias. Sin enforcement no hay forma de limitar instalaciones, hacer caducar un piloto ni cobrar — y el software instalable que ya salió no se puede des-distribuir.

**Consecuencias.**
- Difiere tres costes externos: certificado de firma EV, verificación OAuth de Google y canal de soporte.
- Permite **validar el motor de ejecución con datos reales y consecuencias reales** antes de exponerlo a clientes.
- El trámite de firma de código arranca igualmente en la Fase 1: son plazos externos que no se comprimen después (riesgo R-14).

---

### D-5 · Mont se usa durante el desarrollo; la licencia se resuelve antes de la demo
**Decisión.** Se desarrolla con Mont con normalidad. **La pregunta por la licencia se vuelve a plantear cuando llegue el momento de la demo de la aplicación**, y esa es la puerta que sustituye a la de la Fase 2.

**Qué desbloquea.** La Fase 2 arranca sin esperar. El Design System puede congelarse, incluida la escala tipográfica y el logotipo.

**Y también desbloquea P-02.** El §21 define el logotipo como **exclusivamente tipográfico** —«ARLES RELAY» en Mont Black, sin isotipo—, así que con Mont disponible el logotipo se puede producir aquí. La flecha y el escudo que menciona T-9 no son necesarios: el §21 los excluye por diseño.

**Dónde está la línea real, para que nadie la cruce por descuido.**

| Uso | ¿Autorizado hoy? |
|---|---|
| Componer documentos, láminas y maquetas con Mont | **Sí** — uso de documento, cubierto por una licencia de escritorio |
| Compilaciones de desarrollo en máquinas del equipo | **Sí** |
| Logotipo entregado como SVG con contornos | **Sí** — es geometría, no contiene la fuente |
| **Instalar la aplicación con Mont incrustada en cualquier máquina fuera del equipo de desarrollo** | **No, hasta resolver D-3** |
| **Enseñar la aplicación a un cliente o prospecto como demo** | **No, hasta resolver D-3** |

Lo que exige la App License no es el desarrollo: es **incrustar el binario de la fuente en un artefacto que se instala o se enseña como producto**. Mientras el binario no salga del equipo, no hay supuesto de distribución.

**Cómo se vuelve a preguntar.** Al planificar la demo, antes de generar el primer instalador destinado a enseñarse. Registrado como **P-01** con disparador actualizado, y como elemento de la puerta de salida de la Fase 9.

**Por qué es razonable ahora.** El plan B —Mont en marketing, logotipo en SVG con contornos, Figtree en la interfaz— sigue listo, **y es más barato de activar que antes**: toda la tipografía vive en `--arles-font-family` y los colores en `tokens.json`. Cambiar de familia es editar una línea, no rehacer el Design System. Desarrollar con Mont no crea deuda: crea una sustitución de una línea.

**Riesgo residual, dicho con claridad.** Si la licencia no llega y hay que activar el plan B, el trabajo perdido es el ajuste fino tipográfico —interletrado, pesos ópticos, verificación de métricas en ambas plataformas—, no la arquitectura del Design System. Lo estimo en días, no semanas. Registrado en **R-01**, ahora con impacto rebajado de Alto a Medio.

---

### D-6 · WhatsApp como segundo canal, y el contacto deja de ser un correo
**Fecha:** 2026-09-16 · **Responsable:** Dirección · **Documento:** [LOGISTICA_DE_CAMPANAS](LOGISTICA_DE_CAMPANAS.md)

Dirección autorizó **L-2**: el contacto pasa a tener **canales**, cada uno con su
dirección, su permiso y su historial, y la clave única de los envíos gana la
columna `channel`.

**Qué lo hizo necesario.** No fue una preferencia de diseño. Dirección pidió que
una misma campaña, sobre una tabla con columna de correo y columna de celular,
mande el correo y después el WhatsApp. Con la clave única actual
—`UNIQUE(campaign_id, contact_id)`— esos **dos envíos son la misma fila**, y la
base de datos rechaza el segundo. La función pedida no se podía guardar.

**Por qué ahora y no después.** Hoy no hay ninguna campaña guardada que
convertir. Después de la Fase 4 habría que reescribir el motor, que concentra el
70 % del riesgo técnico. Registrado como **R-23**.

**Lo que no cambia:** la importación, la tabla de contactos en pantalla, el motor
y las pantallas ya construidas. Es fontanería, no arquitectura.

**Decisiones asociadas que quedan asentadas en la logística:** L-1 corregida
(una campaña puede tener dos etapas, cada una de un canal), **L-7** (ARLES no
comprueba si un número existe en WhatsApp — Meta apagó esa vía y las
alternativas son no oficiales), **L-8** (se rota entre plantillas del usuario y
se mide cuál rinde; no se genera ni muta texto para esquivar filtros), **L-9**
(dos modos: «Seguimiento» y «Prospección Directa») y **L-10** (CONVERSACIONES es
para trabajar; las estadísticas de los dos canales van en ACTIVIDAD).

### D-7 · La revisión jurídica se aparta del camino crítico, con dos condiciones

**Fecha:** 17 de septiembre de 2026 · **Pedida por:** Dirección · **Cierra:** el
bloqueo de P-09 sobre la entrega 3.3

Dirección preguntó si se puede mover la revisión jurídica sin que dañe lo que
viene después. **Se puede, y conviene**, porque llevaba abierta desde el
principio y está frenando trabajo que no depende de ella.

**Lo que se separa:**

| | Se construye ya | Espera al abogado |
|---|---|---|
| Leer XLSX y CSV, con todas las defensas | ✅ | |
| Reconocer las columnas solo | ✅ | |
| Informe de filas rechazadas | ✅ | |
| Guardar el permiso con su fecha y su prueba | ✅ | |
| **El texto del aviso de privacidad** | | ⏳ |
| **El texto de la afirmación de origen lícito** | | ⏳ |
| **Los plazos de los derechos ARCO** | | ⏳ |
| **Si el correo en frío a fuentes públicas es lícito** | | ⏳ **C-3 de la consulta** |

**Las dos condiciones que lo hacen seguro:**

1. **Los textos legales viven como datos, no dentro de los componentes.** Están
   en el catálogo de textos, con una marca que dice que son borradores. Cambiar
   la redacción cuando llegue la respuesta es editar una cadena, no rehacer una
   pantalla.
2. **La única pieza de esquema que puede moverse queda identificada por
   adelantado:** la lista cerrada de `consent_entry.basis` —las cinco bases
   jurídicas del permiso—. Si el abogado dice que alguna no vale, o que falta
   una, es **una migración de una columna**, no de un modelo. Queda anotado en
   `MODELO_DE_DATOS.md` para que nadie tenga que volver a razonarlo.

**Lo que NO se puede hacer sin la respuesta,** y conviene decirlo sin adornos:

!x **No se envía ni un solo correo en frío a nadie de fuera de TELEMETRY hasta
que C-3 esté contestada.** La v1.2.0 es despliegue interno (D-4), así que esto
no frena la versión. Pero si la respuesta llega tarde y alguien quiere enseñar
el producto enviando de verdad, la respuesta es no.

**Coste honesto del aplazamiento:** si C-3 resulta ser «no», hay que añadir una
puerta que exija prueba de consentimiento antes de activar una campaña. Es
trabajo de la Fase 6, no de la 3, así que aplazar **no aumenta** ese coste — lo
deja donde ya estaba.

---

### D-8 · Google Drive y Sheets se difieren a la v1.3, con el camino ya elegido

**Fecha:** 18 de septiembre de 2026 · **Pedida por:** Dirección · **Abre:** P-16

Dirección preguntó si ARLES puede leer una hoja de Google directamente de Drive,
en vez de obligar a bajarla como CSV o XLSX. **Sí se puede**, y se hará — pero
no en la v1.2.0.

**Por qué no ahora.** Es funcionalidad que no está en el ROADMAP de esta
versión, y cae justo sobre la dependencia que **D-1 decidió evitar a propósito**
para poder entregar sin esperar a Google. Entrarla hoy la pagaría la entrega
3.3, que aún tiene abierta su puerta de salida, y la 3.4, que es obligación
legal y no comodidad.

**Por qué sí después, y cómo.** Dirección confirmó que **TELEMETRY tiene correo
de dominio propio**. Eso cambia la cuenta entera: con Google Workspace, la
pantalla de consentimiento de OAuth se marca como **«Interna»**, y entonces
**la verificación de Google no aplica**. Lo que era el camino crítico más largo
del proyecto se convierte en trabajo puramente técnico, sin terceros y sin
calendario ajeno.

**Las tres condiciones que se fijan ahora,** para que quien lo construya en la
v1.3 no tenga que volver a razonarlas:

| | Decidido | Por qué |
|---|---|---|
| **Alcance de OAuth** | **`drive.file`**, nunca Drive completo | `drive.file` sólo ve los archivos que el usuario elige uno a uno en el selector de Google. Es alcance **no sensible**. Drive completo es alcance **restringido**: exige verificación **y** una evaluación de seguridad por un auditor autorizado por Google. Dos clics menos no valen una auditoría externa |
| **Cómo se elige el archivo** | **Google Picker**, y la CSP se abre **sólo** a los dominios del Picker | El Picker es JavaScript servido por Google, y `tauri.conf.json` declara `default-src 'self'` a propósito: es lo que impide que un XLSX o una plantilla hostil llame a ningún servidor. La excepción va acotada por dominio y documentada, no abierta en general |
| **Dónde vive la credencial** | **El llavero del sistema**, como todo lo demás | El `refresh_token` de OAuth es una credencial de largo plazo. ADR-0011 no admite excepciones: ni JSON, ni SQLite en claro, ni bitácora |

**Lo que se hace mientras tanto:** nada. Google Sheets ya exporta con *Archivo →
Descargar → CSV*, y ARLES importa ese archivo hoy, con todas sus defensas. Son
dos clics más para el usuario, cero credenciales nuevas y cero dependencia de
Google.

**Coste honesto del aplazamiento:** ninguno técnico. La maquinaria de
importación —mapeo de columnas, análisis de choques, escritura en lote, registro
de origen— es la misma venga el archivo del disco o de Drive. Lo que se añade en
la v1.3 es **cómo llega el archivo**, no qué se hace con él. Aplazar no encarece
nada; adelantarlo sí hubiera encarecido la 3.3 y la 3.4.

---

### D-9 · Un archivo con gente que ya pidió la baja se enseña antes de importar

**Fecha:** 21 de septiembre de 2026 · **Pedida por:** Dirección · **Gobierna:**
la entrega 3.4 y el informe de importación de la 3.3

Es la hermana de la decisión del 18/09 sobre los choques, y se resolvió igual:
**enseñar antes de importar**.

**El problema.** Un archivo de diez mil filas donde trescientas personas ya
habían pedido la baja produce una lista que dice diez mil y a la que sólo se
puede escribir a nueve mil setecientas. ARLES no les escribiría —la lista de
supresión las bloquea— pero nadie se entera de que están ahí. La diferencia
aparece meses después, cuando una campaña «llega» a menos gente de la esperada
y no hay forma de saber por qué.

**Lo decidido:**

| | Qué pasa |
|---|---|
| **Entran igual** | El nombre, la empresa y el resto valen, y esa persona puede volver a darse de alta algún día. Tirar sus datos sería perder información por una razón que no lo pide |
| **El informe las nombra** | Con su fila de Excel, su dirección y el motivo por el que dejó de recibir |
| **Y la cifra distingue** | «Entran 4 · se les puede escribir a 3». No es lo mismo cuántos contactos se crean que a cuántos se les puede escribir |

**La distinción fina, que parece un detalle y no lo es:** alguien que dijo «no
me escribas por correo» **pero tiene WhatsApp sigue siendo alcanzable**. Sólo se
descuenta a quien se queda sin ningún canal libre. Contar las dos cosas igual
daría una cifra tan falsa como no decir nada.

**Y lo que Dirección pidió además:** una lista de quién canceló, por correo y
por WhatsApp, con su motivo y su fecha. Es la pantalla de supresión de la
entrega 3.4, y sale de la misma tabla que alimenta este aviso.

---

### D-10 · El horario de envío es de la campaña, no de la empresa

**Fecha:** 22 de septiembre de 2026 · **Pedida por:** Dirección · **Gobierna:**
la Fase 4 · la ventana de ejecución y el simulador de duración

Dirección lo planteó al revisar la planeación de la Fase 4, con una frase que
reordena el problema: **«no importa a qué hora lo envíe el remitente, sino a qué
hora lo recibe el destinatario»**.

Es la forma correcta de plantearlo. Una ventana de envío no existe para la
comodidad de quien manda: existe para que el correo caiga en la bandeja de
alguien a una hora en que esa persona lo lea.

**El caso que Dirección describió.** Clientes de negocio en México, campaña de
9 a 17. Si mañana ARLES se lleva a Madrid, la campaña mexicana **no** debe pasar
a horario español: sus clientes siguen en México. Y si algún día hay clientes en
Europa, esa campaña sí debe correr en horario europeo.

**Lo decidido:**

| | Qué pasa |
|---|---|
| **La ventana pertenece a la campaña** | No a la empresa. Una empresa puede tener varias y cada campaña elige la suya |
| **Cada ventana lleva su zona horaria escrita** | `America/Mexico_City`, `Europe/Madrid`. Un nombre, no un desfase: los desfases cambian con el horario de verano y los nombres no |
| **Nunca se usa la zona del sistema operativo** | Es lo que impide que llevarse el portátil a otro huso mueva los envíos sin que nadie lo haya pedido |
| **Por defecto, la de la empresa** | Es lo que acierta en el caso normal. Se cambia cuando la campaña apunta a otro sitio |

**Lo que esto NO es.** No es la hora del destinatario, uno a uno. Eso se difiere
a la v1.3 y está razonado abajo.

**Un dato práctico que salió al discutirlo:** «la zona horaria de la Unión
Europea» no existe. La UE abarca tres —WET en Portugal e Irlanda, CET en la
mayor parte, EET en Finlandia, Grecia y los bálticos—, y entre Lisboa y Helsinki
hay dos horas. Una campaña «para Europa» tendrá que elegir una o partirse en
varias. No es un obstáculo; es algo que conviene saber al planear, no al
ejecutar.

**Lo que esta decisión corrigió de paso.** `MOTOR_DE_EJECUCION.md` §4 decía «en
la zona horaria de la empresa», en singular, como si sólo pudiera haber una. El
esquema **ya soportaba** una por ventana desde la V1 —`execution_window.timezone`
existe, y `campaign.execution_window_id` la elige—, así que lo que estaba mal era
el texto, no el diseño. Encontrado al contrastar la pregunta de Dirección con la
base de datos.

---

### D-11 · La hora de cada destinatario se difiere a la v1.3

**Fecha:** 22 de septiembre de 2026 · **Pedida por:** Dirección · **Gobierna:**
lo que la Fase 4 **no** hace

Existe una versión más ambiciosa de D-10: que cada correo salga a las 9 de la
mañana **de cada destinatario**, cada uno en su huso. Es lo que hacen las
plataformas grandes, y en abstracto es la respuesta correcta.

Se difiere a la v1.3, y no por falta de tiempo:

**1 · ARLES no sabe dónde vive la gente.** Lo que se importa es una dirección de
correo, y una dirección no dice nada de ubicación. Un teléfono con `+52` dice el
país, no la hora: **México tiene cuatro zonas horarias**; Estados Unidos, seis. Y
un dominio `.mx` no implica que la persona esté en México.

**2 · Adivinar mal es peor que no adivinar.** Si ARLES dedujera «está en México»
de un `+52` y esa persona viviera en Madrid, enviaría a las 3 de la madrugada
—y el usuario creería que ARLES se ocupó de eso—. Un error silencioso que el
usuario cree resuelto es peor que una regla explícita que el usuario controla.
Es el mismo criterio que gobierna el caso ambiguo del motor (ADR-0004):
**preferimos fallar del lado visible y recuperable**.

**3 · Enreda el motor donde está el riesgo.** Repartir envíos entre husos
distintos interactúa con los topes por hora y por día de formas que multiplican
los casos frontera, justo en la fase que concentra el 70 % del riesgo técnico.

**4 · Al volumen de ARLES aporta poco.** Con 50 correos diarios la campaña dura
meses de todos modos. Afinar la hora de cada correo es una mejora marginal para
quien manda cien mil al día, no para quien manda cincuenta.

**Qué lo desbloquea, en concreto:** que cada contacto traiga **su zona horaria
declarada**, como una columna más del archivo que se importa. Ese día deja de ser
una adivinanza y pasa a ser un dato, y entonces la decisión se puede reabrir
sobre otra base.

**Registrado también en** `01-producto/FUERA_DE_ALCANCE.md` y en el apartado
«Después de v1.2.0» del roadmap.

---

## Decisiones pendientes

Ver **`02-auditoria/PREGUNTAS_ABIERTAS.md`** para el registro completo.

| # | Pregunta | Bloquea |
|---|---|---|
| P-01 | ¿Qué licencia de Mont posee exactamente TELEMETRY? | **La demo** (antes: Fase 2) |
| P-03 | Origen y consentimiento de los contactos | Fase 3 |
| P-09 | Marco de protección de datos vigente | ~~Entrega 3.3~~ → **apartado del camino crítico por D-7.** Frena los textos legales visibles y el primer envío real a terceros, no la construcción |
| P-05 | Dominio y aviso de privacidad para Google | Fase 5 |

**Ya no hay bloqueantes de la Fase 2.**
