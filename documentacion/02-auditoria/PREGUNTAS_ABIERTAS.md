# Preguntas pendientes para Dirección

**Proyecto:** ARLES RELAY I · v1.2.0
**Fecha:** 2026-09-11
**Estado:** esperando respuesta

> Registro vivo. Cada respuesta se traslada a `01-producto/DECISIONES_DE_DIRECCION.md` como decisión numerada y se cierra aquí.

---

## Puerta en la demo

### P-01 · ¿Qué licencia de Mont posee exactamente TELEMETRY?
**Disparador:** al planificar la **demo de la aplicación** · **Riesgo:** R-01 · **Decisiones:** D-3, **D-5**

> **D-5 (2026-09-11):** se desarrolla con Mont con normalidad. Esta pregunta **ya no bloquea la Fase 2**; se vuelve a plantear antes de generar el primer instalador destinado a enseñarse fuera del equipo de desarrollo.

T-2 afirma que TELEMETRY posee «la licencia comercial de Mont para uso de texto». El alcance importa:

| Licencia de Fontfabric | ¿Cubre incrustar la fuente en el `.exe` / `.dmg` distribuido? |
|---|---|
| Desktop | No |
| Web | No |
| **App** | **Sí** — es la que ARLES necesita |

Un bundle de Tauri lleva los archivos de fuente dentro del binario que se instala en la máquina del cliente. Eso es exactamente el supuesto de la App License.

**Lo que necesitamos:**
1. Tipo de licencia adquirida y su alcance.
2. Comprobante (factura o contrato) subido a `documentacion/08-legal/`.
3. Si no cubre app embedding: ¿se adquiere la App License, o se activa el plan B?

**Plan B, ya documentado y listo:** Mont queda para marketing y material comercial; el logotipo ARLES RELAY se entrega como **SVG con contornos** (uso legítimo de una licencia de escritorio, porque ya no hay fuente que incrustar); la interfaz usa una geométrica de licencia libre con métricas similares.

**Si al planificar la demo no hay comprobante, se activa el plan B.**

---

### P-02 · ¿Quién produce los activos de marca? — ✅ CERRADA

**Cerrada el 2026-09-11 por D-5.** El §21 define el logotipo como **exclusivamente tipográfico** («ARLES RELAY» en Mont Black, sin isotipo), así que con Mont autorizada para desarrollo el logotipo se produce en la Fase 2, aquí. La flecha y el escudo que menciona T-9 **no son necesarios**: el §21 los excluye por diseño.

Queda como recomendación para v1.3 que Dirección entregue un brandbook propio —eliminaría el stock de IA del linaje de la marca (A-03)— pero ya no bloquea nada.

---

## Necesarias para la Fase 3 (Contactos)

### P-03 · ¿Cuál es el origen y el consentimiento de los contactos?
**Riesgo:** R-13 · **Afecta a:** LFPDPPP, texto de la interfaz de importación

ARLES no puede verificar que el cliente tenga consentimiento para los contactos que importa, pero sí puede —y debe— exigir una afirmación explícita y registrarla.

**Lo que necesitamos saber:** de dónde salen las listas que TELEMETRY va a importar (formularios propios, clientes existentes, bases adquiridas, ferias…). Determina el texto exacto de la afirmación de origen lícito y la exposición real ante la LFPDPPP.

**Advertencia:** si el origen incluye bases adquiridas a terceros, la exposición legal es significativa y debe valorarse antes de la primera campaña, no después.

---

### P-04 · ¿Se acepta que la detección de rebotes sea parcial en v1.2.0?
**Riesgo:** R-06 · **ADR:** 0009

El brief se contradice: §37 exige supresión automática por hard bounce, §69 prohíbe pedir permisos de lectura de bandeja. Sin leer un buzón no hay rebotes asíncronos.

**Propuesta:** v1.2.0 captura sólo rechazos 5xx síncronos de SMTP y permite supresión manual. La solución completa —buzón de rebotes dedicado leído por IMAP, sin tocar nunca la bandeja personal del usuario— entra en v1.3.

