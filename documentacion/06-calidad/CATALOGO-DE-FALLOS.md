# Catálogo de fallos · qué puede salir mal y qué hace ARLES

> **Para qué es este documento.** Dirección pidió «estar siempre un paso
> adelante del usuario». Esto es ese paso: la lista de lo que puede salir mal,
> qué hace ARLES cuando pasa, y **qué ve la persona que lo está usando**.
>
> Se escribió el 30/09/2026 recorriendo el código, no de memoria. Cada fila
> marcada ✅ está construida y tiene una prueba que se ha visto fallar (§2).

---

## 0 · Una aclaración necesaria, antes de la lista

Dirección pidió «todas las soluciones de todos los posibles problemas que
podrían surgir». Tengo que decir con honestidad por qué eso, tomado al pie de
la letra, haría daño al proyecto — y qué propongo en su lugar.

**Problema.** La lista de cosas que *podrían* pasar no tiene fin. Un disco que
se llena a mitad de un `INSERT`, un antivirus que bloquea el archivo, un
usuario que cambia la hora del sistema a mano, una actualización de Windows que
reinicia el equipo, Meta que cambia su API un martes.

**Riesgo.** Construir para todas ellas antes de entregar significa no entregar.
Y cada defensa es código que hay que mantener, probar y que puede fallar por su
cuenta — la prueba de la ráfaga al despertar (§4.3 de abajo) es exactamente eso:
**la solución contenía el fallo que debía impedir**, y pasó su propia prueba
durante días.

**Impacto.** Alto y silencioso: el alcance crece, la fecha se aleja, y la
calidad baja porque hay más superficie que vigilar.

**Alternativa.** Separar en tres, que es lo que hace este documento:

1. **Lo que ya está resuelto** — se lista para no volver a resolverlo, y para
   que quien retome el trabajo no lo crea pendiente.
2. **Lo que se resuelve ahora**, porque va a pasar de verdad en el uso normal.
3. **Lo que queda escrito sin construir** — con su plan. Escribirlo cuesta diez
   minutos y vale por dos motivos: el día que ocurra ya sabemos qué hacer, y
   **no se nos cuela como sorpresa delante de un cliente**.

**Trade-off.** No estamos protegidos contra todo. Estamos protegidos contra lo
que pasa, y **avisados** de lo demás.

**Recomendación.** Esta alternativa. Y mantener el catálogo vivo: cada fase
añade lo suyo, y lo que se sube de nivel 3 a nivel 2 se decide con una fecha
delante, no cuando duele.

---

## 1 · «No pasa nada» · las seis razones, y son seis

**El fallo más frecuente de un producto así no es un error: es el silencio.**
ARLES puede estar completamente quieto por seis motivos distintos, todos
correctos, y hoy el usuario no ve ninguno — sólo ve que no pasa nada.

| Nº | Por qué está quieto | Qué hace ARLES | Qué tiene que ver el usuario | Estado |
|---|---|---|---|---|
| 1 | **Fuera de la ventana horaria** | No toma trabajo | «Vuelve a enviar a las 9:00» | 🔨 7.3 |
| 2 | **Se acabó el cupo** de la hora o del día | No toma trabajo | «Quedan 0 de 500 hoy · se renueva a las 00:00» | 🔨 7.3 |
| 3 | **El disyuntor de la cuenta está abierto** tras varios fallos | Deja esa cuenta en reposo | «Esa cuenta vuelve en 23 minutos» | 🔨 7.3 |
| 4 | **La campaña está en pausa** | Nada, a propósito | «En pausa desde el martes» + botón de reanudar | 🔨 7.0 y 7.3 |
| 5 | **Parada de emergencia puesta** | Todo congelado | «Parado por Adrián, hoy a las 11:04» + levantar | 🔨 7.0 |
| 6 | **La cuenta no tiene contraseña** en el llavero | Se salta esa cuenta | «ventas@… necesita su contraseña» + enlace | 🟡 parcial: la pantalla de Remitentes lo dice, Actividad no |

**El motor ya calcula las seis** — son variantes de `Paso` en
`arles-motor/src/bucle.rs`, cada una **con su texto en español ya escrito**. El
problema no es calcularlas: es que `una_vuelta()` las convierte en un booleano
y las tira. Está en el plan de la Fase 7 §1 y en la propuesta P-A.

> **Ésta es la mejora con mejor relación valor/coste de todo el proyecto.**
> Sin ella, cada una de estas seis situaciones normales genera una llamada de
> «ARLES no funciona».

---

## 2 · «Pasó algo que no quería»

