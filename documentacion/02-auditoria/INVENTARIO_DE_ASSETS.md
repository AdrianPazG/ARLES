# Inventario forense de assets

**Proyecto:** ARLES RELAY I · v1.2.0
**Fecha de auditoría:** 2026-09-11
**Commit auditado:** `46966e4`
**Estado:** cerrado — los hallazgos están reflejados en ADR-0005 y ADR-0012

---

## 1. Resumen

El repositorio contiene **111 archivos**, todos binarios de referencia, más un `README.md` de 7 bytes. **No hay código de ningún tipo**: ni `package.json`, ni `Cargo.toml`, ni configuración de build, ni CI.

La auditoría encontró cuatro discrepancias materiales entre lo que el brief afirma sobre estos archivos y lo que los archivos realmente son. Ninguna es cosmética: dos afectan a la dirección de diseño, una al riesgo legal de distribución y otra a la integridad del repositorio.

---

## 2. Estructura real

```
/CONCEPTOS_DE_DISEÑO/            27 JPEG, 1277×1600      2.0 MB
/REFERENCIAS_VISUALES_DEL_SITIO/  7 JPEG + 1 MP4         4.7 MB
/TIPOGRAFIA/                     89 archivos             11 MB
/REFERENCIA_DE_COLOR/             1 archivo              9.3 MB
/RECURSOS/
   ├── CONCEPTOS_DE_DISEÑO/       1 JPEG
   ├── REFERENCIAS_VISUALES_DEL_SITIO/  8 archivos
   └── TIPOGRAFIA/                1 archivo             4.9 MB total
README.md                         7 bytes
```

---

## 3. Hallazgo A-01 — `/RECURSOS` es una copia parcial y obsoleta

**Severidad:** media · **Afecta a:** integridad del repositorio, §11 y §12 del brief

El brief designa `/RECURSOS` como la carpeta crítica de referencia, a tratar como sólo lectura. En realidad `/RECURSOS` es un subconjunto degradado de las carpetas de raíz:

| Carpeta | En raíz | En `/RECURSOS` |
|---|---:|---:|
| `CONCEPTOS_DE_DISEÑO` | 27 archivos | 1 archivo |
| `TIPOGRAFIA` | 89 archivos | 1 archivo |
| `REFERENCIAS_VISUALES_DEL_SITIO` | 8 archivos | 8 archivos |

Los archivos solapados son **idénticos** (md5 coincidente, verificado sobre `1.jpeg` de ambas carpetas de conceptos, `1.jpeg` de ambas de referencias visuales y `Mont-Black.eot` de ambas tipografías).

**Riesgo:** con dos ubicaciones para el mismo contenido y una de ellas incompleta, cualquier persona o proceso que siga literalmente la instrucción del brief («inspecciona `/RECURSOS` antes de proponer la UI») verá 1 de 27 conceptos de diseño y concluirá que apenas hay material.

**Recomendación:** ver ADR-0012. `/RECURSOS` pasa a ser fuente única de verdad y se rehidrata desde raíz; las copias de raíz se eliminan. **Pendiente de autorización de Dirección**, porque el brief marcó esas rutas como intocables.

---

## 4. Hallazgo A-02 — La referencia cromática es un PNG renombrado

**Severidad:** baja (técnica) · **Afecta a:** pipeline de assets

`REFERENCIA_DE_COLOR/farm-lifestyle-digital-art.jpg` **no es un JPEG**.

```
Primeros 8 bytes: 89 50 4E 47 0D 0A 1A 0A   ← firma PNG
IHDR: 2320 × 3080, profundidad 8, color type 2 (RGB), sin entrelazado
Tamaño: 9 721 743 bytes
```

Un parser de JPEG aplicado al archivo produce dimensiones absurdas (65245×60914) porque interpreta datos de chunks PNG como marcadores.

**Riesgo:** cualquier herramienta que enrute por extensión (conversores, optimizadores, el propio bundler) fallará o producirá basura silenciosamente.

**Recomendación:** renombrar a `.png` al consolidar `/RECURSOS`. No convertir ni recomprimir: es el original.

---

## 5. Hallazgo A-03 — La referencia cromática es stock generado por IA

**Severidad:** **alta** · **Afecta a:** T-9, propiedad intelectual de la marca

El XMP incrustado en el archivo declara:

