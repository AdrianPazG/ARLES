# ADR-0015 · Un escritor con cerrojo, y lectores en conexión aparte

**Estado:** aceptado · **Fecha:** 2026-10-28 · **Decide:** Dirección
**Aceptado por Dirección el 2026-10-28**, el mismo día que se propuso
**Acota** las consecuencias negativas 1 de [ADR-0002](0002-rusqlite-sqlcipher-refinery.md) · su **decisión tecnológica sigue intacta**
**Relacionado:** auditoría C del §140 · presupuesto de rendimiento §0 · DOC-12

---

## Contexto

ADR-0002 decidió `rusqlite` + SQLCipher + `refinery` con repositorios propios, y
eso **está construido y es correcto**. Lo que no está construido es el mecanismo
de concurrencia que ese mismo ADR describe bajo sus consecuencias negativas:

> `rusqlite` es síncrono. No encaja directamente en Tokio. **Mitigación:** un
> **hilo dedicado de base de datos** que recibe peticiones por canal, en lugar
> de repartir `spawn_blocking` por todo el código.
>
> […] Serializar las escrituras por un único hilo **elimina de raíz** la clase
> de errores `SQLITE_BUSY` […]. **Los lectores usan conexiones separadas.**

Medido el 28/10/2026 sobre `crates/arles-db`:

| Qué dice ADR-0002 | Qué hay |
|---|---|
| Hilo dedicado de base de datos | **No existe.** Cero `thread::spawn` y cero `mpsc` en `arles-db` |
| Peticiones por canal | **No existe** |
| Lectores en conexiones separadas | **No existe.** Una sola `Connection::open` |
| Escrituras serializadas por un único punto | **Sí**, por `Mutex<Connection>` (`db.rs:24`) |

Es el tercer documento del proyecto que describe una biblioteca o un mecanismo
que nunca se adoptó —tras `tracing` en `THREAT_MODEL.md` §8 y Tokio en
`PRESUPUESTO_RENDIMIENTO.md` §3, los dos cazados por las auditorías B y C el 6 y
7 de octubre— y el primero que es un **ADR**, que es el documento al que se va a
preguntar *por qué* algo se decidió así. Queda anotado como **DOC-12**.

### Lo que el cerrojo único cuesta hoy, medido

Tres operaciones largas tienen la conexión tomada de principio a fin, y mientras
duran **no se puede leer nada ni escribe el motor**:

| Operación | Cuánto | Medido |
|---|---|---|
| Importar 500 000 contactos | toda la importación | **91.4 s** · 06/10/2026 |
| Activar una campaña de 451 196 | toda la transacción | **73.5 s** · 07/10/2026 |
| Exportar un respaldo | toda su transacción de lectura | 244 ms a 30 000, proporcional |

La tercera es la que más pesa para quien usa ARLES, y está dicha desde la 8.2b
sin que se leyera como un problema: *«el motor no escribe mientras se exporta:
el cerrojo es uno solo»*. Dicho de otra forma, **hacer una copia de seguridad
detiene los envíos**.

### Y dos hechos que cambian el análisis

**Uno · la base está en WAL.** `conexion.rs:137` aplica
`journal_mode = WAL`, y hay una prueba que lo comprueba
(`los_pragma_quedan_aplicados`). En WAL **un lector no espera a un escritor**.
Así que lo que bloquea a las pantallas hoy **no es SQLite: es nuestro `Mutex`**.

**Dos · no hay Tokio.** Lo midió la auditoría C el 07/10/2026. La premisa de la
mitigación del hilo dedicado —«no encaja directamente en Tokio»— **no existe en
este proyecto**.

---

## Decisión

### 1 · La decisión tecnológica de ADR-0002 no se toca

`rusqlite` 0.37 con `bundled-sqlcipher-vendored-openssl`, `refinery` 0.9, y capa
de repositorios propia sin ORM. **Sigue vigente palabra por palabra.**

### 2 · El escritor es `Mutex<Connection>`, y el hilo dedicado con canal se descarta

