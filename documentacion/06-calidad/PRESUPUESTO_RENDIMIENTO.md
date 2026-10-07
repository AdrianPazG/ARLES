# Presupuesto de rendimiento

**Proyecto:** ARLES RELAY I · v1.2.0
**Escala objetivo:** 200 000 contactos operativos · **500 000 techo arquitectónico** (T-7)

> **Se miden, no se estiman.** Los presupuestos se comprueban en CI contra un conjunto sintético de 500 000 contactos con semilla fija. Una regresión rompe la compilación.

---

## 0. Lo que se midió el 06/10/2026, y lo que no

> **Hasta esta fecha, las trece cifras de abajo eran estimaciones de la Fase 0 y
> ninguna se había comprobado.** La entrega 8.5 construyó el banco de pruebas
> (`crates/arles-banco`) y midió lo que se puede medir desde un contenedor sin
> escritorio. **«Sin medir» no es «cumple»**, y así está escrito.

**Medido el 06/10/2026**, con 500 000 contactos sintéticos (semilla 20261006,
38 MB de CSV) en un contenedor Linux sin escritorio. **No es la máquina de
referencia del §2**, así que las cifras sirven para comparar entre sí y para
detectar lo que está mal por un orden de magnitud — no para certificar.

| Operación | Presupuesto | Medido (V10) | Medido (V11) | Veredicto |
|---|---|---|---|---|
| Importar 500 000 filas | < 5 min | 51.7 s | **91.4 s** · 110.7 s | ✅ cumple (37 % del techo) |
| Memoria durante importación | < 500 MB | 497 MB | 497 MB | ⚠️ cumple por 3 MB |
| **Búsqueda por correo** | < 100 ms | ❌ 331 ms | ✅ **< 1 ms** | ✅ arreglado en la 8.6 |
| **Abrir tabla de contactos** (la consulta) | < 500 ms | ❌ 1 424 ms | ✅ **357 ms** | ✅ arreglado en la 8.6 |
| Simulación de campaña | < 200 ms | 5.9 ms | 6.0 ms | ✅ cumple |
| **Calcular audiencia de campaña** (activar) | < 2 s | — | ❌ **73.5 s** | ❌ **36× por encima** |
| **Filtro del registro por texto** | < 1 s | — | ❌ **3 709 ms** | ❌ **3.7× por encima** |
| **Memoria del motor en corrida larga** | < 250 MB | — | ✅ **13.9 MB, plana** | ✅ cumple · parcial (sin webview) |
| Arranque en frío | < 2 s | — | — | ⬜ sin medir (R-07) |
| Desplazamiento en tabla virtualizada | 60 fps | — | — | ⬜ sin medir (R-07) |
| Filtro sobre 500 k **contactos** | < 1 s | — | — | ⬜ **no existe todavía** |
| Preflight completo | < 3 s | — | — | ⬜ sin medir (R-24) |
| Importar 100 k filas XLSX | < 60 s | — | — | ⬜ sin medir |
| **Exportar un respaldo cifrado** | < 30 s | — | ✅ **244 ms / 30 k** · ~4 s / 500 k | ✅ cumple · medido en la auditoría F |
| **Restaurar un respaldo** | < 60 s | — | ✅ **745 ms / 30 k** · ~12 s / 500 k | ✅ cumple · medido en la auditoría F |
| **Revisar un respaldo** (el preflight) | < 1 s | — | ✅ **0.1 ms** · no crece con las filas | ✅ cumple · lee sólo la cabecera |
| **Memoria al exportar y restaurar** | < 500 MB | — | ✅ **15 → 19 MB** | ✅ cumple · el cursor de la 8.2b |
| Tamaño del instalador | < 20 MB | — | — | ⬜ sin medir (P-01) |

