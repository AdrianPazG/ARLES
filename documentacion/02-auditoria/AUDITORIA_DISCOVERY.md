# ARLES RELAY I — Auditoría de Discovery y Propuesta de Arquitectura

**Versión objetivo:** v1.2.0
**Producto de:** TELEMETRY INSIGHT
**Fase:** 0 — Discovery · Auditoría · Arquitectura
**Fecha:** 2026-09-11
**Estado:** propuesta, pendiente de aprobación de Dirección

> Este documento responde al formato A–U solicitado en el §163 del brief maestro.
> **No contiene ni autoriza código de producción.** La puerta de salida está en la sección U.

---

## A. Resumen de entendimiento

ARLES RELAY I es una **aplicación de escritorio para Windows y macOS** que permite a una empresa ejecutar campañas de correo electrónico de forma **controlada, auditable y honesta**, usando sus propias cuentas de correo y sus propios límites.

El bucle central del producto es:

> Configurar empresa → Conectar cuenta de correo → Importar contactos → Organizar audiencia → Crear campaña → Redactar mensaje → Fijar límites de ejecución → Simular → Enviar prueba → Validar (preflight) → Ejecutar → Pausar/Reanudar/Detener → Revisar resultados → Respaldar

Lo que define el producto no es la capacidad de enviar correo —eso lo hace cualquier cosa— sino **el control sobre el envío**: límites que el cliente fija, una simulación que dice cuánto va a tardar de verdad, un motor que respeta los límites incluso cuando el equipo se suspende, idempotencia que garantiza que nadie reciba dos veces el mismo correo, y métricas que no mienten.

Tres principios gobiernan cada decisión de este documento:

1. **Honestidad.** Si el proveedor sólo confirmó «aceptado», la interfaz dice «aceptado». Nunca «entregado».
2. **Seguridad sobre comodidad.** Ante la duda entre un flujo cómodo y uno seguro, gana el seguro, y se documenta el coste.
3. **Autonomía informada.** ARLES advierte y explica; no dicta la lógica de negocio del cliente.

**Lo que ARLES no es**, y que este documento rechazará activamente si reaparece: un CRM, un Mailchimp, una plataforma omnicanal, una herramienta de scraping, un evasor de filtros antispam, o cualquier mecanismo para saltarse los límites de Gmail.

---

## B. Inventario del repositorio

El repositorio contiene **111 archivos, todos binarios de referencia**, y un `README.md` de 7 bytes. **No existe código**: ni `package.json`, ni `Cargo.toml`, ni configuración de build, ni CI, ni tests.

```
/CONCEPTOS_DE_DISEÑO/             27 JPEG          2.0 MB
/REFERENCIAS_VISUALES_DEL_SITIO/   7 JPEG + 1 MP4  4.7 MB
/TIPOGRAFIA/                      89 archivos      11 MB
/REFERENCIA_DE_COLOR/              1 archivo       9.3 MB
/RECURSOS/                        10 archivos      4.9 MB   ← copia parcial de las anteriores
README.md                          7 bytes
```

Rama de trabajo: `claude/loving-faraday-5chmmy`. Historial: 4 commits, todos subidas de assets.

La auditoría forense completa está en **`INVENTARIO_DE_ASSETS.md`**. Siete hallazgos, resumidos:

| Id | Hallazgo | Severidad |
|---|---|---|
| A-01 | `/RECURSOS` es copia parcial obsoleta (1 de 27 conceptos, 1 de 89 tipografías) | Media |
| A-02 | La referencia cromática es un PNG renombrado a `.jpg` | Baja |
| A-03 | La referencia cromática es stock generado por IA (IPTC `trainedAlgorithmicMedia`) | Alta |
| A-04 | La paleta real no es la que asume el brief: 0.73 % de azul profundo | Alta |
| A-05 | `#2CA4D4` falla WCAG contra todo texto candidato | Alta |
| A-06 | No existe ningún activo de marca de ARLES (ni logo, ni flecha, ni escudo) | Alta |
| A-07 | La licencia de Mont probablemente no cubre incrustarla en una app distribuida | **Crítica** |

---

## C. Revisión de `/RECURSOS`

`/RECURSOS` **no contiene lo que el brief afirma que contiene.**

Es un subconjunto degradado de las carpetas de raíz: 1 de 27 conceptos de diseño, 1 de 89 archivos tipográficos, y las 8 referencias visuales completas. Los archivos solapados son idénticos por md5. Quien siga literalmente la instrucción del §12 verá una fracción del material.

Y sobre el contenido en sí: **no hay identidad de marca**. El §12 promete «referencias de interfaz, estilo de UI, identidad de marca, tipografía y activos gráficos», y T-9 menciona logos concretos (flecha, escudo). Ninguno existe en el repositorio.

Lo que sí hay, y su valor real:

**`/CONCEPTOS_DE_DISEÑO` (27 imágenes).** Carruseles de consejos de UX de redes sociales, con marca de agua de @ux_snacks y @uxwithvamshi. Uno de ellos (`20.jpeg`) es directamente un anuncio de un playbook de pago. Sirven como recordatorio heurístico genérico —«tarjeta seleccionable en vez de radio button cuando las opciones tienen atributos»— pero **no son referencias de la identidad de ARLES**.

**`/REFERENCIAS_VISUALES_DEL_SITIO` (7 imágenes + 1 vídeo).** Mockups de terceros: un dashboard fintech de marca «COINEST» (© 2024 Peterdraw) y una lámina de design system de «Milray Park». Éstos sí tienen valor directo, y exactamente el que el §12 prescribe: referencia de **densidad de información, ritmo vertical, jerarquía tipográfica y tratamiento de tablas**. El dashboard COINEST en particular resuelve bien el problema que ARLES también tiene —muchas cifras y varias listas en una sola pantalla sin que se sienta apretado— mediante tarjetas de altura desigual en una rejilla de tres columnas con un carril lateral estrecho.

Son IP de terceros: se estudian, no se reproducen.

**Conclusión de C:** `/RECURSOS` aporta vocabulario de densidad y jerarquía. No aporta identidad. La identidad hay que construirla, y depende de que Dirección entregue activos de marca (A-06) y resuelva la licencia tipográfica (A-07).

---

## D. Revisión de `/REFERENCIA_DE_COLOR`

El archivo es `farm-lifestyle-digital-art.jpg`, 9.3 MB. Tres cosas que hay que decir antes de extraer un solo color.

