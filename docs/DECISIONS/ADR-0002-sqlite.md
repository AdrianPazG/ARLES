# ADR-0002 — SQLite en modo WAL como almacenamiento

**Estado:** Propuesto · **Fecha:** 2026-09-01

## Contexto
Producto local-first, monousuario por instalación, con un motor de ejecución escribiendo continuamente
mientras la interfaz lee. Volumen objetivo: hasta ~100 mil contactos por organización.

## Decisión
**SQLite** con `journal_mode = WAL`, `foreign_keys = ON` y `busy_timeout` configurado.

## Alternativas
- **PostgreSQL embebido:** exige gestionar un servidor dentro de la instalación; complica instalador,
  desinstalación y respaldo. Desproporcionado para el volumen.
- **Archivos JSON o similar:** sin transacciones ni restricciones. Cierra por sí solo la posibilidad de
  garantizar idempotencia y supresión mediante índices únicos.

## Consecuencias

**Positivas.** Un archivo respaldable y copiable; transacciones reales; restricciones `UNIQUE` que
convierten requisitos del brief en propiedades estructurales; cero infraestructura en el equipo del cliente.

**Negativas.** SQLite admite un solo escritor. Un motor de larga duración más una interfaz que lee producen
`SQLITE_BUSY` si se gestiona mal — caso obligatorio del §108.

**Mitigación:** WAL permite lecturas concurrentes con una escritura; el motor mantiene la **única** conexión
de escritura (ver [ADR-0004](ADR-0004-motor-en-proceso.md)); `busy_timeout` cubre los solapes cortos.

**Nota.** `PRAGMA foreign_keys` viene **apagado** por omisión en SQLite y debe activarse en cada conexión.
Ese detalle ha arruinado más bases de datos que cualquier otro.
