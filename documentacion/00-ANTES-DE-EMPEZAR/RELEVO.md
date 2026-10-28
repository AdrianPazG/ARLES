# Relevo entre sesiones

**Para:** quien retome ARLES sin el contexto de la sesión anterior — otra
persona, u otro modelo.
**Última actualización:** 2026-10-27

Dirección puede cambiar de modelo por ahorro de créditos o por cualquier otro
motivo. **Este archivo existe para que eso no cueste nada.**

No es un resumen del proyecto: eso es
[`ESTADO_ACTUAL.md`](../07-entrega/ESTADO_ACTUAL.md). Es la lista de lo que se
pierde al cambiar de sesión y de dónde recuperarlo.

---

## 1 · Los primeros veinte minutos

En este orden, y sin saltarse ninguno:

1. [`CLAUDE.md`](../../CLAUDE.md) — las reglas del proyecto.
2. [`PROHIBICIONES.md`](PROHIBICIONES.md) — lo que nunca se hace.
3. [`METODO.md`](METODO.md) — cómo se verifica. **Es lo que más se salta y lo
   que más cuesta.**
4. [`ESTADO_ACTUAL.md`](../07-entrega/ESTADO_ACTUAL.md) — dónde está el trabajo.
5. [`REGISTRO-DE-REQUISITOS.md`](../10-requisitos/REGISTRO-DE-REQUISITOS.md) —
   qué se ha pedido y qué bloquea.

Y después, **antes de tocar una línea**, comprobar que el proyecto está sano:

```bash
cd /ruta/a/ARLES
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd app && npm test && npm run lint && npx vue-tsc --noEmit -p tsconfig.json
cd .. && for f in 0 1 2 3 4 5 6 W 7 8; do python3 herramientas/validar/validar.py --fase $f --rapido; done
```

**Si algo sale en rojo antes de tocar nada, eso es lo primero** — y hay que
decirlo antes de empezar cualquier otra cosa. Nunca se construye encima de un
rojo heredado sin nombrarlo.

---

## 2 · Lo que no está en ningún archivo y hay que preguntar

Esto se pierde al cambiar de sesión y **no se adivina**:

| Qué | Cómo se recupera |
|---|---|
| El enlace de la sesión para la línea `Claude-Session:` del commit | **Cada sesión tiene el suyo.** No se copia de aquí ni de un commit anterior: apuntaría a una conversación que no explica ese commit |
| Qué estaba a medias cuando se cortó | `git log -3` y `git status`. Si hay trabajo sin confirmar, **leerlo antes de seguir** |
| Qué decidió Dirección en la última conversación | Está en `10-requisitos/REGISTRO-DE-REQUISITOS.md` si se anotó. Si no aparece, **se pregunta** |

---

## 3 · Lo que se da por sabido y no se puede dar por sabido

Las seis cosas que han costado trabajo rehecho. Están en `CLAUDE.md`, pero aquí
juntas porque son las que un relevo repite:

1. **Se responde SIEMPRE en español.** Todo: respuestas, planificación,
   comentarios del código, documentación, mensajes de commit.
2. **La rama es `claude/loving-faraday-5chmmy`.** No se empuja a otra sin
   permiso explícito.
3. **No se abre un pull request** salvo que Dirección lo pida con esas palabras.
4. **Cada commit termina con las dos líneas** de coautoría y sesión, y la de
   sesión es **la de esta sesión**.
5. **Ningún identificador de modelo** en nada que acabe en el repositorio.
6. **Para restaurar una ruptura se usa una copia hecha antes, nunca
   `git checkout`.** Ha costado trabajo rehecho dos veces.

---

## 4 · Los seis patrones de fallo de este proyecto

Un relevo que no los conozca los repetirá. Cada uno tiene su recuento porque la
frecuencia es el dato:

| # | El patrón | Veces | Dónde está contado |
|---|---|---|---|
| 1 | **Una pieza correcta y probada que nadie llama** | 25+ | `CLAUDE.md`, «El patrón que ya ha aparecido ocho veces» |
| 2 | **Una comprobación que se caza a sí misma** en un comentario | 10 | `METODO.md` §1 |
| 3 | **Una lista escrita a mano** que dice cubrir una regla y cubre una parte | 11 de 17 hallazgos | las cuatro auditorías del §140 |
| 4 | **Un documento que afirma de más** | 9 anotados | `10-requisitos/REGISTRO-DE-REQUISITOS.md`, «Documentos que afirmaban de más» |
| 5 | **Una afirmación de verificación falsa** — «verificada rompiéndola» sin haberlo hecho | 6+ | la 8.3 bis, la 8.6 E, el tramo 3b |
| 6 | **Una ruptura mal elegida** que deja el síntoma en pie o no compila | 4 | `METODO.md` §1 |

