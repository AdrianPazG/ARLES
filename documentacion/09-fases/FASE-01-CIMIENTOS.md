# Fase 1 · Cimientos

**Estado:** ✅ cerrada y **revisada** · **Fecha:** 2026-09-11
**Validación:** 32/32 comprobaciones, 0 omitidas — `validar.py --fase 1`
**Revisión:** 13 hallazgos, todos corregidos — ver §6
**Para Dirección:** el mismo contenido sin tecnicismos en [FASE-01-PARA-DIRECCION.md](FASE-01-PARA-DIRECCION.md)

> **Objetivo.** Levantar el esqueleto sobre el que se construye todo lo demás, y —más importante— **hacer que las decisiones documentadas sean ejecutables**. Un principio que solo vive en un documento se erosiona; convertido en test o en regla de lint, se defiende solo.

---

## 1. Qué se construyó

### `crates/arles-core` — dominio sin I/O · 40 tests

Tipos que no tocan base de datos, red ni disco. Es la regla de frontera 3.2 de [ARQUITECTURA.md](../03-arquitectura/ARQUITECTURA.md), y lo que permite probar la lógica sin levantar nada.

| Módulo | Qué contiene |
|---|---|
| `ids.rs` | Identificadores tipados con UUID v7 y `IdempotencyKey` |
| `secret.rs` | `Secret<T>`: `Debug`/`Display` redactados, sin `Deref`, limpieza al soltarse |
| `email.rs` | `EmailAddress` con validación y normalización |
| `attempt.rs` | Máquina de estados de un intento de envío |
| `error.rs` | Errores tipados con clave de i18n |

**Por qué ids tipados.** Que `ContactId` y `CampaignId` sean tipos distintos hace que el compilador rechace pasar uno donde va el otro. En `message_attempt`, que referencia campaña, contacto y cuenta a la vez, eso no es ceremonia: es la diferencia entre un error de compilación y un correo al destinatario equivocado.

**Por qué UUID v7 y no v4.** Lleva marca de tiempo en los bits altos, así que ordena por creación y mantiene los índices compactos. Un v4 aleatorio dispersa las escrituras por todo el índice — con 500 000 contactos, eso se nota.

### `crates/arles-db` — datos cifrados · 29 tests

SQLite con SQLCipher, migraciones con `refinery`, y un asa tipada que **no deja salir `rusqlite` del crate**.

- `conexion.rs` — `ClaveMaestra` (32 bytes) y apertura con `PRAGMA key` primero
- `db.rs` — `Db`: métodos tipados, un solo escritor
- `migraciones.rs` — migraciones embebidas en el binario
- `migrations/V1__esquema_inicial.sql` — **19 tablas, 367 líneas**

**Por qué el asa tipada.** El shell pedía la conexión y usaba `rusqlite` directamente; eso filtraba el motor de base de datos a la capa de presentación. Se corrigió moviendo las consultas a `arles-db`: ahora el SQL vive en un solo sitio y se puede auditar de una pasada ([ADR-0002](../03-arquitectura/adr/0002-rusqlite-sqlcipher-refinery.md)).

**Por qué un mutex y no un pool.** SQLite con WAL admite lectores concurrentes pero un único escritor. Serializar las escrituras **elimina de raíz** la clase de errores `SQLITE_BUSY`, en vez de gestionarlos con reintentos que funcionan el 99 % de las veces.

### `crates/arles-app` — shell de escritorio · 19 tests

Tauri 2 con **capabilities denegadas por defecto**.

- `estado.rs` — arranque: rutas → llavero → base abierta y migrada
- `llavero.rs` — Keychain, Credential Manager o Secret Service
- `comandos.rs` — los comandos IPC
- `error.rs` — `ErrorIpc` tipado con clave de i18n
- `rutas.rs` — directorios estándar de cada plataforma
- `capabilities/principal.json` — sin `shell`, sin `fs`, sin `http`
- `icons/` — generados desde los tokens

### `app/` — interfaz · 14 tests

Vue 3 con TypeScript estricto, Pinia, vue-router e i18n, consumiendo el CSS generado desde `tokens.json`.

### `herramientas/` — utilidades versionadas