**No es un JPEG.** Sus primeros bytes son `89 50 4E 47 0D 0A 1A 0A`: es un PNG de 2320×3080, RGB de 8 bits, renombrado. Cualquier herramienta que enrute por extensión fallará.

**Es contenido generado por IA.** Su XMP declara `Iptc4xmpExt:DigitalSourceType = trainedAlgorithmicMedia`, el código IPTC estándar para media sintética, creado el 2024-05-17 en Photoshop 25.7. El nombre del archivo y el vídeo rawpixel hermano apuntan a stock de rawpixel. Esto **contradice T-9**: no es un activo propiedad de TELEMETRY.

El matiz legal, que es favorable: **los colores no son protegibles por derecho de autor.** Extraer una paleta es limpio. Lo que no se puede hacer es distribuir el archivo dentro del producto ni presentarlo como activo de marca propio.

**No es una paleta de *La noche estrellada*.** Decodifiqué el PNG completo y medí la distribución real:

```
cian / azur   (H 185-210°)   54.35 %    ← una sola familia domina
ocre/naranja  (H  15- 38°)   19.48 %
amarillo/oro  (H  38- 62°)   13.44 %
teal, verde, neutro, rojo      11.77 %
AZUL PROFUNDO (H 210-250°)    0.73 %    ← prácticamente ausente
```

El azul profundo —el color que *define* el cuadro de Van Gogh— no está. La imagen es una composición complementaria cian-contra-cálido, más cercana a un paisaje al atardecer que a un cielo nocturno. El 76 % de los píxeles vive entre L 20 y L 60; sólo el 1 % baja de L 10.

**Las dos consecuencias que el brief no anticipa:**

1. **No hay separación de tono disponible.** El §16 pide tres azules distintos para tres roles distintos (estructura, superficie, interacción), pero el 54 % de la imagen cabe en ±12°. Extraerlos literalmente los colapsa, y el resultado es una interfaz monocromática cian donde nada distingue el chasis de lo pulsable.

2. **El fondo profundo hay que fabricarlo.** El §18 pide dark-first por capas de azul, pero los oscuros que la imagen tiene son cian apagado (`#041C2C`) o tierras cálidas turbias (`#341C14`). Ninguno sirve tal cual como fondo de aplicación.

**Y el dato de accesibilidad que decide el sistema entero:**

| Superficie | `#F4ECE4` | `#FCCC0C` | Blanco |
|---|---|---|---|
| `#041C2C` profundo | 14.88:1 ✅ | 11.41:1 ✅ | 17.39:1 ✅ |
| `#045484` superficie | 6.89:1 ✅ | 5.29:1 ✅ | 8.06:1 ✅ |
| `#0C749C` elevada | 4.50:1 ⚠️ | 3.45:1 ❌ | 5.26:1 ✅ |
| `#2CA4D4` cian vivo | 2.44:1 ❌ | 1.87:1 ❌ | 2.85:1 ❌ |

`#2CA4D4` **no puede ser una superficie**: falla contra todo, blanco incluido. Y `#FCCC0C` tiene luminancia relativa 0.639 —más cerca del blanco que del negro—, así que se comporta como un color *claro*: brilla sobre `#041C2C` (11.41:1) y es inutilizable como relleno de botón con texto blanco (1.56:1).

Esto invierte una intuición habitual: **el amarillo de ARLES es luz sobre oscuro, no señal sobre claro.** Encaja perfectamente con la estrategia dark-first, y prohíbe de raíz el patrón «botón amarillo, texto blanco».

**Decisión (D-2):** se extraen el cian, el amarillo y los cremas reales, y se **construyen** por rampa los azules profundos que faltan. La imagen queda como referencia interna de estudio; nunca se distribuye. Ver ADR-0005.

---

## E. Auditoría del producto (crítica)

Aquí está lo que creemos que está mal en la idea, en el formato *Problema → Riesgo → Impacto → Alternativa → Trade-off → Recomendación*.

### E-1. La detección de rebotes es imposible tal y como está especificada

**Problema.** El §37 exige que los hard bounces alimenten automáticamente la lista de supresión. El §69 prohíbe pedir permisos de lectura de bandeja de entrada en v1.2.0. Los rebotes duros llegan **como correo asíncrono a la bandeja del remitente**, minutos u horas después del envío. Sin leer un buzón no hay forma de verlos.

**Riesgo.** La lista de supresión —el control anti-abuso central del producto— se queda sin su principal fuente de alimentación. Los contactos muertos se reintentan campaña tras campaña.

**Impacto.** Alto, y creciente. Una tasa de rebote sostenida por encima del ~2 % daña la reputación del dominio; por encima del 5 % un proveedor puede empezar a rechazar todo el flujo. Es el mecanismo exacto por el que un cliente de ARLES podría quemarse su propio dominio **usando el producto correctamente**.

**Alternativas.**
- (a) Capturar sólo los rechazos **síncronos** 5xx de SMTP en el momento del envío. Atrapa buzones inexistentes que el servidor destino rechaza en el diálogo SMTP; no atrapa los asíncronos.
- (b) VERP: `Return-Path` único por destinatario apuntando a un **buzón de rebotes dedicado** que el cliente configura, leído por IMAP. Sólo esa cuenta, nunca la bandeja personal del usuario. Respeta el espíritu del §69 (no fisgar correo del usuario) cumpliendo el §37.
- (c) Diferir la detección automática y permitir sólo supresión manual en v1.2.0.

**Trade-off.** (b) es la solución correcta y la que usa la industria, pero añade un dominio entero al alcance (configuración IMAP, parseo de DSN según RFC 3464, clasificación duro/blando) y pide al cliente que configure un buzón extra. (a) es barato pero parcial. (c) no resuelve nada por sí solo.

**Recomendación.** **(a) + (c) en v1.2.0, (b) en v1.3.** Y —esto es lo importante— la interfaz debe **decir explícitamente** que la detección de rebotes es parcial en esta versión. Un cliente que cree que ARLES le protege de los rebotes cuando no lo hace está peor que uno que sabe que debe revisarlo a mano. Ver ADR-0009.

### E-2. «Beta comercial» sin licenciamiento es una contradicción

**Problema.** T-4 define v1.2.0 como beta comercial. T-6 excluye el enforcement de licencias de v1.2.0.

**Riesgo.** Sin enforcement no hay forma de limitar instalaciones, hacer caducar un piloto ni cobrar. Y el software instalable que ya salió no se puede «des-distribuir».

**Impacto.** Medio-alto, comercial y contractual.

**Alternativa.** Despliegue interno de TELEMETRY en v1.2.0; comercial en v1.3 con licenciamiento real.