No se construye. Queda escrito aquí que **no se debe construir**, con su motivo,
para que nadie lo intente creyendo que arregla algo.

### 3 · Los lectores de pantalla van en una segunda conexión de sólo lectura

Con tres condiciones que no son de estilo:

1. **Se abre al arrancar**, con la misma clave maestra que la primera, que se
   suelta igual que hoy.
2. **`PRAGMA query_only = 1`**, de modo que una escritura por ahí no falle a
   veces: falle siempre.
3. **Sirve a las pantallas y a nadie más.** `arles-motor` y la activación de
   campañas **nunca** la usan.

El accesor nuevo se llama `leer(...)` y **los métodos se mueven de uno en uno**,
empezando por las lecturas de pantalla que pesan.

---

## Justificación

### Por qué no cambiar la tecnología

| Razón | El dato |
|---|---|
| **T-3 no es opcional** | En SQLx el soporte de SQLCipher depende de cómo esté compilado `libsqlite3-sys` en la máquina del cliente. En rusqlite es una característica que empaqueta OpenSSL |
| **§140 exige SQL auditable** | Hay **295 sentencias SQL** y **nueve planes de consulta fijados por pruebas** en `tests/planes_de_consulta.rs`, que fallan si una consulta vuelve a hacer `SCAN` o a ordenar en memoria |
| **Dos presupuestos se arreglaron leyendo planes** | Abrir la tabla **1 424 → 357 ms**; buscar por correo **331 → menos de 1 ms** (06/10/2026). Uno de los índices diseñados **no cambiaba el plan** y se descartó. Un ORM deja esos nueve planes sin poder verificarse |
| **Lo que se ganaría ya está sustituido** | La comprobación de consultas en compilación de SQLx es real, y la mitigación 3 de ADR-0002 —pruebas de integración contra base real— **existe y corre** |

### Por qué el hilo dedicado con canal se descarta en vez de construirse

**No es que falte: es que sería peor que lo que hay.** Tres razones:

1. **Su motivo escrito no existe.** La premisa era «no encaja en Tokio» y no hay
   Tokio. Sin él, el canal no compra nada.
2. **La garantía que prometía ya se cumple.** `Mutex<Connection>` serializa las
   escrituras por un único punto, que es exactamente lo que «elimina de raíz la
   clase de errores `SQLITE_BUSY`». Se cumple el objetivo; lo que no se cumple
   es el mecanismo descrito.
3. **Trabajaría contra la justificación principal de ADR-0002.** Esa
   justificación es «el SQL se lee». Con un canal, cada llamada pasa a ser un
   mensaje y una respuesta, y el camino entre la consulta y quien la pide deja
   de poder seguirse de un tirón. Y añade un modo de fallo que hoy no existe:
   **si ese hilo muere, todos los llamadores esperan para siempre**, sin error y
   sin traza — el mismo fallo silencioso que ya tiene el cerrojo no reentrante,
   que cuelga el programa sin dejar rastro.

### Por qué la segunda conexión sí, y medida

Las cuatro cifras salen de correr `query_only` **contra SQLCipher de verdad**, no
de la documentación de SQLite, el 28/10/2026:

| Qué se midió | Resultado |
|---|---|
| `query_only` se aplica en una conexión cifrada | `query_only = 1` |
| Leer por la lectora | `Ok(0)` |
| **Escribir por la lectora** | **`Err(ReadOnly, 8)`** · «attempt to write a readonly database» |
| Leer **con una transacción de escritura abierta en otra conexión** | `Ok(0)` — **no espera** |
| Leer tras el `commit` de esa transacción | `Ok(1)` |

Las dos últimas filas son las que importan: **la lectora no se bloquea** y ve la
foto anterior mientras la transacción está abierta. Es WAL funcionando, y es a la
vez la garantía y su límite.

### Por qué no se marca `con` como «el accesor de lectura»

Porque no lo es, y el número lo dice. Medido el 28/10/2026 sobre
`crates/arles-db/src`, descontando los módulos de pruebas:

