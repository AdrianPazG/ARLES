# La verificación OAuth de Google · qué es, qué gana ARLES y qué hay que hacer

**Para Dirección · 15 de octubre de 2026**

Esto es el trámite **P-05**, abierto desde la Fase 0 con la decisión **D-1**
(*«SMTP en la v1.2.0, verificación OAuth en paralelo»*). Lo que sigue son las
instrucciones para hacerlo y lo que ARLES gana a cambio.

> **Límite de esta guía, dicho antes que nada.** Las páginas oficiales de Google
> —`support.google.com` y `developers.google.com`— **no se alcanzan desde el
> contenedor donde se desarrolla ARLES**: la red los bloquea. Lo que sigue sale
> de los resultados de búsqueda y de guías de terceros, contrastado entre
> varias. Los pasos del Cloud Console cambian de nombre cada pocos meses, así
> que **al recorrerlo hay que ir corrigiendo esta guía** — es el §2 aplicado a
> la documentación: una instrucción que nadie ha seguido no está verificada.

---

## 1 · La diferencia, en una pantalla

| Hoy · contraseña de aplicación | Con OAuth |
|---|---|
| Activar la verificación en dos pasos | — |
| Entrar a una página escondida de Google | — |
| Generar 16 letras que **sólo se enseñan una vez** | — |
| Copiarlas a ARLES | — |
| | **Pulsar un botón, elegir tu cuenta, aceptar** |

Eso es todo lo que cambia para quien usa ARLES. Lo que cuesta está del otro
lado: un trámite con Google de semanas, que es lo que esta guía explica.

---

## 2 · Las ventajas, en orden de importancia real

### 2.1 · Funciona con las cuentas de empresa, que hoy pueden no poder

**Ésta es la grande, y es comercial, no técnica.**

Una cuenta de Google Workspace —un correo `@tuempresa.com` gestionado por
Google— puede tener las contraseñas de aplicación **desactivadas para todo el
dominio**, y lo decide el administrador. Desde 2025 Google empuja a las
empresas hacia OAuth precisamente por eso.

Quien compre ARLES va a querer usar su correo corporativo. Hoy existe la
posibilidad real de que **no pueda**, y de que la solución esté en manos de un
administrador de sistemas que no es él.

### 2.2 · El alta deja de expulsar gente

Dirección no pudo dar de alta su cuenta al primer intento (H-03). Si le pasa a
quien construyó el producto, le pasa al cliente — y el alta es el primer minuto
de uso, donde más caro sale perder a alguien.

### 2.3 · ARLES nunca llega a ver la contraseña

Hoy la contraseña entra por el formulario, cruza la frontera y va al llavero.
Está bien resuelto —no se guarda en claro ni vuelve nunca (ADR-0011)— pero
**existe dentro de ARLES un instante**.

Con OAuth no existe nunca: quien teclea la contraseña es Google, en su propia
página, y ARLES recibe un *token* que no sirve para entrar a la cuenta.

### 2.4 · El permiso es exactamente «enviar», y nada más

`gmail.send` **sólo puede mandar correo**. No puede leer la bandeja, ni borrar,
ni ver contactos. Una contraseña de aplicación abre el acceso por protocolo, no
por operación: la misma sirve para IMAP, que **sí lee todo el buzón**.

Que ARLES hoy no use IMAP no cambia el argumento — lo que importa es qué podría
hacer la credencial que el cliente nos confía, y es la pregunta que hace un
responsable de seguridad antes de autorizar una compra.

### 2.5 · Se revoca desde Google, con nombre y fecha

El cliente ve «ARLES RELAY» en la lista de aplicaciones con acceso a su cuenta,
con cuándo se lo dio, y lo quita de ahí.

> **Esto ya se puede hoy** borrando la contraseña de aplicación, así que la
> ventaja es menor de lo que parece. La diferencia es que la lista de OAuth es
> donde la gente **mira**, y una contraseña de aplicación llamada «ARLES» en una
> página escondida, no.

