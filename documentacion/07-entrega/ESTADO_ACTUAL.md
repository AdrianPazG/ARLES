# Estado actual · dónde está ARLES y qué sigue

**Proyecto:** ARLES RELAY I · v1.2.0
**Última actualización:** 27 de octubre de 2026 (**D-WA cerrado: una campaña entera por WhatsApp** · el semáforo de calidad · las plantillas de Meta · el asistente por canal · la 7.5 de entregabilidad · el umbral de rebotes · el orden del proyecto)
**Para qué sirve este documento:** que quien retome el trabajo —otra sesión, otra persona, otro equipo— sepa en una lectura qué está construido, qué decisiones ya están tomadas, qué **no** hay que volver a discutir y qué es lo siguiente.

> Se actualiza **al cerrar cada entrega**, no al empezarla. Un documento de estado que describe intenciones en vez de hechos es peor que no tenerlo: se lee como si fuera cierto.

---

## 1. En una frase

**ARLES ya manda correos, y ya sabe qué mandar.** La parte de correo de la Fase 5 está cerrada entera (5.1 a 5.6), y la Fase 6 está cerrada: plantillas con variables, saneado en Rust, versiones que no se editan, campañas con la audiencia congelada al activar, el preflight de ocho puntos enchufado de verdad, y el asistente de nueve pasos.

**Se puede crear una campaña de principio a fin y activarla**, con las nueve pantallas del asistente, el simulador de duración, las ocho comprobaciones y el correo de prueba obligatorio.

**Y ARLES cuenta lo que hace.** La Fase 7 está cerrada salvo la 7.5: el freno —pausar, reanudar, detener y la parada de emergencia, con botón—, el registro de actividad, las cifras de cada campaña, la pantalla de Actividad con **por qué no sale nada**, el registro con sus seis filtros y el resumen exportable.

**El canal de WhatsApp (W.1 a W.8) está construido**, y desde la 7.7 se puede mandar un WhatsApp de prueba desde la aplicación — hasta entonces el comando existía sin botón.

**Y desde el 26/10/2026 WhatsApp es un canal entero por dentro** (D-WA): el motor recorre las dos clases de etapa, el contenido viaja hasta el adaptador, una campaña de WhatsApp se crea y se activa congelando **los móviles y no los correos**, y el canal cruza la frontera como una variante etiquetada que rechaza cualquier mezcla. El alta de un número son **un dato y una elección**: se pega el token y ARLES le pregunta a Meta cuáles son tus números.

**Y desde el 27/10/2026 se puede crear y lanzar una campaña entera por WhatsApp desde la aplicación.** El tramo 3c cerró las tres piezas que faltaban: el semáforo de calidad se llena —al dar de alta el número y con un botón de refresco, porque la calidad cambia sola—, las plantillas aprobadas de Meta se apuntan en Plantillas › WhatsApp, y **el asistente pregunta el canal en el paso 1**: el paso 3 ofrece números con su calidad y su tope, el 4 plantillas de Meta sin firma, el 7 la puerta del canal con las dos cifras de permiso, y el 8 el WhatsApp de prueba.

**Lo que WhatsApp todavía NO hace:** contar las respuestas. Meta las manda por *webhook* y ARLES no tiene URL a la que llamar, así que la cifra de «cuánta gente contestó» es cero en WhatsApp — y **una baja pedida por WhatsApp se apunta a mano** en Contactos › Bajas (decisión de Dirección del 27/10/2026, con su motivo en el registro de requisitos).

**Y ARLES ya sabe guardar y recuperar todo lo que tiene.** La Fase 8 está cerrada entera (8.0 a 8.7, más la 8.3 bis y las cinco auditorías): respaldo cifrado con dos alcances, restauración que reemplaza y une las bajas, archivado del historial a un disco externo verificado antes de borrar, la pantalla de recuperación para el día que se pierda la llave del llavero, y la migración V11 que dejó la lista de contactos en 357 ms y la búsqueda por correo en menos de 1.

**Y el 22/10/2026 ARLES mandó su primer correo real, a tres bandejas principales de tres.** R-24 —el bloqueo más viejo del proyecto, abierto desde la Fase 5— **está cerrado**: Dirección dio de alta `enlace@telemetrymx.com` (Titan, SMTP real con TLS) y el correo llegó a la bandeja principal de Outlook, de Gmail y de un corporativo. Hasta entonces todo lo de la Fase 5 estaba probado contra un servidor de mentira.

> **Lo que costó no fue el código.** Desde una dirección `@gmail.com` el correo caía en no deseados de Outlook **con toda la autenticación en verde** —`spf=pass`, `dkim=pass`, `dmarc=pass`, `compauth=pass`, y aun así `SCL: 5`—. Lo que pesaba era el patrón «nombre de marca + dirección personal de Gmail», que para un filtro es la firma de una suplantación y **no se arregla con código**: se arregla mandando desde el dominio propio. Está contado en [`QUE-HACER-CON-EL-SPAM.md`](QUE-HACER-CON-EL-SPAM.md) y los pasos en [`PASO-A-PASO-DEL-CORREO.md`](PASO-A-PASO-DEL-CORREO.md).

