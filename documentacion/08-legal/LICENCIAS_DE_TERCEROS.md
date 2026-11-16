# Licencias de terceros

**Proyecto:** ARLES RELAY I · v1.2.0
**Última revisión:** 2026-09-11

> ⚠️ **No es asesoría legal.** Es el registro de lo que hay en el repositorio, su procedencia verificada y su situación de licencia. Los puntos marcados requieren validación jurídica de TELEMETRY INSIGHT.

---

## 1. Estado

| Activo | Situación | Puerta |
|---|---|---|
| **Mont (Fontfabric)** | 🟡 **Licencia no verificada** · desarrollo autorizado (D-5) | **Antes de la demo** |
| Referencia cromática | 🟡 Stock de IA de terceros — uso interno | No |
| Conceptos de diseño | 🟡 IP de terceros — estudio | No |
| Referencias visuales | 🟡 IP de terceros — estudio | No |
| Dependencias Rust y npm | ⚪ Por auditar en Fase 1 | No |

---

## 2. Mont — Fontfabric 🟡

**Decisiones D-3 y D-5. Riesgo R-01. Pregunta P-01.**

> **Qué aplica hoy (D-5, 2026-09-11).** Se desarrolla con Mont con normalidad. Lo que sigue requiriendo la App License es **incrustar el binario de la fuente en un artefacto que se instala o se enseña como producto**. Mientras el binario no salga del equipo de desarrollo, no hay supuesto de distribución.
>
> | Uso | ¿Autorizado hoy? |
> |---|---|
> | Documentos, láminas, maquetas | **Sí** — uso de documento |
> | Compilaciones en máquinas del equipo | **Sí** |
> | Logotipo como SVG con contornos | **Sí** — es geometría, no contiene la fuente |
> | **Instalar la app con Mont incrustada fuera del equipo** | **No** |
> | **Enseñar la app a un cliente o prospecto** | **No** |
>
> La pregunta se vuelve a plantear al planificar la demo (Fase 9).

### Qué hay en el repositorio

`/TIPOGRAFIA`, 89 archivos: la familia completa en `.eot`, `.ttf`, `.woff` y `.woff2`, más `font.zip` (66 entradas, 4.16 MB) y `demo.html`.

### Procedencia verificada

Leyendo la tabla `name` de `Mont-Regular.ttf`:

```
Familia:      Mont
Versión:      1.003;PS 001.003;hotconv 1.0.88;makeotf.lib2.5.64775
Diseñador:    Svetoslav Simov
Créditos:     Svetoslav Simov, Mirela Belova
Fundición:    http://fontfabric.com/
```

Fuente comercial de **Fontfabric**.

### El indicio que preocupa

`demo.html` contiene `<title>Transfonter demo</title>`. Transfonter es un conversor de webfonts online. La presencia de `.eot` —formato muerto desde IE11— junto a un kit generado por Transfonter es el perfil característico de un paquete descargado de un agregador de fuentes, no de una entrega comercial de Fontfabric, que suministra OTF/TTF de escritorio y, por separado, webfonts con licencia propia.

**No hay licencia, factura ni comprobante en el repositorio.**

### Por qué importa para ARLES en concreto

T-2 afirma que TELEMETRY posee «la licencia comercial de Mont para uso de texto». El alcance decide:

| Licencia de Fontfabric | ¿Cubre incrustar la fuente en el `.exe` / `.dmg` distribuido? |
|---|---|
| Desktop | **No** |
| Web | **No** |
| **App** | **Sí** — es la que ARLES necesita |

Un bundle de Tauri lleva los archivos de fuente **dentro del binario que se instala en la máquina del cliente**. Ése es exactamente el supuesto de la App License.

El §21 del brief ordena detenerse ante esto.

### Qué se necesita

1. Tipo exacto de licencia adquirida por TELEMETRY.
2. Comprobante (factura o contrato) en `documentacion/08-legal/comprobantes/`.
3. Si no cubre app embedding: adquirir la App License, o activar el plan B.

### Plan B

**Se activa automáticamente si al inicio de la Fase 2 no hay comprobante.** Detalle en `05-diseno/TIPOGRAFIA.md`:

- Mont para marketing y material comercial (cubierto por Desktop/Web)
- **Logotipo como SVG con contornos** — uso legítimo de una licencia de escritorio: el archivo es geometría vectorial y **no contiene la fuente**
- Interfaz con **Figtree** (SIL OFL), geométrica de métricas similares

Coste bajo: la escala y los tokens no cambian, sólo la familia.

---

## 3. Referencia cromática 🟡

`REFERENCIA_DE_COLOR/farm-lifestyle-digital-art.jpg`

### Qué es realmente

- **Un PNG**, no un JPEG. 2320×3080, RGB 8 bits. Renombrado.
- **Contenido generado por IA.** Su XMP declara `Iptc4xmpExt:DigitalSourceType = trainedAlgorithmicMedia`, el código IPTC estándar para media sintética. Creado 2024-05-17, Photoshop 25.7 Mac.
- **Stock de terceros.** El nombre y el vídeo rawpixel hermano en `/REFERENCIAS_VISUALES_DEL_SITIO` apuntan a rawpixel.

