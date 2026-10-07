# Auditoría C · QA y rendimiento (§140)

**Fecha:** 07/10/2026 · **Entrega:** 8.6 C · **Estado al empezar:** las diez
fases del validador en verde.

Tercera de las cuatro pasadas separadas. `ESTRATEGIA_QA.md` §5 le fija tres
cosas, y se hicieron las tres:

> Casos frontera de §3 cubiertos · presupuestos de `PRESUPUESTO_RENDIMIENTO.md`
> **medidos, no estimados** · **sin fugas de memoria en ejecuciones largas del
> motor**

---

## 0 · El resultado, en una tabla

| | Qué se auditó | Veredicto |
|---|---|---|
| 1 | Los 32 casos frontera del §3, uno a uno | ⚠️ **2 sin test, 3 describían caminos inexistentes** |
| 2 | Las trece cifras del presupuesto | ❌ **dos presupuestos en rojo, y tres «sin medir» que sí se podían medir** |
| 3 | Fugas de memoria en corrida larga del motor | ✅ **plana: 0.00 MB en 20 000 envíos** — y nunca se había medido |
| 4 | Lo que el §3 del presupuesto dice de la arquitectura | ❌ **nombra Tokio, que el proyecto no usa** |

**Seis hallazgos. Cuatro arreglados, dos declarados con su cifra** — uno de los
dos es una decisión de Dirección y está planteado abajo con su trade-off.

---

## 1 · Los hallazgos

### C-1 · Activar una campaña tarda 73.5 s de un presupuesto de 2 s, y congela la interfaz

**Qué.** Medido el 07/10/2026 sobre 451 196 destinatarios reales del conjunto
sintético: **73.5 s**. El presupuesto es de 2 s. Son 36 veces.

**Por qué es el peor de los seis.** `activar_campana` corre dentro de un comando
de Tauri, en una sola transacción, con el cerrojo de la conexión tomado.
Mientras dura:

- la interfaz **no contesta** —es una llamada síncrona—,
- no hay barra de avance ni cifra: el §94 pide cifras y aquí no hay ninguna,
- no se puede **cancelar**,
- y el motor no puede escribir, porque el cerrojo es uno solo.

`PRESUPUESTO_RENDIMIENTO.md` §7 lo tenía escrito como señal de alarma desde la
Fase 0 — *«la interfaz se congela al activar una campaña → trabajo en el hilo
equivocado»*— y nadie lo había medido.

**Y no es sólo el techo.** La curva, medida a cuatro escalas:

| Destinatarios | Activar |
|---|---|
| 18 121 | 1.2 s |
| 45 253 | 3.8 s |
| 135 588 | 16.7 s |
| 451 196 | 73.5 s |

**El presupuesto se rompe alrededor de los 30 000 destinatarios.** Eso no es el
medio millón del techo arquitectónico: es una lista de tamaño ordinario.

**La causa medida, que no es la que parecía.** La hipótesis obvia era el bucle de
UUID: 451 196 filas × 2 identificadores, cada uno pidiendo entropía al sistema.
Se midió aparte: **0.58 s, el 0.8 %**. Lo que cuesta es escribir ~902 000 filas
con sus índices sobre SQLCipher, que es el mismo orden que la importación
(~1.35 M filas en 111 s).

Decirlo importa: con la hipótesis sin medir se habría «optimizado» la generación
de identificadores —tocando ADR-0004, la parte más delicada del producto— para
ganar medio segundo de setenta y tres.

**No se arregla en esta pasada, y es deliberado.** Las cuatro salidas, con su
coste:

| Alternativa | Coste | Qué rompe |
|---|---|---|
| **A · Rechazar activar por encima de un tope** (p. ej. 30 000) | Horas | Nada técnico. Es una decisión de producto |
| **B · Sacarla al hilo del motor con avance** | Días | Nada, pero `activar_campana` deja de ser síncrona y la pantalla necesita estados nuevos |
| **C · Llenar la cola en tandas al vuelo**, no al activar | Semanas | Toca la audiencia congelada (§65) y ADR-0004. **La idempotencia depende de que la clave se escriba antes** |
| **D · Dejarlo y subir el presupuesto** | Cero | La honestidad de la tabla |

