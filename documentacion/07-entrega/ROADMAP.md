# Roadmap

**Proyecto:** ARLES RELAY I · v1.2.0

> Las 16 fases del §141 se consolidan en **10**. Los dos cambios estructurales respecto al brief están justificados en §3.

---

> 🟢 **Actualizado el 17/09/2026: WhatsApp entra en la v1.2.0** (P-15). Este
> roadmap se escribió para un producto de un solo canal; esto es lo que cambia,
> ya decidido y en parte construido:
>
> | Fase | Qué le hace WhatsApp | Estado |
> |---|---|---|
> | **1** | Migraciones **V3** (el contacto deja de ser un correo; consentimiento y supresión por canal) y **V4** (etapas de campaña, número de WhatsApp, calidad de Meta) | ✅ hechas |
> | **2** | Nada | — |
> | **3** | Gana una entrega nueva: la pantalla de **CANALES** y el alta del número con la casilla de Coexistencia (L-11) | ⬜ |
> | **4** | El motor ejecuta **etapas**, no campañas, y cada etapa tiene su ritmo y su parada | ⬜ |
> | **5** | Además del SMTP, el adaptador de **WhatsApp Cloud API** | ⬜ |
> | **6** | El asistente ofrece una o dos etapas; en WhatsApp sólo plantillas aprobadas por Meta | ⬜ |
> | **7** | **No** gana una séptima sección: P-13 se resolvió en que no hay bandeja. ACTIVIDAD gana **dos pestañas** y la línea de tiempo de la calificación | ⬜ |
>
> El detalle está en
> [LOGISTICA_DE_CAMPANAS](../01-producto/LOGISTICA_DE_CAMPANAS.md), decisiones
> L-1 a L-11. **El avance de la versión, con este alcance ampliado, se calcula
> en [`avance.json`](avance.json).**

## Fases

### Fase 0 — Discovery y auditoría ✅ *en curso*

Este cuerpo documental.

**Puerta de salida:** ✅ **cruzada el 2026-09-11.** Dirección aprobó la auditoría, D-5 desbloqueó la tipografía y P-02 quedó cerrada.

---

### Fase 1 — Cimientos

Workspace de Cargo con los crates de `ARQUITECTURA.md` · esqueleto de Tauri 2 con capabilities denegadas por defecto · `arles-db` con SQLCipher y `refinery` · integración con el llavero · `Secret<T>` y capa de redacción en `tracing` · andamiaje de Vue 3 con TypeScript estricto · i18n · CI completo.

**Y en paralelo, lo que no depende de nosotros:**

| Trámite | Por qué ahora |
|---|---|
| **Verificación OAuth de Google** | 4–8 semanas (D-1, P-05) |
| **Certificado de firma de Windows** | 1–3 semanas |
| **Apple Developer ID** | Trámite de alta |

Arrancarlos aquí es la diferencia entre que estén listos cuando hagan falta y que congelen el release.

---

### Fase 2 — Design System ✅ cerrada

Once primitivas, los cuatro estados de pantalla, Mont incrustada y el catálogo
visual. **18/18 comprobaciones**, tres sondas de navegador y una revisión
adversaria de 10 hallazgos. Resumen en
[FASE-02-DESIGN-SYSTEM.md](../09-fases/FASE-02-DESIGN-SYSTEM.md).

✅ **Desbloqueada por D-5.** Se desarrolla con Mont; la puerta de la licencia se traslada a antes de la demo.

Tokens de color con verificación de contraste en CI · escala tipográfica sobre Mont · **logotipo tipográfico** (§21, ya producible) · primitivas: botón, entrada, selector, modal, tabla virtualizada, menú, insignia, aviso, pestañas · los cuatro estados por componente · accesibilidad AA verificada en Windows y macOS.

Los tokens de color ya están operativos en [`herramientas/design-tokens/`](../../herramientas/design-tokens/README.md), con `--verificar` listo para el paso 5 del CI.

---

### Fase 3 — Empresa y contactos

