# ADR-0008 — Licencia por empresa con cupo de dispositivos

**Estado:** Propuesto · **Fecha:** 2026-09-01

## Contexto
§142.7 pregunta si la licencia debe ser por dispositivo o por empresa. §77 pide tolerar interrupciones de
conectividad sin bloquear.

## Decisión
Licencia **por organización**, con `device_limit`. Cada instalación registra una huella de dispositivo y
ocupa un cupo. **El cliente puede liberar un cupo desde la propia aplicación**, sin llamar a soporte.
Periodo de gracia sin conexión configurable, de 14 a 30 días.

## Razones
Las empresas compran por empresa, no por máquina. El área de TI reinstala, reimagina y reemplaza equipos
constantemente: una licencia atada rígidamente a un dispositivo convierte cada cambio de laptop en un
ticket. **El soporte es el costo real de un modelo de licenciamiento agresivo, y lo paga TELEMETRY.**

## Regla innegociable
**Una licencia vencida o no verificable nunca impide leer los datos propios ni exportar un respaldo.**
Degrada a solo lectura: se puede consultar, exportar y respaldar; no se puede enviar.

Secuestrar los datos de un cliente por un problema de facturación es indefendible, y comercialmente suicida
cuando quien evalúa el producto es un área de TI.

## Capacidades, no condicionales de edición
Nada de `if (edicion == "premium")` repartido por el código (§78). Un único módulo de capacidades: la
licencia trae una lista de identificadores y el código pregunta `entitlements.has("multi_account")`.
Cambiar el empaquetado comercial pasa a ser configuración, no un cambio de código — que es exactamente lo
que exige §160.

## Consecuencias

**Positivas.** Menor carga de soporte. Empaquetado comercial modificable sin tocar código. Confianza del
cliente sobre sus propios datos.

**Negativas.** Ligeramente más fácil de compartir de lo debido. Se mitiga con `device_limit` y auditoría,
no con agresividad de DRM.
