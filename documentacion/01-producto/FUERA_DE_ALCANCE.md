# Fuera de alcance

**Proyecto:** ARLES RELAY I · v1.2.0

> Este documento existe para combatir un riesgo concreto (R-11): el brief maestro tiene 175 secciones y la tentación de «ya que estamos» es permanente. **Cada elemento diferido tiene aquí un destino explícito.** Nada queda en el limbo de «ya veremos».
>
> Regla de oro: si algo no sirve al bucle central descrito en `VISION_Y_ALCANCE.md`, es candidato a salir.

---

## 1. Nunca — el producto no será esto

Estos elementos no están diferidos: están **rechazados por definición del producto** (§6, §125). Si reaparecen en una conversación de alcance, la respuesta es no, independientemente de la versión.

| Elemento | Motivo |
|---|---|
| WhatsApp, SMS, notificaciones push | ARLES es correo. Añadir canales lo convierte en una plataforma omnicanal, que es explícitamente lo que no es |
| CRM (oportunidades, embudos, pipeline) | Otro producto |
| ERP (inventario, facturación, contabilidad) | Otro producto |
| Generación de contenido con IA | No aporta al bucle central y añade una dependencia externa, un coste variable y una superficie de privacidad nueva |
| Scraping o recolección de contactos | ARLES importa los contactos que el cliente ya tiene. Recolectarlos cambia el perfil legal del producto por completo |
| Aplicaciones móviles o tablet | §4. El producto está diseñado para ratón, teclado y monitor |
| Evasión de filtros antispam | Ofuscación de contenido, cabeceras falseadas, retardos aleatorios para «parecer humano» |
| Rotación automática de remitentes para saltar límites | T-1. Es la funcionalidad estrella de la competencia y es exactamente lo que quema dominios |
| Plataforma publicitaria, llamadas en frío | Otro producto |

---

## 2. v1.2.x — entra en cuanto se desbloquee

### `GoogleProvider` (Gmail API con OAuth 2.0)
**Bloqueado por:** verificación OAuth de Google · **Decisión:** D-1 · **Riesgo:** R-02

El scope `gmail.send` es sensible: evita CASA Tier 2 pero exige verificación OAuth con política de privacidad pública, dominio verificado, vídeo demostrativo y semanas de revisión. El trámite arranca en la Fase 1, en paralelo al desarrollo.

**La abstracción `EmailProvider` se diseña completa desde el día uno**, con `GoogleProvider` como segundo adaptador previsto. No es un añadido posterior: es un hueco ya dimensionado.

Requisitos cuando entre: Authorization Code + PKCE, autenticación en el navegador del sistema (nunca en una webview embebida), scopes mínimos, tokens en el llavero del sistema operativo.

**Desbloquea:** P-05.

---

## 3. v1.3 — diferido con arquitectura preparada

### `MicrosoftProvider`
**Motivo:** T-6

Mismo patrón que Google: OAuth 2.0 con PKCE contra Microsoft Identity Platform. La abstracción ya lo contempla; sólo falta el adaptador y su proceso de verificación.

### Detección automática de rebotes (VERP + IMAP)
**Motivo:** contradicción del brief entre §37 y §69 · **ADR:** 0009 · **Riesgo:** R-06 · **Pregunta:** P-04

§37 exige que los hard bounces alimenten la lista de supresión. §69 prohíbe pedir permisos de lectura de bandeja. Sin leer un buzón no hay rebotes asíncronos — llegan como correo, minutos u horas después del envío.

**En v1.2.0:** sólo rechazos **síncronos** 5xx de SMTP (atrapan buzones inexistentes que el servidor destino rechaza durante el diálogo SMTP) más supresión manual.

**En v1.3:** `Return-Path` único por destinatario (VERP) apuntando a un **buzón de rebotes dedicado** que el cliente configura, leído por IMAP. Sólo esa cuenta, nunca la bandeja personal del usuario — respeta el espíritu del §69 cumpliendo el §37. Incluye parseo de DSN según RFC 3464 y clasificación duro/blando.

**Condición innegociable para v1.2.0:** la interfaz **declara explícitamente** que la detección es parcial. Un cliente informado puede compensarlo; uno que se cree protegido descubre el problema cuando ya quemó su dominio.

### Cifrado de respaldos
**Motivo:** T-6 · **Riesgo:** R-10

La base de datos está cifrada en reposo (SQLCipher, T-3), pero el archivo `.arles` exportado **no lo está** en v1.2.0.

**Consecuencia que hay que declarar en la interfaz:** el respaldo es el único camino de recuperación si se pierde la clave del llavero, y es un archivo sin cifrar con datos personales. El usuario debe saberlo para custodiarlo en consecuencia.

