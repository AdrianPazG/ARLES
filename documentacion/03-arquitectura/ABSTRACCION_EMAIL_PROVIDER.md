# Abstracción `EmailProvider`

**Proyecto:** ARLES RELAY I · v1.2.0
**Crate:** `arles-email`

> El §26 exige que la lógica central no dependa de Gmail. Esto no es una precaución teórica: en v1.2.0 **Gmail no está disponible** (D-1), así que la abstracción es lo único que permite que el producto funcione hoy con SMTP y gane Gmail en v1.2.x sin reescribir el motor.

---

## 1. El trait

```rust
#[async_trait]
pub trait EmailProvider: Send + Sync {
    fn kind(&self) -> ProviderKind;

    /// Capacidades declaradas. El motor consulta esto en vez de
    /// preguntar "¿eres SMTP?" en algún sitio.
    fn capabilities(&self) -> Capabilities;

    /// Límites que el proveedor impone, distintos de los que el
    /// usuario configura. El motor respeta el más restrictivo.
    fn rate_limits(&self) -> ProviderRateLimits;

    /// Establece o refresca credenciales. Toca el llavero.
    async fn authenticate(&mut self) -> Result<(), ProviderError>;

    /// "Probar conexión" de la interfaz (§29). No envía nada.
    async fn validate(&self) -> Result<ValidationReport, ProviderError>;

    /// Envía. La única operación que produce efectos externos.
    async fn send(&self, message: &OutboundMessage)
        -> Result<SendReceipt, ProviderError>;
}
```

### `Capabilities`

```rust
pub struct Capabilities {
    pub html_body: bool,
    pub custom_message_id: bool,   // ¿podemos fijar Message-Id?
    pub custom_return_path: bool,  // VERP — sólo SMTP propio (ADR-0014)
    pub bounce_forwarding: bool,   // admite reenviar rebotes a un buzón externo
    pub max_recipients_per_message: u32,
    pub max_message_bytes: u64,
    pub reports_sync_rejection: bool,  // ¿hay 5xx en el diálogo?
}
```

`custom_return_path` y `reports_sync_rejection` existen ya aunque la detección de rebotes esté diferida: son las capacidades que v1.3 necesitará consultar, y declararlas ahora evita tener que cambiar el trait después.

> **Corregido por [ADR-0014](adr/0014-deteccion-de-rebotes-sin-verp.md).** Se
> verificó y **`custom_return_path` será `false` en Gmail y en Microsoft 365**:
> los dos reescriben o no exponen el `Return-Path`. Sólo es `true` con SMTP
> propio del cliente. Por eso se añade `bounce_forwarding`, que es el camino
> viable para la mayoría: el cliente reenvía los rebotes a un buzón externo y
> ARLES los correlaciona por `Message-Id` —el que ya generamos desde la clave
> de idempotencia (ADR-0004)—, sin pedir ningún scope de Google.
>
> Que la capacidad ya estuviera declarada es lo que hace que este cambio cueste
> una línea en vez de un rediseño.

### `SendReceipt` — el tipo que hace cumplir la honestidad

```rust
pub struct SendReceipt {
    /// SIEMPRE es "aceptado por el proveedor". Nunca "entregado".
    pub accepted_at: DateTime<Utc>,
    pub provider_message_id: Option<String>,
    pub raw_response: Option<String>,
}
```

**No existe ningún campo `delivered`.** No es un descuido: el tipo hace imposible que el motor registre una entrega que nadie confirmó. El §65 se cumple porque el sistema de tipos no ofrece otra opción.

### `ProviderError`

```rust
pub enum ProviderError {
    Transient { detail: String, retry_after: Option<Duration> },
    RateLimited { retry_after: Duration },
    PermanentRecipient { code: u16, detail: String },  // → supresión
    PermanentMessage { code: u16, detail: String },    // → no reintentar
    Authentication { detail: String },                 // → abrir circuito
    Configuration { detail: String },                  // → preflight
    Network { detail: String },
}
```

La clasificación es **responsabilidad del adaptador**, no del motor. El motor no sabe qué significa un `550` de SMTP ni un `403` de Gmail: pregunta al adaptador y actúa según la variante. Añadir un proveedor nuevo no toca el motor.

`PermanentRecipient` es la variante que alimenta la lista de supresión.

---

## 2. `SmtpProvider` — el único en v1.2.0

