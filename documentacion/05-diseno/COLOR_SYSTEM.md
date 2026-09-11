# Sistema de color

**Proyecto:** ARLES RELAY I · v1.2.0
**Decisión base:** ADR-0005 (D-2)
**Objetivo de accesibilidad:** WCAG 2.2 AA

> Todos los ratios de este documento están **calculados**, no estimados. El método está en §6 para que cualquiera pueda reproducirlos.

📐 **Lámina visual:** [`/REFERENCIA_DE_COLOR/ARLES_RELAY-paleta-v1.2.0.png`](../../REFERENCIA_DE_COLOR/ARLES_RELAY-paleta-v1.2.0.png) — la paleta completa con tokens, valores y matriz de contraste en una sola hoja. Es la referencia rápida; **este documento sigue siendo la fuente de verdad.**

---

## 1. De dónde sale la paleta

`/REFERENCIA_DE_COLOR/farm-lifestyle-digital-art.jpg` es en realidad un **PNG de 2320×3080**. Se decodificó completo y se muestrearon 1 027 líneas de barrido.

### Lo que la referencia sí tiene

| Familia | Rango | Presencia |
|---|---|---:|
| Cian / azur | H 185–210° | **54.35 %** |
| Ocre / naranja | H 15–38° | 19.48 % |
| Amarillo / oro | H 38–62° | 13.44 % |
| Teal | H 160–185° | 5.39 % |
| **Azul profundo** | **H 210–250°** | **0.73 %** |

Luminosidad: 76 % entre L 20–60. Sólo 1 % bajo L 10.

### Lo que no tiene, y hay que construir

**El azul profundo está ausente** (0.73 %) y **no hay fondos oscuros utilizables** (1 % bajo L 10).

Esto tiene dos consecuencias que determinan todo el sistema:

1. **El §16 no se puede extraer literalmente.** Asigna tres azules distintos a estructura, superficie e interacción, pero el 54 % de la imagen cabe en ±12°. Extraerlos los colapsa, y el resultado es una interfaz monocromática donde nada distingue el chasis de lo pulsable.
2. **El fondo dark-first hay que fabricarlo.**

**Decisión (D-2):** se extraen los colores de carácter —cian, oro, cremas— y se **deriva por rampa** la estructura de superficies que falta.

---

## 2. Colores extraídos

Medidos directamente en la referencia. Estos son los que dan el carácter cromático de ARLES.

| Token | Hex | RGB | H/S/L | Presencia |
|---|---|---|---|---:|
| Cian vivo | `#2CA4D4` | 44,164,212 | 197/66/50 | 0.81 % |
| Azul medio | `#045484` | 4,84,132 | 202/94/27 | 3.11 % |
| Cian medio | `#0C749C` | 12,116,156 | 197/86/33 | 3.12 % |
| **Amarillo girasol** | `#FCCC0C` | 252,204,12 | 48/98/52 | 0.30 % |
| Crema | `#F4ECE4` | 244,236,228 | 30/42/93 | 0.33 % |
| Crema apagado | `#E4E4CC` | 228,228,204 | 60/31/85 | 0.36 % |
| Ocre | `#AC5C0C` | 172,92,12 | 30/87/36 | 0.27 % |

> **Nota sobre el §16:** el brief advierte, con razón, que no se elijan colores por frecuencia de píxel. El amarillo ocupa el 0.30 % y es el acento fundamental del sistema. Los roles se asignan por **función**, no por presencia.

---

## 3. Rampa de superficies derivada

Construida sobre H≈200° —la media de los cianes dominantes— con saturación descendente al aclarar, que es cómo se comporta la luz en la propia referencia.