```xml
Iptc4xmpExt:DigitalSourceType =
  "http://cv.iptc.org/newscodes/digitalsourcetype/trainedAlgorithmicMedia"
xmp:CreateDate   = "2024-05-17T10:35:55+07:00"
xmp:CreatorTool  = "Adobe Photoshop 25.7 (Macintosh)"
Iptc4xmpExt:DigImageGUID = "5197c1cc-ba98-4024-9f0b-3ad3c0678314"
```

`trainedAlgorithmicMedia` es el código IPTC estándar para **contenido generado por un modelo entrenado**. El nombre del archivo (`farm-lifestyle-digital-art`) y la presencia en el repositorio de `video-from-rawpixel-id-17145707-sd.mp4` señalan un origen de banco de imágenes rawpixel.

**Contradice T-9**, que afirma que TELEMETRY INSIGHT posee todos los derechos sobre marcas, logos y paletas. Este archivo es stock de un tercero, no un activo propio.

**Matiz legal importante:** los colores y las combinaciones de colores **no son protegibles por derecho de autor**. Extraer una paleta de esta imagen es legalmente limpio. Lo que no se puede hacer es (a) distribuir el archivo dentro del producto, (b) presentarlo como activo de marca propiedad de TELEMETRY, ni (c) construir el logotipo o material comercial a partir de él.

**Recomendación:** ver D-2 y ADR-0005. La imagen queda como referencia **interna** de estudio. No se distribuye, no se versiona dentro del bundle, y `08-legal/LICENCIAS_DE_TERCEROS.md` registra su procedencia.

---

## 6. Hallazgo A-04 — La paleta real no corresponde a la dirección del brief

**Severidad:** **alta** · **Afecta a:** §14, §16, §18 del brief

Decodifiqué el PNG completo (21 439 880 bytes sin comprimir) y muestreé 1 027 líneas de barrido, cuantificando a 5 bits por canal.

### Distribución por familia cromática

| Familia | Rango de tono | Presencia |
|---|---|---:|
| Cian / azur | H 185–210° | **54.35 %** |
| Ocre / naranja | H 15–38° | 19.48 % |
| Amarillo / oro | H 38–62° | 13.44 % |
| Teal | H 160–185° | 5.39 % |
| Verde | H 62–160° | 3.31 % |
| Neutro | S < 12 | 2.18 % |
| Rojo | H < 15 / > 330° | 0.89 % |
| **Azul profundo** | **H 210–250°** | **0.73 %** |
| Violeta | H 250–330° | 0.22 % |

### Distribución de luminosidad (HSL)

| L | Presencia |
|---|---:|
| 0–10 | 1.01 % |
| 10–20 | 7.31 % |
| 20–30 | 21.45 % |
| 30–40 | 22.05 % |
| 40–50 | 16.67 % |
| 50–60 | 16.52 % |
| 60–70 | 5.70 % |
| 70–80 | 3.16 % |
| 80–90 | 4.71 % |
| 90–100 | 1.41 % |

### Lectura

**No es una paleta de *La noche estrellada*.** El azul profundo —el color que define el cuadro— está prácticamente ausente: 0.73 %. La imagen es una composición complementaria cian-contra-cálido, más cercana a un paisaje al atardecer que a un cielo nocturno.

Esto tiene dos consecuencias que el brief no anticipa:

**1. No hay separación de tono disponible.** El §16 asigna tres roles distintos a tres azules distintos: azul profundo para estructura, azul medio para superficies, cian para interacción. Pero el 54 % de la imagen cabe en una banda de ±12°. Extraer literalmente esos tres roles los colapsa en la misma familia, y el resultado es una interfaz monocromática cian donde nada distingue visualmente «esto es el chasis» de «esto se puede pulsar».

**2. El fondo profundo hay que fabricarlo.** El §18 pide dark-first construido por capas de azul. Sólo el 1 % de los píxeles está por debajo de L 10, y los oscuros que existen son o bien cian apagado (`#041C2C`, `#042C44`) o bien tierras cálidas turbias (`#341C14`, `#4C2C14`). Ninguno sirve como fondo de aplicación tal cual.

### Colores ancla extraídos