Basado en `lettre`.

### Configuración

```
host, port, security (StartTls | ImplicitTls), username,
credential_ref, from_address, from_display_name
```

### TLS es obligatorio (§29)

**No se ofrece la opción de desactivar TLS.** Ni siquiera como casilla avanzada con una advertencia.

El brief dice «no permitir desactivar TLS por defecto». Vamos más lejos: no se permite en absoluto. Una casilla de «desactivar TLS» es una casilla que alguien acaba marcando para hacer funcionar un servidor mal configurado, y a partir de ahí las credenciales de correo del cliente viajan en claro por su red corporativa, para siempre, sin que nadie lo recuerde.

Si un servidor no soporta TLS, el mensaje de error explica el problema en vez de ofrecer una salida insegura.

Certificados: validación estricta. Sin opción de aceptar certificados autofirmados.

#### Cómo está construido (entrega 5.2)

`arles_email::tls::Cifrado` tiene **dos modos y ninguno sin cifrar**: `Starttls` (puerto 587, el de Gmail) y `Directo` (puerto 465). No existe una tercera variante, así que la regla no depende de que nadie la olvide: no hay nada que elegir.

Se usa `Tls::Required` y **no `Opportunistic`**. La diferencia es el agujero clásico del protocolo: con «oportunista», un servidor que no anuncie `STARTTLS` —o alguien en medio que borre esa línea de la respuesta al `EHLO`— deja la conversación en claro y la contraseña sale legible, sin que nada avise.

Las raíces de confianza son las de `webpki-roots`, no las del sistema operativo. Con las del sistema, un equipo con el almacén de raíces tocado —una empresa que inspecciona el tráfico, un programa que instaló una raíz propia— validaría distinto que el de al lado, y ARLES no podría decir qué comprobó.

**Y ARLES lo comprueba por su cuenta.** Antes de mandar `AUTH`, el adaptador le pregunta al socket si está cifrado y se niega a continuar si no lo está. Configurar `lettre` correctamente es necesario y no basta: es confiar en que una biblioteca de terceros haga lo que promete justo donde equivocarse cuesta la contraseña de correo de la empresa. Se comprobó rompiéndolo: hace falta tumbar **las dos** defensas para que la contraseña aparezca sin cifrar.

#### «Probar conexión» (entrega 5.3)

`ProveedorSmtp::probar()` conecta, cifra y autentica, y **no manda ningún correo**: ni `MAIL FROM`, ni `RCPT TO`, ni `DATA`.

La tentación de añadir un `MAIL FROM` + `RSET` es real, porque comprobaría además que el servidor acepta esa dirección de remitente. No se hace por dos razones. Algunos servidores cuentan ese diálogo como un intento de envío a efectos de reputación, y quien configura una cuenta pulsa el botón muchas veces. Y la que cierra la discusión: un `RCPT TO` de prueba apuntaría a una persona real, y la respuesta a ese comando es, en todo ARLES, el único camino que da de baja a alguien. Una prueba mira el servidor de mentira desde fuera y exige que esos tres comandos no aparezcan.

**Recorre el mismo camino que un envío.** La conexión, el cifrado y la autenticación están extraídos a una función que usan los dos. Una prueba de conexión con código propio podría decir «correcto» mientras los envíos fallan —o al revés—, y en cuanto eso pasa una vez, deja de servir para diagnosticar: manda a buscar el problema al lado equivocado.

**Lo que devuelve no es un `Fallo`.** Un `Fallo` dice «este envío no salió», y con ese vocabulario una cuenta mal configurada sólo se puede describir como «pasajero»: el motor reintentaría a los 5, 15 y 30 minutos y el disyuntor pararía la cuenta diciendo «cinco fallos seguidos». Cierto e inútil. Los seis diagnósticos corresponden cada uno a una acción distinta:

| Diagnóstico | Qué hacer |
|---|---|
| Correcto | Guardar la cuenta |
| No se llegó al servidor | Revisar nombre y puerto, o la red de la oficina |
| El servidor tardó demasiado | Esperar — el nombre y el puerto están bien, porque sí se llegó |
| No se pudo cifrar | Revisar nombre y puerto. **No hay forma de desactivarlo** |
| La contraseña no sirve | Generar una contraseña de aplicación |
| El servidor no está disponible | Esperar. **No** es la contraseña |

