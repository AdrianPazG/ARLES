# Arquitectura

**Proyecto:** ARLES RELAY I · v1.2.0
**Estilo:** monolito modular
**Fecha:** 2026-09-11

---

## 1. Forma general

ARLES es **una aplicación de escritorio de un solo proceso** con un motor de ejecución en segundo plano. No hay microservicios, ni Kubernetes, ni colas distribuidas, ni ningún componente de red propio.

```
┌─────────────────────────────────────────────────────────┐
│  WEBVIEW  (Vue 3 + TypeScript estricto)                 │
│  Presentación. Sin autoridad sobre nada de negocio.     │
└───────────────────────────┬─────────────────────────────┘
                            │  comandos Tauri (IPC tipado)
┌───────────────────────────▼─────────────────────────────┐
│  arles-app        Orquestación, comandos, estado        │
├─────────────────────────────────────────────────────────┤
│  arles-campaign   arles-engine      arles-import        │
│  arles-contacts   arles-suppression arles-template      │
│  arles-email      arles-company     arles-backup        │
│  arles-audit                                            │
├─────────────────────────────────────────────────────────┤
│  arles-db         SQLCipher, migraciones, repositorios  │
├─────────────────────────────────────────────────────────┤
│  arles-core       Tipos, errores, Secret<T>. Sin I/O.   │
└─────────────────────────────────────────────────────────┘
        │                              │
  ┌─────▼──────┐              ┌────────▼────────┐
  │  SQLite    │              │  Llavero del SO │
  │ (cifrado)  │              │ Keychain / CM   │
  └────────────┘              └─────────────────┘
```

---

## 2. Crates

Workspace de Cargo. Las dependencias **sólo apuntan hacia dentro**: un crate de dominio nunca depende de otro crate de dominio hermano, sólo de `arles-core` y `arles-db`.

| Crate | Responsabilidad | Depende de |
|---|---|---|
| `arles-core` | Tipos de dominio, ids tipados, errores, `Secret<T>`, normalización de correo. **Sin I/O** | — |
| `arles-db` | Conexión SQLCipher, migraciones, repositorios | `core` |
| `arles-company` | Empresa, preferencias, licencia (esquema) | `core`, `db` |
| `arles-contacts` | Contactos, listas, etiquetas, campos, filtros | `core`, `db` |
| `arles-import` | Lectura XLSX/CSV, detección, mapeo, validación, deduplicación | `core`, `db`, `contacts` |
| `arles-suppression` | Lista de supresión. **Autoridad final sobre si un envío procede** | `core`, `db` |
| `arles-email` | Trait `EmailProvider` + `SmtpProvider` | `core` |
| `arles-template` | Sustitución de variables, sanitizado HTML | `core` |
| `arles-campaign` | Campañas, audiencias, simulador, preflight | `core`, `db`, `contacts`, `suppression`, `template` |
| `arles-engine` | Cola, planificador, workers, throttling, reintentos, disyuntor | `core`, `db`, `email`, `suppression`, `template` |
| `arles-audit` | Bitácora append-only | `core`, `db` |
| `arles-backup` | Exportación e importación `.arles` | `core`, `db` |
| `arles-app` | Comandos Tauri, estado de aplicación, arranque | todos |

**Por qué tantos crates y no módulos.** Cargo impone el grafo de dependencias en tiempo de compilación. Con módulos dentro de un solo crate, nada impide que `arles-contacts` acabe importando `arles-engine` un martes por la tarde y el monolito modular deje de ser modular. Con crates, el compilador lo rechaza.

Coste honesto: tiempos de compilación algo mayores y más ceremonia en el `Cargo.toml`. Aceptable a cambio de que la frontera sea real.

---

## 3. Las cuatro reglas de frontera

Estas reglas son lo que mantiene la arquitectura viva. Romper cualquiera de ellas degrada el producto de forma difícil de revertir.

### 3.1 La webview no decide nada de negocio

Ningún límite, ninguna validación de supresión, ningún sanitizado es autoritativo en el frontend. La webview es una **superficie de presentación**; la autoridad vive en Rust.

