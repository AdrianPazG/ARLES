# Hallazgos · Revisión en Windows del 14 de septiembre de 2026

**Quién revisó:** Adrián Paz (Dirección)
**Equipo:** Windows 11, pantalla de 1920 × 1080, escala habitual sin declarar
**Material:** las 9 capturas, plantilla rellenada y 4 grabaciones de pantalla
**Duración real:** de 22:58 a 23:29 — 31 minutos, frente a los 40 estimados

---

## Resumen

| | |
|---|---|
| Pendientes de la Fase 2 cerrados | **3 de 3** · ventana nativa ✅ · escalado ✅ · lector de pantalla ✅ *(automatizado)* |
| Defectos encontrados | **1**, y no lo vio el revisor: lo vi yo en su captura |
| Peticiones de producto | **2**, las dos sobre la barra lateral |
| Confirmaciones valiosas | 3 |

---

## R-01 · Desbordamiento horizontal por encima del 150 % de escala

**Es un defecto real.** El revisor marcó «No lo noté» en las tres escalas, pero
está a la vista en su propia captura `W-09-escala-200.png`: una **barra de
desplazamiento horizontal** en el borde inferior, y la tarjeta «Campos» cortada
por el borde derecho de la ventana.

### Medido, no deducido

Servido el catálogo y recorridos los anchos que produce cada escala en una
pantalla de 1920:

| Escala | Viewport CSS | Ancho que pide la página | ¿Desborda? |
|---|---|---|---|
| 100 % | 1920 px | 1920 | no |
| 125 % | 1536 px | 1536 | no |
| 150 % | 1280 px | 1280 | no |
| **175 %** | **1097 px** | **1120** | **sí** |
| 200 % | 960 px | 1120 | sí |
| 250 % | 768 px | 1120 | sí |

El umbral es exacto: **1120 px de viewport CSS**.

### La causa

`app/src/app/App.vue` fija `min-width: var(--arles-window-min-width)`, que vale
**1120 px**. El razonamiento al escribirlo fue: «la ventana no puede ser más
estrecha de 1120, así que el contenido nunca tiene que bajar de ahí».

**Ese razonamiento es falso en cuanto la escala pasa del 100 %.** El mínimo de
`tauri.conf.json` está en píxeles **lógicos**: a 200 %, pedir 1120 lógicos es
pedir 2240 físicos, más que la pantalla entera. La ventana no puede cumplirlo,
Windows la deja en los 1920 físicos que hay —960 lógicos—, y el `min-width` de
CSS, que no cede, provoca el desplazamiento.

Dicho corto: **confundimos el mínimo de la ventana con el mínimo del diseño**,
y a escala alta dejan de ser el mismo número.

### Por qué no lo detectó ninguna comprobación

Las sondas de navegador corren a 1440 × 900 y a 1120 × 720, las dos por encima
del umbral. Nunca se probó por debajo, porque el razonamiento anterior decía
que era imposible. Era imposible en píxeles físicos, no en lógicos.

### Alcance

Visto en el **catálogo**, que es pantalla de revisión y no viaja al cliente.
Pero el `min-width` está en el armazón de la aplicación, no en el catálogo: le
va a pasar a **todas** las pantallas de la Fase 3 en adelante.

---

## P-11 · La barra lateral debe quedarse fija y poder plegarse

Textual del revisor:

> *«necesito que el menú ubicado en la lateral [izquierda] permanezca fijo y
> que lo demás que está en pantalla sea posible desplazarse. También necesito
> que ese menú fijo pueda comprimirse.»*

Son **dos peticiones distintas**, y conviene no mezclarlas:

| | Qué es | Dificultad |
|---|---|---|
| **a · Barra fija** | Al desplazar una pantalla larga, la navegación no se va hacia arriba: sólo se mueve el contenido | Pequeña. Es un cambio de `overflow` en el armazón |
| **b · Barra plegable** | Un control que la reduce a sólo iconos, o la esconde | Mediana. Necesita decidir el estado plegado, si se recuerda, y qué pasa con los textos |

### Lo que las une con R-01

**La barra plegable es también el arreglo de R-01.** A 200 % la navegación se
come 240 de los 960 px disponibles. Plegarla automáticamente por debajo del
umbral devuelve el ancho que falta, y el `min-width` deja de hacer daño.

Dos peticiones y un defecto que se resuelven con la misma pieza. Por eso
recomiendo tratarlas juntas y no por separado.

### Recomendación

Va a **UX_NAVEGACION.md** y se construye en la **entrega 3.1**, que es la
primera que monta pantallas de verdad sobre este armazón. Hacerlo antes de 3.1
significaría rehacerlo; hacerlo después significa arrastrar el defecto por
todas las pantallas que se hayan construido encima.

---

## R-02 · SmartScreen no apareció · **no es defecto nuestro, pero deja un hueco**

> *«Nada raro, pero no me solicitó ningún permiso para proteger mi computadora
> de softwares.»*

