# Fase 0 · Discovery, auditoría y arquitectura

**Estado:** ✅ cerrada · **Fecha:** 2026-09-11
**Validación:** 4/4 comprobaciones — `validar.py --fase 0`

> **Objetivo.** Entender y cuestionar antes de construir, tal como exigen el §1 y el §175 del brief maestro. Ninguna línea de código de producción hasta cruzar la puerta de salida.

---

## 1. Qué se produjo

**38 documentos**, ~42 000 palabras, organizados en nueve carpetas. El índice completo está en [00-INDICE.md](../00-INDICE.md).

El entregable central es [AUDITORIA_DISCOVERY.md](../02-auditoria/AUDITORIA_DISCOVERY.md), con el formato A–U que pidió el §163.

---

## 2. Los hallazgos que cambiaron el plan

La auditoría forense del repositorio encontró **siete discrepancias** entre lo que el brief asumía sobre los assets y lo que los archivos realmente eran. Cuatro cambiaron decisiones:

### La referencia cromática no era lo que decía ser
Un PNG renombrado a `.jpg`, marcado por IPTC como generado por IA (`trainedAlgorithmicMedia`), procedente de un banco de imágenes. Al medirla completa: **54 % cian, 19 % ocre, 13 % amarillo, y solo 0.73 % de azul profundo**.

No es una paleta de *La noche estrellada*. El §16 asignaba tres roles a tres azules distintos, pero el 54 % de la imagen cabe en ±12°: extraerlos literalmente los colapsa en una interfaz monocromática sin jerarquía. Y `#2CA4D4` falla WCAG contra todo texto candidato, blanco incluido.

→ **D-2** y [ADR-0005](../03-arquitectura/adr/0005-paleta-derivada.md): extraer el carácter cromático, construir la estructura.

### La licencia tipográfica no cubría el caso de uso
El kit es de Transfonter e incluye `.eot` — perfil de agregador, no de entrega comercial. Y las licencias Desktop y Web de Fontfabric **no cubren incrustar el binario de la fuente en una aplicación distribuida**.

→ **D-3**, luego revisada por **D-5**: se desarrolla con Mont, la puerta pasa a antes de la demo.

### No existía ningún activo de marca
Ni logotipo, ni la flecha ni el escudo que menciona T-9.

→ Resuelto por D-5: el §21 define el logotipo como exclusivamente tipográfico, así que se produce en la Fase 2.

### La verificación OAuth de Google no estaba en el roadmap
`gmail.send` es scope sensible: evita CASA Tier 2, pero exige verificación con política de privacidad pública, dominio verificado y semanas de revisión. Era el camino crítico más largo del proyecto y no aparecía en ninguna fase.

→ **D-1**: SMTP en v1.2.0, verificación OAuth en paralelo desde la Fase 1.

---

## 3. Las tres contradicciones del brief, cerradas

| Contradicción | Resolución |
|---|---|
| §37 (supresión automática por rebote) **vs** §69 (sin leer bandeja) | [ADR-0009](../03-arquitectura/adr/0009-alcance-deteccion-rebotes.md): detección parcial en v1.2.0, **declarada en la interfaz**; VERP en v1.3 |
| T-4 («beta comercial») **vs** T-6 (sin licenciamiento) | **D-4**: v1.2.0 es despliegue interno |
| §16 (tres azules distintos) **vs** la paleta medida | **D-2** / ADR-0005 |

Ninguna se resolvió por omisión. Las tres tienen registro escrito.

---

## 4. Cambios estructurales al plan del brief

**El motor de ejecución se adelantó** de la fase 9 a la fase 4. Concentra el 70 % del riesgo técnico y no produce capturas atractivas, así que la presión natural es postergarlo. Construir la interfaz de campañas antes significaría diseñarla sobre supuestos no validados.

**Los trámites externos arrancan en la Fase 1.** Verificación de Google (4–8 semanas) y certificados de firma (1–3 semanas) tienen plazos que no dependen del equipo. Dejarlos para el final es la forma más común de que un producto terminado no se pueda entregar.

**La navegación se recortó** de siete secciones a seis: las plantillas son un insumo de las campañas, no un destino propio, y «Actividad» duplicaba la salud de envío.

**El roadmap se consolidó** de 16 fases a 10.

---

## 5. Qué se verificó

| Comprobación | Método |
|---|---|
| El cuerpo documental está completo | 14 documentos clave presentes |
| Los 12 ADRs existen | Recuento en `03-arquitectura/adr/` |
| Los enlaces internos resuelven | 88 enlaces comprobados |
| Las contradicciones están cerradas por escrito | Presencia de §37, §69 y T-4 en el índice |

Y una comprobación que no automatiza el script pero sí consta: **cada sección numerada del brief (§1–§175) tiene destino** en la tabla de trazabilidad del [índice](../00-INDICE.md) — aceptada, modificada con justificación, o diferida. Ninguna quedó sin resolver.

---

## 6. Qué NO se verificó

| Sin verificar | Motivo |
|---|---|
| Que las decisiones sean **correctas** | Un script comprueba que están documentadas y son coherentes, no que sean acertadas. Eso lo dirá el producto en uso |
| Las estimaciones de esfuerzo | El roadmap fija **orden y puertas de salida**, no fechas: ponerlas antes de la Fase 1 habría sido inventarlas |
| La viabilidad comercial | Fuera del alcance técnico |

---

## 7. Puerta de salida

Cruzada el **2026-09-11**:

- [x] Dirección aprobó `AUDITORIA_DISCOVERY.md`
- [x] D-5 desbloqueó la tipografía
- [x] P-02 cerrada — el logotipo es tipográfico por diseño

**Abiertas, sin bloquear:** P-01 (licencia, puerta en la demo), P-03 (origen de los contactos, Fase 3), P-05 (dominio y aviso de privacidad, Fase 5).
