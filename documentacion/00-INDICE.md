# Documentación de ARLES RELAY I

**Versión objetivo:** v1.2.0 · **Desarrollado por:** TELEMETRY INSIGHT
**Fase actual:** 0 — Discovery, auditoría y arquitectura
**Última actualización:** 2026-09-11

---

## Por dónde empezar

| Si eres… | Lee esto |
|---|---|
| **Dirección** | [Auditoría A–U](02-auditoria/AUDITORIA_DISCOVERY.md) → [Preguntas abiertas](02-auditoria/PREGUNTAS_ABIERTAS.md) |
| Tech Lead / Arquitecto | [Arquitectura](03-arquitectura/ARQUITECTURA.md) → [Motor de ejecución](03-arquitectura/MOTOR_DE_EJECUCION.md) → ADRs |
| Backend / Rust | [Modelo de datos](03-arquitectura/MODELO_DE_DATOS.md) → [Motor](03-arquitectura/MOTOR_DE_EJECUCION.md) → [Proveedores](03-arquitectura/ABSTRACCION_EMAIL_PROVIDER.md) |
| Frontend | [Design System](05-diseno/DESIGN_SYSTEM.md) → [Color](05-diseno/COLOR_SYSTEM.md) → [UX y navegación](05-diseno/UX_NAVEGACION.md) |
| Seguridad | [Modelo de amenazas](04-seguridad/THREAT_MODEL.md) → [Secretos](04-seguridad/MODELO_DE_SECRETOS.md) |
| QA | [Estrategia de QA](06-calidad/ESTRATEGIA_QA.md) → [Rendimiento](06-calidad/PRESUPUESTO_RENDIMIENTO.md) |
| Nuevo en el proyecto | [Visión y alcance](01-producto/VISION_Y_ALCANCE.md) → [Glosario](01-producto/GLOSARIO.md) |

---

## Índice

### 01 · Producto
| Documento | Contenido |
|---|---|
| [VISION_Y_ALCANCE](01-producto/VISION_Y_ALCANCE.md) | Qué es y qué no es ARLES, bucle central, criterios de éxito |
| [DECISIONES_DE_DIRECCION](01-producto/DECISIONES_DE_DIRECCION.md) | T-1…T-10 y D-1…D-4. **Registro autoritativo** |
| [FUERA_DE_ALCANCE](01-producto/FUERA_DE_ALCANCE.md) | Todo lo diferido, con destino explícito |
| [GLOSARIO](01-producto/GLOSARIO.md) | Vocabulario compartido, incluido el de la interfaz |

### 02 · Auditoría
| Documento | Contenido |
|---|---|
| [**AUDITORIA_DISCOVERY**](02-auditoria/AUDITORIA_DISCOVERY.md) | **El entregable A–U del §163** |
| [INVENTARIO_DE_ASSETS](02-auditoria/INVENTARIO_DE_ASSETS.md) | Los 7 hallazgos forenses con datos crudos |
| [MATRIZ_DE_RIESGOS](02-auditoria/MATRIZ_DE_RIESGOS.md) | R-01…R-18, con dueño y disparador |
| [PREGUNTAS_ABIERTAS](02-auditoria/PREGUNTAS_ABIERTAS.md) | P-01…P-08. **Lo que bloquea** |

### 03 · Arquitectura
| Documento | Contenido |
|---|---|
| [ARQUITECTURA](03-arquitectura/ARQUITECTURA.md) | Monolito modular, crates, reglas de frontera |
| [MODELO_DE_DATOS](03-arquitectura/MODELO_DE_DATOS.md) | Entidades, índices, normalización, ARCO |
| [**MOTOR_DE_EJECUCION**](03-arquitectura/MOTOR_DE_EJECUCION.md) | **Cola, idempotencia, suspensión, límites** |
| [ABSTRACCION_EMAIL_PROVIDER](03-arquitectura/ABSTRACCION_EMAIL_PROVIDER.md) | Trait, SMTP, OAuth |

**ADRs** → [`03-arquitectura/adr/`](03-arquitectura/adr/)

