# Fase 8 · Respaldos y endurecimiento — lo construido y lo que costó

**Abierta** el 05/10/2026 con [`FASE-08-PLAN.md`](FASE-08-PLAN.md).
**Cerrada** el 07/10/2026 con la 8.7.

> **En una frase:** ARLES escribe un respaldo cifrado de su base entera, lo
> vuelve a leer, lo restaura en otro equipo, archiva el historial a un disco
> externo y sabe salir del día en que se pierde la llave del llavero — y las
> cuatro auditorías del §140 están hechas.

---

## 1 · Qué se construyó

| Entrega | Qué | Dónde vive |
|---|---|---|
| **8.0** | El **manifiesto**: las 29 tablas del esquema clasificadas en cuatro alcances, cada una con su motivo escrito | `arles-core/src/respaldo.rs` |
| **8.1** | El **formato `.arles`**: cabecera, una línea por fila, suma sha256 al final. Cero I/O | `arles-core/src/formato_de_respaldo.rs` |
| **8.2a** | El **sobre cifrado**: Argon2id + XChaCha20-Poly1305 en trozos de 64 KiB | `arles-core/src/cifrado_de_respaldo.rs` |
| **8.2b** | **Leer la base con un cursor**, memoria acotada, todo en una transacción de lectura | `arles-db/src/respaldo.rs` |
| **8.2c** | El **comando, el diálogo y la pantalla**. ARLES escribe un respaldo entero | `arles-app/src/respaldo.rs` · `/ajustes/respaldos` |
| **8.3a** | **Leerlo de vuelta**: el sobre que se abre a trozos, y el preflight que dice qué se pierde | `arles-db/src/restauracion.rs` |
| **8.3b** | **Reemplazar** (D-8C), con la unión de bajas en SQL | `Db::restaurar` |
| **8.3c** | Los **comandos y la pantalla**. ARLES restaura un respaldo entero | `arles-app/src/restauracion.rs` · `/ajustes/restaurar` |
| **8.3 bis a** | **Archivar el historial** (D-8D): el tercer alcance, el preflight y el borrado verificado | `arles-db/src/archivo.rs` |
| **8.3 bis b** | El **botón de archivar**: escribe, relee, verifica y sólo entonces borra | `arles-app/src/archivo.rs` · `/ajustes/archivar` |
| **8.4** | **El llavero perdido**: la salida que el producto llevaba desde la Fase 1 recomendando sin poder dar | `arles-app/src/recuperacion.rs` · `/recuperar` |
| **8.5** | El **banco de pruebas** y las trece cifras del presupuesto | `crates/arles-banco` |
| **8.6** | La **migración V11** y los planes de consulta | `migrations/V11__indices_de_rendimiento.sql` |
| **8.6 A–D** | Las **cuatro auditorías del §140** | `AUDITORIA-{A,B,C,D}-*.md` |
| **8.6 E** | El **desenlace conectado**: rechazar lo que no puede terminar, avisar de lo que va a tardar | `alta_de_campana.rs` · `AsistenteDeCampana.vue` |
| **8.6 F** | La **auditoría de la fase entera**, por sus junturas | [`AUDITORIA-F-FASE-08.md`](AUDITORIA-F-FASE-08.md) |
| **8.7** | El cierre: la revisión a ojo, este documento, el informe y el avance | — |

---

## 2 · Las cuatro decisiones de Dirección, construidas

Dirección las contestó el 05/10/2026, el mismo día del plan.

| | Decisión | Cómo quedó |
|---|---|---|
| **D-8A** | El respaldo va **cifrado**, con opción explícita de desactivarlo | La contraseña **no se guarda en ningún sitio**; sin ella el respaldo no se abre. Sin contraseña, la pantalla lo dice con un aviso delante |
| **D-8B** | **Dos alcances**, «Esencial» por defecto | El esencial recorta el historial a 12 meses; el completo se lo lleva todo. La columna por la que se recorta vive dentro de la variante, porque `whatsapp_reply` no tiene `created_at` |
| **D-8C** | Restaurar **reemplaza, nunca mezcla**; las bajas se **unen** | Y el preflight **cuenta lo que se pierde** antes de que nadie confirme, porque reemplazar arrastra cascadas que D-8C no dice |
| **D-8D** | **Archivar** el historial y sólo entonces borrarlo, con cuatro condiciones | Nunca automático · el archivo **verificado leyéndolo de vuelta** antes del borrado · las bajas y el consentimiento fuera · todo en `audit_log` |