Las dos últimas se separan a propósito: un `4xx` autenticando no es una contraseña mala, y confundirlos hace que alguien tire la contraseña que funcionaba para arreglar algo que se arregla solo.

**Un detalle que sólo se supo probándolo:** `lettre` tiene un `Error::is_tls()` y la primera versión lo usaba para distinguir «no llegué» de «el certificado no cuadra». No funciona — ante un certificado inválido, `lettre` devuelve un error de red con el de `rustls` dentro, y `is_tls()` contesta `false` en los dos modos de cifrado. La rama no se ejecutaba nunca, y salió porque la prueba del certificado ajeno falló. Lo que sí distingue es el error de E/S subyacente: un handshake fallido da `InvalidData`; no llegar da `ConnectionRefused`. Clasificación por tipo, no por el texto del mensaje.

#### Cómo se prueba sin abrir un agujero

Ésta es la pregunta que hunde a la mayoría de los proyectos que se ponen esta regla, porque la respuesta fácil es exactamente la casilla prohibida —`dangerous_accept_invalid_certs`, «sólo para los tests»—, y una vez escrita, alguien la mueve.

Aquí se hace lo otro: el servidor de mentira genera un certificado de verdad para `localhost` y al adaptador se le da **ese certificado concreto como raíz de confianza de más**. La verificación sigue entera —firma y nombre—; lo único que cambia es a quién se cree. Una prueba comprueba que un certificado que no es el suyo se rechaza.

El validador de fase (`--fase 5`) comprueba sobre el árbol entero de Rust, **pruebas incluidas**, que ninguna de las llamadas que desactivan la verificación esté escrita, y que confiar en un certificado concreto no se llame desde el código de producción.

El servidor cifrado vive detrás de `#[cfg(test)]`: `rustls` y `rcgen` no entran en el programa que se entrega.

### Capacidades

```rust
Capabilities {
    html_body: true,
    custom_message_id: true,
    // OJO: true sólo con un servidor SMTP que admita un MAIL FROM arbitrario.
    // El SMTP de Gmail y el de Microsoft 365 lo reescriben, así que el
    // adaptador lo determina al conectar, no lo declara a ciegas (ADR-0014).
    custom_return_path: true,
    bounce_forwarding: true,        // el cliente puede reenviar a un buzón
    max_recipients_per_message: 1,  // ARLES siempre envía 1 a 1
    max_message_bytes: 25 * 1024 * 1024,
    reports_sync_rejection: true,   // el diálogo SMTP da 5xx inmediato
}
```

**Siempre un destinatario por mensaje.** Nunca CC ni BCC masivo: cada contacto recibe un mensaje propio, personalizado, y con su propio intento rastreable. Es más lento y es lo correcto.

### Rechazos síncronos

El diálogo SMTP da códigos en el momento. Es la **única** detección de rebotes de v1.2.0 (ADR-0009):

| Código | Significado | Acción |
|---|---|---|
| `550`, `553` | Buzón inexistente | `PermanentRecipient` → supresión |
| `551`, `554` | Rechazo permanente | `PermanentRecipient` → supresión |
| `450`, `451`, `452` | Temporal | `Transient` → reintento |
| `421` | Servicio no disponible | `Transient` |
| `535` | Autenticación fallida | `Authentication` → abrir circuito |

Atrapa buzones inexistentes que el servidor destino rechaza durante el diálogo. **No atrapa los rebotes asíncronos**, que son la mayoría y llegan como correo a la bandeja del remitente horas después.

### Cómo se construyó la traducción (entrega 5.1, 23/09/2026)

Vive en `arles_email::codigos`, y la tabla de arriba resultó ser la mitad de la historia. Al construirla aparecieron **cuatro filas que el resumen no tenía** y que, mal puestas, dan de baja a gente alcanzable.

**Tres niveles de certeza, en orden.** Un servidor contesta con hasta tres cosas:

```text
550 5.1.1 The email account that you tried to reach does not exist
^^^ ^^^^^ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 |    |    texto libre: NO se interpreta. Sólo se enseña.
 |    código extendido (RFC 3463): lo más preciso que hay
 código básico (RFC 5321)
```