| # | Decisión | Estado |
|---|---|---|
| [0001](03-arquitectura/adr/0001-tauri-2-sobre-electron.md) | Tauri 2 sobre Electron | aceptado |
| [0002](03-arquitectura/adr/0002-rusqlite-sqlcipher-refinery.md) | rusqlite + SQLCipher + refinery | aceptado |
| [0003](03-arquitectura/adr/0003-smtp-primero-oauth-en-paralelo.md) | SMTP primero, OAuth en paralelo | aceptado |
| [0004](03-arquitectura/adr/0004-idempotencia-y-envio-ambiguo.md) | Idempotencia y envío ambiguo | aceptado |
| [0005](03-arquitectura/adr/0005-paleta-derivada.md) | Paleta derivada | aceptado |
| [0006](03-arquitectura/adr/0006-plantillas-no-turing-completas.md) | Plantillas no Turing-completas | aceptado |
| [0007](03-arquitectura/adr/0007-pinia-y-tanstack-virtual.md) | Pinia + TanStack Virtual | aceptado |
| [0008](03-arquitectura/adr/0008-sin-rotacion-de-remitentes.md) | Sin rotación de remitentes | aceptado |
| [0009](03-arquitectura/adr/0009-alcance-deteccion-rebotes.md) | Alcance de detección de rebotes | aceptado · **corregido por 0014** |
| [0010](03-arquitectura/adr/0010-numeral-romano-i.md) | Numeral «I» | aceptado |
| [0011](03-arquitectura/adr/0011-cifrado-en-reposo-y-clave-maestra.md) | Cifrado en reposo y clave maestra | aceptado |
| [0012](03-arquitectura/adr/0012-recursos-fuente-unica.md) | `/RECURSOS` fuente única | **propuesto** |
| [0013](03-arquitectura/adr/0013-origen-de-contactos-y-purificacion.md) | Origen de contactos, purificación y canario | aceptado |
| [0014](03-arquitectura/adr/0014-deteccion-de-rebotes-sin-verp.md) | Detección de rebotes sin VERP | aceptado |

### 04 · Seguridad
| Documento | Contenido |
|---|---|
| [THREAT_MODEL](04-seguridad/THREAT_MODEL.md) | STRIDE por dominio, riesgos aceptados, checklist |
| [MODELO_DE_SECRETOS](04-seguridad/MODELO_DE_SECRETOS.md) | Llavero, clave maestra, `Secret<T>` |
| [PRIVACIDAD_LFPDPPP](04-seguridad/PRIVACIDAD_LFPDPPP.md) | Derechos ARCO, consentimiento, bajas |

### 05 · Diseño
| Documento | Contenido |
|---|---|
| [COLOR_SYSTEM](05-diseno/COLOR_SYSTEM.md) | Paleta medida, rampas, contraste calculado |
| [DESIGN_SYSTEM](05-diseno/DESIGN_SYSTEM.md) | Tokens, componentes, movimiento, accesibilidad |
| [TIPOGRAFIA](05-diseno/TIPOGRAFIA.md) | ⚠️ **Bloqueado por D-3**. Plan A y plan B |
| [UX_NAVEGACION](05-diseno/UX_NAVEGACION.md) | 6 secciones, flujos, onboarding, teclado |
| [UX_WRITING](05-diseno/UX_WRITING.md) | Tono, errores, vocabulario de la honestidad |

### 06 · Calidad
| Documento | Contenido |
|---|---|
| [ESTRATEGIA_QA](06-calidad/ESTRATEGIA_QA.md) | Camino dorado, casos frontera, las 4 auditorías |
| [PRESUPUESTO_RENDIMIENTO](06-calidad/PRESUPUESTO_RENDIMIENTO.md) | Objetivos a 500 k, medidos en CI |

### 07 · Entrega
| Documento | Contenido |
|---|---|
| [DISTRIBUCION_Y_FIRMA](07-entrega/DISTRIBUCION_Y_FIRMA.md) | Instaladores, firma, notarización, updater |
| [ROADMAP](07-entrega/ROADMAP.md) | 10 fases con dependencias reales |

### 08 · Legal
| Documento | Contenido |
|---|---|
| [LICENCIAS_DE_TERCEROS](08-legal/LICENCIAS_DE_TERCEROS.md) | Mont, assets stock/IA, dependencias |

