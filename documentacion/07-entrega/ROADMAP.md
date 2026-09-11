# Roadmap

**Proyecto:** ARLES RELAY I · v1.2.0

> Las 16 fases del §141 se consolidan en **10**. Los dos cambios estructurales respecto al brief están justificados en §3.

---

## Fases

### Fase 0 — Discovery y auditoría ✅ *en curso*

Este cuerpo documental.

**Puerta de salida:** Dirección aprueba `AUDITORIA_DISCOVERY.md` · se resuelve D-3 · se responden P-01 a P-04.

**No se escribe código de producción hasta cruzarla.**

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

### Fase 2 — Design System

⚠️ **Bloqueada por D-3** (licencia de Mont) y por P-02 (activos de marca).

Tokens de color con verificación de contraste en CI · primitivas: botón, entrada, selector, modal, tabla virtualizada, menú, insignia, aviso, pestañas · los cuatro estados por componente · accesibilidad AA verificada en Windows y macOS.

**Lo que sí puede avanzar aunque D-3 siga abierta:** color, espaciado, elevación, movimiento y comportamiento. La tipografía es lo único bloqueado, y el plan B de `TIPOGRAFIA.md` se activa automáticamente si al inicio de la fase no hay comprobante.

---

### Fase 3 — Empresa y contactos

Configuración de empresa · lista de verificación de onboarding · contactos con listas, etiquetas y campos personalizados · tabla virtualizada a 500 k · filtros · importación XLSX/CSV completa con todas las defensas del modelo de amenazas · **lista de supresión**.

**Primer valor tangible.** Al final de esta fase alguien puede cargar sus contactos y trabajar con ellos.

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
Fase 0 ──► Fase 1 ──┬──► Fase 2 (bloqueada por D-3, P-02)
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
| **v1.3** | `MicrosoftProvider` · detección de rebotes VERP+IMAP · cifrado de respaldos · licenciamiento · **lanzamiento comercial** |
| **v1.4+** | White labeling · tracking opcional si se justifica |

Detalle y motivos en `01-producto/FUERA_DE_ALCANCE.md`.

---

## Lo que este roadmap no dice

**No hay fechas.** Ponerlas ahora sería inventarlas: dependen de la disponibilidad del equipo, de cuándo responda Dirección a P-01 a P-04, y de cuánto tarde Google.

Lo que sí está fijado es el **orden** y las **puertas de salida**. Las fechas se añaden cuando la Fase 1 dé una medida real del ritmo del equipo.
