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

### D-12 · Los reintentos esperan 5, 15 y 30 minutos, y son tres

**Fecha:** 23 de septiembre de 2026 · **Pedida por:** Dirección · **Gobierna:**
la entrega 4.1 · `arles_core::reintento`

Cuando un envío falla por algo pasajero —se cayó el internet, el proveedor está
saturado, se fue la luz— ARLES vuelve a intentarlo. La pregunta es cuántas veces
y cuánto esperar entre una y otra.

**Lo que se proponía.** La receta de manual: empezar en 30 segundos y doblar
—30 s, 1, 2 y 4 minutos—, cinco intentos.

**Lo que Dirección observó**, y es correcto:

> «Es común que los proveedores de correo o de internet, o un apagón, demoren
> bastantito.»

Aquella tabla daba **siete minutos y medio** de margen total. Un corte de luz o
una caída de proveedor duran bastante más, así que los cinco intentos se habrían
gastado enteros dentro del mismo corte: todos los envíos habrían terminado en
fallo definitivo por un problema que se resolvió solo veinte minutos después.

**Lo decidido:**

| | Espera antes del reintento |
|---|---|
| Reintento 1 | **5 minutos** |
| Reintento 2 | **15 minutos** |
| Reintento 3 | **30 minutos** |
| Después | Se rinde, con su motivo |

**Margen total: cincuenta minutos.** Es la cifra que de verdad decide si la
política sirve, y hay una prueba que la comprueba para que nadie recorte las
esperas sin ver lo que pierde.

**Lo que hubo que recalibrar por el camino.** Cada reintento espera un rato
ligeramente distinto **al azar**, para que mil envíos fallidos no reintenten en
el mismo instante y tumben el servidor que acaba de volver. La receta habitual
para eso es esperar «entre cero y el máximo» — y aplicada a estas cifras haría
que algunos reintentos salieran a los diez segundos, **anulando la decisión**.

Así que el azar se mueve **alrededor** del número, no desde cero: ±20 %. Cinco
minutos pasa a ser entre cuatro y seis. Reparte la carga igual de bien y respeta
lo que se pidió. Hay una prueba cuyo único trabajo es proteger esto.

**Y lo que esta decisión NO cambia:** cuando el proveedor dice explícitamente
cuánto esperar —«no me mandes nada durante diez minutos»—, se respeta
**literalmente**, por encima de esta tabla. Ni se acorta por prisa ni se alarga
por precaución. Acortarlo se interpreta como intentar saltarse el límite, que es
lo que hace el software de spam.

Tampoco cambia que **un fallo definitivo no se reintenta ni una vez**: una
dirección que no existe se da de baja en el acto, y una credencial caducada para
la cuenta entera. Reintentar eso no recupera nada y sí quema la reputación del
dominio.

---

### D-13 · El tope diario de un remitente **se suma** entre campañas

**Fecha:** 22 de octubre de 2026 · **Pedida por:** Dirección · **Gobierna:**
`arles-db/src/trabajo.rs`, `presupuesto.rs` y la pantalla de Remitentes ·
**Sustituye** la defensa nº 1 que D-REM había dejado en pie

Hasta hoy, dos campañas con el mismo remitente **compartían** el cupo: con 100
y 100, salían **100 en total** — medido. El total de un día era el mayor de los
límites, nunca la suma.

**Lo que se propuso**, y era yo quien lo proponía: dejarlo así, porque es lo que
protege al dominio de que cinco campañas sumen mil correos diarios desde un
buzón que nunca ha mandado doscientos.

**Lo que Dirección respondió**, y tiene razón en el porqué:

> «Si en una campaña coloco 100 mensajes por día, y en otra campaña coloco 100
> también con el mismo remitente, tengo que respetar esa solicitud, ya que el
> cliente está colocando que se envíen 100 en cada campaña, por lo tanto sí se
> tienen que sumar… para respetar el tiempo que el cliente calculó para su
> campaña de correos.»

El argumento no es de volumen, es de **promesa cumplida**. El paso 6 del
asistente le dice al cliente cuántos días va a tardar su campaña, y esa cifra
se calcula con el límite que él escribió. Compartir el cupo hacía que el plazo
prometido se **duplicara en silencio** en cuanto alguien activaba una segunda
campaña. ARLES no puede prometer un plazo y entregar la mitad.

**Lo decidido:**

