# Fase 2 · Design System

**Estado:** ✅ cerrada · **Fecha:** 2026-09-11
**Validación:** 12/12 comprobaciones, 0 omitidas — `validar.py --fase 2`

> **Objetivo.** Convertir el sistema de diseño escrito en la Fase 0 en
> **componentes que lo hacen cumplir**. Una regla de diseño en un documento se
> erosiona en la tercera pantalla; metida dentro de la primitiva que todo el
> mundo usa, se defiende sola.

---

## 1. Qué se construyó

### `app/src/design/componentes/` — once primitivas y cuatro estados

| Componente | Qué defiende |
|---|---|
| `ABoton` | Cuatro variantes. El texto del primario es consecuencia de la variante, no una propiedad: sobre el acento sólo hay un color legible |
| `AEntrada` | El error va **debajo, con icono y texto**, enlazado por `aria-describedby` |
| `ASelector` | `<select>` nativo: la navegación por teclado y el lector de pantalla salen gratis |
| `AInsignia` | Relleno sólido con texto oscuro, y el icono se **deriva del tono**: no hay forma de construir una sin él |
| `AAviso` | Donde ARLES no sabe algo, lo declara en el sitio donde aparece la duda |
| `AModal` | `<dialog>` nativo. En una acción destructiva, `Esc` y el clic fuera **no cierran** |
| `AMenu` | Patrón ARIA de menú de botón; `Esc` devuelve el foco al disparador |
| `APestanas` | Tabindex móvil: sólo la activa está en el orden de tabulación |
| `ATabla` | Virtualizada. Rejilla ARIA con `aria-rowcount` real |
| `AIcono` | Conjunto cerrado y tipado: un nombre que no existe no compila |
| `ALogotipo` | «ARLES RELAY» en Mont Black, sin isotipo y **sin el numeral** |
| `EstadoVacio` · `EstadoCargando` · `EstadoError` · `EstadoExito` | Los cuatro estados del §97 |

`EstadoError` toma `que`, `como` y `salvo` como **tres propiedades
obligatorias**: el compilador rechaza un error al que le falte una. Es el mismo
contrato que `ErrorIpc` en Rust y que `resolverError()` en el frontend, de
extremo a extremo.

### `app/src/design/tipografia.css` — Mont incrustada

Cuatro cortes en `.woff2`: Regular 400, SemiBold 600, Bold 700 y Black 900.
180 KB. Sin cursivas: ningún componente las usa.

### `app/src/design/CatalogoDelSistema.vue` — el catálogo

Cada primitiva en sus cuatro estados, en cuatro pestañas. Sólo se registra en
el router fuera de producción: no viaja en el bundle del cliente.

Existe por un motivo concreto. La Fase 1 cerró con **«la aplicación abriéndose
en una ventana real»** en la lista de lo no verificado. Este catálogo es lo que
se abre en Windows y en macOS para cerrar ese punto —y es también donde se
encontraron los dos defectos de contraste de §4.

### Tokens nuevos

| Token | Valor | Por qué |
|---|---|---|
| `--arles-accent-hover` | `#FFD83A` | El hover del primario tiene que pasar AA con el texto oscuro encima. 12.83:1 |
| `--arles-danger-hover` | `#F4867A` | Lo mismo para el botón de peligro. 7.24:1 |
| `--arles-text-disabled` | `#9FB4BF` | Ver §4 |
| `--arles-control-height` · `-compact` | 40 / 32 px | Alturas de control |
| `--arles-row-height-compact` · `-comfortable` | 36 / 44 px | Alturas de fila |
| `--arles-modal-max-width` · `--arles-menu-min-width` | 640 / 200 px | Contenedores flotantes |
| `--arles-shadow-modal` · `-menu` · `--arles-backdrop` | — | Las dos únicas sombras del sistema, y el fondo del modal |

Los tres primeros llevan **contrato de contraste verificado en CI**. Ninguno se
escribió a mano: salen de `tokens.json` y el CSS se regenera.

---

## 2. Decisiones tomadas durante la fase

### Qué cuenta como «píxel suelto»

El §17 dice que ningún componente contiene un píxel literal. Al pie de la letra
eso obliga a inventar un token por cada filete de 1 px, y cuarenta tokens de
borde no son más rigor: son menos legibilidad.

