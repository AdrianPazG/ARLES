# Fase 8 · Respaldos y endurecimiento — plan

**Estado:** 📋 planificada, sin empezar · **Escrito:** 2026-10-05
**Peso en la v1.2.0:** 7 de 100 (`avance.json`) · **Depende de:** Fase 7 ✅
**Lo que la sigue:** Fase 9 · Release, que no puede empezar sin las cuatro
auditorías de aquí.

> **En una frase.** Las fases 1 a 7 construyeron un producto que **acumula datos
> que no se pueden reponer**: contactos importados, el consentimiento de cada
> uno, la lista de bajas y el registro de todo lo enviado. La Fase 8 es la que
> responde a «¿y si se pierde?» — y la que comprueba, midiendo, que lo
> construido aguanta el tamaño que prometimos.

---

## 1 · Los tres hallazgos que ordenan la fase

Antes de planificar nada miré qué existe. Salieron tres cosas, y cambian el
orden de las entregas.

### 1.1 · El camino de recuperación **nunca se ha recorrido**

La base está cifrada con SQLCipher y **la clave vive en el llavero del sistema**
(`mx.telemetryinsight.arlesrelay` / `db-master-key`). El ADR-0011 decide que si
el llavero no está, **la aplicación no arranca** — y eso es correcto: lo
contrario sería degradar a texto plano en silencio.

Pero eso deja una consecuencia que nadie ha probado:

> Si se pierde la entrada del llavero, el archivo `arles.db` **sigue ahí y es
> ilegible para siempre**. No hay segunda copia de la clave. El único camino de
> vuelta es un respaldo… que todavía no existe.

Hay una prueba (`F2`, Fase 1) que comprueba que **no** se genera una clave nueva
en silencio, que era el defecto peligroso. Lo que no hay es la otra mitad: qué
hace el usuario a continuación. Hoy la respuesta honesta es «nada».

**Consecuencia para el plan:** el respaldo no es una función de conveniencia. Es
el único seguro contra una pérdida total, y por eso va primero.

### 1.2 · El respaldo `.arles` está decidido **sin cifrar**, y conviene volver a mirarlo

Tres documentos lo dicen, y dicen lo mismo:

- `FUERA_DE_ALCANCE.md` — «la base de datos está cifrada en reposo, pero el
  archivo `.arles` exportado **no lo está** en v1.2.0» (T-6, R-10).
- `PRIVACIDAD_LFPDPPP.md` — «**Hueco declarado** … contiene datos personales. Se
  advierte al usuario; el cifrado entra en v1.3».
- `GLOSARIO.md` — «⚠️ **No cifrado en v1.2.0**».

La decisión está tomada y documentada. **La vuelvo a plantear igualmente**,
porque se tomó en la Fase 0 —cuando el producto no existía— y las dos frases,
juntas, describen algo que entonces no se veía: *el único camino de recuperación
es un archivo sin cifrar con todos los datos personales de los clientes del
cliente*. Va como **D-8A**, con su estructura, en §4.

### 1.3 · El paso de rendimiento del CI lleva desde la Fase 4 «pendiente»

En `.github/workflows/ci.yml`, al final:

```yaml
# ── Pendientes de sus fases del roadmap ──
# Fase 4 · rendimiento: presupuestos con 500 000 contactos sintéticos
```

No se construyó en la Fase 4 y nadie lo movió de sitio. **No hay un solo
`bench`, ni un generador de datos sintéticos, ni una sola cifra medida** contra
las trece de `PRESUPUESTO_RENDIMIENTO.md`. Todas son estimaciones.

Es el §2 en su forma más cara: trece garantías de rendimiento que **nunca se han
visto fallar porque nunca se han comprobado**. Y la auditoría C del §140 exige
«presupuestos medidos, **no estimados**», así que sin esto la fase no cierra.

---

## 2 · Lo que hay hoy, medido

Verificado leyendo el código y el esquema el 05/10/2026.

### Ya existe y sirve

| Qué | Dónde | Para qué sirve en la Fase 8 |
|---|---|---|
| **Entregar un archivo sin cruzar una ruta** | `comandos.rs::entregar_informe_de_acceso` | **El patrón exacto** que debe seguir exportar: el diálogo lo abre el núcleo, se escribe con `std::fs::write`, y lo único que vuelve es el **nombre** del archivo. Regla 4 de la frontera |
| **Decir antes de tocar nada** | `registro.rs::que_se_recortaria` (7.4) | El precedente de un preflight destructivo: primero se dice cuánto se va y cuánto se queda, después se pulsa |
| `audit_log` append-only, por disparador | V1 | «restaurar respaldos» ya está en la lista de acciones críticas del glosario |
| `ElLlaveroDelSistema` y su contrato | `arles-app/src/llavero.rs` | Nombre del servicio fijado por una prueba, precisamente para no dejar credenciales huérfanas (§75) |
| 33 tablas, 13 migraciones (V1–V13) | `arles-db/migrations/` | El inventario de lo que un respaldo tiene que llevar. La cifra se actualiza al añadir una migración: la V13 llegó con la D-13 el 22/10/2026, y una comprobación de la fase 0 la caza si se queda atrás |
| Derechos ARCO completos | 3.4 · `arco.rs`, `PantallaDerechos.vue` | **El ROADMAP los pone en esta fase y ya están.** Ver §3.1 |
| Lectura en streaming con topes | `arles-import` | El patrón de memoria acotada, ya probado con archivos maliciosos |
| `ningun_comando_se_queda_sin_quien_lo_llame` | validador, fase W (7.7) | Los comandos nuevos entran vigilados desde el primer día |

### No existe, y hay que construirlo

| Qué falta | Por qué importa |
|---|---|
| **El formato `.arles`** | No está definido en ningún sitio: ni su estructura, ni su versión, ni qué lleva dentro |
| **Exportar y restaurar** | Nada. Cero líneas |
| **El camino del llavero perdido** | §1.1. Hoy la aplicación no arranca y no hay nada que hacer |
| **Datos sintéticos de 500 000** | §1.3. `ESTRATEGIA_QA.md` §7 los especifica con detalle y no existen |
| **Un solo `bench`** | Trece presupuestos, cero medidos |
| **Las cuatro auditorías** | Nunca se han corrido como las cuatro pasadas aisladas que el §140 pide |

---

## 3 · Qué es la Fase 8, y qué no

El ROADMAP la define en una línea:

> Exportación e importación `.arles` · migración entre sistemas con manejo
> elegante del llavero (§75) · derechos ARCO · **las cuatro auditorías** (§140)
> · presupuestos de rendimiento a 500 k verificados.

### 3.1 · Una de las cinco ya está hecha

**Los derechos ARCO se construyeron en la 3.4**, dos fases antes: Acceso
(`entregar_informe_de_acceso`), Cancelación (`que_se_borraria` +
`cancelar_datos_de`), la pantalla `/contactos/derechos` y la lista de
supresión. El ROADMAP no se actualizó.

No es trabajo perdido: **la auditoría B de §140 vuelve a pasar por encima**, que
es donde toca verificarlos, no reconstruirlos. Lo anoto para que nadie planifique
dos veces lo mismo y para corregir el ROADMAP al cerrar la fase.

### 3.2 · Las tres preguntas que la fase contesta

En este orden, porque es el orden en que importan:

1. **¿Qué pasa si se pierde todo?** → hay un archivo, se restaura, y la
   aplicación dice con precisión qué volvió y qué no.
2. **¿Aguanta lo que prometimos?** → trece cifras medidas en lugar de trece
   estimaciones.