1. **El extendido**, si viene. `5.1.1` dice «la dirección está mal» y `5.3.4` dice «tu mensaje es demasiado grande» — los dos son `5xx`, y confundirlos suprimiría a alguien por culpa de un archivo adjunto.
2. **El básico**, si no hay extendido.
3. **La clase**, para lo desconocido: `4xx` temporal, `5xx` permanente. **Nunca al revés**: un `5xx` raro tratado como temporal reintentaría para siempre, y un `4xx` raro tratado como permanente suprimiría a quien no debía.

**Las cuatro filas que no son obvias:**

| Caso | Lo que parece | Lo que es |
|---|---|---|
| `552` buzón lleno | Permanente, por el número | **Temporal.** El martes que viene tendrá sitio |
| `5.3.4` mensaje demasiado grande | Rechazo del destinatario | **Culpa nuestra.** Esa dirección funciona |
| `5.7.x` relay denegado, IP bloqueada | Rechazo del destinatario | **De la CUENTA.** Lo tendrá con todos los envíos |
| `5.1.8` dirección mal | Del destinatario, como el resto de `5.1.x` | **Del remitente.** Es el único de su familia |

Suprimir a alguien porque su buzón estaba lleno un día, o porque nuestro adjunto pesaba de más, es un error irreversible por una causa que no es suya.

**El texto libre no decide nada.** Buscar palabras —«full», «unknown», «try again»— funciona con el servidor con el que se probó y falla con todos los demás, en idiomas que nadie leyó. Hay una prueba que manda el mismo código con textos opuestos y exige el mismo resultado.

**Y de SMTP nunca sale una espera concreta.** El protocolo no tiene un campo legible por máquina que diga «vuelve en N segundos»; eso es del mundo HTTP y lo usará el adaptador de WhatsApp. Sacar el número de un texto libre es tentador y peligroso: en `550 5.1.1 user 3600 not found` hay un `3600` que no es una espera. Por eso `Fallo::NoMeMandesNadaDurante` **no lo produce nunca** este adaptador.

### El servidor de mentira

`arles_email::servidor` levanta un servidor de correo **en el propio proceso**, que habla el protocolo de verdad —saluda, entiende `EHLO`, `AUTH`, `MAIL FROM`, `RCPT TO`, `DATA` y el punto solo que cierra el cuerpo— y al que se le puede ordenar qué contestar en cada paso.

Existe porque **a Gmail no se le puede pedir que conteste `550`**. Sin él, todas las ramas de error del adaptador —la mitad del código y la mitad de las decisiones difíciles— se probarían el día del lanzamiento, con los contactos de un cliente.

Es la misma idea que el proveedor simulado de la Fase 4, un piso más abajo: allí se simulaba **el proveedor** y aquí **el servidor**. El adaptador que hablará con Gmail es exactamente el mismo que habla con éste.

**Guarda lo que le dijeron, y no guarda el cuerpo.** Lo primero permite afirmar lo que **no** se pidió —que a una persona dada de baja no se le mandó ni un `RCPT TO`—; lo segundo evita que el contenido de los correos acabe en la salida de las pruebas y de ahí en una bitácora de CI.

**La conversación no sabe por dónde viaja.** Trabaja sobre cualquier cosa legible y escribible, no sobre un socket. En la 5.2 se cobró: el servidor cifrado envuelve el flujo en TLS y llama a la misma función, sin tocarla.

**Y sabe qué se dijo antes de cifrar.** `dicho_en_claro()` devuelve sólo lo que el servidor recibió sin cifrado, que es lo que un vecino de red habría podido leer. Es lo que permite afirmar que la contraseña no viajó en claro mirándolo desde fuera, en vez de creerse la configuración propia.

#### Por qué la conversación se lleva paso a paso

`lettre` trae una función que hace el envío entero de una vez. Es cómoda y **no se usa**: cuando falla dice «el servidor contestó 550» y no dice en qué momento.

Eso no es un detalle. Un `550` contestando a `RCPT TO` significa «esa dirección no existe» → esa persona entra en la lista de bajas, para siempre y en todas las campañas. El mismo `550` contestando a `MAIL FROM` significa «tu cuenta no puede enviar desde esa dirección» → el problema es de la cuenta y no dice nada de quien iba a recibir el correo. Sin distinguirlos, ARLES daría de baja a un contacto sano por un problema del remitente, y una baja no se deshace.

Así que `MAIL FROM`, `RCPT TO` y `DATA` se mandan uno a uno y cada respuesta se atribuye a quien le corresponde. **`RCPT TO` y el acuse final son los únicos dos puntos que pueden acabar en una baja.**

