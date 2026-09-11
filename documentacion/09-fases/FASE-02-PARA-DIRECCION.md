# Fase 2 · Qué hicimos, explicado sin tecnicismos

**Para Dirección** · 11 de septiembre de 2026 · ARLES RELAY I, versión objetivo 1.2.0

> Este documento cuenta lo mismo que [FASE-02-DESIGN-SYSTEM.md](FASE-02-DESIGN-SYSTEM.md),
> pero sin dar por sabido nada de programación. Se lee en diez minutos.

---

## 1. En una frase

**La Fase 1 puso los cimientos. La Fase 2 fabricó las piezas con las que se van a construir todas las pantallas.**

Piense en una constructora que, antes de levantar cien casas, fabrica sus propias puertas, ventanas y escaleras con una medida única. Cuando llega el momento de construir, ya no se discute cada puerta: se toma del almacén y encaja. Eso es lo que hicimos.

Todavía no hay pantallas que un usuario pueda usar para enviar correo. Lo que hay es **el juego de piezas completo**, comprobado una por una.

---

## 2. La idea que hay que llevarse de aquí

En la Fase 1 convertimos decisiones en **reglas automáticas**. En la Fase 2 dimos un paso más: metimos las reglas **dentro de las piezas**.

Un ejemplo. Decidimos que un aviso de estado —«Aceptado», «Falló»— nunca se comunique sólo con un color, porque una de cada doce personas no distingue bien los colores. Eso podía quedar como una norma que alguien recuerda. En vez de eso, la pieza «aviso de estado» **elige el icono ella sola, a partir del estado**. No existe forma de construir un aviso sin icono: no es que esté prohibido, es que no se puede.

Otro. Todo mensaje de error del producto tiene que decir tres cosas: **qué pasó, cómo arreglarlo y qué está a salvo**. La pieza «error» exige las tres. Si alguien intenta usarla con dos, **el proyecto no compila**. No es un recordatorio: es un impedimento.

Hoy hay **18 comprobaciones automáticas** de esta fase. Las 18 pasan, ninguna está desactivada, y todas se probaron rompiendo algo a propósito para ver que saltan.

---

## 3. Qué se construyó, en lenguaje llano

**Once piezas básicas**, que son de las que están hechas todas las pantallas:

| Pieza | Para qué |
|---|---|
| Botón | Cuatro clases: la acción principal, las normales, las discretas y las peligrosas |
| Campo de texto | Dónde se escribe. Si hay un error, lo explica debajo |
| Desplegable | Para elegir de una lista |
| Aviso de estado | «Aceptado», «En cola», «Falló» — siempre con icono |
| Nota informativa | Donde el producto tiene que advertir algo |
| Ventana de confirmación | Para las decisiones importantes |
| Menú de acciones | Las opciones secundarias |
| Pestañas | Para dividir una pantalla en secciones |
| **Tabla** | La más importante: donde se ven los contactos |
| Iconos | Un juego cerrado, dibujado a mano |
| Logotipo | «ARLES RELAY» — sin símbolo, tal como manda la marca |

Y **las cuatro situaciones** por las que pasa cualquier pantalla: cuando está **vacía**, cuando está **cargando**, cuando hay un **error** y cuando algo **salió bien**. Ninguna pantalla se dará por terminada sin las cuatro.

Además: la **tipografía Mont** ya está dentro de la aplicación, y un **catálogo** —una pantalla interna— donde se ven todas las piezas juntas para revisarlas con los ojos.

---

## 4. Lo que encontramos al revisar, y por qué importa

Esta es la parte que de verdad tiene valor para Dirección. Las piezas pasaban todas las comprobaciones automáticas. Al atacarlas a propósito **encontramos 10 defectos**. Los 10 están corregidos.

### 4.1 El grave: la tabla no funcionaba como creíamos

La tabla de contactos está pensada para que, aunque haya 500 000 personas en la lista, el programa **sólo dibuje las veinte filas que caben en pantalla**. Es lo que permite que no se congele.

Estaba dibujando **las 5 001 de la prueba**. Todas.

Lo importante no es el fallo —era una línea— sino que **era invisible por todos los caminos que teníamos**: en una captura de pantalla se veía perfecta, las pruebas automáticas no podían detectarlo, y con 5 000 filas de prueba el equipo aguanta sin quejarse.

Habría llegado intacto hasta la fase donde se cargan los contactos reales. Y ahí, con medio millón, **habría congelado la aplicación del cliente**. Encontrarlo ahora costó una tarde; encontrarlo allí habría costado semanas y la confianza de quien lo estuviera usando.

### 4.2 Tres de seguridad

**Los campos ofrecían guardar la contraseña.** Windows y macOS meten dentro de nuestra aplicación el mismo gestor de contraseñas que tienen sus navegadores. Un formulario de correo con eso activo acabaría guardando la contraseña del cliente **fuera de nuestra caja fuerte**, que es justo lo que la Fase 1 se comprometió a impedir. El formulario todavía no existe —es de la Fase 5—, así que esto es una barandilla puesta antes del precipicio.

**Aflojamos una defensa que ya no hacía falta.** La Fase 1 había dejado apuntado que una de las protecciones contra código malicioso tenía que quedar a medio gas porque el producto no funcionaría sin ello. Al medirlo, resultó **falso**: la aplicación funciona perfectamente con la protección al máximo. Se apretó. Un riesgo que se acepta sin medirlo es una suposición disfrazada de decisión.

