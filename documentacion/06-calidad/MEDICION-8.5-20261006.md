generado: 500000 filas, 38 MB en 0.7 s
leído el CSV en 0.7 s
importadas 455864 filas en 51.7 s (19447 choques, 24689 inválidas)
| Operación | Presupuesto | Medido | Veredicto |
|---|---|---|---|
| Importar 500 000 filas | < 5 min, memoria acotada | 51.7 s (500000 filas: 455864 escritas, 19447 choques, 24689 inválidas) | **CUMPLE** |
| Memoria durante importación de 500 000 | < 500 MB | 497 MB | **CUMPLE** |
| Búsqueda por correo | < 100 ms | 331 ms (1 resultado(s)) | **NO CUMPLE** |
| Abrir tabla de contactos (500 k): la consulta | < 500 ms hasta primer pintado | 1424 ms para la primera página (100 de 455864) | **NO CUMPLE** |
| Simulación de campaña | < 200 ms | 5.928 ms | **CUMPLE** |
| Arranque en frío hasta interfaz usable | < 2 s | — | **SIN MEDIR** |
| Desplazamiento en tabla virtualizada | 60 fps sostenidos | — | **SIN MEDIR** |
| Filtro sobre 500 k contactos | < 1 s | — | **NO EXISTE** |
| Calcular audiencia de campaña | < 2 s | — | **SIN MEDIR** |
| Preflight completo | < 3 s | — | **SIN MEDIR** |
| Memoria en reposo con motor activo | < 250 MB | — | **SIN MEDIR** |
| Importar 100 k filas XLSX | < 60 s | — | **SIN MEDIR** |
| Tamaño del instalador | < 20 MB | — | **SIN MEDIR** |

### Cómo se midió cada una

- **Importar 500 000 filas** — el camino del asistente entero —leer, analizar choques y escribir— de reloj a reloj, en --release.
- **Memoria durante importación de 500 000** — VmHWM de /proc/self/status: la marca de agua del sistema operativo, que no baja.
- **Búsqueda por correo** — Db::duenos_de_direcciones con una dirección que existe, sobre el índice único.
- **Abrir tabla de contactos (500 k): la consulta** — Db::listar_contactos(0, 100). NO incluye el pintado: eso necesita la webview (R-07).
- **Simulación de campaña** — arles_core::duracion::estimar con 500 000 de audiencia, media de 1 000 llamadas.
- **Arranque en frío hasta interfaz usable** — Necesita abrir la aplicación: ventana de Tauri y llavero del sistema. No se midió: sin escritorio ni llavero en el contenedor (R-07).
- **Desplazamiento en tabla virtualizada** — Necesita una webview midiendo cuadros por segundo. No se midió: sin WKWebView ni WebView2 aquí (R-07).
- **Filtro sobre 500 k contactos** — No hay filtro de contactos que medir. No se midió: la audiencia no se puede filtrar todavía: está anotado desde la Fase 6.
- **Calcular audiencia de campaña** — La audiencia se congela al activar, desde el asistente. No se midió: exige una campaña activa con su etapa y remitente: es un banco aparte.
- **Preflight completo** — Tres de sus ocho puntos son de red: autenticación, conexión y DNS. No se midió: la red del contenedor bloquea 587, 465 y 25 (R-24).
- **Memoria en reposo con motor activo** — Exige la aplicación abierta con su hilo del motor corriendo. No se midió: sin llavero no arranca la aplicación entera (R-07).
- **Importar 100 k filas XLSX** — Haría falta escribir un XLSX de 100 000 filas, y no hay con qué. No se midió: no hay escritor de XLSX en el proyecto, y añadir uno sólo para medir es una dependencia que se distribuye.
- **Tamaño del instalador** — Exige `cargo tauri build` con el empaquetador del sistema. No se midió: el contenedor no tiene el empaquetador ni la licencia de Mont (P-01).

### El conjunto

500000 filas, semilla 20261006, 38 MB de CSV, generado en 0.7 s.
