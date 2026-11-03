# Registro de requisitos

**Última actualización:** 2026-10-28

**Este es el único sitio donde vive el estado de un requisito.** Los
identificadores se usaban dispersos en siete documentos, y un requisito que
aparece dos veces con dos estados distintos es peor que uno que falta.

## Cómo se lee

| Estado | Significa |
|---|---|
| **abierto** | Se pidió y no está hecho. Se puede trabajar |
| **bloqueado** | No depende de nosotros. La columna «quién» dice de quién |
| **espera decisión** | Hace falta que Dirección elija. La alternativa está planteada |
| **hecho** | Construido y verificado, con la entrega que lo cerró |
| **descartado** | Se decidió no hacerlo, y el motivo está escrito |

| Prefijo | Qué es |
|---|---|
| **D-** | Decisión de Dirección |
| **P-** | Pendiente de alguien de fuera |
| **R-** | Requisito o riesgo |
| **H-** | Hallazgo encontrado usando ARLES |
| **F-** | Fallo de diseño que ya costó datos |

> **Un requisito nuevo se anota aquí el día que se pide**, no el día que se
> construye. Lo que no está anotado se olvida: pasó con `contact_list`, que
> estuvo en el esquema desde la V1 con su motivo escrito en «pendientes
> conocidos» y sin escritor hasta la D-15, dos años después.

---

## Lo que impide entregar la v1.2.0

Ninguno impide construir. **Los seis impiden entregar**, y dos de ellos tienen
plazo externo que no se comprime.

| Id | Qué | Quién | Estado | Qué bloquea exactamente |
|---|---|---|---|---|
| **P-01** | Licencia de la tipografía Mont | Dirección | bloqueado | **No se puede distribuir un instalador** con la tipografía dentro. Plan B listo: una línea en `--arles-font-family` (D-5, `TIPOGRAFIA.md` §3) |
| **FIRMA-WIN** | Certificado de firma de código para Windows | Dirección | bloqueado | Que el `.msi` se instale **sin la advertencia de «editor desconocido»** de SmartScreen, que en un entorno corporativo bloquea la instalación en la práctica. **Plazo externo de 1 a 3 semanas.** OV para la v1.2.0, EV para el lanzamiento comercial |
| **FIRMA-MAC** | Apple Developer Program de organización | Dirección | bloqueado | **Que ARLES abra en macOS.** Desde Catalina, una aplicación descargada sin notarizar no se ejecuta: no es una advertencia, es un bloqueo de Gatekeeper. Alta de la cuenta más notarización por build |
| **P-09** | Revisión jurídica | Abogado | bloqueado | Los textos legales que lee el usuario, y el primer envío real a alguien de fuera de TELEMETRY. Las dieciocho preguntas están en `08-legal/CONSULTA-JURIDICA.md`. La construcción **no** se detiene (ADR-0013) |
| **META** | Cuenta de WhatsApp Business con Coexistencia | Operaciones | bloqueado | Todo WhatsApp está probado contra un servidor de mentira. **Nunca ha hablado con Meta.** Por dentro el canal está completo desde D-WA (27/10/2026): campañas enteras incluidas |
| **R-07** | Revisión en macOS | quien tenga un Mac | bloqueado | Nadie ha abierto ARLES en WKWebView. La vista previa no cuenta: ahí el motor es el del navegador. **Lo arrastra FIRMA-MAC**: el mismo Mac hace las dos cosas |
| **R-24** | Un servidor de correo real | Dirección | **hecho** 22/10/2026 | Cerrado: ARLES manda por SMTP con TLS desde una cuenta de Titan, y llega a la bandeja principal de Outlook, Gmail y un corporativo |

> **Los dos trámites de firma se anotaron el 28/10/2026, y llevaban declarados
> desde la Fase 0.** `DISTRIBUCION_Y_FIRMA.md` §1 los pone como trámites que
> **arrancan en la Fase 1** precisamente porque son plazos externos, D-4 lo
> repite, y la matriz de riesgos los tiene como **R-14** con la mitigación
> escrita: *«trámite iniciado en Fase 1»*. No hay rastro de que se iniciara, y
> **ninguna de las tres listas de bloqueos del proyecto los nombraba**, así que
> nadie los estaba esperando.
>
> Es el décimo documento que afirmaba de más, y la frase exacta era de esta
> misma tabla: P-01 decía ser *«lo único entre ARLES y el `.exe`»*. Entre ARLES
> y un `.exe` que se instala sin advertencia hay también un certificado, y entre
> ARLES y un `.dmg` que **abre** hay un Developer ID. Una tabla que se lee para
> saber qué falta y deja fuera dos plazos de semanas no es una tabla incompleta:
> es la que hace que se descubran al final, que es el riesgo R-14 literal.
>
> Lo vigila `los_bloqueos_se_cuentan_en_un_solo_sitio` en la fase 0.

