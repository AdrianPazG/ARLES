# Antes de empezar

**Para:** quien vaya a trabajar en ARLES — una persona nueva, o un modelo
distinto del que escribió la última entrega.
**Última actualización:** 2026-10-27

Esta carpeta existe por un motivo concreto: **este proyecto ha fallado más
veces por no leer que por programar mal.** El registro está en
[`REGISTRO-DE-TROPIEZOS`](../07-entrega/REGISTRO-DE-TROPIEZOS.md) y en las seis
auditorías, y el patrón se repite: alguien da por sabido algo que estaba
escrito, o lee un documento que afirmaba de más.

Son cinco archivos y se leen en veinte minutos. **No son opcionales.**

---

## El orden

| # | Lee | Por qué |
|---|---|---|
| 1 | [`CLAUDE.md`](../../CLAUDE.md) (raíz) | Las reglas. No son preferencias: casi todas están escritas porque algo salió mal antes |
| 2 | [`PROHIBICIONES.md`](PROHIBICIONES.md) | Lo que nunca se hace, en una lista que se puede recorrer |
| 3 | [`METODO.md`](METODO.md) | Cómo se trabaja: la regla de la ruptura, los tramos, la verificación |
| 4 | [`ESTADO_ACTUAL.md`](../07-entrega/ESTADO_ACTUAL.md) | **Dónde está el trabajo hoy.** Es el único archivo de estado |
| 5 | [`REGISTRO-DE-REQUISITOS.md`](../10-requisitos/REGISTRO-DE-REQUISITOS.md) | Qué se ha pedido, qué falta y quién lo bloquea |

Y según lo que vayas a hacer:

| Si vas a… | Lee también |
|---|---|
| Tocar código | [`ROLES.md`](ROLES.md) — qué sombrero toca ponerse |
| Tocar una pantalla | [`MAPA-DE-PANTALLAS`](../05-diseno/MAPA-DE-PANTALLAS.md) y [`DESIGN_SYSTEM`](../05-diseno/DESIGN_SYSTEM.md) |
| Escribir un informe para Dirección | [`guia-de-documentos/`](../05-diseno/guia-de-documentos/README.md) |
| Retomar sin contexto previo | [`RELEVO.md`](RELEVO.md) — el traspaso entre sesiones |

---

## Las tres cosas que más caro han salido

Están aquí porque si sólo se lee una pantalla de esta carpeta, que sea ésta.

**1 · Una comprobación que nunca se ha visto fallar no está verificada.**
Nueve veces se ha escrito una comprobación que pasaba en verde sin comprobar
nada —porque casaba su propio comentario, porque miraba la línea equivocada,
porque la lista estaba a mano—. El método está en [`METODO.md`](METODO.md) y no
se salta.

**2 · Una pieza correcta y probada que nadie llama.** Veinticinco veces. Una
prueba comprueba que la pieza funciona, no que otro módulo la use, y el hueco
vive entre dos capas que cada una pasa lo suyo. **Al añadir algo, la pregunta no
es «¿funciona?» sino «¿quién lo llama?».**

**3 · Un documento que afirma de más es peor que no tenerlo, porque se cree.**
El índice de la documentación decía «Fase actual: 0 — Discovery» con la Fase 8
cerrada. Si cambias algo y un documento deja de ser verdad, el documento es
parte del cambio.

---

## Lo que esta carpeta NO es

No es un segundo sitio donde vivan las cosas. **Cada cosa vive en un solo
sitio**, y aquí sólo hay punteros:

- El estado del proyecto está en `07-entrega/ESTADO_ACTUAL.md`. **No hay otro.**
- Los defectos encontrados usando ARLES están en
  `06-calidad/BITACORA-DE-PRUEBAS.md`.
- Los tropiezos de instalación y arranque, en
  `07-entrega/REGISTRO-DE-TROPIEZOS.md`.
- El mapa completo de la documentación, en `00-INDICE.md`.

Si te hace falta escribir algo y no sabes dónde va, mira
[`DONDE-VA-CADA-COSA.md`](DONDE-VA-CADA-COSA.md). Crear un archivo en la raíz o
en una carpeta nueva «por ahora» es cómo se llega a dos verdades que se
contradicen.
