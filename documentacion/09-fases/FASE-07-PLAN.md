# Fase 7 · Actividad y entregabilidad — plan

**Estado:** 📋 planificada, sin empezar · **Escrito:** 2026-09-29
**Peso en la v1.2.0:** 9 de 100 (`avance.json`) · **Depende de:** Fase 6 ✅, Canal W ✅

> **En una frase.** Hoy ARLES **envía y no cuenta nada**: se activa una campaña
> y la única forma de saber qué está pasando es abrir la base de datos. La
> Fase 7 convierte lo que el motor ya hace en algo que se ve, se entiende y
> —esto es lo importante— **se puede parar**.

---

## 1 · El hallazgo que cambia el orden de la fase

Antes de planificar nada miré qué existe de verdad. Y apareció esto:

> ### Una campaña activada no se puede parar desde la interfaz.

La Fase 4 construyó Pausar, Reanudar, Detener y la parada de emergencia.
Funcionan, están probados, sobreviven a cerrar el programa. `arles-db` expone
`aplicar_a_la_campana` y `poner_parada_de_emergencia`. **Ningún comando de
Tauri los llama, y ninguna pantalla tiene el botón.** Los textos en español ya
están escritos en `es.ts` —`campana.pausar`, `campana.detener`,
`campana.pausarExplicacion`— desde la Fase 6, esperando una pantalla que nunca
llegó.

Es el **séptimo caso** del patrón que el `CLAUDE.md` §9 ya documenta cinco
veces, y el más grave de todos: los anteriores dejaban una función sin usar;
este deja al usuario **sin freno** sobre un proceso que escribe a personas
reales.

| Dónde | Qué faltaba | Consecuencia |
|---|---|---|
| Fase 6 | `guardar_plantilla`, sin pantalla | No se podían escribir plantillas |
| Canal W | `ProveedorDeWhatsApp`, sin dependencia | No se podía mandar nada |
| 6.8b | `whatsapp_template`, tabla sin escritor | La puerta del canal no abría |
| 6.8a | `PantallaPlantillas.vue`, sin ruta | 13 kB inalcanzables |
| Vista previa | Once comandos sin respuesta | Cinco pantallas con error |
| **Fase 4** | **Pausar/Detener, sin comando ni botón** | **No se puede parar un envío** |

**Por qué ninguna comprobación lo cazó.** `lo_que_se_construye_se_usa()` mira
que `arles-app` dependa de los crates; `toda_pantalla_se_puede_abrir()` mira
que cada `.vue` tenga ruta. Ninguna de las dos pregunta lo contrario: **qué
capacidades del núcleo no tienen puerta**. Esa comprobación se escribe en la
7.0, antes de arreglar el hueco, para verla marcarlo en rojo (§2).

**Consecuencia para el plan:** la 7.0 deja de ser «preparar el terreno» y pasa
a ser **la primera entrega, y es urgente**. No se puede enseñar ARLES a nadie
—ni hacer la prueba real con correo de R-24— sin un botón de parar.

---

## 2 · Lo que hay hoy, medido

Inventario de lo que la Fase 7 puede usar, verificado leyendo el código y el
esquema el 29/09/2026.

### Ya existe y sirve

| Qué | Dónde | Para qué sirve en la Fase 7 |
|---|---|---|
| `message_attempt` | V1, migrada en V3 y V4 | **La fuente de toda cifra.** Estado, fecha de envío, error, número de intento, cuenta y canal, por destinatario |
| Nueve estados de envío | `CHECK (state IN …)` | `queued`, `claimed`, `sending`, `sent`, `failed`, `permanently_failed`, `suppressed`, `cancelled`, **`presumed_sent`** |
| `audit_log` | V1, **append-only por disparador** | La bitácora de decisiones: quién activó, quién aceptó un riesgo, quién dio de baja |
| `Paso` del motor | `arles-motor/src/bucle.rs` | Once desenlaces **con su clave de i18n ya escrita y pensada para esta pantalla** |
| `aplicar_a_la_campana` | `arles-db/src/campanas.rs` | Pausar, reanudar, detener, con transición validada por el núcleo |
| `poner/levantar_parada_de_emergencia` | ídem | El interruptor general, persistente |
| `whatsapp_reply` | V9 (W.4) | Respuestas contadas **sin guardar lo que dice la gente** |
| `whatsapp_quality_event` | V4 | La calidad del número según Meta, con su histórico |
| `rate_budget` | V1/V4 | Cuánto cupo queda esta hora y este día |
| Textos de campaña | `es.ts` | `pausar`, `reanudar`, `detener` y sus explicaciones, ya redactados |