---

## 3 · Lo que la fase costó encontrar

Seis entregas de esta fase arreglaron algo que **ninguna prueba podía ver**,
porque vivía entre dos módulos que cada uno pasaba lo suyo. Vale la pena
tenerlos juntos, porque son el mismo defecto con seis caras:

1. **El orden del manifiesto no se podía restaurar** — tres hijos antes que sus
   padres. El orden del cuerpo **se calcula** del esquema.
2. **`import_batch` estaba fuera del esencial**, y `contact` tiene una clave
   ajena a ella: el esencial traía los contactos y no entraba.
3. **El cuerpo no decía qué columna es cada valor**, así que sólo se podía
   restaurar por posición: una migración que mueva una columna metería el
   apellido donde va la empresa, sin que una fila fallara.
4. **La sal y el nonce no tenían dónde ir**, y sin ellos un respaldo cifrado no
   se abre nunca.
5. **`cifrar` recibía el cuerpo entero en memoria**, deshaciendo el cursor de la
   8.2b — las dos piezas correctas y el agujero en la junta.
6. **Y el peor, encontrado en la 8.6 F**: el cuerpo cifrado se convertía a texto
   **trozo a trozo**, así que un acento partido por una frontera de 64 KiB hacía
   que **un respaldo cifrado de tamaño real no se pudiera restaurar**.

> **El patrón, dicho de una vez.** Cinco de los seis los encontró **medir el
> esquema o los datos**, no leer el código: contar las claves ajenas de verdad,
> exportar medio millón de filas, cronometrar treinta mil contactos. El sexto lo
> encontró ir a medir otra cosa.
>
> Y los seis estaban en verde. Es la frase que esta fase deja escrita: **una
> prueba de ida y vuelta con dos filas no prueba una ida y vuelta.**

---

## 4 · Las cuatro auditorías del §140, y la quinta

| Pasada | Hallazgos | Arreglados | Declarados |
|---|---|---|---|
| A · Arquitectura | 5 | 5 | — |
| B · Seguridad | 2 + 3 puntos del documento | 5 | — |
| C · QA y rendimiento | 6 | 4 | 2 |
| D · UX y accesibilidad | 4 | 4 | — |
| **F · la fase entera** (extra) | 5 | 5 | — |

**Veintidós hallazgos. Veinte arreglados, dos declarados con su cifra.**

Y el dato que esta fase deja como conclusión, no como anécdota: **once de los
diecisiete de las cuatro pasadas no eran defectos del producto sino de sus
comprobaciones.** Una lista escrita a mano, un alcance demasiado corto, un
comentario que se cazaba a sí mismo. El producto aguantó mejor que sus
instrumentos.

Los dos declarados son de la pasada C: **C-2**, el filtro del registro por
texto (3 709 ms de un presupuesto de 1 s), y la cifra de memoria en importación
(497 MB de 500, que cumple por el 0.6 %). **C-1**, la activación de campañas, lo
cerró la 8.6 E — y no como se planteó: no hay un tope por número de
destinatarios, hay un **rechazo de lo que no puede terminar** y un **aviso de lo
que va a tardar**.

---

## 5 · Las cifras, con su fecha

**Medidas el 06 y el 07/10/2026** sobre 500 000 contactos sintéticos (8.5) o
sobre 30 000 (8.6 F), con el banco y los comandos de verdad.