Esto no es purismo. La webview es la superficie con mayor exposición a contenido no confiable: nombres de contactos importados de un XLSX ajeno, HTML de plantillas, mensajes de error de servidores SMTP remotos. Si la decisión de «este contacto está suprimido» viviera en TypeScript, bastaría con un fallo de renderizado para enviar a quien pidió no recibir nada.

El frontend **sí** valida — pero sólo para dar respuesta inmediata al usuario. El backend valida otra vez, y la suya es la que cuenta.

### 3.2 `arles-core` no hace I/O

Sin base de datos, sin red, sin sistema de archivos. Es lo que permite testear la lógica de dominio —normalización de correo, cálculo de ventanas, transiciones de estado, estimación del simulador— sin levantar nada.

### 3.3 La supresión se consulta en el momento del envío

No es un filtro que se aplica al construir la audiencia. Es una comprobación inmediatamente antes de entregar el mensaje al proveedor.

Motivo: una campaña de 5 000 correos a 50 diarios dura 100 días. En ese tiempo la gente se da de baja. Filtrar sólo al principio significaría enviar a personas que pidieron no recibir nada, semanas después de pedirlo.

### 3.4 Las credenciales nunca cruzan la frontera IPC

Ningún comando Tauri devuelve un token, una contraseña ni la clave maestra. El frontend recibe un identificador de cuenta y un estado; la credencial se resuelve dentro de Rust en el momento del uso, desde el llavero.

---

## 4. Frontend

Vue 3 con TypeScript en modo estricto, Vite, organizado por la **misma división de dominios** que el backend, para que un cambio de producto toque una sola columna vertical.

```
src/
  app/          Arranque, router, layout, proveedores globales
  design/       Tokens, primitivas, componentes del sistema
  domains/
    company/    { views, components, stores, api }
    contacts/
    imports/
    suppression/
    senders/
    campaigns/
    templates/
    activity/
    settings/
  shared/       Utilidades, composables, tipos generados
```

**Estado: Pinia.** Oficial de Vue, tipado, con devtools, y sin la ceremonia de Vuex. Un store por dominio. Ver ADR-0007.

**Tablas: TanStack Virtual** (headless). Virtualización sobre nuestros propios componentes, no una cuadrícula pesada de terceros. Con 500 000 contactos, renderizar sólo lo visible no es una optimización: es la diferencia entre funcionar y no funcionar.

**Tipos compartidos.** Los tipos de los comandos Tauri se **generan desde Rust** (`ts-rs` o equivalente) en vez de mantenerse a mano en ambos lados. Una desincronización entre el contrato de Rust y el de TypeScript es un bug silencioso que sólo aparece en tiempo de ejecución.

**i18n desde el principio** (§139). Todo texto visible pasa por la capa de internacionalización, aunque v1.2.0 sólo tenga español. Retrofitear i18n sobre una interfaz terminada es un trabajo mecánico, tedioso y propenso a dejar cadenas huérfanas; hacerlo desde el inicio no cuesta casi nada. Además es requisito del white labeling futuro (§117).

---

## 5. Frontera IPC

Los comandos Tauri son el único puente. Principios:

**Comandos gruesos, no finos.** `crear_campana(borrador)` en vez de doce llamadas para poner doce campos. Cada cruce de la frontera es una oportunidad de estado inconsistente.

**Errores tipados, no cadenas.** Un enum de error con variantes que el frontend puede distinguir y traducir. El §95 exige errores que digan qué pasó, cómo arreglarlo y qué está a salvo — eso es imposible con `Result<T, String>`.

**Eventos para el progreso.** El motor emite eventos de progreso; el frontend se suscribe. No hay sondeo.

**Nada de rutas del sistema de archivos por IPC.** El frontend nunca envía ni recibe rutas absolutas. El selector de archivos devuelve un identificador opaco que Rust resuelve internamente.

---

## 6. Concurrencia

**Tokio** como runtime. El motor corre en su propio conjunto de tareas, independiente de la ventana (§51).

**La base de datos es síncrona.** `rusqlite` no es async (ADR-0002). Se resuelve con un **hilo dedicado de base de datos** que recibe peticiones por canal, en lugar de repartir `spawn_blocking` por todo el código.