### No existe, y hay que construirlo

| Qué falta | Por qué importa |
|---|---|
| **Comandos de control** | Sin ellos no hay botón de parar (§1) |
| **Registro de sucesos** | `una_vuelta()` hace `.map(\|ultimo\| ultimo.hay_mas_que_hacer())`: **convierte el `Paso` en un booleano y lo tira.** Nada queda de «a las 10:04 la cuenta se paró» |
| **Comandos de lectura agregada** | No hay ninguno que cuente envíos. Ni uno |
| **La pantalla** | `/actividad` es andamio |
| **Resolución de DNS** | Para SPF, DKIM y DMARC no hay ninguna dependencia en el workspace. `registros_del_dominio` está en `None` a la espera desde la Fase 6 |

---

## 3 · Qué es Actividad, y qué no

**Las tres preguntas que la pantalla contesta**, en este orden, porque es el
orden en que se hacen:

1. **¿Está pasando algo ahora mismo?** → el motor, las campañas en marcha, el
   ritmo real, cuánto cupo queda.
2. **¿Cómo fue lo de ayer?** → por campaña: cuántos salieron, cuántos fallaron,
   cuántos se dieron de baja, cuántos quedan.
3. **¿Por qué falló esto?** → el detalle de un envío concreto, con lo que dijo
   el servidor y qué hacer al respecto.

**Lo que NO va a decir, y hay que escribirlo antes de que alguien lo pida:**

- **«Entregado».** ARLES no lee buzones (§69), así que no puede saberlo. Dice
  «aceptado por el servidor», que es lo que de verdad ocurrió (§65). Esta
  distinción no es cautela: es la diferencia entre un dato y una suposición.
- **«Tasa de apertura», «clics».** Requieren píxel de rastreo y enlaces
  reescritos. ARLES **no rastrea a nadie**, y eso está en la visión del
  producto, no es una carencia pendiente.
- **«Tasa de rebote real».** Sólo se ven los rechazos síncronos del servidor
  (5xx en el momento). Los rebotes asíncronos llegan por correo horas después
  y necesitan leer un buzón → v1.3 (ADR-0009). **La pantalla lo declara**, en
  vez de enseñar un número incompleto sin avisar.
- **Cifras de WhatsApp de campaña.** El motor sólo recorre `Canal::Correo`. De
  WhatsApp hoy sólo hay envíos de prueba y respuestas; decir otra cosa sería
  inventar.

---

## 4 · Las tres perspectivas

Dirección pidió mirarlo desde los tres lados. Cada uno pide cosas distintas y
alguna se contradice: eso se resuelve aquí y no durante la construcción.

### 4.1 · Desde quien usa ARLES

Adrián activa una campaña de 1 342 personas un martes a las 9:00 y cierra la
tapa del portátil. Lo que necesita, en sus palabras:

| Lo que se pregunta | Lo que la pantalla tiene que responder |
|---|---|
| «¿Está funcionando?» | **Una frase, arriba, sin interpretar nada.** «Enviando · 340 de 1 342 · va por la mitad del martes» |
| «¿Cuánto falta?» | Una estimación con su método a la vista: «al ritmo de ahora, termina el jueves a las 11». Nunca una barra sin número |
| «¿Algo va mal?» | Los fallos **agrupados por causa**, no una lista de 1 342 líneas. «12 direcciones no existen», «3 rebotaron» |
| «Para esto ya» | **Un botón visible siempre, no escondido en un menú.** Y que diga qué va a pasar: pausar guarda el sitio, detener no se deshace |
| «¿Le llegó a Ana?» | Buscar una dirección y ver su historia: cuándo salió, con qué cuenta, qué contestó el servidor |

**La decisión de diseño que se deriva de esto:** la pantalla se ordena por
**urgencia**, no por completitud. Arriba lo que pide acción; abajo el detalle
que se consulta cuando alguien pregunta.

### 4.2 · Desde el equipo que lo construye

| Necesidad | Cómo se resuelve |
|---|---|
| **No inventar una segunda verdad** | Toda cifra sale de `message_attempt` con `SELECT … GROUP BY state`. Ningún contador paralelo que se pueda desincronizar |
| **Que el motor no se ralentice** | El registro de sucesos escribe **lo que ya se sabe**, en la misma transacción del intento. Sin segunda escritura por envío |
| **Que la tabla no crezca sin fin** | Retención declarada: los sucesos de bucle se recortan; los desenlaces de envío **no se borran nunca** (son la prueba del §37) |
| **Que se pueda probar sin esperar** | El simulador de la Fase 4 ya sabe fallar de seis maneras a la orden. La sonda de Actividad lo usa para producir cada estado en segundos |
| **Que no se repita el hueco de §1** | Una comprobación nueva: **toda capacidad del núcleo tiene puerta** |

