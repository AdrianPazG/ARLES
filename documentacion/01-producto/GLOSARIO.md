# Glosario

**Proyecto:** ARLES RELAY I · v1.2.0

> Vocabulario compartido entre producto, ingeniería, diseño y la propia interfaz. **Los términos marcados con 🖥️ son los que el usuario ve**: si aquí dice «aceptado», la interfaz dice «aceptado», no «enviado».

---

## Dominio del producto

**Empresa** (`company`) 🖥️
La organización que opera ARLES. Una instalación sirve a una empresa, pero el modelo de datos nunca asume que sólo existe una.

**Cuenta remitente** (`email_account`) 🖥️
Una cuenta de correo conectada desde la que se envía. Tiene sus propios límites y su propio estado de salud. **Nunca almacena credenciales**: guarda una referencia al llavero del sistema operativo.

**Contacto** (`contact`) 🖥️
Un destinatario potencial. Se identifica de forma única por `email_normalized` dentro de una empresa.

**Correo normalizado** (`email_normalized`)
El correo recortado de espacios y en minúsculas. **No se eliminan los puntos ni se recorta el sufijo `+`**: eso es específico de Gmail, y aplicarlo de forma general fusionaría contactos que en otros dominios son personas distintas. Es la clave de deduplicación.

**Lista** (`contact_list`) 🖥️
Agrupación explícita de contactos. El usuario decide quién entra.

**Etiqueta** (`tag`) 🖥️
Marca transversal aplicable a contactos, usable como criterio de filtrado.

**Campo personalizado** (`contact_field`) 🖥️
Dato adicional definido por la empresa. Disponible como variable en las plantillas.

**Importación** (`import_batch`) 🖥️
Un archivo procesado: origen, mapeo de columnas, totales y errores. Queda registrada para poder rastrear de dónde salió cada contacto.

**Supresión** (`suppression_entry`) 🖥️
Registro que **bloquea todo envío futuro** a un correo. Tiene motivo y origen (rebote duro, baja voluntaria, exclusión manual).

Dos propiedades que definen su comportamiento:
- **Autoridad absoluta.** Se consulta en el momento del envío, no sólo al construir la audiencia. Una campaña de tres días puede acumular bajas mientras corre.
- **Inmune a las importaciones.** Una importación nueva **nunca** sobrescribe una supresión (§39). Si un contacto suprimido reaparece en un archivo, se importa el contacto y la supresión permanece.

---

## Campañas y mensajes

**Campaña** (`campaign`) 🖥️
Un envío planificado a una audiencia, con un mensaje, una cuenta remitente y unos límites.

**Audiencia** (`campaign_audience`) 🖥️
El conjunto de contactos de una campaña. Se **congela** al activar, para que la campaña sea reproducible y auditable.

**Plantilla** (`template`) y **versión de plantilla** (`template_version`) 🖥️
El mensaje reutilizable y cada una de sus versiones inmutables (§44). Una campaña apunta a una **versión** concreta: editar la plantilla después no altera lo que ya se envió.

**Variable** 🖥️
Marcador `{{clave}}` que se sustituye por un dato del contacto. La sustitución es **puramente textual contra un mapa cerrado**, sin lógica.

**Firma** (`signature`) 🖥️
Bloque reutilizable que se añade al final del mensaje.

---

## Ejecución — donde el vocabulario importa más

**Intento** (`message_attempt`)
**Un intento por contacto y por campaña.** Es la unidad atómica del motor y la tabla más importante del sistema. La restricción `UNIQUE(campaign_id, contact_id)` hace que el duplicado sea imposible a nivel de esquema, no una esperanza del código.

**Clave de idempotencia** (`idempotency_key`)
UUID generado y persistido **antes** de llamar al proveedor, que viaja como `Message-Id` del correo. Es lo que permite reconocer un envío tras un fallo.

### Estados de un intento