**Y desde el 08/10/2026 la aplicación arranca de verdad.** Hasta ese día ARLES compilaba, abría su ventana y **dentro no había nada**: faltaba la característica `custom-protocol` de Tauri, sin la que la interfaz no viaja dentro del ejecutable. **Se abre con `cargo arles`**, nunca con `cargo run --release` a secas. Está en [`COMO-ABRIR-ARLES.md`](COMO-ABRIR-ARLES.md), con los cuatro tropiezos ya vistos en Windows.

---

## 2. Lo que está construido y verificado

| Fase | Estado | Lo que hay |
|---|---|---|
| F0 · Discovery | ✅ cerrada | Auditoría, riesgos, decisiones |
| F1 · Cimientos | ✅ cerrada | Workspace, SQLCipher, migraciones V1–V2, llavero, CI |
| F2 · Design System | ✅ cerrada | Tokens, 9 primitivas, 4 estados, catálogo visual, contrastes medidos |
| F3 · Empresa y contactos | ✅ cerrada | Empresa, contactos, importación CSV/XLSX, supresión, derechos |
| F3W · Canales | 🟡 55 % | V3 y V4 aplicadas: `contact_channel`, consentimiento, etapas de campaña |
| F4 · Motor de ejecución | ✅ cerrada | Máquina de estados, ritmo, ventana con zona IANA, reintentos, disyuntor, cola, presupuesto, pausa/detención, preflight |
| **F5 · Proveedores de envío** | **🟡 70 %** | **La parte de correo cerrada (5.1 a 5.6). Falta WhatsApp. Ver §3** |
| **F6 · Campañas y mensajes** | **✅ cerrada** | **Plantillas, saneado, versiones, campañas, preflight y el asistente de nueve pasos (6.1 a 6.7), con su auditoría. Ver §3 bis** |
| F7 · Actividad | 🟡 92 % | El freno, el registro, las cifras, la pantalla de Actividad, el registro con filtros, el resumen exportable y la auditoría de cierre (7.0 a 7.4, 7.6 y 7.7). Falta sólo la 7.5 · SPF/DKIM/DMARC, movida al final por Dirección. Ver `09-fases/FASE-07-ACTIVIDAD.md` |
| **F8 · Respaldos y endurecimiento** | **✅ cerrada** | **Respaldo cifrado con dos alcances, restauración que reemplaza y une las bajas, archivado del historial verificado antes de borrar, la pantalla de recuperación del llavero perdido, el banco de pruebas con las trece cifras, la migración V11 y las cinco auditorías (8.0 a 8.7, más la 8.3 bis). Ver `09-fases/FASE-08-RESPALDOS.md`** |
| F9 · Release v1.2.0 | ⬜ sin empezar | Instaladores firmados, updater, despliegue interno |

**Cifras del 22/10/2026:** las **diez** fases del validador **con las sondas corriendo de verdad** y corridas **de una en una** — **340 comprobaciones, 0 fallos y 0 omitidas** (5·53·38·95·42·36·20·21·11·19). `fmt`, `clippy`, `eslint` y `vue-tsc` limpios.

> **El denominador encoge en silencio si falta algo del entorno.** Ese mismo día, en un contenedor recién creado, la fase 1 dio «45 pasan, 0 fallan» **de 49**: sin `node_modules` ni llavero, cuatro comprobaciones se marcaron omitidas y el total bajó solo. Nada en rojo. Con `npm ci` y `gnome-keyring dbus-x11` puestos son **53 de 53**, y el arranque con llavero se ejercita de verdad en vez de declararse.
>
> **Se lee la cifra entera** —pasan, fallan, omitidas y total—, nunca sólo el «0 fallan».

> **Y el validador se corre de una en una, por un defecto suyo que se arregló el 22/10/2026.** Midiendo estas cifras la fase 1 dio **52 de 53** con tres validadores a la vez, y **53 de 53** corrida sola. La comprobación del CSS de los tokens ejecutaba `generar.py --css` —que escribe **dentro del repositorio**— y después preguntaba a `git diff`: uno escribía mientras el otro comparaba. **Medido: 1 falso de 40.** Ahora genera a un temporal y compara contenidos: **0 de 120**, y comprobar ya no ensucia el árbol de trabajo.
>
> **No se sabe qué comprobación dio aquel 52/1**, porque la salida no se guardó, y no se afirma: lo que se midió es el mecanismo. Un hallazgo falso en un instrumento cuesta más que uno ausente (auditoría B).

> **El bucle son diez fases, no nueve**, y correr sólo la tuya no basta: en la 5.2 y la 5.3 se rompieron cuatro comprobaciones de las fases 2, 3 y 4 sin que nadie lo viera (§8 bis).
>
> ```bash
> for f in 0 1 2 3 4 5 6 W 7 8; do python3 herramientas/validar/validar.py --fase $f; done
> ```

**Avance de la v1.2.0: 89 %**, calculado desde `avance.json` el 07/10/2026.

