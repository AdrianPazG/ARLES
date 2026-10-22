# ARLES RELAY

**Software de ejecución y gestión controlada de campañas de correo y de WhatsApp.**

Aplicación de escritorio para Windows y macOS · Versión objetivo **1.2.0**
Desarrollado por **TELEMETRY INSIGHT**

---

<!-- AVANCE:INICIO -->
![Avance de la v1.2.0](https://img.shields.io/badge/avance_v1.2.0-89%25-brightgreen?style=flat-square)

**89 % de la versión 1.2.0 construido y verificado**, a 2026-10-22.

<details>
<summary>Cómo sale ese número, y qué no mide</summary>

Suma de **peso × hecho** sobre las fases de abajo. Los **pesos** son una
estimación del tamaño relativo de cada fase y suman 100; se dice que son una
estimación porque presentarlos como medición exacta sería mentir. Lo que **no**
es estimación es la fracción hecha: cada una se apoya en algo comprobable, y esa
comprobación está escrita en
[`avance.json`](documentacion/07-entrega/avance.json).

**No mide si el producto sirve**, sino cuánto del alcance de la v1.2.0 está
construido *y verificado*. Los bloqueos externos no restan porcentaje, porque no
son trabajo pendiente nuestro: van aparte.

| Fase | Peso | Hecho |
|---|---|---|
| F0 · Discovery y auditoría | 5 % | ✅ 100 % |
| F1 · Cimientos | 9 % | ✅ 100 % |
| F2 · Design System | 9 % | ✅ 100 % |
| F3 · Empresa y contactos | 14 % | ✅ 100 % |
| F3W · Canales: esquema de contactos, consentimiento y etapas | 8 % | 🟨 55 % |
| F4 · Motor de ejecución | 14 % | ✅ 100 % |
| F5 · Proveedores de envío | 7 % | 🟨 70 % |
| F6 · Campañas y mensajes | 13 % | ✅ 100 % |
| F7 · Actividad y entregabilidad | 9 % | 🟨 92 % |
| F8 · Respaldos y endurecimiento | 7 % | ✅ 100 % |
| F9 · Release v1.2.0 | 5 % | ⬜ 0 % |

**Esperando a alguien de fuera:**

| | Qué | Quién | Qué frena |
|---|---|---|---|
| **P-01** | Licencia de Mont para incrustarla en la aplicación distribuida | Dirección | El primer instalador descargable. Hay plan B: tipografía del sistema, una línea de CSS |
| **P-09** | Revisión jurídica: dieciocho preguntas en 08-legal/CONSULTA-JURIDICA.md | Abogado | Ya NO bloquea la construcción (ADR-0013). Frena los textos legales visibles y el primer envío real a alguien de fuera de TELEMETRY. Incluye P-17: consent_entry es de sólo-añadir y guarda la dirección en claro, lo que contradice el derecho de cancelación. |
| **META** | Cuenta de empresa en Meta, verificada y con Coexistencia activada (L-11) | Operaciones | El alta del número de WhatsApp contra una cuenta real. El adaptador de Cloud API, su servidor de mentira, la pantalla y el envío de prueba están construidos (canal W), pero nunca han hablado con Meta. Y por debajo hay un límite mayor que no depende de Meta: el motor sólo recorre Canal::Correo, así que hoy no se puede lanzar una campaña entera por WhatsApp — sólo el envío de prueba. |
| **R-07** | Revisión visual en macOS (WKWebView) | Quien tenga un Mac | Nada de la construcción, y por eso lleva abierto desde la Fase 2. La fecha que decía «antes de la F6» caducó: la F6, el canal W, la F7 y la F8 se cerraron sin que nadie abriera ARLES en WKWebView. Lo que frena es poder AFIRMAR que las dieciocho pantallas se ven bien en macOS: todo lo medido —contraste, foco, objetivos, textos encimados— sale de Chromium, y la vista previa no cuenta porque ahí el motor es el del navegador. Se cierra antes de la F9. |

**Ya no frenan nada** (se dejan escritos: un bloqueo que desaparece parece uno que nadie resolvió):

| | Qué era | Cerrado el | Cómo se cerró |
|---|---|---|---|
| **R-24** | Probar ARLES contra un servidor de correo real, con una cuenta desechable | 2026-10-22 | Dirección dio de alta enlace@telemetrymx.com (Titan, SMTP real con TLS) y ARLES mandó un correo de prueba a tres destinos: Outlook, Gmail y un corporativo. Los tres llegaron a la BANDEJA PRINCIPAL. Antes, desde una dirección @gmail.com, Outlook lo mandaba a no deseados con SCL 5 — la autenticación pasaba (spf, dkim, dmarc y compauth) y lo que pesaba era el patrón «marca + gmail personal», que no se arregla con código. |

El número se genera; no se escribe a mano en dos sitios:

```bash
python3 herramientas/avance/calcular.py --escribir
```

</details>
<!-- AVANCE:FIN -->

---

## Estado

**Al 08/10/2026.** Cada fase cerrada tiene su documento técnico y su resumen
para Dirección, sin tecnicismos.

| Fase | | Validación |
|---|---|---|
| [00 · Discovery y auditoría](documentacion/09-fases/FASE-00-DISCOVERY.md) | ✅ cerrada | 5/5 |
| [01 · Cimientos](documentacion/09-fases/FASE-01-CIMIENTOS.md) · [↳ Dirección](documentacion/09-fases/FASE-01-PARA-DIRECCION.md) | ✅ cerrada y revisada | 53/53 |
| [02 · Design System](documentacion/09-fases/FASE-02-DESIGN-SYSTEM.md) · [↳ Dirección](documentacion/09-fases/FASE-02-PARA-DIRECCION.md) | ✅ cerrada y revisada | 37/37 |
| [03 · Empresa y contactos](documentacion/09-fases/FASE-03-EMPRESA-Y-CONTACTOS.md) · [↳ Dirección](documentacion/09-fases/FASE-03-PARA-DIRECCION.md) | ✅ cerrada | 84/84 |
| [04 · Motor de ejecución](documentacion/09-fases/FASE-04-MOTOR-DE-EJECUCION.md) · [↳ Dirección](documentacion/09-fases/FASE-04-PARA-DIRECCION.md) | ✅ cerrada | 41/41 |
| [05 · Proveedores de envío](documentacion/09-fases/FASE-05-PROVEEDORES-DE-ENVIO.md) · [↳ Dirección](documentacion/09-fases/FASE-05-PARA-DIRECCION.md) | 🟨 la parte de correo cerrada | 36/36 |
| [06 · Campañas y mensajes](documentacion/09-fases/FASE-06-CAMPANAS-Y-MENSAJES.md) · [↳ Dirección](documentacion/09-fases/FASE-06-PARA-DIRECCION.md) | ✅ cerrada | 20/20 |
| [Canal W · WhatsApp](documentacion/09-fases/FASE-W-CANAL-WHATSAPP.md) · [↳ Dirección](documentacion/09-fases/FASE-W-PARA-DIRECCION.md) | 🟨 manda mensajes de prueba, **no campañas** | 21/21 |
| [07 · Actividad](documentacion/09-fases/FASE-07-ACTIVIDAD.md) · [↳ Dirección](documentacion/09-fases/FASE-07-PARA-DIRECCION.md) | 🟨 92 % · falta sólo SPF/DKIM/DMARC | 9/9 |
| [08 · Respaldos y endurecimiento](documentacion/09-fases/FASE-08-RESPALDOS.md) · [↳ Dirección](documentacion/09-fases/FASE-08-PARA-DIRECCION.md) | ✅ cerrada | 19/19 |
| 09 · Release v1.2.0 | ⬜ sin empezar | — |

**325 comprobaciones, 0 fallos y 0 omitidas** (13/10/2026), con las sondas
abriendo un navegador de verdad. Una sonda que no se puede correr se marca
**omitida**, que no es lo mismo que pasada, y así se dice.

> **Y el denominador tampoco es fijo, que es la parte fácil de leer mal.** El
> 13/10/2026 esta misma fase 1 dio «45 pasan, 0 fallan» **de 49** en un
> contenedor recién creado: sin `node_modules` ni llavero, cuatro comprobaciones
> se marcaron omitidas y **el total encogió**. Nada estaba roto y nada salía en
> rojo. Con las dependencias puestas son **53 de 53**.
>
> Por eso la cifra se lee entera —pasan, fallan, omitidas **y total**— y nunca
> sólo el «0 fallan».

```bash
# Las diez fases. Correr sólo la fase en curso no basta: en la 5.2 y la 5.3
# se rompieron cuatro comprobaciones de las fases 2, 3 y 4 sin que nadie lo
# viera.
for f in 0 1 2 3 4 5 6 W 7 8; do python3 herramientas/validar/validar.py --fase $f; done
```

👉 **[Empieza por la documentación](documentacion/00-INDICE.md)**

---

## Ver ARLES funcionando, sin instalar nada

**Hay un archivo que abre la aplicación entera en el navegador, con datos de
ejemplo.** Un solo HTML de 798 kB, sin instalador, sin conexión y sin dejar nada
en el equipo: se abre con doble clic y se puede mandar por correo a quien tenga
que verlo.

**No está en el repositorio, y es deliberado**: son 798 kB generados que
`.gitignore` deja fuera. Se produce con un comando y se manda a quien tenga que
verlo:

```bash
python3 herramientas/vista-previa/generar.py   # deja vista-previa/ARLES-vista-previa.html
```

> Hasta el 13/10/2026 esta sección enlazaba el archivo como si estuviera aquí,
> con instrucciones de *Download raw file*. **Ese enlace nunca funcionó**, porque
> el archivo nunca estuvo — está ignorado a propósito desde que existe.

**Si lo que quieres es ARLES de verdad, no una maqueta**, hay instalador:
[`COMO-PROBAR-ARLES-SIN-COMPILAR.md`](documentacion/07-entrega/COMO-PROBAR-ARLES-SIN-COMPILAR.md).

### Qué se ve, y qué no

**Se recorren las dieciocho pantallas** con contactos, campañas, plantillas,
remitentes y un número de WhatsApp ya dados de alta. Se puede escribir una
plantilla, navegar el asistente de nueve pasos, confirmar Coexistencia y ver las
dos cifras de permiso antes de activar una campaña — y desde la 7.7, ver
Actividad con las cifras de cada campaña, el registro con sus filtros, y pulsar
el freno.

**No manda nada.** El motor que envía vive en Rust, detrás de la frontera de la
aplicación de escritorio, y ahí no hay nadie al otro lado: lo que responde es un
simulador que vive en
[`herramientas/vista-previa/simulador.js`](herramientas/vista-previa/simulador.js).
Nada de lo que se escriba se guarda de verdad — sobrevive a recargar la página y
desaparece al cerrar la pestaña.

**Los datos son inventados y se nota**: todas las direcciones llevan
`ejemplo.mx`.

### Por qué no hay un enlace público

Porque publicarlo en internet distribuiría la tipografía **Mont**, y eso es lo
que la licencia no cubre mientras P-01 siga abierta (el mismo motivo por el que
no hay instalador todavía). Este archivo **sí** la lleva, porque es interno —
equivale a mirar el entorno de desarrollo del equipo.

Si algún día hace falta un enlace abierto, se genera sin ella:

```bash
python3 herramientas/vista-previa/generar.py --sin-mont
```

### Cómo se regenera

```bash
python3 herramientas/vista-previa/generar.py
```

---

## Abrir ARLES de verdad, en tu equipo

La vista previa de arriba es HTML en un navegador: **no hay nada de Rust
detrás**, así que no guarda, no cifra y no manda. Para abrir la aplicación de
verdad —con su base cifrada y su motor— hacen falta Rust y Node, y dos comandos:

```powershell
cd app
npm install
npm run build
cd ..
cargo arles
```

📘 **[`COMO-ABRIR-ARLES.md`](documentacion/07-entrega/COMO-ABRIR-ARLES.md)** · qué
instalar, qué pasa la primera vez, y **los cuatro tropiezos que ya se han visto
en Windows** con su salida: PowerShell bloqueando `npm`, una base de una versión
anterior, «cargo no se reconoce» tras instalar Rust, y la ventana que enseña
«localhost refused to connect».

> **`cargo arles`, no `cargo run --release`.** El alias añade la característica
> `custom-protocol`, que es lo que mete la interfaz **dentro** del ejecutable.
> Sin ella la aplicación compila, arranca, abre su ventana — y dentro no hay
> nada. Estuvo nueve fases así porque nadie la había construido.

---

## Descargar ARLES

**Todavía no hay ninguna versión descargable, y conviene decir por qué en vez de
dejar el apartado vacío.**

Cuando la haya, estará en **[Releases](https://github.com/AdrianPazG/ARLES/releases)** — un `.exe` para
Windows y un `.dmg` para macOS. Ése es el sitio, y no un archivo suelto en el
repositorio: un instalador dentro del árbol de código no se puede firmar, no
lleva número de versión asociado y nadie sabe cuál es el bueno.

Para que exista el primero hacen falta tres cosas, y **ninguna es programar**:

| | Qué | Quién | Sin esto pasa que… |
|---|---|---|---|
| **1** | **Licencia de Mont** para incrustarla en una aplicación distribuida (P-01) | Dirección | Repartir el instalador con la tipografía dentro infringe la licencia. Hay un plan B: una línea de CSS y ARLES usa la tipografía del sistema |
| **2** | **Certificado de firma de Windows** | Operaciones | Windows enseña la advertencia de «editor desconocido» y SmartScreen bloquea la instalación |
| **3** | **Apple Developer ID** y notarización | Operaciones | macOS se niega a abrir la aplicación |

### Mientras tanto: la versión de prueba

Se puede construir **hoy**, sin firmar y sin Mont, para revisar la aplicación de
verdad en lugar de con capturas. Hay dos caminos:

📦 **[`COMO-PROBAR-ARLES-SIN-COMPILAR.md`](documentacion/07-entrega/COMO-PROBAR-ARLES-SIN-COMPILAR.md)**
· descargar el instalador de Releases y abrirlo con doble clic. **No hace falta
Visual Studio, ni VS Code, ni Rust, ni Node.** Incluye **qué borrar y en qué
orden** si ya habías instalado ARLES antes — hacerlo al revés deja datos
ilegibles para siempre.

🚧 **[`REGISTRO-DE-TROPIEZOS.md`](documentacion/07-entrega/REGISTRO-DE-TROPIEZOS.md)**
· si algo te para, empieza aquí: **los siete tropiezos que ya han ocurrido**, con
su causa, su salida y la regla que salió de cada uno. Ninguno fue un fallo del
producto: fueron instrucciones mal escritas y cosas que el programa sabía y no
decía.

🔧 **[`COMO-ABRIR-ARLES.md`](documentacion/07-entrega/COMO-ABRIR-ARLES.md)** ·
compilarlo en tu propio equipo, con Mont y sin esperar al servidor.

### Dónde se descarga

En **[Releases](https://github.com/AdrianPazG/ARLES/releases)** — el enlace de
la derecha de la portada, o el de aquí. **No en la pestaña Actions**: ahí sólo
se ve cómo se construyó.

Cada versión de prueba trae dos archivos:

| Archivo | Para | Cómo se abre |
|---|---|---|
| `ARLES RELAY_1.2.0_x64-setup.exe` | **Windows** | Doble clic → Windows dirá «editor desconocido» → **Más información** → *Ejecutar de todas formas* |
| `ARLES RELAY_1.2.0_aarch64.dmg` | **macOS** | Doble clic → arrastrar a Aplicaciones → la **primera vez**, clic derecho sobre la app → *Abrir* |

> ⚠️ Esos dos avisos salen **porque la versión no está firmada**, no porque algo
> vaya mal. Es el punto 2 de la tabla de arriba.

### Cómo se genera una nueva

**Creando una etiqueta que empiece por `prueba-`.** Eso lanza la compilación y
publica la release al terminar:

```bash
git tag prueba-2026-09-18
git push origin prueba-2026-09-18
```

> 🔵 **Por qué una etiqueta y no el botón de Actions.** GitHub sólo enseña el
> botón *Run workflow* cuando el archivo del flujo está en la rama **por
> defecto** (`main`), y el trabajo vive en una rama de desarrollo. El botón
> aparecerá solo el día que esto llegue a `main`; mientras tanto, la etiqueta
> hace lo mismo.

> ⚠️ **Nunca se dispara sola.** No hay compilación en cada push: una versión sin
> firmar que se publica sola acaba instalada donde nadie la pidió, y sin firma
> quien la instale no puede comprobar de dónde salió.

Lo que se verá distinto en ella: **la tipografía**, porque va sin Mont. Los
tamaños, los pesos y el ritmo sí son los definitivos — salen de los tokens. Un
hallazgo del tipo «la letra no es la de la marca» en esa versión es esperado.

**Qué se puede hacer ya con una versión de prueba** (actualizado el 08/10/2026 —
este párrafo decía «la aplicación todavía no envía correos ni carga contactos», y
llevaba meses sin ser cierto):

- Configurar la empresa, **cargar contactos** desde CSV o XLSX con sus choques
  revisados antes de importar, y gestionar bajas y derechos ARCO.
- Dar de alta una cuenta de correo y **mandarse un correo de prueba de verdad**.
- Escribir plantillas y firmas, crear una campaña con el asistente de nueve
  pasos y **activarla**.
- Verla enviar en Actividad, con sus cifras y **por qué no sale nada**, y
  **pararla** en cualquier momento.
- Dar de alta un número de WhatsApp y **mandarse un WhatsApp de prueba**.
- **Hacer un respaldo cifrado de todo, restaurarlo en otro equipo**, archivar el
  historial viejo a un disco externo, y recuperar la base el día que Windows
  pierda la llave.

**Lo que todavía no:** una campaña entera por WhatsApp (el motor sólo recorre el
correo), filtrar la audiencia, y editar una campaña ya creada.

---

## Qué es

ARLES permite a una empresa ejecutar campañas de **correo y de WhatsApp** usando **sus propias cuentas** y **sus propios límites**.

WhatsApp entró en el alcance de la v1.2.0 el 17/09/2026. **Dos canales y sólo dos**: cada envío es de un canal, las métricas no se suman entre canales y el permiso se registra por canal.

Lo que define el producto no es enviar correo —eso lo hace cualquier cosa— sino **el control sobre el envío**:

- Límites que el cliente fija, no que le imponemos
- Una simulación que dice cuánto tardará la campaña de verdad, antes de activarla

- Un motor que respeta esos límites incluso cuando el equipo se suspende y despierta
- Idempotencia que garantiza que nadie reciba dos veces el mismo correo
- Métricas que no mienten

### Los tres principios

**Honestidad.** Si el proveedor sólo confirmó «aceptado», la interfaz dice *aceptado*. Nunca «entregado». Y si el producto no sabe algo, lo declara.

> **Y el principio se aplica a esta misma página.** «Límites que el cliente
> fija» estuvo escrito aquí durante semanas mientras el producto **no lo
> cumplía**: el campo del límite diario existía, con su aviso y su casilla, y el
> motor aplicaba el menor entre lo escrito y un 50 fijo en el código. Se
> escribía 300, se aceptaba el riesgo, y salían 50 sin que nada lo dijera. Lo
> encontró Dirección usando ARLES el 22/10/2026 y está corregido — pero la línea
> de arriba era una promesa, no una descripción, y conviene saber distinguirlas.
> Está contado en
> [AUDITORIA-G](documentacion/09-fases/AUDITORIA-G-FUNCIONALIDAD.md).

**Seguridad sobre comodidad.** Ante la duda entre un flujo cómodo y uno seguro, gana el seguro, y se documenta el coste.

**Autonomía informada.** ARLES advierte y explica; no dicta la lógica de negocio del cliente. Pero nunca facilita la evasión de límites.

### Qué no es

Mailchimp · un CRM · un ERP · una plataforma omnicanal · una herramienta de scraping · un evasor de filtros antispam · un sistema para saltarse los límites de Gmail.

---

## Documentación

| | |
|---|---|
| [📍 Estado actual](documentacion/07-entrega/ESTADO_ACTUAL.md) | **Empieza aquí si retomas el trabajo**: qué hay construido, qué no hay que volver a discutir y qué sigue |
| [📋 Índice completo](documentacion/00-INDICE.md) | Mapa de lectura y trazabilidad del brief |
| [🔍 Auditoría de discovery](documentacion/02-auditoria/AUDITORIA_DISCOVERY.md) | El entregable A–U, con lo que el repositorio de referencia no era |
| [⚠️ Preguntas abiertas](documentacion/02-auditoria/PREGUNTAS_ABIERTAS.md) | Lo que bloquea el avance |
| [🏗️ Arquitectura](documentacion/03-arquitectura/ARQUITECTURA.md) | Monolito modular en Rust + Tauri 2 |
| [⚙️ Motor de ejecución](documentacion/03-arquitectura/MOTOR_DE_EJECUCION.md) | El corazón del producto |
| [🔒 Modelo de amenazas](documentacion/04-seguridad/THREAT_MODEL.md) | STRIDE por dominio |
| [🎨 Sistema de color](documentacion/05-diseno/COLOR_SYSTEM.md) | Paleta medida, contraste calculado |
| [🗺️ Roadmap](documentacion/07-entrega/ROADMAP.md) | Las fases con sus dependencias reales |
| [🖥️ Mapa de pantallas](documentacion/05-diseno/MAPA-DE-PANTALLAS.md) | Qué archivo dibuja cada pantalla y qué comprobación salta si se rompe |
| [🔌 Guía de conexión real](documentacion/07-entrega/GUIA-DE-CONEXION-REAL.md) | Cómo hacer una demostración con correo y WhatsApp que llegan de verdad |
| [📘 Cómo abrir ARLES](documentacion/07-entrega/COMO-ABRIR-ARLES.md) | Qué instalar, los dos comandos, y los cuatro tropiezos ya vistos en Windows |
| [📦 Probar ARLES sin compilar](documentacion/07-entrega/COMO-PROBAR-ARLES-SIN-COMPILAR.md) | Descargar el instalador, y qué borrar —y en qué orden— si ya lo tenías |
| [🚧 Registro de tropiezos](documentacion/07-entrega/REGISTRO-DE-TROPIEZOS.md) | **Si algo te para, empieza aquí.** Los siete que ya han ocurrido, con su causa y su salida |

---

## Stack previsto

| Capa | Tecnología | Decisión |
|---|---|---|
| Shell de escritorio | Tauri 2 | [ADR-0001](documentacion/03-arquitectura/adr/0001-tauri-2-sobre-electron.md) |
| Núcleo | Rust | T-10 |
| Base de datos | SQLite + SQLCipher | [ADR-0002](documentacion/03-arquitectura/adr/0002-rusqlite-sqlcipher-refinery.md) |
| Interfaz | Vue 3 · TypeScript estricto · Vite | |
| Estado y tablas | Pinia · TanStack Virtual | [ADR-0007](documentacion/03-arquitectura/adr/0007-pinia-y-tanstack-virtual.md) |
| Pruebas | cargo test · Vitest · Playwright | |

---

## En seguimiento

**Nada impide construir. Lo de arriba —«esperando a alguien de fuera»— es lo que
impide *entregar* la v1.2.0**, y no es trabajo nuestro: por eso no resta
porcentaje y va aparte.

Lo demás que sigue abierto, con quién lo cierra:

| # | Asunto | Quién |
|---|---|---|
| 🟡 **P-03** | Origen y consentimiento de los contactos cargados | Dirección |
| 🟡 **P-05** | Dominio y aviso de privacidad para la verificación de Google | Dirección · sólo hace falta el día que se use OAuth de Google (ADR-0003); con SMTP no |
| 🟡 **P-17** | `consent_entry` es de sólo-añadir y guarda la dirección en claro, lo que roza el derecho de cancelación | Abogado · va dentro de P-09 |
| 🟡 **D-WA** | ¿Se construye la campaña entera por WhatsApp, o la v1.2.0 ofrece sólo correo? | Dirección · planteado en [FASE-05-PARA-DIRECCION](documentacion/09-fases/FASE-05-PARA-DIRECCION.md) |
| 🟡 **R-05** | Imagen en la firma del correo: ¿embebida —engorda cada envío y dispara filtros— o alojada, que hay que alojarla en algún sitio? El saneado del HTML tendría que admitir `<img>` con sus reglas | Dirección · pedido el 21/10/2026 |
| 🟡 **D-REM** | Dos campañas con el mismo remitente se reparten su cupo diario (medido: 50 y 50 dan **25 y 25**, por orden de llegada). Hoy **se avisa** en el paso 6; ¿se prefiere **impedirlo**? | Dirección · planteado el 22/10/2026 en [AUDITORIA-G](documentacion/09-fases/AUDITORIA-G-FUNCIONALIDAD.md) |

Detalle en [PREGUNTAS_ABIERTAS](documentacion/02-auditoria/PREGUNTAS_ABIERTAS.md).

---

## Carpetas de referencia

`/RECURSOS`, `/REFERENCIA_DE_COLOR`, `/CONCEPTOS_DE_DISEÑO`, `/REFERENCIAS_VISUALES_DEL_SITIO` y `/TIPOGRAFIA` son **material de referencia de sólo lectura** (§11).

⚠️ Su contenido **no es lo que el brief asume**. Antes de usarlos, lee el [inventario forense](documentacion/02-auditoria/INVENTARIO_DE_ASSETS.md): la referencia cromática es un PNG generado por IA, los «conceptos de diseño» son consejos de UX de redes sociales, y no hay ningún logotipo de ARLES.

---

## Versionado

Versionado semántico `MAYOR.MENOR.PARCHE`. Ver [CHANGELOG.md](CHANGELOG.md).

El numeral «I» del nombre comercial **nunca aparece adyacente al número de versión** ([ADR-0010](documentacion/03-arquitectura/adr/0010-numeral-romano-i.md)).

---

*Software desarrollado por TELEMETRY INSIGHT*