| Herramienta | Qué hace |
|---|---|
| `design-tokens/` | `tokens.json` → CSS, lámina de paleta, verificación WCAG |
| `iconos/` | Iconos de la app desde los mismos tokens, incluidos `.ico` e `.icns` |
| `validar/` | Valida una fase completa y emite informe |

### Infraestructura

`deny.toml` (licencias y avisos) · `.github/workflows/ci.yml` (6 jobs) · `.gitignore`

**Volumen:** ~2 970 líneas de Rust (1 073 en `arles-core`, 990 en `arles-db`, 909 en
`arles-app`), ~850 de TypeScript y Vue, ~1 190 de herramientas.

---

## 2. Decisiones tomadas durante la fase

### D-5 · Mont autorizada para desarrollo
La puerta de la licencia pasa de la Fase 2 a **antes de la demo**. Cerró P-02 de paso: el logotipo es tipográfico por diseño (§21), así que con Mont disponible se produce aquí. Detalle en [DECISIONES_DE_DIRECCION.md](../01-producto/DECISIONES_DE_DIRECCION.md).

### ADR-0002 corregido · `refinery` 0.9, no 0.8
La 0.8 fija `rusqlite` entre 0.23 y 0.26. Como ambos declaran `links = "sqlite3"`, **no pueden coexistir** con el `rusqlite` 0.37 que SQLCipher necesita, y Cargo lo rechaza en la resolución. Anotado en el ADR para quien actualice.

### El icono de la aplicación
El §21 define el logotipo como exclusivamente tipográfico, sin isotipo. El sistema operativo, en cambio, **exige** un icono cuadrado. Se resolvió usando la inicial del logotipo en Mont Black sobre el fondo profundo de la marca: es la marca, no un símbolo inventado. Se genera desde `tokens.json`, así que un cambio de paleta se propaga sin editar nada a mano.

### Dos umbrales para `npm audit`
Lo que se empaqueta llega a la máquina del cliente, así que no tolera avisos moderados. Las herramientas de desarrollo, que nunca se distribuyen, solo se detienen ante los graves.

---

## 3. Qué se verificó, y cómo

**32 comprobaciones automáticas, 0 omitidas.** Reproducible con `validar.py --fase 1`.

### Fronteras de arquitectura

| Comprobación | Método |
|---|---|
| `arles-core` no hace I/O | Búsqueda de `std::fs`, `std::net`, `rusqlite`, `reqwest` en su código |
| `arles-core` no depende de `arles-db` | Inspección del `Cargo.toml` |
| Las capabilities no exponen `shell`, `fs` ni `http` | Análisis de `principal.json` |
| La CSP no permite `unsafe-inline` ni `unsafe-eval` en scripts | Análisis de `tauri.conf.json` |
| Borrar un contacto no borra el registro de envío | Análisis de la migración |
| El frontend tiene prohibido `fetch` y `localStorage` | **Se ejecuta eslint** contra un archivo cebo que viola ambas reglas |
| El PDF para Dirección corresponde a su Markdown | Huella SHA-256 del `.md` de origen, escrita al generar |

Estas siete no las cubre ningún test: son las que evitan que la arquitectura se erosione sin que nadie lo note.

### Decisiones convertidas en tests

| Decisión | Test que la defiende |
|---|---|
| La normalización de correo no elimina puntos ([MODELO_DE_DATOS §3.1](../03-arquitectura/MODELO_DE_DATOS.md)) | `no_elimina_puntos_porque_eso_es_especifico_de_gmail` |
| `presumed_sent` es terminal ([ADR-0004](../03-arquitectura/adr/0004-idempotencia-y-envio-ambiguo.md)) | `un_intento_ambiguo_nunca_se_reenvia` |
| `Secret<T>` no filtra al formatear una estructura | `anidado_en_una_estructura_tampoco_filtra` |
| Duplicado imposible por esquema (§55) | `no_puede_haber_dos_intentos_para_el_mismo_contacto_y_campana` |
| Borrar un contacto no borra su supresión (§39) | `borrar_el_contacto_no_borra_su_supresion` |
| La bitácora es append-only (§91) | `la_bitacora_no_admite_update_ni_delete` |
| Ningún estado dice «entregado» (§65) | `ningun_texto_de_estado_afirma_una_entrega` |
| Todo error tiene sus tres partes (§95) | `todos_dicen_que_paso_como_arreglarlo_y_que_esta_a_salvo` |
| «I» nunca junto al número de versión ([ADR-0010](../03-arquitectura/adr/0010-numeral-romano-i.md)) | `la_info_separa_el_numeral_de_la_version` |

