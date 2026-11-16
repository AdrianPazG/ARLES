# El logotipo de TELEMETRY dentro de ARLES

> Estado: **vigente** · Fase 3.1 · Depende de `tokens.json`

ARLES es un producto de TELEMETRY INSIGHT y lo dice en la interfaz. Este
documento fija qué archivo se usa, con qué tinta, con cuánto aire y a dónde
enlaza.

No confundir con el logotipo de **ARLES**, que es distinto: §21 lo define como
exclusivamente tipográfico, sin isotipo, y el icono del sistema operativo es la
inicial en Mont Black (`herramientas/iconos/generar-iconos.py`).

---

## 1 · Los originales, y por qué no se usan tal cual

Dirección entregó dos archivos en `RECURSOS/marca/`:

| Archivo | Tinta | Para |
|---|---|---|
| `Logo_Telemetry_Horizontal_Blanco.png` | `#EFE7DC` | fondos oscuros |
| `Recurso 45@4x-8.png` | `#001638` | fondos claros |

Son **la misma pieza en dos tintas** — comprobado: las dos máscaras de opacidad
coinciden salvo en el 1,26 % de los píxeles, que es el borde y el píxel de
desplazamiento que deja una reexportación.

Ninguna de las dos tintas está en la paleta de ARLES:

| Entregado | Token más cercano | Diferencia |
|---|---|---|
| `#EFE7DC` | `--arles-text` `#F4ECE4` | un blanco algo más apagado; al lado del texto de la interfaz se lee como blanco sucio |
| `#001638` | `--arles-bg-deep` `#041A25` | **matiz 216° frente a 200°**: es un azul de otra familia. Sobre el fondo de ARLES tira a violeta mientras todo lo demás tira a cian |

Eso es exactamente lo que desentona. No es un defecto del logotipo: es que se
dibujó para su propio sistema, no para éste.

## 2 · La solución: teñir, no repintar

La pieza es **de un solo color sobre transparencia**. El color no forma parte
del dibujo: es una tinta aplicada a una máscara. Cambiarla no altera la marca,
sólo la pone en el sistema donde vive.

`herramientas/marca/derivar-logos.py` lee los originales —**sin tocarlos**, son
material de sólo lectura por instrucción de Dirección— y escribe en
`app/src/app/activos/marca/`:

| Salida | Tinta | Token | Para |
|---|---|---|---|
| `telemetry-horizontal-tema-oscuro.png` | `#F4ECE4` | `--arles-text` | tema oscuro |
| `telemetry-horizontal-tema-claro.png` | `#041A25` | `--arles-bg-deep` | tema claro y papel |
| `telemetry-horizontal-mascara.png` | — | ninguno | **la que usa la interfaz**, barra desplegada |
| `telemetry-isotipo-mascara.png` | — | ninguno | **sólo el símbolo**, barra plegada |

### Dentro de la aplicación va la máscara, no la pieza teñida

La primera versión llevaba las dos piezas teñidas y elegía con una regla de
CSS. **Esa regla se descartó al compilar** —un `:global()` dentro de estilos
con ámbito— y el pie se quedó con la tinta crema sobre papel claro, casi
invisible. No falló nada: simplemente la regla no existía en el CSS final.

=> Ahora la interfaz usa **una sola pieza como máscara** y el color lo pone
`--arles-text-muted`. No hay regla que descartar, no hay dos archivos entre los
que elegir, y el logotipo **no puede quedarse con la tinta del otro tema**.

Las dos piezas teñidas siguen existiendo para el papel y para cualquier sitio
donde no se pueda enmascarar.

### El símbolo solo, y por qué el corte no está escrito a mano

Con la barra plegada el logotipo horizontal no entra en 64 px y **desaparecía
entero**: el pie se quedaba sin ninguna marca. Dirección pidió conservar el
símbolo sin el texto.

Son **dos piezas distintas**, no una recortada con `overflow`. Recortar dejaría
la «T» de TELEMETRY partida asomando por el borde.

=> Dónde acaba el símbolo lo **encuentra el script**, no una constante: busca el
hueco vertical más ancho de la pieza. Entre el símbolo y el texto hay **118 px
sin tinta**; los espacios entre letras rondan los **22**. El corte cae en
x=834 de 2776.

!i **El hueco se busca sobre el original, antes de reducir.** A 560 px de ancho
mediría 24 px y quedaría a la altura del espaciado entre letras, que es
exactamente la confusión que el método evita.

Si algún día la marca reexporta el archivo con otro encuadre, el script vuelve
a encontrar el corte solo. Y si no encuentra ningún hueco suficientemente
ancho, **para y lo dice** en vez de entregar medio logotipo.

