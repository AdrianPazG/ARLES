# Estado actual · dónde está ARLES y qué sigue

**Proyecto:** ARLES RELAY I · v1.2.0
**Última actualización:** 23 de septiembre de 2026
**Para qué sirve este documento:** que quien retome el trabajo —otra sesión, otra persona, otro equipo— sepa en una lectura qué está construido, qué decisiones ya están tomadas, qué **no** hay que volver a discutir y qué es lo siguiente.

> Se actualiza **al cerrar cada entrega**, no al empezarla. Un documento de estado que describe intenciones en vez de hechos es peor que no tenerlo: se lee como si fuera cierto.

---

## 1. En una frase

**ARLES ya manda correos.** La parte de correo de la Fase 5 está cerrada entera (5.1 a 5.6): el programa abre una conversación con un servidor, cifra, entrega un mensaje y lo deja anotado, y está ejercitado de punta a punta contra un servidor que habla cifrado. **Lo que falta de la Fase 5 es el adaptador de WhatsApp**, que no está empezado — hay una decisión pendiente de Dirección sobre cuándo. Lo siguiente en el plan es la **Fase 6 · Campañas y mensajes**.

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
| F6 a F9 | ⬜ sin empezar | Campañas, actividad, respaldos, release |

**Cifras del 23/09/2026:** 677 pruebas de Rust, 110 del frontend, validador de fase 5 en 22/22, `fmt`, `clippy`, `eslint` y `vue-tsc` limpios, `cargo deny` sin avisos.

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

Ninguno impide construir. Los cuatro primeros impiden **entregar** la v1.2.0.

| Qué | Quién | Qué bloquea exactamente |
|---|---|---|
| **P-01 · Licencia de Mont** | Dirección | Distribuir el instalador con la tipografía dentro. Plan B: tipografía del sistema |
| **P-09 · Revisión jurídica** | Abogado | Los textos legales visibles. **No** frena la construcción (ADR-0013) |
| **META · Cuenta con Coexistencia** | Operaciones | P-14 no se puede probar contra nada real |
| **R-07 · Revisión en macOS** | Quien tenga un Mac | Nadie ha abierto ARLES en WKWebView |
| **R-24 · Ningún servidor de correo real** | Dirección | Ver §7 |

---

## 7. R-24 · ARLES nunca ha hablado con un servidor de correo real

**Es el riesgo abierto más importante de la fase.** Todo lo de la Fase 5 está probado contra un servidor de mentira que habla el protocolo de verdad y que sabe fallar de seis maneras distintas. Eso cubre las ramas de error mucho mejor que una prueba manual contra Gmail — pero no cubre lo que sólo aparece con un servidor real: un saludo inesperado, una extensión que no habíamos visto, un límite de tasa, una peculiaridad del certificado.

**No se puede comprobar desde un contenedor de desarrollo**, y se midió: la política de red de la sesión bloquea los tres puertos de correo (587, 465 y 25 dan tiempo de espera agotado; el nombre sí resuelve). Aunque se abriera la red, faltaría lo esencial: **ARLES exige el llavero del sistema y no arranca sin él**, y un Linux sin escritorio no tiene ninguno.

**Cómo se cierra:** en la máquina de quien vaya a usar ARLES —Windows o macOS—, con una cuenta de Gmail con verificación en dos pasos y una **contraseña de aplicación**, y **con una cuenta desechable**, no la de trabajo: mientras el producto no esté cerrado, una credencial que se pueda tirar sin consecuencias vale más que la comodidad.

**Lo que se comprueba:** que la prueba de conexión da «la cuenta funciona», y —ya con 5.4 cerrada— que una campaña activada manda un correo que llega.

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

**Antes de empujar, correr las seis fases**, no sólo la que se está tocando.
Cuesta un minuto con `--rapido`.

**Y el patrón del comentario que se caza a sí mismo ya va por la tercera vez**
—la invariante del TLS en la 5.2, la del secreto en la webview en la 5.3, y ésta.
Cualquier comprobación nueva que busque un texto en el código tiene que quitar
los comentarios antes de mirar, porque la documentación de la regla contiene la
palabra que la regla prohíbe.

## 9. Lo que este documento no cubre

- **El porqué de cada decisión de producto** está en `01-producto/DECISIONES_DE_DIRECCION.md` y en los ADR de `03-arquitectura/adr/`.
- **El estado de cada fase con su cifra** está en `07-entrega/avance.json`, que es la fuente única del porcentaje.
- **Lo que pasó en cada entrega** está en `CHANGELOG.md`, escrito para que lo lea Dirección.