**Trade-off.** Retrasa la validación con clientes reales. A cambio difiere el certificado de firma EV, la verificación OAuth de Google y el canal de soporte —los tres costes de calendario del proyecto.

**Recomendación.** Aceptada como **D-4**.

### E-3. La verificación OAuth de Google no está en el roadmap y es el camino crítico

**Problema.** El brief planifica 16 fases y ninguna contempla el proceso de verificación de Google. `gmail.send` es un scope **sensible**: evita la auditoría CASA Tier 2 (que sí exigirían `gmail.modify` o `mail.google.com`, con coste de auditor externo y renovación anual), pero **exige verificación OAuth**: política de privacidad pública, dominio verificado, vídeo demostrativo y una revisión de semanas.

**Riesgo.** Descubrirlo en la Fase 6 congela el release de 4 a 8 semanas con el producto terminado.

**Impacto.** Crítico para el calendario.

**Alternativas.** (a) Un único client ID de TELEMETRY verificado. (b) Cada organización registra su propio proyecto de Google Cloud. (c) SMTP primero, OAuth en paralelo.

**Trade-off.** (a) da onboarding de dos clics pero concentra el riesgo: un cliente abusivo puede tumbar el client ID de todos. (b) elimina el bloqueo pero es inviable para un cliente no técnico. (c) desacopla el release del calendario de Google.

**Recomendación.** **(c)**, aceptada como **D-1**. En v1.2.0 se envía por SMTP —Gmail lo admite con contraseña de aplicación, sin verificación alguna—, la abstracción `EmailProvider` se diseña completa desde el día uno, y el trámite de verificación arranca en la Fase 1 para que `GoogleProvider` aterrice en v1.2.x. Ver ADR-0003.

### E-4. El numeral «I» junto al número de versión confunde

**Problema.** «ARLES RELAY I / v1.2.0» pone dos significantes de generación en la misma línea. El lector natural asume que «I» es la versión 1 y se pregunta por qué dice 1.2.0.

**Riesgo.** Bajo, pero permanente: está en cada pantalla y en cada instalador.

**Alternativa.** «I» vive en la marca comercial y el empaque; el número de versión aparece solo, en el pie y en el «Acerca de».

**Recomendación.** Retener «I» según T-5, pero **no mostrarlos adyacentes**. Ver ADR-0010.

### E-5. Lo que el brief acierta y conviene blindar

No todo es crítica. Tres decisiones del brief son inusualmente buenas y hay que protegerlas de la erosión de alcance:

- **Prohibir la rotación automática de remitentes (T-1).** La mayoría de herramientas del sector la venden como funcionalidad estrella. Es evasión de límites, viola las políticas de Google y es exactamente la razón por la que se queman dominios. Pausar la cola con un aviso claro es la decisión correcta y hay que sostenerla cuando un cliente la pida.
- **Exigir envío de prueba antes de activar (§45).** Barato de implementar, evita la clase de error más cara del producto.
- **Prohibir métricas falsas (§65).** «Aceptado» no es «entregado», y decirlo cuesta credibilidad a corto plazo y la gana a largo.

---

## F. Auditoría de alcance (viabilidad de v1.2.0)

Evaluación honesta de lo que cabe en la versión.

### Cabe, y es el núcleo irreductible

Configuración de empresa · Importación XLSX/CSV con mapeo y deduplicación · Contactos, listas, etiquetas, campos personalizados · Lista de supresión · `SMTPProvider` · Editor de mensaje simple con variables y firmas · Plantillas versionadas · Límites de ejecución configurables · Simulador de campaña · Preflight · Envío de prueba · Motor de ejecución en segundo plano con cola persistente, idempotencia, throttling, reintentos y circuit breaker · Pausa/Reanudar/Detener · Dashboard y actividad en vivo · Respaldo y restauración · Cifrado en reposo · Bitácora de auditoría

### No cabe, y debe diferirse explícitamente

| Elemento | Destino | Motivo |
|---|---|---|
| `GoogleProvider` (OAuth) | v1.2.x | Bloqueado por verificación de Google (D-1) |
| `MicrosoftProvider` | v1.3 | T-6 |
| Detección automática de rebotes por VERP+IMAP | v1.3 | E-1 / ADR-0009 |
| Cifrado de respaldos | v1.3 | T-6 |
| Enforcement de licencias | v1.3 | T-6 / D-4 |
| Tracking de aperturas y clics | v1.3+ | §67, y sólo si es opt-in y respetuoso |
| White labeling | v1.4+ | §117 — sólo preparar la arquitectura |
| Centro de entregabilidad completo (SPF/DKIM/DMARC) | **parcial** en v1.2.0 | Ver abajo |

**Sobre el centro de entregabilidad (§64).** Comprobar SPF, DKIM y DMARC de un dominio es una consulta DNS: es barato y de altísimo valor, y entra en v1.2.0. Lo que no entra son las tasas de rebote y de queja, porque dependen de E-1.

### El riesgo de alcance real

El brief tiene 175 secciones. El peligro no es que ninguna sea irrazonable —casi todas lo son— sino que **el motor de ejecución es el 70 % del valor y el 70 % del riesgo técnico**, y es tentador dejarlo para el final porque las pantallas se ven más. Idempotencia, reanudación tras suspensión del equipo, respeto de límites por ventana horaria y parada de emergencia son problemas de sistemas distribuidos disfrazados de aplicación de escritorio.

**Recomendación de secuenciación:** el motor se construye **antes** que la UI de campañas, y se prueba con un proveedor simulado. Si el motor no es correcto, nada de lo demás importa.

---

## G. Arquitectura propuesta

**Monolito modular**, confirmado. Sin microservicios, sin Kubernetes, sin colas distribuidas. Es una aplicación de escritorio de un solo proceso con un motor en segundo plano.

Workspace de Cargo con un crate por dominio, y dependencias que sólo apuntan hacia dentro:

```
arles-core        Tipos de dominio, errores, ids tipados, Secret<T>. Sin I/O.
arles-db          Conexión SQLCipher, migraciones, repositorios.
arles-company     Empresa, preferencias, licencia (esquema, sin enforcement en v1.2.0).
arles-contacts    Contactos, listas, etiquetas, campos personalizados, filtros.
arles-import      Lectura XLSX/CSV, detección, mapeo, validación, deduplicación.
arles-suppression Lista de supresión. Autoridad final sobre si un envío procede.
arles-email       Trait EmailProvider + SmtpProvider (+ GoogleProvider en v1.2.x).
arles-template    Sustitución de variables y sanitizado HTML.
arles-campaign    Campañas, audiencias, simulador, preflight.
arles-engine      Cola, planificador, workers, throttling, reintentos, breaker.
arles-audit       Bitácora append-only de acciones críticas.
arles-backup      Exportación/importación .arles.
arles-app         Orquestación: comandos Tauri, estado, arranque.
```

