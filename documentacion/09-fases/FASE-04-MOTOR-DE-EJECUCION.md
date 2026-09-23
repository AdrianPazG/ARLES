# Fase 4 · El motor de ejecución

**Estado:** ✅ cerrada — las seis entregas · **Fecha:** 2026-09-23
**Validación:** 36/36 comprobaciones de la fase 4, **0 omitidas** — `validar.py --fase 4`
**Pruebas:** 595 de Rust · 102 de frontend
**Auditoría de la fase:** 23/09/2026 — cinco hallazgos, todos corregidos (§7)

> **Objetivo de la fase.** Que ARLES sepa **mandar**: con qué ritmo, en qué
> horario, qué hacer cuando algo falla, y sobre todo qué **no** hacer nunca —
> mandarle a alguien el mismo correo dos veces.

**Se construyó entera contra un proveedor simulado.** Al cerrarla, ARLES sabe
enviar perfectamente y no ha mandado ni un solo correo real. Eso es deliberado:
las decisiones más delicadas del motor son todas respuestas a un fallo, y no se
le puede pedir a Gmail que se caiga a voluntad.

Seis entregas, cada una con su puerta de salida:

| | Qué | Puerta de salida | Estado |
|---|---|---|---|
| **4.1** | El reloj y las reglas | Cruzar el cambio de horario sin hueco ni doble envío | ✅ §1 |
| **4.2** ⭐ | La cola que no duplica | **Matar el proceso a mitad de un envío y que nadie reciba dos veces** | ✅ §2 |
| **4.3** | El proveedor simulado y el bucle | Los casos de red y credenciales de `ESTRATEGIA_QA.md` §3 | ✅ §3 |
| **4.4** | Pausar, reanudar, detener | Parar con un envío en curso y que no salga ni uno más | ✅ §4 |
| **4.5** | Duración y preflight | Los ocho puntos, con errores accionables (§95) | ✅ §5 |
| **4.6** | Cierre: auditoría, documentos e informe | `--fase 4` en verde, 0 omitidas | ✅ §7 y este documento |

---

## 1. El reloj y las reglas (4.1)

Lo que no toca base de datos ni red: reglas puras en `arles-core`, donde se
pueden probar sin montar nada.

| Módulo | Qué decide |
|---|---|
| `attempt` | Los nueve estados de un envío y qué transición existe |
| `ritmo` | Cuántos caben por hora y por día, y si el equipo estuvo suspendido |
| `ventana` | En qué días y horas puede enviar esta campaña, **en su zona** |
| `reintento` | Qué hacer ante cada fallo, y cuánto esperar (D-12) |

**El horario es de la campaña, no del equipo** (D-10). Una campaña a clientes de
Madrid se envía en horario de Madrid aunque quien la lance esté en Monterrey, y
sigue siendo horario de Madrid si esa persona se lleva el portátil a Tokio.

**Una biblioteca que mentía sobre México.** `time-tz` era la elección natural
—extiende la biblioteca de tiempo que el proyecto ya usa—, y su base de zonas
horarias **sigue aplicando horario de verano a México**, abolido en octubre de
2022. Medido: daba UTC-5 en junio de 2026, cuando es UTC-6 todo el año. Con esa
base, **todas las campañas mexicanas habrían enviado una hora fuera de su
horario durante medio año**, en silencio. Se cambió a `jiff`, contenida en un
solo módulo.

**Los reintentos son tres: a los 5, 15 y 30 minutos** (D-12, decisión de
Dirección). El rango original —7,5 minutos de margen total— no cubría un apagón
ni una incidencia de proveedor; el actual da cincuenta minutos.

---

## 2. La cola que no duplica (4.2)

**Es la entrega que concentraba el riesgo de la fase.** Un duplicado es el peor
fallo posible de ARLES: es invisible para nosotros, irreversible para quien lo
recibe, y acumulativo para el dominio.

### Tres capas, y ninguna sobra

**1 · El esquema.** `UNIQUE(campaign_id, channel, contact_address)` hace que dos
intentos para la misma persona sean **imposibles**, no improbables. Es la capa
que sigue funcionando cuando el código no.

La clave es la **dirección**, no el contacto: alguien borrado y vuelto a
importar tiene identificador nuevo, y una clave por identificador le permitiría
recibir otra vez.

