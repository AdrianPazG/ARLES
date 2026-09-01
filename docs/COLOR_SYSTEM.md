# Sistema de color

La paleta de ARLES RELAY se deriva **exclusivamente** de `REFERENCIA_DE_COLOR/farm-lifestyle-digital-art.jpg`.
No se usó ninguna paleta externa ni ninguna imagen descargada de internet.

Reproducible con `python3 tools/extract_palette.py`.

## 1. El archivo

| Propiedad | Valor |
|---|---|
| Formato real | **PNG** (pese a la extensión `.jpg`) — verificado por firma de bytes |
| Resolución | 2320 × 3080 px = 7,145,600 px |
| Densidad | 240 dpi |
| Origen declarado | `DigitalSourceType = trainedAlgorithmicMedia` — imagen generada por IA |
| Edición | Adobe Photoshop 25.7 (macOS), mayo 2024 |

Que sea PNG es relevante: es sin pérdida, así que los colores son exactos y no hay artefactos de
compresión contaminando el muestreo, como advierte §13.

**Implicación de licencia.** Extraer valores cromáticos no crea obra derivada: los colores no son objeto
de derecho de autor. La paleta es usable sin restricción. Lo que **no** debe hacerse es incluir la imagen
misma en el producto, el instalador o el material comercial sin verificar su procedencia.

## 2. Análisis por región

Mediana por zona de la composición. §16 advierte que no se seleccionen colores solo por frecuencia de
píxel, por eso el primer análisis es composicional. Se usa mediana, no media: resiste mejor los atípicos.

| Región | Mediana | L* | Rol |
|---|---|---:|---|
| Cielo cenital (0–8 %) | `#055784` | 35.0 | Fondo profundo |
| Cielo profundo (12–22 %) | `#06638F` | 39.5 | Superficie estructural |
| Cielo medio (25–40 %) | `#0B749F` | 45.7 | Azul dominante |
| Cielo bajo / azur (45–55 %) | `#2AA4CE` | 62.9 | Interacción e información |
| Nube brillante (50–58 %) | `#9BCFD6` | 79.9 | Texto secundario |
| Horizonte dorado (60–64 %) | `#E0B926` | 76.5 | Acento de energía |
| Campo medio (68–76 %) | `#AB6917` | 50.5 | Advertencia controlada |
| Sombras del follaje (78–88 %) | `#6E521B` | 36.8 | Ocre de profundidad |

## 3. Agrupamiento k-means en CIELAB

446,600 muestras, K = 12, espacio perceptual.

| % imagen | HEX | L* | a* | b* | Familia |
|---:|---|---:|---:|---:|---|
| 16.66 % | `#0F759E` | 45.9 | −11.2 | −29.6 | Azul dominante |
| 11.76 % | `#055C88` | 37.0 | −5.7 | −30.8 | Azul profundo |
| 11.65 % | `#2B9DC3` | 60.3 | −17.6 | −28.5 | Azur / cian |
| 11.47 % | `#75B2C0` | 69.2 | −15.7 | −12.9 | Azul claro |
| 8.56 % | `#D5DBCB` | 86.5 | −4.6 | 7.1 | Crema / nube |
| 7.83 % | `#B96A10` | 52.6 | 25.9 | 56.5 | Ocre |
| 6.52 % | `#13303D` | 18.5 | −5.4 | −11.0 | Oscuro azulado |
| 6.23 % | `#F5D21F` | 84.9 | −2.8 | 79.7 | **Amarillo girasol** |
| 5.08 % | `#E79A13` | 69.7 | 19.6 | 70.7 | Oro |

Acentos de alta pureza, aislados por saturación en sus zonas propias:

- Pétalo en el cielo `#F4E027` (38,736 px), pico `#FCED4A`
- Girasol saturado `#F0AC0E` (766,946 px)
- Ocre en sombra `#9A580F` (449,894 px)
- Azul más profundo `#01212D` (percentil 1 de los azules)
- Crema de nube `#E6E4D5`, pico `#F7F1E8`

## 4. Tokens

