# Design System

**Proyecto:** ARLES RELAY I · v1.2.0
**Depende de:** `COLOR_SYSTEM.md` · `TIPOGRAFIA.md`
**Estado:** ✅ **desbloqueado por D-5** — se desarrolla con Mont; la licencia se resuelve antes de la demo

---

## 1. Principios

**Es una aplicación de escritorio, no una web.** De 1366×768 a 4K, con escalado de Windows del 100 % al 200 % (§22). Ratón, trackpad y teclado. Nada de patrones móviles: sin menú hamburguesa, sin gestos, sin diseño adaptable a pantallas de teléfono (§4).

**Densidad alta.** El usuario pasa horas con tablas de contactos. Las interfaces espaciadas pensadas para móvil obligan a desplazarse tres veces para ver lo que cabría de una.

**Tokens, siempre.** Ningún componente contiene un hex, un píxel suelto ni una duración literal (§17).

**Sobrio.** Tecnológico y premium, no decorado. Cada elemento visible justifica su presencia.

---

## 2. Rejilla y espaciado

Base de **4 px**.

```css
--arles-space-1:  4px;    --arles-space-5: 24px;
--arles-space-2:  8px;    --arles-space-6: 32px;
--arles-space-3: 12px;    --arles-space-7: 48px;
--arles-space-4: 16px;    --arles-space-8: 64px;
```

**Ventana mínima: 1120 × 720.** Por debajo, el contenido se recorta en vez de reorganizarse: reorganizar es un patrón adaptable y aquí no aplica.

**Diseño base:**

```
┌────────────────────────────────────────────────┐
│ Barra de título (nativa)                       │
├──────────┬─────────────────────────────────────┤
│ Nav      │ Encabezado de pantalla              │
│ 240px    ├─────────────────────────────────────┤
│ fija     │ Contenido                           │
│          │                                     │
│          ├─────────────────────────────────────┤
│ Pie      │ Barra de estado (motor)             │
└──────────┴─────────────────────────────────────┘
```

La barra lateral es **fija, no colapsable**. Seis destinos caben sin problema y un panel que se pliega añade un estado que el usuario tiene que gestionar sin ganar nada en una pantalla de escritorio.

---

## 3. Elevación

Cuatro capas por **luminosidad y borde**, no por sombra:

| Capa | Token de fondo | Borde |
|---|---|---|
| Fondo | `--arles-bg-deep` | — |
| Panel | `--arles-surface` | `--arles-border` |
| Tarjeta, modal | `--arles-surface-raised` | `--arles-border-strong` |
| Hover | `--arles-surface-hover` | `--arles-border-strong` |

**Por qué no sombras.** Sobre `#041A25` una sombra es casi invisible: no hay contraste donde proyectarla. Además, las sombras son uno de los puntos donde WebView2 y WKWebView divergen visiblemente (riesgo R-07), y cuestan rendimiento al desplazar tablas largas. Un borde separa mejor, se renderiza idéntico y es gratis.

Excepción: los **modales** sí llevan sombra, pero su función ahí es oscurecer el fondo, no elevar.

---

## 4. Radios y trazos

```css
--arles-radius-sm:  4px;   /* insignias, entradas */
--arles-radius-md:  6px;   /* botones, tarjetas */
--arles-radius-lg:  8px;   /* modales, paneles */
--arles-border-width: 1px;
--arles-focus-width:  2px;
```

Radios contenidos. Los radios grandes leen como software de consumo; ARLES es una herramienta de trabajo.

---

## 5. Foco — no negociable

```css
:focus-visible {
  outline: var(--arles-focus-width) solid var(--arles-accent);
  outline-offset: 2px;
}
```

`#FCCC0C` como anillo de foco, verificado contra cada superficie:

| Superficie | Ratio | AA no textual (3.0) |
|---|---|---|
| `bg-deep` | 11.68 | ✅ |
| `surface` | 9.05 | ✅ |
| `surface-raised` | 5.14 | ✅ |
| `surface-hover` | 3.65 | ✅ |

**Nunca `outline: none` sin sustituto visible.** Es la regla que más a menudo se rompe en una revisión de diseño y la que más rompe la navegación por teclado (§100).

---

## 6. Componentes

### Botones

| Variante | Fondo | Texto | Uso |
|---|---|---|---|
| **Primario** | `--arles-accent` | `--arles-text-on-accent` | **Uno por pantalla** |
| Secundario | transparente | `--arles-text` | Acciones habituales |
| Sutil | transparente | `--arles-text-muted` | Terciarias |
| Peligro | `--arles-danger` | `--arles-text-on-accent` | Detener, eliminar |

**El botón primario lleva texto oscuro.** 11.68:1. Con texto blanco sería 1.52:1 — ilegible (`COLOR_SYSTEM.md` §7.2).

Altura 32 px (compacto) y 40 px (normal). Nunca menos de 32: el §22 pide escalado hasta 200 % y los objetivos pequeños se vuelven imprecisos.

### Entradas

Fondo `--arles-surface`, borde `--arles-border`, texto `--arles-text` (11.80:1).

**El error va debajo, con icono y texto**, nunca sólo con borde rojo (§19, regla 7.3).

### Tablas — el componente crítico

Es donde el usuario pasa la mayor parte del tiempo.

