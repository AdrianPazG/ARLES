# ADR-0006 · Sustitución de variables no Turing-completa

**Estado:** aceptado · **Fecha:** 2026-09-11 · **Decide:** Security + Tech Lead

---

## Contexto

Las plantillas necesitan variables: `Hola {{nombre}}` (§42). La tentación inmediata es usar un motor de plantillas existente —Handlebars, Tera, MiniJinja— porque están probados, son rápidos y resuelven el problema en una tarde.

Pero hay que mirar de dónde vienen los datos que ese motor va a evaluar: **archivos XLSX y CSV que el usuario importa desde fuera**. Listas compradas, exportaciones de otros sistemas, archivos que alguien envió por correo. Contenido no confiable por definición.

El §89 exige prevenir inyección de plantillas explícitamente.

## Decisión

**Motor de sustitución propio, puramente textual, contra un mapa cerrado de claves.**

```rust
pub fn render(template: &str, vars: &HashMap<String, String>) -> Result<String, TemplateError>
```

Reglas:

- Sólo se reconoce `{{clave}}`. Nada más.
- La clave debe existir en un **conjunto previamente declarado** (`template_version.variables_used`). Una clave desconocida es error de validación, no una cadena vacía silenciosa.
- El valor se inserta **como texto**, escapado según el contexto (HTML o texto plano).
- **Sin condicionales. Sin bucles. Sin helpers. Sin acceso a propiedades anidadas. Sin filtros.**
- El resultado pasa después por `ammonia` con lista blanca, en Rust.

## Justificación

**Un motor con lógica es un intérprete.** Handlebars con helpers, Tera con `{% if %}` y `{% for %}`, Jinja con acceso a atributos — todos evalúan expresiones sobre datos. Cuando esos datos vienen de un archivo ajeno, se está ejecutando un lenguaje sobre entrada no confiable.

Los modos de fallo no son teóricos:

- **Denegación de servicio por expansión.** Bucles anidados sobre datos controlados por el atacante expanden la salida exponencialmente. Un motor sin límites de recursión ni de tamaño de salida se queda sin memoria.
- **Escape de sandbox.** Es una categoría de vulnerabilidad con historial propio: SSTI. Handlebars y Jinja han tenido escapes de sandbox documentados que permitían acceso a objetos del runtime.
- **Fuga de información.** Acceso a propiedades anidadas puede alcanzar objetos de contexto que no se pretendía exponer.

**Ninguna funcionalidad de v1.2.0 necesita lógica.** El §41 pide explícitamente priorizar la simplicidad: texto enriquecido, negrita, enlaces, variables, firmas. Nada de eso requiere un condicional.

**La superficie de ataque es proporcional a la expresividad.** Sustitución textual contra un mapa cerrado tiene una superficie de ataque que cabe en un test unitario.

## Consecuencias

**Positivas.** Superficie de ataque mínima, auditable de un vistazo · sin dependencia de terceros en el camino crítico de seguridad · rendimiento trivial y predecible · errores de plantilla detectables en el **preflight**, antes de enviar nada.

**Negativas.**

1. **No hay `{{#if empresa}}`.** Un usuario que quiera «Estimado {{nombre}}» cuando hay nombre y «Estimado cliente» cuando no, no puede expresarlo.

   **Mitigación:** valores por defecto declarados por variable —`{{nombre|Cliente}}`— que cubren el 90 % de ese caso sin introducir lógica. Si la sintaxis con `|` resulta confusa para el público objetivo, se define el valor por defecto en la interfaz al declarar la variable, y la plantilla queda limpia. **Esto último es probablemente lo correcto.**

2. **Código propio que mantener.** Son unas doscientas líneas con una batería de tests. Barato.

3. **Si en v1.3 alguien pide condicionales**, la respuesta no es «añadamos Handlebars»: es volver a este ADR y decidir explícitamente, con el modelo de amenazas delante.

## Alternativas descartadas

**Handlebars / Tera / MiniJinja con sandbox y helpers desactivados.** Configurables para ser seguros, pero la seguridad depende de que **nadie** active un helper en los próximos tres años. Una configuración segura es más frágil que una capacidad ausente: la configuración se puede cambiar en un commit de viernes por la tarde.

**Sanitizar sólo la salida.** Insuficiente: no protege contra la denegación de servicio por expansión ni contra la fuga de información dentro del propio motor. La sanitización de salida se hace **además**, no en lugar de.

**Permitir HTML crudo del usuario.** Rechazado por el §43. El editor es de texto enriquecido y el HTML se genera desde una estructura controlada.

## Notas de implementación

```rust
// Escapado según contexto
match context {
    Html => html_escape(value),   // & < > " '
    Text => value.to_string(),
}
```

Límites duros: tamaño de plantilla, número de variables y tamaño de salida. No porque el motor pueda expandirse —no puede— sino porque un valor de 50 MB en una celda de XLSX sigue siendo un problema de memoria.

**Validación en el preflight** (§63): todas las variables de la plantilla se resuelven para todos los contactos de la audiencia. Un `{{apellido}}` vacío en 400 contactos se detecta **antes** de activar, no al recibir la primera queja.
