# Changelog

Todos los cambios notables de ARLES RELAY I se documentan aquí.

Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).
Versionado según [Versionado Semántico](https://semver.org/lang/es/) (§3).

> **Nomenclatura prohibida** en cualquier artefacto: FINAL, FINAL2, DEFINITIVO, NUEVO.
> Toda versión se identifica por su número semántico.

---

## [Sin publicar]

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
