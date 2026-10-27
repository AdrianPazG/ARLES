# Cómo se diseña un documento para Dirección

**Última actualización:** 2026-10-27

Esta carpeta dice cómo se escribe y cómo se maqueta lo que Dirección lee: un
informe de fase, un resumen ejecutivo, una consulta, un tablero.

No es estilo por estilo. **Dirección no es programador, y lo que lee decide
cosas.** Un documento que se entiende a la primera ahorra una conversación; uno
que hay que descifrar hace tomar la decisión equivocada o ninguna.

| Archivo | Para |
|---|---|
| Este | Las reglas: lenguaje, estructura, maquetación, lo que nunca se hace |
| [`PLANTILLA-INFORME.md`](PLANTILLA-INFORME.md) | El esqueleto de un informe de entrega o de fase |
| [`PLANTILLA-DECISION.md`](PLANTILLA-DECISION.md) | Cuando hace falta que Dirección elija |

**La implementación de referencia** es
[`informe/ARLES-informe-para-direccion.html`](../../../informe/ARLES-informe-para-direccion.html):
ocho secciones, tipografía Mont, los tokens del producto, y estilos de
impresión. Si hay duda sobre un espaciado o un color, se mira ahí.

---

## 1 · El lenguaje

**Profesional y sencillo.** Esas dos cosas no se contradicen: lo contrario de
sencillo no es profesional, es descuidado.

| Se escribe | No se escribe |
|---|---|
| «Activar una campaña de 136 000 personas tarda 17 segundos» | «El rendimiento de la activación es subóptimo» |
| «No se puede lanzar una campaña entera por WhatsApp. Sólo el envío de prueba» | «La funcionalidad de WhatsApp está parcialmente implementada» |
| «Falta que un abogado revise dieciocho preguntas» | «Pendiente de validación legal» |
| «Meta rechaza cada mensaje, uno por uno» | «Se produciría un fallo en la entrega» |

**Las reglas, en orden de cuánto importan:**

1. **Cifras, no adjetivos** (§94). «2 de 6», «34 %», «73.5 segundos de un
   presupuesto de 2». Nunca «casi listo», «bastante rápido», «prácticamente
   terminado». Y **todo porcentaje lleva fecha**, porque un porcentaje sin
   fecha se lee como si fuera de hoy para siempre.
2. **Lo técnico no se esconde, se traduce.** Si una decisión depende de
   entender algo técnico, se explica — y se dice por qué importa. Esconderlo
   obliga a decidir a ciegas.
3. **Se dice lo que no funciona.** Si algo falla, se enseña la salida. Si un
   paso se saltó, se dice. **La sección más importante de un informe suele ser
   la de lo que todavía no hace**, porque es la que impide prometer lo que no
   se puede cumplir.
4. **Al cuestionar algo, con esta estructura:**
   `Problema → Riesgo → Impacto → Alternativa → Trade-off → Recomendación`.
   Una opinión suelta no se puede evaluar.
5. **Español de México, y entero.** Sin anglicismos que tengan palabra:
   «ritmo» y no «throughput», «bandeja de entrada» y no «inbox», «respaldo» y
   no «backup».

### Lo que nunca aparece

- **Ni un emoji.** En ninguna parte: ni en títulos, ni en tablas, ni como
  viñeta, ni como indicador de estado.
- **Nada que suene a texto generado.** Sin «En el mundo actual…», sin
  «Es importante destacar que…», sin «En conclusión, podemos afirmar…», sin
  listas de tres adjetivos («robusto, escalable y seguro»). Si una frase
  podría estar en cualquier informe de cualquier proyecto, sobra.
- **Sin superlativos sin medida.** «Óptimo», «de clase mundial», «altamente
  confiable» no dicen nada. Una cifra con su método al lado, sí.
- **Sin promesas de fecha** que no salgan de algo medido.

---

## 2 · La estructura

Un documento para Dirección se lee **en tres capas**, y cada una tiene que
funcionar sola:

| Capa | Qué es | Cuánto |
|---|---|---|
| **1** | El titular y la primera frase | Quien sólo lea esto tiene que saber si hay que actuar |
| **2** | Los encabezados y las tablas | Quien hojee tiene que poder decidir |
| **3** | El cuerpo | Quien lea entero tiene que poder cuestionar |

**Reglas de estructura:**

- **Un `h1` y nada más.** Después `h2` para las secciones y `h3` dentro. Nunca
  se salta un nivel: un `h3` debajo de un `h1` deja al lector sin saber dónde
  está.
- **Ninguna sección sin su cifra o su conclusión.** Una sección que sólo
  describe se puede quitar.
- **Lo que bloquea va arriba**, no al final. Si hay cuatro cosas que impiden
  entregar, son lo primero después del resumen.
- **Las tablas para comparar, la prosa para explicar.** Una tabla de una
  columna es una lista; una lista de pares es una tabla.
- **Las citas (`>`) sólo para lo que no se puede perder**: una decisión, una
  advertencia, un límite declarado. Si todo está citado, nada destaca.

---

## 3 · La maquetación

Vale para Markdown y para HTML. En HTML sale todo de los tokens del producto
(`app/src/app/estilos/tokens.json`), **nunca de un valor escrito a mano** — es
el §17 aplicado a los documentos.

### Espaciado

| Entre | Cuánto |
|---|---|
| Un título y su entradilla | `space-2` (8 px) — **no 48**, que es el defecto que tuvieron Restaurar y Archivar |
| Una entradilla y el cuerpo | `space-6` (24 px) |
| Dos secciones de primer nivel | `space-10` (40 px), con una línea de separación |
| Dentro de una lista | `space-2` |

**El ritmo vertical se mantiene.** Dos secciones con huecos distintos se leen
como si una perteneciera a la otra.

### Tipografía

- **Mont** para todo, con los pesos del producto. La familia se carga desde
  `/TIPOGRAFIA`, **sin copiar los archivos** (ADR-0012).
- **Jerarquía por tamaño y peso, nunca por color.** Un título en otro color se
  lee como un enlace.
- **Medida de línea: 60 a 75 caracteres.** Una línea de 120 pierde al lector al
  volver al margen.
- **Las cifras con separador de miles**, siempre: `12 536` y no `12536`. Es un
  defecto que ha aparecido **siete veces** en el producto.

### Color

- **El contraste se mide, no se estima.** Mínimo 4.5:1 para texto, 3:1 para
  texto grande. «Detener» estuvo en 2.41:1.
- **Rojo sólo para lo que está mal.** «Sin medir» no es un fallo: es que no se
  sabe, y pintarlo en rojo hace creer que algo está roto.
- **Ningún estado se distingue sólo por color.** Lleva texto al lado, siempre.
  Quien mira ve los colores; quien no, no.
- **Claro y oscuro.** Todo documento HTML funciona en los dos, con
  `prefers-color-scheme` y un fondo explícito en `body`.

### Impresión

Un informe para Dirección **se va a imprimir o se va a pasar a PDF.** Así que:
estilos de impresión, sin fondos oscuros que gasten tinta, sin elementos
interactivos que en papel queden mudos, y los enlaces con su URL visible o
innecesaria.

---

## 4 · Antes de entregarlo

- [ ] **¿Se entiende la primera frase sin leer el resto?**
- [ ] **¿Toda cifra lleva su método al lado?** «1 424 ms → 357 ms, medido el
      06/10/2026 sobre los mismos 500 000 contactos» — no «mejoró mucho».
- [ ] **¿Está lo que no funciona?** Y dicho con esas palabras.
- [ ] **¿Hay algún emoji?** Ninguno.
- [ ] **¿Hay alguna frase que podría estar en cualquier informe?** Fuera.
- [ ] **¿Lo he mirado?** Abierto, a 1024 px y a 1920, en claro y en oscuro.
      Las sondas no cazan un documento que empieza en otro sitio.
- [ ] **¿Está en su carpeta y nombrado en el índice?** Ver
      [`DONDE-VA-CADA-COSA.md`](../../00-ANTES-DE-EMPEZAR/DONDE-VA-CADA-COSA.md).
