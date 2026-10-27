# Design System

**Proyecto:** ARLES RELAY I · v1.2.0
**Depende de:** [`COLOR_SYSTEM.md`](COLOR_SYSTEM.md) · [`TIPOGRAFIA.md`](TIPOGRAFIA.md)
**Última revisión contra el código:** 2026-10-27
**Lo vigila:** `validar.py --fase 2`, comprobación
`el_sistema_de_diseno_dice_lo_que_hay`

---

## 0 · Qué es este archivo, y qué no

Es **el inventario y las decisiones**: qué piezas existen, qué tokens hay, y por
qué cada cosa se decidió así. Es lo que se lee antes de dibujar una pantalla
nueva.

**No es la referencia de cada componente.** Esa vive en la cabecera de cada
archivo, al lado del código, porque ahí no se puede desalinear. Para saber qué
hace exactamente `ATabla`, se abre `ATabla.vue`: su cabecera explica por qué no
es un `<table>` y qué pasa con el virtualizador.

> **Este archivo se reescribió el 27/10/2026 con el código delante, y tenía
> cinco afirmaciones falsas.** Ninguna de las cuatro auditorías del §140 las
> encontró, porque las auditorías miden el producto y esto es un documento. Van
> listadas en el §14, cada una con lo que decía y lo que hay.
>
> Es el patrón número 4 de [`RELEVO.md`](../00-ANTES-DE-EMPEZAR/RELEVO.md): **un
> documento que afirma de más**. Y es el más barato de los seis de cometer,
> porque un documento no falla: se cree. Por eso ahora hay una comprobación que
> compara este archivo con el código — el §13.

**Regla de este archivo, que la comprobación impone:** aquí no se escribe el
nombre de un token ni de un componente que no exista. Ni como ejemplo de lo que
no hay. La comprobación lee los nombres de este archivo y exige que cada uno
esté definido, así que un ejemplo inventado la pondría en rojo sobre código
correcto — el defecto de la comprobación que se caza a sí misma, que este
proyecto lleva nueve veces cometiendo.

---

## 1 · Principios

**Es una aplicación de escritorio, no una web.** De 1366×768 a 4K, con escalado
de Windows del 100 % al 200 % (§22). Ratón, trackpad y teclado. Nada de patrones
móviles: sin menú hamburguesa, sin gestos, sin diseño adaptable a pantallas de
teléfono (§4).

> **El escalado del sistema no encoge los píxeles CSS: encoge la ventana.** Un
> portátil de 1920×1080 al 200 % le da al navegador **960×540**. Lo encontró la
> auditoría D del §140 el 07/10/2026, y por eso ésa es una de las cuatro
> ventanas que mide `sonda:auditoria`. Hasta entonces el §22 estaba escrito
> desde la Fase 0 y no se medía.

**Densidad alta.** El usuario pasa horas con tablas de contactos. Las interfaces
espaciadas pensadas para móvil obligan a desplazarse tres veces para ver lo que
cabría de una.

**Tokens, siempre.** Ningún componente contiene un hex, un píxel suelto ni una
duración literal (§17). La línea exacta está en el §2.1.

**Sobrio.** Tecnológico y premium, no decorado. Cada elemento visible justifica
su presencia.

**Dos temas, y los dos se verifican.** El oscuro es el de origen; el claro es su
traducción medida, no una inversión automática. Ver el §11.

---

## 2 · Rejilla y espaciado

Base de **4 px**, ocho pasos:

```css
--arles-space-1:  4px;    --arles-space-5: 24px;
--arles-space-2:  8px;    --arles-space-6: 32px;
--arles-space-3: 12px;    --arles-space-7: 48px;
--arles-space-4: 16px;    --arles-space-8: 64px;
```

Y una proporción, que se usa para repartir dos columnas sin inventar fracciones:

```css
--arles-aureo:    1.618;
--arles-aureo-fr: 1.618fr;
```

### 2.1 Qué cuenta como «píxel suelto» — precisado en la Fase 2

El principio del §1 dice que ningún componente contiene un píxel literal. Al
implementar las primitivas quedó claro que eso, tomado al pie de la letra,
obliga a inventar un token para cada filete de 1 px, y un sistema con cuarenta
tokens de borde no es más riguroso: es menos legible.

La línea que se trazó, y que el validador hace cumplir:

| Medida | Regla | Por qué |
|---|---|---|
| **≥ 4 px** | **Token obligatorio** | Es una decisión de escala: altura de control, altura de fila, ancho de un modal, geometría de una sombra. Cambiarla afecta a toda la aplicación |
| 1–3 px | Se escribe literal | Filetes, desplazamientos de borde y compensaciones ópticas. No son escala: son detalle de dibujo del propio componente |
| Colores | **Token siempre, sin excepción** | Es lo único que pasa por la verificación de contraste de CI. Un hex escrito en un componente se salta esa verificación entera |
| Duraciones | **Token siempre, sin excepción** | Hay tres. Una cuarta suelta es una que nadie decidió |

