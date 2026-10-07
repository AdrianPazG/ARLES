# Auditoría F · la Fase 8 entera, por sus junturas

**Fecha:** 07/10/2026
**Alcance:** las once entregas de la Fase 8 (8.0 a 8.6, 8.3 bis a y b, 8.4, 8.5,
8.6 A–E), vistas **como un sistema** y no una por una.
**Estado al empezar:** las diez fases del validador en verde, 1 000+ pruebas en
verde, las cuatro auditorías del §140 cerradas.

**Resultado: cinco hallazgos, cinco arreglados. Uno de ellos impedía restaurar
cualquier respaldo cifrado de tamaño real** — es decir, el camino que D-8A hace
el valor por defecto, en cualquier base que no sea de juguete.

---

## 0 · Por qué esta pasada, y por qué encontró lo que las otras no

Las cuatro auditorías del §140 miran el producto por capas: arquitectura,
seguridad, QA, UX. Cada entrega de la Fase 8, además, se cerró con sus pruebas
en verde. Lo que ninguna de las dos cosas mira es **la junta entre dos
entregas**: la 8.2c comprobaba que el archivo se relee, la 8.3b que la base se
reemplaza, la 8.4 que el llavero perdido tiene salida — y cada una con los datos
que le convenía.

Esta pasada hizo una sola pregunta, once veces:

> **¿Qué pasa si lo que sale de una entrega entra en otra, a tamaño real y por
> el camino que el usuario recorre de verdad?**

Los cinco hallazgos salieron de ahí. Ninguno es un error de lógica dentro de un
módulo: los cinco viven entre dos.

---

## 1 · Los cinco hallazgos

| Id | Qué | Gravedad | Estado |
|---|---|---|---|
| **F-1** | Ninguna prueba de recuperación usaba un respaldo **cifrado**, que es el valor por defecto (D-8A) | Media · un camino entero sin recorrer | Arreglado |
| **F-2** | La recuperación **apartaba la base y creaba la clave del llavero antes de validar la contraseña** | **Alta** · un error de tecleo volvía definitivo | Arreglado |
| **F-3** | El mismo fallo de orden tenía dos disparadores más, y la pantalla **decía que sí se podía restaurar** un archivo de historial | **Alta** · misma consecuencia que F-2 | Arreglado |
| **F-4** | Ninguna de las trece cifras del presupuesto medía exportar, restaurar ni archivar | Media · la fase que recorre la base entera, sin medir | Medido |
| **F-5** | **Un respaldo cifrado de más de 64 KiB con acentos no se podía restaurar** | **Crítica** · la copia de seguridad no abría | Arreglado |

---

## 2 · F-5 · El respaldo que no se podía restaurar

**Es el peor hallazgo que esta fase podía dejar abierto**, y conviene entender
por qué estuvo invisible hasta el final.

El cuerpo de un respaldo cifrado se descifra **en trozos de 64 KiB** (8.2a), y
cada trozo se convertía a texto por su cuenta:

```rust
self.pendiente.push_str(std::str::from_utf8(&claro)?);   // ← el fallo
```

Un trozo de 64 KiB **no es una unidad de texto**: la frontera cae donde cae, y
una `Á`, una `ñ` o una `é` ocupan dos bytes. En cuanto una frontera parte una
letra acentuada —en una base mexicana, constantemente— el trozo no es UTF-8
válido y el lector devuelve `RespaldoCorrupto`: *«el contenido del respaldo no
es texto válido»*, **sobre un archivo perfectamente bueno**.

### Lo que esto significaba en la práctica

Con 30 000 contactos el cuerpo son 8.3 MB, unas 130 fronteras de trozo. La
probabilidad de que **ninguna** caiga en un carácter de dos bytes es
despreciable. Así que:

- El respaldo se escribía bien, con su suma correcta.
- La pantalla de restaurar lo leía bien: cabecera, fecha, recuento, qué se
  pierde. Todo correcto.
- Y al pulsar «Restaurar y reemplazar», **fallaba diciendo que el archivo estaba
  estropeado**.

Una copia de seguridad que existe, parece correcta, y no abre el día que hace
falta. Es exactamente el fallo que la Fase 8 existía para no tener.

### Por qué ninguna prueba lo cazaba

Dos razones, y las dos son la lección:

1. **Todas las pruebas de viaje completo usaban dos filas.** Un cuerpo de dos
   filas cabe en un trozo, así que no hay ninguna frontera. La prueba que la
   fase estaba orgullosa de tener —«base poblada → archivo → otra base → los
   mismos contactos»— no podía ver este fallo nunca.
2. **El último trozo siempre acaba en `\n`**, que es un byte ASCII. Así que el
   único trozo que las pruebas ejercitaban era precisamente el único que no
   puede fallar.