> **Problema** · Activar una campaña de más de ~30 000 personas congela ARLES
> entre 4 y 74 segundos, sin cifra, sin avance y sin poder cancelar.
>
> **Riesgo** · Quien lo vea pensará que ARLES se colgó, y lo cerrará a mitad. La
> transacción se deshace entera —eso funciona—, pero el usuario no lo sabe y
> volverá a intentarlo.
>
> **Impacto** · La acción central del producto. No hay forma de usar ARLES sin
> pasar por aquí.
>
> **Alternativa** · La A o la B. La C es la correcta a largo plazo y la que no
> cabe en esta fase.
>
> **Trade-off** · La A son horas y cierra el agujero hoy, a cambio de decirle a
> alguien «esta campaña es demasiado grande» — que además **es verdad**: a 50
> correos diarios (§48), 451 196 personas son **24 años** de campaña, y el
> simulador ya se lo enseña en el paso 8. La B es la buena de verdad y son días.
>
> **Recomendación** · **La A ahora y la B en la Fase 9.** Un tope que diga su
> cifra convierte un cuelgue en un mensaje, y el mensaje es correcto: una
> campaña que ARLES tardaría años en enviar no debería poder activarse sin que
> nadie lo diga.

---

### C-2 · El filtro del registro tarda 3 709 ms de 1 s, y la urgencia es otra

**Qué.** El filtro de texto de D-7C compara contra
`message_attempt.contact_address` con `LIKE '%…%'`, que ningún índice sirve.
Medido sobre 451 196 sucesos: **3 709 ms** de un presupuesto de 1 s.

**Pero la cifra sola engaña.** La curva:

| Sucesos | Filtrar |
|---|---|
| 18 121 | 137 ms |
| 45 253 | 486 ms |
| 135 588 | 1 003 ms |
| 451 196 | 3 709 ms |

Se rompe a los **~135 000 sucesos**. A 50 envíos diarios eso son **más de siete
años**, y el recorte bimestral de la 7.4 los quita mucho antes.

**Lo que sí lo hace alcanzable:** ese recorte **lo pulsa el usuario**, no un
reloj — por decisión de la 7.4, porque un desenlace de envío no se borra jamás
sin que alguien lo confirme. Quien no lo pulse en siete años llega aquí.

**Declarado, no arreglado**, con su número y su condición. La respuesta correcta
está escrita en el propio módulo desde la 7.4 y sigue siendo la correcta: **FTS5**,
no un índice sobre un `LIKE` que no lo puede aprovechar. No entra en esta fase.

> Decir que C-1 y C-2 «fallan» sin esta diferencia habría sido exacto y
> useless: uno es trabajo de esta semana y el otro es una nota con fecha.

---

### C-3 · Dos casos frontera llevaban desde la Fase 0 sin ningún test, y el documento afirmaba que lo tenían

**Qué.** `ESTRATEGIA_QA.md` §3 dice de su lista: *«Cada uno tiene un test con
nombre»*. Se buscaron los 32, uno a uno. **Dos no existían.**

**1 · «Disco lleno a mitad de importación → transacción revertida, sin estado a
medias».** Seguramente porque llenar un disco en una prueba no se puede. Pero
**provocar el error que daría, sí**: `PRAGMA max_page_count` hace que la
escritura que pase el tope falle con `SQLITE_FULL`, que es literalmente el
código de un disco lleno — el mismo error por el mismo camino.

Escrito: `un_disco_lleno_a_mitad_de_importacion_no_deja_nada_a_medias`.
Verificado **partiendo la atomicidad** —un `tx.commit()` colado entre el
registro del lote y el bucle de contactos—, no quitando el tope: lo que hay que
ver caer es la garantía, no el disparador. Sale en rojo por la aserción del
registro: *«quedó el registro de una importación que no ocurrió, y la pantalla
lo enseñaría como hecha»*.

**2 · «Archivo con BOM, UTF-16, CRLF mezclado → se detecta y se lee
correctamente».** Ninguna prueba, y es el caso de un Excel en español
guardando «Texto Unicode».

