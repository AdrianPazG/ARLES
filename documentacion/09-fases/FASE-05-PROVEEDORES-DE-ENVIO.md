# Fase 5 · Proveedores de envío

**Estado:** 🟡 **la parte de correo, cerrada — WhatsApp sigue sin empezar** · **Fecha:** 2026-09-29
**Validación:** 35/35 comprobaciones de la fase 5, **0 omitidas** — `validar.py --fase 5`
**Pruebas:** 720 de Rust · 120 de frontend
**Auditoría de la fase:** 29/09/2026 — dos hallazgos (§8)

> **Objetivo de la fase.** Que ARLES deje de hablar con un proveedor imaginario
> y hable con uno de verdad: entender lo que contesta un servidor de correo,
> guardar una cuenta sin guardar su contraseña, y mandar un correo entero.

---

## 0. Lo primero, porque cambia cómo se lee todo lo demás

**Esta fase no está cerrada del todo, y el título no lo va a disimular.**

El ROADMAP incluye en la Fase 5 el adaptador de **WhatsApp Cloud API**
(P-15, 17/09/2026). No está empezado: no hay ni una línea. Lo que se cierra hoy
es **la parte de correo**, que son las seis entregas 5.1 a 5.6.

Se dice aquí, en la primera línea del documento, porque un documento de cierre
que diga «fase cerrada» sobre una fase con un canal entero sin construir es
exactamente la clase de papel que hace que alguien planifique la Fase 6 contando
con algo que no existe.

**Y hay un segundo asterisco, más grande:** ARLES **todavía no ha hablado con un
servidor de correo real**. Todo lo que sigue está probado contra un servidor de
mentira que habla el protocolo de verdad. Es el riesgo **R-24**, y no se puede
cerrar desde el entorno de desarrollo. Ver §9.

---

## 1. Las seis entregas

| | Qué | Puerta de salida | Estado |
|---|---|---|---|
| **5.1** | Traducir lo que dice un servidor | Un `550` al destinatario y uno al remitente no significan lo mismo | ✅ §2 |
| **5.2** ⭐ | El adaptador con cifrado obligatorio | **No existe ninguna forma de no cifrar**, y la contraseña no sale en claro | ✅ §3 |
| **5.3** | Probar, guardar y dar de alta una cuenta | Seis diagnósticos accionables; la contraseña va al llavero y no vuelve | ✅ §4 |
| **5.4** | El motor encendido | Al abrir ARLES hay un hilo que atiende las campañas, aparte de la ventana | ✅ §5 |
| **5.5** ⭐ | El envío de prueba | **ARLES manda un correo entero**, y no da de baja a nadie al hacerlo | ✅ §6 |
| **5.6** | Cierre: auditoría, documentos e informe | `--fase 5` en verde, 0 omitidas | ✅ §8 y este documento |

---

## 2. Traducir lo que dice un servidor (5.1)

El motor entiende **cinco fallos**. Un servidor de correo contesta con números
de tres cifras y texto libre. Toda la entrega es esa traducción, y una fila mal
puesta no se deshace:

- Confundir un `450` («ahora no puedo») con un `550` («ese buzón no existe»)
  **mete en la lista de bajas a alguien que sí existe**.
- Y al revés, insiste tres veces contra un buzón muerto — que es justo lo que
  sube el contador con el que los proveedores marcan a un remitente como spam.

**El texto libre no decide nada.** Sólo el número. Un servidor que conteste
`250 no such user` se cree al `250`, porque el texto es de quien lo escribió y
el número es el protocolo.

**El servidor de mentira** (`arles-email/src/servidor.rs`) habla SMTP de verdad
y sabe fallar a la orden. Existe por una razón simple: **a Gmail no se le puede
pedir que conteste `550`**, y probar el adaptador sólo contra Gmail significa
probarlo el día del lanzamiento, con los contactos de un cliente.

---

## 3. El adaptador, con cifrado obligatorio (5.2)

### No hay forma de no cifrar, y está escrita tres veces

`CifradoDeCorreo` no tiene una variante sin cifrar. `arles-email::tls` no puede
construir parámetros sin cifrado. Y la columna `smtp_security` de la V7 sólo
admite `starttls` y `directo`.

