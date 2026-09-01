# ADR-0005 — Secretos en el llavero del SO; base de datos sin cifrar

**Estado:** Propuesto · **Fecha:** 2026-09-01

## Contexto
§30 prohíbe guardar credenciales en JSON, texto plano, localStorage, registros o archivos temporales, y
establece que la base no debe convertirse en un almacén improvisado de contraseñas.

## Decisión
1. Credenciales y tokens en el **llavero del sistema** (`keyring`): Keychain en macOS, Credential Manager
   en Windows. `EmailAccount` guarda solo un `secret_ref`.
2. **La base de datos no se cifra** en v1.2.0.
3. **Los respaldos sí se cifran**, con frase del usuario, y **excluyen credenciales**.

## Alternativas
- **SQLCipher para toda la base:** complica respaldo, restauración y herramientas de diagnóstico. La clave
  seguiría viviendo en el llavero del mismo equipo, así que no cambia el modelo de atacante relevante
  (malware con la misma cuenta de usuario). Coste alto, beneficio marginal.
- **Cifrar solo columnas sensibles:** rompe índices y consultas sobre esas columnas; los datos de contacto
  se filtran igual.

## Consecuencias

**Positivas.** Cero credenciales en la base. Respaldos seguros de transportar. Herramientas de diagnóstico
y recuperación siguen funcionando sobre la base.

**Negativas, documentadas explícitamente.** El llavero del sistema **no protege frente a código que se
ejecuta con la misma cuenta de usuario**. Esto es cierto para toda aplicación de escritorio. La defensa real
contra el robo del equipo es el cifrado de disco del sistema.

**Acción derivada:** Diagnóstico comprueba el estado de FileVault o BitLocker y recomienda activarlo.
`SECURITY.md` debe declarar este límite en lugar de insinuar protección de grado bóveda.