---

## 3. La Fase 5 por dentro

### Cerrado

**5.1 · Traducir lo que dice un servidor** — `arles-email/src/codigos.rs`
Un servidor habla en números de tres cifras; el motor entiende cinco fallos. Confundir un `450` con un `550` da de baja a alguien alcanzable, y una baja no se deshace. El texto libre **no decide nada**: sólo el código y el código extendido de RFC 3463.

**Y un servidor SMTP de mentira** — `arles-email/src/servidor.rs`
Habla el protocolo de verdad y contesta lo que se le ordene. Existe porque a Gmail no se le puede pedir que conteste `550`. Guarda lo que le dijeron y **no guarda el cuerpo**, para que el contenido no acabe en la salida de una prueba en CI.

**5.2 · El adaptador SMTP** — `arles-email/src/smtp.rs`, `tls.rs`
Cifrado obligatorio, sin forma de desactivarlo. El adaptador **comprueba el cifrado por su cuenta** antes de mandar la contraseña, además de configurar `lettre` para exigirlo: son dos defensas y hacen falta las dos (comprobado tumbando cada una por separado).

La conversación se lleva paso a paso —`MAIL FROM`, `RCPT TO`, `DATA`— y no con la función cómoda de `lettre`, porque un mismo `550` significa cosas opuestas según cuándo llegue, y sólo el `RCPT TO` y el acuse final pueden acabar en una baja.

**5.3 · La cuenta: probarla, guardarla y darla de alta**
- `arles-email/src/prueba.rs` — «Probar conexión»: seis diagnósticos, cada uno con **una acción distinta**. No manda ningún correo.
- `arles-core/src/correo.rs` — la cuenta validada. El servidor **no puede ser una IP**.
- `arles-db/migrations/V7__cuenta_smtp.sql` — las cuatro columnas de SMTP que faltaban en `email_account` desde la V1.
- `arles-db/src/correo.rs` — el repositorio. Ninguna contraseña.
- `arles-app/src/llavero.rs` — una entrada de llavero por cuenta.
- `arles-app/src/comandos.rs` — `remitentes`, `probar_remitente`, `guardar_remitente`.
- `app/src/app/pantallas/PantallaRemitentes.vue` — la pantalla, con la guía de la contraseña de aplicación de Gmail.

**5.4 · El motor corriendo de verdad** (cerrada el 28/09/2026)

Resultó ser bastante más de lo que este documento decía. La frase era «construir el `ProveedorSmtp` y dárselo al trabajador», y al ir a buscar ese trabajador **no existía**: `arles-app` ni siquiera dependía de `arles-motor`. Estaban construidas las dos mitades —la que gestiona la cola y la que habla SMTP— y no estaban presentadas. Ninguna prueba lo cazaba, porque cada mitad pasaba la suya.

- `arles-app/src/fabrica.rs` — de una cuenta guardada al adaptador. **El único sitio donde una contraseña sale del llavero.** Seis motivos distintos por los que una cuenta puede no enviar, cada uno con su acción.
- `arles-db/src/trabajo.rs` — `etapas_en_marcha`: qué está enviando ahora mismo, con su ventana y el **más bajo** de sus topes.
- `arles-app/src/motor.rs` — el hilo, aparte de la ventana (§51). **No muere por un error**: se reintenta en la vuelta siguiente.
- `arles-app/src/estado.rs` — toma el cerrojo de instancia única y arranca el motor.
- `app/src/app/pantallas/PantallaInicio.vue` — el aviso de «esta ventana no está enviando».

**Tres cosas que conviene saber antes de tocar esto:**

1. **La puerta se abre con `connected`, no se cierra ante una lista.** Escrita al revés, un estado nuevo enviaría correo hasta que alguien se acordara de prohibirlo.
2. **Los límites de envío ya no están en `campaign`.** La V4 los movió a `campaign_stage`, junto con la ventana y el remitente. El esquema de la V1 todavía los enseña, y buscarlos ahí cuesta un rato.
3. **`localhost` no pasa la validación del núcleo** — un servidor de correo sin punto no es un nombre de dominio. Para una prueba local hace falta algo como `a.localhost`.

**Lo que 5.4 NO deja verificado, y hay que decirlo:** el hilo del motor **no se ha visto mandar un correo de punta a punta**. Su pegamento está probado contra un puerto cerrado, y el envío en sí contra el servidor de mentira de `arles-email`, pero las dos cosas no se han visto juntas. El servidor de mentira que habla cifrado no se exporta a propósito: sacarlo metería un motor de TLS en el programa que se entrega. Su sitio natural es la 5.5.

**5.5 · El envío de prueba** (cerrada el 29/09/2026)

