# Manual de revisión · **Mac**

**Para quien va a mirar la aplicación en un Mac.** No hace falta programar ni
instalar herramientas de desarrollo.

> **Qué cierra esta revisión.** El pendiente más incierto de la Fase 2: **el
> motor de macOS**. ARLES se dibuja aquí con **WKWebView**, el motor de Safari,
> que no es el mismo con el que construimos ni el que usa Windows. Es el
> riesgo **R-07**, y es la única forma de cerrarlo.
>
> Los otros tres pendientes se cierran en [el manual de Windows](MANUAL_WINDOWS.md).

**Tiempo:** unos 25 minutos.
**Vas a hacer 6 capturas**, todas con nombre `M-…`.

> 📱 **Este mismo manual, como página web:**
> **https://claude.ai/artifact/9bDahVQ69DUPBf1J8QJx5Z**
>
> Se lee bien en el móvil mientras trabajas en el Mac, lleva la cuenta de las
> capturas y un botón para copiar la plantilla.

---

## Índice

1. [Lo que necesitas](#1-lo-que-necesitas)
2. [Descargar el instalador](#2-descargar-el-instalador)
3. [Abrirlo: Gatekeeper y el clic derecho](#3-abrirlo-gatekeeper-y-el-clic-derecho)
4. [Llegar al catálogo](#4-llegar-al-catálogo)
5. [Las 6 capturas](#5-las-6-capturas)
6. [Lo que de verdad busco en el Mac](#6-lo-que-de-verdad-busco-en-el-mac)
7. [Comparar con las referencias](#7-comparar-con-las-referencias)
8. [VoiceOver, el lector de pantalla](#8-voiceover-el-lector-de-pantalla) — **que hable español**
9. [Ordenar y enviar](#9-ordenar-y-enviar)

---

## 1. Lo que necesitas

| | |
|---|---|
| Un **Mac con macOS 11** o posterior | Da igual Intel o Apple Silicon: el instalador es **universal** y abre en los dos |
| Acceso al repositorio en GitHub | Para descargar |

**No hace falta** instalar Rust, Node, ni ninguna herramienta de desarrollo.

---

## 2. Descargar el instalador

👉 **https://github.com/AdrianPazG/ARLES/actions/runs/34879107021**

1. Abre ese enlace.
2. Baja hasta el final de la página, a la sección **«Artifacts»**.
3. Descarga **`arles-revision-macos`** — son 11,2 MB.
4. GitHub te lo da dentro de un **`.zip`**. Descomprímelo: dentro va un `.dmg`.

> Pesa más que el de Windows porque lleva las dos arquitecturas dentro. Así no
> hay que preguntarte qué Mac tienes.

> ⏳ **Caduca el 28 de septiembre de 2026.** Si ya pasó esa fecha, compilar una
> versión nueva son seis clics desde el navegador:
> [REVISION_VISUAL.md](REVISION_VISUAL.md).

> ❗ **Si «Artifacts» sale vacío, párate y dímelo.** No es normal: la
> compilación comprueba que el instalador existe antes de subirlo.

---

## 3. Abrirlo: Gatekeeper y el clic derecho

**Esta compilación no está firmada ni notarizada.** Los certificados llegan en
la Fase 9 (riesgo R-14). macOS va a negarse, y hay que rodearlo **a propósito**.

1. Abre el `.dmg` y arrastra ARLES a **Aplicaciones**.
2. **Clic derecho** sobre la aplicación → **«Abrir»**.
3. Confirma en el diálogo.

**Con doble clic normal no te va a dejar. Con clic derecho → Abrir, sí.**
Sólo hace falta la primera vez.

> Si macOS Sequoia o posterior tampoco deja así: **Ajustes del Sistema →
> Privacidad y seguridad**, baja hasta el aviso sobre ARLES y pulsa
> **«Abrir de todos modos»**.

> Si esto te incomoda, es la reacción correcta: es exactamente lo que sentiría
> un cliente. Por eso la notarización está en el roadmap.

---

## 4. Llegar al catálogo

La aplicación abre en **Inicio**. El catálogo está en la barra lateral,
**abajo del todo y con borde punteado**:

![El enlace al catálogo, señalado](imagenes/ref-01b-enlace-senalado.png)

Está ahí abajo y no parece una sección del producto **a propósito**: no lo es.
Es una herramienta de revisión que no viaja al cliente.

Al pulsarlo verás cuatro pestañas: **Primitivas · Los cuatro estados · Tabla
virtualizada · Tipografía**.

> Ese enlace **sólo existe en esta compilación**. En la que se instalaría en
> casa de un cliente no se pinta, y el catálogo no va dentro del programa — lo
> comprueba una verificación automática que inspecciona el paquete compilado.

---

## 5. Las 6 capturas

**Antes de la primera:** arrastra el borde de la ventana para hacerla pequeña.
**No debe poder bajar de 1120 × 720.** Si baja más, es un fallo — anótalo.

| # | Nombre del archivo | Qué capturar | Qué mirar mientras |
|---|---|---|---|
| 1 | `M-01-ventana.png` | La ventana recién abierta, **con la barra de título visible**. No recortes | ¿Dice «ARLES RELAY»? ¿Hay icono en el Dock? **¿Se ve nítido o borroso?** |
| 2 | `M-02-maximizada.png` | La ventana maximizada | ¿El contenido se reparte, o se queda en una esquina? |
| 3 | `M-03-primitivas.png` | Pestaña «Primitivas», pantalla completa | ¿El botón amarillo se lee bien? ¿El **deshabilitado** se distingue del normal **y aún se puede leer**? |
| 4 | `M-04-estados.png` | Pestaña «Los cuatro estados» | ¿El bloque de «Cargando» respira suavemente, sin parpadear? |
| 5 | `M-05-tabla.png` | Pestaña «Tabla virtualizada». Desplázate rápido antes de capturar | ¿Va fluido o da tirones? |
| 6 | `M-06-tipografia.png` | Pestaña «Tipografía» | ¿Los números de la columna quedan alineados en vertical? |

> **Cómo capturar:** `Cmd + Shift + 4` y luego **`espacio`** — el cursor se
> vuelve una cámara y captura **la ventana entera con su sombra**. Es la forma
> más limpia.
> `Cmd + Shift + 5` te da más opciones, incluida grabar vídeo.

**No hay escalado que probar.** macOS no expone escalas al 125/150/200 % como
Windows: eso es sólo del manual de Windows. Aquí son seis capturas y ya.

---

## 6. Lo que de verdad busco en el Mac

**Esto es lo importante de esta revisión.** Las seis capturas son el registro;
esto es lo que hay que mirar con atención, porque es donde WKWebView se separa
de Chromium.

### 6.1 El desplegable «Proveedor»

En la pestaña **«Primitivas»**, campo **«Proveedor»**.

macOS dibuja las listas desplegables **a su manera**, y no se puede evitar del
todo. **Es una divergencia esperada, no un fallo** — pero necesito verla para
decidir si la aceptamos o si hay que construir un desplegable propio.

Ábrelo y mira: ¿se parece al de Windows, o es claramente un control del
sistema?

### 6.2 El grosor del texto claro

Todo ARLES es texto claro sobre fondo oscuro. **macOS adelgaza ese texto** —
suaviza distinto que Windows.

**La pregunta:** ¿se ve notablemente **más fino** que en Windows, hasta
costarte leerlo? Si sí, hay que ajustar el peso tipográfico y quiero saberlo
ahora, no en la Fase 9.

Si no tienes un Windows al lado para comparar, compara contra las capturas de
referencia de la sección 7.

### 6.3 Las sombras

El **modal** y el **menú** llevan sombra. macOS las compone distinto.

Abre el modal (botón **«Modal destructivo»** en «Primitivas») y mira si la
sombra se ve mucho más difusa, más dura, o directamente no está.

### 6.4 El icono en el Dock

macOS pinta el icono a un tamaño grande y con esquinas propias.
¿Se ve nítido? ¿Se ve con la forma correcta?

---

## 7. Comparar con las referencias

Así se ve el catálogo **desde Chromium**, que es el motor de Windows —
capturado automáticamente desde el repositorio.

> ⚠️ **Esto NO es cómo «debería» verse en tu Mac.** Es el punto de partida de
> la comparación. Aquí el motor es otro, y **las diferencias son justo lo que
> estamos buscando**. Tu trabajo no es confirmar que se ve igual: es decirme
> **en qué se separa**.

### Primitivas

![Referencia · Primitivas](imagenes/ref-02-primitivas.png)

### Los cuatro estados

![Referencia · Los cuatro estados](imagenes/ref-03-estados.png)

### Tabla virtualizada

![Referencia · Tabla](imagenes/ref-04-tabla.png)

### Tipografía

![Referencia · Tipografía](imagenes/ref-05-tipografia.png)

### Modal

![Referencia · Modal](imagenes/ref-06-modal.png)

### ⚠️ Diferencias que SON normales — no las reportes

La referencia sale de un navegador, no de la aplicación empaquetada. Vas a ver
estas cuatro diferencias y **ninguna es un fallo**:

| En la referencia | En tu pantalla | Por qué |
|---|---|---|
| Abajo a la izquierda pone **`v—`** | Pondrá **`v1.2.0`** | La versión la dice el núcleo de la aplicación. En un navegador no hay núcleo, y el guion es deliberado: se ve, y así el hueco no se disimula |
| **No hay barra de título** | La tendrá, con los tres botones de macOS | La referencia es una página web; la tuya es una ventana del sistema |
| Barras de desplazamiento distintas | Las de macOS, que además se esconden solas | Cada sistema pinta las suyas |
| El borde y las esquinas de la ventana | Los de macOS | Igual |

**Todo lo demás sí cuenta** — y en el Mac, todo lo demás es precisamente lo que
he venido a buscar.

---

## 8. VoiceOver, el lector de pantalla

**No hay que ser experto.** Basta con escuchar y decirme si lo que dice tiene
sentido. **Ya viene incluido en el Mac**, no hay que instalar nada.

1. **`Cmd + F5`** lo enciende. (En Macs con Touch ID, puede ser pulsar tres
   veces el botón de Touch ID.)
2. Abre ARLES, ve al catálogo, y **navega sólo con el tabulador**. Nada de ratón.
3. Escucha.

**`Cmd + F5` otra vez lo apaga.**

> Si el tabulador no salta entre todos los controles, revisa
> **Ajustes del Sistema → Teclado → «Navegación por teclado»** y actívalo.
> Es un ajuste de macOS que viene apagado, y sin él el tabulador sólo pasa por
> los campos de texto.

### 8.1 Que hable en español

VoiceOver usa el idioma del sistema, así que **si tu Mac está en español no
tienes que hacer nada**. Si lo tienes en inglés dirá *«button»*, *«tab»*,
*«row 40 of 5001»*, y la tabla de abajo te pide escuchar «botón», «pestaña»,
«fila 40 de 5001»: no podrías comprobar lo que hay que comprobar.

Para cambiarlo sin tocar el idioma de todo el Mac:

1. **`Cmd + F8`** abre la **Utilidad de Configuración de VoiceOver**.
2. **Voz → Voz predeterminada**.
3. Elige una voz en español — **Paulina** o **Juan** son las mexicanas.
   Si no aparecen: **Personalizar…** y descárgalas ahí.

> Lo que hay que juzgar es **qué** dice, no cómo suena. Cualquier voz en
> español sirve.

### 8.2 Qué escuchar

| Al llegar a | Debe decir algo como |
|---|---|
| El logotipo | *«ARLES RELAY, imagen»* — una sola cosa, no «ARLES» y «RELAY» por separado |
| Un botón | Su texto y la palabra «botón» |
| El campo con error | El texto del error **al entrar**, no sólo al salir |
| Una pestaña | *«pestaña, seleccionada»* o *«2 de 4»* |
| La tabla | **«fila 40 de 5001»** o parecido — el total real, no lo que se ve |

**Reporta esto, aunque parezca menor:**

- Algo que **no dice nada** al llegar.
- Algo que dice **«botón botón»** o repite.
- Un sitio donde **te pierdes** y no sabes dónde estás.
- La tabla anunciando **20 filas** en vez de las 5 001 reales.

🎙️ En vez de captura: **graba la pantalla** con `Cmd + Shift + 5`, o apúntame
en la plantilla lo que oigas mal. Cualquiera de las dos me sirve.

---

## 9. Ordenar y enviar

Crea una carpeta con **exactamente** estos nombres:

```
revision-mac/
├── M-01-ventana.png
├── M-02-maximizada.png
├── M-03-primitivas.png
├── M-04-estados.png
├── M-05-tabla.png
├── M-06-tipografia.png
└── PLANTILLA-MAC.md     ← rellenada
```

**Si una captura no la pudiste hacer, no la inventes: déjala fuera y dilo en la
plantilla.** Un hueco declarado es información; un hueco silencioso es un
agujero.

### La plantilla

1. Abre **[PLANTILLA-MAC.md](PLANTILLA-MAC.md)**.
2. Cópiala y rellénala. Se lee y se escribe en cualquier editor de texto —
   TextEdit vale, en modo texto plano.
3. Donde pone `[ ]`, marca con una `x` si es que sí: `[x]`.
   Donde pone `_____`, escribe encima.
4. Mándame **la plantilla rellenada** y **las capturas**, aquí en el chat.

> **No hace falta que salga todo bien. Al contrario.**
>
> En el Mac es donde **espero** encontrar diferencias: es el motor que no
> controlamos. Si me dices que todo se ve idéntico a Windows, la sospecha es
> que no miramos bastante, no que esté perfecto.

**Escribe lo que te chirríe aunque no sepas explicarlo.** *«Se ve raro»* es
información útil; el motivo lo busco yo.

---

## Lo que yo ya comprobé — no gastes tiempo aquí

| Ya verificado | Cómo |
|---|---|
| Contraste de todos los colores | Calculado, no a ojo. En cada compilación |
| La tabla virtualiza de verdad | 18 nodos para 5 000 filas — se midió porque **estaba mal** |
| El foco no se pierde ni se escapa del modal | Sonda de teclado automatizada |
| Ninguna parada de tabulación sin su anillo | 15 paradas, todas con señal visible |
| La política de seguridad no necesita relajarse | Recorriendo la aplicación bajo la política real |
| El catálogo no entra en la compilación de cliente | Inspeccionando el paquete compilado |

**Ojo:** todo eso está medido **en Chromium**. Que el contraste dé 4,5:1
calculado no garantiza que el texto se **perciba** igual cuando WKWebView lo
suaviza distinto. Por eso el punto 6.2 no es opcional.

---

## ¿Tienes también un Windows?

Entonces hace falta la otra mitad: **[MANUAL_WINDOWS.md](MANUAL_WINDOWS.md)**.
Son 40 minutos más y cierra los otros tres pendientes, incluido el escalado del
sistema, que es donde más se rompen las interfaces.

Si no tienes Windows, **manda sólo esto**. Media revisión sirve; ninguna, no.