**2 · El compare-and-swap.** Tomar un trabajo es una sola operación que
comprueba y escribe a la vez, y mira cuántas filas cambiaron. Si cambiaron cero,
otro trabajador ganó la carrera.

**3 · La clave de idempotencia**, que viaja como `Message-Id` del correo y se
persiste **antes** de llamar al proveedor.

### El caso ambiguo, que es el que nadie documenta

```
1. UPDATE state = 'sending'   <- queda escrito
2. proveedor.enviar(...)      <- el proveedor ACEPTA
3. X se va la luz aqui
4. UPDATE state = 'sent'      <- nunca ocurre
```

Al reabrir, esa fila dice `sending`, y **el correo pudo salir o no**. Averiguarlo
exigiría leer la bandeja de enviados, un permiso que la v1.2.0 no pide.

| Si eligiéramos… | Y el correo sí salió | Y no salió |
|---|---|---|
| **Reenviar** | **Duplicado.** Irreversible, invisible | Correcto |
| **No reenviar** | Correcto | Alguien no recibió. **Visible y recuperable** |

Se falla del lado recuperable (ADR-0004): esas filas pasan a «envío no
confirmado» y **nunca se reenvían solas**.

### El presupuesto sobrevive a cerrar el programa

El cupo por hora y por día se guarda en la base con la etiqueta de su ventana
(`2026-09-23T14`). Un cupo en memoria se reiniciaría al abrir ARLES y dejaría
enviar el límite diario varias veces al día.

---

## 3. El proveedor simulado y el bucle (4.3)

**Siete formas de fallar a la orden**: se cayó la red, tarda demasiado, «espera
diez minutos», esa dirección no existe, la contraseña no sirve, contestar
basura, y «esa persona pidió la baja».

**Y deja constancia de lo que le pidieron.** Eso permite afirmar algo que de
otro modo habría que suponer: que a una persona dada de baja **no se la llamó ni
una vez**. Mirar la base diría «no consta un segundo envío»; mirar al proveedor
dice «no se pidió», que es lo que importa — lo que llega al proveedor ya no se
puede retirar.

### El bucle, paso por paso (§13)

Por cada correo, en este orden:

1. ¿Alguien pulsó parar? Se para.
2. ¿Estamos en el horario de la campaña? Si no, se espera **sin tocar la base**.
3. ¿El disyuntor de la cuenta está abierto? Si sí, se espera.
4. ¿Queda cupo esta hora y este día? Si no, se pausa y se avisa. **Sin buscar
   otra cuenta** (T-1).
5. Se toma un envío de la cola, y sólo uno.
6. **¿Esta persona pidió la baja?** Se pregunta ahora, no al preparar la campaña.
7. **¿Alguien pulsó parar, otra vez?**
8. Se anota «enviando» **antes** de llamar a nadie.
9. Se envía, y se anota qué pasó.

El punto 6 importa más de lo que parece: a cincuenta correos diarios una campaña
de cinco mil dura cuatro meses, y quien pide la baja en octubre no puede seguir
recibiendo en enero.

El punto 7 parece redundante y no lo es: entre el 1 y el 7 hay dos consultas a
la base, tiempo de sobra para pulsar el botón. Y el botón significa «ningún
correo más», no «uno más y paro».

### Un solo ARLES sobre una sola base

Dos ARLES no producen duplicados —lo impide el esquema—, pero el segundo, al
arrancar, marcaría como «envío no confirmado» los envíos que el primero está
haciendo bien en ese momento. El usuario acabaría revisando una lista larga para
nada.

El cerrojo es un archivo con **latido**, reescrito por el propio bucle. Sin
latido, una caída deja el archivo puesto y **nadie puede volver a abrir el
programa** — un fallo peor que el que se evita.

---

## 4. Pausar, reanudar, detener (4.4)

| Acción | Qué le pasa a la cola | ¿Se deshace? |
|---|---|---|
| **Pausar** | Se conserva **intacta** | Sí |
| **Reanudar** | Sigue donde estaba | — |
| **Detener** | Lo que no salió se cancela | **No** |

Detener pide confirmación **con la cifra**: «esto cancela 4 312 correos que no
han salido». Y hay una prueba que comprueba que **la cifra avisada es
exactamente la que se cancela** — si miraran listas distintas, la pantalla
prometería una cosa y pasaría otra sobre algo irreversible.

