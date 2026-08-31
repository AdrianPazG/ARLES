# FASE 0 — Discovery, Auditoría y Arquitectura

**Producto:** ARLES RELAY I · **Versión objetivo:** v1.2.0 · **Desarrolla:** TELEMETRY INSIGHT

> Esta carpeta **no contiene código de aplicación**. Contiene la evidencia reproducible del análisis
> previo exigido por §1 y §175 del brief maestro. No se ha escrito código de producción y el proyecto
> queda a la espera de la autorización de Dirección.

---

## Contenido

| Archivo | Qué es |
|---|---|
| `dossier_fase_0.html` | Entregable completo, secciones A–U según §163, incluyendo la propuesta visual de §143. |
| `extraccion_paleta.py` | Extracción cromática sobre `/REFERENCIA_DE_COLOR` (k-means en OKLab + minería de acentos). |
| `derivacion_tokens.py` | Derivación de los tokens de ARLES y **verificación de contraste WCAG 2.2**. |

Reproducir:

```bash
pip install pillow numpy
python3 FASE_0_DISCOVERY/extraccion_paleta.py     # mide la referencia oficial
python3 FASE_0_DISCOVERY/derivacion_tokens.py     # deriva tokens y verifica contraste
```

Ambos scripts son **solo lectura** sobre `/REFERENCIA_DE_COLOR`. Ningún archivo de las carpetas de
referencia fue movido, renombrado, convertido, comprimido ni editado (§11).

---

## Hallazgos que bloquean el inicio de desarrollo

1. **`/RECURSOS` no existe.** El brief la referencia once veces como origen del logotipo y de los datos
   de contacto de TELEMETRY INSIGHT (§118, §145, §170). No está en el repositorio, y ningún archivo
   contiene esos datos. §118 prohíbe inventarlos → **el pie de aplicación no se puede construir.**

2. **Licencia de la familia Mont sin resolver.** Los archivos de `/TIPOGRAFIA` son un kit web convertido
   con Transfonter (incluye `.eot`), de **Fontfabric** (fuente comercial), **sin EULA ni registro de
   licencia en los metadatos**. §21 ordena detener esa parte y reportarlo. Incrustarla en un instalador
   comercial requiere licencia de aplicación.

3. **Contradicción interna del brief sobre rebotes.** §39 y §64 exigen supresión automática por *hard
   bounce* y reporte de rebotes; §69 y §85 prohíben pedir permisos de lectura de buzón. Sin leer el
   buzón, ARLES no puede detectar rebotes asíncronos. Requiere decisión de Dirección.

---

## Paleta medida sobre la referencia oficial

Archivo: `REFERENCIA_DE_COLOR/farm-lifestyle-digital-art.jpg` — **PNG real de 2320×3080** (7 145 600 px),
sin recomprimir. Método: k-means (k=12) en espacio OKLab sobre 400 000 px muestreados con semilla fija,
más una segunda pasada que busca acentos de alto croma por familia de tono aunque ocupen poca
superficie (§16: el amarillo importa aunque sea minoritario).

| Ancla | Hex | % área | L (OKLab) | Rol |
|---|---|---|---|---|
| Cielo dominante | `#0B6690` | 23.12 | 0.483 | Azul de estructura |
| Cielo medio/alto | `#50A6C3` | 14.01 | 0.683 | Superficies claras |
| Transición | `#2586A9` | 9.90 | 0.580 | Azul intermedio |
| Cian pálido | `#92C1C5` | 6.80 | 0.780 | Borde de nube |
| Crema | `#DFE2D3` | 6.68 | 0.906 | Respiración / texto |
| Amarillo girasol | `#F7D42C` | 5.94 | 0.874 | **Acento** |
| Oro | `#EAA11C` | 4.51 | 0.762 | Advertencia |
| Ámbar profundo | `#C77617` | 5.98 | 0.642 | Variación cálida |
| Oscuro neutro | `#22262A` | 6.45 | 0.266 | Profundidad |

**Decisión clave:** el azul más oscuro de la referencia (`#055480`, L 0.427) es demasiado claro y
saturado para ser fondo de aplicación. La rampa de ARLES **extiende el tono medido (236.5°) hacia
abajo** manteniendo el hue y reduciendo croma, en vez de tomar colores literales.

---

## Tokens propuestos (contraste verificado, no estimado)

```css
:root{
  /* Rampa de profundidad — hue 236.5° medido del cielo */
  --arles-background-deep:      #010E17;
  --arles-background:           #041722;
  --arles-surface:              #0B222F;
  --arles-surface-raised:       #142D3C;
  --arles-surface-interactive:  #1E3A4B;
  --arles-surface-selected:     #204860;

  /* Bordes — separados por función (WCAG 1.4.11 solo exige 3:1 en controles) */
  --arles-border-subtle:        #263843;  /* divisorias decorativas */
  --arles-border-control:       #6F8390;  /* inputs, checkboxes — 4.15:1 sobre surface */

  /* Texto */
  --arles-text-primary:         #ECEEE5;  /* 13.96:1 sobre surface — AAA */
  --arles-text-secondary:       #ACBAC3;  /*  8.23:1 — AAA */
  --arles-text-tertiary:        #85959F;  /*  5.29:1 — AA  */
  --arles-text-disabled:        #5E6B73;
  --arles-text-on-accent:       #071822;

  /* Acentos derivados de la referencia */
  --arles-yellow:               #F5D32B;  /* 11.09:1 — girasol, acento principal */
  --arles-yellow-dim:           #D4B629;
  --arles-gold:                 #E8A127;  /*  7.44:1 */
  --arles-blue-primary:         #009ACB;  /*  5.05:1 */
  --arles-blue-light:           #62BEE7;  /*  7.82:1 */
  --arles-blue-deep:            #006593;
  --arles-cream:                #DFE2D2;  /* 12.42:1 */
  --arles-info:                 #3AACDA;  /*  6.30:1 */
  --arles-warning:              #EFA831;  /*  8.03:1 */
  --arles-focus:                #F5D32B;  /* 12.37:1 sobre background */

  /* EXTENSIONES: no existen en la referencia. Declaradas explícitamente (§168). */
  --arles-success:              #53C48E;  /*  7.52:1 */
  --arles-danger:               #E8605B;  /*  4.87:1 */
  --arles-danger-dim:           #BC4945;
}
```

**Regla de uso del amarillo:** es el color de lo que exige atención o decisión — acción principal,
elemento de navegación activo, anillo de foco, progreso en curso. Nada más. El azul es el color de lo
navegable. Si en una pantalla hay dos cosas amarillas, una está mal (§18).

---

## Trampa tipográfica detectada en `/TIPOGRAFIA`

Los pesos declarados (`usWeightClass`) de este kit **no coinciden** con lo que sugieren sus nombres:

| Archivo | usWeightClass | Consecuencia |
|---|---|---|
| Mont-Light | 400 | `font-weight:400` devuelve **Light**, no Regular |
| Mont-Regular | 600 | Solo se activa pidiendo 600 |
| Mont-SemiBold | 700 | Ocupa el lugar de «bold» |
| Mont-Bold | 800 | — |
| Mont-Heavy / Black | 900 / 950 | 950 requiere mapeo manual |

**Mitigación obligatoria:** declarar cada `@font-face` con su `font-weight` numérico explícito.
Sin esto la aplicación entera se ve mal sin causa aparente.

---

## Estado

**No se ha escrito código de producción.** Pendiente de autorización de Dirección sobre las seis
decisiones de la sección T del dossier.
