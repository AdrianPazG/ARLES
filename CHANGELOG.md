# Changelog

Todos los cambios notables de ARLES RELAY I se documentan aquí.

Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).
Versionado según [Versionado Semántico](https://semver.org/lang/es/) (§3).

> **Nomenclatura prohibida** en cualquier artefacto: FINAL, FINAL2, DEFINITIVO, NUEVO.
> Toda versión se identifica por su número semántico.

---

## [Sin publicar]

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
