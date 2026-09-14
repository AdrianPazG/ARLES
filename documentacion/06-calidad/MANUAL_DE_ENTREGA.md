# Manual de entrega · Revisión visual

**Para quien va a mirar la aplicación.** No hace falta programar ni instalar
herramientas de desarrollo.

> **Qué se cierra con esto.** La Fase 2 quedó con cuatro cosas sin verificar,
> porque el servidor donde se construye no tiene pantalla: la ventana nativa,
> el motor de macOS, el escalado de Windows y el lector de pantalla. **Esta
> entrega cierra las cuatro** y abre la Fase 3.

**Tiempo:** unos 40 minutos en Windows, unos 25 en Mac.
Se puede partir en dos ratos; no hace falta hacerlo del tirón.

> 📱 **Este mismo manual, como página web:**
> **https://claude.ai/artifact/PTa8K9q6moJC6znzNeUk2D**
>
> Se lee bien en el móvil mientras haces la revisión en el ordenador, lleva una
> cuenta de las capturas que ya tienes y un botón para copiar la plantilla.
> El contenido es el mismo que este archivo.

---

## Índice

1. [Lo que necesitas](#1-lo-que-necesitas)
2. [Descargar los instaladores](#2-descargar-los-instaladores)
3. [Abrirlos: las advertencias son normales](#3-abrirlos-las-advertencias-son-normales)
4. [Llegar al catálogo](#4-llegar-al-catálogo)
5. [Las 15 capturas, una por una](#5-las-15-capturas-una-por-una)
6. [Cómo nombrarlas y ordenarlas](#6-cómo-nombrarlas-y-ordenarlas)
7. [Comparar con las referencias](#7-comparar-con-las-referencias)
8. [El lector de pantalla](#8-el-lector-de-pantalla)
9. [Rellenar la plantilla y enviarlo](#9-rellenar-la-plantilla-y-enviarlo)

---

## 1. Lo que necesitas

| | Por qué |
|---|---|
| Un equipo con **Windows 10 u 11** | Es el motor WebView2, distinto del que usamos para construir |
| Un **Mac** con macOS 11 o posterior | Es WKWebView, el motor de Safari. Dibuja distinto, y ése es el riesgo R-07 |
| Una cuenta de GitHub con acceso al repositorio | Para descargar |

**Si sólo tienes uno de los dos, hazlo igual y dímelo.** Media revisión sirve;
ninguna, no.

---

## 2. Descargar los instaladores

**Ya están compilados. No tienes que construir nada.**

👉 **https://github.com/AdrianPazG/ARLES/actions/runs/34879107021**

1. Abre ese enlace.
2. Baja hasta el final de la página, a la sección **«Artifacts»**.
3. Descarga los dos:

| Artefacto | Qué lleva dentro | Tamaño |
|---|---|---|
| `arles-revision-windows` | un `.exe` instalador | 4,4 MB |
| `arles-revision-macos` | un `.dmg` **universal** (Intel y Apple Silicon) | 11,2 MB |

4. GitHub los entrega dentro de un **`.zip`**. Descomprímelos antes de abrir.

> ⏳ **Caducan el 28 de septiembre de 2026.** Si ya pasó esa fecha, el
> procedimiento para compilar una versión nueva está en
> [REVISION_VISUAL.md](REVISION_VISUAL.md) — son seis clics desde el navegador.

> ❗ **Si «Artifacts» sale vacío, párate y dímelo.** No es normal: la
> compilación comprueba que el instalador existe antes de subirlo.

---

## 3. Abrirlos: las advertencias son normales

**Estas compilaciones no están firmadas.** Los certificados llegan en la Fase 9
(riesgo R-14). Los dos sistemas van a protestar, y hay que saltárselo **a
propósito**.

### Windows

Al abrir el `.exe` sale una pantalla azul:

> *Windows protegió su PC*

Pulsa **«Más información»** → **«Ejecutar de todas formas»**.

### macOS

Al abrir el `.dmg` y arrastrar la aplicación, macOS dirá que no puede verificar
al desarrollador.

**Clic derecho sobre la aplicación → «Abrir»** → confirmar.
Con doble clic normal no deja; con clic derecho sí. Sólo la primera vez.

> Si esto te incomoda, es la reacción correcta: es exactamente lo que sentiría
> un cliente. Por eso los certificados están en el roadmap.

---

## 4. Llegar al catálogo

La aplicación abre en **Inicio**. El catálogo está en la barra lateral,
**abajo del todo y con borde punteado**:

![El enlace al catálogo, señalado](imagenes/ref-01b-enlace-senalado.png)

Está ahí abajo y no parece una sección del producto **a propósito**: no lo es.
Es una herramienta de revisión que no viaja al cliente.

> Ese enlace **sólo existe en esta compilación**. En la que se instalaría en
> casa de un cliente no se pinta, y el catálogo **no va dentro del programa** —
> lo comprueba una verificación automática que inspecciona el paquete
> compilado, no el código.

Al pulsarlo verás cuatro pestañas: **Primitivas · Los cuatro estados · Tabla
virtualizada · Tipografía**.

---

## 5. Las 15 capturas, una por una

Son **9 en Windows** y **6 en Mac**. Cada una tiene su nombre de archivo exacto
— respétalos, así sé qué estoy mirando sin preguntarte.

### En Windows

| # | Nombre del archivo | Qué capturar | Qué mirar mientras |
|---|---|---|---|
| 1 | `W-01-ventana.png` | La ventana recién abierta, **con la barra de título visible**. No recortes | ¿Dice «ARLES RELAY»? ¿Hay icono en la barra de tareas? |
| 2 | `W-02-maximizada.png` | La ventana maximizada | ¿El contenido se reparte, o se queda en una esquina? |
| 3 | `W-03-primitivas.png` | Pestaña «Primitivas», pantalla completa | ¿El botón amarillo se lee bien? ¿El **deshabilitado** se distingue del normal **y aún se puede leer**? |
| 4 | `W-04-estados.png` | Pestaña «Los cuatro estados» | ¿El bloque de «Cargando» respira suavemente, sin parpadear? |
| 5 | `W-05-tabla.png` | Pestaña «Tabla virtualizada» | Desplázate rápido antes de capturar. ¿Va fluido o da tirones? |
| 6 | `W-06-tipografia.png` | Pestaña «Tipografía» | ¿Los números de la columna quedan alineados en vertical? |
| 7 | `W-07-escala-125.png` | Pestaña «Primitivas» al **125 %** | ↓ ver abajo |
| 8 | `W-08-escala-150.png` | Pestaña «Primitivas» al **150 %** | ↓ |
| 9 | `W-09-escala-200.png` | Pestaña «Primitivas» al **200 %** | ↓ |

**Antes de la 1:** arrastra el borde para hacer la ventana pequeña.
**No debe poder bajar de 1120 × 720.** Si baja más, es un fallo — anótalo.

#### Cómo cambiar el escalado (capturas 7, 8 y 9)

Esto es el §22, y es donde más se rompen las interfaces.

1. **Cierra ARLES.**
2. **Configuración → Sistema → Pantalla → Escala.**
3. Ponlo al **125 %**. Windows puede pedir cerrar sesión: hazlo.
4. Abre ARLES, ve a «Primitivas», captura.
5. Repite con **150 %** y con **200 %**.
6. **Devuelve la escala a lo que tenías.**

**Qué buscar:** texto cortado, botones que se salen de su caja, dos cosas
encima de la otra, o barras de desplazamiento donde antes no había.

### En Mac

Las mismas seis primeras, sin el escalado.

| # | Nombre del archivo | Qué capturar |
|---|---|---|
| 1 | `M-01-ventana.png` | La ventana recién abierta, con su barra de título |
| 2 | `M-02-maximizada.png` | La ventana maximizada |
| 3 | `M-03-primitivas.png` | Pestaña «Primitivas» |
| 4 | `M-04-estados.png` | Pestaña «Los cuatro estados» |
| 5 | `M-05-tabla.png` | Pestaña «Tabla virtualizada» |
| 6 | `M-06-tipografia.png` | Pestaña «Tipografía» |

**Lo que más me interesa del Mac**, porque es donde esperamos diferencias:

- El **desplegable «Proveedor»**: macOS dibuja las listas a su manera. Es una
  divergencia esperada, no un fallo — pero quiero verla.
- El **texto claro sobre fondo oscuro**: macOS lo adelgaza. Si se ve
  notablemente más fino que en Windows, hay que ajustarlo.
- Las **sombras** del modal y del menú.

> **Cómo capturar:** Windows `Win + Shift + S`, o la tecla `Impr Pant`.
> Mac `Cmd + Shift + 4` y luego `espacio` para capturar la ventana entera.

---

## 6. Cómo nombrarlas y ordenarlas

Crea una carpeta y mete dentro las capturas con **exactamente** estos nombres:

```
revision-visual/
├── W-01-ventana.png
├── W-02-maximizada.png
├── W-03-primitivas.png
├── W-04-estados.png
├── W-05-tabla.png
├── W-06-tipografia.png
├── W-07-escala-125.png
├── W-08-escala-150.png
├── W-09-escala-200.png
├── M-01-ventana.png
├── M-02-maximizada.png
├── M-03-primitivas.png
├── M-04-estados.png
├── M-05-tabla.png
├── M-06-tipografia.png
└── PLANTILLA-ENTREGA.md     ← rellenada
```

`W` = Windows, `M` = Mac. **Si una captura no la pudiste hacer, no la
inventes: déjala fuera y dilo en la plantilla.** Un hueco declarado es
información; un hueco silencioso es un agujero.

---

## 7. Comparar con las referencias

Aquí está cómo se ve el catálogo **desde el mismo motor que usa Windows**
(Chromium, igual que WebView2), capturado automáticamente desde el repositorio.
**No es cómo «debería» verse en Mac** — ahí el motor es otro, y la diferencia
es justo lo que buscamos.

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

Estas referencias salen de un navegador, no de la aplicación empaquetada. Vas a
ver estas cuatro diferencias y **ninguna es un fallo**:

| En la referencia | En tu pantalla | Por qué |
|---|---|---|
| Abajo a la izquierda pone **`v—`** | Pondrá **`v1.2.0`** | La versión la dice el núcleo de la aplicación. En un navegador no hay núcleo, y el guion es deliberado: se ve, y así el hueco no se disimula |
| **No hay barra de título** | La tendrá | La referencia es una página web; la tuya es una ventana del sistema |
| Barras de desplazamiento distintas | Las de tu sistema | Cada sistema pinta las suyas |
| El borde de la ventana | El de tu sistema | Igual |

**Todo lo demás sí cuenta.** Si algo se ve distinto y no está en esta tabla,
anótalo.

---

## 8. El lector de pantalla

**No hay que ser experto.** Basta con escuchar y decirme si lo que dice tiene
sentido.

### Windows — NVDA

1. Descarga **NVDA** de `nvaccess.org`. Es gratuito y de código abierto.
   Instálalo, o usa la versión portable si prefieres no instalar nada.
2. Arráncalo. Empezará a leer en voz alta lo que tengas enfocado.
3. Abre ARLES y **navega sólo con el tabulador**. Nada de ratón.
4. Escucha.

Para apagarlo: `Insert + Q`.

### Mac — VoiceOver

Ya viene incluido. **`Cmd + F5`** lo enciende y lo apaga. Recorre con el
tabulador.

### Qué escuchar, en las dos

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

🎙️ En vez de captura: **grábalo**, o apúntame en la plantilla lo que oigas mal.
Cualquiera de las dos me sirve.

---

## 9. Rellenar la plantilla y enviarlo

1. Abre **[PLANTILLA-ENTREGA.md](PLANTILLA-ENTREGA.md)**.
2. Cópiala entera y rellénala. Se lee y se escribe en cualquier editor de
   texto — Bloc de notas vale.
3. Mándame **la plantilla rellenada** y **las capturas**, aquí en el chat.

### Cómo rellenarla

- Donde pone `[ ]`, marca con una `x` si es que sí: `[x]`.
- Donde pone `_____`, escribe encima.
- **Si algo no lo hiciste, déjalo sin marcar y explica por qué.** No pasa nada.

### Lo más importante

> **No hace falta que salga todo bien. Al contrario.**
>
> Si todo sale perfecto significa que esta revisión no encontró nada, y las dos
> fases anteriores dicen que eso es improbable: la Fase 2 se cerró con 18
> comprobaciones en verde y después, atacándola a propósito, aparecieron diez
> defectos más — uno de ellos habría tumbado la aplicación con 500 000
> contactos.
>
> Lo que encuentres ahora es lo que no vamos a arrastrar durante siete
> pantallas más.

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

**Lo que sólo tú puedes ver** es lo de las secciones 5 y 8: si en un sistema
operativo de verdad, con su motor de verdad, esto se ve y se oye como debe.

---

## Qué pasa después

1. Leo tu entrega y abro un hallazgo por cada cosa.
2. Arreglo lo que sea de la Fase 2 y lo verifico.
3. Con eso, los cuatro pendientes quedan cerrados y **arranca la entrega 3.1**:
   configuración de empresa y lista de onboarding.

La entrega **3.3** sigue bloqueada por P-09 (marco legal vigente en México),
que necesita abogado y no depende de esto.
