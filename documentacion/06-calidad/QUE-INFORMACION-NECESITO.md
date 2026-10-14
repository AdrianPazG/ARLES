# Qué información necesito para arreglar un error

**Para Dirección.** Esto no es burocracia: cada punto de aquí está porque su
ausencia ya nos costó tiempo una vez, y al lado de cada uno está qué pasó.

> **Si tienes prisa, con esto me basta:** en qué pantalla estabas, qué pulsaste,
> qué salió y una captura. Lo demás te lo pregunto yo.

---

## 1 · Lo mínimo, para cualquier error

| | Qué | Por qué |
|---|---|---|
| **1** | **En qué pantalla estabas** | Hay dieciocho. «En ARLES» no me dice cuál |
| **2** | **Qué hiciste justo antes** — el botón que pulsaste, lo que escribiste | La mitad de los defectos sólo salen con una secuencia concreta |
| **3** | **Qué pasó**, con el texto del mensaje | Nuestros errores llevan clave propia: con el texto exacto llego al sitio |
| **4** | **Qué esperabas que pasara** | A veces lo que falla no es el código, es que la pantalla promete otra cosa |
| **5** | **Una captura**, si se ve | Encontró el defecto del usuario que no coincidía con el correo (H-03) |

**Nunca des por supuesto que algo es poco importante.** En este proyecto, lo que
encontraste en dos tardes no lo había encontrado ninguna de las cuatro
auditorías.

---

## 2 · Lo que me hace falta una vez, y vale para siempre

Son datos de tu equipo, no de un error concreto. Dámelos una vez y no los
vuelvo a pedir.

| | Qué | Para qué |
|---|---|---|
| **A** | **Windows o macOS**, y qué versión | Las rutas, el llavero y los permisos son distintos. Y **nadie ha abierto ARLES en un Mac todavía** (R-07) |
| **B** | **Si lo abres compilando (`cargo arles`) o con el instalador** | Son dos cosas distintas, y un error puede existir en una y no en la otra |
| **C** | **Si ya habías instalado ARLES antes en ese equipo** | Una base de datos de una versión anterior da «fallo al migrar el esquema», que no es un defecto nuevo |

---

## 3 · Lo que necesito para cerrar H-03, que es el bloqueante de hoy

**H-03 · no se pudo dar de alta la cuenta de Gmail.** Es el único defecto
bloqueante abierto, y no lo puedo cerrar desde aquí: la red de este contenedor
bloquea los puertos de correo (587, 465 y 25), así que **no puedo probar contra
Gmail**. Eso es R-24.

Cuando vuelvas a intentarlo con la versión nueva:

| | Qué necesito saber | Por qué |
|---|---|---|
| **1** | **Si funcionó o no** | R-01 hace que el servidor, el puerto y el cifrado ya no se escriban a mano. Si la causa era ésa, hoy ya no puede pasar — pero **no lo doy por arreglado sin que lo compruebes** |
| **2** | Si falló: **el mensaje completo**, con la captura | «No se pudo cifrar la conexión» y «no se pudo autenticar» apuntan a sitios distintos |
| **3** | **¿La cuenta tiene verificación en dos pasos activada?** | Sin ella, Gmail **ni siquiera ofrece** crear una contraseña de aplicación. Es la causa más probable |
| **4** | **¿La contraseña que escribiste es la de aplicación o la de entrar a Gmail?** | La de entrar **no funciona nunca** con ARLES. La de aplicación son dieciséis letras |
| **5** | Si abriste «Configuración avanzada»: **qué decía dentro** | Ahí se vería si el servidor o el puerto se quedaron en algo raro |

> **Por qué pregunto en vez de suponer.** El mensaje que saliste decía «no se
> pudo **cifrar**», que apunta al servidor o al puerto, no a la contraseña — una
> credencial mala daría un error de autenticación. Pero en la captura se veía un
> usuario que no coincidía con el correo y una contraseña que parecía de ocho
> caracteres, y eso apunta a otro sitio. **Dar por buena una hipótesis sin los
> datos es inventarse un diagnóstico**, y en este proyecto eso ya ha costado
> caro dos veces.

---

## 4 · Decisiones que espero de ti, y que no son errores

Éstas no las puedo tomar yo. Cada una está parada hasta que me digas.

| | Qué decidir | Qué pasa mientras tanto |
|---|---|---|
| **P-01** | **La licencia de Mont.** ¿Se puede incrustar la tipografía en una aplicación que se distribuye? | Los instaladores salen **sin Mont**, con la letra del sistema. Y hay algo peor, abajo |
| **MONT** | **La tipografía está dentro de un repositorio público.** Cualquiera puede descargarla | Es P-01 por otra puerta. Sacarla cambia cómo se ve ARLES al compilar, así que **no la he tocado**: la decisión es tuya |
| **P-09** | **La revisión jurídica.** Las dieciocho preguntas están escritas y esperando a un abogado | Los textos legales que lee el usuario van marcados como provisionales. **No frena construir** |
| **R-24** | **Una cuenta de correo desechable** para probar ARLES contra un servidor real | Toda la Fase 5 está probada contra un servidor de mentira. **No se puede cerrar desde aquí** |
| **R-07** | **Alguien con un Mac** que abra ARLES una vez | Nadie lo ha hecho. No podemos afirmar que funcione en macOS |
| **META** | **Una cuenta de empresa en Meta** con Coexistencia activada | WhatsApp no se puede probar contra nada real |
| **F9** | **Cuándo quieres el instalador firmado**, que es lo que quita el «editor desconocido» | Hoy Windows avisa de que no conoce al editor. Es correcto: no está firmado |

---

## 5 · Lo que NO necesito que me digas

Para que no pierdas tiempo:

- **No hace falta que lo describas con términos técnicos.** «Se veía feo»,
  «estaba encimado», «no entendí qué quería» son reportes perfectamente útiles
  — el de «el texto se ve extremadamente desordenado» encontró un defecto que
  dieciocho pantallas en verde no habían encontrado.
- **No hace falta que lo reproduzcas dos veces** antes de contármelo.
- **No hace falta que investigues la causa.** Si tienes una sospecha dímela,
  pero como sospecha: la última vez la causa obvia resultó ser el 0.8 % del
  problema.
- **No hace falta que esperes a tener varios.** Si es bloqueante, dímelo al
  momento; si no, agrúpalos y los arreglo en tanda — cada instalador nuevo
  cuesta unos veinte minutos.

---

## 6 · Dónde queda registrado lo que me cuentes

| Dónde | Qué lleva |
|---|---|
| [`BITACORA-DE-PRUEBAS.md`](BITACORA-DE-PRUEBAS.md) | Cada defecto con su número, su gravedad y su estado. **La fila se queda ahí para siempre**, también cuando se arregla |
| [`REGISTRO-DE-TROPIEZOS.md`](../07-entrega/REGISTRO-DE-TROPIEZOS.md) | Los tropiezos de instalación, con su causa y su salida exacta |
| `tablero/ARLES-tablero.html` | Todo lo anterior, en gráficos. Doble clic |

Cuando arreglo algo, escribo **qué lo causaba** y **qué comprobación impide que
vuelva** — y esa comprobación se rompe a propósito para verla fallar, porque una
que nunca se ha visto fallar no está verificada.
