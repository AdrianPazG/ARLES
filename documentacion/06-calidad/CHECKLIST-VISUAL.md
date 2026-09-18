# Checklist para revisar ARLES con tus propios ojos

> **Para:** Dirección · **Fecha:** 18 de septiembre de 2026
> **Sobre:** ARLES RELAY I v1.2.0 · **32 % construido y verificado**

Esto **no** es la lista de comprobaciones automáticas —ésas corren solas con
`validar.py`—. Esto es lo que **una persona tiene que mirar**, porque una
máquina no puede decir si algo se entiende o si se ve bien.

---

## Con qué se recorre

**Un solo archivo: `ARLES-vista-previa.html`.** Doble clic y se abre en el
navegador. No hay que instalar nada, ni descomprimir, ni pasar por GitHub.

| | |
|---|---|
| **Qué es** | La interfaz real: los mismos componentes, los mismos colores, los mismos textos y las dos versiones del tema |
| **Qué hay detrás** | Un núcleo **simulado**, el mismo que usan las pruebas automáticas |
| **Dónde se guarda lo que escribas** | En la pestaña del navegador. Sobrevive a **recargar** —que es el «cerrar y volver a abrir» de esta lista— y desaparece al cerrarla |

!! **No es la aplicación de escritorio.** No hay ventana nativa, ni base de
datos cifrada, ni llavero del sistema. Lo que se revisa aquí es **lo que se ve
y lo que se entiende**, que es justo lo que una máquina no puede juzgar.

=> **Para «cerrar y volver a abrir»**, recarga la página (**F5** en Windows,
**⌘R** en Mac). Para **cerrar del todo y empezar de cero**, cierra la pestaña y
vuelve a abrir el archivo.

---

## Antes de empezar · qué se puede revisar hoy y qué no

Conviene saberlo para no buscar lo que todavía no existe.

| Pantalla | Estado |
|---|---|
| **Inicio** | ✅ construida, en sus dos versiones (primera vez y panel) |
| **Ajustes** | ✅ construida: empresa, apariencia y la lista de alta |
| **Catálogo del sistema** | ✅ construido — es la muestra de todas las piezas de diseño |
| Campañas · Contactos · Remitentes · Actividad | ⬜ **son marcadores de sitio**: entras y dicen que aún no están |

=> Que esas cuatro digan «aún no» **es lo correcto**, no un fallo. Lo que se
revisa de ellas es que lo digan con claridad, no que funcionen.

**Y todo lo de WhatsApp está en la base de datos, no en pantalla.** Las tablas
existen y están probadas; la pantalla de CANALES es el paso siguiente. No hay
nada que mirar todavía de ese lado.

**Lo mismo con los contactos, y conviene decirlo claro.** ARLES ya sabe
guardarlos, editarlos, listarlos por páginas y darlos de baja, y ya existen los
cinco comandos que la pantalla usará para pedírselo. Pero **la pantalla todavía
no está**: al entrar en CONTACTOS sigue diciendo que aún no llega. Es lo que se
avisó antes de empezar este tramo — se construyó por dentro, y por dentro no se
ve. Lo que hay hecho se comprueba con las 49 pruebas de la capa de datos, no con
los ojos.

---

## Bloque A · Lo primero que ve alguien que abre ARLES

| | Qué mirar | Cómo saber si está bien |
|---|---|---|
| **A-1** | Abre la aplicación **sin haber configurado nada** | Sale **una sola cosa que hacer**: «Empieza por aquí» y un botón. No un panel de cajas vacías |
| **A-2** | Lee esa portada | Dice **por qué** hace falta la empresa, y avisa de que después vienen cinco pasos más. No promete que con eso ya se puede enviar |
| **A-3** | Pulsa el botón | Lleva a Ajustes, al formulario de empresa |
| **A-4** | Rellena la empresa y guarda | Aparece «Configuración guardada» y la lista de alta pasa de **1 de 6** a **2 de 6** |
| **A-5** | Vuelve a Inicio | Ya **no** sale la portada de primera vez: sale el panel |

!i **Lo que se juzga en A-1 y A-2** es si alguien que nunca ha visto ARLES sabe
qué hacer sin preguntar. Si dudas un segundo, es un hallazgo.

---

## Bloque B · La navegación

| | Qué mirar | Cómo saber si está bien |
|---|---|---|
| **B-1** | Pulsa cada una de las seis secciones | La sección activa se **rellena de otro color** y le crece una barra amarilla a la izquierda |
| **B-2** | Mira la barra al cambiar de sección | La barra **crece**, no aparece de golpe. Dura un parpadeo (200 ms) |
| **B-3** | Pulsa el botón de plegar (arriba a la derecha de la barra) | Quedan sólo iconos. **Los iconos no saltan de sitio** |
| **B-4** | Con la barra plegada, mira el pie | Sigue el **símbolo** de TELEMETRY, sin el texto |
| **B-5** | Recarga la página (**F5**) | La barra sigue **como la dejaste**, plegada o abierta |
| **B-6** | Estrecha la ventana mucho | La barra se pliega **sola**, y el botón queda deshabilitado **explicando por qué** |
| **B-7** | Pulsa el logotipo de TELEMETRY del pie | Abre **telemetrymx.com en una pestaña nueva** |
| **B-8** | Entra en Campañas, Contactos, Remitentes y Actividad | Cada una dice que aún no está y **en qué entrega llega** |

---

## Bloque C · Apariencia · el tema

