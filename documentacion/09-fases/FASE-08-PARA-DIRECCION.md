# Fase 8 cerrada · Respaldos, y la copia de seguridad que no abría

**Para Adrián Paz · 7 de octubre de 2026**

---

## Lo que ya se puede hacer

**ARLES ya sabe guardar y recuperar todo lo que tiene.** Antes, si se perdía el
equipo, se perdía el trabajo.

En Ajustes hay tres pantallas nuevas:

- **Respaldos** · Crea un archivo con todos tus datos, cifrado con una
  contraseña que eliges tú. Puedes llevarte lo esencial —contactos, permisos,
  bajas, campañas y el último año de envíos— o absolutamente todo. Antes de
  crearlo te dice **cuántas filas y de cuántas tablas** se va a llevar.

- **Restaurar** · Eliges un archivo y, **antes de tocar nada**, ARLES te dice de
  cuándo es, qué trae y —esto es lo importante— **qué se va a perder**. Sólo
  entonces hay un botón.

- **Archivar historial** · Saca los envíos viejos a un archivo que tú guardas
  donde quieras, y sólo después los borra de aquí. Nunca se borra nada sin haber
  **vuelto a abrir el archivo y comprobado que se lee**.

Y una cuarta, que esperemos que no veas nunca: **Recuperar**. Es la pantalla que
aparece el día que Windows pierde la llave que abre tu base. Hasta ahora ARLES
te decía *«restaura tu último respaldo»*… y se cerraba antes de dejarte hacerlo.

---

## Lo que te quiero contar bien: la copia de seguridad no abría

Esto es lo más importante de la fase, y es incómodo.

Al terminar todo —con las diez verificaciones del proyecto en verde y más de mil
pruebas pasando— **fui a medir cuánto tardaba un respaldo**. Nadie lo había
cronometrado nunca. Puse treinta mil contactos de prueba, creé el respaldo, y al
restaurarlo ARLES dijo que **el archivo estaba estropeado**.

No lo estaba.

### Qué pasaba

El archivo se cifra en pedazos. Al abrirlo, cada pedazo se convertía a texto por
su cuenta — y en español una **Á**, una **ñ** o una **é** ocupan dos espacios en
lugar de uno. Cuando el corte entre dos pedazos caía justo en la mitad de una
letra con acento, ARLES ya no entendía ese pedazo y daba el archivo por
corrupto.

Con treinta mil contactos hay unos ciento treinta cortes. Que ninguno caiga mal
es prácticamente imposible. **Es decir: cualquier respaldo de una base real no
se podía restaurar.**

Y la forma que tenía es la peor posible:

- el respaldo se creaba bien,
- la pantalla lo leía bien —te decía la fecha, las filas, lo que se perdía—,
- y al pulsar «Restaurar y reemplazar», fallaba.

Una copia de seguridad que existe, que parece correcta, y que no sirve el día
que la necesitas.

### Por qué ninguna prueba lo vio

Porque **todas las pruebas usaban dos contactos**. Dos contactos caben en un
solo pedazo, así que no hay ningún corte. El único pedazo que se probaba era el
único que no puede fallar.

Está arreglado, y ahora hay una prueba con catorce mil filas llenas de acentos
que **comprueba que de verdad haya quince cortes** antes de dar nada por bueno.
Mi primera versión de esa prueba usaba mil doscientas filas y **pasaba por
suerte**: la rompí a propósito y seguía en verde. Esa también está corregida.

> **La lección, en una frase:** una prueba de ida y vuelta con dos filas no
> prueba una ida y vuelta.

---

## Otras tres cosas que encontré y arreglé

### 1 · La recuperación podía dejarte sin salida por un error de tecleo

La pantalla de Recuperar hacía las cosas en este orden: apartaba tu base,
**creaba una llave nueva**, y *después* leía el respaldo. Si te equivocabas
escribiendo la contraseña del respaldo, la llave nueva ya estaba puesta — y la
vieja, que quizá habría vuelto sola, ya no servía de nada.

Ahora se comprueba **todo lo que se puede comprobar antes de tocar nada**: que
el archivo sea un respaldo y no un archivo de historial, que sea de esta versión
de ARLES, y que la contraseña abra. Sólo entonces se toca algo.

### 2 · La pantalla te decía que sí a un archivo que no servía

Si por error elegías el archivo de historial del disco externo en vez del
respaldo, la pantalla decía **que sí se podía restaurar**. Luego fallaba. Ahora
lo dice antes.

