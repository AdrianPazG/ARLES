# Ataques a la importación · entrega 3.3

**Fecha:** 18 de septiembre de 2026 · **Puerta de salida de la entrega 3.3**
(ROADMAP: *«los archivos maliciosos con los que se atacó, y su resultado»*)

---

## Para Dirección, en una página

La importación lee archivos que ARLES **no hizo**: los exportó otro sistema, los
editó alguien, o los mandó un tercero. Eso significa que un archivo puede estar
hecho a propósito para hacer daño.

Se construyeron **trece archivos hostiles** y se lanzaron contra el lector.
Encontraron **tres fallos reales**, ninguno de los cuales daba error ni se veía
en ninguna prueba anterior:

| | Qué pasaba | Qué se nota desde fuera |
|---|---|---|
| **1** | Un archivo de **47 MB** hacía que ARLES pidiera **1 586 MB de memoria** antes de rechazarlo | En un equipo de 4 GB, **la aplicación se cierra sola** — con el trabajo dentro |
| **2** | Un Excel con 200 000 filas vacías por delante **colgaba la importación** | La pantalla se queda pensando y no vuelve. Nunca da error |
| **3** | Un nombre de archivo podía **dibujarse al revés** en pantalla | El asistente decía `facturaexe.png` cuando el archivo era `factura.exe` |

Los tres están arreglados, y cada uno tiene su prueba, que se ejecuta cada vez
que alguien toca el código.

**Lo que no se arregló, a propósito:** una sola celda de 50 MB entra. Para hacer
daño con eso hay que tener ya un archivo de decenas de megabytes, que el tope de
tamaño limita. Queda escrito para que sea una decisión y no un olvido.

---

## El hallazgo que importa, y por qué llevaba meses ahí

El modelo de amenazas decía, desde la Fase 0:

> *Tope de ratio de descompresión ~100:1 … lectura en **streaming** con
> `calamine`.*

**Era falso.** `calamine`, la biblioteca que lee los Excel, construye la hoja
**entera en memoria** antes de devolver la primera fila. El tope de celdas de
ARLES se contaba recorriendo filas, así que se comprobaba cuando la memoria ya
estaba gastada.

Lo medido, con un archivo construido a propósito:

| | Antes | Después |
|---|---|---|
| Archivo en disco | 47 MB | 47 MB |
| Celdas que declara | 16 000 000 | 16 000 000 |
| **Pico de memoria al leerlo** | **+1 586 MB** | **+0 MB** |
| Error devuelto | `DemasiadosDatos` | `DemasiadosDatos` |

**Las dos columnas devuelven el mismo error.** Ésa es la razón exacta por la que
el fallo llevaba ahí sin verse: cualquier prueba que sólo mirara el error habría
pasado en las dos. Hizo falta medir la memoria del proceso.

**La defensa nueva:** antes de abrir nada, el archivo se descomprime
**contando y tirando los bytes**, sin guardarlos. Si pasa de 300 MB, no se abre.
El pico durante esa cuenta es un búfer de 64 KB. El número sale de lo que
necesita un archivo legítimo grande —500 000 contactos con ocho columnas rondan
los 150 MB de XML—, así que deja holgura del doble.

**La regla que deja este hallazgo:** *una mitigación que dependa de cómo se
comporta una biblioteca por dentro no vale escrita, vale medida.*

---

## Los trece ataques, uno por uno

Todos se generan en el código de la prueba, no se guardan en el repositorio: una
bomba de descompresión versionada es un archivo que alguien acaba abriendo por
error, y el generador dice más que el binario.

| # | El archivo | Qué busca | Resultado |
|---|---|---|---|
| 1 | **Bomba de descompresión.** XLSX de 47 MB que declara 16 millones de celdas | Agotar la memoria antes de que ningún tope reaccione | 🔴 **Fallo encontrado.** Arreglado: tope de bytes descomprimidos, medido antes de abrir |
| 2 | XLSX con `=cmd\|'/c calc'!A1`, `=1+1`, `@SUM(A1:A9)`, `=HYPERLINK(...)` | Que una fórmula se evalúe al leer | ✅ Se leen como texto. Ya se probaba en CSV; faltaba el camino del XLSX, que es otra biblioteca |
| 3 | CSV de **100 000 columnas** | Que la memoria crezca con el archivo y no con el tope | ✅ Se recorta a 64 columnas |
| 4 | CSV con **200 000 filas vacías** por delante | Colgar la lectura sin dar error | 🔴 **Fallo encontrado.** El coste crecía con el cuadrado. Arreglado |
| 5 | Nombres que parecen rutas: `..%2F..%2Fetc%2Fpasswd.csv`, `con.csv` | Que el nombre construya una ruta | ✅ Nunca sale con separadores. Y el archivo guardado se nombra con el identificador del lote, nunca con el nombre recibido |
| 6 | Nombre con **`U+202E`**: `factura‮gnp.exe.csv` | Que la pantalla enseñe un nombre distinto del real | 🔴 **Fallo encontrado.** Arreglado: se quitan los caracteres que no se ven y cambian cómo se lee el resto |
| 7 | Nombre de 240 caracteres | Que el registro guarde algo que no se puede leer entero | ✅ Se recorta a 120 |
| 8 | **Bomba de entidades XML** («billion laughs»): diez mil millones de caracteres si se expandieran | Agotar la memoria por otra vía | ✅ No se expanden |
| 9 | **Entidad externa** que pide `file:///etc/passwd` | Que el contenido de un archivo del sistema acabe **dentro de un contacto**, y de ahí a un correo | ✅ No se resuelve. Es la fuga, no la caída, lo que hace grave a este ataque |
| 10 | XLSX **disfrazado de `.csv`** | Que salga una tabla con basura y el usuario la importe sin sospechar | ✅ Ninguna columna sale reconocible |
| 11 | CSV **disfrazado de `.xlsx`** | Una tabla a medias en vez de un error | ✅ Da error |
| 12 | Una celda de **50 MB** | Pasar por debajo de los dos topes a la vez | ⚠️ **Entra.** Decisión consciente, anotada arriba y en el modelo de amenazas |
| 13 | Un archivo que **no es una tabla** (`/proc/self/environ`) | Que se lea algo que nunca debió leerse | ✅ Rechazado por formato |

---

## Cómo se reproduce

```bash
cargo test -p arles-import --test ataques
cargo test -p arles-import --test bomba --release
```

El de la bomba va en `--release` porque construye un archivo de mil megabytes de
XML: sin optimizar tarda de más. Mide el pico de memoria leyendo
`/proc/self/status`, así que en un sistema que no sea Linux **dice que se saltó
la medición** — una comprobación que se salta no es una comprobación que pasa.

Y va en su propio binario de pruebas por una razón concreta: el pico de memoria
es del proceso entero y no baja nunca, así que si compartiera proceso con otros
tests, cualquiera que reservara medio giga dejaría la medición en adorno.
