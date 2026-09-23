# Fase 3 · Qué hicimos, explicado sin tecnicismos

**Para Dirección** · 22 de septiembre de 2026 · ARLES RELAY I, versión objetivo 1.2.0

> Este documento cuenta lo mismo que [FASE-03-EMPRESA-Y-CONTACTOS.md](FASE-03-EMPRESA-Y-CONTACTOS.md),
> pero sin dar por sabido nada de programación. Se lee en quince minutos.

---

## 1. En una frase

**La Fase 3 construyó todo lo que hay que tener antes de poder mandar un solo mensaje.**

De qué empresa salen los envíos. A quién se le puede escribir. Cómo entran esos contactos. Y —la parte que más trabajo costó— **a quién no se le vuelve a escribir nunca**.

Lo que todavía no existe es el envío en sí. Eso es la Fase 4, y se apoya entera en esto.

---

## 2. La idea que hay que llevarse de aquí

ARLES no tiene botón de deshacer, y eso no es un descuido: **no puede tenerlo**. Un correo que salió, salió. Una lista de diez mil contactos que se fusionó mal no se desfusiona a mano. Una persona a la que se le escribió después de pedir que no, ya recibió el correo.

Así que toda esta fase está construida sobre la misma idea: **lo caro se pregunta antes, no se arregla después**.

Se ve en tres sitios concretos:

**Al importar.** Cuando el archivo trae gente que ya está en su lista, ARLES **le enseña los choques antes de escribir nada**. No los salta ni los fusiona por su cuenta. Es una pantalla más de trabajo, y es lo que separa «ya está» de «ahora hay que revisar diez mil filas».

**Al borrar.** Para borrar los datos de alguien hay que **escribir su dirección completa**. Una casilla de «entiendo que es irreversible» se marca sin leerla — la prueba es que todo el mundo las marca. Escribir la dirección le obliga a mirarla, y si se equivocó de persona al teclear arriba, ahí lo ve.

**Al dar de baja.** Una baja **no vive colgada del contacto**. Vive por su cuenta. Eso importa por un motivo muy concreto que explico en §4.

---

## 3. Qué se construyó, en lenguaje llano

**Siete pantallas**, que es la primera vez que ARLES se parece a un programa y no a un conjunto de piezas:

| Pantalla | Para qué |
|---|---|
| **Inicio** | Qué puede hacer hoy y qué le falta para poder enviar |
| **Configuración** | Los datos de su empresa: nombre, país y zona horaria |
| **Contactos** | La lista, con su alta, su edición y su ficha |
| **Importar** | Un asistente de cuatro pasos para traer un CSV o un Excel |
| **Bajas** | Quién pidió no recibir nada, y por qué |
| **Derechos** | Responder a quien le pida sus datos, o que se los borre |
| **Ajustes** | Tema y versión. Sólo lo que es de la aplicación |

Y por debajo: la base de datos cifrada con cinco migraciones al cerrar la fase, la lectura defensiva de archivos, y las reglas de validación de correos y móviles mexicanos.

**La zona horaria no es un ajuste cosmético.** Es la que decide a qué hora sale cada correo. Por eso los datos de su empresa no viven en «Ajustes» —que es tema y versión— sino en su propia pantalla dentro de Inicio.

---

## 4. La decisión de la que más orgulloso estoy

Y es aburrida de contar, así que la cuento con el caso real que evita.

Imagine que en marzo alguien le pide que deje de escribirle. Usted lo da de baja. En junio, limpiando, borra ese contacto de la lista. En septiembre importa otra vez el archivo original de clientes, que todavía tiene a esa persona dentro.

**Si la baja estuviera colgada del contacto, al borrar el contacto se habría borrado la baja.** Esa persona volvería a su lista como si nada hubiera pasado, y le llegaría el siguiente correo.

Nadie se daría cuenta hasta que esa persona se queje — y entonces ya es una queja ante el INAI, no un error interno.

Por eso la baja **vive por su cuenta**, indexada por la dirección, no por el contacto. Y por eso la puerta de salida de esta entrega es exactamente ese recorrido: dar de baja, borrar el contacto, reimportarlo seis meses después, y comprobar que **sigue sin escribírsele**. Es una prueba automática que corre en cada cambio.