3. **¿Está lo construido a la altura de sus propias reglas?** → las cuatro
   auditorías, en cuatro pasadas separadas.

### 3.3 · Lo que la fase NO hace, y hay que escribirlo antes de que alguien lo pida

- **No es un respaldo automático ni programado.** Es una acción que el usuario
  pulsa. Un respaldo automático necesita decidir dónde, cada cuánto y qué pasa
  cuando el disco se llena — tres decisiones que no están tomadas y que no
  bloquean entregar.
- **No es sincronización entre equipos.** Un `.arles` se exporta y se restaura a
  mano. No hay servidor, y no lo habrá: los datos no salen de la máquina del
  cliente (`PRIVACIDAD_LFPDPPP.md` §6).
- **No lleva credenciales**, y es deliberado (§75). Un respaldo que las llevara
  sería un archivo con acceso a las cuentas de correo del cliente. La
  restauración lo dice con el texto que el modelo de secretos ya redactó.
- **No restaura «a medias».** O entra el respaldo entero o no entra nada. Una
  restauración parcial deja una base en un estado que nadie ha pensado.
- **No es el enforcement de licencias**, que está fuera de alcance por T-6 y D-4
  aunque el esquema se prepare.

---

## 4 · Las tres decisiones — **contestadas el 05/10/2026**

> **Dirección contestó las tres.** Lo que sigue conserva el razonamiento
> completo porque explica *por qué* se construye así, que es lo que hace falta
> dentro de seis meses. La respuesta va arriba de cada una.
>
> | | Decisión | Respuesta |
> |---|---|---|
> | **D-8A** | ¿El respaldo va cifrado? | ✅ **Sí, con opción explícita de desactivarlo** |
> | **D-8B** | ¿Qué se lleva? | ✅ **Dos alcances, «Esencial» por defecto** |
> | **D-8C** | ¿Restaurar sobre datos existentes? | ✅ **Reemplazar, nunca mezclar; bajas unidas** |
>
> Y una pregunta de Dirección que abre una decisión nueva: **D-8D**, en §4 bis.

---

### D-8A · ¿El respaldo va cifrado?  ✅ **Sí, con opción de desactivarlo**

**Problema.** La decisión vigente (T-6) es que el `.arles` de la v1.2.0 **no va
cifrado**, con un aviso al usuario. Al mismo tiempo, los documentos dicen que ese
archivo es **el único camino de recuperación si se pierde la clave del llavero**.
Las dos cosas juntas describen un archivo que contiene todos los contactos, su
consentimiento, la lista de bajas y el registro de envíos — sin cifrar — y que
además es el que el usuario tiene más incentivo en copiar a muchos sitios.

**Riesgo.** Un respaldo se manda por correo, se sube a Drive y se deja en una USB
en un cajón. Cualquiera de esos sitios convierte una fuga de un archivo en una
fuga de la base de clientes entera. Bajo LFPDPPP eso es una vulneración de
seguridad notificable, y el aviso en la interfaz **no la evita**: documenta que
sabíamos.

**Impacto.** Si ocurre, afecta a los titulares de los datos —los clientes de
nuestro cliente—, no a nosotros. Es el tipo de incidente que se descubre tarde y
no tiene marcha atrás.

**Alternativa.** Cifrar el `.arles` con una contraseña que el usuario elige al
exportar, derivada con Argon2id. La dependencia ya está en el árbol por
SQLCipher; el trabajo es de una entrega, no de una fase.

**Trade-off honesto.** Una contraseña que el usuario olvida convierte el respaldo
en inservible — y el respaldo es justo lo que se usa cuando ya se perdió algo. Se
cambia el riesgo de *fuga* por el riesgo de *no poder restaurar*. Por eso la
contraseña se pide dos veces, se avisa con todas las letras de que no hay forma
de recuperarla, y **el cifrado se puede desactivar a propósito** en el mismo
diálogo, con un aviso que diga qué significa.

**Recomendación.** **Cifrar, con opción explícita de no hacerlo.** El riesgo de
olvidar una contraseña lo decide y lo asume quien exporta, en el momento, con la
advertencia delante. El riesgo de la fuga lo asumen terceros que no eligieron
nada. Entre un riesgo que alguien elige y uno que nadie eligió, se protege el
segundo.

*Si la respuesta es «no», se construye sin cifrar y el aviso de la interfaz pasa
a ser obligatorio y no omitible — y se anota como riesgo aceptado con fecha.*

---

### D-8B · ¿Qué se lleva el respaldo?  ✅ **Los dos alcances**

**Problema.** `message_attempt` guarda **un renglón por destinatario y envío, y
no se borra jamás** (D-7C). En una instalación de dos años con campañas
mensuales a 50 000 contactos son millones de filas. Un respaldo que lo lleve
todo es enorme y lento; uno que no lo lleve pierde la prueba de qué se envió a
quién.

**Riesgo.** Si el respaldo tarda veinte minutos y produce un archivo de 2 GB,
nadie lo hace. Un respaldo que no se hace no protege de nada.

**Impacto.** Directo sobre el valor de toda la fase.

**Alternativa.** Dos alcances en el mismo diálogo:
- **Completo** — todo, incluido el registro de envíos. Para migrar de equipo.
- **Esencial** — contactos, consentimiento, **bajas**, plantillas, campañas y
  configuración; del registro de envíos, sólo los desenlaces definitivos de los
  últimos doce meses. Para el respaldo del día a día.

En los dos, las **bajas van completas y nunca se recortan**: son el control
anti-abuso del producto y perderlas significa volver a escribir a quien pidió que
no.

**Trade-off.** Dos modos son dos caminos que probar y dos que explicar. La
alternativa —un solo modo completo— es más simple de construir y peor de usar.

**Recomendación.** **Los dos, con «Esencial» por defecto**, y el diálogo diciendo
en cifras qué se lleva cada uno antes de elegir (§94). La 8.0 declara el
manifiesto de forma que añadir el segundo modo sea una línea.

---

### D-8C · Restaurar sobre una base que ya tiene datos  ✅ **Reemplazar, bajas unidas**

**Problema.** El caso normal de restaurar es un equipo nuevo y vacío. El caso que
rompe cosas es restaurar sobre una instalación **que ya tiene datos**: ¿se
fusiona, se reemplaza, o se rechaza?

**Riesgo.** Fusionar sin querer duplica contactos y, peor, puede **resucitar una
baja**: alguien que pidió no recibir vuelve a estar en la lista porque el
respaldo es de antes de que lo pidiera. Reemplazar sin avisar destruye datos que
no estaban en el respaldo.

**Impacto.** El primero es el daño más caro que este producto puede hacer:
escribir a quien dijo que no.

**Alternativa.** **Reemplazar, nunca fusionar**, con el mismo patrón que el
recorte bimestral de la 7.4: un preflight que dice **antes de tocar nada** qué
hay ahora, qué trae el respaldo y qué se va a perder, con cifras. Y una regla
que no se negocia: **las bajas se unen, no se reemplazan** — la lista final es la
unión de las dos, porque una baja nunca se deshace sola.

**Trade-off.** Fusionar sería más flexible. También sería imposible de explicar y
de verificar, y la flexibilidad aquí no vale lo que cuesta.

**Recomendación.** **Reemplazar con preflight, bajas unidas.** Y queda anotado en
`audit_log`, que ya lista «restaurar respaldos» entre sus acciones críticas.

---

## 4 bis · D-8D · ¿Se puede archivar el historial y borrarlo de la base?

**La pregunta de Dirección, del 05/10/2026**, al contestar D-8B:

> *«¿En esto no podríamos eliminar el registro después de cierto periodo de
> tiempo, con la opción de guardar en una SSD o HDD, para evitar saturar el
> espacio de registros?»*

### Antes de nada: hay dos cosas distintas que se llaman «registro»

Conviene separarlas porque una ya hace lo que se pide y la otra no puede.

| | Qué es | Cuánto crece | Qué se hace hoy |
|---|---|---|---|
| **`activity_event`** · el registro de actividad | Por qué el motor hizo lo que hizo: «a las 10:04 se acabó el cupo» | Mucho, y envejece rápido | **Ya se recorta cada dos meses** (D-7C, 7.4), diciendo antes cuánto se va |
| **`message_attempt`** · el historial de envíos | Un renglón por destinatario: a quién se escribió y cómo acabó | Un renglón por persona y envío, para siempre | **No se borra jamás** (D-7C) |

Lo que satura es el segundo. Y es justo el que no se borra.

### Problema

`message_attempt` crece sin techo por diseño. En una instalación de dos años
mandando a 50 000 contactos al mes son del orden de **un millón de renglones al
año**, y no hay nada que los recorte.

### Riesgo de borrarlos sin más

**Es la prueba de a quién se le escribió.** Tres cosas dependen de ella:

1. **Los derechos ARCO.** Una persona puede preguntar qué le mandamos. Si el
   renglón no está, no hay con qué contestarle — y la obligación legal no
   desaparece porque el dato se haya borrado para ahorrar espacio.
2. **La idempotencia.** `UNIQUE(campaign_id, contact_id)` es lo que impide que
   alguien reciba dos veces el mismo correo. Borrar el renglón quita esa
   garantía para esa campaña.
3. **El resumen exportable** (7.6) y las cifras de Actividad se calculan de ahí.

### Impacto

El primero es el caro: dejar al cliente sin con qué responder a una autoridad.
Los otros dos son recuperables; ése no.

### Alternativa — **lo que Dirección propone, y tiene nombre: archivar**

No es borrar: es **mover**. Sacar los renglones viejos a un archivo que el
usuario guarda donde quiera —esa SSD o ese disco externo— y sólo entonces
quitarlos de la base. El dato sigue existiendo; cambia de sitio.

Y encima **la pieza ya está construida**: un archivo de archivo es un respaldo
con un alcance distinto. El formato de la 8.1 y el exportador de la 8.2 valen
tal cual.

### Trade-off, dicho sin adornos

El archivo pasa a ser **responsabilidad del usuario**. Si lo pierde, perdió el
historial — y con él la capacidad de contestar un ARCO de ese periodo. Hoy ese
riesgo no existe porque el dato nunca sale de la base.

Se cambia **espacio en disco** por **una obligación de custodia**. Y la ley no
acepta «lo tenía en un disco que se rompió» como respuesta.

### Recomendación

**Sí, pero con cuatro condiciones que no son negociables:**

1. **Nunca automático.** Lo pulsa una persona, sabiendo qué hace. Un proceso que
   borra historial solo es un proceso que un día borra el que hacía falta.
2. **El archivo primero, el borrado después** — y sólo si el archivo se escribió
   entero y se volvió a leer para comprobarlo. Si falla cualquier cosa, no se
   borra nada.
3. **Las bajas y el consentimiento nunca entran en esto.** Son de
   `suppression_entry` y `consent_entry`, no del historial, y no se archivan ni
   se recortan jamás.
4. **Queda en `audit_log`**: qué periodo se archivó, cuántos renglones y cuándo.
   Si un día hay que explicar por qué falta un año, que esté escrito.

Y una quinta que es de producto: **la pantalla dice, antes de tocar nada**, qué
periodo se va, cuántos renglones son y la frase que importa — *«a partir de
ahora, para responder una solicitud de datos de ese periodo vas a necesitar este
archivo»*. Mismo patrón que el recorte bimestral de la 7.4.

### Cuándo

**No en la 8.0 ni en la 8.2.** Va **después de la 8.3**, cuando exportar y
restaurar ya funcionen y estén probados: archivar es exportar con otro alcance
más borrar, y construir el borrado antes de que el exportar esté verificado es
construir la parte destructiva primero.

Queda anotado como **entrega 8.3 bis**, y no mueve el resto del plan.

---

## 5 · Las entregas, en orden

El orden no es arbitrario: cada una desbloquea la siguiente, y las dos de
auditoría van al final porque **auditan lo que las anteriores construyeron**.

---

### 8.0 · El manifiesto del respaldo, y la comprobación que impide olvidar una tabla  ✅ **cerrada el 05/10/2026**

**Qué.** Una declaración, en `arles-core`, de qué tablas entran en un respaldo y
en qué modo (completo / esencial / nunca). Más la comprobación que la vigila.

**Por qué va primero, y por qué es la entrega más importante de la fase.**

Es el patrón del §9 esperando a ocurrir, en su forma más cara. El día que
alguien añada una tabla en la V11 y no la ponga en el respaldo, **nada falla**:
se exporta, se restaura, y los datos de esa tabla desaparecen sin un solo error.
Lo descubre el usuario, meses después, cuando ya no hay de dónde sacarlos.

La comprobación —`ninguna_tabla_se_queda_fuera_del_respaldo`— lee las tablas
**del esquema**, no de una lista a mano, y exige que cada una esté clasificada.
Es hermana de `ninguna_tabla_se_queda_sin_escritor`, que ya existe, y se escribe
**antes** que el exportador para verla en rojo (§2).

Una tabla puede declararse «nunca» —`ui_preference` es de este equipo, no de
estos datos— pero hay que **decirlo y por qué**. El olvido no tiene dónde
esconderse.

**Hecho cuando:** las 33 tablas están clasificadas con su motivo, la comprobación
falla al añadir una tabla sin clasificar, y se ha visto fallar.

---

### 8.1 · El formato `.arles`, en el núcleo  ✅ **cerrada el 05/10/2026**

**Qué.** La estructura del respaldo y su serialización. **Cero I/O**: entra una
estructura, sale un texto; entra un texto, sale una estructura o un error
tipado.

**Decisiones de formato, y por qué:**

| Decisión | Por qué |
|---|---|
| Lleva un **número de versión del formato**, separado de la versión de la aplicación | Una v1.3 tiene que poder leer un respaldo de la v1.2. Sin el número, la única forma de saberlo es adivinar |
| Lleva la **versión del esquema** (`V10`) | Restaurar un respaldo más nuevo que la aplicación tiene que **rechazarse con un mensaje claro**, no intentarse |
| Lleva una **suma de verificación** del contenido | Un archivo truncado por un disco lleno o un correo que lo cortó se detecta **antes** de tocar la base, no a mitad |
| Lleva **cuántas filas de cada tipo** declara | Permite enseñar el preflight de la restauración sin leer el archivo entero |
| **No** lleva rutas, ni nombres de equipo, ni nada del sistema | Regla 4 de la frontera, y un respaldo no debe decir de qué máquina salió |

**Hecho cuando:** un respaldo se redacta y se vuelve a leer sin perder nada
(ida y vuelta probada sobre un caso con acentos, comillas y campos vacíos), un
archivo truncado se rechaza con su clave de error, y uno de una versión futura
también.

---

### 8.2 · Exportar

> **Partida en tres el 05/10/2026**, por tamaño. Cifrar, leer la base por tandas
> y construir el comando con su pantalla son tres cosas distintas, y juntarlas
> significa empujar un bloque que no se puede revisar. Es la misma partición que
> la 5.3 y la 6.8.
>
> - **8.2a · El sobre cifrado** ✅ **cerrada el 05/10/2026**
> - **8.2b · Leer la base por tandas**
> - **8.2c · El comando, el diálogo y la pantalla**