> **Las cuatro filas del respaldo son nuevas, y sus presupuestos también.** Los
> midió la auditoría F de la Fase 8 el 07/10/2026, y conviene ser claro en lo que
> son: **la Fase 8 construyó exportar, restaurar y archivar sin ningún
> presupuesto declarado** —ninguna de las trece cifras originales los
> mencionaba—, así que las columnas «< 30 s», «< 60 s», «< 1 s» y «< 500 MB» son
> **propuestas de esta auditoría**, no requisitos que vinieran de la Fase 0.
>
> Están puestas donde está la frase que hay que poder decir: *«pulsé respaldar y
> no me dio tiempo a preguntarme si se había colgado»*. Las cuatro medidas caben
> con dos órdenes de magnitud de margen, así que afinar el número no cambia nada
> hoy; **Dirección puede bajarlas cuando quiera** y seguirán cumpliéndose.
>
> La de memoria no es una propuesta suelta: es la garantía que la 8.2b y la 8.2c
> escribieron —cursor y sobre a trozos, nunca el cuerpo entero en memoria— y que
> hasta hoy **no tenía una sola cifra al lado**.
>
> Y lo importante de esa pasada no fue ninguna de las cuatro: fue que al ir a
> medirlas apareció que **un respaldo cifrado de tamaño real no se podía
> restaurar**. Está contado en
> [`AUDITORIA-F-FASE-08.md`](../09-fases/AUDITORIA-F-FASE-08.md).

> **Las tres filas del motor y las campañas las midió la auditoría C del §140 el
> 07/10/2026**, y
> las tres estaban declaradas «sin medir» por motivos que no se sostenían al
> mirarlos:
>
> - «Calcular audiencia» decía *«exige una campaña activa con su etapa y
>   remitente: es un banco aparte»*. Eso es una **molestia**, no un
>   impedimento: la etapa y el remitente son cuatro `INSERT`. Y era la peor
>   cifra del producto.
> - «Filtro sobre 500 k» decía *«no existe todavía»*. Es verdad de la **lista de
>   contactos** —`listar_contactos` sólo toma página y tamaño— y falso del
>   producto: la 7.4 construyó seis filtros combinables sobre el registro. Las
>   dos filas están ahora separadas, porque son dos cosas.
> - «Memoria en reposo con motor activo» decía *«sin llavero no arranca la
>   aplicación entera»*. Verdad de la aplicación, falso del motor, que corre
>   contra un proveedor simulado desde la 4.3a.
>
> **Un «sin medir» con un motivo falso es peor que un hueco**, porque cierra la
> pregunta. Los cuatro que quedan arriba sí tienen un motivo que se sostiene:
> necesitan una webview, una red sin puertos bloqueados o un empaquetador.

### Importar 500 000: una cuarta medición, y lo que dice del ruido

La 8.6 dio **91.4 s** sobre tres corridas (91.1, 93.8, 91.4). La de hoy, mismo
conjunto, misma semilla, misma máquina: **110.7 s**, un 21 % más.

No se cambia la cifra de la 8.6 ni se tacha: **se escriben las dos**. Lo que
esto dice es que la máquina de medición tiene un ruido de ese orden, así que una
regresión por debajo del 25 % **no se puede distinguir aquí** — y el margen del
20 % del §5 es demasiado estrecho para este contenedor. En la máquina de
referencia del §2 habrá que volver a medirlo.

Sigue dentro de presupuesto con 2.7× de margen (111 s de 300).

> **La columna V11 se midió el 06/10/2026** con el mismo conjunto y la misma
> semilla, en la misma máquina, el mismo día. La importación se corrió **tres
> veces** (91.1 s, 93.8 s, 91.4 s) antes de dar su cifra por buena: no se
> reporta una regresión sin saber si es ruido.

### Lo que costó arreglarlo, dicho con su cifra

**La importación pasó de 51.7 s a 91.4 s: un 78 % más.** No es un efecto
secundario inesperado, es el precio de los índices: cada fila que entra tiene
que actualizar también las listas ordenadas. Medido aparte, en inserciones
puras, los dos índices nuevos cuestan un **+40 %**; sobre SQLCipher el efecto
se amplifica.

