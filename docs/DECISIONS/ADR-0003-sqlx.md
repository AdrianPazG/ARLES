# ADR-0003 — `sqlx` en lugar de un ORM

**Estado:** Propuesto · **Fecha:** 2026-09-01

## Contexto
El acceso a datos de un producto que será auditado en seguridad y que debe garantizar integridad
transaccional en el motor de envío.

## Decisión
**`sqlx`** con verificación de consultas en tiempo de compilación. Migraciones versionadas.

## Alternativas
- **Diesel:** DSL con macros pesadas; más fricción con particularidades de SQLite.
- **SeaORM:** capa de abstracción adicional que aporta poco en un esquema conocido y estable.
- **SQL crudo sin verificación:** pierde la detección de deriva de esquema en CI.

## Consecuencias

**Positivas.**
- *Auditabilidad:* las consultas son SQL literal, legible por un revisor de seguridad. Un DSL obliga a
  auditar también el generador.
- *Verificación en compilación:* una migración que rompe una consulta falla en CI, no en casa del cliente.
- *Inyección SQL cerrada por construcción:* los parámetros son parámetros; no hay ruta de concatenación.
  Convierte un punto del modelo de amenazas en propiedad estructural en vez de disciplina que recordar.

**Negativas.** Exige una base disponible al compilar, o un archivo de metadatos preparado. Añade un paso a
la configuración de CI. Se acepta a cambio de lo anterior.
