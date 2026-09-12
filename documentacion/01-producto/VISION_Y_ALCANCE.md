# Visión y alcance

**Producto:** ARLES RELAY I
**Versión objetivo:** v1.2.0
**Desarrollado por:** TELEMETRY INSIGHT

---

## 1. Qué es ARLES RELAY

Una **aplicación de escritorio para Windows y macOS** que permite a una empresa ejecutar campañas de correo electrónico de forma controlada, auditable y honesta, usando **sus propias cuentas de correo** y **sus propios límites**.

Lo que define el producto no es la capacidad de enviar correo —eso lo hace cualquier cosa— sino **el control sobre el envío**:

- Límites que el cliente fija, no que le imponemos.
- Una simulación que dice cuánto va a tardar la campaña de verdad, antes de activarla.
- Un motor que respeta esos límites incluso cuando el equipo se suspende y despierta.
- Idempotencia que garantiza que nadie reciba dos veces el mismo correo.
- Métricas que no mienten.

## 2. El bucle central de v1.2.0

```
Configurar empresa
   → Conectar cuenta de correo
   → Importar contactos
   → Organizar audiencia
   → Crear campaña
   → Redactar y personalizar el mensaje
   → Fijar límites de ejecución
   → Simular
   → Enviar prueba
   → Validar (preflight)
   → Ejecutar
   → Pausar / Reanudar / Detener
   → Revisar resultados
   → Respaldar
```

Todo lo que no sirve a este bucle es candidato a quedar fuera.

## 3. Los tres principios

### Honestidad
Si el proveedor sólo confirmó «aceptado», la interfaz dice **aceptado**. Nunca «entregado». Y si el producto no sabe algo —como la detección de rebotes asíncronos en v1.2.0— lo declara en la interfaz en vez de dejar que el cliente asuma una protección que no tiene.

### Seguridad sobre comodidad
Ante la elección entre un flujo cómodo y uno seguro, gana el seguro, y se documenta el coste. Si el llavero del sistema no está disponible, la aplicación se niega a arrancar antes que guardar credenciales en claro.

### Autonomía informada
ARLES **advierte y explica; no dicta** la lógica de negocio del cliente. Si alguien quiere enviar 200 correos diarios desde una cuenta, ARLES le muestra el riesgo, registra que lo aceptó, y lo deja decidir.

El límite de este principio: ARLES nunca debe **facilitar la evasión**. Permitir que un cliente asuma un riesgo que entiende es autonomía. Rotar remitentes automáticamente para saltarse un límite es otra cosa, y no se hace (T-1).

## 4. Qué NO es ARLES

No es, y el proyecto rechazará activamente que se convierta en:

| No es | Por qué importa decirlo |
|---|---|
| Mailchimp | No hay constructor de newsletters de arrastrar y soltar. El editor es deliberadamente simple (§41) |
| Un CRM | No gestiona oportunidades, embudos ni relaciones comerciales |
| Un ERP | No toca inventario, facturación ni contabilidad |
| Una plataforma omnicanal | Sólo correo. Sin WhatsApp, sin SMS, sin notificaciones push |
| Software de llamadas en frío | No hay marcador ni telefonía |
| Una plataforma publicitaria | No compra ni gestiona medios |
| Una herramienta de scraping | **No recolecta contactos.** Sólo importa los que el cliente ya tiene |
| Un evasor de antispam | No ofusca contenido, no rota remitentes, no falsea cabeceras |
| Un sistema para saltarse los límites de Gmail | Al alcanzar un límite, la cola **pausa** y avisa |
| Un simulador de actividad humana | No introduce retardos aleatorios para «parecer humano» |

**Es:** software de ejecución y gestión controlada de campañas de correo electrónico.

## 5. Visión comercial

TELEMETRY INSIGHT usará ARLES internamente primero, pero el producto se construye para comercializarse.

Eso impone cinco restricciones desde la primera línea de código:

1. **No se codifica TELEMETRY como la única empresa operadora.** La entidad `company` es un dato, no una constante.
2. **No se codifican correos ni límites internos.** Todo límite es configuración.
3. **La base de datos no asume un único usuario global.**
4. **No se usan rutas absolutas atadas a una máquina.**
5. **Las funciones locales básicas no dependen de infraestructura privada de TELEMETRY.** Una instalación debe funcionar sin conexión a nada nuestro.