**Una rendija teórica en el manejo del teclado**, cerrada. No era aprovechable, pero no es un patrón que se deje escrito donde entra lo que teclea el usuario.

### 4.3 Y cuatro de accesibilidad y comportamiento

Un elemento al que se llegaba con el tabulador **no marcaba dónde estaba** el cursor. Cambiar de pestaña desde otro sitio **le robaba el cursor** al usuario. La tipografía no cargaba en el entorno de trabajo y nadie lo notaba porque el sistema la sustituía en silencio. Y dos de nuestras propias comprobaciones **no comprobaban nada**: una pasaba aunque la regla estuviera mal escrita.

### 4.4 Lo último, y quizá lo más revelador

Con todo corregido y en verde, quedaba una pregunta que nadie había hecho: **las comprobaciones nuevas, ¿se ejecutan también en el servidor donde se revisa el trabajo de todos?**

No. Faltaba un componente y el servidor las habría **saltado en silencio**, terminando en verde.

Es decir: las tres comprobaciones que encontraron el defecto más grave de la fase **no habrían protegido nada** en el único sitio donde la protección importa, que es cuando otra persona sube un cambio. Ya se ejecutan, y ahora el servidor **falla si se salta una sola**.

---

## 5. Lo que todavía NO está comprobado

Esta sección importa tanto como la anterior, y está declarada a propósito.

| Qué falta | Por qué | Cuándo |
|---|---|---|
| **Verlo en una ventana de Windows o Mac de verdad** | Lo revisamos en el servidor donde se construye, que usa el mismo motor pero no es el mismo programa | Fase 3, en un equipo con pantalla |
| **Que se vea igual en Mac que en Windows** | Cada sistema dibuja la interfaz con un motor distinto. Riesgo conocido y aceptado desde el principio | Fase 3, pantalla por pantalla |
| **Con la letra al 125 %, 150 % y 200 %** | Muchos usuarios de Windows agrandan la letra. Necesita Windows real | Fase 3 |
| **Con un lector de pantalla real** | Las piezas están preparadas y comprobadas, pero no se han escuchado con NVDA ni VoiceOver | Fase 3 |
| **La tabla con 500 000 contactos** | Ya sabemos que dibuja sólo lo visible. Falta medir con volumen real | Fase 4 |

Y la limitación honesta del método: las comprobaciones verifican **que las reglas siguen aplicadas**, no que el resultado sea bonito ni cómodo. Eso lo decide una persona mirando.

---

## 6. Lo que necesita a Dirección

Un solo asunto nuevo, y es el mismo de la fase anterior.

| Asunto | Qué hace falta | Para cuándo |
|---|---|---|
| **Licencia de la tipografía Mont** | Ya está funcionando dentro de la aplicación en los equipos de desarrollo. La licencia que permite **distribuirla dentro del programa instalado** sigue sin verificarse. Si no se resuelve, el plan alternativo está listo y cuesta cambiar una línea | **Antes de la demo** |

Además, revisando la tipografía encontramos dos cosas que refuerzan la duda sobre ese paquete de fuentes: **le falta uno de los grosores** que la guía de diseño pedía, y **los archivos declaran un grosor distinto del que realmente tienen**. Ninguna de las dos rompe nada —ya están resueltas—, pero son señales de que el paquete que hay en el proyecto no parece una entrega comercial del fabricante. Conviene tenerlo presente al pedir la licencia.

Los otros dos asuntos siguen igual que en la Fase 1: la **verificación de Google** (trámite ya arrancado, es el plazo externo más largo) y los **certificados de firma** (1 a 3 semanas, antes de la Fase 9).

---

## 7. Cómo se comprueba todo esto sin creernos nada

Una sola orden:

```
python3 herramientas/validar/validar.py --fase 2
```

Resultado esperado: **18 pasan, 0 fallan, 0 omitidos**.

Y para verlo con los ojos, el catálogo se abre en `/#/catalogo` y muestra todas las piezas juntas, en todos sus estados.

---

## 8. Conclusión

La Fase 2 está **cerrada y revisada**. Lo que deja:

- **Once piezas básicas y las cuatro situaciones de pantalla**, con las reglas de diseño metidas dentro, no escritas aparte.
- **La tipografía de la marca** funcionando en la aplicación.
- **10 defectos encontrados y corregidos** antes de construir encima, incluido uno que habría congelado la aplicación del cliente con los contactos reales.
- **Una defensa de seguridad apretada** que llevaba una fase entera aflojada sin necesidad.
- Las comprobaciones **ejecutándose también en el servidor**, que es donde protegen de verdad.
- Una lista escrita y honesta de lo que aún no se ha visto, con fecha para cada punto.

La lección de esta fase, en una línea: **una prueba que no puede fallar de cierta manera, no está probando esa manera.** El defecto más grave de la fase era invisible para todas las pruebas que teníamos, y sólo apareció cuando construimos una que sí podía verlo.

**Lo siguiente es la Fase 3: empresa y contactos** — las primeras pantallas con las que un usuario hará algo de verdad.

---

*TELEMETRY INSIGHT · ARLES RELAY I*
