# Auditoría H · El cupo diario, de punta a punta

**22/10/2026 · pedida por Dirección**

> *«Me preocupa que salgan más correos del que se asignó en el día. Eso busquemos
> cómo corregirlo, respetando siempre lo que el cliente coloca como entrada: si
> pone que 50 al día en una campaña y 50 en otra campaña, no pueden salir ni más
> ni menos al día. Revisa cada detalle de este proceso y las posibles fallas que
> puede tener y corrígelo.»*

Esta auditoría recorre **un solo camino** —desde el número que alguien escribe en
el paso 5 hasta el correo que sale— y busca cada forma de que salga una cantidad
distinta. No mide arquitectura, ni seguridad, ni pantallas: mide **si la cifra
que se pide es la que se cumple**.

**Cuatro hallazgos, cuatro corregidos. Seis puntos limpios, y dejarlos escritos
es la mitad del valor** — porque dos de ellos estuve a punto de anotarlos como
defectos antes de leer el código entero, que es el error que las pasadas B y C
del §140 cometieron cada una por las malas.

---

## Resumen

| | Qué | Lado | Estado |
|---|---|---|---|
| **H-1** | La migración V13 **reiniciaba el contador del día** | salen **más** | ✅ corregido |
| **H-2** | El tope por hora **impedía alcanzar el del día**: se piden 500, salen 90 | salen **menos** | ✅ corregido |
| **H-3** | Siete textos y comentarios afirmaban el comportamiento **retirado** esa misma tarde | — | ✅ corregido |
| **H-4** | Código muerto cuyo comentario dice *«el presupuesto se reinicia»* | salen **más**, si alguien lo conecta | ✅ corregido |

---

## H-1 · La migración reiniciaba el contador — **salen más**

**Lo que pasaba.** La primera versión de la V13, escrita esa misma mañana,
vaciaba `rate_budget` al migrar. El motivo estaba escrito y era cierto: los
contadores viejos cuelgan del **buzón** y no se pueden repartir entre las
campañas que lo usan — un contador de 37 envíos de una cuenta que sirve a tres
campañas no sabe cuántos puso cada una.

**El motivo era cierto y la conclusión estaba mal.** Vaciar hace exactamente lo
que Dirección señaló: quien actualiza ARLES a media tarde habiendo gastado parte
del cupo vuelve a cero y ese día manda de más.

**Cómo se corrige sin inventar el reparto.** No hace falta saberlo:

> **cada etapa del buzón hereda el contador entero.**

Si el buzón gastó 37, las tres campañas arrancan con 37 gastados cada una. Es
falso en el reparto y **verdadero en lo que importa**: ninguna puede mandar más
de lo que le queda, y el total del día **nunca sube**. El error cae del lado de
enviar de menos — la misma asimetría de ADR-0004 y la del orden en que el motor
anota el presupuesto antes del estado.

El día de la actualización sale **menos** de lo previsto, y se recoloca solo a
la mañana siguiente cuando la etiqueta del día cambia.

**La migración se corrigió en vez de parchearse con una V14** porque ninguna
base la tenía aplicada: se escribió y se corrigió el mismo día, sin instalador
publicado en medio. La regla de no editar una migración publicada protege de
tener dos esquemas con el mismo número; aquí no había ninguno.

> **Y el orden de la migración estaba mal, que lo cazó la prueba de la V4.** La
> copia corría con los índices viejos todavía puestos, y el único de
> `(cuenta, ventana)` rechazaba la segunda etapa del mismo buzón con un
> `UNIQUE constraint failed`: **la migración entera fallaba y ARLES no
> arrancaba**. Ahora se suelta lo que estorba, se copia, y los índices nuevos se
> crean al final.

---

## H-2 · El tope por hora dejaba el del día sin cumplir — **salen menos**

**El peor de los cuatro, y el que más probablemente iba a pasar.**

Los dos topes se piden por separado en el paso 5, y el de la hora puede frenar
al del día. Con lo que la pantalla rellenaba sola —el por hora de la cuenta,
diez— y la ventana de nueve a dieciocho, **medido**:

| Lo que se pide | Lo que salía |
|---|---|
| **500 al día** | **90** |
| **100 al día** | **90** |
| 50 al día | 50 ✓ |