- `arles-db/migrations/V8__envio_de_prueba.sql` — dónde consta la prueba de una **etapa**, y con qué cuenta se hizo. Cambiar de remitente la invalida, y **no la borra**: «se probó con otro remitente» es lo que hay que poder enseñar.
- `arles-db/src/prueba.rs` — registrarla y saber si todavía vale.
- `arles-app/src/comandos.rs` — dos caminos: `enviar_correo_de_prueba` (por etapa, anota) y `probar_envio_de_la_cuenta` (por cuenta, **no anota**). Son dos preguntas distintas.
- `arles-app/tests/envio_de_prueba.rs` — **ARLES mandando un correo entero**, contra un servidor que habla cifrado, comprobando lo que el servidor RECIBIÓ.

**Un envío de prueba no toca nada más:** no entra en la cola, no gasta presupuesto y **no puede dar de baja a nadie**. Si pudiera, probar una cuenta mal configurada contra la dirección propia daría de baja a quien la escribió.

**5.6 · Cierre de fase** (cerrada el 29/09/2026) — auditoría, documento de fase, informe para Dirección y PDF. Ver `09-fases/FASE-05-PROVEEDORES-DE-ENVIO.md`.

### Lo siguiente

**Fase 6 · Campañas y mensajes.** Plantillas con versionado, sustitución de variables no Turing-completa, sanitizado con `ammonia` en Rust, firmas, el asistente de nueve pasos, el simulador y el preflight.

**Y una decisión pendiente de Dirección:** el adaptador de **WhatsApp Cloud API** es de la Fase 5 y no está empezado. La Fase 6 ofrece dos etapas en el asistente, y la segunda no podrá enviar mientras eso falte. Planteado en `09-fases/FASE-05-PARA-DIRECCION.md`.

---

## 3 bis. La Fase 6 por dentro

### Cerrado

**6.1 · El motor de sustitución** — `arles-core/src/plantilla.rs`
`{{clave}}` contra una lista cerrada de cuatro variables. Sin condicionales, sin
bucles, sin funciones (ADR-0006). **Lo sustituido no se vuelve a mirar**: si el
nombre de un contacto fuera `{{empresa}}`, se queda así. Eso cierra la vía de
ejecución desde un XLSX que no escribimos nosotros.

**6.2 · El saneado del HTML** — `arles-core/src/sanear.rs`
Lista blanca con `ammonia`, en Rust, al guardar **y** al enviar. El saneado del
navegador no protege a quien recibe el correo: el correo sale de Rust.

**6.3 · Plantillas y firmas** — `arles-db/src/plantillas.rs`
Versiones inmutables. Una versión es lo que se envió; editarla reescribiría el
pasado sin que nada fallara. Las variables se leen del contenido, nunca se
aceptan de quien llama.

**6.4 · Campañas** — `arles-db/src/alta_de_campana.rs`
Crear, listar y activar. Activar hace tres cosas en **una** transacción:
congelar la audiencia, llenar la cola y poner la campaña en marcha. Aquí se
encontró un fallo real: la consulta de la audiencia no filtraba por canal
activo, así que un canal archivado habría recibido correo.

**6.5 · El preflight, enchufado** — `arles-db/src/revision.rs` y el comando
`revisar_campana`
Cinco puntos salen de la base; los tres de red los completa `arles-app`
sondeando la cuenta, porque `arles-db` no habla con nadie de fuera. Los que no
se pudieron mirar se dicen «sin comprobar» y bloquean igual — rellenarlos a mano
con `Some(true)` es el atajo que el validador vigila.

Y el envío de prueba de la 5.5, que llevaba tres entregas anotándose sin que
nadie lo preguntara, por fin llega a su punto. Vale **sólo con el remitente que
la etapa usa hoy**: si se cambia de cuenta, la prueba anterior demostró que otra
cuenta podía enviar.

**6.6 · El asistente de nueve pasos** — `app/src/app/pantallas/`
Nombre, a quién, desde qué cuenta, qué se manda, ritmo y horario, cuánto va a
tardar, las ocho comprobaciones, el correo de prueba y activar. Tres de los
nueve pasos no piden nada: se leen, y son los que hacen reconsiderar.

La pantalla **no decide** ninguna de las tres reglas que parece decidir: si se
puede activar lo dice el preflight, el aviso del §48 lo exige `crear_campana`, y
la prueba obligatoria es uno de los ocho puntos. La pantalla apaga botones; las
reglas viven en Rust, donde no se saltan por otro camino.

**6.7 · El cierre** — auditoría, documento e informe
Cinco hallazgos. Tres corregidos: el asistente sondeaba el servidor dos veces
seguidas para calcular lo mismo, un paso podía tardar un minuto diciendo sólo
«Calculando», y faltaba el rótulo de respaldo de un estado desconocido. Dos
resultaron estar bien y ahora tienen prueba: que la cifra del paso 6 sea la que
se congela al activar, y que un ritmo de cero no deje una campaña colgada.

Y una de esas pruebas **hubo que arreglarla**: pasaba igual con el programa
roto, porque a los datos de prueba les faltaba un canal archivado.

### Lo que la Fase 6 deja fuera, dicho y no prometido

Los filtros de audiencia, el editor de plantillas, y editar una campaña ya
creada. El asistente dice en el paso 2 que llegan todos los contactos con
correo: enseñar un filtro que no filtra sería peor que no tenerlo.

---