---

## Lo que espera una decisión

| Id | Qué | Planteado | Estado |
|---|---|---|---|
| **R-05** | Imagen en la firma | Embebida dispara filtros de spam y engorda cada correo; alojada hay que alojarla, y el saneado tendría que admitir `<img>` | espera decisión |
| **R-25** | Baja automática por correo | La recomendación es IMAP: leer la bandeja y dar de baja a quien responda BAJA | espera decisión |
| **D-19** | ¿El registro tiene que dar el total exacto? | C-2 midió que **el único cambio que baja el tiempo del filtro es dejar de contar todas las coincidencias**: traer las 500 filas cuesta la mitad. Pero el total al lado es D-7C, y se pidió para que el tope no escondiera nada. Las opciones y las cifras, abajo | espera decisión |
| **V11→V12** | Restaurar un respaldo de un esquema anterior | Hoy se rechaza con un error claro. Arreglarlo exige correr las migraciones sobre el propio respaldo, que es otra entrega | abierto |
| **Fase 9** | Sacar la activación al hilo del motor | Activar tarda ~73 s con 451 196 destinatarios y el presupuesto de 2 s se rompe **alrededor de los 30 000**. Desde el 28/10 la ventana **sí responde**, dice en qué va y se puede cancelar; lo que queda es que no corra dentro de un comando | abierto · medido en la auditoría C |

---

## Decisiones tomadas

Las que cambian cómo se construye. **No se vuelven a discutir** salvo que
Dirección lo pida.

| Id | Fecha | Decisión |
|---|---|---|
| **D-7A** | 29/09/2026 | Las respuestas de WhatsApp **se cuentan, nunca se guarda qué dijeron** |
| **D-7B** | — | La parada de emergencia vive en Campañas |
| **D-7C** | — | El registro lleva seis criterios de filtro combinables |
| **D-8A** | 05/10/2026 | El respaldo va **cifrado**, con opción explícita de desactivarlo |
| **D-8B** | 05/10/2026 | **Dos alcances**, «Esencial» por defecto |
| **D-8C** | 05/10/2026 | Restaurar **reemplaza, nunca mezcla** — y las bajas se **unen** |
| **D-8D** | 05/10/2026 | Se puede **archivar** el historial a un disco externo y sólo entonces borrarlo, con cuatro condiciones |
| **D-13** | 22/10/2026 | **El límite diario lo fija la campaña**, hacia arriba o hacia abajo. El `min` con la cuenta se quitó |
| **D-15** | 23/10/2026 | Cada campaña carga **su propia lista** de contactos. `None` es «toda la empresa», y la pantalla lo ofrece con esas palabras |
| **D-16** | 23/10/2026 | **Ventana de descanso** de 14 días entre campañas al mismo contacto |
| **D-WA** | 26/10/2026 | El canal de WhatsApp entra en la v1.2.0, **con campañas enteras** y no sólo el envío de prueba |
| **D-18** | 28/10/2026 | **[ADR-0015](../03-arquitectura/adr/0015-un-escritor-y-lectores-separados.md) aceptado.** Un escritor con cerrojo y lectores en conexión aparte. El **hilo dedicado con canal de ADR-0002 queda descartado por escrito**, no pendiente: su premisa era Tokio y este proyecto no lo usa. La conexión de lectura queda **programada**, y no empieza hasta medir el crecimiento del `-wal` |
| **DMARC** | 27/10/2026 | El DMARC se queda en `p=none` **dos o tres semanas** leyendo informes, y después se sube `pct=25` → 50 → 100. Saltar a `quarantine` manda correo legítimo a spam **en silencio**: sin rebote, sin error y sin aviso |
| **Baja en WhatsApp** | 27/10/2026 | **No se exige** que la plantilla lleve la salida escrita, porque ARLES no podría atenderla: no recibe los mensajes y no lee su texto (D-7A). Se avisa **a mano** en Contactos › Bajas, y se dice con esas palabras en la pantalla, la guía y el manual |

