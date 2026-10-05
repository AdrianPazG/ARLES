# Fase 8 · Respaldos — qué es, y tres cosas que tienes que decidir

**Para Adrián Paz · 5 de octubre de 2026**

---

## De qué va esta fase, en dos frases

Las siete fases anteriores construyeron un producto que **acumula cosas que no
se pueden volver a conseguir**: los contactos que el cliente importó, el permiso
de cada uno, la lista de quién pidió no recibir, y el registro de todo lo que se
envió.

La Fase 8 contesta **«¿y si se pierde?»** — y comprueba, midiendo, que lo
construido aguanta el tamaño que prometimos.

---

## Lo primero: encontré algo que conviene que sepas

**Hoy, si se pierde la llave, los datos se pierden para siempre.**

La base de datos de ARLES está cifrada, y la llave vive en el almacén de
contraseñas de Windows o del Mac. Eso está bien hecho: si alguien copia el
archivo, no puede leerlo.

Lo que pasa es que **no hay una segunda copia de esa llave**. Si el almacén del
sistema se pierde —un formateo, un perfil de usuario corrupto, un equipo que
muere— el archivo con todos los datos sigue ahí y es **ilegible para siempre**.
No hay forma de recuperarlo. Ninguna.

La salida a eso es un respaldo. Y el respaldo es justo lo que esta fase
construye. Por eso va primero, antes que las auditorías: no es una comodidad, es
el único seguro que tiene el producto.

---

## Las tres decisiones

Ninguna frena arrancar: puedo empezar por la primera entrega sin respuesta a
ninguna. La primera decisión sí frena la tercera entrega.

---

### 1 · ¿El archivo de respaldo va con contraseña?

**Lo que hay decidido hoy.** En la Fase 0 se decidió que el respaldo de la v1.2.0
**no iría cifrado**, con un aviso al usuario, y que el cifrado entraría en la
v1.3. Está escrito en tres documentos.

**Por qué lo vuelvo a plantear.** Esa decisión se tomó cuando el producto no
existía. Hoy las dos frases que lo describen, juntas, dicen esto:

> *El respaldo es el único camino de recuperación si se pierde la llave.*
> *El respaldo no está cifrado y contiene datos personales.*

Es decir: el archivo que el usuario tiene **más incentivo en copiar a muchos
sitios** —porque es su único seguro— es también el que lleva dentro la base de
clientes entera, en claro. Acaba en Drive, en el correo, en una USB.

**El riesgo no es nuestro, y eso es lo que más pesa.** Si ese archivo se filtra,
los datos expuestos son los de los clientes de nuestro cliente — gente que no
eligió nada. Bajo la ley mexicana de datos personales, eso es una vulneración
que hay que notificar, y el aviso que pusimos en la pantalla no la evita:
documenta que lo sabíamos.

**Lo que cuesta cifrarlo:** una entrega. La pieza criptográfica ya está en el
proyecto, por la base de datos. No es trabajo nuevo de investigación.

**El coste honesto de cifrarlo:** una contraseña que el usuario olvida convierte
el respaldo en inservible — y el respaldo se usa justo cuando ya se perdió algo.
Se cambia un riesgo por otro.

**Mi recomendación: cifrarlo, pero dejando desactivarlo a propósito**, en el
mismo diálogo y con el aviso delante. Así, quien decide correr el riesgo de
olvidar la contraseña es quien exporta, en ese momento, sabiéndolo. El riesgo de
la fuga, en cambio, lo corren terceros que no eligieron nada. **Entre un riesgo
que alguien elige y uno que nadie eligió, protejo el segundo.**

Si prefieres mantener lo decidido, se construye sin cifrar, el aviso pasa a ser
obligatorio y no se puede omitir, y lo anoto como riesgo aceptado con fecha.

---

### 2 · ¿Qué se lleva el respaldo: todo, o lo esencial?

**El problema.** El registro de envíos no se borra nunca — lo decidiste tú en la
7.4, y está bien decidido: es la prueba de a quién se escribió. Pero crece. En
una instalación de dos años mandando a 50 000 contactos al mes son **millones de
renglones**.

Un respaldo que se lo lleve todo puede tardar veinte minutos y pesar un par de
gigas. **Y un respaldo que tarda veinte minutos no se hace nunca.**

**Mi recomendación: dos opciones en el mismo botón**, con las cifras delante
antes de elegir:

- **Completo** — todo, incluido el historial de envíos. Para cambiar de equipo.
- **Esencial** (por defecto) — contactos, permisos, **bajas**, plantillas,
  campañas y configuración; del historial, sólo lo definitivo del último año.
  Para el respaldo del día a día.

En los dos casos, **las bajas van completas y nunca se recortan**. Perder la
lista de quién pidió no recibir significa volver a escribirle, y eso es lo peor
que este producto puede hacer.

---

### 3 · ¿Qué pasa si se restaura encima de datos que ya están?

**El caso normal** es un equipo nuevo y vacío. El que rompe cosas es restaurar
sobre una instalación que ya tiene información.

Hay dos formas de hacerlo y una es peligrosa:

- **Mezclar** duplica contactos y, mucho peor, **puede resucitar una baja**:
  alguien que pidió no recibir vuelve a la lista porque el respaldo es anterior a
  su petición. Le volveríamos a escribir.
- **Reemplazar** destruye lo que había y no estaba en el respaldo.

**Mi recomendación: reemplazar, nunca mezclar** — pero con el mismo mecanismo que
ya usa el recorte del registro: **antes de tocar nada**, una pantalla que dice
con cifras qué hay ahora, qué trae el respaldo y qué se va a perder. Y una regla
que no se negocia: **las bajas se suman, no se reemplazan**. La lista final es
la unión de las dos, porque una baja no se deshace sola.

---

## Lo demás de la fase, sin decisiones que tomar

**Medir de verdad el rendimiento.** Hay trece promesas escritas —«abrir 500 000
contactos en menos de medio segundo», «importar 100 000 filas en menos de un
minuto»— y **ninguna se ha medido nunca**. Son estimaciones de la Fase 0. El paso
que iba a medirlas lleva marcado «pendiente» en el sistema de compilación desde
la Fase 4.

Voy a generar medio millón de contactos de prueba y medir las trece. **Y espero
que algo falle**: que salieran las trece a la primera sería señal de que la
medición no está midiendo.

**Las cuatro auditorías.** Arquitectura, seguridad, calidad/rendimiento y
accesibilidad, en cuatro pasadas separadas — no una revisión general. Es lo que
el brief pide, y tiene motivo: una revisión general encuentra lo que salta a la
vista; cuatro pasadas con una sola pregunta cada una encuentran lo que no. La
Fase 9 —la del instalador— no debería empezar sin esto.

**Una corrección:** el plan original ponía los derechos ARCO en esta fase. **Ya
están construidos desde la 3.4.** No es trabajo perdido; la auditoría de
seguridad vuelve a pasar por encima, que es donde tocaba verificarlos. Corrijo el
plan al cerrar.

---

## Lo que sigue sin poder probarse desde aquí

Lo digo porque va a aparecer en el informe de cierre y prefiero que no te
sorprenda:

- **El camino de «se perdió la llave» sólo lo puedo probar en Linux**, donde el
  almacén se simula. En Windows y Mac se prueba cuando se cierre **R-07** — la
  revisión en un Mac, que sigue esperando.
- **Las medidas de rendimiento no salen de la máquina de referencia** que
  definimos (4 núcleos, 8 GB). Lo que se mida aquí vale como línea base y como
  detector de empeoramiento, no como la cifra final. Lo diré así, y no «cumple».
- **R-24** sigue siendo tuya, y sigue siendo lo que más valor tiene de todo lo
  pendiente.

---

## Cómo queda el orden

| | Entrega | Necesita de ti |
|---|---|---|
| **8.0** | La lista de qué va en un respaldo, y la comprobación que impide olvidar una tabla | — **puedo empezar hoy** |
| **8.1** | El formato del archivo | — |
| **8.2** | Exportar | **Decisiones 1 y 2** |
| **8.3** | Restaurar, con el aviso previo | **Decisión 3** |
| **8.4** | Qué hacer cuando se perdió la llave | — |
| **8.5** | Medio millón de contactos y trece cifras medidas | — |
| **8.6** | Las cuatro auditorías | — |
| **8.7** | Cierre: documento, informe y mirar las pantallas | — |

**La 8.0 no depende de ninguna decisión, así que puedo arrancar en cuanto me
digas.** Es, además, la entrega que más valor tiene a largo plazo: evita que
dentro de un año alguien añada una tabla nueva, se olvide de meterla en el
respaldo, y los datos de esa tabla desaparezcan **sin que nada dé error**.

El detalle técnico completo está en
[`FASE-08-PLAN.md`](FASE-08-PLAN.md).