### 2.6 · Lo que NO es una ventaja, y conviene no venderlo como tal

- **No manda más correo.** La cuota de Gmail no cambia de forma que a ARLES le
  importe: el producto limita a 50 diarios por decisión propia (§48), muy por
  debajo de cualquier tope.
- **No mejora la entregabilidad.** Eso lo deciden SPF, DKIM y DMARC (la 7.5),
  no el método de autenticación.
- **No quita la verificación en dos pasos** al cliente si ya la tenía. Sólo
  deja de **exigirla** para dar de alta.

---

## 3 · Lo que cuesta, dicho entero

| | |
|---|---|
| **Tiempo de calendario** | **2 a 8 semanas** de revisión de Google, que no controlamos |
| **Un dominio público** | Con la página del producto y la política de privacidad **en el mismo dominio** |
| **La política de privacidad publicada** | Toca **P-09**: es un texto legal, y hoy están marcados como provisionales |
| **Un vídeo** | Demostrando el flujo completo |
| **Sólo resuelve Google** | Outlook es otro trámite con Microsoft. Zoho, iCloud y los servidores propios **siguen con contraseña** |
| **Mantenimiento** | Cambiar de scopes obliga a volver a enviar |
| **Una dependencia nueva** | Si Google retira la verificación, los clientes dejan de poder enviar |

---

## 4 · Los pasos, y quién hace cada uno

> **Separado a propósito.** Hay pasos que son decisiones y papeles —tuyos— y
> pasos que son configuración —míos—. Los tuyos son los que tienen plazo
> externo, así que son los que marcan el reloj.

### Paso 1 · El dominio · **Dirección** 🔴 *bloquea todo lo demás*

Hace falta **un dominio que controlemos**, con:

- la **página del producto** de ARLES, pública y accesible;
- la **política de privacidad**, pública, en **ese mismo dominio**.

No vale un documento de Google Drive ni un PDF suelto: Google comprueba que el
dominio es nuestro a través de **Google Search Console**.

**Esto es literalmente la pregunta P-05**, abierta desde la Fase 0: *qué dominio
se usa, quién redacta el aviso de privacidad y quién lo publica.*

### Paso 2 · La política de privacidad · **Dirección + abogado**

Tiene que decir qué datos toca ARLES y para qué. Va dentro de **P-09**, las
dieciocho preguntas jurídicas de `08-legal/CONSULTA-JURIDICA.md`.

Hay una ventaja de calendario: **ya existe** `04-seguridad/PRIVACIDAD_LFPDPPP.md`
con el análisis hecho. No se parte de cero; falta la versión publicable y el
visto bueno.

### Paso 3 · El proyecto en Google Cloud · **yo**

Crear el proyecto, activar la API de Gmail, configurar la pantalla de
consentimiento: nombre de la aplicación, logotipo, correo de soporte y contacto
del desarrollador.

> **El correo de soporte importa más de lo que parece:** toda la conversación
> con el equipo de revisión de Google va por ahí, a los propietarios y editores
> del proyecto. Un correo que nadie mira son semanas perdidas esperando una
> respuesta que ya llegó.

### Paso 4 · Verificar el dominio en Search Console · **yo, con tu acceso**

Se demuestra que el dominio es nuestro — normalmente subiendo un archivo o
añadiendo un registro DNS. Necesito que alguien con acceso al dominio lo haga o
me lo dé.

### Paso 5 · El adaptador en ARLES · **yo**

`GoogleProvider`, con Authorization Code + PKCE, la autenticación **en el
navegador del sistema** y nunca en una ventana embebida, y los tokens al
llavero.

> **Esto ya está dimensionado, no es un añadido.** La abstracción
> `EmailProvider` se diseñó desde el día uno con este adaptador previsto
> (`01-producto/FUERA_DE_ALCANCE.md` §2). Y `lettre` habla XOAUTH2, así que el
> motor de envío **no cambia**.

### Paso 6 · El vídeo · **yo grabo, tú apruebas**