Frontend Vue 3 con TypeScript estricto, organizado por la misma división de dominios para que un cambio de producto toque una sola columna vertical.

**Tres reglas de frontera que hay que sostener:**

1. **El frontend no decide nada de negocio.** Ningún límite, ninguna validación de supresión, ningún sanitizado es autoritativo en la webview. La webview es una superficie de presentación; la autoridad vive en Rust. Esto no es purismo: la webview es la superficie con mayor exposición a contenido no confiable (nombres de contactos importados, HTML de plantillas).
2. **`arles-core` no hace I/O.** Es lo que mantiene el dominio testeable sin base de datos ni red.
3. **`arles-suppression` se consulta en el momento del envío, no sólo al construir la audiencia.** Una campaña de tres días puede acumular bajas entre que se calculó la audiencia y que le toca el turno a un contacto.

Detalle completo en `03-arquitectura/ARQUITECTURA.md`.

---

## H. Comparación Tauri frente a alternativas

**Veredicto: Tauri 2 confirmado** (línea 2.11.x, estable desde octubre de 2024). Detalle en ADR-0001.

| Criterio | Tauri 2 | Electron | Valoración |
|---|---|---|---|
| **Superficie de ataque** | Sin Chromium ni Node empaquetados. Sistema de *capabilities*: se deniega por defecto y se habilitan permisos por ventana | Chromium + Node completos en el bundle; el acceso a `fs`, `child_process` y la red está a un `require` de distancia si se cuela XSS | **Tauri, decisivo** |
| **Cadencia de CVEs** | Se hereda la del WebView del SO, que el SO parchea | Cada CVE de Chromium o Node obliga a reempaquetar y redistribuir | **Tauri** |
| **RAM** | 50–75 % menor | Referencia | **Tauri** |
| **Tamaño del bundle** | ~3–10 MB | ~96 MB+ | **Tauri** |
| **Consistencia de renderizado** | WebView2 (Chromium) en Windows vs WKWebView (Safari) en macOS — **divergen** | Idéntico en todas las plataformas | **Electron** |
| **Madurez del ecosistema** | Menor; algunos plugins jóvenes | Muy maduro | **Electron** |
| **Actualizaciones** | Plugin updater con firma propia; hay que custodiar claves | Mecanismos muy rodados | Empate con matices |
| **Encaje con el equipo** | Núcleo en Rust — T-10 | Node/TS en el backend | **Tauri** |

**El argumento que decide no es el tamaño, es la superficie de ataque.** ARLES custodia listas de datos personales de terceros bajo la LFPDPPP, credenciales de correo y la reputación del dominio del cliente. En Electron, una inyección de HTML en una plantilla de correo está a un paso del sistema de archivos. En Tauri, con capabilities denegadas por defecto y sin `shell` ni `fs` amplio expuestos, ese paso no existe.

**El coste que se acepta, y que hay que decir en voz alta:** WKWebView y WebView2 **no renderizan igual**. Es la fuente número uno de bugs visuales en Tauri. La mitigación es que cada pantalla se valide en ambas plataformas antes de darse por hecha, y que el presupuesto de QA lo contemple desde el principio, no como un remate final.

**Alternativas descartadas.** *Electron*: superficie de ataque y peso, ambos incompatibles con el perfil del producto. *Nativo (WinUI + SwiftUI)*: mejor rendimiento y sensación de plataforma, pero duplica el frontend entero y no hay equipo para dos UIs. *Flutter Desktop*: buena consistencia, pero aleja el núcleo de Rust y su soporte de escritorio en Windows sigue siendo el eslabón débil.

---

## I. Modelo de datos

SQLite con **SQLCipher** (cifrado en reposo, T-3). Detalle en `03-arquitectura/MODELO_DE_DATOS.md`.

Entidades principales:

```
company                 Datos de la empresa, zona horaria, firma por defecto
email_account           Cuenta remitente. NUNCA credenciales: sólo una referencia al llavero
contact                 Contacto. email_normalized con índice único por empresa
contact_field           Campos personalizados (clave/valor tipado)
contact_list            Listas
contact_list_member     Pertenencia
tag / contact_tag       Etiquetas
import_batch            Un archivo importado: origen, mapeo, totales, errores
suppression_entry       Supresión. Motivo, origen, fecha. Autoridad absoluta
template                Plantilla
template_version        Versión inmutable de una plantilla (§44)
campaign                Campaña, estado, audiencia congelada, límites
campaign_audience       Snapshot de la audiencia al activar
message_attempt         **La tabla crítica.** Un intento por contacto y campaña
execution_window        Configuración de días y horas operativas
rate_budget             Estado del token bucket por cuenta y ventana
audit_log               Append-only. Acciones críticas
schema_migration        Control de versión del esquema
```

### Decisiones que importan

**Normalización de correo.** `email_normalized` = recorte de espacios + minúsculas en todo el valor. **No se eliminan los puntos ni se recorta el sufijo `+`**: eso es específico de Gmail y aplicarlo de forma general fusiona contactos que son personas distintas en otros dominios. Índice único `(company_id, email_normalized)`. La deduplicación al importar usa esta columna (§36).

**`message_attempt` es un outbox, no un log.**

```sql
UNIQUE (campaign_id, contact_id)          -- imposible duplicar por diseño
idempotency_key  TEXT NOT NULL            -- UUID, generado ANTES de enviar
                                          -- viaja como Message-Id del correo
state            TEXT NOT NULL            -- ver máquina de estados en J
```

La restricción única es lo que hace que el §55 (idempotencia) sea una propiedad del esquema y no una esperanza del código.

**La supresión gana siempre.** No es un filtro que se aplica al construir la audiencia: es una comprobación en el momento del envío. Una importación nueva **nunca** sobrescribe una supresión (§39); si un contacto suprimido reaparece en un archivo, se importa el contacto y la supresión permanece.

**Escala (T-7: 500 000 contactos).** SQLite aguanta esto sin dificultad. Lo que hay que hacer bien: modo WAL, paginación por *keyset* y nunca `OFFSET` en tablas grandes, importación por lotes dentro de una transacción, e índices sobre `(campaign_id, state)` y `(company_id, email_normalized)`. SQLCipher añade del orden de 5–15 % de sobrecoste, asumible.

