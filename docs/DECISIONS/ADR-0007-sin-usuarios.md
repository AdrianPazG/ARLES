# ADR-0007 — Sin usuarios internos con contraseña

**Estado:** Propuesto · **Fecha:** 2026-09-01

## Contexto
§142.6 pregunta si hacen falta usuarios internos en una aplicación local. §32 lista `ApplicationUser`.

## Decisión
**No.** Se elimina `ApplicationUser`. Se añade `operator_label` — un nombre elegido en la configuración,
editable — que firma cada fila de `AuditLog`.

## Razones
El sistema operativo ya autenticó a la persona. Añadir contraseñas crea un almacén de credenciales, flujos
de recuperación y una **falsa** sensación de seguridad: quien tenga la sesión de Windows o macOS puede leer
el archivo SQLite directamente, con o sin contraseña de ARLES.

## Consecuencias

**Positivas.** Se elimina un subsistema completo de v1.2.0: almacén de contraseñas, hash, recuperación,
sesiones, bloqueo por intentos. La atribución en la bitácora (§90) se conserva.

**Negativas.** En un equipo compartido por varias personas con la misma cuenta del sistema, la etiqueta de
operador es declarativa y no verificable. Debe documentarse: si se requiere separación real, la recomendación
es usar cuentas de sistema distintas.

**Futuro.** Multiusuario real con autenticación llega con la edición Cloud, donde hay un servidor que puede
verificar identidad de verdad.