### 3 · Dos pantallas no empezaban igual que su vecina

Restaurar y Archivar tenían su texto de presentación a 54 píxeles del título,
donde Respaldos —que está justo encima en el mismo menú— lo tiene a 8. Cada
pantalla, por separado, se ve bien; **sólo se nota al pasar de una a otra**. Y
la pantalla de Recuperar decía «ARLES RELAY» a secas, sin el «I», siendo la
única de las dieciocho que lo hacía.

Las dos salieron de **abrir la demostración y mirarla**, con todo en verde. Las
dos las vigila ahora una comprobación automática.

---

## Las cifras, con su fecha

Medido el 6 y 7 de octubre con quinientos mil contactos de prueba:

| | Resultado |
|---|---|
| Crear un respaldo cifrado | **menos de un cuarto de segundo** con 30 000 contactos |
| Restaurarlo en otro equipo | **menos de un segundo** |
| Ver qué trae un respaldo | **inmediato** |
| Abrir la lista de contactos | **357 ms** — era 1 424 antes de esta fase |
| Buscar por correo | **menos de 1 ms** — eran 331 |
| Importar 500 000 contactos | **91 segundos**, de cinco minutos de presupuesto |

**Dos números siguen en rojo y te los repito porque uno necesita decisión tuya:**

- **Activar una campaña grande tarda 73 segundos** y la ventana se queda
  congelada. Empieza a notarse hacia los 30 000 destinatarios, que es una lista
  normal. Lo que hice en esta fase fue **evitar el daño**: ARLES ahora te dice
  cuánto va a tardar antes de empezar, y **se niega a activar una campaña que no
  pueda terminar nunca** (con medio millón de personas a 50 correos diarios,
  386 046 no recibirían nada). Arreglarlo del todo —que la campaña se prepare en
  segundo plano, con barra de avance y botón de cancelar— es trabajo de la
  Fase 9.
- **Buscar texto en el registro tarda 3.7 segundos.** Declarado, pendiente.

---

## Lo que ARLES todavía no hace

- **No restaura un respaldo hecho con una versión anterior del programa.** Te lo
  dice con un mensaje claro. Hoy no existe ninguno fuera de las pruebas, así que
  no cuesta nada; dentro de un mes de uso sí costaría.
- **No hace respaldos solo.** Los pides tú y eliges dónde se guardan. Es a
  propósito: un respaldo automático acaba guardándose en el mismo equipo que se
  va a perder.
- **Si olvidas la contraseña de un respaldo, ese archivo no se abre.** Nunca. El
  cifrado no tiene puerta de atrás, y la pantalla te lo dice con esas palabras
  para que no pierdas horas buscando algo que no existe.
- **Nada de esto se ha probado en un Mac** (sigue pendiente R-07, necesita que
  alguien con un Mac lo abra).

---

## Las cinco revisiones completas

Esta fase incluía las cuatro auditorías que el proyecto tiene previstas desde el
principio —arquitectura, seguridad, calidad y accesibilidad—, y les añadí una
quinta: revisar la fase entera como un conjunto.

**Veintidós problemas encontrados. Veinte arreglados, dos declarados con su
cifra.**

Y un dato que creo que te interesa más que los veintidós: **once de ellos no
eran fallos del programa, sino de las comprobaciones que lo vigilan.** Listas
escritas a mano que se habían quedado cortas, medidas que decían cubrir una
regla y cubrían la mitad. El producto aguantó mejor que sus instrumentos — y eso
sólo se descubre mirándolos a ellos.

---

## Lo que necesito de ti

**Nada bloqueante para seguir construyendo.** Pero cuatro cosas siguen esperando
a alguien de fuera y **las cuatro impiden entregar la versión 1.2.0**:

| | Quién | Qué bloquea |
|---|---|---|
| **Licencia de la tipografía Mont** | Tú | No se puede distribuir el instalador con la tipografía dentro |
| **Revisión jurídica** | Un abogado | Los dieciocho textos legales están preparados y marcados como provisionales |
| **Un servidor de correo real** | Tú | Todo lo de envío está probado contra un servidor simulado |
| **Alguien con un Mac** | Quien lo tenga | Nadie ha abierto ARLES en macOS |

**Y una decisión, cuando quieras:** ¿sigo con la Fase 9 —sacar la activación de
campañas a segundo plano, el filtro del registro y SPF/DKIM/DMARC— o prefieres
que antes montemos una demostración real con tu correo y tu WhatsApp? La guía
para eso ya está escrita.