La regla se comprueba: «ninguna primitiva contiene un color literal», «…una
duración literal» y «…una medida suelta», las tres en `validar.py --fase 2`, las
tres probadas rompiéndolas a propósito.

### 2.2 El armazón

**Ventana mínima: 1120 × 720** (`--arles-window-min-width`,
`--arles-window-min-height`). Por debajo, el contenido se recorta en vez de
reorganizarse: reorganizar es un patrón adaptable y aquí no aplica.

```
┌────────────────────────────────────────────────┐
│ Barra de título (nativa)                       │
├──────────┬─────────────────────────────────────┤
│ Nav      │ Encabezado de pantalla              │
│ 240px    ├─────────────────────────────────────┤
│          │ Contenido · máx. 1160px             │
│          │                                     │
│          ├─────────────────────────────────────┤
│ Pie      │ Barra de estado (motor)             │
└──────────┴─────────────────────────────────────┘
```

| Token | Valor | Qué es |
|---|---|---|
| `--arles-nav-width` | 240px | La barra lateral desplegada |
| `--arles-nav-width-plegada` | 64px | Plegada: iconos sin texto |
| `--arles-ancho-pagina` | 1160px | Tope del contenido, para que una línea no cruce un monitor de 4K |
| `--arles-ancho-lectura` | 68ch | Tope de un párrafo seguido |
| `--arles-isotipo` | 40px | La placa de la marca |
| `--arles-icono-nav` | 20px | El icono de una sección |

> **La barra lateral SÍ se plega, y el umbral está medido.** Esta sección decía
> *«es fija, no colapsable»* y razonaba que un panel que se pliega añade un
> estado que el usuario tiene que gestionar sin ganar nada. Dejó de ser verdad:
> `stores/interfaz.ts` tiene el estado, y el ancho a partir del cual se plega
> **sale de `npm run sonda:plegado`** y no de una estimación a ojo — es el ancho
> por debajo del cual el contenido deja de caber a su medida de diseño con los
> 240 px de barra delante.
>
> Las dos formas de plegar —la que elige el usuario y la que impone el ancho— no
> compiten: el estado es la unión de las dos.

**Seis secciones** en la barra: Inicio, Campañas, Contactos, Remitentes,
Actividad y Ajustes. **Dieciocho pantallas** en total; el mapa de cuál dibuja qué
está en [`MAPA-DE-PANTALLAS.md`](MAPA-DE-PANTALLAS.md).

---

## 3 · Elevación

Cuatro capas por **luminosidad y borde**, no por sombra:

| Capa | Token de fondo | Borde |
|---|---|---|
| Fondo | `--arles-bg-deep` | — |
| Panel | `--arles-surface` | `--arles-border` |
| Tarjeta, modal | `--arles-surface-raised` | `--arles-border-strong` |
| Hover | `--arles-surface-hover` | `--arles-border-strong` |

Y una quinta que no es una capa sino un estado: `--arles-nav-activo`, el relleno
de la sección abierta.

> **Es un token propio y no `--arles-surface-raised`, y la razón es el tema
> claro:** allí esa superficie es blanco puro sobre una barra casi blanca —
> **1.06:1** — y el relleno no marcaba nada. Por eso tiene su propio contrato de
> contraste contra la barra, con un mínimo de 1.5 que no es de WCAG: es el suelo
> por debajo del cual un relleno deja de aportar información.

**Por qué no sombras.** Sobre `#041A25` una sombra es casi invisible: no hay
contraste donde proyectarla. Además, las sombras son uno de los puntos donde
WebView2 y WKWebView divergen visiblemente (riesgo R-07), y cuestan rendimiento
al desplazar tablas largas. Un borde separa mejor, se renderiza idéntico y es
gratis.

Excepciones, con su token: `--arles-shadow-modal`, `--arles-shadow-menu` y
`--arles-backdrop`. En los tres casos la función es **separar del fondo un
elemento que flota**, no elevar una superficie.

**Las tres veladuras** —`--arles-veladura-superficie`,
`--arles-veladura-tarjeta`, `--arles-veladura-acento`— son degradados de
transparencia sobre una superficie ya verificada. No cargan texto ni cambian el
contraste: dan volumen al relleno. Van en un `background-image` aparte del
`background-color` precisamente para que el color de debajo siga siendo el
token, y el contraste siga siendo el medido.

---

## 4 · Radios y trazos

```css
--arles-radius-sm:  4px;   /* insignias, entradas */
--arles-radius-md:  6px;   /* botones, tarjetas */
--arles-radius-lg:  8px;   /* modales, paneles */
--arles-border-width: 1px;
--arles-focus-width:  2px;
```

Radios contenidos. Los radios grandes leen como software de consumo; ARLES es
una herramienta de trabajo.

---

