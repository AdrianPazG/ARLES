# Presupuesto de rendimiento

**Proyecto:** ARLES RELAY I · v1.2.0
**Escala objetivo:** 200 000 contactos operativos · **500 000 techo arquitectónico** (T-7)

> **Se miden, no se estiman.** Los presupuestos se comprueban en CI contra un conjunto sintético de 500 000 contactos con semilla fija. Una regresión rompe la compilación.

---

## 0. Lo que se midió el 06/10/2026, y lo que no

> **Hasta esta fecha, las trece cifras de abajo eran estimaciones de la Fase 0 y
> ninguna se había comprobado.** La entrega 8.5 construyó el banco de pruebas
> (`crates/arles-banco`) y midió lo que se puede medir desde un contenedor sin
> escritorio. **«Sin medir» no es «cumple»**, y así está escrito.

**Medido el 06/10/2026**, con 500 000 contactos sintéticos (semilla 20261006,
38 MB de CSV) en un contenedor Linux sin escritorio. **No es la máquina de
referencia del §2**, así que las cifras sirven para comparar entre sí y para
detectar lo que está mal por un orden de magnitud — no para certificar.

| Operación | Presupuesto | Medido | Veredicto |
|---|---|---|---|
| Importar 500 000 filas | < 5 min | **51.7 s** (455 864 escritas, 19 447 choques, 24 689 inválidas) | ✅ cumple |
| Memoria durante importación de 500 000 | < 500 MB | **497 MB** | ⚠️ cumple por 3 MB |
| Búsqueda por correo | < 100 ms | **331 ms** | ❌ **no cumple** |
| Abrir tabla de contactos: la consulta | < 500 ms | **1 424 ms** | ❌ **no cumple** |
| Simulación de campaña | < 200 ms | **5.9 ms** | ✅ cumple |
| Arranque en frío | < 2 s | — | ⬜ sin medir (R-07) |
| Desplazamiento en tabla virtualizada | 60 fps | — | ⬜ sin medir (R-07) |
| Filtro sobre 500 k contactos | < 1 s | — | ⬜ **no existe todavía** |
| Calcular audiencia de campaña | < 2 s | — | ⬜ sin medir |
| Preflight completo | < 3 s | — | ⬜ sin medir (R-24) |
| Memoria en reposo con motor activo | < 250 MB | — | ⬜ sin medir (R-07) |
| Importar 100 k filas XLSX | < 60 s | — | ⬜ sin medir |
| Tamaño del instalador | < 20 MB | — | ⬜ sin medir (P-01) |

### Los dos que fallan, y por qué

No son «va un poco lento»: los dos tienen una causa concreta, encontrada con
`EXPLAIN QUERY PLAN` sobre el esquema V10.

**1 · Abrir la tabla de contactos — 1 424 ms de 500.**

`contact` **no tiene ningún índice para su propia consulta**. Los únicos que
hay son el de la clave primaria y uno sobre `import_batch_id`. El plan dice:

```
SCAN contact
USE TEMP B-TREE FOR ORDER BY
```

Es decir: para enseñar **cien** filas, SQLite lee las 455 864 y las ordena en
memoria. Y el recuento del total hace un segundo barrido completo. Es la
pantalla donde el usuario pasa más tiempo.

**2 · Búsqueda por correo — 331 ms de 100.**

Hay un índice único sobre `contact_channel(company_id, channel,
value_normalized)`, y la consulta **no filtra por `channel`**. Sin la segunda
columna, SQLite sólo puede aprovechar la primera:

```
SEARCH ch USING INDEX idx_channel_unique (company_id=?)
```

Así que recorre **todos los canales de la empresa** —correos y WhatsApp, más de
900 000 filas— para encontrar uno. El índice existe y está bien puesto; lo que
no encaja es la consulta.

> **Las dos se arreglan con una migración V11**, y eso tiene una consecuencia
> que conviene decidir y no asumir: **un respaldo tomado con el esquema V10 no
> se podrá restaurar** en una versión con V11 (ver 8.3b). Hoy no hay ningún
> `.arles` fuera de las pruebas, así que el coste es cero; dentro de un mes de
> uso, no. **Pendiente de Dirección.**

### Y el que pasa por tres megas

**497 MB de 500.** Entra, pero no sobra nada, y la causa está documentada desde
la Fase 3: `TablaLeida` guarda **todas** las filas en memoria a propósito,
porque la importación las recorre dos veces —una para los choques y otra para
escribir—. Con 500 000 filas el margen es del 0.6 %; con 600 000 no habría.

No es un defecto hoy —el techo declarado es medio millón— pero sí es la cifra
que primero se romperá, y ahora se sabe en lugar de suponerse.

---

## 1. Presupuestos

Medidos en la máquina de referencia (§2), con la base de datos poblada al techo.

> **Ojo:** la tabla de abajo son los presupuestos **estimados en la Fase 0**.
> Lo que de verdad se ha medido está arriba, en el §0, con su fecha.

| Operación | Presupuesto | Por qué ese número |
|---|---|---|
| Arranque en frío hasta interfaz usable | **< 2 s** | Por encima de 2 s la aplicación se percibe pesada |
| Abrir tabla de contactos (500 k) hasta primer pintado | **< 500 ms** | Umbral de «instantáneo» percibido |
| Desplazamiento en tabla virtualizada | **60 fps sostenidos** | Es la pantalla donde el usuario pasa más tiempo |
| Filtro sobre 500 k contactos | **< 1 s** | Por encima se necesita indicador de carga, lo que rompe el flujo |
| Búsqueda por correo | **< 100 ms** | Hay índice único: debe ser instantáneo |
| Importar 100 k filas XLSX | **< 60 s**, memoria acotada | Referencia realista de un archivo grande |
| Importar 500 k filas | **< 5 min**, memoria acotada | El techo |
| Calcular audiencia de campaña | **< 2 s** | Ocurre en el paso 2 del asistente |
| Simulación de campaña | **< 200 ms** | Es aritmética; debe sentirse inmediato |
| Preflight completo | **< 3 s** | Incluye red (auth, DNS) |
| Memoria en reposo con motor activo | **< 250 MB** | La aplicación corre todo el día en segundo plano |
| Memoria durante importación de 500 k | **< 500 MB** | La lectura en streaming lo hace posible |
| Tamaño del instalador | **< 20 MB** | Beneficio directo de Tauri (ADR-0001) |

