# Probar ARLES sin compilar nada

**Para Adrián Paz · 13 de octubre de 2026**

[`COMO-ABRIR-ARLES.md`](COMO-ABRIR-ARLES.md) explica cómo **compilar** ARLES, que
exige instalar Rust, Node y las Build Tools de Visual Studio. Este documento es
la otra vía: **descargar un instalador ya construido** y abrirlo con doble clic.

> **Para probar la aplicación no hace falta Visual Studio, ni VS Code, ni Rust,
> ni Node.** Nada de eso. Se descarga un `.exe`, se instala y se abre. Las
> herramientas sólo hacen falta para **compilar**, que es lo que hace el
> servidor de GitHub por nosotros.

---

## 0 · La ruta corta · cuatro pasos

**Si ya tenías ARLES instalado, haz el paso 1. Si es la primera vez, sáltatelo.**

### 1 · Aparta la base anterior

Pega esto en el **Explorador de archivos**, en la barra de la dirección:

```
%APPDATA%\TelemetryInsight\ArlesRelay\data
```

Si hay un `arles.db`, **renómbralo** a `arles-anterior.db`. No lo borres: si
algo sale mal, ese archivo es lo único que tiene tus datos.

> **Por qué hace falta.** Desinstalar ARLES no borra tus datos —y hace bien, son
> tuyos—, pero el esquema de la versión anterior ya no encaja. Sin esto sale
> «fallo al migrar el esquema» nada más abrir.
>
> **No toques el llavero de Windows.** Borrar la credencial dejando la base deja
> esos datos ilegibles para siempre. El orden está explicado en el §4.

### 2 · Descarga el instalador