## 5 · Foco — no negociable

```css
:focus-visible {
  outline: var(--arles-focus-width) solid var(--arles-accent-ink);
  outline-offset: 2px;
}

:focus:not(:focus-visible) {
  outline: none;
}
```

> **El anillo es `--arles-accent-ink` y no `--arles-accent`.** Esta sección
> decía el segundo. El oro de relleno sobre papel da **1.32:1**: en el tema
> claro el anillo se habría perdido, y un anillo de foco invisible es la
> navegación por teclado entera. `--arles-accent-ink` es el mismo oro en oscuro
> y una tinta en claro, y es el que está bajo contrato.

Contraste del anillo contra cada superficie, tema oscuro, mínimo no textual 3.0
(1.4.11):

| Superficie | Ratio |
|---|---|
| `--arles-bg-deep` | 11.68 |
| `--arles-surface` | 9.05 |
| `--arles-surface-raised` | 5.14 |
| `--arles-surface-hover` | 3.65 |

Las cuatro parejas están entre los **39 contratos** de `tokens.json`, así que se
verifican en los dos temas en cada compilación. Las cifras de esta tabla no son
la fuente: son una lectura de aquel día.

**La única excepción permitida a `outline: none`** es
`:focus:not(:focus-visible)`, que apaga el anillo al llegar con el ratón y lo
deja intacto para el teclado. La comprobación es literal sobre ese selector.

> **La versión anterior de la comprobación perdonaba cualquier selector que
> contuviera `:focus`**, y así dejó pasar un `.panel:focus { outline: none }`
> que apagaba el anillo también para el teclado. Lo encontró `sonda:teclado`, no
> la comprobación.

---

## 6 · Las dieciséis piezas

Doce primitivas y cuatro estados. **Una pantalla importa desde
`componentes/index.ts` y nunca desde el archivo de un componente**: así hay un
solo sitio donde ver qué existe, y añadir una pieza obliga a pasar por esa lista
— que es lo que impide que aparezca un decimotercer botón local en una pantalla
cualquiera.

### 6.1 Las doce primitivas

| Pieza | Qué es | Variantes y opciones |
|---|---|---|
| `ABoton` | El botón | `primario` · `secundario` · `sutil` · `peligro`; `compacto` / `normal`; `ocupado`, `deshabilitado`, `icono`, y `a` para dibujarse como enlace |
| `AEntrada` | Un campo | `text` · `email` · `password` · `number` · `date`; `ayuda`, `error`, `soloLectura`, `requerido`, `numerico` |
| `ASelector` | Una lista de opciones | `opciones`, `ayuda`, `error`, `soloLectura` |
| `ATabla` | La tabla virtualizada | `densidad` `compacta` / `comoda`; columnas con `ancho`, `numerica`, `ordenable`, `tituloOculto` |
| `AModulo` | El contenedor de una sección de pantalla | `nota` (una cifra, nunca un adjetivo), `principal`, `nivel` `h1` / `h2` |
| `AAviso` | Un aviso en línea | `info` · `aviso` · `peligro` · `exito`; `urgente` lo anuncia al aparecer |
| `AInsignia` | El estado de una fila | `neutro` · `exito` · `aviso` · `peligro` · `info` · `incierto` |
| `AModal` | Un diálogo | `destructiva` quita el cierre con `Esc` y con el clic fuera |
| `AMenu` | Un menú de acciones | `acciones` |
| `APestanas` | Pestañas | `pestanas`, con `deshabilitada` por pestaña |
| `AIcono` | Un icono del conjunto cerrado | `nombre` entre los 24; `etiqueta` obligatoria si va solo |
| `ALogotipo` | La marca | `nav` · `display` · `icono`; `numeral` añade la «I» |

**Tres de ellas tienen un porqué que no se adivina:**

`AModulo.nivel` existe porque tres pantallas no tienen cabecera propia —son un
módulo y ya—, y como el módulo siempre pintaba un `h2`, **esas tres no tenían
ningún `h1`**. Para quien usa un lector de pantalla, «ir al encabezado 1» no
llevaba a ninguna parte. La sonda decía vigilar la jerarquía de encabezados y no
lo veía: medía los saltos *entre* encabezados y nunca comprobaba **por cuál
empieza**.

`AEntrada.soloLectura` **no es lo mismo que `deshabilitado`**, y la diferencia
importa. `deshabilitado` dice «esto no aplica ahora»: se atenúa, sale del orden
de tabulación y no se puede seleccionar. `soloLectura` dice «esto ya está
decidido»: contraste normal, enfocable, y el valor se puede seleccionar y
copiar. Es lo que hace falta en una pantalla de configuración guardada, donde el
dato es justo lo que se viene a leer.

