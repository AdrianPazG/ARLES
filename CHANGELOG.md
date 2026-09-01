# Changelog

Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).
Versionado según [Semantic Versioning](https://semver.org/lang/es/).

## [No publicado] — Fase 0 · Descubrimiento

Sin código de producción. Esta entrada registra la fase de descubrimiento, auditoría y arquitectura.

### Añadido
- Documentación de descubrimiento, arquitectura, sistema de color, modelo de amenazas y preguntas abiertas.
- ADR-0001 a ADR-0008.
- `tools/extract_palette.py` — extracción reproducible de la paleta desde la referencia oficial.

### Seguridad
- Identificados dos bloqueos previos al desarrollo: licencia tipográfica de Mont y ausencia de datos de marca.
- Modelo de amenazas preliminar con seis fronteras de confianza.
- Detectado vector de inyección de encabezado vía variables de personalización interpoladas en el asunto
  (ver `docs/THREAT_MODEL.md`).

### Notas
- La numeración de la primera versión pública está pendiente de decisión. Ver `docs/OPEN_QUESTIONS.md` #5.
  Si se publica como 1.2.0 sin que hayan existido 1.0.0 ni 1.1.0, debe quedar registrado aquí explícitamente.