| | Bloques | De ellos, escriben |
|---|---|---|
| `self.con(...)` | 49 | **20** |
| `self.con_dominio(...)` | 56 | 28 |
| **Total** | **105** | **48** |

`con` y `con_dominio` **sólo se diferencian en el tipo de error**, no en lo que
hacen. Marcar `con` como lectura mandaría **veinte escrituras** a una conexión de
sólo lectura: es la lista escrita a mano otra vez, el patrón que este proyecto
lleva once hallazgos aprendiendo a no repetir.

Por eso el accesor es **nuevo** y vacío al nacer: lo que se mueve a `leer` es lo
que alguien mira y mueve a propósito, no lo que el nombre de un accesor dio por
supuesto. Y no hay que tocar los 105: **mover las lecturas de pantalla que pesan
—lista de contactos, actividad, registro— se lleva casi todo el beneficio**.

---

## Seguridad

**La clave maestra: sin cambio.** `ClaveMaestra` es `Secret<Vec<u8>>`, se borra al
soltarse y no se imprime. Las dos conexiones se abren **al arrancar**, con la
misma clave, que se suelta igual que hoy: cero aumento de cuánto vive en memoria
y **cero accesos nuevos al llavero**. Abrirla más tarde obligaría a retener la
clave o a volver al llavero, y por eso no se hace así — es una condición de esta
decisión, no un detalle de implementación.

**Superficie de ataque: ninguna nueva.** SQLCipher cifra también el `-wal`. Mismo
archivo, misma clave, mismo proceso. Y la webview no alcanza `rusqlite`: la
conexión es `pub(crate)` y **no sale de `arles-db`**, que es lo que ADR-0002
mantiene y esto no cambia.

**El riesgo real es de corrección, y tiene nombre.** `Db::esta_suprimida` es la
consulta que decide si un mensaje sale, y hoy usa `con`. Su propio comentario
dice por qué se pregunta justo antes de cada envío: *«alguien que pide la baja el
martes seguiría recibiendo correos de esa campaña hasta enero»*. Es además una de
las tres reglas de la Fase 5 que no se negocian: **nada antes del `RCPT TO` puede
dar de baja a nadie**, y esta comprobación es su puerta.

Mandarla a la conexión de lectura costaría dos cosas, **para siempre**:

1. Se pierde la posibilidad de poner la comprobación y la escritura del intento
   en **una misma transacción**. Hoy no lo están, pero se puede; con dos
   conexiones, no se podrá nunca.
2. Se pierde **leer lo que acabas de escribir** entre las dos conexiones.

Por eso la condición 3 —pantallas y nadie más— es parte de esta decisión y no una
recomendación. **Se vigila con una comprobación**: ningún método que `arles-motor`
llame puede usar `leer`, y se verifica rompiéndola.

---

## Consecuencias

**Positivas.**

- Las tres operaciones largas dejan de congelar la aplicación: importar, activar
  y **exportar un respaldo sin detener los envíos**.
- Se cierra el hueco entre ADR-0002 y el código, en la mitad que valía la pena.
- La prohibición de escribir por la lectora es del motor de base, no una
  convención: `ReadOnly` siempre, medido.

**Negativas, y las tres dichas antes de construir.**

1. **Una pantalla enseña la foto anterior durante una operación larga.** Medido:
   la lectora devolvió `Ok(0)` con la transacción abierta y `Ok(1)` tras el
   `commit`. Es consistente y es lo correcto, pero **está viva, no al día**, y hay
   que decirlo donde se note. No es un defecto: es lo que WAL ofrece.
2. **El archivo `-wal` crece mientras haya un lector abierto**, porque un lector
   largo retrasa el *checkpoint*. **Sin medir.** Se mide con los 500 000 del
   banco antes de mover el primer método, y la cifra se publica entonces — no
   ahora.
3. **105 bloques de acceso hay que triar**, aunque no haya que moverlos todos.
   Mientras el reparto esté a medias, conviven dos accesores y hay que saber por
   qué cada método está donde está.

