# Tipografía

**Proyecto:** ARLES RELAY I · v1.2.0
**Estado:** ⚠️ **BLOQUEADO POR D-3** — la licencia de Mont no está verificada

---

## 1. El bloqueo, primero

`/TIPOGRAFIA` contiene la familia **Mont** de **Fontfabric** (Svetoslav Simov, Mirela Belova, v1.003), verificado leyendo la tabla `name` de los TTF.

El kit fue generado con **Transfonter** —`demo.html` lo declara— y contiene `.eot`, un formato muerto desde IE11. Ese perfil es característico de un paquete descargado de un agregador de fuentes, no de una entrega comercial de Fontfabric, que suministra OTF/TTF de escritorio y, por separado, webfonts con licencia propia.

**No hay licencia, factura ni comprobante en el repositorio.**

### Por qué importa para ARLES en concreto

T-2 afirma que TELEMETRY posee «la licencia comercial de Mont para uso de texto». El alcance decide:

| Licencia de Fontfabric | ¿Cubre incrustar la fuente en el `.exe` / `.dmg` distribuido? |
|---|---|
| Desktop | **No** |
| Web | **No** |
| **App** | **Sí** — es la que ARLES necesita |

Un bundle de Tauri lleva los archivos de fuente **dentro del binario que se instala en la máquina del cliente**. Ése es exactamente el supuesto de la App License, y ni la Desktop ni la Web lo autorizan.

El §21 del propio brief ordena detenerse ante esto. Nos detenemos.

**Riesgo R-01 · Pregunta P-01 · Decisión D-3.**

---

## 2. Plan A — Mont, si la licencia lo permite

### Escala

Base **14 px**, escala ~1.2. Densidad de aplicación de escritorio, no de página web.

| Rol | Tamaño / Interlínea | Peso | Uso |
|---|---|---|---|
| Display | 32 / 40 | Black 900 | Sólo pantallas de bienvenida |
| H1 | 24 / 32 | Bold 700 | Título de pantalla |
| H2 | 20 / 28 | Bold 700 | Sección |
| H3 | 16 / 24 | SemiBold 600 | Subsección, título de tarjeta |
| H4 | 14 / 20 | SemiBold 600 | Etiqueta de grupo |
| **Body** | **14 / 20** | Regular 400 | **Por defecto** |
| Body small | 13 / 18 | Regular 400 | Celdas de tabla densas |
| Caption | 12 / 16 | Regular 400 | Ayudas, marcas de tiempo |
| Button | 14 / 20 | SemiBold 600 | Botones |
| **Numeric** | **14 / 20** | Medium 500, **tabulares** | **Cifras en tablas** |

### Mont Black se usa con moderación (§20)

**Sólo** en el logotipo y en Display. Nada más.

Mont Black en encabezados de sección o en botones produce el aspecto de software promocional, que es lo contrario de «sobrio y corporativo» (§15). El peso extremo pierde su significado cuando se reparte.

### Cifras tabulares — un detalle que importa

ARLES es una aplicación llena de números en columnas: enviados, pendientes, límites, porcentajes.

```css
font-variant-numeric: tabular-nums;
```

Sin esto, las cifras bailan al actualizarse y las columnas no se alinean. En una interfaz de tablas es la diferencia entre parecer una herramienta profesional y parecer un prototipo.

**Verificar que el corte de Mont disponible incluye figuras tabulares.** Si no, es un argumento adicional a favor del plan B.

### Formatos a incrustar

**Sólo `.woff2`.** Es lo único que WebView2 y WKWebView necesitan. `.eot`, `.ttf` y `.woff` no se incluyen en el bundle: son peso muerto y, en el caso del `.eot`, una señal confusa sobre la procedencia del kit.

Pesos a incrustar: Regular 400, Medium 500, SemiBold 600, Bold 700, Black 900. **Sin cursivas** salvo que algún componente las requiera — cada peso son unos 45 KB.

---

## 3. Plan B — si D-3 no se resuelve

**Se activa automáticamente si al inicio de la Fase 2 no hay comprobante de licencia.** No requiere una decisión nueva.

### Reparto

| Dónde | Qué |
|---|---|
| Marketing, web, material comercial | **Mont** (cubierto por licencia Desktop/Web) |
| **Logotipo dentro de la aplicación** | **SVG con contornos** |
| Interfaz de la aplicación | Geométrica de licencia libre |

**Por qué el logotipo en SVG con contornos es legítimo:** convertir texto a trazados es uso normal de una licencia de escritorio — es lo mismo que exportar un logotipo desde Illustrator. El resultado es un archivo de geometría vectorial, **no contiene la fuente**, y por tanto no hay binario de fuente que distribuir. El logotipo conserva Mont exactamente; lo que no se incrusta es la familia.

