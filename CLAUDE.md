# ARLES RELAY I · v1.2.0 — instrucciones para Claude Code

Este archivo lo lee Claude Code al abrir el proyecto, y existe por una razón
concreta: **las reglas de ARLES vivían sólo en una conversación**. Cualquier
sesión nueva —en la terminal de VS Code, en otro equipo, dentro de un mes—
arrancaba sin saber nada de ellas y las rompía sin enterarse.

Lo que sigue no son preferencias: son decisiones ya tomadas, casi todas porque
algo salió mal antes.

---

## 1 · Lo que no se negocia

**Responde SIEMPRE en español.** Todo: respuestas, planificación, comentarios
del código, documentación, mensajes de commit. El equipo es mexicano y el
producto también.

**Rama de trabajo: `claude/loving-faraday-5chmmy`.** Nunca se empuja a otra sin
permiso explícito de Dirección.

**Nunca abras un pull request** salvo que Dirección lo pida con esas palabras.

**Credenciales:** jamás en texto plano, JSON, `localStorage`, bitácoras ni
tablas de SQLite sin cifrar. Van al llavero del sistema —Keychain en macOS,
Credential Manager en Windows—. Si el llavero no está disponible, **la
aplicación no arranca**; nunca se degrada a texto plano (ADR-0011).

**Claves de firma:** no se guardan en el repositorio. Ninguna.

**Carpetas de sólo lectura**, material de referencia de Dirección:

    /RECURSOS   /REFERENCIA_DE_COLOR   /CONCEPTOS_DE_DISEÑO
    /REFERENCIAS_VISUALES_DEL_SITIO    /TIPOGRAFIA

No se borran, mueven, renombran, convierten, comprimen ni editan.

**Cada commit termina con:**

    Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
    Claude-Session: <el enlace de LA SESIÓN QUE ESCRIBE EL COMMIT>

La línea de sesión **no se copia de aquí ni de un commit anterior**: cada sesión
tiene la suya, y es lo que permite volver a la conversación donde se decidió
algo. Copiar la de otra apunta a una conversación que no explica ese commit.

**Ningún identificador de modelo** en commits, cuerpos de PR, comentarios del
código ni nada que acabe en el repositorio. En el chat sí.

---

## 1 bis · El orden del proyecto · requisito de Dirección del 27/10/2026

Esta sección **no se omite nunca**, y Dirección la pidió con esas palabras.

**Antes de tocar nada, se lee
[`documentacion/00-ANTES-DE-EMPEZAR/LEEME-PRIMERO.md`](documentacion/00-ANTES-DE-EMPEZAR/LEEME-PRIMERO.md).**
Son cinco archivos y veinte minutos: las reglas, lo que nunca se hace, el
método de verificación, el estado y los requisitos. Esa carpeta existe porque
este proyecto ha fallado más veces por no leer que por programar mal.

**Cada archivo que se crea va en su carpeta**, y la carpeta no se decide sobre
la marcha: está en
[`DONDE-VA-CADA-COSA.md`](documentacion/00-ANTES-DE-EMPEZAR/DONDE-VA-CADA-COSA.md).
Un archivo en la raíz o en una carpeta nueva «por ahora» es cómo se llega a dos
verdades que se contradicen — y este proyecto ya tuvo un índice diciendo «Fase
0 — Discovery» con la Fase 8 cerrada. **Todo documento nuevo se nombra en
`00-INDICE.md`**: lo vigila `todo_documento_esta_en_el_indice` en la fase 0,
que lee la carpeta y no una lista.

> **Esa última frase fue falsa hasta el 28/10/2026**, y es el noveno documento
> del proyecto que afirma de más. La fase 0 tenía una lista de catorce
> documentos esperados y comprobaba que existieran; **la dirección contraria no
> la miraba nada.** Es el peor de los nueve, porque los otros dicen algo falso
> del producto y éste decía falsamente que **había una red** — y una red que se
> cree hace dejar de mirar.

**Al cerrar un tramo se actualiza el estado.** Las cuatro cosas que tienen un
solo sitio, y escribir una segunda es crear una contradicción:

| Qué | Dónde, y sólo ahí |
|---|---|
| El estado del proyecto | `07-entrega/ESTADO_ACTUAL.md` |
| El porcentaje de avance | `07-entrega/avance.json` → `herramientas/avance/calcular.py --escribir` |
| Los requisitos y lo que bloquea | `10-requisitos/REGISTRO-DE-REQUISITOS.md` |
| Los defectos encontrados usando ARLES | `06-calidad/BITACORA-DE-PRUEBAS.md` |

**Y se escribe el poka-yoke.** Cuando algo falla, la pregunta no es sólo «¿cómo
lo arreglo?» sino **«¿qué comprobación habría hecho imposible esto?»** — y se
escribe, y se verifica rompiéndola. Gana el tipo sobre el validador, y el
validador sobre la prueba: una variante con el dato dentro no necesita que
nadie se acuerde. Los cinco fallos propios conocidos al escribir uno —la lista
a mano, el alcance corto, la hoja y no la ruta, el comentario que se caza a sí
mismo, y escribir en el repositorio para medirlo— están en
[`METODO.md`](documentacion/00-ANTES-DE-EMPEZAR/METODO.md) §4.

**Las cuatro preguntas antes de empujar**, siempre:

1. **¿Quién lo llama?** La pieza nueva tiene un llamador de verdad, no una
   prueba. Si todavía no lo tiene, **no se registra** y queda dicho dónde
   vuelve.
2. **¿Qué documento deja de ser verdad?** El estado, el índice, el manual, el
   mapa de pantallas, el registro de requisitos.
3. **¿Qué poka-yoke hace falta?** Y verificado rompiéndolo.
4. **¿Está todo en su carpeta?**

**Los informes y documentos para Dirección** siguen
[`05-diseno/guia-de-documentos/`](documentacion/05-diseno/guia-de-documentos/README.md):
cifras y no adjetivos, sin un solo emoji, nada que pudiera estar en cualquier
informe de cualquier proyecto, y la sección de lo que todavía no hace es
obligatoria.

**Y los roles.** Antes de cerrar un tramo se mira desde los sombreros que el
cambio toca —arquitecto, datos, seguridad, QA, rendimiento, diseño,
accesibilidad, redacción, operaciones, legal—, que es como se encontraron
diecisiete cosas con las diez fases en verde. Están en
[`ROLES.md`](documentacion/00-ANTES-DE-EMPEZAR/ROLES.md), con lo que cada uno
mira y lo que cada uno ha encontrado.

---

## 2 · La regla de trabajo

> **Una comprobación que nunca se ha visto fallar no está verificada.**

Cada garantía que se afirme —en un test, en el validador de fase, en una sonda—
se rompe a propósito para ver si la comprobación la caza. Si no la caza, la
comprobación no vale y hay que arreglarla antes de seguir.

Esto no es ceremonia. En este proyecto ya ha encontrado, entre otras:

- Una prueba de la página de contactos con **un solo contacto**: pedir diez mil
  también devolvía uno, así que el recorte de página se podía quitar entero.
- Una comprobación del validador que miraba **la línea equivocada** encima de
  una estructura y pasaba siempre, dijera lo que dijera el `derive`.
- Una sonda que buscaba `+521` en un texto donde el número se enseña con
  espacios: el patrón no podía coincidir **nunca**.
- Un troceado en lotes que no estaba verificado porque la prueba pedía 1 500
  variables y el tope real de SQLite resultó ser **32 766**, no las 999 que el
  comentario repetía de documentación vieja.

El método: cambia el código para romper la garantía, corre la prueba, comprueba
que **falla con el mensaje correcto**, y restaura. Se deja constancia en el
comentario de la prueba de que se hizo.

---

## 3 · Cómo se responde a Dirección

Adrián Paz no es programador. Las explicaciones van en sus términos, sin
esconder lo técnico cuando importa.

**Al cuestionar una decisión**, se hace con esta estructura, no con una opinión
suelta:

    Problema → Riesgo → Impacto → Alternativa → Trade-off → Recomendación

**Al informar**, se dice lo que hay: si algo falla, se enseña la salida; si un
paso se saltó, se dice. Los porcentajes llevan fecha. Las cifras son cifras y no
adjetivos.

**Paso a paso.** Dirección lo ha pedido muchas veces. Un tramo, se verifica, se
empuja, se explica, y se pregunta antes del siguiente.

---

## 4 · Arquitectura, en una pantalla

Monolito modular. Tauri 2 + Rust + Vue 3. Cuatro crates y una app:

| Crate | Qué es | Regla que lo define |
|---|---|---|
| `arles-core` | Dominio puro | **Cero I/O.** Ni base, ni red, ni disco. `forbid(unsafe_code)` |
| `arles-db` | SQLCipher, migraciones, repositorios | **No conoce la IPC**: no depende de `serde` |
| `arles-import` | Lectura de CSV y XLSX | El archivo que lee **no es de fiar** |
| `arles-motor` | El bucle que envía: cola, ritmo, ventana, disyuntor | **No escribe SQL.** Pide a `arles-db` con métodos tipados. Desde la 5.4 lo arranca `arles-app` **en su propio hilo** (§51) |
| `arles-email` | El adaptador SMTP y un servidor de mentira para probarlo | **No hay forma de no cifrar** (§29). El motor no sabe que existe |
| `arles-app` | Shell de Tauri y comandos | La única frontera con la webview |
| `app/` | Vue 3 + TS estricto | `exactOptionalPropertyTypes` activo |

**Las cuatro reglas de la frontera IPC** (`ARQUITECTURA.md` §5):

1. Comandos **gruesos**, no finos. Cada cruce es una oportunidad de estado
   inconsistente.
2. Errores **tipados**, nunca cadenas.
3. **Ningún comando devuelve un secreto.**
4. **Ninguna ruta del sistema de archivos cruza la frontera.**

Y una más que se ganó a golpes: **un tipo validado no se puede deserializar.**
`DatosDeContacto` y `CanalValidado` sólo salen del validador del núcleo. Si
pudieran entrar por JSON, la webview mandaría canales «normalizados» a su gusto
y la deduplicación y la supresión —que comparan esa forma— dejarían de funcionar
sin que nada fallara.

---

## 5 · Convenciones que el validador comprueba

- **§17 · Sin literales en los componentes.** Ni un hex, ni un píxel suelto, ni
  una duración. Todo sale de los tokens.
- **§98 · El movimiento tiene propósito**, y `prefers-reduced-motion` es
  obligatorio.
- **§65 · Aceptado ≠ entregado.** ARLES no puede saber si un correo llegó, así
  que la interfaz no dice «entregado».
- **§94 · Cifras, no adjetivos.** «2 de 6», «34 %» — nunca «casi listo».
- **§95 · Todo error dice tres cosas**: qué pasó, cómo arreglarlo y **qué está
  a salvo**.
- **§139 · Los errores llevan clave de i18n**, no texto. Cada variante de
  `CoreError`, `DbError` y `AppError` tiene su `clave_i18n()`, y hay un test que
  falla si alguien añade una sin texto.
- **ADR-0010 · El numeral «I» nunca va junto al número de versión.** «ARLES
  RELAY I» en la marca; «v1.2.0» aparte.
- **§21 · El logotipo de ARLES es exclusivamente tipográfico.**

**Las migraciones no se editan una vez aplicadas.** Y `DROP TABLE` de una tabla
padre está prohibido: con `PRAGMA foreign_keys=ON` borra en cascada a los hijos
—así se perdieron una vez la audiencia congelada y el registro de envíos—. Se
usa `ALTER TABLE … DROP COLUMN`.

---

## 6 · Cómo se verifica, en orden

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings   # cero advertencias
cargo test --workspace
cargo deny check licenses advisories                    # antes de añadir dependencias

cd app && npm test && npm run lint && npx vue-tsc --noEmit -p tsconfig.json

python3 herramientas/validar/validar.py --fase 6        # 0 a 6
python3 herramientas/validar/validar.py --fase 6 --rapido   # omite las sondas

