# ARLES RELAY I · v1.2.0 — instrucciones para Claude Code

Este archivo lo lee Claude Code al abrir el proyecto, y existe por una razón
concreta: **las reglas de ARLES vivían sólo en una conversación**. Cualquier
sesión nueva —en la terminal de VS Code, en otro equipo, dentro de un mes—
arrancaba sin saber nada de ellas y las rompía sin enterarse.

Lo que sigue no son preferencias: son decisiones ya tomadas, casi todas porque
algo salió mal antes.

---

## 1 · Lo que no se negocia

**Responde SIEMPRE en español.** Todo: respuestas, planificación, comentarios
del código, documentación, mensajes de commit. El equipo es mexicano y el
producto también.

**Rama de trabajo: `claude/loving-faraday-5chmmy`.** Nunca se empuja a otra sin
permiso explícito de Dirección.

**Nunca abras un pull request** salvo que Dirección lo pida con esas palabras.

**Credenciales:** jamás en texto plano, JSON, `localStorage`, bitácoras ni
tablas de SQLite sin cifrar. Van al llavero del sistema —Keychain en macOS,
Credential Manager en Windows—. Si el llavero no está disponible, **la
aplicación no arranca**; nunca se degrada a texto plano (ADR-0011).

**Claves de firma:** no se guardan en el repositorio. Ninguna.

**Carpetas de sólo lectura**, material de referencia de Dirección:

    /RECURSOS   /REFERENCIA_DE_COLOR   /CONCEPTOS_DE_DISEÑO
    /REFERENCIAS_VISUALES_DEL_SITIO    /TIPOGRAFIA

No se borran, mueven, renombran, convierten, comprimen ni editan.

**Cada commit termina con:**

    Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
    Claude-Session: https://claude.ai/code/session_01LMpU9g9CeXzSDyzEWzDzhe

**Ningún identificador de modelo** en commits, cuerpos de PR, comentarios del
código ni nada que acabe en el repositorio. En el chat sí.

---

## 2 · La regla de trabajo

> **Una comprobación que nunca se ha visto fallar no está verificada.**

Cada garantía que se afirme —en un test, en el validador de fase, en una sonda—
se rompe a propósito para ver si la comprobación la caza. Si no la caza, la
comprobación no vale y hay que arreglarla antes de seguir.

Esto no es ceremonia. En este proyecto ya ha encontrado, entre otras:

- Una prueba de la página de contactos con **un solo contacto**: pedir diez mil
  también devolvía uno, así que el recorte de página se podía quitar entero.
- Una comprobación del validador que miraba **la línea equivocada** encima de
  una estructura y pasaba siempre, dijera lo que dijera el `derive`.
- Una sonda que buscaba `+521` en un texto donde el número se enseña con
  espacios: el patrón no podía coincidir **nunca**.
- Un troceado en lotes que no estaba verificado porque la prueba pedía 1 500
  variables y el tope real de SQLite resultó ser **32 766**, no las 999 que el
  comentario repetía de documentación vieja.

El método: cambia el código para romper la garantía, corre la prueba, comprueba
que **falla con el mensaje correcto**, y restaura. Se deja constancia en el
comentario de la prueba de que se hizo.

---

## 3 · Cómo se responde a Dirección

Adrián Paz no es programador. Las explicaciones van en sus términos, sin
esconder lo técnico cuando importa.

**Al cuestionar una decisión**, se hace con esta estructura, no con una opinión
suelta:

    Problema → Riesgo → Impacto → Alternativa → Trade-off → Recomendación

**Al informar**, se dice lo que hay: si algo falla, se enseña la salida; si un
paso se saltó, se dice. Los porcentajes llevan fecha. Las cifras son cifras y no
adjetivos.

**Paso a paso.** Dirección lo ha pedido muchas veces. Un tramo, se verifica, se
empuja, se explica, y se pregunta antes del siguiente.

---

## 4 · Arquitectura, en una pantalla

Monolito modular. Tauri 2 + Rust + Vue 3. Cuatro crates y una app:

| Crate | Qué es | Regla que lo define |
|---|---|---|
| `arles-core` | Dominio puro | **Cero I/O.** Ni base, ni red, ni disco. `forbid(unsafe_code)` |
| `arles-db` | SQLCipher, migraciones, repositorios | **No conoce la IPC**: no depende de `serde` |
| `arles-import` | Lectura de CSV y XLSX | El archivo que lee **no es de fiar** |
| `arles-app` | Shell de Tauri y comandos | La única frontera con la webview |
| `app/` | Vue 3 + TS estricto | `exactOptionalPropertyTypes` activo |

**Las cuatro reglas de la frontera IPC** (`ARQUITECTURA.md` §5):

1. Comandos **gruesos**, no finos. Cada cruce es una oportunidad de estado
   inconsistente.
2. Errores **tipados**, nunca cadenas.
3. **Ningún comando devuelve un secreto.**
4. **Ninguna ruta del sistema de archivos cruza la frontera.**

Y una más que se ganó a golpes: **un tipo validado no se puede deserializar.**
`DatosDeContacto` y `CanalValidado` sólo salen del validador del núcleo. Si
pudieran entrar por JSON, la webview mandaría canales «normalizados» a su gusto
y la deduplicación y la supresión —que comparan esa forma— dejarían de funcionar
sin que nada fallara.

