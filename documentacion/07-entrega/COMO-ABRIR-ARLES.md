# Cómo abrir ARLES en tu equipo

**Para Adrián Paz · 7 de octubre de 2026**

Hasta hoy el proyecto explicaba cómo **preparar** una demostración real
([`GUIA-DE-CONEXION-REAL.md`](GUIA-DE-CONEXION-REAL.md)) y no decía en ninguna
parte **cómo abrir el programa**. Este documento es esa parte.

> **Lo primero, y es importante que lo sepas antes de empezar.** Hasta el
> 07/10/2026 **ARLES no se había arrancado nunca como aplicación**. Ni una vez.
> Todo lo que está verificado lo está por pruebas automáticas —que no abren
> ninguna ventana— y por la vista previa, que es HTML en un navegador sin nada
> del motor detrás.
>
> Ese día se arrancó por primera vez, en el contenedor donde se escribe el
> código. Lo que quedó comprobado y lo que no está en el §5.

> **Y si sólo quieres probarla, no compilarla**, hay un camino más corto que no
> pide instalar nada:
> [`COMO-PROBAR-ARLES-SIN-COMPILAR.md`](COMO-PROBAR-ARLES-SIN-COMPILAR.md) —
> descargar el instalador de Releases y abrirlo con doble clic. Ahí está también
> **qué borrar, y en qué orden**, si ya habías instalado ARLES antes.

---

## 1 · Lo que hay que instalar en Windows

Tres cosas, una vez:

| Qué | Dónde | Para qué |
|---|---|---|
| **Rust** | [rustup.rs](https://rustup.rs) — el instalador de 64 bits | Compila el motor de ARLES |
| **Node.js 20 o superior** | [nodejs.org](https://nodejs.org) — versión LTS | Compila la interfaz |
| **Visual Studio Build Tools** | El propio instalador de Rust te lo ofrece y lo descarga | Rust en Windows necesita el enlazador de Microsoft |

**WebView2 no hace falta instalarlo**: viene con Windows 11 y con cualquier
Windows 10 actualizado. Es el mismo motor que usa Edge.

Para comprobar que quedó bien, abre PowerShell y escribe:

```powershell
rustc --version
node --version
```

Si las dos contestan con un número, ya está.

> ### Si dice que «cargo no se reconoce» y acabas de instalar Rust
>
> ```
> cargo : The term 'cargo' is not recognized as the name of a cmdlet,
> function, script file, or operable program.
> ```
>
> **No falta Rust: falta cerrar la ventana.** El instalador deja `cargo` en
> `C:\Users\<tu usuario>\.cargo\bin` y añade esa carpeta al PATH, pero el PATH
> se lee **al abrir** cada programa. VS Code y cualquier terminal que ya
> estuviera abierta siguen con el de antes, y abrir una pestaña nueva **no
> basta**: hereda el de VS Code.
>
> **La salida permanente:** cierra VS Code entero y vuelve a abrirlo.
>
> **La salida inmediata**, si no quieres perder lo que tienes abierto:
>
> ```powershell
> $env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
> cargo --version
> ```
>
> Vale sólo para esa ventana.
>
> **Y si aun así no aparece**, comprueba si el programa llegó a instalarse:
>
> ```powershell
> Test-Path "$env:USERPROFILE\.cargo\bin\cargo.exe"
> ```
>
> Un `False` quiere decir que el instalador no terminó. `rustup-init.exe` abre
> una **pantalla de texto que se queda esperando**: hay que escribir `1` y pulsar
> Enter. Si se cierra la ventana ahí, no instala nada y no dice nada.
>
> **Encontrado el 08/10/2026, en el segundo equipo.** Mismo tropiezo que el de
> PowerShell de más abajo: ninguno de los dos es de ARLES, y los dos paran la
> primera vez.

---

## 2 · Traer el código

```powershell
git clone https://github.com/AdrianPazG/ARLES.git
cd ARLES
git checkout claude/loving-faraday-5chmmy
```

Esa última línea importa: es la rama donde está todo el trabajo.

---

## 3 · Abrir ARLES

**Dos comandos.** El primero tarda bastante la primera vez —compila el motor
entero, entre cinco y quince minutos según el equipo—; las siguientes veces son
segundos.

```powershell
cd app
npm install
npm run build
cd ..
cargo arles
```

Y se abre la ventana.

> **`cargo arles`, no `cargo run --release`.** Es un alias del propio proyecto
> (`.cargo/config.toml`) y existe por una razón concreta: añade la característica
> `custom-protocol`, que es lo que hace que la interfaz viaje **dentro** del
> ejecutable. Sin ella la ventana abre pidiendo un servidor de desarrollo que no
> está, y enseña «can't reach this page». Está contado en el §5.

> ### Si PowerShell dice que «no se pueden cargar scripts»
>
> ```
> npm : File C:\Program Files\nodejs\npm.ps1 cannot be loaded because
> running scripts is disabled on this system.
> ```
>
> **Le pasa a todo el mundo la primera vez**, y no tiene nada que ver con ARLES:
> Windows bloquea por defecto los scripts de PowerShell, y `npm` es uno. Hay dos
> salidas.
>
> **La rápida, sin tocar nada del sistema** — añade `.cmd`:
>
> ```powershell
> npm.cmd install
> npm.cmd run build
> ```
>
> `npm` a secas es un script de PowerShell; `npm.cmd` es el programa de toda la
> vida y no pasa por esa restricción.
>
> **La permanente**, para no acordarte del `.cmd` nunca más:
>
> ```powershell
> Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned
> ```
>
> Responde **S**. Permite ejecutar scripts hechos en tu equipo y sigue exigiendo
> firma digital a los descargados de internet; es lo que Microsoft recomienda
> para estaciones de trabajo, afecta **sólo a tu usuario** y se deshace con el
> mismo comando poniendo `Restricted`.
>
> **Encontrado el 08/10/2026, en el primer intento real.** Esta guía se escribió
> sin recorrerla en Windows, y el primer paso falló: queda aquí porque el
> siguiente que la lea no debería tropezar con lo mismo.

> **Por qué así y no con el modo de desarrollo de Tauri.** Esta vía no necesita
> instalar nada más, y el programa arranca con la interfaz ya compilada dentro
> del ejecutable — que es exactamente lo que verá un usuario. El modo de
> desarrollo (recarga en caliente al editar) es cómodo para programar y exige un
> paso extra; está en el §6.

Cuando termine, el programa queda en
`target\release\arles-app.exe`. Puedes abrirlo con doble clic las veces
siguientes, sin volver a compilar.

---

## 4 · Qué va a pasar la primera vez

**Windows te va a pedir permiso para guardar una credencial.** Eso es correcto y
es lo que tiene que pasar: ARLES genera una llave de cifrado para tu base de
datos y la guarda en el **Administrador de credenciales** de Windows, nunca en
un archivo (ADR-0011).

Esto no es un detalle de implementación, es la decisión central del producto:

- Tus datos viven cifrados en `%APPDATA%\TelemetryInsight\ArlesRelay\data\`,
  en un archivo llamado `arles.db`.
- **La llave vive en Windows, no en ARLES.** Si alguien se lleva el archivo de
  la base, no puede leerlo.
- **Si el Administrador de credenciales no está disponible, ARLES no arranca.**
  Nunca se degrada a guardar la llave en texto plano. Lo verificamos el
  07/10/2026: al arrancar, lo primero que hace el programa es pedir el llavero y
  **se queda esperando** hasta que conteste.

Después verás **Inicio**, con la lista de seis pasos. El primero es configurar
los datos de tu empresa.

> ### Si dice «fallo al migrar el esquema»
>
> ```
> fallo al migrar el esquema: applied migration V1__esquema_inicial is
> different than filesystem one V1__esquema_inicial
> ```
>
> **Tienes una base de una versión anterior de ARLES.** El esquema inicial
> cambió después de que se creara, y el motor de migraciones guarda la huella
> del SQL de cada una: al no coincidir, **se niega a tocarla**. Y hace bien —
> aplicar migraciones sobre una base que no es la que esperan puede corromperla.
>
> **Desde el 14/10/2026 el propio cuadro te dice qué hacer**, con la ruta de tu
> base dentro. Hasta entonces decía dos cosas que no servían —«cierra ARLES y
> vuelve a abrirlo», que da el mismo error siempre, y «restaura tu último
> respaldo», que puede no existir— y **no decía dónde está el archivo**, que es
> lo único que se puede hacer. Dirección lo vio el 13/10/2026 al abrir el primer
> instalador descargable sobre una base de la versión anterior.
>
> **Lo que funciona: apartar la base.** No la borres — la llave sigue en el
> llavero, así que esos datos se podrán recuperar el día que exista la entrega
> que migra un respaldo de otro esquema.
>
> **Sin terminal, que es lo que no se puede escribir mal:**
>
> 1. Cierra ARLES.
> 2. **Win + R** → pega `%APPDATA%\TelemetryInsight\ArlesRelay\data` → Enter.
> 3. Clic derecho en `arles.db` → **Cambiar nombre** → `arles-vieja.db` → Enter.
> 4. Abre ARLES.
>
> Con terminal, **y la sintaxis depende de cuál abras**:
>
> ```powershell
> # PowerShell (dice «Windows PowerShell» o «Terminal» arriba)
> cd "$env:APPDATA\TelemetryInsight\ArlesRelay\data"
> Rename-Item arles.db "arles-vieja-$(Get-Date -Format yyyyMMdd).db"
> ```
>
> ```bat
> rem Símbolo del sistema (dice «Command Prompt»)
> cd "%APPDATA%\TelemetryInsight\ArlesRelay\data"
> ren arles.db arles-vieja.db
> ```
>
> **`$env:APPDATA` en Símbolo del sistema no funciona** y da «The filename,
> directory name, or volume label syntax is incorrect» — pasó el 14/10/2026 por
> dar el comando sin decir la terminal. Está en
> [`REGISTRO-DE-TROPIEZOS.md`](REGISTRO-DE-TROPIEZOS.md) §5.
>
> Al volver a abrir, ARLES crea una base nueva y vacía. **Lo que había dentro no
> vuelve**: si eran datos que te importan, dímelo antes de hacer esto.
>
> **Encontrado el 08/10/2026, en el primer intento real.**

---

## 5 · Lo que está verificado y lo que no

**Medido el 07/10/2026, con la aplicación compilada en modo release:**

| | |
|---|---|
| Compila sin un solo error ni advertencia | ✅ |
| El ejecutable arranca y se mantiene abierto | ✅ |
| Pide el llavero del sistema antes de abrir nada, y espera | ✅ |
| **La interfaz dentro de la ventana** | ⬜ **no se pudo ver** |

Lo último merece la explicación completa, porque es la diferencia entre «no
falló» y «funciona»: el llavero del contenedor donde se escribe el código pide
una contraseña interactiva, y desbloquearlo por línea de comandos **termina la
sesión de trabajo**. Se intentó tres veces por caminos distintos. Así que lo que
está comprobado es que el programa arranca y pide la llave; **que la ventana
pinte la interfaz lo vas a ver tú primero.**

En Windows ese paso no existe: el Administrador de credenciales se desbloquea
con tu sesión y no pregunta nada.

**Si la ventana se abre en blanco o en negro**, es que el frontend no se embebió:
vuelve a correr `npm run build` dentro de `app` y después `cargo arles` otra
vez. Si sigue, dímelo con lo que diga la consola.

**Si la ventana dice «can't reach this page» o «localhost refused to
connect»**, estás arrancando sin la característica `custom-protocol`: usa
`cargo arles`, no `cargo run --release` a secas. Está explicado arriba.

---

## 6 · El modo de desarrollo, si alguna vez lo necesitas

Recarga la interfaz al guardar un archivo, sin recompilar el motor:

```powershell
cargo install tauri-cli --version "^2"
cd crates\arles-app
cargo tauri dev
```

**Tiene que ser desde `crates\arles-app`**, que es donde vive `tauri.conf.json`.
Comprobado el 07/10/2026: desde cualquier otra carpeta el comando arranca pero
no encuentra la configuración, y lo hace **sin decirlo** — la sección «App» de
`cargo tauri info` sale vacía.

---

## 7 · Una cosa que conviene tener presente

El ejecutable que compiles lleva dentro la tipografía **Mont**, y la licencia que
cubre incrustarla en una aplicación distribuida es el bloqueo **P-01**, que sigue
abierto.

Compilarlo y usarlo **en tu propio equipo** no es distribuirlo; lo que P-01
impide es **entregar el instalador** a terceros. Aun así, mientras P-01 siga
abierto conviene que ese `.exe` no salga de tus manos — ni por correo, ni en una
carpeta compartida, ni a un cliente.

Si quieres verlo sin la tipografía, la vista previa tiene un modo para eso:
`python3 herramientas/vista-previa/generar.py --sin-mont`.

---

## 8 · Y después, la demostración real

Con ARLES abierto, el orden es el de
[`GUIA-DE-CONEXION-REAL.md`](GUIA-DE-CONEXION-REAL.md):

1. **Ajustes → Empresa**: los datos de TELEMETRY.
2. **Remitentes**: añade la cuenta de Gmail desechable con su contraseña de
   aplicación, y pulsa **«Probar conexión»**. Aquí se cierra **R-24**.
3. **Remitentes → WhatsApp**: el número de prueba de Meta y su token.
4. **Contactos**: date de alta a ti mismo, con tu correo y tu móvil.
5. **Mandar un correo de prueba** y **mandar un WhatsApp de prueba**.
6. Si los dos llegan, **crea una campaña entera** por correo desde Campañas →
   Nueva campaña.

**Lo que no va a salir**, y conviene no intentarlo delante de nadie: una campaña
completa **por WhatsApp**. El motor sólo recorre correo; de WhatsApp sale el
envío de prueba y nada más.