# Y ANTES DE EMPUJAR, las siete. Cuesta un minuto:
for f in 0 1 2 3 4 5 6 W 7 8; do python3 herramientas/validar/validar.py --fase $f --rapido; done
```

**Correr sólo la fase en curso no basta.** En la 5.2 y la 5.3 se verificó con
`--fase 5`, que pasaba entera, y se rompieron cuatro comprobaciones de las fases
2, 3 y 4 sin que nadie lo viera: seis tokens de diseño inventados —que el
navegador resuelve a nada, sin error—, la huella de una migración sin registrar,
la vista previa desalineada con el núcleo y una comprobación que se cazaba a sí
misma en un comentario. Está contado en `ESTADO_ACTUAL.md` §8 bis.

**Las sondas abren un navegador de verdad y miden.** `npm run sonda:contactos`,
`sonda:tema`, `sonda:plegado`, `sonda:cabecera`, `sonda:vista-previa`,
`sonda:encimado`… Necesitan Chromium; sin él el validador las marca como
**omitidas**, que no es lo mismo que pasadas y así se dice.

**El tablero visual** que Dirección abre con doble clic:

```bash
python3 herramientas/tablero/generar.py --medir   # deja tablero/ARLES-tablero.html
```

No lleva **ni una cifra escrita a mano**: avance, hallazgos, bloqueos,
tropiezos y comprobaciones salen de sus fuentes. Un tablero con cifras dentro
es el §9 de este archivo con colores, que dan más confianza todavía — y el
validador compara su porcentaje con `avance.json`.

**La vista previa** que Dirección abre con doble clic:

```bash
python3 herramientas/vista-previa/generar.py     # deja vista-previa/ARLES-vista-previa.html
```

**El porcentaje de avance** sale de un solo sitio y se escribe a mano al cerrar
trabajo:

```bash
# editar documentacion/07-entrega/avance.json  (campos «hecho» y «porque»)
python3 herramientas/avance/calcular.py --escribir
```

Va **sólo en el README de GitHub**, nunca dentro de la aplicación: es un dato
del proyecto, no del producto.

---

## 7 · Dónde está escrito lo demás

`documentacion/00-INDICE.md` es el mapa. Lo que más se consulta:

| Para | Mira |
|---|---|
| Qué es y qué no es ARLES | `01-producto/VISION_Y_ALCANCE.md` |
| Los dos canales, decisiones L-1 a L-14 | `01-producto/LOGISTICA_DE_CAMPANAS.md` |
| Tablas, índices, invariantes | `03-arquitectura/MODELO_DE_DATOS.md` |
| Por qué algo se decidió así | `03-arquitectura/adr/` |
| Superficie de ataque | `04-seguridad/THREAT_MODEL.md` |
| Colores y contrastes medidos | `05-diseno/COLOR_SYSTEM.md` |
| Lo que Dirección revisa a ojo | `06-calidad/CHECKLIST-VISUAL.md` |
| Fases y dependencias reales | `07-entrega/ROADMAP.md` |

---

## 8 · Lo que está bloqueado esperando a alguien de fuera

Ninguno impide construir. **Los cuatro impiden entregar la v1.2.0.**

| Qué | Quién | Qué bloquea exactamente |
|---|---|---|
| **P-01 · Licencia de Mont** | Dirección | **No se puede distribuir un instalador** con la tipografía dentro. Es lo que impide entregar el `.exe` |
| **P-09 · Revisión jurídica** | Abogado | Los textos legales que el usuario lee. Las dieciocho preguntas están en `08-legal/CONSULTA-JURIDICA.md`. La construcción **no** se detiene (ADR-0013: se construye con texto marcado como provisional) |
| **META · Cuenta con Coexistencia** | Operaciones | P-14 no se puede probar contra nada real |
| **R-07 · Revisión en macOS** | Quien tenga un Mac | Nadie ha abierto ARLES en WKWebView. La vista previa **no cuenta**: ahí el motor es el del navegador |
| **R-24 · Ningún servidor de correo real** | Dirección | Todo lo de la Fase 5 está probado contra un servidor de mentira. **No se puede cerrar desde un contenedor**: la red de la sesión bloquea los puertos 587, 465 y 25 —medido—, y además ARLES exige el llavero del sistema, que un Linux sin escritorio no tiene. Se cierra en la máquina del usuario, con una cuenta desechable |

---

## 9 · Dónde va el trabajo ahora

> **Esta sección se actualiza al cerrar cada entrega.** Estuvo diciendo
> «entrega 3.3» hasta el 23/09/2026, con la Fase 4 cerrada y la 5 a medias: un
> archivo que se lee al abrir el proyecto y que miente sobre el estado es peor
> que no tenerlo, porque se cree.

**El estado completo está en
[`documentacion/07-entrega/ESTADO_ACTUAL.md`](documentacion/07-entrega/ESTADO_ACTUAL.md)**
— qué hay construido, qué decisiones no hay que volver a discutir y qué sigue.
Quien retome el trabajo empieza por ahí.

**Cerrada la parte de correo de la Fase 5** (entregas 5.1 a 5.6): traducir lo
que dice un servidor, el adaptador con cifrado obligatorio, la cuenta guardada
sin su contraseña, el motor encendido en su propio hilo, y el envío de prueba —
**ARLES manda un correo entero**, probado contra un servidor que habla cifrado.

**Lo que falta de la Fase 5 es el adaptador de WhatsApp Cloud API**, que el
ROADMAP pone aquí y no está empezado. Hay una decisión pendiente de Dirección:
construirlo antes de la Fase 6, o entregar la Fase 6 ofreciendo sólo correo.
Está planteada en `documentacion/09-fases/FASE-05-PARA-DIRECCION.md`.

**La Fase 6 está cerrada** (6.1 a 6.7): el motor de
sustitución no Turing-completo, el saneado del HTML en Rust, plantillas con
versiones que no se editan, campañas con la audiencia congelada al activar, y el
preflight de ocho puntos **enchufado de verdad** a la base y a la sonda de la
cuenta — incluido el envío de prueba de la 5.5, que llevaba tres entregas
anotándose sin que nadie lo preguntara.

…y el asistente de nueve pasos, con el aviso de más de 50 diarios que el núcleo
exige y registra en la bitácora. **Se puede crear una campaña de principio a fin
y activarla.**

**Tres cosas que la Fase 6 no cubre y conviene no prometer:** la audiencia no se
puede filtrar, una campaña creada no se puede editar, y no hay pantalla para
escribir plantillas ni firmas —los comandos están, el editor no—.

**Cerrado el canal de WhatsApp** (W.1 a W.8): el núcleo con sus dos modos, el
adaptador de Cloud API con su servidor de mentira, el número en el llavero con
Coexistencia y la aceptación de riesgo, el procesador de avisos, la puerta del
canal con las dos cifras de permiso, la pantalla, y **el envío de prueba que
manda un WhatsApp real**. `validar.py --fase W` en 21/21.

> **Esta frase fue falsa durante tres entregas, y conviene saber en qué sentido.**
> El comando existía, probado de punta a punta contra el servidor de mentira; lo
> que no existía era **el botón**, así que desde ARLES no se podía mandar un
> WhatsApp. Lo cerró la 7.7 el 01/10/2026, con sus seis textos — que tampoco
> estaban escritos. Un archivo que se lee al abrir el proyecto y afirma de más
> es peor que no tenerlo, porque se cree: §9, dicho de otra forma.

**Dos huecos del mismo tipo, y la comprobación que ahora los caza.** El canal
llegó a su cierre con setenta pruebas en verde y **sin poder mandar nada**: el
adaptador estaba probado y `arles-app` ni siquiera dependía de él. En la Fase 6
pasó igual con el editor de plantillas. Una prueba comprueba que la pieza
funciona, no que otro módulo la use, y el hueco vive justo entre dos capas que
cada una pasa lo suyo. Lo caza `lo_que_se_construye_se_usa()` en la fase W del
validador — escrita antes de arreglar el hueco, para verla marcarlo en rojo.

**Lo que WhatsApp todavía NO hace:** el motor solo recorre `Canal::Correo`, así
que **no se puede lanzar una campaña entera por WhatsApp**, solo el envío de
prueba. Y las respuestas no se pueden contar sin el «relay ciego» que Dirección
decidió el 29/09/2026 y que no existe todavía.

**Cerrada la 6.8** —el editor de plantillas y firmas—, que era lo que
bloqueaba crear una campaña entera desde la interfaz.

> **Esta línea decía además «y el registro de las plantillas aprobadas de
> Meta», y era falso.** `Db::anotar_plantilla_de_meta` y
> `Db::plantillas_de_meta` entraron en la 6.8 con sus pruebas, y **no las
> llamaba nadie**: ni comando ni pantalla. La función existía y el registro no.
>
> Lo que costaba no se vio hasta el 27/10/2026, al llegar al asistente: el paso
> 4, al elegir WhatsApp, habría sido un desplegable vacío sin ninguna pista de
> dónde se registran — el H-06 otra vez y en el mismo paso. Lo cerró el tramo
> 3c-2a de D-WA, con su sección en Plantillas › WhatsApp.
>
> Y es el patrón del §9 con una cara nueva: no sólo la pieza sin llamador, sino
> **un archivo que da la pieza por conectada**. `ninguna_tabla_se_queda_sin_escritor`
> no lo cazaba porque `whatsapp_template` sí tenía un escritor en `arles-db` —
> lo que no tenía era quien llamara al escritor.

**Y cerrada la auditoría de recorrido** (A1 a A3): el inventario de cada
pantalla y cada acción, las nueve insignias ilegibles, y la revisión **a ojo**
de las trece pantallas en un navegador, que encontró siete cosas con todas las
sondas en verde. Está contado más arriba, en «Lo que sólo se ve mirando».

**Lo siguiente:** la **Fase 7 · Actividad**, planificada en
[`documentacion/09-fases/FASE-07-PLAN.md`](documentacion/09-fases/FASE-07-PLAN.md):
siete entregas, tres decisiones que esperan a Dirección y cinco propuestas.

**Cerrada la 7.0** —el freno—, **la 7.1** —el registro de actividad—, **la 7.2**
—las cifras de cada campaña— y **la 7.3, la pantalla de Actividad**: una tarjeta
por campaña con sus cifras, cuánta gente contestó (D-7A, nunca qué dijeron), el
cupo de cada remitente con las campañas que se lo reparten, y **por qué no sale
nada** — cuál de las seis razones es, con su cifra. `validar.py --fase 7` en 6/6.

Con Actividad construida **las seis secciones tienen su pantalla**, así que el
andamio (`PantallaPendiente.vue`) se fue con ella.

**Cerrada la 7.4** —el registro con sus filtros (D-7C)—: seis criterios
combinables que viajan en la dirección, nunca más de 500 filas con el total al
lado, la dirección leída del intento y no guardada en el registro (P-17), y el
recorte bimestral que dice antes de tocar nada cuánto se va y cuánto se queda.
**Un desenlace de envío no se borra jamás.**

**Cerrada la 7.6** —el resumen exportable—: un archivo de texto por campaña con
**el método de cada cifra** al lado, los fallos agrupados por causa en español,
y **sin una sola dirección de correo** — lo garantiza una comprobación, no sólo
una prueba: la estructura que se redacta no tiene dónde meter una.

**Cerrada la 7.7, el cierre de la fase** —auditoría, revisión a ojo de las
catorce pantallas, documento e informe—. Con las nueve fases en verde encontró
**siete defectos**: los cinco comandos y claves de la tabla de arriba, «Detener»
con 2.41:1 de contraste, y tres pantallas de catorce **sin `h1`**. Está contado
en [`FASE-07-ACTIVIDAD.md`](documentacion/09-fases/FASE-07-ACTIVIDAD.md).

**Dos comprobaciones que decían cubrir una regla cubrían una parte**, las dos
por una lista escrita a mano, y las dos están reescritas para leer del código:

- La del §139 tenía una lista de prefijos sin `whatsapp` ni `plantilla`, y
  comparaba **la hoja y no la ruta** — así pasó en verde una clave guardada en la
  rama equivocada. Ahora los prefijos **son los bloques del propio catálogo** y
  se resuelve la ruta entera.
- La sonda de accesibilidad decía vigilar la jerarquía de encabezados y nunca
  miraba **por cuál empieza**: `previo` arrancaba en 0, así que un `h2` inicial
  sin `h1` no era un salto. Tres pantallas llevaban así desde la Fase 3.

**La 7.5 (SPF, DKIM y DMARC) se hace al final**, por decisión de Dirección del
30/09/2026: es lo único que necesita red y una dependencia nueva, y sólo se
puede verificar de verdad el día que se cierre R-24. No bloquea nada.

**La parada de emergencia ya se puede pulsar.** La Fase 4 la construyó, la 7.0 le
puso sus tres comandos… y ninguna pantalla los llamaba. Está en Campañas (D-7B),
y Actividad la enseña porque es la razón número uno por la que no sale nada.

**En marcha la Fase 8 · Respaldos y endurecimiento**, planificada el 05/10/2026
en [`FASE-08-PLAN.md`](documentacion/09-fases/FASE-08-PLAN.md). **Dirección
contestó las cuatro decisiones el mismo día:**

- **D-8A** · el respaldo va **cifrado**, con opción explícita de desactivarlo.
- **D-8B** · **dos alcances**, «Esencial» por defecto.
- **D-8C** · restaurar **reemplaza, nunca mezcla** — y las bajas se **unen**.
- **D-8D** · se puede **archivar** el historial a un disco externo y sólo
  entonces borrarlo, con cuatro condiciones: nunca automático, el archivo
  verificado antes del borrado, las bajas y el consentimiento fuera, y todo en
  `audit_log`. Va en la **8.3 bis**, después de que restaurar funcione.

**Cerrada la 8.0 · el manifiesto del respaldo** (`arles-core/src/respaldo.rs`):
las **29 tablas** del esquema clasificadas en cuatro alcances, cada una con su
motivo escrito. Tres comprobaciones nuevas en `--fase 8` y seis pruebas.

> **Al añadir una tabla en una migración, hay que clasificarla en el
> manifiesto.** No es burocracia: una tabla que no está en el respaldo se
> exporta, se restaura, y **sus datos no están** — sin excepción, sin aviso y
> sin ninguna prueba en rojo. Es el patrón del §9 en su forma más cara, porque
> aquí lo que se pierde no es una función, son datos de personas.
>
> Lo caza `ninguna_tabla_se_queda_fuera_del_respaldo`, que lee **el esquema**
> —descontando las cuatro tablas `_nueva` que las migraciones crean y renombran—
> y no una lista a mano. Marcar una tabla como `Nunca` es válido; omitirla, no.

**Cerrada la 8.1 · el formato `.arles`** (`arles-core/src/formato_de_respaldo.rs`),
con cero I/O: una línea por fila, cabecera con la versión **del formato** —no la
de ARLES—, la del esquema, el alcance y el recuento por tabla, y la suma sha256
al final. Doce pruebas.

> **Cuatro decisiones del formato que no son estéticas:**
>
> 1. **Se escapan `\`, tabulador, salto y retorno.** Una plantilla lleva saltos
>    de línea **siempre** y un nombre pegado de un Excel puede llevar
>    tabuladores: sin escaparlos la fila se parte al restaurar y el contacto sale
>    con los datos corridos, sin un solo error.
> 2. **Vacío y nulo son dos cosas.** `""` es un apellido que alguien borró; `\N`
>    es uno que no se sabe. Confundirlos cambia el dato en silencio.
> 3. **Un respaldo de un formato posterior se rechaza, no se intenta.** Leerlo
>    «lo que se pueda» restauraría una base incompleta sin decirlo.
> 4. **La suma va al final**, porque calcularla exige haber escrito el cuerpo, y
>    ponerla arriba obligaría a dos pasadas sobre medio millón de filas.
>
> Tres errores nuevos en `CoreError` —`NoEsUnRespaldo`,
> `RespaldoDeVersionFutura`, `RespaldoCorrupto`— y son **tres y no uno** a
> propósito: los tres impiden restaurar, pero uno manda a elegir otro archivo,
> otro a actualizar ARLES y el tercero a buscar otra copia.

**Cerrada la 8.2a · el sobre cifrado** (`arles-core/src/cifrado_de_respaldo.rs`).
Argon2id + XChaCha20-Poly1305 **en trozos de 64 KiB**. Las dos dependencias
nuevas pasaron `cargo deny`: `advisories ok, licenses ok`.

> **Tres cosas de este módulo que no se tocan sin entender por qué:**
>
> 1. **Se cifra por trozos**, no el cuerpo entero. De una vez obliga a tenerlo
>    en memoria dos veces —texto y cifrado—, y el techo del presupuesto son
>    500 MB. Además el troceado cierra dos agujeros: **reordenar trozos** y
>    **cortar el final**, que daría un respaldo más corto y perfectamente válido.
>    El último trozo va marcado como último; quitarlo hace que falle.
> 2. **La sal y el nonce entran como parámetros.** El núcleo no pide entropía al
>    sistema (cero I/O) y el cifrado se prueba con vectores fijos. Una función
>    que genera su propia aleatoriedad sólo se puede probar cifrando y
>    descifrando, que no distingue «cifra bien» de «cifra y descifra mal igual».
> 3. **Contraseña equivocada y archivo manipulado dan el MISMO error**, y es
>    correcto: el cifrado autenticado no los distingue. Fingir que sí sería
>    inventarse un diagnóstico.
>
> **La contraseña del respaldo no se guarda en ningún sitio** — ni llavero, ni
> base, ni vuelta por la frontera. Si se olvida, el respaldo no se abre.

**Cerrada la 8.2b · leer la base por tandas** (`arles-db/src/respaldo.rs`):
`Db::exportar` recorre las tablas del alcance y le da cada fila a un rasgo
`Destino`. Trece pruebas, dos comprobaciones nuevas en `--fase 8`.

> **«Por tandas» era la unidad equivocada, y la forma obvia es la mala.** La
> tanda obvia es `LIMIT 500 OFFSET n`, y SQLite no guarda un atajo a la fila n:
> medio millón en tandas de 500 visita unas **125 000 millones** de filas para
> leer 500 000. Y entre dos tandas la tabla cambia —el motor escribe en su
> hilo—, así que **una fila se repite o se salta sin que nada falle**. Lo que es
> memoria acotada es un **cursor**; las tandas sólo sirven para avisar del
> avance, cada 2 000 filas. Lo vigila `el_respaldo_no_se_lee_en_memoria`.
>
> **Todo el respaldo va en una transacción de lectura**, o el recuento de la
> cabecera y las filas del cuerpo serían de instantes distintos y el preflight
> de la 8.3 enseñaría una cifra que el archivo no cumple. A cambio, el motor no
> escribe mientras se exporta: el cerrojo es uno solo.

> **Dos defectos que encontró medir el esquema, y que ninguna prueba del
> manifiesto podía cazar.** Son el patrón del §9 con otra cara: cada capa pasaba
> lo suyo y el hueco vivía en medio.
>
> 1. **El orden del manifiesto no se puede restaurar.** Deja tres hijos antes
>    que sus padres —`contact` antes de `import_batch`, `campaign_stage` y
>    `message_attempt` antes de `email_account`—, y con `foreign_keys=ON` esos
>    `INSERT` se rechazan. El orden del cuerpo **se calcula** de
>    `PRAGMA foreign_key_list`, con el manifiesto como desempate para que dos
>    exportaciones de la misma base den el mismo archivo.
> 2. **`import_batch` estaba fuera del respaldo esencial** porque «es
>    trazabilidad, no dato operativo». Falso: `contact` tiene una clave ajena a
>    ella, así que el esencial traía los contactos y **no se podía restaurar**.
>    Lo vigila `ningun_hijo_viaja_sin_su_padre`, que lee el esquema y comprueba
>    los dos alcances — incluido el caso de un padre que sí viaja pero recortado.
>
> **Al clasificar una tabla, la pregunta no es sólo «¿hace falta?» sino «¿de
> quién depende?».** Una tabla sin su padre no es un respaldo incompleto: es un
> respaldo que no entra.

> **Y un arreglo del formato de la 8.1**: el cuerpo no decía qué columna es cada
> valor, así que sólo se podía restaurar **por posición**. Una migración que
> mueva una columna insertaría los campos corridos —el apellido donde va la
> empresa— sin que una sola fila falle. La cabecera declara ahora las columnas
> de cada tabla; la versión del formato **no sube**, porque no hay un solo
> `.arles` escrito fuera de las pruebas.
>
> **La columna por la que se recorta vive dentro de la variante**
> —`RecortadaEnElEsencial { fecha }`—: `whatsapp_reply` no tiene `created_at`
> sino `first_reply_at`, y darlo por supuesto habría hecho fallar el `WHERE`
> delante de quien pulsó exportar.

**Cerrada la 8.2c · el comando, el diálogo y la pantalla**
(`arles-app/src/respaldo.rs` y `/ajustes/respaldos`). **ARLES escribe un
respaldo entero**, cifrado o no. El hueco del §9 que la 8.2b dejó abierto lo
cerró `el_exportador_tiene_quien_lo_llame`, escrita **antes** y vista en rojo.

> **La cabecera va en claro y el cuerpo cifrado.** Es lo que permite que la 8.3
> diga «este respaldo va con contraseña, y es de hace cuatro meses» antes de
> pedirla: cifrarla obligaría a pedir la contraseña para saber si hace falta una
> contraseña. Y no filtra nada — versión del esquema, recuento por tabla y
> nombres de columna; ni un dato de una persona, ni una ruta.
>
> **La suma va dentro del sobre y cubre la cabecera Y el cuerpo en claro**, que
> es lo que hay que poder comprobar al restaurar.

> **Dos arreglos más del formato de la 8.1**, del mismo tipo que el de la 8.2b:
> cosas que la 8.1 no podía ver porque el cifrado no existía todavía.
>
> 1. **La sal y el nonce no tenían dónde ir.** La clave sale de la contraseña
>    **y de la sal**, así que sin ellas en el archivo un respaldo cifrado **no
>    se abre nunca**. Van dentro de la variante
>    `Cifrado::Argon2idXChaCha20 { sal, nonce }` para que no exista el estado
>    imposible. No son secretos: lo que protege es la contraseña, que no está
>    en el archivo.
> 2. **`cifrar` recibía el cuerpo entero en memoria**, deshaciendo el cursor de
>    la 8.2b. Es el §9 en su forma más silenciosa: **las dos piezas correctas y
>    el agujero en la junta**, que ninguna prueba de una u otra podía cazar. Lo
>    cierra `SobreEnCurso`, y produce **el mismo archivo byte a byte** que
>    `cifrar` — hay que retener un trozo lleno hasta saber si viene más, porque
>    el último se marca como último; sellarlo en cuanto se llena daría otro
>    formato con los múltiplos exactos de 64 KiB.

> **Tres defectos que encontró mirar la demostración**, con las diez fases en
> verde:
>
> - **«12536 filas»** sin separador, cuando Actividad y el registro escriben
>   `toLocaleString('es-MX')` desde la 7.2. Ninguna comprobación lo caza: es un
>   número correcto.
> - **«Repite la contraseña» bloqueado sin decir por qué.** Bloquearlo no es
>   cosmética — si sólo se escribiera ahí, el respaldo saldría **sin cifrar**
>   mientras alguien cree haberle puesto clave—, pero un campo que no responde
>   y no se explica se lee como una avería (§95).
> - **`--arles-font-family-mono` no existe.** Ésta sí la cazó el validador: el
>   navegador resuelve un `var()` inexistente **a nada, sin error**, así que la
>   lista se habría visto bien. Es el caso de los seis tokens de la 5.2.

**Cerrada la 8.3a · leer un `.arles` de vuelta**: el sobre que se abre a trozos
(`SobreQueSeAbre`) y el preflight (`arles-db/src/restauracion.rs`). Diez
pruebas. **Todavía no se restaura nada**: eso es la 8.3b.

> **El descifrado a trozos es la misma junta que ya falló una vez.** `descifrar`
> recibía el cuerpo entero, igual que `cifrar` antes de la 8.2c: restaurar medio
> millón de filas tendría el archivo **y su versión en claro** en memoria a la
> vez. Se escribió antes de que apareciera el problema, porque ya se sabía dónde
> estaba.

> **D-8C no significa lo que parece, y lo descubrió medir el esquema.**
> «Reemplazar, nunca mezclar» suena a que cada tabla del respaldo sustituye a la
> suya y lo demás se queda igual. **No es lo que pasa**: `company` se reemplaza
> en los dos alcances y casi todo cuelga de ella con `ON DELETE CASCADE`.
> Restaurar un **esencial** se lleva por delante `activity_event` y
> `whatsapp_quality_event` —que son `SoloCompleto` y **no vuelven**—, más
> `rate_budget` y `engine_control`, que son de este equipo y está bien perder.
>
> Es el fallo que en la Fase 1 costó la audiencia congelada y el registro de
> envíos (F1). No se arregla tocando el esquema —las cascadas protegen de otra
> cosa— ni escondiéndolo: **el preflight lo cuenta y lo dice** antes de que
> nadie confirme, y separa lo que son datos de lo que es estado del equipo.
>
> La cascada **se calcula del esquema** y no se da por buena:
> `la_cascada_calculada_es_la_que_ejecuta_sqlite` borra de verdad dentro de una
> transacción que se deshace y compara. Un cálculo corto callaría una pérdida;
> uno largo asustaría con una que no ocurre.
>
> **Las bajas prometen un mínimo, no una cifra exacta.** La unión exacta exige
> leer el cuerpo entero; el preflight lee sólo la cabecera a propósito. Lo que
> importa es que ese número **nunca baja**.

**Cerrada la 8.3b · reemplazar, con la unión de bajas** (`Db::restaurar`). Ocho
pruebas. **Sigue sin haber botón**: los comandos y la pantalla son la 8.3c.

> **Entero o nada, probado provocando el fallo**: el origen revienta en la
> tercera fila, con la base ya vaciada y dos filas metidas, y la prueba compara
> la huella de todas las tablas antes y después.

> **Las bajas se unen en SQL, no en memoria** —una tabla temporal dentro de la
> misma transacción—, y vuelven con un **identificador nuevo**. Lo que hace
> única a una baja es (empresa, canal, dirección), no su `id`: reinsertando con
> el original, el choque sería contra la clave primaria y el `OR IGNORE` se
> **tragaría la baja en silencio**.

> **Una baja cuelga de su empresa, y eso casi rompe D-8C.** Lo encontró la
> prueba, no el diseño: `suppression_entry` tiene clave ajena a `company`, que
> también se reemplaza. Si el respaldo trae otra empresa, las bajas de aquí
> apuntan a una que ya no existe y **la restauración entera se deshace**.
> Descartarlas pierde bajas; dejar que falle convierte D-8C en «no se puede
> restaurar». Así que se **reapuntan** a la empresa que quedó: ARLES es de una
> sola empresa, y en el caso raro el error cae del lado seguro —de más, nunca de
> menos—. Si el respaldo no trae ninguna empresa, **se para**.

> **Las cuentas vuelven en `needs_reauth`**, y las filas entran **por nombre de
> columna**; una columna que esta base no tiene para la restauración en vez de
> insertar las que encajen.

> **Un respaldo de otro esquema se rechaza, y es una limitación real.** Hoy, un
> respaldo tomado antes de una migración futura **no se podrá restaurar**: haría
> falta correr las migraciones sobre el propio respaldo, que es otra entrega.
> Entre eso y restaurar «lo que encaje», se para.

**Cerrada la 8.3c · los comandos y la pantalla** (`arles-app/src/restauracion.rs`
y `/ajustes/restaurar`). **ARLES restaura un respaldo entero.** Seis pruebas, y
`el_restaurador_tiene_quien_lo_llame` escrita **antes** y vista en rojo nombrando
los seis tramos.

> **Dos comandos, y el orden no se puede saltar.** `revisar_respaldo` lee sólo
> la cabecera y dice qué trae y qué se pierde; `restaurar_respaldo` usa la ruta
> que el primero guardó en `EstadoApp`. La ruta no cruza la frontera (regla 4),
> y eso además impide que la webview pida restaurar **otro** archivo distinto
> del que se revisó.

> **La suma se comprueba DENTRO de la transacción**, al llegar a la línea de
> cierre. Un archivo alterado o cortado revienta antes del `COMMIT`. Al revés
> dejaría la base reemplazada con datos de un archivo que no salió de aquí, y un
> aviso después.

> **La prueba que la fase existía para tener**: base poblada → archivo → **otra
> base** → los mismos contactos, con y sin contraseña. Ninguna entrega anterior
> podía escribirla: la 8.2c comprobaba que el archivo se relee, la 8.3b que la
> base se reemplaza, y **nadie** que lo que sale de una entre en la otra.

> **Tres defectos que encontró mirar la demostración**, con las diez fases en
> verde: el botón de elegir medía 540 px y los de abajo 200 y 96; «Restaurar y
> reemplazar» salía apagado **sin decir que faltaba la contraseña** (§95, y el
> mismo defecto que la 8.2c con «Repite la contraseña»); y «2 cuenta(s)» con el
> paréntesis, ahora pluralizado como Remitentes.
>
> Y uno que cazó el validador: **tres tokens de diseño inventados** —
> `--arles-text-primary`, `--arles-text-secondary`, `--arles-border-subtle`.
> **Tercera vez en esta fase.** Los de verdad son `--arles-text`,
> `--arles-text-muted` y `--arles-border`.

> **Lo que todavía NO hace:** archivar el historial a un disco externo (D-8D,
> la 8.3 bis), y **restaurar un respaldo de un esquema anterior** — hoy se
> rechaza con un error claro, y arreglarlo exigiría correr las migraciones sobre
> el propio respaldo.

**Cerrada la 8.4 · el llavero perdido** (`arles-app/src/recuperacion.rs` y
`/recuperar`). Cinco pruebas, y `la_clave_perdida_tiene_salida` escrita **antes**
y vista en rojo nombrando los seis tramos.

> **El consejo que el producto no podía cumplir.** Desde la Fase 1, perder la
> entrada del llavero daba un mensaje correcto con las tres partes del §95, y la
> de «cómo se arregla» decía *«Restaura tu último respaldo»* — y `run()` llamaba
> a `exit(1)` antes de abrir ninguna ventana. No faltaba un botón: el producto
> **daba una instrucción que él mismo impedía seguir**.

> **Las dos cosas que aquí no se pueden hacer mal:**
>
> 1. **No se crea una clave nueva al arrancar** (F2 de la Fase 1). El llavero
>    puede estar vacío por motivos pasajeros, y una clave nueva dejaría la base
>    ilegible **para siempre**. Sólo nace cuando alguien elige un respaldo y
>    confirma.
> 2. **La base ilegible se aparta, no se borra.** Un llavero puede volver, y ese
>    día esos datos valen más que el respaldo. Recuperar dos veces el mismo día
>    tampoco pisa la copia de la primera.

> **El modo registra tres comandos y ninguno más**, y la pantalla no lleva barra
> lateral: sin base no hay secciones a las que ir. **Si no hay respaldo se dice
> con esas palabras** — no hay salida, el cifrado no tiene puerta de atrás—,
> porque dejar lugar a la duda haría buscar durante horas algo que no existe.

> **Límite declarado.** Este contenedor no tiene llavero ni D-Bus, así que la
> prueba del arranque **declara su rama y se omite aquí**. Lo que sí corre
> entero es la mecánica de la recuperación, escrita a propósito **sin** depender
> del llavero: con él habría pasado en verde sin ejercitar una línea. En Windows
> y macOS se prueba al cerrar **R-07**.

> **Y el defecto que encontró mirar la demostración**, el peor posible en esta
> pantalla: decía **«no se ha perdido nada»** con 48 MB de base en disco, porque
> nadie llamaba a `cargar()`. El §9 otra vez, y aquí la consecuencia no es una
> función que falta: es alguien que borra el archivo porque le dijimos que
> estaba vacío.
>
> Y **los derechos ARCO, que el ROADMAP pone en la Fase 8, están construidos
> desde la 3.4** — se verifican en la auditoría B, no se reconstruyen.

**Cerrada la 8.5 · el banco de pruebas y las trece cifras**
(`crates/arles-banco`). Medio millón de contactos sintéticos reproducibles, y
**las trece cifras medidas o declaradas** en `PRESUPUESTO_RENDIMIENTO.md` §0.

> **Cinco medidas, ocho sin medir, y dos de las cinco fallan** — el plan lo
> anticipaba. Importar 500 000: 51.7 s de 5 min ✅. Memoria: 497 MB de 500 ⚠️.
> Simulación: 5.9 ms de 200 ✅. **Búsqueda por correo: 331 ms de 100 ❌.**
> **Abrir la tabla: 1 424 ms de 500 ❌.**

> **Los dos fallos tienen causa concreta**, con `EXPLAIN QUERY PLAN` y no
> supuesta:
>
> 1. **`contact` no tiene ningún índice para su propia consulta.** Sólo están el
>    de la clave primaria y uno sobre `import_batch_id`. Para enseñar **cien**
>    filas, SQLite lee las 455 864 y las ordena en memoria (`SCAN contact` +
>    `USE TEMP B-TREE FOR ORDER BY`), y el recuento hace un segundo barrido.
> 2. **La búsqueda por correo no filtra por `channel`.** El índice único es
>    `(company_id, channel, value_normalized)`, así que sin la segunda columna
>    sólo se aprovecha la primera y se recorren **todos los canales** de la
>    empresa.
>
> Las dos se arreglan con una migración V11 — **y eso deja sin restaurar los
> respaldos V10** (ver 8.3b). Hoy no hay ninguno fuera de las pruebas, así que
> el coste es cero; dentro de un mes de uso, no. **Pendiente de Dirección.**

> **497 MB de 500 no es aprobar.** `TablaLeida` guarda todas las filas en
> memoria a propósito —la importación las recorre dos veces—, así que el margen
> con 500 000 filas es del 0.6 % y con 600 000 no habría. El techo declarado es
> medio millón, así que hoy no es un defecto; es la cifra que primero se rompe.

> **El banco no viaja en el instalador** —es un crate del que `arles-app` no
> depende— porque uno de los trece presupuestos es el tamaño del instalador, y
> meter dentro un generador de medio millón de filas para medirlo sería el
> instrumento falseando lo que mide. **El azar es propio** (SplitMix64, diez
> líneas): una biblioteca promete calidad estadística, no que su algoritmo no
> cambie entre versiones, y lo que hace falta aquí es reproducibilidad.

> **Lo que el banco encontró de sí mismo**: su primera versión se saltaba el
> análisis de choques y reventó en la primera corrida con medio millón. No era
> «medir sin la interfaz»: era medir un camino que el usuario nunca recorre.

> **Y dos defectos de las propias comprobaciones**, los dos del §2: el patrón de
> fecha tenía la alternancia sin paréntesis y casaba «20» en cualquier parte
> —daba por fechado un documento sin fechas—, y la de reproducibilidad **se
> cazaba a sí misma en un comentario**, saliendo en rojo sobre un generador
> correcto. Es la trampa que la 5.3 ya documentó, repetida.

**Cerrada la 8.6 · la migración V11 y los planes de consulta**
(`crates/arles-db/migrations/V11__indices_de_rendimiento.sql`). **Los dos
presupuestos que la 8.5 midió en rojo están en verde**, medido el 06/10/2026
sobre los mismos 500 000 contactos: abrir la tabla **1 424 ms → 357 ms**, y
buscar por correo **331 ms → menos de 1 ms**. Seis índices, tres pruebas
nuevas, y la fase 8 en 12/12.

> **El precio está medido y no se esconde: importar 500 000 pasa de 51.7 s a
> 91.4 s** (+78 %, confirmado en tres corridas: 91.1, 93.8, 91.4). Cada fila
> que entra hay que anotarla también en los índices nuevos. Se paga **una vez
> al importar** para que la pantalla que se abre a diario tarde un tercio, y
> usa el 30 % de los 5 minutos de presupuesto. Se consideró tirar los índices
> antes de importar y recrearlos después: **se descartó**, porque recrear un
> índice cuesta en proporción a las filas **totales**, no a las nuevas, así que
> importar mil contactos sobre medio millón saldría peor.

> **La auditoría recorrió las 295 sentencias SQL del proyecto** y encontró 23
> `SCAN` sobre tablas que crecen. Lo importante es lo que **no** encontró: el
> **camino de envío ya estaba bien indexado** —supresión, cola y registro—. El
> motor nunca fue el problema; lo eran dos pantallas.

> **Cada índice se escribió DESPUÉS de ver que cambia el plan**, con
> `EXPLAIN QUERY PLAN` y no por intuición. **Uno de los diseños no lo
> cambiaba** y se descartó. Lo vigilan las tres pruebas de
> `tests/planes_de_consulta.rs`, que leen el plan de nueve consultas críticas y
> fallan si alguna vuelve a hacer `SCAN`, o si la lista de contactos vuelve a
> ordenar en memoria (`USE TEMP B-TREE FOR ORDER BY`).

> **El defecto más grave de esta entrega no está en el producto: está en la
> verificación.** `refinery::embed_migrations!` **no provocaba recompilación al
> cambiar un `.sql`**, así que cada «rompe y comprueba» del §2 sobre una
> migración probaba el SQL **anterior** — pasaba en verde sin ejercitar nada. Lo
> arregla un `build.rs` con `rerun-if-changed=migrations`, y lo vigila
> `cambiar_una_migracion_recompila`. Invalida verificaciones que se dieron por
> hechas antes de hoy.

> **Y un comentario que afirmaba una verificación falsa**: decía que invertir el
> índice a `created_at ASC` devolvería el ordenado en memoria. **No lo hace** —
> SQLite recorre un índice hacia atrás igual de bien—. Lo que sí rompe el plan
> es quitar `created_at` de la clave, o mezclar direcciones (`ASC, DESC`); el
> comentario lleva ahora esa ruptura, que es la que se vio en rojo.

> **Al añadir una migración, hay que mirar qué pruebas fijan el esquema a
> mano.** Varias escribían `"V10"` como literal, así que **toda migración
> futura las rompería**; ahora leen `ultima_version()` del propio motor. El
> `"V9"` que queda es deliberado: es la prueba de que un respaldo de otro
> esquema se rechaza.

> **Lo que esto cuesta:** un respaldo tomado con el esquema V10 **ya no se
> puede restaurar** — la limitación que la 8.3b dejó declarada, y esta
> migración es la primera que la dispara. Hoy no hay ningún `.arles` fuera de
> las pruebas, así que el coste real es cero; dentro de un mes de uso no lo
> sería.

**Cerrada la 8.3 bis a · archivar el historial, por dentro** (D-8D):
`EN_EL_ARCHIVO` en el núcleo, el tercer alcance del formato con su propia marca,
y `arles-db/src/archivo.rs` con el preflight y el borrado. Ocho pruebas y tres
comprobaciones nuevas en `--fase 8`, escritas antes y vistas en rojo.
**Todavía no hay botón**: la pantalla y los comandos son la 8.3 bis b.

> **El archivo lleva DOS tablas: el historial y las campañas.** La segunda no es
> relleno: un intento sin su campaña queda como una dirección y un desenlace,
> sin a qué envío pertenecía. Hoy la campaña está en la base, pero el archivo
> tiene que servir **dentro de tres años**, y entonces puede no estar. Son
> decenas de filas y sin ellas el archivo no contesta un ARCO, que es lo único
> para lo que existe.

> **Un archivo de historial no se puede restaurar, y es la pérdida más grave que
> esta fase podía dejar abierta.** Restaurar reemplaza y nunca mezcla (D-8C), así
> que darle un archivo con un año de envíos **vaciaría la base entera** —
> contactos, bajas y campañas— y **nada protestaría**, porque reemplazar es lo
> que se le pidió.
>
> Se cierra con **dos puertas**: `MARCA_ARCHIVO` en la primera línea, y
> `Alcance::se_puede_restaurar()` que el restaurador pregunta aparte. La marca
> sola no basta: una cabecera se puede armar a mano, y entonces la marca la pone
> quien escribe. Y la cabecera **exige que las dos digan lo mismo**: un archivo
> de historial con la marca de un respaldo se rechaza en vez de creerle a una de
> las dos.

> **El tercer alcance le quitó el `Copy` al tipo**, y es a propósito.
> `Historial { hasta }` lleva la fecha dentro porque es un dato de ejecución. La
> otra forma —la fecha en un campo aparte de la cabecera— habría dejado `Copy` y
> habría creado el estado imposible: un alcance de historial **sin periodo**. Es
> la misma razón que `RecortadaEnElEsencial { fecha }` en la 8.2b. El precio fue
> cuatro `.clone()`; el de la otra forma, una pérdida silenciosa.
>
> **Y añadir la variante hizo que el compilador exigiera decidir** en los dos
> sitios del restaurador, que es exactamente donde la decisión importaba. Un
> `_ =>` ahí habría dejado el archivo caer en «no se lleva nada», y un reemplazo
> que no trae nada es una base vacía.

> **Borrar el historial arrastra el registro de actividad, y el archivo no lo
> lleva.** `activity_event.message_attempt_id` cuelga de `message_attempt` con
> `ON DELETE CASCADE`, y `activity_event` no entra en el archivado porque ya se
> recorta cada dos meses (7.4). Así que **eso sí es una pérdida**, y el preflight
> la cuenta antes de que nadie confirme. Es el fallo de la Fase 1 (F1) por
> **tercera vez**, y la única defensa que ha funcionado las tres es la misma:
> leer la cascada del esquema y contarla, no tocarla.
>
> `la_cascada_calculada_es_la_que_borra_sqlite` borra de verdad dentro de una
> transacción que se deshace y compara. Y compara **la cifra**, no sólo los
> nombres: una lista a mano con el número mal pasaría una comparación de nombres.

> **El borrado se niega si el historial cambió.** Recibe cuántas filas escribió
> el archivo y falla si la base ya no tiene exactamente ésas. El motor envía en
> su propio hilo (§51): entre escribir el archivo y confirmar puede haber entrado
> un intento con fecha anterior al corte. Borrar «lo que haya» se llevaría una
> fila que el archivo no tiene, y **el recuento final cuadraría consigo mismo**.

> **El corte parte el historial sin solaparse y sin dejar hueco.** El archivo se
> lleva `< hasta` y la base se queda `>= hasta`, escritos en dos funciones
> distintas. Cambiar uno a `<=` no hace fallar nada: pierde o duplica
> exactamente un día. Lo vigila una prueba con un envío **en el instante exacto
> del corte**, que es el único que distingue las dos fronteras.

> **El filtro de «qué tablas se lleva este alcance» estaba escrito tres veces** —
> exportador, preflight y la prueba `ningun_hijo_viaja_sin_su_padre`—. Ahora es
> `tablas_incluidas()`. Tres copias del mismo `match` son tres sitios donde
> añadir un alcance y olvidar uno; y el peor de los tres es la prueba, porque
> entonces **vigila un alcance que no es el que corre**.

> **Tres de las comprobaciones nuevas estaban mal, y las tres lo encontró
> romperlas de verdad** en vez de darlas por buenas. Es el §2 aplicado a sí
> mismo, y el resultado del día:
>
> 1. Una buscaba el nombre de un error **inventado al escribir la
>    comprobación** —`HistorialCambiado`— cuando el del código es
>    `ElHistorialCambio`: en rojo sobre código correcto.
> 2. Otra buscaba `foreign_key_list` en el archivo entero y **se cazaba en la
>    cabecera del módulo**, que lo nombra al explicar por qué la cascada se lee
>    del esquema. Al cambiar el `PRAGMA` de verdad, seguía en verde. **Tercera
>    vez** (5.3 y 8.5 las otras). Se quitan los comentarios antes de mirar.
> 3. La tercera decía vigilar las dos puertas del restaurador y vigilaba **una**,
>    y su comentario afirmaba estar verificada quitando la otra. Era falso: al
>    quitarla, el `match` del preflight seguía nombrando la variante. Ahora cada
>    puerta se busca por lo que la distingue.
>
> **Una afirmación de verificación es una afirmación como cualquier otra**, y en
> este proyecto llevan tres entregas siendo las que más veces resultan falsas.

> **Y un error mío de herramienta, que costó trabajo rehecho:** un
> `git checkout` dentro de un bucle de rupturas revirtió `formato_de_respaldo.rs`
> **entero** —no sólo la ruptura—, y se llevó el alcance nuevo y sus tres
> pruebas. Para restaurar una ruptura se usa una copia del archivo hecha antes,
> nunca `git checkout` sobre un archivo con trabajo sin confirmar.
>
> **Y volvió a pasar el 23/10/2026, con D-16**, sobre `cola.rs`: se llevó la
> consulta del descanso, el aparcado y su helper. **Saberlo no basta** — es la
> misma conclusión que los comentarios que se cazan a sí mismos. La copia se
> hace **antes de la primera ruptura**, siempre, y se restaura desde ella.

**Cerrada la 8.3 bis b · el botón de archivar** (`arles-app/src/archivo.rs` y
`/ajustes/archivar`). **ARLES archiva el historial entero**: lo escribe, lo
vuelve a leer, lo verifica y sólo entonces borra. Seis pruebas, y
`el_archivado_tiene_quien_lo_llame` escrita **antes** y vista en rojo nombrando
los siete tramos.

> **La relectura usa el lector de la restauración, no uno escrito para esto.**
> Lo que hay que poder afirmar no es «el archivo se escribió», es «el archivo se
> podrá abrir el día que haga falta» — y ese día lo abrirá `LeeElArchivo`. Da
> tres cosas de una pasada: la cabecera se entiende, cada fila se vuelve a
> partir en sus campos —ahí se cazaría un escapado mal hecho en un nombre con
> tabuladores— y la suma cuadra. Las filas se cuentan y se tiran: medio millón
> se recorre sin tenerlo en memoria.

> **Se comprueban las filas totales, no sólo las del historial.** Comparar sólo
> los envíos dejaría pasar un archivo **sin las campañas**: con su suma
> correcta, y sin con qué contestar un ARCO.

> **El archivo va sin contraseña, y es una decisión, no un olvido.** Un respaldo
> se restaura dentro de ARLES, que puede pedirla. Un archivo de historial se
> abre años después, puede que en otro equipo, para contestar una solicitud de
> datos. La contraseña del respaldo no se guarda en ningún sitio (8.2a): aquí,
> una que nadie recuerde dejaría el archivo igual que no tenerlo — y el
> historial se habría borrado para nada. Cifrarlo con la clave del equipo
> tampoco vale: vive en el llavero de **este** equipo, y el archivo tiene que
> sobrevivir a que este equipo no exista. La pantalla lo dice con esas palabras.

> **No se escribe un archivo vacío.** Si no hay nada anterior al corte se para
> con su error: un `.arles` de cero filas en un disco externo es una copia de
> seguridad que alguien contará como copia, y el día que la busque no habrá nada
> — y el archivo existiendo es lo que le habrá impedido darse cuenta antes.

> **El mínimo del corte es un año y la frontera no se fía de la pantalla.** Un
> `meses` menor se **sube** al mínimo en vez de obedecerse. Doce porque es el
> periodo que se consulta y el que el respaldo esencial guarda: archivar lo que
> el esencial sí se lleva dejaría el historial reciente fuera de la base y
> dentro de dos archivos distintos.

> **Tres cosas que encontró mirar la pantalla**, con todo en verde:
>
> - **No decía «no se puede deshacer»** en ninguna parte. Los tres avisos decían
>   qué se pierde, de quién pasa a ser el archivo y en qué orden se hace; la
>   frase llana faltaba. Restaurar, que hace lo mismo de grave, sí la dice en su
>   entradilla.
> - **No había forma de arrepentirse**: debajo de las cifras sólo estaba el
>   botón rojo. Restaurar tiene su «Mejor no» al lado.
> - **«Con sus campañas · 3 campañas»**: la misma palabra dos veces en la fila.

> **Y el cuarto defecto de una comprobación propia en el mismo día.** La del
> **orden** —verificar antes de borrar— lo medía comparando dónde aparece cada
> nombre **en el archivo**, y salió en rojo sobre código correcto: la relectura
> vive en una función escrita después aunque se ejecute antes. **El orden del
> archivo no es el orden de ejecución.** Ahora se mide dentro del cuerpo de
> `archivar_historial`, que es donde esa pregunta tiene respuesta. Verificada
> rompiéndola en los dos sentidos: borrar sin verificar, y verificar después.

**Cerrada la 8.6 A · la auditoría de arquitectura** (§140, pasada 1 de 4).
**Cinco hallazgos, cinco arreglados**, con las diez fases en verde al empezar.
Está contado en
[`AUDITORIA-A-ARQUITECTURA.md`](documentacion/09-fases/AUDITORIA-A-ARQUITECTURA.md).

> **Cuatro de los cinco no son defectos del producto: son defectos de sus
> comprobaciones.** Las cuatro decían cubrir una regla y cubrían una parte, y la
> parte que faltaba era justo la que nadie había roto:
>
> 1. **El cero I/O del núcleo no miraba el reloj ni el azar**, y las dos reglas
>    están escritas en tres sitios del proyecto. Verificado metiendo un
>    `OffsetDateTime::now_utc()` en `arles-core`: **45 pasan, 0 fallan**. Y el
>    `os.listdir` no entraba en subcarpetas. Reescrita como
>    `el_nucleo_no_toca_el_mundo`: `os.walk`, quince patrones, y salta las
>    pruebas.
> 2. **El `forbid(unsafe_code)` se vigilaba en 2 de 7 crates**, por una lista a
>    mano. Ahora sale de las carpetas que hay. `arles-banco` no lo tenía.
> 3. **El grafo de dependencias se vigilaba con una arista de doce.**
>    `el_grafo_de_crates_solo_apunta_hacia_dentro` declara qué puede depender de
>    qué y lo compara con los `Cargo.toml`; un crate nuevo sin fila también
>    falla.
> 4. **`AVISO_DIARIO` y los meses del historial viven en Rust y en TypeScript**,
>    duplicados a propósito —la pantalla avisa antes de llamar a nadie— y **sin
>    nada que los mantuviera iguales**. Si se separan, el asistente pide aceptar
>    un aviso que el núcleo ya no exige, o no lo enseña y `crear_campana`
>    rechaza al final nueve pasos de formulario.
>
> Y el quinto: **un `unreachable!()` en producción**, el único panic de los ocho
> crates. Correcto —la rama acaba de ver `Some`— y evitable con un `if let`.

> **El quinto defecto de comprobación apareció verificando el arreglo del
> segundo.** La primera ruptura fue **comentar** el `#![forbid(unsafe_code)]`, y
> la comprobación nueva siguió en verde: la cadena seguía ahí, dentro del
> comentario. **Quinta vez** (5.3, 8.5 y dos en la 8.3 bis). Si la ruptura se
> hubiera dado por hecha, habría entrado una comprobación rota **junto con el
> arreglo que venía a corregir otra igual**.

