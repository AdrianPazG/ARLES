# Los roles, y qué mira cada uno

**Última actualización:** 2026-10-27

ARLES lo construye una sola persona —o un solo modelo— haciendo el trabajo de
un equipo. Esta lista no es organigrama: es **una forma de no mirar siempre
desde el mismo sitio**, que es como se llega a un producto que pasa todas sus
pruebas y no funciona.

La prueba de que hace falta: las cuatro auditorías del §140 encontraron
diecisiete cosas con las diez fases en verde, y **once no eran defectos del
producto sino de sus comprobaciones**. Cada una la encontró un sombrero
distinto.

---

## Cómo se usa

Antes de cerrar un tramo, se recorre la columna «la pregunta» de los roles que
toca el cambio. No hay que recorrer los nueve: hay que saber cuáles toca.

| Si el cambio toca… | Los roles que miran |
|---|---|
| Un tipo, un crate, una dependencia | Arquitecto, Seguridad |
| SQL, una migración, el esquema | Datos, Seguridad, Rendimiento |
| Un comando de Tauri | Arquitecto, Seguridad, QA |
| Una pantalla | Diseño, Accesibilidad, Redacción, QA |
| Un texto | Redacción, Legal |
| El motor, la cola, los límites | Rendimiento, Operaciones, Datos |
| WhatsApp o el correo de verdad | Operaciones, Legal, Seguridad |
| Un documento | Redacción, Dirección |

---

## 1 · Arquitecto

**Lo que mira:** que cada pieza esté en el crate que le toca y que las flechas
apunten hacia dentro. Que un estado imposible deje de ser representable en vez
de vigilarse con una regla.

**La pregunta:** *¿esto se puede hacer imposible en vez de prohibirlo?* Y la
otra: *¿quién lo llama?*

**Lo que ha encontrado:** el cero I/O del núcleo no miraba el reloj ni el azar.
El `forbid(unsafe_code)` se vigilaba en 2 de 7 crates. El grafo de dependencias
se vigilaba con una arista de doce.

## 2 · Datos

**Lo que mira:** el esquema, las claves ajenas, las cascadas, lo que una
migración se lleva por delante. Que un hijo no viaje sin su padre.

**La pregunta:** *si esto se borra o se reemplaza, ¿qué se va con ello?* Y:
*¿esta consulta se come la tabla entera?*

**Lo que ha encontrado:** el orden del manifiesto del respaldo no se podía
restaurar. `import_batch` fuera del esencial dejaba contactos sin padre.
Restaurar un esencial se llevaba `activity_event` por cascada — el mismo fallo
que en la Fase 1 costó la audiencia congelada, por tercera vez.

## 3 · Seguridad

**Lo que mira:** los quince puntos de `THREAT_MODEL.md` §8. Qué sale por la
frontera, qué entra, qué se guarda y dónde.

**La pregunta:** *¿esto lo puede provocar alguien de fuera?* Y: *¿este dato
tiene que existir?*

**Lo que ha encontrado:** la regla 3 de la frontera se vigilaba en 17 de 52
estructuras. El saneado «al usar» no existe, y el día que el cuerpo de una
campaña llegue al SMTP pasará por ahí sin que nada pida sanear otra vez.

> **Y un error propio, que vale más que los hallazgos:** buscó los comandos de
> ARCO por nombres **inventados**, salieron a cero, y estuvo a punto de
> anotarlo. **Un hallazgo falso en una auditoría de seguridad cuesta más que
> uno ausente:** manda a arreglar algo que funciona y gasta la credibilidad del
> resto. Antes de anotar que algo falta, se busca cómo se llama de verdad.

## 4 · QA

**Lo que mira:** los 32 casos frontera. Qué pasa con cero, con uno, con
quinientos mil, con el disco lleno, con el archivo en UTF-16.

**La pregunta:** *¿esta prueba puede fallar?* Y: *¿cuántas fronteras cruza?*

**Lo que ha encontrado:** la prueba de la página de contactos con **un solo
contacto** —pedir diez mil devolvía uno—. Un troceado verificado con 1 500
variables cuando el tope real de SQLite son 32 766.

> **Y el peor fallo de la Fase 8:** todas las pruebas de ida y vuelta del
> respaldo usaban **dos filas**, un trozo, ninguna frontera. El lector
> reventaba en cuanto una `ñ` caía partida entre dos trozos de 64 KiB, y la
> copia de seguridad no abría el día que hacía falta. **Una prueba de ida y
> vuelta con dos filas no prueba una ida y vuelta.**