Entra en presupuesto con **3.2× de margen** (91 s de 300), y el cambio vale la
pena sin discusión: se paga **una vez, al importar** —una operación que un
usuario hace al empezar y casi nunca más— a cambio de que la pantalla más usada
del producto baje de 1.4 s a 0.36 s **cada vez que se abre**.

Se consideró y se descartó la técnica habitual de carga masiva —quitar los
índices, cargar, recrearlos—: el coste de recrear un índice es proporcional al
**total** de filas de la tabla, no a las que se insertan, así que una
importación de cien contactos sobre una base de 450 000 saldría muchísimo peor.

### Los dos que fallaban, y por qué (medición del 06/10/2026, esquema V10)



No son «va un poco lento»: los dos tienen una causa concreta, encontrada con
`EXPLAIN QUERY PLAN` sobre el esquema V10.

**1 · Abrir la tabla de contactos — 1 424 ms de 500.**

`contact` **no tiene ningún índice para su propia consulta**. Los únicos que
hay son el de la clave primaria y uno sobre `import_batch_id`. El plan dice:

```
SCAN contact
USE TEMP B-TREE FOR ORDER BY
```

Es decir: para enseñar **cien** filas, SQLite lee las 455 864 y las ordena en
memoria. Y el recuento del total hace un segundo barrido completo. Es la
pantalla donde el usuario pasa más tiempo.

**2 · Búsqueda por correo — 331 ms de 100.**

Hay un índice único sobre `contact_channel(company_id, channel,
value_normalized)`, y la consulta **no filtra por `channel`**. Sin la segunda
columna, SQLite sólo puede aprovechar la primera:

```
SEARCH ch USING INDEX idx_channel_unique (company_id=?)
```

Así que recorre **todos los canales de la empresa** —correos y WhatsApp, más de
900 000 filas— para encontrar uno. El índice existe y está bien puesto; lo que
no encaja es la consulta.

> **Las dos las arregló la migración V11**, aprobada por Dirección el
> 06/10/2026. La consecuencia que se avisó antes de hacerlo: **un respaldo
> tomado con el esquema V10 ya no se puede restaurar** (ver 8.3b). No existía
> ninguno fuera de las pruebas, que es justo por lo que se hizo ese día.

### Los dos que falló la auditoría C (07/10/2026), con su curva

**1 · Activar una campaña — 73.5 s de 2 s.**

No es «va lento»: es una interfaz congelada tres cuartos de minuto en la acción
central del producto. `activar_campana` corre **dentro de un comando de Tauri**,
en una sola transacción y con el cerrojo de la conexión tomado, así que mientras
dura no contesta nada y el motor no escribe.

Se midió a cuatro escalas, con el mismo conjunto y la misma semilla:

| Destinatarios | Activar | µs por fila |
|---|---|---|
| 18 121 | 1.2 s | 66 |
| 45 253 | 3.8 s | 84 |
| 135 588 | 16.7 s | 123 |
| 451 196 | 73.5 s | 163 |

**El presupuesto de 2 s se rompe alrededor de los 30 000 destinatarios**, que es
una lista de tamaño ordinario. No es un problema del techo arquitectónico: es un
problema de una campaña mediana.

**Y la causa medida no es la que parecía.** La hipótesis obvia era el bucle que
genera dos UUID por fila —451 196 pares, dos llamadas a entropía cada uno—. Se
midió aparte: **0.58 s**, un 0.8 % del total. Lo que cuesta es la **escritura**:
la activación mete ~451 000 filas en `campaign_audience` y otras ~451 000 en
`message_attempt`, cada una actualizando sus índices, sobre SQLCipher. Es el
mismo orden que la importación (~1.35 M filas en 111 s).