El instalador **no está firmado**, así que SmartScreen debería haber avisado.
Que no lo hiciera apunta a una de estas: SmartScreen desactivado en ese equipo,
una política de empresa, o un antivirus que lo sustituye.

**Lo que importa no es la causa: es que ese paso quedó sin verificar.** No
sabemos qué va a ver un cliente al abrirlo por primera vez, que es justo lo que
esa parte de la revisión tenía que averiguar. Se repite en el siguiente equipo
que toque.

---

## Lo que sí quedó confirmado

| | Prueba |
|---|---|
| **La ventana nativa funciona** | Barra de título con «ARLES RELAY» e icono propio. No deja encoger por debajo de 1120 × 720 |
| **La versión sale del núcleo** | El pie dice `v1.2.0`, no el `v—` de reserva. La corrección de la Fase 1 funciona en la aplicación empaquetada |
| **El icono se ve nítido** | Confirmado en barra de tareas. La regeneración del `.ico` con las seis medidas sirvió de algo |
| **Mont carga en WebView2** | La tipografía se ve correcta en las ocho capturas |
| **A 125 % y 150 % no se rompe nada** | Verificado por el revisor y reproducido aquí |

---

## Lo que falta de esta entrega

| | Estado |
|---|---|
| **`W-01-ventana.png`** | El `.rar` llegó truncado. Dirección la reenvió como captura en el chat el 15/09: **ventana sin maximizar a su tamaño mínimo**, barra de título con «ARLES RELAY» e icono, `v1.2.0` en el pie, enlace al catálogo visible y la maqueta correcta a 1120 px. **Cierra el punto de la ventana nativa** |
| **NVDA, sección 5 entera** | **Resuelto de otra forma.** Ver abajo: `sonda:lector` comprueba en cada compilación lo que esa sección pedía escuchar |
| **Las grabaciones de pantalla** | Recibidas y revisadas por fotogramas. **No añaden defectos** sobre las capturas. El de  confirma además el frenado de la ventana en su tamaño mínimo, que en una captura no se ve |

---

## Observación menor, sin acción

El revisor escribió en los campos del catálogo (`asndfajsdfkafda`, `afsdfadsf`)
y el primero **no mostró error** aunque no es un correo válido.

**No es un defecto:** en el catálogo los campos son estáticos — uno enseña el
estado normal y otro el de error, y ninguno valida. Pero revela una expectativa
razonable, y conviene tenerla presente: cuando la Fase 5 monte el alta de
remitentes, la validación **en vivo** es lo que la gente va a esperar.

---

## Qué hacer con esto

1. **R-01 y P-11 juntos**, en la entrega 3.1, antes de montar pantallas encima.
2. **Una sonda nueva** que compruebe que la aplicación no desborda por debajo
   de 1120 px de viewport. Sin ella, esto vuelve.
3. **Reenviar `W-01`** y hacer la sección de NVDA.
4. La revisión de **Mac** sigue pendiente, y es la que cierra el cuarto punto.


---

## Cómo se cerró el punto del lector de pantalla · **15 de septiembre**

Dirección preguntó, después de no conseguir configurar NVDA:

> *«¿Lo consideras relevante? La verdad no pude configurarlo y tampoco creo que
> los clientes lo ocupen.»*

**Tenía razón en que la comprobación a oído no valía lo que costaba**, y estaba
equivocada en el motivo. Lo que hacía valiosa esa sección no era el usuario que
no ve: era que **el nombre y el rol accesibles son el mismo dato del que
dependen el teclado, las pruebas automáticas y la corrección estructural**. Un
botón sin nombre accesible suele ser un `aria-label` que falta, y ese mismo
`aria-label` que falta rompe otras cosas.

Así que en vez de insistir, se automatizó. **NVDA no inventa lo que dice**: lo
deriva del árbol de accesibilidad que expone el motor, y ese árbol se puede
leer. «¿Dice botón?» dejó de ser una pregunta de oído y pasó a ser una de dato.

`sonda:lector` comprueba, en cada compilación, los cinco puntos que la tabla
del manual pedía escuchar, más uno:

| | Resultado |
|---|---|
| Todo control visible tiene nombre accesible | ✅ |
| El logotipo se anuncia como **una sola cosa** | ✅ «ARLES RELAY» |
| Sólo **una** pestaña se anuncia como seleccionada | ✅ 4 pestañas |
| El error va **unido a su campo** (`aria-describedby` + `aria-invalid`) | ✅ |
| La tabla anuncia el **total real**, no lo que se ve | ✅ 5 001 con 18 en el DOM |
| Hay puntos de referencia de navegación y contenido | ✅ |

Probada rompiéndola: quitando `aria-rowcount` y quitando `aria-selected`, falla
en los dos casos.

**Lo que esto no cubre, dicho sin adornos:** si el recorrido *tiene sentido*
para alguien que no ve la pantalla. Eso sólo se descubre escuchando, y sigue en
el manual como opcional. Pero ya no bloquea nada, y **la comprobación es ahora
más fiable que una persona**: se repite igual en cada compilación y no depende
de que alguien recuerde qué oyó.