Por qué un hilo dedicado y no un pool: SQLite con SQLCipher y WAL admite lectores concurrentes pero un solo escritor. Serializar las escrituras a través de un único hilo elimina de raíz la clase de errores `SQLITE_BUSY` en vez de gestionarlos con reintentos. Los lectores pueden usar conexiones separadas.

**Cancelación cooperativa.** El motor usa `CancellationToken`. La parada de emergencia se comprueba **antes de cada envío**, no entre lotes — un lote de 50 correos tarda demasiado en terminar como para que «parar» signifique «parar dentro de un rato».

---

## 7. Configuración y rutas

**Nada de rutas absolutas atadas a una máquina** (§7). Se usan los directorios estándar de la plataforma (`directories-rs`):

| Qué | Windows | macOS |
|---|---|---|
| Base de datos | `%APPDATA%\TelemetryInsight\ArlesRelay\` | `~/Library/Application Support/…` |
| Configuración | idem | idem |
| Registros | `%LOCALAPPDATA%\…\logs\` | `~/Library/Logs/…` |
| Respaldos | elegido por el usuario | elegido por el usuario |

**Ningún límite ni correo se codifica en el binario** (§7). Todo es configuración de la empresa.

---

## 8. Observabilidad

`tracing` con salida estructurada, **con una capa de redacción**. La capa es obligatoria, no opcional: es lo que impide que un `tracing::debug!("{:?}", account)` filtre un token. Complementa a `Secret<T>`, no lo sustituye.

Rotación de registros con retención acotada. Los registros contienen datos personales (direcciones de correo) y por lo tanto caen bajo la LFPDPPP: no se guardan indefinidamente.

**La bitácora de auditoría (`audit_log`) no es un registro.** Vive en la base de datos, es append-only, y registra acciones de negocio críticas. Un registro se puede borrar y rotar; la bitácora no.

---

## 9. Preparación para el futuro, sin construirlo

Tres cosas se dejan posibles sin implementarse:

**ARLES RELAY CLOUD** (§8). No se asume «una sola empresa por base de datos»: `company_id` está presente en las entidades desde el principio. La lógica de negocio vive fuera de la interfaz, así que un futuro servidor podría reutilizar los mismos crates de dominio.

**White labeling** (§117). Tokens de diseño centralizados, textos por i18n, logotipo como activo reemplazable. La funcionalidad no se construye.

**Licenciamiento** (§76). El esquema existe —organización, edición, asientos, caducidad, `FeatureEntitlement`— y el enforcement no. Así activar una funcionalidad en v1.3 no obliga a migrar datos ni reescribir el arranque.

**Lo que deliberadamente no se prepara:** una capa de abstracción sobre la base de datos para «poder cambiar a Postgres algún día». Eso es especulación, y el coste de la indirección se paga todos los días a cambio de una flexibilidad que probablemente nunca se use.

---

## 10. Decisiones registradas

| ADR | Decisión |
|---|---|
| [0001](adr/0001-tauri-2-sobre-electron.md) | Tauri 2 sobre Electron |
| [0002](adr/0002-rusqlite-sqlcipher-refinery.md) | rusqlite + SQLCipher + refinery |
| [0003](adr/0003-smtp-primero-oauth-en-paralelo.md) | SMTP primero, OAuth en paralelo |
| [0004](adr/0004-idempotencia-y-envio-ambiguo.md) | Idempotencia y política de envío ambiguo |
| [0005](adr/0005-paleta-derivada.md) | Paleta derivada |
| [0006](adr/0006-plantillas-no-turing-completas.md) | Sustitución de variables no Turing-completa |
| [0007](adr/0007-pinia-y-tanstack-virtual.md) | Pinia + TanStack Virtual |
| [0008](adr/0008-sin-rotacion-de-remitentes.md) | Sin rotación automática de remitentes |
| [0009](adr/0009-alcance-deteccion-rebotes.md) | Alcance de la detección de rebotes |
| [0010](adr/0010-numeral-romano-i.md) | Retención del numeral «I» |
| [0011](adr/0011-cifrado-en-reposo-y-clave-maestra.md) | Cifrado en reposo y clave maestra |
| [0012](adr/0012-recursos-fuente-unica.md) | `/RECURSOS` como fuente única |