### Las dos verificaciones que más costaron

**El cifrado, comprobado de verdad.** T-3 no pide «usar SQLCipher»: pide que los datos personales no estén legibles en disco. El test inserta una cadena buscable, cierra la conexión, **lee el archivo crudo** y comprueba que la cadena no aparece y que la cabecera no es `SQLite format 3`. Después intenta abrir con otra clave y exige que falle.

**El arranque, en sus dos ramas.** [ADR-0011](../03-arquitectura/adr/0011-cifrado-en-reposo-y-clave-maestra.md) dice que sin llavero la aplicación **no arranca**, sin degradación a texto plano.

- *Sin llavero* — ocurre de forma natural en un contenedor sin D-Bus. Se comprueba que el arranque se rechaza **y que no queda ninguna base en disco**: si existiera, se habría creado sin cifrado real.
- *Con llavero* — hubo que levantar `dbus-run-session` y un `gnome-keyring` desbloqueado. La receta está en [`con-llavero.sh`](../../herramientas/validar/con-llavero.sh) y corre igual en local y en CI.

El test **declara qué rama tomó**. Sin eso, un «ok» no distingue entre haber probado el arranque y haber probado el rechazo — y durante esta fase la rama feliz estuvo sin ejercitar sin que se notara.

---

## 4. Qué NO se verificó

La parte más importante del documento.

| Sin verificar | Motivo | Cuándo |
|---|---|---|
| **La aplicación abriendo una ventana real** | El contenedor no tiene servidor gráfico. Compila y arranca su estado, pero nadie la ha visto abrirse | Fase 2, en una máquina con escritorio |
| **Renderizado en WebView2 y WKWebView** | Solo se ha visto en Chromium sobre Linux. Es el riesgo R-07 y el coste aceptado de ADR-0001 | Fase 2, por pantalla |
| **Instaladores** | Requieren firma y notarización, diferidos por D-4 | Fase 9 |
| **Escalado de Windows al 125–200 %** (§22) | Necesita Windows real | Fase 2 |
| **Rendimiento a 500 000 contactos** | El esquema y los índices están, los presupuestos no se han medido | Fase 4 |
| **El llavero nativo de macOS y Windows** | Aquí solo se probó Secret Service. CI los cubrirá en sus runners | Con el primer CI verde |

Y una limitación de método que conviene tener presente: el validador comprueba **que las fronteras siguen donde se pusieron**, no que la arquitectura sea la correcta. Eso lo decide una revisión humana, no un script.

---

## 5. Problemas encontrados durante la fase

### Un test encontró un caso que merecía pensarse
`"juan@empresa.com\r"` se recortaba antes de la comprobación anti-inyección. Revisado: **es correcto** — un CSV de Windows deja ese carácter y la dirección resultante es limpia. Se ajustó la expectativa del test, no el código, y quedaron dos tests que fijan ambos comportamientos: CRLF **alrededor** se recorta, CRLF **dentro** se rechaza.

### `refinery` 0.8 no era compatible
Conflicto de `links = "sqlite3"` con el `rusqlite` que SQLCipher necesita. La 0.9 sí sirve. Anotado en ADR-0002.

### Un diagnóstico equivocado, y cómo se detectó
Pareció que la barra lateral no llegaba al fondo de la ventana. Se «arregló» el CSS. Al medir la geometría real en el navegador resultó que **el viewport en headless es 87px más bajo que `--window-size`**: era artefacto de la captura, no un fallo de estilos. Se revirtió el cambio falso.

Ese mismo desfase reapareció luego recortando los iconos por abajo. Esa vez sí era real, y el generador ahora **lo mide en tiempo de ejecución** y recorta el PNG en consecuencia, en vez de dar por buena una constante.