---

## Requisitos de Dirección, cerrados

| Id | Qué | Cerrado |
|---|---|---|
| **R-01** | Dar de alta un correo era «demasiado complicado» | ✅ 14/10/2026 · dos campos, con `PROVEEDORES` deduciendo servidor, puerto y cifrado de cinco dominios |
| **R-02** | Lo mismo con WhatsApp | ✅ 26/10/2026 · **un dato y una elección**: se pega el token y ARLES le pregunta a Meta cuáles son tus números |
| **R-06** | Una plantilla creada no se podía corregir | ✅ 17/10/2026 |
| **H-01** | Remitentes decía «no has configurado tu empresa» después de configurarla | ✅ |
| **H-02** | Las tres partes de un error salían pegadas | ✅ |
| **H-03** | No se pudo dar de alta una cuenta de Gmail | ✅ cerrado por R-01 |
| **H-05** | Error al dar de alta un contacto | ✅ |
| **H-06** | El horario de envío salía vacío, sin salida | ✅ |
| **H-12** | **La firma elegida nunca se guardaba.** Todas las campañas salían con la misma, y nada fallaba | ✅ 22/10/2026 |
| **H-07** | El límite diario se escribía y no se aplicaba: se pedía 300 y salían 50 | ✅ 22/10/2026 · D-13 |
| **H-08** | Dos campañas con el mismo remitente salían 25 y 25, por orden de llegada | ✅ 22/10/2026 · hoy salen las dos enteras, y el paso 6 avisa de que el plazo cambia |
| **H-13** | **Escribir `%` o `_` en el filtro del registro devolvía todas las filas.** El texto del usuario entraba dentro del patrón del `LIKE`, así que era sintaxis | ✅ 02/11/2026 · lo encontró medir C-2 |
| **H-14** | **El error de «fallo al migrar» afirmaba que se había hecho una copia de la base**, y no hay ninguna copia en todo el proyecto. De las tres partes del §95, la falsa era la tercera: la que existe para decidir qué se puede borrar sin perder nada | ✅ 03/11/2026 |
| **H-15** | **Un fallo del generador de aleatoriedad se diagnosticaba como «tu base es de una versión anterior»**, y mandaba a apartar un archivo que está bien — en el primer arranque, donde ni existe | ✅ 03/11/2026 · `DbError::SinAleatoriedad` |
| **R-26** | **La lista de contactos no se podía filtrar**: `listar_contactos` sólo tomaba página y tamaño. Cuatro criterios combinables desde el 03/11/2026 — nombre/apellido/empresa, dirección completa, canal y si tiene alguna baja. Las cifras y lo que se descartó, en su sección | ✅ 03/11/2026 |
| **H-16** | **Buscar «TÉLLEZ» no encontraba a Téllez.** `lower()` de SQLite sólo baja ASCII, así que la `É` quedaba intacta y el filtro decía que esa persona no está — **sin ningún error**. En un producto mexicano no es un caso raro | ✅ 03/11/2026 · lo encontró una prueba con un acento |
| **H-17** | **El filtro del registro usaba un índice del lado inútil.** La forma obvia de buscar por dirección —un `JOIN` con la empresa en el `WHERE`— nombra el índice correcto en el plan y tarda **609 ms** donde la forma buena tarda **0.0**: SQLite conduce desde `contact` por el `ORDER BY` y prueba contacto por contacto | ✅ 03/11/2026 · lo cazó el plan, no el resultado |

---

## C-2 · el filtro del registro, cerrado al contrario de lo previsto

**02/11/2026.** La auditoría C lo declaró con **3 709 ms de un presupuesto de
1 s** y escribió la solución: FTS5. **Se construyó y es 18 veces peor.**

Medido sobre 451 196 sucesos, con tres corridas de cada camino y las dos
búsquedas que importan — la de **una persona**, que es la que alguien hace, y
la de un dominio entero:

| Camino | Una persona (1 coincidencia) | Un dominio (119 532) |
|---|---|---|
| `LIKE '%…%'`, el de hoy | 1 257, 1 192, 1 151 ms | 1 212, 1 222, 1 146 ms |
| La pregunta al revés, sin índice | 1 015, 1 010, 1 145 ms | 1 192, 1 221, 1 217 ms |
| Índice FTS5 `trigram` | 21 556, 21 906, 20 653 ms | 23 242, 23 793, 24 448 ms |

**Cuatro cosas que cambian lo que se sabía:**