## 4. Las tres reglas de esta fase que no se negocian

Están escritas varias veces a propósito, y hay comprobaciones que fallan si alguien toca una.

**1 · No existe ninguna forma de no cifrar (§29).** No es que la casilla esté desaconsejada: no hay ningún valor, en ningún sitio, que represente una conexión sin cifrar.

- `arles_core::correo::CifradoDeCorreo` tiene dos variantes y las dos cifran.
- `arles_email::tls` no sabe construir parámetros sin cifrado.
- `smtp_security` sólo admite `starttls` y `directo` (V7).

Y el validador comprueba, sobre el árbol entero **con las pruebas incluidas**, que las llamadas de `lettre` que desactivan la verificación de certificados no estén escritas en ninguna parte.

**Cómo se prueba sin abrir un agujero:** el servidor de mentira genera un certificado de verdad y al adaptador se le da **ése y ninguno más** como raíz de confianza. La verificación sigue entera.

**2 · La contraseña entra y no sale.** Vive en el llavero del sistema. En la base hay una referencia opaca. Ningún comando la devuelve, ninguna estructura que se serialice tiene un campo para ella, y la pantalla la borra al cerrar el formulario. Cuatro comprobaciones distintas lo vigilan; dos miran el nombre del campo y dos miran el valor, y **hacen falta las cuatro** (una rotura con un campo llamado `clave_de_acceso` sólo la cazaron las que miran el valor).

**3 · Nada antes del `RCPT TO` puede dar de baja a nadie.** Es la única puerta a una supresión, junto con el acuse final. Un error de conexión, de cifrado o de `MAIL FROM` es, como mucho, parar la cuenta.

---

## 5. Decisiones ya tomadas · no volver a discutirlas

| Qué | Decisión | Dónde está el porqué |
|---|---|---|
| Guardar una cuenta | **La prueba antes**, y la guarda igual si la prueba falla, marcándola como «hay que configurarla» | `comandos.rs`, sobre `guardar_remitente` |
| Orden de escritura | **Primero el llavero, después la base.** Si falla la base, el secreto recién escrito se retira | ídem |
| Probar una cuenta guardada | **No exige reescribir la contraseña** | ídem |
| La prueba de conexión | **No manda `MAIL FROM` ni `RCPT TO`** | `prueba.rs`, cabecera |
| Obligatoriedad de las columnas SMTP | **En el código, no en el esquema.** SQLite revalida las filas existentes al añadir una columna con `CHECK`, y eso hace imposible migrar una base con cuentas | `V7__cuenta_smtp.sql` |
| Cuentas SMTP anteriores a la V7 | Quedan en `needs_reauth`. **No se les inventa un servidor** | ídem |
| El puerto en el formulario | Viaja **como texto**, para que un «58a» marque su casilla en vez de romper la petición | `arles-core/src/correo.rs` |

---

## 6. Lo que está bloqueado esperando a alguien de fuera

Ninguno impide construir. Los cuatro impiden **entregar** la v1.2.0.

| Qué | Quién | Qué bloquea exactamente |
|---|---|---|
| **P-01 · Licencia de Mont** | Dirección | Distribuir el instalador con la tipografía dentro. Plan B: tipografía del sistema |
| **P-09 · Revisión jurídica** | Abogado | Los textos legales visibles. **No** frena la construcción (ADR-0013) |
| **META · Cuenta con Coexistencia** | Operaciones | El alta del número contra una cuenta real. Y por debajo hay un límite mayor que no depende de Meta: el motor sólo recorre `Canal::Correo` |
| **R-07 · Revisión en macOS** | Quien tenga un Mac | Nadie ha abierto ARLES en WKWebView. La fecha que decía «antes de la F6» caducó: la F6, el canal W, la F7 y la F8 cerraron sin eso |

**Eran cinco.** `R-24` se cerró el 22/10/2026 — ver §7.

---

## 7. R-24 · cerrado el 22/10/2026 · ARLES manda correo de verdad

**Era el riesgo abierto más viejo del proyecto**, desde la Fase 5. Todo lo de esa fase estaba probado contra un servidor de mentira que habla el protocolo de verdad y sabe fallar de seis maneras — eso cubre las ramas de error mucho mejor que una prueba manual, pero no cubría lo que sólo aparece con un servidor real.

**Cómo se cerró.** Dirección dio de alta `enlace@telemetrymx.com` (Titan, SMTP real con TLS) y ARLES mandó un correo de prueba a tres destinos: Outlook, Gmail y un corporativo. **Los tres llegaron a la bandeja principal.**

### Lo que costó, y por qué conviene no olvidarlo

**No fue el código.** Durante días el correo salía desde una dirección `@gmail.com` y Outlook lo mandaba a no deseados **con toda la autenticación en verde**. Las cabeceras reales que mandó Dirección lo dicen:

```text
spf=pass          la IP que envió está autorizada
dkim=pass         la firma del mensaje verifica
dmarc=pass        el dominio del remitente cuadra con la firma
compauth=pass     reason=100 — la autenticación compuesta, perfecta
BCL:0             no lo considera envío masivo problemático

X-MS-Exchange-Organization-SCL: 5     ← esto es lo que lo mandaba a Junk
```

