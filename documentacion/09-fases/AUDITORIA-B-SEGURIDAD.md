# Auditoría B · Seguridad (§140)

**Fecha:** 07/10/2026 · **Entrega:** 8.6 B · **Estado al empezar:** las diez
fases del validador en verde.

Segunda de las cuatro pasadas separadas. Se recorrió **`THREAT_MODEL.md` §8
entero**, los quince puntos, uno a uno y midiendo.

---

## 0 · El resultado, en una tabla

| | Punto | Veredicto |
|---|---|---|
| 1 | Sin concatenación en las consultas | ✅ limpio |
| 2 | `Secret<T>` y la redacción | ⚠️ **el punto pedía algo que no existe** |
| 3 | Ningún comando devuelve una credencial | ❌ **se vigilaba un tercio** |
| 4 | Capabilities denegadas por defecto | ✅ limpio |
| 5 | CSP sin `unsafe-inline` ni `unsafe-eval` | ✅ ya verificado en la Fase 2 |
| 6 | Sin autocompletado en campos de credencial | ✅ ya verificado |
| 7 | Saneado al guardar **y al usar** | ⚠️ **la segunda mitad no existe todavía** |
| 8 | Topes de importación con archivos reales | ✅ ya verificado (13 ataques) |
| 9 | Fórmulas en la exportación CSV | ➖ **no aplica: no hay CSV** |
| 10 | No arranca sin llavero | ✅ con su límite declarado |
| 11 | Respuestas SMTP no confiables | ✅ limpio |
| 12 | Firma del updater | ➖ **no aplica: no hay updater** |
| 13 | `audit_log` sin `UPDATE` ni `DELETE` | ✅ limpio, y por esquema |
| 14 | `cargo audit` y `npm audit` en CI | ✅ limpio |
| 15 | Los cuatro derechos ARCO (3.4) | ✅ limpio |

**Dos hallazgos arreglados, tres puntos del documento corregidos.**

---

## 1 · Los hallazgos

### B-1 · La regla más cargada de la frontera se vigilaba en 17 de 52 estructuras

**Qué.** `nada_que_cruce_la_frontera_lleva_contrasena` leía **sólo
`comandos.rs`**. La frontera creció a once módulos —respaldo, restauración,
archivo, recuperación, whatsapp, campañas, cifras, control, registro, error— y
ninguno se miraba.

**Por qué importa.** Es la regla 3 de las cuatro de la frontera: *ningún comando
devuelve un secreto*. La que impide que la contraseña de una cuenta de correo o
el token de Meta acaben en la memoria de la webview, donde cualquier extensión o
cualquier `localStorage` mal puesto los alcanza.

**Cómo se verificó.** Se metió un `pub contrasena: String` en una estructura
serializable de `respaldo.rs`: **pasó en verde**.

**Arreglo.** Lee `arles-app/src` entero, salta los bloques de prueba, y busca
seis nombres en vez de tres —se añadieron `token`, `api_key` y `credencial`, que
son secretos igual y no los buscaba nadie—. `lleva_contrasena: bool` queda
fuera a propósito: dice si hace falta una, no cuál es, y el preflight de la 8.3
lo necesita. Verificado rompiéndolo en **cuatro módulos distintos**.

**Lo que esta comprobación sigue sin cubrir, y está dicho desde la Fase 5:**
busca el **nombre** del campo. Un campo llamado `clave_de_acceso` se le escapa.
A ése lo caza la prueba `lo_que_cruza_la_frontera_no_lleva_la_contrasena`, que
serializa una cuenta de verdad y busca el **valor**. Las dos hacen falta.

---

### B-2 · El saneado «al usar» no existe, y el día que haga falta no habrá nada que lo recuerde

**Qué.** `THREAT_MODEL.md` §8 pide sanear el HTML **al guardar y al usar**. Al
guardar está hecho y probado. Al usar **no existe**.

**Y no es un olvido.** `Mensaje` —lo que el motor le da al adaptador— lleva sólo
el destinatario y la clave de idempotencia. El único `Contenido` que llega al
SMTP es un texto fijo escrito en el código: el del envío de prueba. **El cuerpo
de una campaña no llega al remitente todavía.**

**Por qué es un hallazgo igualmente.** El día que alguien conecte el cuerpo de
la plantilla al envío, el camino nuevo pasará por ahí **sin que nada pida sanear
otra vez**. Y el saneado al guardar no cubre ese caso: entre guardar y enviar
está la sustitución de variables, que mete datos de contacto importados de un
XLSX ajeno **dentro del HTML ya saneado**.