> **Respondida el 2026-09-12.** Dirección pidió adelantarla y aceptó la degradación elegante. Al verificarlo, el VERP resultó **inviable** en los tres canales previstos; la arquitectura pasa a reenvío + IMAP. Ver [ADR-0014](../03-arquitectura/adr/0014-deteccion-de-rebotes-sin-verp.md).

**Condición que pedimos aceptar:** la interfaz declara explícitamente que la detección es parcial en esta versión. Un cliente informado puede compensarlo; uno que se cree protegido descubre el problema cuando ya quemó su dominio.

---

## Necesarias para la Fase 5 (Proveedores de correo)

### P-05 · ¿Qué dominio y qué aviso de privacidad se usan para la verificación de Google?
**Riesgo:** R-02 · **Decisión relacionada:** D-1

La verificación OAuth de Google para el scope `gmail.send` exige:
- Política de privacidad **pública** alojada en un dominio **verificado** en Google Search Console.
- Página de inicio del producto en ese mismo dominio.
- Vídeo demostrativo del flujo de consentimiento.
- Revisión de semanas.

**Lo que necesitamos:** el dominio, quién redacta el aviso de privacidad, y quién lo publica. **El trámite arranca en la Fase 1**, así que esto se necesita pronto aunque `GoogleProvider` no entre hasta v1.2.x.

---

### P-06 · ¿Cuál es el volumen operativo realista por campaña?
**Afecta a:** presupuestos de rendimiento, conversación sobre entregabilidad

T-7 fija 500 000 contactos como **techo arquitectónico**, y la arquitectura lo soportará. Pero el volumen **operativo** real es otra cosa y cambia decisiones:

- Con 200 correos por campaña, la entregabilidad casi no es un problema y el simulador es una comodidad.
- Con 50 000, la reputación del dominio es la variable dominante del producto y el aviso de más de 50 diarios (§48) se convierte en la pantalla más importante de ARLES.

**Lo que necesitamos:** tamaño típico de campaña y número de campañas al mes previstos.

---

## Para v1.3, conviene decidirlo pronto

### P-07 · ¿Cuál es el modelo comercial?
**Afecta a:** esquema de licenciamiento, que se prepara ya en v1.2.0

El enforcement está fuera de v1.2.0 (T-6, D-4), pero el **esquema** se define ahora para no migrar datos después.

**Lo que necesitamos:** ¿licencia perpetua o suscripción? ¿Por asientos, por instalación o por organización? ¿Hay ediciones con capacidades distintas?

**Nuestra recomendación** (detalle en la sección Q de la auditoría): licencia firmada criptográficamente y verificable **sin conexión**, con periodo de gracia. Al caducar, la aplicación entra en modo restringido —consultar, exportar y respaldar siguen funcionando; activar campañas nuevas no—. **Nunca se bloquea el acceso a los datos del cliente.**

---

### P-08 · ¿Se autoriza la consolidación de `/RECURSOS`?
**Riesgo:** R-15 · **ADR:** 0012

`/RECURSOS` es hoy una copia parcial y obsoleta de las carpetas de raíz: 1 de 27 conceptos de diseño, 1 de 89 archivos tipográficos. Quien siga literalmente la instrucción del §12 verá una fracción del material.

**Propuesta:** `/RECURSOS` pasa a ser fuente única de verdad, se rehidrata desde raíz, y las copias de raíz se eliminan. La referencia cromática se renombra a `.png`, que es lo que realmente es.

**Pedimos autorización explícita** porque el brief marcó esas rutas como intocables. Nada se ha movido.

---

## Tabla de seguimiento

| # | Pregunta | Bloquea | Riesgo | Estado |
|---|---|---|---|---|
| P-01 | Licencia de Mont | **La demo** | R-01 | 🟡 |
| P-02 | Activos de marca | — | R-08 | ✅ cerrada (D-5) |
| P-03 | Origen de los contactos | Fase 3 | R-13 | 🟡 |
| P-04 | Rebotes parciales aceptados | Fase 3 | R-06 | 🟡 |
| P-05 | Dominio y aviso de privacidad | Fase 5 | R-02 | 🟡 |
| P-06 | Volumen operativo real | Fase 4 | R-09 | 🟡 |
| P-07 | Modelo comercial | v1.3 | — | ⚪ |
| P-08 | Consolidación de `/RECURSOS` | — | R-15 | ⚪ |
| **P-09** | **¿Cuál es el marco de protección de datos vigente?** | **Antes de congelar cualquier texto legal visible** | **R-19** | 🔴 |
| **P-10** | ¿Qué cobertura real de correo tiene la lista del DENUE? | Antes de la primera campaña | R-13 | 🟡 |

