# Mapa de pantallas · dónde se toca cada cosa

> **Para qué es este documento.** Para cambiar el diseño sin buscar. Dice, para
> cada pantalla, qué archivo la dibuja, de dónde saca los datos, dónde están sus
> textos y qué comprobación salta si se rompe algo.
>
> Escrito el 29/09/2026, recorriendo la aplicación pantalla por pantalla en un
> navegador. Si una fila deja de ser cierta, la sonda del recorrido lo dice.

---

## 0 · Las tres reglas antes de tocar nada

**1 · Ni un color, ni un píxel, ni una duración sueltos** (§17). Todo sale de los
tokens. Se ven en `app/src/design/tokens.css`, y se usan así:

```css
color: var(--arles-text-muted);
gap: var(--arles-space-4);
```

Los nombres van **en inglés y con prefijo `--arles-`**. Escribir
`var(--espacio-4)` no da error: el navegador lo resuelve a nada y el elemento
sale sin margen. La fase 2 del validador lo caza — ha pasado dos veces.

**2 · Ni una palabra escrita en el componente** (§139). Todo texto sale de
`app/src/app/locales/es.ts` con `t('clave')`. Una clave que falta se enseña en
crudo: la pantalla diría `plantillas.titulo` en vez de «Plantillas».

**3 · Los componentes se reutilizan, no se copian.** Los nueve de
`app/src/design/componentes/` cubren casi todo. Si hace falta uno nuevo, entra
ahí y en el catálogo, no dentro de una pantalla.

---

## 1 · La estructura, de fuera adentro

```
app/src/
├── design/                 ← El sistema visual. Cambiar aquí cambia TODO.
│   ├── tokens.css              Colores, espacios, tipos, sombras, duraciones
│   ├── tipografia.css          La familia Mont y sus pesos
│   └── componentes/            Los nueve ladrillos + los cuatro estados
│
├── app/
│   ├── App.vue             ← El armazón: barra lateral, migas, contenido
│   ├── router.ts           ← Qué ruta abre qué pantalla, y su sección
│   ├── locales/es.ts       ← TODOS los textos
│   ├── nucleo.ts           ← La única puerta hacia Rust
│   ├── stores/             ← Los datos de cada pantalla
│   └── pantallas/          ← Una pantalla, un archivo
│
└── pruebas/sondas/         ← Lo que mide la app de verdad en un navegador
```

**Para un cambio de aspecto general** —otra paleta, otra tipografía, otros
redondeos— se toca `design/tokens.css` y nada más. Las trece pantallas cambian
solas, porque ninguna tiene un valor propio.

---

## 2 · Las trece pantallas

| Ruta | Archivo que la dibuja | De dónde saca los datos | Sección en la barra |
|---|---|---|---|
| `/inicio` | `PantallaInicio.vue` | `stores/arranque.ts` | Inicio |
| `/inicio/configuracion` | `PantallaConfiguracion.vue` | `stores/empresa.ts` | Inicio › Configuración |
| `/contactos` | `PantallaContactos.vue` | `stores/contactos.ts` | Contactos |
| `/contactos/importar` | `PantallaImportar.vue` | `stores/importacion.ts` | Contactos › Importar |
| `/contactos/supresion` | `PantallaSupresion.vue` | `stores/supresion.ts` | Contactos › Bajas |
| `/contactos/derechos` | `PantallaDerechos.vue` | `stores/derechos.ts` | Contactos › Derechos |
| `/campanas` | `PantallaCampanas.vue` | `stores/campanas.ts` | Campañas |
| `/campanas/plantillas` | `PantallaPlantillas.vue` | `stores/campanas.ts` | Campañas › Plantillas |
| `/campanas/nueva` | `AsistenteDeCampana.vue` | `stores/campanas.ts` | Campañas (sin entrada fija) |
| `/remitentes` | `PantallaRemitentes.vue` | `stores/remitentes.ts` | Remitentes |
| `/remitentes/whatsapp` | `PantallaWhatsApp.vue` | `stores/whatsapp.ts` | Remitentes › WhatsApp |
| `/actividad` | `app/PantallaPendiente.vue` | — | Actividad *(andamio)* |
| `/ajustes` | `PantallaAjustes.vue` | `stores/interfaz.ts` | Ajustes |

**Las secciones son seis y no se tocan** (P-11). Lo que cuelga de ellas son
subsecciones, y se declaran en `router.ts` con `meta: { seccion, subseccion }`.
Una séptima sección rompería la barra lateral y su comprobación.

**Un formulario no es una pantalla.** `FormularioDeContacto.vue` se abre desde
Contactos y no tiene ruta propia — por eso la comprobación de «toda pantalla se
puede abrir» no lo exige.

---

## 3 · Dónde vive cada texto

Todo en `app/src/app/locales/es.ts`, agrupado por pantalla. Las claves siguen la
ruta: lo de la pantalla de plantillas está bajo `plantillas:`, lo del número de
WhatsApp bajo `numeroDeWhatsapp:`.

Tres grupos que **no** son de una pantalla y conviene conocer:

| Grupo | Qué es | Quién lo nombra |
|---|---|---|
| `nav.*` | Los rótulos de la barra y de las migas | `router.ts`, por `meta.subseccion` |
| `error.*` | Las tres partes de cada error (§95) | Rust, con `clave_i18n()` |
| `campana.estado.*`, `whatsapp.modo.*`, `plantilla.variable.*` | Los rótulos de valores del núcleo | Rust, con `clave_i18n()` |

**Los del tercer grupo son los que más se han roto.** Rust dice **cómo se llama**
el texto, no si alguien lo escribió, así que una clave que falta compila
perfectamente y se enseña en crudo. Ha pasado dos veces: los modos del canal y
las variables de plantilla.