Lo que pesaba era el patrón **«nombre de marca + dirección personal de Gmail»**, que para un filtro es la firma de una suplantación. **Eso no se arregla con código**: se arregla mandando desde el dominio propio.

> **Y tiene una consecuencia que afecta a lo que se construye:** la entrega 7.5 (SPF, DKIM y DMARC desde ARLES) **no iba a resolver esto**, porque las tres ya pasaban. El ROADMAP la tenía apuntada como la solución al spam y no lo era.

### Lo que ARLES ganó por el camino

- **El alta de una cuenta son dos campos** desde el 14/10 (R-01): correo y contraseña. `PROVEEDORES` relaciona cinco dominios con su servidor, puerto y cifrado, y el par **nunca se cruza** — 587 con STARTTLS, 465 con TLS directo, y ningún tercer puerto.
- **La cabecera `List-Unsubscribe`** en los correos de campaña, y **fuera** de los envíos sueltos, que no son de ninguna lista.
- **Las dos versiones del mensaje**, HTML y texto plano, con el HTML como documento entero con su idioma y su título.

### Lo que sigue sin probarse contra algo real

**El número de WhatsApp** (META). El adaptador de Cloud API, su servidor de mentira, la pantalla y el envío de prueba están construidos, pero nunca han hablado con Meta.

---

## 8. Cómo se trabaja aquí

Lo esencial está en `CLAUDE.md`, en la raíz. Tres cosas que conviene no descubrir a golpes:

**La regla del proyecto es que una comprobación que nunca se ha visto fallar no está verificada.** Cada garantía se rompe a propósito para ver si la comprobación la caza. No es ceremonia: en esta fase encontró, entre otras, que la función de `lettre` para detectar fallos de cifrado **no funciona** y su rama no se ejecutaba nunca; que una prueba buscaba una contraseña por su texto cuando viaja codificada, y por tanto no podía fallar; y que la lista de alta llevaba semanas anunciando como futura una pantalla ya construida.

**Se responde siempre en español**, incluidos comentarios y mensajes de commit.

**Un tramo, se verifica, se empuja, se explica, y se pregunta antes del siguiente.** Dirección no es programadora: las explicaciones van en sus términos, con cifras y no adjetivos, y diciendo lo que falló.

### Verificación, en orden

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check licenses advisories

cd app && npm test && npm run lint && npx vue-tsc --noEmit -p tsconfig.json