### 4.3 · Desde Dirección

| Lo que necesita | Dónde lo ve |
|---|---|
| Enseñar ARLES funcionando sin miedo | Un botón de parar visible cambia la conversación de la demostración entera |
| Saber si el producto es vendible | Las métricas honestas son un argumento comercial, no una limitación: **ningún competidor dice «aceptado ≠ entregado»**, y eso protege de una reclamación |
| Que R-24 se pueda cerrar | La prueba con correo real necesita ver qué pasó. Hoy no hay dónde mirar |
| Cifras para el informe | El resumen exportable de una campaña, en un archivo que se manda por correo |

---

## 5 · Las siete entregas

Cada una: se construye, **se verifica rompiéndola** (§2), se empuja, se
explica. Una detrás de otra, con permiso antes de la siguiente (§3).

---

### 7.0 · El freno · **primero, y es urgente**

**El problema del §1.** Dos días de trabajo que desbloquean todo lo demás.

- Comandos `pausar_campana`, `reanudar_campana`, `detener_campana`, sobre el
  `aplicar_a_la_campana` que ya existe y valida la transición en el núcleo.
- `parada_de_emergencia` y `levantar_parada`, con su constancia en `audit_log`:
  **quién y cuándo**.
- Los botones en la pantalla de Campañas, con el texto ya escrito. Detener
  pide confirmación y dice qué se pierde; pausar no, porque se deshace.
- La parada de emergencia **en el armazón**, no dentro de una pantalla: si hay
  que parar todo, no se busca dónde.

**Se verifica rompiéndola:** quitar el comando y ver fallar la comprobación
nueva `toda_capacidad_del_nucleo_tiene_puerta()`; y detener una campaña con el
simulador enviando, comprobando que el motor **deja de tomar trabajo en la
vuelta siguiente** y que los intentos ya tomados acaban en `cancelled`.

---

### 7.1 · El registro de sucesos

La tabla que hoy no existe, con la migración **V10**.

- `activity_event`: momento, campaña, etapa, cuenta, canal, clave del suceso
  (la de `Paso::clave_i18n`, que ya existe), intento asociado si lo hay.
- **Sin datos personales en la tabla**: el intento se referencia por id, y la
  dirección se lee de `message_attempt` al enseñarla. Así una cancelación ARCO
  que vacía el contacto **no deja la dirección viva en la bitácora** — que es
  exactamente el problema P-17 que ya está abierto sobre `consent_entry`.
- `una_vuelta()` deja de tirar el `Paso`.
- Retención: los sucesos de bucle (`NoHayTrabajo`, `FueraDeVentana`) se
  recortan a 30 días; los desenlaces de envío no se borran.

**Se verifica rompiéndola:** hacer que el motor descarte un `Paso::Rendido` y
ver fallar la prueba que exige que todo desenlace quede escrito.

---

### 7.2 · Las cifras, y un solo sitio que las calcula

Todo **por campaña**, porque puede haber tres (§6 bis). El total es una suma de
lo mismo, nunca una consulta aparte que pueda desviarse.

- `resumen_de_campana(id)`: los nueve estados agrupados, en una consulta, con
  desglose **por etapa** — una campaña puede tener correo y WhatsApp (L-1).
- `respuestas_de_campana(id)`: la cifra de D-7A, sumando sus etapas. Sobre el
  `count` que `respuestas.rs` ya tiene.
- `cupo_de_los_remitentes()`: cuánto queda por cuenta, **y qué campañas tiran
  de cada una**. Es lo que resuelve el problema del §6 bis.
- `ritmo_real(id)`: envíos por hora de las últimas seis, para la estimación.
- **La estimación de término lleva su método**: «al ritmo de las últimas 6 h».
  Un número sin método es una promesa.

**Dos trampas a evitar, las dos ya pagadas antes (§2):**

1. Una prueba con **un solo envío** pasa con cualquier agregación. Las de esta
   entrega usan un caso de cada estado, **dos cuentas y tres campañas**, para
   que un `GROUP BY` al que le falte una columna falle.
2. El total **se calcula sumando las campañas**, y hay una prueba que lo exige:
   si alguien escribe un `SELECT` general aparte, el día que las dos consultas
   discrepen nadie sabrá cuál creer.