**Lo que detener no toca:** los correos que ya salieron, y los que están saliendo
en ese instante. Cancelar una fila que ya se le pasó al proveedor diría que no
salió, y puede que sí.

**La parada de emergencia se guarda en la base** (migración V6). El caso: alguien
ve que la campaña recién activada lleva la plantilla equivocada, pulsa parar y
cierra el programa. En memoria, al día siguiente ARLES arrancaría sin parada.

Se guarda **desde cuándo**, no un sí/no. Y **levantarla no reanuda nada**: se
pulsa porque algo iba mal, y lo último que debe hacer el botón de quitarla es
poner a enviar todo a la vez.

---

## 5. Duración y preflight (4.5)

### «Terminará alrededor del 9 de febrero»

Es probablemente la información más valiosa del producto: es lo que hace que
alguien reconsidere su configuración **antes** de lanzar.

**Y se dice cuál de los dos topes frena.** «Tarda cuatro meses» deja sin saber
qué tocar; «tarda cuatro meses **porque tu tope diario es 50**, cuando tu horario
daría para 90» dice dónde está el freno. Sin eso, lo normal es ampliar el
horario y no notar ninguna diferencia.

**Dos respuestas que no son una fecha**, y las dos valen más que la fecha:
«con este tope no saldría ni un mensaje» —la campaña quedaría marcada como
enviando sin enviar nada y sin nada roto que mirar— y «no termina en cinco años:
faltarían 435 000».

Se **simula hora a hora** en vez de dividir, y por eso acierta en tres cosas que
una división se salta: que hay dos topes y manda el más pequeño; que el día en
que se activa casi nunca está entero; y que los días del cambio de horario duran
veintitrés o veinticinco horas.

### Las ocho comprobaciones, con una tercera respuesta

Cada punto sale «bien», «hay que arreglarlo» o **«sin comprobar»** — y sin
comprobar **no es un aprobado**. Varias comprobaciones necesitan piezas que
llegan después (los proveedores son de la Fase 5, las plantillas de la Fase 6).

Es la misma regla que aplicamos a las pruebas de navegador: **omitida no es
pasada, y se dice**. Un punto sin comprobar impide activar igual que uno que
falla, y las cifras se cuentan aparte: uno se arregla, el otro se comprueba.

**Siete de los ocho bloquean.** El de SPF/DKIM/DMARC avisa y no cierra la puerta:
entrega peor, pero entrega, y bloquear dejaría fuera a quien no administra su
propio DNS por algo que ARLES no le puede arreglar (§154).

---

## 6. Los casos frontera de `ESTRATEGIA_QA.md` §3

La puerta de salida de la fase. Cada uno tiene su prueba con nombre.

| Caso | Qué debe pasar | Cubierto |
|---|---|---|
| El equipo se suspende y despierta al día siguiente | **No hay ráfaga** | ✅ §7-A |
| La app muere entre el `accept` y el commit | `presumed_sent`, jamás un duplicado | ✅ |
| Dos instancias de ARLES a la vez | La segunda avisa; no corrompe nada | ✅ |
| Reinicio a mitad de envío | La cola se reanuda donde estaba | ✅ |
| Parada de emergencia con envío en curso | Ninguno nuevo sale. **Antes de cada envío** | ✅ |
| Límite alcanzado a mitad de campaña | Pausa con aviso. **Sin rotación** | ✅ |
| Contacto suprimido mientras la campaña corre | No recibe. Comprobación **al enviar** | ✅ |
| Cambio de horario dentro de la ventana | Sin doble envío ni hueco | ✅ |
| Campaña que cruza medianoche | El cupo diario cambia a la hora de la empresa | ✅ |
| El portátil viaja a otro huso | La política no cambia | ✅ |
| **Reloj del sistema ajustado hacia atrás** | No se duplican permisos | ✅ §7-B |
| Conexión perdida a mitad de envío | Reintento con backoff | ✅ |
| Token revocado / contraseña caducada | **Circuito abierto, `needs_reauth`** | ✅ §7-C |
| 429 con `Retry-After` | Se respeta **literalmente** | ✅ |
| 5 000 caracteres de basura | Se trunca, no se interpola, no se cae | ✅ |

**Lo que queda para la Fase 5** y se dice: el caso «el servidor tarda diez
minutos» necesita un tiempo de espera real contra una red real. El simulador lo
representa, pero el tiempo de espera lo pone el adaptador.

---

## 7. La auditoría de la fase (23/09/2026)

