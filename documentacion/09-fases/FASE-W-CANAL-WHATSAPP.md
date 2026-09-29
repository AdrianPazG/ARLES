# Canal de WhatsApp · W.1 a W.6

> **Estado al 29/09/2026.** Seis entregas construidas y empujadas. El canal
> **no está terminado**: falta el cable entre el adaptador y el motor, y está
> documentado abajo sin rodeos porque determina qué se puede enseñar.

---

## 1 · Qué se construyó

| Entrega | Qué es | Dónde vive |
|---|---|---|
| **W.1** | El núcleo: los dos modos del canal (L-9), la plantilla aprobada de Meta y el permiso | `arles-core/src/whatsapp.rs` |
| **W.2** | El adaptador de Cloud API y un servidor de mentira que imita a Meta | `arles-whatsapp/src/cloud.rs`, `servidor.rs` |
| **W.3** | El número guardado: llavero, Coexistencia (L-11) y la aceptación de riesgo | `arles-db/src/numero.rs` |
| **W.4** | El procesador de avisos: firma verificada, respuestas contadas, texto descartado | `arles-whatsapp/src/entrante.rs` |
| **W.5** | La puerta del canal: las dos cifras de permiso y lo que bloquea activar | `arles-db/src/revision_whatsapp.rs` |
| **W.6** | Los comandos y la pantalla del número | `arles-app/src/whatsapp.rs`, `app/…/PantallaWhatsApp.vue` |

**70 pruebas** cubren el canal, dentro de las 850 del proyecto.

---

## 2 · Lo que NO está, y por qué importa

### 2.1 · ~~El adaptador no está enchufado al motor~~ → **cerrado el 29/09/2026 (W.8)**

> **Resuelto en la entrega W.8**, escrita el mismo día que esta auditoría lo
> encontró. `arles-app` ahora depende de `arles-whatsapp`, la fábrica arma el
> proveedor leyendo el token del llavero, y el comando
> `enviar_whatsapp_de_prueba` manda un mensaje real. Hay una prueba de punta a
> punta (`crates/arles-app/tests/whatsapp_de_prueba.rs`) que recorre el camino
> entero y comprueba **lo que llegó al servidor**, no solo lo que devolvió el
> comando.
>
> Lo que sigue es el diagnóstico original, porque explica cómo pasó.

`ProveedorDeWhatsApp` sabe hablar con Cloud API y tiene su prueba de punta a
punta contra el servidor de mentira (`arles-whatsapp/tests/envio.rs`). Pero:

- `arles-app/src/fabrica.rs` **solo construye `ProveedorSmtp`**. Nadie construye
  nunca el de WhatsApp.
- `arles-motor` **solo trabaja con `Canal::Correo`**.
- **No hay comando de envío de prueba** para WhatsApp, mientras que correo tiene
  `enviar_correo_de_prueba`.

Consecuencia: **ARLES no puede mandar un WhatsApp real.** Puede dar el número de
alta, confirmar Coexistencia, aceptar el riesgo y revisar una campaña — y ahí se
detiene.

Se encontró el 29/09/2026 al preparar la guía de conexión real, buscando qué
enseñar en una demostración. **Ninguna prueba podía cazarlo**: las de
`arles-whatsapp` prueban el adaptador contra el servidor de mentira y pasan
perfectamente; lo que falta es que alguien lo llame, y una prueba de un módulo
no comprueba que otro módulo lo use.

Es el mismo patrón que el editor de plantillas de la Fase 6: **la pieza existe,
probada, y nadie la invoca**. Conviene anotarlo como algo a vigilar, porque ya
van dos.

### 2.2 · Las decisiones de logística que quedan fuera

| Decisión | Qué pedía | Estado |
|---|---|---|
| **L-5** | Una sección CONVERSACIONES, visible solo con WhatsApp configurado | **No construida.** Además sigue pendiente de Dirección como pregunta **L-b** |
| **L-6** | Semáforo por canal, visible mientras hay campaña activa | La calidad se enseña en la pantalla del número; el semáforo durante la campaña es Fase 7 |
| **L-8** | Plantillas rotativas, midiendo cuál rinde | **No construida** |
| **L-10** | Estadísticas de los dos canales juntas en ACTIVIDAD | Fase 7 |

### 2.3 · Las respuestas no se pueden contar todavía

El procesador de avisos está escrito y probado, pero **los avisos de Meta son
webhooks**: Meta llama a una URL pública, y ARLES es una aplicación de escritorio
sin componente de red. No hay URL a la que llamar.

Dirección decidió el 29/09/2026 construir un **relay ciego** —un servicio que
recibe los webhooks, guarda solo identificadores y estados, y al que ARLES
pregunta con una API Key—. No existe todavía. El análisis y sus tres correcciones
están en el historial de esta rama.

---

## 3 · Las decisiones que sí quedaron hechas