**No queda ninguna pregunta bloqueante de las Fases 1 y 2.** P-03 y P-05 se necesitan más adelante y conviene irlas resolviendo en paralelo.

---

### P-09 · ¿Cuál es el marco de protección de datos vigente en México?
**Riesgo:** R-19 · **Bloquea:** todo texto legal que la aplicación muestre al usuario

Toda la documentación de privacidad se escribió contra la **LFPDPPP de 2010**.
La investigación de la Fase 3 encontró indicios sólidos —en fuentes secundarias,
no en el texto primario— de que esa ley fue **abrogada en marzo de 2025** por
una nueva LFPDPPP, y de que el **INAI fue extinguido**, pasando sus facultades a
la **Secretaría Anticorrupción y Buen Gobierno**.

**Lo que necesitamos:** confirmación de un abogado mexicano sobre (a) qué ley
rige hoy, (b) si el Reglamento de 2011 sigue aplicando, (c) qué obliga el marco
vigente cuando los datos se obtienen **indirectamente de una fuente de acceso
público**, y (d) qué exige para **mercadotecnia directa**.

**Por qué bloquea:** un aviso de privacidad o una afirmación de consentimiento
que cita una ley abrogada no protege a nadie — y es peor que no citar ninguna,
porque aparenta rigor. Las decisiones de ingeniería no dependen de esto; los
textos visibles, sí.

Detalle en [PRIVACIDAD_LFPDPPP.md](../04-seguridad/PRIVACIDAD_LFPDPPP.md).

---

### P-10 · ¿Qué cobertura real de correo tiene la lista del DENUE?
**Riesgo:** R-13

El campo de correo del DENUE (`DIR_ELEC_1`) existe, pero **su captura es
voluntaria** por parte de cada unidad económica. No hay cifra oficial publicada
del porcentaje de los ~6 millones de establecimientos que lo tienen poblado.

**Se resuelve en diez minutos**: descargar el CSV del DENUE y contar no nulos.
Conviene hacerlo **antes** de dimensionar cualquier campaña sobre esa lista, y
antes de decidir cuánto invertir en validación.

---

### P-11 · La barra lateral: ¿fija y plegable? · **decidido en la revisión visual**
**Riesgo:** R-01 del informe de revisión · **Se construye en:** entrega 3.1

Dirección pidió dos cosas al revisar la aplicación en Windows:

> *«necesito que el menú ubicado en la lateral permanezca fijo y que lo demás
> que está en pantalla sea posible desplazarse. También necesito que ese menú
> fijo pueda comprimirse.»*

Son dos peticiones distintas —**barra fija** al desplazar, y **barra
plegable**— y la segunda resuelve además un defecto encontrado en la misma
revisión: por encima del **150 % de escala de Windows**, el armazón desborda
horizontalmente porque `App.vue` fija `min-width: 1120px`, y a esa escala el
viewport CSS baja de 1120.

**Lo que hay que decidir, y no está decidido:**

1. ¿La barra se pliega **a mano**, **sola** por debajo de un ancho, o **las dos
   cosas**?
2. Plegada, ¿queda en **iconos** o **desaparece**? Con iconos hacen falta
   iconos para las seis secciones, que hoy no existen.
3. ¿Se **recuerda** el estado entre arranques? Si se recuerda, va a la
   configuración, y eso es esquema.

**Por qué no se resuelve antes de 3.1:** hacerlo ahora sería construirlo sobre
un armazón sin pantallas reales que lo pongan a prueba. **Por qué no después:**
cada pantalla que se monte encima hereda el defecto y habrá que rehacerla.

Medición completa en
[revision-visual/2026-09-14-windows/HALLAZGOS.md](../06-calidad/revision-visual/2026-09-14-windows/HALLAZGOS.md).