### 09 · Fases
Qué se construyó en cada fase, **cómo se comprobó** y qué quedó sin verificar.

| Documento | Estado |
|---|---|
| [README · índice de fases](09-fases/README.md) | — |
| [Fase 00 · Discovery y auditoría](09-fases/FASE-00-DISCOVERY.md) | ✅ 4/4 |
| [Fase 01 · Cimientos](09-fases/FASE-01-CIMIENTOS.md) | ✅ 32/32 |
| [Fase 01 · resumen para Dirección](09-fases/FASE-01-PARA-DIRECCION.md) | Sin tecnicismos |
| [Fase 02 · Design System](09-fases/FASE-02-DESIGN-SYSTEM.md) | ✅ 18/18 |
| [Fase 02 · resumen para Dirección](09-fases/FASE-02-PARA-DIRECCION.md) | Sin tecnicismos |

---

## Estado del proyecto

### Fases 0, 1 y 2 ✅ cerradas · Fase 3 es la siguiente

La puerta de salida se cruzó el **2026-09-11**: Dirección aprobó la auditoría, **D-5** autorizó desarrollar con Mont, y **P-02** quedó cerrada porque el logotipo es tipográfico por diseño (§21).

**No quedan bloqueantes para la Fase 3.**

### 🟡 En seguimiento

| # | Asunto | Puerta | Dueño |
|---|---|---|---|
| **P-01** | Licencia de Mont | **Antes de la demo** (D-5) | Dirección |
| P-03 | Origen y consentimiento de los contactos | Fase 3 | Dirección + Legal |
| P-05 | Dominio y aviso de privacidad para Google | Fase 5 | Dirección |

### Herramientas

| | |
|---|---|
| [Generador de tokens](../herramientas/design-tokens/README.md) | Fuente única de color → CSS, lámina y verificación WCAG en CI |
| `herramientas/iconos/generar-iconos.py` | Iconos de la app desde los mismos tokens, con `.ico` e `.icns` |
| `herramientas/validar/validar.py` | Valida una fase completa y emite informe |

```bash
python3 herramientas/validar/validar.py            # todas las fases cerradas
python3 herramientas/validar/validar.py --fase 1   # solo una
```

---

## Trazabilidad del brief maestro

Cada sección del brief tiene destino. **Ninguna queda sin resolver.**