---

## 2. Máquina de referencia

Deliberadamente modesta: el público objetivo no tiene estaciones de trabajo.

```
CPU:  4 núcleos, ~2.4 GHz
RAM:  8 GB
Disco: SSD SATA
SO:   Windows 11 y macOS 14
Pantalla: 1920×1080 al 100 %
```

Si los presupuestos se cumplen aquí, se cumplen en cualquier equipo corporativo actual.

---

## 3. Cómo se consigue

### Tablas — virtualización obligatoria

TanStack Virtual (ADR-0007). Con 500 000 filas, renderizar sólo lo visible **no es una optimización: es la diferencia entre funcionar y no funcionar.** Un `v-for` sobre 500 k bloquea el hilo principal durante minutos y agota la memoria.

Sólo se montan las filas visibles más un margen pequeño.

### Paginación por keyset, nunca `OFFSET`

```sql
-- Mal: O(n) en el desplazamiento. Al final de 500 k filas, segundos.
SELECT … ORDER BY created_at LIMIT 50 OFFSET 400000;

-- Bien: O(log n) siempre. Milisegundos en cualquier posición.
SELECT … WHERE (created_at, id) > (?, ?) ORDER BY created_at, id LIMIT 50;
```

`OFFSET` obliga a SQLite a recorrer y descartar las filas saltadas. El keyset salta directamente por el índice.

### Índices — sólo los que se usan

```sql
CREATE UNIQUE INDEX idx_contact_unique ON contact (company_id, email_normalized)
  WHERE deleted_at IS NULL;
CREATE INDEX idx_attempt_work ON message_attempt (state, scheduled_for)
  WHERE state IN ('queued', 'failed');
CREATE INDEX idx_attempt_campaign_state ON message_attempt (campaign_id, state);
CREATE INDEX idx_contact_field ON contact_field (contact_id, field_key);
```

**Los índices parciales importan.** `idx_attempt_work` sólo cubre filas accionables: en una campaña de 500 k con 499 k completadas, el índice tiene 1 000 entradas en vez de 500 000.

Cada índice cuesta en escritura, y la importación es la operación de escritura más pesada del producto. No se añade ninguno sin un plan de consulta que lo justifique.

### Importación por lotes

Una transacción por cada **~1 000 filas**. Una sola transacción gigante consume memoria sin límite; una por fila hace 500 000 sincronizaciones a disco.

Lectura XLSX en **streaming** con `calamine`: el archivo nunca se carga entero en memoria. Esto además es la mitigación de zip bomb del modelo de amenazas.

### El motor no bloquea la interfaz

Corre en su propio conjunto de tareas de Tokio (§51). La base de datos vive en un hilo dedicado con canal (ADR-0002), lo que además elimina de raíz los errores `SQLITE_BUSY` en vez de gestionarlos con reintentos.

---

## 4. Sobrecoste conocido

| Fuente | Coste | ¿Negociable? |
|---|---|---|
| **SQLCipher** | 5–15 % en lectura y escritura | **No** (T-3) |
| Índice único sobre contactos | Escritura más lenta en importación | No — garantiza la deduplicación |
| `audit_log` append-only | Una escritura extra por acción crítica | No — §91 |
| Congelar la audiencia al activar | Una escritura por contacto | No — es lo que hace la campaña auditable |

Los presupuestos de §1 **ya incluyen** estos costes. No son excusas para incumplirlos.

---

## 5. Medición en CI

```
1. Generar 500 000 contactos sintéticos (semilla fija)
2. Poblar una base de datos limpia
3. Ejecutar el banco de pruebas
4. Comparar contra el presupuesto
5. Fallar si alguna operación lo supera en más de un 20 %
```

El margen del 20 % absorbe el ruido de los ejecutores compartidos de CI. Una regresión real lo supera con holgura.

**Los resultados se registran históricamente** para poder detectar degradación lenta: tres regresiones del 15 % consecutivas no rompen ninguna compilación y duplican el tiempo.

---

## 6. Lo que NO se optimiza

Optimizar lo que no lo necesita cuesta legibilidad, y la legibilidad es lo que permite auditar el producto (§140).

| No se optimiza | Por qué |
|---|---|
| Velocidad de envío | **El cuello de botella es el límite del usuario, no nosotros.** 50 correos al día no exigen nada |
| Pantallas de configuración | Se abren una vez |
| Simulador | Es aritmética sobre números pequeños |
| Tamaño del bundle más allá de 20 MB | Ya es 25 veces menor que Electron |
| Arranque por debajo de 2 s | No se percibe la diferencia |

---

## 7. Señales de alarma

Comportamientos que indican un problema de diseño, no de ajuste fino:

- Memoria que **crece sin estabilizarse** con el motor corriendo → fuga
- Desplazamiento que se degrada al bajar en la tabla → la virtualización no funciona
- Tiempo de importación **no lineal** con el número de filas → falta un índice o el lote es incorrecto
- La interfaz se congela al activar una campaña → trabajo en el hilo equivocado
- Consultas que empeoran al crecer la base pese al índice → el plan de consulta no lo está usando

Cada una de estas se investiga como bug, no como ajuste.