| Qué | Presupuesto | Medido | |
|---|---|---|---|
| Importar 500 000 | < 5 min | **91.4 s** | ✅ 30 % del techo |
| Abrir la tabla de contactos | < 500 ms | **357 ms** | ✅ arreglado en la 8.6 (era 1 424) |
| Buscar por correo | < 100 ms | **< 1 ms** | ✅ arreglado en la 8.6 (era 331) |
| Memoria en importación | < 500 MB | **497 MB** | ⚠️ cumple por 3 MB |
| Simular una campaña | < 200 ms | **6.0 ms** | ✅ |
| Memoria del motor, 20 000 envíos | < 250 MB | **13.9 MB, plana** | ✅ |
| **Activar una campaña de 451 k** | < 2 s | **73.5 s** | ❌ declarado · se rompe hacia los 30 000 |
| **Filtrar el registro por texto** | < 1 s | **3 709 ms** | ❌ declarado (C-2) |
| Exportar un respaldo cifrado | < 30 s | **244 ms / 30 k** | ✅ |
| Restaurar un respaldo | < 60 s | **745 ms / 30 k** | ✅ |
| Revisar un respaldo | < 1 s | **0.1 ms** | ✅ no crece con las filas |
| Memoria al exportar y restaurar | < 500 MB | **15 → 19 MB** | ✅ |

> Los cuatro presupuestos del respaldo son **propuestas de la auditoría F**, no
> requisitos de la Fase 0: la fase se construyó entera sin que nadie hubiera
> declarado uno. Están puestos donde está la frase que hay que poder decir —
> *«pulsé respaldar y no me dio tiempo a preguntarme si se había colgado»*—, y
> las cuatro medidas caben con dos órdenes de magnitud de margen.

**Cuatro cifras siguen sin medir, y las cuatro tienen un motivo que se
sostiene**: arranque en frío y desplazamiento en tabla necesitan una webview
(**R-07**), el preflight completo necesita los puertos 587/465/25 (**R-24**) y
el tamaño del instalador necesita el empaquetador y Mont (**P-01**).

---

## 6 · La revisión a ojo (8.7)

Las **dieciocho pantallas** abiertas en Chromium, una tras otra. Las sondas
estaban en verde, y aun así salieron **dos cosas**:

1. **Restaurar y Archivar no empezaban como Respaldos**, siendo hermanas en la
   misma subsección de Ajustes: su título arrancaba 19 px más abajo y su
   entradilla quedaba a **54 px** donde Respaldos la tiene a 8.

   La causa no era un número mal puesto: `.pantalla` es un flex con
   `gap: space-6`, y esas dos dejaban el `h1` y el `<p>` **sueltos**, así que
   eran dos hijos del flex y el hueco de 48 px se metía entre ellos. Las otras
   siete los envuelven en un `<header>`. Es el defecto de la Fase 3 —«Ajustes
   empezaba 300 px más a la derecha»— girado 90°: **cada pantalla, sola, se ve
   bien**.

2. **La pantalla de recuperación decía «ARLES RELAY», sin el numeral.** Las
   otras diecisiete dicen «ARLES RELAY I», porque la barra lateral pasa
   `numeral` y esta pantalla no lo pasaba. Y es justo la pantalla donde más
   falta hace que la marca esté entera: su propio comentario dice que sin ella
   *«una ventana suelta con un aviso rojo no dice ni de qué programa es»*.

**Las dos las vigila ahora `sonda:recorrido`**, que ya comparaba el borde
izquierdo de todas las pantallas y ahora compara también **el hueco entre el
título y su entradilla**. Verificada quitando el `<header>` de Restaurar: la
sonda dice «entre el título y su entradilla hay 40 px y en las demás 8».

> **Y la comprobación nueva nació con un defecto propio, que es la lección de
> las pasadas B, C y D otra vez.** Su primera versión comparaba el hueco de las
> dieciocho y sacó **tres hallazgos que no lo eran**: en Remitentes midió −24 px
> —el `<p>` que encontraba iba **al lado** del título, no debajo— y en Plantillas
> y WhatsApp midió 19, porque ahí el título comparte fila con sus botones y eso
> mueve su línea base.
>
> **Comparar un número entre maquetaciones distintas es comparar dos cosas.** El
> arreglo no fue subir el umbral hasta que pasaran —eso es ajustar el
> instrumento hasta que calle—: fue preguntar por **la estructura**, como ya
> hacía con el armazón. Se mide sólo cuando el título va solo en su banda y lo
> que sigue es un párrafo debajo.