---

## Alternativas descartadas

**Construir el hilo dedicado con canal, como ADR-0002 decía.** Descartada por las
tres razones de arriba: su premisa no existe, su garantía ya se cumple, y
trabajaría contra la justificación principal del propio ADR.

**Un *pool* de lectores en vez de una conexión.** ARLES tiene una ventana y un
usuario. Varias conexiones de lectura añadirían un gestor de *pool* para una
concurrencia que no ocurre. Una basta; si algún día no bastara, se vería medido.

**Partir la activación en varias transacciones** para soltar el cerrojo cada
segundo, dejando la campaña en `draft` hasta una última transacción diminuta.
Funciona y conserva lo que el usuario necesita —no sale ni un mensaje hasta que
la activación esté completa, porque el motor sólo mira `running`—, pero cambia la
garantía de «entero o nada» por «puede quedar basura si se cae a mitad», con
limpieza al reintentar, en la parte más delicada del producto. **Y la conexión de
lectura la hace innecesaria.**

**Cambiar a SQLx.** Su comprobación de consultas en compilación sigue siendo
genuinamente atractiva, y sigue descartada por lo mismo que en ADR-0002: el
soporte de SQLCipher es frágil y T-3 no es opcional. A lo que ya decía ADR-0002
se añade lo que hay hoy y no había entonces: **nueve planes de consulta fijados
por pruebas** y dos presupuestos que se cumplen gracias a índices escritos
después de leer el plan.

---

## Las tres incógnitas · medidas el 28/10/2026

Estaban declaradas aquí como «por medir antes de construir». Las tres están
medidas, y **la primera contestó algo distinto de lo que preguntaba**.

### 1 · El `-wal` no crece por el lector: deja de vaciarse

La pregunta estaba mal hecha, y medirla es lo que lo enseñó. Cinco casos con
`banco wal`, que corre **la importación de verdad** —el mismo camino que la
pantalla— con un vigilante que mira el `-wal` cada 100 ms:

| Caso | `-wal` máximo | Base | Escribir |
|---|---|---|---|
| Importar 500 000, **sin lector** | 716.3 MB | 711.6 MB | 81.2 s |
| Importar 500 000, lector **de una en una** (pantallas) | 716.3 MB | 711.6 MB | 70.7 s |
| Importar 500 000, lector **con transacción abierta** (respaldo) | **716.3 MB** | **0.5 MB** | 68.5 s |
| 20 000 transacciones pequeñas, **sin lector** | **4.0 MB** | 4.4 MB | 3.4 s |
| 20 000 transacciones pequeñas, lector **con transacción abierta** | **373.3 MB** | 0.5 MB | 2.1 s |

**Lo que dicen las tres primeras filas:** el `-wal` mide **lo mismo** con lector
y sin él. Una importación es **una sola transacción**, así que cabe entera en el
`-wal` haya quien lea o no, y el *checkpoint* no puede correr hasta que
confirme. Lo que cambia es **la base**: 711.6 MB sin lector y **0.5 MB** con la
transacción abierta. Es decir, **un lector largo no engorda el `-wal`: impide
que se vuelque a la base.** El disco total es el mismo.

**Y lo que dicen las dos últimas, que es el hallazgo:** con **muchas
transacciones pequeñas** —la forma del motor, que confirma cada envío— un lector
fijado lleva el `-wal` de **4.0 MB a 373.3 MB**. Noventa y tres veces. Cada
transacción añade sus páginas y el *checkpoint* no recupera ninguna mientras el
lector viva.

> **Las dos filas de referencia son la mitad de la medición.** Sin la de «sin
> lector», los 373 MB no dirían nada: no se sabría cuánto es por el lector y
> cuánto crecería igual. La primera tanda se corrió **sin** la referencia de las
> transacciones pequeñas, y hubo que volver a añadirla.

