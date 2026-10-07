# Auditoría A · Arquitectura (§140)

**Fecha:** 07/10/2026 · **Entrega:** 8.6 A · **Estado al empezar:** las diez
fases del validador en verde, 1 148 pruebas pasando.

Es la primera de las cuatro pasadas **separadas** que el §140 pide. Separadas y
no una revisión general, y el motivo se vio aquí: cuatro de los cinco hallazgos
están en sitios que una revisión general no mira, porque **todo estaba en
verde**.

---

## 0 · Qué se miró

Lo que el plan de la Fase 8 fija para esta pasada:

| | Qué | Resultado |
|---|---|---|
| 1 | Fronteras entre crates | **2 hallazgos** |
| 2 | `arles-core` sin I/O | **1 hallazgo** |
| 3 | La webview sin lógica de negocio | **1 hallazgo** |
| 4 | Sin `unwrap()` ni `any` indiscriminados (§138) | **1 hallazgo** |

**Cinco hallazgos, cinco arreglados.** Ninguno declarado sin arreglar.

---

## 1 · Lo que estaba bien, y conviene dejar escrito

No todo hallazgo es un defecto. Esto se midió y salió limpio:

- **`arles-core` no toca el mundo.** Ni disco, ni red, ni base. Comprobado
  recorriendo el crate entero, no sólo sus dependencias declaradas.
- **`arles-motor` no escribe SQL** fuera de sus pruebas. Las cinco apariciones
  de `INSERT` están dentro del bloque `#[cfg(test)]`.
- **`arles-db` no depende de `serde`**, que es la regla que lo mantiene ignorante
  de la IPC.
- **El grafo de dependencias es un DAG limpio** y coincide con el documentado,
  incluida la dirección que más se confunde: `arles-email` y `arles-whatsapp`
  dependen del motor, no al revés. El motor define el rasgo y no sabe quién lo
  implementa.
- **Cero `any`, cero `@ts-ignore`, cero `@ts-expect-error`** en el frontend
  fuera de las pruebas.

---

## 2 · Los cinco hallazgos

### A-1 · La comprobación del cero I/O no miraba el reloj, y pasaba en verde

**Qué.** `arles-core no hace I/O` buscaba cinco cadenas —`std::fs`, `std::net`,
`rusqlite`, `reqwest`, `tokio::net`— en los archivos **sueltos** de `src/`, con
un `os.listdir` que no entra en subcarpetas.

**Por qué importa.** El proyecto repite en tres sitios que el núcleo no tiene
reloj — *«la fecha la calcula `arles-app`, que es quien tiene reloj: ni el
núcleo ni este crate lo tienen»* — y la 8.2a declara que la sal y el nonce
entran como parámetros **porque el núcleo no pide entropía al sistema**. Las dos
reglas estaban escritas y ninguna estaba vigilada. Un reloj o un `getrandom`
dentro del núcleo hacen que la misma entrada dé salidas distintas, que es
exactamente lo que impide probar sin infraestructura.

**Cómo se verificó.** Se metió `OffsetDateTime::now_utc()` en
`arles-core/src/respaldo.rs` y se corrió la fase 1 entera: **45 pasan, 0 fallan**.
La comprobación dio el visto bueno a un núcleo con reloj.

**Arreglo.** Reescrita como `el_nucleo_no_toca_el_mundo`: recorre el crate con
`os.walk`, salta el bloque de pruebas, y busca quince patrones — disco, red,
proceso, entorno, reloj y azar. Verificada con dos rupturas (reloj y
`getrandom`): falla nombrando archivo y línea.

---

### A-2 · El `forbid(unsafe_code)` se vigilaba en 2 de 7 crates

**Qué.** La lista estaba escrita a mano: `("arles-core", "arles-db")`.

**Por qué importa.** `arles-import`, `arles-motor`, `arles-email` y
`arles-whatsapp` lo tienen puesto y **nada lo sostenía**. Y `arles-banco` no lo
tenía: un crate que lee medio millón de filas y genera datos, sin la prohibición
que los demás sí llevan.

**Arreglo.** La lista sale ahora de las carpetas que hay en `crates/`,
descontando `arles-app`, que es la única con una excepción auditada y tiene su
propia comprobación. Y se le puso el `forbid` a `arles-banco`.

**Y un defecto de la propia comprobación, encontrado al verificarla.** La
primera ruptura fue comentar el atributo —`// #![forbid(unsafe_code)]`— y la
comprobación **siguió en verde**: la cadena seguía ahí, dentro del comentario.
Es la **quinta vez** que este patrón aparece en el proyecto (5.3, 8.5 y dos
veces en la 8.3 bis). Ahora se quitan los comentarios antes de mirar, y está
verificada con las dos rupturas: comentado y borrado.

---