---

### 7.3 · La pantalla · lo de ahora

**Se organiza por campaña** (§6 bis), no por métricas globales.

- Una línea arriba: «3 campañas enviando · 47 en la última hora».
- **Una tarjeta por campaña en marcha**, con su progreso en cifras (§94), su
  cifra de respuestas (D-7A) y su botón de detener (D-7B).
- **Los remitentes y su cupo**, con las campañas que compiten por cada cuenta.
  Es el problema que va a salir el primer día y la sección que lo explica.
- La razón por la que algo está quieto, **cuál de las seis es** (propuesta P-A),
  con su cifra: «vuelve a las 9:00», «quedan 20 para las tres».
- «Parar todo» en la cabecera, sólo si hay algo enviando (D-7B).
- Actualización **cada 5 s mientras la pestaña está visible**; al ocultarse, se
  para. Y sin saltos: los números cambian, la pantalla no se rehace.

### 7.4 · El registro, con sus filtros

Lo que D-7C define, y es la mitad más delicada de la fase.

- **Nunca más de 500 filas en pantalla**, siempre, con el total a la vista:
  «500 de 3 412 · usa el filtro para encontrar los demás».
- **Cuatro filtros combinables**: campaña, qué pasó, cuándo, dirección o número.
  Se reflejan en la dirección para poder volver a un estado concreto.
- Cada fallo dice las **tres cosas del §95**: qué pasó, cómo se arregla y qué
  está a salvo.
- **«Aceptado» con su aclaración a la vista**, no en un pie de página.
- El recorte bimestral con su aviso previo y la opción de guardar copia; los
  desenlaces de envío **no se borran jamás**, sólo dejan de salir por defecto.

**La prueba del tope usa 501 registros**, no uno: comprueba que llegan 500 y
que el total dice 501. Es exactamente el defecto que ya nos costó una vez.

### 7.5 · Entregabilidad: SPF, DKIM y DMARC

Lo único de la fase que necesita **red y una dependencia nueva**.

- Resolución DNS con `hickory-resolver` (antes `trust-dns`), que es Rust puro y
  no arrastra C. Pasa por `cargo deny` antes de entrar.
- Tres comprobaciones sobre el dominio del remitente, con **el texto de qué
  poner en el DNS**, no sólo un aspa roja.
- Completa el punto `registros_del_dominio` que el preflight lleva esperando
  desde la Fase 6.
- **Se cachea**: el DNS no cambia cada cinco minutos y no se consulta en cada
  envío.

**Riesgo declarado:** el contenedor de desarrollo puede tener el DNS filtrado,
igual que bloquea los puertos de correo (R-24). Si es el caso, la comprobación
se prueba contra un resolutor de mentira y **se dice que no se probó contra
uno real**, como se hizo con SMTP.

### 7.6 · El resumen exportable

- Un archivo de una campaña: cifras, fallos por causa, y el método de cada
  número.
- **Ninguna ruta cruza la frontera IPC** (regla 4): se usa el mismo camino que
  el informe de acceso ARCO, que ya lo resuelve.

### 7.7 · Cierre

Auditoría de la fase, la revisión **a ojo** en Chromium que el A3 demostró que
hace falta, documento de fase, informe para Dirección y el avance recalculado.

---

## 6 · Las tres decisiones, resueltas por Dirección

Contestadas el **30/09/2026**. Dejan de ser preguntas y pasan a ser requisitos:
lo que sigue es lo que se construye, y el motivo por el que se construye así.

---

### D-7A ✅ · Las respuestas se **cuentan**, no se leen — y van por campaña

**Lo que decidió Dirección.** Contestar los mensajes es trabajo del asesor de
ventas, no de ARLES. De la respuesta sólo interesa **la cifra**: cuántas
personas contestaron a cada campaña. Y como se pueden tener **tres campañas a
la vez**, las cifras tienen que ir **por campaña**, no sólo en total.

**Lo que esto significa técnicamente** — verificado en el esquema antes de
prometerlo:

`whatsapp_reply` (V9, entrega W.4) ya guarda exactamente eso y nada más:

```sql
campaign_stage_id  TEXT NOT NULL REFERENCES campaign_stage(id)
from_normalized    TEXT NOT NULL        -- para no contar dos veces a la misma persona
first_reply_at     TEXT NOT NULL
UNIQUE (campaign_stage_id, from_normalized)
```

Tres cosas que salen gratis porque ya se decidieron bien en la W.4:

1. **El texto del mensaje no existe en la base.** No hay que quitarlo: nunca se
   guardó. Se descarta al recibirlo, antes de escribir nada.
2. **El `UNIQUE` cuenta personas, no mensajes.** Alguien que conteste cinco
   veces suma uno. Es la cifra que Dirección quiere, y ya está bien contada.
3. **`arles-db` ya sabe contarlas**: `SELECT count(*) … WHERE campaign_stage_id = ?1`
   existe en `respuestas.rs`. Falta el comando y la pantalla — otro caso del
   patrón del §1, y esta vez con el trabajo de datos hecho.

**Lo que cambia en el plan.** La reestructuración de las pantallas está en §6
bis, porque «tres campañas a la vez» no es un detalle de esta decisión: es la
forma de toda la pantalla.

**Lo que hay que seguir declarando.** Sin el relay ciego, `whatsapp_reply` sólo
se llena en el envío de prueba. La cifra existirá y será correcta; **estará en
cero hasta que haya relay**, y la pantalla lo dirá con esas palabras en vez de
enseñar un cero que parece un fracaso.

---

### D-7B ✅ · Detener vive en la campaña. «Parar todo» es otra cosa, y también

**Lo que pidió Dirección.** El botón en el apartado de Campañas — y que yo
proponga y elijamos la más sencilla.

**La propuesta, y por qué.** Aquí hay **dos acciones que se parecen y no son la
misma**, y confundirlas es lo que hace daño cuando hay tres campañas en marcha:

| | Detener una campaña | Parar todo |
|---|---|---|
| Qué hace | Termina **esa** campaña; las otras siguen | Congela **las tres** de golpe |
| Cuándo se usa | «Esta ya no la quiero» | «Algo va muy mal, para ya» |
| Se deshace | No: hay que crear otra | **Sí**: se levanta y todo sigue donde iba |
| Quién lo pulsa | Alguien que decidió | Alguien asustado |

Con tres campañas, un solo botón rojo que signifique las dos cosas es una
trampa: quien quiere apagar una apaga las tres, o al revés.

**Las cuatro opciones que consideré:**

| | Dónde | Problema |
|---|---|---|
| A | Sólo en Campañas | En una emergencia hay que navegar. Y si se está mirando Actividad —que es donde se ve que algo va mal—, hay que salir de ahí |
| B | Un botón rojo fijo en la barra lateral | La barra es **navegación**; un botón destructivo entre enlaces se pulsa sin querer. Y no distingue las dos acciones |
| C | En la barra, sólo si hay algo enviando | Mejor que B, pero sigue mezclando navegar con actuar |
| **D** | **Detener en cada campaña · Parar todo en la cabecera de Actividad, sólo si hay algo enviando** | Ninguno |

**Recomendación: D.** El razonamiento en una frase: **cada acción vive donde
está la información que la justifica.**

- **Detener esta campaña** → en la ficha de la campaña, y también en su tarjeta
  dentro de Actividad. Ahí se ve de qué campaña se habla, con su nombre y sus
  cifras delante. Pide confirmación escribiendo el nombre, porque no se deshace.
- **Parar todo** → en la cabecera de Actividad, **visible sólo cuando hay al
  menos una campaña enviando**, porque el resto del tiempo no significa nada.
  Dice cuántas va a parar —«parar las 3»— y **no pide confirmación**: es
  reversible, y una emergencia con un diálogo de por medio es una emergencia
  peor. Lo que sí hace es dejar constancia de quién y cuándo en `audit_log`.
- **Reanudar** aparece en el mismo sitio en cuanto hay una parada puesta, para
  que levantarla sea tan fácil como ponerla.

**El trade-off que acepto:** en una emergencia hay que estar en Actividad o ir
a ella — un clic. A cambio, ningún botón destructivo vive en la navegación, y
las dos acciones no se confunden nunca.

---

### D-7C ✅ · Se recorta, no se borra — y nunca se enseñan más de 500

**Lo que decidió Dirección.** Registros bimestrales; con más de 20 000 correos
y mensajes juntos, que sólo se vean los últimos 500; y un filtro para buscar
entre todos.

**Lo que se construye, con una precisión y una propuesta de ajuste:**

**1 · «Se recorta pero no se borra» — hay que separar dos cosas**, porque la
frase puede significar dos cosas muy distintas y una de ellas es ilegal:

| Qué | Qué se hace a los 2 meses | Por qué |
|---|---|---|
| **Sucesos de bucle** (`NoHayTrabajo`, `FueraDeVentana`, «sin cupo») | **Se borran.** Son ruido: dicen que ARLES estaba esperando | Nadie los consulta a dos meses vista, y son el 95 % del volumen |
| **Desenlaces de envío** (salió, falló, se dio de baja, se rindió) | **No se borran jamás** | Son la prueba de que se escribió a alguien. Es lo que sostiene el control anti-abuso (§37) y lo que contesta un derecho ARCO. Borrarlos nos deja sin respuesta ante una reclamación |

Lo que sí ocurre con los desenlaces al pasar dos meses es que **dejan de
salir por defecto**: siguen ahí, y el filtro los encuentra. «Recortado» pasa a
significar *fuera de la vista*, no *fuera de la base* — que es, entiendo, lo que
Dirección quiere decir con «se recorta pero no se borra».

**Y antes de borrar los sucesos de bucle, ARLES ofrece guardarlos en un
archivo.** Un aviso una vez, con la cifra: «se van a quitar 42 100 líneas de
espera de hace más de dos meses. ¿Guardar una copia antes?». El plazo se
configura en Ajustes; dos meses es el valor de partida, no una imposición.

**2 · El tope de 500 · propongo aplicarlo siempre, no a partir de 20 000**

**Problema.** El umbral de 20 000 deja sin proteger el tramo de en medio: una
campaña de 3 000 envíos produce una tabla de 3 000 líneas que ya es
inmanejable a ojo, y no llega al umbral.

**Riesgo.** La pantalla se vuelve lenta y confusa mucho antes de los 20 000, y
el usuario no entiende por qué a veces hay tope y a veces no.

**Impacto.** Bajo en lo técnico, alto en confianza: una pantalla que cambia de
comportamiento sin avisar se siente rota.

**Alternativa.** **Siempre 500**, con la cifra completa a la vista arriba: «se
enseñan 500 de 3 412 · usa el filtro para encontrar los demás». Un
comportamiento único, predecible, y que no cambia según el tamaño.

**Trade-off.** Con pocos registros el aviso sobra — se resuelve no enseñándolo
cuando el total cabe en 500, que es el caso normal al principio.

**Recomendación: siempre 500.** Es más simple de entender, más simple de
construir y más simple de probar.

**3 · El filtro, que es lo que hace útil el tope**

Cuatro filtros que se combinan, ninguno más:

| Filtro | Por qué éste |
|---|---|
| **Campaña** | Con tres a la vez, es el primero que se necesita |
| **Qué pasó** | Salió · Falló · Se dio de baja · Se rindió. Agrupado, no nueve estados sueltos |
| **Cuándo** | Hoy · Esta semana · Este mes · Todo |
| **Dirección o número** | «¿Le llegó a Ana?» — la pregunta más frecuente |

Los filtros **se reflejan en la dirección** (`/actividad?campana=…&estado=fallo`),
para que un estado concreto se pueda guardar y volver a él.

**La trampa a evitar, que ya nos costó una vez:** una prueba de paginación con
**un solo registro** pasa aunque el `LIMIT` esté mal — pedir diez mil devolvía
uno y el recorte se podía quitar entero (§2). La prueba de este tope usa **501
registros** y comprueba que llegan 500 y que el total dice 501.

---

## 6 bis · Tres campañas a la vez: lo que eso cambia

Dirección apuntó algo que reordena la pantalla entera: **se pueden lanzar tres
campañas**. No es un detalle de D-7A; es la forma de Actividad.

### La pantalla se organiza por campaña, y el total va debajo

```
ACTIVIDAD
├─ ¿Está pasando algo?          ← una línea: «3 campañas enviando · 47 en la última hora»
│
├─ Campaña «Clientes de agosto»    340 de 1 342 · 12 fallos · 3 respuestas
├─ Campaña «Reactivación»           89 de   420 ·  1 fallo  · 0 respuestas
├─ Campaña «Bienvenida»          en pausa · 0 de 88
│
├─ Los remitentes y su cupo     ← aquí está el problema de abajo
└─ El registro, con sus filtros
```

Cada campaña es una tarjeta con **sus** cifras, **su** progreso y **su** botón
de detener. El total existe, pero debajo: nadie pregunta «cuántos correos
salieron en total», preguntan «cómo va la de agosto».

### El problema que nadie ha visto todavía, y que va a salir el primer día

**Tres campañas pueden compartir la misma cuenta de correo. El cupo es de la
cuenta, no de la campaña.**

