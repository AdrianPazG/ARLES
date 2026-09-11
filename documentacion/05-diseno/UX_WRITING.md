# UX Writing

**Proyecto:** ARLES RELAY I · v1.2.0
**Idioma:** español de México · todo texto pasa por i18n desde el día uno (§139)

---

## 1. Tono

**Profesional, claro, humano.** Le hablamos a alguien que administra las comunicaciones de su empresa y que, cuando algo falla, tiene un problema real.

| Sí | No |
|---|---|
| «No se pudo conectar con el servidor.» | «¡Ups! Algo salió mal 😅» |
| «Campaña activada.» | «¡Genial! ¡Vamos allá! 🎉» |
| «1 240 mensajes quedarán sin enviar.» | «¿Estás seguro?» |

El §94 lo prohíbe explícitamente para errores de negocio. Un «¡Ups!» delante de una campaña fallida no suaviza nada: comunica que el software no entiende la gravedad de lo que acaba de pasar.

**Tuteo.** Cercano sin ser informal. «Revisa tu conexión», no «revise su conexión» ni «revisa tu conexión, crack».

**Sin emoji en la interfaz.** Iconos sí; emoji no.

---

## 2. La estructura del error (§95)

Todo error dice tres cosas, **en este orden**:

1. **Qué pasó** — concreto, sin jerga
2. **Cómo arreglarlo** — accionable
3. **Qué está a salvo** — la parte que casi nadie escribe

La tercera es la que importa. Cuando alguien ve fallar una campaña de 5 000 correos, lo primero que necesita saber no es el código de error: es **si perdió algo**.

### Ejemplos

**Conexión perdida**

> **No se pudo conectar con smtp.gmail.com.**
> Revisa tu conexión a internet y que el puerto 587 no esté bloqueado por tu red o firewall.
> **Tu campaña está en pausa, no cancelada.** Los 1 240 envíos pendientes se reanudarán automáticamente al restablecerse la conexión.

**Credencial rechazada**

> **La cuenta ventas@empresa.com ya no puede enviar.**
> El servidor rechazó las credenciales. Si cambiaste tu contraseña de Google, genera una contraseña de aplicación nueva y vuelve a conectar la cuenta.
> **Tus campañas están en pausa.** No se ha perdido ningún contacto ni mensaje. Se reanudarán en cuanto reconectes.

**Archivo rechazado**

> **No se pudo leer «contactos.xlsx».**
> El archivo supera el límite de 500 000 filas. Divídelo en archivos más pequeños e impórtalos por separado.
> **No se importó nada.** Tu base de contactos no cambió.

**Envío sin confirmar** (ADR-0004)

> **3 envíos sin confirmar**
> ARLES se cerró inesperadamente mientras enviaba. Estos mensajes pudieron salir o no, y no hay forma de comprobarlo.
> **No los reenviamos automáticamente** para evitar que alguien reciba el mismo correo dos veces. Revisa tu bandeja de enviados y decide.
> `[Ver los 3 envíos]`

---

## 3. Nombres y versión (ADR-0010)

| Contexto | Texto |
|---|---|
| Marca, empaque, comercial | **ARLES RELAY I** |
| Logotipo | **ARLES RELAY** |
| Título de ventana | **ARLES RELAY** |
| Pie | `v1.2.0` |
| Acerca de | ARLES RELAY I<br>Versión 1.2.0 (build 2026.09.11)<br>Software desarrollado por TELEMETRY INSIGHT |

**Regla: «I» y el número de versión nunca en el mismo renglón.**

El nombre visible sale de **una constante en el código**, no escrito a mano en cada sitio.

**Prohibido en cualquier artefacto:** FINAL, FINAL2, DEFINITIVO, NUEVO (§3).

---

## 4. El vocabulario de la honestidad

Es la parte del UX writing donde el producto se juega su principio central.

| Estado interno | 🖥️ Texto en interfaz | **Nunca** |
|---|---|---|
| `sent` | **Aceptado** | ~~Entregado~~ ~~Recibido~~ |
| `presumed_sent` | **Envío no confirmado** | ~~Enviado~~ |
| `failed` | Reintentando | ~~Error~~ (aún no es definitivo) |
| `permanently_failed` | Falló | |
| `suppressed` | Suprimido | ~~Bloqueado~~ |

**«Aceptado» significa que el proveedor se hizo cargo del mensaje. No significa que llegara.** ARLES no sabe eso en v1.2.0 y por tanto no lo dice (§65).