Al probar esto salió un fallo de la propia prueba, que es el ejemplo de manual: el guion usaba `550 5.7.1 Sender address rejected`, y ese código extendido ya se traduce solo a «problema de la cuenta» — o sea que la prueba pasaba con la atribución rota. Ahora usa un `550` pelado, que es lo que contestan de verdad los relés mal configurados.

### Gmail por SMTP

Gmail admite SMTP con **contraseña de aplicación**, sin verificación OAuth alguna. Esto es lo que hace viable D-1: el caso de uso principal está cubierto desde el día uno.

```
host: smtp.gmail.com   puerto: 587   seguridad: StartTls
```

Requiere verificación en dos pasos activada en la cuenta de Google. La interfaz debe guiarlo con enlaces, porque es el punto donde más usuarios se atascan.

Límites de Gmail: ~500 destinatarios diarios en cuentas personales, ~2 000 en Workspace. Se declaran en `rate_limits()` y el motor respeta el más restrictivo entre éste y el del usuario.

---

## 3. `GoogleProvider` — v1.2.x

**Bloqueado por la verificación OAuth de Google** (D-1, R-02). El hueco está dimensionado; falta el adaptador y el trámite.

### OAuth 2.0 (§27)

- **Authorization Code + PKCE.** Obligatorio para aplicaciones instaladas (RFC 8252).
- **En el navegador del sistema**, nunca en una webview embebida. Una webview embebida puede leer las credenciales que el usuario teclea, y por eso Google la rechaza.
- **Redirección a `localhost`** en un puerto efímero.
- **Scope mínimo: `gmail.send` y nada más.** Es scope *sensible*: exige verificación OAuth pero evita la auditoría CASA Tier 2 que sí requerirían `gmail.modify` o `mail.google.com`.
- **Sin permisos de lectura de bandeja** (§69).
- Tokens en el llavero del sistema operativo. El *refresh token* nunca toca la base de datos.

### El client secret no es secreto

Google emite un «client secret» para aplicaciones instaladas, pero en un binario distribuido **es extraíble**. Es un cliente público en el sentido de RFC 8252, y PKCE es lo que aporta la seguridad real. No debe tratarse ni documentarse como un secreto de verdad: pretender lo contrario lleva a decisiones de seguridad basadas en una garantía que no existe.

### Requisitos de la verificación (P-05)

Política de privacidad pública en un dominio verificado en Search Console · página de inicio del producto en ese dominio · vídeo demostrativo del flujo de consentimiento · semanas de revisión.

**El trámite arranca en la Fase 1**, no cuando se necesite el adaptador.

---

## 4. `MicrosoftProvider` — v1.3

Diferido por T-6. Mismo patrón: OAuth 2.0 con PKCE contra Microsoft Identity Platform, scope `Mail.Send`, envío vía Graph API.

---

## 5. Selección y registro

```rust
pub fn build_provider(
    account: &EmailAccount,
    keyring: &dyn CredentialStore,
) -> Result<Box<dyn EmailProvider>, ProviderError>
```

El motor **nunca** conoce el tipo concreto. Recibe un `Box<dyn EmailProvider>` y trabaja contra el trait.

**La prueba de que la abstracción es real:** el proveedor simulado de los tests implementa el mismo trait y el motor no nota la diferencia. Si el motor necesitara alguna rama `if provider_kind == …`, la abstracción estaría rota.

---

## 6. Composición del mensaje

`OutboundMessage` lo construye `arles-template`, no el proveedor:

```rust
pub struct OutboundMessage {
    pub to: EmailAddress,
    pub from: EmailAddress,
    pub from_display_name: String,
    pub reply_to: Option<EmailAddress>,
    pub subject: String,
    pub body_html: String,   // ya sanitizado en Rust
    pub body_text: String,   // alternativa en texto plano
    pub message_id: String,  // = idempotency_key
    pub headers: Vec<(String, String)>,
}
```

**El HTML llega ya sanitizado.** El proveedor no sanea: su trabajo es transportar. Colocar el saneamiento en el proveedor significaría repetirlo en cada adaptador y olvidarlo en alguno.

**Siempre se envía alternativa en texto plano** (`multipart/alternative`). Mejora la entregabilidad y respeta a quien lee en texto.

**Nunca se inyecta atribución de TELEMETRY en los correos del cliente** (§121). El correo es del cliente.
