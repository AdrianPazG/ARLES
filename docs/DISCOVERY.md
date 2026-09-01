# Descubrimiento — Fase 0

Inventario e inspección del repositorio previos a cualquier decisión de arquitectura o diseño.
Fecha: 2026-09-01.

## 1. Inventario

| Ruta | Contenido | Archivos | Lectura |
|---|---|---:|---|
| `/REFERENCIA_DE_COLOR` | `farm-lifestyle-digital-art.jpg` | 1 | Autoridad cromática. Usable. |
| `/TIPOGRAFIA` | Familia Mont, 16 pesos × 4 formatos, + `demo.html`, `font.zip` | 66 | **Bloqueo de licencia** |
| `/CONCEPTOS_DE_DISEÑO` | Carruseles de terceros sobre buenas prácticas de UI | 27 | No es material de marca |
| `/REFERENCIAS_VISUALES_DEL_SITIO` | 6 dashboards SaaS, 2 hojas de design system, 1 video | 8 | Referencia de densidad |
| `/RECURSOS` | Espejo **parcial** de las tres anteriores | 10 | **Contradicción** |
| `README.md` | 7 bytes | 1 | Vacío |

No existe código fuente. El proyecto es un campo verde.

## 2. Bloqueo 1 — Licencia tipográfica

**Evidencia.** Lectura de la tabla `name` del binario `TIPOGRAFIA/Mont-Black.ttf`:

```
Family      : Mont Black
UniqueID    : 1.003;FBRC;Mont-Black
Manufacturer: Svetoslav Simov
Designer    : Svetoslav Simov, Mirela Belova
VendorURL   : http://fontfabric.com/
```

`TIPOGRAFIA/demo.html` declara `<title>Transfonter demo</title>`: los archivos fueron convertidos con
Transfonter, una herramienta de conversión a fuentes web. `font.zip` (fechado 2019-01-10) **no contiene
ningún archivo de licencia**.

**Problema.** Una licencia de fuente *web* o *desktop* normalmente no cubre la incrustación de binarios
dentro de una aplicación distribuida e instalada en equipos de terceros. Eso requiere una licencia de
aplicación / *embedding*.

**Riesgo.** Distribuir el instalador con Mont incrustada sin la licencia correcta constituye incumplimiento
en cada instalación vendida. El costo de remediación crece con cada cliente.

**Recomendación.** Adquirir la licencia de aplicación a Fontfabric antes de la Fase 2. Es la única opción
que preserva la identidad ya elegida. **Mientras no exista comprobante, Mont no se incrusta en ningún
entregable.**

## 3. Bloqueo 2 — Datos de marca ausentes

Búsqueda exhaustiva en el repositorio: no existe logotipo, archivo SVG, ni ningún archivo cuyo nombre
contenga `logo`, `brand` o `telemetry`. No hay teléfono, correo ni sitio web en ningún archivo.

Los apartados §118 y §170 del brief exigen un pie con logotipo, número, correo y web; §145 prohíbe
inventarlos e indica buscarlos en `/RECURSOS`. No están ahí.

**Recomendación.** Dirección entrega: logotipo vectorial (versión clara y oscura), teléfono en formato
internacional, correo de soporte y URL canónica. Hasta entonces, el pie de la aplicación queda como
marcador visible, no con datos de relleno.

## 4. Contradicción — `/RECURSOS` es un espejo parcial

Comparación por hash MD5: `RECURSOS/CONCEPTOS_DE_DISEÑO/1.jpeg`, `RECURSOS/TIPOGRAFIA/Mont-Black.eot` y
`RECURSOS/REFERENCIAS_VISUALES_DEL_SITIO/1.jpeg` son **byte a byte idénticos** a los homónimos de la raíz.
Pero `/RECURSOS` contiene 1 de 27 conceptos y 1 de 66 archivos de fuente: es una subida interrumpida.

El brief trata `/RECURSOS` como autoritativa. Construir el design system leyendo esa carpeta significaría
trabajar con 1 de 27 referencias y un solo peso tipográfico.

**Recomendación.** Que Dirección consolide en una sola jerarquía. **No se ejecuta automáticamente**: §11
prohíbe mover, renombrar o borrar esos archivos, y una subida incompleta es exactamente el caso donde una
operación automática pierde material. Entre tanto se trabaja con la raíz, que es la copia completa.

