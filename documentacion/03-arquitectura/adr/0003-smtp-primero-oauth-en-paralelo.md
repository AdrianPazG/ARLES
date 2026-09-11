# ADR-0003 · SMTP primero, verificación OAuth en paralelo

**Estado:** aceptado · **Fecha:** 2026-09-11 · **Decide:** Dirección (D-1)

---

## Contexto

El brief asume Gmail vía OAuth 2.0 como integración principal (§27) y planifica 16 fases **sin contemplar el proceso de verificación de Google**.

La auditoría verificó la situación real:

- `gmail.send` es un scope **sensible**, no restringido. Eso es buena noticia: **evita la auditoría CASA Tier 2** que sí exigirían `gmail.modify` o `mail.google.com`, con auditor externo aprobado por Google, coste significativo y renovación anual.
- Pero **sigue exigiendo verificación OAuth**: política de privacidad pública alojada en un dominio verificado en Search Console, página de inicio del producto en ese dominio, vídeo demostrativo del flujo de consentimiento, y una revisión que se mide en semanas.

Descubrir esto en la Fase 6 congelaría el release entre 4 y 8 semanas con el producto terminado. Era el camino crítico más largo del proyecto y no estaba en ningún plan.

## Decisión

**v1.2.0 envía por SMTP.** La verificación OAuth arranca en la **Fase 1**, en paralelo al desarrollo, para que `GoogleProvider` aterrice en **v1.2.x**.

La abstracción `EmailProvider` se diseña **completa desde el día uno**, con `GoogleProvider` como hueco ya dimensionado, no como añadido posterior.

## Por qué funciona

**Gmail admite SMTP con contraseña de aplicación, sin verificación alguna.** El caso de uso principal —«quiero enviar desde mi Gmail»— está cubierto desde el primer día, con el coste de que el usuario debe activar la verificación en dos pasos y generar una contraseña de aplicación.

Es más fricción que dos clics de OAuth. No es un bloqueo.

## Consecuencias

**Positivas.**
- El release **no depende del calendario de Google**.
- La abstracción se valida con un proveedor real desde el principio, en vez de diseñarse en abstracto contra un único caso.
- El adaptador SMTP no es trabajo desechable: cubre proveedores corporativos, servidores propios y cualquier cosa que no sea Google ni Microsoft. Es valor permanente.

**Negativas.**
- **El onboarding de Gmail es más áspero en v1.2.0.** La interfaz debe guiar el flujo de contraseña de aplicación con enlaces directos; es el punto donde más usuarios se atascan.
- **Se pierde el gancho comercial de «conecta tu Gmail en dos clics»** hasta v1.2.x. Con D-4 (despliegue interno) esto no tiene coste comercial real ahora.
- **Depende de P-05** (dominio y aviso de privacidad), que hay que resolver pronto aunque el adaptador no entre hasta después.

## Alternativas descartadas

**Un único client ID de TELEMETRY verificado, embebido en la aplicación.** Da el onboarding de dos clics, pero **concentra el riesgo**: todos los clientes comparten el mismo client ID, así que un solo cliente abusivo puede provocar que Google restrinja el acceso **de todos**. Para un producto que se va a comercializar, es un punto único de fallo con consecuencias desproporcionadas. Además, el «client secret» de una aplicación instalada es extraíble del binario, de modo que la protección real la aporta PKCE, no el secreto.

**Cada organización registra su propio proyecto de Google Cloud.** Elimina completamente el bloqueo de verificación y aísla el riesgo entre clientes. Es la opción técnicamente más limpia. Se descarta porque exige que el cliente cree un proyecto en Google Cloud, configure una pantalla de consentimiento y pegue credenciales — inviable para el público objetivo descrito en `VISION_Y_ALCANCE.md`, que sabe usar Excel y no es técnico.

**Diferir Gmail entero a v1.3.** El alcance más pequeño, pero renuncia al caso de uso principal durante demasiado tiempo.

## Nota sobre la elección de scope

Que `gmail.send` sea sensible y no restringido no es casualidad: es la consecuencia directa de aplicar el principio de mínimo privilegio (§87). Pedir `gmail.modify` «por si acaso» habría añadido CASA Tier 2, un auditor externo, coste anual recurrente y meses de calendario.

**Es un buen ejemplo de que la decisión segura resultó además la más barata.** Conviene recordarlo cuando alguien proponga pedir permisos de lectura de bandeja para alguna funcionalidad futura.