Las alternativas y su coste están en
[`AUDITORIA-C-QA.md`](../09-fases/AUDITORIA-C-QA.md) §C-1. **Pendiente de
Dirección**, porque la opción barata no es técnica: a 50 correos diarios (§48)
el simulador del núcleo dice que una campaña de 451 196 personas **no termina en
cinco años** y que **386 046 mensajes se quedarían sin enviar** (medido el
07/10/2026 con `estimar`). Esta línea decía «24 años» hasta esa fecha: era una
cuenta a mano, y mal — el núcleo no devuelve una duración para ese caso, devuelve
`NoCabeEnElHorizonte`.

**2 · Filtro del registro por texto — 3 709 ms de 1 s.**

El filtro de texto de D-7C compara contra `message_attempt.contact_address` con
`LIKE '%…%'`, que ningún índice puede aprovechar. El módulo ya lo decía, medido
sobre 20 000 filas: 11 ms. La curva:

| Sucesos | Coincidencias | Filtrar |
|---|---|---|
| 18 121 | 4 888 | 137 ms |
| 45 253 | 12 160 | 486 ms |
| 135 588 | 36 029 | 1 003 ms |
| 451 196 | 119 532 | 3 709 ms |

**El presupuesto se rompe justo a los ~135 000 sucesos**, y aquí la urgencia es
otra: a 50 envíos diarios se tarda **más de siete años** en acumularlos, y el
recorte bimestral de la 7.4 los quita antes. Lo que lo hace alcanzable es que
ese recorte **lo pulsa el usuario**, no un reloj.

Así que C-1 es trabajo y C-2 es una cifra declarada con su condición. Decir que
los dos «fallan» sin esa diferencia habría sido exacto y useless.

### La memoria del motor en una corrida larga, que nunca se había medido

**20 000 envíos consecutivos: 13.9 MB en los cuatro cuartos de la corrida, y
0.00 MB de crecimiento.** Es el tercer punto de la pasada C del §140 —«sin fugas
de memoria en ejecuciones largas del motor»— y hasta hoy no se había intentado.

No se mide un número, se mide una **forma**. Un proceso que envía veinte mil
correos tiene que crecer al principio —SQLite llena su caché, el asignador pide
arenas— y ese crecimiento es correcto y se detiene. Lo que el §7 llama fuga es
«memoria que **crece sin estabilizarse**», así que la corrida se parte en cuatro
tramos y se compara el último con el **segundo**. Comparar con el primero
metería dentro todo el arranque y una corrida sana saldría en rojo.

De paso sale el rendimiento del motor: 20 000 pasos en 261 s, **13 ms por
envío**. Unas 77 por segundo, contra los 50 **diarios** que el producto permite:
el motor no es el cuello de botella de nada, que es lo que el §6 ya decía y
ahora está medido.

**Es una medida parcial y hay que decirlo:** son el motor y la base, sin la
ventana de Tauri ni la webview. El presupuesto de 250 MB habla de la aplicación
entera, y esa mitad es R-07.

### Y el que pasa por tres megas

**497 MB de 500.** Entra, pero no sobra nada, y la causa está documentada desde
la Fase 3: `TablaLeida` guarda **todas** las filas en memoria a propósito,
porque la importación las recorre dos veces —una para los choques y otra para
escribir—. Con 500 000 filas el margen es del 0.6 %; con 600 000 no habría.

No es un defecto hoy —el techo declarado es medio millón— pero sí es la cifra
que primero se romperá, y ahora se sabe en lugar de suponerse.

---

## 1. Presupuestos

Medidos en la máquina de referencia (§2), con la base de datos poblada al techo.

> **Ojo:** la tabla de abajo son los presupuestos **estimados en la Fase 0**.
> Lo que de verdad se ha medido está arriba, en el §0, con su fecha.