Tres sitios para una regla parece mucho hasta que se piensa en el día en que
alguien «arregle» uno de los tres para hacer funcionar un servidor mal
configurado. Con la restricción en la base, el valor **no cabe** aunque el
código lo dejara pasar.

### Y ARLES comprueba el cifrado por su cuenta

Configurar la biblioteca para exigir TLS es necesario y no basta: es confiar en
que un tercero haga lo que promete justo donde equivocarse cuesta la contraseña
de correo de la empresa. Antes de mandar `AUTH`, el adaptador **le pregunta al
socket** si está cifrado y se niega a seguir si no lo está.

### La conversación se lleva paso a paso

La biblioteca trae una función que hace el envío entero de una vez. Es cómoda y
**no se usa**: cuando falla dice «el servidor contestó 550» y no dice **en qué
momento**.

Eso aquí no es un detalle. Un `550` contestando a `RCPT TO` significa «esa
dirección no existe» → esa persona acaba en la lista de bajas, para siempre. El
mismo `550` contestando a `MAIL FROM` significa «tu cuenta no puede enviar desde
esa dirección» → el problema es del remitente y no tiene nada que ver con quien
iba a recibir el correo.

Sin saber cuál de los dos fue, ARLES daría de baja a un contacto inocente por un
problema del remitente.

---

## 4. La cuenta: probarla, guardarla y darla de alta (5.3)

**«Probar conexión» da seis diagnósticos, y cada uno pide una acción distinta**:
no se llegó al servidor · tardó demasiado · el cifrado no se pudo establecer ·
la contraseña no sirve · el servidor no está disponible · correcto. Un «no se
pudo conectar» a secas deja a alguien probando puertos media hora.

**La contraseña entra y no sale.** Va al llavero del sistema —una entrada por
cuenta, no una compartida— y en la base queda una referencia opaca que no sirve
para entrar a ningún sitio. Ninguna estructura que cruce a la interfaz tiene un
campo donde quepa.

**La migración V7** añadió las cuatro columnas de SMTP que faltaban en
`email_account` **desde la V1**. La tabla prometía guardar una cuenta de correo
y no tenía dónde guardar el servidor: un esquema que promete algo y un código
que no lo cumple no falla, simplemente no hace nada.

---

## 5. El motor encendido (5.4)

Aquí apareció el hallazgo más grande de la fase, y no estaba en ningún plan:
**`arles-app` ni siquiera dependía de `arles-motor`**. El motor de la Fase 4
estaba construido, probado entero y **nadie lo arrancaba**. Dos mitades hechas y
sin presentar, y ninguna prueba lo cazaba porque cada mitad pasaba la suya.

Tres piezas:

| Pieza | Qué hace |
|---|---|
| `arles-app/src/fabrica.rs` | De una cuenta guardada al adaptador. **El único sitio donde una contraseña sale del llavero** |
| `arles-db/src/trabajo.rs` | Qué etapas están enviando, con su ventana y el **más bajo** de sus topes |
| `arles-app/src/motor.rs` | El hilo que las atiende, aparte de la ventana (§51) |

### Tres decisiones que conviene conocer antes de tocar esto

**La puerta se abre con `connected`; no se cierra ante una lista.** Escrita al
revés, el día que una migración añada un quinto estado esa cuenta **enviaría
correo** hasta que alguien se acordara de prohibirlo. Y nadie se acuerda.

**Sin contraseña guardada no se intenta nada.** El atajo —continuar con la
cadena vacía— haría que el servidor contestara «credenciales incorrectas» y
ARLES le dijera al usuario que su contraseña de aplicación no sirve, cuando el
problema es que ARLES nunca la buscó bien. Y varios intentos fallidos seguidos
marcan la cuenta.

**El hilo no muere por un error.** Va al revés que el resto del programa y es
deliberado: si ese hilo termina, ARLES deja de enviar con la ventana abierta
como si nada, las campañas siguen diciendo «en marcha» y el usuario se entera
horas después. Un fallo se reintenta en la vuelta siguiente. Lo único que sí lo
detiene es perder el cerrojo de instancia única — ahí callarse es lo correcto.

### Si se abren dos ARLES