| | Qué mirar | Cómo saber si está bien |
|---|---|---|
| **C-1** | Ajustes → Apariencia → elige **Claro** | La pantalla cambia **al momento**, sin pulsar Guardar |
| **C-2** | Revisa todo en tema claro: Inicio, Ajustes, la barra | Todo se lee. Ningún texto gris sobre gris, ningún borde invisible |
| **C-3** | Recarga la página (**F5**) | Sigue en **Claro** |
| **C-4** | Vuelve a **Automático** y cambia el tema de tu Windows o Mac | ARLES lo sigue **sin cerrarlo** |
| **C-5** | Elige **Oscuro** a mano y vuelve a cambiar el del sistema | ARLES **no** le hace caso: tu elección manda |

---

## Bloque D · El avance de la versión · lo que pediste hoy

**Se revisa en GitHub, no en la aplicación.** Dirección decidió que el avance va
sólo en la portada del repositorio: es un dato **del proyecto**, no del
producto. A quien use ARLES no le sirve saber que está al 32 %; le sirve saber
qué puede hacer hoy, que es lo que dice la lista de alta.

| | Qué mirar | Cómo saber si está bien |
|---|---|---|
| **D-1** | Abre el repositorio en GitHub | Arriba, una insignia con **32 %** y la frase con **la fecha del dato** |
| **D-2** | Despliega «Cómo sale ese número» | Explica el método, enseña las once fases con su peso y su avance, y **lo que el número no mide** |
| **D-3** | Mira la tabla de bloqueos, debajo | Están los cuatro trámites que esperan a alguien de fuera, con nombre de responsable |
| **D-4** | Comprueba que **dentro de ARLES no sale** ningún porcentaje de avance | Inicio habla de «1 de 6» pasos del alta, que es otra cosa |

!i La fecha está al lado del número a propósito: **el porcentaje no se
recalcula solo**. Sale de un archivo que se actualiza a mano al cerrar trabajo,
y un porcentaje sin fecha no dice nada.

---

## Bloque E · Los textos

Esto es lo que más rinde revisar, porque es donde una máquina no llega.

| | Qué mirar | Cómo saber si está bien |
|---|---|---|
| **E-1** | Cualquier mensaje de error que consigas provocar | Dice **tres cosas**: qué pasó, cómo arreglarlo y **qué está a salvo** |
| **E-2** | Guarda la empresa con el correo mal escrito | Marca **ese campo**, no un aviso general que obligue a revisarlo todo |
| **E-3** | Busca en toda la interfaz la palabra «entregado» | **No debería existir todavía.** ARLES no puede saber si un correo llegó |
| **E-4** | Busca adjetivos tipo «casi listo» o «excelente» | No debería haber ninguno. Sólo cifras: «2 de 6», «32 %» |
| **E-5** | Lee los seis pasos del alta en Ajustes | Cada uno dice **qué es** y **en qué entrega llega** |

---

## Bloque F · Accesibilidad, en dos minutos

| | Qué mirar | Cómo saber si está bien |
|---|---|---|
| **F-1** | Recorre toda la pantalla con **Tab**, sin ratón | Siempre se ve **dónde está el foco**, con un recuadro amarillo |
| **F-2** | Llega al botón de plegar con Tab y pulsa **Espacio** | Funciona igual que con el ratón |
| **F-3** | Amplía el navegador al **200 %** (**Ctrl** y **+**) | Nada se corta ni se solapa |
| **F-4** | Activa «reducir movimiento» en el sistema | La barra de la sección activa deja de animarse: **llega directa** |

---

## Bloque G · Lo que NO se puede revisar todavía, y por qué

Para que no se busque:

| | Por qué no |
|---|---|
| Enviar un correo de prueba | El motor de envío es la Fase 4. No existe |
| Cargar contactos a mano | La pantalla es la segunda mitad de la 3.2. Por dentro ya funciona; por fuera todavía no hay dónde escribirlos |
| Conectar una cuenta de correo | Fase 5 |
| Conectar el número de WhatsApp | Pantalla de CANALES, siguiente paso — y además hace falta la cuenta de Meta |
| Ver estadísticas | Fase 7 |
| **Ver ARLES en un Mac** | **Nadie lo ha hecho.** Es el riesgo R-07, sigue abierto. Y esta vista previa **no cuenta**: ahí el motor es WKWebView y aquí es el del navegador |
| La ventana nativa, el cifrado y el llavero | No existen en la vista previa. Se revisan cuando haya instalador |

---

## Cómo devolver los hallazgos

Con el **código del renglón** y qué esperabas ver. Por ejemplo:

> **B-3** — al plegar, el icono de Actividad se mueve un poco.

No hace falta explicar la causa. Si un renglón no se entiende, **eso ya es un
hallazgo**: quiere decir que la pantalla no se explica sola.

---

## Lo automático, para que conste

Esto ya corre solo y está en verde. No hay que revisarlo a mano.

| | Qué comprueba | Resultado |
|---|---|---|
| `validar.py --fase 1` | Cimientos, esquema, fronteras de seguridad | **48/48** |
| `validar.py --fase 2` | Design System y contrastes en los dos temas | **37/37** |
| `validar.py --fase 3` | Empresa, navegación, tema, avance y las fronteras de contactos | **38/38** |
| `cargo test` | Núcleo, base de datos, frontera IPC e invariantes del esquema | **222 pruebas** |
| `npm test` | Componentes, pantallas y textos | **63 pruebas** |

Y cinco sondas que abren un navegador de verdad y **miden**: el umbral de
plegado, que la navegación no salte, que el tema haga lo que dice, el recorrido
por teclado y el ancho de la ventana.
