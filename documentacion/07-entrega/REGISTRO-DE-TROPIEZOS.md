# Registro de tropiezos

**Para Adrián Paz · abierto el 14 de octubre de 2026**

Cada vez que algo te para al usar ARLES, entra aquí: **qué viste, por qué pasó y
qué hacer**. Y al final, las reglas que salieron de cada uno para que no vuelva a
ocurrir.

> **Por qué existe este archivo.** Entre el 8 y el 14 de octubre de 2026
> tropezaste **siete veces** antes de poder abrir ARLES. Ninguna fue un fallo del
> producto en sí: fueron instrucciones mal escritas, un carácter en un script, y
> cosas que el programa sabía y no decía. Todas eran evitables, y varias se
> repitieron porque la lección estaba escrita **en un comentario que nadie
> leía**, en vez de en un sitio donde se busca.
>
> La regla del proyecto es «una comprobación que nunca se ha visto fallar no está
> verificada» (§2). Este archivo es su versión para las instrucciones: **una
> instrucción que nadie ha seguido no es una instrucción verificada.**

---

## Cómo se usa

**Si algo te para:** busca el síntoma en la tabla de abajo, ve a su apartado.

**Si no está:** dímelo con la captura. Lo arreglo, y lo añado aquí con su fecha.

| # | Lo que ves | Dónde |
|---|---|---|
| 1 | `npm : File ... npm.ps1 cannot be loaded` | [§1](#1--npm-no-se-puede-cargar) |
| 2 | `cargo : The term 'cargo' is not recognized` | [§2](#2--cargo-no-se-reconoce) |
| 3 | La ventana de ARLES dice «can't reach this page» | [§3](#3--la-ventana-abre-vacía) |
| 4 | `fallo al migrar el esquema` | [§4](#4--fallo-al-migrar-el-esquema) |
| 5 | `The filename, directory name, or volume label syntax is incorrect` | [§5](#5--el-comando-no-funciona-en-esta-terminal) |
| 6 | No encuentras la carpeta de datos | [§6](#6--la-carpeta-de-datos-no-está-donde-decía) |
| 7 | No hay nada que descargar en Releases | [§7](#7--no-había-ningún-instalador) |

---

## 1 · «npm no se puede cargar»

```
npm : File C:\Program Files\nodejs\npm.ps1 cannot be loaded because
running scripts is disabled on this system.
```

**Cuándo:** 08/10/2026, primer intento de compilar ARLES.

**Por qué pasa.** Windows bloquea por defecto los scripts de PowerShell, y `npm`
es uno. **No tiene nada que ver con ARLES** — le pasa a todo el mundo la primera
vez que usa Node en un equipo nuevo.

**Qué hacer.** Dos salidas, las dos válidas:

**La rápida**, sin tocar nada del sistema — añade `.cmd`:

```powershell
npm.cmd install
npm.cmd run build
```

**La permanente**, para no acordarte nunca más:

```powershell
Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned
```

Responde **S**. Afecta **sólo a tu usuario**, permite scripts hechos en tu equipo
y sigue exigiendo firma a los descargados de internet. Se deshace con el mismo
comando poniendo `Restricted`.

**Sólo te afecta si compilas.** Si descargas el instalador, no.

---

## 2 · «cargo no se reconoce»

```
cargo : The term 'cargo' is not recognized as the name of a cmdlet,
function, script file, or operable program.
```

**Cuándo:** 08/10/2026, en un segundo equipo, **con Rust ya instalado**.

**Por qué pasa.** No falta Rust: **falta cerrar la ventana**. El instalador deja
`cargo` en `C:\Users\<tu usuario>\.cargo\bin` y añade esa carpeta al PATH, pero
**el PATH se lee al abrir cada programa**. VS Code y las terminales que ya
estaban abiertas siguen con el de antes — y abrir una pestaña nueva **no basta**,
porque hereda el de VS Code.

**Qué hacer.**

1. **Cierra VS Code entero** y vuelve a abrirlo. Ésa es la salida permanente.
2. Si no quieres perder lo que tienes abierto, para esa ventana:
   ```powershell
   $env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
   cargo --version
   ```
3. Si aún así no aparece, comprueba si llegó a instalarse:
   ```powershell
   Test-Path "$env:USERPROFILE\.cargo\bin\cargo.exe"
   ```
   Un `False` quiere decir que el instalador no terminó. `rustup-init.exe` abre
   **una pantalla de texto que se queda esperando**: hay que escribir `1` y pulsar
   Enter. Cerrar la ventana ahí no instala nada y no avisa.

**Sólo te afecta si compilas.**

---

## 3 · La ventana abre vacía

> Hmmm… can't reach this page — **localhost refused to connect**

**Cuándo:** 08/10/2026, la primera vez que se construyó ARLES como aplicación.

**Por qué pasó.** Tauri **no decide por el modo de compilación** si lo que
construye es un programa terminado o un entorno de desarrollo: lo decide por una
característica del compilador llamada `custom-protocol`. Sin ella, **incluso
compilando en modo release**, la aplicación busca su interfaz en un servidor de
desarrollo que no está.

**Estuvo nueve fases sin verse porque nadie había construido la aplicación.**
Todo se verificaba con pruebas —que no abren ninguna ventana— y con la vista
previa, que es HTML en un navegador sin nada de Rust detrás.

**Qué hacer.** Si compilas tú, usa **`cargo arles`**, nunca `cargo run --release`
a secas:

```powershell
cd app
npm install
npm run build
cd ..
cargo arles
```

**Arreglado el 08/10/2026.** Los instaladores descargables lo traen bien: está
medido extrayendo el `.exe` — el binario trae siete apariciones de `index.html`,
que es la interfaz viajando dentro.

---

## 4 · «fallo al migrar el esquema»

```
ARLES RELAY no pudo arrancar.
Qué pasó: fallo al migrar el esquema: applied migration V1__esquema_inicial
is different than filesystem one V1__esquema_inicial.
```

**Cuándo:** 13/10/2026, al instalar una versión nueva **teniendo ya datos de una
anterior**.

**Por qué pasa, y por qué está bien que pase.** Desinstalar ARLES **no borra tus
datos** — y hace bien, son tuyos. Pero el esquema de esa base es el de la versión
vieja, y el motor de migraciones guarda la huella del SQL de cada una: al no
coincidir, **se niega a tocarla**. Aplicar migraciones sobre una base que no es la
que esperan puede corromperla.

**Qué hacer: apartar la base. No borrarla.**

1. **Cierra ARLES** (pulsa OK en el cuadro).
2. **Win + R** → pega esto → Enter:
   ```
   %APPDATA%\TelemetryInsight\ArlesRelay\data
   ```
3. **Clic derecho** en `arles.db` → **Cambiar nombre** → `arles-vieja.db` → Enter.
4. Abre ARLES.

**Por qué renombrar y no borrar:** la llave que descifra ese archivo sigue en el
Administrador de credenciales de Windows, así que esos datos se podrán recuperar
el día que exista la entrega que migra un respaldo de otro esquema. Borrado, no.

> **El mensaje de ARLES decía algo que no servía, hasta el 14/10/2026.** Decía
> «cierra ARLES y vuelve a abrirlo» —que da **exactamente el mismo error,
> siempre**— y «restaura tu último respaldo» —que puede no existir—. Y no decía
> **dónde** está el archivo, que es lo único que se puede hacer.
>
> Desde el 14/10/2026 el propio cuadro trae la ruta de tu base, avisa de que
> reabrir no sirve, y dice que renombres sin borrar.

---

## 5 · El comando no funciona «en esta terminal»

```
The filename, directory name, or volume label syntax is incorrect.
```

**Cuándo:** 14/10/2026, al intentar apartar la base.

**Por qué pasa.** Windows tiene **dos terminales distintas**, y **no entienden la
misma sintaxis**:

| | Se llama | Cómo se escribe una carpeta del sistema |
|---|---|---|
| **PowerShell** | «Windows PowerShell» o «Terminal» | `$env:APPDATA` |
| **Símbolo del sistema** | «Command Prompt» o `cmd` | `%APPDATA%` |

Si pegas un comando de PowerShell en Símbolo del sistema, `$env:APPDATA` se toma
como texto literal y la ruta no existe. **Ése fue el error, y fue mío por no
decirte en qué terminal.**

**Qué hacer.**

**Lo más seguro: no uses terminal.** Para abrir una carpeta del sistema, usa
**Win + R** y pega la ruta con `%APPDATA%`. Funciona siempre, venga de donde
venga.

**Si usas terminal**, mira arriba del todo de la ventana:

- Dice **«Windows PowerShell»** o el fondo es azul → usa `$env:APPDATA`
- Dice **«Command Prompt»** o **«Símbolo del sistema»** → usa `%APPDATA%`

Las dos versiones del mismo comando:

```powershell
# PowerShell
cd "$env:APPDATA\TelemetryInsight\ArlesRelay\data"
```

```bat
rem Símbolo del sistema
cd "%APPDATA%\TelemetryInsight\ArlesRelay\data"
```

---

## 6 · La carpeta de datos no está donde decía

**Cuándo:** 13/10/2026.

**Por qué pasó.** La guía decía que tus datos están en
`%APPDATA%\mx.telemetryinsight.arlesrelay\`. **Eso no existe.** Ese nombre es el
del **servicio del llavero** —cómo se llama ARLES dentro del Administrador de
credenciales—, no el de la carpeta.

**Dónde están de verdad:**

```
%APPDATA%\TelemetryInsight\ArlesRelay\data\arles.db
```

Comprobado leyendo el código de la librería que construye esa ruta, no de
memoria. En un Mac sería
`~/Library/Application Support/mx.TelemetryInsight.ArlesRelay/arles.db` — ahí sí
lleva el `mx.`, que es lo que causó la confusión.

**Corregido el 13/10/2026** en la guía, en el documento de arquitectura y en el
código.

---

## 7 · No había ningún instalador

**Cuándo:** hasta el 13/10/2026, la pestaña Releases estaba vacía.

**Por qué.** El flujo que construye los instaladores se intentó **una vez**, el
17/09/2026, y falló. No por el código: el script que quita la tipografía Mont
antes de empaquetar hizo su trabajo entero, reescribió lo que tenía que
reescribir, y **reventó al imprimir «✓ Aplicado»** — la consola de Windows usa
una codificación que no tiene ese carácter. Error, código de salida 1, y el flujo
entero cayó.

**Veintiséis días sin instalador por un carácter decorativo.**

**Arreglado el 13/10/2026**, y verificado reproduciendo el fallo exacto antes de
tocar nada.

---

# Las reglas que salieron de estos siete

Están aquí para que **yo** no las repita, y para que puedas señalarme cuando lo
haga.

### 1 · Toda instrucción dice en qué terminal se ejecuta

El tropiezo §5 fue exactamente esto. Si un comando lleva `$env:` es PowerShell; si
lleva `%VARIABLE%` es Símbolo del sistema. **Cuando haya una forma sin terminal
—el Explorador, Win+R—, ésa va primero.**

### 2 · Una instrucción que nadie ha seguido no está verificada

La guía de arranque se escribió «verificando por lectura» y falló **tres veces
seguidas** en el primer intento real. Es el §2 del proyecto aplicado a la
documentación: **al escribir una guía, se recorre.**

### 3 · Una ruta se lee del código, no de la memoria

El tropiezo §6 salió de confundir dos nombres parecidos. La ruta correcta se
obtuvo leyendo el código de la librería que la construye.

### 4 · Una herramienta de apoyo no puede tumbar lo que apoya

El tropiezo §7: un `print` decorativo tiró una compilación entera. Ahora la
salida de esos scripts no puede abortar aunque un carácter no se pueda escribir.

### 5 · Un mensaje de error dice tres cosas, y las tres tienen que servir

Qué pasó, **cómo arreglarlo** y qué está a salvo (§95). El tropiezo §4 cumplía la
forma y fallaba el fondo: mandaba a un bucle infinito y a restaurar un archivo
inexistente. **Una instrucción que no se puede seguir es peor que ninguna**,
porque se intenta.

### 6 · Medir que una cadena aparece no es medir que esté donde importa

La comprobación que vigilaba la guía preguntaba si el comando bueno «aparecía» en
el archivo — y aparecía, en una nota al pie, mientras el bloque que se copia
seguía diciendo el malo. Por eso escribiste el comando viejo en el segundo equipo.

### 7 · Lo que se construye hay que abrirlo

El tropiezo §3 estuvo nueve fases invisible porque **nadie había ejecutado la
aplicación**: mil pruebas en verde y una ventana vacía. Antes de dar algo por
hecho, se abre y se mira.

---

## Lo que todavía puede pasarte

No son defectos: son límites conocidos, y conviene no buscarles solución.

| Qué | Por qué |
|---|---|
| **Windows dice «editor desconocido»** | Las versiones de prueba no están firmadas. Falta el certificado, que es trabajo de la Fase 9 y no de programar. **Más información → Ejecutar de todas formas** |
| **La letra no es la de la marca** | Van sin Mont mientras P-01 siga abierto. Los tamaños y pesos sí son los definitivos |
| **Una campaña entera por WhatsApp no sale** | El motor sólo recorre correo. De WhatsApp sale el mensaje de prueba |
| **No se puede filtrar la audiencia ni editar una campaña creada** | No están construidos |
| **Activar una campaña grande congela la ventana** | 73,5 s con 451 196 destinatarios; se nota desde unos 30 000. Está en la Fase 9 |
| **Nada se ha probado en un Mac** | Es el riesgo R-07, sigue abierto |
| **Dos campañas con el mismo remitente suman sus correos** | Ya no tardan el doble: desde el 22/10/2026 cada campaña manda su propio máximo y el plazo se cumple. Lo que pasa es lo otro: **de ese buzón sale la suma**, así que dos campañas de 50 son 100 al día. Está a la vista en Remitentes con un semáforo —verde hasta 60, ámbar hasta 150, rojo por encima— y **ARLES ya no te pone ningún techo ahí**. La salida, si la suma es mucha: un remitente distinto para cada campaña |
| **Una firma no puede llevar imagen** | Sólo texto, con negrita y cursiva. Falta decidir cómo (R-05) |

> **Y uno que dejó de pasar el 22/10/2026:** el correo caía en no deseados de
> Outlook mandando desde una dirección `@gmail.com`, con toda la autenticación
> en verde. **No se arregló: se dejó de hacer.** Se manda desde una dirección
> del dominio propio y llega a la bandeja principal — ver
> [`QUE-HACER-CON-EL-SPAM.md`](QUE-HACER-CON-EL-SPAM.md) §9.