**Los nombres dicen a qué tema sirven, no de qué color son.** Los originales
hacen lo contrario —«Blanco» describe la tinta— y es justo lo que lleva a
colocar el blanco sobre blanco.

Las tintas salen de `tokens.json`. Un cambio de paleta se propaga regenerando,
sin editar ningún hex a mano (§17).

### El guardián del script

Si los dos originales dejaran de ser la misma pieza, teñir uno solo produciría
dos logotipos distintos sin que nadie se entere. El script mide la divergencia
y **se niega a generar** por encima del 4 %.

Probado rompiéndolo a propósito: apuntando el segundo original a otra imagen,
el script para con «difieren en el 71,2 % de los píxeles». La comprobación está
verificada, no sólo escrita.

## 3 · Contraste medido

| Tinta | Fondo | Ratio | |
|---|---|---|---|
| `#F4ECE4` | `--arles-bg-deep` `#041A25` | **15,23:1** | ✅ |
| `#F4ECE4` | `--arles-surface` `#053048` | **11,80:1** | ✅ |
| `#041A25` | crema `#F4ECE4` | **15,23:1** | ✅ |
| `#041A25` | blanco | **17,80:1** | ✅ |

Un logotipo no es texto y no le aplica el 1.4.3 de WCAG, pero sí el **1.4.11
(componentes no textuales, 3:1)** cuando transmite información. Las cuatro
combinaciones lo superan con mucho margen, así que la decisión de tinta no
depende del contraste: depende de sobre qué se pose.

!! Las tintas **entregadas** también pasarían el contraste. Se cambian por
**coherencia cromática**, no por accesibilidad. Conviene no confundir las dos
razones.

## 4 · Cómo se coloca

**La pieza llega a sangre.** La tinta toca los cuatro bordes; no hay margen
dentro del archivo. Comprobado: la caja de tinta es la imagen entera.

=> Consecuencia: el aire lo pone la pantalla, no el archivo. Mínimo la altura
de la «T» por cada lado; en la barra lateral, `--espacio-3`.

- **Proporción 3,108:1.** Nunca se deforma: se fija el ancho y el alto sale
  solo.
- **Ancho de salida 560 px**, suficiente para el doble de densidad en el uso
  más grande. En la barra lateral ocupa unos 140 px.
- **Nunca sobre una superficie intermedia** (`--arles-surface-hover`,
  `--arles-border-strong`) sin volver a medir: la rampa cian sube rápido.

## 5 · Enlace

El logotipo enlaza a **https://telemetrymx.com**, confirmado por Dirección, y
vive en el pie de la barra lateral junto al número de versión.

Al ser una aplicación de escritorio, **no navega dentro de la ventana**: abre el
navegador del sistema. Una WebView que navega a internet deja de ser una
aplicación y pasa a ser un navegador sin barra de direcciones, donde el usuario
no puede saber dónde está.

### Por qué el comando no recibe la URL

Lo normal sería añadir un plugin de shell con un permiso `allow-open` y un
ámbito que valide la dirección contra una expresión regular. Eso deja **la
webview eligiendo el destino**, y la seguridad pasa a depender de que la
expresión esté bien escrita; una regular mal anclada convierte «abrir el sitio
de la empresa» en «abrir cualquier cosa», que es una de las rutas clásicas para
ejecutar algo en la máquina del usuario.

=> `abrir_sitio_de_telemetry` **no tiene parámetros**. El destino es una
constante compilada en Rust (`comandos::SITIO_DE_TELEMETRY`). El frontend no
puede pasar otra dirección porque no hay dónde ponerla: no hay ámbito que
validar, no hay expresión regular que revisar, y no hace falta una dependencia
más.

Si el navegador no abre, **no se interrumpe nada**: queda constancia en la
consola. Un logotipo que no abre el sitio es una molestia; un diálogo de error
por ello, una avería aparente.

## 6 · Pendientes

| | Qué | Estado |
|---|---|---|
| **M-1** | `RECURSOS/marca/Logo` es un archivo de texto de 19 bytes creado sin querer desde la web de GitHub. No se borra: `/RECURSOS` es de sólo lectura y el borrado lo autoriza Dirección | abierto |
| **M-2** | Falta versión vectorial (SVG/AI). El PNG a 2776 px basta para pantalla y para el PDF; para imprenta grande, no | abierto |
| **M-3** | El PDF corporativo aún no lleva el logotipo en portada. El activo ya está disponible | abierto |

---

**Para regenerar:**

```
python3 herramientas/marca/derivar-logos.py
```
