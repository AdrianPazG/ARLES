# ADR-0001 — Tauri 2 como cascarón de escritorio

**Estado:** Propuesto · **Fecha:** 2026-09-01

## Contexto
ARLES es una aplicación de escritorio para Windows y macOS cuyo argumento central es la seguridad de
credenciales y una superficie de ataque pequeña, y cuyo corazón es un planificador de larga duración.

## Decisión
Usar **Tauri 2** con núcleo en Rust y frontend Vue 3.

## Alternativas evaluadas

| Criterio | Tauri 2 | Electron |
|---|---|---|
| Superficie de ataque | Núcleo Rust; capacidades por ventana | Seguro si se configura bien; defaults históricamente problemáticos |
| Tamaño del instalador | Usa el webview del sistema; orden de magnitud menor | Empaqueta Chromium y Node |
| Memoria en reposo | Menor | Mayor |
| **Motor de render** | **Dos: WebView2 y WKWebView** | **Uno: Chromium** |
| Madurez | Producción, menos precedentes | Años de producto masivo |
| Motor en segundo plano | Tarea Rust, sin recolector de basura | Hilo de Node o proceso aparte |
| Costo de aprendizaje | Exige Rust real | TypeScript en toda la pila |

## Consecuencias

**Positivas.** Superficie de ataque menor; el sistema de capacidades del v2 es exactamente lo que pide §86;
instalador y memoria significativamente menores; Rust es mejor lenguaje para un planificador confiable.

**Negativas, y hay que presupuestarlas.** Se prueba contra **dos motores de render distintos**. WKWebView
va por detrás de Chromium en algunas propiedades de CSS y su comportamiento depende de la versión de macOS.
Esto impacta directamente en §139.

**Mitigación:** fijar versión mínima de macOS y declararla; prohibir CSS de vanguardia sin respaldo probado;
ejecutar E2E de Playwright en ambas plataformas desde la Fase 3; tratar toda diferencia entre motores como
defecto bloqueante.

## Condición que invierte esta decisión
Si el equipo no tiene capacidad real en Rust y no hay presupuesto para adquirirla, **elegir Electron**.
Un Electron bien configurado y lanzado vale más que un Tauri a medio construir, y la diferencia de seguridad
entre ambos es mucho menor que la diferencia entre entregar y no entregar.
Ver [pregunta abierta #7](../OPEN_QUESTIONS.md).