Un vídeo que enseñe el flujo entero, subido a YouTube como **no listado**. Tiene
que verse:

- la pantalla de consentimiento **con el nombre correcto** de la aplicación;
- el **identificador de cliente** en la barra de direcciones del navegador;
- **cada permiso pedido, usándose** — es decir, ARLES mandando un correo.

Se graba con cuentas de prueba, nunca con datos de clientes.

### Paso 7 · Enviar y justificar · **yo**

En el Cloud Console se envía a revisión, explicando **por qué ARLES necesita
`gmail.send`**: porque manda campañas de correo desde la cuenta del propio
usuario. La justificación se lee: una vaga alarga la revisión.

### Paso 8 · Esperar y contestar · **los dos**

Google responde por correo, a veces pidiendo cambios. **Cada ida y vuelta añade
días**, así que el objetivo es que la primera entrega esté completa.

---

## 5 · Mientras no esté verificada, ARLES puede probarse igual

Sin la verificación, la aplicación queda en modo de pruebas:

- **hasta 100 cuentas** dadas de alta a mano como usuarios de prueba;
- sale una pantalla de **«Google no ha verificado esta aplicación»**, con un
  «Configuración avanzada → Continuar» para pasar.

Es feo pero **funciona**, y sirve para grabar el vídeo y para probarlo nosotros.
Lo que no sirve es para vendérselo a un cliente: esa pantalla, en una
demostración, da la impresión contraria a la que queremos.

---

## 6 · La contradicción que hay que resolver antes de empezar

`09-fases/FASE-01-CIMIENTOS.md` §7 dice, del trámite de verificación de Google:

> *«Arrancado en paralelo — P-05»*

Y **P-05 sigue abierta**, preguntando el dominio, el aviso de privacidad y quién
lo publica — que es exactamente lo que hace falta para arrancarlo.

**Las dos cosas no pueden ser ciertas a la vez.** O el trámite arrancó y P-05
debería estar cerrada, o lleva meses sin arrancar y el documento afirma de más.
Lo que ocurrió fuera del repositorio no me consta: es información de Dirección.

> **Importa porque es el camino crítico.** Si el trámite no está en marcha, el
> reloj de esas 2–8 semanas **no ha empezado a correr**, y todo lo demás de esta
> guía espera al Paso 1.

---

## 7 · La recomendación

**Arrancar los pasos 1 y 2 ya, y no esperar a construir nada.** Son los únicos
con plazo externo, y hasta que exista el dominio con su política publicada **no
se puede ni empezar** el trámite. Lo demás —el adaptador, el vídeo, el envío—
se hace mientras Google revisa, no antes.

**No bloquear la v1.2.0 con esto.** Es la decisión D-1 y sigue siendo correcta:
el producto entrega con contraseña de aplicación, y OAuth entra en la v1.2.x
cuando la verificación llegue. Lo contrario sería parar una entrega terminada
esperando a un tercero.

**Y mientras tanto, bajar el dolor del alta sin cambiar el mecanismo** — el
botón que abre la página de Google, el error traducido a instrucciones, los
cuatro casos detectados. Son horas de trabajo y sirven igual el día que OAuth
exista, porque los proveedores que no son Google ni Microsoft seguirán pidiendo
contraseña.

---

## 8 · Dónde está lo demás

| Para | Mira |
|---|---|
| Por qué no entró en la v1.2.0 | `01-producto/FUERA_DE_ALCANCE.md` §2 |
| La decisión D-1 | `09-fases/FASE-00-DISCOVERY.md` |
| La pregunta P-05, entera | `02-auditoria/PREGUNTAS_ABIERTAS.md` |
| El análisis de privacidad ya hecho | `04-seguridad/PRIVACIDAD_LFPDPPP.md` |
| Las dieciocho preguntas al abogado | `08-legal/CONSULTA-JURIDICA.md` |
| Cómo se conecta hoy, con contraseña | `07-entrega/GUIA-DE-CONEXION-REAL.md` |