### Contradice T-9

T-9 afirma que TELEMETRY posee todos los derechos sobre marcas, logotipos y paletas. **Este archivo no es un activo propio.**

### Pero el matiz es favorable

**Los colores y sus combinaciones no son protegibles por derecho de autor.** Extraer una paleta de esta imagen no crea ninguna obligación (D-2, ADR-0005).

### Qué sí y qué no

| Sí | No |
|---|---|
| Estudiarla y medirla | Distribuirla dentro del producto |
| Extraer una paleta | Presentarla como activo de marca |
| Conservarla como referencia interna | Usarla en material comercial |
| | Derivar el logotipo de ella |

**Recomendación para v1.3:** encargar una pieza cromática original, o que Dirección entregue un brandbook propio. Elimina la contradicción con T-9 y limpia el linaje de la marca. Registrado en P-02.

---

## 4. Conceptos y referencias visuales 🟡

### `/CONCEPTOS_DE_DISEÑO` — 27 imágenes

Carruseles de consejos de UX de redes sociales, con marca de agua de **@ux_snacks** y **@uxwithvamshi**. `20.jpeg` es directamente un anuncio de un playbook de pago.

**Uso legítimo:** leerlos como heurística general de UX.
**No son** referencias de la identidad de ARLES, pese a lo que el §12 sugiere.

### `/REFERENCIAS_VISUALES_DEL_SITIO` — 7 imágenes + 1 vídeo

Mockups de terceros de Pinterest y Dribbble: un dashboard fintech de marca «COINEST» (© 2024 Peterdraw) y una lámina de design system de «Milray Park». Más un vídeo de rawpixel.

**Uso legítimo:** exactamente el que prescribe el §12 — estudiar proporciones, densidad, navegación, jerarquía y tratamiento de tablas.

**Prohibido:** reproducir literalmente cualquier composición, o incorporar estos archivos al producto.

> El §12 ya lo dice: «no copies literalmente otras interfaces». Es a la vez una instrucción de diseño y la única postura defendible sobre IP ajena.

---

## 5. Dependencias de software ⚪

Se auditan en la Fase 1 y se mantiene un inventario generado automáticamente.

### Política de licencias

| Permitidas | MIT, Apache-2.0, BSD (2 y 3 cláusulas), ISC, Zlib, Unicode-3.0 |
|---|---|
| **Requieren revisión** | MPL-2.0, LGPL |
| **Prohibidas** | GPL, AGPL |

**Por qué se prohíbe GPL/AGPL:** ARLES se distribuye como binario comercial. Enlazar código GPL obligaría a liberar el código fuente del producto.

### Automatización

```
cargo-deny     licencias, avisos de seguridad y duplicados de Rust
license-checker  licencias de npm
```

Ambos en CI. **Una licencia prohibida rompe la compilación**, no se descubre en una revisión.

### Dependencias notables previstas

| Dependencia | Licencia | Nota |
|---|---|---|
| Tauri 2 | MIT / Apache-2.0 | ✅ |
| rusqlite + SQLCipher | MIT / **BSD-3 (SQLCipher)** | ✅ SQLCipher Community es BSD. **Verificar que se usa Community y no una edición comercial** |
| OpenSSL (vendored) | Apache-2.0 | ✅ desde la 3.0 |
| lettre | MIT | ✅ |
| calamine | MIT | ✅ |
| ammonia | MIT / Apache-2.0 | ✅ |
| Vue 3, Pinia, Vite | MIT | ✅ |
| TanStack Virtual | MIT | ✅ |
| Lucide (iconos) | ISC | ✅ |
| Figtree (plan B) | SIL OFL 1.1 | ✅ permite incrustación |

**El punto a verificar es SQLCipher:** la edición Community es BSD y sirve; Zetetic ofrece además ediciones comerciales con otras condiciones. Hay que confirmar cuál se enlaza.

---

## 6. Atribución en el producto

`AJUSTES → Acerca de → Licencias de terceros` muestra el inventario completo con sus textos de licencia, generado automáticamente en el build.

Requisito de MIT, Apache-2.0 y BSD — y buena práctica en cualquier caso.

---

## 7. Pendiente de Dirección y Legal

| # | Asunto | Puerta |
|---|---|---|
| 1 | **Licencia de Mont: tipo y comprobante** (P-01) | **Antes de la demo** (D-5) |
| 2 | ~~Activos de marca de ARLES (P-02)~~ | ✅ cerrada por D-5 |
| 3 | Confirmar que se enlaza SQLCipher Community | Fase 1 |
| 4 | Reparto responsable/encargado en los términos de licencia | v1.3 |
| 5 | Aviso de privacidad para la verificación de Google (P-05) | Fase 5 |

---

## 8. Dónde van los comprobantes

```
documentacion/08-legal/
├── LICENCIAS_DE_TERCEROS.md   este documento
└── comprobantes/              ← facturas y contratos
    └── .gitkeep
```

**Los comprobantes no contienen secretos** —son facturas y contratos de licencia— así que pueden versionarse. Si alguno incluyera datos sensibles (números de cuenta, datos personales), se archiva fuera del repositorio y aquí queda sólo la referencia.