| Qué | Qué hace ARLES | Estado |
|---|---|---|
| **Se envió dos veces a la misma persona** | No puede pasar: `UNIQUE(campaign_id, contact_email)` más una `idempotency_key` persistida **antes** de llamar al proveedor, que viaja como `Message-Id` | ✅ |
| **Se cerró ARLES a mitad de un envío** y no se sabe si salió | La fila queda en `presumed_sent` y **nunca se reenvía** (ADR-0004). Preferimos un no-envío a un duplicado: el duplicado quema el dominio y no se deshace | ✅ |
| **Se importaron 10 000 filas por error** | Por eso los choques se enseñan **antes** de importar y nada se guarda hasta confirmar (decisión de Dirección, 18/09/2026) | ✅ |
| **Se escribió a alguien que pidió la baja** | La supresión se comprueba **antes del `RCPT TO`**, y es la única puerta junto al acuse final | ✅ |
| **Se activó una campaña sin querer** | El preflight de ocho puntos y el aviso de más de 50 diarios | ✅ |
| **Se lanzó y hay que pararlo ya** | **Hoy no se puede desde la interfaz.** Es el hallazgo del §1 del plan | 🔨 7.0 · urgente |
| **Se detuvo la campaña equivocada** de las tres | Detener pide confirmación escribiendo el nombre; «parar todo» es reversible y dice cuántas para | 🔨 7.0 |

---

## 3 · «Se rompió algo de fuera»

| Qué | Qué hace ARLES | Estado |
|---|---|---|
| **El servidor de correo rechaza la contraseña** | Detiene esa cuenta entera, no ese envío. Y el diagnóstico distingue seis causas: de ellas **sólo dos dicen algo sobre la contraseña**; las otras cuatro son «no se llegó a intentar» | ✅ |
| **Un puerto cerrado** | Nunca se enseña como «la contraseña no sirve». Fue un defecto real y ahora hay una comprobación | ✅ |
| **Una dirección no existe** (rebote duro síncrono) | Se rinde y **entra en la lista de bajas** | ✅ |
| **Un rebote que llega horas después**, por correo | **No se ve.** Requiere leer un buzón → v1.3 (ADR-0009). La interfaz lo declara en vez de enseñar una cifra incompleta | 📋 v1.3 |
| **Meta inhabilita el número** de WhatsApp | La calidad se lee y se registra con histórico; la puerta del canal se cierra | ✅ |
| **Meta cambia el estado de una plantilla** por su cuenta | `anotar_plantilla_de_meta` usa `ON CONFLICT … DO UPDATE`: el estado nuevo entra sin romper nada | ✅ |
| **Se cae internet a mitad de campaña** | El disyuntor abre, se reintenta con espera creciente, el trabajo queda en la cola | ✅ |
| **El DNS no responde** al comprobar SPF/DKIM | Dirá «no se pudo comprobar», que **no es lo mismo** que «está mal». La distinción ya existe en el preflight | 🔨 7.5 |

---

## 4 · «Fue cosa de la máquina»

| Qué | Qué hace ARLES | Estado |
|---|---|---|
| **Se cierra la tapa 8 horas** y al abrirla se manda una ráfaga | **No.** El presupuesto vive en disco con etiquetas de día y hora: la etiqueta de un día no cambia porque alguien duerma. Ver §4.3 | ✅ |
| **Se abre ARLES dos veces** | Un cerrojo con latido: la segunda instancia no envía. Si la primera murió sin soltarlo, la edad del latido lo dice | ✅ |
| **Se cierra el programa** y se pierde el cupo gastado | El presupuesto está en disco, no en memoria: cerrar y abrir no regala envíos | ✅ |
| **No hay llavero del sistema** (Linux sin escritorio) | **La aplicación no arranca.** Nunca se degrada a texto plano (ADR-0011) | ✅ |
| **Se cambia la hora del sistema a mano** | El presupuesto usa etiquetas de fecha; mover el reloj hacia atrás no devuelve cupo del día en curso. **Moverlo un día entero sí abriría uno nuevo** — es la misma laguna que tendría un calendario de pared | 📋 escrito, sin construir |
| **El disco se llena** a mitad de escribir | SQLite aborta la transacción; el intento queda en su estado anterior y se reintenta. **No se ha probado a propósito** | 📋 probarlo en la Fase 8 |
| **Un antivirus bloquea el archivo** de la base | Hoy daría un error técnico. Debería decir qué carpeta excluir | 📋 Fase 8 |

### 4.3 · La lección que este catálogo no debe olvidar