## 6. Modelo de distribución

ARLES es **software instalable**. Cada organización lo instala en una o varias máquinas según su licencia.

No es SaaS, y v1.2.0 no lo será. Pero la arquitectura **no debe cerrar la puerta** a una futura edición ARLES RELAY CLOUD. En la práctica esto significa: dominios bien aislados, lógica de negocio fuera de la interfaz, y ningún supuesto de «sólo hay una empresa en esta base de datos».

No se construye la versión cloud ahora. Sólo se evita bloquearla.

## 7. Alcance de v1.2.0

### Dentro

**Núcleo irreductible** — si falta algo de esto, no hay producto:

- Configuración de empresa y onboarding
- Importación XLSX/CSV con detección, mapeo, validación, previsualización y deduplicación
- Contactos, listas, etiquetas y campos personalizados
- **Lista de supresión** con autoridad absoluta sobre el envío
- `SmtpProvider` con TLS obligatorio y prueba de conexión
- Editor de mensaje simple: texto enriquecido, enlaces, variables `{{nombre}}`, firmas
- Plantillas con versionado
- Límites de ejecución configurables por cuenta (diario, horario, días y horas operativas)
- Simulador de campaña con duración estimada real
- Preflight con bloqueo y errores accionables
- Envío de prueba obligatorio antes de activar
- Motor de ejecución en segundo plano: cola persistente, idempotencia, throttling, reintentos con backoff, circuit breaker
- Pausar / Reanudar / Detener, con parada de emergencia
- Panel de inicio y actividad en vivo
- Comprobación de SPF, DKIM y DMARC
- Respaldo y restauración `.arles`
- Cifrado de la base de datos en reposo (SQLCipher)
- Bitácora de auditoría append-only

### Fuera

El detalle, con destino explícito para cada elemento, está en **`FUERA_DE_ALCANCE.md`**. Resumen:

| Elemento | Destino |
|---|---|
| `GoogleProvider` (OAuth) | v1.2.x — bloqueado por verificación de Google |
| `MicrosoftProvider` | v1.3 |
| Detección automática de rebotes (buzón dedicado por IMAP) | v1.3 · ver ADR-0014 |
| Cifrado de respaldos | v1.3 |
| Enforcement de licencias | v1.3 |
| Tracking de aperturas y clics | v1.3+ |
| White labeling | v1.4+ |

## 8. Público y contexto de uso

**Quién lo usa:** personal administrativo y comercial de una empresa pequeña o mediana. No son ingenieros. Saben usar Excel.

**Cómo lo usa:** sesiones largas frente a un monitor, con ratón y teclado. Tablas grandes. Filtrado. Trabajo administrativo sostenido.

**Qué implica para el diseño:** densidad alta de información, navegación completa por teclado, nada de patrones móviles (§4). De 1366×768 a 4K, con escalado de Windows del 100 % al 200 %.

## 9. Criterios de éxito de v1.2.0

La versión está terminada cuando:

1. Una persona sin formación técnica completa el bucle central de principio a fin sin ayuda.
2. Una campaña de 5 000 correos con límite de 50 diarios se ejecuta durante 100 días, sobrevive a suspensiones, reinicios y cierres de ventana, y **no envía un solo duplicado**.
3. Ningún dato personal ni credencial aparece en texto claro en disco, en registros ni en respaldos.
4. La interfaz nunca afirma «entregado» cuando sólo sabe «aceptado».
5. Todas las pantallas pasan WCAG 2.2 AA, verificado en Windows y macOS.
6. Los presupuestos de rendimiento se cumplen con 500 000 contactos.

## 10. Nomenclatura

- **Nombre oficial:** ARLES RELAY I
- **Nombre visual del producto:** ARLES RELAY
- **Versión:** v1.2.0, versionado semántico (`MAYOR.MENOR.PARCHE`)

El numeral «I» se retiene como identificador comercial (T-5), pero **no se muestra adyacente al número de versión**: «ARLES RELAY I / v1.2.0» sugiere que «I» es la versión 1. El «I» vive en la marca y el empaque; el número de versión aparece solo, en el pie y en «Acerca de». Ver ADR-0010.

**Prohibido** cualquier nombre informal en artefactos: FINAL, FINAL2, DEFINITIVO, NUEVO (§3).
