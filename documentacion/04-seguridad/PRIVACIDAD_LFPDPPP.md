# Privacidad y cumplimiento — LFPDPPP

**Proyecto:** ARLES RELAY I · v1.2.0
**Marco:** Ley Federal de Protección de Datos Personales en Posesión de los Particulares (México)
**Alcance:** estricto a normativa mexicana (T-8). GDPR y normativa estadounidense diferidos.

> ⚠️ **Este documento no es asesoría legal.** Describe las decisiones de ingeniería tomadas con la LFPDPPP en mente y señala dónde se requiere validación jurídica de TELEMETRY INSIGHT.

---

## 🔴 AVISO DE VIGENCIA — añadido el 2026-09-12

**Este documento se escribió contra la LFPDPPP de 2010. Hay indicios sólidos de
que esa ley está abrogada.**

Lo que encontró la investigación de la Fase 3, en fuentes secundarias de
despachos y consultoras (EY México, Littler):

- Una **nueva LFPDPPP** se habría publicado el **20 de marzo de 2025**, en vigor
  desde el **21 de marzo de 2025**, abrogando la de 2010.
- El **INAI** habría sido extinguido por decreto constitucional de diciembre de
  2024, y sus facultades sobre datos en posesión de particulares habrían pasado
  a la **Secretaría Anticorrupción y Buen Gobierno**.
- El **Reglamento de 2011 seguiría vigente en lo que no contradiga** a la ley
  nueva.
- Las multas serían de **100 a 160 000 UMA** y de **200 a 320 000 UMA** según la
  infracción, duplicables en datos sensibles.

**No está verificado contra el texto primario.** El entorno donde se investigó
tiene bloqueado el acceso a `diputados.gob.mx` y al DOF, así que todo lo
anterior viene de fuentes secundarias. **Es exactamente por eso que esto es un
aviso y no una reescritura.**

### Qué significa para el proyecto

| | |
|---|---|
| **Las decisiones de ingeniería** de este documento | Siguen siendo válidas. Cifrado, ARCO por dirección, bitácora inmutable, afirmación de origen — ninguna depende del número de un artículo |
| **Toda cita de artículo concreto** | Sospechosa. La numeración cambió |
| **Cualquier texto que la aplicación MUESTRE al usuario** | **Bloqueado** hasta revisión jurídica. Un aviso de privacidad que cita una ley abrogada es peor que no citar ninguna |

**Puerta:** ningún texto legal visible para el usuario final —afirmación de
origen, aviso de privacidad, EULA— se congela sin que un abogado mexicano
confirme el marco vigente. Registrado como **P-09** y **R-19**.

---

## 1. Quién es quién

Esta distinción determina todo lo demás.

| Figura | Quién es | Responsabilidad |
|---|---|---|
| **Titular** | La persona cuyos datos están en la base | Ejercer derechos ARCO |
| **Responsable** | **El cliente que usa ARLES** | Legitimidad del tratamiento, aviso de privacidad, atender ARCO |
| **Encargado** | ARLES / TELEMETRY, si tratara datos por cuenta del cliente | Seguridad, confidencialidad |

**El cliente es el responsable, no nosotros.** ARLES es software instalable que corre en la máquina del cliente, con datos que no salen de ahí. TELEMETRY no accede a los contactos de sus clientes.

**Esto debe quedar explícito en los términos de licencia.** Un cliente que crea que ARLES «se encarga del cumplimiento» está equivocado, y esa equivocación es un riesgo para ambas partes.

---

## 2. La advertencia que hay que dar a Dirección

**ARLES no puede verificar que el cliente tenga consentimiento para los contactos que importa.** Ningún software puede.

Lo que ARLES sí puede y debe hacer:

1. **Exigir una afirmación explícita de origen lícito en cada importación**, con texto concreto en vez de una casilla genérica.
2. **Registrarla** en `import_batch.consent_affirmation` y en `audit_log`.
3. **Hacer trazable el origen** de cada contacto hasta el archivo del que vino.

Texto propuesto para la importación:

> **Origen de estos contactos**
> Declaro que cuento con el consentimiento de estas personas o con una base legítima para contactarlas con fines comerciales, conforme a la LFPDPPP.
> ☐ Confirmo
> Origen: [formulario propio · clientes existentes · evento o feria · otro: ___]

**Riesgo R-13.** Si el origen incluye bases adquiridas a terceros, la exposición legal es significativa y debe valorarse **antes** de la primera campaña, no después. Pendiente: P-03.

---

## 3. Derechos ARCO

La arquitectura debe permitir ejercerlos sin intervención de un desarrollador.

### Acceso
Localizar a un titular por correo y exportar todos sus datos: contacto, campos personalizados, listas, etiquetas, historial de envíos, origen de importación.

**Implementación:** búsqueda por `email_normalized` más un comando de exportación que reúne todo en un archivo legible. La exportación queda registrada en `audit_log`.