| | Antes | Desde el 22/10/2026 |
|---|---|---|
| Campaña A: 100/día · Campaña B: 100/día | **100** al día en total | **200** al día: 100 de cada una |
| Y una tercera con 50 | 100 | **250** |
| Cada campaña cumple su propio plazo | ❌ no | ✅ sí |

**Y no es sólo sumar, que es lo que hace esta decisión difícil.** Un solo
contador de 200 compartido dejaría que la campaña que llegue primero se gaste
los 200 y la otra no mande nada — el plazo de la segunda seguiría roto, y el
total cuadraría. Para que sean «100 de una y 100 de otra» hace falta que **cada
campaña lleve su propio recuento**, y entonces el total es la suma por
construcción.

**El riesgo que esto abre, declarado:** el agregado de un remitente **ya no
tiene techo**. Cinco campañas de 200 son mil correos al día desde un buzón, y
eso es el patrón que marca una cuenta como spam. Era exactamente lo que el cupo
compartido impedía.

**Lo que ocupa su lugar** (y lo pidió Dirección en la misma frase, que es lo
que hace que la decisión se sostenga):

1. **El semáforo de la D-14**, que enseña la suma real antes de que nadie
   active nada.
2. **El aviso del §48** sigue en cada campaña que pase de 50.
3. **ADR-0008 intacto**: no hay rotación de remitentes. Repartir para esquivar
   el agregado es lo que no se hace.

Es el §154 llevado a su consecuencia: **informar, no dictar.** Quien decide
cuánto aguanta su buzón es quien lo paga, y ARLES tiene que enseñarle la cifra
con la que está decidiendo.

---

### D-14 · El semáforo del cupo real en Remitentes

**Fecha:** 22 de octubre de 2026 · **Pedida por:** Dirección · **Gobierna:** la
pantalla de Remitentes · **Cierra** D-REM

Si el tope se suma (D-13), la cifra que importa deja de estar en la campaña y
pasa a estar en el remitente. Dirección lo pidió así:

> «En la pestaña de remitentes tiene que venir una ficha o algo que notifique
> que ese remitente está enviando 200 correos por día. Y si se crea una tercera
> campaña y se agrega ese remitente y que envíe 50 correos, tiene que
> actualizarse e indicar que está enviando 250.»

**Lo decidido:** cada remitente enseña **la suma de los límites diarios de las
campañas en marcha que lo usan**, con las campañas nombradas, y un semáforo:

| Correos al día del remitente | Semáforo |
|---|---|
| **hasta 60** | verde — dentro de lo que un buzón nuevo aguanta |
| **de 61 a 150** | ámbar — hay que estar mirando la tasa de quejas |
| **más de 150** | rojo — se está arriesgando el dominio |

Los umbrales son de Dirección. Los colores se dejaron a criterio del proyecto, y
salen de los tokens del sistema de diseño (§17): ninguno es un hex escrito en el
componente.

**Y el semáforo lleva texto, no sólo color.** Es la regla que la auditoría D
dejó escrita: *ningún estado se distingue sólo por color*. Un semáforo que sólo
es un punto de color no lo puede leer quien no distingue el rojo del verde, y es
justo la persona a la que hay que avisar.

**Lo que este indicador NO hace: no impide nada.** Es un aviso. Si alguien
quiere mandar 400 al día desde un buzón, ARLES se lo enseña en rojo y lo deja.

---

### D-15 bis · El mismo nombre en dos campañas, con correos distintos · ✅ **medido y resuelto el 23/10/2026**

> «En caso que en dos o más campañas diferentes se repitan los contactos, pero
> tengan correo o números diferentes… para mí lo correcto sería que se
> respetara la lista ingresada en cada campaña, pero al agregarlos en la lista
> de contactos, sería separarlos — o no sé si sea posible esto, cambiando la
> nomenclatura del nombre ingresado.»

**Lo primero es que ARLES ya hacía lo correcto, y se midió antes de opinar.**
Lo que hace único a un contacto es **su dirección**, nunca su nombre
(`idx_channel_unique` sobre `(empresa, canal, valor_normalizado)`, desde la
V3). Así que las dos filas del ejemplo entran como **dos contactos separados**,
cada uno con su correo y su teléfono, y ninguna pisa a la otra:

    importados: 2
    fila: nombre="Telemetry Insight"  canales=["apazg@telemetrymx.com", "+525611644230"]
    fila: nombre="Telemetry Insight"  canales=["veor@telemetrymx.com", "+525534003402"]