> **Lo que salió limpio, y conviene dejarlo escrito:** `arles-core` no toca el
> mundo, el motor no escribe SQL fuera de sus pruebas, `arles-db` no depende de
> `serde`, el grafo es un DAG que coincide con el documentado —los adaptadores
> dependen del motor y no al revés— y hay cero `any` y cero `@ts-ignore` en el
> frontend.

**Cerrada la 8.6 B · la auditoría de seguridad** (§140, pasada 2 de 4). Los
**quince puntos** de `THREAT_MODEL.md` §8, medidos uno a uno. **Dos hallazgos
arreglados y tres puntos del documento corregidos.** Está contado en
[`AUDITORIA-B-SEGURIDAD.md`](documentacion/09-fases/AUDITORIA-B-SEGURIDAD.md).

> **La regla 3 de la frontera —ningún comando devuelve un secreto— se vigilaba
> en 17 de 52 estructuras.** `nada_que_cruce_la_frontera_lleva_contrasena` leía
> **sólo `comandos.rs`**, y la frontera había crecido a once módulos. Verificado
> metiendo un `pub contrasena: String` en una estructura de `respaldo.rs`: **pasó
> en verde**. Ahora lee `arles-app/src` entero y busca seis nombres —se añadieron
> `token`, `api_key` y `credencial`—, verificado rompiéndolo en cuatro módulos.
>
> `lleva_contrasena: bool` queda fuera a propósito: dice si hace falta una, no
> cuál es. Y lo que esta comprobación **sigue sin cubrir** está dicho desde la
> Fase 5: busca el **nombre** del campo, así que un `clave_de_acceso` se le
> escapa — a ése lo caza la prueba que serializa y busca el **valor**.