Sólo envía el primero. El segundo **abre igual**: se pueden mirar contactos,
campañas y la configuración. Negarle el arranque dejaría a alguien sin poder
trabajar por un doble clic. Y lo dice en pantalla, con las tres partes del §95
— incluida la que importa: **no se ha perdido ni duplicado ningún correo**.

---

## 6. El envío de prueba (5.5)

### Dónde consta, y cuándo deja de valer

La marca vive en la **etapa** de la campaña, no en la campaña: lo que una prueba
prueba es una etapa concreta, con su remitente y su canal. Guardada en la
campaña, mandar una prueba por correo daría por probada también la etapa de
WhatsApp — otra cuenta, otro proveedor, otro formato de mensaje.

Y se guarda **con qué cuenta se probó**. Una prueba dice «este remitente mandó
un correo que salió»; si después se cambia el remitente, la frase deja de ser
cierta. Una marca que sobreviviera al cambio haría que el preflight diera luz
verde **basándose en una prueba de otra cosa**.

La marca no se borra al cambiar de cuenta: se conserva y se compara. «Se probó
el 28/09 con otro remitente» es justo lo que hay que poder enseñar cuando
alguien pregunta por qué el preflight se puso en rojo.

### Un envío de prueba no toca nada más

No entra en la cola, no gasta presupuesto, no aparece en el registro de envíos y
**no puede dar de baja a nadie**.

Es la regla 3 de la fase escrita como código: si un `550` de esta función
pudiera suprimir, probar una cuenta mal configurada contra la dirección propia
del usuario **lo daría de baja a él**, y una baja no se deshace. Hay una prueba
que lo sostiene y una comprobación del validador que lo vigila.

### Y ARLES manda un correo entero

`arles-app/tests/envio_de_prueba.rs` recorre la cadena completa contra un
servidor que habla **cifrado**, y comprueba lo que el servidor **recibió** —
`MAIL FROM`, `RCPT TO`, `DATA`— no lo que el adaptador dice que mandó.

**Hay una costura, y está en un sitio elegido.** El adaptador de esa prueba se
construye a mano y no con la fábrica, porque la fábrica verifica certificados
como debe y el servidor de mentira presenta uno que ninguna autoridad ha
firmado. Que se niegue es lo correcto. Queda fuera el tramo «de la fila de la
base y el llavero al adaptador», que está probado aparte con ocho casos.

### Dos botones, no uno

En Remitentes hay **«Probar conexión»** y **«Mandar un correo de prueba»**, y son
dos cosas:

- El primero dice que la cuenta **autentica**. No manda nada.
- El segundo dice que **puede enviar**, que no es lo mismo: una cuenta puede
  autenticar perfectamente y que el servidor rechace el remitente.

El segundo avisa de que manda un correo de verdad, y cuando sale dice **«el
servidor aceptó»**, nunca «entregado» (§65). Es el punto de toda la aplicación
donde más tienta decir lo contrario.

---

## 7. Las tres reglas de la fase, y dónde están escritas

| Regla | Dónde vive | Qué la vigila |
|---|---|---|
| **No existe ninguna forma de no cifrar** (§29) | El tipo del núcleo, el adaptador y el `CHECK` de la V7 | `no_hay_forma_de_no_cifrar`, `la_base_no_admite_una_conexion_sin_cifrar` |
| **La contraseña entra y no sale** | El llavero; en la base una referencia opaca | `solo_el_llavero_toca_contrasenas`, `nada_que_cruce_la_frontera_lleva_contrasena`, `la_contrasena_no_se_guarda_en_la_webview` |
| **Nada antes del `RCPT TO` da de baja a nadie** | El adaptador lleva la conversación paso a paso | `un_envio_de_prueba_no_da_de_baja_a_nadie`, y su prueba de integración |

---

## 8. La auditoría de la fase (29/09/2026)

Se recorrieron los seis requisitos del ROADMAP para esta fase. **Dos hallazgos,
y sólo uno obligaba a hacer algo.**

### Hallazgo 1 · La fase incluye WhatsApp, y no está empezado

Ya está en §0. Es lo que impide declarar la fase cerrada, y lo que hay que tener
delante al planificar la Fase 6: el asistente de campañas ofrece una o dos
etapas, y la segunda **no tiene adaptador**.