#### 8.2a · El sobre cifrado (D-8A)

Argon2id para derivar la clave de la contraseña y XChaCha20-Poly1305 para
cifrar, **en trozos de 64 KiB** con la construcción STREAM. Las dos
dependencias pasaron `cargo deny check licenses advisories`: `advisories ok,
licenses ok`.

**Por trozos y no de una vez**, porque cifrar el cuerpo entero obliga a tenerlo
en memoria dos veces —el texto y su versión cifrada— y el presupuesto fija
500 MB de techo. Y porque el troceado cierra dos agujeros que el cifrado de un
solo bloque deja abiertos: **reordenar trozos** y **cortar el final**, que daría
un respaldo más corto y perfectamente válido.

**La sal y el nonce entran como parámetros**, no se generan dentro: así el
núcleo sigue sin pedir entropía al sistema —cero I/O— y el cifrado se puede
probar con vectores fijos. Una función que genera su propia aleatoriedad sólo se
puede probar cifrando y descifrando, que no distingue «cifra bien» de «cifra y
descifra mal de la misma forma».

**La contraseña entra y no sale.** No va al llavero, no va a la base y no vuelve
por la frontera. Si se olvida, el respaldo no se abre — y construir una puerta
de atrás sería construir justo lo que el cifrado existe para impedir.

#### 8.2b · Leer la base por tandas ✅ **cerrada el 05/10/2026**

`arles-db/src/respaldo.rs`: `Db::exportar` lee todas las tablas del alcance y le
va dando cada fila a un rasgo `Destino`, que en la 8.2c escribirá al archivo.
Trece pruebas y dos comprobaciones nuevas en `--fase 8`.

**«Por tandas» era la unidad equivocada, y conviene dejar dicho por qué**, porque
la forma obvia es la mala. La tanda obvia es `LIMIT 500 OFFSET n`, y SQLite no
guarda un atajo a la fila n: para servir `OFFSET 400000` recorre y descarta
cuatrocientas mil filas. Medio millón en tandas de 500 son mil consultas que
visitan unas **125 000 millones** de filas para leer 500 000. Y lo peor no es la
lentitud: entre dos tandas la tabla puede cambiar —el motor escribe en su propio
hilo—, con lo que **una fila se repite o se salta sin que nada falle**.

Lo que sí es memoria acotada es un **cursor**: `SELECT … ORDER BY rowid` y las
filas se consumen de una en una, con una en memoria. Las tandas sobreviven en su
único sitio útil, **avisar del avance**: cada 2 000 filas, que con medio millón
son 250 avisos en vez de 500 000 cruces de frontera.

**Todo el respaldo va en una sola transacción de lectura.** Sin ella, el
recuento de la cabecera y las filas del cuerpo serían de instantes distintos, y
el preflight de la 8.3 enseñaría una cifra que el archivo no cumple. El coste es
honesto: mientras se exporta, el motor no puede escribir, porque el cerrojo de la
conexión es uno solo (ADR-0002). Exportar es una acción que alguien pulsa y
espera mirando.

**Dos defectos que encontró medir el esquema, y que ninguna prueba del
manifiesto podía cazar:**

1. **El orden del manifiesto no se puede restaurar.** Ordena las tablas «como el
   esquema, agrupando lo que se lee junto», que es bueno para leerlo una persona
   y deja tres hijos antes que sus padres: `contact` antes de `import_batch`,
   `campaign_stage` y `message_attempt` antes de `email_account`. Con
   `PRAGMA foreign_keys=ON` esos `INSERT` se rechazan. Ahora el orden del cuerpo
   **se calcula** leyendo `PRAGMA foreign_key_list`, con el orden del manifiesto
   como desempate para que dos exportaciones de la misma base den el mismo
   archivo.
2. **`import_batch` estaba como `SoloCompleto`** con el motivo «es trazabilidad,
   no dato operativo: sin ella los contactos siguen sirviendo». Suena razonable
   y es falso: `contact` tiene una clave ajena a ella, así que un respaldo
   esencial traía los contactos y **no se podía restaurar**. Reclasificada a
   `SiempreEntera` —es una fila por archivo importado, no pesa— y vigilada por
   `ningun_hijo_viaja_sin_su_padre`, que lee el esquema y comprueba los dos
   alcances, incluido el caso de un padre que sí viaja pero **recortado**.

**Y un arreglo del formato de la 8.1.** El cuerpo era `tabla→v1→v2` sin nada que
dijera qué columna es cada valor, así que un respaldo sólo se podía restaurar
**por posición**. El día que una migración mueva una columna, los campos se
insertan corridos —el apellido donde va la empresa— y ni una fila falla. La
cabecera ahora declara las columnas de cada tabla, y la versión del formato
**no sube** porque no hay un solo `.arles` escrito fuera de las pruebas.

**La columna por la que se recorta vive dentro de la variante del alcance**
—`RecortadaEnElEsencial { fecha }`—, y no en un campo aparte: las dos tablas que
se recortan la llaman distinto, porque `whatsapp_reply` **no tiene `created_at`**
sino `first_reply_at`. Dar por supuesto el nombre habría hecho fallar el `WHERE`
delante de quien pulsó exportar.

> **Hueco del §9 abierto a propósito, y quién lo cierra.** `Db::exportar` está
> construido, probado y **nadie lo llama**: es exactamente el patrón que este
> proyecto ya ha visto trece veces. Lo cierra la 8.2c, y con una comprobación,
> no con buena intención: `el_exportador_tiene_quien_lo_llame` en `--fase 8`,
> escrita **antes** de enchufarlo para verla en rojo.

#### 8.2c · El comando, el diálogo y la pantalla ✅ **cerrada el 06/10/2026**

`arles-app/src/respaldo.rs` y `/ajustes/respaldos`. **ARLES escribe un respaldo
entero**, cifrado o no, y la comprobación que la 8.2b dejó en rojo pasa a verde.

**La forma del archivo: cabecera en claro, cuerpo cifrado.** No es un descuido —
es lo que permite que la restauración (8.3) diga «este respaldo va con
contraseña, y es de hace cuatro meses» **antes** de pedirla. Cifrar la cabecera
obligaría a pedir la contraseña para averiguar si hace falta una contraseña. Y
no filtra nada: versión del esquema, recuento por tabla y nombres de columna.
Ni un dato de una persona, ni una ruta, ni el nombre del equipo.

**Dos arreglos más del formato de la 8.1**, los dos del mismo tipo que el de la
8.2b — cosas que la 8.1 no podía ver porque el cifrado todavía no existía:

1. **La sal y el nonce no tenían dónde ir.** La clave se deriva de la contraseña
   **y de la sal**, y la sal es distinta en cada respaldo a propósito. Sin ellas
   en el archivo, un respaldo cifrado **no se puede abrir nunca**. Ahora viajan
   dentro de la variante `Cifrado::Argon2idXChaCha20 { sal, nonce }`, para que no
   exista el estado imposible: sin cifrar no puede llevar sal, y cifrado no puede
   no llevarla. No son secretos — lo que protege el respaldo es la contraseña,
   que no está en el archivo.