> **El saneado «al usar» no existe, y no es un olvido.** `Mensaje` lleva sólo
> destinatario y clave de idempotencia, y el único `Contenido` que llega al SMTP
> es un texto fijo escrito en el código. **El cuerpo de una campaña no llega al
> remitente todavía.**
>
> El hallazgo es lo que pasará el día que se conecte: el camino nuevo pasará por
> ahí **sin que nada pida sanear otra vez**, y entre guardar y enviar está la
> sustitución de variables, que mete datos de un XLSX ajeno **dentro del HTML ya
> saneado**. Es el §9 aplicado a una regla de seguridad. Lo vigila
> `nada_sale_por_correo_sin_pasar_por_el_saneado`, que comprueba que `Contenido`
> se arme de constantes y **falla el día que deje de ser cierto**.

> **Tres puntos del §8 no se podían marcar con verdad**, y un punto que nunca se
> puede marcar es peor que uno que falta: hace dudar de la lista entera.
>
> - Uno pedía la «capa de redacción de `tracing`» y **el proyecto no usa
>   `tracing`**. Lo que sí existe es `redactar_rutas` en `ErrorIpc`.
> - **No hay exportación CSV** —sólo `.txt` y `.arles`—, así que la
>   neutralización de fórmulas no aplica.
> - **No hay updater**, así que su firma no se puede verificar.
>
> Los dos últimos quedan `[–] no aplica` **con fecha y condición de
> reactivación**, no borrados.