## 5 · Rendimiento

**Lo que mira:** las trece cifras del presupuesto. Los planes de consulta con
`EXPLAIN QUERY PLAN`, no con intuición.

**La pregunta:** *¿esto crece con los datos?* Y: *¿lo he medido o lo supongo?*

**Lo que ha encontrado:** activar una campaña tarda 73.5 s de un presupuesto de
2 s, y se rompe **alrededor de los 30 000 destinatarios** — una lista
ordinaria. Y la causa no era la que parecía: la hipótesis obvia costaba 0.58 s
de setenta y tres. Sin medirlo se habría «optimizado» la parte más delicada del
producto para ganar medio segundo.

## 6 · Diseño

**Lo que mira:** que la pantalla esté en su sitio, que empiece donde las otras,
que los huecos sean los mismos. Y lo que sólo se ve pasando de una pantalla a
otra.

**La pregunta:** *¿esto se ve bien **junto a** lo de al lado?*

**Lo que ha encontrado:** Ajustes empezaba 300 px más a la derecha que las otras
doce — cada pantalla sola se veía bien. Ocho insignias con un reloj que no
esperaba nada. Dos entradillas a 54 px del título donde su hermana las tiene a 8.

## 7 · Accesibilidad

**Lo que mira:** contraste medido, foco visible, nombres accesibles, jerarquía
de encabezados, 32 px de objetivo, y que ningún estado se distinga sólo por
color.

**La pregunta:** *¿esto se puede usar sin ver los colores y sin ratón?*

**Lo que ha encontrado:** «Detener» con 2.41:1. Tres pantallas de catorce sin
`h1`. Cinco enlaces de 20 px de alto. Y que el escalado al 200 % **no encoge los
píxeles**: encoge la ventana, y nadie lo medía.

## 8 · Redacción

**Lo que mira:** que el texto diga las tres cosas del §95, que lleve cifras y
no adjetivos, que no prometa lo que el producto no hace.

**La pregunta:** *¿quién lee esto, y qué tiene que hacer después de leerlo?*

**Lo que ha encontrado:** Actividad hablaba de «roadmap», «andamiaje» y
«tokens». «1303 días» sin separador de miles, siete veces el mismo defecto. Y
un botón apagado **sin decir por qué**, tres veces en tres entregas distintas.

## 9 · Operaciones

**Lo que mira:** qué pasa en la máquina del usuario, con su red, su llavero y su
proveedor. Lo que no se puede probar desde aquí.

**La pregunta:** *¿esto lo he visto funcionar, o sólo lo he probado contra un
servidor de mentira?*

**Lo que ha encontrado:** **ARLES compilaba, arrancaba y la ventana salía
vacía** — nueve fases sin verse, porque todo se verifica con pruebas, que no
abren ninguna ventana. Y que la guía de arranque decía el comando equivocado,
porque se escribió sin recorrerla.

> **La regla que salió de ahí: al escribir una guía, se recorre.** Una
> instrucción que nadie ha seguido no es una instrucción verificada.

## 10 · Legal y datos personales

**Lo que mira:** el consentimiento, los derechos ARCO, la salida de baja, qué se
guarda de quién y durante cuánto.

**La pregunta:** *¿esta persona puede pedir que paremos, y lo podemos atender?*

**Lo que ha encontrado:** que en WhatsApp **no se puede** atender un «BAJA»
automáticamente —ni hay transporte ni se lee el texto—, así que exigir la frase
en la plantilla habría sido prometer lo que no se cumple. Decisión de Dirección
del 27/10/2026: se avisa a mano y se dice con esas palabras.

## 11 · Dirección

No es un rol que yo interprete: **es a quien se le informa.** Pero su forma de
mirar hay que anticiparla, porque es la que más ha encontrado.

**Lo que mira:** abre la aplicación y la usa.

**Lo que ha encontrado en dos tardes, con las cuatro auditorías del §140 en
verde:** que Remitentes decía «todavía no has configurado tu empresa» después de
configurarla. Que las tres partes de un error salían pegadas. Que no se podía
dar de alta una cuenta de Gmail. Que el texto «se ve extremadamente
desordenado» con las dos sondas sin un hallazgo. Y, leyendo un informe, que dos
campañas con el mismo remitente no mandaban 50 cada una.

> **Lo que falla es lo que sólo se ve usando.** Ninguna auditoría lo encontró, y
> por eso el último punto del método es: antes de cerrar, se abre y se mira.
