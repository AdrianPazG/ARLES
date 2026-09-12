# Matriz de riesgos

**Proyecto:** ARLES RELAY I · v1.2.0
**Última revisión:** 2026-09-11
**Cadencia de revisión:** al cierre de cada fase del roadmap

> Registro vivo. Cada riesgo tiene dueño y disparador. Un riesgo sin dueño no está gestionado.

**Escala de probabilidad:** Baja (< 25 %) · Media (25–60 %) · Alta (> 60 %)
**Escala de impacto:** Bajo (molestia) · Medio (retrasa o degrada) · Alto (compromete el release) · Crítico (daño irreversible a datos, reputación o legalidad)

---

## Riesgos críticos y altos

### R-01 · Licencia de Mont insuficiente para distribución
**Prob.** Alta · **Impacto** ~~Alto~~ **Medio** · **Dueño:** Dirección · **Estado:** 🟡 abierto, **ya no bloqueante**

> **Revisado el 2026-09-11 (D-5).** Se desarrolla con Mont; la puerta pasa de la Fase 2 a **antes de la demo**. El impacto baja de Alto a Medio porque el plan B cuesta una línea: toda la tipografía vive en `--arles-font-family`. Lo que se perdería si hay que activarlo es el ajuste fino tipográfico, no la arquitectura del Design System.
>
> **La línea que no se cruza:** el binario de la fuente no sale del equipo de desarrollo —ni instalado, ni enseñado como demo— hasta resolver D-3.

Las licencias Desktop y Web de Fontfabric no cubren incrustar el binario de la fuente en una aplicación distribuida; eso exige una *App License* separada. No hay comprobante en el repositorio y el kit presente fue generado con Transfonter, lo que sugiere un origen de agregador.

**Mitigación (D-3):** verificar el alcance exacto de la licencia que posee TELEMETRY y adquirir la App License si falta. Comprobante en `08-legal/`. Plan B documentado en `05-diseno/TIPOGRAFIA.md`: Mont sólo en marketing, logotipo como SVG con contornos, geométrica de licencia libre dentro de la aplicación.

**Disparador de escalada:** si al planificar la demo no hay comprobante, se activa el plan B sin más discusión.

---

### R-03 · Envío de correos duplicados
**Prob.** Media · **Impacto** **Crítico** · **Dueño:** Tech Lead · **Estado:** 🟡 mitigado por diseño

Un duplicado es irreversible: no se puede des-enviar, daña la relación con el destinatario y contribuye a la tasa de queja que quema la reputación del dominio.

**Mitigación:** `UNIQUE(campaign_id, contact_id)` hace el duplicado imposible a nivel de esquema. Transiciones por compare-and-swap. `idempotency_key` persistida antes de llamar al proveedor y usada como `Message-Id`. El caso ambiguo (la aplicación muere entre el `accept` del proveedor y el commit) se resuelve con `presumed_sent` y **sin reenvío automático**: se prefiere un no-envío a un duplicado. Ver ADR-0004.

**Verificación:** test de integración que mata el proceso en el punto exacto y comprueba que el reinicio no reenvía.

---

### R-10 · Pérdida de la clave maestra de SQLCipher
**Prob.** Baja · **Impacto** **Crítico** · **Dueño:** Security Engineer · **Estado:** 🟡 requiere documentación de usuario

Si se pierde la clave del llavero del sistema operativo —reinstalación del sistema, cambio de equipo, perfil de usuario corrupto— la base de datos cifrada es irrecuperable. No hay puerta trasera, y esa es precisamente la propiedad que se buscaba.

**Mitigación:** documentar el procedimiento de respaldo y migración entre equipos. La interfaz debe advertirlo en el primer arranque, no en un manual que nadie lee. El formato `.arles` debe permitir restaurar en una máquina nueva.

**Nota:** el cifrado de respaldos está diferido a v1.3 (T-6), lo que significa que en v1.2.0 el respaldo es el único camino de recuperación **y no está cifrado**. Hay que decirlo en la interfaz.

---

### R-12 · El motor se posterga por ser menos vistoso
**Prob.** Media · **Impacto** **Crítico** · **Dueño:** Tech Lead · **Estado:** 🟢 mitigado por el roadmap

Idempotencia, reanudación tras suspensión y respeto de ventanas horarias son problemas de sistemas distribuidos disfrazados de aplicación de escritorio. Concentran el 70 % del riesgo técnico y no producen capturas de pantalla atractivas.

