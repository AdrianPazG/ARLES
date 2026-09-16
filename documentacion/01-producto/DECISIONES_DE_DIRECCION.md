# Decisiones de Dirección

**Proyecto:** ARLES RELAY I · v1.2.0
**Última actualización:** 2026-09-11

> Registro autoritativo. Una decisión aquí se considera firme hasta que Dirección la revoque por escrito en este mismo documento. Las decisiones técnicas derivadas viven en `03-arquitectura/adr/`.

---

## Serie T — Decisiones previas al discovery

Tomadas por Dirección antes de esta auditoría, comunicadas en el brief maestro.

### T-1 · Sin rotación automática de remitentes
**Firme.** Los usuarios pueden registrar varias cuentas, pero al alcanzarse un límite **la cola se pausa** con un aviso en la interfaz: «La cuenta [correo] alcanzó su límite…». No se salta automáticamente a otra cuenta.

*Valoración de la auditoría:* **decisión correcta y hay que defenderla.** La mayoría de herramientas del sector venden la rotación como funcionalidad estrella; es evasión de límites, viola las políticas de Google y es el mecanismo exacto por el que se queman dominios. Cuando un cliente la pida —y la pedirá—, la respuesta es no. Ver ADR-0008.

### T-2 · Tipografía Mont
**Firme con salvedad.** Dirección declara poseer licencia comercial de Mont para uso de texto.

*Valoración de la auditoría:* **insuficiente para lo que ARLES necesita.** Las licencias Desktop y Web de Fontfabric no cubren incrustar el binario de la fuente en una aplicación distribuida; eso exige una App License separada. No hay comprobante en el repositorio. Ver hallazgo A-07, riesgo R-01 y decisión **D-3**, que es bloqueante.

### T-3 · Cifrado de la base de datos en reposo
**Firme.** SQLCipher en v1.2.0 para proteger datos personales bajo la legislación mexicana.

*Valoración:* correcta y no negociable. Implica que la clave maestra vive en el llavero del sistema operativo y que **la aplicación se niega a arrancar si el llavero no está disponible** — sin degradación a texto plano. Ver ADR-0011 y riesgo R-10.

### T-4 · Versionado
**Firme con corrección.** El primer lanzamiento público es v1.2.0.

*Corrección:* T-4 lo describía como «beta comercial» mientras T-6 excluía el enforcement de licencias — sin enforcement no hay forma de limitar instalaciones ni cobrar. Resuelto por **D-4**: v1.2.0 es despliegue interno de TELEMETRY.

### T-5 · El numeral «I»
**Firme con matiz de presentación.** Se retiene «I» como identificador comercial.

*Matiz de la auditoría:* «ARLES RELAY I / v1.2.0» pone dos significantes de generación en la misma línea y hace pensar que «I» es la versión 1. Se retiene el numeral, pero **no se muestra adyacente al número de versión**. Ver ADR-0010.

### T-6 · Diferimientos de alcance
**Firme.** Fuera de v1.2.0: integración con Microsoft, cifrado de respaldos y enforcement de licencias. La arquitectura base debe soportarlos después.

*Nota:* el cifrado de respaldos diferido tiene una consecuencia que hay que declarar en la interfaz — en v1.2.0 el respaldo es el único camino de recuperación **y no está cifrado**.

### T-7 · Escala
**Firme.** Objetivo base 200 000 contactos; la arquitectura debe soportar cómodamente un techo de 500 000.

*Nota:* es un techo **arquitectónico**. El volumen **operativo** real está pendiente (P-06) y cambia decisiones de producto, no de arquitectura.

### T-8 · Cumplimiento
**Firme.** Foco estricto en la normativa mexicana de privacidad para v1.2.0 (LFPDPPP). GDPR y normativa estadounidense diferidos.

### T-9 · Activos
**Firme en la intención, no verificable en el repositorio.** Dirección declara poseer todos los derechos sobre marcas, logotipos (flecha, escudo) y paletas.

*Valoración de la auditoría:* **no hay ningún activo de marca en el repositorio** — ni logotipo, ni flecha, ni escudo (hallazgo A-06, riesgo R-08). Y la referencia cromática es stock generado por IA de un tercero, no un activo propio (hallazgo A-03). Los colores no son protegibles, así que extraer una paleta es limpio; distribuir el archivo o presentarlo como activo de marca, no.

### T-10 · Equipo Rust
**Firme.** El equipo son expertos en Rust. Se confía a Rust la complejidad del backend y el motor de ejecución.

*Valoración:* respalda la elección de Tauri (ADR-0001) y permite poner toda la lógica autoritativa en el núcleo, dejando la webview como pura superficie de presentación.

---

## Serie D — Decisiones tomadas en el discovery

Tomadas por Dirección el 2026-09-11, tras la presentación de los hallazgos de auditoría.

### D-1 · Híbrido: SMTP en v1.2.0, verificación OAuth en paralelo
**Decisión.** v1.2.0 envía por **SMTP**. La verificación OAuth de Google arranca en la Fase 1, en paralelo al desarrollo, para que `GoogleProvider` aterrice en v1.2.x.