1. **El presupuesto se incumple por un 20 %, no por 3.7×.** La cifra de la
   auditoría no se reproduce, y no lo explica un cambio de esquema: el índice
   que usa esa consulta es de la V10, anterior a aquella medición. No se
   inventa la causa — aquella corrida no se guardó.
2. **Ningún índice sirve.** El FTS5 `trigram` empeora cuanto más largo es el
   texto, porque cada trigrama de una dirección está en casi todas las filas.
   Un índice cubriente no cambia el plan de consulta, así que se descartó sin
   medirlo.
3. **La pregunta al revés gana un 13 %**: no justifica cambiar una consulta
   correcta.
4. **El tiempo está en el total**, no en la búsqueda. Traer las 500 filas de la
   página cuesta la mitad. De ahí **D-19**, arriba: el total exacto es lo único
   que baja esta cifra, y es una decisión de producto.

Y los tres costes que la migración traía aparte del rendimiento, cada uno
suficiente por sí solo: una segunda copia de una dirección de correo que hay
que mantener sincronizada para que un ARCO no la deje encontrable; romper
`la_cascada_calculada_es_la_que_borra_sqlite`, que es la defensa contra **F1**;
e invalidar todos los respaldos anteriores, con un instalador ya fuera.

> **Lo que esto deja construido**, aunque el índice se haya ido: el banco mide
> los tres caminos y cualquiera puede volver a correr la comparación, en vez de
> creerse esta tabla; H-13 arreglado; y
> `ninguna_tabla_se_queda_fuera_del_respaldo` ya ve una tabla virtual — no la
> veía, y la fase 8 pasó en verde con una tabla nueva sin clasificar.

---

## R-26 · el filtro de contactos, y los tres defectos que encontró medirlo

**03/11/2026.** Cuatro criterios combinables en Contactos. Lo entregado es lo
de menos: lo que esta entrega deja son **tres defectos que llevaban tiempo ahí
y pasaban todas las pruebas**, y una cifra de ayer corregida.

### Qué cuesta cada criterio, sobre 455 470 contactos

| Criterio | Milisegundos | Veredicto |
|---|---|---|
| Dirección completa | **0.2** · no crece con la tabla | ✅ |
| Canal | 0.1 | ✅ |
| Texto, cuando lo encuentra | 1.4 | ✅ |
| **Texto, cuando NO lo encuentra** | **~2 600** | ❌ 2.6× el presupuesto, declarado |

**El texto cuesta lo que no encuentra**, y es lo contrario de la intuición:
para poder decir «no hay nada» hay que visitar las 455 470 filas. Es el caso
que se toca mientras se escribe, así que el campo no busca al teclear.

**Y un trozo de dirección no se ofrece**, con su cifra: 2.1 s, porque un sufijo
no lo acota ningún índice ordenado por el principio de la cadena. La pared de
C-2. También se descartó «empieza por» sobre el nombre: sin índice cuesta igual
que `%x%`, y con uno habría que pagar una migración que deja sin restaurar todo
respaldo anterior.

### Los tres defectos

**H-17 · Un índice usado del lado inútil.** La forma obvia de buscar por
dirección —un `JOIN` con la empresa en el `WHERE`— **nombra el índice correcto
en el plan** y tarda 609 ms donde la forma buena tarda 0.0: SQLite conduce desde
`contact` por el `ORDER BY` y prueba contacto por contacto, y como una dirección
casa una sola fila el tope de cien no se cumple nunca. Seis mil veces, con el
mismo índice en el plan y el mismo resultado devuelto.

Y enseña el límite de lo que ya había: `las_consultas_criticas_usan_su_indice`
pregunta si el plan **nombra** el índice, no por qué lado.

**H-16 · Buscar «TÉLLEZ» no encontraba a Téllez.** `lower()` de SQLite sólo
baja ASCII. El patrón se baja ahora en Rust.

**La conexión de lectura es 12× más lenta recorriendo una tabla.** 302 ms por el
escritor y 3 742 por la lectora, la misma consulta y la misma base de 182 261
contactos. Es el hallazgo grande y **no es del filtro**: es de ADR-0015, y está
contado con sus cifras y lo que se descartó en
[`PRESUPUESTO_RENDIMIENTO.md`](../06-calidad/PRESUPUESTO_RENDIMIENTO.md) §8.