Apareció **midiendo F-4**, con 30 000 contactos. No lo encontró una revisión del
código: lo encontró poner datos de verdad.

### El arreglo

Una cola de bytes que espera al trozo siguiente:

```rust
let mut bytes = std::mem::take(&mut self.cola);
bytes.extend_from_slice(&claro);
// …decodificar hasta donde haya caracteres completos…
self.cola = bytes.split_off(valido);
```

Y dos cosas que el arreglo **no puede** perder, las dos con su prueba:

- **Un corte de trozo y un archivo manipulado son distintos.**
  `Utf8Error::error_len()` los separa: `None` es «se acabó la entrada a mitad de
  un carácter» —la frontera, normal— y `Some` es una secuencia imposible. Lo que
  esa rama garantiza **no es el rechazo** (la guarda del final ya rechazaría el
  archivo, al llegar al cierre) sino *cuándo y a qué coste*: sin ella el byte
  malo se atasca en la cola, cada trozo siguiente se le pega sin poder
  decodificar nada, y **la cola crece hasta el tamaño del archivo** — medio
  millón de filas en memoria, con la memoria acotada de la 8.2b por el suelo.
  Lo mide la prueba: con la rama, 3 bytes; sin ella, el resto del archivo.
- **Un archivo que acaba a mitad de carácter se rechaza.** Eso ya no es una
  frontera: es un archivo cortado, y tiene que decirlo en vez de entregar una
  fila truncada.

---

## 3 · F-2 y F-3 · Lo irreversible ocurría antes de lo comprobable

`recuperar_desde_respaldo` —la salida del llavero perdido, 8.4— hacía las cosas
en este orden:

1. Apartar la base ilegible.
2. **Crear una clave nueva en el llavero.**
3. Abrir la base nueva.
4. Leer el respaldo… y **ahí** descubrir que la contraseña estaba mal.

El paso 2 no tiene vuelta. El llavero puede estar vacío por un motivo pasajero y
arreglarse solo, pero **no después de que le escribamos encima**: eso es el
hallazgo F2 de la Fase 1, el que esta pantalla existe para no repetir. Así que un
error de tecleo en la contraseña del respaldo —la que nadie guarda en ningún
sitio (8.2a)— convertía un problema reversible en uno definitivo.

Y había dos disparadores más del mismo fallo, los dos conocibles **de la cabecera
antes de tocar nada**:

- Elegir un **archivo de historial** en vez de un respaldo. El escenario
  realista: quien perdió el llavero busca «el archivo de ARLES» y encuentra dos.
- Un respaldo de **otro esquema**, que hoy se rechaza por diseño (8.3b).

Peor: la pantalla de recuperación **decía que sí se podía restaurar** un archivo
de historial, porque el campo `se_puede_restaurar` sólo comparaba esquemas y no
preguntaba por el alcance. Luz verde, confirmación, clave nueva, y rechazo.

**El arreglo es una regla, no un parche:** `comprobar_el_respaldo_antes_de_tocar_nada`
comprueba **todo lo que `Db::restaurar` rechazaría y se puede saber de
antemano** —alcance, esquema, y que el archivo se abra y se descifre— y sólo
entonces se toca la base y el llavero.

> **Nada irreversible ocurre hasta que todo lo conocible está comprobado.**

El pre-chequeo pide **la primera fila**, no sólo abre el archivo: el cifrado
autenticado no sabe si la contraseña es buena hasta que una etiqueta cuadra
(8.2a), así que validarla exige descifrar algo.

### Un hallazgo falso descartado

La pantalla **de restaurar** parecía tener el mismo defecto: su
`se_puede_restaurar` también compara sólo esquemas. **No lo tiene**: allí el
preflight de `arles-db` corre antes y ya rechaza el archivo de historial, así que
el comando entero falla y nunca se ofrece el botón. Descartado leyendo el
recorrido completo, no la línea.

Es el cuarto falso descartado en las cinco pasadas, y la lección de B y C sigue
siendo la misma: **antes de anotar que algo falta, se lee cómo funciona de
verdad**.

---

## 4 · F-1 · El camino por defecto, sin recorrer

Tres decisiones se cruzan en la pantalla de recuperación y ninguna prueba cruzaba
las tres:

- **D-8A** · el respaldo va cifrado, y es el valor por defecto.
- **8.4** · perder el llavero se arregla restaurando un respaldo.
- **8.2a** · la contraseña del respaldo **no se guarda en ningún sitio**.

Así que el desastre realista no es «perdí el llavero»: es **«perdí el llavero y
mi único respaldo está cifrado»**. El comando aceptaba una contraseña desde la
8.4 y nunca se había probado con una.