**Contexto.** `gmail.send` es un scope **sensible**: evita la auditoría CASA Tier 2 —que sí exigirían `gmail.modify` o `mail.google.com`, con auditor externo y renovación anual— pero **exige verificación OAuth**: política de privacidad pública, dominio verificado, vídeo demostrativo y semanas de revisión. El roadmap del brief no contemplaba esta dependencia y era el camino crítico más largo del proyecto.

**Consecuencias.**
- El release **no depende del calendario de Google**.
- Gmail admite SMTP con contraseña de aplicación, sin verificación alguna: el caso de uso principal está cubierto desde el día uno.
- La abstracción `EmailProvider` se diseña **completa** desde el principio, con `GoogleProvider` como segundo adaptador, no como añadido posterior.
- Requiere resolver P-05 (dominio y aviso de privacidad) pronto.

**Alternativas descartadas.** Un único client ID de TELEMETRY verificado (concentra el riesgo: un cliente abusivo tumba el acceso de todos) · cada organización registra su propio proyecto de Google Cloud (inviable para un cliente no técnico) · diferir Gmail entero a v1.3 (pierde el gancho comercial).

**Ver:** ADR-0003, riesgo R-02.

---

### D-2 · Extraer el cian, el oro y los cremas; construir los azules profundos
**Decisión.** Se anclan la marca y el sistema en los colores que la referencia **sí** tiene, y se **derivan por rampa** los azules profundos que faltan. La imagen de referencia queda como material **interno** de estudio: nunca se distribuye ni se presenta como activo de marca.

**Contexto.** La medición del PNG completo arrojó: 54.35 % cian-azur (H 185–210°), 19.48 % ocre, 13.44 % amarillo, y **0.73 % de azul profundo** (H 210–250°). No es una paleta de *La noche estrellada*. El §16 asigna tres roles a tres azules distintos, pero el 54 % de la imagen cabe en ±12°: extraerlos literalmente los colapsa en una interfaz monocromática cian sin jerarquía. Y sólo el 1 % de los píxeles baja de L 10, así que el fondo profundo que exige el dark-first no está en la imagen.

**Consecuencias.**
- El sistema tiene jerarquía real entre estructura, superficie e interacción.
- `#2CA4D4` queda restringido a trazo, borde y foco: falla WCAG como superficie contra todo texto, blanco incluido (2.85:1).
- El amarillo `#FCCC0C` se usa como **luz sobre oscuro** (11.41:1 sobre `#041C2C`), nunca como relleno de botón con texto blanco (1.56:1).
- Legalmente limpio: los colores no son protegibles.

**Ver:** ADR-0005, hallazgos A-03/A-04/A-05, `05-diseno/COLOR_SYSTEM.md`.

---

### D-3 · Verificar o adquirir la App License de Fontfabric
**Decisión.** Dirección confirma el alcance exacto de la licencia de Mont que posee y, si no cubre app embedding, la adquiere. El comprobante se archiva en `documentacion/08-legal/`.

> ⚠️ **Revisada por D-5 (2026-09-11).** La puerta ya no está en la Fase 2, sino antes de la demo. Lee D-5 para saber qué aplica hoy.

**Contexto.** El kit del repositorio fue generado con Transfonter y contiene `.eot` —formato muerto desde IE11—, perfil característico de un paquete de agregador y no de una entrega comercial de Fontfabric. La tabla `name` identifica la fuente como Mont de Fontfabric (Svetoslav Simov, Mirela Belova). No hay licencia, factura ni comprobante en el repositorio. Un bundle de Tauri lleva la fuente dentro del binario instalado en la máquina del cliente: eso es exactamente el supuesto de la App License, que las licencias Desktop y Web no cubren.

El §21 del propio brief ordena detenerse ante esto.

**Plan B, listo para activarse.** Si al inicio de la Fase 2 no hay comprobante: Mont queda para marketing y material comercial · el logotipo ARLES RELAY se entrega como **SVG con contornos** (uso legítimo de una licencia de escritorio, porque ya no hay fuente que incrustar) · la interfaz usa una geométrica de licencia libre con métricas similares.

**Ver:** hallazgo A-07, riesgo R-01, pregunta P-01, `05-diseno/TIPOGRAFIA.md`.

---

### D-4 · v1.2.0 es despliegue interno de TELEMETRY; comercial en v1.3
**Decisión.** v1.2.0 se instala únicamente en máquinas de TELEMETRY INSIGHT. El lanzamiento comercial es v1.3.

**Contexto.** T-4 describía v1.2.0 como «beta comercial» mientras T-6 excluía el enforcement de licencias. Sin enforcement no hay forma de limitar instalaciones, hacer caducar un piloto ni cobrar — y el software instalable que ya salió no se puede des-distribuir.

**Consecuencias.**
- Difiere tres costes externos: certificado de firma EV, verificación OAuth de Google y canal de soporte.
- Permite **validar el motor de ejecución con datos reales y consecuencias reales** antes de exponerlo a clientes.
- El trámite de firma de código arranca igualmente en la Fase 1: son plazos externos que no se comprimen después (riesgo R-14).

