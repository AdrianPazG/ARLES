# ADR-0002 · rusqlite + SQLCipher + refinery

**Estado:** aceptado · **Fecha:** 2026-09-11 · **Decide:** Tech Lead + Database

---

## Contexto

El brief pide evaluar SQLite como almacén primario y proponer «la alternativa de ORM / acceso a datos más segura y mantenible» (§9, §31).

Dos restricciones acotan la elección:

- **T-3** exige cifrado en reposo con SQLCipher.
- **§140** exige auditorías formales de seguridad. Un producto auditable necesita que el SQL que se ejecuta sea **legible**, no generado por capas de macros.

## Decisión

- **`rusqlite`** con la característica `bundled-sqlcipher-vendored-openssl`
- **`refinery`** para migraciones SQL versionadas y embebidas
- **Capa de repositorios propia**, sin ORM

## Justificación

**SQLCipher es de primera clase en rusqlite.** Es la razón principal. En SQLx el soporte de SQLCipher es frágil y depende de cómo esté compilado `libsqlite3-sys` en el sistema; en rusqlite es una característica de compilación soportada que además empaqueta OpenSSL, eliminando la dependencia de lo que haya instalado en la máquina del desarrollador o del cliente.

**El SQL se lee.** En una auditoría de seguridad, poder leer exactamente la consulta que se ejecuta vale más que la comodidad de un ORM. SeaORM y Diesel esconden el SQL emitido tras capas de abstracción, y en un producto que debe pasar revisión formal eso es un coste real, no una molestia estética.

**Sin ORM, con repositorios.** Un módulo de repositorio por agregado, con consultas parametrizadas explícitas. Los filtros dinámicos —los de la pantalla de contactos— se construyen con un **constructor tipado** que sólo puede producir fragmentos parametrizados, nunca por concatenación de cadenas.

## Consecuencias

**Positivas.** Cifrado en reposo sin fricción · SQL auditable · sin magia de macros en el camino crítico · control total sobre índices y planes de consulta, que es lo que permite cumplir los presupuestos a 500 k contactos.

**Negativas.**

1. **`rusqlite` es síncrono.** No encaja directamente en Tokio. **Mitigación:** un **hilo dedicado de base de datos** que recibe peticiones por canal, en lugar de repartir `spawn_blocking` por todo el código.

   Por qué un hilo dedicado y no un pool: SQLite con WAL admite lectores concurrentes pero **un solo escritor**. Serializar las escrituras por un único hilo **elimina de raíz** la clase de errores `SQLITE_BUSY`, en vez de gestionarlos con reintentos que funcionan el 99 % de las veces. Los lectores usan conexiones separadas.

2. **Más código a mano.** Cada consulta se escribe. Es el precio de que se pueda leer.

3. **Sin comprobación en tiempo de compilación.** SQLx verifica las consultas contra la base de datos al compilar; nosotros no. **Mitigación:** tests de integración que ejecutan cada consulta contra una base de datos real, y que fallan si una migración rompe algo. Es una red de seguridad distinta, no peor, pero hay que mantenerla.

4. **Sobrecoste de SQLCipher:** 5–15 %. No negociable (T-3).

## Alternativas descartadas

**SQLx.** Su comprobación de consultas en tiempo de compilación es genuinamente atractiva y fue la alternativa más seria. Se descarta porque el soporte de SQLCipher es frágil, y T-3 no es opcional. Si el cifrado fuera opcional, SQLx sería la elección.

**Diesel.** Maduro y con buen tipado, pero su DSL aleja del SQL real y su historia con SQLCipher tampoco es limpia.

**SeaORM.** Async y ergonómico, pero es la opción con más magia, y es exactamente lo contrario de lo que un producto auditable necesita.

**Postgres embebido.** Sobredimensionado para una aplicación de escritorio: añade un proceso, complica la instalación y no aporta nada a 500 k filas.

## Notas de implementación

```
PRAGMA key = <32 bytes del llavero>;   -- primero, siempre
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;           -- NORMAL basta con WAL
PRAGMA busy_timeout = 5000;
```

**`PRAGMA key` va antes que cualquier otra cosa.** Si se ejecuta cualquier sentencia antes de la clave, SQLCipher falla, y el mensaje de error no es evidente.

**Copia antes de migrar.** Toda migración hace primero una copia de la base de datos. Una migración fallida sobre los datos de un cliente sin copia previa es una pérdida irrecuperable.