La lección que queda escrita: **medir antes de arreglar**. Un cambio sobre un diagnóstico equivocado parece funcionar y deja el código peor.

### El validador tenía una comprobación que era teatro
Al terminar se probó el validador contra sí mismo: se inyectó `shell:allow-execute` en las capabilities y **lo detectó**; se desactivaron las reglas de eslint que prohíben `fetch` y `localStorage` y **no se enteró**.

El motivo: esa comprobación era una búsqueda de texto en el archivo de configuración, y pasaba mientras la palabra apareciera en cualquier sitio. Se sustituyó por una que **ejecuta eslint contra un archivo cebo** que viola ambas reglas y exige que las rechace, contando errores en vez de buscar palabras —el texto está en español y puede reescribirse; las dos violaciones tienen que seguir produciendo dos errores.

Queda como regla de método: **una comprobación que nunca se ha visto fallar no está verificada**. Romper algo a propósito y confirmar que salta es parte de escribirla.

### El shell filtraba `rusqlite`
`arles-app` usaba tipos del motor de base de datos directamente. El compilador lo detectó al faltar la dependencia, y la tentación era declararla. Se resolvió al revés: `arles-db` expone una API tipada y el shell no conoce el motor.

---

## 6. Revisión a fondo de la fase

Tras cerrar la fase se hizo una revisión completa del rango de commits. **Encontró 13 defectos que la validación no veía**, y esa es la conclusión más útil: un validador comprueba que las fronteras siguen donde se pusieron, no que la lógica dentro de ellas sea correcta.

Cada hallazgo se **reprodujo antes de arreglarlo**. Los dos críticos tenían en común que los tests existentes pasaban mientras la garantía estaba rota.

### Críticos

**F1 · Borrar un contacto borraba la prueba de que se le envió un correo.**
`message_attempt` y `campaign_audience` cascadeaban desde `contact`. Reproducido: insertar un envío, borrar el contacto, y el registro desaparece. Rompía dos garantías a la vez — la auditoría («¿a quién le llegó esto?» se quedaba sin respuesta) y la idempotencia, porque la fila única que impide el duplicado ya no existía: **reimportar al contacto permitía enviarle otra vez**.

La causa de fondo era la misma que el §39 ya había resuelto para la supresión, y que aquí no se aplicó: **la clave tiene que ser la dirección, no el `contact_id`**. Un contacto borrado y reimportado es un id nuevo; una dirección es la misma persona. Ahora `message_attempt` y `campaign_audience` llevan `contact_email`, la unicidad va por ahí, y el `contact_id` se anula al borrar —se conserva el hecho, se elimina la identidad—, que es justo lo que `MODELO_DE_DATOS.md` §3.4 describía para el derecho de cancelación.

**F2 · Perder la clave del llavero generaba otra en silencio.**
Si la entrada desaparecía —reinstalación, cambio de equipo, perfil corrupto— pero la base seguía en disco, `obtener_o_crear_clave_maestra` hacía lo que su nombre decía: creaba una nueva. El usuario veía «no se pudo descifrar la base de datos» y nada más. No se enteraba de que sus datos seguían ahí, ni de que lo único que los recupera es un respaldo.

Leer y crear ahora están separados, porque **una función «obtener-o-crear» no puede distinguir el primer arranque de una entrada perdida**. Quien llama sí sabe si hay base en disco, así que la decisión le corresponde. Hay un error propio, `ClaveMaestraPerdida`, con su acción propia.

### Altos

| # | Hallazgo | Corrección |
|---|---|---|
| F3 | `EmailAddress` comparaba por `raw`, así que `VENTAS@…` y `ventas@…` eran valores distintos y la deduplicación fallaba en cualquier `HashSet` | Igualdad y hash por `normalized` |
| F4 | El `Deserialize` derivado saltaba `parse()`: serde podía construir una dirección con CRLF —la carga de inyección que los tests daban por imposible— o con `normalized` contradiciendo a `raw` | Serialización como cadena; toda deserialización pasa por `parse` |
| F5 | `como_message_id` interpolaba un dominio sin validar en una cabecera SMTP | Toma una `EmailAddress` ya validada: el caso deja de ser representable |
| F6 | Con `windows_subsystem = "windows"`, el fallo de arranque iba a `eprintln!` y **en Windows la aplicación moría sin decir nada** — el fallo silencioso que ADR-0011 existe para evitar | Cuadro de diálogo nativo, y el mensaje pasa a ser una estructura de tres campos que el compilador obliga a rellenar |