**Y cuando se borran los datos de alguien de verdad**, hay una prueba que recorre la base de datos entera —todas las tablas, todas las columnas, una por una— buscando su dirección. Después del borrado sólo puede quedar en **un sitio**: la lista de bajas, que es justo la que impide que vuelva. Si mañana alguien añade una tabla nueva y se olvida de limpiarla, esa prueba lo dice, porque no trabaja con una lista que haya que mantener: mira la base que hay.

---

## 5. Lo que encontramos al revisar, y por qué importa

Hicimos **dos auditorías** en esta fase, el 15 y el 22 de septiembre.

### 5.1 El más revelador: la vista previa le estaba mintiendo

El 18 de septiembre usted reportó que «Configurar mi empresa» le llevaba a Ajustes. Lo corregimos. Lo corregimos **en el programa**, y el programa lleva bien desde ese día.

Pero **la vista previa que usted abre para revisar no se tocó**. Siguió llevando a Ajustes cuatro días más.

Por eso usted reportó dos veces el mismo defecto. La segunda vez, el defecto ya no existía en el producto: sólo en la copia que estaba mirando.

Esto es peor que un error de pantalla, porque **convierte cada revisión suya en ruido**: usted no puede saber si lo que ve es el producto o un desajuste de la copia. Ahora hay una comprobación que compara las rutas de la vista previa con las del programa y falla si se separan.

### 5.2 Un botón apagado que parecía pulsable

Un botón deshabilitado conservaba su marco dorado: texto apagado dentro de un marco de acción. A simple vista, un botón disponible que no responde — que es el peor de los dos errores posibles.

Lo interesante es **por qué no se había visto**: los colores del sistema estaban verificados desde la Fase 2. Lo que no estaba verificado era **cómo se combinan las reglas** cuando dos de ellas se pisan.

### 5.3 La documentación decía cuatro cosas que ya no eran ciertas

Usted pidió revisar toda la documentación de GitHub. Son **66 documentos**. Leerlos de uno en uno no sirve: para cuando termina de leer el último, el primero ya cambió.

Así que en vez de leerlos, escribí algo que los **mide**. Encontró esto:

- **Tres documentos decían «las ocho preguntas»** para el abogado. Son dieciocho desde hace días.
- **La consulta jurídica decía «nueve preguntas nuevas: C-9 a C-18»**. De C-9 a C-18 hay **diez**, no nueve. Las dos mitades de la frase eran creíbles por separado, y contarlas a mano es justo lo que nadie hace.
- **El índice no nombraba diez documentos**, entre ellos dos importantes: la propia consulta jurídica y el informe de la fase anterior. Un mapa que no nombra media biblioteca es peor que no tener mapa, porque parece completo.

Los cuatro están corregidos. Y ahora hay una comprobación que se ejecuta sola y revisa, cada vez: que ningún enlace lleve a un archivo que ya no existe, que ninguna cifra afirmada haya caducado, que ningún documento prometa un anexo que nunca se escribió, y que el índice nombre todo.

**Una cosa que decidí a propósito:** las cifras con fecha **no se tocan**. Cuando un documento dice «133 pruebas al cerrar la entrega 3.1», eso era cierto entonces y lo seguirá siendo siempre. Reescribirlo para que cuadre con hoy destruiría exactamente lo que un documento de fase sirve para guardar: cómo estaban las cosas ese día.

---

### 5.4 Dos comprobaciones que no servían para nada

Esta es la que quiero que se lleve.

Una de las comprobaciones nuevas preguntaba si algo se salía de la pantalla. Decía «todo bien» en las diez pantallas. **Le metimos un bloque gigante a propósito y siguió diciendo «todo bien»**: estaba preguntando algo que no podía ser cierto nunca.

Su segunda versión hacía lo contrario: señalaba un problema en las diez pantallas, cada vez, sin que hubiera nada que arreglar. Un aviso que siempre está es un aviso que se deja de leer.

Las dos se arreglaron **porque se les rompió algo a propósito**, no porque alguien las leyera y le pareciera que estaban bien.

Ésta es la regla de trabajo del proyecto, y quiero explicarla bien porque es la que más valor da:

> **Una comprobación que nunca se ha visto fallar no está verificada.**