**Funcionaba.** Lo grave no es el defecto —no hay— es **de qué dependía**:
`decodificar` sólo mira a mano el BOM de UTF-8, y quien decodifica un UTF-16 es
`encoding_rs::Encoding::decode`, que **olfatea el BOM por su cuenta** y cambia
de codificación aunque se le haya pedido Windows-1252. Eso no lo decía ninguna
línea del módulo: su comentario llama «Windows-1252» a esa rama.

Una garantía viva por una cortesía de la biblioteca que nadie había escrito.
Cambiar esa rama a `decode_without_bom_handling` —el cambio natural para quien
lea el comentario y crea que el BOM ya se trató arriba— convierte cada
importación en UTF-16 en basura silenciosa: los nombres salen con un carácter
nulo intercalado, la columna del correo no valida, y **no falla nada**.

Escritas tres pruebas, y **verificado ese cambio: la de UTF-16 sale en rojo.**

> **Y de las otras dos hay que decir algo incómodo: no se pueden romper.** Se
> intentó, porque el §2 obliga. La del BOM de UTF-8 sigue en verde incluso
> quitando entera la rama que lo mira —lo quita el lector de `csv` por su
> cuenta, así que esa rama es **redundante hoy**—, y la de los saltos de línea
> sigue en verde sin el `.trim()`, porque `csv` ya trata `\r\n` como
> terminador.
>
> Las dos vigilan **a la biblioteca, no a ARLES**. Eso no las hace inútiles —si
> `csv` cambiara de comportamiento, las importaciones se romperían en silencio
> y éstas lo dirían— pero sí obliga a etiquetarlas, y así están en el archivo.

**Y la causa de raíz, arreglada:** la lista no nombraba sus tests, así que la
afirmación no se podía comprobar. Ahora cada fila lleva el nombre, y
`cada_caso_frontera_nombra_su_test` (fase 8) falla si alguno deja de existir.
Verificada de dos formas: renombrando `quien_pidio_la_baja_no_recibe_nada`
—falla nombrándolo— y quitando la columna «Su test» de una tabla, que falla por
nombrar 22 cuando el mínimo son 25.

> **Lo que esa comprobación NO hace, y está escrito en ella:** verifica que el
> test *existe*, no que *cubra el caso*. Para eso no hay comprobación
> automática — es lo que hace la auditoría, y por eso la auditoría existe.

---

### C-4 · Tres casos frontera describían caminos que el producto no tiene

Igual que los dos puntos del `THREAT_MODEL.md` §8 que corrigió la auditoría B el
día anterior: no se borran, quedan con su fecha y su condición de reactivación.
Un caso frontera que no se puede cumplir hace dudar de la lista entera.

| Caso | Por qué no aplica |
|---|---|
| **Exportar `=HYPERLINK(...)` → se antepone `'` en el CSV** | **No hay CSV.** Y en el `.txt` que sí hay, antepoper `'` al apellido de alguien **estropea el documento**: es un informe ARCO que una persona lee, nadie quita el apóstrofo |
| **Nombre de archivo `CON`, `PRN`, `NUL` en Windows** | El nombre de fuera **no toca el disco nunca**: `arles-db` guarda con el id del lote |
| **Token OAuth revocado** | **No hay OAuth en v1.2.0** (ADR-0003). El equivalente real —token de Meta, código 190— sí está cubierto |

**Y un cabo del primero, que estuve a punto de anotar como hallazgo y no lo es.**
`neutralizar_formula` existe en `arles-import`, está probada, y **no la llama
nadie**: a primera vista, el patrón del §9. Y la cabecera de
`arles-core/src/arco.rs` la nombra diciendo «**hay que usarla**», que leído así
es una instrucción que, aplicada al informe ARCO, sería un defecto —ese informe
es prosa que una persona lee, no una hoja de cálculo, y un apóstrofo delante de
un apellido lo estropea—.

**Pero la frase lleva su condición delante**, y la cité sin ella: *«**Si algún
día hace falta exportar en CSV**, la neutralización ya está escrita […] y hay que
usarla»*. Es exactamente lo correcto: una defensa dormida, con la condición que
la despierta escrita al lado — la misma forma que la auditoría B le dio a los dos
puntos del `THREAT_MODEL.md` que marcó `[–] no aplica`.