Y la lista de Contactos **sí los distingue**: tiene columna de correo y de
WhatsApp, además de nombre y empresa.

---

**Problema.** Lo que sí pasa es lo que Dirección estaba viendo venir: en la
columna «Nombre» hay dos filas que dicen lo mismo, y —más importante— **el
`{{nombre}}` del correo será el mismo para los dos**. Si la columna «Nombre»
del archivo trae la empresa y no a la persona, los dos reciben «Hola Telemetry
Insight».

**Riesgo de cambiar el nombre automáticamente** —«Telemetry Insight (2)»—:
ARLES estaría **inventando un dato que el cliente no escribió**, y ese dato no
se queda en la pantalla: `{{nombre}}` lo lee la plantilla y acaba **dentro del
correo**. La persona recibiría «Hola Telemetry Insight (2)», que es peor que
el problema que resuelve. Y el nombre es dato del cliente: ARLES no lo toca.

**Impacto de no hacer nada.** Ninguno en los datos ni en el envío. Sólo en el
saludo, y sólo cuando el archivo no trae el nombre de la persona.

**Alternativas.**

| | Qué | Por qué no / sí |
|---|---|---|
| **A** · Renombrar solo | «Telemetry Insight (2)» | ❌ Inventa un dato que acaba **en el correo** |
| **B** · Separar en dos contactos | | ✅ **Ya lo hace**, medido |
| **C** · Avisar antes de importar | Decir cuántos nombres van a quedar repetidos | ✅ **Construido** |
| **D** · Un campo propio de referencia | El usuario lo rellena | Más trabajo para el usuario, y no arregla el saludo |

**Recomendación, y lo que se hizo: B + C.** Los contactos siguen separándose
—que es lo correcto— y la pantalla de importar **avisa antes de confirmar**:
cuántos nombres se van a repetir, cuántos serán de cada uno, cuántos ya
estaban en ARLES de un archivo anterior, y qué hacer — poner el nombre de la
persona en el archivo, porque **ése es el dato que leen las plantillas**.

> **Se compara el nombre ENTERO, no el de pila.** La primera versión comparaba
> sólo `nombre`, y «Ana Ruiz» y «Ana Pérez» salían como un nombre repetido. En
> una tabla de trescientas filas hay varias Anas: el aviso habría salido en
> casi todas las importaciones, y **un aviso que sale siempre deja de leerse**.
> Corregido antes de entrar, con su prueba.
>
> Las **tres capas** que comparan —el núcleo, la base y el simulador de la
> demostración— tienen que hacerlo igual, o las dos cifras que se suman no se
> refieren a lo mismo. Lo vigila
> `el_nombre_repetido_se_compara_igual_en_las_tres_capas` en `--fase 3`.

> **Y el hueco del §9 que apareció al lado.** El núcleo calcula desde la Fase 3
> **qué filas entran con una dirección que ya pidió no recibir**, lo serializa,
> y el tipo de TypeScript no lo declaraba: ninguna pantalla lo leía. Quien
> importaba un archivo con diez bajas dentro los metía —correcto, el contacto
> es suyo— sin que nadie dijera que **a esos diez ARLES no les va a escribir**.
> Ahora se enseñan, con su motivo y distinguiendo a quien cerró un canal de
> quien los cerró todos.

### D-15 · Cada campaña carga su propia lista de contactos · ✅ **construida el 23/10/2026**

**Fecha:** 22 de octubre de 2026 · **Pedida por:** Dirección · **Gobierna:** el
asistente de campaña y `arles-import` · **Resuelve** el riesgo R-1 del reporte
de reputación

