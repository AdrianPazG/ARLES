# ARLES RELAY

**Software de ejecución y gestión controlada de campañas de correo electrónico.**

Aplicación de escritorio para Windows y macOS · Versión objetivo **1.2.0**
Desarrollado por **TELEMETRY INSIGHT**

---

<!-- AVANCE:INICIO -->
![Avance de la v1.2.0](https://img.shields.io/badge/avance_v1.2.0-32%25-orange?style=flat-square)

**32 % de la versión 1.2.0 construido y verificado**, a 2026-09-17.

<details>
<summary>Cómo sale ese número, y qué no mide</summary>

Suma de **peso × hecho** sobre las fases de abajo. Los **pesos** son una
estimación del tamaño relativo de cada fase y suman 100; se dice que son una
estimación porque presentarlos como medición exacta sería mentir. Lo que **no**
es estimación es la fracción hecha: cada una se apoya en algo comprobable, y esa
comprobación está escrita en
[`avance.json`](documentacion/07-entrega/avance.json).

**No mide si el producto sirve**, sino cuánto del alcance de la v1.2.0 está
construido *y verificado*. Los bloqueos externos no restan porcentaje, porque no
son trabajo pendiente nuestro: van aparte.

| Fase | Peso | Hecho |
|---|---|---|
| F0 · Discovery y auditoría | 5 % | ✅ 100 % |
| F1 · Cimientos | 9 % | ✅ 100 % |
| F2 · Design System | 9 % | ✅ 100 % |
| F3 · Empresa y contactos | 14 % | 🟨 30 % |
| F3W · Canales: esquema de contactos, consentimiento y etapas | 8 % | 🟨 55 % |
| F4 · Motor de ejecución | 14 % | ⬜ 0 % |
| F5 · Proveedores de envío | 7 % | ⬜ 0 % |
| F6 · Campañas y mensajes | 13 % | 🟨 5 % |
| F7 · Actividad y entregabilidad | 9 % | ⬜ 0 % |
| F8 · Respaldos y endurecimiento | 7 % | ⬜ 0 % |
| F9 · Release v1.2.0 | 5 % | ⬜ 0 % |

**Esperando a alguien de fuera:**

| | Qué | Quién | Qué frena |
|---|---|---|---|
| **P-01** | Licencia de Mont para incrustarla en la aplicación distribuida | Dirección | El primer instalador descargable. Hay plan B: tipografía del sistema, una línea de CSS |
| **P-09** | Revisión jurídica: ocho preguntas en 08-legal/CONSULTA-JURIDICA.md | Abogado | Ya NO bloquea la construcción (decisión D-7). Frena los textos legales visibles y el primer envío real a alguien de fuera de TELEMETRY |
| **META** | Cuenta de empresa en Meta, verificada y con Coexistencia activada (L-11) | Operaciones | F3W · el alta del número, y con ella toda campaña de WhatsApp |
| **R-07** | Revisión visual en macOS (WKWebView) | Quien tenga un Mac | Nada todavía; la fecha límite acordada es antes de la F6 |

El número se genera; no se escribe a mano en dos sitios:

```bash
python3 herramientas/avance/calcular.py --escribir
```

</details>
<!-- AVANCE:FIN -->

---

## Estado

| Fase | | Validación |
|---|---|---|
| [00 · Discovery y auditoría](documentacion/09-fases/FASE-00-DISCOVERY.md) | ✅ cerrada | 4/4 |
| [01 · Cimientos](documentacion/09-fases/FASE-01-CIMIENTOS.md) | ✅ cerrada y revisada | 32/32 |
| ↳ [Resumen para Dirección](documentacion/09-fases/FASE-01-PARA-DIRECCION.md) | sin tecnicismos | — |
| [02 · Design System](documentacion/09-fases/FASE-02-DESIGN-SYSTEM.md) | ✅ cerrada y revisada | 18/18 |
| ↳ [Resumen para Dirección](documentacion/09-fases/FASE-02-PARA-DIRECCION.md) | sin tecnicismos | — |
| 03 · Empresa y contactos | 🟨 entrega 3.1 cerrada | 31/31 |
| ↳ Canales: esquema de contactos, consentimiento y etapas | 🟨 migraciones V3 y V4 | incluidas arriba |

```bash
python3 herramientas/validar/validar.py     # valida todas las fases cerradas
```

👉 **[Empieza por la documentación](documentacion/00-INDICE.md)**

---

## Descargar ARLES

**Todavía no hay ninguna versión descargable, y conviene decir por qué en vez de
dejar el apartado vacío.**

Cuando la haya, estará en **[Releases](https://github.com/AdrianPazG/ARLES/releases)** — un `.exe` para
Windows y un `.dmg` para macOS. Ése es el sitio, y no un archivo suelto en el
repositorio: un instalador dentro del árbol de código no se puede firmar, no
lleva número de versión asociado y nadie sabe cuál es el bueno.

Para que exista el primero hacen falta tres cosas, y **ninguna es programar**:

| | Qué | Quién | Sin esto pasa que… |
|---|---|---|---|
| **1** | **Licencia de Mont** para incrustarla en una aplicación distribuida (P-01) | Dirección | Repartir el instalador con la tipografía dentro infringe la licencia. Hay un plan B: una línea de CSS y ARLES usa la tipografía del sistema |
| **2** | **Certificado de firma de Windows** | Operaciones | Windows enseña la advertencia de «editor desconocido» y SmartScreen bloquea la instalación |
| **3** | **Apple Developer ID** y notarización | Operaciones | macOS se niega a abrir la aplicación |

### Mientras tanto: la versión de prueba

Se puede construir **hoy**, sin firmar y sin Mont, para revisar la aplicación de
verdad en lugar de con capturas.

### Dónde se descarga

En **[Releases](https://github.com/AdrianPazG/ARLES/releases)** — el enlace de
la derecha de la portada, o el de aquí. **No en la pestaña Actions**: ahí sólo
se ve cómo se construyó.

Cada versión de prueba trae dos archivos:

| Archivo | Para | Cómo se abre |
|---|---|---|
| `ARLES RELAY_1.2.0_x64-setup.exe` | **Windows** | Doble clic → Windows dirá «editor desconocido» → **Más información** → *Ejecutar de todas formas* |
| `ARLES RELAY_1.2.0_aarch64.dmg` | **macOS** | Doble clic → arrastrar a Aplicaciones → la **primera vez**, clic derecho sobre la app → *Abrir* |

> ⚠️ Esos dos avisos salen **porque la versión no está firmada**, no porque algo
> vaya mal. Es el punto 2 de la tabla de arriba.

### Cómo se genera una nueva

**Creando una etiqueta que empiece por `prueba-`.** Eso lanza la compilación y
publica la release al terminar:

```bash
git tag prueba-2026-09-18
git push origin prueba-2026-09-18
```

> 🔵 **Por qué una etiqueta y no el botón de Actions.** GitHub sólo enseña el
> botón *Run workflow* cuando el archivo del flujo está en la rama **por
> defecto** (`main`), y el trabajo vive en una rama de desarrollo. El botón
> aparecerá solo el día que esto llegue a `main`; mientras tanto, la etiqueta
> hace lo mismo.

> ⚠️ **Nunca se dispara sola.** No hay compilación en cada push: una versión sin
> firmar que se publica sola acaba instalada donde nadie la pidió, y sin firma
> quien la instale no puede comprobar de dónde salió.

Lo que se verá distinto en ella: **la tipografía**, porque va sin Mont. Los
tamaños, los pesos y el ritmo sí son los definitivos — salen de los tokens. Un
hallazgo del tipo «la letra no es la de la marca» en esa versión es esperado.

Y aunque el instalador existiera hoy, **no habría mucho que hacer con él**: la
aplicación todavía no envía correos ni carga contactos. El avance de arriba dice exactamente qué sí.

---

## Qué es

ARLES permite a una empresa ejecutar campañas de **correo y de WhatsApp** usando **sus propias cuentas** y **sus propios límites**.

WhatsApp entró en el alcance de la v1.2.0 el 17/09/2026. **Dos canales y sólo dos**: cada envío es de un canal, las métricas no se suman entre canales y el permiso se registra por canal.

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