python3 herramientas/validar/validar.py --fase 5          # 0 a 5
python3 herramientas/validar/validar.py --fase 5 --rapido  # omite las sondas
```

> ### El disco de la sesión se llena, y pasa a menudo
>
> La compilación de este workspace ocupa decenas de GB: medido el 23/09/2026, `target/debug/deps` sola llegó a 20 GB y la caché incremental a 5 GB. Cuando el disco se agota, el síntoma **no** parece de disco: el validador falla en las fases 1 a 4 con pruebas «rotas» que en realidad no llegaron a compilar. Antes de buscar un fallo en el código, mirar el final del error: `No space left on device`.
>
> ```bash
> df -h /                                   # ¿queda sitio?
> rm -rf target/debug/incremental           # lo primero, y suele bastar
> rm -rf target/debug/build                 # si hace falta más
> find target/debug/deps -name "*.rmeta" -delete
> CARGO_INCREMENTAL=0 cargo test --workspace
> ```
>
> No es un fallo del proyecto y no hace falta empezar una sesión nueva: borrar libera al instante y lo borrado se regenera al compilar.

---

## 8 bis. Una lección de esta fase: correr **todas** las fases, no sólo la tuya

Durante las entregas 5.2 y 5.3 se verificó con `--fase 5`, que pasaba entera, y
**se rompieron cuatro comprobaciones de las fases 2, 3 y 4 sin que nadie lo
viera**. Salieron todas juntas el 24/09/2026, al repasar el proyecto:

| Qué se rompió | Por qué importaba |
|---|---|
| **Seis tokens de diseño inventados** en la pantalla de Remitentes (`--espacio-4` en vez de `--arles-space-4`) | El navegador resuelve un token inexistente **a nada**, sin dar error: la pantalla se veía mal y ninguna prueba lo notaba. Es el fallo de CSS que más lejos llega sin que nadie lo vea |
| La **huella de la migración V7** no se registró | La comprobación que impide editar una migración ya publicada no podía vigilarla |
| El **generador de la vista previa** se quedó con las rutas viejas | La vista previa es **donde revisa Dirección**. Ya pasó una vez en septiembre: se reportó dos veces el mismo defecto porque el simulador mentía |
| La comprobación del `localStorage` **se cazó a sí misma** | Los archivos nuevos explican en sus comentarios que la contraseña NO va ahí, y la comprobación leía esos comentarios |

Los cuatro están arreglados. Dos cosas que conviene llevarse:

**Antes de empujar, correr las diez fases**, no sólo la que se está tocando.
Cuesta un minuto con `--rapido`.

**Y el patrón del comentario que se caza a sí mismo ya va por la novena vez.**
Cualquier comprobación nueva que busque un texto en un archivo tiene que quitar
los comentarios antes de mirar, porque la documentación de la regla contiene la
palabra que la regla prohíbe. **Saberlo no basta** — ha vuelto a pasar dentro de
comprobaciones escritas el mismo día para arreglar otra igual, y una vez dentro
del propio bucle de rupturas, dando por restaurado un archivo que seguía roto.

**La novena tiene otra cara y conviene distinguirla:** no fue un comentario, fue
**el sitio**. La comprobación de la guía de arranque preguntaba si la cadena
`cargo arles` aparecía **en el archivo entero**, y aparecía — en la prosa de
solución de problemas, mientras el bloque que se copia seguía mandando el
comando que no sirve. **Medir que una cadena aparece no es medir que el sitio
que importa la tenga.**

## 9. Lo que este documento no cubre

- **El porqué de cada decisión de producto** está en `01-producto/DECISIONES_DE_DIRECCION.md` y en los ADR de `03-arquitectura/adr/`.
- **El estado de cada fase con su cifra** está en `07-entrega/avance.json`, que es la fuente única del porcentaje.
- **Lo que pasó en cada entrega** está en `CHANGELOG.md`, escrito para que lo lea Dirección.
- **Cómo se abre la aplicación** está en [`COMO-ABRIR-ARLES.md`](COMO-ABRIR-ARLES.md). No es un detalle de instalación: hasta el 08/10/2026 **nadie había construido ARLES como aplicación**, y el día que se hizo apareció un hueco del §9 que nueve fases de pruebas no podían ver.

---

## 10. Lo siguiente

**La Fase 9 · Release**, con tres cosas que la Fase 8 dejó anotadas y una decisión pendiente:

| | Qué | De dónde viene |
|---|---|---|
| **1** | **Sacar la activación de campañas a segundo plano**, con avance y botón de cancelar. Hoy tarda 73.5 s con 451 196 destinatarios y la interfaz no contesta; el presupuesto de 2 s se rompe hacia los 30 000 | Auditoría C |
| **2** | **El filtro del registro** (C-2): buscar texto tarda 3 709 ms. Declarado con su cifra, no arreglado | Auditoría C |
| **3** | **La 7.5 · SPF, DKIM y DMARC**, que Dirección movió al final el 30/09/2026: es lo único que necesita red y una dependencia nueva | Fase 7 |
| **4** | **El mensaje de «fallo al migrar el esquema» no cumple el §95**: manda a cerrar y reabrir —que da el mismo error siempre— y a restaurar un respaldo que puede no existir, y no dice dónde vive la base. La salida real está hoy sólo en la guía de arranque | 08/10/2026, primer intento real |
| **5** | **La 7.5 ya se puede verificar de verdad.** Su condición era cerrar R-24, y se cerró el 22/10/2026: el dominio `telemetrymx.com` tiene SPF, DKIM de Titan y DMARC en `p=none`, comprobados con mxtoolbox. Ojo con lo que **no** arregla: el spam de Outlook no venía de ahí (§7) | 22/10/2026 |

**Y tres decisiones que esperan a Dirección:**

| | Qué | Dónde está planteada |
|---|---|---|
| **D-WA** | Construir el adaptador de WhatsApp para el motor antes de la Fase 9, o entregar la v1.2.0 con campañas sólo por correo | `09-fases/FASE-05-PARA-DIRECCION.md` |
| **R-05** | La imagen en la firma del correo: embebida —engorda cada envío y dispara filtros— o alojada, que hay que alojarla. El saneado del HTML tendría que admitir `<img>` con sus reglas | Pedido el 21/10/2026 |
| ✅ **D-REM** | **Cerrada el 22/10/2026 con D-13 y D-14**, y no como se planteó: en vez de impedirlo, los topes **se suman** —cada campaña manda lo suyo y el plazo se cumple— y el total del buzón sale en Remitentes con su semáforo. El agregado ya no tiene techo, a propósito (§154) | `01-producto/DECISIONES_DE_DIRECCION.md` D-13 y D-14 |

---

## 11. Las dos rondas de Dirección usando ARLES · 13 al 22/10/2026

**Lo que encontró usando el producto no lo había encontrado ninguna de las cinco auditorías**, y conviene saber por qué: las auditorías miden, y lo que falló es lo que sólo se ve usando.

| | Qué pasaba | Estado |
|---|---|---|
| **H-01** | Remitentes decía «todavía no has configurado tu empresa» **después** de configurarla | ✅ |
| **H-02** | Las tres partes de un error salían **pegadas**: «…con el servidor.El servidor no ofrece…» | ✅ |
| **H-05** | **No se podía dar de alta ni un contacto.** `Canal` cruzaba la frontera como `correo`/`whats_app` y la base lo llama `email`/`whatsapp`: un enum con dos nombres, y la junta era justo la frontera | ✅ |
| **H-06** | El horario de envío salía vacío y **ninguna pantalla podía crear uno** | ✅ |
| **R-01** | Dar de alta un correo era «demasiado complicado»: siete campos, tres técnicos | ✅ dos campos |
| **R-03** | Negrita y cursiva en las plantillas | ✅ |
| **R-04** | La firma no puede llevar datos de quien recibe | ✅ |
| **R-06** | Borrar plantillas y firmas sin perderlas: se retiran, no se borran | ✅ |
| **R-09** | Editar y dar de baja desde la fila del contacto, con iconos | ✅ |
| **R-05** | **Imagen en la firma** | ⬜ espera decisión |

**Y la auditoría de funcionalidad** (22/10/2026, `09-fases/AUDITORIA-G-FUNCIONALIDAD.md`): **siete hallazgos, siete arreglados**, con las diez fases en verde al empezar. El más grave y el séptimo los encontró Dirección, no la medición:

- **El límite diario se escribía y no se aplicaba.** Se escribía 300, se leía el aviso de riesgo, se marcaba la casilla — **y salían 50**, porque el motor tomaba el menor entre la campaña y la cuenta, y toda cuenta nacía con 50 escritos en el código. **Peor que faltar la función: dejaba creer que se usó.**
- **Dos campañas con el mismo remitente se repartían el cupo** —medido: 50 y 50 daban **25 y 25**, por orden de llegada—, así que el plazo del paso 6 no se cumplía. **Resuelto esa misma tarde con la D-13**: los topes se suman, las dos sacan 50, y el aviso del paso 6 pasó a avisar de lo otro — que de ese buzón salen 100 al día y el agregado ya no tiene techo.

---

## 12. El sistema de diseño, al día y comprobado · 27/10/2026

Dirección pidió el 23/10 un sistema de diseño. Existía
—[`DESIGN_SYSTEM.md`](../05-diseno/DESIGN_SYSTEM.md), de la Fase 2— y estaba
desalineado: documentaba **24 de los 64 tokens** que no son color, **5 de las 16
piezas**, y no mencionaba el tema claro, con los 25 colores llevando su valor
claro desde la Fase 3 y una sonda midiéndolo.

**Reescrito con el código delante, tenía cinco afirmaciones falsas.** Las cinco
desde la Fase 2, y **ninguna de las cuatro auditorías del §140 las vio**: una
auditoría mide el producto, y esto es un documento. Están en el registro de
requisitos como DOC-1 a DOC-5, y en el §14 del propio documento con lo que
habría costado cada una.

La peor de las cinco no es la más vistosa: el documento daba el anillo de foco
como `--arles-accent`, el oro de relleno. Quien copiara ese fragmento tendría
**1.32:1** en el tema claro, que es un anillo de foco invisible, que es la
navegación por teclado entera.

**El poka-yoke es la mitad del trabajo.** Ese documento es una lista escrita a
mano, y el patrón número 3 de este proyecto dice qué les pasa a las listas a
mano: once de los diecisiete hallazgos de las cuatro auditorías fueron eso. Así
que `el_sistema_de_diseno_dice_lo_que_hay`, en `--fase 2`, mide cuatro cosas
leyendo el código:

1. Todo token que el documento nombra existe en los dos CSS.
2. Todo token que existe está nombrado en el documento.
3. Toda pieza de `componentes/` está nombrada, **y toda pieza nombrada existe** —
   las dos direcciones, porque son dos defectos distintos.
4. Las cifras del documento son las que hay: 89 tokens, 25 colores, 39 contratos,
   24 iconos.

Verificada rompiéndola por los **cinco** caminos: un token inventado, un token
que deja de estar documentado, una pieza fantasma, una pieza de la carpeta sin
documentar, y una cifra que se queda atrás. Las cinco en rojo, y el documento
restaurado desde una copia hecha **antes** de la primera ruptura — comprobado con
`diff`, no supuesto.

> **Y el décimo comentario que se caza a sí mismo**, en el peor sitio posible: la
> primera versión del documento escribía el nombre de un token inventado para
> ilustrar que un nombre mal escrito no da error, **dentro de la misma sección
> que advierte de esta trampa** y en el documento cuya comprobación nueva lee
> precisamente esos nombres. Lo cazó medir antes de escribir la comprobación.
>
> Por eso el documento tiene ahora una regla propia en su §0: **ahí no se escribe
> el nombre de un token ni de una pieza que no exista, ni como contraejemplo.**

> **Y un hallazgo falso dentro de mi propio instrumento, cazado antes de que
> entrara.** La expresión que reconoce el nombre de una pieza exige minúscula en
> la tercera letra. Sin eso casa **cinco siglas** del documento —`AA`, `ADR`,
> `ANTES`, `ARIA`, `ARLES`— y la comprobación habría salido en rojo acusando al
> documento de nombrar cinco componentes inexistentes. El comentario que lo
> explica decía «tres, entre ellas `ARCO`»: tres en vez de cinco, y una que el
> documento no menciona. **Se corrigió midiéndolo**, que es la diferencia entre
> un comentario y un dato.