| Operación | Presupuesto | Por qué ese número |
|---|---|---|
| Arranque en frío hasta interfaz usable | **< 2 s** | Por encima de 2 s la aplicación se percibe pesada |
| Abrir tabla de contactos (500 k) hasta primer pintado | **< 500 ms** | Umbral de «instantáneo» percibido |
| Desplazamiento en tabla virtualizada | **60 fps sostenidos** | Es la pantalla donde el usuario pasa más tiempo |
| Filtro sobre 500 k contactos | **< 1 s** | Por encima se necesita indicador de carga, lo que rompe el flujo |
| Búsqueda por correo | **< 100 ms** | Hay índice único: debe ser instantáneo |
| Importar 100 k filas XLSX | **< 60 s**, memoria acotada | Referencia realista de un archivo grande |
| Importar 500 k filas | **< 5 min**, memoria acotada | El techo |
| Calcular audiencia de campaña | **< 2 s** | Ocurre en el paso 2 del asistente |
| Simulación de campaña | **< 200 ms** | Es aritmética; debe sentirse inmediato |
| Preflight completo | **< 3 s** | Incluye red (auth, DNS) |
| Memoria en reposo con motor activo | **< 250 MB** | La aplicación corre todo el día en segundo plano |
| Memoria durante importación de 500 k | **< 500 MB** | La lectura en streaming lo hace posible |
| Tamaño del instalador | **< 20 MB** | Beneficio directo de Tauri (ADR-0001) |

---

## 2. Máquina de referencia

Deliberadamente modesta: el público objetivo no tiene estaciones de trabajo.

```
CPU:  4 núcleos, ~2.4 GHz
RAM:  8 GB
Disco: SSD SATA
SO:   Windows 11 y macOS 14
Pantalla: 1920×1080 al 100 %
```

Si los presupuestos se cumplen aquí, se cumplen en cualquier equipo corporativo actual.

---

## 3. Cómo se consigue

### Tablas — virtualización obligatoria

TanStack Virtual (ADR-0007). Con 500 000 filas, renderizar sólo lo visible **no es una optimización: es la diferencia entre funcionar y no funcionar.** Un `v-for` sobre 500 k bloquea el hilo principal durante minutos y agota la memoria.

Sólo se montan las filas visibles más un margen pequeño.

### Paginación por keyset, nunca `OFFSET`

```sql
-- Mal: O(n) en el desplazamiento. Al final de 500 k filas, segundos.
SELECT … ORDER BY created_at LIMIT 50 OFFSET 400000;

-- Bien: O(log n) siempre. Milisegundos en cualquier posición.
SELECT … WHERE (created_at, id) > (?, ?) ORDER BY created_at, id LIMIT 50;
```

`OFFSET` obliga a SQLite a recorrer y descartar las filas saltadas. El keyset salta directamente por el índice.

### Índices — sólo los que se usan

```sql
CREATE UNIQUE INDEX idx_contact_unique ON contact (company_id, email_normalized)
  WHERE deleted_at IS NULL;
CREATE INDEX idx_attempt_work ON message_attempt (state, scheduled_for)
  WHERE state IN ('queued', 'failed');
CREATE INDEX idx_attempt_campaign_state ON message_attempt (campaign_id, state);
CREATE INDEX idx_contact_field ON contact_field (contact_id, field_key);
```

**Los índices parciales importan.** `idx_attempt_work` sólo cubre filas accionables: en una campaña de 500 k con 499 k completadas, el índice tiene 1 000 entradas en vez de 500 000.

Cada índice cuesta en escritura, y la importación es la operación de escritura más pesada del producto. No se añade ninguno sin un plan de consulta que lo justifique.

### Importación por lotes

Una transacción por cada **~1 000 filas**. Una sola transacción gigante consume memoria sin límite; una por fila hace 500 000 sincronizaciones a disco.

Lectura XLSX en **streaming** con `calamine`: el archivo nunca se carga entero en memoria. Esto además es la mitigación de zip bomb del modelo de amenazas.

### El motor no bloquea la interfaz