> **Cómo quedó, y las cuatro decisiones que no son obvias.**
>
> La lista se le pone a un archivo **al importarlo**, en Contactos › Importar,
> y la campaña la elige en su paso 2. No se carga el archivo dentro del
> asistente, y es a propósito: la importación tiene su revisión de choques
> —**la decisión de Dirección del 18/09/2026**, que los enseña antes y nunca
> fusiona sola—, y meter un cargador aparte dentro del asistente la saltaría.
>
> 1. **Los repetidos entran en la lista igual.** Una tabla de trescientas filas
>    con cincuenta que ya estaban en ARLES crea doscientos cincuenta contactos;
>    si la lista se armara con «los que acabo de crear», la campaña saldría a
>    250 de las 300 que se cargaron **sin un solo error**. Y las cincuenta que
>    faltan son las que más importan, porque con ellas ya hay relación. Así que
>    la pertenencia se resuelve **por dirección**.
> 2. **«Toda la empresa» sigue existiendo**, y sigue siendo lo que hacen las
>    campañas ya creadas. Lo que cambia es que ahora **se elige**: el paso 2 lo
>    ofrece con esas palabras y avisa de lo que significa.
> 3. **Una lista vacía no se convierte en toda la empresa.** Es el error que
>    convertiría esto en un peligro: alguien crea «Videovigilancia», no le mete
>    a nadie todavía, y activa. Para con `AudienciaVacia`, y hay una prueba.
> 4. **Una campaña pequeña cuesta lo que cuesta ser pequeña.** La forma obvia
>    —dejar el `FROM contact` de siempre y añadir un `EXISTS`— recorre
>    `contact_channel` entera aunque la lista tenga trescientos: medido con
>    `EXPLAIN QUERY PLAN`, su plan empieza con `SCAN ch`, y eso son los 73.5 s
>    que la auditoría C midió al activar con 451 196 contactos. Entrando por
>    `contact_list_member` **no queda ningún `SCAN`**. Lo vigila
>    `planes_de_consulta.rs`.
>
> **Las tablas ya estaban desde la V1** —`contact_list` y
> `contact_list_member`—, en la lista de «pendientes conocidos» del validador
> desde la Fase 6 con su motivo escrito. Por eso la migración V14 es **una
> columna**, no dos tablas.
>
> Lo vigila `la_lista_de_la_campana_llega_desde_el_archivo_hasta_el_envio` en
> `--fase 6`, escrita antes y vista en rojo en sus cinco tramos.
>
> **Lo que esto cuesta:** un respaldo `.arles` tomado con el esquema V13 ya no
> se puede restaurar — la limitación que la 8.3b dejó declarada. Dirección
> confirmó el 23/10/2026 que no tiene ninguno, así que hoy el coste es cero.

Hoy, al activar una campaña, la audiencia se congela como **todos los contactos
activos de la empresa menos las bajas**. No hay filtro — y eso significa que dos
campañas activas mandan el correo dos veces a todo el padrón (medido el
22/10/2026).

**Lo que Dirección pidió**, y es la solución correcta y no el parche:

> «Al iniciar una campaña podemos colocar una opción para cargar los contactos
> en un archivo xls o csv… Así cuando creen una campaña por ejemplo
> "Videovigilancia" y carguen todos los contactos donde podrían tener interés de
> contratar videovigilancia. Pero si se crea otra campaña de "Desarrollo de
> aplicaciones" puedan cargar su tabla de contactos que imagino sería totalmente
> diferente porque son dos intereses diferentes, y no mezclar contactos donde no
> sean de interés del público y así arriesgar el dominio de que le den spam.»

**Por qué esto es más importante que parecer una comodidad.** La relevancia **es**
una defensa de reputación, y de las fuertes: quien recibe una oferta de
videovigilancia cuando pidió información de software no se da de baja, marca
spam. El duplicado y la irrelevancia son las dos causas principales de una queja,
y esta decisión ataca las dos a la vez.

**Lo decidido:** la campaña tiene su propia audiencia, cargada desde un XLS o un
CSV en el asistente. La tabla pasa por el mismo lector que la importación general
—con sus choques enseñados antes de entrar (D-9) y el reconocimiento de columnas
en cualquier orden—, los contactos entran también en la lista general de
Contactos, y la campaña se queda con **los de esa tabla**.

**Lo que hay que resolver al construirlo**, y se escribe aquí para que no se
decida por descuido:

- **Qué pasa con una campaña ya creada sin lista.** La audiencia de «toda la
  empresa» sigue existiendo, y tiene que seguir siendo elegible a propósito, no
  por omisión.
- **Las bajas y la supresión siguen por encima de todo.** Una lista cargada que
  traiga a alguien que pidió la baja **no se le escribe**, y se dice cuántos se
  quedaron fuera.
- **La ventana de descanso de la D-16** se aplica igual: una lista propia no
  autoriza a escribir dos veces en la misma semana.

---

### D-16 · Un contacto en dos campañas recibe las dos, pero no en la misma semana

**Fecha:** 22 de octubre de 2026 · **Pedida por:** Dirección · **Gobierna:** la
activación de campañas