> **Obliga a corregir una cifra de ayer.** Los 1 151–1 257 ms que C-2 publicó
> para el filtro del registro se midieron corriendo el SQL **por el escritor**,
> y la pantalla leía **por la lectora**: el coste real era del orden de
> **catorce segundos**. Es DOC-13 otra vez, un día después de escribir esa
> lección y en el mismo archivo. Queda **por remedir**.

### Lo que esta entrega enseña del método

**Medir los candidatos no es medir lo entregado.** Los nueve caminos candidatos
daban 899 ms en el peor caso; el camino real daba 22 000. La diferencia no
estaba en ninguno de los nueve: estaba en la junta —qué conexión, qué consultas
de más— que ninguno de los nueve recorría. Es el patrón del §9 aplicado a una
medición.

---

## Fallos que ya costaron datos

Se guardan porque **cada uno ha vuelto**, y la defensa que funcionó es la misma
las tres veces: leer la cascada del esquema y contarla, no tocarla.

| Id | Qué pasó | Veces |
|---|---|---|
| **F1** | Un `DROP TABLE` de una tabla padre se llevó en cascada la audiencia congelada y el registro de envíos | 3 — Fase 1, restaurar un esencial (8.3a), archivar el historial (8.3 bis) |
| **F2** | Perder la entrada del llavero y crear una clave nueva al arrancar deja la base **ilegible para siempre** | 2 — Fase 1, y la recuperación de la 8.4 que escribía la clave antes de leer el respaldo |

---

## Documentos que afirmaban de más

El patrón número 4 de [`RELEVO.md`](../00-ANTES-DE-EMPEZAR/RELEVO.md), y el más
barato de cometer: **un documento no falla, se cree.** Se anotan aquí porque un
documento equivocado cuesta trabajo rehecho igual que un defecto, y porque la
defensa que funciona es siempre la misma — una comprobación que lee el código,
no una relectura con buena voluntad.

| Id | Qué decía | Lo que había | Cerrado |
|---|---|---|---|
| **DOC-1** | `DESIGN_SYSTEM.md`: «la barra lateral es fija, **no colapsable**» | Se plega, con el umbral medido por `sonda:plegado` | ✅ 27/10/2026 |
| **DOC-2** | `DESIGN_SYSTEM.md`: la altura de fila es «**preferencia del usuario**» | Es una opción de cada tabla; esa pantalla no existe | ✅ 27/10/2026 |
| **DOC-3** | `DESIGN_SYSTEM.md`: el anillo de foco usa el oro de relleno | Usa la tinta de acento. En tema claro, el relleno daría **1.32:1** | ✅ 27/10/2026 |
| **DOC-4** | `DESIGN_SYSTEM.md`: los iconos vienen de «**Lucide (ISC)**» | 24 trazos propios. No es una dependencia del proyecto | ✅ 27/10/2026 |
| **DOC-5** | `DESIGN_SYSTEM.md`: «**cada** insignia lleva icono además del color» | El tono neutro no lo lleva, a propósito desde el 29/09/2026 | ✅ 27/10/2026 |
| **DOC-6** | `THREAT_MODEL.md` §8 pedía «la capa de redacción de `tracing`» | El proyecto no usa `tracing` | ✅ 06/10/2026 · auditoría B |
| **DOC-7** | `PRESUPUESTO_RENDIMIENTO.md` §3 describía una arquitectura con Tokio | No hay Tokio; la conexión es un `Mutex<Connection>` | ✅ 07/10/2026 · auditoría C |
| **DOC-8** | El `ROADMAP` ponía los derechos ARCO en la Fase 8 | Construidos desde la 3.4. La auditoría B perdió tiempo buscándolos | ✅ 07/10/2026 |
| **DOC-9** | `CLAUDE.md` §1 bis: «Todo documento nuevo se nombra en `00-INDICE.md`: **lo vigila la fase 0**» | No lo vigilaba nadie. La fase 0 tenía una lista de catorce documentos esperados y comprobaba que existieran; la dirección contraria no la miraba nada | ✅ 28/10/2026 · ahora la vigila `todo_documento_esta_en_el_indice` |
| **DOC-10** | Esta misma tabla: P-01 era «**lo único entre ARLES y el `.exe`**», y los bloqueos eran «los cuatro» | Son cinco. Faltaban los **dos trámites de firma**, declarados en `DISTRIBUCION_Y_FIRMA.md` §1 desde la Fase 0 como plazos externos que arrancan en la Fase 1, y ausentes de las **tres** listas de bloqueos del proyecto | ✅ 28/10/2026 · ahora los cuenta `los_bloqueos_se_cuentan_en_un_solo_sitio` |
| **DOC-11** | `avance.json`: **89 % a 2026-10-22**, con F3W al 55 %, F5 al 70 % y F7 al 92 % | Doce tramos cerrados después de esa fecha —la 7.5 entera y D-WA entera— dejaban las tres fichas describiendo un estado que ya no existía. F7 decía «falta sólo la 7.5» con la 7.5 construida. Recalculado: **95 %** | ✅ 28/10/2026 |
| **DOC-12** | `ADR-0002`: la base «vive en un hilo dedicado que recibe peticiones por canal», y «los lectores usan conexiones separadas» | Ni una cosa ni la otra. Cero `thread::spawn` y cero `mpsc` en `arles-db`, y **una sola** `Connection::open`: lo que hay es un `Mutex<Connection>`. Su decisión tecnológica sí está construida y es correcta; lo que no se construyó es una **mitigación** de sus consecuencias negativas | ✅ 28/10/2026 · acotado por [ADR-0015](../03-arquitectura/adr/0015-un-escritor-y-lectores-separados.md) |
| **DOC-13** | `actividad.rs` desde la 7.4, y la auditoría C repitiéndolo el 07/10: «si algún día hiciera falta más, **la respuesta es FTS5**» | Se construyó y es **18 veces más lento** (21 556 ms contra 1 257). Ningún índice sirve para este filtro, y el tiempo está en contar el total, no en buscar. Es el peor sitio donde dejar una recomendación sin medir: un comentario del código, que el siguiente lee como una decisión tomada | ✅ 02/11/2026 · la cabecera del módulo lleva ahora las nueve cifras, y el banco repite la comparación |
| **DOC-14** | `ESTADO_ACTUAL.md` §10 punto 3 y `RELEVO.md`: «el mensaje de **fallo al migrar** no cumple el §95, manda a cerrar y reabrir, y no dice dónde vive la base» | El del arranque se arregló el **14/10/2026** y cumple el §95 con la ruta dentro. Los dos documentos lo siguieron pidiendo tres semanas, y el registro de tropiezos sí tenía el arreglo escrito. El hallazgo de verdad estaba en otro sitio y era peor (H-14): **lo tapó una descripción desfasada que sonaba a lo mismo** | ✅ 03/11/2026 |