2. **`cifrar` recibía el cuerpo entero en memoria.** La 8.2b se tomó el trabajo
   de leer la base con un cursor para no tenerla en RAM… y cifrarla con esa
   función la habría juntado otra vez. Es el patrón del §9 en su forma más
   silenciosa: **las dos piezas correctas y el agujero en la junta**. Lo cierra
   `SobreEnCurso`, que sella a medida que se escribe y produce **el mismo
   archivo byte a byte** que `cifrar` — lo fija una prueba en los tamaños
   frontera, porque sellar el trozo en cuanto se llena daría otro formato.

**Dos comandos, no uno.** `que_llevaria_el_respaldo` dice con cifras qué se va a
llevar sin escribir nada; `exportar_respaldo` escribe. Es el patrón del recorte
del registro (7.4) y del derecho de cancelación (3.4), y aquí importa por un
motivo propio: «Lo esencial» y «Todo» son dos palabras, «12 536 filas» y
«48 207 filas» son una decisión (§94).

**La pantalla cuelga de Ajustes** (`/ajustes/respaldos`), que es lo que la
cabecera de `PantallaAjustes` ya decía que pertenece ahí: «tema, densidad,
versión, **respaldos**».

**Lo que encontró abrir la demostración y mirar**, con las diez fases en verde:

- **«12536 filas»**, sin separador de miles, cuando Actividad y el registro
  escriben `12,536` desde la 7.2. Ninguna comprobación lo caza: es un número
  correcto.
- **«Repite la contraseña» bloqueado sin decir por qué.** Bloquearlo no es
  cosmética: si sólo se escribiera ahí, el respaldo saldría **sin cifrar**
  mientras alguien cree haberle puesto clave. Pero un campo que no responde y no
  se explica se lee como una avería (§95).

Y una tercera que cazó el validador: **`--arles-font-family-mono` no existe**.
El navegador resuelve un `var()` inexistente a nada, sin error, así que la lista
se habría visto bien. Es el caso de los seis tokens inventados de la 5.2.

**Hecho:** se exporta una base poblada, el archivo se vuelve a leer, se descifra
y cuadra. Falta medir la memoria del proceso con 500 000 filas, que es la 8.5.

#### 8.2c · lo que se escribió

**Qué.** Escribir el archivo: cabecera, cuerpo, suma y sobre cifrado.

**Depende de D-8A** (cifrado) y **D-8B** (alcance).

- Sigue **el patrón del informe ARCO**: el diálogo lo abre el núcleo, lo único
  que vuelve a la webview es el **nombre** del archivo (regla 4).
- **Avance visible**: con medio millón de filas, una barra parada veinte segundos
  es indistinguible de un programa colgado. Cifras, no una rueda (§94). El
  exportador ya avisa cada 2 000 filas.
- La contraseña se pide dos veces, con el aviso de que no se puede recuperar, y
  el cifrado se puede desactivar a propósito (D-8A).
- Queda en `audit_log`.

**Hecho cuando:** se exporta una base poblada, el archivo se vuelve a leer, y la
memoria del proceso se mide y se queda bajo el techo.

---

### 8.3 · Restaurar, con su preflight

> **Partida en dos el 06/10/2026**, por el mismo motivo que la 8.2: leer el
> archivo y reemplazar la base son dos cosas distintas, y la segunda no se puede
> revisar si la primera viene en el mismo bloque.
>
> - **8.3a · Leer un `.arles` de vuelta** ✅ **cerrada el 06/10/2026**
> - **8.3b · Reemplazar, con la unión de bajas y la pantalla**

#### 8.3a · Leer un `.arles` de vuelta ✅ **cerrada el 06/10/2026**

El sobre que se abre a trozos (`SobreQueSeAbre`) y el preflight que dice qué
pasaría (`arles-db/src/restauracion.rs`). Diez pruebas; no se restaura nada.

**El descifrado a trozos es la misma junta que ya falló una vez.** `descifrar`
recibía el cuerpo entero, igual que `cifrar` antes de la 8.2c: restaurar medio
millón de filas con esa función tendría el archivo **y su versión en claro** en
memoria a la vez. Se escribió ahora y no cuando apareciera, porque ya se sabía
dónde estaba.

**Y lo que encontró medir el esquema, que ordena la 8.3b.** D-8C dice
«reemplazar, nunca mezclar», y suena a que cada tabla del respaldo sustituye a
la suya y lo demás se queda igual. Medido el 06/10/2026 sobre el V10, **no es lo
que pasa**: `company` se reemplaza en los dos alcances y casi todo cuelga de
ella con `ON DELETE CASCADE`. Restaurar un respaldo **esencial** se lleva por
delante, sin que nada lo diga:

| Tabla | Qué es | ¿Vuelve? |
|---|---|---|
| `activity_event` | El registro de actividad (7.1) | **No**: es `SoloCompleto` |
| `whatsapp_quality_event` | El histórico de calidad de Meta | **No**: es `SoloCompleto` |
| `rate_budget` | El cupo gastado de esta hora | No, y está bien: es `Nunca` |
| `engine_control` | La parada de emergencia de este equipo | No, y está bien: es `Nunca` |

Es el mismo fallo que en la Fase 1 costó la audiencia congelada y el registro de
envíos (F1). **No se arregla cambiando el esquema** —las cascadas están bien
puestas, protegen de otra cosa— ni escondiéndolo: el preflight lo cuenta y lo
dice antes de que nadie confirme, y distingue lo que son datos de lo que es
estado de este equipo, para que nadie lea cuatro pérdidas donde hay dos.

**La cascada se calcula del esquema**, no de una lista a mano, igual que el
orden de inserción de la 8.2b. Y no se da por buena: `la_cascada_calculada_es_la_que_ejecuta_sqlite`
**borra de verdad** dentro de una transacción que se deshace y compara lo que
desapareció contra lo que el cálculo dijo. Un cálculo corto callaría una
pérdida; uno largo asustaría con una que no ocurre.

**Las bajas prometen un mínimo, no una cifra exacta.** La unión exacta exige
leer el cuerpo entero y comparar canal por canal, y el preflight lee **sólo la
cabecera** a propósito: con 2 GB, leerlo dos veces para enseñar un aviso es
medio minuto de espera antes de una pantalla. Lo que importa de ese número es
que **nunca baja**, y eso sí se garantiza.

#### 8.3b · Reemplazar, con la unión de bajas ✅ **cerrada el 06/10/2026**

`Db::restaurar` en `arles-db/src/restauracion.rs`. Ocho pruebas. **Sigue sin
haber botón**: los comandos y la pantalla son la 8.3c.

**Entero o nada**, y se prueba **provocando el fallo**: el origen revienta en la
tercera fila, con la base ya vaciada y dos filas metidas. Sin la transacción ahí
se quedaría — vacía y a medias. La prueba compara la huella de todas las tablas
antes y después.

**Las bajas se unen en SQL, no en memoria.** Una tabla temporal guarda las de
esta base dentro de la misma transacción y al final vuelven con `INSERT OR
IGNORE`. Leerlas a un `Vec` sería meter la lista entera en RAM justo en la fase
que evita eso en todo lo demás.

**Y vuelven con un identificador nuevo**, que es un detalle que cuesta una baja
si se hace mal. Lo que hace única a una baja es (empresa, canal, dirección), no
su `id`; dos bases generan identificadores aparte y pueden coincidir.
Reinsertando con el `id` original, el choque sería contra la **clave primaria**
y el `OR IGNORE` se tragaría la baja **en silencio**. Con `id` nuevo, el
`OR IGNORE` sólo salta por el índice único: la misma persona en las dos listas,
que es el caso que sí se quiere ignorar.