Dos pruebas nuevas: una recupera de un respaldo cifrado de verdad —comprobando
antes que el cuerpo **no** se lee en claro en el archivo, porque si saliera sin
cifrar el resto de la prueba pasaría igual sin demostrar nada—, y otra comprueba
que con la contraseña mal escrita **no se toca nada**: la base sigue en su sitio,
con su tamaño, y el llavero sin clave nueva.

---

## 5 · F-4 · Las cuatro cifras que el presupuesto no tenía

`PRESUPUESTO_RENDIMIENTO.md` tiene trece cifras y **ninguna medía el respaldo, la
restauración ni el archivado** — todo lo que construyó la Fase 8, y lo único del
producto que recorre la base entera, dentro de un comando de Tauri, con el
cerrojo de la conexión tomado.

Es el mismo hueco que la auditoría C encontró con la activación de campañas, y
ahí resultó ser el peor número del producto (73.5 s de un presupuesto de 2 s).

**Medido el 07/10/2026 con 30 000 contactos** (nombres y empresas con acentos y
comas, que es lo que obliga a escapar de verdad):

| Qué | Medido | Extrapolado a 500 000 |
|---|---|---|
| Exportar sin cifrar | **99 ms** (8.3 MB) | ~1.7 s |
| Exportar cifrado (el camino por defecto) | **244 ms** | ~4 s |
| Revisar el respaldo (el preflight) | **0.1 ms** | 0.1 ms — no crece |
| Restaurar en otra base | **745 ms** | ~12 s |
| Memoria residente | **15 → 16 → 19 MB** | acotada: el cursor cumple |

**Las cuatro están holgadas, y eso también es un resultado**: la fase que más
datos mueve no tiene un problema de rendimiento. Lo que tenía era F-5, que ningún
presupuesto habría encontrado — pero que apareció al ir a medirlo.

Que el preflight **no crezca** con las filas es la cifra que hay que vigilar: lee
la cabecera y para. El día que alguien le meta el cuerpo, la pantalla de
restaurar se quedará pensando delante de quien eligió el archivo, y nada
fallaría. Hay una aserción para eso.

### Dónde vive la medición, y por qué no en el banco

El banco de pruebas (8.5) es donde viven las otras medidas, y **aquí no servía**:
el camino que el usuario pulsa lo orquestan `ElArchivo` y `LeeElArchivo`, que
viven en `arles-app` —la frontera— y el banco no puede depender de ella sin
arrastrar Tauri dentro de un binario de medición. La alternativa era repetir la
orquestación en el banco con las piezas públicas del núcleo, y entonces se
mediría **otra cosa parecida**: exactamente el defecto que esta fase lleva ocho
entregas cazando.

Así que la medida vive donde vive el código que mide, en un módulo que sólo se
compila en pruebas y marcado `#[ignore]`:

```bash
cargo test --release -p arles-app presupuesto -- --ignored --nocapture
```

Y lo que **sí** corre siempre es la prueba de que el sembrado mete las filas que
dice: sin ella, la medición podría estar cronometrando una base casi vacía y
devolver una cifra excelente y falsa. Es la prueba de contactos con un solo
contacto, otra vez.

---

## 6 · Seis defectos de mis propias comprobaciones, en una tarde

Las cuatro pasadas del §140 encontraron que **once de diecisiete hallazgos no
eran defectos del producto sino de sus comprobaciones**. Esta pasada lo repite
con las comprobaciones que escribí hoy, y los seis merecen quedar escritos
porque ninguno es nuevo.

1. **Me inventé el nombre de un error.** Escribí `RespaldoDeOtroEsquema`; el del
   código es `RespaldoNoEncaja { motivo }`. **Tercera vez en esta auditoría** (la
   B con los comandos ARCO, la 8.3 bis a con `HistorialCambiado`).

2. **Comparé el nombre de la variante de Rust, no la clave de i18n.** La primera
   versión de dos pruebas buscaba `EsoEsUnArchivoDeHistorial` en el `Debug` del
   error y **salió en rojo sobre código correcto**: `ErrorIpc` no lleva nombres
   de Rust al otro lado de la frontera, lleva la clave y el detalle traducido —
   que es precisamente lo que el §139 pide. La clave es además lo que el usuario
   acaba leyendo, así que es la garantía que importa.

3. **La prueba de regresión de F-5 pasaba por suerte.** Puse 1 200 filas
   (~100 KB): una o dos fronteras de trozo, y que una caiga en una letra
   acentuada es improbable. La prueba salió en verde **con el lector roto a
   propósito** — la definición exacta de una comprobación que no vale. Ahora
   lleva 14 000 filas, campos densos en acentos, y **comprueba que de verdad haya
   quince fronteras** en vez de darlas por supuestas: si alguien baja la cifra, la
   prueba lo dice en voz alta.

   > **Una prueba probabilística tiene que declarar su margen.** Si no lo
   > declara, el día que alguien la haga más rápida deja de probar algo y nadie
   > se enterará.

