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
| 2 | **Una comprobación que se caza a sí misma** en un comentario | 9 | `METODO.md` §1 |
| 3 | **Una lista escrita a mano** que dice cubrir una regla y cubre una parte | 11 de 17 hallazgos | las cuatro auditorías del §140 |
| 4 | **Un documento que afirma de más** | 5+ | el índice decía «Fase 0» con la Fase 8 cerrada |
| 5 | **Una afirmación de verificación falsa** — «verificada rompiéndola» sin haberlo hecho | 6+ | la 8.3 bis, la 8.6 E, el tramo 3b |
| 6 | **Una ruptura mal elegida** que deja el síntoma en pie o no compila | 4 | `METODO.md` §1 |

> **El quinto es el que más duele**, porque es el único que no se puede
> distinguir de lo correcto sin volver a correrlo. Si un comentario dice
> «verificada rompiéndola» y hay dudas, **se vuelve a romper.**

---

## 5 · Lo que está a medias ahora mismo

**Esta sección se actualiza al cerrar cada tramo.** Si la fecha de arriba es de
hace más de una semana, no te la creas: mira `git log` y `ESTADO_ACTUAL.md`.

**D-WA · el canal de WhatsApp como canal entero.** Tres tramos cerrados y
empujados, uno abierto:

| Tramo | Qué | Estado |
|---|---|---|
| 1 | El motor recorre las etapas de WhatsApp | ✅ `bb1051e` |
| 2 | El contenido de WhatsApp viaja hasta el proveedor | ✅ |
| alta | El número se da de alta con un solo dato | ✅ `4dfd084` |
| 3a | Crear y activar una campaña de WhatsApp, por dentro | ✅ `3501757` |
| 3b | El canal cruza la frontera, nada mezclado entra | ✅ `52ce92b` |
| **3c** | **El asistente, el preflight en pantalla y el semáforo de calidad** | **abierto** |

**Lo que el 3c tiene que cerrar**, y lo primero no es cosmético:

1. **El semáforo de calidad no se llena nunca.** `whatsapp_account.calidad` y
   `limite_diario_meta` **se leen y no los escribe nadie** — ningún `INSERT` ni
   `UPDATE` los toca. Y `arles_whatsapp::estado::consultar`, la sonda que le
   pregunta a Meta, **no tiene llamador**. El dato ya está en la mano: el alta
   en un solo dato lo pregunta, lo enseña para elegir y al guardar lo tira.
   Consecuencia: con el escalón vacío, una campaña de WhatsApp sin tope diario
   propio **no se puede activar**.
2. El paso 1 del asistente pregunta el canal (hoy está escrito a fuego en
   `'email'`).
3. El paso 3 ofrece números en vez de cuentas de correo; el paso 4, plantillas
   aprobadas de Meta en vez de plantillas de correo, y sin firma.
4. El paso 7 llama a `revisar_campana_de_whatsapp`, **y ahí vuelve a
   registrarse** en `lib.rs`: hoy está fuera a propósito porque no tiene quien
   lo llame, y la comprobación de la fase W lo caza.
5. El simulador de la vista previa aprende el canal, para que la demostración no
   mienta.

---

## 6 · Cómo se cierra un tramo

```
un tramo → se verifica → se empuja → se explica → se pregunta
```

Y las cuatro de la lista final de [`METODO.md`](METODO.md): quién lo llama, qué
documento deja de ser verdad, qué poka-yoke hace falta, y si está todo en su
carpeta.