Configuración de empresa · lista de verificación de onboarding · contactos con listas, etiquetas y campos personalizados · tabla virtualizada a 500 k · filtros · importación XLSX/CSV completa con todas las defensas del modelo de amenazas · **lista de supresión** · derechos ARCO.

**Primer valor tangible.** Al final de esta fase alguien puede cargar sus contactos y trabajar con ellos.

#### Se parte en cinco entregas, cada una con su puerta

| Entrega | Qué incluye | Puerta |
|---|---|---|
| **3.1** ✅ | Configuración de empresa · lista de onboarding · barra lateral fija y plegable (P-11) | **Cerrada el 15/09/2026**, 19/19. De los cuatro pendientes visuales de la Fase 2 se cerraron tres —ventana nativa, escalado de Windows y lector de pantalla—; **WKWebView sigue abierto** (R-07) y se declaró riesgo aceptado con fecha límite antes de la Fase 6 |
| **3.2** | Contactos, listas, etiquetas, campos propios · filtros · la tabla medida con 500 k | Sonda de navegador con volumen real |
| **3.3** ✅ | Importación XLSX/CSV con todas las defensas · purificación · informe de rechazados | **Cerrada el 18/09/2026.** 13 ataques con archivos construidos a propósito (`crates/arles-import/tests/ataques.rs` y `bomba.rs`). Encontraron **tres fallos reales**: la bomba de descompresión se comía 1 586 MB antes de que el tope reaccionara, 200 000 filas vacías colgaban la lectura, y un nombre de archivo con `U+202E` se dibujaba al revés en pantalla. Los tres arreglados y con su prueba |
| **3.4** | Lista de supresión · derechos ARCO | Borrar un contacto, reimportarlo, y comprobar que **sigue sin escribírsele** |
| **3.5** | Cierre: documento de fase, informe para Dirección y PDF | `validar.py --fase 3` en verde, 0 omitidas |

**Decisiones que gobiernan esta fase:**
[ADR-0013](../03-arquitectura/adr/0013-origen-de-contactos-y-purificacion.md) —
origen de contactos, purificación corregida y envío canario.
[ADR-0014](../03-arquitectura/adr/0014-deteccion-de-rebotes-sin-verp.md) — sólo
su parte de esquema; el resto llega en las Fases 5 y 7.

> ~~🔴 **La 3.3 está bloqueada por P-09**~~ → **desbloqueada por D-7 el
> 17/09/2026.** La construcción no espera al abogado: el mecanismo se hace igual
> y el texto legal se enseña **marcado como provisional y visible como tal**
> (ADR-0013 §1). Lo que sigue esperando a P-09 es la **redacción definitiva** y
> el primer envío real a terceros, no el código. La 3.1 y la 3.2 nunca
> dependieron de ninguna pregunta abierta.

---

### Fase 4 — Motor de ejecución ⭐

**Adelantada a propósito. Concentra el 70 % del riesgo técnico.**

Cola persistente · máquina de estados · idempotencia con restricción única y CAS · política `presumed_sent` · cubo de tokens persistido · ventanas de ejecución en la zona de la empresa · **detección de suspensión sin ráfaga** · reintentos con backoff · disyuntor · pausar/reanudar/detener · parada de emergencia · independencia de la ventana.

**Se construye contra un proveedor simulado**, sin enviar un solo correo real.

**Puerta de salida:** todos los casos frontera de `ESTRATEGIA_QA.md` §3 pasan, en particular el de suspensión sin ráfaga y el de muerte del proceso sin duplicado.

---

### Fase 5 — Proveedores de correo

Trait `EmailProvider` · `SmtpProvider` con TLS obligatorio · prueba de conexión · clasificación de errores SMTP · captura de rechazos 5xx síncronos hacia la supresión · guía de onboarding de contraseña de aplicación de Gmail.

`GoogleProvider` entra **cuando Google verifique** (D-1), no antes.

---

### Fase 6 — Campañas y mensajes

Plantillas con versionado · motor de sustitución no Turing-completo · sanitizado con `ammonia` · firmas · editor de texto enriquecido simple · el asistente de 9 pasos · simulador · preflight · envío de prueba obligatorio · aviso de más de 50 diarios con registro de aceptación.