`ColumnaDeTabla.tituloOculto` guarda el rótulo para el lector de pantalla y no
lo dibuja. Nació el 22/10/2026 para la columna de acciones de Contactos: medido
a 1024 px —un portátil al 150 %—, **cinco cabeceras con su icono de ordenar no
caben**, y «Acciones» ocupaba 96 px para decir algo que dos iconos ya dicen.
Repartir los `fr` no lo arregla: reparten el espacio, no lo crean.

### 6.2 Los cuatro estados

| Pieza | Qué pide | Por qué así |
|---|---|---|
| `EstadoVacio` | `titulo`, `cuerpo` | Explica qué es esto y da la acción para empezar. No un dibujo con «Nada por aquí» |
| `EstadoCargando` | `lineas`, `explicacion` | Esqueleto con la forma del contenido real, no un girador. La explicación aparece a partir de los 2 s |
| `EstadoError` | **`que`, `como`, `salvo`** | Las tres partes del §95 son **tres parámetros obligatorios**: no se puede construir un error sin decir qué está a salvo |
| `EstadoExito` | `titulo`, `detalle` | Breve y con la cifra. Sin celebraciones (§94) |

> **`EstadoError` es el mejor poka-yoke de este sistema**, y conviene ver por
> qué: la regla del §95 —todo error dice qué pasó, cómo arreglarlo y qué está a
> salvo— no vive en una comprobación que repasa los errores escritos. Vive en el
> **tipo**. Un error sin la tercera parte **no compila**. Gana el tipo sobre el
> validador, y el validador sobre la prueba.

### 6.3 El botón, con detalle

| Variante | Relleno | Texto | Uso |
|---|---|---|---|
| **Primario** | `--arles-accent` | `--arles-text-on-accent` | **Uno por pantalla** |
| Secundario | transparente, borde `--arles-border-strong` | `--arles-text` | Acciones habituales |
| Sutil | transparente | `--arles-text-muted` | Terciarias |
| Peligro | `--arles-danger` | `--arles-text-on-danger` | Detener, eliminar |

**El botón primario lleva texto oscuro.** 11.68:1. Con texto blanco sería
1.52:1 — ilegible (`COLOR_SYSTEM.md` §7.2). En hover el relleno pasa a
`--arles-accent-hover` y `--arles-danger-hover`, **que también están bajo
contrato**: un hover que degrada el contraste es un contraste que no se cumple
durante el clic.

Altura `--arles-control-height` (40px) y `--arles-control-height-compact`
(32px). Nunca menos de 32: es `--arles-objetivo-minimo`, y el §22 lo pide por
encima de los 24 px del WCAG 2.2 a propósito.

> **El mínimo de objetivo vivía en el §22 y en ningún token.** Lo encontró la
> auditoría D: **cinco enlaces de navegación medían 20 px de alto**, el alto de
> su línea sin un píxel de relleno. A ojo un enlace corto no se ve pequeño, que
> es por lo que hacía falta medirlo.

#### Deshabilitado no es opacidad — corregido en la Fase 2

La primera implementación atenuaba el botón entero con `opacity: 0.45`. Medido
en el catálogo abierto en el navegador: un **primario deshabilitado daba
1.89:1** y un **secundario ocupado, 3.56:1**. La etiqueta no se leía.

WCAG 1.4.3 exime a los controles deshabilitados del mínimo de contraste, así que
ninguna herramienta automática lo habría marcado. Pero un botón cuyo texto no se
distingue no comunica *qué* está deshabilitado, que es justo lo contrario de lo
que pide el §95.

El estado se pinta, no se atenúa: relleno `--arles-surface-raised` y texto
`--arles-text-disabled` —bajo contrato con mínimo 3.0 contra las tres
superficies—, iguales para las cuatro variantes, más `cursor: not-allowed` como
segunda señal (regla 7.3).

Y **«ocupado» no se atenúa en absoluto**: no está deshabilitado por una regla de
negocio, está trabajando, y el usuario lo está mirando precisamente ahora.
Conserva su contraste completo y sólo deja de aceptar clics.

**Un botón apagado dice por qué.** Es el §95 aplicado a un control, y es el
defecto que han tenido tres pantallas de la Fase 8 —«Repite la contraseña»,
«Restaurar y reemplazar», «Continuar» del paso 6—: bloquear sin explicar se lee
como una avería. El aviso va **pegado al botón**, no al final de la pantalla.

### 6.4 La tabla, el componente crítico

Es donde el usuario pasa la mayor parte del tiempo.

- **Virtualización siempre**, con `@tanstack/vue-virtual` (ADR-0007). Medio
  millón de filas.
- **No es un `<table>`**, y la cabecera del archivo explica por qué: un `<table>`
  real no se puede virtualizar sin romper su propio modelo de layout. Los roles
  ARIA los pone el componente a mano, y por eso `etiqueta` es obligatoria.
- Altura de fila `--arles-row-height-compact` (36px) o
  `--arles-row-height-comfortable` (44px), por la opción `densidad` **de cada
  tabla**, no por una preferencia global.