> **Lo que salió limpio:** cero concatenación en las consultas —los siete
> `format!` interpolan identificadores, que es lo único que SQLite no
> parametriza—; capabilities a `core:default` y nada más; `audit_log` imborrable
> **por esquema**, con dos *triggers* y una prueba que los ejercita; las
> respuestas SMTP recortadas y filtradas; `cargo deny` con `advisories` y
> `npm audit` en CI; y **los cuatro derechos ARCO con comando y pantalla**.

> **Y un error mío, del mismo tipo que los que la auditoría caza.** Busqué los
> comandos de ARCO por nombres **que me inventé** —`cancelar_contacto`,
> `rectificar`—, salieron a cero, y estuve a punto de anotarlo como hallazgo.
> Los de verdad son `cancelar_datos_de`, `que_se_borraria` y `editar_contacto`.
> **Un hallazgo falso en una auditoría de seguridad cuesta más que uno ausente:**
> manda a arreglar algo que funciona y gasta la credibilidad del resto.
> Antes de anotar que algo falta, se busca cómo se llama de verdad.

**Cerrada la 8.6 C · la auditoría de QA y rendimiento** (§140, pasada 3 de 4).
Los 32 casos frontera uno a uno, las trece cifras del presupuesto, y **la
corrida larga del motor, que nunca se había hecho**. Seis hallazgos: cuatro
arreglados, dos declarados con su cifra. Está contado en
[`AUDITORIA-C-QA.md`](documentacion/09-fases/AUDITORIA-C-QA.md).

> **Activar una campaña tarda 73.5 s de un presupuesto de 2 s**, y es el peor
> número del producto. Corre dentro de un comando de Tauri, en una transacción,
> con el cerrojo de la conexión tomado: la interfaz no contesta, no hay cifra de
> avance y no se puede cancelar. El §7 del presupuesto lo tenía escrito como
> señal de alarma desde la Fase 0 —«la interfaz se congela al activar una
> campaña»— y nadie lo había medido.
>
> Y **no es un problema del techo**: medido a cuatro escalas (18 k → 1.2 s,
> 45 k → 3.8 s, 136 k → 16.7 s, 451 k → 73.5 s), el presupuesto se rompe
> **alrededor de los 30 000 destinatarios**. Eso es una lista ordinaria.
>
> **Pendiente de Dirección**, con cuatro alternativas y una recomendación: un
> tope que diga su cifra ahora (horas) y sacarlo al hilo del motor en la Fase 9
> (días). Lo barato no es técnico: a 50 correos diarios (§48), el simulador dice
> que esa campaña **no termina en cinco años** y que **386 046 mensajes se
> quedarían sin enviar**.
>
> **Corrección del 07/10/2026:** esta línea decía «24 años», que era una cuenta
> a mano y estaba mal — el núcleo no devuelve una duración para ese caso,
> devuelve `NoCabeEnElHorizonte { faltan }`. Y decía que «el simulador ya se lo
> enseña en el paso 8»: **no lo enseña.** Ver el hueco de abajo.

> **Y la causa no era la que parecía, que es la razón de medir.** La hipótesis
> obvia eran los 451 196 pares de identificadores, cada uno pidiendo entropía al
> sistema. **Medido aparte: 0.58 s, el 0.8 %.** Lo que cuesta es escribir
> ~902 000 filas con sus índices sobre SQLCipher. Sin medirlo se habría
> «optimizado» la generación de claves de idempotencia —tocando ADR-0004, la
> parte más delicada del producto— para ganar medio segundo de setenta y tres.

> **Tres cifras estaban «sin medir» por un motivo que no se sostenía**, y un
> «sin medir» con un motivo falso es peor que un hueco: el hueco se ve, el
> motivo escrito se cree. «Exige un banco aparte» era una **molestia**, no un
> impedimento. «No existe el filtro» era verdad de la lista de contactos y falso
> del registro, que la 7.4 construyó con seis criterios. «Sin llavero no arranca
> la aplicación» era verdad de la **aplicación** y falso del **motor**.
>
> Los cuatro que quedan sin medir sí tienen motivo: necesitan una webview
> (R-07), los puertos 587/465/25 (R-24) o el empaquetador y Mont (P-01).

> **La corrida larga: 20 000 envíos, 13.9 MB en los cuatro cuartos, 0.00 MB de
> crecimiento.** No se mide un número, se mide una **forma** — la corrida se
> parte en cuatro tramos y se compara el último con el **segundo**, porque
> comparar con el primero metería dentro todo el arranque de la caché de SQLite
> y una corrida sana saldría en rojo.
>
> Y se vio fallar: con una fuga de verdad, 13.9 → 15.0 MB y +0.79 de
> crecimiento. La fuga no es inventada — es exactamente lo que hace
> `ProveedorSimulado`, que guarda cada pedido, y la razón por la que la corrida
> larga **no lo usa**: un instrumento que acumula acusaría al motor de una fuga
> que es suya.
>
> De paso: el motor hace **13 ms por envío**, unas 77 por segundo, contra los 50
> **diarios** que el producto permite.

> **Dos casos frontera llevaban desde la Fase 0 sin ningún test**, y el
> documento afirmaba de la lista entera que «cada uno tiene un test con nombre»
> **sin decir cuál** — así que no se podía comprobar. Ahora cada caso nombra el
> suyo y `cada_caso_frontera_nombra_su_test` falla si alguno desaparece.
>
> - **Disco lleno a mitad de importación.** Llenar un disco en una prueba no se
>   puede; **provocar su error, sí**: `PRAGMA max_page_count` da el
>   `SQLITE_FULL` de verdad. Verificada **partiendo la atomicidad**, no quitando
>   el tope — lo que hay que ver caer es la garantía, no el disparador.
> - **BOM, UTF-16, CRLF mezclado.** Funcionaba, y lo grave era **de qué
>   dependía**: del olfateo de BOM de `encoding_rs::decode`, que ninguna línea
>   del módulo menciona. Su comentario llama «Windows-1252» a esa rama. Cambiarla
>   a `decode_without_bom_handling` —el cambio natural para quien lea el
>   comentario— convierte cada importación en UTF-16 en basura silenciosa.
>
> **Y de las tres pruebas nuevas, dos no se pueden romper desde nuestro código**:
> el BOM de UTF-8 lo quita el lector de `csv` y los `\r\n` también. Vigilan a la
> biblioteca, no a ARLES. No son inútiles —si `csv` cambiara, las importaciones
> se romperían en silencio— pero hay que etiquetarlas, y así están.

> **El presupuesto describía una arquitectura con Tokio, y el proyecto no usa
> Tokio.** `PRESUPUESTO_RENDIMIENTO.md` §3 decía que el motor corre «en su
> propio conjunto de tareas de Tokio» y que la base «vive en un hilo dedicado
> con canal (ADR-0002), lo que elimina de raíz los `SQLITE_BUSY`». No hay Tokio
> en el taller, la conexión es un `Mutex<Connection>`, y el `busy_timeout(5 s)`
> sigue puesto.
>
> **Es el mismo defecto que la auditoría B encontró un día antes** en el
> `THREAT_MODEL.md` §8, que pedía «la capa de redacción de `tracing`» sin que el
> proyecto use `tracing`. Dos documentos de la Fase 0 nombrando bibliotecas que
> nunca se adoptaron, a una pasada de distancia.

> **Y un hallazgo falso mío, el segundo en dos pasadas.** Anoté que la cabecera
> de `arco.rs` ordena usar `neutralizar_formula` en un `.txt`, que sería un
> defecto. La frase lleva delante «**si algún día hace falta exportar en CSV**»
> y la cité sin ella: el comentario es correcto, una defensa dormida con su
> condición al lado. La auditoría B cometió el mismo error buscando comandos por
> nombres inventados. **Dar por sabido lo que dice el código en vez de leerlo
> entero** — dos veces, en la misma dirección.

**Cerrada la 8.6 D · la auditoría de UX y accesibilidad** (§140, pasada 4 de 4,
y con ella **las cuatro están hechas**). Las dieciocho pantallas medidas en
Chromium, no leídas. Cuatro hallazgos, cuatro arreglados. Está contado en
[`AUDITORIA-D-UX.md`](documentacion/09-fases/AUDITORIA-D-UX.md).

> **Las dos sondas que recorren el producto iban cinco pantallas por detrás, y
> el aviso llevaba ocho días escrito en una de ellas.** `auditoria.mjs`
> —contraste real, foco, desbordes, nombres accesibles, encabezados, ids— y
> `recorrido.mjs` llevaban **cada una su lista de rutas a mano**, parada el
> 29/09/2026 en trece. El producto tiene dieciocho.
>
> **Registro, Respaldos, Restaurar, Archivar y Recuperar nunca se habían
> medido** — las cuatro últimas son todo lo que construyó la Fase 8. Y la
> cabecera de `auditoria.mjs` dice desde el 29/09/2026: *«una auditoría que no
> recorre una pantalla no dice nada de esa pantalla, aunque termine en verde —
> que es peor que no tenerla, porque se cree»*. **Se escribió la lección y
> volvió a pasar cinco veces**, porque iba en un comentario y la lista seguía
> siendo a mano.
>
> Ahora salen de `rutas.mjs`, que las lee del router, y lo vigila
> `las_sondas_recorren_todas_las_pantallas`. **Medidas las dieciocho, las cinco
> pasan**: el hueco estaba en la medición, no en las pantallas.

> **`/recuperar` no tenía landmark, y eso hacía que dos medidas la saltaran en
> silencio.** Es la única pantalla que no pasa por el armazón de `App.vue` y
> abría con un `<div>`. Dos de las siete medidas se acotan a `main` —contraste y
> jerarquía de encabezados—, así que ahí **medían cero elementos** mientras la
> sonda la contaba como medida. La primera corrida dijo «no tiene ningún
> encabezado» y la pantalla **tiene un `h1`** desde que se escribió.

> **Cinco enlaces de navegación medían 20 px de alto**, el alto de su línea sin
> un píxel de relleno: «Ver los 6 pasos» en Inicio, tres de las salidas que la
> 7.3 añadió a Actividad, y una del Registro. A ojo un enlace corto no se ve
> pequeño, que es por lo que hacía falta medirlo.
>
> **Y «32 px al 200 %» se leía mal.** El escalado del sistema **no encoge los
> píxeles CSS**: encoge la ventana. Un portátil de 1920×1080 al 200 % le da al
> navegador **960×540**, y ésa es ahora una de las cuatro ventanas que la sonda
> prueba. Hasta hoy no se medía ni el tamaño de los objetivos ni ese escalado,
> aunque el §5 los pide desde la Fase 0.
>
> Arreglado con `--arles-objetivo-minimo: 32px`, porque la regla **no vivía en
> ningún sitio**. El suelo de ARLES es más alto que el 24 del WCAG 2.2 a
> propósito (§22).

> **Que ningún estado se distinga sólo por color no lo medía nada**, y es el
> punto que no se puede mirar a ojo: quien mira ve los colores. Ahora cada
> insignia tiene que llevar **texto** o nombre accesible. **No se exige icono**,
> y es deliberado: la revisión del 29/09/2026 encontró lo contrario —ocho
> insignias con un reloj que no esperaba nada—.

> **El §97 no se puede cumplir como está escrito**, y lo que hay que reescribir
> es la regla. «Los cuatro estados en cada pantalla» no aplica a seis de las
> dieciocho —son formularios sin ninguna colección— y Ajustes no enseña errores
> **a propósito**: guardar el tema puede fallar y el fallo se traga con su
> motivo escrito desde la Fase 3. Lo verificable —toda pantalla que pinta una
> colección **variable** tiene su estado vacío— se cumple en las nueve que la
> tienen. Va a la 8.7.

> **Tres hallazgos falsos descartados antes de anotarlos**, que es la lección
> que las pasadas B y C aprendieron cada una por las malas: que la pantalla de
> recuperación no tenía `h1` (lo tiene), que Ajustes no enseña errores
> (correcto), y que cinco pantallas pintan listas sin estado vacío (las cinco
> recorren datos fijos o posteriores a una acción). Las tres se descartaron
> **mirando el código entero** en vez de la cifra de un `grep`.

> **Y dos defectos de mi propia comprobación, el segundo es el de siempre.** La
> primera versión **contaba rutas** y salió en rojo señalando la tabla de
> mínimos de `recorrido.mjs`, que es legítima: contar rutas era medir lo que se
> ve, no quién decide qué pantallas se recorren. Y la ruptura de quitarle el
> mínimo a `rutas.mjs` **no la cazó**, porque `MINIMO` sigue nombrado en la
> cabecera que explica por qué existe — **sexta vez** (5.3, 8.5, dos en la
> 8.3 bis, la 8.6 A), y la primera **dentro de una comprobación escrita el mismo
> día para arreglar otra igual**.
>
> Verificada al final con cuatro rupturas, las cuatro en rojo. Y una de ellas
> estaba mal elegida: renombrar sólo la declaración de `MINIMO` dejaba el
> identificador en sus usos. **Una ruptura que deja el síntoma en pie no prueba
> nada**: hay que quitar la garantía, no su nombre.

> **Y el hueco del §9 número diecisiete, encontrado al preparar la
> recomendación del tope.** El núcleo **ya decide** si una campaña puede
> terminar: `Desenlace` tiene tres variantes —`Termina`, `NoAvanza`,
> `NoCabeEnElHorizonte { faltan }`— y los tres textos están escritos en
> `es.ts`, incluido el que dice *«la campaña no termina en cinco años:
> {faltan} mensajes se quedarían sin enviar»*. El comentario de uno de ellos
> dice literalmente **«el caso que hay que cazar antes de activar»**.
>
> **No los llama nadie.** El asistente pinta `techoDiario` y `diasConEnvio` y
> nada más, y `activar_campana` no consulta la estimación. Así que con 451 196
> personas a 50 diarios el paso 8 enseña «1 303 días» —una cifra correcta— y
> **nunca dice que 386 046 personas no van a recibir un mensaje**, que es la
> cifra que el núcleo calculó y entregó.
>
> Es el patrón de siempre: tres piezas correctas —el cálculo, las variantes y
> los textos— y el cable sin conectar. Y la comprobación que persigue este
> patrón no lo caza porque mira **comandos** registrados y quién los llama; un
> campo de una respuesta que la pantalla ignora no es un comando.

**Cerrada la 8.6 E · el desenlace conectado y el aviso de espera.** Lo que
Dirección pidió como «el tope de activación» **ya estaba decidido y
construido**; faltaba enseñarlo y obedecerlo. Cuatro pruebas en `arles-db`,
cuatro de pantalla, y la comprobación escrita **antes** y vista en rojo
nombrando los seis tramos.

