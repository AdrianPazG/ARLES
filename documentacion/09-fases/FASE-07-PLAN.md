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

- `resumen_de_campana(id)`: los nueve estados agrupados, en una consulta.
- `resumen_del_dia`: lo de hoy, por cuenta y por canal.
- `ritmo_real`: envíos por hora de las últimas seis, para la estimación.
- **La estimación de término lleva su método**: «al ritmo de las últimas 6 h».
  Un número sin método es una promesa.

**La trampa a evitar, que ya nos costó una vez:** una prueba con **un solo
envío** pasa con cualquier agregación. Las pruebas de esta entrega usan al
menos un caso de cada estado y **dos cuentas**, para que un `GROUP BY` que
olvide una columna falle.

---

### 7.3 · La pantalla · lo de ahora

- El estado del motor: en marcha, parado, fuera de ventana, sin cupo.
- Las campañas en marcha con su progreso en cifras (§94).
- Cuánto cupo queda, por cuenta, con la hora a la que se renueva.
- Actualización **cada 5 s mientras la pestaña está visible**; al ocultarse, se
  para. Y sin saltos: los números cambian, la pantalla no se rehace.

### 7.4 · La pantalla · lo que pasó

- Línea de tiempo por campaña, agrupada por causa.
- Buscar por dirección: la historia de un destinatario.
- Cada fallo dice las **tres cosas del §95**: qué pasó, cómo se arregla y qué
  está a salvo.
- **«Aceptado» con su aclaración a la vista**, no en un pie de página.

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

## 6 · Lo que necesito que decida Dirección

Tres decisiones. Ninguna frena el arranque —la 7.0 y la 7.1 no dependen de
ellas— pero sí condicionan de la 7.3 en adelante.

### D-7A · ¿Actividad enseña las respuestas de WhatsApp?

**Problema.** `whatsapp_reply` cuenta respuestas desde la W.4, pero sin el
relay ciego sólo se llenan en el envío de prueba. Enseñar «2 respuestas» junto
a las cifras de correo sugiere una paridad que no existe.

**Riesgo.** Que la demostración prometa un seguimiento de WhatsApp que el
producto no da hasta que exista el relay.

**Impacto.** Bajo si se decide ahora; alto si se descubre delante de un cliente.

**Alternativa.** Una sección aparte, con la limitación escrita: «de WhatsApp,
ARLES ve lo que Meta le cuenta en el momento del envío. Las respuestas
necesitan el relay».

**Trade-off.** Una pantalla algo menos redonda a cambio de no prometer de más.

**Recomendación.** La alternativa. Y decidir el relay ciego por separado, con
su propio plan.

### D-7B · ¿La parada de emergencia vive en el armazón?

**Problema.** Si hay que parar todo, buscar la pantalla correcta cuesta
segundos que importan.

**Riesgo.** Un botón rojo siempre visible se puede pulsar sin querer.

**Impacto.** Pulsarlo sin querer detiene envíos —recuperable—. No encontrarlo
manda correo que no se quería mandar —irreversible—.

**Alternativa.** En el armazón pero **sólo visible cuando hay algo enviando**,
con confirmación.

**Trade-off.** Un elemento más en la barra, presente sólo cuando significa algo.

**Recomendación.** La alternativa: los dos errores no cuestan lo mismo.

### D-7C · ¿Se recorta el registro de sucesos?

**Problema.** Una campaña de 500 000 con reintentos puede generar millones de
filas.

**Riesgo.** La base crece sin techo; las consultas se frenan.

**Impacto.** Medible en la Fase 8, cuando se verifiquen los presupuestos.

**Alternativa.** Recortar sólo los sucesos de bucle a 30 días; **los desenlaces
de envío no se borran jamás** — son la prueba de que se envió y la base del
control anti-abuso.

**Trade-off.** La bitácora deja de ser completa hacia atrás en lo que no es
prueba de nada.

**Recomendación.** La alternativa, con el plazo configurable en Ajustes.

---

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
6. **Tres decisiones** esperan a Dirección: D-7A, D-7B y D-7C.
7. **Cinco propuestas** fuera del plan, con recomendación explícita en cada una.
   Dos son para ahora (P-A y P-D), una para la 7.6 (P-B), y dos para después
   (P-C y P-E).