- Encabezado pegajoso, ordenación por columna con indicador de dirección.
- **Selección con `--arles-surface-raised` más un borde izquierdo de acento.** No
  sólo color: regla 7.3.
- Navegación completa por teclado: flechas, `Inicio`/`Fin`, `Espacio` para
  seleccionar, `Mayús`+flechas para rango.
- La columna de acciones, fija a la derecha.

> **La densidad es una opción del componente, no una preferencia del usuario.**
> Esta sección decía «preferencia del usuario»: no existe esa pantalla ni ese
> ajuste. Cada tabla elige la suya al usarla. Decirlo de otra forma prometía un
> control que nadie puede encontrar.

> **Y una trampa de verificación que vale para cualquier prueba de tabla:** en
> jsdom el contenedor mide 0 px de alto, así que el virtualizador no renderiza
> **ninguna** fila y una prueba de virtualización pasa en vacío. Lo que de
> verdad mide la tabla es `npm run sonda:tabla`, en un navegador.

### 6.5 Las insignias

**Relleno sólido con tinta propia**, no color de texto sobre transparente.
Motivo: sobre `--arles-surface-raised` los colores semánticos fallan todos
(2.58–2.92). Con relleno sólido, el contraste es siempre el del relleno contra
su tinta — y las cuatro parejas están bajo contrato:

| Tono | Relleno | Tinta |
|---|---|---|
| Éxito | `--arles-success` | `--arles-text-on-success` |
| Aviso | `--arles-warning` | `--arles-text-on-warning` |
| Peligro | `--arles-danger` | `--arles-text-on-danger` |
| Info | `--arles-info` | `--arles-text-on-info` |

> **Tres de esas cuatro parejas entraron tarde, y las encontró el navegador.** En
> el tema claro el verde es oscuro (`#0F6B52`) y el ámbar también, así que la
> tinta oscura daba **2.47:1** en la insignia de «puede enviar». Lo midió la
> auditoría del 29/09/2026, **no el sistema de tokens**: el contrato no existía,
> así que no había nada que fallar. Ahora las cuatro están declaradas, la de
> peligro incluida — que ya pasaba, pero no estaba bajo contrato, y eso es lo
> mismo que no estar verificada.

> **No se exige icono, y es deliberado.** Esta sección decía *«cada insignia
> lleva icono además del color»*. La revisión del 29/09/2026 encontró lo
> contrario: **las ocho insignias neutras llevaban un reloj** —el icono de «en
> cola»— y ninguna esperaba nada. Un contador de bajas, «Todos sus canales», una
> calidad que no hemos preguntado: tres cosas que no están en cola, con el
> dibujo de estarlo.
>
> Hoy `AInsignia` **no pinta icono en el tono neutro** y sí en los cinco
> semánticos. Lo que la regla 7.3 exige de verdad es que ningún estado se
> distinga **sólo por color**, y eso lo cumple el **texto** de la insignia, que
> siempre está. Lo mide `sonda:auditoria`: cada insignia tiene que llevar texto
> o nombre accesible.

### 6.6 Los modales

Máximo `--arles-modal-max-width` (640px). Fondo oscurecido con
`--arles-backdrop`. Foco atrapado dentro. `Esc` cierra, **salvo con
`destructiva`**, que exige decisión explícita.

Un `AMenu` nunca baja de `--arles-menu-min-width` (200px): un menú que se
ajusta a su opción más corta cambia de ancho cada vez que cambian las acciones,
y el usuario apunta a un blanco que se mueve.

**Detener una campaña abre un modal que dice cuántos mensajes quedarán sin
enviar.** Un número concreto, no «¿estás seguro?».

---

## 7 · Movimiento

```css
--arles-duration-fast:   120ms;   /* hover, foco */
--arles-duration-normal: 200ms;   /* paneles, modales */
--arles-duration-slow:   320ms;   /* transiciones de pantalla */

--arles-ease:         cubic-bezier(0.2, 0, 0.2, 1);
--arles-ease-entrada: cubic-bezier(0.16, 1, 0.3, 1);
--arles-ease-resorte: cubic-bezier(0.34, 1.3, 0.64, 1);

--arles-pulsado: 0.98;   /* la escala de un control mientras se pulsa */
```

**Con propósito** (§98): mostrar progreso, orientar en una transición, confirmar
una acción. **Nunca decorativo.**

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

Obligatorio, no opcional. Y la comprobación exige **las dos** propiedades: una
regla que sólo neutraliza `transition` deja corriendo cualquier `animation`, como
la del esqueleto de carga.

**Nada parpadea.** Ni siquiera para llamar la atención sobre un error: el
parpadeo es un riesgo de accesibilidad y una molestia en sesiones largas.

---

## 8 · Tipografía

Una escala de siete tamaños, cada uno con su interlineado, y cinco pesos:

| Uso | Tamaño | Interlineado |
|---|---|---|
| Display | `--arles-font-size-display` 36px | `--arles-line-height-display` 44px |
| h1 | `--arles-font-size-h1` 28px | `--arles-line-height-h1` 36px |
| h2 | `--arles-font-size-h2` 20px | `--arles-line-height-h2` 28px |
| h3 | `--arles-font-size-h3` 16px | `--arles-line-height-h3` 24px |
| Cuerpo | `--arles-font-size-body` 14px | `--arles-line-height-body` 20px |
| Pequeño | `--arles-font-size-small` 13px | `--arles-line-height-small` 18px |
| Pie | `--arles-font-size-caption` 12px | `--arles-line-height-caption` 16px |

Cinco pesos: `--arles-font-weight-regular` (400),
`--arles-font-weight-medium` (500), `--arles-font-weight-semibold` (600),
`--arles-font-weight-bold` (700) y `--arles-font-weight-black` (900).

Familias: `--arles-font-family` —Mont, con Figtree y la del sistema detrás— y
`--arles-font-family-mono` para datos que se alinean en columna.

> **Cada `@font-face` declara su propio `font-weight`, y no es redundancia.** El
> `usWeightClass` de este kit está desplazado: `Mont-Regular` declara 600. Si un
> `@font-face` no fija su peso, el motor usa el del archivo y asigna el corte
> equivocado — sin que nada falle. Lo comprueba la fase 2 contando caras y pesos.

**Los cuatro cortes van incrustados** (Regular, SemiBold, Bold, Black). Si falta
uno, el motor lo sintetiza engordando el trazo y la tipografía deja de ser Mont
en silencio.

**La licencia de Mont es P-01 y bloquea el instalador**, no la construcción. El
plan B de [`TIPOGRAFIA.md`](TIPOGRAFIA.md) §3 sigue listo y cuesta una línea:
`--arles-font-family`.

---

## 9 · Iconografía

**Un conjunto cerrado de 24 iconos, dibujados a mano**, trazo de 1.5 sobre
rejilla de 24, sin relleno y heredando `currentColor`.

> **No hay biblioteca de iconos.** Esta sección decía *«biblioteca base Lucide
> (ISC, sin restricciones)»*, y era falso en los dos sentidos: no es una
> dependencia del proyecto, y afirmar una licencia que no aplica es el peor sitio
> para equivocarse. Lo que hay es `AIcono.vue` con 24 trazos escritos dentro.
>
> El motivo de que sea cerrado no es el tamaño del paquete, aunque también:
> **con un tipo unión el compilador rechaza un nombre que no existe**, en vez de
> renderizar nada. Un `<AIcono nombre="candadito" />` no compila. Con un `src`
> libre, saldría un hueco.

Los 24 se agrupan así: cuatro semánticos (uno por estado, regla 7.3), cuatro de
estado de envío —incluido `incierto`, que tiene icono propio porque un envío
ambiguo (ADR-0004) **no es un éxito ni un fallo** y pintarlo con cualquiera de
los dos afirmaría algo que no se sabe—, el candado de «esto no se puede
deshacer», los seis de las secciones, y los de interacción.

`aria-hidden` por defecto: el icono acompaña a un texto que ya dice lo mismo.
Sólo cuando va solo se le pasa `etiqueta`, y entonces es obligatoria.

---

## 10 · Los cuatro estados, donde cada uno significa algo (§97)

> **Reescrita el 07/10/2026, en la 8.7.** Esta regla decía *«ninguna pantalla se
> da por terminada sin los cuatro»*, y la auditoría D del §140 la midió en las
> dieciocho pantallas: **no se puede cumplir, y no debería.** Seis no tienen
> ninguna colección —son formularios o pantallas de acción— y un «estado vacío»
> en un formulario no significa nada; Ajustes **no enseña errores a propósito**,
> con el motivo escrito en el store desde la Fase 3.
>
> Una regla que nunca se puede marcar es peor que una que falta: hace dudar de
> la lista entera, y la lista es lo que se revisa antes de cerrar una fase.

Lo que se exige, que es lo que se puede comprobar:

**1 · Toda pantalla que pinta una colección variable tiene su estado vacío.** Se
cumple en las nueve que la tienen: Actividad, Campañas, Contactos, Importar,
Plantillas, Registro, Remitentes, Bajas y WhatsApp. Las que recorren una lista
**fija** —los seis pasos de Inicio, los campos de Configuración— o **posterior a
una acción** —las cifras del preflight en Restaurar y Archivar— no la necesitan.

**2 · Toda espera que pueda pasar de un segundo tiene su estado de carga.**

**3 · Todo fallo que el usuario pueda provocar se enseña con las tres partes del
§95.** Diecisiete de dieciocho pantallas; la excepción es Ajustes y está
razonada arriba.

**4 · Todo final tiene su acuse**, breve y con el siguiente paso si lo hay.

Un ejemplo de cada extremo:

> **Aún no hay campañas.**
> Una campaña envía un mensaje a una lista de contactos, respetando los límites
> que definas.
> `[Crear campaña]`

> **No se pudo conectar con smtp.gmail.com.**
> Revisa tu conexión a internet y que el puerto 587 no esté bloqueado.
> **Tu campaña está en pausa, no cancelada.** Los 1 240 envíos pendientes se
> reanudarán al restablecerse la conexión.

La tercera parte es la que falta en casi todo el software y la que importa cuando
alguien acaba de ver fallar una campaña.

---

## 11 · Los tokens: dónde vive cada cosa

**89 tokens, en dos sitios que no se solapan.** La separación no es
organizativa: es lo que hace que el contraste se pueda verificar.

| | Cuántos | Dónde | Por qué ahí |
|---|---|---|---|
| **Color** | 25 | [`herramientas/design-tokens/tokens.json`](../../herramientas/design-tokens/README.md) → `dist/arles-tokens.css` | Cada color lleva su valor en los dos temas y sus contratos de contraste. Es lo único que pasa por la verificación de CI |
| **Todo lo demás** | 64 | `app/src/design/base.css` | Escala, tipografía, movimiento, geometría del armazón. No tienen contraste que verificar |

Los 25 colores van en cinco grupos —superficies, contenido y acento,
semánticos, tintas sobre relleno, y marca—, con **39 contratos** de contraste y
**3 advertencias**, que son máximos: parejas que **no deben** pasar AA, porque si
pasaran significaría que una regla documentada se quedó obsoleta.

**Para añadir un color:** se edita `tokens.json`, se declara su contrato, y se
regenera con `./herramientas/design-tokens/generar.py --todo`. **Si el contraste
no pasa, el color no entra.** `--verificar` falla la compilación ante una
regresión.

**Los dos temas se verifican los dos.** El claro no es una inversión automática:
cada color lleva su traducción medida, y hay casos donde la tinta se invierte y
otros donde no —el oro de la marca es claro en los dos temas, así que su tinta es
oscura en los dos—. El tema vive en `:root[data-tema="claro"]` y **no** en
`@media (prefers-color-scheme)`, porque es una preferencia de ARLES que el
usuario elige, no del sistema. Lo mide `npm run sonda:tema`.

> **En el tema claro, todo relleno de acento lleva borde.** El oro sobre papel da
> **1.32:1**: el botón se ve, pero *como forma no existe*. El borde es
> `--arles-accent-ink`, que contra la página da 5.47:1, y es lo que cumple el
> 1.4.11.

**Dos colores no se invierten nunca:** `--arles-marca-fondo` y
`--arles-marca-tinta`, la placa del isotipo y su «A». Son los del icono que
genera `herramientas/iconos/generar-iconos.py`, y **el icono de la barra de
tareas no cambia con el tema del sistema**. Su contrato es de 3.0 y no de 4.5:
es un logotipo, y le aplica el 1.4.11 y no el mínimo de texto.

**Y `--arles-info` no puede ser superficie que cargue texto**, en el tema
oscuro. El cian vivo falla contra todo texto candidato, blanco incluido: es
color de trazo, borde, foco e icono. Ésa es una de las tres advertencias — si
algún día pasara AA, significaría que la regla documentada se quedó obsoleta, y
es la comprobación la que tiene que avisar.

**Y una cosa que ningún token resuelve:** un `var()` con el nombre mal escrito
**no da error**. El navegador lo resuelve a nada y el elemento se queda
transparente.
Es el fallo de CSS que más lejos llega sin que nadie lo vea, y ha pasado cuatro
veces en este proyecto —seis tokens inventados en la 5.2, catorce en una pantalla
de la Fase 3, tres en la 8.3c—. Lo caza «todo token que se usa está definido»,
que compara cada `var()` de `app/src` con lo definido, sumando lo que cada
archivo declara para sí mismo.

---

## 12 · Accesibilidad: qué se verifica y con qué

Diez puntos, y al lado **qué** los mide. Un punto sin instrumento es un punto que
nadie ha comprobado, y por eso van juntos:

| Qué | Con qué |
|---|---|
| Contraste AA en todo par texto/fondo | `generar.py --verificar`, en CI, los dos temas |
| Contraste **real** medido en el navegador | `sonda:auditoria`, las 18 pantallas a cuatro ventanas |
| Foco visible en todo elemento interactivo | `sonda:teclado` y la comprobación de `outline: none` |
| Navegación completa por teclado y orden lógico | `sonda:teclado` |
| Ningún estado comunicado sólo por color | `sonda:auditoria`: cada insignia con texto o nombre accesible |
| `prefers-reduced-motion` respetado | fase 2, exigiendo las dos propiedades |
| Objetivos de 32 px, a escalado 200 % | `sonda:auditoria`, con 960×540 entre sus ventanas |
| Roles y nombres accesibles en componentes compuestos | `sonda:lector` y las pruebas de componente |
| Jerarquía de encabezados, empezando por `h1` | `sonda:auditoria` |
| Sin desbordamiento ni texto encimado | `sonda:ancho` y `sonda:encimado`, 1 836 textos en 72 pantallas-ventana |
| **Verificado en WebView2 y WKWebView** | **Nadie. Es R-07** |