Cuando el usuario pregunte por qué no decimos «entregado», hay una explicación en la interfaz:

> **¿Por qué «aceptado» y no «entregado»?**
> Tu proveedor de correo confirma que recibió el mensaje y que intentará entregarlo, pero no informa si llegó al buzón del destinatario. ARLES sólo muestra lo que puede verificar.

Esa nota es un activo comercial, no una disculpa.

---

## 5. Declarar lo que el producto no sabe

El mismo principio aplica a las capacidades (ADR-0009).

Permanente en el centro de entregabilidad y en la pantalla de supresiones:

> **La detección automática de rebotes es parcial en esta versión.**
> ARLES detecta los rechazos que el servidor de destino comunica durante el envío, pero no los que llegan después como correo a tu bandeja.
> **Qué hacer:** revisa tu bandeja periódicamente y suprime a mano las direcciones que reboten.

Y sobre los respaldos (T-6):

> **Los respaldos no están cifrados en esta versión.**
> El archivo `.arles` contiene los datos de tus contactos. Guárdalo en un lugar seguro.

---

## 6. Botones

**Verbo + objeto.** Nunca «Aceptar», «OK» ni «Continuar» a secas en acciones con consecuencia.

| Sí | No |
|---|---|
| `Activar campaña` | `Aceptar` |
| `Detener campaña` | `Sí` |
| `Importar 1 087 contactos` | `Continuar` |
| `Suprimir dirección` | `OK` |

El botón debe poder leerse **solo**, sin el título del diálogo, y seguir siendo inequívoco. Es lo que salva a quien pulsa por reflejo.

`Cancelar` sí es aceptable: es la salida, y es universal.

---

## 7. Estados vacíos (§97)

Qué es esto + cómo empezar. Nunca un dibujo con «Nada por aquí».

> **Aún no hay campañas.**
> Una campaña envía un mensaje a una lista de contactos, respetando los límites que definas.
> `[Crear campaña]`

> **No hay direcciones suprimidas.**
> Las direcciones suprimidas quedan excluidas de todos los envíos futuros. Se añaden cuando alguien se da de baja, cuando un correo rebota de forma permanente, o cuando las excluyes a mano.
> `[Suprimir una dirección]`

El segundo enseña qué es la supresión a quien no lo sabe. Un estado vacío es el mejor momento para explicar un concepto.

---

## 8. Números y fechas

**Separador de miles siempre.** `1 240`, no `1240`.

**Fechas relativas en lo reciente, absolutas en lo antiguo.** «hace 5 minutos», «ayer a las 14:30», «12 de marzo de 2026».

**Duraciones en lenguaje llano.** «alrededor de 25 días», no «600 horas».

**La hora siempre en la zona horaria de la empresa**, y se dice cuál si difiere de la del sistema:

> Siguiente envío: **11:34** (hora de Ciudad de México)

---

## 9. Confirmaciones

**Sólo para lo irreversible.** Confirmar lo reversible entrena a la gente a pulsar «Sí» sin leer, y entonces la confirmación que sí importaba tampoco se lee.

| Acción | ¿Confirma? |
|---|---|
| Pausar campaña | No — reversible |
| Reanudar | No |
| **Detener campaña** | **Sí** — con número concreto |
| **Eliminar contactos** | **Sí** — con número |
| Suprimir dirección | No — reversible |
| **Restaurar respaldo** | **Sí** — sobrescribe datos |
| Desconectar cuenta | Sí — borra la credencial del llavero |

---

## 10. Atribución (§117–§124)

**En el pie de la aplicación**, discreta, en `--arles-text-muted`:

> Software desarrollado por TELEMETRY INSIGHT

**Nunca en los correos que el cliente envía** (§121). El correo es del cliente.

---

## 11. i18n

Todo texto visible pasa por la capa de internacionalización, aunque v1.2.0 sólo tenga español (§139).

**Sin concatenación.** Interpolación con parámetros nombrados:

```
✓  "Se importarán {count} contactos"
✗  "Se importarán " + count + " contactos"
```

**Plurales con la API de plurales**, no con `if count === 1`.

Retrofitear i18n sobre una interfaz terminada es mecánico, tedioso y siempre deja cadenas huérfanas. Hacerlo desde el inicio no cuesta casi nada, y es requisito del white labeling futuro (§117).