Dicho en simple: si usted pone una alarma en su casa y nunca la ha oído sonar, no sabe si funciona. Sabe que no ha sonado, que es distinto. La única forma de saberlo es abrir una puerta a propósito y comprobar que suena.

Eso es lo que hacemos con cada comprobación antes de confiar en ella: rompemos la cosa que vigila, miramos que salte, y lo dejamos anotado. **En esta fase eso ha descubierto cinco comprobaciones que no servían para nada** — incluida una de las que escribí esta misma semana para vigilar la documentación, que decía «todo correcto» sin haber mirado ni un solo caso.

---

## 6. Lo que se midió y estaba bien

Con cifras, no con adjetivos:

- **Cero** errores en las diez pantallas, en los dos temas.
- **Ningún** texto por debajo de su mínimo legible de contraste.
- **Nada** cortado ni fuera de la ventana, en pantallas de 1280, 1024 y 900 píxeles de ancho.
- **88 paradas** de tabulador, todas marcando dónde está el teclado.
- **Trece archivos** construidos para hacer daño —archivos que se hinchan al abrirse, fórmulas escondidas, CSV disfrazados de Excel—: ninguno pasa.

---

## 7. Lo que todavía NO está comprobado

Lo digo porque callarlo sería lo fácil:

**Nadie ha abierto ARLES en un Mac.** En Windows el motor de pantalla es uno y en macOS es otro, y dibujan distinto. La vista previa no cuenta: ahí el motor es el del navegador. Fecha límite acordada: antes de la Fase 6.

**Ninguna pantalla se ha visto en una ventana de verdad.** Lo comprobado es Chromium, que es el mismo motor que usa Windows por dentro, pero no la ventana del programa instalado.

**El instalador no se puede distribuir todavía**, por la licencia de la tipografía (P-01). Hay plan B y es una línea de código, pero la decisión es suya.

---

## 8. Lo que necesita a Dirección

| | Quién | Qué bloquea exactamente |
|---|---|---|
| **P-01 · licencia de Mont** | Dirección | El instalador descargable. Sin esto no se puede entregar el `.exe` con la tipografía dentro |
| **P-09 · revisión jurídica** | Abogado | Las 18 preguntas están en su PDF. **No frena construir**; frena los textos legales que el usuario lee y el primer envío a alguien de fuera |
| **META · cuenta con Coexistencia** | Operaciones | El alta del número de WhatsApp. Hasta que exista, esa parte no se puede probar contra nada real |
| **R-07 · revisión en macOS** | Quien tenga un Mac | Nada todavía. Fecha límite: antes de la Fase 6 |

**Ninguno de los cuatro impide seguir construyendo.** Los cuatro impiden entregar la v1.2.0.

---

## 9. Sobre el diseño

Usted revisó la dirección visual el 22 de septiembre y le pareció genérica. Tiene razón, y el motivo está escrito: lo que se hizo fue pulido —capas sutiles sobre lo que ya había—, y el pulido no cambia el carácter de nada. Lo genérico no es el color: es la composición, y eso no se tocó.

Decisión suya, registrada: **se retoma cuando tenga referencias**. No bloquea nada; la aplicación funciona igual y todo lo construido se conserva.

---

## 10. Cómo se comprueba todo esto sin creernos nada

Una sola orden lo corre todo:

```
python3 herramientas/validar/validar.py --fase 3
```

Hoy da **77 de 77, con 0 omitidas**, más las comprobaciones de la documentación. «Omitida» es importante: significa que una comprobación no se pudo correr. Una omitida **no es una que pasa**, y el validador las cuenta aparte para que nadie las confunda.

Por debajo: **386 pruebas** del motor en Rust, **102** de las pantallas, y **9 sondas** que abren un navegador de verdad, pulsan y miden. Las sondas existen porque hay cosas que sólo se ven así: que una pantalla **se encuentre desde la barra**, que un botón no se arme solo, que el contraste real sea el que decimos.

---

## 11. Conclusión

La Fase 3 deja ARLES con todo lo que necesita para saber **a quién escribir y a quién no**. La Fase 4 construye el motor que escribe.

Lo que se lleva de aquí, si sólo se lleva una cosa: **esta fase está construida asumiendo que no hay deshacer**, y por eso pregunta antes en vez de arreglar después. Es más lenta de usar y es la razón de que se pueda usar con datos reales.