La garantía de «dormir no produce una ráfaga» **estuvo probada y era falsa**.
La prueba se llamaba `ocho_horas_dormido_no_producen_una_rafaga`, pasaba, y no
cubría el caso que importaba: una siesta **dentro del mismo día**. Al despertar,
el contador volvía a cero y esa cuenta mandaba otros cincuenta. **Cien envíos
con un tope de cincuenta, escrito dentro de su propia solución.**

Se encontró el 23/09/2026 auditando la fase, no construyéndola.

> **Un problema resuelto no es un problema cerrado.** Lo que cierra un problema
> es una prueba que se ha visto fallar.

---

## 5 · «Los datos»

| Qué | Qué hace ARLES | Estado |
|---|---|---|
| **Un XLSX con una bomba de descompresión** | Tope de ratio ~100:1, de filas, de celdas y tiempo | ✅ |
| **Una fórmula de Excel inyectada** (`=cmd\|…`) | Al importar **no se evalúa nada**; al exportar se neutraliza con `'` delante de `= + - @` | ✅ |
| **Un CSV disfrazado de XLSX** | Se detecta y se dice | ✅ |
| **El mismo móvil escrito de dos formas** (`+52 1 81…` y `+52 81…`) | Se normaliza al guardar. Si no, la deduplicación y la supresión dejarían de funcionar **sin que nada fallara** | ✅ |
| **Alguien ejerce su derecho de cancelación** | El contacto se vacía y **el intento de envío se conserva**: `ON DELETE SET NULL`, no `CASCADE`. Borrar la prueba de que se envió sería lo peor de los dos mundos | ✅ |
| **`consent_entry` guarda la dirección en claro** y es de sólo-añadir | Contradice el derecho de cancelación. **Abierto: es P-17**, esperando la revisión jurídica | 🟡 abierto |
| **La base crece sin techo** | D-7C: recorte bimestral de lo que es ruido; los envíos no se borran jamás | 🔨 7.4 |

---

## 6 · «Las personas» · lo que puede costar caro

| Qué | Qué hace ARLES | Estado |
|---|---|---|
| **Se escribe a quien no dio permiso** | En correo se puede y es decisión del usuario; en WhatsApp **exige aceptar el riesgo por escrito**, con la constancia en la bitácora | ✅ |
| **Se quema el dominio** por mandar demasiado | Límites por cuenta, ventana horaria, y el aviso de más de 50 diarios | ✅ parcial |
| **Se quema y nadie lo ve venir** | La proporción de rechazos por cuenta y semana avisaría antes. **Propuesta P-C**: no se construye ahora, pero el dato se guarda desde la 7.1 porque después es imposible reconstruirlo | 📋 dato desde 7.1 |
| **Se promete «entregado»** y no llegó | La interfaz **no dice esa palabra** (§65), y hay una comprobación que falla si aparece | ✅ |
| **Una reclamación pregunta qué se envió y cuándo** | `audit_log` es append-only **por disparador de SQLite**, no por disciplina: ni un `UPDATE` ni un `DELETE` pasan | ✅ |

---

## 7 · Lo que decidimos NO resolver

Está aquí para que sea una decisión y no un olvido.

| Qué | Por qué no |
|---|---|
| **Aperturas y clics** | Requieren rastrear personas. ARLES no rastrea a nadie: es el producto, no una carencia |
| **Rebotes asíncronos** | Requieren leer un buzón. v1.3 (ADR-0009), declarado en la interfaz |
| **Reenviar automáticamente un `presumed_sent`** | Un duplicado es irreversible y quema reputación; un no-envío se repite a mano |
| **Que ARLES conteste los mensajes** | Decisión de Dirección (30/09/2026): eso es trabajo del asesor de ventas |
| **Priorizar unas campañas sobre otras** cuando comparten cuenta | No está en la v1.2.0. Queda escrito en el plan de la Fase 7 §6 bis para el día que se pida |

---

## 8 · Cómo se mantiene vivo

Un catálogo que envejece es peor que no tenerlo, porque se cree.

1. **Cada fase añade su sección** al cerrarse, con lo que encontró.
2. **Cada 🟡 y cada 📋 llevan dueño y fase.** Un pendiente sin fecha no es un
   pendiente: es un deseo.
3. **Cada ✅ apunta a una prueba que se ha visto fallar.** Sin eso es una
   opinión (§2).
4. **La auditoría de fin de fase lo relee.** El §4.3 explica por qué: lo que
   encontró el defecto no fue construir, fue volver a mirar.