O sea que **cualquier tope diario por encima de noventa no se cumplía**. Y no se
veía por una razón incómoda: con los 50 del §48 todo cuadra, así que **el defecto
empezaba justo donde Dirección quiere trabajar**.

La pantalla tampoco lo decía. El paso 6 enseñaba el techo real —90— sin decir de
dónde salía ni que no era el número que se había escrito. Y `quienManda`, que es
el campo del núcleo que dice *cuál de los dos topes es el freno*, **viajaba por
la frontera y no lo pintaba nadie**: el patrón del §9 por enésima vez.

**Se cierra por los dos lados, porque uno solo no basta:**

1. **El por hora se calcula** del tope diario y de las horas de la ventana
   (`por_hora_que_hace_falta` en el núcleo, y su gemelo en el asistente para
   poder rellenar el campo antes de llamar a nadie). 500 en nueve horas son 56
   por hora, y entonces salen los 500.
2. **Si alguien lo baja a mano, se avisa con las dos cifras**: *«Pediste 500 al
   día, pero con 10 por hora y 9 horas de envío sólo caben 90»*, y debajo, cómo
   se arregla. **No se bloquea** (§154): repartir más despacio puede ser lo que
   se quiere; mandar menos de lo pedido sin saberlo nunca lo es.

El aviso va **pegado al campo que lo causa** y no al final del paso, que es la
lección del aviso del remitente compartido: un aviso que desmiente una cifra
tiene que verse con la cifra delante.

> **Redondea hacia arriba, y eso es el detalle que se rompe.** 500 en nueve horas
> son 55.5. Con 55 salen 495 —faltan cinco— y con 56 caben 504, que el tope del
> día corta en 500. Un `/` o un `Math.floor` en cualquiera de los dos lados deja
> el tope sin alcanzar por poco: el mismo defecto en pequeño y más difícil de
> ver. Lo vigila `el_techo_del_dia_se_calcula_igual_en_los_dos_lados` en la
> fase 1, verificada con tres rupturas.

> **Y la prueba que había protegía el defecto.** `dice CUÁL es el máximo diario
> que se va a usar` exigía que la pantalla dijera «Tu cuenta viene con 10 por
> hora» — el número que producía los 90 de 500. Estaba en verde **sobre** el
> defecto. Dejarla habría impedido el arreglo.

---

## H-3 · Siete sitios afirmaban el comportamiento de la mañana

La D-13 se decidió a mediodía: los topes de las campañas que comparten remitente
**se suman**. Antes se repartían. Y siete sitios seguían contando el reparto,
incluido **un aviso que el usuario lee** y **dos pruebas que lo exigían**:

| Dónde | Qué decía |
|---|---|
| `es.ts` · el aviso del paso 6 | *«se reparten ese máximo, así que ésta va a tardar más… No saldrán más correos en total»* |
| `campanas.spec.ts` | exigía la frase `'se reparten ese máximo'` |
| `revision.rs` ×2 | «sacan **25 y 25**, por orden de llegada» |
| `campanas.ts` | lo mismo en el tipo del store |
| `AsistenteDeCampana.vue` | lo mismo en el comentario de la plantilla |
| `cifras.rs` | «el total de una cuenta es el mayor de sus campañas, nunca la suma» |

**Un aviso que describe un comportamiento retirado es peor que ninguno**, porque
hace desconfiar de la cifra que sí es buena. El aviso no se quitó: **cambió de
sentido**. Ahora dice que el plazo sí se cumple, y avisa de lo otro — que de ese
buzón va a salir la suma, y que el total es lo que arriesga el dominio.

Y hace más falta que antes: **antes el agregado tenía un techo y ahora no lo
tiene.**

---

## H-4 · Código muerto que invita a reintroducir el defecto

`arles_core::ritmo::Despertar` y `al_despertar` **no los llama nadie** — medido:
siete apariciones, todas dentro de su propio archivo.

Son un vestigio de un diseño anterior en el que el presupuesto era un cubo de
fichas que se rellenaba con el tiempo. En un cubo así, volver de ocho horas de
suspensión acredita ocho horas de fichas y suelta una ráfaga, y por eso hacía
falta saber si el equipo había dormido.

**El presupuesto de ARLES no es un cubo**: es un contador con la etiqueta de su
ventana. Dormir no cambia la etiqueta del día, así que el contador sobrevive a la
suspensión sin que nadie haga nada.