---

## 4 · Cómo llega un dato a la pantalla

Siempre el mismo camino, sin excepciones:

```
Rust (comando)  →  nucleo.ts (invocar)  →  store  →  pantalla
```

**La pantalla nunca llama a Rust directamente.** Si hace falta un dato nuevo:

1. El comando en `crates/arles-app/` (o el que ya exista).
2. Registrarlo en `crates/arles-app/src/lib.rs`.
3. Una función en el store que lo invoque.
4. La pantalla lo lee del store.
5. **Y una respuesta en `herramientas/vista-previa/simulador.js`**, o la vista
   previa enseñará un error donde debería haber datos.

El paso 5 se olvidó once veces seguidas, y por eso cinco pantallas recibían con
«Ocurrió un error». Ahora lo caza la sonda del recorrido.

---

## 5 · Qué salta si se rompe algo

| Si se rompe | Lo dice | Cómo se corre |
|---|---|---|
| Un token inventado | `validar.py --fase 2` | `python3 herramientas/validar/validar.py --fase 2` |
| Un texto que falta y sale en crudo | sonda del recorrido | `npm run sonda:recorrido` |
| Contraste, foco, desbordes | sonda de auditoría | `npm run sonda:auditoria` |
| Una pantalla sin ruta | `validar.py --fase W` | `python3 herramientas/validar/validar.py --fase W` |
| Una pantalla que recibe con un error | sonda del recorrido | `npm run sonda:recorrido` |
| Un adjetivo donde va una cifra (§94) | sonda del recorrido | `npm run sonda:recorrido` |
| Decir «entregado» (§65) | sonda del recorrido | `npm run sonda:recorrido` |
| Una pantalla que empieza en otro borde | sonda del recorrido | `npm run sonda:recorrido` |

**Las trece arrancan en el mismo borde izquierdo**, y hay una comprobación que
lo mide. Se escribió el 29/09/2026 después de encontrarlo a ojo: Ajustes
llevaba `max-width` sobre `.pantalla` —que el armazón centra con
`margin-inline: auto`— y su título empezaba 300 px más a la derecha que el de
las otras doce. **Cada pantalla por separado se veía bien.** Si hace falta
limitar la medida de lectura, va en el bloque de texto, nunca en `.pantalla`;
y `--arles-medida` sí se declara en `.pantalla`, porque es lo que lee la sonda
de plegado.

**Las sondas necesitan Chromium.** Sin él, el validador las marca como
**omitidas**, que no es lo mismo que pasadas y así lo dice.

---

## 6 · Un cambio de diseño, paso a paso

**Cambiar la paleta entera:** `design/tokens.css`. Después,
`validar.py --fase 2`, que mide el contraste real sobre lo que pinta el
navegador, no sobre el token.

**Cambiar una pantalla:** su `.vue` y su grupo de textos en `es.ts`. Después,
`npm run sonda:recorrido` y `npm run sonda:auditoria`.

**Mover algo de sitio en la navegación:** `router.ts` y `nav.sub.*` en `es.ts`.
Hay una prueba que exige que toda subsección tenga nombre en español.

**Añadir una pantalla:** el `.vue`, la ruta en `router.ts`, sus textos, y
añadirla al `RECORRIDO` de `pruebas/sondas/recorrido.mjs`. **Sin lo último, la
sonda no la mira y su resultado en verde no dice nada de ella.**

**Y siempre, antes de empujar:** las ocho fases del validador, no solo la que se
tocó. Está explicado en `CLAUDE.md` §6, con los cuatro fallos que costó
aprenderlo.

---

## 7 · Lo que hoy no se puede cambiar desde la interfaz

Para no prometer lo que no hay:

- **La audiencia de una campaña no se filtra.** Etiquetas y listas están en el
  esquema y no tienen pantalla (§40).
- **Una campaña creada no se edita.**
- **Una versión de plantilla no se toca**, y eso es a propósito: guardar crea
  otra, para que una campaña activa siga mandando el texto que se aprobó.
- **Actividad es andamio** hasta su fase. Dice qué traerá y enlaza a las dos
  cosas que hoy sí se pueden ver —las cifras de cada campaña y la lista de
  bajas—, porque una pantalla de la que sólo se sale por la barra lateral es un
  callejón, y la sonda del recorrido la contaba con **cero acciones**.

---

## 8 · Lo que sólo se ve recorriendo, y cómo se caza ahora

Cuatro defectos de esta revisión no los podía encontrar ninguna prueba de
componente, porque **ninguno estaba dentro de una pantalla**: vivían entre dos.
Quedan aquí porque el siguiente cambio de diseño los puede repetir.

| Lo que pasaba | Por qué ninguna prueba lo veía | Qué lo caza ahora |
|---|---|---|
| Ajustes empezaba 300 px a la derecha | cada pantalla, sola, se ve bien | garantía 8 del recorrido |
| Las insignias neutras llevaban un reloj | el icono existía y se pintaba | prueba del neutro sin icono |
| Actividad hablaba de «roadmap» y «andamiaje», con el texto dentro del componente | era texto legítimo, en español y sin claves sueltas | la revisión a ojo; §139 lo prohibía y nadie lo medía |
| Actividad no tenía salida | la sonda pedía 0 acciones mínimas | ahora pide 1 |

Y dos de las comprobaciones **eran ellas mismas el defecto**: la que compara el
simulador con el núcleo se inventaba el nombre de una clave (`primeracampana`
en vez de `campana`) y su patrón no sabía leer una fila que contuviera `[]`, así
que llevaba tiempo sin mirar el paso de Contactos sin decirlo. Es el mismo
patrón del §2: **una comprobación que nunca se ha visto fallar no está
verificada**.