- **L-9 · Dos modos.** Seguimiento y prospección directa, con reglas distintas.
  El desplegable empieza en seguimiento: si empezara en frío, quien solo pulsa
  «guardar» daría de alta un número en frío sin haber leído nada.
- **L-11 · Coexistencia.** Confirmación explícita, con autor y fecha. Sin ella el
  número no puede enviar, y eso sí bloquea: sin Coexistencia nadie puede
  responder desde el móvil, y eso no se arregla escribiendo con cuidado.
- **L-3 · El permiso es un registro con prueba.** La revisión de campaña lee la
  **última** entrada de cada dirección, no cualquiera: quien dio permiso en enero
  y lo retiró en marzo no tiene permiso.
- **L-7 · No se comprueba si un número tiene WhatsApp.** Cumplido por omisión, que
  aquí es la forma correcta de cumplirlo.

---

## 4 · Las reglas que gobiernan este canal

Están escritas más de una vez a propósito, y hay pruebas que fallan si alguien
toca una.

1. **El token entra y no sale.** Vive en el llavero, con espacio de nombres
   propio (`whatsapp-number/`). En la base hay una referencia opaca. Ningún
   comando lo devuelve, y una prueba serializa la salida y lo busca.
2. **Lo que dice la gente no se guarda.** El texto de un mensaje entrante muere
   en la misma función que lo lee. Se cuenta quién contestó, no qué dijo.
3. **La firma se comprueba antes de mirar el cuerpo**, y en tiempo constante.
   Analizarlo primero sería procesar lo que mande cualquiera que descubra la
   dirección, y preguntarse después si tenía derecho.
4. **Escribir sin permiso no se bloquea: se hace difícil.** El §154 pide
   informar, no dictar. La cifra de quien no dio permiso se enseña siempre, y
   activar pide escribir una palabra.
5. **Una calidad que no consta no se pinta en verde.** `null` significa «no se
   sabe». Un semáforo que se pone en verde cuando no ha podido mirar es peor que
   no tener semáforo.

---

## 5 · Lo que se rompió a propósito

Catorce garantías, todas vistas fallar con el mensaje correcto antes de darlas
por buenas. Las que más enseñaron:

| Se rompió | Qué habría pasado |
|---|---|
| Devolver el total como si todos tuvieran permiso | El aviso de campaña en frío desaparece de la pantalla |
| Leer la primera entrada de consentimiento en vez de la última | Quien retiró su permiso vuelve a contar como que lo dio |
| Bloquear el modo en frío | El modo queda construido y sin poder usarse nunca |
| Sacar el número de Meta por la frontera IPC | Una credencial de dirección cruza a la webview |
| Pintar como alta una calidad sin consultar | Un semáforo en verde por no haber mirado |

---

## 6 · Dos fallos que encontró esta fase, y que no eran suyos

**La firma de `guardar_numero` hacía imposible la llamada.** Generaba el
identificador dentro, pero la referencia del llavero lo lleva dentro y hay que
construirla fuera. La prueba de la W.3 pasaba una referencia acabada en `/x` —un
identificador que no existía— y pasaba, porque el método generaba el suyo y la
referencia no se volvía a leer. Es la regla del proyecto en forma de dato
inventado que ninguna prueba cruzaba contra nada.

**Los textos de los dos modos no existían.** El núcleo los declaraba desde la
W.1 y en `es.ts` no estaban, así que la pantalla habría enseñado el nombre de la
clave. Ninguna prueba de Rust podía verlo: el núcleo dice **cómo se llama** el
texto, no si alguien lo escribió.

Y **seis tokens de diseño inventados** en la pantalla, que el navegador resuelve
a nada sin dar error. Los cazó el validador de la fase 2 al correr las siete —
exactamente el fallo que `ESTADO_ACTUAL.md` §8 bis documenta de la 5.2,
repetido.

---

## 7 · Verificación

```
853 pruebas de Rust          0 fallan
139 pruebas de frontend      0 fallan
7 fases del validador        0 fallan
cargo clippy                 0 advertencias
eslint · vue-tsc             limpios
```

**Nada de esto se ha probado contra Meta de verdad.** El bloqueo **META ·
Cuenta con Coexistencia** sigue abierto y depende de Operaciones. Todo lo
anterior se probó contra un servidor de mentira que imita a Cloud API, incluidos
sus códigos de error.

---

## 8 · Lo siguiente, en orden de lo que desbloquea

1. ~~**W.8 · Enchufar el adaptador**~~ ✅ **hecho el 29/09/2026.** Queda que el
   *motor* entienda `Canal::Whatsapp` para una campaña entera; el envío de
   prueba, que es lo que hace demostrable el canal, ya funciona.
2. **6.8 · El editor de plantillas y firmas**, que la Fase 6 dejó pendiente.
3. **El relay ciego** (§2.3), para las métricas.
4. **Fase 7**, donde caen L-5, L-6 y L-10.