## 5. `/CONCEPTOS_DE_DISEÑO` — qué es realmente

No son conceptos de ARLES. Son carruseles guardados de dos cuentas de divulgación de UX
(`@uxwithvamshi`, "Do's and Dont's of UI Design" partes 5 y 6; `@pixselacademy`, "Tips to design better
UI Cards"), con marca de agua de sus autores y una llamada comercial.

Son útiles como **checklist de reglas de UI**, no como identidad. Reglas extraídas y adoptadas:

| Regla | Aplicación en ARLES |
|---|---|
| Etiqueta visible, nunca placeholder como etiqueta | Todos los formularios |
| `(opcional)` en vez de asterisco en obligatorios | Empresa, remitente, SMTP |
| Formularios de una sola columna | Asistente de campaña, alta SMTP |
| Un CTA primario por vista | Un solo botón amarillo por pantalla |
| Diferenciar placeholder / etiqueta / valor | Tres tonos distintos |
| Sin MAYÚSCULAS en botones | Capitalización normal |
| Radio interior menor que el exterior | Escala 4 / 8 / 12 / 16 px |
| Error junto al campo, no agrupado | Estado bajo cada campo |

**Advertencia de propiedad intelectual.** Llevan marca de agua y no consta licencia de uso. No deben
redistribuirse, incluirse en material comercial ni copiarse estéticamente. Se recomienda moverlas a un
espacio de investigación fuera del repositorio del producto.

## 6. `/REFERENCIAS_VISUALES_DEL_SITIO` — dirección aprovechable

Seis dashboards de producto y dos hojas de design system. Lo que se adopta: riel de iconos con barra
lateral etiquetada; fila de KPIs arriba y detalle abajo; tablas densas con chips de estado (filas de
34–38 px, no 56); panel-resumen a la derecha; radios contenidos y un solo acento.

**Tensión declarada.** Las ocho referencias son de tema claro con acentos verdes y naranjas; el brief pide
dark-first azul y amarillo. No se contradicen porque aportan cosas distintas: de esta carpeta se toma
composición, densidad y ritmo; de `/REFERENCIA_DE_COLOR` se toma el color.

## 7. Riesgos

| Riesgo | Prob. | Impacto | Mitigación |
|---|---|---|---|
| La verificación OAuth de Google retrasa el lanzamiento | Alta | Crítico | Iniciar trámite de inmediato. Google tras bandera de función. Lanzar con SMTP y Microsoft. |
| Licencia de Mont sin resolver | Media | Crítico | Comprar licencia de aplicación antes de la Fase 2. |
| Mensajes duplicados tras una caída | Baja | Crítico | Idempotencia + toma condicional + estado `unknown` con decisión humana. |
| Fuga de token por malware local | Baja | Crítico | No evitable del todo. Reducir daño: scope mínimo, revocación real, límite documentado. |
| Cuenta restringida por el proveedor | Media | Alto | Disyuntor, obediencia a `Retry-After`, advertencia de volumen, sin evasión. |
| Diferencias WebView2 / WKWebView | Media | Alto | E2E en ambas plataformas desde la Fase 3. Versión mínima de macOS declarada. |
| El alcance de v1.2.0 no cabe en calendario | Muy alta | Alto | Adoptar los 20 pasos del §171 como contrato de versión. |
| Corrupción de base o disco lleno | Baja | Crítico | WAL, transacciones, verificación de espacio, respaldo previo a migración. |
| Robo del archivo de respaldo | Baja | Alto | Cifrado autenticado con frase del usuario; credenciales excluidas. |
| Uso indebido por un cliente | Media | Medio | Detección de anomalías, advertencias registradas, sin funciones de evasión. |
| Cumplimiento de datos personales en México | Media | Alto | El producto facilita, no garantiza. Textos revisados por abogado antes de vender. |

## 8. Método

Herramientas usadas: lectura directa de bytes para verificar formatos reales; parser propio de la tabla
`name` de OpenType para las fuentes; lectura de XMP para la imagen de referencia; k-means en CIELAB y
análisis por regiones para la paleta; cálculo de contraste WCAG 2.2 sobre luminancia relativa.

**Ningún archivo de las carpetas de referencia fue modificado, movido ni renombrado.**