| Estado | Significado | 🖥️ Texto en interfaz |
|---|---|---|
| `queued` | En cola, esperando turno | En cola |
| `claimed` | Un worker lo tomó | En cola |
| `sending` | Se está enviando ahora | Enviando |
| `sent` | **El proveedor lo aceptó** | Aceptado |
| `failed` | Falló de forma transitoria; se reintentará | Reintentando |
| `permanently_failed` | Falló de forma definitiva | Falló |
| `suppressed` | Bloqueado por la lista de supresión | Suprimido |
| `cancelled` | Cancelado por parada | Cancelado |
| `presumed_sent` | **Estado ambiguo** — ver abajo | Envío no confirmado |

**`presumed_sent` — envío presunto** 🖥️
El estado que resulta cuando la aplicación muere **después** de que el proveedor aceptó el mensaje y **antes** de que pudiéramos registrar el commit. Al reiniciar, la fila que quedó en `sending` es genuinamente ambigua: el correo pudo salir o no, y no hay forma de saberlo sin leer la bandeja de enviados (permiso que v1.2.0 no tiene, §69).

**Nunca se reenvía automáticamente.** Se marca, se muestra, y el usuario decide. Preferimos un no-envío a un duplicado: el duplicado es irreversible y quema reputación. Ver ADR-0004.

---

## Honestidad de las métricas — la distinción central

**Aceptado** (`ACCEPTED_BY_PROVIDER`) 🖥️
El proveedor (servidor SMTP o Gmail API) recibió el mensaje y se hizo responsable de entregarlo. **Es lo único que ARLES sabe con certeza.**

**Entregado** (`DELIVERED`)
El mensaje llegó al buzón del destinatario. **ARLES no sabe esto en v1.2.0** y por lo tanto **nunca lo afirma** (§65). Requiere webhooks del proveedor o lectura de acuses, ninguno disponible en esta versión.

> Esta distinción no es pedantería: es el principio de honestidad del producto hecho vocabulario. Toda la competencia dice «entregado» cuando sólo sabe «aceptado». Nosotros no.

**Rebote duro** (*hard bounce*) 🖥️
Rechazo permanente: el buzón no existe. Debe alimentar la lista de supresión.

**Rebote blando** (*soft bounce*) 🖥️
Rechazo temporal: buzón lleno, servidor caído. No suprime; se reintenta.

> ⚠️ **En v1.2.0 sólo se detectan los rechazos síncronos 5xx del diálogo SMTP.** Los rebotes asíncronos —que son la mayoría— llegan como correo a la bandeja del remitente y ARLES no puede verlos sin permisos de lectura. La interfaz debe declararlo. Ver ADR-0009.

---

## Motor de ejecución

**Motor** (`arles-engine`)
El componente que ejecuta las campañas. Corre **independiente de la ventana principal** (§51): cerrar la ventana no detiene una campaña. La interfaz debe dejarlo claro.

**Cola**
El conjunto persistente de intentos pendientes. Sobrevive a cierres, reinicios y caídas.

**Cubo de tokens** (*token bucket*) (`rate_budget`)
El mecanismo que hace cumplir los límites. Persistido por cuenta remitente, con ventana horaria y diaria en la **zona horaria de la empresa** — no la del sistema operativo: un portátil que viaja no debe cambiar la política de envío.

**Ventana de ejecución** (`execution_window`) 🖥️
Los días y las horas en que la empresa permite enviar.

**Reanudación tras suspensión**
Cuando el equipo se suspende y despierta, el cubo **se reinicia, no se acumula**. Un cubo ingenuo acumularía permisos mientras duerme y los liberaría todos de golpe al despertar — exactamente la ráfaga que el §53 prohíbe. Un correo no enviado ayer **no se recupera hoy**: la campaña simplemente dura más.

**Disyuntor** (*circuit breaker*) 🖥️
Tras N fallos consecutivos en una cuenta, se abre el circuito y esa cuenta se pausa automáticamente, con aviso explícito.