**Acción:** el ROADMAP, el `ESTADO_ACTUAL` y el avance dicen ahora «la parte de
correo, cerrada», no «Fase 5 cerrada».

### Hallazgo 2 · Una brecha que no existía

Se sospechó que la cadena **credencial mala → la cuenta se marca → nadie la
vuelve a usar** sólo estaba probada por tramos: el disyuntor en `disyuntor.rs`,
el estado en `cuentas.rs`, el rechazo en `fabrica.rs`. Se escribió una prueba
nueva para recorrerla entera.

Al romper `tras_un_fallo` a propósito falló **también la prueba que ya existía**
—`una_credencial_caducada_para_la_cuenta_a_la_primera`—, y con una aserción más
fuerte que la nueva: comprueba que sólo se pidió **un** envío al proveedor.

**Acción: la prueba nueva se borró**, y la existente lleva ahora una nota que lo
cuenta. Una prueba redundante no es gratis — da una falsa sensación de cobertura
y hace creer que hay dos garantías donde hay una.

### Lo que la auditoría comprobó y encontró en orden

| Requisito del ROADMAP | Estado |
|---|---|
| Trait `EmailProvider` | ✅ `arles_motor::proveedor::Proveedor`, implementado por `ProveedorSmtp` |
| `SmtpProvider` con TLS obligatorio | ✅ y comprobado contra el socket, no sólo configurado |
| Prueba de conexión | ✅ seis diagnósticos accionables |
| Clasificación de errores SMTP | ✅ y el texto libre no decide nada |
| Captura de rechazos 5xx hacia la supresión | ✅ el `550` al `RCPT TO` llega a `suppression_entry` con origen `RespuestaSmtp` |
| Guía de la contraseña de aplicación de Gmail | ✅ en la pantalla, plegada |
| **Adaptador de WhatsApp Cloud API** | ⬜ **sin empezar** |

---

## 9. R-24 · ARLES nunca ha hablado con un servidor real

**Es el riesgo abierto más importante de la fase**, y esta entrega no lo cierra.

Todo lo anterior está probado contra un servidor de mentira que habla el
protocolo de verdad y sabe fallar de seis maneras. Eso cubre las ramas de error
mucho mejor que una prueba manual contra Gmail — pero no cubre lo que sólo
aparece con un servidor real: un saludo inesperado, una extensión que no
habíamos visto, un límite de tasa, una peculiaridad del certificado.

**No se puede comprobar desde el entorno de desarrollo, y se midió:** la red
bloquea los tres puertos de correo (587, 465 y 25 agotan el tiempo de espera; el
nombre sí resuelve). Y aunque se abriera la red, ARLES exige el llavero del
sistema y no arranca sin él.

**Lo que esta fase cambia es que ahora se puede cerrar del todo.** Antes sólo se
podía comprobar que la cuenta conectaba; desde la 5.5 se puede mandar un correo
y ver si llega:

1. Dar de alta la cuenta en **Remitentes**, con una **contraseña de aplicación**
   y una **cuenta desechable** — no la de trabajo.
2. Pulsar **«Mandar un correo de prueba»** con la dirección propia.
3. Mirar la bandeja de entrada **y la carpeta de correo no deseado**.

---

## 10. Lo que esta fase deja listo para la Fase 6

- **Un motor que envía de verdad.** La Fase 6 construye encima: plantillas,
  sustitución de variables y el asistente de nueve pasos.
- **El punto del preflight «se hizo el envío de prueba» ya tiene dónde
  contestarse.** Falta quien lo pregunte, que es la pantalla de campañas.
- **El comando del envío de prueba por etapa existe y está probado**, y hoy no
  tiene desde dónde llamarse: no hay pantalla de campañas. Es lo primero que la
  Fase 6 puede enchufar.

### Y una deuda declarada

El adaptador de **WhatsApp Cloud API** sigue siendo de esta fase. La Fase 6
ofrece dos etapas en el asistente, y mientras eso no exista, la segunda etapa no
puede enviar. Conviene decidir si se construye antes de la Fase 6 o si el
asistente se entrega ofreciendo sólo correo.