---

## J. Modelo de ejecución

Es el corazón del producto y donde se concentra el riesgo técnico. Detalle en `03-arquitectura/MOTOR_DE_EJECUCION.md`.

### Máquina de estados

```
queued ──claim──► claimed ──► sending ──┬──► sent
   │                  │                 ├──► failed ──(reintento)──► queued
   │                  │                 └──► permanently_failed
   │                  └──(parada)──► cancelled
   └──(supresión en el momento del envío)──► suppressed

                    sending ──(la app murió aquí)──► presumed_sent
```

### Idempotencia: el caso que nadie documenta

La restricción única impide duplicar un intento. Las transiciones se hacen por **compare-and-swap** —`UPDATE ... WHERE state = 'queued'` y se verifica el número de filas afectadas— para que sólo un worker gane la carrera.

Pero queda un caso genuinamente irresoluble: **la aplicación muere después de que el proveedor aceptó el mensaje y antes de que podamos registrar el commit.** Al reiniciar, toda fila que quedó en `sending` es ambigua: el correo pudo salir o no. No hay forma de saberlo sin leer la bandeja de enviados, y en v1.2.0 no tenemos ese permiso (§69).

**Decisión: nunca se reenvía automáticamente.** La fila pasa a `presumed_sent`, se marca en la interfaz y el usuario decide. Preferimos un no-envío a un duplicado, porque el duplicado es irreversible y quema reputación. Ver ADR-0004.

### Suspensión del equipo y el problema de la ráfaga

El §53 prohíbe que al despertar el equipo se dispare una ráfaga de correos acumulados. Esto es más sutil de lo que parece: un token bucket ingenuo *acumula* permisos mientras duerme y los libera todos de golpe.

**Detección:** se comparan el reloj monótono (`Instant`) y el reloj de pared (`SystemTime`). Si el salto del reloj de pared supera con mucho al del monótono, hubo suspensión.

**Respuesta:** el bucket **se reinicia, no se acumula**. Los presupuestos horario y diario se recalculan desde cero contra la ventana actual en la zona horaria de la empresa. Un correo no enviado ayer no se recupera hoy: se queda sin enviar y la campaña dura más. Eso es lo correcto.

### Throttling, reintentos y freno

- **Token bucket persistido** por cuenta remitente, con ventana horaria y diaria en la zona horaria de la empresa (no la del sistema operativo: un portátil que viaja no debe cambiar la política de envío).
- **429 con `Retry-After`** se respeta literalmente. Nunca se ignora.
- **Backoff exponencial con jitter** para fallos transitorios. Los 5xx permanentes no se reintentan: van a `permanently_failed` y alimentan la supresión.
- **Circuit breaker por cuenta**: N fallos consecutivos abren el circuito y pausan esa cuenta con un aviso explícito en la interfaz.
- **Sin rotación automática de remitentes (T-1).** Al alcanzar el límite, la cola **pausa** y avisa: «La cuenta [correo] alcanzó su límite…». No se salta a otra cuenta. Ver ADR-0008.
- **Parada de emergencia**: bandera en base de datos más un token de cancelación. Los workers la comprueban **antes de cada envío**, no entre lotes. `PAUSAR` conserva la cola; `DETENER` la cancela y exige confirmación explícita.

### El motor vive fuera de la ventana

Corre en su propio conjunto de tareas, independiente de la ventana principal (§51). Cerrar la ventana no detiene la campaña; se mantiene el icono en la bandeja del sistema o la barra de menús. Cerrar la ventana **nunca** debe parecer que detiene una campaña: la interfaz debe decirlo.

---

## K. Modelo de seguridad

Alineado con OWASP ASVS y Top 10. Detalle en `04-seguridad/THREAT_MODEL.md`.

### Secretos

**Ninguna credencial toca la base de datos, la configuración, el `localStorage` ni los registros.** Van al llavero del sistema operativo vía `keyring-rs`: Keychain en macOS, Credential Manager en Windows. La tabla `email_account` guarda una *referencia*, no un secreto.

La clave maestra de SQLCipher son 32 bytes aleatorios generados en el primer arranque y custodiados en el mismo llavero. **Si el llavero no está disponible, la aplicación se niega a arrancar.** No hay degradación a texto plano — esa es la clase de «fallo cómodo» que convierte un cifrado en teatro.

En código, un tipo `Secret<T>` cuyas implementaciones de `Debug` y `Display` emiten `[REDACTADO]`, para que un `tracing` descuidado no pueda filtrar un token.

### Superficie de Tauri

Capabilities **denegadas por defecto**. Sin plugin `shell`. Sin acceso amplio al sistema de archivos: sólo el diálogo de selección de archivos con alcance acotado, y las rutas elegidas nunca se usan tal cual para escribir. CSP estricta en la webview: sin `unsafe-inline`, sin `unsafe-eval`, sin orígenes remotos.

### Inyección, por vector

| Vector | Defensa |
|---|---|
| **SQL** | Consultas parametrizadas siempre. Los filtros dinámicos se construyen con un constructor tipado, jamás por concatenación |
| **Plantillas** | Motor propio de sustitución `{{clave}}` contra un mapa cerrado. **No Turing-completo**: nada de Handlebars con helpers ni Tera con lógica. Un motor con lógica evaluando datos de un XLSX ajeno es una vía de ejecución |
| **HTML / XSS** | Sanitizado con `ammonia` (lista blanca) **en Rust**, al guardar y otra vez al enviar. El sanitizado del frontend no es autoritativo |
| **Fórmulas CSV** | Al **exportar**, se antepone `'` a toda celda que empiece por `= + - @`, tabulador o retorno de carro. Al importar jamás se evalúa nada |
| **Zip bomb (XLSX)** | `calamine` en streaming, con tope de ratio de descompresión (~100:1), tope de filas, tope de celdas y timeout |
| **Nombre de archivo malicioso** | El nombre suministrado **nunca** se usa para escribir en disco. Se genera un UUID y el nombre original se guarda como metadato |

### Registro y auditoría

`tracing` con una capa de redacción. Nunca contraseñas, tokens ni contenido íntegro de mensajes. Bitácora append-only (`audit_log`) de las acciones críticas: activar campaña, detener, conectar o desconectar cuenta, importar, suprimir, restaurar respaldo, y la **aceptación explícita del aviso de más de 50 correos diarios** (§48).

### Privacidad (T-8, LFPDPPP)