| Token | Hex | H/S/L | Lum. rel. |
|---|---|---|---:|
| `--arles-bg-deep` | `#041A25` | 200/82/8 | 0.0090 |
| `--arles-surface` | `#053048` | 201/88/15 | 0.0261 |
| `--arles-surface-raised` | `#045686` | 202/94/27 | 0.0840 |
| `--arles-surface-hover` | `#0C6F9D` | 199/86/33 | 0.1388 |
| `--arles-border` | `#0A4E71` | 200/84/24 | 0.0671 |
| `--arles-border-strong` | `#1380AE` | 198/80/38 | 0.1863 |

`--arles-surface-raised` es esencialmente el `#045484` extraído: la rampa **converge con la referencia** en el punto medio, que es lo que mantiene el parentesco cromático.

---

## 4. Paleta completa

```css
:root {
  /* Estructura — derivada (D-2) */
  --arles-bg-deep:         #041A25;
  --arles-surface:         #053048;
  --arles-surface-raised:  #045686;
  --arles-surface-hover:   #0C6F9D;
  --arles-border:          #0A4E71;
  --arles-border-strong:   #1380AE;

  /* Texto — extraído */
  --arles-text:            #F4ECE4;
  --arles-text-muted:      #E4E4CC;
  --arles-text-on-accent:  #041A25;   /* texto OSCURO sobre rellenos claros */

  /* Acento e información — extraído */
  --arles-accent:          #FCCC0C;
  --arles-info:            #2CA4D4;

  /* Semánticos — derivados, verificados contra las superficies */
  --arles-warning:         #E08A1C;
  --arles-danger:          #F06A5A;
  --arles-success:         #4FC3A1;
}
```

`--arles-text-on-accent` es un token, no un color suelto: garantiza que nadie ponga texto claro sobre un relleno claro.

---

## 5. Tabla de contraste (calculada)

AA exige **4.5:1** para texto normal y **3.0:1** para texto grande, iconos y componentes de interfaz.

| Superficie | `text` | `muted` | `accent` | `info` | `warning` | `danger` | `success` |
|---|---|---|---|---|---|---|---|
| `bg-deep` `#041A25` | 15.23 ✅ | 13.79 ✅ | 11.68 ✅ | 6.25 ✅ | 6.63 ✅ | 5.86 ✅ | 8.18 ✅ |
| `surface` `#053048` | 11.80 ✅ | 10.68 ✅ | 9.05 ✅ | 4.84 ✅ | 5.14 ✅ | 4.54 ✅ | 6.34 ✅ |
| `surface-raised` `#045686` | 6.70 ✅ | 6.07 ✅ | 5.14 ✅ | 2.75 ❌ | 2.92 ❌ | 2.58 ❌ | 3.60 ⚠️ |
| `surface-hover` `#0C6F9D` | 4.76 ✅ | 4.31 ⚠️ | 3.65 ⚠️ | 1.95 ❌ | 2.07 ❌ | 1.83 ❌ | 2.56 ❌ |

✅ AA texto · ⚠️ sólo texto grande e interfaz · ❌ no usar

### Lo que esta tabla obliga

**Los colores semánticos sólo viven sobre `bg-deep` y `surface`.** Sobre `surface-raised` fallan todos. Consecuencia práctica: **las insignias de estado no se ponen sobre tarjetas elevadas** con el color como texto — se usa relleno sólido con texto oscuro.

**`surface-hover` sólo admite `--arles-text`.** Es un estado transitorio, no una superficie de contenido.

**`surface-raised` es el techo para superficies con texto normal.** Todo lo más claro es decoración, borde o relleno.

---

## 6. Método (reproducible)

1. Decodificar el PNG completo (zlib + desfiltrado PNG).
2. Muestrear una de cada tres líneas, una de cada tres columnas.
3. Cuantificar a 5 bits por canal (32 768 cubos).
4. Convertir a HSL, agrupar por familia de tono.
5. Contraste con la fórmula de luminancia relativa de WCAG 2.2:

```
f(c) = c/12.92                  si c ≤ 0.03928
       ((c+0.055)/1.055)^2.4    en otro caso
L    = 0.2126·f(R) + 0.7152·f(G) + 0.0722·f(B)
ratio = (L_claro + 0.05) / (L_oscuro + 0.05)
```