Esto preserva la identidad de marca donde más se ve, con coste cero y riesgo cero.

### Sustituta para la interfaz

Criterios: geométrica grotesca, métricas y altura de x similares a Mont, licencia SIL OFL, cifras tabulares, y al menos cinco pesos.

| Candidata | Licencia | Valoración |
|---|---|---|
| **Figtree** | OFL | **Recomendada.** Geométrica, altura de x cercana a Mont, 9 pesos, tabulares |
| Plus Jakarta Sans | OFL | Excelente calidad; algo más de carácter propio |
| Manrope | OFL | Buena, pero sólo 6 pesos y menos neutra |
| Inter | OFL | Impecable en interfaz, pero es neutra-grotesca, no geométrica: se aleja del carácter de Mont |

**Recomendación: Figtree**, por ser la más cercana a Mont en proporción y en sensación geométrica.

### Coste del plan B

Bajo. La escala, la jerarquía y los tokens **no cambian** — sólo la familia. Si toda la tipografía vive en `--arles-font-family`, cambiar el plan A por el B es editar una línea.

**Por eso la escala se define ahora aunque la familia esté bloqueada.** El bloqueo afecta a qué fuente, no a cómo se estructura.

---

## 4. Tokens

```css
:root {
  /* Plan A: 'Mont'. Plan B: 'Figtree'. Una sola línea cambia. */
  --arles-font-family: 'Mont', 'Figtree', system-ui, -apple-system,
                       'Segoe UI', sans-serif;

  --arles-font-size-display: 32px;  --arles-line-height-display: 40px;
  --arles-font-size-h1:      24px;  --arles-line-height-h1:      32px;
  --arles-font-size-h2:      20px;  --arles-line-height-h2:      28px;
  --arles-font-size-h3:      16px;  --arles-line-height-h3:      24px;
  --arles-font-size-body:    14px;  --arles-line-height-body:    20px;
  --arles-font-size-small:   13px;  --arles-line-height-small:   18px;
  --arles-font-size-caption: 12px;  --arles-line-height-caption: 16px;

  --arles-font-weight-regular:  400;
  --arles-font-weight-medium:   500;
  --arles-font-weight-semibold: 600;
  --arles-font-weight-bold:     700;
  --arles-font-weight-black:    900;
}
```

**La pila de reserva importa.** Las fuentes se cargan de forma local desde el bundle, así que un fallo es improbable — pero si ocurre, `system-ui` produce una interfaz usable en vez de una rota.

---

## 5. Logotipo (§21)

**Exclusivamente tipográfico: «ARLES RELAY» en Mont Black.** Sin isotipo, sin símbolo.

Prohibido: estilizar letras, estirar, condensar, aplicar degradados o efectos 3D, rotar, añadir sombras.

**Se entrega como SVG con contornos** en ambos planes. En el plan A por consistencia de renderizado entre plataformas; en el plan B además por necesidad legal.

Área de respeto: la altura de la «A» por cada lado.
Tamaño mínimo: 120 px de ancho.

Sobre el numeral «I»: el logotipo dice **ARLES RELAY**, sin «I». El «I» pertenece al nombre comercial y aparece en el empaque y en «Acerca de», nunca en el logotipo ni adyacente al número de versión (ADR-0010).

---

## 6. Renderizado entre plataformas

WebView2 y WKWebView **no renderizan el texto igual**: difieren en suavizado, aplicación de hinting y ajuste óptico del tamaño (riesgo R-07).

Lo que hay que hacer:

- Verificar la escala completa en Windows y macOS antes de congelarla.
- Comprobar a escalado 100 %, 125 %, 150 % y 200 % en Windows (§22).
- Comprobar en pantallas Retina y no-Retina.
- No usar `-webkit-font-smoothing: antialiased` a ciegas: en macOS adelgaza el texto y sobre fondos oscuros como los de ARLES puede reducir la legibilidad por debajo de lo cómodo.

**Texto claro sobre fondo oscuro parece más pesado** que el mismo texto en negativo. Puede ser necesario bajar un peso respecto a lo que se elegiría en un tema claro — por ejemplo, Regular donde intuitivamente se pondría Medium.

---

## 7. Qué se necesita de Dirección

**P-01.** Tipo exacto de licencia de Mont, y comprobante en `documentacion/08-legal/`.

**Sin comprobante al inicio de la Fase 2, se activa el plan B sin más discusión.** No es una amenaza: es que el Design System no puede quedarse bloqueado indefinidamente, y el plan B es perfectamente bueno.