Dirección cerró la pregunta del contacto repetido así:

> «Ya si en ambas campañas o más se repite un contacto, como son dos plantillas
> diferentes de dos o más campañas diferentes, se le envía el correo de las dos
> campañas, sólo que me gustaría tener cuidado que no se envíen en el mismo día,
> si es posible que pase un lapso de 1 o 2 semanas para no hostigar.»

**Lo decidido:** **no se excluye** a quien ya está en otra campaña — si las
plantillas son distintas, los dos correos tienen sentido. Lo que se impone es
una **ventana de descanso**: nadie recibe dos correos de ARLES antes de que pase
el lapso.

| | |
|---|---|
| **Lapso por defecto** | **14 días** — el extremo alto de lo que pidió Dirección |
| **Mínimo** | **7 días** |
| **De dónde sale el dato** | del registro de envíos, que no se borra nunca (7.4) |
| **Qué hace** | el envío **espera**, no se cancela: sale cuando pasa el lapso |

**Por qué 14 por defecto y no 7.** Quedarse corto es el único error que importa:
un lapso de más retrasa un correo, uno de menos produce la queja que esta
decisión existe para evitar. El mínimo de 7 está para que se pueda bajar a
sabiendas, no por descuido.

**Y esperar, no descartar**, porque descartar silenciosamente volvería a dejar
una campaña que dice «enviando» y no envía a parte de su gente — el defecto que
este proyecto lleva nueve entregas aprendiendo a no repetir. Si un envío está
esperando su lapso, **Actividad tiene que decirlo con su cifra**: es una de las
razones por las que no sale nada.

---

### D-17 · El número de correos por hora lo calcula ARLES

**Fecha:** 22 de octubre de 2026 · **Pedida por:** Dirección · **Gobierna:** el
paso 5 del asistente · **Cierra** el hallazgo H-2 de la auditoría del cupo

> «Que calcule automáticamente ARLES el número de correos por hora, yo sólo le
> asigno el horario con el que quiero enviar la campaña —ya decido si quiero que
> sea de 9:00 a 19:00 hrs u otro horario—. Pero el valor de entrada que se
> respete es el de correos por día.»

**Por qué esto no es una comodidad.** El paso 5 pedía los dos números, y el de
la hora podía frenar al del día. Medido el mismo día, con el por hora que la
pantalla rellenaba sola —diez, el de la cuenta— y una ventana de nueve horas:

| Lo que se pedía | Lo que salía |
|---|---|
| **500 al día** | **90** |
| **100 al día** | **90** |
| 50 al día | 50 |

**Cualquier tope diario por encima de noventa no se cumplía.** La auditoría H lo
arregló calculando el por hora y avisando si alguien lo bajaba; esta decisión va
un paso más allá y **quita la posibilidad de escribirlo**, así que el defecto no
puede volver por ninguna puerta.

**Lo decidido:** en el paso 5 se piden **dos cosas** — el horario y los correos
al día. El reparto por hora se calcula y **se enseña**: «ARLES los repartirá a
lo largo de tu horario: unos 56 por hora durante las 9 horas de envío».

**Y no se pierde nada**, que era la duda legítima: el cálculo **reparte por
definición**. 500 en nueve horas son 56 por hora, que es exactamente lo que el
campo servía para conseguir cuando alguien sabía hacer la división. Lo único que
desaparece es la posibilidad de equivocarse en ella.

> **El aviso de «con ese máximo por hora no saldrán todos» se queda puesto**, y
> es deliberado: no tiene que aparecer nunca. Es la alarma de una puerta que ya
> está cerrada — si algún día vuelve a haber una forma de fijar el por hora, lo
> que no puede pasar es que salgan menos correos de los pedidos y nadie lo diga.
> Su prueba lo siembra a mano, porque una alarma que nunca se ha visto sonar no
> está verificada.

---

### R-25 · La baja automática, pedida el 23/10/2026 · 🔎 **espera una decisión**

> **Se numeró R-09 el 23/10/2026 y estaba ocupado** —«editar y dar de baja
> desde la fila del contacto», cerrado el 21/10—. Dos cosas distintas con el
> mismo identificador en documentos que Dirección lee es exactamente lo que un
> identificador existe para evitar. Corregido el mismo día: la letra R va de
> R-01 a R-24 entre peticiones y riesgos, así que ésta es **R-25**.

