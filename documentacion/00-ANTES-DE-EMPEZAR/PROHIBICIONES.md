# Lo que nunca se hace

**Última actualización:** 2026-10-27

Esta lista no es un resumen de `CLAUDE.md`: es su parte **verificable**, puesta
en una forma que se puede recorrer antes de empujar. Cada línea dice qué
comprobación la vigila, si alguna lo hace.

Una prohibición sin comprobación es una intención. Las que están marcadas
**«a mano»** son las que todavía dependen de que alguien se acuerde — y son
las candidatas a convertirse en un poka-yoke.

---

## 1 · Datos de personas

| Nunca | Por qué | Lo vigila |
|---|---|---|
| Guardar una credencial en texto plano, JSON, `localStorage`, bitácora o tabla sin cifrar | ADR-0011. Va al llavero del sistema, y si el llavero no está, la aplicación no arranca | `fase 5`, `nada_que_cruce_la_frontera_lleva_contrasena` |
| Devolver un secreto por un comando | Regla 3 de la frontera IPC | `fase 5` (lee los once módulos de `arles-app/src`) |
| Que una ruta del sistema de archivos cruce la frontera | Regla 4 | a mano |
| Guardar el **texto** de una respuesta de WhatsApp | D-7A: se cuentan, nunca qué dijeron | el tipo `Respuesta` no tiene dónde |
| Borrar un desenlace de envío | Es la prueba de qué se mandó y a quién | `audit_log` imborrable por esquema, con dos *triggers* |
| Dar de baja a alguien antes del `RCPT TO` | Es la única puerta a una supresión, junto con el acuse final | a mano |

## 2 · Base de datos

| Nunca | Por qué | Lo vigila |
|---|---|---|
| Editar una migración ya aplicada | Su huella está registrada | `fase 2`, la huella de cada migración |
| `DROP TABLE` de una tabla padre | Con `foreign_keys=ON` borra en cascada a los hijos. Así se perdieron la audiencia congelada y el registro de envíos | a mano · se usa `ALTER TABLE … DROP COLUMN` |
| Añadir una tabla sin clasificarla en el manifiesto del respaldo | Se exporta, se restaura, y sus datos no están | `fase 8`, `ninguna_tabla_se_queda_fuera_del_respaldo` |
| Añadir una tabla sin quien la escriba | Columna muerta que alguien rellenará esperando algo | `fase W`, `ninguna_tabla_se_queda_sin_escritor` |
| Llamar a un método público de `Db` dentro de un `con_dominio` | El cerrojo no es reentrante: cuelga sin error ni traza | a mano |

## 3 · Frontera y comandos

| Nunca | Por qué | Lo vigila |
|---|---|---|
| Registrar un comando que nadie llama | Es una puerta que nadie vigila | `fase W`, `ningun_comando_se_queda_sin_quien_lo_llame` |
| Devolver un error como cadena | Errores tipados, con clave de i18n | `fase 3`, el §139 sobre el catálogo entero |
| Añadir una variante de error sin su texto | La pantalla enseñaría la clave cruda | `fase 3` + la lista de `errores.spec.ts` |
| Dejar que un tipo validado se deserialice | `DatosDeContacto` y `CanalValidado` sólo salen del validador | una prueba que no compila si alguien les pone `Deserialize` |

## 4 · Interfaz

| Nunca | Por qué | Lo vigila |
|---|---|---|
| Un literal en un componente: un hex, un píxel, una duración | §17. Todo sale de los tokens | `fase 2` |
| Un token inventado | El navegador resuelve un `var()` inexistente **a nada, sin error** | `fase 2` · ha pasado nueve veces |
| Texto escrito dentro de un componente | §139. Va al catálogo | `fase 3` |
| Una `@` suelta en un texto | `vue-i18n` la usa para enlaces: tumba el componente entero | `fase 7`, `ningun_texto_lleva_una_arroba_suelta` |
| `**asteriscos**` en un texto | No hay Markdown: salen literales | `fase 7`, `ningun_texto_lleva_markdown` |
| Decir «entregado» de un correo | §65. ARLES no puede saberlo | a mano |
| Un adjetivo donde cabe una cifra | §94 | a mano |
| Un error que no diga qué pasó, cómo se arregla y qué está a salvo | §95 | a mano |
| Un objetivo pulsable de menos de 32 px | §22, más alto que el 24 del WCAG a propósito | `sonda:auditoria`, a cuatro ventanas |
| El numeral «I» junto al número de versión | ADR-0010 | a mano |

## 5 · Herramientas y método

| Nunca | Por qué |
|---|---|
| `git checkout` para restaurar una ruptura | Revierte el archivo entero, no sólo la ruptura. **Ha costado trabajo rehecho dos veces.** Se usa una copia hecha **antes de la primera ruptura** |
| Verificar una ruptura con un `grep` de un identificador | Casa el comentario que explica la ruptura. Se compara el archivo entero |
| Correr sólo la fase que se está tocando | En la 5.2 y la 5.3 se rompieron cuatro comprobaciones de otras fases sin que nadie lo viera. **Las diez, siempre** |
| Dar por buena una ruptura sin verla en rojo | Nueve comprobaciones han pasado en verde sin comprobar nada |
| Una comprobación que escriba en el repositorio para medirlo | Dos validadores a la vez se pisan y sale un rojo que no existe. Medido: 1 falso de 40 |
| Empujar a otra rama que `claude/loving-faraday-5chmmy` | Sin permiso explícito de Dirección |
| Abrir un pull request | Salvo que Dirección lo pida con esas palabras |

## 6 · Material de Dirección

**Carpetas de sólo lectura.** No se borran, mueven, renombran, convierten,
comprimen ni editan:

    /RECURSOS   /REFERENCIA_DE_COLOR   /CONCEPTOS_DE_DISEÑO
    /REFERENCIAS_VISUALES_DEL_SITIO    /TIPOGRAFIA

**Claves de firma:** ninguna, nunca, en el repositorio.

**Ningún identificador de modelo** en commits, cuerpos de PR, comentarios del
código ni nada que acabe en el repositorio. En el chat sí.