> **`activar_campana` rechaza ahora los dos desenlaces imposibles**, con su
> cifra y las tres partes del §95. **Se pregunta con la audiencia ya congelada y
> antes del bucle de la cola**: lo limpio sería rechazar antes de tocar la base,
> pero la audiencia no se sabe hasta contarla y contarla aparte es otro
> recorrido completo del `JOIN`. Devolver el error ahí deshace la transacción
> entera —audiencia incluida— y ahorra las 451 196 inserciones. **Entero o nada
> lo mantiene la transacción, no el orden**, y hay una prueba de eso.

> **El paso 6 enseña el desenlace y bloquea «Continuar»** cuando impide activar.
> Dejar seguir sería hacer rellenar el preflight y el envío de prueba —que habla
> con el servidor— para fallar en el paso 9: es el defecto que este archivo
> nombra al hablar del `AVISO_DIARIO` («rechaza al final nueve pasos de
> formulario»). El aviso queda justo encima del botón, así que el botón apagado
> dice por qué (§95, y el defecto que la 8.2c y la 8.3c tuvieron).

> **El aviso de espera NO es un tope, y la diferencia importa.** Por encima de
> 30 000 destinatarios dice cuántos segundos va a tardar en prepararse. Una
> campaña de 136 000 que termina en nueve meses es razonable, y prohibirla por
> tardar diecisiete segundos sería cambiar un problema de rendimiento por una
> limitación de producto. El umbral sale de la curva medida en la auditoría C, y
> los segundos del peor caso medido (163 µs por fila), redondeados a cinco:
> quedarse corto es el único error que importa.

> **Tres cosas estaban desalineadas, y las tres por la misma razón: nadie las
> leía.** El tipo de TypeScript decía `{ resultado, cuando }` cuando Rust manda
> `{ desenlace, cuando?, faltan? }`; el fixture de la prueba repetía el mismo
> error; y el simulador de la vista previa devolvía `{ horas, dias, porDia }`,
> una forma que no existe en `Estimacion` — así que el paso 6 de la demostración
> llevaba tiempo pintando huecos.
>
> **`vue-tsc` no podía cazar el primero**: un tipo sólo se contradice al
> **leerlo**. Y la comprobación de la fase 7 no podía cazar el tercero: compara
> **qué comandos** contesta el simulador, no la **forma** de lo que devuelven.
> Es la cola del patrón del §9 — cuando nadie usa una pieza, todo lo que la
> describe se desalinea sin que nada falle.

> **Tres defectos que encontró mirar la demostración**, con todo en verde:
> «1303 días» sin separador de miles (séptima vez de este defecto, el de las
> «12536 filas» de la 8.2c); la frase del ritmo prometiendo que «saldrán» unos
> mensajes que el aviso de debajo decía que no van a salir; y «Continuar»
> habilitado hacia tres pasos que iban a terminar en un rechazo.

> **Y el séptimo comentario que se caza a sí mismo.** La comprobación nueva no
> cazaba `duracion.termina`: casaba el comentario «Cuándo termina una campaña»
> del store, así que dos de los tres tramos se habrían dado por conectados sin
> estarlo. **Séptima vez en el proyecto y segunda en la misma sesión**, dentro de
> una comprobación escrita horas después de arreglar la anterior por lo mismo.
>
> La conclusión, que ya no es una anécdota: **saberlo no basta.** Se quitan los
> comentarios siempre, por defecto, en cualquier comprobación que busque una
> cadena en un archivo.

**Cerrada la 8.6 F · la auditoría de la Fase 8 entera**, mirada **como un
sistema** y no entrega por entrega: qué pasa si lo que sale de una entra en otra,
a tamaño real y por el camino que el usuario recorre. **Cinco hallazgos, cinco
arreglados**, con las diez fases en verde al empezar. Está contado en
[`AUDITORIA-F-FASE-08.md`](documentacion/09-fases/AUDITORIA-F-FASE-08.md).

> **La Fase 8 estaba en verde y su función principal no funcionaba.** El cuerpo
> de un respaldo cifrado se descifra en trozos de 64 KiB y cada trozo se
> convertía a texto **por su cuenta**. Un trozo no es una unidad de texto: una
> `Á`, una `ñ` o una `é` ocupan dos bytes, y en cuanto una frontera partía una
> letra acentuada el lector devolvía `RespaldoCorrupto` **sobre un archivo
> perfectamente bueno**. Con 30 000 contactos hay unas 130 fronteras: que
> ninguna caiga mal es despreciable.
>
> Así que el respaldo se escribía bien, la pantalla lo leía bien, y al pulsar
> «Restaurar y reemplazar» decía que el archivo estaba estropeado. **Una copia
> de seguridad que existe, parece correcta y no abre el día que hace falta** —
> que es exactamente lo que esta fase existía para no tener.
>
> **Ninguna prueba podía cazarlo**, y las dos razones son la lección: todas las
> pruebas de ida y vuelta usaban **dos filas** —un trozo, ninguna frontera— y el
> último trozo siempre acaba en `\n`, que es ASCII. El único trozo que se
> ejercitaba era el único que no puede fallar. Y no lo encontró leer el código ni
> correr las pruebas: lo encontró **poner treinta mil contactos y cronometrar**.

> **Una prueba de ida y vuelta con dos filas no prueba una ida y vuelta.** Si lo
> que se recorre tiene trozos, páginas, lotes o cursores, la prueba tiene que
> cruzar **varias** fronteras — y decir cuántas. La prueba de regresión de este
> fallo salió en verde en su primera versión, con el lector roto a propósito,
> porque 1 200 filas dan una sola frontera: **una prueba probabilística tiene que
> declarar su margen**, o el día que alguien la haga más rápida deja de probar
> algo y nadie se entera.

> **Nada irreversible ocurre hasta que todo lo comprobable está comprobado.** La
> recuperación del llavero perdido apartaba la base y **escribía una clave nueva
> en el llavero** antes de leer el respaldo, así que una contraseña mal teclada
> —la del respaldo, que nadie guarda (8.2a)— volvía definitivo un problema
> reversible: el F2 de la Fase 1, en la pantalla que existe para no repetirlo. Y
> con dos disparadores más que se saben **de la cabecera**: un archivo de
> historial elegido por error y un respaldo de otro esquema. La pantalla además
> **decía que sí se podía restaurar** un archivo de historial, porque el
> veredicto comparaba esquemas y no preguntaba por el alcance.

> **Las cuatro cifras del respaldo, que el presupuesto no tenía** (30 000
> contactos, 07/10/2026): exportar cifrado **244 ms**, restaurar **745 ms**,
> revisar **0.1 ms**, memoria **15 → 19 MB**. Las cuatro holgadas — la fase que
> más datos mueve no tiene un problema de rendimiento. Los presupuestos son
> **propuestas de esta auditoría**, no requisitos de la Fase 0, y así están
> escritos.

> **Y el octavo comentario que se caza a sí mismo, el peor sitio posible: mi
> propio bucle de rupturas.** Verifiqué «la ruptura está restaurada» con un grep
> que casaba el comentario de dos líneas encima del código, y estuve trabajando
> sobre un archivo roto creyéndolo bueno — así que la ruptura siguiente salió en
> verde por eso y no por el código. La conclusión de la 8.6 E era correcta y no
> la había aplicado donde más duele: **una restauración que no restaura deja sin
> significado todo lo que viene después.** Se verifica contra el bloque de código
> entero, nunca contra un identificador.

**Cerrada la 8.7 y con ella la Fase 8 entera** — documento de fase, informe para
Dirección, el avance al **89 % a 2026-10-07** y la revisión a ojo de las
dieciocho pantallas. Está contado en
[`FASE-08-RESPALDOS.md`](documentacion/09-fases/FASE-08-RESPALDOS.md), y el
informe en
[`FASE-08-PARA-DIRECCION.md`](documentacion/09-fases/FASE-08-PARA-DIRECCION.md).

> **Dos cosas que encontró mirar, con todo en verde.** Restaurar y Archivar
> tenían su entradilla a **54 px** del título donde Respaldos —su hermana en la
> misma subsección— la tiene a **8**, porque dejaban el `h1` y el `<p>` sueltos
> dentro de un flex con `gap: space-6`: dos hijos, y los 48 px del hueco en
> medio. Las otras siete los envuelven en un `<header>`. Es el defecto de la
> Fase 3 girado 90°. Y **la pantalla de recuperación decía «ARLES RELAY» sin el
> numeral**, la única de las dieciocho, justo donde más falta hace que la marca
> esté entera.
>
> Las dos las vigila ahora `sonda:recorrido`, que ya comparaba el borde
> izquierdo y compara también el hueco vertical.

> **Y la comprobación nueva nació con un defecto propio, el de siempre con otra
> cara.** Comparaba el hueco de las dieciocho y sacó **tres hallazgos falsos**:
> −24 px en Remitentes —el `<p>` iba **al lado** del título, no debajo— y 19 en
> Plantillas y WhatsApp, donde el título comparte fila con sus botones y eso
> mueve su línea base.
>
> **Comparar un número entre maquetaciones distintas es comparar dos cosas.** El
> arreglo no fue subir el umbral hasta que callaran —eso es ajustar el
> instrumento hasta que deje de medir—: fue preguntar por **la estructura**,
> como la misma sonda ya hace con el armazón. Se mide sólo cuando el título va
> solo en su banda y lo que sigue es un párrafo debajo.

> **El §97 está reescrito**, que era lo que la pasada D dejó pendiente. Pedía
> «los cuatro estados en cada pantalla» y eso **no se puede cumplir**: seis de
> las dieciocho no tienen ninguna colección y Ajustes no enseña errores a
> propósito. Ahora exige lo verificable: estado vacío en toda pantalla con
> colección **variable**, carga en toda espera de más de un segundo, error con
> las tres partes del §95, y acuse de todo final. **Una regla que nunca se puede
> marcar es peor que una que falta**, porque hace dudar de la lista entera.

> **Y el ROADMAP ponía los derechos ARCO en la Fase 8**, construidos desde la
> 3.4. Un plan que pide dos veces lo mismo hace perder el tiempo buscando lo que
> ya existe — que es exactamente lo que le pasó a la auditoría B.

**Y el 08/10/2026, el último hueco del §9 que quedaba: ARLES compilaba,
arrancaba, abría su ventana — y dentro no había nada.** Lo encontró Dirección al
abrirlo por primera vez: la ventana enseñó «can't reach this page — localhost
refused to connect».

> **Tauri no decide por el perfil de cargo** si lo que construye es un programa
> terminado o un entorno de desarrollo. Lo decide por una característica del
> compilador:
>
> ```rust
> pub const fn is_dev() -> bool { !cfg!(feature = "custom-protocol") }
> ```
>
> Sin ella, **incluso compilando en `--release`**, la aplicación busca la
> interfaz en el servidor de desarrollo. El andamiaje estándar de Tauri la
> declara desde el primer día; aquí **no estaba declarada en ninguna parte**, así
> que ni siquiera se podía activar de la forma normal.
>
> **Estuvo nueve fases sin verse porque nadie había construido la aplicación**:
> todo se verifica con pruebas, que no abren ninguna ventana, y con la vista
> previa, que es HTML en un navegador sin nada de Rust detrás. Medido en el mismo
> binario: sin la característica, 20 017 216 bytes y **cero** apariciones de
> `index.html`; con ella, 20 377 184 y **siete**.
>
> **Se abre con `cargo arles`** (alias de `.cargo/config.toml`), nunca con
> `cargo run --release` a secas. Antes, `cd app && npm run build`. Lo vigila
> `la_aplicacion_se_construye_como_aplicacion` en `--fase 1`, que comprueba las
> tres piezas: la característica declarada, el alias que la usa, y que
> [`COMO-ABRIR-ARLES.md`](documentacion/07-entrega/COMO-ABRIR-ARLES.md) mande
> ese comando — **esa guía dijo el comando equivocado en su primera versión,
> porque se escribió sin recorrerla**.

> **Al escribir una guía, se recorre.** La de arranque se escribió en una tarde
> con todo verificado «por lectura», y en el primer intento real de Dirección
> falló **tres veces seguidas**: PowerShell bloqueando `npm`, una base de una
> versión anterior que impide migrar, y el comando de arranque equivocado. Las
> tres están ahora en la guía, con su fecha. **Una instrucción que nadie ha
> seguido no es una instrucción verificada** — es el §2 aplicado a la
> documentación.

> **Y al día siguiente, un cuarto tropiezo y la novena vez del mismo defecto de
> comprobación.** En un segundo equipo, `cargo arles` dio *«cargo no se
> reconoce»* con Rust ya instalado: el PATH se lee **al abrir** cada programa, y
> una pestaña nueva de VS Code hereda el de VS Code. Está en la guía, con cómo
> distinguirlo de un instalador que no llegó a terminar — `rustup-init.exe` abre
> una pantalla de texto que espera un `1` y un Enter, y cerrarla ahí no instala
> nada ni avisa.
>
> Lo grave fue lo otro: **Dirección escribió el comando viejo porque lo leyó en
> el §3 de la guía**, que seguía diciéndolo mientras el §5 explicaba por qué no
> se usa. Y la comprobación escrita el día anterior para esto **pasó en verde**:
> preguntaba `"cargo arles" in guia` sobre el archivo entero, y la cadena buena
> estaba… en los avisos de solución de problemas, no en el bloque que se copia.
>
> **Medir que una cadena aparece no es medir que el sitio que importa la tenga.**
> Ahora mira sólo los bloques de código, y hay una cuarta comprobación: que
> ninguno mande el arranque que no sirve. **Exigir que el bueno esté no impide
> que el malo siga ahí**, que es literalmente lo que pasó. Verificada por los dos
> caminos, incluido colar el comando viejo en otro bloque dejando el §3 correcto
> — ahí la tercera sigue en verde, que es lo que prueba que la cuarta hace falta.
>
> **Y la cuarta nació con un hueco propio, visto antes de darla por buena:** los
> bloques de los avisos viven dentro de **citas**, y `^\s*cargo run` no casa
> `> cargo run`. Se quita el `> ` antes de medir. **Al buscar en Markdown, los
> bloques citados son bloques**, y son justo donde se escriben los comandos de
> salida de un tropiezo.

**Y el 27/10/2026, el sistema de diseño: `DESIGN_SYSTEM.md` reescrito con el
código delante, y tenía cinco afirmaciones falsas.** Documentaba 24 de los 64
tokens que no son color y 5 de las 16 piezas, y no mencionaba el tema claro.
Está contado en [`ESTADO_ACTUAL.md`](documentacion/07-entrega/ESTADO_ACTUAL.md)
§12, y las cinco en el registro de requisitos como DOC-1 a DOC-5.

> **Las cinco llevaban ahí desde la Fase 2 y las cuatro auditorías del §140 no
> vieron ninguna**, porque una auditoría mide el producto y esto es un
> documento. Un documento no falla: **se cree.** La peor no es la más vistosa —
> daba el anillo de foco como el oro de relleno, así que quien copiara ese
> fragmento tendría 1.32:1 en el tema claro, que es un anillo invisible, que es
> la navegación por teclado entera.
>
> Lo caza `el_sistema_de_diseno_dice_lo_que_hay` en `--fase 2`: todo token que
> el documento nombra existe, todo token que existe está nombrado, toda pieza de
> `componentes/` está nombrada **y toda pieza nombrada existe** —las dos
> direcciones—, y las cifras son las que hay. Verificada por los cinco caminos.

> **Décima vez del comentario que se caza a sí mismo, y en el peor sitio.** La
> primera versión de ese documento escribía el nombre de un token inventado para
> ilustrar que un nombre mal escrito no da error — **dentro de la misma sección
> que advierte de esta trampa**, y en el archivo cuya comprobación nueva lee
> precisamente esos nombres. Lo cazó medir antes de escribir la comprobación.
>
> Por eso ese documento tiene ahora una regla propia en su §0: **ahí no se
> escribe el nombre de un token ni de una pieza que no exista, ni como
> contraejemplo.** Una comprobación no distingue una afirmación de un ejemplo.

**Y el 28/10/2026, mirar las dieciocho pantallas: siete hallazgos de diseño y
tres descartados por falsos.** Están en
[`HALLAZGOS-DE-DISENO.md`](documentacion/10-requisitos/HALLAZGOS-DE-DISENO.md),
con los descartados incluidos — un hallazgo falso cuesta más que uno ausente.