### A-3 · El grafo de dependencias se vigilaba con una sola arista

**Qué.** Sólo se comprobaba que `arles-core` no dependiera de `arles-db`.

**Por qué importa.** El resto del grafo es donde vive la arquitectura, y no lo
miraba nadie: que `arles-db` no dependa de `arles-app`, que el motor no dependa
de los adaptadores, que nadie dependa de la frontera. Cualquiera de esas
flechas al revés convierte el monolito modular en un monolito, y **compila**.

**Arreglo.** `el_grafo_de_crates_solo_apunta_hacia_dentro` declara qué puede
depender de qué y lo compara contra los `Cargo.toml` de verdad. Un crate nuevo
sin fila en la tabla también falla: un crate cuya posición en el grafo no ha
decidido nadie es un hallazgo barato de cazar el día que se añade, y caro
después. Verificada añadiendo `arles-app` a las dependencias de `arles-db`.

---

### A-4 · Dos cifras de negocio viven en Rust y en TypeScript, y nada las ataba

**Qué.** `AVISO_DIARIO = 50` está en `arles-core/src/ritmo.rs` y en
`app/src/app/stores/campanas.ts`. Los meses de historial están en
`MESES_EN_EL_ESENCIAL` (núcleo) y en `MESES_QUE_SE_OFRECEN[0]` (archivado).

**La duplicación es deliberada y está escrita**: la pantalla necesita enseñar el
aviso *antes* de llamar a nadie, así que no puede preguntárselo al núcleo. Lo
que faltaba no era quitar la copia — era que alguien comprobara que las dos
dicen lo mismo.

**Por qué importa, con lo que pasaría.** Si el núcleo sube el aviso a 100 y la
pantalla se queda en 50, el asistente exige aceptar un aviso que el núcleo ya no
pide. Al revés es peor: la pantalla no lo enseña, el usuario rellena nueve
pasos, y `crear_campana` lo rechaza al final con un error que no vio venir. Con
los meses, la pantalla ofrecería un periodo que la frontera sube en silencio: el
preflight enseña una cifra y se archiva otra.

Es del mismo tipo que los seis tokens de diseño inventados de la 5.2: nada
falla, nada cambia de color, y lo que se rompe es lo que el usuario ve.

**Arreglo.** `las_cifras_del_nucleo_son_las_mismas_en_la_pantalla` lee las
cuatro constantes y compara las dos parejas. Que una no se encuentre también
falla — si no, la comprobación pasaría en verde sin mirar nada. Verificada
rompiendo las dos parejas por separado.

---

### A-5 · El único `panic!` del código de producción

**Qué.** `arles-app/src/restauracion.rs:373` tenía
`self.sobre.take().unwrap_or_else(|| unreachable!())`.

**Por qué importa.** Era correcto —la rama acaba de ver `Some`— y aun así es la
única forma de que el proceso muera a mitad de una restauración. El §138 pide
que no haya panics indiscriminados; éste era discriminado, pero evitable.

**Arreglo.** Un `if let Some(sobre) = self.sobre.take()`: el caso imposible
simplemente no hace nada. El recuento vuelve a cero panics en producción en los
ocho crates.

---

## 3 · Lo que esta auditoría dice del método

Cuatro de los cinco hallazgos **no son defectos del producto: son defectos de
sus comprobaciones.** Las cuatro decían cubrir una regla y cubrían una parte, y
la parte que faltaba era justo la que nadie había roto.

Y el quinto defecto apareció **verificando el arreglo del segundo**: una
comprobación nueva que se cazaba en un comentario. Si la ruptura se hubiera dado
por hecha en vez de hacerla, habría entrado una comprobación rota junto con el
arreglo que venía a corregir otra igual.

> **La lección, por quinta vez:** una afirmación de verificación es una
> afirmación como cualquier otra, y en este proyecto son las que más veces
> resultan falsas. El §2 no es ceremonia; es lo único que separa una
> comprobación de un comentario optimista.

---

## 4 · Qué queda

Las otras tres pasadas del §140, cada una aislada:

- **B · Seguridad** — `THREAT_MODEL.md` §8, los secretos que no cruzan la
  frontera, capabilities, CSP, saneado, topes de importación, `cargo audit`,
  `npm audit`, y los derechos ARCO de la 3.4.
- **C · QA y rendimiento** — casos frontera, los presupuestos de la 8.5 medidos,
  y el motor en ejecuciones largas.
- **D · UX y accesibilidad** — contraste, foco, teclado, los cuatro estados de
  cada pantalla (§97) y los errores con sus tres partes (§95).

**Límite declarado:** la pasada D no puede mirar las pantallas en WKWebView
desde aquí. Eso es **R-07** y sigue esperando a alguien con un Mac. La vista
previa no cuenta: ahí el motor es el del navegador.
