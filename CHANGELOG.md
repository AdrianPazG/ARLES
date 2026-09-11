# Changelog

Todos los cambios notables de ARLES RELAY I se documentan aquí.

Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).
Versionado según [Versionado Semántico](https://semver.org/lang/es/) (§3).

> **Nomenclatura prohibida** en cualquier artefacto: FINAL, FINAL2, DEFINITIVO, NUEVO.
> Toda versión se identifica por su número semántico.

---

## [Sin publicar]

### Fase 2 — Design System ✅ cerrada

**Validación: 12/12 comprobaciones de la fase, 0 omitidas** — `validar.py --fase 2`
(48/48 en la ejecución completa). 17 tests de componente.
Resumen en [`FASE-02-DESIGN-SYSTEM.md`](documentacion/09-fases/FASE-02-DESIGN-SYSTEM.md).

- **Once primitivas** en `app/src/design/componentes/`: botón, entrada,
  selector, insignia, aviso, modal, menú, pestañas, tabla virtualizada, icono y
  logotipo. Más los cuatro estados de pantalla del §97.
- **Mont incrustada** — Regular 400, SemiBold 600, Bold 700 y Black 900 en
  `.woff2`, 180 KB. Cada `@font-face` fija su peso explícitamente.
- **Catálogo del design system** en `/#/catalogo`, sólo en desarrollo: cada
  primitiva en sus cuatro estados, para revisarla en Windows y en macOS.
- **Tokens nuevos** con contrato de contraste verificado en CI:
  `--arles-accent-hover`, `--arles-danger-hover` y `--arles-text-disabled`;
  más alturas de control y de fila, anchos de modal y menú, y las dos sombras.
- **7 comprobaciones nuevas** en el validador, todas probadas rompiéndolas a
  propósito.

#### Corregido

- **El botón deshabilitado no se leía.** `opacity: 0.45` daba **1.89:1** en el
  primario y **3.56:1** en un secundario ocupado. WCAG exime a los controles
  deshabilitados, así que ninguna herramienta lo marcaba. Se pinta el estado en
  vez de atenuar el elemento: 3.64:1. Y «ocupado» deja de atenuarse — pasa a
  11.8:1, porque está trabajando, no deshabilitado.
- **`TIPOGRAFIA.md` pedía un corte Medium 500 que el kit no tiene.** Medidos los
  grosores reales, salta de Regular (87 por mil) a SemiBold (115). Las cifras
  usan Regular 400 con figuras tabulares.
- **El `usWeightClass` del kit está desplazado** un escalón: `Mont-Regular`
  declara 600. Documentado, y cada `@font-face` fija su peso para no depender
  del metadato.
- La comprobación de `prefers-reduced-motion` era una búsqueda de texto y pasaba
  con la propiedad mal escrita. Ahora exige la regla `@media` completa y que
  apague animación **y** transición.

### Fase 1 — Cimientos ✅ cerrada

**Validación: 32/32 comprobaciones de la fase, 0 omitidas** — `validar.py --fase 1`
(36/36 contando las 4 de la Fase 0 en la ejecución completa).
Revisión a fondo: **13 hallazgos, todos corregidos** (§6 del documento de fase).
Resumen completo en [`FASE-01-CIMIENTOS.md`](documentacion/09-fases/FASE-01-CIMIENTOS.md);
versión sin tecnicismos para Dirección en [`FASE-01-PARA-DIRECCION.md`](documentacion/09-fases/FASE-01-PARA-DIRECCION.md).


- **`arles-core`** — tipos de dominio sin I/O: ids tipados con UUID v7, `Secret<T>`
  con `Debug`/`Display` redactados, normalización de correo y máquina de estados
  de intento. 40 tests.
- **`arles-db`** — SQLite cifrado con SQLCipher, migraciones con `refinery` y el
  esquema inicial completo. Incluye los tests que verifican que el archivo
  en disco está realmente cifrado y que los invariantes del esquema se cumplen.
  29 tests.
- **`arles-app`** — shell de Tauri 2 con capabilities denegadas por defecto, CSP
  estricta, integración con el llavero del sistema e iconos generados desde los
  tokens. 19 tests, incluido el arranque en sus dos ramas.
- **`app/`** — andamiaje de Vue 3 con TypeScript estricto, Pinia, vue-router e
  i18n, consumiendo el CSS generado desde `tokens.json`. 14 tests.
- **`herramientas/design-tokens/`** — fuente única de color: genera el CSS, la
  lámina de la paleta y la verificación WCAG de CI.
- **`herramientas/iconos/`** — iconos de la aplicación desde los mismos tokens,
  con `.ico` e `.icns` empaquetados sin dependencias externas.
- **`herramientas/validar/`** — valida una fase completa: estructura, fronteras
  de arquitectura, compilación, tests y documentación. Emite informe JSON.
- **CI** — contraste, rustfmt, clippy con `-D warnings`, tests en Linux, Windows
  y macOS, `cargo-deny`, auditoría de npm con dos umbrales y validación de fase.

### Decisiones

- **D-5** — Se desarrolla con Mont; la puerta de la licencia pasa de la Fase 2 a
  **antes de la demo**. Cierra P-02: el logotipo es tipográfico por diseño (§21).
- **ADR-0002** — `refinery` 0.9, no 0.8: la 0.8 fija `rusqlite` ≤ 0.26 y ambos
  declaran `links = "sqlite3"`, así que no coexisten con el 0.37 que SQLCipher
  necesita.

### Fase 0 — Discovery y auditoría

#### Añadido
- Cuerpo documental completo de la Fase 0 en `documentacion/`:
  - Auditoría de discovery con el entregable A–U del §163
  - Inventario forense de assets con 7 hallazgos
  - Matriz de riesgos (R-01…R-18) y preguntas abiertas (P-01…P-08)
  - Arquitectura, modelo de datos, motor de ejecución y abstracción de proveedores
  - 12 registros de decisión de arquitectura (ADR-0001…ADR-0012)
  - Modelo de amenazas STRIDE, modelo de secretos y cumplimiento LFPDPPP
  - Sistema de color, design system, tipografía, UX y UX writing
  - Estrategia de QA y presupuesto de rendimiento
  - Distribución, firma y roadmap de 10 fases
  - Inventario de licencias de terceros

### Decisiones de Dirección
- **D-1** — SMTP en v1.2.0; verificación OAuth de Google en paralelo desde la Fase 1
- **D-2** — Extraer cian, oro y cremas de la referencia; construir por rampa los azules profundos ausentes
- **D-3** — Verificar o adquirir la App License de Mont antes de la Fase 2 (**bloqueante**)
- **D-4** — v1.2.0 es despliegue interno de TELEMETRY; lanzamiento comercial en v1.3

### Contradicciones del brief resueltas
- §37 frente a §69 (detección de rebotes) → ADR-0009: parcial en v1.2.0, declarada en la interfaz
- T-4 frente a T-6 (beta comercial sin licenciamiento) → D-4
- §16 frente a la paleta medida (0.73 % de azul profundo) → D-2 y ADR-0005

### Pendiente tras la Fase 0
- 🔴 **P-01** — Licencia de Mont sin verificar. Bloqueaba la Fase 2
- 🔴 **P-02** — No existe ningún activo de marca de ARLES. Bloqueaba la Fase 2

*Ambas resueltas en la Fase 1 por D-5: se desarrolla con Mont y la puerta de la
licencia pasa a antes de la demo; P-02 se cierra porque el logotipo es
tipográfico por diseño (§21).*

---

## Plantilla para futuras versiones

```markdown
## [1.2.0] — AAAA-MM-DD

**Build:** 2026.09.11 · **Commit:** abc1234

### Añadido
### Cambiado
### Obsoleto
### Eliminado
### Corregido
### Seguridad
### Migraciones
### Cambios incompatibles
```

Toda entrada de versión publicada debe incluir, como mínimo (§3): versión,
fecha, cambios, correcciones, actualizaciones de seguridad, migraciones,
cambios incompatibles, build y commit.