```css
:root {
  /* Profundidad — cuatro capas de azul, no negro */
  --arles-background-deep:  #04101B;
  --arles-background:       #071C2C;
  --arles-surface:          #0B2739;
  --arles-surface-raised:   #103247;
  --arles-surface-interactive: #17405A;

  /* Bordes — decorativos, nunca indicador de estado */
  --arles-border:           #1B3B52;
  --arles-border-strong:    #27516D;

  /* Azules de la referencia */
  --arles-blue-deep:        #055C88;
  --arles-blue-primary:     #0F759E;  /* SOLO RELLENO — ver §5 */
  --arles-azure:            #2B9DC3;
  --arles-blue-light:       #8CC5D4;

  /* Energía */
  --arles-yellow:           #F5D21F;
  --arles-yellow-hi:        #FCED4A;
  --arles-gold:             #F0AC0E;
  --arles-ochre:            #9A580F;

  /* Texto */
  --arles-text-primary:     #F2F6F8;
  --arles-text-secondary:   #AFC6D6;
  --arles-text-tertiary:    #8AA6B8;
  --arles-cream:            #E6E4D5;

  /* Semánticos */
  --arles-success:          #3DC98F;  /* introducido */
  --arles-warning:          #F0AC0E;  /* de la referencia */
  --arles-danger:           #F0645C;  /* introducido */
  --arles-info:             #5FB8DC;  /* de la referencia */

  --arles-focus:            #F5D21F;
}
```

**La referencia no contiene verde ni rojo.** Éxito y peligro tuvieron que introducirse, elegidos para
convivir con la paleta sin competir con el oro. Se declara porque §169 fija que la referencia manda en
decisiones visuales, y aquí hubo que salirse de ella por una razón funcional: no se puede señalar un error
con el mismo color que una advertencia.

## 5. Contraste verificado — WCAG 2.2

Ratios calculados sobre luminancia relativa. AA exige 4.5:1 para texto normal, 3.0:1 para texto grande y
elementos no textuales.

| Token | HEX | vs `#04101B` | vs `#071C2C` | vs `#0B2739` | vs `#103247` | vs `#17405A` |
|---|---|---:|---:|---:|---:|---:|
| text-primary | `#F2F6F8` | 17.64 | 15.94 | 14.16 | 12.31 | 10.07 |
| text-secondary | `#AFC6D6` | 10.84 | 9.80 | 8.71 | 7.56 | 6.19 |
| text-tertiary | `#8AA6B8` | 7.51 | 6.79 | 6.03 | 5.24 | 4.29 † |
| yellow | `#F5D21F` | 12.89 | 11.66 | 10.36 | 9.00 | 7.36 |
| gold | `#F0AC0E` | 9.69 | 8.76 | 7.78 | 6.76 | 5.53 |
| azure | `#2B9DC3` | 6.13 | 5.54 | 4.92 | 4.28 † | 3.50 † |
| blue-light | `#8CC5D4` | 10.09 | 9.12 | 8.11 | 7.04 | 5.76 |
| **blue-primary** | `#0F759E` | 3.71 † | 3.35 † | **2.98 ✗** | **2.59 ✗** | **2.12 ✗** |
| success | `#3DC98F` | 9.09 | 8.21 | 7.30 | 6.34 | 5.19 |
| danger | `#F0645C` | 6.10 | 5.52 | 4.90 | 4.26 † | 3.49 † |
| info | `#5FB8DC` | 8.56 | 7.74 | 6.88 | 5.97 | 4.89 |

† solo texto grande / elementos no textuales · ✗ insuficiente

Texto oscuro `#04101B` sobre acentos:

| Acento | Con `#04101B` | Con `#F2F6F8` |
|---|---:|---:|
| `#F5D21F` yellow | **12.89** | 1.37 ✗ |
| `#F0AC0E` gold | **9.69** | 1.82 ✗ |
| `#3DC98F` success | **9.09** | 1.94 ✗ |
| `#F0645C` danger | **6.10** | 2.89 ✗ |
| `#2B9DC3` azure | **6.13** | 2.88 ✗ |
| `#0F759E` blue-primary | 3.71 † | **4.76** |

## 6. Reglas de uso

1. **Un solo botón amarillo por pantalla.** El amarillo marca *la* acción, no *una* acción (§18).
2. **El anillo de foco es siempre girasol.** Los bordes dan 1.32:1 — invisibles como indicador. El girasol
   da 10.36:1 y supera el mínimo de 3:1 para elementos no textuales.
3. **Texto oscuro sobre todo acento.** Crema sobre girasol da 1.37:1. Sin excepciones.
4. **`--arles-blue-primary` nunca es tinta.** 2.98:1 sobre superficie. Como relleno con texto crema
   funciona: 4.76:1. Es el color más abundante de la referencia y solo sirve como superficie.
5. **Cero colores literales en componentes.** Todo pasa por token (§17). Es también lo que hace viable la
   marca blanca futura sin refactorizar.
6. **Ningún estado se comunica solo con color** (§19). Cada chip lleva punto, texto y forma.

## 7. Verificación continua

El contraste debe verificarse **en CI**, no a ojo. Una prueba que recorra los pares token-sobre-fondo
declarados y falle si alguno cae bajo su umbral. Es la única forma de que la accesibilidad no se degrade
con el tiempo.