- **Virtualización siempre** (TanStack Virtual, ADR-0007). 500 000 filas.
- Altura de fila: 36 px compacta, 44 px cómoda. Preferencia del usuario.
- Encabezado pegajoso, ordenación por columna con indicador de dirección.
- Filas alternas con `--arles-surface` sobre `--arles-bg-deep`: suficiente para guiar el ojo, sin llegar a rayado.
- **Selección con `--arles-surface-raised` más un borde izquierdo de acento.** No sólo color: regla 7.3.
- Navegación completa por teclado: flechas, `Inicio`/`Fin`, `Espacio` para seleccionar, `Mayús`+flechas para rango.
- La columna de acciones fija a la derecha.

### Insignias de estado

**Relleno sólido con texto oscuro**, no color de texto sobre transparente. Motivo: sobre `surface-raised` los colores semánticos fallan todos (2.58–2.92). Con relleno sólido y `--arles-text-on-accent`, el contraste es siempre el del relleno contra el texto oscuro.

Cada insignia lleva **icono además del color**.

### Modales

Máximo 640 px de ancho. Fondo oscurecido. Foco atrapado dentro. `Esc` cierra, salvo en acciones destructivas, que exigen decisión explícita.

**Detener una campaña abre un modal que dice cuántos mensajes quedarán sin enviar.** Un número concreto, no «¿estás seguro?».

---

## 7. Movimiento

```css
--arles-duration-fast:   120ms;   /* hover, foco */
--arles-duration-normal: 200ms;   /* paneles, modales */
--arles-duration-slow:   320ms;   /* transiciones de pantalla */
--arles-ease: cubic-bezier(0.2, 0, 0.2, 1);
```

**Con propósito** (§98): mostrar progreso, orientar en una transición, confirmar una acción. **Nunca decorativo.**

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

Obligatorio, no opcional.

**Nada parpadea.** Ni siquiera para llamar la atención sobre un error: el parpadeo es un riesgo de accesibilidad y una molestia en sesiones largas.

---

## 8. Los cuatro estados de cada pantalla (§97)

Ninguna pantalla se da por terminada sin los cuatro.

**Vacío.** Explica qué es esto y da la acción para empezar. No un dibujo con «Nada por aquí».

> **Aún no hay campañas.**
> Una campaña envía un mensaje a una lista de contactos, respetando los límites que definas.
> `[Crear campaña]`

**Cargando.** Esqueleto con la forma del contenido real, no un girador centrado. Si supera 2 s, se explica qué está pasando.

**Error.** Qué pasó, cómo arreglarlo, **y qué está a salvo** (§95). La tercera parte es la que falta en casi todo el software y la que importa cuando alguien acaba de ver fallar una campaña.

> **No se pudo conectar con smtp.gmail.com.**
> Revisa tu conexión a internet y que el puerto 587 no esté bloqueado.
> **Tu campaña está en pausa, no cancelada.** Los 1 240 envíos pendientes se reanudarán al restablecerse la conexión.

**Éxito.** Breve, concreto, con el siguiente paso si lo hay. Sin celebraciones (§94).

---

## 9. Iconografía

Trazo lineal, 1.5 px, rejilla de 20 px. Sin relleno, sin color propio: heredan `currentColor`.

Biblioteca base **Lucide** (ISC, sin restricciones), con iconos propios sólo donde haga falta. No se usa un set con marca reconocible: contradiría el §15.

Todo icono que comunique estado va **acompañado de texto**.

---

## 10. Accesibilidad — lo que se verifica

- [ ] Contraste AA en todos los pares texto/fondo, **calculado en CI**
- [ ] Foco visible en todo elemento interactivo
- [ ] Navegación completa por teclado, orden de tabulación lógico
- [ ] Ningún estado comunicado sólo por color
- [ ] `prefers-reduced-motion` respetado
- [ ] Objetivos de al menos 32 px, verificados a escalado 200 %
- [ ] Roles y nombres ARIA en componentes compuestos
- [ ] Regiones activas para el progreso del motor
- [ ] Sin desbordamiento horizontal a 1120 px
- [ ] **Verificado en WebView2 y WKWebView** (riesgo R-07)

La última línea es la que más a menudo se olvida y la que ADR-0001 nos obliga a sostener.

---

## 11. Tokens: fuente única

Los colores **no se escriben aquí ni en ningún componente**. Viven en [`herramientas/design-tokens/tokens.json`](../../herramientas/design-tokens/README.md) y de ahí se genera el CSS que consume el frontend:

```bash
./herramientas/design-tokens/generar.py --todo
```

`--verificar` comprueba los contratos de contraste y falla la compilación ante una regresión — es el paso 5 del CI de `ESTRATEGIA_QA.md` §6.

**Para añadir un color:** se edita `tokens.json`, se declara su contrato de contraste, y se regenera. Si el contraste no pasa, el color no entra.

## 12. Estado de la tipografía

✅ **Desbloqueado por D-5.** Se desarrolla con Mont: escala tipográfica, logotipo y componentes.

La puerta de la licencia se traslada a **antes de la demo** — el binario de la fuente no sale del equipo de desarrollo hasta resolver D-3. El plan B de `TIPOGRAFIA.md` §3 sigue listo y cuesta una línea: `--arles-font-family`.