Alcance estricto a la legislación mexicana en v1.2.0. Derechos ARCO: la arquitectura debe permitir localizar, exportar, rectificar y eliminar todos los datos de un titular. Se documenta en `04-seguridad/PRIVACIDAD_LFPDPPP.md`.

**Una advertencia que hay que dar a Dirección:** ARLES no puede garantizar que el cliente tenga consentimiento para los contactos que importa. El producto debe pedir una afirmación explícita de origen lícito en cada importación, registrarla en la bitácora y dejar claro que la responsabilidad como responsable del tratamiento es del cliente.

---

## L. UX y navegación

### Crítica de la navegación propuesta (§23)

Las siete secciones propuestas se solapan en dos puntos: **MESSAGES** (plantillas y firmas) es un insumo de las campañas, no un destino por derecho propio; y **ACTIVITY** duplica en buena parte **EMAIL > Deliverability Health**.

**Propuesta a seis secciones:**

| Sección | Contiene |
|---|---|
| **INICIO** | Panel: campañas activas, progreso de hoy, próximos envíos, errores, avisos |
| **CAMPAÑAS** | Activas · Programadas · Historial · **Plantillas y firmas** |
| **CONTACTOS** | Todos · Listas · Etiquetas · Importaciones · **Supresiones** |
| **REMITENTES** | Cuentas conectadas · Salud de envío (SPF/DKIM/DMARC) · Límites |
| **ACTIVIDAD** | Ejecución en vivo · Eventos · Errores |
| **AJUSTES** | Empresa · Preferencias · Respaldos · Actualizaciones · Licencia |

«REMITENTES» en vez de «EMAIL» porque nombra lo que el usuario administra ahí —sus cuentas de envío— y no una tecnología.

### Onboarding

Nada de tours emergentes (§25). Una lista de verificación persistente en INICIO, que se contrae cuando se completa y se puede volver a abrir: **1** Configurar empresa · **2** Conectar cuenta de correo · **3** Preferencias de ejecución · **4** Importar contactos · **5** Primera campaña.

### Escritorio de verdad

De 1366×768 a 4K, con escalado de Windows del 100 % al 200 % (§22). Ventana mínima estricta. **Nada de patrones móviles**: sin menú hamburguesa, sin gestos, sin diseño adaptable a pantallas de teléfono. Navegación completa por teclado, densidad de tabla alta, atajos para las acciones frecuentes.

### Tono (§94)

Profesional y claro. Los errores dicen **qué pasó, cómo arreglarlo y qué está a salvo** — esa tercera parte es la que falta en casi todo el software y la que importa cuando alguien acaba de ver fallar una campaña. Nada de «¡Ups!» ni «¡Genial!» en errores de negocio.

Toda pantalla define sus cuatro estados: vacío, cargando, error y éxito (§97).

---

## M. Dirección del Design System

**Dark-first** (§18), construido por capas sobre azules profundos **derivados**, no extraídos (D-2, ADR-0005).

### Arquitectura de superficies

```
--arles-bg-deep        #041C2C   fondo de aplicación
--arles-surface        #062F47   paneles
--arles-surface-raised #045484   tarjetas, filas elevadas
--arles-surface-hover  #0C749C   estado interactivo
--arles-border         #0C5C8C   separadores
--arles-accent         #FCCC0C   acento — trazo, foco, dato clave
--arles-info           #2CA4D4   sólo trazo y texto, NUNCA relleno con texto encima
--arles-warning        #AC5C0C   advertencia cálida
--arles-text           #F4ECE4   texto primario
--arles-text-muted     #E4E4CC   texto secundario
```

Tokens centralizados. **Ningún color se escribe a mano en un componente** (§17). Se documenta en `05-diseno/COLOR_SYSTEM.md` junto al método de extracción, de modo que cualquiera pueda recalcular las cifras sobre el mismo PNG.

### Las tres reglas que salen de los datos

1. **`#2CA4D4` nunca es relleno de una superficie con texto.** Falla WCAG contra todo, blanco incluido (2.85:1). Es color de trazo, de borde y de foco.
2. **El amarillo es luz, no señal.** Luminancia relativa 0.639. Brilla sobre `#041C2C` (11.41:1) y es inservible con texto blanco encima. Se usa con intención estricta: si todo es amarillo, nada es importante (§18).
3. **La información nunca se comunica sólo por color** (§19). Todo estado lleva forma, icono o texto además del color.

### Interpretación visual

La inspiración de *La noche estrellada* se traduce **abstractamente**: profundidad por capas, contraste entre frío y un acento cálido, ritmo en el espaciado. **Ningún elemento literal**: sin girasoles, sin cielos estrellados, sin pinceladas decorativas (§15). El resultado buscado es tecnológico, corporativo, premium y sobrio.

### Tipografía

Mont, escala definida en `05-diseno/TIPOGRAFIA.md`, con Mont Black reservado al logotipo y a los titulares de nivel superior (§20). **Bloqueado por D-3** hasta que se resuelva la licencia. El plan B —Mont sólo en marketing, logotipo como SVG con contornos, y una geométrica de licencia libre dentro de la aplicación— está documentado y listo para activarse.

---

## N. Estrategia de QA

Detalle en `06-calidad/ESTRATEGIA_QA.md`.

**Reparto.** Tests unitarios de Rust sobre la lógica de dominio (el grueso) · tests de integración sobre el motor con un proveedor simulado · Vitest sobre los componentes de Vue · Playwright sobre el camino dorado completo.

**El camino dorado**, extremo a extremo: configurar empresa → conectar SMTP → importar 1 000 contactos → crear lista → crear campaña → redactar con variables → fijar límites → simular → preflight → envío de prueba → activar → pausar → reanudar → completar → revisar resultados → respaldar.

**Los casos frontera que de verdad rompen el producto**, y que hay que escribir explícitamente:

- El equipo se suspende a mitad de campaña y despierta al día siguiente → **no hay ráfaga**
- Cambio de horario de verano dentro de la ventana de ejecución
- La campaña cruza la medianoche en la zona horaria de la empresa
- Se revoca el token OAuth o caduca la contraseña de aplicación con la campaña corriendo
- Se pierde la conexión a mitad de envío
- Doble clic en «Activar»
- La aplicación muere entre el `accept` del proveedor y el commit → `presumed_sent`, **jamás duplicado**
- Base de datos bloqueada por otra instancia
- Un XLSX con una bomba zip, con 500 000 filas, con fórmulas, con nombres de archivo maliciosos
- Un contacto suprimido a mitad de campaña → **no recibe**
- Límite alcanzado a mitad de campaña → **pausa con aviso, sin rotación**