Antes de cerrar se recorrió la fase entera contra sus requisitos. **Cinco
hallazgos, ninguno menor.** Tres eran fallos, dos eran incoherencias de cara al
usuario.

### A · La protección contra la ráfaga hacía justo lo que quería impedir

`Despertar::presupuesto` devolvía el presupuesto a cero al detectar una
suspensión. Tenía su prueba, se llamaba `ocho_horas_dormido_no_producen_una_rafaga`,
y **pasaba**.

El caso que la tumba es una **siesta dentro del mismo día**: el equipo duerme de
10:00 a 18:00 con el cupo diario agotado, despierta, el contador vuelve a cero,
y esa cuenta manda otros cincuenta. **Cien envíos en un día con un tope de
cincuenta.**

El fallo era arquitectónico. Esa regla se escribió para un cupo **en memoria**,
donde hay que decidir qué hacer con lo gastado al despertar. En la 4.2b se
sustituyó por el cupo **persistido por etiquetas**, donde lo gastado no se
arrastra: se consulta por la etiqueta de la hora y del día actuales. El problema
se disolvió y **la regla se quedó, apuntando**.

La prueba estaba escrita para confirmar el comportamiento, no para cazarlo — y
su nombre decía lo contrario de lo que medía. Es el caso más puro de la regla de
trabajo de este proyecto: *una comprobación que nunca se ha visto fallar no está
verificada*.

### B · Atrasar el reloj regalaba el cupo diario

Bastaba con mover el reloj un día hacia atrás para que la etiqueta cambiara, el
contador se reiniciara y la cuenta mandara el cupo entero otra vez. **Mover el
reloj era una forma de saltarse el límite del proveedor**, y no hace falta mala
fe: un ajuste de NTP tras una deriva hace exactamente eso.

Era el caso «reloj del sistema ajustado hacia atrás» de `ESTRATEGIA_QA.md` §3:
estaba enunciado y sin cubrir. Ahora una etiqueta más vieja no reinicia nada.

### C · El disyuntor del §7 no existía

Las columnas `consecutive_failures` y `circuit_open_until` estaban en el esquema
desde la V1, el §13 listaba la comprobación como un paso del bucle, y **ningún
código las leía ni las escribía**. Un esquema que promete algo y un código que no
lo cumple no falla: simplemente no protege de nada.

Se construyó, y la parte delicada es a quién se le echa la culpa:

| Fallo | ¿Cuenta para el disyuntor? |
|---|---|
| Red caída, tiempo de espera, servidor saturado | **Sí** — habla del canal |
| La credencial no sirve | **Sí, y abre a la primera** |
| Esa dirección no existe | No — habla del destinatario |
| Esa persona pidió la baja | No — ni siquiera es un fallo |

Si los rebotes contaran, una lista importada con direcciones viejas pararía la
campaña cada pocos minutos sin que el usuario entendiera por qué.

### D · El mismo suceso se contaba de dos maneras

Había una clave de texto propia del motor para cada paso del bucle, y eso creaba
**dos textos para el mismo hecho**: la pantalla de Actividad habría dicho «Se
volverá a intentar» y el detalle del mismo envío, a un clic de distancia,
«Reintento 2 de 3». Una de las dos sin la cifra que exige el §94.

Ahora cada paso apunta al texto de quien decide ese suceso, y hay una prueba que
impide que la duplicación vuelva.

### E · El preflight ignoraba lo que la base ya sabía

Si el disyuntor había marcado una cuenta como «hay que renovar la credencial»,
el preflight seguía diciendo «sin comprobar» — cuando la respuesta estaba escrita
en la base. Le habría hecho buscar el problema en otra parte.

El disyuntor y el preflight se construyeron en entregas distintas y no se
hablaban. Ahora sí.

---

## 8. Lo que esta fase deja listo para la Fase 5

El motor está entero y probado contra un simulador. La Fase 5 sustituye el
simulador por proveedores reales, y **no tiene que tocar nada de lo anterior**:
sólo implementar `Proveedor::enviar` para SMTP y traducir los códigos de cada
proveedor a los cinco fallos que el núcleo entiende.

**Lo que sigue bloqueado por fuera** y no lo desbloquea esta fase: la licencia
de la tipografía (P-01), la revisión jurídica (P-09), la cuenta de Meta con
Coexistencia (META) y la revisión en macOS (R-07).