> **El primero era que Inicio decía que el producto no está terminado.** Con los
> seis pasos del alta en **6 de 6**, la primera pantalla seguía diciendo «No hay
> nada más que puedas hacer aquí todavía» y «Lo que falta por construir · 1
> paso», con el cuerpo nombrando tres secciones que tienen pantalla desde la
> Fase 7.
>
> Salía de una línea del núcleo: `disponible: paso.ruta().is_some()`. Eran **dos
> preguntas distintas metidas en un campo** —«¿tiene pantalla propia?» y «¿se
> puede hacer hoy?»— y la ventana de envío contesta *no* a la primera y *sí* a
> la segunda, porque se define en el paso 5 del asistente.
>
> **El arreglo no fue separarlas, que es lo que el registro proponía: fue
> juntarlas del todo.** Separarlas habría creado un paso que se puede hacer y no
> lleva a ningún sitio — el H-06 otra vez. Lo que pasaba es que `ruta()` llevaba
> seis semanas diciendo que la ventana no tenía a dónde llevar. Con eso
> corregido, `disponible` sobraba, y con él el estado inconsistente.
>
> Lo vigila `todo_paso_lleva_a_su_pantalla` en el núcleo — **sin lista de
> excepciones**, que es lo que falló las tres veces anteriores (Contactos en la
> 3.2, Campañas y Plantillas el 29/09, la ventana ahora). La lista a mano estaba
> defendida con un buen argumento, y falló igual tres veces.

> **Y una comprobación mía nació con el defecto que yo le había atribuido a la
> de al lado.** La que compara la lista de reserva del store con el núcleo
> recorría **sólo** lo que declara el núcleo, así que un paso que el núcleo deja
> sin ruta y el store sí lleva no se comparaba con nada: salió en verde con el
> núcleo roto a propósito. La del simulador, a la que yo había acusado de mirar
> una sola dirección, miraba las dos desde el principio.

**Y los seis que quedaban, cerrados el mismo día: cinco arreglados y uno
descartado con su cifra.** Lo vigilan dos medidas nuevas en `sonda:auditoria` —
todo botón apagado dice por qué con `aria-describedby` o `title`, y ningún texto
visible lleva un número de corrido—, las dos verificadas rompiéndolas.

> **Tres de los cinco resultaron ser otra cosa de la que el registro anotaba**,
> y el más instructivo es el hueco entre los campos del asistente. No era del
> asistente: `AModulo` lleva desde el 16/10/2026 una regla escrita para el
> **H-09 de Dirección** cuyo comentario calcula «16 + 8 = 24 px», los 16 del
> `gap` del módulo más 8 de margen. **Esos 16 no existen** —el cuerpo del módulo
> es un bloque y no tiene `gap`—, así que durante doce días esa corrección
> entregó 8 px.
>
> Y el primer arreglo fue peor que el defecto: poner el cuerpo como columna flex
> con `gap` **estiró un botón del Registro a 1 086 px**, que es el mismo defecto
> que se estaba arreglando dos pantallas más allá. Lo cazó volver a medir los
> anchos **después** del cambio, no ninguna sonda.

> **Una ruptura en verde significa dos cosas, y no son la misma.** O la
> comprobación no caza el defecto, **o la comprobación no está**. Las dos
> primeras rupturas de ese día salieron en verde porque el script que insertaba
> las medidas falló a mitad —después de preparar el texto y antes de escribir el
> archivo—, y la conclusión fácil habría sido «no sirven». Se distinguen mirando
> el archivo, no deduciendo.

### Las cuatro auditorías del §140, cerradas el 07/10/2026

| Pasada | Hallazgos | Arreglados | Declarados |
|---|---|---|---|
| A · Arquitectura | 5 | 5 | — |
| B · Seguridad | 2 + 3 puntos del documento | 5 | — |
| C · QA y rendimiento | 6 | 4 | 2 |
| D · UX y accesibilidad | 4 | 4 | — |

**Diecisiete hallazgos. Quince arreglados, dos declarados con su cifra.**

Y el patrón que atraviesa las cuatro: **once de los diecisiete no eran defectos
del producto sino de sus comprobaciones** — una lista a mano, un alcance
demasiado corto, un comentario que se cazaba a sí mismo. El producto aguantó
mejor que sus instrumentos, y eso es lo que había que descubrir.

**Lo que queda abierto:** C-2 (el filtro del registro, declarado), reescribir el
§97, **sacar la activación a segundo plano con avance** (Fase 9) y **R-07** —
nada de la pasada D se ha visto en WKWebView.

**C-1 lo cerró la 8.6 E**, y no como se planteó: no hay tope por número de
destinatarios, hay un rechazo de lo que no puede terminar y un aviso de lo que
va a tardar. Ver más arriba.

**Para una demostración real** con correo y WhatsApp que llegan de verdad:
`documentacion/07-entrega/GUIA-DE-CONEXION-REAL.md`.

**Para cambiar el diseño sin buscar:**
`documentacion/05-diseno/MAPA-DE-PANTALLAS.md` — qué archivo dibuja cada
pantalla, de dónde saca los datos, dónde están sus textos y qué comprobación
salta si se rompe.

### El patrón que ya ha aparecido ocho veces

**Una pieza correcta, probada, y nadie la usa.** Ninguna prueba lo caza, porque
una prueba comprueba que la pieza funciona, no que otro módulo la llame: el
hueco vive justo entre dos capas que cada una pasa lo suyo.

| Dónde | Qué faltaba |
|---|---|
| Fase 6 | `guardar_plantilla` y `guardar_firma`, sin pantalla que los llamara |
| Canal W | `ProveedorDeWhatsApp`, y `arles-app` ni dependía de él |
| 6.8b | `whatsapp_template`, una tabla sin nadie que escribiera en ella |
| 6.8a | `PantallaPlantillas.vue`, trece kilobytes sin ruta |
| Vista previa | Once comandos sin respuesta en el simulador |
| 7.0 | La **parada de emergencia**: tres comandos con pruebas, y ninguna pantalla los llamaba. La comprobación que la 7.0 escribió para este patrón miraba **dos de seis** comandos |
| 7.3 | Seis comandos más que la vista previa no contestaba: en la demostración no se podía crear ni activar una campaña |
| 7.3 | `ritmo_por_hora`, construido en la 7.2 sin nadie que lo llamara — y la comprobación no lo cazaba porque verificaba que el comando existiera, no que llamara |
| W.8 | **`enviar_whatsapp_de_prueba`: comando probado de punta a punta, y sin botón.** El canal llegó a su cierre y a la 6.8 sin que se pudiera mandar un WhatsApp desde la aplicación, y este archivo lo daba por hecho |
| W.5 | `revisar_campana_de_whatsapp`, registrado en la frontera para un flujo que no existe: el motor no recorre WhatsApp. Desregistrado en la 7.7 |
| Fase 3 | `contacto`, un comando que además **no hacía falta**: la lista ya trae todos los canales |
| Fase 2 | **El token `--arles-text-on-danger`, definido y sin usar en `ABoton`.** «Detener» pintaba tinta de acento sobre rojo: 2.41:1 |
| W.1 / 6.1 | Once claves de i18n que el núcleo devuelve y el catálogo no tenía, más una —`plantilla.variable.miEmpresa`— que apuntaba a un texto inexistente |
| Fase 1 → 23/10 | **`contact_list` y `contact_list_member`: dos tablas en el esquema desde la V1 y nadie las escribía.** Estaban declaradas en los «pendientes conocidos» del validador con su motivo (§40), así que el hueco **se sabía**. D-15 les dio escritor y lector, y por eso su migración fue **una columna** y no dos tablas |
| Fase 4 → 23/10 | **`Db::suprimir_direccion`, probada desde la Fase 4 y sin ningún comando.** `suprimir_contacto` exige un `ContactId`, así que **una dirección que no es un contacto no se podía dar de baja de ninguna forma** — y es justo la que llega: a quien le reenviaron el correo, o quien contesta desde otra dirección suya. Aquí lo que faltaba no es una función: es que alguien que pidió no recibir vuelve a recibir |

Los cinco primeros los caza `validar.py --fase W`:
`lo_que_se_construye_se_usa`, `ninguna_tabla_se_queda_sin_escritor`,
`toda_pantalla_se_puede_abrir` y la sonda del recorrido.

**Y el tramo que faltaba, que es el único que el usuario pisa:**
`ningun_comando_se_queda_sin_quien_lo_llame` (fase W) compara los comandos de
`generate_handler!` con quien los invoca en todo `app/src`. Había tres
comprobaciones persiguiendo este patrón y cada una miraba un tramo del cable —
store→pantalla, núcleo→comando, y los seis comandos de `control.rs`→pantalla—;
**comando registrado → alguien que lo llame** no lo miraba nadie. Encontró tres
el día que se escribió, con las nueve fases en verde.

Cada comando registrado es **superficie de ataque**: uno que nadie llama es una
puerta que nadie vigila. Por eso se quita, no se deja «por si acaso».

Y los tres últimos, `--fase 7`: `el_freno_llega_hasta_la_pantalla` —que ahora
lee los comandos de `control.rs` en vez de una lista a mano— y **`la vista previa
contesta lo que las pantallas piden`**, que compara las llamadas del frontend
con los `case` del simulador **en las dos direcciones**. Esa última es la que
encontró los tres huecos de la 7.3: un comando sin nadie que lo llame y un
decorado sin nadie que lo use son el mismo hueco visto desde los dos lados.

**Al añadir algo, la pregunta no es «¿funciona?» sino «¿quién lo llama?».**

### Tres formas de tumbar una pantalla con un texto

Las tres son cadenas válidas de TypeScript, así que ni `vue-tsc` ni ESLint las
ven. Las dos primeras aparecieron escribiendo los textos del registro (7.4) y
la tercera los de las plantillas de Meta (D-WA, 27/10/2026):

1. **Una `@` suelta.** `vue-i18n` la usa para sus enlaces —`@:otra.clave`—, así
   que un correo de ejemplo en un texto no compila: «Invalid linked format», y
   **se cae el componente entero**, no ese texto. Un `{'@'}` escapado sí vale.
2. **Los `**asteriscos**`.** No hay Markdown: salen literales, y justo en la
   frase que alguien resaltó porque era la importante.
3. **Unas llaves dobles, `{{1}}`.** `vue-i18n` usa `{...}` para interpolar, así
   que eso es **un marcador dentro de otro**: «Not allowed nest placeholder», y
   se cae el componente entero igual que con la `@`. Y **va a volver a pasar**,
   porque los huecos de una plantilla de Meta se escriben exactamente así. Se
   escapa con interpolación literal: `{'{{1}}'}`.

Las dos primeras las cazan `ningun_texto_lleva_una_arroba_suelta` y
`ningun_texto_lleva_markdown` en la fase 7. **La tercera la cazó
`es.spec.ts`**, que compila los textos uno a uno — y es la comprobación que de
verdad las cubre todas, porque no busca un patrón: ejecuta el compilador.

### Lo que sólo se ve mirando, y no midiendo

Las sondas miden contrastes, focos, desbordes y textos. Nada de eso caza un
dibujo que miente ni una pantalla que empieza en otro sitio. La revisión del
29/09/2026 —abrir la demostración y **mirar** las trece pantallas una tras
otra— encontró cuatro cosas que estaban en verde:

1. **Ajustes empezaba 300 px más a la derecha** que las otras doce: un
   `max-width` sobre `.pantalla`, que el armazón centra. Cada pantalla, sola,
   se veía bien; el defecto sólo existe al pasar de una a otra.
2. **Las insignias neutras llevaban un reloj** —el icono de «en cola»— en las
   ocho que existen: un contador de bajas, «Todos sus canales», una calidad que
   no hemos preguntado. Ninguna espera nada.
3. **Actividad hablaba de «roadmap», «andamiaje» y «tokens»**, y con el texto
   escrito dentro del componente, contra el §139.
4. **Actividad no tenía salida**: cero acciones, sólo la barra lateral.

Las dos primeras las caza ahora el recorrido y una prueba del componente; la
lección de las cuatro es la misma: **una pantalla puede pasar todas sus
comprobaciones y seguir estando mal.** Antes de cerrar una entrega, se abre la
demostración y se mira.

Y dos de las comprobaciones eran ellas mismas el defecto: la que compara el
simulador con el núcleo se inventaba el nombre de una clave y su patrón no
sabía leer una fila que contuviera `[]`, así que llevaba tiempo sin mirar el
paso de Contactos **sin decirlo**. Es el §2 otra vez.

### Tres trampas de las herramientas

1. **`generar.py --sin-mont` reescribe `tipografia.css`.** Desde el 29/09/2026
   lo restaura al terminar, pase lo que pase. Antes lo dejaba reescrito y la
   siguiente vista previa salía sin la tipografía sin que nadie supiera por qué.
2. **`generar.py` aborta si la compilación falla, pero deja el HTML anterior en
   disco.** Una sonda que no comprueba su código de salida mide **otra versión
   del código**. La del recorrido ya lo comprueba; cualquier sonda nueva debe
   hacerlo.
3. **Una comprobación que escribe en el repositorio para medirlo da rojo sobre
   código correcto.** La del CSS de los tokens generaba `dist/arles-tokens.css`
   y después preguntaba a `git diff`: dos validadores a la vez se pisaban el
   archivo —uno escribía mientras el otro comparaba— y la fase 1 salía **52 de
   53** sin que nada estuviera roto. **Medido el 22/10/2026: 1 falso de 40**
   con dos escritores, y **0 de 120** después de generar a un temporal y
   comparar contenidos. Un validador no puede exigir ser el único proceso del
   equipo, porque nadie se lo va a conceder: alguien lo correrá mientras hace
   otra cosa, y lo que verá es un rojo que no existe.

> **Y lo que aquí no se pudo hacer, que es la mitad de la lección.** El 52/1 se
> vio **una vez**, con tres validadores a la vez, y la salida no se guardó: así
> que **no se sabe qué comprobación fue** y no se va a decir. Lo que sí se
> midió es el mecanismo —esa comprobación escribe en el repositorio, y bajo
> concurrencia falla— y eso es un hallazgo; «fue ésa» habría sido un hallazgo
> falso de los que la auditoría B dejó dicho que cuestan más que uno ausente.
> **Una corrida que no se guarda no se puede diagnosticar.** Las cifras que se
> publican salen de una corrida **secuencial**, de una en una.

**Seis trampas del código de la Fase 6**, por si se toca:

1. La **empresa del contacto** vive en la columna `source`, no en una llamada
   `company_name`. Viene del esquema V1 y no se ha renombrado.
2. El **cerrojo de la conexión no es reentrante**: llamar a un método público de
   `Db` desde dentro de un `con_dominio` cuelga el programa sin error ni traza.
   Por eso existe `cuentas::salud_en`, que recibe la conexión ya abierta.
3. Los **tres puntos de red del preflight** (`cuenta_autentica`, `hay_conexion`,
   `registros_del_dominio`) no se pueden contestar desde `arles-db`. Se
   completan en `arles-app`, y el validador falla si alguien los pone en
   `Some(true)` a mano.
4. De los **seis diagnósticos** de la sonda, sólo dos dicen algo sobre la
   contraseña. Los otros cuatro son `None`: no se llegó a intentar.
5. El umbral del §48 es **más de 50**, no 50 o más, y está escrito en dos
   sitios: `Limites::AVISO_DIARIO` y la pantalla, que necesita enseñar el aviso
   antes de llamar a nadie. Quien lo **exige** es `crear_campana`.
6. Las pantallas usan `useRouter()`, que lee de la inyección. Un `mocks:
   { $router }` en una prueba **no** la cubre: Vue avisa y la prueba pasa igual.

**Tres trampas del código de la Fase 5**, por si se toca:

1. La comprobación del estado de la cuenta **exige `connected`**; no enumera los
   estados que rechaza. Escrita al revés, un estado nuevo enviaría correo hasta
   que alguien se acordara de prohibirlo.
2. **Los límites de envío no están en `campaign`.** La V4 los movió a
   `campaign_stage` junto con la ventana y el remitente. El esquema de la V1
   todavía los enseña.
3. **`localhost` no pasa la validación del núcleo**: un servidor de correo sin
   punto no es un nombre de dominio.

### Las tres reglas de esta fase