> **Una baja cuelga de su empresa, y eso casi rompe D-8C.**
>
> Lo encontró la prueba, no el diseño: `suppression_entry` tiene clave ajena a
> `company`, y `company` también se reemplaza. Si el respaldo trae otro
> identificador de empresa, las bajas de aquí vuelven apuntando a una empresa
> que ya no existe, SQLite las rechaza y **la restauración entera se deshace**.
>
> De las tres salidas, dos son inaceptables: descartar las bajas huérfanas
> **pierde bajas**, que es lo único que esta fase existe para impedir; y dejar
> que falle convierte D-8C en «no se puede restaurar».
>
> Así que las bajas que sobreviven se **reapuntan** a la empresa que quedó.
> ARLES es de una sola empresa, de modo que en el caso real es la misma
> organización; y en el caso raro —restaurar el respaldo de otra— el error cae
> del lado seguro: **de más, nunca de menos**. Suprimir de más significa no
> escribirle a alguien; de menos, escribirle a quien pidió que no.
>
> **Y si el respaldo no trae ninguna empresa, se para en vez de restaurar**: sin
> empresa no hay dónde colgarlas, y perderlas no es una opción.

**Las cuentas vuelven en `needs_reauth`**, nunca en `connected`: el respaldo no
lleva credenciales (§75), y una cuenta dada por buena sin contraseña haría que
el motor fallara contacto a contacto gastando el cupo del día.

**Las filas entran por nombre de columna**, nunca por posición, y una columna que
esta base no tiene **para la restauración** en vez de insertar las que encajen:
los campos que faltaran se quedarían en su valor por defecto, que para un
apellido es nulo y para una fecha es basura.

**Un respaldo de otro esquema se rechaza.** Es una limitación real y conviene no
esconderla: hoy, un respaldo tomado antes de una migración futura **no se podrá
restaurar**. Restaurarlo exigiría correr las migraciones sobre el propio
respaldo, que es otra entrega. Entre eso y restaurar «lo que encaje», se para:
media base restaurada arranca, parece funcionar, y lo que falta se descubre
semanas después.

#### 8.3c · Los comandos y la pantalla ✅ **cerrada el 06/10/2026**

`arles-app/src/restauracion.rs` y `/ajustes/restaurar`. **ARLES restaura un
respaldo entero.** Seis pruebas, y la comprobación
`el_restaurador_tiene_quien_lo_llame` escrita antes y vista en rojo nombrando
los seis tramos.

**Dos comandos, y el orden no se puede saltar.** `revisar_respaldo` lee sólo la
cabecera y dice qué trae y qué se pierde; `restaurar_respaldo` usa la ruta que
el primero guardó en `EstadoApp`. La ruta no cruza la frontera (regla 4), y eso
además impide que la webview pida restaurar **otro** archivo distinto del que se
revisó — la misma ventana que la importación cerró en la Fase 3.

**La suma se comprueba dentro de la transacción**, al llegar a la línea de
cierre, no después. Un archivo alterado o cortado revienta antes del `COMMIT`.
Al revés dejaría la base reemplazada con datos de un archivo que no es el que
salió de aquí, y un aviso después.

**La prueba que la fase existía para tener**: base poblada → archivo → **otra
base** → los mismos contactos, con y sin contraseña. Ninguna entrega anterior
podía escribirla: la 8.2c comprobaba que el archivo se relee, la 8.3b que la
base se reemplaza, y nadie que lo que sale de una entre de verdad en la otra.

**Tres defectos que encontró mirar la demostración**, con las diez fases en
verde:

- **El botón de elegir medía 540 px** y los de abajo 200 y 96: era el único hijo
  de una columna flexible y se estiraba. No lo caza ninguna sonda y se ve a un
  metro.
- **«Restaurar y reemplazar» apagado sin decir por qué.** Falta la contraseña, y
  un botón que no responde en esta pantalla concreta —a la que se llega después
  de perder algo— se lee como que ARLES tampoco funciona (§95). Es el mismo
  defecto que la 8.2c encontró con «Repite la contraseña».
- **«2 cuenta(s)»**, con el paréntesis. Pluralizado como Remitentes desde la
  5.3c.

Y uno que cazó el validador: **tres tokens de diseño inventados**
(`--arles-text-primary`, `--arles-text-secondary`, `--arles-border-subtle`).
Tercera vez en la fase. Los de verdad son `--arles-text`, `--arles-text-muted` y
`--arles-border`.

**La vista previa contesta `revisar_respaldo` con cifras y rechaza
`restaurar_respaldo`.** Revisar es una lectura, y fabricar lecturas es para lo
que existe el decorado; fingir el éxito de la acción más destructiva del
producto, no. El error que devuelve —«no se pudo abrir el archivo»— es verdad
allí: no hay ningún archivo en disco.

#### 8.3 bis · Archivar el historial (D-8D)

**Qué.** Leer un `.arles`, decir **qué va a pasar**, y sólo entonces hacerlo.

**Depende de D-8C.**

El preflight dice, antes de tocar nada y con cifras:

- Qué trae el respaldo: contactos, bajas, campañas, plantillas, y de cuándo es.
- Qué hay ahora, y **qué se va a perder**.
- Qué **no** trae: las credenciales. Con el texto que `MODELO_DE_SECRETOS.md` §7
  ya redactó, palabra por palabra.
- Si el respaldo es de una versión que esta aplicación no sabe leer, lo dice
  aquí y no deja continuar.

Y cuando se confirma: se restaura **entero o nada** (una transacción), las
cuentas aparecen en `needs_reauth` con su botón de reconectar, y queda en
`audit_log`.

**Hecho cuando:** restaurar sobre una base vacía funciona; restaurar sobre una
poblada avisa de lo que se pierde y, confirmado, lo reemplaza; **las bajas de las
dos se unen**; y una restauración interrumpida a mitad deja la base como estaba.
Lo último se prueba **provocando el fallo**, no suponiéndolo.

---

### 8.4 · El llavero perdido: el camino que nadie ha recorrido ✅ **cerrada el 06/10/2026**

`arles-app/src/recuperacion.rs` y `/recuperar`. Cinco pruebas, y
`la_clave_perdida_tiene_salida` escrita antes y vista en rojo nombrando los
seis tramos.

**El consejo que el producto no podía cumplir.** Desde la Fase 1, perder la
entrada del llavero producía un mensaje correcto con las tres partes del §95, y
la de «cómo se arregla» decía *«Restaura tu último respaldo .arles»* — y no
había forma de hacerlo: `run()` llamaba a `exit(1)` antes de abrir ninguna
ventana. No es que faltara un botón: el producto **daba una instrucción que él
mismo impedía seguir**, el día en que alguien ya ha perdido algo.

**Las dos cosas que aquí no se pueden hacer mal:**

1. **No se crea una clave nueva al arrancar.** Es el hallazgo F2 de la Fase 1.
   El llavero puede estar vacío por motivos pasajeros —un Keychain bloqueado, un
   perfil que no ha montado—, y una clave nueva dejaría la base existente
   ilegible **para siempre**. La clave nueva sólo nace cuando alguien elige un
   respaldo y confirma.
2. **La base ilegible no se borra: se aparta** con fecha en el nombre. Un
   llavero puede volver, y el día que vuelva esos datos valen más que el
   respaldo porque son los de hoy. Recuperar dos veces el mismo día tampoco pisa
   la copia de la primera.

**El modo de recuperación registra tres comandos y ninguno más.** Ninguno de los
normales puede funcionar sin base, y un comando registrado que siempre falla es
una puerta que nadie vigila. La pantalla tampoco lleva barra lateral: sin base
no hay secciones a las que ir, y enseñarlas sería ofrecer seis puertas pintadas.