Ve a **[Releases](https://github.com/AdrianPazG/ARLES/releases)** y baja el
primero de la lista. El archivo que quieres es:

```
ARLES.RELAY_1.2.0_x64-setup.exe
```

### 3 · Instálalo

Doble clic. **Windows va a decir «editor desconocido»**: es correcto y esperado,
la aplicación no está firmada todavía. Pulsa **«Más información» → «Ejecutar de
todas formas»**.

### 4 · Ábrelo

Se abre solo al terminar, o desde el menú de Inicio.

**Lo primero que verás es «Empieza por aquí»**, porque sin los datos de tu
empresa no se puede hacer nada más. A partir de ahí, el
[manual de usuario](MANUAL-DE-USUARIO.md) te lleva paso a paso.

> **Si algo te para**, la respuesta está casi seguro en
> [`REGISTRO-DE-TROPIEZOS.md`](REGISTRO-DE-TROPIEZOS.md): los siete tropiezos
> que ya nos han pasado de verdad, con su causa y su salida exacta.

---

## 1 · De dónde se descarga

De **[Releases](https://github.com/AdrianPazG/ARLES/releases)**, el enlace de la
derecha de la portada del repositorio.

**No de la pestaña Actions**: ahí sólo se ve cómo se construyó.

**La versión al día es
[`prueba-20261015-2105`](https://github.com/AdrianPazG/ARLES/releases/tag/prueba-20261015-2105)**,
del 15/10/2026, y trae tres archivos:

| Archivo | Para | Tamaño |
|---|---|---|
| `ARLES.RELAY_1.2.0_x64-setup.exe` | **Windows** · el recomendado | 6.1 MB |
| `ARLES.RELAY_1.2.0_x64_en-US.msi` | Windows · para instalación desatendida | 8.6 MB |
| `ARLES.RELAY_1.2.0_aarch64.dmg` | **macOS** (Apple Silicon) | 8.1 MB |

**Qué trae de nuevo respecto a la del 14/10:**

- **A-05** · la contraseña de aplicación de Gmail ya se puede pegar **con los
  espacios** con que Google la enseña. Antes viajaban al servidor y Gmail la
  rechazaba, **sin que nada avisara**.
- **A-06** · la guía del alta dice **la dirección exacta** donde se crea esa
  contraseña, con un botón para copiarla, el aviso de que **sólo se enseña una
  vez**, y los **cuatro casos** en que esa página no aparece.

Y lo que ya traía la del 14/10: el alta de correo en dos campos (R-01), el alta
de WhatsApp con avisos de pegado (R-02), el aviso de la empresa que ya no
sobrevive a la recarga (H-01), las tres partes de un error que ya no salen
pegadas (H-02) y la cabecera de Contactos que ya no se lee «WhatsApp pri…»
(A-01). Todo está en
[`BITACORA-DE-PRUEBAS.md`](../06-calidad/BITACORA-DE-PRUEBAS.md).

> **Medida también, y con la misma regla.** Se descargó el `.exe`, se extrajo el
> NSIS y se miró dentro: el binario pesa **22 273 024 bytes** y trae **siete**
> apariciones de `index.html`. Siete es el número de una compilación con
> `custom-protocol`; **sin ella serían cero**, y la ventana abriría pidiendo
> `localhost` — que es lo que le pasó a Dirección el 08/10/2026.

> ### Y una corrección del método, del 15/10/2026
>
> Buscar en el binario **el texto de la interfaz no sirve**, y conviene saberlo
> antes de sacar conclusiones de ahí. Al verificar esta versión busqué
> `apppasswords` dentro del `.exe` y salió **cero** — lo que parecía decir que
> el texto nuevo no viajaba.
>
> **No era eso.** El frontend va comprimido dentro del binario, así que ninguna
> de sus cadenas se encuentra en claro. Lo que se encontraba —«Remitentes»,
> «contraseña», `smtp.gmail.com`— son cadenas de **Rust**, y por eso «Campañas»
> salía a cero mientras «Remitentes» salía a dos: la diferencia no era el
> idioma ni el acento, era de qué lado venía cada una.
>
> `index.html` **sí** vale como medida, y es la que está escrita arriba, porque
> es una referencia que Tauri deja en claro y **discrimina**: cero sin la
> característica, siete con ella.
>
> Para comprobar un texto de la interfaz, lo que vale es construir el frontend
> del mismo commit y buscarlo en `app/dist`. Hecho aquí: `apppasswords` aparece
> dos veces, «sólo se enseña una vez» una, y el texto viejo —«busca
> «Contraseñas»»— **cero**.
>
> **Una medida que no discrimina no mide.** Si una cadena ausente y una cadena
> presente dan el mismo resultado, lo que falla es el instrumento.

> **Por qué no había ninguna hasta hoy, y no era por falta de código.** El flujo
> que las construye se intentó una vez, el 17/09/2026, y falló: `sin-mont.py`
> reescribió el CSS —su trabajo entero, hecho— y reventó **al imprimir
> «✓ Aplicado»**, porque la consola de Windows usa cp1252 y no tiene ese
> carácter. `UnicodeEncodeError`, código de salida 1, y el flujo entero cayó.
> **Veintiséis días sin instalador, por un carácter decorativo.**

> **Esta versión está medida, no supuesta.** Se extrajo el instalador NSIS y se
> miró dentro: el binario pesa **22 266 368 bytes** y trae **siete** apariciones
> de `index.html` más los assets del frontend. Son las mismas siete que el
> 08/10/2026 con `custom-protocol` activada — es decir, **la interfaz viaja
> dentro del ejecutable** y la ventana no va a abrir pidiendo `localhost`.
> Lo vigila ahora `la_aplicacion_se_construye_como_aplicacion` en `--fase 1`,
> que hasta hoy miraba sólo el camino de `cargo arles` y no el del instalador.

---

## 2 · Cómo se instala en Windows

1. Descarga el `.exe`.
2. Doble clic. **Windows dirá «editor desconocido»** y SmartScreen pedirá
   confirmación.
3. **Más información** → *Ejecutar de todas formas*.

> **Ese aviso sale porque la versión no está firmada**, no porque algo vaya mal.
> El certificado de firma de Windows es uno de los tres bloqueos de la Fase 9, y
> no es trabajo de programar. Mientras tanto, cada versión de prueba lo enseña.

### Lo que verás distinto, y es esperado

**La tipografía no es la de la marca.** Estas versiones van **sin Mont**, porque
la licencia para incrustarla en una aplicación que se distribuye sigue sin
confirmarse (P-01). ARLES usa Figtree o la letra del sistema.

Los **tamaños, los pesos y el ritmo vertical sí son los definitivos**: salen de
los tokens, no de la tipografía. Un hallazgo del tipo «la letra no es la nuestra»
en una versión de prueba es esperado y no hace falta anotarlo.

---

## 3 · Qué pasa la primera vez

**Windows te pedirá permiso para guardar una credencial.** Es correcto y es lo
que tiene que pasar: ARLES genera una llave para cifrar tu base de datos y la
guarda en el **Administrador de credenciales**, nunca en un archivo (ADR-0011).

- Tus datos viven cifrados en
  `%APPDATA%\TelemetryInsight\ArlesRelay\data\arles.db`.
- **La llave vive en Windows, no en ARLES.** Si alguien se lleva el archivo de la
  base, no puede leerlo.
- **Si el Administrador de credenciales no responde, ARLES no arranca.** Nunca
  se degrada a texto plano.

Después verás **Inicio**, con la lista de seis pasos.

---

## 4 · Si ya habías instalado ARLES antes

> ### Lo primero que te va a pasar: «fallo al migrar el esquema»
>
> ```
> ARLES RELAY no pudo arrancar.
> Qué pasó: fallo al migrar el esquema: applied migration V1__esquema_inicial
> is different than filesystem one V1__esquema_inicial.
> ```
>
> **Es lo esperado, no un defecto del instalador.** Desinstalar ARLES **no borra
> tus datos** —y hace bien, son tuyos—, así que la base de la versión anterior
> sigue ahí. Su esquema ya no encaja con el de esta versión, y el motor de
> migraciones guarda la huella del SQL de cada una: al no coincidir, **se niega
> a tocarla**. Aplicar migraciones sobre una base que no es la que esperan puede
> corromperla.
>
> **Desde el 14/10/2026 el propio cuadro te dice qué hacer, con la ruta dentro.**
> Antes decía «cierra ARLES y vuelve a abrirlo» —que da el mismo error siempre—
> y «restaura tu último respaldo» —que puede no existir—, sin decir dónde está
> el archivo. Lo vio Dirección el 13/10/2026, con el primer instalador
> descargable.
>
> La salida es la de abajo: **apartar la base, no borrarla.**

Aquí hay **un orden que importa**, y hacerlo al revés deja datos ilegibles para
siempre. Esto sale de leer `estado.rs`, no de suponerlo:

| Llavero | Base en disco | Qué hace ARLES al abrir |
|---|---|---|
| sin clave | **con** base | **Pantalla de recuperación.** La base ya no se puede leer: hace falta un respaldo |
| sin clave | sin base | Crea una clave nueva. Arranque limpio ✅ |
| con clave | sin base | Crea una base nueva con esa clave. Arranque limpio ✅ |

> **La regla, en una frase: nunca borres la credencial dejando la base.**
>
> ARLES **no** crea una clave nueva al encontrarse una base que no puede
> descifrar, y es deliberado (hallazgo F2 de la Fase 1): un llavero puede estar
> vacío por motivos pasajeros, y una clave nueva dejaría esos datos perdidos
> **para siempre** sin que nadie se entere. Por eso te manda a la pantalla de
> recuperación en vez de seguir.

### Empezar de cero · la forma corta y segura

**Basta con borrar la base.** La credencial puede quedarse: ARLES creará una base
nueva y la cifrará con la misma llave.

1. **Cierra ARLES.**
2. Pulsa `Win + R`, pega esto y Enter:

   ```
   %APPDATA%\TelemetryInsight\ArlesRelay\data
   ```

3. **Clic derecho** sobre `arles.db` → **Cambiar nombre** → escribe
   `arles-vieja.db` → **Enter**.

   Si Windows no te muestra la parte `.db`, escribe sólo `arles-vieja`.

4. Abre ARLES.

**Renombrar y no borrar**: la llave sigue en Windows, así que se puede volver
atrás. Borrándolo, no.

> **Se hace con el Explorador y no con la terminal a propósito.** Windows tiene
> dos terminales que **no entienden la misma sintaxis** —PowerShell usa
> `$env:APPDATA` y Símbolo del sistema usa `%APPDATA%`—, y dar un comando sin
> decir cuál paró el trabajo el 14/10/2026. `Win + R` funciona en cualquier caso.
> Está en [`REGISTRO-DE-TROPIEZOS.md`](REGISTRO-DE-TROPIEZOS.md) §5.

Al abrir ARLES otra vez, estarás en el primer arranque.

### Desinstalar del todo, credencial incluida

Sólo si quieres no dejar rastro. **Primero la base, después la credencial**:

1. **Cierra ARLES** y desinstálalo desde *Configuración → Aplicaciones*.
2. Borra la carpeta `%APPDATA%\TelemetryInsight\ArlesRelay`.
3. Abre el **Administrador de credenciales** (`Win` → escribe «credenciales»)
   → **Credenciales de Windows** → **Credenciales genéricas**.
4. Busca las entradas que mencionan **`mx.telemetryinsight.arlesrelay`** y
   quítalas. Puede haber varias:

   | Entrada | Qué guarda |
   |---|---|
   | `db-master-key` | **La llave que cifra la base.** Es la crítica |
   | `email-account/<identificador>` | La contraseña de aplicación de cada cuenta de correo |
   | `whatsapp-number/<identificador>` | El token de Meta de cada número |

> **El nombre exacto con el que Windows las muestra depende de la versión de la
> biblioteca `keyring`, así que no se promete aquí**: búscalas por
> `mx.telemetryinsight.arlesrelay`, que es el servicio y sí está fijado en el
> código (`llavero.rs`). Si no aparece ninguna, es que nunca llegaste a abrir
> ARLES en este equipo.

**Si borras una contraseña de correo pero dejas la base**, no pasa nada grave: esa
cuenta aparecerá como «hay que configurarla» y se vuelve a escribir. La única que
no se puede reponer es `db-master-key`.

---

## 5 · Qué se puede hacer con una versión de prueba

Al 08/10/2026, con las diez fases del validador en **324/324**:

- Configurar la empresa y **cargar contactos** desde CSV o XLSX, con los choques
  revisados antes de importar; bajas y derechos ARCO.
- Dar de alta una cuenta de correo y **mandarse un correo real**.
- Escribir plantillas y firmas, crear una campaña con el asistente de nueve pasos
  y **activarla**.
- Verla enviar en Actividad, con sus cifras y **por qué no sale nada**, y
  **pararla** en cualquier momento.
- Dar de alta un número de WhatsApp y **mandarse un WhatsApp real**.
- **Respaldo cifrado, restauración, archivado del historial** y la pantalla de
  recuperación del llavero perdido.

### Lo que todavía NO hace

- Una **campaña entera por WhatsApp**: el motor sólo recorre correo. De WhatsApp
  sale el mensaje de prueba y nada más.
- **Filtrar la audiencia** y **editar una campaña ya creada**.
- **Activar una campaña grande** tarda y congela la ventana: 73.5 s con 451 196
  destinatarios, y empieza a notarse hacia los 30 000. Con una lista de prueba no
  se nota.

---

## 6 · Cómo se genera una versión nueva

**Con una etiqueta que empiece por `prueba-`:**

```bash
git tag prueba-2026-10-13
git push origin prueba-2026-10-13
```

O desde la API, que es como se lanzó la del 13/10/2026 —desde el contenedor de
desarrollo las etiquetas dan 403, pero `workflow_dispatch` sí pasa—.

> **El botón «Run workflow» de la pestaña Actions no aparece**, y no es un fallo:
> GitHub sólo lo enseña cuando el archivo del flujo está en la rama **por
> defecto** (`main`), y el trabajo vive en `claude/loving-faraday-5chmmy`.
> Aparecerá solo el día que esto llegue a `main`.

**Nunca se dispara sola.** No hay compilación en cada push: una versión sin firmar
que se publica sola acaba instalada donde nadie la pidió.

---

## 7 · Lo que esta vía no cubre

| | Por qué |
|---|---|
| **Ver la tipografía definitiva** | Va sin Mont hasta que se cierre P-01 |
| **Una instalación sin avisos de Windows** | Falta el certificado de firma (Fase 9) |
| **macOS** | Se construye, pero **nadie ha abierto ARLES en un Mac todavía** — es el riesgo R-07, y sigue abierto |

Para compilar en tu propio equipo, con Mont y sin esperar al servidor:
[`COMO-ABRIR-ARLES.md`](COMO-ABRIR-ARLES.md).