La línea quedó en **4 px**. De ahí arriba es escala —altura de control, de fila,
ancho de modal, geometría de sombra— y va a token; de 1 a 3 px es detalle de
dibujo del propio componente y se escribe. Color y duración **no tienen
excepción**. Detalle en [DESIGN_SYSTEM §2.1](../05-diseno/DESIGN_SYSTEM.md).

### La tabla no es un `<table>`

Un `<table>` real no se puede virtualizar sin romper su modelo de cajas: las
filas ausentes se llevan por delante el ancho de las columnas. Se usa una
rejilla con roles ARIA explícitos, que es el patrón que la especificación
prevé para este caso, con `aria-rowcount` declarando el total real. El lector
de pantalla anuncia «fila 40 231 de 500 000» aunque en el DOM existan veinte.

### El catálogo es una pantalla, no una primitiva

La regla de «cero medidas sueltas» se aplica a `componentes/`, no al catálogo.
Una primitiva se reutiliza en toda la aplicación y su escala es una decisión de
sistema; una pantalla compone primitivas y elige su propio reparto. Mezclar las
dos cosas convierte la regla en ruido y acaba con alguien desactivándola.

---

## 3. Lo que la tipografía obligó a corregir

**No existe un corte Medium 500.** La escala del `TIPOGRAFIA.md` pedía Medium
500 para las cifras. El kit no lo tiene. Medidos los grosores reales sobre el
asta de la «I», los cortes saltan de Regular (87 por mil) a SemiBold (115), sin
nada intermedio. Las cifras usan **Regular 400 con figuras tabulares**, que
además es lo que pide el §6 del propio documento: texto claro sobre fondo
oscuro pesa más de lo que aparenta.

**El `usWeightClass` del kit está desplazado.** Leído en los TTF:

| Archivo | Declara | Asta de la «I» | Peso en ARLES |
|---|---|---|---|
| `Mont-Light` | 400 | 50 | — |
| `Mont-Regular` | **600** | 87 | **400** |
| `Mont-SemiBold` | **700** | 115 | **600** |
| `Mont-Bold` | **800** | 152 | **700** |
| `Mont-Black` | **950** | 238 | **900** |

Los archivos son lo que su nombre dice —el grosor medido crece monótono y en
orden—; lo que no es de fiar es su metadato. Por eso **cada `@font-face` fija su
`font-weight` explícitamente**, y hay una comprobación que cuenta caras contra
pesos declarados. Es también otra señal de que el kit no es una entrega
comercial de Fontfabric, que refuerza P-01.

**Las figuras tabulares sí hacían falta.** El `TIPOGRAFIA.md` pedía verificarlo.
Los cuatro cortes declaran `tnum` en su GSUB. Y medido en el navegador a 16 px:
«1240» ocupa 36.61 px con figuras tabulares y 32.27 sin ellas; «1102», 36.61
frente a 28.95. Casi ocho píxeles de diferencia entre dos cifras de cuatro
dígitos — suficiente para que una columna de miles no alinee.

---

## 4. Lo que sólo apareció al mirarlo

Las primitivas pasaban tipos, lint y tests. Los dos defectos siguientes
aparecieron al **abrir el catálogo en un navegador y medir los colores
renderizados**, no antes.

**Un botón primario deshabilitado daba 1.89:1. Un secundario ocupado, 3.56:1.**
La causa era `opacity: 0.45` sobre el botón entero, que es el patrón por defecto
en casi cualquier interfaz. WCAG 1.4.3 exime a los controles deshabilitados del
mínimo de contraste, así que **ninguna herramienta automática lo habría
marcado**. Pero un botón cuyo texto no se lee no comunica *qué* está
deshabilitado, que es lo contrario de lo que pide el §95.

Se corrigió pintando el estado en vez de atenuar el elemento: relleno neutro y
`--arles-text-disabled` —3.64:1, ahora verificado en CI—, más el cursor como
segunda señal. Y **«ocupado» dejó de atenuarse del todo**: no está deshabilitado
por una regla de negocio, está trabajando, y el usuario lo está mirando
precisamente entonces. Pasó de 3.56:1 a 11.8:1.

