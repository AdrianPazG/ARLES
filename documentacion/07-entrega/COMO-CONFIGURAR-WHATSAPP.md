# Cómo conectar tu WhatsApp a ARLES

**Para:** Dirección
**Última actualización:** 26 de octubre de 2026

Esta guía te lleva de cero a mandar un WhatsApp de prueba desde ARLES.

> **Lo que ARLES te pide son dos cosas: un token y elegir tu número de una
> lista.** Todo lo demás —los identificadores largos de Meta, el nombre, la
> calidad del número, cuántos mensajes al día te deja mandar— lo pregunta ARLES
> y lo rellena solo.
>
> El trabajo está en Meta, no en ARLES. Esta guía es sobre todo para esa parte.

---

## Antes de empezar: lo que hace falta

| Qué | Por qué |
|---|---|
| **Un número de teléfono que no esté en WhatsApp** | Meta no deja usar un número que ya tenga WhatsApp normal o WhatsApp Business instalado. Si el tuyo lo tiene, tienes dos salidas: borrar esa cuenta desde la app, o usar otro número |
| **Una cuenta de Facebook** | Es con la que se entra al panel de Meta. No hace falta una página ni publicar nada |
| **Un correo y un rato** | El alta en Meta son unos 20 minutos la primera vez |

> **Para probar, usa un número que no te importe.** Un número que empieza a
> mandar mensajes fríos puede acabar con la calidad baja o bloqueado, y eso **no
> se arregla**: se cambia de número. Para las pruebas, uno desechable; el bueno,
> cuando el flujo ya funcione.

---

## Paso 1 · Dar de alta WhatsApp Business en Meta

1. Entra en **developers.facebook.com** y pulsa **«Mis aplicaciones»**.
2. **«Crear aplicación»** → tipo **«Empresa»** → ponle un nombre (el de tu
   empresa sirve).
3. Dentro de la aplicación, busca **WhatsApp** en la lista de productos y pulsa
   **«Configurar»**.
4. Meta te pedirá asociar o crear una **cuenta de empresa**. Créala con el
   nombre de tu empresa.

Al terminar verás una pantalla de WhatsApp con un número de prueba que Meta te
regala.

> **El número de prueba de Meta sirve para probar y no para trabajar.** Sólo
> puede escribir a cinco números que tú apuntes a mano, y caduca. Vale
> perfectamente para la primera prueba con ARLES — de hecho es lo que
> recomiendo— pero para una campaña de verdad hace falta el paso 2.

---

## Paso 2 · Añadir tu número de verdad

Sáltate este paso si sólo vas a probar.

1. En la misma pantalla de WhatsApp, pulsa **«Agregar número de teléfono»**.
2. Escribe el número **con su código de país** (+52 para México).
3. Meta te manda un código por SMS o por llamada. Mételo.
4. Pon el **nombre que verá quien reciba tus mensajes**. Meta lo revisa, y
   tarda de unas horas a un par de días.

> **Ese nombre es lo que la gente ve, y cambiarlo después cuesta.** Pon el
> nombre comercial, no una abreviatura interna.

---

## Paso 3 · Crear una plantilla

**Por WhatsApp no se puede mandar texto libre a alguien que no te escribió
primero.** Sólo plantillas que Meta haya aprobado. Es la regla de Meta, no de
ARLES.

1. En el panel, ve a **«Administrador de WhatsApp»** → **«Plantillas de
   mensajes»** → **«Crear plantilla»**.
2. Elige categoría **«Marketing»** o **«Utilidad»**:
   - **Utilidad** si es algo que la persona espera (una confirmación, un aviso
     de su pedido). Se aprueba más rápido y cuesta menos.
   - **Marketing** si es promocional. Es lo normal para una campaña.
3. Idioma: **Español (MEX)**.
4. Escribe el texto. Donde quieras meter un dato de la persona, usa `{{1}}`,
   `{{2}}`…

**Ejemplo que suele aprobarse:**

```
Hola {{1}}, somos {{2}}.
Tenemos una propuesta que puede interesarte para tu empresa.
¿Te parece si te cuento en dos minutos?
```

5. Envía a revisión. Meta tarda de minutos a 24 horas.

> **Por qué se rechazan las plantillas, en orden de frecuencia:** empezar
> directamente con una oferta sin presentarse; prometer resultados
> («garantizado», «el mejor»); escribir TODO EN MAYÚSCULAS; meter un enlace
> acortado; o pedir datos personales. Si te la rechazan, Meta dice el motivo y
> se puede corregir y reenviar.

---

## Paso 4 · Sacar el token

Esto es **lo único que ARLES te va a pedir**.

### Para probar: el token temporal

1. En la pantalla de WhatsApp del panel, verás **«Token de acceso temporal»**.
2. Pulsa **«Copiar»**.

> **Caduca en 24 horas.** Para la primera prueba es lo más rápido. Cuando
> caduque, ARLES te dirá que el token ya no vale y vuelves aquí a copiar otro.

