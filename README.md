# ARLES RELAY

Aplicación de escritorio para la ejecución y gestión controlada de campañas de correo.
Desarrollada por **TELEMETRY INSIGHT**.

- **Versión objetivo:** v1.2.0 — ver [pregunta abierta #5](docs/OPEN_QUESTIONS.md)
- **Plataformas:** Windows y macOS. No es una aplicación móvil ni una PWA.
- **Estado actual:** Fase 0 — Descubrimiento y arquitectura. **No hay código de producción.**

## Estado del proyecto

| Fase | Estado |
|---|---|
| 0 · Descubrimiento y auditoría | Entregado — pendiente de autorización de Dirección |
| 0.5 · Legal y compras | **No iniciada. Debe arrancar de inmediato** (ver [RIESGOS](docs/DISCOVERY.md#riesgos)) |
| 1 · Arquitectura | Bloqueada hasta autorización |

## Dos bloqueos activos

1. **Licencia tipográfica.** La familia Mont incluida en `/TIPOGRAFIA` es de Fontfabric y el paquete no
   contiene licencia. Distribuirla dentro de un instalador requiere licencia de aplicación.
   **No debe incrustarse hasta tener comprobante.**
2. **Datos de marca ausentes.** No existe logotipo, teléfono, correo ni sitio web de TELEMETRY INSIGHT
   en el repositorio. Los apartados §118 y §170 del brief los exigen y §145 prohíbe inventarlos.

Detalle completo en [`docs/DISCOVERY.md`](docs/DISCOVERY.md).

## Documentación

| Documento | Contenido |
|---|---|
| [DISCOVERY.md](docs/DISCOVERY.md) | Inventario del repositorio, hallazgos y contradicciones |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Arquitectura, dominios, modelo de datos y motor de ejecución |
| [COLOR_SYSTEM.md](docs/COLOR_SYSTEM.md) | Extracción de paleta, tokens y verificación de contraste |
| [THREAT_MODEL.md](docs/THREAT_MODEL.md) | Fronteras de confianza, amenazas y controles |
| [OPEN_QUESTIONS.md](docs/OPEN_QUESTIONS.md) | Decisiones pendientes de Dirección |
| [DECISIONS/](docs/DECISIONS/) | Architecture Decision Records |

## Carpetas de solo lectura

`/RECURSOS`, `/REFERENCIA_DE_COLOR`, `/TIPOGRAFIA`, `/CONCEPTOS_DE_DISEÑO` y
`/REFERENCIAS_VISUALES_DEL_SITIO` **no deben modificarse, moverse, renombrarse ni comprimirse**.

## Reproducir el análisis de color

```
pip install pillow numpy
python3 tools/extract_palette.py
```

Lee `REFERENCIA_DE_COLOR/` sin modificarlo y reproduce las cifras de `docs/COLOR_SYSTEM.md`.