**Las cuatro auditorías** (§140) se ejecutan tras cada hito, de forma aislada: Arquitectura, Seguridad, QA/Rendimiento, UX/Accesibilidad.

**Y lo que Tauri obliga:** cada pantalla se valida en Windows y en macOS antes de darse por terminada. WebView2 y WKWebView divergen, y descubrirlo al final es caro.

---

## O. Estrategia de rendimiento

Objetivos con **500 000 contactos** (T-7). Detalle en `06-calidad/PRESUPUESTO_RENDIMIENTO.md`.

| Operación | Presupuesto |
|---|---|
| Arranque en frío hasta interfaz usable | < 2 s |
| Abrir la tabla de contactos con 500 k filas | < 500 ms hasta primer pintado |
| Desplazamiento en tabla virtualizada | 60 fps sostenidos |
| Filtro sobre 500 k contactos | < 1 s |
| Importación de 100 k filas XLSX | < 60 s, memoria acotada |
| Cálculo de audiencia de campaña | < 2 s |
| Memoria en reposo con el motor activo | < 250 MB |

**Cómo se consigue:** virtualización de tablas con TanStack Virtual · paginación por keyset, nunca `OFFSET` · índices sobre las columnas que de verdad se filtran · importación por lotes en transacción · lectura XLSX en streaming · el motor en su propio conjunto de tareas para que nunca bloquee la interfaz.

**Medir, no suponer:** se genera un conjunto sintético de 500 000 contactos y los presupuestos se comprueban en CI, no a ojo.

---

## P. Estrategia de distribución

`.exe` (MSI/NSIS) para Windows, `.dmg` para macOS. Detalle en `07-entrega/DISTRIBUCION_Y_FIRMA.md`.

**Firma de código — dos costes de calendario que hay que arrancar pronto:**

- **Windows:** certificado OV o EV. Sin firma, SmartScreen muestra una advertencia que en la práctica bloquea la instalación en un entorno corporativo. La emisión lleva de una a tres semanas.
- **macOS:** Apple Developer ID más **notarización**. Sin notarizar, Gatekeeper directamente no abre la aplicación.

**D-4 difiere ambos** al ser v1.2.0 un despliegue interno de TELEMETRY. Pero el trámite debe iniciarse en la Fase 1: son plazos externos que no se pueden comprimir después.

**Actualizaciones (§79).** El plugin updater de Tauri verifica firma criptográfica antes de aplicar. **Las claves privadas de firma nunca entran al repositorio** — van a los secretos del sistema de CI, y la clave pública se empaqueta con la aplicación.

---

## Q. Recomendación de licenciamiento

El enforcement está fuera de v1.2.0 (T-6, D-4), pero el **esquema** se prepara ahora para no tener que migrar datos después.

**Forma recomendada:** licencia firmada criptográficamente, verificable **sin conexión**. Un archivo o cadena firmado con la clave privada de TELEMETRY, que la aplicación verifica con la clave pública empaquetada. Contiene: organización, edición, límites de asientos, fecha de caducidad y capacidades habilitadas.

**Por qué sin conexión:** el §77 pide un periodo de gracia, y tiene toda la razón. Un producto de escritorio que deja de funcionar porque el servidor de licencias no responde es un producto que un día deja tirado a un cliente en mitad de una campaña. La validación en línea, cuando exista, será para *renovar* y *revocar*, nunca para *autorizar el arranque*.

**Periodo de gracia:** al caducar, la aplicación entra en modo restringido —se pueden consultar datos, exportar y respaldar; no se pueden activar campañas nuevas—. **Nunca se bloquea el acceso a los datos del cliente.** Los datos son suyos.

**Capacidades por edición (`FeatureEntitlement`)** desde el principio, para que activar una funcionalidad en v1.3 no obligue a reescribir el arranque.

---

## R. Matriz de riesgos

| # | Riesgo | Prob. | Impacto | Mitigación |
|---|---|---|---|---|
| R-01 | La licencia de Mont no cubre app embedding (A-07) | **Alta** | **Alto** | **D-3, bloqueante.** Verificar alcance y adquirir App License antes de Fase 2. Plan B documentado y listo |
| R-02 | La verificación OAuth de Google retrasa el release (E-3) | Alta | Alto | **D-1.** SMTP primero; el trámite arranca en Fase 1 en paralelo |
| R-03 | Enviar duplicados por fallo del motor | Media | **Crítico** | Restricción única + CAS + `presumed_sent` sin reenvío automático (ADR-0004) |
| R-04 | Ráfaga de envíos al despertar el equipo | Media | Alto | Detección de suspensión por reloj monótono + reinicio del bucket, sin acumulación |
| R-05 | Un cliente quema su dominio usando ARLES correctamente | Media | Alto | Aviso de más de 50 diarios con aceptación registrada, simulador, comprobación SPF/DKIM/DMARC, sin rotación de remitentes |
| R-06 | Sin detección de rebotes la supresión se degrada (E-1) | **Alta** | Medio | Rechazos 5xx síncronos + supresión manual en v1.2.0; VERP+IMAP en v1.3. **Decirlo en la interfaz** |
| R-07 | Divergencia de renderizado WebView2 / WKWebView | **Alta** | Medio | QA en ambas plataformas por pantalla, desde el inicio |
| R-08 | No existen activos de marca (A-06) | **Alta** | Medio | Dirección debe entregarlos o encargarlos antes de cerrar Fase 2 |
| R-09 | Degradación de rendimiento a 500 k contactos | Media | Medio | Presupuestos medidos en CI con conjunto sintético |
| R-10 | Pérdida de la clave maestra de SQLCipher → datos irrecuperables | Baja | **Crítico** | Clave en llavero del SO; documentar respaldo y migración entre equipos; advertir en la interfaz |
| R-11 | Expansión de alcance desde las 175 secciones del brief | **Alta** | Alto | `FUERA_DE_ALCANCE.md` con destino explícito para cada elemento diferido |
| R-12 | El motor se deja para el final por ser menos vistoso | Media | **Crítico** | Se construye **antes** que la UI de campañas, con proveedor simulado |
| R-13 | Incumplimiento de la LFPDPPP por contactos sin consentimiento | Media | Alto | Afirmación de origen lícito por importación, registrada en bitácora; responsabilidad del cliente documentada |
| R-14 | Certificados de firma no listos al cerrar el desarrollo | Media | Medio | Trámite iniciado en Fase 1 pese a que D-4 lo difiere |

---

## S. Roadmap