> «Al momento que un usuario ya no quiera recibir correos, que ARLES lo dé de
> baja automáticamente. Así como lo hacen otras páginas web o aplicaciones, eso
> estaría genial, y no yo tener que darlos de baja manualmente.»

**Lo que ARLES hace hoy**, y conviene decirlo entero porque la mitad ya está:

| | Estado |
|---|---|
| Quien pidió la baja **no entra** en una campaña nueva | ✅ verificado el 23/10/2026 rompiéndolo |
| Quien se da de baja **con la campaña ya en marcha** no recibe | ✅ verificado: la lista se consulta en el instante de enviar, no al encolar |
| Un rebote duro (`550`) da de baja solo | ✅ desde la Fase 5 |
| Meta avisa de un bloqueo en WhatsApp y da de baja solo | ✅ desde W.4 |
| El correo **dice cómo darse de baja** | ✅ desde el 22/10/2026 (R-7) |
| **Que esa petición llegue a la lista sin que nadie la teclee** | ❌ **esto es R-25** |

Hoy la respuesta con «BAJA» llega al buzón del remitente y **la apunta una
persona en Bajas**. Eso no es un olvido: es que falta la pieza que lee el buzón.

> **Y hasta el 23/10/2026 ni eso se podía hacer entero.** Medir cuánto cuesta
> apuntar una baja a mano —para saber de qué tamaño es el problema que R-25
> resuelve— encontró que la pantalla de Bajas sólo tenía **un buscador**: la
> única forma de apuntar una baja era la ficha de un contacto, y
> `suprimir_contacto` exige un `ContactId`.
>
> Así que **una dirección que no es un contacto no se podía dar de baja de
> ninguna forma**, y es justo la que llega: a alguien le reenviaron el correo,
> o contesta desde otra dirección suya, o su contacto se borró después.
> `Db::suprimir_direccion` existía desde la Fase 4 —probada, con su cabecera
> explicando por qué hace falta— y **ningún comando la llamaba**: el patrón del
> §9 por vigesimotercera vez, y aquí lo que falta no es una función, es que una
> persona que pidió no recibir vuelve a recibir en la siguiente campaña.
>
> Cerrado el 23/10/2026 con el comando, el formulario en `/bajas` y
> `una_baja_no_exige_que_la_direccion_sea_un_contacto` en `--fase 3`, escrita
> antes y vista en rojo nombrando los tres tramos. **No sustituye a R-25**: el
> buzón sigue vaciándolo una persona. Lo que cambia es que ahora puede.

---

**Problema.** Darlo de baja a mano tiene tres fallos, y el tercero es el grave:
se olvida, tarda —y entre la petición y el apunte puede salir otro correo—, y
**si nadie vacía ese buzón la persona pidió la baja creyendo que servía**. Eso
es peor que no haber dado la salida, y con la revisión jurídica abierta (P-09)
es exactamente el tipo de cosa que un abogado señala.

**Riesgo.** Una petición de baja sin atender es una queja de spam a punto de
ocurrir, y la tasa de quejas es la cifra que más pesa en la reputación del
dominio (§1 del reporte). También es un incumplimiento: quien pide no recibir
tiene derecho a no recibir.

**Impacto.** Hoy, cada semana sin vaciar el buzón. Con una campaña de miles de
personas, decenas de peticiones.

**Alternativas.** Son tres, y «como lo hacen otras páginas web» es justamente la
que ARLES no puede hacer:

| | Cómo | Lo que cuesta |
|---|---|---|
| **A · Leer el buzón por IMAP** | ARLES entra al buzón del remitente con las mismas credenciales del llavero, busca las respuestas a sus propios envíos, y da de baja a quien escribió «BAJA» | Una dependencia nueva (`cargo deny`), un adaptador con su servidor de mentira —el patrón de `arles-email`—, y **decidir qué cuenta como una baja**: ver abajo |
| **B · Un enlace de un clic, como las webs** | Lo que hace todo el mundo: `https://…/baja?token=…`. Necesita **un servidor** — ARLES es una aplicación de escritorio | Hosting, dominio, mantenimiento, y **datos personales saliendo del equipo del usuario**, que es justo lo que el producto evita. Es el «relay ciego» que Dirección decidió el 29/09 para contar respuestas de WhatsApp, y no existe |
| **C · Dejarlo a mano** | Lo de hoy | Los tres fallos de arriba |