---

## 7 · Lo que la Fase 8 NO hace, y conviene no prometer

- **No restaura un respaldo de un esquema anterior.** Hoy se rechaza con un
  error claro que manda a actualizar; hacerlo exigiría correr las migraciones
  sobre el propio respaldo, que es otra entrega. La V11 de la 8.6 es la primera
  migración que lo dispara: **un `.arles` tomado con el esquema V10 ya no
  entra.** Hoy no hay ninguno fuera de las pruebas, así que el coste real es
  cero; dentro de un mes de uso, no.
- **No hay respaldo automático ni programado.** Lo pide el usuario y elige dónde
  se guarda. Es deliberado: un respaldo que se hace solo acaba en la carpeta por
  defecto del equipo que se va a perder.
- **Un archivo de historial no se restaura**, nunca, y hay dos puertas para que
  no pueda ocurrir por accidente.
- **La contraseña del respaldo no se recupera.** Si se olvida, el archivo no se
  abre: el cifrado no tiene puerta de atrás y la pantalla lo dice con esas
  palabras.
- **Nada de esto se ha visto en macOS** (**R-07**). La prueba del arranque sin
  llavero **declara su rama y se omite** en este contenedor, que no tiene
  llavero ni D-Bus.

---

## 8 · Cómo se verificó

```
cargo fmt --all --check                               limpio
cargo clippy --workspace --all-targets -- -D warnings  cero advertencias
cargo test --workspace                                 todo en verde (1 000+)
cd app && npm test                                     16 archivos · 191 pruebas
         npm run lint                                  cero advertencias
         npx vue-tsc --noEmit                          limpio
         npm run sonda:recorrido                       sin hallazgos
validar.py --fase {0,1,2,3,4,5,6,W,7,8}                las diez en verde
```

Y lo que ninguna orden hace: **abrir la demostración y mirar**. Dos hallazgos,
los dos arreglados.

### La puerta de salida del plan

El plan declaró que la fase no estaría cerrada mientras alguna de estas cinco
cosas fuera cierta. Las cinco, una por una:

| Condición | |
|---|---|
| Una tabla pueda quedarse fuera del respaldo sin que nada falle | ✅ lo caza `ninguna_tabla_se_queda_fuera_del_respaldo`, que lee el esquema |
| Restaurar pueda destruir datos sin avisar antes, con cifras | ✅ el preflight cuenta la cascada **calculada del esquema**, y se comprueba borrando de verdad dentro de una transacción que se deshace |
| Los presupuestos sigan siendo estimaciones | ✅ nueve medidos, cuatro declarados con su motivo |
| Alguna de las cuatro auditorías no se haya corrido aislada | ✅ las cuatro, más una quinta |
| Un hallazgo esté sin arreglar **y sin declarar** | ✅ veintidós hallazgos: veinte arreglados, dos declarados con su cifra |

---

## 9 · Lo que sigue

**La Fase 9**, con tres cosas que esta fase deja encima de la mesa:

1. **Sacar la activación de campañas al hilo del motor**, con avance y
   cancelación. Hoy son 73.5 s con la interfaz congelada a partir de unos 30 000
   destinatarios, que es una lista ordinaria.
2. **C-2 · el filtro del registro por texto**: 3 709 ms contra 1 s de
   presupuesto.
3. **La 7.5 · SPF, DKIM y DMARC**, que Dirección dejó para el final el
   30/09/2026 porque es lo único que necesita red y sólo se puede verificar de
   verdad el día que se cierre **R-24**.

Y lo que no depende de nosotros: **R-07** (nadie ha abierto ARLES en WKWebView),
**R-24**, **P-01** (la licencia de Mont) y **P-09** (la revisión jurídica).