**Si no hay respaldo, se dice con esas palabras**: no hay forma de recuperar
esos datos, ni desde ARLES ni de ninguna otra manera, porque el cifrado no tiene
puerta de atrás. Dejar lugar a la duda haría buscar durante horas una salida que
no existe.

> **Límite declarado.** El contenedor de integración no tiene llavero —ni D-Bus—,
> así que la prueba del arranque **declara su rama y se omite aquí**. Lo que sí
> corre entero es la mecánica de la recuperación, escrita a propósito sin
> depender del llavero: con él, en este contenedor, habría pasado en verde sin
> ejercitar una línea. En Windows y macOS se prueba el día que se cierre **R-07**.

**Un defecto que encontró mirar la demostración**, y era el peor posible en esta
pantalla: decía **«no se ha perdido nada»** con 48 MB de base en disco, porque
nadie llamaba a `cargar()`. El patrón del §9 otra vez, y aquí la consecuencia no
es una función que falta: es alguien que borra el archivo porque le dijimos que
estaba vacío. También le faltaba la marca — es la única pantalla sin barra
lateral, y sin ella una ventana suelta con un aviso rojo no dice de qué programa
es.

---

#### 8.4 · lo que pedía el plan

**Qué.** La otra mitad del ADR-0011.

Hoy, sin la entrada del llavero, la aplicación no arranca — correcto — y punto.
Esta entrega hace que a continuación haya algo que hacer:

- **Decir qué pasó, con las tres partes del §95**: qué pasó, cómo se arregla y
  **qué está a salvo**. La tercera es la que importa: los datos no se han
  borrado, están cifrados con una clave que ya no está.
- Ofrecer **empezar de cero restaurando un respaldo**, que es la única salida
  real.
- **No ofrecer nada que no funcione.** Si no hay respaldo, se dice que no hay
  salida, con esas palabras. Un botón que no puede funcionar es peor que ninguno.

**Hecho cuando:** se borra la entrada del llavero a propósito, se arranca, y la
pantalla dice las tres cosas y lleva a restaurar. Se verifica **rompiéndolo**.

> **Límite declarado:** esto se puede probar entero en Linux, donde el llavero se
> simula. En Windows y macOS se prueba cuando se cierre **R-07**. Se dirá así, y
> no «probado» a secas.

---

### 8.5 · Los 500 000 contactos sintéticos, y trece cifras medidas ✅ **cerrada el 06/10/2026**

El banco (`crates/arles-banco`), el generador reproducible y **las trece cifras
medidas o declaradas** en `PRESUPUESTO_RENDIMIENTO.md` §0.

**Cinco medidas, ocho sin medir, y dos de las cinco fallan.** El plan lo
anticipaba —«que salgan los trece a la primera sería la señal de que el banco no
está midiendo lo que dice»— y así fue:

| | |
|---|---|
| Importar 500 000 | **51.7 s** de 5 min ✅ |
| Memoria importando | **497 MB** de 500 ⚠️ |
| Búsqueda por correo | **331 ms** de 100 ❌ |
| Abrir la tabla (la consulta) | **1 424 ms** de 500 ❌ |
| Simulación de campaña | **5.9 ms** de 200 ✅ |

**Los dos fallos tienen causa concreta**, encontrada con `EXPLAIN QUERY PLAN` y
no supuesta: `contact` **no tiene índice para su propia consulta** —lee las
455 864 filas y las ordena en memoria para enseñar cien—, y la búsqueda por
correo no filtra por `channel`, así que del índice único sólo puede aprovechar
la primera columna y recorre todos los canales de la empresa.

**El generador no viaja en el instalador.** Es un crate aparte del que no
depende `arles-app`, porque uno de los trece presupuestos es el tamaño del
instalador y meter un generador de medio millón de filas dentro para poder
medirlo sería el instrumento falseando lo que mide. Lo vigila
`el_banco_no_viaja_en_el_instalador`.

**El azar es propio, no de una dependencia.** El requisito es reproducible, y
una biblioteca de números aleatorios promete calidad estadística —que aquí da
igual— y no que su algoritmo no cambie entre versiones. SplitMix64 cabe en diez
líneas y da lo mismo en cualquier máquina.

**Lo que el propio banco encontró de sí mismo**, y es el patrón de siempre: la
primera versión validaba cada fila y se las daba todas a `importar_contactos`,
saltándose el análisis de choques. Con medio millón de filas sintéticas eso
reventó en la primera corrida — el conjunto trae duplicados a propósito. No era
«medir sin la interfaz»: era medir un camino que el usuario nunca recorre.

**Y dos defectos de las propias comprobaciones**, los dos del §2:

1. El patrón de fecha era `\b0[1-9]|[12]\d|3[01]/\d{2}/20\d{2}\b`, y la
   alternancia ata antes que la concatenación: casaba «20» en cualquier parte.
   Daba por fechado un documento sin una sola fecha.
2. La comprobación de reproducibilidad **se cazaba a sí misma en un
   comentario**: salió en rojo señalando un generador correcto porque el
   comentario de la prueba cuenta que se verificó sembrando con `SystemTime`.
   Es la misma trampa que la 5.3 ya documentó.

---

#### 8.5 · lo que pedía el plan

**Qué.** El generador, el banco de pruebas y el paso de CI que lleva pendiente
desde la Fase 4 (§1.3).

**El generador**, tal como `ESTRATEGIA_QA.md` §7 ya lo especificó: semilla fija y
reproducible, con acentos, correos en mayúsculas y con `+`, campos vacíos,
duplicados exactos y por normalización, direcciones inválidas y **nombres que
parecen fórmulas**. Nunca datos reales, ni anonimizados.

**Las trece cifras** de `PRESUPUESTO_RENDIMIENTO.md` se miden. Las que no se
puedan medir desde aquí —las que necesitan la máquina de referencia o una
pantalla— **se declaran sin medir**, que no es lo mismo que cumplidas. Es la
misma regla que el validador aplica a las sondas omitidas.

**Y se espera que algo falle.** Trece presupuestos estimados en la Fase 0 y nunca
comprobados: que salgan los trece a la primera sería la señal de que el banco de
pruebas no está midiendo lo que dice. Lo que se mida se escribe con su fecha,
pase o no.

**Hecho cuando:** el conjunto se genera dos veces y sale idéntico; las cifras
están medidas y escritas; y lo que no se pudo medir dice por qué.

---

### 8.6 · Las cuatro auditorías (§140)

**Qué.** Cuatro pasadas **separadas**, no una revisión general. Es la condición
del §140 y tiene motivo: una revisión general encuentra lo que salta a la vista,
cuatro pasadas con una pregunta cada una encuentran lo que no.

| | Pasada | Qué se mira |
|---|---|---|
| **A** | Arquitectura | Fronteras entre crates · `arles-core` sin I/O · la webview sin lógica de negocio · sin `unwrap()` ni `any` indiscriminados (§138) |
| **B** | Seguridad | `THREAT_MODEL.md` §8 entero · ningún comando devuelve un secreto · capabilities denegadas por defecto · CSP · sanitizado en Rust · topes de importación · `cargo audit` y `npm audit` · **y los derechos ARCO de la 3.4, que es donde tocaba verificarlos** |
| **C** | QA y rendimiento | Casos frontera · **los presupuestos de la 8.5, medidos y no estimados** · sin fugas de memoria en ejecuciones largas del motor |
| **D** | UX y accesibilidad | Contraste AA calculado · foco visible · teclado · ningún estado sólo por color · `prefers-reduced-motion` · objetivos ≥32 px al 200 % · **los cuatro estados en cada pantalla (§97)** · errores con las tres partes (§95) |