### Para trabajar: el token permanente

1. Ve a **business.facebook.com** → **«Configuración del negocio»**.
2. **«Usuarios»** → **«Usuarios del sistema»** → **«Agregar»**.
3. Ponle un nombre (`ARLES` sirve) y rol **«Administrador»**.
4. Pulsa **«Asignar activos»**, elige tu **cuenta de WhatsApp** y dale
   **control total**.
5. Pulsa **«Generar nuevo token»**:
   - Elige tu aplicación.
   - Caducidad: **«Nunca»**.
   - Marca estos dos permisos: **`whatsapp_business_messaging`** y
     **`whatsapp_business_management`**.
6. **Copia el token ahora.** Meta no lo vuelve a enseñar.

> **Los dos permisos hacen falta, y no es lo mismo.** `messaging` es para
> mandar; `management` es para que ARLES pueda **preguntar cuáles son tus
> números** — que es lo que te ahorra copiar los identificadores a mano. Sin el
> segundo, ARLES te dirá que el token no vale aunque sirva para enviar.

---

## Paso 5 · Conectarlo en ARLES

1. Abre ARLES → **Remitentes** → pestaña **WhatsApp**.
2. Pulsa **«Conectar un número»**.
3. **Pega el token** y pulsa **«Buscar mis números»**.
4. ARLES le pregunta a Meta y te enseña tus números, con su nombre, su calidad
   y cuántos mensajes al día te deja mandar Meta. **Elige el tuyo.**
5. Elige el **modo**:
   - **Seguimiento** — sólo escribe a quien te dio permiso o te escribió
     primero. Sin riesgo.
   - **Prospección directa** — escribe en frío. Es legítimo y es lo que más
     calidad cuesta: Meta mira cuánta gente te bloquea o te reporta.
6. Guarda.

> **El token entra y no sale.** Se guarda en el llavero de Windows, no en la
> base de datos de ARLES ni en ningún archivo. Ninguna pantalla lo vuelve a
> enseñar, y si alguien se lleva tu base de datos no se lleva el token.

---

## Paso 6 · La prueba

1. En la ficha del número, pulsa **«Enviar un WhatsApp de prueba»**.
2. Escribe **tu propio número** con código de país.
3. Elige la plantilla aprobada del paso 3.
4. Envía.

**Si el mensaje llega a tu teléfono, está todo bien.** Esa es la prueba de que
el token, el número y la plantilla funcionan juntos.

> **Si usas el número de prueba de Meta**, antes tienes que apuntar tu número
> en la lista de destinatarios permitidos, en la misma pantalla del panel donde
> copiaste el token. Si no, Meta rechaza el envío y ARLES te dirá que ese
> número no existe — que es lo que Meta contesta, aunque exista.

---

## Qué hacer cuando algo falla

| ARLES dice | Qué pasa de verdad | Qué hacer |
|---|---|---|
| **El token no vale** | Caducó (el temporal dura 24 h), lo revocaron, o le falta el permiso `whatsapp_business_management` | Genera uno nuevo con los dos permisos del paso 4 |
| **No se pudo llegar a Meta** | Sin internet, o Meta caída | Reintenta en unos minutos. No toques el token |
| **No hay ningún número** | El token es bueno pero tu cuenta no tiene ningún número dado de alta | Vuelve al paso 2 |
| **Ese número no existe** al enviar | Con el número de prueba de Meta, que tu destino no está en la lista de permitidos | Apunta tu número en esa lista, en el panel |
| **La plantilla no está aprobada** | Meta todavía la está revisando, o la rechazó | Mira su estado en el Administrador de WhatsApp |

---

## Lo que ARLES **no** puede hacer, y conviene saberlo antes

**No puede contar las respuestas.** Meta las manda por «webhooks», que son
llamadas a un servidor en internet, y ARLES es una aplicación de escritorio: no
hay servidor al que llamar. La cifra de «cuánta gente contestó» será cero en
WhatsApp hasta que exista esa pieza, y está dicho en la pantalla.

**No puede crear nada en Meta.** No da de alta números, no crea plantillas ni
las manda a revisar. Todo eso es el panel de Meta. ARLES lee y envía.

**No puede recuperar tu token.** Si lo pierdes, se genera otro. Meta no lo
vuelve a enseñar y ARLES no lo puede leer de vuelta: vive en el llavero del
sistema y sólo sale hacia Meta.

---

## Una advertencia que no es técnica

**Un número de WhatsApp se quema y no se desquema.** Si mandas en frío a una
lista que no te espera, la gente bloquea y reporta, Meta baja la calidad del
número, y cuando llega a rojo deja de dejarte enviar. Eso no se arregla
esperando: se cambia de número, y el nuevo empieza otra vez desde el escalón
más bajo.

Por eso ARLES te enseña la calidad en la ficha del número y te avisa antes de
activar una campaña. **Esa cifra en amarillo es una advertencia, no un
adorno.**

Empieza con poco volumen, a gente que al menos sepa quién eres, y sube despacio.