**Pausar / Reanudar / Detener** 🖥️
Tres acciones **distintas** y que nunca deben confundirse en la interfaz (§62):
- **Pausar** — la cola se conserva intacta; se puede reanudar.
- **Reanudar** — continúa desde donde estaba, respetando los límites.
- **Detener** — cancela la cola. Requiere confirmación explícita. No tiene vuelta atrás.

**Parada de emergencia** 🖥️
Detiene todo envío nuevo **de inmediato**. Los workers comprueban la bandera **antes de cada envío**, no entre lotes.

**Simulador** 🖥️
Calcula y muestra la duración estimada real de la campaña **antes** de activarla, según la audiencia, los límites y la ventana de ejecución (§50).

**Preflight** 🖥️
Validación previa a la activación: tokens vigentes, remitente válido, audiencia no vacía, variables resueltas, programación coherente, conectividad (§63). Devuelve **LISTA** o bloquea con errores accionables.

**Envío de prueba** 🖥️
Envío obligatorio a una dirección propia antes de poder activar la campaña (§45).

---

## Seguridad y datos

**Llavero** (*keyring*)
Almacén de credenciales del sistema operativo: Keychain en macOS, Credential Manager en Windows. **Toda credencial vive aquí y en ningún otro sitio.**

**Clave maestra**
32 bytes aleatorios generados en el primer arranque que cifran la base de datos con SQLCipher. Custodiada en el llavero. **Si el llavero no está disponible, la aplicación se niega a arrancar** — no hay degradación a texto plano.

**`Secret<T>`**
Tipo de Rust cuyas implementaciones de `Debug` y `Display` emiten `[REDACTADO]`, para que un registro descuidado no pueda filtrar un token.

**Bitácora de auditoría** (`audit_log`)
Registro **append-only** de acciones críticas: activar o detener campañas, conectar o desconectar cuentas, importar, suprimir, restaurar respaldos, y la aceptación explícita del aviso de más de 50 correos diarios (§48).

**Respaldo `.arles`** 🖥️
Archivo de exportación de los datos. ⚠️ **No cifrado en v1.2.0** (T-6): es el único camino de recuperación si se pierde la clave del llavero, y contiene datos personales. Hay que decírselo al usuario.

**LFPDPPP**
Ley Federal de Protección de Datos Personales en Posesión de los Particulares (México). El marco de cumplimiento de v1.2.0 (T-8).

**Derechos ARCO** 🖥️
Acceso, Rectificación, Cancelación y Oposición. El modelo de datos debe permitir localizar, exportar, rectificar y eliminar todos los datos de un titular.

---

## Proveedores

**`EmailProvider`**
El trait que abstrae el envío. Operaciones estandarizadas: `authenticate`, `validate`, `send`, `capabilities`, `rate_limits` (§26).

**`SmtpProvider`**
Adaptador SMTP con TLS obligatorio. **El único en v1.2.0** (D-1).

**`GoogleProvider`**
Adaptador de Gmail API con OAuth 2.0 (Authorization Code + PKCE). Previsto para v1.2.x, bloqueado por la verificación de Google.

**Scope sensible / scope restringido**
Clasificación de Google. `gmail.send` es **sensible**: exige verificación OAuth pero evita la auditoría CASA Tier 2 que sí requerirían `gmail.modify` o `mail.google.com`.

**VERP** (*Variable Envelope Return Path*)
Técnica de `Return-Path` único por destinatario que permite identificar de qué envío proviene un rebote. Base de la detección automática de rebotes de v1.3.

---

## Diseño

**Token de diseño** 🖥️
Variable CSS centralizada (`--arles-background-deep`). **Ningún color se escribe a mano en un componente** (§17).

**Dark-first**
La interfaz se diseña primero en oscuro, construida por capas de azul sobre un fondo profundo — no negro absoluto (§18).

**WCAG 2.2 AA**
El objetivo de accesibilidad. La referencia artística **nunca** tiene prioridad sobre la legibilidad (§19).