Así que no hay nada que arreglar aquí, y lo que queda escrito es la lección:
**citar media frase de un comentario produce un hallazgo falso**, que es el
error que la auditoría B ya cometió el día anterior buscando los comandos de
ARCO por nombres inventados. Dos pasadas seguidas, el mismo error, en la misma
dirección: dar por sabido lo que dice el código en vez de leerlo entero.

---

### C-5 · El `Retry-After` que se respeta «literalmente» no se lee de ninguna parte

**Qué.** El caso decía: *«Servidor devuelve 429 con `Retry-After` → se respeta
**literalmente**»*.

La mitad de abajo es verdad y está probada por partida doble: cuando el adaptador
devuelve `NoMeMandesNadaDurante(d)`, el motor espera exactamente `d`
(`cuando_el_proveedor_dice_cuanto_esperar_se_le_hace_caso`,
`el_tiempo_que_pide_el_proveedor_se_respeta_al_pie_de_la_letra`).

**Lo que no existe es la cabecera.** `ErrorDeMeta` lleva tres campos —código,
subcódigo y estado HTTP— y **ninguna cabecera de la respuesta se lee**: ni en
`cloud.rs` ni en `estado.rs`. Ante un 429 el adaptador espera los 60 s de
`ESPERA_POR_RITMO`, que es **una constante nuestra**. Y SMTP no tiene ninguna
cabecera equivalente.

**Declarado, no arreglado.** Leer la cabecera es media hora de trabajo, pero la
decisión de qué hacer con lo que diga no es técnica: si Meta pide cinco minutos
y nosotros esperamos sesenta segundos, se gasta un intento de los tres de D-12
para nada; si pide una hora y obedecemos, una campaña se para una hora por una
cabecera que puede venir mal. **Las esperas las fijó Dirección el 23/09/2026
(D-12)** y ésta es de la misma familia.

Lo que esta auditoría arregla es la afirmación: el caso del §3 dice ahora lo que
pasa de verdad, y apunta aquí.

---

### C-6 · El presupuesto describía una arquitectura con Tokio, y el proyecto no usa Tokio

**Qué.** `PRESUPUESTO_RENDIMIENTO.md` §3 decía:

> *«Corre en su propio conjunto de tareas de Tokio (§51). La base de datos vive
> en un hilo dedicado con canal (ADR-0002), lo que además elimina de raíz los
> errores `SQLITE_BUSY` en vez de gestionarlos con reintentos.»*

Tres cosas, y **ninguna describe lo que hay**:

- **No hay Tokio.** Ni en un `Cargo.toml` del taller. `Proveedor` es síncrono a
  propósito, y está escrito por qué: a cincuenta correos diarios la asincronía
  no aporta nada y complica cada prueba. El motor corre en un `std::thread`.
- **No hay hilo dedicado con canal.** Es un `Mutex<Connection>` dentro de `Db`.
  Era el diseño de la Fase 0, pensado cuando se barajaba SQLx.
- **Los reintentos no se eliminaron.** `busy_timeout(5 s)` sigue puesto en
  `conexion.rs`. El Mutex serializa dentro del proceso; entre procesos lo que
  hay es el **cerrojo de instancia**.

**Es el mismo defecto que la auditoría B encontró un día antes** en el
`THREAT_MODEL.md` §8, que pedía «la capa de redacción de `tracing`» sin que el
proyecto use `tracing`. **Dos documentos de la Fase 0 nombrando bibliotecas que
nunca se adoptaron**, una pasada de distancia. Lo que describen suena plausible,
nadie lo relee, y quien llega nuevo busca un `tokio::spawn` que no existe.

**Arreglado**, con lo que hay y medido, incluida la trampa del cerrojo no
reentrante que el CLAUDE.md ya documentaba y este documento no.

---

## 2 · Las tres cifras que estaban «sin medir» por un motivo falso

> **Un «sin medir» con un motivo falso es peor que un hueco**, porque cierra la
> pregunta. Un hueco se ve; un motivo escrito se cree.

