# Revisión visual · por dónde empezar

**La Fase 2 quedó con cuatro cosas sin verificar**, porque el servidor donde se
construye no tiene pantalla: la ventana nativa, el motor de macOS, el escalado
de Windows y el lector de pantalla. Cerrarlas es la puerta de la Fase 3.

Hay **un manual por sistema**, porque las instrucciones se separan: SmartScreen
no es Gatekeeper, NVDA no es VoiceOver, y el escalado sólo existe en Windows.
Mezclados obligaban a saltarse media página.

---

## Elige el tuyo

### 🪟 Windows

| Formato | Manual | Plantilla |
|---|---|---|
| **PDF / Word** | [MANUAL_WINDOWS.pdf](MANUAL_WINDOWS.pdf) | [PLANTILLA-WINDOWS.docx](PLANTILLA-WINDOWS.docx) |
| Web | https://claude.ai/artifact/RAbRXPpaPWAzda6hX4f8w4 | — |
| Markdown | [MANUAL_WINDOWS.md](MANUAL_WINDOWS.md) | [PLANTILLA-WINDOWS.md](PLANTILLA-WINDOWS.md) |

**9 capturas · unos 40 minutos.** Cierra **tres** de los cuatro pendientes: la
ventana nativa, el escalado del sistema y el lector de pantalla. Lo más valioso
de aquí es el escalado a 125, 150 y 200 %: es donde más se rompen las
interfaces (§22).

### 🍎 Mac

| Formato | Manual | Plantilla |
|---|---|---|
| **PDF / Word** | [MANUAL_MAC.pdf](MANUAL_MAC.pdf) | [PLANTILLA-MAC.docx](PLANTILLA-MAC.docx) |
| Web | https://claude.ai/artifact/9bDahVQ69DUPBf1J8QJx5Z | — |
| Markdown | [MANUAL_MAC.md](MANUAL_MAC.md) | [PLANTILLA-MAC.md](PLANTILLA-MAC.md) |

**6 capturas · unos 25 minutos.** Cierra el cuarto pendiente, que es el más
incierto: **el motor de macOS**. ARLES se dibuja con WKWebView, el de Safari,
que no es el mismo con el que construimos ni el que usa Windows. Es el riesgo
**R-07**.

---

## Si tienes los dos

**Haz los dos, en cualquier orden.** Son revisiones independientes y cada una
tiene su propia plantilla: mándame las dos.

Empezar por Windows tiene una ventaja pequeña: llegas al Mac con las
diferencias frescas, y la sección 6 del manual de Mac es justamente comparar.

## Si sólo tienes uno

**Hazlo igual y dímelo.** Media revisión sirve; ninguna, no. Los dos manuales
funcionan por separado — cada uno lleva sus propias instrucciones de descarga,
sus capturas de referencia y su plantilla.

---

## Qué formato usar

| | Cuándo |
|---|---|
| **PDF** | Para leer el manual. Lleva las capturas de referencia dentro y funciona sin conexión |
| **Word (.docx)** | Para rellenar la plantilla. Casillas ☐ y renglones donde escribir |
| **Web** | Para leer en el móvil mientras trabajas en el ordenador. Lleva la cuenta de las capturas |
| **Markdown** | Es el original. Los otros tres se generan de él, no se editan a mano |

El PDF y el .docx llevan al lado la huella SHA-256 de su Markdown, y una
comprobación falla si alguien corrige el texto y reparte un archivo viejo.

---

## Lo común a los dos

| | |
|---|---|
| **Los instaladores ya están compilados** | No hay que construir nada: [esta compilación](https://github.com/AdrianPazG/ARLES/actions/runs/34879107021), sección «Artifacts». Caducan el **28 de septiembre de 2026** |
| **No están firmados** | Los dos sistemas van a protestar y hay que saltárselo a propósito. Cada manual explica cómo, en su paso 03 |
| **Van con el catálogo dentro** | Una pantalla con todas las piezas, que **no** viaja a la compilación de cliente |
| **No hace falta instalar nada** | Ni Rust, ni Node, ni herramientas de desarrollo |

### Los nombres de las capturas no se mezclan

`W-…` son de Windows, `M-…` de Mac. Así, si me llegan las dos entregas a la
vez, sé qué estoy mirando sin preguntarte.

### Las capturas de referencia

Los dos manuales llevan las mismas: el catálogo capturado **desde Chromium**,
que es el motor que hay debajo de WebView2. En Windows sirven para confirmar
que se ve igual; **en Mac sirven para lo contrario** — para que me digas en qué
se separa.

Se generan con un comando (`app/pruebas/sondas/referencias.mjs`), así que no
envejecen calladas.

---

## Lo que yo ya comprobé — no gastes tiempo ahí

| Ya verificado | Cómo |
|---|---|
| Contraste de todos los colores | Calculado, no a ojo. En cada compilación |
| La tabla virtualiza de verdad | 18 nodos para 5 000 filas — se midió porque **estaba mal** |
| El foco no se pierde ni se escapa del modal | Sonda de teclado automatizada |
| Ninguna parada de tabulación sin su anillo | 15 paradas, todas con señal visible |
| La política de seguridad no necesita relajarse | Recorriendo la aplicación bajo la política real |
| El catálogo no entra en la compilación de cliente | Inspeccionando el paquete compilado |

Todo eso está medido **en Chromium**. Que el contraste dé 4,5:1 calculado no
garantiza que el texto se **perciba** igual cuando WKWebView lo suaviza
distinto — por eso la sección 6.2 del manual de Mac no es opcional.

---

## Qué pasa después

1. Leo tu entrega y abro un hallazgo por cada cosa.
2. Arreglo lo que sea de la Fase 2 y lo verifico.
3. Con eso los cuatro pendientes quedan cerrados y **arranca la entrega 3.1**:
   configuración de empresa y lista de onboarding.

La entrega **3.3** sigue bloqueada por P-09 (marco legal vigente en México),
que necesita abogado y no depende de esto.

---

**¿Necesitas compilar una versión nueva?** Seis clics desde el navegador:
[REVISION_VISUAL.md](REVISION_VISUAL.md).