**Lo que lo hace un hallazgo y no una curiosidad** es su comentario, que decía:
*«El equipo estuvo suspendido. **El presupuesto se reinicia.**»* Quien encuentre
esa función mañana y la conecte creyendo que arregla algo hace que un portátil
abierto por la tarde mande el cupo del día otra vez — exactamente lo que Dirección
teme. El comentario ahora dice por qué no hay que conectarlo.

No se borra el tipo: es API pública de un crate y quitarla es otra decisión. Lo
que se cierra es el camino a usarla por error.

---

## Lo que salió limpio

Seis puntos, y **dos de ellos estuve a punto de anotarlos como defectos**.

### ✅ Dos ARLES abiertos no envían a la vez

Iba a anotarlo: no hay `single-instance` de Tauri en la configuración. **Y existe
algo mejor**: un cerrojo de archivo con latido en el directorio de la base
(`arles-motor/src/instancia.rs`). El motor late en cada vuelta y **se detiene si
el cerrojo ya no es suyo**; un cerrojo caducado se roba, y el robo se confirma
tras una espera para que dos arranques simultáneos no se lo queden los dos.

Es la lección de la pasada B del §140, repetida: **antes de anotar que algo
falta, se busca cómo se llama de verdad.**

### ✅ Si el proceso muere a mitad de un envío, no se duplica

`recuperar_al_arrancar` distingue dos estados y **no los trata igual**:
`claimed` (nadie llamó al proveedor) vuelve a la cola; `sending` (se llamó y no
se sabe si aceptó) pasa a `presumed_sent` y **no se reenvía**. Es ADR-0004, y la
asimetría está razonada: duplicar es irreversible e invisible; no enviar es
visible y recuperable.

### ✅ El presupuesto se anota antes del estado

Si el programa muere entre las dos escrituras, queda un permiso gastado por un
envío que la fila no confirma: **se envía de menos**. Al revés —estado primero—
quedaría un envío sin permiso y la cuenta podría pasarse. La misma asimetría.

### ✅ Mover el reloj hacia atrás no reinicia el contador

`sumar_uno` guarda `MAX(window_start, excluded.window_start)`: una etiqueta más
vieja no hace retroceder la ventana. Sin eso, atrasar el reloj un día reiniciaba
el contador y la cuenta mandaba el cupo otra vez — y no hace falta mala fe, un
ajuste de NTP tras una deriva hace eso.

### ✅ Cada campaña lleva su recuento, y no gasta del de la otra

Medido el 22/10: 300 envíos de una campaña dejan la otra en cero sobre el mismo
buzón. Es lo que hace que «100 de una y 100 de otra» sea verdad y no sólo que el
total sume 200.

### ✅ `cabe_uno_mas` compara bien

`>=` y no `>` —un `>` dejaría pasar uno de más—, y pregunta por el **día antes de
la hora**, para informar del motivo que más va a durar.

---

## Un límite declarado, no corregido: el envío de prueba

**El envío de prueba no gasta cupo**, y no es un olvido: está escrito en su
propia función que no toca nada del aparato de campañas.

**Y no se cambia, con una razón que es la de Dirección.** Si el correo de prueba
gastara del cupo de la campaña, la campaña entregaría **menos** de lo que el
cliente pidió — que es la mitad de lo que esta auditoría viene a impedir. Y hay
una razón de seguridad encima: reutilizar el camino de las campañas traería
consigo la supresión, y un `550` al probar contra la dirección propia **daría de
baja al usuario**, que es irreversible.

Lo que esto cuesta: un envío de prueba es un correo real que sale del buzón y el
proveedor lo cuenta. Son dos o tres, no doscientos. **Queda declarado y medido en
lo que es.**

---

## Lo que esta auditoría no mide

- **El primer día de una campaña sale incompleto, y es físico.** Quien activa a
  las 16:30 con ventana hasta las 18:00 tiene hora y media, no nueve horas. La
  estimación ya lo tiene en cuenta —la hora en curso no cuenta— y no hay nada que
  corregir.
- **Un envío que cruza el cambio de hora se anota en la ventana anterior.** Las
  claves se leen al empezar el paso y se usan al anotar, a propósito: el permiso y
  el gasto son de la misma ventana. El desvío es de un correo.
- **Nada de esto se ha visto en una campaña real de más de un día.** Es el único
  instrumento que mide de verdad un cupo diario, y conviene que la primera sea
  pequeña.