Si la cuenta permite 500 diarios y la campaña de agosto se come 480, las otras
dos **se paran** —correctamente, el motor hace lo que debe— y en la pantalla se
ven dos campañas quietas sin ninguna explicación visible. El usuario concluye
que ARLES se rompió.

Esto no es hipotético: es lo que va a pasar la primera vez que se lancen tres
campañas con un solo remitente, que es exactamente el escenario de Dirección.

**La solución, dentro de la 7.3:**

- Una sección **de remitentes** con el cupo de cada cuenta: «quedan 20 de 500
  hoy · se renueva a las 00:00».
- **Y qué campañas compiten por esa cuenta**, nombradas. Que se vea que son
  tres tirando del mismo cubo.
- En la tarjeta de una campaña parada por cupo, la razón con su cifra: «esperando
  cupo de ventas@… · quedan 20 para las tres».

Es la propuesta **P-A** aplicada al caso concreto: el motor ya sabe por qué está
quieto; sólo hay que dejar de tirarlo.

**Y una recomendación de producto que no es técnica:** con tres campañas sobre
una cuenta, lo que el usuario necesita no es sólo ver el reparto — es poder
**ordenarlas**. Una campaña marcada como prioritaria se serviría primero. Eso
**no está en la v1.2.0** y no lo propongo para ahora; lo dejo escrito porque el
día que se pida, se pida sabiendo que lo vimos venir.

## 7 · Cinco propuestas más allá de lo planeado

Dirección pidió ver más allá. Estas **no están en el ROADMAP**; las propongo
con su coste y su motivo, y ninguna se construye sin su visto bueno.

### P-A · El «¿por qué no está enviando?» explicado

Hay **seis razones distintas** por las que ARLES puede estar quieto: fuera de
ventana, sin cupo, disyuntor abierto, campaña en pausa, parada de emergencia,
cuenta sin contraseña. Hoy el usuario no ve ninguna: ve que no pasa nada.

Una sola línea que diga cuál de las seis es, con la cifra que la acompaña
—«vuelve a las 9:00», «quedan 23 minutos»— **ahorra el 90 % de los mensajes de
soporte** que un producto así genera. Coste: bajo, porque el motor ya calcula
las seis; sólo hay que dejar de tirarlas.

**Recomendación: hacerlo, dentro de la 7.3.** Es la mejora con mejor relación
valor/coste de toda la fase.

### P-B · El diario de la campaña, en lenguaje llano

Una narración corta por campaña: «Empezó el martes a las 9:04. Paró a las 11:20
porque se acabó el cupo diario. Siguió el miércoles». Se genera de
`activity_event` sin datos nuevos.

Sirve para tres cosas: que Dirección entienda sin traducción, que un problema
se pueda contar por teléfono, y que el informe de la 7.6 tenga una primera
página legible. Coste: medio.

**Recomendación: proponerla para la 7.6**, no antes.

### P-C · La salud del remitente a lo largo del tiempo

Ya se guarda `whatsapp_quality_event` con histórico. Del correo no se guarda
nada equivalente, pero el dato existe: la proporción de rechazos por cuenta y
por día está en `message_attempt`.

Una línea por cuenta —«esta semana, 2 de cada 100 rechazados; la anterior, 1»—
avisa de que un dominio se está quemando **antes** de que el proveedor lo
bloquee. Es la diferencia entre un susto y una cuenta perdida.

**Recomendación: reservarlo para la v1.3**, pero **guardar el dato desde ya**
en la 7.1 — no cuesta nada ahora y es imposible reconstruirlo después.

### P-D · La comprobación que faltaba: toda capacidad tiene puerta

El §1 de este plan es el séptimo caso del mismo patrón. Las comprobaciones
actuales miran de dentro hacia fuera; ninguna mira **qué sabe hacer el núcleo
que nadie puede pedir**.

Una comprobación que recorra los métodos públicos de `arles-db` que cambian
estado y exija que cada uno tenga un comando —o una excepción escrita y con
motivo— cerraría el patrón de raíz.

**Recomendación: hacerlo en la 7.0**, escrito antes de arreglar el hueco para
verlo en rojo. Es el punto donde una lección se convierte en una barrera.

### P-E · El modo demostración, de verdad

Hoy la vista previa es un simulador de JavaScript que hay que mantener a mano
—y se ha desincronizado del núcleo tres veces—. Con el simulador de proveedor
de la Fase 4 ya construido, ARLES podría tener un **modo demostración real**:
la aplicación entera, con su núcleo de Rust, enviando a un servidor de mentira
en memoria.

