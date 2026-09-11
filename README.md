# ARLES RELAY

**Software de ejecución y gestión controlada de campañas de correo electrónico.**

Aplicación de escritorio para Windows y macOS · Versión objetivo **1.2.0**
Desarrollado por **TELEMETRY INSIGHT**

---

## Estado

| Fase | | Validación |
|---|---|---|
| [00 · Discovery y auditoría](documentacion/09-fases/FASE-00-DISCOVERY.md) | ✅ cerrada | 4/4 |
| [01 · Cimientos](documentacion/09-fases/FASE-01-CIMIENTOS.md) | ✅ cerrada y revisada | 30/30 |
| 02 · Design System | ⬜ siguiente | — |

```bash
python3 herramientas/validar/validar.py     # valida todas las fases cerradas
```

👉 **[Empieza por la documentación](documentacion/00-INDICE.md)**

---

## Qué es

ARLES permite a una empresa ejecutar campañas de correo usando **sus propias cuentas** y **sus propios límites**.

Lo que define el producto no es enviar correo —eso lo hace cualquier cosa— sino **el control sobre el envío**:

- Límites que el cliente fija, no que le imponemos
- Una simulación que dice cuánto tardará la campaña de verdad, antes de activarla
- Un motor que respeta esos límites incluso cuando el equipo se suspende y despierta
- Idempotencia que garantiza que nadie reciba dos veces el mismo correo
- Métricas que no mienten

### Los tres principios

**Honestidad.** Si el proveedor sólo confirmó «aceptado», la interfaz dice *aceptado*. Nunca «entregado». Y si el producto no sabe algo, lo declara.

**Seguridad sobre comodidad.** Ante la duda entre un flujo cómodo y uno seguro, gana el seguro, y se documenta el coste.

**Autonomía informada.** ARLES advierte y explica; no dicta la lógica de negocio del cliente. Pero nunca facilita la evasión de límites.

### Qué no es

Mailchimp · un CRM · un ERP · una plataforma omnicanal · una herramienta de scraping · un evasor de filtros antispam · un sistema para saltarse los límites de Gmail.

---

## Documentación

| | |
|---|---|
| [📋 Índice completo](documentacion/00-INDICE.md) | Mapa de lectura y trazabilidad del brief |
| [🔍 Auditoría de discovery](documentacion/02-auditoria/AUDITORIA_DISCOVERY.md) | El entregable A–U. **Empieza aquí** |
| [⚠️ Preguntas abiertas](documentacion/02-auditoria/PREGUNTAS_ABIERTAS.md) | Lo que bloquea el avance |
| [🏗️ Arquitectura](documentacion/03-arquitectura/ARQUITECTURA.md) | Monolito modular en Rust + Tauri 2 |
| [⚙️ Motor de ejecución](documentacion/03-arquitectura/MOTOR_DE_EJECUCION.md) | El corazón del producto |
| [🔒 Modelo de amenazas](documentacion/04-seguridad/THREAT_MODEL.md) | STRIDE por dominio |
| [🎨 Sistema de color](documentacion/05-diseno/COLOR_SYSTEM.md) | Paleta medida, contraste calculado |
| [🗺️ Roadmap](documentacion/07-entrega/ROADMAP.md) | 10 fases con dependencias reales |

---

## Stack previsto

| Capa | Tecnología | Decisión |
|---|---|---|
| Shell de escritorio | Tauri 2 | [ADR-0001](documentacion/03-arquitectura/adr/0001-tauri-2-sobre-electron.md) |
| Núcleo | Rust | T-10 |
| Base de datos | SQLite + SQLCipher | [ADR-0002](documentacion/03-arquitectura/adr/0002-rusqlite-sqlcipher-refinery.md) |
| Interfaz | Vue 3 · TypeScript estricto · Vite | |
| Estado y tablas | Pinia · TanStack Virtual | [ADR-0007](documentacion/03-arquitectura/adr/0007-pinia-y-tanstack-virtual.md) |
| Pruebas | cargo test · Vitest · Playwright | |

---

## En seguimiento

No hay bloqueantes. Lo que sigue abierto, con su puerta:

| # | Asunto | Puerta |
|---|---|---|
| 🟡 **P-01** | Licencia de Mont para incrustar la fuente en la aplicación distribuida | Antes de la demo (D-5) |
| 🟡 **P-03** | Origen y consentimiento de los contactos | Fase 3 |
| 🟡 **P-05** | Dominio y aviso de privacidad para la verificación de Google | Fase 5 |

Detalle en [PREGUNTAS_ABIERTAS](documentacion/02-auditoria/PREGUNTAS_ABIERTAS.md).

---

## Carpetas de referencia

`/RECURSOS`, `/REFERENCIA_DE_COLOR`, `/CONCEPTOS_DE_DISEÑO`, `/REFERENCIAS_VISUALES_DEL_SITIO` y `/TIPOGRAFIA` son **material de referencia de sólo lectura** (§11).

⚠️ Su contenido **no es lo que el brief asume**. Antes de usarlos, lee el [inventario forense](documentacion/02-auditoria/INVENTARIO_DE_ASSETS.md): la referencia cromática es un PNG generado por IA, los «conceptos de diseño» son consejos de UX de redes sociales, y no hay ningún logotipo de ARLES.

---

## Versionado

Versionado semántico `MAYOR.MENOR.PARCHE`. Ver [CHANGELOG.md](CHANGELOG.md).

El numeral «I» del nombre comercial **nunca aparece adyacente al número de versión** ([ADR-0010](documentacion/03-arquitectura/adr/0010-numeral-romano-i.md)).

---

*Software desarrollado por TELEMETRY INSIGHT*