4. **La prueba del byte imposible aceptaba cualquier rechazo**, así que tampoco
   cazaba su ruptura: sin la rama, el archivo se rechaza igual —por la suma— y la
   aserción se conformaba. Afilada a exigir **el motivo correcto** y, sobre todo,
   a medir **la cola**, que es lo que de verdad distingue tener la rama de no
   tenerla.

5. **Mi comprobación de que había restaurado una ruptura contó el comentario que
   la explica.** Verifiqué «el `if` está de vuelta» con un `grep error_len` que
   casaba el comentario de dos líneas encima, así que trabajé un buen rato sobre
   un archivo roto creyéndolo bueno — y la ruptura siguiente salió en verde por
   eso, no por el código. **Octava vez en el proyecto** (5.3, 8.5, dos en la
   8.3 bis, la 8.6 A, la 8.6 D, la 8.6 E).

   La conclusión de la 8.6 E —«se quitan los comentarios siempre, por defecto, en
   cualquier comprobación que busque una cadena»— era correcta y **no la apliqué
   a mi propio bucle de rupturas**, que es donde más duele: una restauración que
   no restaura deja todo lo que viene después sin significado. Ahora se verifica
   contra el bloque de código entero, no contra un identificador.

6. **Una consulta de prueba buscaba un dato que yo mismo había cambiado**
   —el apellido viejo, tras hacer los campos más densos en acentos—, y salió en
   rojo sobre código correcto. Barato, pero es el mismo error que el 1: dar por
   sabido en vez de mirar.

---

## 7 · Lo que queda declarado, no arreglado

**Un byte alterado a mitad de un respaldo se descubre al llegar a la suma**, es
decir al final. En la restauración normal eso no cuesta nada: la suma se comprueba
**dentro** de la transacción y la base no se toca. En la **recuperación** sí
cuesta algo: la clave nueva del llavero ya está escrita cuando se descubre.

No se arregla porque el precio es desproporcionado —comprobar la suma antes
exigiría leer el archivo entero dos veces, medio millón de filas— y el daño es
bajo: la base vieja está apartada, no borrada, y quien está en esa pantalla ya
había perdido su clave. Se puede reintentar con otro respaldo usando la misma
clave nueva.

**Lo que sí está cerrado** es el caso frecuente: contraseña equivocada, archivo
de historial y esquema distinto se rechazan **antes** de tocar nada, y son los
tres que le pasan a alguien de verdad.

---

## 8 · Verificación de esta pasada

```
cargo fmt --all --check                          limpio
cargo clippy --workspace --all-targets -D warnings   cero advertencias
cargo test --workspace                           todo en verde (1 000+)
cd app && npm test                               16 archivos · 191 pruebas
         npm run lint                            cero advertencias
         npx vue-tsc --noEmit                    limpio
validar.py --fase {0,1,2,3,4,5,6,W,7,8}          las diez en verde
```

**Pruebas nuevas:** cuatro en `recuperacion` (respaldo cifrado, contraseña mal
escrita, archivo de historial, otro esquema), dos en `restauracion` (el respaldo
de varios trozos con acentos, el byte imposible), dos en
`presupuesto_del_respaldo` (el sembrado, y la medición `#[ignore]`).

**Rupturas verificadas**, cada una vista en rojo con su mensaje:

| Garantía | Ruptura | Resultado |
|---|---|---|
| La contraseña se valida antes de tocar nada | quitar `origen.siguiente()` del pre-chequeo | en rojo |
| El alcance se comprueba antes de abrir | quitar la puerta del alcance | en rojo |
| El esquema se comprueba antes de abrir | quitar la comparación | en rojo |
| Las fronteras de trozo no parten caracteres | no arrastrar la cola | en rojo |
| Un byte imposible para la lectura y no engorda la cola | quitar la rama de `error_len` | en rojo, 3 → 280 bytes |

---

## 9 · Qué cambia para quien venga después

**La Fase 8 estaba en verde y su función principal no funcionaba.** No por un
error de lógica: por la unión entre dos piezas correctas, medida con datos de
juguete. Es el patrón del §9 en su versión más cara, porque aquí lo que no
funcionaba era la copia de seguridad.

Tres cosas que esta pasada deja como regla:

1. **Una prueba de ida y vuelta con dos filas no prueba una ida y vuelta.** Si lo
   que se recorre tiene trozos, páginas, lotes o cursores, la prueba tiene que
   cruzar **varias** fronteras, y decir cuántas.
2. **Nada irreversible antes de todo lo comprobable.** Vale para el llavero, para
   el borrado del historial y para cualquier cosa que venga.
3. **Medir es una forma de probar.** F-5 no lo encontró leer el código ni correr
   las pruebas: lo encontró poner treinta mil contactos y cronometrar.