Ventajas: la demostración deja de poder mentir, la sonda del recorrido mide el
producto y no una imitación, y R-24 se prueba dos veces —contra el falso y
contra el real—.

**Recomendación: evaluarlo al cerrar la Fase 7**, no ahora. Es un cambio de
herramienta, y a mitad de fase se paga caro.

---

## 7 bis · Estar un paso adelante · el catálogo de fallos

Dirección pidió «todas las soluciones de todos los posibles problemas». Eso,
literalmente, no se puede y haría daño intentarlo: la lista de lo que *podría*
pasar no tiene fin, y cada defensa es código que mantener, probar, y que puede
fallar por su cuenta.

Lo que sí se puede, y está hecho, es
**[`06-calidad/CATALOGO-DE-FALLOS.md`](../06-calidad/CATALOGO-DE-FALLOS.md)**:
cuarenta y tantas cosas que pueden salir mal, recorridas en el código —no de
memoria—, con qué hace ARLES hoy, qué ve el usuario, y en qué estado está cada
una. Separadas en tres: **resuelto y probado**, **se resuelve en esta fase**, y
**escrito sin construir**, que vale porque el día que ocurra ya sabemos qué
hacer y no nos sorprende delante de un cliente.

De ese catálogo salen dos cosas que esta fase recoge:

1. **Las seis razones del silencio** (§1 del catálogo). El fallo más frecuente
   de un producto así no es un error: es que no pase nada y nadie sepa por qué.
   Las seis ya las calcula el motor. Van a la 7.3.
2. **Un problema resuelto no es un problema cerrado** (§4.3). La garantía de
   «dormir no produce una ráfaga» estuvo probada y era falsa durante días,
   porque la prueba no cubría una siesta dentro del mismo día. Lo que cierra un
   problema es una prueba que se ha visto fallar.

---

## 8 · Riesgos de esta fase

| Riesgo | Probabilidad | Impacto | Mitigación |
|---|---|---|---|
| **Una cifra que miente** —un `GROUP BY` mal— y nadie lo nota | Media | **Alto**: se decide sobre ella | Pruebas con un caso de cada estado y dos cuentas; nunca con uno solo |
| El registro de sucesos frena el motor | Baja | Medio | Se escribe en la transacción del intento; se mide antes de cerrar |
| El DNS filtrado impide probar la 7.5 | **Alta** (ya pasa con SMTP) | Bajo | Resolutor de mentira, y se declara qué no se probó |
| La pantalla en vivo consume batería | Media | Bajo | Se para al ocultarse la pestaña |
| Prometer WhatsApp que no existe | Media | Alto | D-7A |
| Que aparezca un octavo caso del patrón | **Media** | Alto | P-D |

---

## 9 · Cómo se verifica

Lo de siempre, más dos cosas nuevas:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check licenses advisories        # obligatorio: la 7.5 añade dependencia

cd app && npm test && npm run lint && npx vue-tsc --noEmit -p tsconfig.json

for f in 0 1 2 3 4 5 6 W 7; do python3 herramientas/validar/validar.py --fase $f; done
```

**Nuevo 1 · la fase 7 del validador**, con `toda_capacidad_del_nucleo_tiene_puerta()`.

**Nuevo 2 · la revisión a ojo**, que el A3 demostró que encuentra lo que las
sondas no: abrir la demostración y mirar cada pantalla. Va en la 7.7 y no es
opcional.

---

## 10 · Resumen para quien tenga prisa

1. **Lo primero es el freno** (7.0). Hoy no se puede parar una campaña desde
   la interfaz, y eso bloquea la demostración y R-24.
2. **Después, que el motor deje de tirar lo que sabe** (7.1).
3. **Luego las cifras, con un solo sitio que las calcula** (7.2).
4. **Después las dos mitades de la pantalla**: lo de ahora (7.3) y lo que pasó
   (7.4).
5. **La entregabilidad va al final** (7.5) porque es lo único que necesita red.
6. **Las tres decisiones están tomadas** (30/09/2026): las respuestas se
   cuentan y no se leen, detener vive en la campaña y «parar todo» en Actividad,
   y el registro se recorta a dos meses sin borrar nunca un envío.
7. **Todo va por campaña**, porque se pueden lanzar tres a la vez — y tres
   campañas sobre una sola cuenta de correo se pisan el cupo. Eso se ve, con
   nombres y cifras, antes de que parezca una avería.
8. **Cinco propuestas** fuera del plan, con recomendación explícita en cada una.
   Dos son para ahora (P-A y P-D), una para la 7.6 (P-B), y dos para después
   (P-C y P-E).