### Enforcement de licencias
**Motivo:** T-6, D-4

El **esquema** se prepara en v1.2.0 para no migrar datos después: organización, edición, asientos, caducidad y capacidades (`FeatureEntitlement`). Lo que no se construye es la verificación que restringe el uso.

Diseño recomendado, ya documentado en la sección Q de la auditoría: licencia firmada criptográficamente y verificable **sin conexión**, con periodo de gracia. Al caducar, modo restringido —consultar, exportar y respaldar siguen funcionando; activar campañas nuevas no—. **Nunca se bloquea el acceso a los datos del cliente.**

**Pendiente:** P-07 (modelo comercial).

### Métricas de entregabilidad completas
**Motivo:** depende de la detección de rebotes

En v1.2.0 el centro de entregabilidad incluye la comprobación de **SPF, DKIM y DMARC** —es una consulta DNS, barata y de altísimo valor— pero **no** tasas de rebote ni de queja, que dependen de la detección diferida.

---

## 4. v1.3+ — sólo si se justifica

### Tracking de aperturas y clics
**Motivo:** §67 lo prohíbe implementar automáticamente

Si se añade alguna vez, tres condiciones: **opcional** (desactivado por defecto), **transparente** (el usuario sabe qué se rastrea), y **respetuoso con la privacidad** (sin identificadores persistentes entre campañas).

Advertencia de producto: el píxel de seguimiento degrada la entregabilidad en algunos filtros y es cada vez menos fiable —Apple Mail Privacy Protection precarga todas las imágenes desde 2021, lo que hace que la métrica de apertura sea esencialmente ruido para una fracción grande de los destinatarios. **Vender «tasa de apertura» como dato fiable sería incompatible con el principio de honestidad.**

### Permisos de lectura de bandeja de entrada
**Motivo:** §69

No se solicitan en v1.2.0 y no deben solicitarse sin una funcionalidad aprobada que los exija estrictamente. El buzón de rebotes dedicado de v1.3 **no es una excepción a esto**: es una cuenta separada que el cliente crea para ese fin, no su correo personal.

---

## 5. v1.4+ — preparar sin construir

### White labeling
**Motivo:** §117–§124

La arquitectura debe permitirlo: tokens de diseño centralizados, textos externalizados vía i18n, logotipo como activo reemplazable. **Pero no se construye la funcionalidad.**

La atribución «Software desarrollado por TELEMETRY INSIGHT» se mantiene discreta en el pie de la aplicación y **nunca se inyecta en los correos que el cliente envía** (§121). El correo es del cliente, no nuestro.

### ARLES RELAY CLOUD
**Motivo:** §8

No se construye. Sólo se evita bloquearlo: dominios bien aislados, lógica de negocio fuera de la interfaz, y ningún supuesto de «sólo hay una empresa en esta base de datos».

---

## 6. Decisiones de simplificación dentro de v1.2.0

Cosas que **sí** están en alcance pero deliberadamente acotadas.

| Elemento | Se construye | No se construye |
|---|---|---|
| **Editor de mensaje** (§41) | Texto enriquecido: negrita, cursiva, enlaces, listas, variables, firmas | Constructor de newsletters de arrastrar y soltar, plantillas de columnas, editor de HTML crudo |
| **Motor de plantillas** | Sustitución `{{clave}}` contra un mapa cerrado | Condicionales, bucles, helpers. **No Turing-completo**: un motor con lógica evaluando datos de un XLSX ajeno es una vía de ejecución (ADR-0006) |
| **Filtros de contactos** (§38) | Etiquetas, listas, origen, campos personalizados, combinación con Y/O | Constructor de consultas anidado arbitrariamente |
| **Entregabilidad** (§64) | SPF, DKIM, DMARC | Tasas de rebote y queja (dependen de v1.3) |
| **Programación** | Días y horas operativas, zona horaria de la empresa | Reglas de recurrencia complejas, secuencias de goteo, automatizaciones condicionales |
| **Importación** | XLSX, CSV | Conexión directa a Google Sheets, APIs de terceros, sincronización bidireccional |

---

## 7. Cómo se usa este documento

Cuando alguien proponga añadir algo a v1.2.0:

1. **¿Sirve al bucle central?** Si no, no entra.
2. **¿Está en la lista de «Nunca»?** Entonces no entra en ninguna versión.
3. **¿Está diferido aquí?** Entonces ya hay una decisión: respetarla o revocarla explícitamente en `DECISIONES_DE_DIRECCION.md`.
4. **¿Es nuevo?** Se evalúa con el marco del brief: *Problema → Riesgo → Impacto → Alternativa → Trade-off → Recomendación*, y se registra aquí con destino.