| **DOC-15** | `conexion.rs` desde el 28/10/2026: las cascadas de borrado son seguras con dos conexiones «porque la cascada necesita *triggers* recursivos y aquí están activados» | **No están activados**, y la cascada salta igual: `PRAGMA recursive_triggers` no se toca en ninguna línea del proyecto, y un borrado en cascada se midió disparando su `AFTER DELETE`. La garantía es cierta; el mecanismo con que se explicaba, no. Lo que de verdad depende de ese ajuste es `INSERT OR REPLACE`, del que hay **cero** | ✅ 03/11/2026 · el comentario explica ahora el mecanismo medido |
| **DOC-16** | La auditoría C y `ESTADO_ACTUAL.md` §10 punto 1: «**sacar la activación de campañas al hilo del motor**» | Mover el trabajo de hilo **no suelta el cerrojo de escritura**: lo tomaría el hilo del motor en vez del del comando. Medido con una escritura de 800 ms en curso: por la conexión de lectura **0 ms**, por la de escritura **650**. Lo que descongela las pantallas es terminar ADR-0015, y es lo que se hizo — 17 de 24 lecturas movidas. Segunda recomendación de auditoría en dos días que se cae al medirla (la otra es DOC-13) | ✅ 03/11/2026 · lo vigila `toda_lectura_de_pantalla_va_por_su_conexion` |
| **DOC-17** | `MAPA-DE-PANTALLAS.md`: «**las trece pantallas**», y «**Actividad es andamio** hasta su fase» | Son dieciocho desde la Fase 8, y Actividad la construyó la 7.3 el 30/09/2026 llevándose `PantallaPendiente.vue`. Faltaban el Registro, Respaldos, Restaurar, Archivar y Recuperar — **las cuatro últimas son todo lo que construyó la Fase 8**. Es el defecto de las sondas del 28/10 en el documento que se lee **para cambiar el diseño** | ✅ 03/11/2026 · lo vigila `el_mapa_de_pantallas_nombra_todas_las_pantallas` |
| **DOC-18** | C-2 el 02/11/2026: el filtro del registro cuesta «**1 151–1 257 ms**» | Esa medición corrió el SQL **por la conexión de escritura**, y `Db::sucesos` leía **por la lectora**, que es 12× más lenta recorriendo una tabla. El coste real de la pantalla era del orden de **catorce segundos**. Es DOC-13 otra vez —una cifra de rendimiento medida por un camino que el usuario no recorre— **un día después de escribir esa lección y en el mismo archivo**. Queda por remedir | ⚠️ 03/11/2026 · declarado, pendiente de remedir |