**Estos cálculos se ejecutan en CI.** Ningún token puede publicarse sin su ratio verificado contra las superficies en las que se usa: una regresión de contraste debe romper la compilación, no descubrirse en una auditoría.

---

## 7. Las tres reglas

### 7.1 `#2CA4D4` nunca es relleno de una superficie con texto

Falla contra todo, blanco incluido (2.85:1).

**Es color de trazo, borde, foco, icono y dato.** Sobre `bg-deep` alcanza 6.25:1 y es perfectamente legible como texto.

### 7.2 El amarillo es luz, no señal

`#FCCC0C` tiene luminancia relativa **0.639** — más cerca del blanco (1.0) que del negro.

| Combinación | Ratio | |
|---|---|---|
| `#FCCC0C` sobre `#041A25` | **11.68:1** | ✅ |
| `#041A25` sobre `#FCCC0C` | **11.68:1** | ✅ botón de acento |
| Blanco sobre `#FCCC0C` | **1.52:1** | ❌ **prohibido** |

Invierte la intuición habitual. No es «color fuerte sobre claro», es **luz sobre oscuro**. Encaja perfectamente con el dark-first y **prohíbe de raíz el patrón «botón amarillo con texto blanco»**.

Lo mismo aplica a `#2CA4D4` como relleno: exige texto oscuro (6.25:1), nunca blanco (2.85:1).

### 7.3 El color nunca es el único portador de información

§19. Todo estado lleva **forma, icono o texto** además del color.

```
✕ Falló      (no sólo un punto rojo)
✓ Aceptado   (no sólo un punto verde)
⏸ Pausada    (no sólo un punto ámbar)
```

Un daltónico deuteranope debe poder operar ARLES completamente.

---

## 8. Uso del amarillo

El §18 avisa: si todo es amarillo, nada es importante.

**Se usa para:** la acción primaria de la pantalla (una por pantalla) · el anillo de foco · el dato clave del panel (progreso del envío de hoy) · el indicador de campaña activa.

**No se usa para:** bordes decorativos · encabezados de tabla · iconos generales · fondos de sección · más de un elemento en el mismo campo visual.

**Presupuesto:** menos del 5 % del área visible de cualquier pantalla. Es aproximadamente la proporción que ocupa en la referencia, y no es casualidad que funcione.

---

## 9. Profundidad

Dark-first por capas, **nunca negro absoluto** (§18):

```
bg-deep         #041A25   fondo de aplicación
  └ surface     #053048   paneles, barra lateral
      └ raised  #045686   tarjetas, filas seleccionadas, modales
          └ hover #0C6F9D estado transitorio
```

La separación entre capas se hace con **luminosidad y borde**, no con sombra. Las sombras sobre fondos muy oscuros son casi invisibles y cuestan rendimiento; un borde de `--arles-border` separa mejor y se renderiza igual en WebView2 y WKWebView (riesgo R-07).

---

## 10. Interpretación de la referencia artística

*La noche estrellada* aporta **concepto, atmósfera y profundidad** (§14). La referencia aporta **los colores**.

Se traduce abstractamente: profundidad por capas, contraste entre un campo frío dominante y un acento cálido escaso, ritmo en el espaciado.

**Ningún elemento literal** (§15): sin girasoles, sin cielos estrellados, sin pinceladas, sin texturas de óleo, sin degradados «artísticos».

El resultado buscado: **tecnológico, corporativo, premium, sobrio.**

---

## 11. Modo claro

**Fuera de alcance en v1.2.0.** El §18 pide explorar dark-first, y un segundo tema completo duplica el trabajo de verificación de contraste sin aportar al bucle central.

Lo que sí se hace para no cerrar la puerta: **todos los colores viven en tokens** (§17), así que añadir un tema claro en v1.3 es redefinir variables, no tocar componentes. Ningún componente contiene un hex literal.