### Medios

| # | Hallazgo | Corrección |
|---|---|---|
| F7 | `ErrorIpc.detalle` reenviaba mensajes de `rusqlite` y del llavero con rutas dentro, contradiciendo su propia documentación | Redacción de rutas antes de cruzar la frontera IPC |
| F8 | `verificar_clave` convertía **cualquier** error en «clave incorrecta», así que un disco lleno mandaba al usuario a buscar un problema de credenciales inexistente | Se distingue `NotADatabase` del resto |
| F9 | El validador moría con `AttributeError` si la CSP se escribía como cadena | Acepta ambas formas |
| F10 | `recortar_png` escribía un PNG corrupto si la captura salía más baja de lo pedido | Falla explícitamente |
| F11 | La versión del pie estaba escrita a mano y habría quedado obsoleta en la siguiente subida | La reporta el núcleo; la reserva es un guion visible, no un `1.2.0` que se confundiría con el valor real |
| F12 | `createWebHistory` en una aplicación empaquetada: recargar en `/campanas` daría una ventana en blanco, enmascarado por `vite dev` | Historial por hash |
| F13 | El hexadecimal de la clave maestra dejaba 32 `String` sin limpiar en el montón, dentro del código que se toma la molestia de limpiar todo lo demás | Escritura en un búfer ya reservado |
| F14 | `THREAT_MODEL.md` afirmaba una CSP «sin `unsafe-inline`» que no correspondía con la implementación: `style-src` sí lo lleva | Registrado como riesgo aceptado, con el motivo — el vector real es el script, y `script-src` sí es estricto |
| F15 | Las claves de i18n de Rust son planas y en los textos cada error es un objeto: `$t('error.db.sqlite')` devolvía **el objeto** y la interfaz habría mostrado `[object Object]` | Resolución explícita a tres partes, y un test que exige que **ninguna** clave del núcleo caiga en el genérico |
| F16 | `user-select: none` global impedía copiar direcciones en una aplicación llena de datos | El texto se selecciona; se desactiva solo en navegación y controles |

### Lo que la revisión enseñó sobre los propios tests

Tres defectos estaban **en el aparato de verificación**, no en el producto:

1. **Los tests del llavero compartían una cuenta fija.** Uno borraba la entrada mientras otro corría en paralelo. Los resultados parecían correctos. El llavero es ahora inyectable y cada test usa la suya.
2. **Una comprobación del validador era una búsqueda de texto** en vez de una ejecución real. El caso completo está en §5.
3. **Un test nuevo era demasiado permisivo.** Comprobaba que cada error tuviera tres partes, y el texto genérico también las tiene: pasaba con las diez claves cayendo en el genérico. Ahora exige que ninguna lo haga.

Los tres confirman la regla de método de §5: **una comprobación que nunca se ha visto fallar no está verificada.**

---

## 7. Pendiente

| Qué | Dónde |
|---|---|
| Trámite de verificación OAuth de Google | Arrancado en paralelo — P-05 |
| Certificados de firma (Windows OV/EV, Apple Developer ID) | Plazos externos de 1–3 semanas — R-14 |
| Licencia de Mont | Puerta antes de la demo — D-5, P-01 |
| Jobs de CI de rendimiento y E2E | Fases 4 y 6 |

---

## 8. Cómo reproducir esta validación

```bash
# Dependencias de Linux (Windows y macOS no las necesitan)
sudo apt-get install -y libwebkit2gtk-4.1-dev libxdo-dev libssl-dev \
                        libayatana-appindicator3-dev librsvg2-dev \
                        gnome-keyring dbus-x11

npm --prefix app ci
python3 herramientas/validar/validar.py --fase 1
```

Salida esperada: **32 pasan, 0 fallan, 0 omitidos**.