> **El quinto es el que más duele**, porque es el único que no se puede
> distinguir de lo correcto sin volver a correrlo. Si un comentario dice
> «verificada rompiéndola» y hay dudas, **se vuelve a romper.**

> **El segundo llegó a diez el 27/10/2026, y el sitio importa:** el
> `DESIGN_SYSTEM.md` reescrito ese día escribía el nombre de un token
> inventado para ilustrar que un nombre mal escrito no da error — dentro de la
> misma sección que advierte de esta trampa, y en el documento cuya
> comprobación nueva lee precisamente esos nombres. Lo cazó **medir antes de
> escribir la comprobación**, no después.
>
> La conclusión ya no es una anécdota y está en `CLAUDE.md`: **saberlo no
> basta.** Se quitan los comentarios por defecto en cualquier comprobación que
> busque una cadena en un archivo, y no se escribe un contraejemplo donde una
> comprobación lo vaya a leer como afirmación.

---

## 5 · Lo que está a medias ahora mismo

**Esta sección se actualiza al cerrar cada tramo.** Si la fecha de arriba es de
hace más de una semana, no te la creas: mira `git log` y `ESTADO_ACTUAL.md`.

**D-WA está cerrado** (27/10/2026): se puede **crear y lanzar una campaña
entera por WhatsApp** desde la aplicación. Seis tramos, todos empujados:

| Tramo | Qué | Commit |
|---|---|---|
| 1 | El motor recorre las etapas de WhatsApp | `bb1051e` |
| 2 | El contenido de WhatsApp viaja hasta el proveedor | — |
| alta | El número se da de alta con un solo dato | `4dfd084` |
| 3a | Crear y activar una campaña de WhatsApp, por dentro | `3501757` |
| 3b | El canal cruza la frontera, nada mezclado entra | `52ce92b` |
| 3c-1 | El semáforo de calidad se llena | `0b89835` |
| 3c-2a | Las plantillas de Meta se registran y se leen | `05d0975` |
| 3c-2b | El asistente elige canal | este |

**Lo que WhatsApp sigue sin hacer, y no es un defecto:**

1. **Contar las respuestas.** Meta las manda por *webhook* y ARLES no tiene
   ninguna URL a la que Meta pueda llamar (`arles_whatsapp::entrante`). La cifra
   de «cuánta gente contestó» es cero en WhatsApp hasta que exista el «relay
   ciego» que Dirección decidió el 29/09/2026.
2. **Atender un «BAJA» automáticamente.** Ni hay transporte, ni se lee el texto
   de una respuesta (D-7A). Se apunta **a mano** en Contactos › Bajas, y está
   dicho con esas palabras en la pantalla de WhatsApp, en la guía del canal y en
   el manual. Decisión de Dirección del 27/10/2026.

**Cerrado el sistema de diseño como documento** (27/10/2026): el
[`DESIGN_SYSTEM.md`](../05-diseno/DESIGN_SYSTEM.md) reescrito con el código
delante —los 89 tokens y las 16 piezas, que antes documentaba a medias— y con
`el_sistema_de_diseno_dice_lo_que_hay` comparándolo con los dos CSS y con la
carpeta de componentes en las dos direcciones. Tenía **cinco afirmaciones
falsas** desde la Fase 2, anotadas como DOC-1 a DOC-5 en el registro de
requisitos.

**Cerrado el registro de hallazgos de diseño** (28/10/2026):
[`HALLAZGOS-DE-DISENO.md`](../10-requisitos/HALLAZGOS-DE-DISENO.md), llenado
abriendo la demostración y mirando las dieciocho pantallas con las seis sondas
en verde. **Siete hallazgos y tres descartados por falsos** — los dos listados,
porque uno falso cuesta más que uno ausente.

**D-01 cerrado el 28/10/2026**: Inicio ya no dice que el producto no está
terminado. **Quedan seis abiertos**, D-02 a D-07, y cada uno lleva anotado qué
comprobación lo haría imposible.

Y por el camino, el noveno documento que afirmaba de más: `CLAUDE.md` §1 bis
prometía que la fase 0 vigilaba que todo documento estuviera en el índice, y
**no lo vigilaba nadie**. Ahora sí.

**Lo siguiente, sin empezar:** la **Fase 9**, con lo que las auditorías dejaron
abierto — sacar la activación a segundo plano con avance (73.5 s con 451 196
destinatarios, y se rompe alrededor de los 30 000), el filtro de la lista de
contactos (C-2), y **R-07**: nadie ha abierto ARLES en un Mac.

## 6 · Cómo se cierra un tramo

```
un tramo → se verifica → se empuja → se explica → se pregunta
```

Y las cuatro de la lista final de [`METODO.md`](METODO.md): quién lo llama, qué
documento deja de ser verdad, qué poka-yoke hace falta, y si está todo en su
carpeta.
