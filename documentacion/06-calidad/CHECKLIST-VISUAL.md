# Checklist para revisar ARLES con tus propios ojos

> **Para:** Dirección · **Fecha:** 17 de septiembre de 2026
> **Sobre:** ARLES RELAY I v1.2.0 · **31 % construido y verificado**

Esto **no** es la lista de comprobaciones automáticas —ésas son 110 y corren
solas con `validar.py`—. Esto es lo que **una persona tiene que mirar**, porque
una máquina no puede decir si algo se entiende o si se ve bien.

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
| **B-5** | Cierra ARLES y vuelve a abrirlo | La barra sigue **como la dejaste**, plegada o abierta |
| **B-6** | Estrecha la ventana mucho | La barra se pliega **sola**, y el botón queda deshabilitado **explicando por qué** |
| **B-7** | Pulsa el logotipo de TELEMETRY del pie | Abre **telemetrymx.com en tu navegador**, no dentro de la ventana |
| **B-8** | Entra en Campañas, Contactos, Remitentes y Actividad | Cada una dice que aún no está y **en qué entrega llega** |

---

## Bloque C · Apariencia · el tema

| | Qué mirar | Cómo saber si está bien |
|---|---|---|
| **C-1** | Ajustes → Apariencia → elige **Claro** | La pantalla cambia **al momento**, sin pulsar Guardar |
| **C-2** | Revisa todo en tema claro: Inicio, Ajustes, la barra | Todo se lee. Ningún texto gris sobre gris, ningún borde invisible |
| **C-3** | Cierra y vuelve a abrir | Sigue en **Claro** |
| **C-4** | Vuelve a **Automático** y cambia el tema de tu Windows o Mac | ARLES lo sigue **sin cerrarlo** |
| **C-5** | Elige **Oscuro** a mano y vuelve a cambiar el del sistema | ARLES **no** le hace caso: tu elección manda |

---

## Bloque D · El avance de la versión · lo que pediste hoy

| | Qué mirar | Cómo saber si está bien |
|---|---|---|
| **D-1** | Inicio → módulo «Lo que falta por construir» | Sale **31 %**, una barra, y **la fecha del dato** |
| **D-2** | Abre el repositorio en GitHub | La portada enseña la **misma cifra**, en una insignia |
| **D-3** | Despliega «Cómo sale ese número» en GitHub | Explica el método, enseña las fases con su peso, y **lo que el número no mide** |
| **D-4** | Compara D-1 y D-2 | **Tienen que ser iguales.** Si no, hay un fallo — y el validador debería haberlo cazado antes |

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
| **E-4** | Busca adjetivos tipo «casi listo» o «excelente» | No debería haber ninguno. Sólo cifras: «2 de 6», «31 %» |
| **E-5** | Lee los seis pasos del alta en Ajustes | Cada uno dice **qué es** y **en qué entrega llega** |

---

## Bloque F · Accesibilidad, en dos minutos

| | Qué mirar | Cómo saber si está bien |
|---|---|---|
| **F-1** | Recorre toda la pantalla con **Tab**, sin ratón | Siempre se ve **dónde está el foco**, con un recuadro amarillo |
| **F-2** | Llega al botón de plegar con Tab y pulsa **Espacio** | Funciona igual que con el ratón |
| **F-3** | Pon Windows al **200 %** de escala | Nada se corta ni se solapa |
| **F-4** | Activa «reducir movimiento» en el sistema | La barra de la sección activa deja de animarse: **llega directa** |

---

## Bloque G · Lo que NO se puede revisar todavía, y por qué

Para que no se busque:

| | Por qué no |
|---|---|
| Enviar un correo de prueba | El motor de envío es la Fase 4. No existe |
| Cargar contactos | Entrega 3.2. No existe |
| Conectar una cuenta de correo | Fase 5 |
| Conectar el número de WhatsApp | Pantalla de CANALES, siguiente paso — y además hace falta la cuenta de Meta |
| Ver estadísticas | Fase 7 |
| **Ver ARLES en un Mac** | **Nadie lo ha hecho.** Es el riesgo R-07, sigue abierto |

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
| `validar.py --fase 3` | Empresa, navegación, tema y avance | **31/31** |
| `cargo test` | Núcleo, base de datos e invariantes del esquema | **178 pruebas** |
| `npm test` | Componentes, pantallas y textos | **63 pruebas** |

Y cinco sondas que abren un navegador de verdad y **miden**: el umbral de
plegado, que la navegación no salte, que el tema haga lo que dice, el recorrido
por teclado y el ancho de la ventana.