**Qué significa para esta decisión, dicho sin adornos.** El caso malo es real y
ocurre justo en el escenario que esta decisión vende como su beneficio
principal: **un respaldo largo mientras el motor envía.** No la invalida —el
respaldo de 500 000 son 716 MB de `-wal` de todas formas— pero añade dos
condiciones que antes no estaban:

1. **`journal_size_limit` entra con la conexión de lectura**, no después. Hoy no
   está puesto, y se midió: el `-wal` se queda en 716.3 MB **incluso después de
   soltar al lector**, porque SQLite reutiliza el archivo en vez de recortarlo.
2. **`Db::exportar` es el último método que se mueve**, con su propia medida: es
   el más valioso y el único que fija una transacción larga.

### 2 · 37 candidatos de 111, contados y no estimados

Estaba estimado «en cinco o seis por su peso». Contado sobre `impl Db`,
descontando los módulos de prueba:

| | Cuántos |
|---|---|
| Métodos públicos de `impl Db` | **111** |
| Escriben, directa o transitivamente | **60** |
| De sólo lectura | **51** |
| …de ésos, **los llama el motor**, así que se quedan | **5** |
| …**candidatos a `leer`** | **37** |
| …sin llamador fuera de `arles-db` | 8 |
| Fuera a mano: `ejecutar_en_pruebas` | 1 |

Los cinco del motor, que la condición 3 protege: `esta_suprimida`,
`gastado_de`, `hay_que_parar`, `salud_de_la_cuenta` y
`ultimo_envio_de_otra_campana`.

`ejecutar_en_pruebas` queda fuera **a mano y con su motivo**: recibe un cierre
con SQL arbitrario, así que puede escribir aunque su cuerpo no lleve una sola
sentencia. Es el único caso que ninguna regla estática decide, y por eso se dice
en vez de dejarlo en la lista.

> **El script de medición necesitó cuatro versiones**, y vale la pena el
> recuento porque es el patrón de la auditoría G con sus 130 hallazgos falsos,
> cuatro veces seguidas:
>
> 1. El cuerpo se cortaba en la siguiente línea que empezara por `pub `, así que
>    `activar_campana` y `crear_lista` salían como de sólo lectura. **Las dos
>    escriben.**
> 2. Cogía `pub fn` de **todos** los `impl` de los archivos de `arles-db`:
>    `clave_i18n` es de `DbError` y `para_la_cabecera` de `Inventario`.
> 3. La transitividad no miraba los `fn` **privados** de `impl Db`.
>    `aceptar_el_riesgo` y `confirmar_coexistencia` llaman a `self.marcar(…)`,
>    que escribe.
> 4. Y `ejecutar_en_pruebas`, que ninguna versión podía decidir.
>
> Lo cazó las cuatro veces **mirar la lista antes de anotarla**, no correrla
> otra vez. Un hallazgo falso cuesta más que uno ausente.

### 3 · La comprobación existe y está vista en rojo

`el_motor_no_lee_por_la_conexion_de_pantallas`, en `--fase 1`. Lee qué métodos
de `impl Db` usan `self.leer(` y cuáles de ésos llama `arles-motor`: la
intersección tiene que estar vacía.

**Hoy pasa sin mirar nada**, porque `leer` no existe todavía, y eso queda dicho
en su docstring en vez de darla por una red. Lo que sí está verificado es que
caza el caso, por los dos caminos:

- Añadiendo un `leer` a `Db` y pasando **`esta_suprimida`** por él →
  **en rojo**, nombrando el método y su archivo.
- Pasando por `leer` **`plantillas`**, que el motor no llama → **en verde**, que
  es lo que prueba que no acusa a cualquiera.

> **Y un tropiezo propio que vale como aviso.** El bucle de las dos rupturas se
> cortó a mitad al pasarse a segundo plano, así que el restaurado **no corrió**:
> dos archivos se quedaron con la ruptura puesta mientras yo seguía trabajando.
> Es la trampa que `CLAUDE.md` documenta dos veces —la del `git checkout` que se
> llevó un módulo entero— con otra cara: **no basta con tener la copia, hay que
> comprobar que la restauración ocurrió.** Se comprobó con `diff` sobre los tres
> archivos antes de seguir.