La última línea es la que más a menudo se olvida y la que ADR-0001 obliga a
sostener. **Nadie ha abierto ARLES en un Mac**: la vista previa no cuenta, ahí el
motor es el del navegador.

> **Una sonda que no recorre una pantalla no dice nada de esa pantalla, aunque
> termine en verde.** Esa frase llevaba ocho días escrita en la cabecera de
> `auditoria.mjs` cuando la auditoría D descubrió que **las dos sondas que
> recorren el producto iban cinco pantallas por detrás** — con su lista de rutas
> escrita a mano, parada en trece. Las cinco que faltaban eran todo lo que
> construyó la Fase 8. Hoy salen de `rutas.mjs`, que las lee del router, y lo
> vigila `las_sondas_recorren_todas_las_pantallas`.
>
> La lección no es sobre sondas: **se escribió la lección en un comentario y
> volvió a pasar cinco veces**, porque la lista seguía siendo a mano. Un
> comentario no es un poka-yoke.

---

## 13 · El poka-yoke de este archivo

Este documento es **una lista escrita a mano**, y el patrón número 3 de este
proyecto dice que una lista a mano se queda atrás: once de los diecisiete
hallazgos de las cuatro auditorías del §140 eran eso. Así que se comprueba.

`el_sistema_de_diseno_dice_lo_que_hay`, en `validar.py --fase 2`, mide cuatro
cosas **leyendo el código** y no una segunda lista:

1. **Todo token que este archivo nombra existe.** Lee los `--arles-…` del
   documento y los compara con `base.css` y `dist/arles-tokens.css`.
2. **Toda pieza de `componentes/` está nombrada aquí.** Lee la carpeta. Una
   primitiva nueva sin documentar pone la fase en rojo.
3. **Toda pieza que este archivo nombra existe.** La dirección contraria: caza
   una pieza renombrada o retirada que el documento siga dando por viva.
4. **Las cifras de este archivo son las que hay.** Los recuentos del §11 —25
   colores, 64 tokens, 39 contratos— y las dieciséis piezas del §6 se cuentan de
   la fuente. Una cifra a mano en un documento es exactamente lo que el tablero
   tiene prohibido.

**Por qué la 4 y no sólo prosa sin cifras:** el §94 pide cifras y no adjetivos.
«Unos cuantos tokens» no se puede comprobar y no informa. Una cifra comprobada
informa y no puede envejecer.

Y por eso la regla del §0: **aquí no se escribe el nombre de un token ni de una
pieza que no exista**, ni como contraejemplo. La comprobación no sabe distinguir
una afirmación de un ejemplo, y en este proyecto hay nueve casos de una
comprobación cazándose a sí misma en un comentario.

---

## 14 · Las cinco cosas que este archivo afirmaba y no eran verdad

Encontradas el 27/10/2026 al reescribirlo con el código delante. Las cinco
estaban aquí desde la Fase 2; **cuatro fases y cuatro auditorías pasaron por
encima sin verlas**, porque una auditoría mide el producto.

| Decía | Lo que hay | Qué habría costado |
|---|---|---|
| «La barra lateral es fija, **no colapsable**» | Se plega, y el umbral sale de `sonda:plegado` | Quien lea esto para diseñar una pantalla nueva no la probará plegada |
| Altura de fila: «**preferencia del usuario**» | Es una opción de cada tabla; esa pantalla no existe | Prometer un control que nadie puede encontrar |
| Anillo de foco: `--arles-accent` | `--arles-accent-ink` | En el tema claro, 1.32:1. Copiar el fragmento deja el anillo invisible |
| Iconos: «biblioteca base **Lucide** (ISC)» | 24 trazos propios; no es una dependencia | Afirmar una licencia que no aplica, y buscar durante un rato un paquete que no está |
| «**Cada insignia** lleva icono además del color» | El tono neutro no lleva icono, a propósito desde el 29/09/2026 | Volver a poner el reloj en ocho insignias que no esperan nada |

Y dos huecos, que no son afirmaciones falsas pero cuestan lo mismo:

- **El tema claro no se mencionaba en ninguna parte**, con los 25 colores
  llevando su valor claro desde la Fase 3 y una sonda midiéndolo.
- **Faltaban 40 de los 64 tokens que no son color**: la escala tipográfica
  entera, las tres veladuras, las dos curvas de entrada, los anchos del armazón
  y el mínimo de objetivo. Un token que no está documentado es un token que
  alguien va a volver a inventar — y eso ya pasó tres veces en la Fase 8.
