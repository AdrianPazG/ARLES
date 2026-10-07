# Cómo conectar ARLES a Gmail y a Meta, para una demostración real

> **Para quién es esto.** Para quien vaya a enseñar ARLES mandando un correo y
> un WhatsApp de verdad, no contra un servidor simulado. Escrito el 29/09/2026.
>
> **Actualizado el 29/09/2026, el mismo día:** cuando se escribió esta guía,
> ARLES mandaba correo real y **no** mandaba WhatsApp. El hueco se cerró en la
> entrega W.8 unas horas después. **Hoy los dos canales mandan de verdad.**

---

## 0 · Qué se puede demostrar hoy, y qué no

Esto sale de mirar el código el 29/09/2026, no de lo que el ROADMAP prometía.

| | Correo | WhatsApp |
|---|---|---|
| El adaptador que habla con el proveedor | ✅ `arles-email` | ✅ `arles-whatsapp` |
| Probado contra un servidor de mentira | ✅ | ✅ |
| Se puede armar el proveedor desde la app | ✅ | ✅ (W.8) |
| **Hay un comando para mandar una prueba** | ✅ `enviar_correo_de_prueba` | ✅ `enviar_whatsapp_de_prueba` (W.8) |
| Pantalla para dar de alta la cuenta | ✅ Remitentes | ✅ Remitentes › WhatsApp |
| **¿Se puede mandar un mensaje real desde la app?** | ✅ **sí** | ✅ **sí** |
| Una campaña entera por WhatsApp | ✅ | ❌ el motor todavía solo recorre correo |

**Qué significa.** Se puede mandar **un mensaje de prueba** por los dos canales,
que es lo que hace falta para una demostración. Lo que todavía no se puede es
lanzar una **campaña entera** por WhatsApp: el motor —el que reparte los envíos
con su ritmo y su horario— sigue recorriendo solo el canal de correo.

Para la demostración no estorba: el envío de prueba llega al teléfono igual.

---

## 1 · Correo: una cuenta de Gmail desechable

**Esto sí funciona hoy, de punta a punta.**

### 1.1 · Por qué una cuenta desechable y no la de la empresa

Porque la vas a configurar en un programa que todavía no ha pasado por revisión
jurídica, y porque si algo sale mal quieres poder tirar la cuenta sin perder
nada. Crea una cuenta de Gmail nueva, solo para esto.

### 1.2 · La contraseña de aplicación

Google **no deja** que un programa entre con la contraseña normal de la cuenta.
Hay que generar una aparte:

1. **Activa la verificación en dos pasos** en la cuenta de Google. Sin esto, la
   opción de contraseñas de aplicación **no aparece** — es el punto donde más
   gente se atasca.
2. Entra en la configuración de seguridad de Google y busca **«Contraseñas de
   aplicaciones»**.
3. Crea una nueva. Google da **dieciséis letras**. Esa es la que se escribe en
   ARLES, no la de la cuenta.

Si algún día quieres que ARLES deje de tener acceso, basta con borrar esa
contraseña de aplicación desde la cuenta de Google. No hace falta cambiar nada
más.

### 1.3 · Lo que se escribe en ARLES

En **Remitentes › Añadir cuenta**:

| Campo | Valor |
|---|---|
| Servidor | `smtp.gmail.com` |
| Puerto | `587` |
| Cifrado | STARTTLS |
| Usuario | la dirección completa |
| Contraseña | **las dieciséis letras**, no la de la cuenta |

ARLES rellena servidor, puerto y cifrado solos al abrir el formulario, porque
son siempre los mismos y equivocarse en el puerto es el error más común.

**No hay ninguna forma de no cifrar** (§29). No es una casilla que esté marcada
por omisión: no existe la casilla.

### 1.4 · Cómo se demuestra

1. **Remitentes › Probar conexión.** Responde en dos segundos y dice qué falla
   si falla — no un «error de conexión» genérico.
2. **El asistente de campaña, paso 8.** Manda un correo de prueba a la dirección
   que le digas, y **no deja avanzar hasta que el servidor lo acepta** (§45).
   Una prueba rechazada demuestra lo contrario de lo que se quería demostrar.

**Lo que ARLES dirá es «aceptado», nunca «entregado»** (§65). No puede saber si
llegó a la bandeja: sabe que el servidor se hizo cargo. Vale la pena señalarlo
en la demo, porque es una decisión de producto, no una limitación que se esconde.

---

## 2 · WhatsApp: una aplicación en Meta

**Esto ya se puede mandar desde ARLES** desde la entrega W.8.

### 2.1 · Lo que hay que crear en Meta

1. Entra en **Meta for Developers** y crea una aplicación de tipo **Negocios**.
2. Añádele el producto **WhatsApp**.
3. Meta crea automáticamente un **número de prueba gratuito** y una cuenta de
   empresa (WABA). No hace falta dar de alta un número propio para probar.

De ahí salen los tres datos que ARLES pide:

| Lo que ARLES pide | Cómo lo llama Meta | Dónde está |
|---|---|---|
| Identificador de la cuenta de empresa | **WhatsApp Business Account ID** (WABA ID) | En el panel de WhatsApp de la app |
| Identificador del número | **Phone Number ID** | Junto al número de prueba |
| Token de acceso | **Access token** | Ver §2.2 |

**Ninguno de los dos identificadores es un secreto** — Meta los enseña en su
propio panel. El token sí, y ARLES lo guarda en el llavero del sistema, nunca en
la base de datos.

### 2.2 · El token: hay dos, y el fácil caduca

- **Token temporal.** El que Meta ofrece de entrada. **Caduca en menos de 24
  horas.** Sirve para probar el mismo día; si la demo es mañana, mañana ya no
  vale.
- **Token permanente** (usuario de sistema). El que hay que usar de verdad:
  1. En la configuración del negocio, crea un **usuario de sistema**.
  2. **Generar nuevo token**, eligiendo la aplicación.
  3. Caducidad: **nunca**.
  4. Permisos: **`whatsapp_business_messaging`** y
     **`whatsapp_business_management`**.

**Para una demo, genera el permanente.** Un token que caduca a media
presentación es un fallo que no se puede explicar en el momento.

### 2.3 · Dos límites del número de prueba que conviene saber antes

- **Solo escribe a números que hayas verificado antes**, y son **hasta cinco**.
  Añade el tuyo desde el panel de Meta, y hazlo **antes** de la demo: Meta manda
  un código de confirmación.
- **El primer mensaje tiene que ser una plantilla aprobada.** Meta no acepta
  texto libre para iniciar una conversación. La buena noticia: viene una
  plantilla ya aprobada llamada **`hello_world`**, y sirve perfectamente para
  demostrar que el mensaje sale y llega.

Esto último importa porque ARLES **todavía no tiene pantalla para escribir
plantillas** (entrega 6.8, pendiente). Con `hello_world` no hace falta.

### 2.4 · Lo que se escribe en ARLES

En **Remitentes › WhatsApp › Añadir número**: el nombre con que quieras
identificarlo, el teléfono de prueba que dio Meta, los dos identificadores y el
token.

Después el número pide **dos confirmaciones**, cada una con su botón:

- **Coexistencia.** Confirma que el número puede seguir atendiéndose desde la
  aplicación de WhatsApp Business del móvil. ARLES no puede comprobarlo solo.
- **El riesgo del modo en frío**, si eliges prospección directa.

**No están en el formulario a propósito.** Como casillas del alta se marcarían
con el resto de los campos sin leerlas, y son justamente la constancia de que a
alguien se le avisó. Son un buen momento de la demo: enseñan una decisión de
producto, no un trámite.

---

## 3 · Dónde se hace esto, y dónde no

> **Antes de nada hace falta abrir ARLES**, y eso tiene su propia guía desde el
> 07/10/2026: [`COMO-ABRIR-ARLES.md`](COMO-ABRIR-ARLES.md) — qué instalar en
> Windows y los dos comandos que abren la ventana. Esta guía daba por hecho ese
> paso y nunca lo explicaba.

**En tu equipo, no en la nube.** Esta sesión de trabajo corre en un contenedor
de Linux sin escritorio, y ahí dos cosas lo impiden:

- **La red bloquea los puertos 587, 465 y 25** — medido, no supuesto. Ningún
  servidor de correo es alcanzable desde aquí.
- **ARLES exige el llavero del sistema** y no arranca sin él (ADR-0011). Un
  Linux sin escritorio no tiene Keychain ni Credential Manager.

Es el bloqueo **R-24** del proyecto, y por eso sigue abierto: no es algo que se
pueda cerrar desde donde se escribe el código. Se cierra en tu máquina, con la
cuenta desechable del §1.

---

## 4 · Orden sugerido para preparar la demo

1. **Hoy:** crea la cuenta de Gmail desechable y su contraseña de aplicación
   (§1.2). Cinco minutos.
2. **Hoy:** crea la app en Meta, saca los dos identificadores y genera el token
   permanente (§2.2). Verifica tu móvil como destinatario de prueba (§2.3) —
   esto es lo que más tarda, porque depende de que llegue un código.
3. **Antes de la demo:** configura el remitente de correo en ARLES y lanza
   «Probar conexión». Si algo falla, falla aquí y no delante de nadie.
4. **Las dos demos salen.** Correo de punta a punta; WhatsApp hasta el mensaje
   de prueba que llega al teléfono.
5. **Lo que todavía no sale** es una campaña entera por WhatsApp: el motor solo
   recorre correo (§0).

---

## 5 · Lo que esta guía no cubre

- **Dar de alta un número propio de empresa** en lugar del de prueba. Exige
  verificación del negocio ante Meta y puede tardar días.
- **Plantillas propias aprobadas.** Hace falta la pantalla de la entrega 6.8, y
  además Meta las revisa.
- **Las métricas de entrega y respuesta.** Dependen de los webhooks, y eso
  necesita el «relay ciego» que Dirección decidió el 29/09/2026 — un servicio
  aparte que todavía no existe.