Es el patrón del §9 aplicado a una regla de seguridad: las dos piezas correctas
y el agujero en la junta.

**Arreglo.** `nada_sale_por_correo_sin_pasar_por_el_saneado` vigila la condición
de hoy —que `Contenido` se arme de constantes— y **falla el día que deje de ser
cierta**, para que quien lo conecte tenga que decidir dónde se sanea. Verificada
en las dos direcciones: falla con un cuerpo que viene de una variable, y pasa si
ese cuerpo va envuelto en `sanear`.

---

## 2 · Los tres puntos del documento que no se podían marcar con verdad

Un punto que nunca se puede marcar es peor que un punto que falta: hace dudar de
la lista entera, y la lista es lo único que queda cuando nadie se acuerda.

### Pedía una biblioteca que el proyecto no usa

El punto 2 decía *«la capa de redacción de `tracing` está activa»*. **ARLES no
usa `tracing`**, ni lo ha usado nunca. La redacción que sí existe es
`redactar_rutas` en `ErrorIpc`, que quita las rutas del sistema de archivos de
los mensajes de las capas de abajo (F7 de la Fase 1). El punto está reescrito
contra lo que hay.

### Dos puntos describen caminos que todavía no existen

- **Fórmulas en la exportación CSV**: ARLES **no exporta CSV**. Los únicos
  archivos que escribe son `.txt` —el informe ARCO y el resumen de campaña, los
  dos en prosa— y `.arles`. El punto vuelve a contar el día que exista una
  salida que una hoja de cálculo abra directamente.
- **Firma del updater**: **no hay updater**. `tauri.conf.json` no configura
  ninguno, y la distribución está además bloqueada por **P-01**. El punto vuelve
  a contar el día que se añada, y entonces se prueba **con una actualización
  manipulada**, no con una buena.

Los dos quedan marcados `[–] no aplica` **con su fecha y su condición de
reactivación**, no borrados.

---

## 3 · Lo que salió limpio, con lo que se midió

- **Sin concatenación en las consultas.** Siete `format!` dentro de consultas,
  los siete interpolando **identificadores** —nombres de tabla y columna que
  salen del manifiesto o de `sqlite_master`—, que es lo único que SQLite no deja
  parametrizar. Todo dato de usuario va por `?N`, incluido el filtro de texto
  del registro, que usa `'%' || ?7 || '%'`.
- **Capabilities.** `core:default` y `core:window:allow-start-dragging`. Nada de
  `shell`, `fs` ni `http`.
- **`audit_log` imborrable**, y no por convención: dos *triggers* del esquema lo
  abortan, con una prueba que los ejercita.
- **Respuestas SMTP**: `recortar_respuesta` filtra los caracteres que engañan a
  quien mira y corta a 500, con la prueba de los cinco mil caracteres de basura.
- **Dependencias en CI**: `cargo-deny-action` con `advisories` —la misma base de
  RustSec que `cargo audit`— y `npm audit` en dos pasadas, una para lo que se
  distribuye y otra para las herramientas.
- **Los cuatro derechos ARCO** tienen comando registrado y pantalla: acceso,
  rectificación, cancelación (con su preflight) y oposición.

---

## 4 · Una nota sobre el método, y un error propio

Al comprobar los derechos ARCO busqué los comandos por nombres **que me
inventé** —`cancelar_contacto`, `rectificar`— y salieron a cero. Estuve a punto
de anotarlo como hallazgo. Los nombres reales son `cancelar_datos_de`,
`que_se_borraria` y `editar_contacto`, y los cuatro derechos están completos.

Es exactamente el defecto que la auditoría A encontró en una comprobación del
validador —buscar un nombre inventado al escribir la comprobación— repetido **a
mano, por mí, en la misma sesión**. Un hallazgo falso en una auditoría de
seguridad cuesta más que uno ausente: manda a arreglar algo que funciona y
gasta la credibilidad del resto de la lista.

> **Antes de anotar que algo falta, se busca cómo se llama de verdad.**

---

## 5 · Qué queda

- **C · QA y rendimiento** — casos frontera, los presupuestos de la 8.5
  medidos, y el motor en ejecuciones largas.
- **D · UX y accesibilidad** — contraste, foco, teclado, los cuatro estados de
  cada pantalla (§97) y los errores con sus tres partes (§95).

**Límite declarado:** la pasada D no puede mirar las pantallas en WKWebView
desde aquí. Eso es **R-07**.