| Hex | RGB | H / S / L | Presencia | Rol propuesto |
|---|---|---|---:|---|
| `#045484` | 4, 84, 132 | 202 / 94 / 27 | 3.11 % | Superficie base |
| `#0C749C` | 12, 116, 156 | 197 / 86 / 33 | 3.12 % | Superficie elevada |
| `#2CA4D4` | 44, 164, 212 | 197 / 66 / 50 | 0.81 % | Interacción (sólo borde/texto) |
| `#FCCC0C` | 252, 204, 12 | 48 / 98 / 52 | 0.30 % | Acento |
| `#F4ECE4` | 244, 236, 228 | 30 / 42 / 93 | 0.33 % | Texto primario |
| `#E4E4CC` | 228, 228, 204 | 60 / 31 / 85 | 0.36 % | Texto secundario |
| `#AC5C0C` | 172, 92, 12 | 30 / 87 / 36 | 0.27 % | Advertencia cálida |
| `#041C2C` | 4, 28, 44 | 204 / 83 / 9 | 0.11 % | Fondo profundo (a extender) |

**Nota metodológica sobre §16:** el brief advierte, con razón, que no se seleccionen colores por frecuencia de píxel. El amarillo ocupa el 0.30 % y es fundamental. Por eso la tabla anterior asigna roles por **función**, no por presencia.

---

## 7. Hallazgo A-05 — Contraste: el cian brillante no puede cargar texto

**Severidad:** **alta** · **Afecta a:** §19 (WCAG 2.2 AA)

Ratios calculados con la fórmula de luminancia relativa WCAG 2.2:

| Superficie | `#F4ECE4` | `#E4E4CC` | `#FCCC0C` | Blanco |
|---|---|---|---|---|
| `#041C2C` profundo | **14.88:1** ✅ | **13.47:1** ✅ | **11.41:1** ✅ | **17.39:1** ✅ |
| `#045484` superficie | **6.89:1** ✅ | **6.24:1** ✅ | **5.29:1** ✅ | **8.06:1** ✅ |
| `#0C749C` elevada | 4.50:1 ⚠️ | 4.07:1 ⚠️ | 3.45:1 ❌ | **5.26:1** ✅ |
| `#2CA4D4` cian vivo | 2.44:1 ❌ | 2.21:1 ❌ | 1.87:1 ❌ | 2.85:1 ❌ |

Dos conclusiones operativas:

- **`#2CA4D4` no es una superficie.** Falla contra todos los textos candidatos, blanco incluido. Sólo puede usarse como color de trazo, de borde, de foco o de texto **sobre** fondos profundos.
- **`#FCCC0C` se comporta como un color claro.** Su luminancia relativa es 0.639 — más cerca del blanco (1.0) que del negro. Funciona magníficamente como acento sobre `#041C2C` (11.41:1) y es inutilizable como relleno de botón con texto blanco encima (1.56:1).

Esto invierte una intuición común: el amarillo de ARLES no es un color «de señal fuerte sobre claro», es un color **de luz sobre oscuro**. Encaja con la estrategia dark-first, pero prohíbe el patrón «botón amarillo con texto blanco».

**Reproducibilidad:** el método completo está en `05-diseno/COLOR_SYSTEM.md`.

---

## 8. Hallazgo A-06 — No existe ningún activo de marca de ARLES

**Severidad:** **alta** · **Afecta a:** §12, §21, T-9

El brief describe `/RECURSOS` como contenedor de «referencias de interfaz, estilo de UI, identidad de marca, tipografía y activos gráficos», y T-9 menciona logos concretos («Arrow, Shield»).

**No existe ninguno.** No hay logotipo, ni isotipo, ni flecha, ni escudo, ni mockup de ARLES, ni archivo de paleta, ni guía de marca. Ni en `/RECURSOS` ni en raíz.

Lo que sí hay, y qué es realmente:

### `/CONCEPTOS_DE_DISEÑO` — 27 imágenes

Carruseles de consejos de UX publicados en redes sociales, con marca de agua de **@ux_snacks** y **@uxwithvamshi**. Contenido verificado por muestreo: `1.jpeg` es una comparativa «Radio button VS Card» con etiquetas Don't/Do; `20.jpeg` es directamente un **anuncio** de un playbook de UX de pago («Join +2000 UX Designers…»).

Valor real: heurística genérica de UX, útil como recordatorio. **No son referencias de la identidad de ARLES** y no deben tratarse como tales.

### `/REFERENCIAS_VISUALES_DEL_SITIO` — 7 imágenes + 1 vídeo

