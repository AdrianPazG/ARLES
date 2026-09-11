# ADR-0007 · Pinia y TanStack Virtual

**Estado:** aceptado · **Fecha:** 2026-09-11 · **Decide:** Frontend

---

## Contexto

El brief pide «una solución simple y justificable» para el estado de la interfaz (§9) y exige que las tablas soporten virtualización, ordenación y filtrado con grandes volúmenes (§101), con un objetivo de 500 000 contactos (T-7).

## Decisión

- **Pinia** para el estado
- **TanStack Virtual** (headless) para las tablas
- **vue-router** para la navegación
- **Sin biblioteca de componentes de terceros.** El sistema de diseño se construye sobre primitivas propias.

## Justificación

### Pinia

Es la solución oficial de Vue desde que Vuex quedó en mantenimiento. Tipado de primera clase con TypeScript, sin la ceremonia de mutaciones y acciones, con devtools, y con una API que cabe en una tarde de aprendizaje.

Un store por dominio, que refleja la división del backend.

**Y una restricción importante:** los stores contienen **estado de interfaz y datos en caché**, nunca lógica de negocio. La regla de frontera 3.1 de `ARQUITECTURA.md` dice que la webview no decide nada de negocio, y un store es exactamente el sitio donde esa regla se erosiona sin que nadie lo note. Ningún store calcula si un contacto está suprimido, ni si un límite se alcanzó.

### TanStack Virtual, headless

Con 500 000 contactos, renderizar sólo lo visible **no es una optimización: es la diferencia entre funcionar y no funcionar.** Un `v-for` sobre 500 k filas bloquea el hilo principal durante minutos y agota la memoria.

*Headless* significa que la biblioteca aporta el cálculo de qué rango renderizar y nosotros aportamos el marcado. Eso importa por tres razones:

1. **El diseño es nuestro.** Una cuadrícula de terceros trae su propio aspecto, y pelearse con él para que se parezca al sistema de ARLES cuesta más que escribir la tabla.
2. **La accesibilidad es nuestra.** WCAG 2.2 AA (§19) y navegación completa por teclado (§100) son requisitos duros. Las cuadrículas de terceros suelen tener una accesibilidad aproximada y no auditable.
3. **Menos código de terceros en el bundle**, en línea con el espíritu de ADR-0001.

### Sin biblioteca de componentes

El §15 exige un producto visualmente propio, y el sistema de diseño se deriva de una paleta medida con reglas de contraste específicas (ADR-0005). Adoptar una biblioteca significaría sobrescribir sus tokens, pelear con su especificidad CSS y heredar decisiones de accesibilidad ajenas.

**Coste honesto:** hay que construir las primitivas —botón, entrada, selector, modal, tabla, menú, aviso, pestañas—. Es trabajo real de la Fase 2.

**Mitigación:** para los componentes donde la accesibilidad es difícil y está resuelta —menús, diálogos, combos, gestión de foco—, se usa una biblioteca **headless** (Radix Vue o Headless UI) que aporta el comportamiento accesible sin ningún estilo. Se obtiene la parte difícil sin heredar la apariencia.

## Consecuencias

**Positivas.** Rendimiento predecible con 500 k filas · control total sobre el aspecto y la accesibilidad · bundle pequeño · estado tipado.

**Negativas.** Trabajo inicial de la Fase 2 para construir las primitivas · la accesibilidad de los componentes es responsabilidad nuestra, así que hay que auditarla de verdad (§140, auditoría D) · y hay que sostener la disciplina de no meter lógica de negocio en los stores.

## Alternativas descartadas

**Vuex.** En mantenimiento. No hay motivo para empezar algo nuevo con él.

**Sólo `provide`/`inject` y composables.** Suficiente para una aplicación pequeña; ARLES tiene diez dominios y estado compartido entre ellos. Sin devtools, la depuración se vuelve cara.

**AG Grid.** Resuelve las tablas grandes de forma excelente. Se descarta por licencia —la versión Enterprise es de pago y las funcionalidades que ARLES querría están mayormente ahí—, por peso, y porque su aspecto y su accesibilidad no son nuestros.

**PrimeVue / Vuetify / Element Plus.** Ahorran la Fase 2 a costa de que el producto se parezca a todos los demás, lo que contradice el §15, y de heredar una capa de CSS que hay que sobrescribir en cada componente.