**C depende de la 8.5**, por eso va después.

**Cada hallazgo se arregla o se declara**, con su motivo y su fecha. Una auditoría
que produce una lista y no produce cambios no es una auditoría: es un documento.

> **Lo que la auditoría D no puede hacer desde aquí:** mirar las pantallas en
> WKWebView. Eso es **R-07** y sigue esperando a alguien con un Mac. La vista
> previa no cuenta: ahí el motor es el del navegador.

---

### 8.7 · Cierre de fase

Documento de fase, informe para Dirección, avance recalculado, y **la revisión a
ojo** en Chromium que la 7.7 volvió a demostrar que hace falta: tres de los siete
hallazgos de aquella sólo se vieron mirando, con las sondas en verde.

Y una corrección pendiente: **el ROADMAP pone los derechos ARCO en esta fase y
están desde la 3.4** (§3.1).

---

## 6 · Escalabilidad: lo que esta fase decide para las que vienen

Esta fase toca la frontera entre lo que el producto guarda y el mundo exterior.
Lo que se decida aquí es caro de cambiar después, porque habrá archivos `.arles`
en discos de clientes.

**1 · El formato tiene versión desde el primer archivo.** No se añade después:
un archivo sin número de versión es indistinguible de uno de la versión 1, y en
cuanto exista uno de esos en un disco ajeno, la compatibilidad se vuelve
adivinación. La regla: **la aplicación lee su versión y las anteriores; una
posterior se rechaza con un mensaje que dice qué versión hace falta.**

**2 · El manifiesto hace que crecer sea barato.** Con la 8.0 en su sitio, añadir
una tabla en la V11 obliga a clasificarla. Sin ella, cada migración futura es una
oportunidad de perder datos en silencio. Es la diferencia entre un sistema que
aguanta crecer y uno que acumula deuda invisible.

**3 · El cifrado del respaldo es reversible en una dirección y no en la otra.**
Si la v1.2 exporta sin cifrar y la v1.3 cifra, la v1.3 tiene que seguir leyendo
los archivos viejos — fácil. Si la v1.2 cifra, no hay nada que mantener después.
Es un argumento más para D-8A, y es de ingeniería, no de seguridad.

**4 · El alcance «Esencial» es lo que mantiene el respaldo usable con el
tiempo.** `message_attempt` crece sin techo por diseño (un desenlace no se borra
jamás). Sin dos alcances, el respaldo se vuelve impracticable justo en las
instalaciones con más que perder.

**5 · Los presupuestos medidos son la línea base de todo lo que venga.** Trece
cifras con fecha permiten detectar degradación lenta — tres regresiones del 15 %
no rompen nada y duplican el tiempo. Sin la primera medición no hay contra qué
comparar, y la v1.3 empezaría igual de ciega.

**6 · Lo que esta fase deja preparado para la v1.3**, sin construirlo: el
cifrado del respaldo (si D-8A dice que no), el respaldo programado, y el
`FeatureEntitlement` que el esquema ya contempla para el licenciamiento.

---

## 7 · Riesgos de la fase

| Riesgo | Probabilidad | Impacto | Mitigación |
|---|---|---|---|
| Una restauración destruye datos que no estaban en el respaldo | **Alta** si no se controla | **Muy alto**, irreversible | D-8C: preflight con cifras antes de tocar nada, y bajas unidas |
| Se añade una tabla y nadie la mete en el respaldo | **Alta** con el tiempo | Alto y silencioso | La 8.0, escrita antes que el exportador |
| El `.arles` sin cifrar se filtra | Media | Muy alto, y de terceros | D-8A |
| Los presupuestos no se pueden medir aquí como en la máquina de referencia | **Alta** | Medio | Se mide lo que se pueda y **se declara qué no**, como con las sondas omitidas |
| El llavero sólo se prueba en Linux | **Alta** (ya pasa) | Medio | Se declara; se cierra con R-07 |
| Las cuatro auditorías producen más trabajo del que cabe en la fase | Media | Medio | Cada hallazgo se arregla **o se declara con fecha**. No se tapan |
| Las pruebas de 500 k hacen el CI lento | Media | Bajo | Van en su propio trabajo, no en cada push |

---

## 8 · Cómo se verifica la fase

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check licenses advisories        # si D-8A añade dependencia de cifrado

cd app && npm test && npm run lint && npx vue-tsc --noEmit -p tsconfig.json

# Las DIEZ fases, con sondas. Correr sólo la fase en curso no basta: está
# contado en ESTADO_ACTUAL.md §8 bis y volvió a pasar en la 7.7.
for f in 0 1 2 3 4 5 6 W 7 8; do python3 herramientas/validar/validar.py --fase $f; done
```

Y lo que ninguna orden hace: **abrir la demostración y mirar**. Va en la 8.7 y no
es opcional.

**La fase no está cerrada mientras:**

- [ ] Una tabla pueda quedarse fuera del respaldo sin que nada falle
- [ ] Restaurar pueda destruir datos sin avisar antes, con cifras
- [ ] Los presupuestos sigan siendo estimaciones
- [ ] Alguna de las cuatro auditorías no se haya corrido **aislada**
- [ ] Un hallazgo de auditoría esté sin arreglar **y sin declarar**

---

## 9 · Resumen para quien tenga prisa

| | Entrega | Depende de | Bloquea |
|---|---|---|---|
| **8.0** ✅ | El manifiesto, y la comprobación que impide olvidar una tabla | — | todo lo demás |
| **8.1** ✅ | El formato `.arles` en el núcleo | 8.0 | 8.2, 8.3 |
| **8.2** | Exportar | 8.1 · **D-8A** ✅, **D-8B** ✅ | 8.3 |
| **8.3** | Restaurar, con su preflight | 8.2 · **D-8C** ✅ | 8.4 |
| **8.3 bis** | Archivar el historial a un disco externo, y sólo entonces borrarlo | 8.3 · **D-8D** ✅ | — |
| **8.4** | El llavero perdido | 8.3 | — |
| **8.5** | 500 k sintéticos y trece cifras medidas | — *(se puede adelantar)* | 8.6 C |
| **8.6** | Las cuatro auditorías (§140) | 8.5 | Fase 9 |
| **8.7** | Cierre: documento, informe, avance y **mirar** | 8.6 | Fase 9 |

**Cerradas la 8.0, la 8.1, la 8.2a y la 8.2b** (05/10/2026). El manifiesto
clasifica las 29 tablas con su motivo; el formato `.arles` se escribe y se vuelve
a leer sin perder nada, con tres errores tipados que distinguen «este archivo no
es un respaldo», «lo hizo una versión más nueva» y «está roto» — porque lo que
hay que hacer es distinto en cada caso; el sobre va cifrado con Argon2id y
XChaCha20-Poly1305 en trozos de 64 KiB; y la base se lee con un cursor, en una
sola transacción, con el orden de inserción calculado del esquema.

**Cerrada también la 8.2c** (06/10/2026): **ARLES escribe un respaldo entero**,
cifrado o no, desde `/ajustes/respaldos`. El hueco del §9 que la 8.2b dejó
abierto a propósito está cerrado, y lo vigila `el_exportador_tiene_quien_lo_llame`.

**Lo siguiente es la 8.3 · Restaurar, con su preflight** (D-8C). Lo que hay
ahora es la mitad de la fase: se puede escribir un respaldo y **todavía no se
puede leer de vuelta**, que es para lo que existe.