| §§ | Tema | Estado | Dónde |
|---|---|---|---|
| 1–3 | Fase previa, nombre, versionado | ✅ aceptado · ADR-0010 sobre el numeral | VISION_Y_ALCANCE · ADR-0010 |
| 4–8 | Naturaleza, objetivo, visión comercial, distribución | ✅ aceptado · **D-4** ajusta el alcance | VISION_Y_ALCANCE |
| 9 | Auditoría de stack | ✅ **confirmado** con comparación formal | ADR-0001 · ADR-0002 · ADR-0007 |
| 10 | Monolito modular | ✅ aceptado | ARQUITECTURA |
| 11–12 | `/RECURSOS` | ⚠️ **hallazgos A-01, A-06** | INVENTARIO · ADR-0012 |
| 13–16 | Referencia cromática y paleta | ⚠️ **modificado — D-2** | COLOR_SYSTEM · ADR-0005 |
| 17–19 | Tokens, dark-first, accesibilidad | ✅ aceptado, con contraste calculado | COLOR_SYSTEM · DESIGN_SYSTEM |
| 20–21 | Tipografía y logotipo | 🔴 **bloqueado — D-3** | TIPOGRAFIA |
| 22 | Experiencia de escritorio | ✅ aceptado | DESIGN_SYSTEM |
| 23 | Navegación | ⚠️ **modificado**: 7 → 6 secciones | UX_NAVEGACION |
| 24–25 | Empresa y onboarding | ✅ aceptado | UX_NAVEGACION |
| 26–29 | Abstracción de proveedores y OAuth | ✅ aceptado · **D-1** difiere Google | ABSTRACCION · ADR-0003 |
| 30 | Secretos | ✅ aceptado, reforzado | MODELO_DE_SECRETOS · ADR-0011 |
| 31–32 | Base de datos y modelo | ✅ aceptado | MODELO_DE_DATOS · ADR-0002 |
| 33–36 | Contactos e importación | ✅ aceptado | MODELO_DE_DATOS · THREAT_MODEL |
| 37–39 | Listas, filtros, supresión | ⚠️ **contradicción §37/§69 resuelta** | ADR-0009 |
| 40 | Flujo de campaña | ✅ aceptado | UX_NAVEGACION |
| 41–45 | Editor y plantillas | ✅ aceptado, acotado | ADR-0006 · FUERA_DE_ALCANCE |
| 46–50 | Límites y simulador | ✅ aceptado | MOTOR_DE_EJECUCION · UX_NAVEGACION |
| 51–55 | Motor de ejecución | ✅ aceptado, **adelantado a Fase 4** | MOTOR_DE_EJECUCION · ADR-0004 |
| 56–62 | Cola, throttling, reintentos, parada | ✅ aceptado | MOTOR_DE_EJECUCION |
| 63 | Preflight | ✅ aceptado | MOTOR_DE_EJECUCION |
| 64–66 | Entregabilidad y métricas | ⚠️ **parcial**: SPF/DKIM/DMARC sí, rebotes no | ADR-0009 |
| 67–69 | Tracking y bandeja | ✅ aceptado — diferido | FUERA_DE_ALCANCE |
| 70–71 | Panel y actividad | ✅ aceptado | UX_NAVEGACION |
| 72–75 | Respaldos | ✅ aceptado · cifrado diferido (T-6) | MODELO_DE_SECRETOS |
| 76–78 | Licenciamiento | ✅ esquema sí, enforcement a v1.3 | AUDITORIA §Q |
| 79–82 | Actualizaciones y distribución | ✅ aceptado | DISTRIBUCION_Y_FIRMA |
| 83–93 | Seguridad y privacidad | ✅ aceptado | THREAT_MODEL · PRIVACIDAD_LFPDPPP |
| 94–97 | UX writing y estados | ✅ aceptado | UX_WRITING · DESIGN_SYSTEM |
| 98–103 | Movimiento, teclado, tablas, rendimiento | ✅ aceptado | DESIGN_SYSTEM · PRESUPUESTO |
| 104–108 | QA | ✅ aceptado | ESTRATEGIA_QA |
| 117–124 | Atribución y white label | ✅ atribución sí; white label a v1.4+ | UX_WRITING · FUERA_DE_ALCANCE |
| 125–139 | Exclusiones, docs, calidad de código | ✅ aceptado | FUERA_DE_ALCANCE · ARQUITECTURA |
| 140 | Las cuatro auditorías | ✅ aceptado, con checklist ejecutable | ESTRATEGIA_QA §5 |
| 141 | Roadmap | ⚠️ **consolidado**: 16 → 10 fases | ROADMAP |
| 154–162 | Principios | ✅ aceptado, son la base del documento | VISION_Y_ALCANCE |
| 163 | Formato A–U | ✅ **entregado** | AUDITORIA_DISCOVERY |
| 175 | Detenerse antes de programar | ✅ **respetado** | Puerta de salida |

### Las tres contradicciones del brief, cerradas

| Contradicción | Resolución |
|---|---|
| §37 (supresión automática por rebote) **vs** §69 (sin leer bandeja) | ADR-0009: parcial en v1.2.0 y **declarado en la interfaz**; buzón dedicado en v1.3 — por reenvío, no por VERP ([ADR-0014](03-arquitectura/adr/0014-deteccion-de-rebotes-sin-verp.md)) |
| T-4 («beta comercial») **vs** T-6 (sin licenciamiento) | **D-4**: v1.2.0 es despliegue interno |
| §16 (tres azules distintos) **vs** paleta medida (0.73 % azul profundo) | **D-2** / ADR-0005: extraer el carácter, construir la estructura |

---

## Convenciones

- **T-n** — decisión de Dirección previa al discovery
- **D-n** — decisión de Dirección tomada en el discovery
- **A-n** — hallazgo de la auditoría forense
- **R-n** — riesgo
- **P-n** — pregunta abierta
- **ADR-n** — registro de decisión de arquitectura
- **§n** — sección del brief maestro

Una decisión es firme hasta que Dirección la revoque **por escrito** en `DECISIONES_DE_DIRECCION.md`.
