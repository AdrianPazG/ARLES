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
cargo run --release -p arles-app
```

Y se abre la ventana.

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

- Tus datos viven cifrados en `%APPDATA%\mx.telemetryinsight.arlesrelay\`.
- **La llave vive en Windows, no en ARLES.** Si alguien se lleva el archivo de
  la base, no puede leerlo.
- **Si el Administrador de credenciales no está disponible, ARLES no arranca.**
  Nunca se degrada a guardar la llave en texto plano. Lo verificamos el
  07/10/2026: al arrancar, lo primero que hace el programa es pedir el llavero y
  **se queda esperando** hasta que conteste.

Después verás **Inicio**, con la lista de seis pasos. El primero es configurar
los datos de tu empresa.

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
vuelve a correr `npm run build` dentro de `app` y después `cargo run --release
-p arles-app` otra vez. Si sigue, dímelo con lo que diga la consola.

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