| Cifra | El motivo que daba la 8.5 | Qué pasaba de verdad |
|---|---|---|
| Calcular audiencia de campaña | *«exige una campaña activa con su etapa y remitente: es un banco aparte»* | Una **molestia**, no un impedimento: cuatro `INSERT`. Y era la peor cifra del producto (C-1) |
| Filtro sobre 500 k | *«no existe todavía»* | Verdad de la **lista de contactos**; falso del producto: la 7.4 construyó seis filtros sobre el registro (C-2) |
| Memoria con motor activo | *«sin llavero no arranca la aplicación entera»* | Verdad de la **aplicación**, falso del **motor**, que corre contra un proveedor simulado desde la 4.3a |

**Los cuatro que quedan sin medir sí tienen un motivo que se sostiene**, y se
dejan: arranque en frío y desplazamiento a 60 fps necesitan una webview (R-07),
el preflight necesita los puertos 587/465/25 (R-24), y el tamaño del instalador
necesita el empaquetador y la licencia de Mont (P-01).

---

## 3 · La corrida larga del motor, que nunca se había hecho

**20 000 envíos consecutivos. 13.9 MB en los cuatro cuartos. 0.00 MB de
crecimiento.**

Es el tercer punto que `ESTRATEGIA_QA.md` §5 le pide a esta pasada, y hasta hoy
no se había intentado ni una vez.

**No se mide un número, se mide una forma.** Un proceso que envía veinte mil
correos **tiene** que crecer al principio: SQLite llena su caché de páginas, el
asignador pide arenas y no las devuelve. Ese crecimiento es correcto y se
detiene. Lo que el §7 del presupuesto llama fuga es otra cosa —«memoria que
**crece sin estabilizarse**»—, así que la corrida se parte en cuatro tramos y se
compara el último con el **segundo**. Comparar con el primero es el error obvio:
metería dentro todo el arranque y una corrida sana saldría en rojo.

**Y la comprobación se vio fallar, porque si no no está verificada (§2).** La
fuga se mete de verdad con `banco larga 20000 fuga`, que cambia el proveedor por
uno que **guarda cada pedido** — y no es un ejemplo inventado: es exactamente lo
que hace `ProveedorSimulado`, el del taller, y es la razón por la que la corrida
larga **no lo usa**. Un instrumento que acumula por cada envío acusaría al motor
de una fuga que es suya.

**De paso, dos cifras que no se habían medido:** el motor hace **13 ms por
envío** —unas 77 por segundo, contra los 50 **diarios** que el producto
permite—, y su memoria en reposo es de 13.9 MB de un presupuesto de 250.

**Es parcial y hay que decirlo:** son el motor y la base, sin la ventana de Tauri
ni la webview. Esa mitad es R-07.

---

## 4 · Lo que salió limpio, con lo que se midió

- **27 de los 32 casos frontera tienen su test**, y ahora el documento dice
  cuál. Los cinco restantes son los dos de C-3 (escritos hoy) y los tres de C-4
  (no aplican).
- **Los dos presupuestos que la 8.6 arregló siguen arreglados**: buscar por
  correo por debajo de 1 ms, abrir la tabla en 357 ms.
- **La atomicidad de la importación aguanta un `SQLITE_FULL`** a mitad: ni un
  contacto ni un registro de lote quedan detrás.
- **Las tres pruebas de planes de consulta de la 8.6 siguen en verde**: ninguna
  de las nueve consultas críticas volvió a hacer `SCAN`.
- **Las diez fases del validador**, con la comprobación nueva: en verde.

---

## 5 · Qué queda

- **D · UX y accesibilidad** — contraste AA calculado, foco visible, teclado,
  ningún estado sólo por color, `prefers-reduced-motion`, objetivos ≥32 px al
  200 %, los cuatro estados de cada pantalla (§97) y los errores con sus tres
  partes (§95).

**Límite declarado:** la pasada D no puede mirar las pantallas en WKWebView
desde aquí. Eso es **R-07**.

**Y una decisión esperando a Dirección:** C-1, el tope de activación. Está
planteada arriba con sus cuatro alternativas y su recomendación.