---

### D-5 · Mont se usa durante el desarrollo; la licencia se resuelve antes de la demo
**Decisión.** Se desarrolla con Mont con normalidad. **La pregunta por la licencia se vuelve a plantear cuando llegue el momento de la demo de la aplicación**, y esa es la puerta que sustituye a la de la Fase 2.

**Qué desbloquea.** La Fase 2 arranca sin esperar. El Design System puede congelarse, incluida la escala tipográfica y el logotipo.

**Y también desbloquea P-02.** El §21 define el logotipo como **exclusivamente tipográfico** —«ARLES RELAY» en Mont Black, sin isotipo—, así que con Mont disponible el logotipo se puede producir aquí. La flecha y el escudo que menciona T-9 no son necesarios: el §21 los excluye por diseño.

**Dónde está la línea real, para que nadie la cruce por descuido.**

| Uso | ¿Autorizado hoy? |
|---|---|
| Componer documentos, láminas y maquetas con Mont | **Sí** — uso de documento, cubierto por una licencia de escritorio |
| Compilaciones de desarrollo en máquinas del equipo | **Sí** |
| Logotipo entregado como SVG con contornos | **Sí** — es geometría, no contiene la fuente |
| **Instalar la aplicación con Mont incrustada en cualquier máquina fuera del equipo de desarrollo** | **No, hasta resolver D-3** |
| **Enseñar la aplicación a un cliente o prospecto como demo** | **No, hasta resolver D-3** |

Lo que exige la App License no es el desarrollo: es **incrustar el binario de la fuente en un artefacto que se instala o se enseña como producto**. Mientras el binario no salga del equipo, no hay supuesto de distribución.

**Cómo se vuelve a preguntar.** Al planificar la demo, antes de generar el primer instalador destinado a enseñarse. Registrado como **P-01** con disparador actualizado, y como elemento de la puerta de salida de la Fase 9.

**Por qué es razonable ahora.** El plan B —Mont en marketing, logotipo en SVG con contornos, Figtree en la interfaz— sigue listo, **y es más barato de activar que antes**: toda la tipografía vive en `--arles-font-family` y los colores en `tokens.json`. Cambiar de familia es editar una línea, no rehacer el Design System. Desarrollar con Mont no crea deuda: crea una sustitución de una línea.

**Riesgo residual, dicho con claridad.** Si la licencia no llega y hay que activar el plan B, el trabajo perdido es el ajuste fino tipográfico —interletrado, pesos ópticos, verificación de métricas en ambas plataformas—, no la arquitectura del Design System. Lo estimo en días, no semanas. Registrado en **R-01**, ahora con impacto rebajado de Alto a Medio.

---

### D-6 · WhatsApp como segundo canal, y el contacto deja de ser un correo
**Fecha:** 2026-09-16 · **Responsable:** Dirección · **Documento:** [LOGISTICA_DE_CAMPANAS](LOGISTICA_DE_CAMPANAS.md)

Dirección autorizó **L-2**: el contacto pasa a tener **canales**, cada uno con su
dirección, su permiso y su historial, y la clave única de los envíos gana la
columna `channel`.

**Qué lo hizo necesario.** No fue una preferencia de diseño. Dirección pidió que
una misma campaña, sobre una tabla con columna de correo y columna de celular,
mande el correo y después el WhatsApp. Con la clave única actual
—`UNIQUE(campaign_id, contact_id)`— esos **dos envíos son la misma fila**, y la
base de datos rechaza el segundo. La función pedida no se podía guardar.

**Por qué ahora y no después.** Hoy no hay ninguna campaña guardada que
convertir. Después de la Fase 4 habría que reescribir el motor, que concentra el
70 % del riesgo técnico. Registrado como **R-23**.

**Lo que no cambia:** la importación, la tabla de contactos en pantalla, el motor
y las pantallas ya construidas. Es fontanería, no arquitectura.

**Decisiones asociadas que quedan asentadas en la logística:** L-1 corregida
(una campaña puede tener dos etapas, cada una de un canal), **L-7** (ARLES no
comprueba si un número existe en WhatsApp — Meta apagó esa vía y las
alternativas son no oficiales), **L-8** (se rota entre plantillas del usuario y
se mide cuál rinde; no se genera ni muta texto para esquivar filtros), **L-9**
(dos modos: «Seguimiento» y «Prospección Directa») y **L-10** (CONVERSACIONES es
para trabajar; las estadísticas de los dos canales van en ACTIVIDAD).

---

## Decisiones pendientes

Ver **`02-auditoria/PREGUNTAS_ABIERTAS.md`** para el registro completo.

| # | Pregunta | Bloquea |
|---|---|---|
| P-01 | ¿Qué licencia de Mont posee exactamente TELEMETRY? | **La demo** (antes: Fase 2) |
| P-03 | Origen y consentimiento de los contactos | Fase 3 |
| P-05 | Dominio y aviso de privacidad para Google | Fase 5 |

**Ya no hay bloqueantes de la Fase 2.**
