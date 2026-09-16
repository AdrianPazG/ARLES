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

El logotipo enlaza a **https://telemetrymx.com**, confirmado por Dirección.

Al ser una aplicación de escritorio, el enlace **no navega dentro de la
ventana**: abre el navegador del sistema. En Tauri eso exige el permiso
`shell:allow-open` con una lista cerrada de destinos —una sola URL—, no el
permiso abierto. Queda anotado para la entrega que incorpore el logotipo:
abrir un navegador desde la aplicación es superficie de ataque, y la lista
cerrada es lo que la cierra.

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
