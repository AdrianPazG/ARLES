# Dónde va cada cosa

**Última actualización:** 2026-10-27

Este archivo existe para que nunca haga falta decidir sobre la marcha dónde
poner algo. **Crear un archivo en la raíz o en una carpeta nueva «por ahora» es
cómo se llega a dos verdades que se contradicen** — y el proyecto ya tuvo dos
documentos de la Fase 0 describiendo bibliotecas que nunca se adoptaron, y un
índice que decía «Fase 0 — Discovery» con la Fase 8 cerrada.

Si lo que quieres escribir no encaja en ninguna fila de abajo, **no inventes una
carpeta: pregunta.**

---

## Documentación

| Lo que vas a escribir | Va en |
|---|---|
| Qué es o qué no es ARLES, alcance, decisiones de producto | `01-producto/` |
| Una auditoría de descubrimiento, riesgos, preguntas abiertas | `02-auditoria/` |
| Arquitectura, modelo de datos, el motor, una decisión estructural | `03-arquitectura/` · una decisión = un ADR en `adr/` |
| Superficie de ataque, secretos | `04-seguridad/` |
| Colores, tipografía, componentes, UX, el mapa de pantallas | `05-diseno/` |
| **Cómo se diseña un informe para Dirección** | `05-diseno/guia-de-documentos/` |
| Estrategia de pruebas, rendimiento, un defecto encontrado usando ARLES | `06-calidad/` |
| Guías de usuario, roadmap, estado, tropiezos, cómo conectar algo real | `07-entrega/` |
| Consultas jurídicas, textos legales | `08-legal/` |
| El documento de una fase o de una auditoría ya cerrada | `09-fases/` |
| **Un requisito, una decisión pendiente, algo que Dirección pidió** | `10-requisitos/` |
| Lo que hay que leer antes de tocar nada | `00-ANTES-DE-EMPEZAR/` (esta carpeta) |

**Y todo documento nuevo se nombra en [`00-INDICE.md`](../00-INDICE.md).** No es
burocracia: lo vigila `la documentación dice la verdad` en la fase 0, y la
razón es que un documento que el índice no nombra es un documento que nadie
encuentra — se reescribe desde cero seis meses después, distinto.

---

## Las cuatro cosas que tienen **un solo** sitio

Si escribes una segunda, las dos se contradirán y la que se lea será la
equivocada.

| Qué | Dónde, y sólo ahí |
|---|---|
| **El estado del proyecto** | `07-entrega/ESTADO_ACTUAL.md` |
| **El porcentaje de avance** | `07-entrega/avance.json`, y se calcula: `python3 herramientas/avance/calcular.py --escribir` |
| **Los defectos encontrados usando ARLES** | `06-calidad/BITACORA-DE-PRUEBAS.md` |
| **Los requisitos y lo que bloquea** | `10-requisitos/REGISTRO-DE-REQUISITOS.md` |

El tablero visual (`herramientas/tablero/generar.py`) **no lleva ni una cifra
escrita a mano**: todo sale de esas fuentes. Un tablero con cifras dentro es
una mentira con colores, que dan más confianza todavía.

---

## Código

| Lo que vas a escribir | Va en | La regla que lo define |
|---|---|---|
| Lógica de dominio, reglas, validación | `crates/arles-core/` | **Cero I/O.** Ni base, ni red, ni disco, ni reloj, ni azar |
| SQL, migraciones, repositorios | `crates/arles-db/` | No conoce la IPC: no depende de `serde` |
| Leer CSV o XLSX | `crates/arles-import/` | El archivo que lee **no es de fiar** |
| La cola, el ritmo, la ventana, el disyuntor | `crates/arles-motor/` | **No escribe SQL** |
| El adaptador SMTP | `crates/arles-email/` | No hay forma de no cifrar (§29) |
| El adaptador de Cloud API | `crates/arles-whatsapp/` | El motor no sabe que existe |
| Un comando de Tauri | `crates/arles-app/` | Gruesos, no finos. Errores tipados. Ningún secreto. Ninguna ruta |
| Una pantalla, un store, un texto | `app/src/app/` | TS estricto, `exactOptionalPropertyTypes` |
| Un banco de medición | `crates/arles-banco/` | No viaja en el instalador |

**Una comprobación del proyecto** va en `herramientas/validar/validar.py`, en la
fase que le corresponda. **Una sonda** va en `app/pruebas/sondas/`, y sus rutas
salen de `rutas.mjs` — nunca de una lista a mano.

---

## Lo que no va en el repositorio

- Credenciales, tokens, contraseñas. Ninguna, nunca, de ninguna forma.
- Claves de firma.
- Archivos de salida de una corrida (`.t.txt`, volcados, logs). Van al
  directorio temporal de la sesión.
- Bases de datos de prueba, `target/`, `node_modules/`, `dist/`.

Y **no se toca** el material de referencia de Dirección: `/RECURSOS`,
`/REFERENCIA_DE_COLOR`, `/CONCEPTOS_DE_DISEÑO`,
`/REFERENCIAS_VISUALES_DEL_SITIO`, `/TIPOGRAFIA`. Ni para convertir, ni para
comprimir, ni para renombrar.