**Trade-off.** La **A** automatiza sin infraestructura y usa credenciales que ya
están guardadas, pero **hay que leer correo de personas**, y eso exige acotarlo
con cuidado. La **B** da la experiencia que Dirección describe y mete al
producto en el negocio de alojar datos. La **C** no cuesta nada y falla por
olvido.

**Recomendación: la A, con cuatro límites escritos antes de empezar.**

1. **Sólo las respuestas a correos que ARLES mandó**, reconocidas por
   `In-Reply-To` contra el `Message-ID` del envío. Una búsqueda por palabra en
   todo el buzón leería correo que no es de esto.
2. **Sólo se mira si la respuesta contiene la palabra clave**, y **no se guarda
   el cuerpo**: lo que entra en la base es la dirección, el motivo y la fecha —
   igual que hoy—, nunca el texto que escribió la persona.
3. **Una respuesta ambigua no da de baja sola: la deja propuesta.** Alguien que
   escribe «qué precio tiene» no se da de baja, y alguien que escribe «no me
   mandes más» sí. Entre los dos hay un montón de frases, y **dar de baja de
   más también es un daño**: se pierde un contacto que no lo pidió. Las dudosas
   se enseñan para confirmar en un clic, que sigue siendo mejor que teclear.
4. **La baja es irreversible y no se deshace** (es la regla de la Fase 5), así
   que el camino automático tiene que fallar del lado de **no dar de baja** y
   dejarlo propuesto, nunca al contrario.

**Lo que esto necesita en el código**, medido el 23/10/2026:

- Un **quinto origen** en `OrigenDeSupresion` —hoy son `smtp_5xx`, `user`,
  `import`, `whatsapp_webhook`— y por tanto **una migración**, porque el
  `CHECK` de la V3 los limita.
- `message_attempt` tendría que guardar el `Message-ID` que el proveedor
  devolvió, para poder emparejar la respuesta. Hoy guarda el del proveedor en
  `provider_message_id`: hay que comprobar si sirve.
- Un adaptador IMAP en un crate propio con su servidor de mentira, como
  `arles-email`, porque **lo que habla con la red se prueba contra algo que no
  es la red**.

**Y una limitación que no desaparece con ninguna de las tres:** quien quiera
darse de baja y no responda —porque pulsa «no deseado» directamente— no deja
rastro que ARLES pueda leer. Esa persona sólo aparece en Google Postmaster
Tools, como una queja. **Es la razón por la que el pie de baja tiene que ser
fácil de encontrar**, no una letra pequeña.

---

## Decisiones pendientes

Ver **`02-auditoria/PREGUNTAS_ABIERTAS.md`** para el registro completo.

| # | Pregunta | Bloquea |
|---|---|---|
| P-01 | ¿Qué licencia de Mont posee exactamente TELEMETRY? | **La demo** (antes: Fase 2) |
| P-03 | Origen y consentimiento de los contactos | Fase 3 |
| P-09 | Marco de protección de datos vigente | ~~Entrega 3.3~~ → **apartado del camino crítico por D-7.** Frena los textos legales visibles y el primer envío real a terceros, no la construcción |
| P-05 | Dominio y aviso de privacidad para Google | Fase 5 |
| **R-25** | **¿La baja automática se hace leyendo el buzón por IMAP, o con un enlace de un clic que necesita un servidor?** Pedida el 23/10/2026. **Recomendación: IMAP**, con los cuatro límites escritos arriba — automatiza sin infraestructura y sin que los datos salgan del equipo | Que las bajas se apunten solas |
| **R-05** | **¿Una imagen en la firma y en la plantilla va adjunta en línea o alojada en un servidor?** La adjunta no delata nada y pesa más; la alojada pesa menos y **le dice a tu servidor cuándo se abrió el correo**, que es lo que ARLES promete no saber (§65). **Recomendación: adjunta en línea, con topes duros de peso.** Ver `07-entrega/RIESGOS-DE-REPUTACION.md` §4 | Las imágenes en los correos |

**Ya no hay bloqueantes de la Fase 2.**

**Dos pendientes que se cerraron el 22/10/2026:** **D-REM** —cómo se reparte el
cupo entre campañas del mismo remitente— con **D-13 y D-14**, y la pregunta del
contacto repetido con **D-16**. **D-WA** —la campaña entera por WhatsApp— sigue
abierta, y Dirección la puso después de blindar el dominio.