---

## Qué está construido · 29/10/2026

El **primer tramo**, con las dos condiciones que las mediciones añadieron:

| Qué | Dónde |
|---|---|
| `journal_size_limit` a **64 MiB** | `conexion::abrir_crudo` |
| `conexion::abrir_para_leer`, con `query_only` y **sin migrar** | `conexion.rs` |
| La segunda conexión en `Db`, `Option` para que no poder abrirla no impida arrancar | `db.rs` |
| `Db::leer` y `Db::leer_dominio` | `db.rs` |

Y **cuatro lecturas movidas**, las que la medición señala como las que pesan:

| Método | Pantalla | Por qué ésta |
|---|---|---|
| `listar_contactos` | Contactos | 357 ms tras la V11, y es la que se abre a diario |
| `sucesos` | Registro | 3 709 ms con filtro de texto (C-2) |
| `resumen_de_campana` | Actividad | Una por tarjeta |
| `listar_supresiones` | Bajas | Crece sola y no se recorta |

**Cuatro de 37, a propósito.** Mover los 37 de golpe sería el cambio ancho que
esta decisión evita: cada método se mueve mirando quién lo llama. Y
`Db::exportar` **sigue en el escritor**, que es lo que esta decisión dejó
escrito: es el más valioso y el único que fija una transacción larga.

### Lo que está probado, y lo que una prueba no puede decir

Cuatro pruebas en `db.rs`, y tres rupturas vistas en rojo:

- **Una escritura por la lectora da `ReadOnly`** — contra SQLCipher de verdad,
  no contra la documentación.
- **Lee lo que hay**, que es la otra mitad: una conexión que no deja escribir y
  tampoco lee no serviría.
- **No espera al escritor y ve la foto anterior**: `Ok(0)` con una transacción
  de escritura abierta y sin confirmar, `Ok(1)` tras el `commit`.
- **El `-wal` tiene su tope** de 64 MiB.

Rupturas: quitar `query_only` → rojo; quitar `journal_size_limit` → rojo; dejar
la lectora en `None` → rojo, **dos pruebas**.

> **Y la tercera ruptura enseñó un defecto de mi prueba antes de enseñar uno del
> código.** Con `lectora: None` la prueba de la foto **se colgaba** en vez de
> fallar: `leer` cae al escritor y se bloquea contra el cerrojo que ella misma
> tiene tomado — la no reentrancia que `CLAUDE.md` documenta como trampa de la
> Fase 6. **Una prueba que se cuelga no es una prueba en rojo**: no dice qué
> pasó, y en CI parece una máquina lenta. Ahora afirma `lectora.is_some()`
> antes de tomar el cerrojo, y la ruptura sale en rojo con su mensaje.

### Lo que falta

- **Los otros 33 candidatos**, de uno en uno y mirando quién los llama.
- **`Db::exportar` al final**, con su propia medida del `-wal`: es el que
  convierte «hacer un respaldo detiene los envíos» en que no los detenga.
- **Medir si se nota.** Lo construido es correcto y está probado; que una
  pantalla responda durante una importación de 91 s **no se ha visto**, porque
  hace falta una ventana (R-07).

---

## Estado de la decisión

**Aceptada por Dirección el 2026-10-28**, el mismo día que se propuso, con las
tres mediciones de arriba delante y las tres incógnitas declaradas.

Lo que eso significa, y lo que no:

- **El hilo dedicado con canal queda descartado por escrito.** Si alguien lo
  propone de nuevo, la respuesta está aquí con su motivo, y no hay que volver a
  discutirla.
- **La conexión de lectura queda programada**, no construida. Va después de que
  la activación corra en su propio hilo con avance y cancelación, y **no empieza
  hasta que estén medidas las tres incógnitas** de la sección anterior. Mover un
  método antes de medir el `-wal` sería publicar una cifra que nadie tomó.