**Sobre un motor ya probado.**

---

### Fase 7 — Actividad y entregabilidad

Panel de INICIO · actividad en vivo · eventos y errores · métricas honestas (aceptado ≠ entregado) · comprobación de SPF, DKIM y DMARC.

**Sin tasas de rebote** — dependen de v1.3 (ADR-0009). La limitación se declara en la interfaz.

---

### Fase 8 — Respaldos y endurecimiento

Exportación e importación `.arles` · migración entre sistemas con manejo elegante del llavero (§75) · derechos ARCO · **las cuatro auditorías** (§140) · presupuestos de rendimiento a 500 k verificados.

---

### Fase 9 — Release v1.2.0

Instaladores firmados y notarizados · updater · `CHANGELOG.md` · despliegue interno en TELEMETRY (D-4).

⚠️ **Puerta de la licencia de Mont (D-5).** Antes de generar el primer instalador destinado a enseñarse o instalarse fuera del equipo de desarrollo, se vuelve a plantear **P-01**. Sin comprobante, se activa el plan B de `TIPOGRAFIA.md` §3 — una línea en `--arles-font-family`.

---

## Los dos cambios respecto al brief

### 1 · El motor se adelanta

El §141 lo sitúa en la fase 9, después de las campañas. **Aquí ocupa la fase 4, antes.**

Idempotencia, reanudación tras suspensión y respeto de ventanas horarias son problemas de sistemas distribuidos con ropa de aplicación de escritorio. Son el 70 % del valor y el 70 % del riesgo.

Construir primero la interfaz de campañas significaría diseñarla sobre supuestos no validados sobre cómo se comporta el motor — y descubrir en la fase 9 que un supuesto era falso obliga a rehacer la interfaz. Al revés, la interfaz se construye sobre comportamiento conocido.

**Y hay un riesgo de proceso que esto mitiga (R-12):** el motor no produce capturas de pantalla atractivas, así que la presión natural es dejarlo para después. Ponerlo en la fase 4 lo hace inevitable.

### 2 · Los trámites externos arrancan en la Fase 1

El §141 no los menciona. La verificación de Google (4–8 semanas) y los certificados de firma (1–3 semanas) tienen plazos que no dependen del equipo.

Arrancarlos al final es la forma más común de que un producto terminado no se pueda entregar.

---

## Dependencias

```
Fase 0 ──► Fase 1 ──┬──► Fase 2 (desbloqueada por D-5)
                    │         │
                    ├──► Fase 3 ◄─┘
                    │         │
                    ├──► Fase 4 ⭐ (independiente de la UI)
                    │         │
                    └──► Fase 5 ◄─┘
                              │
                          Fase 6 ──► Fase 7 ──► Fase 8 ──► Fase 9

Trámites externos (arrancan en Fase 1, terminan cuando terminan):
  Verificación Google ····························► GoogleProvider (v1.2.x)
  Firma Windows / Apple ··························► Fase 9
```

**Las fases 3 y 4 pueden avanzar en paralelo**: el motor no depende de la interfaz de contactos, y la interfaz de contactos no depende del motor.

---

## Después de v1.2.0

| Versión | Contenido |
|---|---|
| **v1.2.x** | `GoogleProvider` cuando Google verifique |
| **v1.3** | `MicrosoftProvider` · detección de rebotes por buzón dedicado + IMAP ([ADR-0014](../03-arquitectura/adr/0014-deteccion-de-rebotes-sin-verp.md)) · cifrado de respaldos · licenciamiento · **lanzamiento comercial** |
| **v1.4+** | White labeling · tracking opcional si se justifica |

Detalle y motivos en `01-producto/FUERA_DE_ALCANCE.md`.

---

## Lo que este roadmap no dice

**No hay fechas.** Ponerlas ahora sería inventarlas: dependen de la disponibilidad del equipo, de cuándo responda Dirección a P-01 a P-04, y de cuánto tarde Google.

Lo que sí está fijado es el **orden** y las **puertas de salida**. Las fechas se añaden cuando la Fase 1 dé una medida real del ritmo del equipo.