Mockups de terceros recolectados de Pinterest/Dribbble. Verificados: un dashboard fintech de marca «COINEST» (© 2024 Peterdraw) y una lámina de design system de «Milray Park». Más un vídeo de rawpixel.

Valor real: referencia de **densidad, ritmo, jerarquía tipográfica y tratamiento de tablas** — exactamente el uso que el §12 prescribe («analiza las proporciones, la densidad, la navegación… no copies literalmente»). Son IP de terceros y no pueden reproducirse.

**Recomendación:** Dirección debe entregar los activos de marca (o encargarlos) antes de que la Fase 2 pueda cerrar el Design System. Mientras tanto, el logotipo se trata como puramente tipográfico según §21, lo que lo hace dependiente de D-3.

---

## 9. Hallazgo A-07 — La tipografía tiene un riesgo legal de distribución

**Severidad:** **crítica** · **Afecta a:** §21, T-2 · **Bloqueante:** D-3

`/TIPOGRAFIA` contiene 89 archivos: la familia Mont completa en `.eot`, `.ttf`, `.woff` y `.woff2`, más `font.zip` (66 entradas, 4.16 MB) y `demo.html`.

### Procedencia

La tabla `name` de `Mont-Regular.ttf` identifica inequívocamente el origen:

```
Familia:      Mont
Subfamilia:   Regular
Versión:      1.003;PS 001.003;hotconv 1.0.88;makeotf.lib2.5.64775
Diseñador:    Svetoslav Simov
Créditos:     Svetoslav Simov, Mirela Belova
Fundición:    http://fontfabric.com/
Descripción:  "Modern and elegant sans serif font family."
```

Mont es una fuente comercial de **Fontfabric**.

### El problema

`demo.html` contiene `<title>Transfonter demo</title>`. Transfonter es un **conversor de webfonts online**. La presencia de `.eot` —un formato muerto desde IE11— junto a un kit generado por Transfonter es el perfil característico de un paquete descargado de un agregador de fuentes, no de una entrega comercial de Fontfabric (que suministra OTF/TTF de escritorio y, por separado, webfonts con licencia propia).

**No hay ningún archivo de licencia, factura ni comprobante en el repositorio.**

### Por qué importa para ARLES concretamente

T-2 afirma: «poseemos oficialmente la licencia comercial de Mont para uso de texto». Aun asumiendo que sea cierto, el alcance importa:

| Tipo de licencia Fontfabric | ¿Cubre incrustar la fuente en un `.exe` / `.dmg` distribuido? |
|---|---|
| Desktop | No |
| Web | No |
| **App** | **Sí** — es la que se necesita, con precio por aplicación |

Un bundle de Tauri lleva los archivos de fuente dentro del binario que se instala en la máquina del cliente. Eso es exactamente el supuesto de la App License. Una licencia Desktop o Web **no lo autoriza**.

El §21 del propio brief instruye: «si la fuente tiene problemas de licencia para distribución comercial, detente y señálalo». Esto es ese caso.

**Decisión tomada (D-3):** Dirección verifica el alcance exacto de la licencia que posee y, si no cubre app embedding, la adquiere. El comprobante se archiva en `08-legal/`. **El Design System no se congela hasta entonces.** El plan B —Mont sólo en marketing, logotipo entregado como SVG con contornos (uso legítimo de una licencia de escritorio), y una geométrica de licencia libre dentro de la aplicación— queda documentado en `05-diseno/TIPOGRAFIA.md`.

---

## 10. Tabla resumen

| Id | Hallazgo | Severidad | Resolución |
|---|---|---|---|
| A-01 | `/RECURSOS` es copia parcial obsoleta | Media | ADR-0012 · pendiente de autorización |
| A-02 | Referencia cromática es PNG renombrado | Baja | Renombrar al consolidar |
| A-03 | Referencia cromática es stock generado por IA | Alta | D-2 · ADR-0005 · uso interno únicamente |
| A-04 | La paleta real no es la del brief (0.73 % azul profundo) | Alta | D-2 · rampas derivadas |
| A-05 | `#2CA4D4` falla WCAG como superficie | Alta | `COLOR_SYSTEM.md` · rol restringido |
| A-06 | No existe ningún activo de marca de ARLES | Alta | Pendiente de Dirección |
| A-07 | Licencia de Mont no cubre app embedding | **Crítica** | **D-3 · bloqueante de Fase 2** |
