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
| **C-2** | Filtro de la lista de contactos | El del **registro** sí existe, con seis criterios (7.4). El de la lista de contactos, no | abierto · declarado en la auditoría C |
| **V11→V12** | Restaurar un respaldo de un esquema anterior | Hoy se rechaza con un error claro. Arreglarlo exige correr las migraciones sobre el propio respaldo, que es otra entrega | abierto |
| **Fase 9** | Activar una campaña en segundo plano, con avance | Activar tarda 73.5 s con 451 196 destinatarios y se rompe **alrededor de los 30 000**. Hoy la interfaz se congela y no se puede cancelar | abierto · medido en la auditoría C |

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