**Mitigación:** el motor ocupa la Fase 4, **antes** que la interfaz de campañas, y se valida con un proveedor simulado.

---

### R-02 · La verificación OAuth de Google retrasa el release
**Prob.** Alta · **Impacto** Alto · **Dueño:** Dirección + DevOps · **Estado:** 🟢 mitigado por D-1

`gmail.send` es scope sensible: evita CASA Tier 2, pero exige verificación OAuth con política de privacidad pública, dominio verificado, vídeo demo y semanas de revisión.

**Mitigación (D-1):** v1.2.0 envía por SMTP y no depende de Google. La abstracción `EmailProvider` se diseña completa; `GoogleProvider` entra en v1.2.x. El trámite arranca en la Fase 1.

**Disparador:** requiere respuesta a la pregunta T-5 (qué dominio y aviso de privacidad se usan).

---

### R-05 · Un cliente quema su dominio usando ARLES correctamente
**Prob.** Media · **Impacto** Alto · **Dueño:** Product + UX · **Estado:** 🟡 mitigado parcialmente

El producto permite deliberadamente que el cliente fije sus propios límites (§46). Un cliente que configura 500 correos diarios desde un dominio nuevo, sin SPF ni DKIM, se destruye la reputación — y lo hace con el producto funcionando según lo diseñado.

**Mitigación:** aviso de más de 50 diarios con aceptación registrada en bitácora (§48) · simulador que muestra la duración real · comprobación SPF/DKIM/DMARC antes de activar · sin rotación automática de remitentes (T-1).

**Tensión que hay que sostener:** el §154 pide autonomía, informar sin dictar. El límite está en que ARLES nunca debe *facilitar* la evasión, sólo permitir que el cliente asuma un riesgo que entiende.

---

### R-06 · La supresión se degrada sin detección de rebotes
**Prob.** Alta · **Impacto** Medio · **Dueño:** Product · **Estado:** 🟡 aceptado con declaración

Contradicción del brief: §37 exige supresión automática por hard bounce, §69 prohíbe leer la bandeja de entrada. Sin buzón no hay rebotes asíncronos.

**Mitigación:** en v1.2.0 sólo rechazos 5xx síncronos de SMTP y supresión manual. Buzón de rebotes dedicado leído por IMAP en v1.3 — **por reenvío, no por VERP**, que resultó inviable en Gmail y Microsoft 365 (ADR-0009 corregido por [ADR-0014](../03-arquitectura/adr/0014-deteccion-de-rebotes-sin-verp.md)). **La interfaz debe declarar que la detección es parcial** — un cliente que se cree protegido y no lo está está peor que uno informado.

---

### R-07 · Divergencia de renderizado entre WebView2 y WKWebView
**Prob.** Alta · **Impacto** Medio · **Dueño:** Frontend · **Estado:** 🟢 mitigado por proceso

Es el coste aceptado de elegir Tauri (ADR-0001). WebView2 es Chromium; WKWebView es Safari. Divergen en flexbox de casos límite, en tipografía y en animación.

**Mitigación:** cada pantalla se valida en ambas plataformas antes de darse por terminada, no en un pase final.

---

### R-08 · No existen activos de marca
**Prob.** Alta · **Impacto** Bajo · **Dueño:** Diseño · **Estado:** 🟢 resuelto por D-5

No hay logotipo, isotipo, flecha ni escudo en el repositorio, pese a que T-9 los menciona.

**Resuelto.** El §21 define el logotipo como **exclusivamente tipográfico** —«ARLES RELAY» en Mont Black, sin isotipo—, así que con Mont autorizada para desarrollo (D-5) el logotipo se produce aquí, en la Fase 2. La flecha y el escudo **no son necesarios**: el §21 los excluye por diseño.

Queda como recomendación para v1.3 que Dirección entregue un brandbook propio, pero ya no bloquea nada.

---

### R-11 · Expansión de alcance desde las 175 secciones del brief
**Prob.** Alta · **Impacto** Alto · **Dueño:** Product · **Estado:** 🟢 mitigado por documento

**Mitigación:** `01-producto/FUERA_DE_ALCANCE.md` asigna destino explícito a cada elemento diferido. Nada queda en un limbo de «ya veremos».

---

### R-13 · Incumplimiento de la LFPDPPP por contactos sin consentimiento
**Prob.** Media · **Impacto** Alto · **Dueño:** Dirección + Privacy · **Estado:** 🟡 requiere respuesta de Dirección