**Las cinco primeras llevaban ahí desde la Fase 2** y ninguna de las cuatro
auditorías del §140 las vio, porque una auditoría mide el producto y esto es un
documento. Las caza ahora `el_sistema_de_diseno_dice_lo_que_hay` en
`validar.py --fase 2`, que compara el documento con los dos CSS y con la carpeta
de componentes, en las dos direcciones.

**DOC-6 y DOC-7 son el mismo defecto a una pasada de distancia:** dos documentos
de la Fase 0 nombrando bibliotecas que nunca se adoptaron. No tienen todavía
ninguna comprobación que los vigile, y está dicho así en vez de darlos por
seguros.

> **DOC-9 es el que más duele de los nueve, y conviene ver por qué.** Los otros
> ocho dicen algo falso sobre el producto. Éste decía falsamente que **había una
> red**, y una red que se cree es una red que hace dejar de mirar: quien leyera
> esa frase daría por comprobado algo que nadie comprobaba.
>
> Medido el día que se escribió la comprobación: **114 de 115 documentos** sí
> estaban en el índice. La regla se venía cumpliendo a mano, así que el hueco no
> se notaba — y un hueco que no se nota es el que un día se cuela.

> **DOC-10 y DOC-11 son los dos únicos de la lista que afirman de más sobre el
> propio estado del proyecto**, y por eso son los que más caro salen: no mandan
> a buscar una función que no existe, mandan a no buscar algo que falta. DOC-11
> dejaba la portada del repositorio diciendo **89 %** con el 95 % construido,
> que es el error en la dirección cómoda de ver y la incómoda de cometer —
> presenta menos trabajo hecho del que hay—; DOC-10 dejaba fuera de la lista de
> bloqueos **dos plazos de semanas** que nadie estaba esperando.
>
> **Los dos los encontró contestar una pregunta de Dirección**, no una
> auditoría: «qué sigue en el roadmap y qué falta para una beta». Las cinco
> auditorías corridas en este proyecto miden el producto; para que un documento
> de estado se vea falso hay que ir a usarlo para algo.

> **DOC-12 es el tercero que nombra algo que nunca se adoptó** —tras `tracing`
> en el `THREAT_MODEL.md` (DOC-6) y Tokio en el presupuesto (DOC-7)—, y es el
> **primero que es un ADR**. La diferencia importa: un ADR es el documento al
> que se va a preguntar *por qué* algo se decidió así, así que quien lo leyera
> construiría el hilo dedicado con canal creyendo que arregla algo.
>
> **Y de las dos mitades que describía, una estaba equivocada.** No es que
> faltara el hilo: es que su premisa escrita —«`rusqlite` es síncrono, no encaja
> en Tokio»— dejó de existir cuando el proyecto no adoptó Tokio, y construirlo
> ahora bajaría la calidad. Es el primer caso de los doce en que lo correcto no
> era construir lo prometido sino **descartarlo por escrito y decir por qué**.
>
> Los dos de la misma familia tienen además el mismo origen: la auditoría C
> corrigió esta frase **en el presupuesto** el 07/10/2026 y **no en el ADR**.
> Corregir una copia y dejar la fuente es cómo un documento falso sobrevive a su
> propio hallazgo.

---

## Dónde vive lo que no está aquí

| Qué | Dónde |
|---|---|
| Los defectos encontrados **usando** ARLES, con su estado | `06-calidad/BITACORA-DE-PRUEBAS.md` |
| Los tropiezos de instalación y arranque | `07-entrega/REGISTRO-DE-TROPIEZOS.md` |
| Qué contarme al reportar un error | `06-calidad/QUE-INFORMACION-NECESITO.md` |
| Los hallazgos de las seis auditorías | `09-fases/AUDITORIA-*.md` |
| El estado del proyecto | `07-entrega/ESTADO_ACTUAL.md` |