Las 16 fases del §141 se consolidan en 9. Los cambios respecto al brief son deliberados y están justificados.

| Fase | Contenido | Dependencias y notas |
|---|---|---|
| **0. Discovery y auditoría** | Este cuerpo documental | ✅ En curso. Puerta: aprobación de Dirección |
| **1. Cimientos** | Workspace Rust, esqueleto Tauri 2, SQLCipher, migraciones, CI, tokens de diseño, i18n | **Arranca en paralelo:** verificación OAuth de Google y certificados de firma |
| **2. Design System** | Componentes, tablas, estados, accesibilidad AA | **Bloqueada por D-3** (licencia Mont) y por A-06 (activos de marca) |
| **3. Empresa y contactos** | Configuración, onboarding, contactos, listas, etiquetas, campos, importación, supresión | Primer valor tangible |
| **4. Motor de ejecución** | Cola, estados, idempotencia, throttling, suspensión, breaker, parada de emergencia | **Adelantada a propósito.** Con proveedor simulado. Es el 70 % del riesgo |
| **5. Proveedores de correo** | Trait `EmailProvider`, `SmtpProvider`, prueba de conexión, TLS obligatorio | `GoogleProvider` entra cuando Google verifique |
| **6. Campañas y mensajes** | Plantillas, versiones, variables, firmas, flujo de campaña, simulador, preflight, envío de prueba | Sobre un motor ya probado |
| **7. Actividad y entregabilidad** | Panel, actividad en vivo, métricas honestas, SPF/DKIM/DMARC | Sin tasas de rebote (R-06) |
| **8. Respaldos y endurecimiento** | Respaldo/restauración `.arles`, migración entre sistemas, las cuatro auditorías, rendimiento a 500 k | |
| **9. Release v1.2.0** | Instaladores, updater, CHANGELOG, despliegue interno TELEMETRY | D-4 |

**Los dos cambios estructurales respecto al brief:**

1. **El motor se adelanta** a la posición 4, antes que las campañas. Concentra el riesgo técnico y no tiene sentido construir su interfaz sobre cimientos no validados.
2. **Los trámites externos arrancan en la Fase 1**, no cuando se necesiten. La verificación de Google y los certificados de firma tienen plazos que no dependen de nosotros.

---

## T. Preguntas pendientes para Dirección

Detalle y seguimiento en `PREGUNTAS_ABIERTAS.md`.

**Bloqueantes de la Fase 2:**

1. **Licencia de Mont (D-3).** ¿Qué licencia exacta posee TELEMETRY —Desktop, Web o App— y puede subirse el comprobante al repositorio? Si no cubre app embedding, ¿se adquiere o se activa el plan B?
2. **Activos de marca (A-06).** No existe logotipo ni ningún archivo de marca. ¿Los entrega Dirección, se encargan, o se diseñan aquí?

**Necesarias para la Fase 3:**

3. **Origen y consentimiento de los contactos.** ¿Cuál es la procedencia de las listas que TELEMETRY va a importar? Determina el texto de la afirmación de origen lícito y la exposición ante la LFPDPPP.
4. **Buzón de rebotes (R-06).** ¿Acepta Dirección que la detección de rebotes sea parcial y esté declarada como tal en la interfaz durante v1.2.0?

**Necesarias para la Fase 5:**

5. **Dominio y aviso de privacidad.** La verificación de Google exige una política de privacidad pública en un dominio verificado. ¿Cuál se usa?
6. **Volumen real esperado.** T-7 fija 500 k contactos como techo arquitectónico. ¿Cuál es el volumen **operativo** realista por campaña? Cambia los presupuestos de rendimiento y la conversación sobre entregabilidad.

**Para v1.3, conviene decidirlo pronto:**

7. **Modelo comercial.** ¿Licencia perpetua, suscripción, por asientos, por instalación? Define el esquema de licenciamiento que se prepara ya.

---

## U. Recomendación final

**Construir ARLES RELAY I v1.2.0 tal y como está descrito aquí, con cuatro condiciones.**

El producto es sólido y su tesis es correcta: hay un hueco real entre «mando correos a mano» y «contrato Mailchimp», y ese hueco lo ocupa una herramienta de escritorio que respeta los límites del cliente en vez de ayudarle a saltárselos. Las decisiones más valiosas del brief son justamente las restrictivas —sin rotación de remitentes, sin métricas falsas, prueba obligatoria antes de activar— y son las que hay que defender cuando alguien pida lo contrario.

Las cuatro condiciones:

**Primera: resolver la licencia de Mont antes de la Fase 2.** Es el único riesgo crítico con probabilidad alta del proyecto. Un producto comercial que distribuye una fuente sin licencia de aplicación es un problema legal que escala con cada instalación, y no se arregla después. El §21 del propio brief ordena detenerse aquí, y nos detenemos.

**Segunda: construir el motor antes que su interfaz.** Idempotencia, reanudación tras suspensión y respeto de ventanas horarias son problemas de sistemas distribuidos con ropa de aplicación de escritorio. Son el 70 % del valor y el 70 % del riesgo. Si el motor no es correcto, las pantallas bonitas sólo hacen más visible el fallo.

**Tercera: aceptar que la paleta hay que construirla, no extraerla.** La referencia cromática aporta el cian, el oro y los cremas. No aporta el azul profundo que la dirección artística necesita —0.73 %— ni puede aportarlo. Construir las rampas profundas no es traicionar la referencia: es lo único que permite que el sistema tenga jerarquía y pase AA.

**Cuarta: declarar en la interfaz lo que el producto no sabe.** En v1.2.0 ARLES no detecta rebotes asíncronos y no sabe si un correo fue entregado. Un cliente que lo sabe puede compensarlo. Un cliente que cree estar protegido y no lo está descubre el problema cuando ya quemó su dominio. La honestidad que el §65 exige para las métricas vale igual para las capacidades.

**Sobre el alcance:** v1.2.0 como despliegue interno de TELEMETRY (D-4) es la decisión correcta. Permite validar el motor con datos reales y con consecuencias reales, difiere tres costes externos —firma EV, verificación de Google, soporte— y evita el contrasentido de una beta comercial sin forma de cobrar.

**Puerta de salida de la Fase 0.** No se escribe código de producción hasta que:

- [ ] Dirección apruebe este documento
- [ ] Se resuelva D-3 (licencia de Mont)
- [ ] Se respondan las preguntas 1 a 4 de la sección T

---

**Producto:** ARLES RELAY I
**Versión objetivo:** v1.2.0
**Desarrollado por:** TELEMETRY INSIGHT
