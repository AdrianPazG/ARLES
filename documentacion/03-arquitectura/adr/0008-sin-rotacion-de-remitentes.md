# ADR-0008 · Sin rotación automática de remitentes

**Estado:** aceptado · **Fecha:** 2026-09-11 · **Decide:** Dirección (T-1)

---

## Contexto

Cuando una cuenta remitente alcanza su límite diario, la campaña se detiene hasta el día siguiente. La pregunta obvia es: si el usuario tiene tres cuentas registradas, ¿por qué no saltar automáticamente a la siguiente y seguir enviando?

Es la funcionalidad estrella de buena parte de las herramientas del sector. Se vende como «envío ilimitado», «rotación inteligente de remitentes» o «calentamiento automático de cuentas».

Dirección lo prohibió explícitamente en T-1.

## Decisión

**No hay rotación automática.** Al alcanzarse el límite de una cuenta, la cola **pausa** y la interfaz avisa:

> «La cuenta [correo] alcanzó su límite diario de 50 envíos. La campaña continuará mañana a las 09:00.»

El usuario puede registrar varias cuentas y asignarlas a **campañas distintas**. Una campaña tiene un remitente y lo respeta.

## Justificación

**Esta es una decisión de producto, no una limitación técnica.** Implementar la rotación sería sencillo. No se hace porque el producto no debe hacerlo.

**Es evasión de límites.** Un límite de envío no es un obstáculo de ingeniería: es una política del proveedor para controlar el abuso. Repartir el mismo envío entre varias cuentas para superar el límite agregado es, literalmente, saltarse esa política. El §6 lo dice: ARLES no es «un sistema para saltarse los límites de Gmail».

**Viola las políticas de Google.** La política de datos de usuario de Google prohíbe expresamente las aplicaciones que «usan múltiples cuentas para abusar de las políticas de Google, eludir las limitaciones de las cuentas de Gmail, evadir filtros y spam, o subvertir restricciones de otro modo». Implementar la rotación pondría en riesgo la verificación OAuth de la que depende `GoogleProvider` (ADR-0003).

**Es el mecanismo por el que se queman dominios.** La rotación no reduce el volumen que sale del dominio: lo mantiene y lo reparte. Los filtros de recepción evalúan reputación **a nivel de dominio**, no sólo de buzón. Repartir 500 correos diarios entre diez cuentas del mismo dominio produce la misma señal agregada que enviarlos desde una, con el agravante de que ahora hay diez buzones con patrón sospechoso en vez de uno.

Dicho de otro modo: **la rotación no resuelve el problema, lo oculta hasta que es peor.**

**Y es honesto con el usuario.** Un usuario cuya campaña se pausa entiende que hay un límite. Un usuario cuya campaña sigue mágicamente no entiende nada — hasta que su dominio está en una lista negra y no sabe por qué.

## Consecuencias

**Positivas.** El producto no es una herramienta de evasión · no hay riesgo para la verificación OAuth · el usuario ve la restricción real y puede tomar decisiones informadas · el aviso de más de 50 diarios (§48) tiene sentido, porque el límite de verdad se aplica.

**Negativas.**

1. **Las campañas grandes tardan mucho.** 5 000 correos a 50 diarios son 100 días.

   **Mitigación:** el simulador (§50) lo dice **antes** de activar, en lenguaje llano. Ese número es frecuentemente la información más valiosa del producto: es lo que hace que alguien reconsidere su enfoque antes de lanzar.

2. **Un competidor ofrecerá la rotación y algún cliente la pedirá.** Ocurrirá.

   **Respuesta, y conviene tenerla preparada:** ARLES no lo hace porque protege la reputación del dominio del cliente. Si alguien necesita volumen alto de verdad, la herramienta correcta es un servicio de envío transaccional con dominio dedicado, reputación gestionada y autenticación adecuada — no diez cuentas de Gmail rotando. **Recomendar esa alternativa es mejor servicio que implementar la rotación.**

3. **Una cuenta bloqueada detiene la campaña** en vez de degradarse a otra. Es el comportamiento correcto: un bloqueo es información, no un obstáculo que sortear.

## Alternativas descartadas

**Rotación automática.** Por todo lo anterior.

**Rotación con consentimiento explícito** («entiendo los riesgos»). Tentador, porque encaja con el principio de autonomía del §154. Se descarta por dos motivos: **(a)** el principio de autonomía permite al usuario asumir riesgos que entiende, pero no obliga al producto a *facilitar* la evasión de políticas de terceros; **(b)** seguiría violando la política de Google independientemente del consentimiento del usuario, y pondría en riesgo la verificación OAuth de todos los clientes.

**Repartir la audiencia entre cuentas al crear la campaña.** Es la misma evasión con otro nombre: el volumen agregado del dominio no cambia.

**Lo que sí se permite, y es legítimo:** que el usuario asigne **campañas distintas** a **cuentas distintas**. Departamentos, líneas de producto o personas diferentes que envían sus propias campañas. La diferencia es que cada campaña tiene un remitente coherente y real, en vez de un mismo envío disfrazado de varios.