Están escritas más de una vez a propósito, y hay comprobaciones que fallan si
alguien toca una.

1. **No existe ninguna forma de no cifrar** (§29). Ni un valor, ni una casilla,
   ni una opción avanzada. Está en el tipo del núcleo, en el adaptador y en el
   `CHECK` de la columna.
2. **La contraseña entra y no sale.** Vive en el llavero; en la base hay una
   referencia opaca. Ningún comando la devuelve.
3. **Nada antes del `RCPT TO` puede dar de baja a nadie.** Es la única puerta a
   una supresión, junto con el acuse final.

### Decisión de Dirección del 18/09/2026

Ante un contacto repetido, ARLES **enseña los choques antes de importar**. No
los salta ni fusiona solo: una importación de diez mil filas es irreversible en
la práctica.

### La tabla del cliente · 22/10/2026

Dirección preguntó si ARLES podía cargar la tabla de un cliente —nueve columnas,
de las que usa tres— y quedarse con lo que necesita. **Podía, menos una.**

> **«Teléfono» se ignoraba, y los contactos entraban sin un solo número.** Ocho
> de las nueve columnas se reconocían. La lista de sinónimos tenía «teléfono
> móvil» y «teléfono celular» pero no «teléfono» a secas, y la prueba que decía
> cubrirlo probaba **«Teléfono móvil»**, que es el caso que no falla. El patrón
> de siempre: la comprobación cubría la parte que nadie había roto.
>
> Estaba así **a propósito** y por un motivo bueno —«Teléfono» no dice si ese
> número recibe WhatsApp, y mandar a fijos se paga en calidad del número ante
> Meta—, pero **resolver una ambigüedad callando no es prudente, es silencioso**
> y pierde el dato igual. Ahora se reconoce **y se avisa**, con la columna
> nombrada. Lo que dice a las claras no ser un móvil —«Teléfono fijo»,
> «Conmutador», «Lada», «Fax»— se sigue dejando fuera, y va **antes** de los
> sinónimos porque «telefonofijo» empieza por «telefono».

> **La plantilla descargable, y el único archivo que ARLES escribe para volver a
> leerlo él mismo.** Por eso lleva su propia comprobación:
> `la_plantilla_que_arles_da_entra_en_arles` —escrita antes y vista en rojo— más
> la ida y vuelta de `arles-import`, que la escribe y la relee con el **mismo**
> `leer()` que lee el archivo del cliente. Una plantilla con una columna que la
> importación ignora es peor que no tenerla: quien la rellena ve su dato escrito
> y ese dato no entra.
>
> **XLSX y no CSV**, porque un CSV no dice con qué separador se escribió y Excel
> en México espera punto y coma: se abriría con las tres columnas dentro de la A.
> Cero dependencias nuevas — `zip` ya estaba para medir la descompresión.
> **Y sin fila de ejemplo**: una fila que nadie borra se importa, y es un
> contacto inventado recibiendo un envío real.

> **Tres defectos de mis propias comprobaciones, los tres cazados antes de
> entrar**, y vale la pena el recuento porque el proyecto lleva nueve:
>
> 1. El aviso usaba el prefijo de los genéricos, así que **«Teléfono móvil»
>    avisaba** — y ésa dice exactamente de qué número habla. Lo cazó la prueba.
> 2. La del validador buscaba `guardadaDetalle:` y `cuerpo:` **en todo `es.ts`**,
>    donde hay una de cada por pantalla: midió el primer bloque del archivo y
>    salió **en rojo sobre dos textos correctos**. Es la hoja y no la ruta, el
>    mismo defecto que la 7.7 arregló en la del §139.
> 3. Y su comentario afirmaba estar verificada **en las dos direcciones**. Era
>    falso: quitar una columna del núcleo pasa en verde, y se dio por hecho en
>    vez de correrlo. Está escrito en el docstring, con por qué esa dirección no
>    se puede medir sin hallazgos falsos.

> **Y lo que encontró mirar la pantalla**, con todo en verde: a 1024 px —un
> portátil al 150 %— **las cinco cabeceras de Contactos no caben**, y la columna
> «Acciones» de la entrega anterior cortaba su propio rótulo a las cuatro
> ventanas. Repartir los `fr` no lo arregla: reparten el espacio, no lo crean, y
> cada ajuste robaba a una columna y cortaba otra — tres rondas persiguiendo
> cuatro píxeles, que es **ajustar el instrumento** con otra cara. Se arregló
> estructuralmente: `tituloOculto` en `ATabla` deja el rótulo para el lector de
> pantalla y libera 96 px. Y eso sacó `.sr-only` a `base.css`: estaba duplicada
> en dos componentes con `scoped`, así que el design system no podía usar
> ninguna y la tercera copia habría sido la de pedir.

### La auditoría de funcionalidad · 22/10/2026

Dirección pidió revisar la aplicación entera buscando **errores de
funcionalidad**. Las cuatro auditorías del §140 miden arquitectura, seguridad,
rendimiento y accesibilidad; ésta mide otra cosa: **si lo que el usuario escribe
llega a donde tiene que llegar.** **Siete hallazgos, siete arreglados**, con las
diez fases en verde al empezar — y **el séptimo lo encontró Dirección leyendo
este mismo informe**. Está contado en
[`AUDITORIA-G-FUNCIONALIDAD.md`](documentacion/09-fases/AUDITORIA-G-FUNCIONALIDAD.md).

> **El límite diario se escribía y no se aplicaba.** El campo existe desde la
> Fase 6 con su aviso de riesgo y su casilla; lo que no existía era el efecto.
> El motor tomaba **el menor** entre el límite de la campaña y el de la cuenta,
> y toda cuenta nace con 50 escritos en el código sin ninguna pantalla para
> cambiarlos. Medido: se escribe 300, se lee el aviso, se marca la casilla —
> **y salen 50**. Peor que faltar la función, porque deja creer que se usó.
>
> **Decisión de Dirección: el límite se fija al crear la campaña.** El `min` se
> quitó, y lo que ocupa su sitio está escrito en la cabecera de `trabajo.rs`:
> el presupuesto cuelga de la **cuenta** —dos campañas de 300 sobre el mismo
> remitente comparten contador, no suman—, el aviso del §48 queda en
> `audit_log`, y ADR-0008 sigue intacto. El techo que se quitó no era el del
> proveedor: era una constante.

> **Un `0` creaba una campaña que se activa y no manda nada**, y un negativo se
> descartaba en Rust para caer en el de la cuenta: las dos en silencio. Se
> rechaza en los dos sitios a propósito — en el paso 5, que dice qué pasa y
> deja corregir, y en `crear_campana`, porque la frontera no se fía de la
> webview. **La red que ya existía se quedó**: una validación de entrada no
> protege a las filas que ya están dentro, así que la prueba que cazaba el cero
> por la cadena entera ahora lo siembra en la etapa.

> **La firma elegida nunca se guardaba.** `campaign_stage.signature_id` existe
> desde la V4 y `redaccion_de_la_campana` ya la consultaba —con su caída a la
> de por defecto— **meses antes de que nada la escribiera**. Todas las campañas
> salían con la misma firma y nada fallaba. El §9 en su forma más silenciosa:
> aquí ni siquiera faltaba una pantalla, faltaba **un valor**.
>
> Y **dos pruebas, no una**: la de `plantillas.rs` recorre las dos ramas del
> `COALESCE` sembrando con SQL; la de `alta_de_campana.rs` pasa por
> `crear_campana`. La primera versión del comentario afirmaba estar verificada
> quitando el `INSERT`, y **era falso** —se probó y pasó en verde—, porque
> siembra la etapa a mano. El hueco vivía entre las dos pruebas.

> **El simulador no exigía el aviso de volumen, con el comentario encima
> diciendo que sí.** Preguntaba por `args.avisoAceptado` y el asistente manda
> `aceptoElRiesgoDelVolumen`, así que la condición no se cumplía nunca. **Y la
> clave con la que rechazaba tampoco existía** en el catálogo, así que de
> cumplirse habría enseñado la cadena cruda. Dos defectos en cuatro líneas.
>
> Lo caza `el_simulador_rechaza_con_claves_que_existen`, que encontró **dos
> más** el día que se escribió: `error.canal_vacio` y `error.sin_canales`,
> ninguna con texto. El §139 no llegaba ahí porque recorre los `clave_i18n()`
> de Rust y el simulador es JavaScript.

> **Y el paso 4 dejaba sin salida.** El paso 3 avisa cuando no hay cuentas y el
> 5 cuando no hay horarios; el de la plantilla dejaba un desplegable vacío,
> «Continuar» apagado y ninguna pista de que se escribe en otra pantalla. Es el
> **H-06 en el paso de al lado**, y lo encontró mirar la demostración con todo
> en verde.

> **Lo que se midió y salió limpio, que es la mitad del valor:** los 37
> argumentos que las pantallas mandan a los 67 comandos, comparados uno a uno
> con lo que cada comando declara —**ningún desajuste**, y era la familia del
> H-05—; ningún comando sin quien lo llame; ninguna tabla sin escritor; y las
> dieciocho pantallas a cuatro tamaños sin un texto encimado.
>
> **La primera medición de esos argumentos dio 130 hallazgos, todos falsos:** la
> expresión regular capturaba hasta el `return {` del final de cada store. Se
> rehízo contando llaves. Cuarta vez que hace falta la lección de las pasadas
> B, C y D — **comprobar antes de anotar**.

> **Lo que sigue sin poder hacerse, y no es un defecto:** lanzar una campaña por
> WhatsApp. Sólo el envío de prueba. Medido otra vez: el camino está cableado a
> `channel = 'email'` en **siete sitios**. No es un cable suelto, es una fase.
> Y **R-05 —imagen en la firma—** espera una decisión: embebida dispara filtros
> de spam, alojada hay que alojarla, y el saneado tendría que admitir `<img>`.

> **Y el séptimo hallazgo lo encontró Dirección leyendo el informe**, el mismo
> día: *«si coloco en la campaña uno un límite de 50 al día y otra campaña dos
> un límite de 50 al día con el mismo remitente, no se mandan 50 en ambas»*.
>
> Medido con el mecanismo real: salían **25 y 25**, y **por orden de llegada**,
> que es peor que mitad y mitad porque no se puede predecir.
>
> **Y esa misma tarde Dirección decidió lo contrario de lo que yo defendía, así
> que las dos frases siguientes describen el producto de la mañana y no el de
> hoy** (ver más abajo, D-13 a D-17): entonces con 200 y 200 no salían 400 sino
> 200, porque el total de un día era el mayor de los límites y nunca la suma.
> Ésa era la defensa que sustituyó al `min`. **Hoy sí salen 400.**
>
> Lo que sí se rompía es **la promesa de tiempo**: el paso 6 estima con el
> límite de esa campaña como si el remitente fuera sólo suyo. El aviso va
> **pegado a la frase del plazo** y no al final de la pantalla — un aviso que
> desmiente una cifra tiene que verse con la cifra delante.
>
> **Se avisa en vez de impedir** (§154): dos campañas desde un mismo correo
> corporativo es legítimo; prometer un plazo falso no lo es. La salida —un
> remitente por campaña— está en el propio aviso, y es lo que ADR-0008 dice
> desde el 11/09.

### La ronda de correcciones de Dirección · 13 y 14/10/2026

Dirección abrió ARLES por primera vez y probó. **Lo que encontró en dos tardes
no lo había encontrado ninguna de las cuatro auditorías del §140**, y conviene
saber por qué: las auditorías miden, y lo que falló es lo que sólo se ve
usando.

| | Qué | Estado |
|---|---|---|
| **H-01** | Remitentes decía «todavía no has configurado los datos de tu empresa» **después** de configurarla | ✅ Arreglado |
| **H-02** | Las tres partes de un error salían **pegadas**: «…con el servidor.El servidor no ofrece…» | ✅ Arreglado |
| **H-03** | **No se pudo dar de alta una cuenta de Gmail** | 🔎 Faltan tres datos de Dirección |
| **R-01** | Dar de alta un correo era «demasiado complicado» | ✅ Hecho |
| **R-02** | Lo mismo con WhatsApp | ✅ Hecho, con un límite declarado |

**El alta de correo son dos campos desde el 14/10/2026.** `PROVEEDORES` en
`stores/remitentes.ts` relaciona cinco dominios con su servidor, puerto y
cifrado; la «Configuración avanzada» sólo se abre sola cuando el dominio no se
reconoce. La garantía que pidió Dirección —«comprueba que el servidor, puerto y
cifrado sean siempre correctos»— es `el puerto y el cifrado nunca se cruzan`:
587 con STARTTLS, 465 con TLS directo, y ningún tercer puerto. Verificada
cruzando los de Gmail.

> **WhatsApp no se puede reducir a dos campos, y el límite está declarado.** El
> correo se deduce del dominio; los tres identificadores de Meta **sólo existen
> en su panel** y no hay nada de lo que inferirlos. Lo que sí se puede es cazar
> el error de pegado antes de guardar, que es donde se pierde la gente:
> `revisarDatoDeMeta` **avisa y no bloquea**, porque Meta podría cambiar el
> formato y un formulario que se niega a guardar algo correcto es peor que uno
> que señala algo dudoso.

**Y lo que Dirección vio estaba en verde.** Escribió «el texto se ve
extremadamente desordenado» con `sonda:auditoria` y `sonda:recorrido` dando las
dieciocho pantallas sin un hallazgo. Ninguna de las dos mide lo que vio: la
auditoría caza lo que no cabe **de ancho**, y dos párrafos montados uno sobre
otro caben; el recorrido compara el hueco **del título**, y lo que pase cien
píxeles más abajo no lo mira.

> `sonda:encimado` mide las cuatro cosas que quedaban fuera, las cuatro
> geométricas: dos rectángulos de texto que se cruzan, texto recortado de alto,
> texto que asoma de su padre, y **un rótulo cortado por los puntos
> suspensivos**. Cortar un dato está bien —la dirección entera se lee al abrir
> la ficha—; cortar una cabecera de columna deja «WhatsApp pri…», que es lo que
> Contactos enseñaba. **1 836 textos medidos** en 72 pantallas-ventana.
>
> **Y nació con tres hallazgos falsos, los tres comprobados antes de
> anotarlos:** el contenido de un `<details>` **cerrado** sigue devolviendo
> rectángulos y sus hijos caen apilados —nueve falsos por ventana—, un padre
> con `text-overflow: ellipsis` no deja asomar nada, y un `.sr-only` de 1×1 px
> tiene la forma exacta de un rótulo recortado. La pregunta correcta no es
> «¿tiene área?» sino `checkVisibility()`.

> **Y la comprobación que vigila las sondas tenía el defecto que persigue.**
> `las_sondas_recorren_todas_las_pantallas` decía vigilar «las sondas» y
> vigilaba **dos nombres a mano**; la tercera no la habría mirado nunca. Ahora
> salen de la carpeta, y ninguna puede nombrar más de tres rutas por su cuenta
> — umbral **medido**: las de una pantalla nombran 0 o 1, `vista-previa.mjs`
> nombra 3. Es la lista a mano por enésima vez, y esta vez dentro de la
> comprobación escrita para cazarlas.

> **Una ruptura que no deja síntoma no prueba nada.** La primera de la medida 1
> fue un margen negativo en el aviso del proveedor de Remitentes, que **sólo se
> pinta al escribir una dirección**: en la vista previa ese elemento no existe,
> así que la sonda salió en verde sobre código roto. Se rehízo sobre los
> subtítulos de Respaldos, que se pintan siempre.

**El tablero visual** (`tablero/`, pedido el 14/10/2026) y el **manual de
usuario** (`documentacion/07-entrega/MANUAL-DE-USUARIO.md`, que no existía). El
tablero no lleva ni una cifra propia, y mirarlo encontró cuatro cosas con todo
en verde: «1 hallazgos abiertos» sin pluralizar, un punto verde delante de cada
tropiezo que no indicaba nada, «sin medir» pintado en rojo —que es un fallo, y
lo que pasa es que no se sabe—, y una fase al 0 % con el canal vacío, que se lee
igual que «no hay dato».

**La sección más importante del manual es la última**: lo que ARLES todavía no
hace. Es lo que impide prometer algo que no se puede cumplir, y es la misma
regla del §9 aplicada a lo que lee el usuario.