La lección, que es la misma de la Fase 1 con otra ropa: **una regla que la
norma exime no deja de importar**. La exención dice que no te van a suspender
el examen, no que el usuario vaya a poder leerlo.

---

## 5. Qué se verificó, y cómo

**12 comprobaciones automáticas, 0 omitidas** (`validar.py --fase 2`), más 17
tests de componente. Las siete comprobaciones nuevas **se probaron rompiéndolas
a propósito** antes de darlas por buenas.

| Comprobación | Cómo se rompió para probarla |
|---|---|
| Los cuatro cortes de Mont están incrustados | Borrando un `.woff2` |
| Cada `@font-face` declara su `font-weight` | Quitando el peso de Bold |
| Ninguna primitiva contiene un color literal | Escribiendo `#FCCC0C` en `ABoton` |
| …una duración literal | Escribiendo `120ms` |
| …una medida suelta | Escribiendo `min-height: 40px` |
| Todo token que se usa está definido | Escribiendo `var(--arles-bg-deeep)` |
| Nadie quita el anillo de foco sin sustituto | Añadiendo `outline: none` |
| Se respeta `prefers-reduced-motion` | Ver abajo |

La última encontró la trampa de siempre. La primera versión buscaba la cadena
`prefers-reduced-motion` en `base.css`, y el cebo —renombrarla a
`prefers-reduced-motionXX`— **la seguía pasando**: es exactamente la búsqueda
de texto que la Fase 1 ya había encontrado en la comprobación de eslint. Se
sustituyó por una que exige la regla `@media` completa **y** que apague las dos
cosas, `animation-duration` y `transition-duration`: una que sólo neutralice
`transition` deja corriendo la animación del esqueleto de carga.

### Los tests de componente

17 tests, y cada `describe` nombra la regla que defiende. Tres se comprobaron
rompiendo el componente: quitarle el icono a la insignia, dejar que `Esc` cierre
un modal destructivo y poner las tres pestañas en el orden de tabulación. Los
tres hicieron fallar los tests correspondientes.

No comprueban que las primitivas «se vean bien» —eso lo decide una persona
mirando el catálogo—, sino que **las reglas que se pueden romper en silencio
siguen en pie**.

---

## 6. Qué NO se verificó

| Sin verificar | Motivo | Cuándo |
|---|---|---|
| **La aplicación en una ventana de escritorio real** | El catálogo se revisó en Chromium sobre Linux, que es el motor de WebView2 pero no WebView2. Sigue sin verse una ventana nativa | Fase 3, en un equipo con escritorio |
| **WKWebView (macOS)** | El motor de Safari difiere en suavizado, `appearance` del `<select>` y `<dialog>`. Es R-07, el coste aceptado de ADR-0001 | Fase 3, pantalla por pantalla |
| **Escalado de Windows al 125–200 %** (§22) | Necesita Windows real | Fase 3 |
| **La modalidad real del `<dialog>`** | jsdom no la simula, y los tests no la afirman. El relleno de `entorno.ts` cubre abrir y cerrar, nada más | Con la revisión en ventana real |
| **500 000 filas en la tabla** | El catálogo carga 5 000. La virtualización está, el presupuesto no se ha medido | Fase 4 |
| **Lector de pantalla real** | Los roles y atributos están puestos y probados; NVDA y VoiceOver no se han pasado por encima | Fase 3 |

Y la limitación de método que se arrastra: el validador comprueba **que las
reglas del sistema siguen aplicadas**, no que el resultado sea bonito ni
utilizable. Eso lo decide una persona con el catálogo delante.

---

## 7. Pendiente

| Qué | Dónde |
|---|---|
| Licencia de Mont para distribuir la fuente incrustada | Puerta antes de la demo — D-5, P-01. La fuente ya está en el árbol de fuentes, **no en ningún instalador** |
| Revisión del catálogo en Windows y macOS | Fase 3 |
| Densidad cómoda de tabla como preferencia del usuario | Existe como propiedad; falta dónde elegirla — Fase 6 |

---

## 8. Cómo reproducir esta validación

```bash
npm --prefix app ci
python3 herramientas/validar/validar.py --fase 2

# Y para verlo con los ojos:
npm --prefix app run dev    # luego http://localhost:1420/#/catalogo
```

Salida esperada: **12 pasan, 0 fallan, 0 omitidos**.