> **Este párrafo decía dos cosas que no eran verdad, y las corrigió la auditoría
> C del §140 el 07/10/2026.** Decía que el motor corre «en su propio conjunto de
> tareas de Tokio» —**el proyecto no usa Tokio**, ni lo ha usado nunca, y
> `Proveedor` es síncrono a propósito— y que la base «vive en un hilo dedicado
> con canal», que era el diseño pensado en la Fase 0 y no el que se construyó.
>
> Es el mismo defecto que la auditoría B encontró un día antes en
> `THREAT_MODEL.md` §8, que pedía «la capa de redacción de `tracing`» sin que
> el proyecto use `tracing`. Dos documentos de la Fase 0 nombrando bibliotecas
> que nunca se adoptaron: lo que describen suena plausible, nadie lo relee, y
> quien llega nuevo busca un `tokio::spawn` que no existe.

**Lo que hay de verdad, medido el 07/10/2026:**

El motor corre en **su propio hilo del sistema** (`std::thread`), que
`arles-app` arranca desde la 5.4 (§51). Cerrar la ventana no detiene una
campaña.

La conexión a la base vive en un **`Mutex<Connection>`** dentro de `Db`
(`arles-db/src/db.rs`). Eso serializa los accesos **dentro del proceso**, así
que un `SQLITE_BUSY` entre el motor y la interfaz no puede ocurrir — pero no
por un canal, y el `busy_timeout(5 s)` de `conexion.rs` **sigue puesto** para
lo que venga de fuera. El cerrojo **no es reentrante**: llamar a un método
público de `Db` desde dentro de un `con_dominio` cuelga el programa sin error
ni traza, y es la trampa que documenta el CLAUDE.md.

Entre procesos no hay reintentos ni canal: hay el **cerrojo de instancia** de
`arles-motor/src/instancia.rs`, que impide que un segundo ARLES abra la misma
base.

---

## 4. Sobrecoste conocido

| Fuente | Coste | ¿Negociable? |
|---|---|---|
| **SQLCipher** | 5–15 % en lectura y escritura | **No** (T-3) |
| Índice único sobre contactos | Escritura más lenta en importación | No — garantiza la deduplicación |
| `audit_log` append-only | Una escritura extra por acción crítica | No — §91 |
| Congelar la audiencia al activar | Una escritura por contacto | No — es lo que hace la campaña auditable |

Los presupuestos de §1 **ya incluyen** estos costes. No son excusas para incumplirlos.

---

## 5. Medición en CI

```
1. Generar 500 000 contactos sintéticos (semilla fija)
2. Poblar una base de datos limpia
3. Ejecutar el banco de pruebas
4. Comparar contra el presupuesto
5. Fallar si alguna operación lo supera en más de un 20 %
```

El margen del 20 % absorbe el ruido de los ejecutores compartidos de CI. Una regresión real lo supera con holgura.

**Los resultados se registran históricamente** para poder detectar degradación lenta: tres regresiones del 15 % consecutivas no rompen ninguna compilación y duplican el tiempo.

---

## 6. Lo que NO se optimiza

Optimizar lo que no lo necesita cuesta legibilidad, y la legibilidad es lo que permite auditar el producto (§140).

| No se optimiza | Por qué |
|---|---|
| Velocidad de envío | **El cuello de botella es el límite del usuario, no nosotros.** 50 correos al día no exigen nada |
| Pantallas de configuración | Se abren una vez |
| Simulador | Es aritmética sobre números pequeños |
| Tamaño del bundle más allá de 20 MB | Ya es 25 veces menor que Electron |
| Arranque por debajo de 2 s | No se percibe la diferencia |

---

## 7. Señales de alarma

Comportamientos que indican un problema de diseño, no de ajuste fino:

- Memoria que **crece sin estabilizarse** con el motor corriendo → fuga
- Desplazamiento que se degrada al bajar en la tabla → la virtualización no funciona
- Tiempo de importación **no lineal** con el número de filas → falta un índice o el lote es incorrecto
- La interfaz se congela al activar una campaña → trabajo en el hilo equivocado
- Consultas que empeoran al crecer la base pese al índice → el plan de consulta no lo está usando

Cada una de estas se investiga como bug, no como ajuste.