### Rectificación
Editar cualquier dato del titular. Ya está en la interfaz de contactos.

### Cancelación — la parte con tensión

Da derecho al **borrado físico real**, no a un `deleted_at`.

Y aquí hay una tensión genuina con el §91, que exige bitácora de auditoría — y la auditoría exige no borrar.

**Resolución:**

| Dato | Qué se hace |
|---|---|
| `contact`, `contact_field` | **Borrado físico** |
| Pertenencias a listas y etiquetas | **Borrado físico** |
| `message_attempt` | **Se conserva el intento y su resultado; se anonimiza la identidad.** Se mantiene el hecho «se envió un correo el día X con resultado Y», se elimina a quién |
| `audit_log` | Se registra que hubo una cancelación, con el correo **hasheado**, no en claro |
| `suppression_entry` | **Permanece** |

Así la auditoría conserva el hecho —«el 12 de marzo se canceló un titular»— sin conservar al titular.

**Por qué la supresión permanece:** puede parecer contradictorio conservar el correo de alguien que pidió la cancelación. No lo es: **la entrada de supresión es precisamente lo que protege al titular de futuros envíos.** Borrarla significaría que si su dirección reaparece en una importación futura, volvería a recibir correos. La supresión no es tratamiento comercial: es el registro de una negativa, y conservarla sirve al interés del titular.

Conviene que TELEMETRY valide esta interpretación jurídicamente.

### Oposición
Equivale a una entrada en la lista de supresión con `reason = unsubscribe`. Bloquea todo envío futuro.

---

## 4. Baja en los correos

Toda campaña comercial debe incluir un mecanismo de baja.

**En v1.2.0:** una variable de plantilla `{{baja}}` que inserta instrucciones de baja, y el preflight **advierte** si la plantilla no la incluye.

**Por qué advertencia y no bloqueo:** el §154 pide autonomía. Un correo transaccional legítimo puede no necesitar baja. Pero la advertencia es visible y explica el riesgo.

**Limitación honesta:** ARLES no aloja un servidor, así que no puede ofrecer baja de un clic (RFC 8058). La baja es «responde a este correo» o un enlace que el cliente gestione. **Hay que decirlo**, porque las directrices para remitentes masivos de Google y Yahoo (desde febrero de 2024) exigen baja de un clic por encima de cierto volumen. Un cliente que supere ese umbral necesita infraestructura que ARLES no proporciona.

> Esto es aplicación directa del principio de honestidad: el producto declara lo que no puede hacer.

---

## 5. Seguridad como obligación legal

El artículo 19 de la LFPDPPP exige medidas de seguridad administrativas, técnicas y físicas. Lo que ARLES aporta:

| Medida | Implementación |
|---|---|
| Cifrado en reposo | SQLCipher (T-3, ADR-0011) |
| Cifrado en tránsito | TLS obligatorio, sin opción de desactivar |
| Control de acceso | Llavero del SO; la base de datos no abre sin clave |
| Trazabilidad | `audit_log` append-only |
| Minimización | Sólo los campos que el cliente decide importar |
| Retención acotada | Rotación de registros; los correos en registros no se guardan indefinidamente |

**Hueco declarado:** el respaldo `.arles` **no está cifrado** en v1.2.0 (T-6) y contiene datos personales. Se advierte al usuario; el cifrado entra en v1.3.

---

## 6. Transferencias

**No hay.** Los datos no salen de la máquina del cliente.

ARLES no envía telemetría, no sincroniza con servidores de TELEMETRY y no usa servicios de terceros para procesar contactos. Las únicas conexiones salientes son: el servidor SMTP que el cliente configura, consultas DNS para SPF/DKIM/DMARC, y el endpoint del updater.

**Esto es una ventaja competitiva y conviene decirlo en el material comercial.**

---

## 7. Pendiente de validación jurídica

| # | Asunto | Quién |
|---|---|---|
| 1 | Reparto responsable/encargado en los términos de licencia | Legal |
| 2 | Texto de la afirmación de origen lícito | Legal |
| 3 | Interpretación de cancelación con conservación de supresión (§3) | Legal |
| 4 | Aviso de privacidad de TELEMETRY para la verificación de Google (P-05) | Legal + Dirección |
| 5 | Exposición si el origen incluye bases adquiridas (P-03) | Legal + Dirección |
| 6 | Plazo de conservación de registros con correos | Legal |

---

## 8. Fuera de alcance en v1.2.0

**GDPR.** Diferido por T-8. Nota para cuando llegue: la arquitectura ya cubre buena parte —derecho de acceso, borrado, portabilidad por la exportación— pero faltarían el registro de actividades de tratamiento, la base jurídica explícita por finalidad y la notificación de brechas en 72 horas.

**CAN-SPAM / normativa estadounidense.** Diferida por T-8.

**Certificaciones** (ISO 27001, SOC 2). Fuera de alcance del producto; serían de TELEMETRY como organización.