ARLES no puede verificar que el cliente tenga consentimiento para los contactos que importa.

**Mitigación:** afirmación explícita de origen lícito en cada importación, registrada en la bitácora de auditoría. La responsabilidad como responsable del tratamiento es del cliente, y debe estar documentada. Derechos ARCO implementables desde el modelo de datos.

---

## Riesgos medios

### R-04 · Ráfaga de envíos al despertar el equipo
**Prob.** Media · **Impacto** Alto · **Dueño:** Tech Lead · **Estado:** 🟢 mitigado por diseño

Un token bucket ingenuo acumula permisos mientras el equipo duerme y los libera de golpe al despertar — exactamente lo que el §53 prohíbe.

**Mitigación:** detección de suspensión comparando reloj monótono (`Instant`) contra reloj de pared (`SystemTime`). Al detectarla, el bucket **se reinicia, no se acumula**. Un correo no enviado ayer no se recupera hoy.

**Verificación:** test que simula un salto del reloj de pared y comprueba que no hay ráfaga.

---

### R-09 · Degradación de rendimiento a 500 000 contactos
**Prob.** Media · **Impacto** Medio · **Dueño:** Performance · **Estado:** 🟢 mitigado por presupuesto

**Mitigación:** presupuestos en `06-calidad/PRESUPUESTO_RENDIMIENTO.md`, medidos en CI contra un conjunto sintético de 500 k. Virtualización de tablas, paginación por keyset (nunca `OFFSET`), importación por lotes, lectura XLSX en streaming.

---

### R-14 · Certificados de firma no listos al cerrar el desarrollo
**Prob.** Media · **Impacto** Medio · **Dueño:** DevOps · **Estado:** 🟡 abierto

Certificado OV/EV de Windows: de una a tres semanas. Apple Developer ID y notarización: trámite propio. Sin firma, SmartScreen y Gatekeeper bloquean la instalación.

**Mitigación:** iniciar el trámite en la Fase 1 aunque D-4 (despliegue interno) lo difiera. Son plazos externos que no se comprimen después.

---

## Riesgos de menor severidad, en seguimiento

| # | Riesgo | Prob. | Impacto | Dueño |
|---|---|---|---|---|
| R-15 | `/RECURSOS` duplicado provoca trabajo sobre la copia obsoleta | Media | Bajo | Tech Lead — ADR-0012 |
| R-16 | El numeral «I» junto a la versión confunde al usuario | Alta | Bajo | Product — ADR-0010 |
| R-17 | La referencia cromática es PNG con extensión `.jpg` y rompe herramientas | Baja | Bajo | Frontend — renombrar al consolidar |
| R-18 | Uso indebido del asset stock/IA como material de marca | Baja | Medio | Dirección — uso interno únicamente, registrado en `08-legal/` |
| **R-19** | **La documentación de privacidad cita una ley probablemente abrogada** (LFPDPPP 2010 → nueva ley de marzo de 2025) | **Alta** | **Alto** | Aviso de vigencia en `PRIVACIDAD_LFPDPPP.md`; **ningún texto legal visible al usuario se congela sin revisión jurídica** — P-09 |
| **R-20** | **Revocación de la verificación OAuth por parte de Google** si ARLES se asocia a correo no solicitado. La AUP de Workspace prohíbe expresamente facilitar correo masivo no solicitado, y la revocación afecta a **todos los clientes a la vez** | Media | **Crítico** | Cuentas y límites propios del cliente · sin rotación de remitentes (ADR-0008) · disyuntor por tasa de rebote · no posicionar Gmail para prospección en frío |

---

## Resumen por estado

| Estado | Riesgos |
|---|---|
| 🔴 **Abiertos y bloqueantes** | *ninguno* |
| 🟡 **Abiertos, mitigados parcialmente** | R-01 (licencia Mont, puerta en la demo), R-03, R-05, R-06, R-10, R-13, R-14 |
| 🟢 **Mitigados o resueltos** | R-02, R-04, R-07, R-08, R-09, R-11, R-12 |

**Revisión del 2026-09-11 (D-5):** R-01 pasa de bloqueante a seguimiento con la puerta en la demo, y R-08 se resuelve porque el logotipo es tipográfico por diseño (§21). **No quedan bloqueantes para arrancar las Fases 1 y 2.**