---

## 5 · Convenciones que el validador comprueba

- **§17 · Sin literales en los componentes.** Ni un hex, ni un píxel suelto, ni
  una duración. Todo sale de los tokens.
- **§98 · El movimiento tiene propósito**, y `prefers-reduced-motion` es
  obligatorio.
- **§65 · Aceptado ≠ entregado.** ARLES no puede saber si un correo llegó, así
  que la interfaz no dice «entregado».
- **§94 · Cifras, no adjetivos.** «2 de 6», «34 %» — nunca «casi listo».
- **§95 · Todo error dice tres cosas**: qué pasó, cómo arreglarlo y **qué está
  a salvo**.
- **§139 · Los errores llevan clave de i18n**, no texto. Cada variante de
  `CoreError`, `DbError` y `AppError` tiene su `clave_i18n()`, y hay un test que
  falla si alguien añade una sin texto.
- **ADR-0010 · El numeral «I» nunca va junto al número de versión.** «ARLES
  RELAY I» en la marca; «v1.2.0» aparte.
- **§21 · El logotipo de ARLES es exclusivamente tipográfico.**

**Las migraciones no se editan una vez aplicadas.** Y `DROP TABLE` de una tabla
padre está prohibido: con `PRAGMA foreign_keys=ON` borra en cascada a los hijos
—así se perdieron una vez la audiencia congelada y el registro de envíos—. Se
usa `ALTER TABLE … DROP COLUMN`.

---

## 6 · Cómo se verifica, en orden

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings   # cero advertencias
cargo test --workspace
cargo deny check licenses advisories                    # antes de añadir dependencias

cd app && npm test && npm run lint && npx vue-tsc --noEmit -p tsconfig.json

python3 herramientas/validar/validar.py --fase 3        # o 0, 1, 2
python3 herramientas/validar/validar.py --fase 3 --rapido   # omite las sondas
```

**Las sondas abren un navegador de verdad y miden.** `npm run sonda:contactos`,
`sonda:tema`, `sonda:plegado`, `sonda:cabecera`, `sonda:vista-previa`… Necesitan
Chromium; sin él el validador las marca como **omitidas**, que no es lo mismo
que pasadas y así se dice.

**La vista previa** que Dirección abre con doble clic:

```bash
python3 herramientas/vista-previa/generar.py     # deja vista-previa/ARLES-vista-previa.html
```

**El porcentaje de avance** sale de un solo sitio y se escribe a mano al cerrar
trabajo:

```bash
# editar documentacion/07-entrega/avance.json  (campos «hecho» y «porque»)
python3 herramientas/avance/calcular.py --escribir
```

Va **sólo en el README de GitHub**, nunca dentro de la aplicación: es un dato
del proyecto, no del producto.

---

## 7 · Dónde está escrito lo demás

`documentacion/00-INDICE.md` es el mapa. Lo que más se consulta:

| Para | Mira |
|---|---|
| Qué es y qué no es ARLES | `01-producto/VISION_Y_ALCANCE.md` |
| Los dos canales, decisiones L-1 a L-14 | `01-producto/LOGISTICA_DE_CAMPANAS.md` |
| Tablas, índices, invariantes | `03-arquitectura/MODELO_DE_DATOS.md` |
| Por qué algo se decidió así | `03-arquitectura/adr/` |
| Superficie de ataque | `04-seguridad/THREAT_MODEL.md` |
| Colores y contrastes medidos | `05-diseno/COLOR_SYSTEM.md` |
| Lo que Dirección revisa a ojo | `06-calidad/CHECKLIST-VISUAL.md` |
| Fases y dependencias reales | `07-entrega/ROADMAP.md` |

---

## 8 · Lo que está bloqueado esperando a alguien de fuera

Ninguno impide construir. **Los cuatro impiden entregar la v1.2.0.**

| Qué | Quién | Qué bloquea exactamente |
|---|---|---|
| **P-01 · Licencia de Mont** | Dirección | **No se puede distribuir un instalador** con la tipografía dentro. Es lo que impide entregar el `.exe` |
| **P-09 · Revisión jurídica** | Abogado | Los textos legales que el usuario lee. Las ocho preguntas están en `08-legal/CONSULTA-JURIDICA.md`. La construcción **no** se detiene (ADR-0013: se construye con texto marcado como provisional) |
| **META · Cuenta con Coexistencia** | Operaciones | P-14 no se puede probar contra nada real |
| **R-07 · Revisión en macOS** | Quien tenga un Mac | Nadie ha abierto ARLES en WKWebView. La vista previa **no cuenta**: ahí el motor es el del navegador |

---

## 9 · Dónde va el trabajo ahora

**Entrega 3.3 · Importar CSV y Excel.** Hecho: el mapeo de columnas, la lectura
defensiva, el análisis de choques y la migración V5 del origen declarado.
Falta: la escritura en lote, la pantalla, y atacarla con archivos maliciosos.

**Decisión de Dirección del 18/09/2026:** ante un contacto repetido, ARLES
**enseña los choques antes de importar**. No los salta ni fusiona solo: una
importación de diez mil filas es irreversible en la práctica.
